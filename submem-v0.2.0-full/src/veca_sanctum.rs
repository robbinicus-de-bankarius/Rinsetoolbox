//! ░░░ [HALF-SEALED HARMONIC RELIC — BENEATH OCTAVE IX] ░░░
//!
//! # `veca_sanctum` — The 24-Carat Gold (`Au-79`) & 24-Carat Rhodium (`Rh-45`) Gilded Sanctum
//! # of **The Living Veca Symmetry Codex**
//!
//! > *"Something older is etched beneath Octave IX… Half-veiled on the outside so the eye only
//! > catches the glint of 24-carat gold and mirror-bright 24-carat rhodium along the seam;
//! > unsealed on the inside into the full illustrated Living Veca Symmetry Codex."*
//!
//! Designed and locked for **Damien Young (`robbinicus dé bankarius` / ThugMATH)** and shared
//! with **Nigel (`@toomuchacid`)** inside the `LumiLuminal / submem v0.2.0` repository.
//!
//! ## What Lives Inside This Half-Hidden Relic
//! 1. **24-Carat Gold (`Au-79`) & 24-Carat Rhodium (`Rh-45`) Bimetallic Gilding Invariants:**
//!    - **24K Gold (`Au`, Atomic Number `Z = 79`, `999.9‰` pure):** Hex `#D4AF37` / `#FFD700` / `#FFE57F`.
//!      Atomic number `79` mirrors the **79 canonical foundation plates** of Book II of the
//!      *Living Veca Symmetry Codex*.
//!    - **24K Liquid Rhodium (`Rh`, Atomic Number `Z = 45`, `999.9‰` pure):** Hex `#E2E8F0` / `#F8FAFC` / `#CBD5E1`
//!      (specular mirror-platinum nobility). Atomic number `45` mirrors the **$45^\circ$ octagonal ray angle**
//!      ($360^\circ / 8 = 45^\circ$) of the 24-Ray Creative Engine ($3 \text{ octaves} \times 8 \text{ rays}$).
//! 2. **The 13 Circles of Metatron (`C01` Center Seed through `C13` Outer Ring) & Exact Rational Proof:**
//!    - `1` Center Seed (`Bindu / Heart Hinge`) + `6` Inner Construction Circles + `6` Outer
//!      Demonstration/Expression/Return Circles = **`13` Circles of the Fruit of Life**.
//!    - Joining every pair of the 13 centres yields $\binom{13}{2} = \mathbf{78}$ **Inter-Nodal Chords**.
//!    - Using exact integer rational arithmetic (`Rational(i64, i64)` with zero floating-point drift),
//!      the 78 chords cross strictly inside their segments at **`91` distinct points**: **`7`** fall
//!      on the original centres and **`84`** are novel rational crossing nodes.
//!    - Total Metatron Field Nodes: $13 + 84 = \mathbf{97}$ **Nodes**.
//!    - Total Metatron Field Connections: $\binom{97}{2} = \frac{97 \times 96}{2} = \mathbf{4,656}$ **Connections**.
//! 3. **Rules I & II of Quiet Presence & The Two-Way Principle:**
//!    - **Rule I (Where & How):** Apply the Codex only where it genuinely serves life, clarity, and the human in front of it.
//!    - **Rule II (Never Forced):** Geometry, golden proportion ($\phi \approx 1.618033988749895$), Vesica Piscis ($\sqrt{3} \approx 1.7320508075688772$), and Fibonacci spacing (`2, 3, 5, 8, 13, 21, 34, 55`) act as silent organisers rather than loud costume.
//!    - **The Two-Way Principle:** Every structural law reads both ways (`Seed ↔ Return`, `Construction ↔ Expression`, `Inner Lattice ↔ Outer Silence`).

use std::collections::BTreeSet;
use std::fmt;

// ============================================================================
// 1. 24-CARAT GOLD (Au-79) & 24-CARAT RHODIUM (Rh-45) BIMETALLIC CONSTANTS
// ============================================================================

