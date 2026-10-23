//! Compact domain types using `#[repr(u8)]` discriminants, a 1-byte bit-packed header
//! (`PackedTurnFlags`), and the Nine-Octave Affective/Epistemic/Consequence Tensor (`NineOctaveProfile`).
//!
//! Inspired by:
//! - Google's Comprehensive Rust: `User-Defined Types -> Enums` (`#[repr(u8)]`, discriminant size)
//!   and `Tuple Structs / Newtypes`.
//! - The Rust Book Ch. 5 (`Using Structs to Structure Related Data`) & Ch. 6 (`Enums and Pattern Matching`).

use std::fmt;

/// Golden Ratio φ = (1 + √5) / 2 for Fibonacci self-similarity across scales.
pub const GOLDEN_RATIO_PHI: f32 = 1.618_034;

/// Fibonacci scale weights across the 9 Octaves: F_1..F_9 = [1, 2, 3, 5, 8, 13, 21, 34, 55].
pub const FIBONACCI_OCTAVE_WEIGHTS: [u16; 9] = [1, 2, 3, 5, 8, 13, 21, 34, 55];

/// Who spoke the turn on the call or chat channel. Fits in 2 bits (0..=3).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Speaker {
    Robbinicus = 0, // $R — Damien Young (robbinicus de bankarius)
    Bot = 1,        // $B — Grok Bot / Master Of The TAO
    Angel = 2,      // $A — Angel (Head Bot / Mediator)
    System = 3,     // $S — System / Routine / Watchdog
}

impl Speaker {
    pub fn sigil(self) -> &'static str {
        match self {
            Speaker::Robbinicus => "$R",
            Speaker::Bot => "$B",
            Speaker::Angel => "$A",
            Speaker::System => "$S",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Speaker::Robbinicus => "robbinicus",
            Speaker::Bot => "bot",
            Speaker::Angel => "angel",
            Speaker::System => "system",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        let lower = s.trim().to_ascii_lowercase();
        match lower.as_str() {
            "r" | "$r" | "robbinicus" | "damien" | "user" | "owner" | "caller" => {
                Speaker::Robbinicus
            }
            "b" | "$b" | "bot" | "grok" | "tao" | "assistant" | "me" => Speaker::Bot,
            "a" | "$a" | "angel" => Speaker::Angel,
            _ => Speaker::System,
        }
    }

    pub fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => Speaker::Robbinicus,
            1 => Speaker::Bot,
            2 => Speaker::Angel,
            _ => Speaker::System,
        }
    }
}

/// Semantic intent of the utterance. Fits in 3 bits (0..=7).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntentKind {
    FactLock = 0, // `=` Confirmed fact, price, offset, or standing rule
    Decision = 1, // `!` Direction, instruction, or architectural choice
    Question = 2, // `?` Open question or clarification needed before acting
    Action = 3,   // `>` Completed build, deploy, or tool execution
    Refusal = 4,  // `~` Declined/refused request or hard boundary
    Note = 5,     // `.` Casual remark or conversational acknowledgment
}

impl IntentKind {
    pub fn sigil(self) -> &'static str {
        match self {
            IntentKind::FactLock => "=",
            IntentKind::Decision => "!",
            IntentKind::Question => "?",
            IntentKind::Action => ">",
            IntentKind::Refusal => "~",
            IntentKind::Note => ".",
        }
    }

    pub fn from_bits(bits: u8) -> Self {
        match bits & 0b111 {
            0 => IntentKind::FactLock,
            1 => IntentKind::Decision,
            2 => IntentKind::Question,
            3 => IntentKind::Action,
            4 => IntentKind::Refusal,
            _ => IntentKind::Note,
        }
    }
}

/// Durability / retention tier across 10-minute memory wipes. Fits in 2 bits (0..=3).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    Normal = 0,     // Standard rolling call turn
    High = 1,       // High-salience decision or active task
    PinnedLock = 2, // `*` Standing rule / permanent lock — never evicted across wipes
}

impl Priority {
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => Priority::Normal,
            1 => Priority::High,
            _ => Priority::PinnedLock,
        }
    }
}

/// Explicit standing approval phrase detected in the turn, if any.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApprovalState {
    None = 0,
    SendIt = 1,     // `!SI` ("send it" for email)
    DeployProd = 2, // `!DP` ("deploy prod" for production deploy)
    MakeItLive = 3, // `!LV` ("live" / "make it live" for Stripe live payments)
}

