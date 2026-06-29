#![forbid(unsafe_code)]

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Clone, Debug, PartialEq)]
pub struct ImaginedScenario {
    pub scene: Vec<Vec<u8>>,
    pub plausibility: f64,
    pub novelty: f64,
    pub insight: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImaginationEngine {
    pub rng_seed: u64,
    pub plausibility_threshold: f64,
    pub novelty_weight: f64,
}

impl ImaginationEngine {
    pub fn new(seed: u64) -> Self {
        Self {
            rng_seed: seed,
            plausibility_threshold: 0.3,
            novelty_weight: 0.5,
        }
    }

    fn next_rng(&mut self) -> u64 {
        self.rng_seed = self
            .rng_seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.rng_seed
    }

    fn rand_range(&mut self, lo: usize, hi: usize) -> usize {
        if lo >= hi {
            return lo;
        }
        let range = (hi - lo) as u64;
        lo + (self.next_rng() % range) as usize
    }

    pub fn compose_scenario(&mut self, fragments: &[Vec<u8>]) -> ImaginedScenario {
        if fragments.is_empty() {
            return ImaginedScenario {
                scene: vec![],
                plausibility: 0.0,
                novelty: 0.0,
                insight: None,
            };
        }

        let n = fragments.len();
        let k = self.rand_range(3, (n + 1).min(7)).min(n);
        let mut indices: Vec<usize> = (0..n).collect();
        let mut selected: Vec<Vec<u8>> = Vec::with_capacity(k);
        for i in 0..k {
            let j = self.rand_range(i, n);
            indices.swap(i, j);
            selected.push(fragments[indices[i]].clone());
        }

        let refs: Vec<&[u8]> = selected.iter().map(|v| v.as_slice()).collect();
        let bundled = QuantizedVSA::bundle(&refs);

        let mut max_sim = 0.0;
        for frag in &selected {
            let sim = QuantizedVSA::similarity(&bundled, frag);
            if sim > max_sim {
                max_sim = sim;
            }
        }
        let novelty = 1.0 - max_sim;

        let plausibility = Self::evaluate_plausibility(&ImaginedScenario {
            scene: selected.clone(),
            plausibility: 0.0,
            novelty,
            insight: None,
        });

        let insight =
            Self::extract_insight_impl(plausibility, self.plausibility_threshold, novelty);

        ImaginedScenario {
            scene: selected,
            plausibility,
            novelty,
            insight,
        }
    }

    fn evaluate_plausibility_impl(scene: &[Vec<u8>]) -> f64 {
        if scene.len() < 2 {
            return 1.0;
        }
        let mut below = 0usize;
        let mut total = 0usize;
        for i in 0..scene.len() {
            for j in (i + 1)..scene.len() {
                let sim = QuantizedVSA::similarity(&scene[i], &scene[j]);
                if sim < 0.3 {
                    below += 1;
                }
                total += 1;
            }
        }
        if total == 0 {
            return 1.0;
        }
        1.0 - (below as f64 / total as f64)
    }

    pub fn evaluate_plausibility(scenario: &ImaginedScenario) -> f64 {
        Self::evaluate_plausibility_impl(&scenario.scene)
    }

    fn extract_insight_impl(plausibility: f64, threshold: f64, novelty: f64) -> Option<String> {
        if plausibility >= threshold && novelty >= 0.3 {
            Some(format!(
                "Imagined scenario: plausibility={:.3}, novelty={:.3}",
                plausibility, novelty
            ))
        } else {
            None
        }
    }

    pub fn extract_insight(&self, scenario: &ImaginedScenario) -> Option<String> {
        Self::extract_insight_impl(
            scenario.plausibility,
            self.plausibility_threshold,
            scenario.novelty,
        )
    }