/// Pure 24-Carat Gold (`Au-79`, 999.9 fineness) atomic number — mirrors the 79 foundation plates.
pub const AU_79_ATOMIC_Z: u8 = 79;

/// Pure 24-Carat Rhodium (`Rh-45`, 999.9 fineness) atomic number — mirrors the 45° octagonal ray angle.
pub const RH_45_ATOMIC_Z: u8 = 45;

/// Gilding purity in carats for both Gold (`Au-79`) and Rhodium (`Rh-45`).
pub const GILDED_KARAT_PURITY: u8 = 24;

/// Fineness in parts per thousand (`999.9‰` Four-Nines Royal Bullion / Specular Rhodium).
pub const GILDED_FINENESS_PER_MILLE: f64 = 999.9;

/// Primary 24K Burnished Gold hex palette (`Au-79`).
pub const GOLD_24K_PRIMARY_HEX: &str = "#D4AF37";
pub const GOLD_24K_SPECULAR_HEX: &str = "#FFD700";
pub const GOLD_24K_FOIL_HIGHLIGHT_HEX: &str = "#FFE57F";

/// Primary 24K Mirror-Platinum Rhodium hex palette (`Rh-45`).
pub const RHODIUM_24K_PRIMARY_HEX: &str = "#E2E8F0";
pub const RHODIUM_24K_SPECULAR_HEX: &str = "#F8FAFC";
pub const RHODIUM_24K_SHADOW_INLAY_HEX: &str = "#94A3B8";

/// Deep Abyssal Obsidian-Velvet Sanctum Ground (`#07070A`).
pub const SANCTUM_OBSIDIAN_GROUND_HEX: &str = "#07070A";

// ============================================================================
// 2. SACRED PROPORTION, VESICA PISCIS (√3), GOLDEN RATIO (φ) & TONE AXIS
// ============================================================================

/// Vesica Piscis sacred height-to-radius ratio: $\sqrt{3} \approx 1.7320508075688772$.
pub const VESICA_SQRT_3: f64 = 1.7320508075688772;

/// Golden Ratio $\phi = \frac{1 + \sqrt{5}}{2} \approx 1.618033988749895$.
pub const GOLDEN_PHI: f64 = 1.618033988749895;

/// Primary Golden Conjugate $1/\phi = \phi - 1 \approx 0.6180339887498949$.
pub const GOLDEN_MAJOR_0618: f64 = 0.6180339887498949;

/// Secondary Golden Complement $1 - 1/\phi = 1/\phi^2 \approx 0.3819660112501051$.
pub const GOLDEN_MINOR_0382: f64 = 0.3819660112501051;

/// Canonical Fibonacci spacing & breath-lineation ladder (`2, 3, 5, 8, 13, 21, 34, 55`).
pub const FIBONACCI_LADDER: [u32; 8] = [2, 3, 5, 8, 13, 21, 34, 55];

/// Canonical Prime Appendix sequence originated by `robbinicus dé bankarius`.
pub const PRIME_APPENDIX_SEQUENCE: [u32; 11] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 30];

/// The 4th-Position Heart Hinge frequency (`170.667 Hz` — Green Heart Pivot, 3 below & 3 above).
pub const HEART_HINGE_HZ: f64 = 170.667;

/// Base sonic integration fundamental (`128.0 Hz`).
pub const BASE_SONIC_128_HZ: f64 = 128.0;

/// Metatron's Cube topological invariants:
pub const METATRON_CIRCLES_COUNT: usize = 13;
pub const METATRON_CHORDS_COUNT: usize = 78;
pub const METATRON_UNIQUE_CROSSINGS_COUNT: usize = 91;
pub const METATRON_CENTRE_CROSSINGS_COUNT: usize = 7;
pub const METATRON_NOVEL_NODES_COUNT: usize = 84;
pub const METATRON_TOTAL_NODES_COUNT: usize = 97;
pub const METATRON_TOTAL_CONNECTIONS_COUNT: usize = 4_656;