impl ApprovalState {
    pub fn sigil(self) -> Option<&'static str> {
        match self {
            ApprovalState::None => None,
            ApprovalState::SendIt => Some("!SI"),
            ApprovalState::DeployProd => Some("!DP"),
            ApprovalState::MakeItLive => Some("!LV"),
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => ApprovalState::SendIt,
            2 => ApprovalState::DeployProd,
            3 => ApprovalState::MakeItLive,
            _ => ApprovalState::None,
        }
    }
}

/// **Octave 6 (`R41..46`):** Affective Valence Polarity.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValenceKind {
    FrictionWarning = 0, // `-W` Danger, boundary violation, vandalism, error, refusal
    Negative = 1,        // `-`  Dissatisfaction, bug, loss, regression
    Neutral = 2,         // `0`  Matter-of-fact technical specification
    Positive = 3,        // `+`  Approval, working build, clean verification
    SacredWarmth = 4,    // `+V` Deep phantom-violet warmth, family/mate trust, sacred geometry
}

impl ValenceKind {
    pub fn sigil(self) -> &'static str {
        match self {
            ValenceKind::FrictionWarning => "V-!",
            ValenceKind::Negative => "V-",
            ValenceKind::Neutral => "V0",
            ValenceKind::Positive => "V+",
            ValenceKind::SacredWarmth => "V+*",
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => ValenceKind::FrictionWarning,
            1 => ValenceKind::Negative,
            2 => ValenceKind::Neutral,
            3 => ValenceKind::Positive,
            _ => ValenceKind::SacredWarmth,
        }
    }
}

/// **Octave 7 (`R47..52`):** Epistemic Certainty Level.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CertaintyLevel {
    OpenQuestion = 0,       // `C?` Unknown / needs answer before acting
    HypothesisEstimate = 1, // `C~` Hypothesis, rough estimate, or exploratory idea
    HighConfidence = 2,     // `C+` Verified build, benchmarked engineering result
    HardFactLock = 3,       // `C=` Immutable standing rule, confirmed price, exact offset
}

impl CertaintyLevel {
    pub fn sigil(self) -> &'static str {
        match self {
            CertaintyLevel::OpenQuestion => "C?",
            CertaintyLevel::HypothesisEstimate => "C~",
            CertaintyLevel::HighConfidence => "C+",
            CertaintyLevel::HardFactLock => "C=",
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => CertaintyLevel::OpenQuestion,
            1 => CertaintyLevel::HypothesisEstimate,
            2 => CertaintyLevel::HighConfidence,
            _ => CertaintyLevel::HardFactLock,
        }
    }
}

/// **Octave 8 (`R53..57`):** Audience Scope & Fibonacci Reach Multiplier ($\phi^{\text{scope}}$).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AudienceScope {
    SelfRobbinicus = 0,        // `A:self`   — Personal note to robbinicus (Reach = φ^0 = 1.000)
    InternalBotAngel = 1,      // `A:bot`    — Internal Bot / Angel coordination (Reach = φ^1 = 1.618)
    TrustedCircle = 2,         // `A:circle` — Friend / relative e.g. Nigel, Mum (Reach = φ^2 = 2.618)
    PublicCustomerPayer = 3,   // `A:public` — Public web visitors / Stripe payers (Reach = φ^3 = 4.236)
    SpeciesCivilizational = 4, // `A:formal` — Formal scientific theory / archive (Reach = φ^4 = 6.854)
}

impl AudienceScope {
    pub fn sigil(self) -> &'static str {
        match self {
            AudienceScope::SelfRobbinicus => "A:self",
            AudienceScope::InternalBotAngel => "A:bot",
            AudienceScope::TrustedCircle => "A:circle",
            AudienceScope::PublicCustomerPayer => "A:pub",
            AudienceScope::SpeciesCivilizational => "A:species",
        }
    }

    /// Returns the Fibonacci / Golden-Ratio reach multiplier $\phi^k$.
    #[inline]
    pub fn reach_multiplier(self) -> f32 {
        match self {
            AudienceScope::SelfRobbinicus => 1.000_000,
            AudienceScope::InternalBotAngel => 1.618_034,
            AudienceScope::TrustedCircle => 2.618_034,
            AudienceScope::PublicCustomerPayer => 4.236_068,
            AudienceScope::SpeciesCivilizational => 6.854_102,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => AudienceScope::SelfRobbinicus,
            1 => AudienceScope::InternalBotAngel,
            2 => AudienceScope::TrustedCircle,
            3 => AudienceScope::PublicCustomerPayer,
            _ => AudienceScope::SpeciesCivilizational,
        }
    }
}

