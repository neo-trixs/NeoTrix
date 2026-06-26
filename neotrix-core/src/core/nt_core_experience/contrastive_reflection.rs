#![forbid(unsafe_code)]

use crate::core::nt_core_hcube::QuantizedVSA;
use crate::core::nt_core_hcube::VSA_DIM;

/// Maximum number of contrastive trajectory pairs stored in the ring buffer.
const MAX_PAIRS: usize = 100;

/// A recorded contrastive pair of a successful trajectory and a failed trajectory
/// on the same or similar task, used to extract learning signals.
#[derive(Debug, Clone)]
pub struct ContrastivePair {
    pub success_trajectory: Vec<String>,
    pub failure_trajectory: Vec<String>,
    pub divergences: Vec<String>,
}

/// Maintains a ring buffer of success/failure trajectory pairs and provides
/// methods to extract reasoning divergences, error patterns, and reusable insights.
///
/// Contrastive reflection is the core mechanism for self-improvement: by comparing
/// what went right vs what went wrong on similar tasks, the system identifies
/// the specific reasoning steps that separate success from failure.
#[derive(Debug, Clone)]
pub struct ContrastiveReflection {
    pairs: Vec<ContrastivePair>,
    max_pairs: usize,
}

impl Default for ContrastiveReflection {
    fn default() -> Self {
        Self::new()
    }
}

impl ContrastiveReflection {
    pub fn new() -> Self {
        Self {
            pairs: Vec::with_capacity(MAX_PAIRS),
            max_pairs: MAX_PAIRS,
        }
    }

    pub fn with_max_pairs(max_pairs: usize) -> Self {
        Self {
            pairs: Vec::with_capacity(max_pairs),
            max_pairs,
        }
    }

    /// Returns the number of stored contrastive pairs.
    pub fn pair_count(&self) -> usize {
        self.pairs.len()
    }

    /// Stores a contrastive pair. When capacity is exceeded, the oldest entry
    /// (front of the queue) is evicted.
    pub fn record_contrastive(&mut self, success: Vec<String>, failure: Vec<String>) {
        let divergences = Self::extract_divergence_inner(&success, &failure);
        let pair = ContrastivePair {
            success_trajectory: success,
            failure_trajectory: failure,
            divergences,
        };
        if self.pairs.len() >= self.max_pairs {
            self.pairs.remove(0);
        }
        self.pairs.push(pair);
    }

    /// Compares a success trajectory against a failure trajectory step by step.
    /// Returns the reasoning steps where the VSA cosine similarity between
    /// the corresponding steps differs by more than 0.3.
    ///
    /// Each step is encoded into a VSA vector via a deterministic hash, then
    /// cosine similarity is computed. Steps with `1.0 - cosine_sim > 0.3`
    /// are considered divergences.
    pub fn extract_divergence(success_traj: &[String], fail_traj: &[String]) -> Vec<String> {
        Self::extract_divergence_inner(success_traj, fail_traj)
    }

    fn extract_divergence_inner(success_traj: &[String], fail_traj: &[String]) -> Vec<String> {
        let max_len = success_traj.len().max(fail_traj.len());
        if max_len == 0 {
            return Vec::new();
        }

        let mut divergences = Vec::new();
        for i in 0..max_len {
            let s_step = success_traj.get(i).map(|s| s.as_str()).unwrap_or("");
            let f_step = fail_traj.get(i).map(|s| s.as_str()).unwrap_or("");
            if s_step.is_empty() && f_step.is_empty() {
                continue;
            }
            let s_vsa = str_to_vsa(s_step);
            let f_vsa = str_to_vsa(f_step);
            let sim = QuantizedVSA::cosine(&s_vsa, &f_vsa);
            let diff = 1.0 - sim;
            if diff > 0.3 {
                let label = if s_step.is_empty() {
                    format!("step[{}]: [no success step] | failure: {}", i, f_step)
                } else if f_step.is_empty() {
                    format!("step[{}]: {} | [no failure step]", i, s_step)
                } else {
                    format!("step[{}]: {} | {}", i, s_step, f_step)
                };
                divergences.push(label);
            }
        }
        divergences
    }

