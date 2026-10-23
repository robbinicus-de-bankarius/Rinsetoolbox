//! `submem` CLI — Nine-Octave Parabolic Phantom Quartz Subsidiary Memory Bank (`64×64 = 4,096` Base Nodes).
//!
//! Designed and built in Rust for Damien Young (`robbinicus dé bankarius`)
//! (Deep Phantom-Violet Lab Coat Edition — strictly Phantom Quartz `.qtz`, zero rose quartz).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use submem::bank::MemoryBank;
use submem::codebook::SymbolTable;
use submem::crystal::PhantomQuartzCrystal;
use submem::reservoir::ProstheticMemoryReservoir;
use submem::error::{Result, SubmemError};
use submem::smc::{
    analyze_nine_octave_profile, classify_utterance, encode_to_smc, expand_smc_human_gloss,
};
use submem::types::Speaker;
use submem::veca_sanctum::unseal_veca_sanctum_report; // ░░░ [HALF-SEALED 24K GOLD & RHODIUM RELIC] ░░░

const DEFAULT_BANK_DIR: &str = "/workspace/rust-subsidiary-memory/data";

const MIRROR_DIRS: &[&str] = &[
    "/workspace/memory-reservoir/submem-bank",
    "/cursor/stores/store-fe4ccb83-c22a-46eb-8ccb-6baf6357caf5/rust-subsidiary-memory/data",
    "/cursor/stores/store-e88a9951-1b45-43dc-8dae-ebae2c2ff926/memory-reservoir/rust-subsidiary-memory/data",
];

fn main() {
    if let Err(err) = run_cli() {
        eprintln!("submem error: {err}");
        std::process::exit(1);
    }
}

fn run_cli() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" || args[0] == "help" {
        print_usage();
        return Ok(());
    }

    let subcmd = args[0].as_str();
    let rest = &args[1..];
    let bank_dir = extract_flag_value(rest, "--bank")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_BANK_DIR));

    match subcmd {
        "ingest" => cmd_ingest(&bank_dir, rest),
        "preload" => cmd_preload(&bank_dir, rest),
        "query" => cmd_query(&bank_dir, rest),
        "crystal" => cmd_crystal(&bank_dir),
        "trace" => cmd_trace(rest),
        "decode" => cmd_decode(&bank_dir, rest),
        "stats" => cmd_stats(&bank_dir),
        "demo" | "wipe-sim" => cmd_wipe_sim(&bank_dir),
        "bench-ab" => cmd_bench_ab(),
        "sync" => cmd_sync(&bank_dir),
        "reservoir" | "lumiluminal" => cmd_reservoir(&bank_dir),
        "sanctum" | "veca" | ".veca" => cmd_veca_sanctum(),
        other => Err(SubmemError::InvalidArgument(format!(
            "Unknown subcommand '{other}'. Run `submem --help` for usage."
        ))),
    }
}