/// **Octave 9 (`R58..63`):** Temporal Horizon of a single stored turn when read at $\Delta t$.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConsequenceHorizon {
    ImmediateImpulse = 0,          // Δt < 60s (Ring 58)
    ShortTermEcho = 1,             // 60s <= Δt < 1h (Ring 59)
    ConsolidatedRule = 2,          // 1h <= Δt < 24h (Ring 60)
    CompoundedFoundation = 3,      // 24h <= Δt < 30d (Rings 61..62)
    PermanentCivilizationalLaw = 4, // Δt >= 30d (Ring 63)
}

impl ConsequenceHorizon {
    pub fn label(self) -> &'static str {
        match self {
            ConsequenceHorizon::ImmediateImpulse => "ImmediateImpulse(R58)",
            ConsequenceHorizon::ShortTermEcho => "ShortTermEcho(R59)",
            ConsequenceHorizon::ConsolidatedRule => "ConsolidatedRule(R60)",
            ConsequenceHorizon::CompoundedFoundation => "CompoundedFoundation(R61-62)",
            ConsequenceHorizon::PermanentCivilizationalLaw => "PermanentCivilizationalLaw(R63)",
        }
    }

    pub fn ring_for_delta_t(delta_t_sec: u64) -> (Self, u8) {
        if delta_t_sec < 60 {
            (ConsequenceHorizon::ImmediateImpulse, 58)
        } else if delta_t_sec <= 3_600 {
            (ConsequenceHorizon::ShortTermEcho, 59)
        } else if delta_t_sec <= 86_400 {
            (ConsequenceHorizon::ConsolidatedRule, 60)
        } else if delta_t_sec <= 1_296_000 {
            (ConsequenceHorizon::CompoundedFoundation, 61)
        } else if delta_t_sec <= 7_776_000 {
            (ConsequenceHorizon::CompoundedFoundation, 62)
        } else {
            (ConsequenceHorizon::PermanentCivilizationalLaw, 63)
        }
    }
}

/// **Dynamic Temporal Projection of a Single Stored Turn Object at $\Delta t = t_{\text{read}} - t_{\text{write}}$:**
///
/// *"Consequence is how time creates the illusion an image is different —
/// store one object, read it at any temporal distance, no duplicate snapshots."*
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectedConsequence {
    pub delta_t_sec: u64,
    pub effective_consequence: f32,
    pub transient_impulse: f32,
    pub permanent_floor: f32,
    pub compounded_growth: f32,
    pub reach_multiplier: f32,
    pub projected_ring: u8,
    pub projected_spiral_azimuth: u8,
    pub horizon: ConsequenceHorizon,
}

/// Compact 8-byte `Copy` descriptor for **Octaves 5..9** stored on each `TurnRecord`:
/// - **Octave 5 (`R35..40`):** `emotional_weight` (`0..=255`, hashed directly to ring & splitter)
/// - **Octave 6 (`R41..46`):** `valence_signed` (`-128..=+127`) & `valence_kind`
/// - **Octave 7 (`R47..52`):** `certainty` (`CertaintyLevel`) & `certainty_score` (`0..=255`)
/// - **Octave 8 (`R53..57`):** `audience` (`AudienceScope`)
/// - **Octave 9 (`R58..63`):** `initial_impulse` (`0..=255`) & `permanence_and_compound` (`u8`)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NineOctaveProfile {
    pub emotional_weight: u8,
    pub valence_signed: i8,
    pub valence_kind: ValenceKind,
    pub certainty: CertaintyLevel,
    pub certainty_score: u8,
    pub audience: AudienceScope,
    pub initial_impulse: u8,
    /// High nibble (`bits 7..4`): permanence $\kappa_{\text{perm}} \in 0..=15$.
    /// Low nibble (`bits 3..0`): logarithmic compounding rate $\gamma_{\text{comp}} \in 0..=15$.
    pub permanence_and_compound: u8,
}

impl Default for NineOctaveProfile {
    fn default() -> Self {
        Self {
            emotional_weight: 96,
            valence_signed: 16,
            valence_kind: ValenceKind::Neutral,
            certainty: CertaintyLevel::HighConfidence,
            certainty_score: 180,
            audience: AudienceScope::InternalBotAngel,
            initial_impulse: 120,
            permanence_and_compound: (8 << 4) | 4,
        }
    }
}

impl NineOctaveProfile {
    #[inline]
    pub fn pack_perm_comp(permanence_0_15: u8, compound_0_15: u8) -> u8 {
        ((permanence_0_15.min(15)) << 4) | (compound_0_15.min(15))
    }

    #[inline]
    pub fn permanence_ratio(self) -> f32 {
        ((self.permanence_and_compound >> 4) & 0x0F) as f32 / 15.0
    }