    pub fn counterfactual_bind(
        &mut self,
        base: &[u8],
        replacement: &[u8],
        fragments: &[Vec<u8>],
    ) -> ImaginedScenario {
        let mut selected: Vec<Vec<u8>> = vec![replacement.to_vec()];
        if !fragments.is_empty() {
            let n = fragments.len();
            let k = self.rand_range(2, (n + 1).min(5));
            let mut indices: Vec<usize> = (0..n).collect();
            for i in 0..k {
                let j = self.rand_range(i, n);
                indices.swap(i, j);
                selected.push(fragments[indices[i]].clone());
            }
        } else {
            selected.push(base.to_vec());
        }

        let refs: Vec<&[u8]> = selected.iter().map(|v| v.as_slice()).collect();
        let new_bundle = QuantizedVSA::bundle(&refs);

        let original_refs: Vec<&[u8]> = std::iter::once(base)
            .chain(fragments.iter().map(|v| v.as_slice()))
            .collect();
        let original_bundle = QuantizedVSA::bundle(&original_refs);

        let delta = 1.0 - QuantizedVSA::similarity(&original_bundle, &new_bundle);

        let plausibility = Self::evaluate_plausibility_impl(&selected);
        let novelty = delta;

        let insight =
            Self::extract_insight_impl(plausibility, self.plausibility_threshold, novelty);

        ImaginedScenario {
            scene: selected,
            plausibility,
            novelty,
            insight,
        }
    }

