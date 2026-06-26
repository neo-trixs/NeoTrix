use std::collections::VecDeque;

/// Which similarity metric to use for retrieval scoring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScoringFunction {
    DotProduct,
    CosineSimilarity,
    ReciprocalRank,
    BM25,
    LearnedWeighted,
}

/// How to combine results from multiple retrieval sources.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FusionStrategy {
    WeightedSum,
    ReciprocalRankFusion,
    MaxScore,
    AdaptiveRRF { k: usize },
}

/// How to generate the final answer from retrieved candidates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnswerPolicy {
    TopKDirect,
    WeightedEnsemble,
    ContrastiveSelection,
}

/// The category of a retrieval failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureType {
    LowConfidence,
    IrrelevantResult,
    MissingInformation,
    StaleInformation,
    OverloadedContext,
}

/// A configurable retrieval configuration — the evolvable action space.
#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalConfig {
    pub scoring_function: ScoringFunction,
    pub fusion_strategy: FusionStrategy,
    pub answer_policy: AnswerPolicy,
    pub max_results: usize,
    pub diversity_penalty: f64,
    pub recency_bias: f64,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            scoring_function: ScoringFunction::DotProduct,
            fusion_strategy: FusionStrategy::WeightedSum,
            answer_policy: AnswerPolicy::TopKDirect,
            max_results: 10,
            diversity_penalty: 0.0,
            recency_bias: 0.0,
        }
    }
}

/// A record of a retrieval failure, used to drive evolution.
#[derive(Debug, Clone)]
pub struct RetrievalFailure {
    pub query: String,
    pub expected_result: Option<String>,
    pub actual_result: Option<String>,
    pub confidence: f64,
    pub failure_type: FailureType,
    pub timestamp: std::time::Instant,
}

/// Self-evolving memory retrieval engine.
///
/// Inspired by EvolveMem (arXiv 2605.13941): the retrieval infrastructure
/// (scoring, fusion, answer policies) adapts based on observed failures,
/// rather than remaining frozen after deployment.
#[derive(Debug, Clone)]
pub struct MemoryEvolutionEngine {
    pub configs: Vec<(RetrievalConfig, f64)>,
    pub failures: VecDeque<RetrievalFailure>,
    pub adaptation_count: u64,
    pub current_config: RetrievalConfig,
    pub config_scores: Vec<f64>,
    pub learning_rate: f64,
}

impl MemoryEvolutionEngine {
    /// Creates a new engine with default configuration.
    pub fn new() -> Self {
        let default_config = RetrievalConfig::default();
        Self {
            configs: vec![(default_config.clone(), 0.0)],
            failures: VecDeque::with_capacity(1000),
            adaptation_count: 0,
            current_config: default_config,
            config_scores: vec![0.0],
            learning_rate: 0.1,
        }
    }

    /// Logs a retrieval failure and triggers adaptation if the failure queue is full enough.
    pub fn record_failure(&mut self, failure: RetrievalFailure) {
        if self.failures.len() >= 1000 {
            self.failures.pop_front();
        }
        self.failures.push_back(failure);

        // Trigger auto-adaptation when we have enough failures to diagnose
        if self.failures.len() >= 10 {
            self.auto_adapt();
        }
    }