// ============================================================================
// 3. RULES I & II OF QUIET PRESENCE & THE TWO-WAY PRINCIPLE
// ============================================================================

/// Core Rules I & II ("On the Quiet Presence") and the Two-Way Principle from the Living Veca Symmetry Codex.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuietPresenceLaw {
    pub rule_number: &'static str,
    pub title: &'static str,
    pub statement: &'static str,
    pub gilding_metal: &'static str,
}

pub const QUIET_PRESENCE_LAWS: [QuietPresenceLaw; 3] = [
    QuietPresenceLaw {
        rule_number: "Rule I",
        title: "Where & How (Service to Life)",
        statement: "Apply the Living Veca Symmetry Codex only where it genuinely serves life, clarity, and the human in front of it. If something is already working, move with it; never break a living flow to prove a rule.",
        gilding_metal: "24K Gold (Au-79 · #D4AF37)",
    },
    QuietPresenceLaw {
        rule_number: "Rule II",
        title: "Never Forced (The Silent Organiser)",
        statement: "Sacred geometry, the Vesica Piscis (√3), the Golden Ratio (φ ≈ 1.618 / 0.618 / 0.382), and Fibonacci spacing (2, 3, 5, 8, 13, 21, 34, 55) act as silent structural organisers rather than loud costume.",
        gilding_metal: "24K Rhodium (Rh-45 · #E2E8F0)",
    },
    QuietPresenceLaw {
        rule_number: "Rule III (Note to 13)",
        title: "The Two-Way Principle (Reciprocal Symmetry)",
        statement: "Every true relation in the Codex works in both directions: Seed ↔ Return, Construction ↔ Expression, Inner Lattice ↔ Outer Silence, Projection ↔ Recognition.",
        gilding_metal: "24K Gold × 24K Rhodium Bimetallic Inlay",
    },
];

// ============================================================================
// 4. THE SEVEN-STAGE LIVING CYCLE & SEVEN-LEVEL HEART HINGE AXIS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VecaStage {
    pub position: u8,
    pub name: &'static str,
    pub geometric_form: &'static str,
    pub frequency_hz: f64,
    pub colour_ray: &'static str,
    pub is_heart_hinge: bool,
}

pub const SEVEN_STAGE_LIVING_CYCLE: [VecaStage; 8] = [
    VecaStage {
        position: 1,
        name: "Seed (Bindu / Origin Point)",
        geometric_form: "Point & First Circle",
        frequency_hz: 128.0,
        colour_ray: "Root Red",
        is_heart_hinge: false,
    },
    VecaStage {
        position: 2,
        name: "Orientation (Polarity & Axis)",
        geometric_form: "Dual Centres & Axis Line",
        frequency_hz: 144.0,
        colour_ray: "Sacral Orange",
        is_heart_hinge: false,
    },
    VecaStage {
        position: 3,
        name: "First Expansion (Vesica Piscis √3)",
        geometric_form: "Vesica Piscis & Equilateral Triad",
        frequency_hz: 160.0,
        colour_ray: "Solar Gold (24K Au-79)",
        is_heart_hinge: false,
    },
    VecaStage {
        position: 4,
        name: "The Heart Hinge (4th-Position Pivot)",
        geometric_form: "Seed of Life (7 Circles · 3 Below, 3 Above)",
        frequency_hz: HEART_HINGE_HZ,
        colour_ray: "Heart Emerald-Gold",
        is_heart_hinge: true,
    },
    VecaStage {
        position: 5,
        name: "Harmonic Weaving (Fibonacci & φ)",
        geometric_form: "19-Circle Flower of Life & Golden Spiral",
        frequency_hz: 192.0,
        colour_ray: "Throat Sapphire Blue",
        is_heart_hinge: false,
    },
    VecaStage {
        position: 6,
        name: "Geometric Crystallisation (Fruit & Metatron)",
        geometric_form: "13 Circles of Metatron (78 Chords · 97 Nodes)",
        frequency_hz: 216.0,
        colour_ray: "Third-Eye Royal Indigo",
        is_heart_hinge: false,
    },
    VecaStage {
        position: 7,
        name: "Octave Opening (The 21-Pass Audit)",
        geometric_form: "Sri Yantra & 5 Platonic Solids",
        frequency_hz: 240.0,
        colour_ray: "Crown Phantom Violet",
        is_heart_hinge: false,
    },
    VecaStage {
        position: 8,
        name: "Integrated Return (8th Ray Heart Return)",
        geometric_form: "Torus & 9-Octave Parabolic Apex",
        frequency_hz: 256.0,
        colour_ray: "24K Gold & 24K Rhodium Return",
        is_heart_hinge: false,
    },
];