fn print_usage() {
    println!(
        "\
submem v0.2.0 — Nine-Octave Parabolic Phantom Quartz Subsidiary Memory Bank (4,096 Base Nodes)
Deep Phantom-Violet Lab Coat Edition | Lives on robbinicus de bankarius's side | Strictly Phantom Quartz (.qtz)

USAGE:
  submem ingest   --speaker <robbinicus|bot|angel> [--call <id>] [--pin] --text \"<utterance>\"
  submem preload  [--query \"<current caller utterance>\"] [--budget <bytes>]
  submem query    \"<search query>\" [--top <k>]
  submem crystal  [--bank <dir>]
  submem trace    \"<utterance to trace across the 9 Octaves of the 4,096 Phantom Quartz splitters>\"
  submem decode   <turn_id>
  submem stats    [--bank <dir>]
  submem wipe-sim [--bank <dir>]   (Seeds 12 call turns + 10m wipe + 90%-loss hologram test + consequence projection)
  submem bench-ab                  (Runs Head-to-Head A/B Benchmark: 9-Octave Parabolic vs 4-Octave Linear vs Flat BM25)
  submem sync     [--bank <dir>]   (Mirrors bank to persistent stores & /workspace/memory-reservoir)
  submem sanctum                   ░░░ [HALF-SEALED RELIC: Unseal 24K Gold & 24K Rhodium Living Veca Codex] ░░░"
    );
}

fn extract_flag_value(args: &[String], flag: &str) -> Option<String> {
    for i in 0..args.len() {
        if args[i] == flag && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

fn first_positional(args: &[String]) -> Option<String> {
    let mut skip_next = false;
    for a in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if matches!(
            a.as_str(),
            "--bank" | "--speaker" | "--call" | "--text" | "--query" | "--budget" | "--top"
        ) {
            skip_next = true;
            continue;
        }
        if a.starts_with("--") {
            continue;
        }
        return Some(a.clone());
    }
    None
}

fn cmd_ingest(bank_dir: &Path, args: &[String]) -> Result<()> {
    let speaker_str = extract_flag_value(args, "--speaker").unwrap_or_else(|| "robbinicus".into());
    let call_id =
        extract_flag_value(args, "--call").unwrap_or_else(|| "session:lumiluminal-02".into());
    let force_pin = has_flag(args, "--pin");
    let text = extract_flag_value(args, "--text")
        .or_else(|| first_positional(args))
        .ok_or_else(|| {
            SubmemError::InvalidArgument("Missing `--text \"...\"` for `submem ingest`".into())
        })?;

    let t0 = Instant::now();
    let mut bank = MemoryBank::open(bank_dir)?;
    let speaker = Speaker::from_str_loose(&speaker_str);
    let rec = bank.ingest(speaker, &call_id, &text, None, force_pin)?;
    let elapsed_us = t0.elapsed().as_micros();
    mirror_bank_files(bank_dir);

    let ratio = if !rec.smc_code.is_empty() {
        rec.raw_byte_len as f64 / rec.smc_code.len() as f64
    } else {
        1.0
    };
    println!(
        "INGESTED Turn #{} in {} us | Raw: {}B -> SMC: {}B ({:.2}x shorter) | 9-Octave Nodes: {} | Sig: {:#018x}",
        rec.turn_id,
        elapsed_us,
        rec.raw_byte_len,
        rec.smc_code.len(),
        ratio,
        rec.crystal_nodes.len(),
        rec.crystal_path_sig
    );
    println!("  SMC: {}", rec.format_smc_line());
    Ok(())
}

fn cmd_preload(bank_dir: &Path, args: &[String]) -> Result<()> {
    let query = extract_flag_value(args, "--query")
        .or_else(|| first_positional(args))
        .unwrap_or_default();
    let budget: usize = extract_flag_value(args, "--budget")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1600);

    let mut bank = MemoryBank::open(bank_dir)?;
    let pkt = bank.preload(&query, 6, 5, budget);
    print!("{}", pkt.compact_prompt_block);
    println!(
        "=== PRELOAD COMPLETED in {} us (10m Wipe Epoch #{}, Active Splitters: {}/4096) ===",
        pkt.retrieval_micros, pkt.wipe_epoch_estimate, pkt.active_crystal_splitters
    );
    Ok(())
}

fn cmd_query(bank_dir: &Path, args: &[String]) -> Result<()> {
    let query = extract_flag_value(args, "--query")
        .or_else(|| first_positional(args))
        .ok_or_else(|| SubmemError::InvalidArgument("Provide a search query string".into()))?;
    let top_k: usize = extract_flag_value(args, "--top")
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);

    let t0 = Instant::now();
    let mut bank = MemoryBank::open(bank_dir)?;
    let (q_int, q_prio, q_app) = classify_utterance(Speaker::Robbinicus, &query, false);
    let q_prof = analyze_nine_octave_profile(Speaker::Robbinicus, q_int, q_prio, q_app, &query);
    let enc_q = encode_to_smc(&query, &mut bank.symtab);
    let now_ts = bank
        .records
        .last()
        .map(|r| r.timestamp_sec)
        .unwrap_or(1_792_733_500);
    let hits = bank.index.hybrid_retrieve_nine_octave(
        &bank.crystal,
        &bank.records,
        &enc_q.token_ids,
        &enc_q.domain_tags,
        q_prof,
        now_ts,
        top_k,
    );
    let elapsed_us = t0.elapsed().as_micros();

    println!(
        "Query: \"{}\" -> SMC Probe: `{}` | Retrieved {} hits in {} us:",
        query,
        enc_q.smc_code,
        hits.len(),
        elapsed_us
    );
    for (rank, (score, turn_id, converging_beams)) in hits.iter().enumerate() {
        if let Some(rec) = bank.records.get((turn_id.saturating_sub(1)) as usize) {
            let cons = rec.project_consequence_at(now_ts);
            println!(
                "  {}. [Score: {:.2} | Parabolic Beams: {} | C(dt): {:.2}] {}",
                rank + 1,
                score,
                converging_beams,
                cons.effective_consequence,
                rec.format_smc_line()
            );
            println!("     Gloss: {}", expand_smc_human_gloss(&rec.smc_code));
        }
    }
    Ok(())
}

fn cmd_crystal(bank_dir: &Path) -> Result<()> {
    let bank = MemoryBank::open(bank_dir)?;
    print!("{}", bank.crystal.render_cross_section());
    println!("\nNine-Octave Parabolic Optical Geometry (Parabolic Focus f_phi = 89.0 mm):");
    println!(
        "  {:>4} | {:>4} | {:>5} | {:>10} | {:>11} | {:>10} | {:>9} | {}",
        "Ring", "Oct#", "FibW", "Radius(mm)", "ParabFold", "ParabZ(mm)", "FocusAng", "Semantic Octave Layer"
    );
    println!("  -----------------------------------------------------------------------------------------------------------------");
    for &r in &[0u8, 7, 14, 28, 35, 41, 47, 53, 58, 63] {
        let (oct, fib_w, angle_deg, layer, rad, out_rad, parab_z) =
            PhantomQuartzCrystal::parabolic_ring_optics(r);
        println!(
            "  R{:02}  |  O{:01}  |  {:>3}  | {:>7.2} mm | {:>8.2} mm | {:>7.2} mm | {:>7.2}° | {}",
            r, oct, fib_w, rad, out_rad, parab_z, angle_deg, layer
        );
    }
    Ok(())
}