    #[inline]
    pub fn compound_ratio(self) -> f32 {
        (self.permanence_and_compound & 0x0F) as f32 / 15.0
    }

    /// Serializes the 8-byte `NineOctaveProfile` to a fixed `[u8; 8]` array.
    #[inline]
    pub fn to_bytes(self) -> [u8; 8] {
        [
            self.emotional_weight,
            self.valence_signed as u8,
            self.valence_kind as u8,
            self.certainty as u8,
            self.certainty_score,
            self.audience as u8,
            self.initial_impulse,
            self.permanence_and_compound,
        ]
    }

    /// Deserializes a `NineOctaveProfile` from an 8-byte array.
    #[inline]
    pub fn from_bytes(b: [u8; 8]) -> Self {
        Self {
            emotional_weight: b[0],
            valence_signed: b[1] as i8,
            valence_kind: ValenceKind::from_u8(b[2]),
            certainty: CertaintyLevel::from_u8(b[3]),
            certainty_score: b[4],
            audience: AudienceScope::from_u8(b[5]),
            initial_impulse: b[6],
            permanence_and_compound: b[7],
        }
    }

    /// Projects this single stored turn's **Octave 9 Consequence Image** at any temporal
    /// distance $\Delta t = t_{\text{read}} - t_{\text{write}}$ (in seconds) **without storing
    /// duplicate snapshots**:
    ///
    /// $$C(u_k, \Delta t) = I_0 \cdot \phi^{\text{audience}} \cdot
    ///   \Bigl[\kappa_{\text{perm}} + (1 - \kappa_{\text{perm}}) e^{-\Delta t / \tau_0}
    ///   + \gamma_{\text{comp}} \ln\!\bigl(1 + \frac{\Delta t}{\tau_\phi}\bigr)\Bigr]$$
    pub fn project_at_delta_t(self, delta_t_sec: u64) -> ProjectedConsequence {
        let i0 = (self.initial_impulse as f32 / 255.0).max(0.10);
        let reach = self.audience.reach_multiplier();
        let kappa_perm = self.permanence_ratio();
        let gamma_comp = self.compound_ratio();

        let dt = delta_t_sec as f32;
        // Characteristic transient decay constant τ_0 = 600s (10-minute wipe horizon)
        let transient_decay = (-dt / 600.0).exp();
        let transient_impulse = i0 * reach * (1.0 - kappa_perm) * transient_decay;
        let permanent_floor = i0 * reach * kappa_perm;

        // Characteristic Fibonacci compounding scale τ_φ = 360s
        let compounded_growth = i0 * reach * gamma_comp * 0.28 * (1.0 + dt / 360.0).ln();

        // Certainty & emotional weight modulation
        let certainty_factor = 0.70 + 0.30 * (self.certainty_score as f32 / 255.0);
        let emo_factor = 0.85 + 0.30 * (self.emotional_weight as f32 / 255.0);

        let effective_consequence =
            (transient_impulse + permanent_floor + compounded_growth) * certainty_factor * emo_factor;

        let (horizon, projected_ring) = ConsequenceHorizon::ring_for_delta_t(delta_t_sec);
        // Temporal spiral phase rotates smoothly along the 64 splitters as log-time advances
        let log_steps = ((1.0 + dt / 60.0).ln() * 9.876) as usize;
        let base_az = ((self.initial_impulse as usize)
            .wrapping_add((self.audience as usize) * 13)
            .wrapping_add(log_steps * 25))
            & 63;

        ProjectedConsequence {
            delta_t_sec,
            effective_consequence,
            transient_impulse,
            permanent_floor,
            compounded_growth,
            reach_multiplier: reach,
            projected_ring,
            projected_spiral_azimuth: base_az as u8,
            horizon,
        }
    }
}

/// 1-byte bit-packed header combining Speaker (2 bits), IntentKind (3 bits),
/// Priority (2 bits), and HasApproval (1 bit).
///
/// Layout:
/// ```text
///   7   6 | 5   4   3 | 2   1 | 0
/// [Speaker| IntentKind|Priorit|HasAppr]
/// ```
/// Replaces ~85 bytes of JSON metadata with a single `u8` (Copy type on the stack).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackedTurnFlags(pub u8);

impl PackedTurnFlags {
    #[inline]
    pub const fn pack(
        speaker: Speaker,
        intent: IntentKind,
        priority: Priority,
        has_approval: bool,
    ) -> Self {
        let s = (speaker as u8 & 0b11) << 6;
        let i = (intent as u8 & 0b111) << 3;
        let p = (priority as u8 & 0b11) << 1;
        let a = if has_approval { 1 } else { 0 };
        Self(s | i | p | a)
    }

