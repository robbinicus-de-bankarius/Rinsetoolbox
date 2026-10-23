//! Compact Binary Serialization (`bank.smem` & `bank.qtz`) using `std::io::{Read, Write}` Traits,
//! LEB128 Unsigned Varints, Delta Timestamps, Nine-Octave Profile bytes, and IEEE CRC32 Checksums.
//!
//! Grounded in:
//! - Google's Comprehensive Rust: `Standard Library Traits -> Read and Write` (`fn<R: Read>`, `fn<W: Write>`)
//! - The Rust Book Ch. 10.2 (`Defining Shared Behavior with Traits`) & Ch. 9.2 (`Result<T, E>` + `?`)

use std::io::{Read, Write};

use crate::crystal::{
    gf256_mul, AdaptiveSubCellPair, AssociationSlot, PhantomQuartzCrystal, RootIndexEntry,
    CRYSTAL_RINGS, HOLOGRAM_DATA_SHARDS_K, SLOTS_PER_SPLITTER, SPLITTERS_PER_RING,
};
use crate::error::{Result, SubmemError};
use crate::types::{ApprovalState, NineOctaveProfile, PackedTurnFlags, TurnRecord};

pub const SMEM_MAGIC: [u8; 4] = *b"SMEM";
pub const SMEM_VERSION: u8 = 2;

/// Phantom Quartz `.qtz` magic (`PQTZ` — strictly Phantom Quartz, zero rose quartz!).
pub const QTZ_MAGIC: [u8; 4] = *b"PQTZ";
pub const QTZ_VERSION: u8 = 2;

/// Generic trait for compact binary encoding/decoding over any byte stream `R: Read` / `W: Write`.
pub trait CompactCodec: Sized {
    fn encode_to<W: Write>(&self, writer: &mut W) -> Result<usize>;
    fn decode_from<R: Read>(reader: &mut R) -> Result<Self>;
}

/// Writes an unsigned 64-bit integer in compact LEB128 variable-length format (1..10 bytes).
/// Values `0..=127` take **1 single byte**!
#[inline]
pub fn write_varint_u64<W: Write>(writer: &mut W, mut val: u64) -> Result<usize> {
    let mut written = 0usize;
    loop {
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            byte |= 0x80;
        }
        writer.write_all(&[byte])?;
        written += 1;
        if val == 0 {
            break;
        }
    }
    Ok(written)
}

/// Reads an unsigned 64-bit integer encoded in LEB128 format.
#[inline]
pub fn read_varint_u64<R: Read>(reader: &mut R) -> Result<u64> {
    let mut result = 0u64;
    let mut shift = 0u32;
    let mut buf = [0u8; 1];
    loop {
        reader.read_exact(&mut buf).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                SubmemError::TruncatedFrame {
                    offset: 0,
                    detail: "unexpected EOF while reading LEB128 varint",
                }
            } else {
                SubmemError::Io(e)
            }
        })?;
        let byte = buf[0];
        result |= ((byte & 0x7F) as u64) << shift;
        if (byte & 0x80) == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift >= 64 {
            return Err(SubmemError::TruncatedFrame {
                offset: 0,
                detail: "LEB128 varint overflow (>64 bits)",
            });
        }
    }
}

#[inline]
pub fn write_str<W: Write>(writer: &mut W, s: &str) -> Result<usize> {
    let bytes = s.as_bytes();
    let mut n = write_varint_u64(writer, bytes.len() as u64)?;
    writer.write_all(bytes)?;
    n += bytes.len();
    Ok(n)
}