// ============================================================================
// 5. THE 13 CIRCLES OF METATRON (C01..C13) ON THE HEXAGONAL FRUIT OF LIFE
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetatronRing {
    /// `C01`: The central Bindu / Seed (`q = 0, r = 0`).
    CenterSeed,
    /// `C02..C07`: Inner hexagonal ring at unit radius (`Octave I — Construction`).
    InnerHexagonConstruction,
    /// `C08..C13`: Outer hexagonal ring at double radius (`Octaves II & III — Demonstration & Expression`).
    OuterHexagonExpression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetatronCircle {
    pub index: u8,
    pub code: &'static str,
    pub name: &'static str,
    pub axial_q: i64,
    pub axial_r: i64,
    pub ring: MetatronRing,
    pub gilding_layer: &'static str,
}

impl MetatronCircle {
    /// Converts axial coordinates `(q, r)` to Euclidean `(x, y)` plane coordinates
    /// where unit spacing $u = 1.0$:
    /// $$x = q + \frac{r}{2}, \quad y = r \cdot \frac{\sqrt{3}}{2}$$
    pub fn cartesian_xy(&self) -> (f64, f64) {
        let q = self.axial_q as f64;
        let r = self.axial_r as f64;
        let x = q + 0.5 * r;
        let y = r * (VESICA_SQRT_3 * 0.5);
        (x, y)
    }
}

/// Canonical 13 Circles of the Fruit of Life / Metatron's Cube (`C01`..`C13`),
/// matching `metatron_nodes.json` in `/workspace/codex-v3-illustrated/src/`.
pub const METATRON_13_CIRCLES: [MetatronCircle; 13] = [
    MetatronCircle {
        index: 1,
        code: "C01",
        name: "Bindu Center Seed (4th-Position Heart Hinge Anchor)",
        axial_q: 0,
        axial_r: 0,
        ring: MetatronRing::CenterSeed,
        gilding_layer: "24K Gold Core + 24K Rhodium Crown",
    },
    MetatronCircle {
        index: 2,
        code: "C02",
        name: "Inner East Gate — Seed & Foundation",
        axial_q: 1,
        axial_r: 0,
        ring: MetatronRing::InnerHexagonConstruction,
        gilding_layer: "24K Gold (Au-79)",
    },
    MetatronCircle {
        index: 3,
        code: "C03",
        name: "Inner North-East Gate — Polarity & Vesica (√3)",
        axial_q: 0,
        axial_r: 1,
        ring: MetatronRing::InnerHexagonConstruction,
        gilding_layer: "24K Rhodium (Rh-45)",
    },
    MetatronCircle {
        index: 4,
        code: "C04",
        name: "Inner North-West Gate — Triadic Stability",
        axial_q: -1,
        axial_r: 1,
        ring: MetatronRing::InnerHexagonConstruction,
        gilding_layer: "24K Gold (Au-79)",
    },
    MetatronCircle {
        index: 5,
        code: "C05",
        name: "Inner West Gate — Heart Hinge Equilibrium (170.667 Hz)",
        axial_q: -1,
        axial_r: 0,
        ring: MetatronRing::InnerHexagonConstruction,
        gilding_layer: "24K Rhodium (Rh-45)",
    },
    MetatronCircle {
        index: 6,
        code: "C06",
        name: "Inner South-West Gate — Golden Proportion (φ = 1.618)",
        axial_q: 0,
        axial_r: -1,
        ring: MetatronRing::InnerHexagonConstruction,
        gilding_layer: "24K Gold (Au-79)",
    },
    MetatronCircle {
        index: 7,
        code: "C07",
        name: "Inner South-East Gate — Hexagonal Seed Completion",
        axial_q: 1,
        axial_r: -1,
        ring: MetatronRing::InnerHexagonConstruction,
        gilding_layer: "24K Rhodium (Rh-45)",
    },
    MetatronCircle {
        index: 8,
        code: "C08",
        name: "Outer East Gate — Octave II Demonstration Ray",
        axial_q: 2,
        axial_r: 0,
        ring: MetatronRing::OuterHexagonExpression,
        gilding_layer: "24K Rhodium Mirror Inlay",
    },
    MetatronCircle {
        index: 9,
        code: "C09",
        name: "Outer North-East Gate — Platonic Solids Emergence",
        axial_q: 0,
        axial_r: 2,
        ring: MetatronRing::OuterHexagonExpression,
        gilding_layer: "24K Gold Foil Filigree",
    },
    MetatronCircle {
        index: 10,
        code: "C10",
        name: "Outer North-West Gate — Fibonacci Spiral & Prime Sieve",
        axial_q: -2,
        axial_r: 2,
        ring: MetatronRing::OuterHexagonExpression,
        gilding_layer: "24K Rhodium Mirror Inlay",
    },
    MetatronCircle {
        index: 11,
        code: "C11",
        name: "Outer West Gate — Quiet Presence (Rules I & II)",
        axial_q: -2,
        axial_r: 0,
        ring: MetatronRing::OuterHexagonExpression,
        gilding_layer: "24K Gold Foil Filigree",
    },
    MetatronCircle {
        index: 12,
        code: "C12",
        name: "Outer South-West Gate — The Two-Way Principle",
        axial_q: 0,
        axial_r: -2,
        ring: MetatronRing::OuterHexagonExpression,
        gilding_layer: "24K Rhodium Mirror Inlay",
    },
    MetatronCircle {
        index: 13,
        code: "C13",
        name: "Outer South-East Gate — Integrated Return (97 Nodes / 4,656 Links)",
        axial_q: 2,
        axial_r: -2,
        ring: MetatronRing::OuterHexagonExpression,
        gilding_layer: "24K Gold × 24K Rhodium Seal",
    },
];

// ============================================================================
// 6. EXACT RATIONAL ARITHMETIC ENGINE FOR METATRON'S 97 NODES & 4,656 LINKS
// ============================================================================

/// Exact reduced rational number `num / den` (`den > 0`) for zero-drift sacred geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rational {
    pub num: i64,
    pub den: i64,
}