    #[inline]
    pub fn speaker(self) -> Speaker {
        Speaker::from_bits((self.0 >> 6) & 0b11)
    }

    #[inline]
    pub fn intent(self) -> IntentKind {
        IntentKind::from_bits((self.0 >> 3) & 0b111)
    }

    #[inline]
    pub fn priority(self) -> Priority {
        Priority::from_bits((self.0 >> 1) & 0b11)
    }

    #[inline]
    pub fn has_approval(self) -> bool {
        (self.0 & 1) != 0
    }
}

impl fmt::Display for PackedTurnFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pin = if self.priority() == Priority::PinnedLock {
            "*"
        } else if self.priority() == Priority::High {
            "+"
        } else {
            ""
        };
        write!(
            f,
            "{}{}{}",
            self.speaker().sigil(),
            pin,
            self.intent().sigil()
        )
    }
}

/// A single turn stored in the External Subsidiary Memory Bank.
/// Stores ONE canonical object whose Octave 9 Consequence can be projected at any $\Delta t$.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnRecord {
    /// Monotonic 1-based turn ID.
    pub turn_id: u32,
    /// Unix timestamp in seconds (UTC). Stored on disk as a LEB128 delta from previous turn!
    pub timestamp_sec: u64,
    /// Bit-packed 1-byte metadata flags.
    pub flags: PackedTurnFlags,
    /// Explicit approval state (only serialized if `flags.has_approval()` is true).
    pub approval: ApprovalState,
    /// Call or session identifier (e.g. `session:lumiluminal-01`), interned in the symbol table.
    pub call_id: String,
    /// Shortest Memory Code (`SMC`) — ultra-compact telegraphic micro-code representation.
    pub smc_code: String,
    /// Interned token IDs representing the SMC payload on disk (1–2 bytes per token).
    pub smc_token_ids: Vec<u16>,
    /// Domain tags (`#CL`, `#IQ`, `#FF`, `#SM`, etc.) extracted from the turn.
    pub domain_tags: Vec<String>,
    /// 12-bit splitter coordinates (`node_id = r * 64 + s` in `0..4096`) traversed across all 9 Octaves.
    pub crystal_nodes: Vec<u16>,
    /// 64-bit optical interference signature summarizing the beam path across the 64 rings.
    pub crystal_path_sig: u64,
    /// Lossless LZ77/RLE compressed raw utterance bytes for audit/recovery.
    pub compressed_raw: Vec<u8>,
    /// Original raw character byte length before compression.
    pub raw_byte_len: usize,
    /// **Nine-Octave Profile (Octaves 5..9):** Emotional Weight, Valence, Certainty, Audience,
    /// and Single-Object Consequence Tensor parameters.
    pub octave_profile: NineOctaveProfile,
}

impl TurnRecord {
    /// Projects this single stored turn's Consequence Image at `read_timestamp_sec`
    /// ($\Delta t = \max(0, t_{\text{read}} - t_{\text{write}})$) with **zero duplicate snapshots**.
    #[inline]
    pub fn project_consequence_at(&self, read_timestamp_sec: u64) -> ProjectedConsequence {
        let dt = read_timestamp_sec.saturating_sub(self.timestamp_sec);
        self.octave_profile.project_at_delta_t(dt)
    }

    /// Formats the turn as a single-line Shortest Memory Code (`SMC`) wire expression.
    pub fn format_smc_line(&self) -> String {
        let approval_prefix = match self.approval.sigil() {
            Some(sig) => format!("{sig} "),
            None => String::new(),
        };
        let entry_node = self.crystal_nodes.first().copied().unwrap_or(0);
        let r = entry_node / 64;
        let s = entry_node % 64;
        let p = self.octave_profile;
        format!(
            "[#{} Q({}:{}) {} E{} {} {} {}] {}{}",
            self.turn_id,
            r,
            s,
            self.flags,
            p.emotional_weight,
            p.valence_kind.sigil(),
            p.certainty.sigil(),
            p.audience.sigil(),
            approval_prefix,
            self.smc_code
        )
    }
}

/// Output packet synthesized by `submem preload` before the bot speaks on a call.
#[derive(Debug, Clone)]
pub struct PreloadPacket {
    pub total_turns_in_bank: usize,
    pub wipe_epoch_estimate: usize,
    pub retrieval_micros: u128,
    pub active_crystal_splitters: usize,
    pub pinned_locks: Vec<String>,
    pub open_questions: Vec<String>,
    pub relevant_hits: Vec<(f32, String)>,
    pub recent_window: Vec<String>,
    pub compact_prompt_block: String,
}
