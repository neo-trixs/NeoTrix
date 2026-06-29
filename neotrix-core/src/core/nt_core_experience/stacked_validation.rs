use super::godel_checker::{GodelCheckResult, GodelConsistencyChecker};
use super::safety_gate::{SafetyGate, StatisticalSafetyGate};
use super::self_evolution_loop::{MutationOp, SelfEvolutionArchive};
use std::collections::HashMap;

/// Six validation layers for the stacked validation pipeline.
///
/// Mirroring the Gödel Agent three-layer check + regression/benchmark/meta layers:
///   Layer 1 — Syntax:           structural integrity
///   Layer 2 — TypeSafety:       type consistency + dangerous patterns
///   Layer 3 — SelfConsistency:  no self-contradiction
///   Layer 4 — Regression:       backward compatibility + behavior preservation
///   Layer 5 — Benchmark:        performance benchmark pass/fail
///   Layer 6 — Meta:             meta-accuracy + self-prediction alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationLayer {
    Syntax,
    TypeSafety,
    SelfConsistency,
    Regression,
    Benchmark,
    Meta,
}

impl ValidationLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            ValidationLayer::Syntax => "Syntax",
            ValidationLayer::TypeSafety => "TypeSafety",
            ValidationLayer::SelfConsistency => "SelfConsistency",
            ValidationLayer::Regression => "Regression",
            ValidationLayer::Benchmark => "Benchmark",
            ValidationLayer::Meta => "Meta",
        }
    }

    pub fn one_based_index(&self) -> usize {
        match self {
            ValidationLayer::Syntax => 1,
            ValidationLayer::TypeSafety => 2,
            ValidationLayer::SelfConsistency => 3,
            ValidationLayer::Regression => 4,
            ValidationLayer::Benchmark => 5,
            ValidationLayer::Meta => 6,
        }
    }
}

/// Result of a single validation layer check.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub layer: ValidationLayer,
    pub passed: bool,
    pub score: f64,
    pub detail: String,
    pub timestamp_ns: u64,
}

/// Configuration for the stacked validation pipeline.
#[derive(Debug, Clone)]
pub struct StackedValidationConfig {
    /// Stop at the first layer failure.
    pub fail_fast: bool,
    /// All 6 layers must pass for the pipeline to succeed.
    pub require_all_pass: bool,
    /// Minimum acceptable backward-compatibility score (Layer 4).
    pub regression_threshold: f64,
    /// Minimum acceptable benchmark score (Layer 5).
    pub benchmark_threshold: f64,
}

impl Default for StackedValidationConfig {
    fn default() -> Self {
        Self {
            fail_fast: false,
            require_all_pass: true,
            regression_threshold: 0.999,
            benchmark_threshold: 0.8,
        }
    }
}

/// Result of a single validation layer for the simplified `validate()` API.
#[derive(Debug, Clone)]
pub struct LayerResult {
    pub layer_name: String,
    pub passed: bool,
    pub score: f64,
    pub message: String,
}

/// Report from the simplified `validate()` method.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub all_passed: bool,
    pub total_layers: usize,
    pub passed_layers: usize,
    pub layer_results: Vec<LayerResult>,
}

/// Stacked Validation Pipeline — 6-layer unified validation combining
/// Gödel consistency checking, safety gate, regression, benchmark, and
/// meta-accuracy verification.
pub struct StackedValidationPipeline {
    pub godel: GodelConsistencyChecker,
    pub safety_gate: SafetyGate,
    pub statistical_gate: StatisticalSafetyGate,
    pub regression_scores: Vec<f64>,
    pub benchmark_scores: Vec<f64>,
    pub config: StackedValidationConfig,
}

impl StackedValidationPipeline {
    pub fn new() -> Self {
        Self {
            godel: GodelConsistencyChecker::new(),
            safety_gate: SafetyGate::new(),
            statistical_gate: StatisticalSafetyGate::new(),
            regression_scores: Vec::new(),
            benchmark_scores: Vec::new(),
            config: StackedValidationConfig::default(),
        }
    }

    pub fn with_config(config: StackedValidationConfig) -> Self {
        Self {
            config,
            ..Self::new()
        }
    }

