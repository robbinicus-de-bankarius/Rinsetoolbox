//! Durable External Subsidiary Memory Bank (`MemoryBank`) living on robbinicus de bankarius's side.
//!
//! Coordinates:
//! - `bank.smem`: Append-only binary turn log (LEB128 varint delta timestamps + 1-byte bit-packed flags + 8-byte NineOctaveProfile + CRC32)
//! - `bank.qtz`:  Nine-Octave Parabolic Phantom Quartz Crystal Pyramid (`64×64 = 4,096` base nodes + Adaptive Split/Merge + GF(256) Hologram)
//! - `bank.sym`:  Interned domain & stem symbol table (`SymbolTable`)
//! - `PRELOAD-BRIEF.txt`: Instant pre-speech memory injection block updated on every turn

use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::codebook::SymbolTable;
use crate::codec::{
    decode_turn_frame, encode_turn_frame, read_str, read_varint_u64, write_str, write_varint_u64,
    CompactCodec, SMEM_MAGIC, SMEM_VERSION,
};
use crate::crystal::{HologramRecoveryReport, PhantomQuartzCrystal};
use crate::error::{Result, SubmemError};
use crate::index::InvertedIndex;
use crate::smc::{
    analyze_nine_octave_profile, classify_utterance, decompress_lz_bytes, encode_to_smc,
};
use crate::types::{
    ApprovalState, IntentKind, PackedTurnFlags, PreloadPacket, Priority, Speaker, TurnRecord,
};

pub struct MemoryBank {
    pub dir: PathBuf,
    pub symtab: SymbolTable,
    pub records: Vec<TurnRecord>,
    pub crystal: PhantomQuartzCrystal,
    pub index: InvertedIndex,
    pub recovered_corrupt_tail: bool,
}

#[derive(Debug, Clone)]
pub struct BankStats {
    pub total_turns: usize,
    pub pinned_locks: usize,
    pub raw_transcript_bytes: usize,
    pub smc_code_bytes: usize,
    pub binary_smem_bytes: u64,
    pub binary_qtz_bytes: u64,
    pub active_crystal_splitters: usize,
    pub total_crystal_splitters: usize,
    pub active_octaves: usize,
    pub adaptive_split_subcells: usize,
    pub occupied_crystal_slots: usize,
    pub total_crystal_imprints: u64,
    pub hologram_raw_bytes: usize,
    pub hologram_lz_bytes: usize,
    pub hologram_lz_ratio: f64,
    pub interned_symbols: usize,
    pub smc_compression_ratio: f64,
    pub smc_space_savings_pct: f64,
}

impl MemoryBank {
    /// Opens or initializes a durable `MemoryBank` at `dir`.
    /// Performs CRC32 verification on every frame in `bank.smem` and reconstructs the
    /// Nine-Octave Parabolic Phantom Quartz Crystal Pyramid and BM25 Inverted Index.
    pub fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;

        let sym_path = dir.join("bank.sym");
        let smem_path = dir.join("bank.smem");
        let qtz_path = dir.join("bank.qtz");

        let symtab = if sym_path.exists() {
            Self::load_symtab(&sym_path).unwrap_or_else(|_| SymbolTable::new())
        } else {
            SymbolTable::new()
        };

        let mut records = Vec::new();
        let mut recovered_corrupt_tail = false;

