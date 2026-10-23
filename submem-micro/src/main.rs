use submem_micro::*;

fn main() {
    let payload = "Robbinicus de Bankarius — 9-Octave Phantom Quartz Holographic Memory Micro-Sandbox Proof (2026-10-23). Whole reflected in every part across 64 golden-angle splitters.".as_bytes();
    let (lattice, steps, wipe) = simulate_phone_transfer(
        10_000_000, // 10,000,000 bytes baseline regular files on phone
        262_144,    // 262,144 bytes (256 KB) session file moved into lattice
        "turn-17-phantom-quartz-proof",
        payload,
    );
    let fp = lattice.footprint_report();

    println!("================================================================================");
    println!(" SUBMEM-MICRO v0.2.0 — FIXED-FOOTPRINT HOLOGRAPHIC QUARTZ SANDBOX (NO DEPLOY)");
    println!("================================================================================");
    println!(" FIXED LATTICE MEMORY FOOTPRINT:");
    println!("   Resident Block Size : {:>10} bytes (exact 64 KB fixed allocation)", fp.resident_bytes);
    println!("   Splitter Nodes      : {:>10} nodes (8x8 micro-grid)", fp.node_count);
    println!("   Bytes per Node      : {:>10} bytes (Octave0={}B, Octave1={}B, Octave2={}B)",
        fp.bytes_per_node, fp.octave_breakdown_bytes[0], fp.octave_breakdown_bytes[1], fp.octave_breakdown_bytes[2]);
    println!("   Octaves             : {:>10} nested octaves", fp.octaves);
    println!("   Golden-Angle Step   : {:>10} (25 mod 64, gcd(25,64)=1 full cycle)", fp.golden_step);
    println!("--------------------------------------------------------------------------------");
    println!(" PHONE STORAGE -> LATTICE TRANSFER VISUALIZATION:");
    for s in &steps {
        println!("   {}", s.stage);
        println!("     Phone Regular Files : {:>12} bytes", s.phone_regular_files_bytes);
        println!("     Submem Lattice Block: {:>12} bytes", s.lattice_block_bytes);
        println!("     PHONE TOTAL USED    : {:>12} bytes (Delta vs start: {:+10} bytes)",
            s.phone_total_used_bytes, s.delta_vs_start_bytes);
    }
    println!("--------------------------------------------------------------------------------");
    println!(" RED-TEAM 90% WIPE-AND-RECONSTRUCT SURVIVAL TEST:");
    println!("   Nodes Destroyed     : {:>10} / {} ({:.2}% wiped)", wipe.wiped_nodes, wipe.total_nodes, wipe.destruction_percent);
    println!("   Nodes Surviving     : {:>10} / {} ({:.2}% surviving)", wipe.surviving_nodes, wipe.total_nodes, 100.0 - wipe.destruction_percent);
    println!("   Reconstructed Bytes : {:>10} bytes", wipe.reconstructed_len);
    println!("   Bit-for-Bit Match   : {:>10}", if wipe.bit_for_bit_match { "100% EXACT PASS" } else { "FAIL" });
    println!("   Reconstruction Time : {:>10} ns ({:.3} us)", wipe.reconstruction_ns, wipe.reconstruction_ns as f64 / 1000.0);
    println!("   Associative Recall  : {:>10} ns ({:.3} us)", wipe.recall_ns, wipe.recall_ns as f64 / 1000.0);
    println!("================================================================================");
}
