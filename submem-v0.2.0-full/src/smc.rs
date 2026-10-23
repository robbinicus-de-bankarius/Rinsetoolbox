//! Shortest Memory Code (`SMC`) Compressor, Zero-Copy Slice Tokenizer, and
//! Nine-Octave Affective/Epistemic/Consequence Analyzer.
//!
//! Grounded in:
//! - The Rust Book Ch. 4.1 & 4.3 (`What Is Ownership?` & `The Slice Type` `&str`, `&[u8]`)
//! - The Rust Book Ch. 10.3 (`Validating References with Lifetimes` `'a`)
//! - The Rust Book Ch. 13.2 (`Processing a Series of Items with Iterators`)

use crate::codebook::{is_stopword, stem_word, SymbolTable, DOMAIN_SIGILS, PHRASE_CODEBOOK};
use crate::types::{
    ApprovalState, AudienceScope, CertaintyLevel, IntentKind, NineOctaveProfile, Priority, Speaker,
    ValenceKind,
};

/// Zero-copy slice tokenizer holding a borrowed reference `&'a str` into the source utterance.
/// Demonstrates explicit lifetime annotations from The Rust Book Ch. 10.3 (`struct Tokenizer<'a>`).
pub struct SliceTokenizer<'a> {
    remaining: &'a str,
}

impl<'a> SliceTokenizer<'a> {
    #[inline]
    pub fn new(input: &'a str) -> Self {
        Self { remaining: input }
    }
}

impl<'a> Iterator for SliceTokenizer<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip non-token delimiters while preserving sigil prefixes ($, #, !, @, +)
        let is_token_char = |c: char| {
            c.is_alphanumeric() || matches!(c, '$' | '#' | '!' | '@' | '+' | '_' | '-' | '.')
        };

        let start = self.remaining.find(is_token_char)?;
        let tail = &self.remaining[start..];
        let end = tail.find(|c: char| !is_token_char(c)).unwrap_or(tail.len());
        let token = &tail[..end];
        self.remaining = &tail[end..];
        let trimmed = token.trim_matches(|c: char| c == '.' || c == '-');
        if trimmed.is_empty() {
            self.next()
        } else {
            Some(trimmed)
        }
    }
}

/// Redacts secrets, API keys, tokens, and password assignments before anything touches disk.
/// Enforces standing rule: "Never put passwords, codes or keys in chat or on calls."
pub fn redact_secrets(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for word in raw.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        let clean = word.trim_matches(|c: char| matches!(c, ',' | ';' | '"' | '\'' | '(' | ')'));
        let lower = clean.to_ascii_lowercase();
        if clean.starts_with("sk_live_")
            || clean.starts_with("sk_test_")
            || clean.starts_with("rk_live_")
            || clean.starts_with("ghp_")
            || clean.starts_with("github_pat_")
            || clean.starts_with("ntfp_")
            || clean.starts_with("nfp_")
            || clean.starts_with("AIza")
            || lower.starts_with("#reddev")
            || lower.starts_with("password=")
            || lower.starts_with("passwd=")
            || lower.starts_with("api_key=")
            || lower.starts_with("secret_key=")
        {
            out.push_str("[REDACTED_KEY]");
        } else {
            out.push_str(word);
        }
    }
    out
}