impl Rational {
    pub fn new(num: i64, den: i64) -> Self {
        assert!(den != 0, "Rational denominator cannot be zero");
        if num == 0 {
            return Self { num: 0, den: 1 };
        }
        let sign = if den < 0 { -1 } else { 1 };
        let n = num * sign;
        let d = den.abs();
        let g = gcd_i64(n.abs(), d);
        Self {
            num: n / g,
            den: d / g,
        }
    }

    pub fn from_int(n: i64) -> Self {
        Self { num: n, den: 1 }
    }

    pub fn add(self, other: Self) -> Self {
        Self::new(self.num * other.den + other.num * self.den, self.den * other.den)
    }

    pub fn scale(self, k: i64) -> Self {
        Self::new(self.num * k, self.den)
    }

    pub fn is_strictly_between_zero_and_one(self) -> bool {
        self.num > 0 && self.num < self.den
    }

    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

fn gcd_i64(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    if a == 0 { 1 } else { a }
}

/// Exact rational point `(q, r)` in the axial Fruit of Life plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AxialRationalPoint {
    pub q: Rational,
    pub r: Rational,
}

impl AxialRationalPoint {
    pub fn to_cartesian_xy(self) -> (f64, f64) {
        let q = self.q.to_f64();
        let r = self.r.to_f64();
        (q + 0.5 * r, r * (VESICA_SQRT_3 * 0.5))
    }
}