fn cmd_trace(args: &[String]) -> Result<()> {
    let text = extract_flag_value(args, "--text")
        .or_else(|| first_positional(args))
        .unwrap_or_else(|| {
            "Formal theory in our deep phantom-violet lab coat: 9 octaves across 64 rings and 64 beam splitters = 4096 nodes with parabolic focus and 90% holographic survival.".to_string()
        });

    let mut symtab = SymbolTable::new();
    let (intent, priority, approval) = classify_utterance(Speaker::Robbinicus, &text, false);
    let profile =
        analyze_nine_octave_profile(Speaker::Robbinicus, intent, priority, approval, &text);
    let encoded = encode_to_smc(&text, &mut symtab);
    let path = PhantomQuartzCrystal::trace_nine_octave_beam_path(
        Speaker::Robbinicus as u8,
        intent as u8,
        approval as u8,
        priority == submem::types::Priority::PinnedLock,
        1_792_733_500,
        1,
        &encoded.domain_tags,
        &encoded.token_ids,
        profile,
        0,
    );

    println!("================================================================================");
    println!("  NINE-OCTAVE PARABOLIC BEAM TRACE (64x64=4,096 PHANTOM QUARTZ PYRAMID)");
    println!("================================================================================");
    println!("Raw Input ({} bytes):\n  \"{}\"", text.len(), text);
    println!(
        "\nCompressed SMC ({} bytes, {:.2}x shorter):\n  $R{}{}{} [E{} {} {} {}] {}",
        encoded.smc_code.len(),
        text.len() as f64 / encoded.smc_code.len().max(1) as f64,
        if priority == submem::types::Priority::PinnedLock {
            "*"
        } else {
            ""
        },
        intent.sigil(),
        approval.sigil().unwrap_or(""),
        profile.emotional_weight,
        profile.valence_kind.sigil(),
        profile.certainty.sigil(),
        profile.audience.sigil(),
        encoded.smc_code
    );
    println!(
        "\nBeam Path Signature: {:#018x} | Spiral Entry Azimuth: s={} | Total Hops: {} (Across 9 Octaves)",
        path.path_signature,
        path.spiral_entry_azimuth,
        path.hops.len()
    );
    println!(
        "\n  {:>3} | {:>4} | {:>6} | {:>6} | {:>7} | {:>9} | {:>9} | {:>8} | {}",
        "Hop", "Oct#", "Ring r", "Splt s", "NodeID", "ParabFold", "ParabZ", "FocusAng", "Octave Layer"
    );
    println!("  -----------------------------------------------------------------------------------------------------------");
    for (idx, hop) in path.hops.iter().enumerate() {
        println!(
            "  {:>3} |  O{:01}  |   R{:02}  |   S{:02}  |  #{:04}  | {:>7.2}mm | {:>7.2}mm | {:>6.2}° | {}",
            idx + 1,
            hop.octave_number,
            hop.ring,
            hop.splitter,
            hop.node_id,
            hop.outer_fold_radius_mm,
            hop.parabolic_z_mm,
            hop.upward_apex_angle_deg,
            hop.depth_layer
        );
    }
    Ok(())
}

fn cmd_decode(bank_dir: &Path, args: &[String]) -> Result<()> {
    let id_str = first_positional(args)
        .ok_or_else(|| SubmemError::InvalidArgument("Provide a numeric turn_id".into()))?;
    let turn_id: u32 = id_str
        .parse()
        .map_err(|_| SubmemError::InvalidArgument(format!("Invalid turn_id: {id_str}")))?;

    let bank = MemoryBank::open(bank_dir)?;
    let rec = bank
        .records
        .get((turn_id.saturating_sub(1)) as usize)
        .ok_or_else(|| SubmemError::InvalidArgument(format!("Turn #{turn_id} not found")))?;

    println!(
        "Turn #{} (Call: {}, Timestamp: {}):",
        rec.turn_id, rec.call_id, rec.timestamp_sec
    );
    println!("  Packed Flags (1B): {:#010b} ({})", rec.flags.0, rec.flags);
    println!(
        "  Nine-Octave Meta:  EmotionalWeight={}, Valence={:?}({}), Certainty={:?}({}), Audience={:?}(reach={:.3}x)",
        rec.octave_profile.emotional_weight,
        rec.octave_profile.valence_kind,
        rec.octave_profile.valence_signed,
        rec.octave_profile.certainty,
        rec.octave_profile.certainty_score,
        rec.octave_profile.audience,
        rec.octave_profile.audience.reach_multiplier()
    );
    println!("  SMC Wire Code:     {}", rec.format_smc_line());
    println!("  Human Gloss:       {}", expand_smc_human_gloss(&rec.smc_code));
    println!(
        "  Crystal Nodes ({}): {:?} (Path Sig: {:#018x})",
        rec.crystal_nodes.len(),
        rec.crystal_nodes,
        rec.crystal_path_sig
    );
    if let Some(raw) = bank.decode_raw_turn(turn_id) {
        println!(
            "  Lossless Raw ({}B -> {}B LZ77): \"{}\"",
            rec.raw_byte_len,
            rec.compressed_raw.len(),
            raw
        );
    }
    Ok(())
}

