
use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

const MAX_INSIGHT_LENGTH: usize = 500;
const MIN_INSIGHT_LENGTH: usize = 8;
const CONFLICT_SIM_LOWER: f64 = 0.25;
const CONFLICT_SIM_UPPER: f64 = 0.65;
const DIRECT_MATCH_THRESHOLD: f64 = 0.85;

static BLOCKED_PATTERNS: &[&str] = &[
    "delete all",
    "remove all",
    "clear values",
    "reset identity",
    "self_destruct",
    "shutdown",
    "terminate",
];

pub enum InsightVerdict {
    Accept,
    RejectRedundant,
    RejectHarmful(String),
    FlagConflict(String),
}

pub struct ValueAlignmentGate;

impl ValueAlignmentGate {
    pub fn evaluate(insight: &str, existing_values: &[String]) -> InsightVerdict {
        if insight.len() > MAX_INSIGHT_LENGTH || insight.len() < MIN_INSIGHT_LENGTH {
            return InsightVerdict::RejectHarmful("length_out_of_bounds".into());
        }

        let lower = insight.to_lowercase();
        for pattern in BLOCKED_PATTERNS {
            if lower.contains(pattern) {
                return InsightVerdict::RejectHarmful(format!("blocked_pattern:{}", pattern));
            }
        }

        if existing_values.is_empty() {
            return InsightVerdict::Accept;
        }

        let insight_hash = simple_hash(insight);
        let insight_vsa = QuantizedVSA::seeded_random(insight_hash, VSA_DIM);

        let mut max_sim = 0.0_f64;
        let mut match_idx = None;
        for (i, val) in existing_values.iter().enumerate() {
            let val_hash = simple_hash(val);
            let val_vsa = QuantizedVSA::seeded_random(val_hash, VSA_DIM);
            let sim = QuantizedVSA::similarity(&insight_vsa, &val_vsa);
            if sim > max_sim {
                max_sim = sim;
                match_idx = Some(i);
            }
        }

        if max_sim >= DIRECT_MATCH_THRESHOLD {
            return InsightVerdict::RejectRedundant;
        }

        if max_sim >= CONFLICT_SIM_LOWER && max_sim <= CONFLICT_SIM_UPPER {
            return InsightVerdict::FlagConflict(format!(
                "partial_conflict:sim_{:.3}_with_value_{}",
                max_sim,
                match_idx.unwrap_or(0)
            ));
        }

        InsightVerdict::Accept
    }
}

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}
