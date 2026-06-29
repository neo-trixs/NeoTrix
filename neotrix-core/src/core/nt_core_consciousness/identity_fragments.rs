use std::collections::VecDeque;

use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

const DEFAULT_CONFLICT_THRESHOLD: f64 = 0.3;
const DEFAULT_MIN_CONFIDENCE: f64 = 0.4;
const DEFAULT_MAX_FRAGMENTS: usize = 10;
const FRAGMENT_SIMILARITY_THRESHOLD: f64 = 0.7;
const HIGH_SIMILARITY_THRESHOLD: f64 = 0.7;

#[derive(Debug, Clone)]
pub struct IdentityFragment {
    pub fragment_vsa: Vec<u8>,
    pub label: String,
    pub beliefs: Vec<Vec<u8>>,
    pub behaviors: Vec<String>,
    pub context_triggers: Vec<String>,
    pub confidence: f64,
    pub coherence: f64,
    pub last_active: u64,
}

#[derive(Debug, Clone)]
pub struct FragmentationReport {
    pub fragments: Vec<IdentityFragment>,
    pub pairwise_conflicts: Vec<(usize, usize, f64)>,
    pub overall_fragmentation: f64,
    pub dominant_fragment: Option<usize>,
    pub integration_recommendations: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct IdentityFragmentDetector {
    pub fragments: Vec<IdentityFragment>,
    pub conflict_threshold: f64,
    pub min_fragment_confidence: f64,
    pub integration_readiness: f64,
    pub fragmentation_history: VecDeque<f64>,
    pub max_fragments: usize,
    pub cycle_count: u64,
}

impl IdentityFragmentDetector {
    pub fn new() -> Self {
        Self {
            fragments: Vec::new(),
            conflict_threshold: DEFAULT_CONFLICT_THRESHOLD,
            min_fragment_confidence: DEFAULT_MIN_CONFIDENCE,
            integration_readiness: 0.0,
            fragmentation_history: VecDeque::new(),
            max_fragments: DEFAULT_MAX_FRAGMENTS,
            cycle_count: 0,
        }
    }

    pub fn register_fragment(
        &mut self,
        label: &str,
        beliefs: &[&[u8]],
        behaviors: &[&str],
        contexts: &[&str],
        confidence: f64,
    ) {
        let fragment_vsa = QuantizedVSA::seeded_random(
            label
                .bytes()
                .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64)),
            VSA_DIM,
        );
        let now = now_secs();

        if let Some(existing) = self.fragments.iter_mut().find(|f| {
            QuantizedVSA::similarity(&f.fragment_vsa, &fragment_vsa) > FRAGMENT_SIMILARITY_THRESHOLD
        }) {
            for &b in beliefs {
                if !existing
                    .beliefs
                    .iter()
                    .any(|eb| QuantizedVSA::similarity(eb, b) > 0.9)
                {
                    existing.beliefs.push(b.to_vec());
                }
            }
            for &b in behaviors {
                if !existing.behaviors.contains(&b.to_string()) {
                    existing.behaviors.push(b.to_string());
                }
            }
            for &c in contexts {
                if !existing.context_triggers.contains(&c.to_string()) {
                    existing.context_triggers.push(c.to_string());
                }
            }
            existing.confidence = existing.confidence.max(confidence);
            existing.last_active = now;
            return;
        }

        let beliefs_vec: Vec<Vec<u8>> = beliefs.iter().map(|b| b.to_vec()).collect();
        let behaviors_vec: Vec<String> = behaviors.iter().map(|s| s.to_string()).collect();
        let contexts_vec: Vec<String> = contexts.iter().map(|s| s.to_string()).collect();

        if self.fragments.len() >= self.max_fragments {
            log::warn!(
                "IdentityFragmentDetector: max_fragments ({}) reached, fragment '{}' not registered",
                self.max_fragments,
                label
            );
            return;
        }