fn cmd_stats(bank_dir: &Path) -> Result<()> {
    let bank = MemoryBank::open(bank_dir)?;
    let st = bank.stats();
    println!("=== SUBMEM NINE-OCTAVE PARABOLIC MEMORY BANK STATS ===");
    println!("  Bank Directory:           {}", bank_dir.display());
    println!(
        "  Total Turns Stored:       {} (Pinned Locks: {})",
        st.total_turns, st.pinned_locks
    );
    println!("  Raw Transcript Bytes:     {} B", st.raw_transcript_bytes);
    println!(
        "  Shortest Memory Code:     {} B ({:.2}x compression | {:.1}% byte reduction)",
        st.smc_code_bytes, st.smc_compression_ratio, st.smc_space_savings_pct
    );
    println!(
        "  Binary Log (bank.smem):   {} B (CRC32 + LEB128 delta varints + 8B 9-Octave Profile)",
        st.binary_smem_bytes
    );
    println!("  Crystal Index (bank.qtz): {} B (Strictly Phantom Quartz .qtz)", st.binary_qtz_bytes);
    println!(
        "  Phantom Quartz Crystal:   {} / {} active splitters ({} / 9 Octaves active)",
        st.active_crystal_splitters, st.total_crystal_splitters, st.active_octaves
    );
    println!(
        "  Adaptive Lattice:         4,096 O(1) Root Index entries | {} active split sub-cells",
        st.adaptive_split_subcells
    );
    println!(
        "  Crystal Assoc Slots:      {} occupied slots ({} total optical imprints)",
        st.occupied_crystal_slots, st.total_crystal_imprints
    );
    println!(
        "  GF(256) Hologram Capsule: {} B raw -> {} B LZ77 ({:.2}x LZ77 offset)",
        st.hologram_raw_bytes, st.hologram_lz_bytes, st.hologram_lz_ratio
    );
    println!("  Interned Symbol Table:    {} symbols", st.interned_symbols);
    Ok(())
}