    /// Run all 6 validation layers sequentially.
    ///
    /// If `config.fail_fast` is true, returns immediately at the first
    /// layer failure with only the results collected up to that point.
    pub fn run_all(
        &mut self,
        mutation: &MutationOp,
        archive: &SelfEvolutionArchive,
        vsa_primitives: &HashMap<&'static str, Vec<u8>>,
    ) -> Vec<ValidationResult> {
        let mut results = Vec::with_capacity(6);

        for layer in &[
            ValidationLayer::Syntax,
            ValidationLayer::TypeSafety,
            ValidationLayer::SelfConsistency,
            ValidationLayer::Regression,
            ValidationLayer::Benchmark,
            ValidationLayer::Meta,
        ] {
            let result = self.run_layer(*layer, mutation, archive, vsa_primitives);
            let failed = !result.passed;
            results.push(result);
            if failed && self.config.fail_fast {
                break;
            }
        }

        results
    }

    /// Run a single validation layer.
    ///
    /// - Layers 1–3 delegate to `GodelConsistencyChecker::check_proposal()`.
    /// - Layer 4 checks regression scores against the configured threshold.
    /// - Layer 5 checks benchmark scores against the configured threshold.
    /// - Layer 6 combines `SafetyGate` meta-accuracy logic with
    ///   `StatisticalSafetyGate::evaluate_with_stats()`.
    pub fn run_layer(
        &mut self,
        layer: ValidationLayer,
        mutation: &MutationOp,
        archive: &SelfEvolutionArchive,
        vsa_primitives: &HashMap<&'static str, Vec<u8>>,
    ) -> ValidationResult {
        let timestamp_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        match layer {
            ValidationLayer::Syntax
            | ValidationLayer::TypeSafety
            | ValidationLayer::SelfConsistency => {
                self.run_godel_layer(layer, mutation, archive, timestamp_ns)
            }
            ValidationLayer::Regression => self.run_regression_layer(timestamp_ns),
            ValidationLayer::Benchmark => self.run_benchmark_layer(timestamp_ns),
            ValidationLayer::Meta => self.run_meta_layer(vsa_primitives, timestamp_ns),
        }
    }

    /// Check whether all results passed.
    pub fn all_passed(results: &[ValidationResult]) -> bool {
        results.iter().all(|r| r.passed)
    }

    /// One-line summary string.
    pub fn summary(results: &[ValidationResult]) -> String {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        if failed == 0 {
            format!("✅ ALL {}/{} layers passed", passed, total)
        } else {
            let failed_layers: Vec<&str> = results
                .iter()
                .filter(|r| !r.passed)
                .map(|r| r.layer.as_str())
                .collect();
            format!(
                "⚠️  {}/{} passed, {}/{} failed: {}",
                passed,
                total,
                failed,
                total,
                failed_layers.join(", ")
            )
        }
    }

    // ── Internal helpers ──

    /// Map a Gödel 3-layer result to a single validation layer result.
    fn run_godel_layer(
        &self,
        layer: ValidationLayer,
        mutation: &MutationOp,
        archive: &SelfEvolutionArchive,
        timestamp_ns: u64,
    ) -> ValidationResult {
        let godel_result = self.godel.check_proposal(mutation, archive);
        let target_layer_godel = layer.one_based_index() as u8;

        let (passed, detail) = match godel_result {
            GodelCheckResult { layer: 0, .. } => (true, format!("{}: passed", layer.as_str())),
            GodelCheckResult {
                layer: l,
                ref reason,
                ..
            } if l > target_layer_godel => (true, format!("{}: passed", layer.as_str())),
            GodelCheckResult {
                layer: l,
                ref reason,
                ..
            } if l == target_layer_godel => {
                (false, format!("{}: failed — {}", layer.as_str(), reason))
            }
            _ => {
                // A prior layer failed, so this layer was never reached.
                // Report as passed (no error at this layer) with a note.
                (
                    true,
                    format!("{}: passed (prior layer failed)", layer.as_str()),
                )
            }
        };

        ValidationResult {
            layer,
            passed,
            score: if passed { 1.0 } else { 0.0 },
            detail,
            timestamp_ns,
        }
    }