/// Classifies the utterance's semantic intent, priority tier, and explicit approval sigil.
pub fn classify_utterance(
    speaker: Speaker,
    raw_text: &str,
    force_pin: bool,
) -> (IntentKind, Priority, ApprovalState) {
    let lower = raw_text.to_ascii_lowercase();

    // Detect explicit standing approval words from Damien (robbinicus)
    let approval = if speaker == Speaker::Robbinicus {
        if lower.contains("deploy prod") {
            ApprovalState::DeployProd
        } else if lower.contains("make it live")
            || lower == "live"
            || lower.contains("stripe live")
        {
            ApprovalState::MakeItLive
        } else if lower.contains("send it") {
            ApprovalState::SendIt
        } else {
            ApprovalState::None
        }
    } else {
        ApprovalState::None
    };

    // Detect IntentKind
    let intent = if lower.contains("declined")
        || lower.contains("refuse")
        || lower.contains("never put passwords")
        || lower.contains("won't reopen")
        || lower.contains("stays untouched")
        || lower.contains("no rose quartz")
    {
        IntentKind::Refusal
    } else if raw_text.trim_end().ends_with('?')
        || lower.starts_with("can you")
        || lower.starts_with("are you")
        || lower.contains("wondering if")
        || lower.contains("asking plainly")
    {
        IntentKind::Question
    } else if lower.contains("standing rule")
        || lower.contains("always ")
        || lower.contains("never ")
        || lower.contains("confirmed")
        || lower.contains("offset is")
        || lower.contains("64 rings")
        || lower.contains("4,096")
        || lower.contains("4096")
        || lower.contains("phantom quartz")
        || lower.contains("every 10 minutes")
        || lower.contains("nine octaves")
    {
        IntentKind::FactLock
    } else if lower.contains("design and build")
        || lower.contains("study both")
        || lower.contains("requirements:")
        || lower.contains("please reply")
        || approval != ApprovalState::None
    {
        IntentKind::Decision
    } else if speaker == Speaker::Bot
        && (lower.contains("built")
            || lower.contains("deployed")
            || lower.contains("passed")
            || lower.contains("verified")
            || lower.contains("on it"))
    {
        IntentKind::Action
    } else {
        IntentKind::Note
    };

    // Determine retention Priority across 10-minute wipes
    let priority = if force_pin
        || intent == IntentKind::FactLock
        || intent == IntentKind::Refusal
        || approval != ApprovalState::None
    {
        Priority::PinnedLock
    } else if intent == IntentKind::Decision || intent == IntentKind::Question {
        Priority::High
    } else {
        Priority::Normal
    };

    (intent, priority, approval)
}