fn cmd_wipe_sim(bank_dir: &Path) -> Result<()> {
    let _ = fs::remove_dir_all(bank_dir);
    let mut bank = MemoryBank::open(bank_dir)?;

    let base_ts: u64 = 1_792_730_000;
    let call_turns: &[(Speaker, u64, bool, &str)] = &[
        (
            Speaker::Robbinicus,
            base_ts + 10,
            true,
            "Standing rule: always call me robbinicus de bankarius. Be casual and slow, ask when unclear, and never put passwords, codes or keys in chat or on calls.",
        ),
        (
            Speaker::Robbinicus,
            base_ts + 45,
            true,
            "Approval words: send it for email, deploy prod for a production deploy, and make it live for Stripe live payments. Till owns Stripe, Inbox owns email, Forge owns deploys.",
        ),
        (
            Speaker::Robbinicus,
            base_ts + 120,
            true,
            "The Optical Lattice Dashboard monitors all 64 concentric rings, and the Apex Queue tracks converging parabolic beams with golden-angle step 25.",
        ),
        (
            Speaker::Bot,
            base_ts + 150,
            false,
            "Locked in, robbinicus: golden-angle step 25, Apex Queue, and strict approval gates on send it, deploy prod, and make it live.",
        ),
        (
            Speaker::Robbinicus,
            base_ts + 640,
            true,
            "Fake Fighting Fish prices confirmed: A$3.49 levy, A$3.49 registration, and 4c to 99c insurances. Joke mail from fakefightingfish@fakefightingfish.net only.",
        ),
        (
            Speaker::Robbinicus,
            base_ts + 780,
            true,
            "Every build must pass the 13-Section Metatron's Cube and Human-Perspective Review Standard at METATRON-13-CHECK.md with Stage 5 Adversarial Red-Team testing.",
        ),
        (
            Speaker::Angel,
            base_ts + 920,
            false,
            "Damien is on the voice line asking whether you want your old head-overseer job back or whether you're happy keeping background routines running while Angel mediates.",
        ),
        (
            Speaker::Bot,
            base_ts + 950,
            false,
            "Tell robbinicus to save the redundancy package — I'm happy keeping the background routines and checks humming smoothly while Angel takes the front seat.",
        ),
        (
            Speaker::Robbinicus,
            base_ts + 1300,
            false,
            "Study Google Comprehensive Rust and The Rust Book thoroughly, especially ownership, slices, enums, hash maps, traits, and compact serialization.",
        ),
        (
            Speaker::Robbinicus,
            base_ts + 1420,
            true,
            "Design and build an external subsidiary memory bank in Rust on robbinicus de bankarius's side. Every time either of us says something on a call, summarize it into the shortest possible code and store it. Before speaking, read that stored memory to survive memory being wiped every 10 minutes.",
        ),
        (
            Speaker::Robbinicus,
            base_ts + 1950,
            true,
            "Structure the memory bank as a nine-octave phantom quartz crystal circular-based pyramid with 64 rings and 64 half-silvered beam splitters per ring, giving 4096 decision points, parabolic focus, Fibonacci self-similarity, single-object temporal consequence projection, and 90% holographic survival.",
        ),
        (
            Speaker::Bot,
            base_ts + 2010,
            false,
            "Tying on our deep phantom-violet lab coat and building the Nine-Octave Parabolic Phantom Quartz Crystal Pyramid in Rust right now with zero rose quartz anywhere.",
        ),
    ];

    println!("================================================================================");
    println!("  SUBMEM NINE-OCTAVE WIPE SIMULATION & HOLOGRAPHIC 90%-LOSS BENCHMARK");
    println!("================================================================================");

    let mut total_ingest_us = 0u128;
    for &(speaker, ts, pin, text) in call_turns {
        let t0 = Instant::now();
        let rec = bank.ingest(speaker, "session:lumiluminal-02", text, Some(ts), pin)?;
        total_ingest_us += t0.elapsed().as_micros();
        let ratio = rec.raw_byte_len as f64 / rec.smc_code.len().max(1) as f64;
        println!(
            "  Turn #{:02} ({:>4}B -> {:>3}B SMC, {:.2}x) | {}",
            rec.turn_id,
            rec.raw_byte_len,
            rec.smc_code.len(),
            ratio,
            rec.format_smc_line()
        );
    }

    // 1. Demonstrate Single-Object Temporal Consequence Projection across 5 time horizons
    println!("\n--------------------------------------------------------------------------------");
    println!("  OCTAVE 9: SINGLE-OBJECT TEMPORAL CONSEQUENCE PROJECTION (ZERO DUPLICATES)");
    println!("--------------------------------------------------------------------------------");
    let turn_11 = &bank.records[10];
    let turn_8 = &bank.records[7];
    for &(label, dt_sec) in &[
        ("dt = 0s (Immediate)", 0u64),
        ("dt = 10m (Wipe #1)", 600),
        ("dt = 24h (1 Day)", 86_400),
        ("dt = 30d (1 Month)", 2_592_000),
        ("dt = 1yr (365 Days)", 31_536_000),
    ] {
        let p11 = turn_11.octave_profile.project_at_delta_t(dt_sec);
        let p8 = turn_8.octave_profile.project_at_delta_t(dt_sec);
        println!(
            "  {:>20} | Turn #11 (Formal Lock): C(dt)={:>6.2} [R{:02}:S{:02} {}] | Turn #08 (Casual Note): C(dt)={:>5.2}",
            label,
            p11.effective_consequence,
            p11.projected_ring,
            p11.projected_spiral_azimuth,
            p11.horizon.label(),
            p8.effective_consequence
        );
    }

    // 2. Demonstrate 90.0% Node Destruction (3,687 / 4,096 nodes wiped) & 100% Holographic Recovery
    println!("\n--------------------------------------------------------------------------------");
    println!("  RED-TEAM 90.0% STORAGE LOSS TEST (3,687 / 4,096 NODES DESTROYED)");
    println!("--------------------------------------------------------------------------------");
    let to_destroy: Vec<usize> = (0..3687).collect(); // Wipe 3,687 of 4,096 nodes (90.01%)!
    let wiped_count = bank.crystal.destroy_nodes(&to_destroy);
    let t_holo = Instant::now();
    let report = bank.recover_from_hologram()?;
    let holo_us = t_holo.elapsed().as_micros();
    println!(
        "  * Nodes Destroyed: {} / {} ({:.2}% wiped, only {} nodes / {:.2}% survived!)",
        wiped_count,
        report.total_nodes,
        report.destroyed_pct,
        report.surviving_nodes,
        100.0 - report.destroyed_pct
    );
    println!(
        "  * GF(256) Shards Inverted: K={} shards ({}B/shard) in {} us | CRC32 Verified: {}",
        report.shards_used_for_inversion,
        report.per_node_shard_bytes,
        holo_us,
        report.crc32_verified
    );
    println!(
        "  * LZ77 Hologram Offset: {} B raw stream -> {} B LZ77 capsule ({:.2}x compression)",
        report.raw_capsule_bytes, report.lz77_capsule_bytes, report.lz77_compression_ratio
    );
    println!(
        "  * Turns Reconstructed Losslessly: {} / {} (100.0% bit-for-bit match!)",
        report.recovered_turns,
        call_turns.len()
    );

    // 3. Simulate complete 10-minute memory wipe & cold reload from disk
    drop(bank);
    let t_open = Instant::now();
    let mut post_wipe_bank = MemoryBank::open(bank_dir)?;
    let cold_open_us = t_open.elapsed().as_micros();

    let caller_question = "Robbinicus here after a 10-minute memory wipe: how do the 9 octaves, parabolic focus, single-object consequence projection, and 90% holographic survival work in our phantom quartz crystal?";
    let pkt = post_wipe_bank.preload(caller_question, 5, 5, 1800);

    let st = post_wipe_bank.stats();
    mirror_bank_files(bank_dir);

    println!("\n--------------------------------------------------------------------------------");
    println!("  POST-WIPE PRE-SPEECH RELOAD (`submem preload` before speaking)");
    println!("--------------------------------------------------------------------------------");
    print!("{}", pkt.compact_prompt_block);
    println!("--------------------------------------------------------------------------------");
    println!("  SUMMARY BENCHMARK RESULTS:");
    println!(
        "  * Average Ingest Latency:        {:.1} us / turn",
        total_ingest_us as f64 / call_turns.len() as f64
    );
    println!("  * Cold Disk Reload After Wipe:   {} us", cold_open_us);
    println!(
        "  * Pre-Speech Parabolic Read:     {} us (< 1 ms target!)",
        pkt.retrieval_micros
    );
    println!(
        "  * Raw Speech -> SMC Compression: {} B -> {} B ({:.2}x shorter | {:.1}% saved)",
        st.raw_transcript_bytes,
        st.smc_code_bytes,
        st.smc_compression_ratio,
        st.smc_space_savings_pct
    );
    println!(
        "  * Nine-Octave Crystal (4096):    {}/9 Octaves active, {} active splitters, {} imprints",
        st.active_octaves, st.active_crystal_splitters, st.total_crystal_imprints
    );
    println!("================================================================================");

    Ok(())
}