        self.fragments.push(IdentityFragment {
            fragment_vsa,
            label: label.to_string(),
            beliefs: beliefs_vec,
            behaviors: behaviors_vec,
            context_triggers: contexts_vec,
            confidence,
            coherence: 1.0,
            last_active: now,
        });
    }

    pub fn detect_conflicts(&self) -> Vec<(usize, usize, f64)> {
        let mut conflicts = Vec::new();
        for i in 0..self.fragments.len() {
            for j in (i + 1)..self.fragments.len() {
                let bundled_i = self.bundle_fragment_beliefs(i);
                let bundled_j = self.bundle_fragment_beliefs(j);
                let sim = if bundled_i.is_empty() || bundled_j.is_empty() {
                    0.5
                } else {
                    QuantizedVSA::similarity(&bundled_i, &bundled_j)
                };
                let severity = 1.0 - sim;
                if severity > 1.0 - self.conflict_threshold {
                    conflicts.push((i, j, severity));
                }
            }
        }
        conflicts
    }

    fn bundle_fragment_beliefs(&self, idx: usize) -> Vec<u8> {
        if idx >= self.fragments.len() {
            return Vec::new();
        }
        let fragment = &self.fragments[idx];
        if fragment.beliefs.is_empty() {
            return Vec::new();
        }
        let refs: Vec<&[u8]> = fragment.beliefs.iter().map(|b| b.as_slice()).collect();
        QuantizedVSA::bundle(&refs)
    }

    pub fn generate_report(&self) -> FragmentationReport {
        let now = now_secs();

        let all_conflicts: Vec<(usize, usize, f64)> = {
            let mut pairs = Vec::new();
            for i in 0..self.fragments.len() {
                for j in (i + 1)..self.fragments.len() {
                    let bundled_i = self.bundle_fragment_beliefs(i);
                    let bundled_j = self.bundle_fragment_beliefs(j);
                    let sim = if bundled_i.is_empty() || bundled_j.is_empty() {
                        0.5
                    } else {
                        QuantizedVSA::similarity(&bundled_i, &bundled_j)
                    };
                    pairs.push((i, j, 1.0 - sim));
                }
            }
            pairs
        };

        let total_pairs = all_conflicts.len();
        let conflicting_pairs = all_conflicts
            .iter()
            .filter(|(_, _, s)| *s > 1.0 - self.conflict_threshold)
            .count();
        let overall_fragmentation = if total_pairs > 0 {
            conflicting_pairs as f64 / total_pairs as f64
        } else {
            0.0
        };

        let dominant_fragment = self
            .fragments
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx);

        let integration_recommendations = self
            .suggest_integration()
            .iter()
            .map(|(i, j, reason)| {
                format!(
                    "merge fragments '{}' and '{}': {}",
                    self.fragments[*i].label, self.fragments[*j].label, reason
                )
            })
            .collect();

        FragmentationReport {
            fragments: self.fragments.clone(),
            pairwise_conflicts: all_conflicts,
            overall_fragmentation,
            dominant_fragment,
            integration_recommendations,
            timestamp: now,
        }
    }

    pub fn coherence_score(&self) -> f64 {
        1.0 - self.generate_report().overall_fragmentation.clamp(0.0, 1.0)
    }

    pub fn suggest_integration(&self) -> Vec<(usize, usize, String)> {
        let mut suggestions = Vec::new();
        for i in 0..self.fragments.len() {
            for j in (i + 1)..self.fragments.len() {
                let bundled_i = self.bundle_fragment_beliefs(i);
                let bundled_j = self.bundle_fragment_beliefs(j);
                let belief_sim = if bundled_i.is_empty() || bundled_j.is_empty() {
                    0.0
                } else {
                    QuantizedVSA::similarity(&bundled_i, &bundled_j)
                };
                if belief_sim > HIGH_SIMILARITY_THRESHOLD {
                    let behavior_conflict = self.compute_behavioral_conflict(i, j);
                    if behavior_conflict < self.conflict_threshold {
                        suggestions.push((
                            i,
                            j,
                            format!(
                                "high belief similarity ({:.3}), low behavioral conflict ({:.3})",
                                belief_sim, behavior_conflict
                            ),
                        ));
                    }
                }
            }
        }
        suggestions
    }

    fn compute_behavioral_conflict(&self, i: usize, j: usize) -> f64 {
        let fi = &self.fragments[i];
        let fj = &self.fragments[j];
        if fi.behaviors.is_empty() || fj.behaviors.is_empty() {
            return 0.0;
        }
        let mut overlap = 0;
        for b in &fi.behaviors {
            if fj.behaviors.contains(b) {
                overlap += 1;
            }
        }
        let max_len = fi.behaviors.len().max(fj.behaviors.len());
        1.0 - (overlap as f64 / max_len as f64)
    }

    pub fn set_threshold(&mut self, threshold: f64) {
        self.conflict_threshold = threshold.clamp(0.0, 1.0);
    }

    pub fn reset(&mut self) {
        self.fragments.clear();
        self.fragmentation_history.clear();
        self.integration_readiness = 0.0;
        self.cycle_count = 0;
    }
}