    /// Abstracts common error modes from a collection of failure trajectory steps.
    /// Uses a heuristic: detects repeated words/phrases across failure steps,
    /// returning only unique patterns that appear more than once.
    pub fn summarize_error_patterns(failures: &[String]) -> Vec<String> {
        if failures.is_empty() {
            return Vec::new();
        }
        let mut word_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for step in failures {
            for token in step.split_whitespace() {
                let cleaned = token.trim_matches(|c: char| !c.is_alphanumeric());
                if cleaned.len() > 3 {
                    *word_counts.entry(cleaned.to_lowercase()).or_insert(0) += 1;
                }
            }
        }
        let mut patterns: Vec<String> = word_counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(word, count)| format!("{} (appears {}x)", word, count))
            .collect();
        patterns.sort_by(|a, b| b.len().cmp(&a.len()));
        patterns.truncate(10);
        patterns
    }

    /// Extracts reusable strategy principles from successful trajectory steps.
    /// Returns unique action-oriented phrases (verbs followed by context) that
    /// appear to be generalizable strategies.
    pub fn extract_reusable_insights(successes: &[String]) -> Vec<String> {
        if successes.is_empty() {
            return Vec::new();
        }
        let mut seen = std::collections::HashSet::new();
        let mut insights = Vec::new();
        for step in successes {
            let step_lower = step.to_lowercase();
            let words: Vec<&str> = step_lower.split_whitespace().collect();
            for window in words.windows(3) {
                let phrase = window.join(" ");
                if seen.insert(phrase.clone()) {
                    insights.push(phrase);
                }
            }
        }
        insights.sort();
        insights.truncate(10);
        insights
    }

    /// Returns a formatted report of the last 10 divergences across all
    /// stored contrastive pairs.
    pub fn divergence_report(&self) -> String {
        let mut report = String::from("=== Contrastive Divergence Report ===\n");
        let start = if self.pairs.len() > 10 {
            self.pairs.len() - 10
        } else {
            0
        };
        for (pair_idx, pair) in self.pairs[start..].iter().enumerate() {
            let global_idx = start + pair_idx;
            report.push_str(&format!(
                "\n--- Pair {} ({} success steps, {} failure steps, {} divergences) ---\n",
                global_idx,
                pair.success_trajectory.len(),
                pair.failure_trajectory.len(),
                pair.divergences.len()
            ));
            for (d_idx, div) in pair.divergences.iter().enumerate() {
                report.push_str(&format!("  div[{}]: {}\n", d_idx, div));
            }
            if pair.divergences.is_empty() {
                report.push_str("  (no significant divergences)\n");
            }
        }
        if self.pairs.is_empty() {
            report.push_str("(no contrastive pairs recorded)\n");
        }
        report
    }
}