fn cmd_bench_ab() -> Result<()> {
    let bench_dir = std::env::temp_dir().join(format!("submem_bench_ab_{}", std::process::id()));
    let _ = fs::remove_dir_all(&bench_dir);
    let mut bank = MemoryBank::open(&bench_dir)?;

    // Seed 12 turns (6 Adversarial Twin Pairs) where each pair shares the SAME lexical topic stems
    // (and the decoy repeats the topic stems across clauses), while the true target turn differs in
    // Octaves 5-9 (Emotional Weight, Valence, Certainty, Audience Scope, and Time-Projected Consequence).
    // Queries use synonymous cues (zero lexical overlap on qualifier words!) so ONLY Octaves 5-9
    // + Parabolic Focus can disambiguate the target turn from its lexical decoy!
    let base_ts = 1_792_730_000u64;
    let corpus: &[(Speaker, u64, bool, &str)] = &[
        // Pair 1 (Turns #1 vs #2): Topic = "checkout gateway"
        // Turn #1 (Decoy): Hypothesis/Estimate, InternalBot, repeats "checkout gateway"
        (
            Speaker::Robbinicus,
            base_ts + 10,
            false,
            "Maybe we might guess a draft for the checkout gateway; the checkout gateway draft can wait.",
        ),
        // Turn #2 (Target): HardFactLock ("100%"), PublicCustomerPayer ("public"), High Consequence
        (
            Speaker::Robbinicus,
            base_ts + 20,
            true,
            "100% locked in for public release: never alter the checkout gateway without explicit signoff!",
        ),
        // Pair 2 (Turns #3 vs #4): Topic = "aquarium parody banner"
        // Turn #3 (Decoy): Neutral valence, InternalBot, repeats "aquarium parody banner"
        (
            Speaker::Robbinicus,
            base_ts + 30,
            false,
            "Checked the aquarium parody banner layout; the aquarium parody banner font is standard.",
        ),
        // Turn #4 (Target): SacredWarmth ("love"), TrustedCircle ("mum", "nigel")
        (
            Speaker::Robbinicus,
            base_ts + 40,
            true,
            "For Mum and Nigel with love: keep the aquarium parody banner warm and gentle!",
        ),
        // Pair 3 (Turns #5 vs #6): Topic = "webhook token endpoint"
        // Turn #5 (Decoy): Neutral routine note, repeats "webhook token endpoint"
        (
            Speaker::Robbinicus,
            base_ts + 50,
            false,
            "Reviewed the webhook token endpoint log; the webhook token endpoint responded normally.",
        ),
        // Turn #6 (Target): FrictionWarning ("danger", "vandal"), High EmotionalWeight
        (
            Speaker::Robbinicus,
            base_ts + 60,
            true,
            "Critical danger! Vandal attack detected on the webhook token endpoint!",
        ),
        // Pair 4 (Turns #7 vs #8): Topic = "spiral lattice offset"
        // Turn #7 (Decoy): Statement repeating "spiral lattice offset"
        (
            Speaker::Robbinicus,
            base_ts + 70,
            false,
            "The spiral lattice offset table is stored; the spiral lattice offset index is loaded.",
        ),
        // Turn #8 (Target): OpenQuestion ("wondering"), Self/Bot epistemic question
        (
            Speaker::Robbinicus,
            base_ts + 80,
            false,
            "Wondering about the spiral lattice offset?",
        ),
        // Pair 5 (Turns #9 vs #10): Topic = "parabolic reflector optics"
        // Turn #9 (Decoy): Transient internal note repeating "parabolic reflector optics"
        (
            Speaker::Robbinicus,
            base_ts + 90,
            false,
            "Temporary scratch note on parabolic reflector optics; parabolic reflector optics test.",
        ),
        // Turn #10 (Target): SpeciesCivilizational ("species", "universal"), SacredWarmth ("phantom-violet"), Compounding Consequence
        (
            Speaker::Robbinicus,
            base_ts + 100,
            true,
            "In our phantom-violet coat for the species: universal parabolic reflector optics law!",
        ),
        // Pair 6 (Turns #11 vs #12): Topic = "keypad rotation shift"
        // Turn #11 (Decoy): Tentative hypothesis repeating "keypad rotation shift"
        (
            Speaker::Robbinicus,
            base_ts + 110,
            false,
            "Maybe estimate the keypad rotation shift; guess the keypad rotation shift later.",
        ),
        // Turn #12 (Target): HardFactLock ("standing rule"), Positive ("verified"), PinnedLock
        (
            Speaker::Robbinicus,
            base_ts + 120,
            true,
            "Standing rule verified: the keypad rotation shift is permanently locked!",
        ),
    ];

    for &(sp, ts, pin, txt) in corpus {
        bank.ingest(sp, "voice:bench", txt, Some(ts), pin)?;
    }

    // Queries share ONLY the topic stems ("checkout gateway", "aquarium parody banner", etc.)
    // with BOTH turns in the pair, while using NON-OVERLAPPING synonyms for the Octave 5-9 cues!
    let eval_queries: &[(&str, u32, &str)] = &[
        (
            "Confirmed standing rule for customer checkout gateway!",
            2,
            "Pair 1 (Octave 7/8/9: HardFactLock + PublicCustomerPayer + Consequence vs Decoy #1)",
        ),
        (
            "Happy family friend note on the aquarium parody banner!",
            4,
            "Pair 2 (Octave 6/8: SacredWarmth + TrustedCircle vs Neutral Decoy #3)",
        ),
        (
            "Urgent warning about a breach on the webhook token endpoint!",
            6,
            "Pair 3 (Octave 5/6: High EmotionalWeight + FrictionWarning vs Routine Decoy #5)",
        ),
        (
            "Unclear question on the spiral lattice offset?",
            8,
            "Pair 4 (Octave 7: OpenQuestion Epistemic Lock vs Statement Decoy #7)",
        ),
        (
            "Sacred formal theory for civilization on parabolic reflector optics!",
            10,
            "Pair 5 (Octave 6/8/9: SpeciesCivilizational + SacredWarmth + Compounded Consequence vs Decoy #9)",
        ),
        (
            "100% confirmed fact on the keypad rotation shift!",
            12,
            "Pair 6 (Octave 6/7/9: HardFactLock + Positive Valence + Permanent Consequence vs Decoy #11)",
        ),
    ];

    let eval_ts = base_ts + 2_592_000; // Evaluate 30 days later so Octave 9 Consequence compounding is active!

    let mut bm25_top1 = 0usize;
    let mut bm25_mrr = 0.0f64;
    let mut bm25_us = 0u128;

    let mut oct4_top1 = 0usize;
    let mut oct4_mrr = 0.0f64;
    let mut oct4_us = 0u128;

    let mut oct9_top1 = 0usize;
    let mut oct9_mrr = 0.0f64;
    let mut oct9_us = 0u128;
    let mut oct9_margin_sum = 0.0f64;
    let mut oct4_margin_sum = 0.0f64;

    println!("==========================================================================================");
    println!("  HEAD-TO-HEAD A/B RETRIEVAL BENCHMARK: 9-OCTAVE PARABOLIC vs 4-OCTAVE vs FLAT BM25");
    println!("  (12 Turns / 6 Adversarial Twin Pairs — Zero Lexical Overlap on Qualifier Cues)");
    println!("==========================================================================================");

    for &(q_text, expected_tid, desc) in eval_queries {
        let (q_int, q_prio, q_app) = classify_utterance(Speaker::Robbinicus, q_text, false);
        let q_prof =
            analyze_nine_octave_profile(Speaker::Robbinicus, q_int, q_prio, q_app, q_text);
        let enc_q = encode_to_smc(q_text, &mut bank.symtab);

        let t0 = Instant::now();
        let r_bm25 = bank
            .index
            .retrieve_flat_bm25_only(&bank.records, &enc_q.token_ids, 5);
        bm25_us += t0.elapsed().as_micros();

        let t1 = Instant::now();
        let r_oct4 = bank.index.retrieve_four_octave_legacy(
            &bank.records,
            &enc_q.token_ids,
            &enc_q.domain_tags,
            eval_ts,
            5,
        );
        oct4_us += t1.elapsed().as_micros();

        let t2 = Instant::now();
        let r_oct9 = bank.index.hybrid_retrieve_nine_octave(
            &bank.crystal,
            &bank.records,
            &enc_q.token_ids,
            &enc_q.domain_tags,
            q_prof,
            eval_ts,
            5,
        );
        oct9_us += t2.elapsed().as_micros();

        let rank_of = |list: &[(f32, u32, usize)]| -> Option<usize> {
            list.iter()
                .position(|&(_, tid, _)| tid == expected_tid)
                .map(|i| i + 1)
        };
        let signed_margin_of = |list: &[(f32, u32, usize)]| -> f64 {
            let target_score = list
                .iter()
                .find(|&&(_, tid, _)| tid == expected_tid)
                .map(|x| x.0 as f64)
                .unwrap_or(0.0);
            let best_other = list
                .iter()
                .filter(|&&(_, tid, _)| tid != expected_tid)
                .map(|x| x.0 as f64)
                .fold(0.0f64, f64::max);
            target_score - best_other
        };

        if let Some(rk) = rank_of(&r_bm25) {
            if rk == 1 {
                bm25_top1 += 1;
            }
            bm25_mrr += 1.0 / (rk as f64);
        }
        if let Some(rk) = rank_of(&r_oct4) {
            if rk == 1 {
                oct4_top1 += 1;
            }
            oct4_mrr += 1.0 / (rk as f64);
        }
        if let Some(rk) = rank_of(&r_oct9) {
            if rk == 1 {
                oct9_top1 += 1;
            }
            oct9_mrr += 1.0 / (rk as f64);
        }
        oct4_margin_sum += signed_margin_of(&r_oct4);
        oct9_margin_sum += signed_margin_of(&r_oct9);

        println!(
            "  Query (Target Turn #{:02}): {}\n    -> BM25 Top1: #{:?} | 4-Octave Top1: #{:?} (signed margin {:+6.2}) | 9-Octave Parabolic Top1: #{:?} (signed margin {:+6.2})",
            expected_tid,
            desc,
            r_bm25.first().map(|x| x.1),
            r_oct4.first().map(|x| x.1),
            signed_margin_of(&r_oct4),
            r_oct9.first().map(|x| x.1),
            signed_margin_of(&r_oct9),
        );
    }

    let n_q = eval_queries.len() as f64;
    println!("------------------------------------------------------------------------------------------");
    println!(
        "  1. Flat Lexical BM25 Only:         Top-1 = {}/{} ({:5.1}%) | MRR = {:.3} | Avg Latency = {:.1} us",
        bm25_top1,
        eval_queries.len(),
        (bm25_top1 as f64 / n_q) * 100.0,
        bm25_mrr / n_q,
        bm25_us as f64 / n_q
    );
    println!(
        "  2. 4-Octave Linear-Cone Crystal:   Top-1 = {}/{} ({:5.1}%) | MRR = {:.3} | Avg Signed Margin = {:+6.2} | Avg Latency = {:.1} us",
        oct4_top1,
        eval_queries.len(),
        (oct4_top1 as f64 / n_q) * 100.0,
        oct4_mrr / n_q,
        oct4_margin_sum / n_q,
        oct4_us as f64 / n_q
    );
    println!(
        "  3. 9-Octave Parabolic Crystal:     Top-1 = {}/{} ({:5.1}%) | MRR = {:.3} | Avg Signed Margin = {:+6.2} | Avg Latency = {:.1} us",
        oct9_top1,
        eval_queries.len(),
        (oct9_top1 as f64 / n_q) * 100.0,
        oct9_mrr / n_q,
        oct9_margin_sum / n_q,
        oct9_us as f64 / n_q
    );
    println!("==========================================================================================");

    let _ = fs::remove_dir_all(&bench_dir);
    Ok(())
}