        if smem_path.exists() {
            let file = File::open(&smem_path)?;
            let mut reader = BufReader::new(file);
            let mut magic = [0u8; 4];
            if reader.read_exact(&mut magic).is_ok() {
                if magic != SMEM_MAGIC {
                    return Err(SubmemError::InvalidMagic(magic));
                }
                let mut ver = [0u8; 1];
                reader.read_exact(&mut ver)?;
                if ver[0] != SMEM_VERSION {
                    return Err(SubmemError::UnsupportedVersion(ver[0]));
                }
                let mut prev_ts = 0u64;
                loop {
                    match decode_turn_frame(&mut reader, prev_ts) {
                        Ok(Some(rec)) => {
                            prev_ts = rec.timestamp_sec;
                            records.push(rec);
                        }
                        Ok(None) => break,
                        Err(SubmemError::ChecksumMismatch { .. })
                        | Err(SubmemError::TruncatedFrame { .. }) => {
                            // Crash-safe tail recovery: retain all valid prior turns!
                            recovered_corrupt_tail = true;
                            break;
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
        }

        let mut crystal = if qtz_path.exists() && !recovered_corrupt_tail {
            let mut f = BufReader::new(File::open(&qtz_path)?);
            PhantomQuartzCrystal::decode_from(&mut f).unwrap_or_else(|_| PhantomQuartzCrystal::new())
        } else {
            PhantomQuartzCrystal::new()
        };

        let mut index = InvertedIndex::new();
        let rebuild_crystal = crystal.total_imprints == 0 && !records.is_empty();
        for rec in &records {
            index.add_turn(rec);
            if rebuild_crystal {
                crystal.imprint_turn(rec);
            }
        }
        if rebuild_crystal || (crystal.hologram_lz_len == 0 && !records.is_empty()) {
            let _ = crystal.sync_hologram(&records);
        }

        let bank = Self {
            dir,
            symtab,
            records,
            crystal,
            index,
            recovered_corrupt_tail,
        };

        if recovered_corrupt_tail {
            bank.rewrite_clean_log()?;
            bank.save_crystal()?;
        }

        Ok(bank)
    }

    /// Ingests a new utterance from a voice call or chat turn across all **Nine Octaves**:
    /// 1. Redacts any secrets/keys/passwords.
    /// 2. Compresses the raw speech to Shortest Memory Code (`SMC`) + interned `u16` token IDs.
    /// 3. Computes the `NineOctaveProfile` (Emotional Weight, Valence, Certainty, Audience, Consequence Tensor).
    /// 4. Traverses the Nine-Octave Parabolic Phantom Quartz Crystal (`4,096` base nodes + Adaptive Split/Merge).
    /// 5. Updates the GF(256) Vandermonde + LZ77 Holographic Spiral Shards so 10% survival reconstructs 100%.
    pub fn ingest(
        &mut self,
        speaker: Speaker,
        call_id: &str,
        raw_text: &str,
        timestamp_override: Option<u64>,
        force_pin: bool,
    ) -> Result<TurnRecord> {
        let turn_id = (self.records.len() + 1) as u32;
        let prev_ts = self.records.last().map(|r| r.timestamp_sec).unwrap_or(0);
        let now_ts = timestamp_override.unwrap_or_else(current_unix_sec).max(prev_ts);

        let (intent, priority, approval) = classify_utterance(speaker, raw_text, force_pin);
        let flags = PackedTurnFlags::pack(
            speaker,
            intent,
            priority,
            approval != ApprovalState::None,
        );
        let octave_profile =
            analyze_nine_octave_profile(speaker, intent, priority, approval, raw_text);

        let encoded = encode_to_smc(raw_text, &mut self.symtab);

        let mut rec = TurnRecord {
            turn_id,
            timestamp_sec: now_ts,
            flags,
            approval,
            call_id: call_id.to_string(),
            smc_code: encoded.smc_code,
            smc_token_ids: encoded.token_ids,
            domain_tags: encoded.domain_tags,
            crystal_nodes: Vec::new(),
            crystal_path_sig: 0,
            compressed_raw: encoded.compressed_raw,
            raw_byte_len: raw_text.len(),
            octave_profile,
        };

        // Traverse the Nine-Octave Parabolic Phantom Quartz Crystal
        let (crystal_nodes, crystal_path_sig) = self.crystal.imprint_turn(&rec);
        rec.crystal_nodes = crystal_nodes;
        rec.crystal_path_sig = crystal_path_sig;

        // Append binary frame to `bank.smem`
        self.append_frame_to_disk(&rec, prev_ts)?;
        self.index.add_turn(&rec);
        self.records.push(rec.clone());

        // Synchronize GF(256) + LZ77 Holographic Spiral Shards across all 4,096 nodes
        self.crystal.sync_hologram(&self.records)?;

        // Persist symbol table, Phantom Quartz Crystal lattice, and instant preload brief
        self.save_symtab()?;
        self.save_crystal()?;
        self.write_preload_snapshot("")?;

        Ok(rec)
    }

    /// Rebuilds the entire `MemoryBank` (`records` + `index` + `crystal`) solely from the
    /// surviving holographic crystal nodes after catastrophic storage destruction (e.g. 90% wipe).
    pub fn recover_from_hologram(&mut self) -> Result<HologramRecoveryReport> {
        let (recovered_records, report) = self.crystal.reconstruct_from_hologram()?;
        self.records = recovered_records;
        self.index = InvertedIndex::new();
        for rec in &self.records {
            self.index.add_turn(rec);
        }
        self.rewrite_clean_log()?;
        self.save_crystal()?;
        Ok(report)
    }

    /// **Retrieval-Before-Speaking (`submem preload`)**:
    /// Reconstructs the bot's working memory in `< 1 ms` using Nine-Octave Parabolic Focus
    /// Constructive Interference + Single-Object Temporal Consequence Projection.
    pub fn preload(
        &mut self,
        current_cue: &str,
        recent_count: usize,
        top_k_relevant: usize,
        byte_budget: usize,
    ) -> PreloadPacket {
        let start = Instant::now();
        let now_ts = self
            .records
            .last()
            .map(|r| r.timestamp_sec)
            .unwrap_or_else(current_unix_sec);
        let first_ts = self
            .records
            .first()
            .map(|r| r.timestamp_sec)
            .unwrap_or(now_ts);
        let wipe_epoch_estimate = ((now_ts.saturating_sub(first_ts)) / 600) as usize + 1;

        // 1. Collect Pinned Standing Locks (`*=`)
        let pinned_locks: Vec<String> = self
            .records
            .iter()
            .filter(|r| r.flags.priority() == Priority::PinnedLock)
            .map(TurnRecord::format_smc_line)
            .collect();

        // 2. Collect unresolved Questions (`?`) from recent turns
        let open_questions: Vec<String> = self
            .records
            .iter()
            .rev()
            .filter(|r| r.flags.intent() == IntentKind::Question)
            .take(3)
            .map(TurnRecord::format_smc_line)
            .collect();

        // 3. Probe the Nine-Octave Parabolic Phantom Quartz Crystal + BM25 Index
        let mut relevant_hits = Vec::new();
        if !current_cue.trim().is_empty() {
            let (q_intent, q_prio, q_app) =
                classify_utterance(Speaker::Robbinicus, current_cue, false);
            let q_prof = analyze_nine_octave_profile(
                Speaker::Robbinicus,
                q_intent,
                q_prio,
                q_app,
                current_cue,
            );
            let encoded_q = encode_to_smc(current_cue, &mut self.symtab);
            let ranked = self.index.hybrid_retrieve_nine_octave(
                &self.crystal,
                &self.records,
                &encoded_q.token_ids,
                &encoded_q.domain_tags,
                q_prof,
                now_ts,
                top_k_relevant,
            );
            for (score, turn_id, beams) in ranked {
                if let Some(rec) = self.records.get((turn_id.saturating_sub(1)) as usize) {
                    let cons = rec.project_consequence_at(now_ts);
                    let line = format!(
                        "{} <parab_beams:{} C(dt):{:.2} score:{:.2}>",
                        rec.format_smc_line(),
                        beams,
                        cons.effective_consequence,
                        score
                    );
                    relevant_hits.push((score, line));
                }
            }
        }

        // 4. Rolling recent call turns in SMC code
        let recent_start = self.records.len().saturating_sub(recent_count);
        let recent_window: Vec<String> = self.records[recent_start..]
            .iter()
            .map(TurnRecord::format_smc_line)
            .collect();

        let (active_crystal_splitters, _, _) = self.crystal.stats();
        let (active_octaves, _) = self.crystal.active_octaves_mask();

        let mut block = String::with_capacity(byte_budget.min(4096));
        block.push_str(&format!(
            "=== SUBMEM 9-OCTAVE PARABOLIC BANK (turns:{} | wipe:#{} | octaves:{}/9 | active:{}/4096) ===\n",
            self.records.len(),
            wipe_epoch_estimate,
            active_octaves,
            active_crystal_splitters
        ));

        let lock_cap = (byte_budget * 52) / 100;
        block.push_str("[1. PINNED LOCKS (*= NEVER WIPED)]\n");
        for lock in &pinned_locks {
            if block.len() + lock.len() + 2 > lock_cap && block.lines().count() > 4 {
                break;
            }
            block.push_str(lock);
            block.push('\n');
        }

        let apex_cap = (byte_budget * 82) / 100;
        if !relevant_hits.is_empty() {
            block.push_str("[2. PARABOLIC FOCUS HITS (9-OCTAVE CONSTRUCTIVE INTERFERENCE)]\n");
            for (_, hit_line) in &relevant_hits {
                if block.len() + hit_line.len() + 2 > apex_cap {
                    break;
                }
                block.push_str(hit_line);
                block.push('\n');
            }
        }

        if !open_questions.is_empty() && block.len() + 48 < byte_budget {
            block.push_str("[3. OPEN CALL QUESTIONS (?)]\n");
            for q in &open_questions {
                if block.len() + q.len() + 2 > byte_budget {
                    break;
                }
                block.push_str(q);
                block.push('\n');
            }
        }

        if block.len() + 36 < byte_budget {
            block.push_str("[4. RECENT CALL SPIRAL (SMC)]\n");
            for rline in &recent_window {
                if block.len() + rline.len() + 2 > byte_budget {
                    break;
                }
                block.push_str(rline);
                block.push('\n');
            }
        }

        let retrieval_micros = start.elapsed().as_micros();

        PreloadPacket {
            total_turns_in_bank: self.records.len(),
            wipe_epoch_estimate,
            retrieval_micros,
            active_crystal_splitters,
            pinned_locks,
            open_questions,
            relevant_hits,
            recent_window,
            compact_prompt_block: block,
        }
    }

    /// Decodes the lossless LZ77-compressed raw utterance for a given 1-based `turn_id`.
    pub fn decode_raw_turn(&self, turn_id: u32) -> Option<String> {
        let rec = self.records.get((turn_id.saturating_sub(1)) as usize)?;
        let bytes = decompress_lz_bytes(&rec.compressed_raw);
        String::from_utf8(bytes).ok()
    }

    pub fn stats(&self) -> BankStats {
        let raw_transcript_bytes: usize = self.records.iter().map(|r| r.raw_byte_len).sum();
        let smc_code_bytes: usize = self.records.iter().map(|r| r.smc_code.len()).sum();
        let pinned_locks = self
            .records
            .iter()
            .filter(|r| r.flags.priority() == Priority::PinnedLock)
            .count();
        let binary_smem_bytes = fs::metadata(self.dir.join("bank.smem"))
            .map(|m| m.len())
            .unwrap_or(0);
        let binary_qtz_bytes = fs::metadata(self.dir.join("bank.qtz"))
            .map(|m| m.len())
            .unwrap_or(0);
        let (active_crystal_splitters, occupied_crystal_slots, total_crystal_imprints) =
            self.crystal.stats();
        let (active_octaves, _) = self.crystal.active_octaves_mask();
        let adaptive_split_subcells = self
            .crystal
            .adaptive_subcells
            .iter()
            .filter(|s| !s.is_merged)
            .count();

        let smc_compression_ratio = if smc_code_bytes > 0 {
            raw_transcript_bytes as f64 / smc_code_bytes as f64
        } else {
            1.0
        };
        let smc_space_savings_pct = if raw_transcript_bytes > 0 {
            (1.0 - (smc_code_bytes as f64 / raw_transcript_bytes as f64)) * 100.0
        } else {
            0.0
        };
        let hologram_lz_ratio = if self.crystal.hologram_lz_len > 0 {
            self.crystal.hologram_raw_len as f64 / self.crystal.hologram_lz_len as f64
        } else {
            1.0
        };

        BankStats {
            total_turns: self.records.len(),
            pinned_locks,
            raw_transcript_bytes,
            smc_code_bytes,
            binary_smem_bytes,
            binary_qtz_bytes,
            active_crystal_splitters,
            total_crystal_splitters: crate::crystal::TOTAL_SPLITTERS,
            active_octaves,
            adaptive_split_subcells,
            occupied_crystal_slots,
            total_crystal_imprints,
            hologram_raw_bytes: self.crystal.hologram_raw_len,
            hologram_lz_bytes: self.crystal.hologram_lz_len,
            hologram_lz_ratio,
            interned_symbols: self.symtab.len(),
            smc_compression_ratio,
            smc_space_savings_pct,
        }
    }

    fn append_frame_to_disk(&self, rec: &TurnRecord, prev_ts: u64) -> Result<()> {
        let smem_path = self.dir.join("bank.smem");
        let is_new = !smem_path.exists() || fs::metadata(&smem_path)?.len() == 0;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&smem_path)?;
        let mut writer = BufWriter::new(file);
        if is_new {
            writer.write_all(&SMEM_MAGIC)?;
            writer.write_all(&[SMEM_VERSION])?;
        }
        encode_turn_frame(&mut writer, rec, prev_ts)?;
        writer.flush()?;
        Ok(())
    }

    pub fn rewrite_clean_log(&self) -> Result<()> {
        let tmp_path = self.dir.join("bank.smem.tmp");
        {
            let file = File::create(&tmp_path)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(&SMEM_MAGIC)?;
            writer.write_all(&[SMEM_VERSION])?;
            let mut prev_ts = 0u64;
            for rec in &self.records {
                encode_turn_frame(&mut writer, rec, prev_ts)?;
                prev_ts = rec.timestamp_sec;
            }
            writer.flush()?;
        }
        fs::rename(&tmp_path, self.dir.join("bank.smem"))?;
        Ok(())
    }

    fn save_symtab(&self) -> Result<()> {
        let tmp_path = self.dir.join("bank.sym.tmp");
        {
            let mut writer = BufWriter::new(File::create(&tmp_path)?);
            let entries = self.symtab.entries();
            write_varint_u64(&mut writer, entries.len() as u64)?;
            for s in entries {
                write_str(&mut writer, s)?;
            }
            writer.flush()?;
        }
        fs::rename(&tmp_path, self.dir.join("bank.sym"))?;
        Ok(())
    }

    fn load_symtab(path: &Path) -> Result<SymbolTable> {
        let mut reader = BufReader::new(File::open(path)?);
        let count = read_varint_u64(&mut reader)? as usize;
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            entries.push(read_str(&mut reader)?);
        }
        Ok(SymbolTable::from_entries(entries))
    }

    fn save_crystal(&self) -> Result<()> {
        let tmp_path = self.dir.join("bank.qtz.tmp");
        {
            let mut writer = BufWriter::new(File::create(&tmp_path)?);
            self.crystal.encode_to(&mut writer)?;
            writer.flush()?;
        }
        fs::rename(&tmp_path, self.dir.join("bank.qtz"))?;
        Ok(())
    }

    pub fn write_preload_snapshot(&mut self, cue: &str) -> Result<()> {
        let pkt = self.preload(cue, 8, 5, 2048);
        let brief_path = self.dir.join("PRELOAD-BRIEF.txt");
        fs::write(&brief_path, &pkt.compact_prompt_block)?;
        Ok(())
    }
}

#[inline]
pub fn current_unix_sec() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(1_792_733_000)
}
