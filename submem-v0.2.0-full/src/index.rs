//! Hybrid Nine-Octave Parabolic Phantom Quartz + Okapi BM25 Inverted Index for `< 1ms` Pre-Speech Retrieval.
//!
//! Includes head-to-head retrieval modes (`hybrid_retrieve_nine_octave`, `retrieve_four_octave_legacy`,
//! and `retrieve_flat_bm25_only`) for rigorous A/B benchmarking.

use std::collections::HashMap;

use crate::crystal::PhantomQuartzCrystal;
use crate::types::{CertaintyLevel, NineOctaveProfile, Priority, TurnRecord};

#[derive(Debug, Default, Clone)]
pub struct InvertedIndex {
    /// Maps interned `u16` token ID -> list of `(turn_id, term_frequency)`.
    postings: HashMap<u16, Vec<(u32, u16)>>,
    /// Maps domain sigil (`#SM`, `#CL`, `#IQ`, etc.) -> list of `turn_id`s.
    domain_postings: HashMap<String, Vec<u32>>,
    /// Maps `turn_id` -> token count (document length).
    doc_lengths: HashMap<u32, usize>,
    pub total_docs: usize,
    pub total_tokens: usize,
}

impl InvertedIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Indexes a single `TurnRecord` into the in-memory postings lists using `HashMap::entry`.
    pub fn add_turn(&mut self, rec: &TurnRecord) {
        self.total_docs += 1;
        let doc_len = rec.smc_token_ids.len().max(1);
        self.total_tokens += doc_len;
        self.doc_lengths.insert(rec.turn_id, doc_len);

        let mut tf_map: HashMap<u16, u16> = HashMap::new();
        for &tok_id in &rec.smc_token_ids {
            *tf_map.entry(tok_id).or_insert(0) += 1;
        }
        for (tok_id, tf) in tf_map {
            self.postings
                .entry(tok_id)
                .or_default()
                .push((rec.turn_id, tf));
        }

        for tag in &rec.domain_tags {
            let list = self.domain_postings.entry(tag.clone()).or_default();
            if list.last().copied() != Some(rec.turn_id) {
                list.push(rec.turn_id);
            }
        }
    }

    /// Mode A (Baseline): Pure Flat Lexical Okapi BM25 without Crystal or 9-Octave resonance.
    pub fn retrieve_flat_bm25_only(
        &self,
        records: &[TurnRecord],
        query_token_ids: &[u16],
        top_k: usize,
    ) -> Vec<(f32, u32, usize)> {
        if records.is_empty() {
            return Vec::new();
        }
        let avg_dl = (self.total_tokens as f32 / (self.total_docs.max(1) as f32)).max(1.0);
        let n_docs = self.total_docs as f32;
        let k1 = 1.5f32;
        let b = 0.75f32;

        let mut scores: HashMap<u32, f32> = HashMap::new();
        for &q_tok in query_token_ids {
            if let Some(post_list) = self.postings.get(&q_tok) {
                let df = post_list.len() as f32;
                let idf = ((n_docs - df + 0.5) / (df + 0.5) + 1.0).ln();
                for &(turn_id, tf) in post_list {
                    let dl = *self.doc_lengths.get(&turn_id).unwrap_or(&1) as f32;
                    let tf_f = tf as f32;
                    let bm25 =
                        idf * ((tf_f * (k1 + 1.0)) / (tf_f + k1 * (1.0 - b + b * (dl / avg_dl))));
                    *scores.entry(turn_id).or_insert(0.0) += bm25;
                }
            }
        }

        let mut ranked: Vec<(f32, u32, usize)> = scores
            .into_iter()
            .map(|(tid, sc)| (sc, tid, 0))
            .collect();
        ranked.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.1.cmp(&a.1))
        });
        ranked.truncate(top_k);
        ranked
    }

    /// Mode B (Legacy 4-Octave): BM25 + Domain Sigils + 4-Octave Linear-Cone Apex Convergence
    /// (ignoring Octaves 5..9 Emotional Weight, Valence, Certainty, Audience, and Temporal Consequence).
    pub fn retrieve_four_octave_legacy(
        &self,
        records: &[TurnRecord],
        query_token_ids: &[u16],
        query_domains: &[String],
        current_ts: u64,
        top_k: usize,
    ) -> Vec<(f32, u32, usize)> {
        if records.is_empty() {
            return Vec::new();
        }
        let avg_dl = (self.total_tokens as f32 / (self.total_docs.max(1) as f32)).max(1.0);
        let n_docs = self.total_docs as f32;
        let k1 = 1.5f32;
        let b = 0.75f32;

        let mut scores: HashMap<u32, (f32, usize)> = HashMap::new();
        for &q_tok in query_token_ids {
            if let Some(post_list) = self.postings.get(&q_tok) {
                let df = post_list.len() as f32;
                let idf = ((n_docs - df + 0.5) / (df + 0.5) + 1.0).ln();
                for &(turn_id, tf) in post_list {
                    let dl = *self.doc_lengths.get(&turn_id).unwrap_or(&1) as f32;
                    let tf_f = tf as f32;
                    let bm25 =
                        idf * ((tf_f * (k1 + 1.0)) / (tf_f + k1 * (1.0 - b + b * (dl / avg_dl))));
                    let entry = scores.entry(turn_id).or_insert((0.0, 0));
                    entry.0 += bm25;
                }
            }
        }

        for dom in query_domains {
            if let Some(turn_ids) = self.domain_postings.get(dom) {
                for &tid in turn_ids {
                    let entry = scores.entry(tid).or_insert((0.0, 0));
                    entry.0 += 2.50;
                }
            }
        }

        let mut ranked: Vec<(f32, u32, usize)> = scores
            .into_iter()
            .filter_map(|(turn_id, (base_score, beams))| {
                let rec = records.get((turn_id.saturating_sub(1)) as usize)?;
                let priority_mult = match rec.flags.priority() {
                    Priority::PinnedLock => 1.25,
                    Priority::High => 1.10,
                    Priority::Normal => 1.0,
                };
                let age_sec = current_ts.saturating_sub(rec.timestamp_sec) as f32;
                let recency_bonus = 1.0 / (1.0 + (age_sec / 7200.0));
                Some(((base_score + 0.30 * recency_bonus) * priority_mult, turn_id, beams))
            })
            .collect();

        ranked.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.1.cmp(&a.1))
        });
        ranked.truncate(top_k);
        ranked
    }

    /// Mode C (Full Nine-Octave Parabolic Phantom Quartz + Single-Object Consequence Projection):
    /// Combines Okapi BM25 + Domain Sigils + Parabolic Focus Constructive Interference across
    /// all 9 Octaves + Emotional Weight / Valence / Certainty / Audience / Consequence resonance.
    pub fn hybrid_retrieve_nine_octave(
        &self,
        crystal: &PhantomQuartzCrystal,
        records: &[TurnRecord],
        query_token_ids: &[u16],
        query_domains: &[String],
        query_profile: NineOctaveProfile,
        current_ts: u64,
        top_k: usize,
    ) -> Vec<(f32, u32, usize)> {
        if records.is_empty() {
            return Vec::new();
        }

        let avg_dl = (self.total_tokens as f32 / (self.total_docs.max(1) as f32)).max(1.0);
        let n_docs = self.total_docs as f32;
        let k1 = 1.5f32;
        let b = 0.75f32;

        // 1. Score Okapi BM25 over interned SMC token IDs (Octave 3)
        let mut scores: HashMap<u32, (f32, usize)> = HashMap::new();
        for &q_tok in query_token_ids {
            if let Some(post_list) = self.postings.get(&q_tok) {
                let df = post_list.len() as f32;
                let idf = ((n_docs - df + 0.5) / (df + 0.5) + 1.0).ln();
                for &(turn_id, tf) in post_list {
                    let dl = *self.doc_lengths.get(&turn_id).unwrap_or(&1) as f32;
                    let tf_f = tf as f32;
                    let bm25 =
                        idf * ((tf_f * (k1 + 1.0)) / (tf_f + k1 * (1.0 - b + b * (dl / avg_dl))));
                    let entry = scores.entry(turn_id).or_insert((0.0, 0));
                    entry.0 += bm25;
                }
            }
        }

        // 2. Boost exact domain sigil matches (Octave 2)
        for dom in query_domains {
            if let Some(turn_ids) = self.domain_postings.get(dom) {
                for &tid in turn_ids {
                    let entry = scores.entry(tid).or_insert((0.0, 0));
                    entry.0 += 2.75;
                }
            }
        }

        // 3. Probe the Nine-Octave Parabolic Phantom Quartz Crystal!
        let apex_hits = crystal.probe_nine_octave_parabolic(
            query_domains,
            query_token_ids,
            query_profile,
            current_ts,
            records.len().min(64),
        );
        for hit in apex_hits {
            let entry = scores.entry(hit.turn_id).or_insert((0.0, 0));
            entry.0 += hit.apex_amplitude_score * 2.15;
            entry.1 = hit.converging_beams;
        }

        // 4. Score Octaves 5..9 Affective, Epistemic, Audience & Dynamic Consequence Resonance
        let mut ranked: Vec<(f32, u32, usize)> = scores
            .into_iter()
            .filter_map(|(turn_id, (base_score, beams))| {
                let rec = records.get((turn_id.saturating_sub(1)) as usize)?;
                let prof = rec.octave_profile;

                // Octave 5: Emotional Weight similarity + high-stakes amplification
                let emo_diff =
                    (prof.emotional_weight as f32 - query_profile.emotional_weight as f32).abs();
                let emo_resonance = (1.0 - (emo_diff / 255.0)) * 1.15
                    + (prof.emotional_weight as f32 / 255.0) * 0.65;

                // Octave 6: Valence Polarity match (e.g. FrictionWarning or SacredWarmth)
                let valence_bonus = if prof.valence_kind == query_profile.valence_kind {
                    1.65
                } else if (prof.valence_signed >= 0) == (query_profile.valence_signed >= 0) {
                    0.55
                } else {
                    0.0
                };

                // Octave 7: Epistemic Certainty lock bonus (HardFactLock overrides tentative guesses!)
                let certainty_bonus = match prof.certainty {
                    CertaintyLevel::HardFactLock => 2.10,
                    CertaintyLevel::HighConfidence => 1.15,
                    CertaintyLevel::HypothesisEstimate => 0.35,
                    CertaintyLevel::OpenQuestion => {
                        if query_profile.certainty == CertaintyLevel::OpenQuestion {
                            1.40
                        } else {
                            0.20
                        }
                    }
                };

                // Octave 8: Audience Scope resonance
                let audience_bonus = if prof.audience == query_profile.audience {
                    1.85
                } else {
                    0.30 * (prof.audience.reach_multiplier().ln())
                };

                // Octave 9: Dynamic Single-Object Temporal Consequence at `current_ts`
                let proj_cons = rec.project_consequence_at(current_ts);
                let consequence_bonus = (proj_cons.effective_consequence * 0.55).min(4.50);

                let priority_mult = match rec.flags.priority() {
                    Priority::PinnedLock => 1.45,
                    Priority::High => 1.15,
                    Priority::Normal => 1.0,
                };

                let total = (base_score
                    + emo_resonance
                    + valence_bonus
                    + certainty_bonus
                    + audience_bonus
                    + consequence_bonus)
                    * priority_mult;

                Some((total, turn_id, beams))
            })
            .collect();

        ranked.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.1.cmp(&a.1))
        });
        ranked.truncate(top_k);
        ranked
    }

    /// Backwards-compatible wrapper invoking `hybrid_retrieve_nine_octave`.
    pub fn hybrid_retrieve(
        &self,
        crystal: &PhantomQuartzCrystal,
        records: &[TurnRecord],
        query_token_ids: &[u16],
        query_domains: &[String],
        current_ts: u64,
        top_k: usize,
    ) -> Vec<(f32, u32, usize)> {
        self.hybrid_retrieve_nine_octave(
            crystal,
            records,
            query_token_ids,
            query_domains,
            NineOctaveProfile::default(),
            current_ts,
            top_k,
        )
    }
}