/// Computes the **Nine-Octave Profile (Octaves 5..9)** directly from the utterance:
/// - **Octave 5 (`R35..40`):** `emotional_weight` (`0..=255`)
/// - **Octave 6 (`R41..46`):** `valence_signed` (`-128..=+127`) & `valence_kind`
/// - **Octave 7 (`R47..52`):** `certainty` (`CertaintyLevel`) & `certainty_score` (`0..=255`)
/// - **Octave 8 (`R53..57`):** `audience` (`AudienceScope`)
/// - **Octave 9 (`R58..63`):** `initial_impulse` (`0..=255`) & `permanence_and_compound` (`u8`)
pub fn analyze_nine_octave_profile(
    speaker: Speaker,
    intent: IntentKind,
    priority: Priority,
    approval: ApprovalState,
    raw_text: &str,
) -> NineOctaveProfile {
    let lower = raw_text.to_ascii_lowercase();

    // --- Octave 5 (R35..40): Emotional Weight (0..=255) ---
    let mut emo: u16 = match priority {
        Priority::PinnedLock => 165,
        Priority::High => 125,
        Priority::Normal => 80,
    };
    let exclamations = raw_text.chars().filter(|&c| c == '!').count() as u16;
    emo = emo.saturating_add(exclamations * 18);

    for &(cue, boost) in &[
        ("urgent", 60u16),
        ("critical", 55),
        ("emergency", 65),
        ("vital", 45),
        ("never", 35),
        ("always", 30),
        ("standing rule", 40),
        ("love", 45),
        ("mum", 45),
        ("nigel", 40),
        ("sacred", 40),
        ("phantom-violet", 38),
        ("phantom quartz", 32),
        ("warning", 48),
        ("danger", 55),
        ("breach", 55),
        ("vandal", 50),
        ("destroy", 45),
        ("90%", 30),
        ("100%", 25),
    ] {
        if lower.contains(cue) {
            emo = emo.saturating_add(boost);
        }
    }
    let emotional_weight = emo.min(255) as u8;

    // --- Octave 6 (R41..46): Valence Polarity (-128..=+127 & ValenceKind) ---
    let has_warning = lower.contains("warning")
        || lower.contains("danger")
        || lower.contains("breach")
        || lower.contains("vandal")
        || lower.contains("broken")
        || lower.contains("corrupt")
        || lower.contains("never put passwords")
        || lower.contains("no rose quartz")
        || intent == IntentKind::Refusal;

    let has_negative = lower.contains("bug")
        || lower.contains("fail")
        || lower.contains("error")
        || lower.contains("wrong")
        || lower.contains("lost")
        || lower.contains("problem");

    let has_sacred_warmth = lower.contains("phantom-violet")
        || lower.contains("phantom quartz")
        || lower.contains("lab coat")
        || lower.contains("sacred")
        || lower.contains("metatron")
        || lower.contains("mum")
        || lower.contains("nigel")
        || lower.contains("friend")
        || lower.contains("family")
        || lower.contains("love")
        || lower.contains("happy");

    let has_positive = lower.contains("confirmed")
        || lower.contains("locked in")
        || lower.contains("passed")
        || lower.contains("verified")
        || lower.contains("clean")
        || lower.contains("smooth")
        || lower.contains("great")
        || approval != ApprovalState::None;

    let (valence_kind, valence_signed): (ValenceKind, i8) = if has_warning {
        (ValenceKind::FrictionWarning, -96)
    } else if has_negative {
        (ValenceKind::Negative, -52)
    } else if has_sacred_warmth {
        (ValenceKind::SacredWarmth, 118)
    } else if has_positive {
        (ValenceKind::Positive, 72)
    } else {
        (ValenceKind::Neutral, 12)
    };

    // --- Octave 7 (R47..52): Epistemic Certainty Level ---
    let (certainty, certainty_score): (CertaintyLevel, u8) = if intent == IntentKind::Question
        || lower.contains("unclear")
        || lower.contains("wondering")
    {
        (CertaintyLevel::OpenQuestion, 48)
    } else if lower.contains("maybe")
        || lower.contains("estimate")
        || lower.contains("hypothesis")
        || lower.contains("guess")
        || lower.contains("might")
    {
        (CertaintyLevel::HypothesisEstimate, 116)
    } else if priority == Priority::PinnedLock
        || intent == IntentKind::FactLock
        || intent == IntentKind::Refusal
        || lower.contains("confirmed")
        || lower.contains("standing rule")
        || lower.contains("offset is")
        || lower.contains("100%")
    {
        (CertaintyLevel::HardFactLock, 250)
    } else {
        (CertaintyLevel::HighConfidence, 196)
    };

    // --- Octave 8 (R53..57): Audience Scope & Fibonacci Reach ---
    let audience = if lower.contains("formal theory")
        || lower.contains("theory.md")
        || lower.contains("species")
        || lower.contains("civilization")
        || lower.contains("universal")
        || lower.contains("mathematical")
    {
        AudienceScope::SpeciesCivilizational
    } else if approval != ApprovalState::None
        || lower.contains("deploy prod")
        || lower.contains("make it live")
        || lower.contains("stripe")
        || lower.contains("customer")
        || lower.contains("public")
        || lower.contains("fake fighting fish")
        || lower.contains("federal aqua")
        || lower.contains("iq test")
    {
        AudienceScope::PublicCustomerPayer
    } else if lower.contains("nigel")
        || lower.contains("mum")
        || lower.contains("friend")
        || lower.contains("family")
        || lower.contains("mate")
    {
        AudienceScope::TrustedCircle
    } else if speaker == Speaker::Robbinicus && lower.contains("call me") {
        AudienceScope::SelfRobbinicus
    } else {
        AudienceScope::InternalBotAngel
    };

    // --- Octave 9 (R58..63): Consequence Tensor Parameters (Single-Object Temporal Projection) ---
    let base_impulse: u16 = (emotional_weight as u16 / 2)
        + (certainty_score as u16 / 3)
        + match audience {
            AudienceScope::SelfRobbinicus => 20,
            AudienceScope::InternalBotAngel => 35,
            AudienceScope::TrustedCircle => 55,
            AudienceScope::PublicCustomerPayer => 75,
            AudienceScope::SpeciesCivilizational => 95,
        };
    let initial_impulse = base_impulse.min(255) as u8;

    let permanence_0_15: u8 = if priority == Priority::PinnedLock
        || certainty == CertaintyLevel::HardFactLock
        || approval != ApprovalState::None
    {
        14
    } else if audience >= AudienceScope::PublicCustomerPayer {
        11
    } else if priority == Priority::High {
        8
    } else {
        3
    };

    let compound_0_15: u8 = match audience {
        AudienceScope::SpeciesCivilizational => 15,
        AudienceScope::PublicCustomerPayer => 12,
        AudienceScope::TrustedCircle => 9,
        AudienceScope::InternalBotAngel => {
            if priority == Priority::PinnedLock {
                11
            } else {
                5
            }
        }
        AudienceScope::SelfRobbinicus => {
            if priority == Priority::PinnedLock {
                10
            } else {
                3
            }
        }
    };

    NineOctaveProfile {
        emotional_weight,
        valence_signed,
        valence_kind,
        certainty,
        certainty_score,
        audience,
        initial_impulse,
        permanence_and_compound: NineOctaveProfile::pack_perm_comp(permanence_0_15, compound_0_15),
    }
}