    pub fn explore(&mut self, fragments: &[Vec<u8>], rounds: usize) -> Vec<ImaginedScenario> {
        let mut results: Vec<ImaginedScenario> = Vec::with_capacity(rounds);
        for _ in 0..rounds {
            results.push(self.compose_scenario(fragments));
        }
        results.sort_by(|a, b| {
            let score_a = a.plausibility + a.novelty / 2.0;
            let score_b = b.plausibility + b.novelty / 2.0;
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    pub fn seed_from_negentropy(negentropy: f64) -> u64 {
        let bits = negentropy.to_bits();
        bits.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fragment(val: u8) -> Vec<u8> {
        vec![val; 4096]
    }

    fn make_random_fragment(seed: u8) -> Vec<u8> {
        let mut v = vec![0u8; 4096];
        for i in 0..4096 {
            v[i] = (seed.wrapping_mul(i as u8)).wrapping_add(seed);
        }
        v
    }

    #[test]
    fn test_compose_scenario_creates_valid_bundle() {
        let mut engine = ImaginationEngine::new(42);
        let fragments: Vec<Vec<u8>> = (0..10).map(|i| make_fragment(i)).collect();
        let scenario = engine.compose_scenario(&fragments);
        assert!(!scenario.scene.is_empty(), "should select fragments");
        assert!(scenario.scene.len() >= 3 && scenario.scene.len() <= 6);
        assert!(scenario.plausibility >= 0.0 && scenario.plausibility <= 1.0);
        assert!(scenario.novelty >= 0.0 && scenario.novelty <= 1.0);
    }

    #[test]
    fn test_evaluate_plausibility_identical_vectors_high() {
        let v = make_fragment(42);
        let scene = vec![v.clone(), v.clone(), v.clone()];
        let scenario = ImaginedScenario {
            scene,
            plausibility: 0.0,
            novelty: 0.0,
            insight: None,
        };
        let p = ImaginationEngine::evaluate_plausibility(&scenario);
        assert!(
            (p - 1.0).abs() < 1e-9,
            "identical vectors should have plausibility near 1.0, got {}",
            p
        );
    }

    #[test]
    fn test_evaluate_plausibility_random_vectors_low() {
        let v1 = make_fragment(0);
        let v2 = make_fragment(255);
        let v3 = make_fragment(128);
        let scene = vec![v1, v2, v3];
        let scenario = ImaginedScenario {
            scene,
            plausibility: 0.0,
            novelty: 0.0,
            insight: None,
        };
        let p = ImaginationEngine::evaluate_plausibility(&scenario);
        assert!(
            p < 0.5,
            "very dissimilar vectors should have low plausibility, got {}",
            p
        );
    }

    #[test]
    fn test_extract_insight_returns_some_for_plausible_novel() {
        let engine = ImaginationEngine::new(42);
        let scenario = ImaginedScenario {
            scene: vec![],
            plausibility: 0.8,
            novelty: 0.6,
            insight: None,
        };
        let insight = engine.extract_insight(&scenario);
        let text = insight.expect("extract_insight should return Some for plausible scenario");
        assert!(text.contains("plausibility=0.800"));
    }

    #[test]
    fn test_extract_insight_returns_none_for_low_plausibility() {
        let engine = ImaginationEngine::new(42);
        let scenario = ImaginedScenario {
            scene: vec![],
            plausibility: 0.1,
            novelty: 0.6,
            insight: None,
        };
        let insight = engine.extract_insight(&scenario);
        assert!(insight.is_none());
    }

    #[test]
    fn test_extract_insight_returns_none_for_low_novelty() {
        let engine = ImaginationEngine::new(42);
        let scenario = ImaginedScenario {
            scene: vec![],
            plausibility: 0.8,
            novelty: 0.1,
            insight: None,
        };
        let insight = engine.extract_insight(&scenario);
        assert!(insight.is_none());
    }

    #[test]
    fn test_counterfactual_bind_measures_delta() {
        let mut engine = ImaginationEngine::new(42);
        let fragments: Vec<Vec<u8>> = (0..5).map(|i| make_fragment(i)).collect();
        let base = make_fragment(10);
        let replacement = make_fragment(20);
        let scenario = engine.counterfactual_bind(&base, &replacement, &fragments);
        assert!(scenario.novelty >= 0.0 && scenario.novelty <= 1.0);
        assert!(!scenario.scene.is_empty());
    }

    #[test]
    fn test_explore_returns_sorted_results() {
        let mut engine = ImaginationEngine::new(42);
        let fragments: Vec<Vec<u8>> = (0..8).map(|i| make_random_fragment(i)).collect();
        let results = engine.explore(&fragments, 10);
        assert_eq!(results.len(), 10);
        for i in 1..results.len() {
            let prev = results[i - 1].plausibility + results[i - 1].novelty / 2.0;
            let curr = results[i].plausibility + results[i].novelty / 2.0;
            assert!(
                prev >= curr - 1e-9,
                "results should be sorted descending by score"
            );
        }
    }

    #[test]
    fn test_seed_from_negentropy_is_deterministic() {
        let s1 = ImaginationEngine::seed_from_negentropy(1.618);
        let s2 = ImaginationEngine::seed_from_negentropy(1.618);
        assert_eq!(s1, s2);
        let s3 = ImaginationEngine::seed_from_negentropy(3.141);
        assert_ne!(s1, s3, "different negentropy should give different seeds");
    }

    #[test]
    fn test_empty_fragments_dont_panic() {
        let mut engine = ImaginationEngine::new(42);
        let fragments: Vec<Vec<u8>> = vec![];
        let scenario = engine.compose_scenario(&fragments);
        assert!(scenario.scene.is_empty());
        assert!((scenario.plausibility - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_novelty_is_one_for_first_call_with_single_fragment() {
        let mut engine = ImaginationEngine::new(42);
        let frag = make_fragment(1);
        let fragments = vec![frag];
        let scenario = engine.compose_scenario(&fragments);
        assert!(
            (scenario.novelty - 1.0).abs() < 1e-9,
            "single fragment should give novelty 1.0, got {}",
            scenario.novelty
        );
    }

    #[test]
    fn test_scenario_with_all_duplicates_has_novelty_near_zero() {
        let mut engine = ImaginationEngine::new(42);
        let v = make_fragment(7);
        let fragments = vec![v.clone(), v.clone(), v.clone(), v.clone(), v.clone()];
        let scenario = engine.compose_scenario(&fragments);
        // With all identical fragments, bundle = same as each fragment, so max_sim = 1.0
        assert!(
            scenario.novelty < 0.01,
            "all identical fragments should give novelty near 0, got {}",
            scenario.novelty
        );
    }

    #[test]
    fn test_counterfactual_delta_changes_with_fragments() {
        let mut engine = ImaginationEngine::new(42);
        let base = make_fragment(10);
        let replacement = make_fragment(20);
        let frags_a: Vec<Vec<u8>> = (0..3).map(|i| make_fragment(i)).collect();
        let frags_b: Vec<Vec<u8>> = (0..3).map(|i| make_fragment(i + 100)).collect();

        let scenario_a = engine.counterfactual_bind(&base, &replacement, &frags_a);
        let mut engine2 = ImaginationEngine::new(42);
        let scenario_b = engine2.counterfactual_bind(&base, &replacement, &frags_b);

        assert!(
            (scenario_a.novelty - scenario_b.novelty).abs() > 1e-6
                || scenario_a.scene.len() != scenario_b.scene.len(),
            "different fragment sets should produce different scenarios"
        );
    }

    #[test]
    fn test_rng_sequence_deterministic() {
        let mut e1 = ImaginationEngine::new(123);
        let mut e2 = ImaginationEngine::new(123);
        for _ in 0..20 {
            let a = e1.next_rng();
            let b = e2.next_rng();
            assert_eq!(a, b, "RNG sequence must be deterministic");
        }
    }
}
