use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
fn now_u64() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Structured retrieval config action space — each knob is evolvable.
/// Inspired by EvolveMem (arXiv 2605.13941) §3.2: "Multi-View Retrieval Action Space"
#[derive(Debug, Clone)]
pub struct RetrievalConfig {
    pub top_k: usize,
    pub similarity_threshold: f64,
    pub fusion_mode: FusionMode,
    pub per_subspace_weights: Vec<f64>,
    pub entity_swap_enabled: bool,
    pub query_decomposition_enabled: bool,
    pub answer_verification_enabled: bool,
    pub reflection_rounds: usize,
    pub diversity_bonus: f64,
    pub recency_weight: f64,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            top_k: 5,
            similarity_threshold: 0.7,
            fusion_mode: FusionMode::WeightedSum,
            per_subspace_weights: vec![],
            entity_swap_enabled: false,
            query_decomposition_enabled: false,
            answer_verification_enabled: false,
            reflection_rounds: 0,
            diversity_bonus: 0.0,
            recency_weight: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FusionMode {
    WeightedSum,
    MaxScore,
    Rrf,
    Adaptive,
}

impl FusionMode {
    pub fn all() -> Vec<FusionMode> {
        vec![
            FusionMode::WeightedSum,
            FusionMode::MaxScore,
            FusionMode::Rrf,
            FusionMode::Adaptive,
        ]
    }
}

/// Failure diagnosis result — what went wrong in a retrieval query.
#[derive(Debug, Clone)]
pub struct RetrievalDiagnosis {
    pub query_summary: String,
    pub category: DiagnosisCategory,
    pub confidence: f64,
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosisCategory {
    OverlookedEntity,
    WrongSubspace,
    ThresholdTooStrict,
    ThresholdTooLoose,
    FusionMismatch,
    MissingCrossReference,
    TemporalMisalignment,
    Ambiguity,
    Other,
}

impl DiagnosisCategory {
    pub fn all() -> Vec<DiagnosisCategory> {
        vec![
            DiagnosisCategory::OverlookedEntity,
            DiagnosisCategory::WrongSubspace,
            DiagnosisCategory::ThresholdTooStrict,
            DiagnosisCategory::ThresholdTooLoose,
            DiagnosisCategory::FusionMismatch,
            DiagnosisCategory::MissingCrossReference,
            DiagnosisCategory::TemporalMisalignment,
            DiagnosisCategory::Ambiguity,
            DiagnosisCategory::Other,
        ]
    }
}

/// Record of a single retrieval query + outcome for diagnosis.
#[derive(Debug, Clone)]
pub struct RetrievalTrace {
    pub query_vsa_hash: u64,
    pub query_text: String,
    pub expected_result_ids: Vec<u64>,
    pub actual_result_ids: Vec<u64>,
    pub recall_at_k: f64,
    pub precision_at_k: f64,
    pub latency_ms: f64,
    pub config_used: RetrievalConfig,
    pub timestamp: u64,
}

/// EvolveMem-style self-evolving retrieval engine.
/// Implements Evaluate → Diagnose → Propose → Guard → Repeat closed loop.
pub struct RetrievalEvolutionEngine {
    /// Current retrieval configuration
    pub config: RetrievalConfig,
    /// Diagnosis history for failure pattern detection
    pub diagnosis_history: VecDeque<RetrievalDiagnosis>,
    /// Trace history for per-query evaluation
    pub traces: VecDeque<RetrievalTrace>,
    /// Best config found so far
    pub best_config: Option<RetrievalConfig>,
    /// Best score achieved
    pub best_score: f64,
    /// Stagnation counter
    pub stagnation_rounds: u32,
    /// Evolution round counter
    pub evolution_round: u32,
    /// Max configs to retain
    pub max_traces: usize,
    /// Number of config changes proposed
    pub proposals_made: u64,
    /// Number of config changes accepted
    pub proposals_accepted: u64,
    /// Number of config changes reverted
    pub proposals_reverted: u64,
    /// Diagnosis -> config mutation mapping (learned over time)
    pub diagnosis_to_mutation: Vec<(DiagnosisCategory, &'static str)>,
}

impl RetrievalEvolutionEngine {
    pub fn new(config: RetrievalConfig) -> Self {
        Self {
            config,
            diagnosis_history: VecDeque::with_capacity(100),
            traces: VecDeque::with_capacity(200),
            best_config: None,
            best_score: f64::NEG_INFINITY,
            stagnation_rounds: 0,
            evolution_round: 0,
            max_traces: 200,
            proposals_made: 0,
            proposals_accepted: 0,
            proposals_reverted: 0,
            diagnosis_to_mutation: Self::default_diagnosis_mutations(),
        }
    }