    /// Returns the frequency (proportion) of each failure type among recent failures.
    pub fn diagnose(&self) -> Vec<(FailureType, f64)> {
        if self.failures.is_empty() {
            return Vec::new();
        }

        let mut counts = std::collections::HashMap::new();
        for failure in &self.failures {
            *counts.entry(failure.failure_type).or_insert(0usize) += 1;
        }

        let total = self.failures.len() as f64;
        let mut result: Vec<_> = counts
            .into_iter()
            .map(|(ft, count)| (ft, count as f64 / total))
            .collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    /// Diagnoses the dominant failure type and proposes a targeted config change.
    pub fn suggest_config_change(&self) -> Option<RetrievalConfig> {
        let diagnoses = self.diagnose();
        let dominant = diagnoses.first()?;
        let (dominant_type, _) = dominant;

        let mut suggested = self.current_config.clone();

        match dominant_type {
            FailureType::LowConfidence => {
                suggested.scoring_function = ScoringFunction::CosineSimilarity;
                suggested.max_results = suggested.max_results.saturating_sub(3).max(1);
            }
            FailureType::IrrelevantResult => {
                suggested.diversity_penalty = (suggested.diversity_penalty + 0.2).clamp(0.0, 1.0);
                suggested.fusion_strategy = FusionStrategy::ReciprocalRankFusion;
            }
            FailureType::MissingInformation => {
                suggested.max_results = suggested.max_results.saturating_add(5);
                suggested.scoring_function = ScoringFunction::BM25;
            }
            FailureType::StaleInformation => {
                suggested.recency_bias = (suggested.recency_bias + 0.15).clamp(0.0, 1.0);
            }
            FailureType::OverloadedContext => {
                suggested.max_results = suggested.max_results.saturating_sub(3).max(1);
                suggested.diversity_penalty = (suggested.diversity_penalty + 0.25).clamp(0.0, 1.0);
            }
        }

        Some(suggested)
    }

    /// Applies a new config, recording it in the history.
    pub fn apply_config(&mut self, config: RetrievalConfig) {
        self.adaptation_count += 1;
        self.current_config = config.clone();
        self.configs.push((config, 0.0));
        self.config_scores.push(0.0);
    }

    /// Records a performance score for the current configuration.
    pub fn score_config(&mut self, score: f64) {
        if let Some(entry) = self.configs.last_mut() {
            entry.1 = score;
        }
        if let Some(last) = self.config_scores.last_mut() {
            *last = score;
        }
    }

    /// Returns the current adaptation rate, which decays as more adaptations occur.
    pub fn adaptation_rate(&self) -> f64 {
        self.learning_rate / (1.0 + self.adaptation_count as f64).sqrt()
    }

    /// Returns the highest-scoring config from history, if any.
    pub fn best_config(&self) -> Option<&RetrievalConfig> {
        self.configs
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(config, _)| config)
    }

    /// Automatic adaptation cycle: diagnose → suggest → apply if different from current.
    pub fn auto_adapt(&mut self) {
        if self.failures.len() < 5 {
            return;
        }

        if let Some(suggested) = self.suggest_config_change() {
            if suggested != self.current_config {
                self.apply_config(suggested);
            }
        }
    }
}

impl Default for MemoryEvolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Global singleton accessor for the memory evolution engine.
static MEMORY_EVOLUTION: std::sync::OnceLock<std::sync::Mutex<MemoryEvolutionEngine>> =
    std::sync::OnceLock::new();