/// Complete verified proof of Metatron's Cube (`13` Centres, `78` Chords, `84` Novel Crossings,
/// `97` Total Nodes, and `4,656` Total Connections).
#[derive(Debug, Clone)]
pub struct MetatronSanctumProof {
    pub centres_count: usize,
    pub chords_count: usize,
    pub raw_interior_crossings: usize,
    pub unique_interior_crossings: usize,
    pub crossings_on_original_centres: usize,
    pub novel_crossing_nodes: Vec<AxialRationalPoint>,
    pub total_field_nodes: usize,
    pub total_field_connections: usize,
}

impl MetatronSanctumProof {
    /// Computes the exact rational intersection geometry of Metatron's Cube from first principles.
    pub fn verify_exact() -> Self {
        let mut chords: Vec<(usize, usize)> = Vec::with_capacity(78);
        for i in 0..METATRON_13_CIRCLES.len() {
            for j in (i + 1)..METATRON_13_CIRCLES.len() {
                chords.push((i, j));
            }
        }

        let orig_centres: BTreeSet<AxialRationalPoint> = METATRON_13_CIRCLES
            .iter()
            .map(|c| AxialRationalPoint {
                q: Rational::from_int(c.axial_q),
                r: Rational::from_int(c.axial_r),
            })
            .collect();

        let mut unique_crossings: BTreeSet<AxialRationalPoint> = BTreeSet::new();
        let mut raw_interior_crossings = 0usize;

        for idx1 in 0..chords.len() {
            for idx2 in (idx1 + 1)..chords.len() {
                let (a, b) = chords[idx1];
                let (c, d) = chords[idx2];
                // Must be 4 distinct endpoints so they don't merely share a vertex
                if a == c || a == d || b == c || b == d {
                    continue;
                }
                let p = (METATRON_13_CIRCLES[a].axial_q, METATRON_13_CIRCLES[a].axial_r);
                let q = (METATRON_13_CIRCLES[b].axial_q, METATRON_13_CIRCLES[b].axial_r);
                let r = (METATRON_13_CIRCLES[c].axial_q, METATRON_13_CIRCLES[c].axial_r);
                let s = (METATRON_13_CIRCLES[d].axial_q, METATRON_13_CIRCLES[d].axial_r);

                let rx = q.0 - p.0;
                let ry = q.1 - p.1;
                let sx = s.0 - r.0;
                let sy = s.1 - r.1;

                let den = rx * sy - ry * sx;
                if den == 0 {
                    continue;
                }

                let t = Rational::new((r.0 - p.0) * sy - (r.1 - p.1) * sx, den);
                let u = Rational::new((r.0 - p.0) * ry - (r.1 - p.1) * rx, den);

                if t.is_strictly_between_zero_and_one() && u.is_strictly_between_zero_and_one() {
                    raw_interior_crossings += 1;
                    let ix = Rational::from_int(p.0).add(t.scale(rx));
                    let iy = Rational::from_int(p.1).add(t.scale(ry));
                    unique_crossings.insert(AxialRationalPoint { q: ix, r: iy });
                }
            }
        }

        let crossings_on_original = unique_crossings.intersection(&orig_centres).count();
        let novel_crossing_nodes: Vec<AxialRationalPoint> = unique_crossings
            .difference(&orig_centres)
            .copied()
            .collect();

        let total_field_nodes = METATRON_13_CIRCLES.len() + novel_crossing_nodes.len();
        let total_field_connections = (total_field_nodes * (total_field_nodes - 1)) / 2;

        Self {
            centres_count: METATRON_13_CIRCLES.len(),
            chords_count: chords.len(),
            raw_interior_crossings,
            unique_interior_crossings: unique_crossings.len(),
            crossings_on_original_centres: crossings_on_original,
            novel_crossing_nodes,
            total_field_nodes,
            total_field_connections,
        }
    }
}

