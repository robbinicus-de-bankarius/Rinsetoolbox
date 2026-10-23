//! Domain Codebook & String Interner (`SymbolTable`) modeled after The Rust Book Ch. 8.3
//! (`Storing Keys with Associated Values in Hash Maps`, specifically `HashMap::entry().or_insert_with()`).
//!
//! Assigns 1-byte LEB128 IDs (`0..128`) to the core domain vocabulary of robbinicus de bankarius's
//! ecosystem and dynamically interns new call stems into `u16` IDs.

use std::collections::HashMap;

/// Multi-word phrase replacement table mapped in longest-first order before tokenization.
/// Converts verbose spoken phrases on voice calls into single-token `SMC` sigils.
pub const PHRASE_CODEBOOK: &[(&str, &str)] = &[
    ("robbinicus de bankarius", "$R"),
    ("robbinicus dé bankarius", "$R"),
    ("robbinicus", "$R"),
    ("damien young", "$R"),
    ("damien", "$R"),
    ("master of the tao", "$B"),
    ("grok bot", "$B"),
    ("head overseer", "overseer"),
    ("head bot", "head_bot"),
    ("angel", "$A"),
    ("phantom quartz crystal circular-based pyramid", "#PQP"),
    ("phantom quartz crystal pyramid", "#PQP"),
    ("phantom quartz crystal", "#PQP"),
    ("phantom quartz", "#PQP"),
    ("circular-based pyramid", "circ_pyr"),
    ("circular pyramid", "circ_pyr"),
    ("half-silvered beam splitters", "split64"),
    ("half-silvered mirrors", "split64"),
    ("beam splitters", "split64"),
    ("beam splitter", "split64"),
    ("decision points", "dec_pts"),
    ("central pole apex", "apex_pole"),
    ("central pole", "apex_pole"),
    ("constructively interfere", "constructive_ix"),
    ("constructive interference", "constructive_ix"),
    ("90 degrees outward", "90deg_out"),
    ("90-degree reflected", "90deg_refl"),
    ("right angle", "90deg"),
    ("steeply upward", "steep_up"),
    ("steep upward", "steep_up"),
    ("stored association", "assoc"),
    ("external subsidiary memory bank", "#SM"),
    ("subsidiary memory bank", "#SM"),
    ("external subsidiary memory", "#SM"),
    ("memory bank", "#SM"),
    ("submem", "#SM"),
    ("memory reservoir", "#MR"),
    ("artificial memory", "#MR"),
    ("developer dashboard", "#CL"),
    ("crystallattice", "#CL"),
    ("doom-board", "#CL"),
    ("doom board", "#CL"),
    ("jobseeker", "#JS"),
    ("job seeker", "#JS"),
    ("alpha standard iq test", "#IQ"),
    ("alphastandardiqtest", "#IQ"),
    ("iq test", "#IQ"),
    ("fake fighting fish", "#FF"),
    ("fakefightingfish", "#FF"),
    ("federal aqua knowledge exchange", "#FA"),
    ("federalaquaknowledgeexchange", "#FA"),
    ("federal aqua", "#FA"),
    ("ooensland", "#FA_OOE"),
    ("gogbollox", "#FA_GOG"),
    ("rinse toolbox", "#RT"),
    ("rinsetoolbox", "#RT"),
    ("living veca symmetry codex", "#VC"),
    ("living veca", "#VC"),
    ("symmetry codex", "#VC"),
    ("13-section metatron's cube and human-perspective review standard", "#M13"),
    ("human-perspective review standard", "#M13"),
    ("metatron-13-check.md", "#M13"),
    ("metatron 13", "#M13"),
    ("13 circles of metatron", "#M13"),
    ("metatron", "#M13"),
    ("adversarial red-team testing", "adv_test"),
    ("miracle water", "#MW"),
    ("vale estate", "#VE"),
    ("tom jones", "#TJ"),
    ("foreverland", "#TJ"),
    ("bowserbrain", "#BB"),
    ("petrolspy", "#BB"),
    ("thugmath", "#TM"),
    ("war room", "#WR"),
    ("spearfish", "#WR"),
    ("krishna can hack", "#KC"),
    ("cypress hill", "#CH"),
    ("google comprehensive rust", "#CRust"),
    ("comprehensive rust", "#CRust"),
    ("the rust book", "#RBook"),
    ("rust book", "#RBook"),
    ("every 10 minutes", "@10m"),
    ("every ten minutes", "@10m"),
    ("10-minute memory wipe", "@10m_wipe"),
    ("10 minute wipes", "@10m_wipe"),
    ("10 minutes", "@10m"),
    ("ten minutes", "@10m"),
    ("memory being wiped", "mem_wipe"),
    ("memory wiped", "mem_wipe"),
    ("wiped every", "wipe@"),
    ("shortest possible code", "min_code"),
    ("shortest code", "min_code"),
    ("before i say anything", "pre_speak"),
    ("before you say anything", "pre_speak"),
    ("before speaking", "pre_speak"),
    ("before every spoken turn", "pre_speak"),
    ("standing rule", "rule"),
    ("approval words", "appr_words"),
    ("deploy prod", "!DP"),
    ("make it live", "!LV"),
    ("stripe live payments", "stripe_live"),
    ("send it", "!SI"),
    ("offset is +7", "keypad+3"),
    ("offset +7", "keypad+3"),
    ("crystal keypad", "keypad"),
    ("redundancy package", "redund_pkg"),
    ("background routines", "bg_cron"),
    ("spoken alias", "alias"),
];