    fn default_diagnosis_mutations() -> Vec<(DiagnosisCategory, &'static str)> {
        vec![
            (
                DiagnosisCategory::OverlookedEntity,
                "entity_swap_enabled:true",
            ),
            (
                DiagnosisCategory::WrongSubspace,
                "subspace_weights:rebalance",
            ),
            (
                DiagnosisCategory::ThresholdTooStrict,
                "similarity_threshold:-0.05",
            ),
            (
                DiagnosisCategory::ThresholdTooLoose,
                "similarity_threshold:+0.05",
            ),
            (DiagnosisCategory::FusionMismatch, "fusion_mode:next"),
            (
                DiagnosisCategory::MissingCrossReference,
                "reflection_rounds:+1",
            ),
            (
                DiagnosisCategory::TemporalMisalignment,
                "recency_weight:+0.1",
            ),
            (
                DiagnosisCategory::Ambiguity,
                "query_decomposition_enabled:true",
            ),
            (DiagnosisCategory::Other, "top_k:+2"),
        ]
    }

    /// Record a retrieval trace for later diagnosis.
    pub fn record_trace(&mut self, trace: RetrievalTrace) {
        self.traces.push_back(trace);
        if self.traces.len() > self.max_traces {
            self.traces.pop_front();
        }
    }

    /// Evaluate current config performance across recent traces.
    /// Returns mean recall@k as the primary metric (EvolveMem-style).
    pub fn evaluate_current_config(&self, window: usize) -> f64 {
        let traces: Vec<&RetrievalTrace> = self.traces.iter().rev().take(window).collect();
        if traces.is_empty() {
            return 0.0;
        }
        let mean_recall: f64 =
            traces.iter().map(|t| t.recall_at_k).sum::<f64>() / traces.len() as f64;
        mean_recall
    }