#[inline]
pub fn read_str<R: Read>(reader: &mut R) -> Result<String> {
    let len = read_varint_u64(reader)? as usize;
    if len > 10 * 1024 * 1024 {
        return Err(SubmemError::TruncatedFrame {
            offset: 0,
            detail: "string length exceeds 10MB safety cap",
        });
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}

/// Computes standard IEEE 802.3 CRC32 checksum over a byte slice `&[u8]`.
pub fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// Encodes a single `TurnRecord` into a compact binary frame with LEB128 delta timestamp,
/// 1-byte bit-packed flags, 8-byte `NineOctaveProfile`, and a leading CRC32 checksum.
pub fn encode_turn_frame<W: Write>(
    writer: &mut W,
    rec: &TurnRecord,
    prev_timestamp_sec: u64,
) -> Result<usize> {
    let mut payload = Vec::with_capacity(144);

    write_varint_u64(&mut payload, rec.turn_id as u64)?;
    let delta_ts = rec.timestamp_sec.saturating_sub(prev_timestamp_sec);
    write_varint_u64(&mut payload, delta_ts)?;
    payload.write_all(&[rec.flags.0])?;
    if rec.flags.has_approval() {
        payload.write_all(&[rec.approval as u8])?;
    }
    write_str(&mut payload, &rec.call_id)?;
    write_str(&mut payload, &rec.smc_code)?;

    write_varint_u64(&mut payload, rec.smc_token_ids.len() as u64)?;
    for &id in &rec.smc_token_ids {
        write_varint_u64(&mut payload, id as u64)?;
    }

    write_varint_u64(&mut payload, rec.domain_tags.len() as u64)?;
    for tag in &rec.domain_tags {
        write_str(&mut payload, tag)?;
    }

    write_varint_u64(&mut payload, rec.crystal_nodes.len() as u64)?;
    for &node in &rec.crystal_nodes {
        write_varint_u64(&mut payload, node as u64)?;
    }

    payload.write_all(&rec.crystal_path_sig.to_le_bytes())?;
    payload.write_all(&rec.octave_profile.to_bytes())?;
    write_varint_u64(&mut payload, rec.raw_byte_len as u64)?;
    write_varint_u64(&mut payload, rec.compressed_raw.len() as u64)?;
    payload.write_all(&rec.compressed_raw)?;

    let crc = crc32_ieee(&payload);
    let mut total_written = write_varint_u64(writer, payload.len() as u64)?;
    writer.write_all(&crc.to_le_bytes())?;
    total_written += 4;
    writer.write_all(&payload)?;
    total_written += payload.len();

    Ok(total_written)
}

/// Decodes a single `TurnRecord` frame and verifies its IEEE CRC32 checksum.
pub fn decode_turn_frame<R: Read>(
    reader: &mut R,
    prev_timestamp_sec: u64,
) -> Result<Option<TurnRecord>> {
    let mut first_byte = [0u8; 1];
    match reader.read_exact(&mut first_byte) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(SubmemError::Io(e)),
    }

    let mut frame_len = (first_byte[0] & 0x7F) as u64;
    if (first_byte[0] & 0x80) != 0 {
        let mut shift = 7u32;
        let mut buf = [0u8; 1];
        loop {
            reader.read_exact(&mut buf)?;
            frame_len |= ((buf[0] & 0x7F) as u64) << shift;
            if (buf[0] & 0x80) == 0 {
                break;
            }
            shift += 7;
        }
    }

    let mut crc_bytes = [0u8; 4];
    reader.read_exact(&mut crc_bytes)?;
    let expected_crc = u32::from_le_bytes(crc_bytes);

    let mut payload = vec![0u8; frame_len as usize];
    reader.read_exact(&mut payload).map_err(|_| SubmemError::TruncatedFrame {
        offset: frame_len,
        detail: "truncated turn frame payload",
    })?;

    let actual_crc = crc32_ieee(&payload);
    let mut cursor: &[u8] = &payload;
    let turn_id = read_varint_u64(&mut cursor)? as u32;
    if actual_crc != expected_crc {
        return Err(SubmemError::ChecksumMismatch {
            turn_id,
            expected: expected_crc,
            actual: actual_crc,
        });
    }

    let delta_ts = read_varint_u64(&mut cursor)?;
    let timestamp_sec = prev_timestamp_sec.saturating_add(delta_ts);

    let mut flag_b = [0u8; 1];
    cursor.read_exact(&mut flag_b)?;
    let flags = PackedTurnFlags(flag_b[0]);

    let approval = if flags.has_approval() {
        let mut app_b = [0u8; 1];
        cursor.read_exact(&mut app_b)?;
        ApprovalState::from_u8(app_b[0])
    } else {
        ApprovalState::None
    };

    let call_id = read_str(&mut cursor)?;
    let smc_code = read_str(&mut cursor)?;

    let tok_count = read_varint_u64(&mut cursor)? as usize;
    let mut smc_token_ids = Vec::with_capacity(tok_count);
    for _ in 0..tok_count {
        smc_token_ids.push(read_varint_u64(&mut cursor)? as u16);
    }

    let tag_count = read_varint_u64(&mut cursor)? as usize;
    let mut domain_tags = Vec::with_capacity(tag_count);
    for _ in 0..tag_count {
        domain_tags.push(read_str(&mut cursor)?);
    }

    let node_count = read_varint_u64(&mut cursor)? as usize;
    let mut crystal_nodes = Vec::with_capacity(node_count);
    for _ in 0..node_count {
        crystal_nodes.push(read_varint_u64(&mut cursor)? as u16);
    }

    let mut sig_bytes = [0u8; 8];
    cursor.read_exact(&mut sig_bytes)?;
    let crystal_path_sig = u64::from_le_bytes(sig_bytes);

    let mut prof_bytes = [0u8; 8];
    cursor.read_exact(&mut prof_bytes)?;
    let octave_profile = NineOctaveProfile::from_bytes(prof_bytes);

    let raw_byte_len = read_varint_u64(&mut cursor)? as usize;
    let comp_len = read_varint_u64(&mut cursor)? as usize;
    let mut compressed_raw = vec![0u8; comp_len];
    cursor.read_exact(&mut compressed_raw)?;

    Ok(Some(TurnRecord {
        turn_id,
        timestamp_sec,
        flags,
        approval,
        call_id,
        smc_code,
        smc_token_ids,
        domain_tags,
        crystal_nodes,
        crystal_path_sig,
        compressed_raw,
        raw_byte_len,
        octave_profile,
    }))
}