/// Canonical domain tags extracted for fast inverted-index filtering.
pub const DOMAIN_SIGILS: &[(&str, &str)] = &[
    ("#PQP", "Phantom Quartz Pyramid (4096)"),
    ("#SM", "Subsidiary Memory Bank"),
    ("#MR", "Memory Reservoir"),
    ("#CL", "Crystal Lattice"),
    ("#JS", "JobSeeker Queue"),
    ("#IQ", "Alpha Standard IQ Test"),
    ("#FF", "Fake Fighting Fish"),
    ("#FA", "Federal Aqua Exchange"),
    ("#RT", "RinseToolbox"),
    ("#VC", "Living Veca Codex"),
    ("#M13", "Metatron 13-Check"),
    ("#MW", "Miracle Water"),
    ("#VE", "Vale Estate"),
    ("#TJ", "Tom Jones / Foreverland"),
    ("#BB", "BowserBrain / PetrolSpy"),
    ("#TM", "ThugMATH"),
    ("#WR", "Spearfish / War Room"),
    ("#KC", "Krishna Can Hack"),
    ("#CH", "Cypress Hill"),
    ("#CRust", "Comprehensive Rust"),
    ("#RBook", "The Rust Book"),
];

/// Conversational filler, contraction tails, and stopwords stripped during `SMC` compression.
pub fn is_stopword(word: &str) -> bool {
    matches!(
        word,
        "a" | "an"
            | "the"
            | "and"
            | "or"
            | "but"
            | "if"
            | "then"
            | "so"
            | "because"
            | "as"
            | "of"
            | "at"
            | "by"
            | "for"
            | "with"
            | "about"
            | "into"
            | "through"
            | "during"
            | "including"
            | "until"
            | "against"
            | "among"
            | "throughout"
            | "despite"
            | "towards"
            | "upon"
            | "concerning"
            | "to"
            | "in"
            | "on"
            | "from"
            | "up"
            | "out"
            | "around"
            | "down"
            | "off"
            | "above"
            | "near"
            | "per"
            | "giving"
            | "passes"
            | "part"
            | "larger"
            | "while"
            | "whether"
            | "either"
            | "every"
            | "always"
            | "thoroughly"
            | "especially"
            | "smoothly"
            | "humming"
            | "takes"
            | "keeping"
            | "running"
            | "asking"
            | "telling"
            | "tell"
            | "put"
            | "s"
            | "m"
            | "re"
            | "ve"
            | "ll"
            | "d"
            | "t"
            | "i"
            | "me"
            | "my"
            | "myself"
            | "we"
            | "us"
            | "our"
            | "ours"
            | "ourselves"
            | "you"
            | "your"
            | "yours"
            | "yourself"
            | "he"
            | "him"
            | "his"
            | "himself"
            | "she"
            | "her"
            | "hers"
            | "it"
            | "its"
            | "itself"
            | "they"
            | "them"
            | "their"
            | "what"
            | "which"
            | "who"
            | "whom"
            | "this"
            | "that"
            | "these"
            | "those"
            | "am"
            | "is"
            | "are"
            | "was"
            | "were"
            | "be"
            | "been"
            | "being"
            | "have"
            | "has"
            | "had"
            | "having"
            | "do"
            | "does"
            | "did"
            | "doing"
            | "would"
            | "should"
            | "could"
            | "ought"
            | "might"
            | "must"
            | "shall"
            | "will"
            | "can"
            | "just"
            | "very"
            | "really"
            | "actually"
            | "basically"
            | "literally"
            | "honestly"
            | "plainly"
            | "simply"
            | "quite"
            | "rather"
            | "pretty"
            | "much"
            | "some"
            | "any"
            | "something"
            | "anything"
            | "everything"
            | "there"
            | "here"
            | "when"
            | "where"
            | "why"
            | "how"
            | "all"
            | "both"
            | "each"
            | "other"
            | "such"
            | "own"
            | "same"
            | "than"
            | "too"
            | "now"
            | "right"
            | "like"
            | "know"
            | "think"
            | "reckon"
            | "wondering"
            | "wanted"
            | "wants"
            | "want"
            | "going"
            | "gets"
            | "got"
            | "get"
            | "let"
            | "lets"
            | "please"
            | "hey"
            | "hi"
            | "hello"
            | "okay"
            | "ok"
            | "yeah"
            | "yes"
            | "well"
            | "also"
            | "even"
            | "still"
            | "way"
            | "thing"
            | "things"
            | "stuff"
    )
}