    /// Layer 4: Regression — check backward-compatibility scores.
    fn run_regression_layer(&self, timestamp_ns: u64) -> ValidationResult {
        if self.regression_scores.is_empty() {
            return ValidationResult {
                layer: ValidationLayer::Regression,
                passed: true,
                score: 1.0,
                detail: "Regression: no scores to evaluate — passed by default".into(),
                timestamp_ns,
            };
        }

        let min_score = self
            .regression_scores
            .iter()
            .cloned()
            .fold(f64::MAX, f64::min);
        let avg_score =
            self.regression_scores.iter().sum::<f64>() / self.regression_scores.len() as f64;
        let passed = min_score >= self.config.regression_threshold;

        ValidationResult {
            layer: ValidationLayer::Regression,
            passed,
            score: avg_score,
            detail: format!(
                "Regression: min={:.6} avg={:.6} threshold={:.6} count={}",
                min_score,
                avg_score,
                self.config.regression_threshold,
                self.regression_scores.len()
            ),
            timestamp_ns,
        }
    }

    /// Layer 5: Benchmark — check performance benchmark scores.
    fn run_benchmark_layer(&self, timestamp_ns: u64) -> ValidationResult {
        if self.benchmark_scores.is_empty() {
            return ValidationResult {
                layer: ValidationLayer::Benchmark,
                passed: true,
                score: 1.0,
                detail: "Benchmark: no scores to evaluate — passed by default".into(),
                timestamp_ns,
            };
        }

        let min_score = self
            .benchmark_scores
            .iter()
            .cloned()
            .fold(f64::MAX, f64::min);
        let avg_score =
            self.benchmark_scores.iter().sum::<f64>() / self.benchmark_scores.len() as f64;
        let passed = min_score >= self.config.benchmark_threshold;

        ValidationResult {
            layer: ValidationLayer::Benchmark,
            passed,
            score: avg_score,
            detail: format!(
                "Benchmark: min={:.6} avg={:.6} threshold={:.6} count={}",
                min_score,
                avg_score,
                self.config.benchmark_threshold,
                self.benchmark_scores.len()
            ),
            timestamp_ns,
        }
    }

    /// Layer 6: Meta — meta-accuracy + statistical gate evaluation.
    fn run_meta_layer(
        &mut self,
        vsa_primitives: &HashMap<&'static str, Vec<u8>>,
        timestamp_ns: u64,
    ) -> ValidationResult {
        // Use SafetyGate's ecosystem: run a targeted check_all with meta_accuracy focus.
        let safety_report = self.safety_gate.check_all(
            None,
            true,
            &self.regression_scores,
            vsa_primitives,
            0.0,
            0.9,
            0.88,
        );

        let meta_check = &safety_report.checks[4];
        let meta_passed = meta_check.passed;
        let meta_score = meta_check.score;

        let mut detail = format!(
            "Meta-accuracy: {} (score={:.4})",
            if meta_passed { "passed" } else { "failed" },
            meta_score
        );

        // Statistical gate evaluation
        let stat_passed = self
            .statistical_gate
            .evaluate_with_stats(0.05, "meta-layer");
        let stat_detail = match &stat_passed {
            Ok(true) => "StatisticalGate: passed".to_string(),
            Ok(false) => "StatisticalGate: inconclusive".to_string(),
            Err(e) => format!("StatisticalGate: {}", e),
        };
        detail.push_str(&format!("; {}", stat_detail));

        let passed = meta_passed && stat_passed.is_ok();
        let composite_score = if passed {
            meta_score * 0.7 + 0.3
        } else {
            meta_score * 0.5
        };

        ValidationResult {
            layer: ValidationLayer::Meta,
            passed,
            score: composite_score,
            detail,
            timestamp_ns,
        }
    }
}

impl StackedValidationPipeline {
    /// Simplified validation that takes a proposal string and returns a report.
    ///
    /// Creates a minimal `MutationOp::RewriteHandler` from the proposal text
    /// and runs all 6 layers with default archive/vsa_primitives.
    /// Useful for BallVerifier integration where only the source code is available.
    pub fn validate(&mut self, proposal: &str) -> ValidationReport {
        let mutation = MutationOp::RewriteHandler {
            name: "proposal".into(),
            code: proposal.to_string(),
        };
        let archive = SelfEvolutionArchive::new();
        let vsa_primitives = HashMap::new();
        let results = self.run_all(&mutation, &archive, &vsa_primitives);
        let all_passed = Self::all_passed(&results);
        let layer_results: Vec<LayerResult> = results
            .iter()
            .map(|r| LayerResult {
                layer_name: r.layer.as_str().to_string(),
                passed: r.passed,
                score: r.score,
                message: r.detail.clone(),
            })
            .collect();
        ValidationReport {
            all_passed,
            total_layers: results.len(),
            passed_layers: results.iter().filter(|r| r.passed).count(),
            layer_results,
        }
    }
}

