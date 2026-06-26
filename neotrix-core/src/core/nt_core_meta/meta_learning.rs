use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const META_LEARNING_HISTORY_SIZE: usize = 50;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompressionStats {
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub compression_ratio: f64,
    pub semantic_retention: f64,
    pub abstraction_level: f64,
    pub lossiness: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsolidationOutcome {
    pub sequences_replayed: usize,
    pub patterns_merged: usize,
    pub abstractions_formed: usize,
    pub novelty_score: f64,
    pub coherence_gain: f64,
    pub cross_session_consolidated: usize,
    pub buffer_eviction_count: u64,
    pub buffer_avg_utility: f64,
    pub hebbian_distillations: u64,
    pub scm_nrem_merged: u64,
    pub scm_rem_associations: u64,
    pub triggered_by_recurrence: bool,
    pub learned_retrieval_rate: f64,
    pub compression: Option<CompressionStats>,
    pub consolidation_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaLearningParams {
    pub consolidation_learning_rate: f64,
    pub filter_threshold: f64,
    pub prune_retain_fraction: f64,
    pub replay_reward_threshold: f64,
    pub link_min_similarity: f64,
    pub integrate_confidence_min: f64,
    pub target_compression_ratio: f64,
    pub min_semantic_retention: f64,
    pub abstraction_bias: f64,
    pub lossy_tolerance: f64,
}

impl Default for MetaLearningParams {
    fn default() -> Self {
        Self {
            consolidation_learning_rate: 0.1,
            filter_threshold: 0.92,
            prune_retain_fraction: 0.6,
            replay_reward_threshold: 0.7,
            link_min_similarity: 0.6,
            integrate_confidence_min: 0.5,
            target_compression_ratio: 0.3,
            min_semantic_retention: 0.7,
            abstraction_bias: 0.5,
            lossy_tolerance: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaLearningSignal {
    pub cycle: u64,
    pub outcome: ConsolidationOutcome,
    pub params_before: MetaLearningParams,
    pub params_after: MetaLearningParams,
    pub param_deltas: Vec<(String, f64)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaLearning {
    pub params: MetaLearningParams,
    pub history: VecDeque<MetaLearningSignal>,
    pub cycle: u64,
    pub params_stability: f64,
}

impl MetaLearning {
    pub fn new() -> Self {
        Self {
            params: MetaLearningParams::default(),
            history: VecDeque::with_capacity(META_LEARNING_HISTORY_SIZE),
            cycle: 0,
            params_stability: 1.0,
        }
    }

    pub fn optimize_compression(&mut self, outcome: &ConsolidationOutcome) -> Vec<(String, f64)> {
        let mut deltas = Vec::new();
        let Some(ref comp) = outcome.compression else {
            return deltas;
        };
        let rate = self.params.consolidation_learning_rate;

        if comp.compression_ratio > self.params.target_compression_ratio
            && comp.semantic_retention >= self.params.min_semantic_retention
        {
            let delta = rate * 0.1;
            self.params.target_compression_ratio =
                (self.params.target_compression_ratio - delta).max(0.05);
            deltas.push(("target_compression_ratio".to_string(), -delta));
        }

        if comp.semantic_retention < self.params.min_semantic_retention {
            let delta = rate * 0.15;
            self.params.target_compression_ratio =
                (self.params.target_compression_ratio + delta).min(0.95);
            deltas.push(("target_compression_ratio".to_string(), delta));
        }

        if comp.abstraction_level < 0.3 && outcome.coherence_gain > 0.3 {
            let delta = rate * 0.1;
            self.params.abstraction_bias = (self.params.abstraction_bias + delta).min(1.0);
            deltas.push(("abstraction_bias".to_string(), delta));
        }

        if comp.lossiness > self.params.lossy_tolerance {
            let delta = rate * 0.12;
            self.params.lossy_tolerance = (self.params.lossy_tolerance - delta).max(0.01);
            deltas.push(("lossy_tolerance".to_string(), -delta));
        }

        deltas
    }

    pub fn consolidation_quality_score(&self) -> f64 {
        let n = self.history.len();
        if n == 0 {
            return 0.5;
        }
        let mut total_compression_efficiency = 0.0;
        let mut total_semantic_retention = 0.0;
        let mut total_coherence_gain = 0.0;
        let mut comp_count = 0;

        for signal in &self.history {
            total_coherence_gain += signal.outcome.coherence_gain;
            if let Some(ref comp) = signal.outcome.compression {
                let ratio = comp.compression_ratio.clamp(0.01, 1.0);
                let efficiency = (1.0 / ratio).min(10.0) / 10.0;
                total_compression_efficiency += efficiency;
                total_semantic_retention += comp.semantic_retention;
                comp_count += 1;
            }
        }

        let avg_coherence = total_coherence_gain / n as f64;
        let avg_semantic = if comp_count > 0 {
            total_semantic_retention / comp_count as f64
        } else {
            0.5
        };
        let avg_efficiency = if comp_count > 0 {
            total_compression_efficiency / comp_count as f64
        } else {
            0.3
        };

        avg_efficiency * 0.3 + avg_semantic * 0.4 + avg_coherence.min(1.0) * 0.3
    }

    pub fn ingest_consolidation(&mut self, outcome: ConsolidationOutcome) -> MetaLearningSignal {
        self.cycle += 1;
        let params_before = self.params.clone();
        let mut deltas = Vec::new();

        let abstraction_efficiency = if outcome.patterns_merged > 0 {
            outcome.abstractions_formed as f64 / outcome.patterns_merged as f64
        } else {
            0.0
        };

        let coherence_return = outcome.coherence_gain;

        let retrieval_success = outcome.learned_retrieval_rate;
        let eviction_pressure = if outcome.buffer_eviction_count > 0 {
            outcome.buffer_eviction_count as f64 / (self.cycle.max(1) as f64)
        } else {
            0.0
        };

        if abstraction_efficiency < 0.2 && coherence_return < 0.1 {
            let delta = self.params.consolidation_learning_rate * 0.1;
            self.params.filter_threshold = (self.params.filter_threshold - delta).max(0.8);
            deltas.push(("filter_threshold".to_string(), -delta));
        }

        if abstraction_efficiency > 0.6 && coherence_return > 0.3 {
            let delta = self.params.consolidation_learning_rate * 0.05;
            self.params.filter_threshold = (self.params.filter_threshold + delta).min(0.98);
            deltas.push(("filter_threshold".to_string(), delta));
        }

        if retrieval_success < 0.3 {
            let delta = self.params.consolidation_learning_rate * 0.15;
            self.params.integrate_confidence_min =
                (self.params.integrate_confidence_min - delta).max(0.2);
            deltas.push(("integrate_confidence_min".to_string(), -delta));
        }

        if retrieval_success > 0.7 && coherence_return > 0.2 {
            let delta = self.params.consolidation_learning_rate * 0.05;
            self.params.integrate_confidence_min =
                (self.params.integrate_confidence_min + delta).min(0.7);
            deltas.push(("integrate_confidence_min".to_string(), delta));
        }

        if eviction_pressure > 0.1 {
            let delta = self.params.consolidation_learning_rate * 0.1;
            self.params.prune_retain_fraction =
                (self.params.prune_retain_fraction + delta).min(0.9);
            deltas.push(("prune_retain_fraction".to_string(), delta));
        }

        if outcome.triggered_by_recurrence && outcome.coherence_gain < 0.15 {
            let delta = self.params.consolidation_learning_rate * 0.2;
            self.params.replay_reward_threshold =
                (self.params.replay_reward_threshold - delta).max(0.3);
            deltas.push(("replay_reward_threshold".to_string(), -delta));
        }

        if outcome.cross_session_consolidated > 5 && outcome.coherence_gain > 0.25 {
            let delta = self.params.consolidation_learning_rate * 0.1;
            self.params.link_min_similarity = (self.params.link_min_similarity + delta).min(0.85);
            deltas.push(("link_min_similarity".to_string(), delta));
        }

        let compression_deltas = self.optimize_compression(&outcome);
        deltas.extend(compression_deltas);

        let total_delta: f64 = deltas.iter().map(|d| d.1.abs()).sum();
        self.params_stability =
            (1.0 - total_delta.clamp(0.0, 1.0)) * 0.3 + self.params_stability * 0.7;

        let signal = MetaLearningSignal {
            cycle: self.cycle,
            outcome,
            params_before,
            params_after: self.params.clone(),
            param_deltas: deltas,
        };

        self.history.push_back(signal.clone());
        if self.history.len() > META_LEARNING_HISTORY_SIZE {
            self.history.pop_front();
        }

        signal
    }

    pub fn to_kleos_adjustments(&self) -> KleosAdjustments {
        KleosAdjustments {
            filter_threshold: self.params.filter_threshold,
            replay_reward_threshold: self.params.replay_reward_threshold,
            link_min_similarity: self.params.link_min_similarity,
            abstract_cluster_count: ((self.params.prune_retain_fraction * 12.0) as usize).max(4),
            integrate_confidence_min: self.params.integrate_confidence_min,
            prune_retain_fraction: self.params.prune_retain_fraction,
        }
    }
}

impl Default for MetaLearning {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KleosAdjustments {
    pub filter_threshold: f64,
    pub replay_reward_threshold: f64,
    pub link_min_similarity: f64,
    pub abstract_cluster_count: usize,
    pub integrate_confidence_min: f64,
    pub prune_retain_fraction: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_outcome() -> ConsolidationOutcome {
        ConsolidationOutcome {
            sequences_replayed: 10,
            patterns_merged: 20,
            abstractions_formed: 3,
            novelty_score: 0.4,
            coherence_gain: 0.15,
            cross_session_consolidated: 2,
            buffer_eviction_count: 5,
            buffer_avg_utility: 0.6,
            hebbian_distillations: 3,
            scm_nrem_merged: 8,
            scm_rem_associations: 4,
            triggered_by_recurrence: true,
            learned_retrieval_rate: 0.35,
            compression: None,
            consolidation_duration_ms: 0,
        }
    }

    #[test]
    fn test_new_creates_default_params() {
        let ml = MetaLearning::new();
        assert_eq!(ml.cycle, 0);
        assert!(ml.history.is_empty());
        assert_eq!(ml.params.prune_retain_fraction, 0.6);
    }

    #[test]
    fn test_ingest_first_consolidation() {
        let mut ml = MetaLearning::new();
        let signal = ml.ingest_consolidation(sample_outcome());
        assert_eq!(ml.cycle, 1);
        assert_eq!(signal.cycle, 1);
        assert_eq!(ml.history.len(), 1);
    }

    #[test]
    fn test_params_adapt_on_low_efficiency() {
        let mut ml = MetaLearning::new();
        let mut outcome = sample_outcome();
        outcome.abstractions_formed = 1;
        outcome.patterns_merged = 50;
        outcome.coherence_gain = 0.05;
        let signal = ml.ingest_consolidation(outcome);
        let has_filter_delta = signal
            .param_deltas
            .iter()
            .any(|(name, _)| name == "filter_threshold");
        assert!(has_filter_delta);
        assert!(ml.params.filter_threshold < 0.92);
    }

    #[test]
    fn test_params_adapt_on_high_efficiency() {
        let mut ml = MetaLearning::new();
        let mut outcome = sample_outcome();
        outcome.abstractions_formed = 30;
        outcome.patterns_merged = 40;
        outcome.coherence_gain = 0.4;
        let signal = ml.ingest_consolidation(outcome);
        let has_filter_delta = signal
            .param_deltas
            .iter()
            .any(|(name, _)| name == "filter_threshold");
        assert!(has_filter_delta);
        assert!(ml.params.filter_threshold > 0.92);
    }

    #[test]
    fn test_params_adapt_on_low_retrieval() {
        let mut ml = MetaLearning::new();
        let mut outcome = sample_outcome();
        outcome.learned_retrieval_rate = 0.2;
        ml.ingest_consolidation(outcome);
        assert!(ml.params.integrate_confidence_min < 0.5);
    }

    #[test]
    fn test_high_eviction_increases_retain_fraction() {
        let mut ml = MetaLearning::new();
        let mut outcome = sample_outcome();
        outcome.buffer_eviction_count = 20;
        ml.ingest_consolidation(outcome);
        assert!(ml.params.prune_retain_fraction > 0.6);
    }

    #[test]
    fn test_to_kleos_adjustments_maps_fields() {
        let ml = MetaLearning::new();
        let adj = ml.to_kleos_adjustments();
        assert_eq!(adj.filter_threshold, ml.params.filter_threshold);
        assert_eq!(
            adj.replay_reward_threshold,
            ml.params.replay_reward_threshold
        );
        assert_eq!(adj.link_min_similarity, ml.params.link_min_similarity);
        assert!(adj.abstract_cluster_count >= 4);
    }

    #[test]
    fn test_history_capped() {
        let mut ml = MetaLearning::new();
        for _ in 0..META_LEARNING_HISTORY_SIZE + 10 {
            ml.ingest_consolidation(sample_outcome());
        }
        assert!(ml.history.len() <= META_LEARNING_HISTORY_SIZE);
    }

    #[test]
    fn test_default_implemented() {
        let ml = MetaLearning::default();
        assert_eq!(ml.cycle, 0);
    }

    #[test]
    fn test_default_compression_params() {
        let ml = MetaLearning::new();
        assert_eq!(ml.params.target_compression_ratio, 0.3);
        assert_eq!(ml.params.min_semantic_retention, 0.7);
        assert_eq!(ml.params.abstraction_bias, 0.5);
        assert_eq!(ml.params.lossy_tolerance, 0.3);
    }

    #[test]
    fn test_optimize_compression_high_ratio() {
        let mut ml = MetaLearning::new();
        let outcome = ConsolidationOutcome {
            compression: Some(CompressionStats {
                input_bytes: 1000,
                output_bytes: 800,
                compression_ratio: 0.8,
                semantic_retention: 0.9,
                abstraction_level: 0.4,
                lossiness: 0.1,
            }),
            ..sample_outcome()
        };
        let deltas = ml.optimize_compression(&outcome);
        assert!(deltas
            .iter()
            .any(|(name, d)| name == "target_compression_ratio" && *d < 0.0));
        assert!(ml.params.target_compression_ratio < 0.3);
    }

    #[test]
    fn test_optimize_compression_low_retention() {
        let mut ml = MetaLearning::new();
        let outcome = ConsolidationOutcome {
            compression: Some(CompressionStats {
                input_bytes: 1000,
                output_bytes: 100,
                compression_ratio: 0.1,
                semantic_retention: 0.4,
                abstraction_level: 0.8,
                lossiness: 0.6,
            }),
            ..sample_outcome()
        };
        let deltas = ml.optimize_compression(&outcome);
        assert!(deltas
            .iter()
            .any(|(name, d)| name == "target_compression_ratio" && *d > 0.0));
        assert!(ml.params.target_compression_ratio > 0.3);
    }

    #[test]
    fn test_consolidation_quality_score_default() {
        let ml = MetaLearning::new();
        let score = ml.consolidation_quality_score();
        assert_eq!(score, 0.5);
    }

    #[test]
    fn test_compression_stats_filled() {
        let mut ml = MetaLearning::new();
        let outcome = ConsolidationOutcome {
            compression: Some(CompressionStats {
                input_bytes: 2000,
                output_bytes: 400,
                compression_ratio: 0.2,
                semantic_retention: 0.85,
                abstraction_level: 0.6,
                lossiness: 0.15,
            }),
            consolidation_duration_ms: 150,
            ..sample_outcome()
        };
        ml.ingest_consolidation(outcome);
        let stats = ml
            .history
            .back()
            .unwrap()
            .outcome
            .compression
            .as_ref()
            .unwrap();
        assert_eq!(stats.input_bytes, 2000);
        assert_eq!(stats.output_bytes, 400);
        assert!(stats.compression_ratio - 0.2 < 1e-10);
        assert!((stats.semantic_retention - 0.85) < 1e-10);
        assert!((stats.abstraction_level - 0.6) < 1e-10);
        assert!((stats.lossiness - 0.15) < 1e-10);
    }

    #[test]
    fn test_outcome_with_compression() {
        let mut ml = MetaLearning::new();
        let outcome = ConsolidationOutcome {
            compression: Some(CompressionStats {
                input_bytes: 5000,
                output_bytes: 1500,
                compression_ratio: 0.3,
                semantic_retention: 0.8,
                abstraction_level: 0.5,
                lossiness: 0.2,
            }),
            consolidation_duration_ms: 200,
            coherence_gain: 0.4,
            ..sample_outcome()
        };
        let signal = ml.ingest_consolidation(outcome);
        assert!(signal.outcome.compression.is_some());
        assert_eq!(signal.outcome.consolidation_duration_ms, 200);
        assert!(signal
            .param_deltas
            .iter()
            .any(|(name, _)| name == "target_compression_ratio"));
    }

    #[test]
    fn test_quality_score_with_compression_history() {
        let mut ml = MetaLearning::new();
        for i in 0..3 {
            let outcome = ConsolidationOutcome {
                compression: Some(CompressionStats {
                    input_bytes: 1000,
                    output_bytes: (300 + i * 100) as u64,
                    compression_ratio: 0.3 + (i as f64 * 0.1),
                    semantic_retention: 0.75 + (i as f64 * 0.05),
                    abstraction_level: 0.5,
                    lossiness: 0.2,
                }),
                coherence_gain: 0.2 + (i as f64 * 0.1),
                ..sample_outcome()
            };
            ml.ingest_consolidation(outcome);
        }
        let score = ml.consolidation_quality_score();
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }
}