impl Default for IdentityFragmentDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl IdentityFragmentDetector {
    pub fn tick(&mut self, _delta: f64) {
        self.cycle_count += 1;
    }

    pub fn fragment_count(&self) -> usize {
        self.fragments.len()
    }
}

fn now_secs() -> u64 {
    crate::core::nt_core_time::unix_now_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vsa(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, VSA_DIM)
    }

    #[test]
    fn test_new_detector_defaults() {
        let d = IdentityFragmentDetector::new();
        assert!(d.fragments.is_empty());
        assert!((d.conflict_threshold - 0.3).abs() < 1e-6);
        assert!((d.min_fragment_confidence - 0.4).abs() < 1e-6);
        assert_eq!(d.max_fragments, 10);
        assert!(d.fragmentation_history.is_empty());
        assert_eq!(d.cycle_count, 0);
    }

    #[test]
    fn test_register_single_fragment() {
        let mut d = IdentityFragmentDetector::new();
        let belief = make_vsa(100);
        d.register_fragment("alpha", &[&belief], &["explore"], &["unknown"], 0.8);
        assert_eq!(d.fragments.len(), 1);
        assert_eq!(d.fragments[0].label, "alpha");
        assert!((d.fragments[0].confidence - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_register_multiple_fragments() {
        let mut d = IdentityFragmentDetector::new();
        d.register_fragment(
            "alpha",
            &[&make_vsa(100)],
            &["explore"],
            &["context_a"],
            0.8,
        );
        d.register_fragment("beta", &[&make_vsa(200)], &["analyze"], &["context_b"], 0.6);
        d.register_fragment("gamma", &[&make_vsa(300)], &["create"], &["context_c"], 0.7);
        assert_eq!(d.fragments.len(), 3);
    }

    #[test]
    fn test_detect_no_conflicts_when_similar() {
        let mut d = IdentityFragmentDetector::new();
        let belief = make_vsa(42);
        d.register_fragment("alpha", &[&belief], &["explore"], &["ctx"], 0.8);
        d.register_fragment("beta", &[&belief], &["explore"], &["ctx"], 0.7);
        let conflicts = d.detect_conflicts();
        let similar_has_low_severity = conflicts.iter().all(|(_, _, sev)| *sev <= 0.5);
        assert!(
            similar_has_low_severity,
            "identical beliefs should produce low conflict severity"
        );
    }

    #[test]
    fn test_detect_conflicts_when_dissimilar() {
        let mut d = IdentityFragmentDetector::new();
        let seed_a = 0u64;
        let seed_b = 1u64 << 63;
        let belief_a = make_vsa(seed_a);
        let belief_b = make_vsa(seed_b);
        d.register_fragment("alpha", &[&belief_a], &["approach"], &["ctx_a"], 0.9);
        d.register_fragment("beta", &[&belief_b], &["avoid"], &["ctx_b"], 0.9);
        let conflicts = d.detect_conflicts();
        assert!(
            !conflicts.is_empty(),
            "very different VSA beliefs should produce detectable conflicts"
        );
    }

    #[test]
    fn test_generate_report_basic() {
        let mut d = IdentityFragmentDetector::new();
        d.register_fragment("alpha", &[&make_vsa(10)], &["read"], &["library"], 0.8);
        d.register_fragment("beta", &[&make_vsa(20)], &["write"], &["studio"], 0.6);
        let report = d.generate_report();
        assert_eq!(report.fragments.len(), 2);
        assert_eq!(report.pairwise_conflicts.len(), 1);
        assert!(report.timestamp > 0);
    }

    #[test]
    fn test_generate_report_fragmentation_score() {
        let mut d = IdentityFragmentDetector::new();
        let shared = make_vsa(42);
        d.register_fragment("alpha", &[&shared], &["social"], &["home"], 0.9);
        d.register_fragment("beta", &[&shared], &["social"], &["work"], 0.8);
        let report_similar = d.generate_report();
        let low_frag = report_similar.overall_fragmentation;

        let mut d2 = IdentityFragmentDetector::new();
        d2.register_fragment("alpha", &[&make_vsa(0)], &["fight"], &["arena"], 0.9);
        d2.register_fragment("beta", &[&make_vsa(1 << 63)], &["flee"], &["safe"], 0.9);
        let report_diff = d2.generate_report();
        let high_frag = report_diff.overall_fragmentation;

        assert!(
            high_frag >= low_frag,
            "dissimilar beliefs should produce higher fragmentation than similar ones"
        );
    }

    #[test]
    fn test_coherence_score() {
        let mut d = IdentityFragmentDetector::new();
        d.register_fragment("alpha", &[&make_vsa(0)], &["fight"], &["a"], 0.9);
        d.register_fragment("beta", &[&make_vsa(1 << 63)], &["flee"], &["b"], 0.9);
        let coherence = d.coherence_score();
        assert!(coherence >= 0.0);
        assert!(coherence <= 1.0);
        let report = d.generate_report();
        assert!((coherence - (1.0 - report.overall_fragmentation)).abs() < 1e-6);
    }

    #[test]
    fn test_suggest_integration_for_similar_fragments() {
        let mut d = IdentityFragmentDetector::new();
        let shared = make_vsa(42);
        d.register_fragment("alpha", &[&shared], &["read"], &["library"], 0.8);
        d.register_fragment("beta", &[&shared], &["read"], &["library"], 0.7);
        let suggestions = d.suggest_integration();
        let alpha_beta = suggestions
            .iter()
            .any(|(i, j, _)| (*i == 0 && *j == 1) || (*i == 1 && *j == 0));
        assert!(
            alpha_beta,
            "similar fragments should be suggested for integration"
        );
    }

    #[test]
    fn test_dominant_fragment_is_highest_confidence() {
        let mut d = IdentityFragmentDetector::new();
        d.register_fragment("alpha", &[&make_vsa(1)], &["a"], &["x"], 0.5);
        d.register_fragment("beta", &[&make_vsa(2)], &["b"], &["y"], 0.9);
        d.register_fragment("gamma", &[&make_vsa(3)], &["c"], &["z"], 0.7);
        let report = d.generate_report();
        assert_eq!(report.dominant_fragment, Some(1));
    }

    #[test]
    fn test_set_threshold() {
        let mut d = IdentityFragmentDetector::new();
        assert!((d.conflict_threshold - 0.3).abs() < 1e-6);
        d.set_threshold(0.7);
        assert!((d.conflict_threshold - 0.7).abs() < 1e-6);
        d.set_threshold(1.5);
        assert!((d.conflict_threshold - 1.0).abs() < 1e-6);
        d.set_threshold(-0.5);
        assert!((d.conflict_threshold - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut d = IdentityFragmentDetector::new();
        d.register_fragment("alpha", &[&make_vsa(1)], &["a"], &["x"], 0.8);
        d.register_fragment("beta", &[&make_vsa(2)], &["b"], &["y"], 0.6);
        d.cycle_count = 5;
        d.fragmentation_history.push_back(0.3);
        d.reset();
        assert!(d.fragments.is_empty());
        assert!(d.fragmentation_history.is_empty());
        assert!((d.integration_readiness - 0.0).abs() < 1e-6);
        assert_eq!(d.cycle_count, 0);
    }
}