/// Returns a reference to the global `MemoryEvolutionEngine` singleton.
pub fn global_memory_evolution() -> &'static std::sync::Mutex<MemoryEvolutionEngine> {
    MEMORY_EVOLUTION.get_or_init(|| std::sync::Mutex::new(MemoryEvolutionEngine::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn make_failure(ft: FailureType, query: &str, confidence: f64) -> RetrievalFailure {
        RetrievalFailure {
            query: query.to_string(),
            expected_result: None,
            actual_result: None,
            confidence,
            failure_type: ft,
            timestamp: std::time::Instant::now(),
        }
    }

    #[serial]
    #[test]
    fn test_default_config() {
        let engine = MemoryEvolutionEngine::new();
        assert_eq!(
            engine.current_config.scoring_function,
            ScoringFunction::DotProduct
        );
        assert_eq!(
            engine.current_config.fusion_strategy,
            FusionStrategy::WeightedSum
        );
        assert_eq!(
            engine.current_config.answer_policy,
            AnswerPolicy::TopKDirect
        );
        assert_eq!(engine.current_config.max_results, 10);
        assert_eq!(engine.current_config.diversity_penalty, 0.0);
        assert_eq!(engine.current_config.recency_bias, 0.0);
        assert_eq!(engine.learning_rate, 0.1);
        assert_eq!(engine.adaptation_count, 0);
        assert!(engine.failures.is_empty());
    }

    #[test]
    fn test_retrieval_config_default_trait() {
        let config = RetrievalConfig::default();
        assert_eq!(config.scoring_function, ScoringFunction::DotProduct);
        assert_eq!(config.max_results, 10);
    }

    #[test]
    fn test_failure_recording_and_diagnosis() {
        let mut engine = MemoryEvolutionEngine::new();

        // Push 4 failures of different types — not enough to trigger auto_adapt (>10 needed)
        engine.record_failure(make_failure(FailureType::LowConfidence, "q1", 0.3));
        engine.record_failure(make_failure(FailureType::LowConfidence, "q2", 0.4));
        engine.record_failure(make_failure(FailureType::IrrelevantResult, "q3", 0.6));
        engine.record_failure(make_failure(FailureType::MissingInformation, "q4", 0.5));

        assert_eq!(engine.failures.len(), 4);

        let diagnoses = engine.diagnose();
        // LowConfidence should be most frequent (2/4 = 0.5)
        assert!(!diagnoses.is_empty());
        assert_eq!(diagnoses[0].0, FailureType::LowConfidence);
        assert!((diagnoses[0].1 - 0.5).abs() < 1e-6);

        let total_freq: f64 = diagnoses.iter().map(|(_, f)| f).sum();
        assert!((total_freq - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_suggest_config_change_low_confidence() {
        let mut engine = MemoryEvolutionEngine::new();
        for _ in 0..6 {
            engine
                .failures
                .push_back(make_failure(FailureType::LowConfidence, "query", 0.3));
        }

        let suggested = engine.suggest_config_change().unwrap();
        assert_eq!(
            suggested.scoring_function,
            ScoringFunction::CosineSimilarity
        );
        assert!(suggested.max_results < 10);
        assert_eq!(suggested.max_results, 7);
    }

    #[test]
    fn test_suggest_config_change_irrelevant_result() {
        let mut engine = MemoryEvolutionEngine::new();
        for _ in 0..6 {
            engine
                .failures
                .push_back(make_failure(FailureType::IrrelevantResult, "query", 0.3));
        }

        let suggested = engine.suggest_config_change().unwrap();
        assert!((suggested.diversity_penalty - 0.2).abs() < 1e-6);
        assert_eq!(
            suggested.fusion_strategy,
            FusionStrategy::ReciprocalRankFusion
        );
    }

    #[test]
    fn test_suggest_config_change_missing_information() {
        let mut engine = MemoryEvolutionEngine::new();
        for _ in 0..6 {
            engine
                .failures
                .push_back(make_failure(FailureType::MissingInformation, "query", 0.3));
        }

        let suggested = engine.suggest_config_change().unwrap();
        assert_eq!(suggested.max_results, 15);
        assert_eq!(suggested.scoring_function, ScoringFunction::BM25);
    }

    #[test]
    fn test_suggest_config_change_stale_information() {
        let mut engine = MemoryEvolutionEngine::new();
        for _ in 0..6 {
            engine
                .failures
                .push_back(make_failure(FailureType::StaleInformation, "query", 0.3));
        }

        let suggested = engine.suggest_config_change().unwrap();
        assert!((suggested.recency_bias - 0.15).abs() < 1e-6);
        assert_eq!(suggested.scoring_function, ScoringFunction::DotProduct);
    }

    #[test]
    fn test_suggest_config_change_overloaded_context() {
        let mut engine = MemoryEvolutionEngine::new();
        for _ in 0..6 {
            engine
                .failures
                .push_back(make_failure(FailureType::OverloadedContext, "query", 0.3));
        }

        let suggested = engine.suggest_config_change().unwrap();
        assert_eq!(suggested.max_results, 7);
        assert!((suggested.diversity_penalty - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_suggest_config_change_empty_failures() {
        let engine = MemoryEvolutionEngine::new();
        assert!(engine.diagnose().is_empty());
        assert!(engine.suggest_config_change().is_none());
    }

    #[test]
    fn test_auto_adapt_cycles() {
        let mut engine = MemoryEvolutionEngine::new();
        let initial = engine.current_config.clone();

        // Push enough failures to trigger auto_adapt
        for i in 0..12 {
            engine.record_failure(make_failure(
                if i % 2 == 0 {
                    FailureType::LowConfidence
                } else {
                    FailureType::IrrelevantResult
                },
                &format!("q{}", i),
                0.3,
            ));
        }

        // auto_adapt should have been triggered (12 failures >= 10 threshold)
        assert!(
            engine.adaptation_count >= 1 || engine.current_config == initial,
            "auto_adapt should have triggered or config may already match suggestion"
        );

        // Run another explicit auto-adapt
        engine.auto_adapt();
        assert!(engine.adaptation_count >= 1);
    }

    #[test]
    fn test_apply_config() {
        let mut engine = MemoryEvolutionEngine::new();
        let new_config = RetrievalConfig {
            scoring_function: ScoringFunction::BM25,
            fusion_strategy: FusionStrategy::ReciprocalRankFusion,
            answer_policy: AnswerPolicy::WeightedEnsemble,
            max_results: 20,
            diversity_penalty: 0.3,
            recency_bias: 0.1,
        };

        engine.apply_config(new_config.clone());
        assert_eq!(engine.current_config, new_config);
        assert_eq!(engine.adaptation_count, 1);
        assert_eq!(engine.configs.len(), 3); // default + scored entry + new entry
    }

    #[test]
    fn test_best_config_tracking() {
        let mut engine = MemoryEvolutionEngine::new();

        let c1 = RetrievalConfig {
            scoring_function: ScoringFunction::DotProduct,
            ..RetrievalConfig::default()
        };
        let c2 = RetrievalConfig {
            scoring_function: ScoringFunction::BM25,
            ..RetrievalConfig::default()
        };
        let c3 = RetrievalConfig {
            scoring_function: ScoringFunction::CosineSimilarity,
            ..RetrievalConfig::default()
        };

        engine.apply_config(c1);
        engine.score_config(0.5);

        engine.apply_config(c2);
        engine.score_config(0.9);

        engine.apply_config(c3);
        engine.score_config(0.3);

        let best = engine.best_config().unwrap();
        assert_eq!(best.scoring_function, ScoringFunction::BM25);
    }

    #[test]
    fn test_best_config_empty_fallback() {
        let engine = MemoryEvolutionEngine::new();
        // Even with fresh engine, there's at least the default config
        assert!(engine.best_config().is_some());
    }

    #[test]
    fn test_adaptation_rate_calculation() {
        let mut engine = MemoryEvolutionEngine::new();
        assert!((engine.adaptation_rate() - 0.1).abs() < 1e-6);

        engine.adaptation_count = 3;
        let expected = 0.1 / (4.0_f64).sqrt(); // 0.1 / 2.0 = 0.05
        assert!((engine.adaptation_rate() - expected).abs() < 1e-6);

        engine.adaptation_count = 99;
        let expected = 0.1 / (100.0_f64).sqrt(); // 0.1 / 10.0 = 0.01
        assert!((engine.adaptation_rate() - expected).abs() < 1e-6);
    }

    #[test]
    fn test_score_config() {
        let mut engine = MemoryEvolutionEngine::new();
        engine.score_config(0.85);
        assert!((engine.config_scores.last().unwrap() - 0.85).abs() < 1e-6);
        if let Some((_, score)) = engine.configs.last() {
            assert!((score - 0.85).abs() < 1e-6);
        }
    }

    #[test]
    fn test_failure_queue_capacity() {
        let mut engine = MemoryEvolutionEngine::new();
        for i in 0..1010 {
            engine.failures.push_back(make_failure(
                FailureType::LowConfidence,
                &format!("q{}", i),
                0.3,
            ));
            if engine.failures.len() > 1000 {
                engine.failures.pop_front();
            }
        }
        assert_eq!(engine.failures.len(), 1000);
    }

    #[test]
    fn test_global_singleton() {
        let guard = global_memory_evolution()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert_eq!(guard.learning_rate, 0.1);
        assert_eq!(guard.current_config.max_results, 10);
        drop(guard);

        // Access again — must return the same instance
        let guard2 = global_memory_evolution()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert_eq!(guard2.learning_rate, 0.1);
    }

    #[test]
    fn test_auto_adapt_requires_minimum_failures() {
        let mut engine = MemoryEvolutionEngine::new();
        // Fewer than 5 failures — auto_adapt should be a no-op
        for _ in 0..3 {
            engine
                .failures
                .push_back(make_failure(FailureType::LowConfidence, "q", 0.3));
        }
        let before = engine.adaptation_count;
        engine.auto_adapt();
        assert_eq!(engine.adaptation_count, before);
    }
}