/// Output of the `SMC` encoder for a single utterance.
#[derive(Debug, Clone)]
pub struct EncodedSmc {
    pub smc_code: String,
    pub token_ids: Vec<u16>,
    pub domain_tags: Vec<String>,
    pub compressed_raw: Vec<u8>,
}

/// Encodes a spoken/chat turn into Shortest Memory Code (`SMC`) + interned symbol IDs.
pub fn encode_to_smc(raw_text: &str, symtab: &mut SymbolTable) -> EncodedSmc {
    let safe_text = redact_secrets(raw_text);

    // Normalize whitespace and lowercase for phrase replacement while preserving key numbers/caps
    let mut normalized = safe_text.to_ascii_lowercase();

    // Replace punctuation that separates clauses with `;` so clause structure survives in SMC
    for (from, to) in [
        (" — ", "; "),
        (" -- ", "; "),
        (". ", "; "),
        (": ", ":"),
        (", ", " "),
    ] {
        normalized = normalized.replace(from, to);
    }

    // Apply phrase codebook substitutions (longest domain phrases -> compact sigils)
    for &(phrase, sigil) in PHRASE_CODEBOOK {
        if normalized.contains(phrase) {
            normalized = normalized.replace(phrase, sigil);
        }
    }

    // Collect domain tags present in the normalized string
    let mut domain_tags = Vec::new();
    for &(sigil, _) in DOMAIN_SIGILS {
        if normalized.contains(sigil) && !domain_tags.iter().any(|s| s == sigil) {
            domain_tags.push(sigil.to_string());
        }
    }

    // Use our zero-copy SliceTokenizer<'a> and Iterator adapters (Book Ch. 4.3, 10.3, 13.2)
    let mut clauses: Vec<String> = Vec::new();
    for raw_clause in normalized.split(';') {
        let mut clause_tokens: Vec<&str> = Vec::new();
        for tok in SliceTokenizer::new(raw_clause) {
            // Keep all sigils ($R, #SM, !DP, @10m, +3) and numbers
            let is_sigil = tok.starts_with('$')
                || tok.starts_with('#')
                || tok.starts_with('!')
                || tok.starts_with('@')
                || tok.starts_with('+');
            if !is_sigil && is_stopword(tok) {
                continue;
            }
            let stemmed = if is_sigil { tok } else { stem_word(tok) };
            if stemmed.is_empty() {
                continue;
            }
            // Deduplicate consecutive identical stems inside the same clause
            if clause_tokens.last().copied() != Some(stemmed) {
                clause_tokens.push(stemmed);
            }
        }
        if !clause_tokens.is_empty() {
            clauses.push(clause_tokens.join("/"));
        }
    }

    // Cap to the highest-density clauses so SMC code stays tight
    let smc_code = if clauses.is_empty() {
        "ack".to_string()
    } else {
        clauses.join(";")
    };

    // Intern every token in the SMC code into our u16 SymbolTable
    let mut token_ids = Vec::new();
    for tok in SliceTokenizer::new(&smc_code) {
        token_ids.push(symtab.intern(tok));
    }

    let compressed_raw = compress_lz_bytes(safe_text.as_bytes());

    EncodedSmc {
        smc_code,
        token_ids,
        domain_tags,
        compressed_raw,
    }
}

/// Reconstructs the token stream from interned `u16` symbol IDs using `SymbolTable`.
pub fn decode_tokens_from_symtab(token_ids: &[u16], symtab: &SymbolTable) -> String {
    token_ids
        .iter()
        .filter_map(|&id| symtab.resolve(id))
        .collect::<Vec<&str>>()
        .join("/")
}