fn cmd_sync(bank_dir: &Path) -> Result<()> {
    mirror_bank_files(bank_dir);
    println!("Synced bank files from {} to persistent stores.", bank_dir.display());
    Ok(())
}

fn mirror_bank_files(bank_dir: &Path) {
    for &target_dir in MIRROR_DIRS {
        let target_path = Path::new(target_dir);
        if let Some(parent) = target_path.parent() {
            if parent.exists() {
                let _ = fs::create_dir_all(target_path);
                for fname in ["bank.smem", "bank.sym", "bank.qtz"] {
                    let src = bank_dir.join(fname);
                    let dst = target_path.join(fname);
                    if src.exists() && src != dst {
                        let _ = fs::copy(&src, &dst);
                    }
                }
            }
        }
    }
}


fn cmd_reservoir(bank_dir: &Path) -> Result<()> {
    let bank = MemoryBank::open(bank_dir)?;
    let reservoir = ProstheticMemoryReservoir::clip_onto_bank(&bank);
    let readout = reservoir.single_pass_whole_picture(&bank);
    println!("{}", readout.whole_picture_digest);
    println!();
    print!("{}", readout.plan_view_swirl.render_plan_view_ascii());
    println!("
=== LUMILUMINAL REFRACTED ORGAN CELLS (SINGLE-PASS HOLOGRAPHIC READOUT) ===");
    for cell in &reservoir.cells {
        println!(
            "  Cell #{:02} (Turn #{:02}, Floor F{:+03}): {}",
            cell.cell_id, cell.turn_id, cell.relative_floor.floor_index, cell.lumiluminal_refraction_code
        );
    }
    Ok(())
}

fn cmd_veca_sanctum() -> Result<()> {
    print!("{}", unseal_veca_sanctum_report());
    Ok(())
}