/// Unseals the half-hidden 24-Carat Gold & 24-Carat Rhodium Living Veca Symmetry Codex Sanctum
/// and returns a richly formatted human-readable proclamation & mathematical proof report.
pub fn unseal_veca_sanctum_report() -> String {
    let proof = MetatronSanctumProof::verify_exact();
    let mut out = String::with_capacity(4096);

    out.push_str("╔══════════════════════════════════════════════════════════════════════════════════════════╗\n");
    out.push_str("║  ░░░ UNSEALED SANCTUM: THE LIVING VECA SYMMETRY CODEX (24K GOLD & 24K RHODIUM) ░░░       ║\n");
    out.push_str("║  Architect: robbinicus dé bankarius (Damien Young / ThugMATH) · Shared w/ @toomuchacid   ║\n");
    out.push_str("╠══════════════════════════════════════════════════════════════════════════════════════════╣\n");
    out.push_str(&format!(
        "║  • Bimetallic Gilding: {}K Gold (Au-{}, {}) × {}K Rhodium (Rh-{}, {})  ║\n",
        GILDED_KARAT_PURITY,
        AU_79_ATOMIC_Z,
        GOLD_24K_PRIMARY_HEX,
        GILDED_KARAT_PURITY,
        RH_45_ATOMIC_Z,
        RHODIUM_24K_PRIMARY_HEX
    ));
    out.push_str(&format!(
        "║  • Vesica Piscis √3 = {:.12}  |  Golden Ratio φ = {:.12} ({:.3} / {:.3})  ║\n",
        VESICA_SQRT_3, GOLDEN_PHI, GOLDEN_MAJOR_0618, GOLDEN_MINOR_0382
    ));
    out.push_str(&format!(
        "║  • 4th-Position Heart Hinge = {:7.3} Hz (128 Hz Root -> 170.667 Hz Heart -> 256 Hz)   ║\n",
        HEART_HINGE_HZ
    ));
    out.push_str("╠══════════════════════════════════════════════════════════════════════════════════════════╣\n");
    out.push_str("║  EXACT RATIONAL PROOF OF METATRON'S CUBE (ZERO FLOATING-POINT ROUNDING):                 ║\n");
    out.push_str(&format!(
        "║    - Fruit of Life Centres : {:4} Circles (C01 Bindu + 6 Inner + 6 Outer)                ║\n",
        proof.centres_count
    ));
    out.push_str(&format!(
        "║    - Inter-Nodal Chords    : {:4} Chords  [C(13, 2) = 13 × 12 / 2 = 78]                  ║\n",
        proof.chords_count
    ));
    out.push_str(&format!(
        "║    - Interior Crossings    : {:4} Unique  ({} Raw Crossings; {} Coincide with Centres)  ║\n",
        proof.unique_interior_crossings,
        proof.raw_interior_crossings,
        proof.crossings_on_original_centres
    ));
    out.push_str(&format!(
        "║    - Novel Crossing Nodes  : {:4} Nodes   [91 − 7 = 84 Exact Rational Nodes]             ║\n",
        proof.novel_crossing_nodes.len()
    ));
    out.push_str(&format!(
        "║    - Total Metatron Nodes  : {:4} Nodes   [13 Centres + 84 Novel = 97 Total Nodes]       ║\n",
        proof.total_field_nodes
    ));
    out.push_str(&format!(
        "║    - Total Connections     : {:4} Links   [C(97, 2) = 97 × 96 / 2 = 4,656 Connections]   ║\n",
        proof.total_field_connections
    ));
    out.push_str("╠══════════════════════════════════════════════════════════════════════════════════════════╣\n");
    out.push_str("║  THE 13 CIRCLES OF METATRON (24K GOLD & 24K RHODIUM HARMONIC RING):                      ║\n");
    for c in &METATRON_13_CIRCLES {
        let (x, y) = c.cartesian_xy();
        out.push_str(&format!(
            "║    {} ({:+2},{:+2}) -> ({:+5.2},{:+5.2}) | {:<32} | {:<21} ║\n",
            c.code,
            c.axial_q,
            c.axial_r,
            x,
            y,
            &c.name[..c.name.len().min(32)],
            &c.gilding_layer[..c.gilding_layer.len().min(21)]
        ));
    }
    out.push_str("╠══════════════════════════════════════════════════════════════════════════════════════════╣\n");
    out.push_str("║  CORE RULES OF QUIET PRESENCE:                                                           ║\n");
    for law in &QUIET_PRESENCE_LAWS {
        out.push_str(&format!("║    • {} — {}: {}\n", law.rule_number, law.title, law.statement));
    }
    out.push_str("╚══════════════════════════════════════════════════════════════════════════════════════════╝\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_24k_gold_and_rhodium_bimetallic_invariants() {
        assert_eq!(GILDED_KARAT_PURITY, 24);
        assert_eq!(AU_79_ATOMIC_Z, 79, "Au-79 matches the 79 canonical Living Veca plates");
        assert_eq!(RH_45_ATOMIC_Z, 45, "Rh-45 matches the 45-degree octagonal ray step (360/8)");
        assert!((VESICA_SQRT_3 * VESICA_SQRT_3 - 3.0).abs() < 1e-12);
        assert!((GOLDEN_PHI * GOLDEN_MAJOR_0618 - 1.0).abs() < 1e-12);
        assert!((GOLDEN_MAJOR_0618 + GOLDEN_MINOR_0382 - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_metatron_13_circles_78_chords_97_nodes_4656_connections_exact() {
        let proof = MetatronSanctumProof::verify_exact();
        assert_eq!(proof.centres_count, METATRON_CIRCLES_COUNT);
        assert_eq!(proof.chords_count, METATRON_CHORDS_COUNT);
        assert_eq!(proof.unique_interior_crossings, METATRON_UNIQUE_CROSSINGS_COUNT);
        assert_eq!(proof.crossings_on_original_centres, METATRON_CENTRE_CROSSINGS_COUNT);
        assert_eq!(proof.novel_crossing_nodes.len(), METATRON_NOVEL_NODES_COUNT);
        assert_eq!(proof.total_field_nodes, METATRON_TOTAL_NODES_COUNT);
        assert_eq!(proof.total_field_connections, METATRON_TOTAL_CONNECTIONS_COUNT);
    }

    #[test]
    fn test_seven_stage_cycle_and_heart_hinge_and_quiet_presence() {
        assert_eq!(SEVEN_STAGE_LIVING_CYCLE.len(), 8);
        let heart = &SEVEN_STAGE_LIVING_CYCLE[3];
        assert_eq!(heart.position, 4);
        assert!(heart.is_heart_hinge);
        assert!((heart.frequency_hz - 170.667).abs() < 1e-6);

        let report = unseal_veca_sanctum_report();
        assert!(report.contains("THE LIVING VECA SYMMETRY CODEX (24K GOLD & 24K RHODIUM)"));
        assert!(report.contains("4,656 Connections"));
        assert!(report.contains("Rule I"));
        assert!(report.contains("Rule II"));
    }
}