/// Expands an `SMC` telegraphic code back into a human-readable English gloss.
pub fn expand_smc_human_gloss(smc_code: &str) -> String {
    let mut out = smc_code.to_string();
    let expansions: &[(&str, &str)] = &[
        ("$R", "robbinicus"),
        ("$B", "bot"),
        ("$A", "Angel"),
        ("#PQP", "PhantomQuartzPyramid"),
        ("#SM", "SubsidiaryMemoryBank"),
        ("#MR", "MemoryReservoir"),
        ("#CL", "CrystalLattice"),
        ("#JS", "JobSeeker"),
        ("#IQ", "AlphaIQ"),
        ("#FF", "FakeFish"),
        ("#FA", "FederalAqua"),
        ("#RT", "RinseToolbox"),
        ("#VC", "LivingVecaCodex"),
        ("#M13", "Metatron13Check"),
        ("#CRust", "ComprehensiveRust"),
        ("#RBook", "RustBook"),
        ("!DP", "[APPROVAL:deploy_prod]"),
        ("!SI", "[APPROVAL:send_it]"),
        ("!LV", "[APPROVAL:make_it_live]"),
        ("@10m", "every_10_min"),
        ("min_code", "shortest_code"),
        ("pre_speak", "before_speaking"),
        ("mem_wipe", "memory_wipe"),
    ];
    for &(sig, full) in expansions {
        out = out.replace(sig, full);
    }
    out.replace('/', " ").replace(';', " | ")
}

/// Fast lossless LZ77 sliding-window + literal-run byte compressor for raw transcript backup
/// and Holographic Spiral Wavepacket compression.
pub fn compress_lz_bytes(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() / 2 + 8);
    let mut pos = 0usize;
    let mut lit_start = 0usize;

    let flush_literals = |out: &mut Vec<u8>, slice: &[u8]| {
        for chunk in slice.chunks(128) {
            out.push((chunk.len() - 1) as u8);
            out.extend_from_slice(chunk);
        }
    };

    while pos < input.len() {
        let mut best_len = 0usize;
        let mut best_off = 0usize;
        let max_len = (input.len() - pos).min(34);
        if max_len >= 3 {
            let win_start = pos.saturating_sub(1024);
            for cand in (win_start..pos).rev() {
                let mut mlen = 0usize;
                while mlen < max_len && input[cand + mlen] == input[pos + mlen] {
                    mlen += 1;
                }
                if mlen >= 3 && mlen > best_len {
                    best_len = mlen;
                    best_off = pos - cand;
                    if best_len == max_len {
                        break;
                    }
                }
            }
        }

        if best_len >= 3 {
            if pos > lit_start {
                flush_literals(&mut out, &input[lit_start..pos]);
            }
            let off_minus_1 = best_off - 1;
            let len_minus_3 = best_len - 3;
            let tag = 0x80 | ((len_minus_3 as u8) << 2) | (((off_minus_1 >> 8) & 0x03) as u8);
            let low = (off_minus_1 & 0xFF) as u8;
            out.push(tag);
            out.push(low);
            pos += best_len;
            lit_start = pos;
        } else {
            pos += 1;
        }
    }

    if lit_start < input.len() {
        flush_literals(&mut out, &input[lit_start..]);
    }
    out
}

/// Losslessly decompresses bytes produced by `compress_lz_bytes`.
pub fn decompress_lz_bytes(compressed: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(compressed.len() * 2);
    let mut idx = 0usize;
    while idx < compressed.len() {
        let tag = compressed[idx];
        idx += 1;
        if (tag & 0x80) == 0 {
            let len = (tag as usize) + 1;
            let end = (idx + len).min(compressed.len());
            out.extend_from_slice(&compressed[idx..end]);
            idx = end;
        } else {
            if idx >= compressed.len() {
                break;
            }
            let low = compressed[idx] as usize;
            idx += 1;
            let len = (((tag & 0x7F) >> 2) as usize) + 3;
            let offset = ((((tag & 0x03) as usize) << 8) | low) + 1;
            if offset <= out.len() {
                let start = out.len() - offset;
                for i in 0..len {
                    let b = out[start + i];
                    out.push(b);
                }
            }
        }
    }
    out
}