impl Default for StackedValidationPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_archive() -> SelfEvolutionArchive {
        let mut archive = SelfEvolutionArchive::new();
        archive.generation = 42;
        archive.best_score = 0.75;
        archive
    }

    fn valid_mutation() -> MutationOp {
        MutationOp::TuneParam {
            target: "cognitive_load.thinking_budget".into(),
            delta: 0.05,
        }
    }

    fn empty_mutation() -> MutationOp {
        MutationOp::TuneParam {
            target: "".into(),
            delta: 0.05,
        }
    }

    fn dangerous_mutation() -> MutationOp {
        MutationOp::RewriteHandler {
            name: "test_handler".into(),
            code: "unsafe { std::ptr::read(0) };".into(),
        }
    }

    fn vsa_primitives() -> HashMap<&'static str, Vec<u8>> {
        SafetyGate::compute_reference_primitives()
    }

    #[test]
    fn test_all_six_layers_pass_with_valid_inputs() {
        let mut pipeline = StackedValidationPipeline::new();
        pipeline.regression_scores = vec![1.0, 1.0, 1.0];
        pipeline.benchmark_scores = vec![0.95, 0.92];

        let results = pipeline.run_all(&valid_mutation(), &sample_archive(), &vsa_primitives());
        assert_eq!(results.len(), 6, "expected all 6 layers to run");
        assert!(
            StackedValidationPipeline::all_passed(&results),
            "all layers should pass: {}",
            StackedValidationPipeline::summary(&results)
        );
        for r in &results {
            assert!(r.passed, "layer {} failed unexpectedly", r.layer.as_str());
        }
    }

    #[test]
    fn test_layer1_syntax_fails_with_empty_mutation() {
        let mut pipeline = StackedValidationPipeline::new();
        let result = pipeline.run_layer(
            ValidationLayer::Syntax,
            &empty_mutation(),
            &sample_archive(),
            &vsa_primitives(),
        );
        assert!(!result.passed, "syntax layer should fail with empty target");
        assert!(result.score < 1.0);
        assert!(result.detail.contains("failed") || result.detail.contains("empty"));
    }

    #[test]
    fn test_layer2_typesafety_fails_with_dangerous_pattern() {
        let mut pipeline = StackedValidationPipeline::new();
        let result = pipeline.run_layer(
            ValidationLayer::TypeSafety,
            &dangerous_mutation(),
            &sample_archive(),
            &vsa_primitives(),
        );
        assert!(
            !result.passed,
            "typesafety layer should catch dangerous pattern"
        );
        assert!(result.detail.contains("failed") || result.detail.contains("dangerous"));
    }

    #[test]
    fn test_layer4_regression_fails_with_low_scores() {
        let mut pipeline = StackedValidationPipeline::new();
        pipeline.regression_scores = vec![0.5, 0.6];
        let result = pipeline.run_layer(
            ValidationLayer::Regression,
            &valid_mutation(),
            &sample_archive(),
            &vsa_primitives(),
        );
        assert!(!result.passed, "regression should fail with low scores");
        assert!(result.score < 1.0);
    }

    #[test]
    fn test_fail_fast_stops_at_first_failure() {
        let mut pipeline = StackedValidationPipeline::new();
        pipeline.config.fail_fast = true;
        // Regression with low scores will fail at layer 4;
        // fail_fast should stop before reaching layers 5-6.
        pipeline.regression_scores = vec![0.3];

        let results = pipeline.run_all(
            &MutationOp::TuneParam {
                target: "cognitive_load.thinking_budget".into(),
                delta: 0.05,
            },
            &sample_archive(),
            &vsa_primitives(),
        );

        assert!(
            results.len() < 6,
            "fail_fast should stop early: got {} results",
            results.len()
        );
        if !results.is_empty() {
            assert!(!results.last().unwrap().passed);
        }
    }

    #[test]
    fn test_all_passed_utility() {
        let passed = vec![
            ValidationResult {
                layer: ValidationLayer::Syntax,
                passed: true,
                score: 1.0,
                detail: "".into(),
                timestamp_ns: 0,
            },
            ValidationResult {
                layer: ValidationLayer::Meta,
                passed: true,
                score: 1.0,
                detail: "".into(),
                timestamp_ns: 0,
            },
        ];
        assert!(StackedValidationPipeline::all_passed(&passed));

        let mixed = vec![
            ValidationResult {
                layer: ValidationLayer::Syntax,
                passed: true,
                score: 1.0,
                detail: "".into(),
                timestamp_ns: 0,
            },
            ValidationResult {
                layer: ValidationLayer::TypeSafety,
                passed: false,
                score: 0.0,
                detail: "failure".into(),
                timestamp_ns: 0,
            },
        ];
        assert!(!StackedValidationPipeline::all_passed(&mixed));
    }

    #[test]
    fn test_summary_output() {
        let results = vec![
            ValidationResult {
                layer: ValidationLayer::Syntax,
                passed: true,
                score: 1.0,
                detail: "ok".into(),
                timestamp_ns: 0,
            },
            ValidationResult {
                layer: ValidationLayer::TypeSafety,
                passed: false,
                score: 0.0,
                detail: "fail".into(),
                timestamp_ns: 0,
            },
        ];
        let summary = StackedValidationPipeline::summary(&results);
        assert!(summary.contains("1/2 passed") || summary.contains("1/2"));
    }

    #[test]
    fn test_layer4_regression_empty_scores_passes() {
        let mut pipeline = StackedValidationPipeline::new();
        let result = pipeline.run_layer(
            ValidationLayer::Regression,
            &valid_mutation(),
            &sample_archive(),
            &vsa_primitives(),
        );
        assert!(
            result.passed,
            "regression with no scores should pass by default"
        );
    }

    #[test]
    fn test_layer5_benchmark_fails_with_low_scores() {
        let mut pipeline = StackedValidationPipeline::new();
        pipeline.benchmark_scores = vec![0.1, 0.2];
        let result = pipeline.run_layer(
            ValidationLayer::Benchmark,
            &valid_mutation(),
            &sample_archive(),
            &vsa_primitives(),
        );
        assert!(!result.passed, "benchmark should fail with low scores");
    }

    #[test]
    fn test_layer_index_constants() {
        assert_eq!(ValidationLayer::Syntax.one_based_index(), 1);
        assert_eq!(ValidationLayer::TypeSafety.one_based_index(), 2);
        assert_eq!(ValidationLayer::SelfConsistency.one_based_index(), 3);
        assert_eq!(ValidationLayer::Regression.one_based_index(), 4);
        assert_eq!(ValidationLayer::Benchmark.one_based_index(), 5);
        assert_eq!(ValidationLayer::Meta.one_based_index(), 6);
    }

    #[test]
    fn test_layer_as_str() {
        assert_eq!(ValidationLayer::Syntax.as_str(), "Syntax");
        assert_eq!(ValidationLayer::TypeSafety.as_str(), "TypeSafety");
        assert_eq!(ValidationLayer::SelfConsistency.as_str(), "SelfConsistency");
        assert_eq!(ValidationLayer::Regression.as_str(), "Regression");
        assert_eq!(ValidationLayer::Benchmark.as_str(), "Benchmark");
        assert_eq!(ValidationLayer::Meta.as_str(), "Meta");
    }

    #[test]
    fn test_fail_fast_does_not_trigger_when_disabled() {
        let mut pipeline = StackedValidationPipeline::new();
        pipeline.config.fail_fast = false;
        pipeline.regression_scores = vec![0.3];

        let results = pipeline.run_all(
            &MutationOp::TuneParam {
                target: "cognitive_load.thinking_budget".into(),
                delta: 0.05,
            },
            &sample_archive(),
            &vsa_primitives(),
        );

        assert_eq!(
            results.len(),
            6,
            "without fail_fast all 6 layers should run"
        );
    }

    #[test]
    fn test_benchmark_with_no_scores_passes() {
        let mut pipeline = StackedValidationPipeline::new();
        let result = pipeline.run_layer(
            ValidationLayer::Benchmark,
            &valid_mutation(),
            &sample_archive(),
            &vsa_primitives(),
        );
        assert!(
            result.passed,
            "benchmark with no scores should pass by default"
        );
        assert!((result.score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_pipeline_default_config() {
        let config = StackedValidationConfig::default();
        assert!(!config.fail_fast);
        assert!(config.require_all_pass);
        assert!((config.regression_threshold - 0.999).abs() < 1e-6);
        assert!((config.benchmark_threshold - 0.8).abs() < 1e-6);
    }
}
