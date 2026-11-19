//! Mycelium symbolic cycle integrated with the existing `submem` reservoir.
//!
//! This is deterministic creative/simulation software. It does not provide
//! cryptographic security, clinical value, or evidence of metaphysical effects.

use crate::error::{Result, SubmemError};
use crate::reservoir::PlanViewInterferenceSwirl;
use crate::veca_sanctum::{
    MetatronSanctumProof, METATRON_13_CIRCLES, METATRON_CIRCLES_COUNT,
};
use std::f64::consts::PI;
use std::fmt::Write as _;

pub const NODE_COUNT: usize = METATRON_CIRCLES_COUNT;
pub const DEFAULT_SEED: u64 = 0x1234_5678_9abc_def0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Refuser,
    Silencer,
    Doubter,
}

impl Role {
    pub const ALL: [Role; 3] = [Role::Refuser, Role::Silencer, Role::Doubter];

    pub fn name(self) -> &'static str {
        match self {
            Role::Refuser => "Refuser",
            Role::Silencer => "Silencer",
            Role::Doubter => "Doubter",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    Physical,
    Emotional,
    Mental,
    Causal,
}

impl Body {
    pub const ALL: [Body; 4] = [
        Body::Physical,
        Body::Emotional,
        Body::Mental,
        Body::Causal,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chakra {
    Root,
    Sacral,
    Solar,
    Heart,
    Throat,
    ThirdEye,
    Crown,
}

impl Chakra {
    pub const ALL: [Chakra; 7] = [
        Chakra::Root,
        Chakra::Sacral,
        Chakra::Solar,
        Chakra::Heart,
        Chakra::Throat,
        Chakra::ThirdEye,
        Chakra::Crown,
    ];

    pub fn colour(self) -> &'static str {
        match self {
            Chakra::Root => "#B84A4A",
            Chakra::Sacral => "#D47A3A",
            Chakra::Solar => "#E6C84A",
            Chakra::Heart => "#3FA06A",
            Chakra::Throat => "#2F6FA8",
            Chakra::ThirdEye => "#2C1660",
            Chakra::Crown => "#7B4EA3",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: usize,
    pub state: f64,
    pub energy: f64,
    pub suppressed: bool,
    pub role: Role,
    pub body: Body,
    pub chakra: Chakra,
    pub history: Vec<f64>,
}

impl Node {
    fn new(id: usize) -> Self {
        Self {
            id,
            state: 0.5,
            energy: 1.0,
            suppressed: false,
            role: Role::ALL[id % Role::ALL.len()],
            body: Body::ALL[id % Body::ALL.len()],
            chakra: Chakra::ALL[id % Chakra::ALL.len()],
            history: vec![0.5],
        }
    }
}

pub fn seeded_nodes() -> Vec<Node> {
    (0..NODE_COUNT).map(Node::new).collect()
}

pub fn coverage_complete(nodes: &[Node]) -> bool {
    Role::ALL.iter().all(|role| nodes.iter().any(|n| n.role == *role))
        && Body::ALL.iter().all(|body| nodes.iter().any(|n| n.body == *body))
        && Chakra::ALL
            .iter()
            .all(|chakra| nodes.iter().any(|n| n.chakra == *chakra))
}

#[derive(Clone, Debug, PartialEq)]
pub struct EngineConfig {
    pub signal: f64,
    pub noise_amplitude: f64,
    pub swirl_turns: f64,
    pub random_seed: u64,
    pub aperture_open: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            signal: 0.51,
            noise_amplitude: 0.15,
            swirl_turns: 100.0,
            random_seed: DEFAULT_SEED,
            aperture_open: true,
        }
    }
}

impl EngineConfig {
    fn validate(&self) -> Result<()> {
        if !self.signal.is_finite() || !(0.0..=1.0).contains(&self.signal) {
            return Err(SubmemError::InvalidArgument(
                "signal must be finite and within [0, 1]".into(),
            ));
        }
        if !self.noise_amplitude.is_finite()
            || !(0.0..=1.0).contains(&self.noise_amplitude)
        {
            return Err(SubmemError::InvalidArgument(
                "noise_amplitude must be finite and within [0, 1]".into(),
            ));
        }
        if !self.swirl_turns.is_finite() {
            return Err(SubmemError::InvalidArgument(
                "swirl_turns must be finite".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct XorShift64(u64);

impl XorShift64 {
    fn new(seed: u64) -> Self {
        // Xorshift's all-zero state is absorbing.
        Self(if seed == 0 { DEFAULT_SEED } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn next_f64(&mut self) -> f64 {
        self.next_u64() as f64 / u64::MAX as f64
    }
}

/// Marks lower-energy neighbours as suppressed without deleting nodes.
/// Keeping indices stable preserves the K13 geometry and role assignments.
pub fn lateral_inhibition(nodes: &mut [Node]) {
    let states: Vec<f64> = nodes.iter().map(|n| n.state).collect();
    let energies: Vec<f64> = nodes.iter().map(|n| n.energy).collect();

    for i in 0..nodes.len() {
        nodes[i].suppressed = (0..nodes.len()).any(|j| {
            i != j
                && (states[i] - states[j]).abs() < 0.05
                && energies[i] < energies[j]
        });
    }
}

fn stochastic_resonance_with_rng(
    nodes: &mut [Node],
    config: &EngineConfig,
    rng: &mut XorShift64,
) {
    for n in nodes.iter_mut() {
        if !n.suppressed {
            let noisy = config.signal
                + (rng.next_f64() - 0.5) * 2.0 * config.noise_amplitude;
            if noisy > 0.5 && n.state <= 0.5 {
                n.state = (n.state + 0.25).min(1.0);
            }
        }
        n.history.push(n.state);
    }
}

pub fn activity_ratio(nodes: &[Node]) -> f64 {
    if nodes.is_empty() {
        return 0.0;
    }
    let active = nodes
        .iter()
        .filter(|n| {
            n.history.len() > 1
                && n.history[n.history.len() - 1] != n.history[n.history.len() - 2]
        })
        .count();
    active as f64 / nodes.len() as f64
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeReport {
    pub before: f64,
    pub after: f64,
}

fn regulate_edge_of_chaos(nodes: &mut [Node], rng: &mut XorShift64) -> EdgeReport {
    let before = activity_ratio(nodes);

    if before < 0.3 {
        for n in nodes.iter_mut() {
            if !n.suppressed {
                n.state = ((n.state + rng.next_f64()) / 2.0).clamp(0.0, 1.0);
            }
            n.history.push(n.state);
        }
    } else if before > 0.7 {
        for n in nodes.iter_mut() {
            if !n.suppressed {
                n.state = 0.5 + (n.state - 0.5) * 0.5;
            }
            n.history.push(n.state);
        }
    }

    EdgeReport {
        before,
        after: activity_ratio(nodes),
    }
}

/// A symbolic phase derived from three names, not a physical antenna measurement.
pub fn antenna_phase() -> f64 {
    let byte_sum = b"Michael".iter().map(|c| *c as u32).sum::<u32>()
        + b"Uriel".iter().map(|c| *c as u32).sum::<u32>()
        + b"Metatron".iter().map(|c| *c as u32).sum::<u32>();
    (byte_sum as f64 / 1000.0) % (2.0 * PI)
}

fn apply_aperture(nodes: &mut [Node], open: bool) -> f64 {
    if !open {
        return 0.0;
    }
    let phase = antenna_phase();
    for n in nodes.iter_mut() {
        if n.chakra == Chakra::Crown {
            n.state = (n.state + 0.05 * phase.sin()).clamp(0.0, 1.0);
            n.history.push(n.state);
        }
    }
    phase
}


#[derive(Clone, Debug, PartialEq)]
pub struct FilterDecision {
    pub candidate: f64,
    pub accepted: bool,
    pub reasons: Vec<String>,
}

/// Evaluates transparent Via Negativa rules. Invalid input or an incomplete
/// role set fails closed and returns human-readable reasons.
pub fn via_negativa(nodes: &[Node], candidate: f64) -> FilterDecision {
    if !candidate.is_finite() || !(0.0..=1.0).contains(&candidate) {
        return FilterDecision {
            candidate,
            accepted: false,
            reasons: vec!["candidate must be finite and within [0, 1]".into()],
        };
    }

    let mut reasons = Vec::new();
    for role in Role::ALL {
        if !nodes.iter().any(|n| n.role == role) {
            reasons.push(format!("missing required role: {}", role.name()));
        }
    }

    let refuser_count = nodes.iter().filter(|n| n.role == Role::Refuser).count();
    if refuser_count > 0 && candidate < 0.25 {
        reasons.push("Refuser rejected candidate below 0.25".into());
    }
    if nodes.iter().any(|n| n.role == Role::Silencer)
        && (candidate - 0.5).abs() < 0.03
    {
        reasons.push("Silencer rejected candidate within 0.03 of 0.50".into());
    }
    if nodes.iter().any(|n| n.role == Role::Doubter)
        && candidate > 0.92
        && refuser_count < 2
    {
        reasons.push("Doubter requires at least two Refusers above 0.92".into());
    }

    FilterDecision {
        candidate,
        accepted: reasons.is_empty(),
        reasons,
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct CycleReport {
    pub stages: Vec<&'static str>,
    pub node_count: usize,
    pub metatron_chords: usize,
    pub metatron_field_nodes: usize,
    pub metatron_connections: usize,
    pub raw_interior_crossings: usize,
    pub lit: usize,
    pub unlit: usize,
    pub swirl_balance: f64,
    pub swirl_phase: f64,
    pub lambda_before: f64,
    pub lambda_after: f64,
    pub final_activity: f64,
    pub mean_state: f64,
    pub antenna_phase: f64,
    pub final_valid: bool,
    pub filters: Vec<FilterDecision>,
}

#[derive(Clone, Debug)]
pub struct CycleOutput {
    pub report: CycleReport,
    pub nodes: Vec<Node>,
    pub svg: String,
}

pub fn run_cycle(config: &EngineConfig) -> Result<CycleOutput> {
    config.validate()?;

    // Seed: stable indices mapped directly onto the existing 13 Metatron centres.
    let mut nodes = seeded_nodes();
    let mut rng = XorShift64::new(config.random_seed);

    // Spiral: deterministic dynamics with no node deletion.
    lateral_inhibition(&mut nodes);
    stochastic_resonance_with_rng(&mut nodes, config, &mut rng);
    let edge = regulate_edge_of_chaos(&mut nodes, &mut rng);
    let phase = apply_aperture(&mut nodes, config.aperture_open);

    // Octave: reuse the exact sanctum proof and the reservoir's shared sampler.
    let proof = MetatronSanctumProof::verify_exact();
    let swirl = PlanViewInterferenceSwirl::unveil_symbolic(config.swirl_turns)
        .map_err(SubmemError::InvalidArgument)?;

    // Return: validate the final mutated state and the exact geometric invariants.
    let mean_state = nodes.iter().map(|n| n.state).sum::<f64>() / nodes.len() as f64;
    let final_valid = nodes.len() == NODE_COUNT
        && proof.centres_count == NODE_COUNT
        && proof.chords_count == 78
        && proof.total_field_nodes == 97
        && proof.total_field_connections == 4_656
        && coverage_complete(&nodes)
        && nodes.iter().all(|n| {
            n.state.is_finite()
                && (0.0..=1.0).contains(&n.state)
                && n.history.last() == Some(&n.state)
        });
    let filters = [0.95, 0.60, 0.10, 0.50]
        .iter()
        .map(|candidate| via_negativa(&nodes, *candidate))
        .collect();

    let report = CycleReport {
        stages: vec!["Seed", "Spiral", "Octave", "Return"],
        node_count: nodes.len(),
        metatron_chords: proof.chords_count,
        metatron_field_nodes: proof.total_field_nodes,
        metatron_connections: proof.total_field_connections,
        raw_interior_crossings: proof.raw_interior_crossings,
        lit: swirl.constructive_lit_nodes,
        unlit: swirl.destructive_unlit_nodes,
        swirl_balance: swirl.yin_yang_balance_ratio,
        swirl_phase: swirl.holographic_swirl_phase_rad,
        lambda_before: edge.before,
        lambda_after: edge.after,
        final_activity: activity_ratio(&nodes),
        mean_state,
        antenna_phase: phase,
        final_valid,
        filters,
    };
    let svg = render_svg(&nodes, &proof, &swirl, &report);
    Ok(CycleOutput { report, nodes, svg })
}

pub fn render_svg(
    nodes: &[Node],
    proof: &MetatronSanctumProof,
    swirl: &PlanViewInterferenceSwirl,
    report: &CycleReport,
) -> String {
    let mut svg = String::with_capacity(500_000);
    writeln!(svg, r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 700" role="img" aria-labelledby="title desc">"##).unwrap();
    writeln!(svg, r##"  <title id="title">Mycelium symbolic engine</title>"##).unwrap();
    writeln!(svg, r##"  <desc id="desc">Thirteen Metatron centres with seventy-eight chords and the shared submem interference swirl.</desc>"##).unwrap();
    writeln!(svg, r##"  <rect width="1200" height="700" fill="#FDF5E2"/>"##).unwrap();
    writeln!(svg, r##"  <style>text{{font-family:Georgia,serif;fill:#2C1660}} .label{{font-size:14px}} .small{{font-size:12px}} </style>"##).unwrap();
    writeln!(svg, r##"  <text x="55" y="48" font-size="28">Mycelium · Seed → Spiral → Octave → Return</text>"##).unwrap();
    writeln!(svg, r##"  <text class="label" x="55" y="76">Exact proof · {} centres · {} chords · {} field nodes · {} connections</text>"##, proof.centres_count, proof.chords_count, proof.total_field_nodes, proof.total_field_connections).unwrap();

    // The renderer draws the same 13 centres used by the exact rational proof.
    for i in 0..METATRON_13_CIRCLES.len() {
        for j in (i + 1)..METATRON_13_CIRCLES.len() {
            let (ax, ay) = METATRON_13_CIRCLES[i].cartesian_xy();
            let (bx, by) = METATRON_13_CIRCLES[j].cartesian_xy();
            let (x1, y1) = (290.0 + ax * 105.0, 330.0 - ay * 105.0);
            let (x2, y2) = (290.0 + bx * 105.0, 330.0 - by * 105.0);
            writeln!(svg, r##"  <line class="metatron-edge" x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="#0b1a2e" stroke-width="0.55" opacity="0.42"/>"##).unwrap();
        }
    }
    for (i, circle) in METATRON_13_CIRCLES.iter().enumerate() {
        let node = &nodes[i];
        let (x, y) = circle.cartesian_xy();
        let cx = 290.0 + x * 105.0;
        let cy = 330.0 - y * 105.0;
        let radius = 6.0 + 5.0 * node.state;
        let opacity = 0.45 + 0.55 * node.state;
        writeln!(svg, r##"  <circle class="mycelium-node" cx="{cx:.2}" cy="{cy:.2}" r="{radius:.2}" fill="{}" fill-opacity="{opacity:.3}" stroke="#2C1660" stroke-width="1"><title>{} · {} · {:?} · state {:.6}</title></circle>"##, node.chakra.colour(), circle.code, node.role.name(), node.chakra, node.state).unwrap();
        writeln!(svg, r##"  <text x="{cx:.2}" y="{:.2}" font-size="10" text-anchor="middle">{}</text>"##, cy + 3.5, node.id).unwrap();
    }

    // Share the reservoir's intensity grid; no second swirl implementation.
    writeln!(svg, r##"  <text class="label" x="650" y="76">Swirl · {} lit / {} unlit · balance {:.3}</text>"##, swirl.constructive_lit_nodes, swirl.destructive_unlit_nodes, swirl.yin_yang_balance_ratio).unwrap();
    for (ring, row) in swirl.intensity_grid.iter().enumerate() {
        let r_norm = (ring as f64 + 0.5) / swirl.intensity_grid.len() as f64;
        for (splitter, intensity) in row.iter().enumerate() {
            let theta = splitter as f64 * 2.0 * PI / row.len() as f64;
            let cx = 875.0 + r_norm * theta.cos() * 245.0;
            let cy = 345.0 + r_norm * theta.sin() * 245.0;
            let colour = if *intensity >= 0.5 { "#2C1660" } else { "#C9B86A" };
            writeln!(svg, r##"  <circle class="swirl-cell" cx="{cx:.3}" cy="{cy:.3}" r="1.75" fill="{colour}" opacity="0.82"/>"##).unwrap();
        }
    }
    writeln!(svg, r##"  <circle cx="875" cy="345" r="250" fill="none" stroke="#2C1660" stroke-width="1" opacity="0.45"/>"##).unwrap();

    writeln!(svg, r##"  <text class="label" x="55" y="565">Return · final state valid: {} · mean state {:.6} · activity {:.6}</text>"##, report.final_valid, report.mean_state, report.final_activity).unwrap();
    writeln!(svg, r##"  <text class="small" x="55" y="590">λ before/after regulation: {:.6} / {:.6} · symbolic phase: {:.6} · raw crossings: {}</text>"##, report.lambda_before, report.lambda_after, report.antenna_phase, proof.raw_interior_crossings).unwrap();
    for (index, decision) in report.filters.iter().enumerate() {
        let outcome = if decision.accepted { "accepted" } else { "rejected" };
        let reason = if decision.reasons.is_empty() {
            "all configured rules passed".to_string()
        } else {
            decision.reasons.join("; ")
        };
        writeln!(svg, r##"  <text class="small" x="55" y="{}">candidate {:.2}: {} · {}</text>"##, 616 + index * 17, decision.candidate, outcome, reason).unwrap();
    }
    writeln!(svg, r##"  <text class="small" x="650" y="665">Integrated with submem · deterministic symbolic simulation; no security or metaphysical claims.</text>"##).unwrap();
    writeln!(svg, "</svg>").unwrap();
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    #[test]
    fn exact_sanctum_proof_drives_cycle_counts() {
        let output = run_cycle(&EngineConfig::default()).unwrap();
        assert_eq!(output.report.node_count, 13);
        assert_eq!(output.report.metatron_chords, 78);
        assert_eq!(output.report.metatron_field_nodes, 97);
        assert_eq!(output.report.metatron_connections, 4_656);
        assert_eq!(output.report.raw_interior_crossings, 294);
        assert!(output.report.final_valid);
    }

    #[test]
    fn shared_swirl_reference_case_is_balanced() {
        let field = PlanViewInterferenceSwirl::unveil_symbolic(100.0).unwrap();
        assert_eq!(field.constructive_lit_nodes, 2048);
        assert_eq!(field.destructive_unlit_nodes, 2048);
        close(field.yin_yang_balance_ratio, 1.0);
        close(field.holographic_swirl_phase_rad, 3.318_640_924_126_013_3);
    }

    #[test]
    fn thirteen_nodes_cover_every_category() {
        let nodes = seeded_nodes();
        assert_eq!(nodes.len(), 13);
        assert!(coverage_complete(&nodes));
        assert!(nodes.iter().enumerate().all(|(i, n)| n.id == i));
    }

    #[test]
    fn via_negativa_matches_reference_decisions() {
        let nodes = seeded_nodes();
        assert!(via_negativa(&nodes, 0.95).accepted);
        assert!(via_negativa(&nodes, 0.60).accepted);
        assert!(!via_negativa(&nodes, 0.10).accepted);
        assert!(!via_negativa(&nodes, 0.50).accepted);
    }

    #[test]
    fn via_negativa_fails_closed_on_bad_input_or_roles() {
        let nodes = seeded_nodes();
        for candidate in [f64::NAN, f64::INFINITY, -0.01, 1.01] {
            assert!(!via_negativa(&nodes, candidate).accepted);
        }
        assert!(!via_negativa(&[], 0.60).accepted);

        let mut one_refuser = seeded_nodes();
        for (index, node) in one_refuser.iter_mut().enumerate() {
            node.role = match index {
                0 => Role::Refuser,
                1 => Role::Silencer,
                _ => Role::Doubter,
            };
        }
        assert!(!via_negativa(&one_refuser, 0.95).accepted);
    }

    #[test]
    fn inhibition_preserves_stable_metatron_indices() {
        let mut nodes = seeded_nodes();
        lateral_inhibition(&mut nodes);
        assert_eq!(nodes.len(), 13);
        assert!(nodes.iter().enumerate().all(|(i, n)| n.id == i));
    }

    #[test]
    fn cycle_is_reproducible_and_validates_final_state() {
        let first = run_cycle(&EngineConfig::default()).unwrap();
        let second = run_cycle(&EngineConfig::default()).unwrap();
        assert_eq!(first.report, second.report);
        assert_eq!(first.nodes, second.nodes);
        assert_eq!(first.svg, second.svg);
        assert!(first.report.final_valid);
        assert_eq!(first.report.stages, ["Seed", "Spiral", "Octave", "Return"]);
        assert!(first.nodes.iter().all(|n| n.history.last() == Some(&n.state)));
    }

    #[test]
    fn closed_aperture_is_explicit_and_valid() {
        let config = EngineConfig {
            aperture_open: false,
            ..EngineConfig::default()
        };
        let output = run_cycle(&config).unwrap();
        close(output.report.antenna_phase, 0.0);
        assert!(output.report.final_valid);
    }

    #[test]
    fn renderer_contains_every_chord_node_and_swirl_cell() {
        let output = run_cycle(&EngineConfig::default()).unwrap();
        assert_eq!(output.svg.matches(r##"class="metatron-edge""##).count(), 78);
        assert_eq!(output.svg.matches(r##"class="mycelium-node""##).count(), 13);
        assert_eq!(output.svg.matches(r##"class="swirl-cell""##).count(), 4096);
    }

    #[test]
    fn invalid_configuration_and_swirl_phase_are_rejected() {
        let config = EngineConfig {
            swirl_turns: f64::NAN,
            ..EngineConfig::default()
        };
        assert!(run_cycle(&config).is_err());
        assert!(PlanViewInterferenceSwirl::unveil_symbolic(f64::INFINITY).is_err());
    }
}