/// Ultra-compact sparse binary serializer for the Nine-Octave Parabolic Phantom Quartz Crystal (`bank.qtz`).
impl CompactCodec for PhantomQuartzCrystal {
    fn encode_to<W: Write>(&self, writer: &mut W) -> Result<usize> {
        let mut payload = Vec::with_capacity(4096);
        write_varint_u64(&mut payload, self.total_imprints)?;

        // 1. Collect active primary splitters
        let mut active = Vec::new();
        for r in 0..CRYSTAL_RINGS {
            for s in 0..SPLITTERS_PER_RING {
                let node = &self.rings[r][s];
                if node.is_active() {
                    active.push(node);
                }
            }
        }

        write_varint_u64(&mut payload, active.len() as u64)?;
        for node in active {
            payload.write_all(&[
                node.ring,
                node.splitter,
                node.write_cursor,
                node.collision_count,
            ])?;
            write_varint_u64(&mut payload, node.transmitted_count as u64)?;
            write_varint_u64(&mut payload, node.reflected_apex_count as u64)?;
            let occ_count = node.slots.iter().filter(|sl| sl.turn_id != 0).count();
            payload.write_all(&[occ_count as u8])?;
            for slot in node.slots.iter().filter(|sl| sl.turn_id != 0) {
                write_varint_u64(&mut payload, slot.turn_id as u64)?;
                payload.write_all(&slot.phase_sig.to_le_bytes())?;
                payload.write_all(&[slot.apex_amplitude, slot.is_pinned])?;
            }
        }

        // 2. Serialize adaptive Fibonacci sub-cells
        write_varint_u64(&mut payload, self.adaptive_subcells.len() as u64)?;
        for sub in &self.adaptive_subcells {
            write_varint_u64(&mut payload, sub.parent_node_id as u64)?;
            payload.write_all(&[sub.split_depth, u8::from(sub.is_merged)])?;
            write_varint_u64(&mut payload, sub.split_timestamp_sec)?;
            write_varint_u64(&mut payload, sub.last_hit_timestamp_sec)?;
            for slots_arr in [&sub.left_slots, &sub.right_slots] {
                let occ = slots_arr.iter().filter(|sl| sl.turn_id != 0).count();
                payload.write_all(&[occ as u8])?;
                for slot in slots_arr.iter().filter(|sl| sl.turn_id != 0) {
                    write_varint_u64(&mut payload, slot.turn_id as u64)?;
                    payload.write_all(&slot.phase_sig.to_le_bytes())?;
                    payload.write_all(&[slot.apex_amplitude, slot.is_pinned])?;
                }
            }
        }

        // 3. Serialize Holographic GF(256) metadata + 8 canonical shards (offset by LZ77!)
        write_varint_u64(&mut payload, self.hologram_lz_len as u64)?;
        write_varint_u64(&mut payload, self.hologram_raw_len as u64)?;
        payload.write_all(&self.hologram_crc32.to_le_bytes())?;

        if self.hologram_lz_len > 0 {
            // Find K=8 distinct evaluation point shards to store compactly on disk
            let mut stored_pts = Vec::with_capacity(HOLOGRAM_DATA_SHARDS_K);
            for nid in 0..crate::crystal::TOTAL_SPLITTERS {
                let sh = &self.hologram_shards[nid];
                if !sh.is_empty() {
                    let r = ((nid >> 6) & 63) as u8;
                    let s = (nid & 63) as u8;
                    let x = PhantomQuartzCrystal::node_gf256_eval_point(r, s);
                    if !stored_pts.iter().any(|&(px, _)| px == x) {
                        stored_pts.push((x, sh.as_slice()));
                        if stored_pts.len() == HOLOGRAM_DATA_SHARDS_K {
                            break;
                        }
                    }
                }
            }
            write_varint_u64(&mut payload, stored_pts.len() as u64)?;
            for (x, sh) in stored_pts {
                payload.write_all(&[x])?;
                write_varint_u64(&mut payload, sh.len() as u64)?;
                payload.write_all(sh)?;
            }
        } else {
            write_varint_u64(&mut payload, 0)?;
        }

        let crc = crc32_ieee(&payload);
        writer.write_all(&QTZ_MAGIC)?;
        writer.write_all(&[QTZ_VERSION])?;
        writer.write_all(&crc.to_le_bytes())?;
        let mut written = 9 + write_varint_u64(writer, payload.len() as u64)?;
        writer.write_all(&payload)?;
        written += payload.len();
        Ok(written)
    }