/// Encodes a string into a VSA vector (Vec<u8> of length VSA_DIM) using
/// a deterministic hash: the string's bytes are mixed via a simple
/// folding hash seeded by position, producing ±1-like binary values.
fn str_to_vsa(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut vec = Vec::with_capacity(VSA_DIM);
    for i in 0..VSA_DIM {
        let mut h = i as u64;
        for (j, &b) in bytes.iter().enumerate() {
            h = h.wrapping_mul(31).wrapping_add(b as u64);
            h = h.wrapping_mul(7).wrapping_add(j as u64);
        }
        vec.push((h & 0xFF) as u8);
    }
    vec
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_success_traj() -> Vec<String> {
        vec![
            "identify the core query and break it into subproblems".into(),
            "retrieve relevant knowledge from structured sources".into(),
            "verify each subresult against known constraints".into(),
            "compose partial results into coherent answer".into(),
            "double-check answer for logical consistency".into(),
        ]
    }

    fn make_failure_traj() -> Vec<String> {
        vec![
            "identify the core query and break it into subproblems".into(),
            "retrieve relevant knowledge from unstructured sources".into(),
            "skip verification due to time pressure".into(),
            "compose partial results into coherent answer".into(),
            "proceed without double-check".into(),
        ]
    }

    fn make_empty_traj() -> Vec<String> {
        Vec::new()
    }

    #[test]
    fn test_new_is_empty() {
        let cr = ContrastiveReflection::new();
        assert_eq!(cr.pair_count(), 0);
    }

    #[test]
    fn test_record_single_pair() {
        let mut cr = ContrastiveReflection::new();
        cr.record_contrastive(make_success_traj(), make_failure_traj());
        assert_eq!(cr.pair_count(), 1);
    }

    #[test]
    fn test_extract_divergence_detects_differing_steps() {
        let divs =
            ContrastiveReflection::extract_divergence(&make_success_traj(), &make_failure_traj());
        assert!(!divs.is_empty(), "should find at least one divergence");
    }

    #[test]
    fn test_extract_divergence_identical_trajs_returns_empty() {
        let traj = make_success_traj();
        let divs = ContrastiveReflection::extract_divergence(&traj, &traj);
        assert!(
            divs.is_empty(),
            "identical trajectories should have no divergences"
        );
    }

    #[test]
    fn test_extract_divergence_empty_success() {
        let divs = ContrastiveReflection::extract_divergence(&[], &make_failure_traj());
        assert!(
            !divs.is_empty(),
            "should find divergences when success is empty"
        );
    }

    #[test]
    fn test_extract_divergence_both_empty() {
        let divs =
            ContrastiveReflection::extract_divergence(&make_empty_traj(), &make_empty_traj());
        assert!(divs.is_empty(), "both empty should yield no divergences");
    }

    #[test]
    fn test_lru_eviction_removes_oldest() {
        let mut cr = ContrastiveReflection::with_max_pairs(3);
        for i in 0..5 {
            cr.record_contrastive(
                vec![format!("success step {}", i)],
                vec![format!("failure step {}", i)],
            );
        }
        assert_eq!(cr.pair_count(), 3);
        // oldest (i=0, 1) evicted; youngest (i=2, 3, 4) remain
        if let Some(pair) = cr.pairs.first() {
            assert!(
                pair.success_trajectory[0].contains("2")
                    || pair.success_trajectory[0].contains("3")
                    || pair.success_trajectory[0].contains("4"),
                "evicted oldest, youngest remain"
            );
        }
    }

    #[test]
    fn test_lru_eviction_exact_capacity() {
        let mut cr = ContrastiveReflection::with_max_pairs(2);
        cr.record_contrastive(vec!["first".into()], vec!["first fail".into()]);
        cr.record_contrastive(vec!["second".into()], vec!["second fail".into()]);
        assert_eq!(cr.pair_count(), 2);
        cr.record_contrastive(vec!["third".into()], vec!["third fail".into()]);
        assert_eq!(cr.pair_count(), 2);
        // "first" should be evicted
        assert!(
            cr.pairs.iter().any(|p| p.success_trajectory[0] == "second"),
            "second should remain"
        );
        assert!(
            cr.pairs.iter().any(|p| p.success_trajectory[0] == "third"),
            "third should remain"
        );
    }

    #[test]
    fn test_summarize_error_patterns_finds_repeated_terms() {
        let failures = vec![
            "failed to retrieve data from source".into(),
            "failed to validate the result format".into(),
            "skip due to ambiguous constraint".into(),
        ];
        let patterns = ContrastiveReflection::summarize_error_patterns(&failures);
        assert!(
            patterns.iter().any(|p| p.contains("failed")),
            "should detect repeated 'failed'"
        );
    }

    #[test]
    fn test_summarize_error_patterns_empty() {
        let patterns = ContrastiveReflection::summarize_error_patterns(&[]);
        assert!(patterns.is_empty(), "empty input yields empty patterns");
    }

    #[test]
    fn test_summarize_error_patterns_singleton() {
        let patterns = ContrastiveReflection::summarize_error_patterns(&["one off error".into()]);
        assert!(
            patterns.is_empty(),
            "single step should not produce repeated patterns"
        );
    }

    #[test]
    fn test_extract_reusable_insights_extracts_phrases() {
        let successes = vec![
            "verify the result before composing".into(),
            "verify the constraint before proceeding".into(),
        ];
        let insights = ContrastiveReflection::extract_reusable_insights(&successes);
        assert!(
            insights.iter().any(|i| i.contains("verify")),
            "should extract verify phrases"
        );
    }

    #[test]
    fn test_extract_reusable_insights_empty() {
        let insights = ContrastiveReflection::extract_reusable_insights(&[]);
        assert!(insights.is_empty(), "empty input yields empty insights");
    }

    #[test]
    fn test_divergence_report_empty() {
        let cr = ContrastiveReflection::new();
        let report = cr.divergence_report();
        assert!(report.contains("no contrastive pairs"));
    }

    #[test]
    fn test_divergence_report_with_pairs() {
        let mut cr = ContrastiveReflection::new();
        cr.record_contrastive(make_success_traj(), make_failure_traj());
        let report = cr.divergence_report();
        assert!(report.contains("Pair 0"));
        assert!(report.contains("divergences"));
    }

    #[test]
    fn test_divergence_report_truncates_to_last_10() {
        let mut cr = ContrastiveReflection::with_max_pairs(20);
        for i in 0..15 {
            cr.record_contrastive(vec![format!("s {}", i)], vec![format!("f {}", i)]);
        }
        let report = cr.divergence_report();
        // should only show pairs 5-14 (last 10)
        assert!(report.contains("Pair 5"), "should start from pair 5");
        assert!(report.contains("Pair 14"), "should end at pair 14");
        assert!(!report.contains("Pair 0"), "should not include pair 0");
    }

    #[test]
    fn test_contrastive_pair_stores_divergences() {
        let success = vec!["think step by step".into(), "validate the output".into()];
        let failure = vec!["think step by step".into(), "skip validation".into()];
        let mut cr = ContrastiveReflection::new();
        cr.record_contrastive(success, failure);
        let pair = &cr.pairs[0];
        assert!(
            !pair.divergences.is_empty(),
            "should detect divergence in second step"
        );
        assert!(
            pair.divergences[0].contains("skip validation"),
            "divergence should reference the differing step"
        );
    }

    #[test]
    fn test_default_capacity() {
        let cr = ContrastiveReflection::default();
        // add 101 pairs, should keep exactly 100
        let mut cr = cr;
        for i in 0..101 {
            cr.record_contrastive(vec![format!("s {}", i)], vec![format!("f {}", i)]);
        }
        assert_eq!(cr.pair_count(), 100);
    }
}
