use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ReflectionEntry {
    pub thought: Vec<u8>,
    pub timestamp: Instant,
    pub awareness_score: f64,
    pub coherence: f64,
}

#[derive(Debug, Clone)]
pub struct ReflexiveUnit {
    pub reflections: Vec<ReflectionEntry>,
    next_id: u64,
}

#[derive(Debug, Clone)]
pub struct ReflexiveConfig {
    pub max_depth: u32,
    pub min_coherence: f64,
    pub auto_reflect_interval: std::time::Duration,
}

impl Default for ReflexiveConfig {
    fn default() -> Self {
        Self {
            max_depth: 3,
            min_coherence: 0.5,
            auto_reflect_interval: std::time::Duration::from_secs(300),
        }
    }
}

impl ReflexiveUnit {
    pub fn new(config: ReflexiveConfig) -> Self {
        let _ = config;
        Self {
            reflections: Vec::new(),
            next_id: 1,
        }
    }

    pub fn reflect(&mut self, thought: &[u8]) -> (usize, f64) {
        let coherence = self.compute_vsa_coherence(thought);
        let awareness = self.compute_awareness(thought, coherence);
        self.reflections.push(ReflectionEntry {
            thought: thought.to_vec(),
            timestamp: Instant::now(),
            awareness_score: awareness,
            coherence,
        });
        if self.reflections.len() > 1000 {
            self.reflections.remove(0);
        }
        self.next_id += 1;
        (self.reflections.len(), awareness)
    }

    pub fn self_awareness_score(&self) -> f64 {
        if self.reflections.is_empty() {
            return 0.0;
        }
        let recent: f64 = self
            .reflections
            .iter()
            .rev()
            .take(10)
            .map(|e| e.awareness_score)
            .sum::<f64>()
            / self.reflections.len().min(10) as f64;
        let stability = self.compute_stability();
        0.6 * recent + 0.4 * stability
    }

    pub fn stats(&self) -> (usize, f64, f64, f64) {
        let count = self.reflections.len();
        if count == 0 {
            return (0, 0.0, 0.0, 0.0);
        }
        let avg_aware: f64 = self
            .reflections
            .iter()
            .map(|e| e.awareness_score)
            .sum::<f64>()
            / count as f64;
        let avg_gap: f64 = self
            .reflections
            .iter()
            .map(|e| (1.0 - e.coherence).max(0.0))
            .sum::<f64>()
            / count as f64;
        let stability = self.compute_stability();
        (count, avg_aware, avg_gap, stability)
    }

    fn compute_awareness(&self, thought: &[u8], coherence: f64) -> f64 {
        if thought.is_empty() {
            return 0.0;
        }
        let novelty = if self.reflections.is_empty() {
            0.5
        } else {
            let max_sim = self
                .reflections
                .iter()
                .map(|e| {
                    crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA::similarity(
                        &e.thought, thought,
                    )
                })
                .fold(0.0f64, f64::max);
            1.0 - max_sim
        };
        (coherence * 0.6 + novelty * 0.4).clamp(0.0, 1.0)
    }

    fn compute_vsa_coherence(&self, thought: &[u8]) -> f64 {
        if thought.is_empty() || self.reflections.is_empty() {
            return 0.5;
        }
        let recent: Vec<&[u8]> = self
            .reflections
            .iter()
            .rev()
            .take(5)
            .map(|e| e.thought.as_slice())
            .collect();
        if recent.is_empty() {
            return 0.5;
        }
        let sims: Vec<f64> = recent
            .iter()
            .map(|t| {
                crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA::similarity(thought, t)
            })
            .collect();
        let mean = sims.iter().sum::<f64>() / sims.len() as f64;
        let variance: f64 =
            sims.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / sims.len() as f64;
        (mean * 0.7 + (1.0 - variance.sqrt().min(1.0)) * 0.3).clamp(0.0, 1.0)
    }