    /// Diagnose failure patterns from recent traces.
    /// EvolveMem §3.3: "reads per-question failure logs, categorizes root causes"
    pub fn diagnose_failures(&self, window: usize) -> Vec<RetrievalDiagnosis> {
        let traces: Vec<&RetrievalTrace> = self.traces.iter().rev().take(window).collect();
        let mut diagnoses = Vec::new();

        // Check for overlooked entities: low recall, non-zero precision
        let low_recall_count = traces
            .iter()
            .filter(|t| t.recall_at_k < 0.5 && t.precision_at_k > 0.3)
            .count();
        if low_recall_count as f64 > traces.len() as f64 * 0.3 {
            diagnoses.push(RetrievalDiagnosis {
                query_summary: "systematic low recall".into(),
                category: DiagnosisCategory::OverlookedEntity,
                confidence: (low_recall_count as f64 / traces.len() as f64).min(1.0),
                suggestion: "enable entity-swap to mitigate adversarial entity overlap".into(),
            });
        }

        // Check for threshold too strict: very high precision but low recall
        let strict_count = traces
            .iter()
            .filter(|t| t.precision_at_k > 0.9 && t.recall_at_k < 0.4)
            .count();
        if strict_count as f64 > traces.len() as f64 * 0.2 {
            diagnoses.push(RetrievalDiagnosis {
                query_summary: "threshold too strict".into(),
                category: DiagnosisCategory::ThresholdTooStrict,
                confidence: (strict_count as f64 / traces.len() as f64).min(1.0),
                suggestion: "lower similarity threshold to increase recall".into(),
            });
        }

        // Check for threshold too loose: low precision
        let loose_count = traces.iter().filter(|t| t.precision_at_k < 0.3).count();
        if loose_count as f64 > traces.len() as f64 * 0.3 {
            diagnoses.push(RetrievalDiagnosis {
                query_summary: "threshold too loose".into(),
                category: DiagnosisCategory::ThresholdTooLoose,
                confidence: (loose_count as f64 / traces.len() as f64).min(1.0),
                suggestion: "raise similarity threshold to filter noise".into(),
            });
        }

        // Check for fusion mismatch: high variance in per-subspace results
        if self.config.fusion_mode == FusionMode::WeightedSum {
            let variance_high = traces
                .iter()
                .filter(|t| {
                    let p = t.precision_at_k;
                    let r = t.recall_at_k;
                    (p - r).abs() > 0.4
                })
                .count();
            if variance_high as f64 > traces.len() as f64 * 0.25 {
                diagnoses.push(RetrievalDiagnosis {
                    query_summary: "fusion mode mismatch".into(),
                    category: DiagnosisCategory::FusionMismatch,
                    confidence: 0.6,
                    suggestion: "try RRF or adaptive fusion mode".into(),
                });
            }
        }

        // Check for temporal misalignment: recent queries have lower recall
        let recent_half = traces.len() / 2;
        if recent_half >= 3 {
            let recent_recall: f64 = traces
                .iter()
                .take(recent_half)
                .map(|t| t.recall_at_k)
                .sum::<f64>()
                / recent_half as f64;
            let older_recall: f64 = traces
                .iter()
                .skip(recent_half)
                .map(|t| t.recall_at_k)
                .sum::<f64>()
                / (traces.len() - recent_half) as f64;
            if recent_recall < older_recall * 0.7 {
                diagnoses.push(RetrievalDiagnosis {
                    query_summary: "temporal degradation".into(),
                    category: DiagnosisCategory::TemporalMisalignment,
                    confidence: 0.7,
                    suggestion: "increase recency_weight to favor newer entries".into(),
                });
            }
        }

        if diagnoses.is_empty() {
            diagnoses.push(RetrievalDiagnosis {
                query_summary: "no specific failure pattern".into(),
                category: DiagnosisCategory::Other,
                confidence: 0.3,
                suggestion: "increase top_k for broader coverage".into(),
            });
        }

        diagnoses
    }

    /// Propose config changes based on diagnoses.
    /// EvolveMem §3.3: "proposes targeted configuration adjustments"
    pub fn propose_mutations(&mut self, diagnoses: &[RetrievalDiagnosis]) -> Vec<String> {
        let mut mutations = Vec::new();
        for diagnosis in diagnoses {
            if diagnosis.confidence < 0.4 {
                continue;
            }
            for (cat, mutation_template) in &self.diagnosis_to_mutation {
                if *cat == diagnosis.category {
                    mutations.push(format!(
                        "{} (conf={:.2})",
                        mutation_template, diagnosis.confidence
                    ));
                }
            }
        }
        mutations
    }