    fn decode_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if magic != QTZ_MAGIC {
            return Err(SubmemError::InvalidMagic(magic));
        }
        let mut ver = [0u8; 1];
        reader.read_exact(&mut ver)?;
        if ver[0] != QTZ_VERSION {
            return Err(SubmemError::UnsupportedVersion(ver[0]));
        }
        let mut crc_b = [0u8; 4];
        reader.read_exact(&mut crc_b)?;
        let expected_crc = u32::from_le_bytes(crc_b);

        let len = read_varint_u64(reader)? as usize;
        let mut payload = vec![0u8; len];
        reader.read_exact(&mut payload)?;
        let actual_crc = crc32_ieee(&payload);
        if actual_crc != expected_crc {
            return Err(SubmemError::ChecksumMismatch {
                turn_id: 0,
                expected: expected_crc,
                actual: actual_crc,
            });
        }

        let mut cursor: &[u8] = &payload;
        let total_imprints = read_varint_u64(&mut cursor)?;
        let active_count = read_varint_u64(&mut cursor)? as usize;

        let mut crystal = PhantomQuartzCrystal::new();
        crystal.total_imprints = total_imprints;

        for _ in 0..active_count {
            let mut hdr = [0u8; 4];
            cursor.read_exact(&mut hdr)?;
            let r = (hdr[0] as usize) & 63;
            let s = (hdr[1] as usize) & 63;
            let write_cursor = hdr[2];
            let collision_count = hdr[3];
            let transmitted_count = read_varint_u64(&mut cursor)? as u16;
            let reflected_apex_count = read_varint_u64(&mut cursor)? as u16;
            let mut occ_b = [0u8; 1];
            cursor.read_exact(&mut occ_b)?;
            let occ = (occ_b[0] as usize).min(SLOTS_PER_SPLITTER);

            let node = &mut crystal.rings[r][s];
            node.write_cursor = write_cursor;
            node.collision_count = collision_count;
            node.transmitted_count = transmitted_count;
            node.reflected_apex_count = reflected_apex_count;

            for idx in 0..occ {
                let turn_id = read_varint_u64(&mut cursor)? as u32;
                let mut ph = [0u8; 2];
                cursor.read_exact(&mut ph)?;
                let phase_sig = u16::from_le_bytes(ph);
                let mut tail = [0u8; 2];
                cursor.read_exact(&mut tail)?;
                node.slots[idx] = AssociationSlot {
                    turn_id,
                    phase_sig,
                    apex_amplitude: tail[0],
                    is_pinned: tail[1],
                };
            }
            let nid = ((r << 6) | s) as u16;
            crystal.root_index[nid as usize] = RootIndexEntry::pack(nid, true, false, 0, 0);
        }