    fn compute_stability(&self) -> f64 {
        if self.reflections.len() < 3 {
            return 0.5;
        }
        let scores: Vec<f64> = self.reflections.iter().map(|e| e.awareness_score).collect();
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let variance: f64 =
            scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / scores.len() as f64;
        (1.0 - variance.sqrt().min(1.0)).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

    #[test]
    fn test_empty_stats() {
        let unit = ReflexiveUnit::new(ReflexiveConfig::default());
        let (count, avg_aware, avg_gap, stability) = unit.stats();
        assert_eq!(count, 0);
        assert_eq!(avg_aware, 0.0);
        assert_eq!(avg_gap, 0.0);
        assert_eq!(stability, 0.0);
    }

    #[test]
    fn test_empty_self_awareness_score() {
        let unit = ReflexiveUnit::new(ReflexiveConfig::default());
        assert_eq!(unit.self_awareness_score(), 0.0);
    }

    #[test]
    fn test_single_reflection_returns_positive_awareness() {
        let mut unit = ReflexiveUnit::new(ReflexiveConfig::default());
        let thought = QuantizedVSA::random_vector();
        let (count, awareness) = unit.reflect(&thought);
        assert_eq!(count, 1);
        assert!(awareness > 0.0);
        assert!(awareness <= 1.0);
    }

    #[test]
    fn test_single_reflection_stats() {
        let mut unit = ReflexiveUnit::new(ReflexiveConfig::default());
        let thought = QuantizedVSA::random_vector();
        unit.reflect(&thought);
        let (count, avg_aware, avg_gap, _) = unit.stats();
        assert_eq!(count, 1);
        assert!(avg_aware > 0.0);
        assert!(avg_aware <= 1.0);
        assert!(avg_gap >= 0.0);
        assert!(avg_gap <= 1.0);
    }

    #[test]
    fn test_multiple_reflections_increase_count() {
        let mut unit = ReflexiveUnit::new(ReflexiveConfig::default());
        for i in 0..5 {
            let thought = QuantizedVSA::seeded_random(i, 4096);
            unit.reflect(&thought);
        }
        let (count, ..) = unit.stats();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_self_awareness_score_in_range() {
        let mut unit = ReflexiveUnit::new(ReflexiveConfig::default());
        for i in 0..15 {
            let thought = QuantizedVSA::seeded_random(i, 4096);
            unit.reflect(&thought);
        }
        let score = unit.self_awareness_score();
        assert!(score >= 0.0, "score should be >= 0, got {score}");
        assert!(score <= 1.0, "score should be <= 1, got {score}");
    }

    #[test]
    fn test_identical_thoughts_produce_high_coherence() {
        let mut unit = ReflexiveUnit::new(ReflexiveConfig::default());
        let thought = QuantizedVSA::seeded_random(42, 4096);
        for _ in 0..5 {
            unit.reflect(&thought);
        }
        let (_, avg_aware, avg_gap, _) = unit.stats();
        assert!(
            avg_aware > 0.5,
            "identical thoughts should yield avg_aware > 0.5, got {avg_aware}"
        );
        assert!(
            avg_gap < 0.3,
            "identical thoughts should yield low gap, got {avg_gap}"
        );
    }

    #[test]
    fn test_random_thoughts_produce_lower_awareness_than_identical() {
        let mut identical_unit = ReflexiveUnit::new(ReflexiveConfig::default());
        let thought = QuantizedVSA::seeded_random(42, 4096);
        for _ in 0..5 {
            identical_unit.reflect(&thought);
        }
        let (_, identical_aware, _, _) = identical_unit.stats();

        let mut random_unit = ReflexiveUnit::new(ReflexiveConfig::default());
        for i in 0..5 {
            let rnd = QuantizedVSA::seeded_random(i + 100, 4096);
            random_unit.reflect(&rnd);
        }
        let (_, random_aware, _, _) = random_unit.stats();

        assert!(
            identical_aware > random_aware,
            "identical thoughts ({identical_aware}) should yield higher awareness than random ({random_aware})"
        );
    }

    #[test]
    fn test_reflect_returns_current_count() {
        let mut unit = ReflexiveUnit::new(ReflexiveConfig::default());
        for i in 0..3 {
            let thought = QuantizedVSA::seeded_random(i, 4096);
            let (count, _) = unit.reflect(&thought);
            assert_eq!(count, i as usize + 1);
        }
    }

    #[test]
    fn test_empty_thought_awareness_is_low() {
        let mut unit = ReflexiveUnit::new(ReflexiveConfig::default());
        let (_, awareness) = unit.reflect(&[]);
        assert_eq!(awareness, 0.0);
    }

    #[test]
    fn test_default_config_values() {
        let config = ReflexiveConfig::default();
        assert_eq!(config.max_depth, 3);
        assert!((config.min_coherence - 0.5).abs() < 1e-9);
        assert_eq!(config.auto_reflect_interval.as_secs(), 300);
    }

    #[test]
    fn test_max_capacity_does_not_exceed_1000() {
        let mut unit = ReflexiveUnit::new(ReflexiveConfig::default());
        for i in 0..1100 {
            let thought = QuantizedVSA::seeded_random(i, 4096);
            unit.reflect(&thought);
        }
        assert_eq!(unit.reflections.len(), 1000);
    }

    #[test]
    fn test_stability_default_for_few_reflections() {
        let mut unit = ReflexiveUnit::new(ReflexiveConfig::default());
        let thought = QuantizedVSA::random_vector();
        unit.reflect(&thought);
        let (_, _, _, stability) = unit.stats();
        assert!((stability - 0.5).abs() < 1e-9);
    }
}