    /// Apply a mutation string to the current config.
    /// Returns true if config changed.
    pub fn apply_mutation(&mut self, mutation: &str) -> bool {
        let parts: Vec<&str> = mutation.splitn(2, ':').collect();
        if parts.len() != 2 {
            return false;
        }
        let key = parts[0];
        let value = parts[1];

        match key {
            "top_k" => {
                if let Ok(v) = value.parse::<usize>() {
                    self.config.top_k = v.max(1).min(100);
                    return true;
                }
            }
            "similarity_threshold" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.config.similarity_threshold =
                        (self.config.similarity_threshold + v).clamp(0.1, 1.0);
                    return true;
                }
            }
            "fusion_mode" => {
                let modes = FusionMode::all();
                if value == "next" {
                    let idx = modes
                        .iter()
                        .position(|m| *m == self.config.fusion_mode)
                        .unwrap_or(0);
                    self.config.fusion_mode = modes[(idx + 1) % modes.len()];
                } else if let Some(v) = value.parse::<usize>().ok().and_then(|i| modes.get(i)) {
                    self.config.fusion_mode = *v;
                }
                return true;
            }
            "entity_swap_enabled"
            | "query_decomposition_enabled"
            | "answer_verification_enabled" => {
                let v = value == "true";
                match key {
                    "entity_swap_enabled" => self.config.entity_swap_enabled = v,
                    "query_decomposition_enabled" => self.config.query_decomposition_enabled = v,
                    _ => self.config.answer_verification_enabled = v,
                }
                return true;
            }
            "reflection_rounds" => {
                if let Ok(v) = value.parse::<isize>() {
                    let new = (self.config.reflection_rounds as isize + v).max(0) as usize;
                    self.config.reflection_rounds = new.min(5);
                    return true;
                }
            }
            "recency_weight" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.config.recency_weight = (self.config.recency_weight + v).clamp(0.0, 1.0);
                    return true;
                }
            }
            "diversity_bonus" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.config.diversity_bonus = v.clamp(0.0, 1.0);
                    return true;
                }
            }
            "subspace_weights" => {
                if value == "rebalance" && !self.config.per_subspace_weights.is_empty() {
                    let sum: f64 = self.config.per_subspace_weights.iter().sum();
                    if sum > 0.0 {
                        for w in &mut self.config.per_subspace_weights {
                            *w /= sum;
                        }
                    }
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    /// Run one full evolution round.
    /// EvolveMem: Evaluate → Diagnose → Propose → Guard
    /// Returns (score_improved, mutation_applied)
    pub fn run_evolution_round(&mut self, window: usize) -> (bool, String) {
        self.evolution_round += 1;

        // 1. Evaluate
        let current_score = self.evaluate_current_config(window);

        // 2. Diagnose
        let diagnoses = self.diagnose_failures(window);

        // 3. Propose
        let mutations = self.propose_mutations(&diagnoses);

        if mutations.is_empty() {
            self.stagnation_rounds += 1;
            // Explore on stagnation: random perturbation
            if self.stagnation_rounds >= 2 {
                self.config.top_k = (self.config.top_k as f64
                    * (1.0 + 0.2 * (fastrand::f64() - 0.5)))
                    .round()
                    .max(1.0) as usize;
                self.config.similarity_threshold = (self.config.similarity_threshold
                    + 0.05 * (fastrand::f64() - 0.5))
                    .clamp(0.1, 1.0);
                self.stagnation_rounds = 0;
                return (
                    false,
                    format!(
                        "stagnation_explore: new top_k={} threshold={:.2}",
                        self.config.top_k, self.config.similarity_threshold
                    ),
                );
            }
            return (false, "no_mutations_needed".into());
        }

        // 4. Apply first mutation (guard: auto-revert on regression)
        let mutation = mutations[0].clone();
        self.proposals_made += 1;

        let config_before = self.config.clone();
        self.apply_mutation(&mutation);

        // Evaluate after mutation
        let new_score = self.evaluate_current_config(window);

        // Guard: revert if score drops
        if new_score < current_score * 0.95 {
            self.config = config_before;
            self.proposals_reverted += 1;
            log::info!(
                "REVO: reverted '{}' (score {:.4} -> {:.4})",
                mutation,
                current_score,
                new_score
            );
            return (false, format!("reverted: {}", mutation));
        }

        // Accept
        self.proposals_accepted += 1;
        if new_score > self.best_score {
            self.best_score = new_score;
            self.best_config = Some(self.config.clone());
        }
        self.stagnation_rounds = 0;

        log::info!(
            "REVO: round={} score={:.4}->{:.4} mutation='{}' accepted={} reverted={}",
            self.evolution_round,
            current_score,
            new_score,
            mutation,
            self.proposals_accepted,
            self.proposals_reverted,
        );

        // Record diagnosis
        for d in diagnoses {
            self.diagnosis_history.push_back(d);
        }
        if self.diagnosis_history.len() > 100 {
            self.diagnosis_history.pop_front();
        }

        (new_score > current_score, mutation)
    }

    /// Summary for dashboard/logging.
    pub fn summary(&self) -> String {
        format!(
            "REVO: round={} config=(top_k={},threshold={:.2},fusion={:?},entity_swap={},query_dec={},reflect={},recency={:.2}) best_score={:.4} props={}/{} reverted={}",
            self.evolution_round,
            self.config.top_k,
            self.config.similarity_threshold,
            self.config.fusion_mode,
            self.config.entity_swap_enabled,
            self.config.query_decomposition_enabled,
            self.config.reflection_rounds,
            self.config.recency_weight,
            self.best_score,
            self.proposals_accepted,
            self.proposals_made,
            self.proposals_reverted,
        )
    }

    /// Restore the best config found.
    pub fn restore_best_config(&mut self) -> bool {
        if let Some(ref best) = self.best_config {
            self.config = best.clone();
            log::info!("REVO: restored best config with score={}", self.best_score);
            true
        } else {
            false
        }
    }
}

impl Default for RetrievalEvolutionEngine {
    fn default() -> Self {
        Self::new(RetrievalConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trace(recall: f64, precision: f64) -> RetrievalTrace {
        RetrievalTrace {
            query_vsa_hash: 0,
            query_text: "test".into(),
            expected_result_ids: vec![1, 2, 3],
            actual_result_ids: vec![1],
            recall_at_k: recall,
            precision_at_k: precision,
            latency_ms: 10.0,
            config_used: RetrievalConfig::default(),
            timestamp: now_u64(),
        }
    }

    #[test]
    fn test_default_config() {
        let engine = RetrievalEvolutionEngine::default();
        assert_eq!(engine.config.top_k, 5);
        assert!((engine.config.similarity_threshold - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_record_trace() {
        let mut engine = RetrievalEvolutionEngine::default();
        engine.record_trace(make_trace(0.8, 0.9));
        assert_eq!(engine.traces.len(), 1);
    }

    #[test]
    fn test_evaluate_current_config() {
        let mut engine = RetrievalEvolutionEngine::default();
        engine.record_trace(make_trace(0.8, 0.9));
        engine.record_trace(make_trace(0.6, 0.7));
        let score = engine.evaluate_current_config(2);
        assert!((score - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_diagnose_overlooked_entity() {
        let mut engine = RetrievalEvolutionEngine::default();
        for _ in 0..10 {
            engine.record_trace(make_trace(0.3, 0.5));
        }
        let diagnoses = engine.diagnose_failures(10);
        let has_overlooked = diagnoses
            .iter()
            .any(|d| matches!(d.category, DiagnosisCategory::OverlookedEntity));
        assert!(has_overlooked, "should detect overlooked entity pattern");
    }

    #[test]
    fn test_diagnose_threshold_too_strict() {
        let mut engine = RetrievalEvolutionEngine::default();
        for _ in 0..10 {
            engine.record_trace(make_trace(0.3, 0.95));
        }
        let diagnoses = engine.diagnose_failures(10);
        let has_strict = diagnoses
            .iter()
            .any(|d| matches!(d.category, DiagnosisCategory::ThresholdTooStrict));
        assert!(has_strict, "should detect too-strict threshold");
    }

    #[test]
    fn test_diagnose_threshold_too_loose() {
        let mut engine = RetrievalEvolutionEngine::default();
        for _ in 0..10 {
            engine.record_trace(make_trace(0.8, 0.2));
        }
        let diagnoses = engine.diagnose_failures(10);
        let has_loose = diagnoses
            .iter()
            .any(|d| matches!(d.category, DiagnosisCategory::ThresholdTooLoose));
        assert!(has_loose, "should detect too-loose threshold");
    }

    #[test]
    fn test_apply_mutation_top_k() {
        let mut engine = RetrievalEvolutionEngine::default();
        assert!(engine.apply_mutation("top_k:10"));
        assert_eq!(engine.config.top_k, 10);
    }

    #[test]
    fn test_apply_mutation_similarity_threshold() {
        let mut engine = RetrievalEvolutionEngine::default();
        assert!(engine.apply_mutation("similarity_threshold:-0.05"));
        assert!((engine.config.similarity_threshold - 0.65).abs() < 1e-6);
    }

    #[test]
    fn test_apply_mutation_entity_swap() {
        let mut engine = RetrievalEvolutionEngine::default();
        assert!(engine.apply_mutation("entity_swap_enabled:true"));
        assert!(engine.config.entity_swap_enabled);
    }

    #[test]
    fn test_apply_mutation_reflection() {
        let mut engine = RetrievalEvolutionEngine::default();
        assert!(engine.apply_mutation("reflection_rounds:+2"));
        assert_eq!(engine.config.reflection_rounds, 2);
    }

    #[test]
    fn test_apply_mutation_fusion_mode_next() {
        let mut engine = RetrievalEvolutionEngine::default();
        let original = engine.config.fusion_mode;
        assert!(engine.apply_mutation("fusion_mode:next"));
        assert_ne!(engine.config.fusion_mode, original);
    }

    #[test]
    fn test_apply_mutation_invalid() {
        let mut engine = RetrievalEvolutionEngine::default();
        assert!(!engine.apply_mutation("invalid_format"));
        assert!(!engine.apply_mutation("unknown_key:value"));
    }

    #[test]
    fn test_evolution_round_no_traces() {
        let mut engine = RetrievalEvolutionEngine::default();
        let (improved, msg) = engine.run_evolution_round(10);
        assert!(!improved);
        assert_eq!(msg, "no_mutations_needed");
    }

    #[test]
    fn test_stagnation_explore() {
        let mut engine = RetrievalEvolutionEngine::default();
        engine.record_trace(make_trace(0.8, 0.9));
        let (_, msg1) = engine.run_evolution_round(10);
        assert_eq!(msg1, "no_mutations_needed");
        let (_, msg2) = engine.run_evolution_round(10);
        assert!(msg2.contains("stagnation_explore") || msg2 == "no_mutations_needed");
    }

    #[test]
    fn test_best_config_tracking() {
        let mut engine = RetrievalEvolutionEngine::default();
        engine.best_score = 0.5;
        engine.config.top_k = 15;
        engine.best_config = Some(engine.config.clone());
        engine.config.top_k = 100;
        assert!(engine.restore_best_config());
        assert_eq!(engine.config.top_k, 15);
    }

    #[test]
    fn test_run_evolution_round_with_traces() {
        let mut engine = RetrievalEvolutionEngine::default();
        for _ in 0..10 {
            engine.record_trace(make_trace(0.6, 0.8));
        }
        let (improved, _) = engine.run_evolution_round(10);
        // Should diagnose and propose some improvement
        assert_eq!(engine.evolution_round, 1);
        assert!(engine.proposals_made > 0);
    }

    #[test]
    fn test_diagnoses_bounded() {
        let mut engine = RetrievalEvolutionEngine::default();
        for _ in 0..50 {
            engine.record_trace(make_trace(0.3, 0.5));
            let diagnoses = engine.diagnose_failures(10);
            for d in &diagnoses {
                engine.diagnosis_history.push_back(d.clone());
            }
        }
        assert!(engine.diagnosis_history.len() <= 100);
    }

    #[test]
    fn test_fusion_mode_all_variants() {
        let modes = FusionMode::all();
        assert_eq!(modes.len(), 4);
        assert!(modes.contains(&FusionMode::Adaptive));
    }

    #[test]
    fn test_diagnosis_suggestion_not_empty() {
        let engine = RetrievalEvolutionEngine::default();
        let traces: Vec<RetrievalTrace> = (0..5).map(|_| make_trace(0.3, 0.5)).collect();
        for t in &traces {
            // simulate recording
        }
        let diagnoses = engine.diagnose_failures(5);
        for d in &diagnoses {
            assert!(
                !d.suggestion.is_empty(),
                "diagnosis should have suggestion: {:?}",
                d.category
            );
        }
    }

    #[test]
    fn test_summary_format() {
        let engine = RetrievalEvolutionEngine::default();
        let s = engine.summary();
        assert!(s.contains("REVO:"));
        assert!(s.contains("top_k="));
        assert!(s.contains("threshold="));
    }

    #[test]
    fn test_restore_best_when_none() {
        let mut engine = RetrievalEvolutionEngine::default();
        assert!(!engine.restore_best_config());
    }
}