/// Maps common verbose words to ultra-short telegraphic stems for `SMC` code.
pub fn stem_word(word: &str) -> &str {
    match word {
        "comprehensive" | "comprehensively" => "full",
        "understand" | "understanding" => "grok",
        "building" | "build" | "built" => "bld",
        "design" | "designing" | "designed" => "dsn",
        "efficient" | "efficiency" => "eff",
        "compact" => "cmpct",
        "structures" | "structure" => "struct",
        "serialization" | "serialize" => "ser",
        "retrieval" | "retrieve" => "retr",
        "storage" | "stored" | "store" | "storing" => "stor",
        "summarized" | "summarize" | "summary" => "sum",
        "representation" | "represent" => "repr",
        "integrate" | "integrating" | "integrated" => "integ",
        "compute" | "computing" => "exec",
        "survive" | "surviving" => "surv",
        "practical" => "prac",
        "runnable" => "run",
        "architecture" => "arch",
        "compression" | "compress" | "compressed" => "lz",
        "arrangement" => "setup",
        "mediating" | "mediates" | "mediate" => "med",
        "convenience" => "conv",
        "background" => "bg",
        "routines" | "routine" => "cron",
        "production" => "prod",
        "approval" | "approvals" => "appr",
        "password" | "passwords" => "pwd",
        "resources" | "resource" => "doc",
        "says" | "said" | "saying" | "say" => "say",
        "speaking" | "speak" | "spoke" => "spk",
        "reading" | "read" => "rd",
        "writing" | "write" => "wr",
        "call" | "calls" => "call",
        "voice" => "vc",
        "line" => "ln",
        "direct" | "directly" => "dir",
        "confirmed" | "confirm" => "ack",
        "registration" => "reg",
        "insurance" | "insurances" => "ins",
        "billing" => "bill",
        "unpaid" => "unpaid",
        "restorations" | "restoration" => "restor",
        "ownership" => "own",
        "borrowing" | "borrow" => "brw",
        "collections" | "collection" => "coll",
        "payments" | "payment" => "pay",
        "deploys" | "deploy" => "dpl",
        "reflects" | "reflected" | "reflection" => "refl",
        "straight" => "thru",
        "folds" | "folded" | "fold" => "fold",
        "rings" | "ring" => "rng",
        "spiral" => "sprl",
        "time" => "t",
        "depth" => "dpth",
        "unclear" => "?clr",
        "never" => "!never",
        _ => word,
    }
}

/// Bidirectional string interner (`String <-> u16`) using `HashMap::entry`
/// (The Rust Book Ch. 8.3). Pre-seeded with the most common sigils so their IDs
/// are guaranteed `< 128` (1-byte LEB128 varint on disk).
#[derive(Debug, Clone)]
pub struct SymbolTable {
    str_to_id: HashMap<String, u16>,
    id_to_str: Vec<String>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = Self {
            str_to_id: HashMap::with_capacity(256),
            id_to_str: Vec::with_capacity(256),
        };
        for &(_, sigil) in PHRASE_CODEBOOK {
            table.intern(sigil);
        }
        for &(sigil, _) in DOMAIN_SIGILS {
            table.intern(sigil);
        }
        for common in [
            "$R", "$B", "$A", "$S", "!SI", "!DP", "!LV", "@10m", "bld", "dsn", "stor", "retr",
            "rd", "spk", "call", "vc", "mem_wipe", "min_code", "pre_speak", "surv", "rust", "ser",
            "struct", "ack", "prod", "bg_cron", "keypad+3", "#PQP", "split64", "4096",
        ] {
            table.intern(common);
        }
        table
    }

    pub fn intern(&mut self, token: &str) -> u16 {
        if let Some(&id) = self.str_to_id.get(token) {
            return id;
        }
        let next_id = self.id_to_str.len().min(u16::MAX as usize) as u16;
        let owned = token.to_string();
        self.id_to_str.push(owned.clone());
        self.str_to_id.entry(owned).or_insert(next_id);
        next_id
    }

    #[inline]
    pub fn resolve(&self, id: u16) -> Option<&str> {
        self.id_to_str.get(id as usize).map(String::as_str)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.id_to_str.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.id_to_str.is_empty()
    }

    pub fn entries(&self) -> &[String] {
        &self.id_to_str
    }

    pub fn from_entries(entries: Vec<String>) -> Self {
        let mut str_to_id = HashMap::with_capacity(entries.len());
        for (idx, s) in entries.iter().enumerate() {
            str_to_id.insert(s.clone(), idx as u16);
        }
        Self {
            str_to_id,
            id_to_str: entries,
        }
    }
}