        let sub_count = read_varint_u64(&mut cursor)? as usize;
        for idx in 0..sub_count {
            let parent_node_id = read_varint_u64(&mut cursor)? as u16;
            let mut meta = [0u8; 2];
            cursor.read_exact(&mut meta)?;
            let split_depth = meta[0];
            let is_merged = meta[1] != 0;
            let split_timestamp_sec = read_varint_u64(&mut cursor)?;
            let last_hit_timestamp_sec = read_varint_u64(&mut cursor)?;

            let mut left_slots = [AssociationSlot::default(); SLOTS_PER_SPLITTER];
            let mut right_slots = [AssociationSlot::default(); SLOTS_PER_SPLITTER];

            for target in [&mut left_slots, &mut right_slots] {
                let mut occ_b = [0u8; 1];
                cursor.read_exact(&mut occ_b)?;
                let occ = (occ_b[0] as usize).min(SLOTS_PER_SPLITTER);
                for s_idx in 0..occ {
                    let turn_id = read_varint_u64(&mut cursor)? as u32;
                    let mut ph = [0u8; 2];
                    cursor.read_exact(&mut ph)?;
                    let phase_sig = u16::from_le_bytes(ph);
                    let mut tail = [0u8; 2];
                    cursor.read_exact(&mut tail)?;
                    target[s_idx] = AssociationSlot {
                        turn_id,
                        phase_sig,
                        apex_amplitude: tail[0],
                        is_pinned: tail[1],
                    };
                }
            }

            crystal.adaptive_subcells.push(AdaptiveSubCellPair {
                parent_node_id,
                split_depth,
                is_merged,
                split_timestamp_sec,
                last_hit_timestamp_sec,
                left_slots,
                right_slots,
            });
            if !is_merged && (parent_node_id as usize) < crate::crystal::TOTAL_SPLITTERS {
                crystal.root_index[parent_node_id as usize] = RootIndexEntry::pack(
                    parent_node_id,
                    true,
                    true,
                    split_depth,
                    (idx + 1) as u16,
                );
            }
        }

        crystal.hologram_lz_len = read_varint_u64(&mut cursor)? as usize;
        crystal.hologram_raw_len = read_varint_u64(&mut cursor)? as usize;
        let mut hcrc = [0u8; 4];
        cursor.read_exact(&mut hcrc)?;
        crystal.hologram_crc32 = u32::from_le_bytes(hcrc);

        let shard_count = read_varint_u64(&mut cursor)? as usize;
        if shard_count == HOLOGRAM_DATA_SHARDS_K && crystal.hologram_lz_len > 0 {
            let mut pts = [0u8; HOLOGRAM_DATA_SHARDS_K];
            let mut shards = Vec::with_capacity(HOLOGRAM_DATA_SHARDS_K);
            for i in 0..HOLOGRAM_DATA_SHARDS_K {
                let mut xb = [0u8; 1];
                cursor.read_exact(&mut xb)?;
                pts[i] = xb[0];
                let slen = read_varint_u64(&mut cursor)? as usize;
                let mut sbytes = vec![0u8; slen];
                cursor.read_exact(&mut sbytes)?;
                shards.push(sbytes);
            }
            // Invert the K stored shards to populate all 4,096 node shards in memory
            let shard_len = shards[0].len();
            let mut v_mat = [[0u8; HOLOGRAM_DATA_SHARDS_K]; HOLOGRAM_DATA_SHARDS_K];
            for (row, &x) in pts.iter().enumerate() {
                let mut p = 1u8;
                for col in 0..HOLOGRAM_DATA_SHARDS_K {
                    v_mat[row][col] = p;
                    p = gf256_mul(p, x);
                }
            }
            if let Some(v_inv) = crate::crystal::gf256_invert_8x8(v_mat) {
                let mut padded = vec![0u8; shard_len * HOLOGRAM_DATA_SHARDS_K];
                for j in 0..HOLOGRAM_DATA_SHARDS_K {
                    let out_b = &mut padded[j * shard_len..(j + 1) * shard_len];
                    for m in 0..HOLOGRAM_DATA_SHARDS_K {
                        let c = v_inv[j][m];
                        if c == 0 {
                            continue;
                        }
                        for b_idx in 0..shard_len {
                            out_b[b_idx] ^= gf256_mul(c, shards[m][b_idx]);
                        }
                    }
                }
                let mut eval_shards: Vec<Vec<u8>> = vec![vec![0u8; shard_len]; 255];
                for x in 1..=254u8 {
                    let mut coeffs = [1u8; HOLOGRAM_DATA_SHARDS_K];
                    for j in 1..HOLOGRAM_DATA_SHARDS_K {
                        coeffs[j] = gf256_mul(coeffs[j - 1], x);
                    }
                    let out_sh = &mut eval_shards[x as usize];
                    for j in 0..HOLOGRAM_DATA_SHARDS_K {
                        let c = coeffs[j];
                        let d_slice = &padded[j * shard_len..(j + 1) * shard_len];
                        for (b_idx, &dv) in d_slice.iter().enumerate() {
                            out_sh[b_idx] ^= gf256_mul(c, dv);
                        }
                    }
                }
                for r in 0..CRYSTAL_RINGS {
                    for s in 0..SPLITTERS_PER_RING {
                        let nid = (r << 6) | s;
                        let x = PhantomQuartzCrystal::node_gf256_eval_point(r as u8, s as u8);
                        crystal.hologram_shards[nid] = eval_shards[x as usize].clone();
                    }
                }
            }
        }

        Ok(crystal)
    }
}
