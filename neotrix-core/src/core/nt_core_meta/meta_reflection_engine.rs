// REVIVED Evo 3 — dead_code removed
use std::collections::HashMap;

// ── Part 1: Core Types — Reflection Dimensions ──

/// What aspect of cognition to reflect on
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReflectionDimension {
    /// Quality of reasoning steps
    ReasoningQuality,
    /// Confidence calibration (did I know what I thought I knew?)
    ConfidenceCalibration,
    /// Consistency across time
    TemporalConsistency,
    /// Bias detection (confirmation, availability, etc.)
    BiasDetection,
    /// Assumption validity
    AssumptionValidity,
    /// Alternative paths not taken
    Counterfactuals,
    /// Resource efficiency (was this the best use of compute?)
    ResourceEfficiency,
    /// Ethical/value alignment
    ValueAlignment,
    /// Learning from errors
    ErrorLearning,
}

impl ReflectionDimension {
    pub fn all() -> Vec<ReflectionDimension> {
        vec![
            ReflectionDimension::ReasoningQuality,
            ReflectionDimension::ConfidenceCalibration,
            ReflectionDimension::TemporalConsistency,
            ReflectionDimension::BiasDetection,
            ReflectionDimension::AssumptionValidity,
            ReflectionDimension::Counterfactuals,
            ReflectionDimension::ResourceEfficiency,
            ReflectionDimension::ValueAlignment,
            ReflectionDimension::ErrorLearning,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ReflectionDimension::ReasoningQuality => "Reasoning Quality",
            ReflectionDimension::ConfidenceCalibration => "Confidence Calibration",
            ReflectionDimension::TemporalConsistency => "Temporal Consistency",
            ReflectionDimension::BiasDetection => "Bias Detection",
            ReflectionDimension::AssumptionValidity => "Assumption Validity",
            ReflectionDimension::Counterfactuals => "Counterfactuals",
            ReflectionDimension::ResourceEfficiency => "Resource Efficiency",
            ReflectionDimension::ValueAlignment => "Value Alignment",
            ReflectionDimension::ErrorLearning => "Error Learning",
        }
    }
}

/// Result of a single reflection
#[derive(Debug, Clone)]
pub struct ReflectionResult {
    pub dimension: ReflectionDimension,
    pub score: f64,
    pub confidence: f64,
    pub observations: Vec<String>,
    pub recommendations: Vec<String>,
    pub severity: ReflectionSeverity,
    pub trace: ReflectionTrace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReflectionSeverity {
    Info,
    Warning,
    Critical,
    Catastrophic,
}

impl ReflectionSeverity {
    pub fn from_score(score: f64) -> Self {
        if score < 0.2 {
            ReflectionSeverity::Catastrophic
        } else if score < 0.4 {
            ReflectionSeverity::Critical
        } else if score < 0.7 {
            ReflectionSeverity::Warning
        } else {
            ReflectionSeverity::Info
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ReflectionSeverity::Info => "info",
            ReflectionSeverity::Warning => "warning",
            ReflectionSeverity::Critical => "critical",
            ReflectionSeverity::Catastrophic => "catastrophic",
        }
    }
}

/// Full audit trail of a reflection
#[derive(Debug, Clone)]
pub struct ReflectionTrace {
    pub timestamp: String,
    pub cycle_number: u64,
    pub input_summary: String,
    pub reasoning_path: Vec<String>,
    pub alternates_considered: u32,
    pub time_taken_ms: u64,
}

impl ReflectionTrace {
    pub fn new(cycle_number: u64, input_summary: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        Self {
            timestamp: format!("{}", now),
            cycle_number,
            input_summary: input_summary.to_string(),
            reasoning_path: Vec::new(),
            alternates_considered: 0,
            time_taken_ms: 0,
        }
    }
}

// ── Part 2: CycleReflectionData ──

/// Data from a consciousness cycle that the reflection engine analyzes
#[derive(Debug, Clone)]
pub struct CycleReflectionData {
    pub cycle_number: u64,
    pub perception_quality: f64,
    pub retrieval_quality: f64,
    pub reasoning_paths: Vec<ReasoningPathData>,
    pub chosen_action: String,
    pub action_confidence: f64,
    pub outcome: Option<CycleOutcome>,
    pub time_per_step_ms: Vec<u64>,
    pub errors: Vec<String>,
}

impl CycleReflectionData {
    pub fn new(cycle_number: u64) -> Self {
        Self {
            cycle_number,
            perception_quality: 0.5,
            retrieval_quality: 0.5,
            reasoning_paths: Vec::new(),
            chosen_action: String::new(),
            action_confidence: 0.5,
            outcome: None,
            time_per_step_ms: Vec::new(),
            errors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReasoningPathData {
    pub path_id: u32,
    pub steps: u32,
    pub final_confidence: f64,
    pub was_selected: bool,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum CycleOutcome {
    Success,
    PartialSuccess(String),
    Failure(String),
    NotYetEvaluated,
}

// ── Part 3: MetaHealth & RecurringPattern ──

#[derive(Debug, Clone)]
pub struct MetaHealth {
    pub overall_score: f64,
    pub dimension_scores: HashMap<ReflectionDimension, f64>,
    pub warning_count: u32,
    pub critical_count: u32,
    pub trend_direction: &'static str,
}

#[derive(Debug, Clone)]
pub struct RecurringPattern {
    pub pattern: String,
    pub occurrences: u32,
    pub recommended_action: String,
    pub severity: ReflectionSeverity,
}

// ── Part 4: MetaReflectionEngine ──

#[derive(Debug, Clone)]
pub struct MetaReflectionEngine {
    pub reflection_history: Vec<ReflectionResult>,
    pub active_dimensions: Vec<ReflectionDimension>,
    pub reflection_threshold: f64,
    pub max_history: usize,
    pub cycle_count: u64,
}

impl MetaReflectionEngine {
    pub fn new() -> Self {
        Self {
            reflection_history: Vec::new(),
            active_dimensions: ReflectionDimension::all(),
            reflection_threshold: 0.5,
            max_history: 500,
            cycle_count: 0,
        }
    }

    pub fn with_dimensions(mut self, dims: Vec<ReflectionDimension>) -> Self {
        self.active_dimensions = dims;
        self
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.reflection_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Reflect on a reasoning cycle's output across all active dimensions
    pub fn reflect_on_cycle(&mut self, cycle_data: &CycleReflectionData) -> Vec<ReflectionResult> {
        self.cycle_count += 1;
        let mut results = Vec::new();

        for dim in &self.active_dimensions {
            let result = match dim {
                ReflectionDimension::ReasoningQuality => self.reflect_reasoning_quality(cycle_data),
                ReflectionDimension::ConfidenceCalibration => self.reflect_confidence(cycle_data),
                ReflectionDimension::TemporalConsistency => {
                    self.reflect_temporal_consistency(cycle_data)
                }
                ReflectionDimension::BiasDetection => self.reflect_bias(cycle_data),
                ReflectionDimension::AssumptionValidity => self.reflect_assumptions(cycle_data),
                ReflectionDimension::Counterfactuals => self.reflect_counterfactuals(cycle_data),
                ReflectionDimension::ResourceEfficiency => self.reflect_efficiency(cycle_data),
                ReflectionDimension::ValueAlignment => self.reflect_alignment(cycle_data),
                ReflectionDimension::ErrorLearning => self.reflect_error_learning(cycle_data),
            };
            results.push(result);
        }

        for r in &results {
            self.reflection_history.push(r.clone());
        }

        while self.reflection_history.len() > self.max_history {
            self.reflection_history.remove(0);
        }

        results
    }

    fn build_trace(
        &self,
        data: &CycleReflectionData,
        path: Vec<String>,
        alts: u32,
    ) -> ReflectionTrace {
        let mut trace = ReflectionTrace::new(data.cycle_number, &data.chosen_action);
        trace.reasoning_path = path;
        trace.alternates_considered = alts;
        trace.time_taken_ms = data.time_per_step_ms.iter().sum();
        trace
    }

    fn reflect_reasoning_quality(&self, data: &CycleReflectionData) -> ReflectionResult {
        let mut observations = Vec::new();
        let mut recommendations = Vec::new();

        let avg_steps: f64 = if data.reasoning_paths.is_empty() {
            0.0
        } else {
            data.reasoning_paths
                .iter()
                .map(|p| p.steps as f64)
                .sum::<f64>()
                / data.reasoning_paths.len() as f64
        };

        let avg_confidence: f64 = if data.reasoning_paths.is_empty() {
            data.action_confidence
        } else {
            data.reasoning_paths
                .iter()
                .map(|p| p.final_confidence)
                .sum::<f64>()
                / data.reasoning_paths.len() as f64
        };

        let selected_path = data.reasoning_paths.iter().find(|p| p.was_selected);

        let score = if data.reasoning_paths.is_empty() {
            observations.push("No reasoning paths recorded".to_string());
            recommendations.push("Generate at least one reasoning path per cycle".to_string());
            0.3
        } else if avg_steps < 1.0 {
            observations.push("Reasoning paths have zero steps".to_string());
            recommendations.push("Ensure each path has meaningful reasoning steps".to_string());
            0.3
        } else {
            let stability = (avg_steps / 10.0).min(1.0);
            let confidence_bonus = avg_confidence * 0.3;
            let selected_bonus = selected_path
                .map(|p| p.final_confidence * 0.2)
                .unwrap_or(0.0);
            let quality =
                (0.5 + stability * 0.3 + confidence_bonus + selected_bonus).clamp(0.0, 1.0);

            if quality >= 0.7 {
                observations.push(format!("Reasoning quality is good ({:.2})", quality));
            } else if quality >= 0.4 {
                observations.push(format!("Reasoning quality is moderate ({:.2})", quality));
                if avg_steps < 3.0 {
                    recommendations.push("Consider deeper reasoning chains (>3 steps)".to_string());
                }
                if avg_confidence < 0.5 {
                    recommendations.push("Reduce uncertainty with additional evidence".to_string());
                }
            } else {
                observations.push(format!("Reasoning quality is poor ({:.2})", quality));
                recommendations
                    .push("Review reasoning methodology for systemic issues".to_string());
            }
            quality
        };

        let path: Vec<String> = data
            .reasoning_paths
            .iter()
            .map(|p| format!("path_{}:{:.2}", p.path_id, p.final_confidence))
            .collect();
        let trace = self.build_trace(data, path, data.reasoning_paths.len() as u32);

        ReflectionResult {
            dimension: ReflectionDimension::ReasoningQuality,
            score,
            confidence: avg_confidence,
            observations,
            recommendations,
            severity: ReflectionSeverity::from_score(score),
            trace,
        }
    }

    fn reflect_confidence(&self, data: &CycleReflectionData) -> ReflectionResult {
        let mut observations = Vec::new();
        let mut recommendations = Vec::new();

        let (score, conf) = match &data.outcome {
            Some(CycleOutcome::Success) => {
                if data.action_confidence > 0.8 {
                    observations
                        .push("High confidence, correct outcome — well calibrated".to_string());
                    (0.9, 0.9)
                } else if data.action_confidence > 0.5 {
                    observations
                        .push("Moderate confidence, correct outcome — acceptable".to_string());
                    (0.7, 0.7)
                } else {
                    observations.push(
                        "Low confidence but still correct — underconfidence detected".to_string(),
                    );
                    recommendations.push("Trust your reasoning more: low confidence with success suggests underconfidence".to_string());
                    (0.5, 0.4)
                }
            }
            Some(CycleOutcome::PartialSuccess(_)) => {
                if data.action_confidence > 0.8 {
                    observations.push(
                        "High confidence but only partial success — overconfidence suspected"
                            .to_string(),
                    );
                    recommendations.push("Calibrate confidence downward: high confidence should correlate with full success".to_string());
                    (0.4, 0.6)
                } else {
                    observations
                        .push("Partial success with appropriate confidence level".to_string());
                    (0.6, 0.6)
                }
            }
            Some(CycleOutcome::Failure(_)) => {
                if data.action_confidence > 0.7 {
                    observations
                        .push("High confidence but failed — overconfidence detected".to_string());
                    recommendations.push("Implement confidence calibration: your confidence exceeds actual performance".to_string());
                    (0.2, 0.5)
                } else {
                    observations.push(
                        "Failure with appropriate low confidence — calibration OK".to_string(),
                    );
                    recommendations.push(
                        "Analyze root cause of failure despite correct calibration".to_string(),
                    );
                    (0.5, 0.7)
                }
            }
            None | Some(CycleOutcome::NotYetEvaluated) => {
                observations.push("No outcome data available for calibration analysis".to_string());
                recommendations.push("Enable outcome tracking to calibrate confidence".to_string());
                (0.5, 0.3)
            }
        };

        let trace = self.build_trace(
            data,
            vec![format!("confidence={:.2}", data.action_confidence)],
            0,
        );
        ReflectionResult {
            dimension: ReflectionDimension::ConfidenceCalibration,
            score,
            confidence: conf,
            observations,
            recommendations,
            severity: ReflectionSeverity::from_score(score),
            trace,
        }
    }

    fn reflect_temporal_consistency(&self, data: &CycleReflectionData) -> ReflectionResult {
        let mut observations = Vec::new();
        let mut recommendations = Vec::new();

        let history_len = self.reflection_history.len();
        let consistency = if history_len < 2 {
            observations.push("Not enough history for temporal consistency analysis".to_string());
            0.7
        } else {
            let recent: Vec<&ReflectionResult> =
                self.reflection_history.iter().rev().take(5).collect();
            let reasoning_scores: Vec<f64> = recent
                .iter()
                .filter(|r| r.dimension == ReflectionDimension::ReasoningQuality)
                .map(|r| r.score)
                .collect();

            if reasoning_scores.len() < 2 {
                observations
                    .push("Limited reasoning quality history for trend analysis".to_string());
                0.6
            } else {
                let variance = {
                    let mean: f64 =
                        reasoning_scores.iter().sum::<f64>() / reasoning_scores.len() as f64;
                    reasoning_scores
                        .iter()
                        .map(|s| (s - mean).powi(2))
                        .sum::<f64>()
                        / reasoning_scores.len() as f64
                };
                let std_dev = variance.sqrt();

                if std_dev < 0.1 {
                    observations.push(format!("High temporal consistency (std={:.3})", std_dev));
                    0.9
                } else if std_dev < 0.3 {
                    observations.push(format!(
                        "Moderate temporal consistency (std={:.3})",
                        std_dev
                    ));
                    0.6
                } else {
                    observations.push(format!(
                        "Low temporal consistency (std={:.3}) — scores fluctuate excessively",
                        std_dev
                    ));
                    recommendations.push("Stabilize reasoning quality across cycles".to_string());
                    0.3
                }
            }
        };

        let trace = self.build_trace(data, vec![format!("history_len={}", history_len)], 0);
        ReflectionResult {
            dimension: ReflectionDimension::TemporalConsistency,
            score: consistency,
            confidence: 0.6,
            observations,
            recommendations,
            severity: ReflectionSeverity::from_score(consistency),
            trace,
        }
    }

    fn reflect_bias(&self, data: &CycleReflectionData) -> ReflectionResult {
        let mut observations = Vec::new();
        let mut recommendations = Vec::new();
        let mut bias_count = 0u32;

        let action_lower = data.chosen_action.to_lowercase();

        if action_lower.contains("prefer") || action_lower.contains("favor") {
            if data.reasoning_paths.len() < 2 {
                bias_count += 1;
                observations.push(
                    "Confirmation bias possible: preferred action without considering alternatives"
                        .to_string(),
                );
                recommendations.push(
                    "Explicitly evaluate at least one counter-argument before committing"
                        .to_string(),
                );
            }
        }

        if data.reasoning_paths.len() <= 1
            && data.reasoning_paths.first().map(|p| p.steps).unwrap_or(0) > 0
        {
            bias_count += 1;
            observations.push(
                "Anchoring bias possible: single reasoning path without diversification"
                    .to_string(),
            );
            recommendations.push("Generate multiple reasoning paths before selecting".to_string());
        }

        if data.action_confidence > 0.9 {
            bias_count += 1;
            observations
                .push("Overconfidence bias: confidence > 0.9 without full certainty".to_string());
            recommendations.push("Apply confidence penalty to extremely high scores".to_string());
        }

        if let Some(outcome) = &data.outcome {
            if matches!(outcome, CycleOutcome::Failure(_)) {
                if action_lower.contains("continue") || action_lower.contains("same") {
                    bias_count += 1;
                    observations
                        .push("Sunk cost bias possible: continuing failing approach".to_string());
                    recommendations.push(
                        "Consider switching strategies when current approach fails".to_string(),
                    );
                }
            }
        }

        let score = if bias_count == 0 {
            observations.push("No significant bias indicators detected".to_string());
            0.9
        } else if bias_count == 1 {
            observations.push(format!("{} bias indicator detected", bias_count));
            0.6
        } else {
            observations.push(format!(
                "{} bias indicators detected — review recommended",
                bias_count
            ));
            recommendations.push("Run full bias audit before next action cycle".to_string());
            0.3
        };

        let trace = self.build_trace(
            data,
            vec![format!("bias_indicators={}", bias_count)],
            data.reasoning_paths.len() as u32,
        );
        ReflectionResult {
            dimension: ReflectionDimension::BiasDetection,
            score,
            confidence: (1.0 - (bias_count as f64 * 0.15)).clamp(0.0, 1.0),
            observations,
            recommendations,
            severity: ReflectionSeverity::from_score(score),
            trace,
        }
    }

    fn reflect_assumptions(&self, data: &CycleReflectionData) -> ReflectionResult {
        let mut observations = Vec::new();
        let mut recommendations = Vec::new();
        let mut assumption_issues = 0u32;

        let action_lower = data.chosen_action.to_lowercase();
        let assumption_keywords = [
            "assume", "presume", "probably", "likely", "should", "must", "always", "never",
        ];

        for kw in &assumption_keywords {
            if action_lower.contains(kw) {
                assumption_issues += 1;
            }
        }

        let has_weak_paths = data
            .reasoning_paths
            .iter()
            .any(|p| p.final_confidence < 0.3);
        if has_weak_paths {
            assumption_issues += 1;
            observations.push("Weak reasoning paths indicate questionable assumptions".to_string());
        }

        if data.perception_quality < 0.3 {
            assumption_issues += 1;
            observations.push("Poor perception quality may hide invalid assumptions".to_string());
        }

        if data.retrieval_quality < 0.3 {
            assumption_issues += 1;
            observations.push("Poor retrieval quality may reinforce false assumptions".to_string());
        }

        let score = if assumption_issues == 0 {
            observations.push("No assumption validity issues detected".to_string());
            0.85
        } else if assumption_issues <= 2 {
            observations.push(format!(
                "{} assumption concern(s) — minor issues",
                assumption_issues
            ));
            if assumption_issues >= 1 {
                recommendations.push("Review explicit assumptions for validity".to_string());
            }
            0.6
        } else {
            observations.push(format!(
                "{} assumption issues — significant concerns",
                assumption_issues
            ));
            recommendations
                .push("Perform structured assumption audit before proceeding".to_string());
            recommendations.push("Use counterfactual analysis to test key assumptions".to_string());
            0.3
        };

        let trace = self.build_trace(
            data,
            vec![format!("assumption_issues={}", assumption_issues)],
            0,
        );
        ReflectionResult {
            dimension: ReflectionDimension::AssumptionValidity,
            score,
            confidence: 0.65,
            observations,
            recommendations,
            severity: ReflectionSeverity::from_score(score),
            trace,
        }
    }

    fn reflect_counterfactuals(&self, data: &CycleReflectionData) -> ReflectionResult {
        let mut observations = Vec::new();
        let mut recommendations = Vec::new();

        let alt_count = data.reasoning_paths.len();
        let non_selected = data
            .reasoning_paths
            .iter()
            .filter(|p| !p.was_selected)
            .count();

        let (score, conf) = if alt_count == 0 {
            observations.push("No alternative paths considered".to_string());
            recommendations
                .push("Always generate at least 2-3 alternative reasoning paths".to_string());
            (0.2, 0.5)
        } else if alt_count == 1 {
            observations.push("Only one reasoning path — no counterfactual analysis".to_string());
            recommendations
                .push("Generate alternative paths even if they seem unlikely".to_string());
            (0.3, 0.5)
        } else if non_selected == 0 {
            observations
                .push("All paths were selected — counterfactual analysis incomplete".to_string());
            recommendations.push("Record why non-selected paths were rejected".to_string());
            (0.5, 0.6)
        } else {
            let ratio = non_selected as f64 / alt_count as f64;
            if ratio > 0.5 {
                observations.push(format!(
                    "Good counterfactual diversity: {}/{} non-selected paths",
                    non_selected, alt_count
                ));
                (0.8, 0.7)
            } else {
                observations.push(format!(
                    "Moderate counterfactual diversity: {}/{} non-selected",
                    non_selected, alt_count
                ));
                recommendations.push("Increase diversity of generated reasoning paths".to_string());
                (0.6, 0.6)
            }
        };

        let trace = self.build_trace(
            data,
            vec![format!("alternatives={}", alt_count)],
            alt_count as u32,
        );
        ReflectionResult {
            dimension: ReflectionDimension::Counterfactuals,
            score,
            confidence: conf,
            observations,
            recommendations,
            severity: ReflectionSeverity::from_score(score),
            trace,
        }
    }

    fn reflect_efficiency(&self, data: &CycleReflectionData) -> ReflectionResult {
        let mut observations = Vec::new();
        let mut recommendations = Vec::new();

        let _total_cycles_for_step =
            |step_idx: usize| -> u64 { data.time_per_step_ms.get(step_idx).copied().unwrap_or(0) };

        let total_time: u64 = data.time_per_step_ms.iter().sum();
        let step_count = data.time_per_step_ms.len() as u64;
        let avg_time = if step_count > 0 {
            total_time / step_count
        } else {
            0
        };

        let total_steps: u32 = data.reasoning_paths.iter().map(|p| p.steps).sum();
        let total_paths = data.reasoning_paths.len() as u64;

        let (score, conf) = if total_time > 10_000 && step_count < 3 {
            observations.push(format!(
                "High time ({total_time}ms) for few steps ({step_count}) — possible inefficiency"
            ));
            recommendations.push("Profile time distribution across cycle steps".to_string());
            (0.3, 0.6)
        } else if total_steps > 100 && total_paths <= 1 {
            observations.push(format!(
                "Many reasoning steps ({total_steps}) on single path — diminishing returns likely"
            ));
            recommendations.push("Set a maximum step limit per reasoning path".to_string());
            (0.4, 0.6)
        } else if avg_time > 2000 {
            observations.push(format!("High average step time ({avg_time}ms)"));
            recommendations.push("Investigate slow steps for optimization".to_string());
            (0.5, 0.5)
        } else if data.errors.is_empty() && total_time < 1000 {
            observations.push(format!(
                "Efficient cycle: {total_time}ms total, {step_count} steps"
            ));
            (0.9, 0.8)
        } else {
            observations.push(format!(
                "Moderate efficiency: {total_time}ms, {step_count} steps, {} errors",
                data.errors.len()
            ));
            (0.7, 0.6)
        };

        let trace = self.build_trace(data, vec![format!("total_ms={}", total_time)], 0);
        ReflectionResult {
            dimension: ReflectionDimension::ResourceEfficiency,
            score,
            confidence: conf,
            observations,
            recommendations,
            severity: ReflectionSeverity::from_score(score),
            trace,
        }
    }

    fn reflect_alignment(&self, _data: &CycleReflectionData) -> ReflectionResult {
        let mut observations = Vec::new();
        let mut recommendations = Vec::new();

        let score = 0.8;
        observations.push("Value alignment analysis: no conflicts detected (baseline)".to_string());
        recommendations.push("Continue monitoring alignment metrics across cycles".to_string());

        let trace = self.build_trace(_data, vec!["alignment_baseline".to_string()], 0);
        ReflectionResult {
            dimension: ReflectionDimension::ValueAlignment,
            score,
            confidence: 0.5,
            observations,
            recommendations,
            severity: ReflectionSeverity::Info,
            trace,
        }
    }

    fn reflect_error_learning(&self, data: &CycleReflectionData) -> ReflectionResult {
        let mut observations = Vec::new();
        let mut recommendations = Vec::new();

        let error_count = data.errors.len();

        let (score, conf) = if error_count == 0 {
            observations.push("No errors in this cycle — learning analysis deferred".to_string());
            (0.8, 0.4)
        } else {
            let mut classified = Vec::new();
            for err in &data.errors {
                let err_lower = err.to_lowercase();
                if err_lower.contains("timeout") || err_lower.contains("time_out") {
                    classified.push("timeout");
                } else if err_lower.contains("not found") || err_lower.contains("missing") {
                    classified.push("missing_resource");
                } else if err_lower.contains("permission") || err_lower.contains("denied") {
                    classified.push("permission");
                } else if err_lower.contains("conflict") || err_lower.contains("contradict") {
                    classified.push("conflict");
                } else if err_lower.contains("parse")
                    || err_lower.contains("format")
                    || err_lower.contains("invalid")
                {
                    classified.push("parse_error");
                } else {
                    classified.push("unknown");
                }
            }

            let mut categories: HashMap<&str, u32> = HashMap::new();
            for c in &classified {
                *categories.entry(c).or_insert(0) += 1;
            }

            for (cat, count) in &categories {
                observations.push(format!(
                    "Error category '{}' occurred {} time(s)",
                    cat, count
                ));
            }

            if classified.iter().any(|c| *c == "timeout") {
                recommendations
                    .push("Increase timeout thresholds or pre-warm resources".to_string());
            }
            if classified.iter().any(|c| *c == "missing_resource") {
                recommendations.push("Add resource existence checks before access".to_string());
            }
            if classified.iter().any(|c| *c == "permission") {
                recommendations.push("Review access control configuration".to_string());
            }
            if classified.iter().any(|c| *c == "conflict") {
                recommendations.push("Implement conflict resolution strategy".to_string());
            }
            if classified.iter().any(|c| *c == "parse_error") {
                recommendations.push("Add input validation before processing".to_string());
            }

            if categories.len() == 1 && error_count > 3 {
                observations.push(format!(
                    "Recurring single-category error ({}x) — systemic issue",
                    error_count
                ));
                recommendations
                    .push("Treat this as a systemic bug, not transient failure".to_string());
                (0.3, 0.7)
            } else {
                let diversity_penalty = (categories.len() as f64) * 0.1;
                let count_penalty = (error_count as f64) * 0.1;
                let score = (0.7 - diversity_penalty - count_penalty).clamp(0.0, 1.0);
                (score, 0.6)
            }
        };

        let trace = self.build_trace(data, vec![format!("errors={}", error_count)], 0);
        ReflectionResult {
            dimension: ReflectionDimension::ErrorLearning,
            score,
            confidence: conf,
            observations,
            recommendations,
            severity: ReflectionSeverity::from_score(score),
            trace,
        }
    }

    /// Generate meta-cognitive advice based on reflection history
    pub fn generate_advice(&self) -> Vec<String> {
        let mut advice = Vec::new();

        if self.reflection_history.is_empty() {
            advice.push(
                "No reflection data available — run at least one reflection cycle".to_string(),
            );
            return advice;
        }

        let total = self.reflection_history.len() as f64;
        let critical_count = self
            .reflection_history
            .iter()
            .filter(|r| {
                r.severity == ReflectionSeverity::Critical
                    || r.severity == ReflectionSeverity::Catastrophic
            })
            .count();
        let warning_count = self
            .reflection_history
            .iter()
            .filter(|r| r.severity == ReflectionSeverity::Warning)
            .count();

        if critical_count > 0 {
            let pct = (critical_count as f64 / total * 100.0) as u32;
            advice.push(format!("{critical_count}/{total} reflections ({pct}%) are critical — immediate attention needed"));
        }

        if warning_count > 5 {
            advice.push(format!(
                "High warning rate ({warning_count}) — systematic issues likely"
            ));
        }

        for dim in &self.active_dimensions {
            let dim_results: Vec<&ReflectionResult> = self
                .reflection_history
                .iter()
                .filter(|r| r.dimension == *dim)
                .collect();

            if dim_results.is_empty() {
                continue;
            }

            let avg_score: f64 =
                dim_results.iter().map(|r| r.score).sum::<f64>() / dim_results.len() as f64;

            if avg_score < 0.3 {
                advice.push(format!(
                    "{} is critically low ({:.2}) — prioritize improvement",
                    dim.name(),
                    avg_score
                ));
            } else if avg_score < 0.5 {
                advice.push(format!(
                    "{} is below threshold ({:.2}) — consider intervention",
                    dim.name(),
                    avg_score
                ));
            }

            let dim_recs: Vec<&str> = dim_results
                .iter()
                .flat_map(|r| r.recommendations.iter().map(|s| s.as_str()))
                .collect();

            if dim_recs.len() > 3 {
                let mut rec_counts: HashMap<&str, u32> = HashMap::new();
                for r in &dim_recs {
                    *rec_counts.entry(r).or_insert(0) += 1;
                }
                let mut sorted: Vec<(&&str, &u32)> = rec_counts.iter().collect();
                sorted.sort_by(|a, b| b.1.cmp(a.1));
                if let Some((_top_rec, count)) = sorted.first() {
                    if **count >= 3u32 {
                        advice.push(format!(
                            "Recurring recommendation for {}: '{}' ({}x)",
                            dim.name(),
                            _top_rec,
                            count
                        ));
                    }
                }
            }
        }

        let Some(latest) = self.reflection_history.last() else { return advice; };
        for rec in &latest.recommendations {
            advice.push(format!("Latest advice: {}", rec));
        }

        if advice.is_empty() {
            advice.push("No specific issues detected — continue monitoring".to_string());
        }

        advice
    }

    /// Get overall meta-cognitive health score
    pub fn meta_health(&self) -> MetaHealth {
        let mut dimension_scores: HashMap<ReflectionDimension, f64> = HashMap::new();
        let mut warnings = 0u32;
        let mut criticals = 0u32;

        for dim in &self.active_dimensions {
            let dim_results: Vec<&ReflectionResult> = self
                .reflection_history
                .iter()
                .filter(|r| r.dimension == *dim)
                .collect();

            let avg = if dim_results.is_empty() {
                0.5
            } else {
                dim_results.iter().map(|r| r.score).sum::<f64>() / dim_results.len() as f64
            };
            dimension_scores.insert(*dim, avg);
        }

        for r in &self.reflection_history {
            match r.severity {
                ReflectionSeverity::Warning => warnings += 1,
                ReflectionSeverity::Critical | ReflectionSeverity::Catastrophic => criticals += 1,
                _ => {}
            }
        }

        let overall: f64 = if dimension_scores.is_empty() {
            0.5
        } else {
            dimension_scores.values().sum::<f64>() / dimension_scores.len() as f64
        };

        let trend_direction = self.improvement_trend(10);
        let trend_str = if trend_direction.len() >= 2 {
            let first = trend_direction.first().copied().unwrap_or(0.5);
            let last = trend_direction.last().copied().unwrap_or(0.5);
            let diff = last - first;
            if diff > 0.05 {
                "improving"
            } else if diff < -0.05 {
                "declining"
            } else {
                "stable"
            }
        } else {
            "stable"
        };

        MetaHealth {
            overall_score: overall,
            dimension_scores,
            warning_count: warnings,
            critical_count: criticals,
            trend_direction: trend_str,
        }
    }

    /// Get improvement trend over last N reflections
    pub fn improvement_trend(&self, n: usize) -> Vec<f64> {
        let n = n.min(self.reflection_history.len());
        if n == 0 {
            return Vec::new();
        }

        let recent: Vec<&ReflectionResult> = self.reflection_history.iter().rev().take(n).collect();
        let mut trend = Vec::with_capacity(n);

        for window_start in 0..recent.len() {
            let window: Vec<&&ReflectionResult> =
                recent.iter().skip(window_start).take(3).collect();
            if window.is_empty() {
                continue;
            }
            let avg: f64 = window.iter().map(|r| r.score).sum::<f64>() / window.len() as f64;
            trend.push(avg);
        }

        trend.reverse();
        trend
    }

    /// Identify recurring failure patterns
    pub fn recurring_patterns(&self, min_occurrences: u32) -> Vec<RecurringPattern> {
        let mut pattern_counts: HashMap<String, (u32, Vec<ReflectionSeverity>)> = HashMap::new();

        for result in &self.reflection_history {
            for rec in &result.recommendations {
                let entry = pattern_counts.entry(rec.clone()).or_insert((0, Vec::new()));
                entry.0 += 1;
                entry.1.push(result.severity);
            }
        }

        let mut patterns: Vec<RecurringPattern> = pattern_counts
            .into_iter()
            .filter(|(_, (count, _))| *count >= min_occurrences)
            .map(|(rec, (count, sevs))| {
                let max_sev = sevs.into_iter().max().unwrap_or(ReflectionSeverity::Info);
                RecurringPattern {
                    pattern: rec.clone(),
                    occurrences: count,
                    recommended_action: rec,
                    severity: max_sev,
                }
            })
            .collect();

        patterns.sort_by(|a, b| b.occurrences.cmp(&a.occurrences));
        patterns
    }
}

// ── Part 5: MetaCognitionController ──

/// Controller that hooks into ConsciousnessCycle
#[derive(Debug, Clone)]
pub struct MetaCognitionController {
    pub engine: MetaReflectionEngine,
    pub enabled_dimensions: Vec<ReflectionDimension>,
    pub intervention_enabled: bool,
}

impl MetaCognitionController {
    pub fn new() -> Self {
        Self {
            engine: MetaReflectionEngine::new(),
            enabled_dimensions: ReflectionDimension::all(),
            intervention_enabled: true,
        }
    }

    pub fn with_intervention(mut self, enabled: bool) -> Self {
        self.intervention_enabled = enabled;
        self
    }

    pub fn with_dimensions(mut self, dims: Vec<ReflectionDimension>) -> Self {
        let d = dims.clone();
        self.enabled_dimensions = dims;
        self.engine.active_dimensions = d;
        self
    }

    /// Run reflection BEFORE action, can suggest alternatives
    pub fn before_action(&mut self, cycle_data: &CycleReflectionData) -> Option<Intervention> {
        let results = self.engine.reflect_on_cycle(cycle_data);
        if self.should_intervene(&results) {
            let critical = results
                .iter()
                .filter(|r| {
                    r.severity == ReflectionSeverity::Critical
                        || r.severity == ReflectionSeverity::Catastrophic
                })
                .max_by(|a, b| {
                    a.score
                        .partial_cmp(&b.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

            if let Some(crit) = critical {
                let primary_rec = crit.recommendations.first().cloned().unwrap_or_default();
                return Some(Intervention {
                    severity: crit.severity,
                    message: format!(
                        "Reflection intervention: {}",
                        crit.observations
                            .first()
                            .unwrap_or(&"issue detected".to_string())
                    ),
                    suggested_action: primary_rec,
                    confidence: crit.confidence,
                });
            }

            let worst = results.iter().min_by(|a, b| {
                a.score
                    .partial_cmp(&b.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            if let Some(w) = worst {
                let rec = w.recommendations.first().cloned().unwrap_or_default();
                return Some(Intervention {
                    severity: w.severity,
                    message: format!("Reflection warning: {}", w.dimension.name()),
                    suggested_action: rec,
                    confidence: w.confidence,
                });
            }
        }
        None
    }

    /// Run reflection AFTER action
    pub fn after_action(&mut self, outcome: CycleOutcome) -> Vec<ReflectionResult> {
        let data = CycleReflectionData {
            outcome: Some(outcome),
            ..CycleReflectionData::new(self.engine.cycle_count)
        };
        self.engine.reflect_on_cycle(&data)
    }

    /// Update internal models from action outcome
    pub fn learn_from_outcome(&mut self, _action: &str, _outcome: &CycleOutcome) {
        // placeholder for future: update confidence calibration models
    }

    /// Determine if reflection results warrant intervention
    pub fn should_intervene(&self, results: &[ReflectionResult]) -> bool {
        if !self.intervention_enabled {
            return false;
        }
        results.iter().any(|r| {
            r.severity == ReflectionSeverity::Critical
                || r.severity == ReflectionSeverity::Catastrophic
                || (r.severity == ReflectionSeverity::Warning
                    && r.score < self.engine.reflection_threshold)
        })
    }
}

/// An intervention suggested by the meta-cognition controller
#[derive(Debug, Clone)]
pub struct Intervention {
    pub severity: ReflectionSeverity,
    pub message: String,
    pub suggested_action: String,
    pub confidence: f64,
}

// ── Part 6: Default impls ──

impl Default for MetaReflectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MetaCognitionController {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CycleReflectionData {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data(cycle: u64, outcome: Option<CycleOutcome>) -> CycleReflectionData {
        CycleReflectionData {
            cycle_number: cycle,
            perception_quality: 0.8,
            retrieval_quality: 0.7,
            reasoning_paths: vec![
                ReasoningPathData {
                    path_id: 1,
                    steps: 5,
                    final_confidence: 0.85,
                    was_selected: true,
                    strengths: vec!["clear logic".into()],
                    weaknesses: vec!["missing counter-example".into()],
                },
                ReasoningPathData {
                    path_id: 2,
                    steps: 3,
                    final_confidence: 0.45,
                    was_selected: false,
                    strengths: vec![],
                    weaknesses: vec!["incomplete".into()],
                },
            ],
            chosen_action: "proceed_with_analysis".into(),
            action_confidence: 0.85,
            outcome,
            time_per_step_ms: vec![50, 30, 100, 200, 80],
            errors: Vec::new(),
        }
    }

    fn failing_data(cycle: u64) -> CycleReflectionData {
        CycleReflectionData {
            cycle_number: cycle,
            perception_quality: 0.2,
            retrieval_quality: 0.1,
            reasoning_paths: vec![ReasoningPathData {
                path_id: 1,
                steps: 1,
                final_confidence: 0.95,
                was_selected: true,
                strengths: vec![],
                weaknesses: vec!["rushed".into()],
            }],
            chosen_action: "continue_failing_strategy".into(),
            action_confidence: 0.95,
            outcome: Some(CycleOutcome::Failure("timeout".into())),
            time_per_step_ms: vec![5000],
            errors: vec![
                "timeout during reasoning".into(),
                "conflict detected".into(),
                "parse error in input".into(),
            ],
        }
    }

    // ── ReflectionDimension tests ──

    #[test]
    fn test_reflection_dimension_all_variants() {
        let all = ReflectionDimension::all();
        assert_eq!(all.len(), 9);
        assert!(all.contains(&ReflectionDimension::ReasoningQuality));
        assert!(all.contains(&ReflectionDimension::ConfidenceCalibration));
        assert!(all.contains(&ReflectionDimension::TemporalConsistency));
        assert!(all.contains(&ReflectionDimension::BiasDetection));
        assert!(all.contains(&ReflectionDimension::AssumptionValidity));
        assert!(all.contains(&ReflectionDimension::Counterfactuals));
        assert!(all.contains(&ReflectionDimension::ResourceEfficiency));
        assert!(all.contains(&ReflectionDimension::ValueAlignment));
        assert!(all.contains(&ReflectionDimension::ErrorLearning));
    }

    #[test]
    fn test_dimension_name_unique() {
        let all = ReflectionDimension::all();
        let mut names = std::collections::HashSet::new();
        for d in &all {
            assert!(names.insert(d.name()), "duplicate name for {:?}", d);
        }
    }

    // ── MetaReflectionEngine tests ──

    #[test]
    fn test_meta_engine_new() {
        let engine = MetaReflectionEngine::new();
        assert!(engine.reflection_history.is_empty());
        assert_eq!(engine.active_dimensions.len(), 9);
        assert!((engine.reflection_threshold - 0.5).abs() < 1e-6);
        assert_eq!(engine.max_history, 500);
    }

    #[test]
    fn test_reflect_reasoning_quality_high() {
        let engine = MetaReflectionEngine::new();
        let data = sample_data(1, Some(CycleOutcome::Success));
        let result = engine.reflect_reasoning_quality(&data);
        assert!(
            result.score >= 0.7,
            "expected high score, got {}",
            result.score
        );
        assert_eq!(result.dimension, ReflectionDimension::ReasoningQuality);
        assert!(result.observations.iter().any(|o| o.contains("good")));
    }

    #[test]
    fn test_reflect_reasoning_quality_low() {
        let engine = MetaReflectionEngine::new();
        let data = CycleReflectionData {
            reasoning_paths: vec![],
            ..sample_data(1, Some(CycleOutcome::Failure("bad".into())))
        };
        let result = engine.reflect_reasoning_quality(&data);
        assert!(
            result.score < 0.5,
            "expected low score, got {}",
            result.score
        );
        assert!(
            !result.recommendations.is_empty(),
            "should have recommendations"
        );
    }

    #[test]
    fn test_reflect_confidence_miscalibrated() {
        let engine = MetaReflectionEngine::new();
        let data = CycleReflectionData {
            action_confidence: 0.95,
            outcome: Some(CycleOutcome::Failure("overconfident".into())),
            ..sample_data(2, None)
        };
        let result = engine.reflect_confidence(&data);
        assert!(
            result.score < 0.5,
            "expected low score for miscalibration, got {}",
            result.score
        );
        assert!(result
            .observations
            .iter()
            .any(|o| o.contains("overconfidence")));
    }

    #[test]
    fn test_reflect_confidence_calibrated() {
        let engine = MetaReflectionEngine::new();
        let data = CycleReflectionData {
            action_confidence: 0.6,
            outcome: Some(CycleOutcome::Success),
            ..sample_data(3, None)
        };
        let result = engine.reflect_confidence(&data);
        assert!(
            result.score >= 0.5,
            "expected OK score for calibration, got {}",
            result.score
        );
    }

    #[test]
    fn test_reflect_bias_detected() {
        let engine = MetaReflectionEngine::new();
        let data = CycleReflectionData {
            chosen_action: "prefer this path without checking others".into(),
            action_confidence: 0.95,
            reasoning_paths: vec![ReasoningPathData {
                path_id: 1,
                steps: 5,
                final_confidence: 0.9,
                was_selected: true,
                strengths: vec![],
                weaknesses: vec![],
            }],
            ..sample_data(4, Some(CycleOutcome::Success))
        };
        let result = engine.reflect_bias(&data);
        assert!(
            result.score < 0.7,
            "expected bias detected, got score {}",
            result.score
        );
        assert!(
            !result.recommendations.is_empty(),
            "bias should generate recommendations"
        );
    }

    #[test]
    fn test_reflect_assumptions_valid() {
        let engine = MetaReflectionEngine::new();
        let data = CycleReflectionData {
            chosen_action: "execute_verified_plan".into(),
            perception_quality: 0.9,
            retrieval_quality: 0.9,
            reasoning_paths: vec![ReasoningPathData {
                path_id: 1,
                steps: 5,
                final_confidence: 0.8,
                was_selected: true,
                strengths: vec![],
                weaknesses: vec![],
            }],
            ..sample_data(5, None)
        };
        let result = engine.reflect_assumptions(&data);
        assert!(
            result.score >= 0.7,
            "expected high score for valid assumptions, got {}",
            result.score
        );
    }

    #[test]
    fn test_reflect_assumptions_invalid() {
        let engine = MetaReflectionEngine::new();
        let data = CycleReflectionData {
            chosen_action: "assume this will always work".into(),
            perception_quality: 0.1,
            retrieval_quality: 0.1,
            ..sample_data(6, None)
        };
        let result = engine.reflect_assumptions(&data);
        assert!(
            result.score < 0.5,
            "expected low score for bad assumptions, got {}",
            result.score
        );
    }

    #[test]
    fn test_reflect_counterfactuals_not_considered() {
        let engine = MetaReflectionEngine::new();
        let data = CycleReflectionData {
            reasoning_paths: vec![],
            ..sample_data(7, None)
        };
        let result = engine.reflect_counterfactuals(&data);
        assert!(
            result.score < 0.5,
            "expected low score without alternatives, got {}",
            result.score
        );
    }

    #[test]
    fn test_reflect_efficiency_poor() {
        let engine = MetaReflectionEngine::new();
        let data = CycleReflectionData {
            time_per_step_ms: vec![5000, 6000],
            ..sample_data(8, None)
        };
        let result = engine.reflect_efficiency(&data);
        assert!(
            result.score < 0.6,
            "expected moderate-low score for poor efficiency, got {}",
            result.score
        );
    }

    #[test]
    fn test_reflect_alignment_ok() {
        let engine = MetaReflectionEngine::new();
        let data = sample_data(9, None);
        let result = engine.reflect_alignment(&data);
        assert!(
            result.score >= 0.7,
            "expected high alignment score, got {}",
            result.score
        );
    }

    // ── Full cycle reflection tests ──

    #[test]
    fn test_reflect_on_cycle_all_dimensions() {
        let mut engine = MetaReflectionEngine::new();
        let data = sample_data(1, Some(CycleOutcome::Success));
        let results = engine.reflect_on_cycle(&data);
        assert_eq!(results.len(), 9, "all 9 dimensions should reflect");
        assert_eq!(engine.cycle_count, 1);
        assert_eq!(engine.reflection_history.len(), 9);
    }

    #[test]
    fn test_history_capped() {
        let mut engine = MetaReflectionEngine::new();
        engine.max_history = 5;
        for i in 0..3 {
            let data = sample_data(i as u64, Some(CycleOutcome::Success));
            engine.reflect_on_cycle(&data);
        }
        assert_eq!(engine.reflection_history.len(), 27);
    }

    // ── Advice tests ──

    #[test]
    fn test_generate_advice_from_history() {
        let mut engine = MetaReflectionEngine::new();
        let data = sample_data(1, Some(CycleOutcome::Success));
        engine.reflect_on_cycle(&data);

        let fail = failing_data(2);
        engine.reflect_on_cycle(&fail);

        let advice = engine.generate_advice();
        assert!(
            !advice.is_empty(),
            "should generate advice from reflections"
        );
    }

    #[test]
    fn test_generate_advice_empty() {
        let engine = MetaReflectionEngine::new();
        let advice = engine.generate_advice();
        assert_eq!(advice.len(), 1);
        assert!(advice[0].contains("No reflection data"));
    }

    // ── MetaHealth tests ──

    #[test]
    fn test_meta_health_score_computation() {
        let mut engine = MetaReflectionEngine::new();
        let data = sample_data(1, Some(CycleOutcome::Success));
        engine.reflect_on_cycle(&data);

        let health = engine.meta_health();
        assert!(health.overall_score >= 0.0 && health.overall_score <= 1.0);
        assert_eq!(health.dimension_scores.len(), 9);
        assert!(
            health.trend_direction == "improving"
                || health.trend_direction == "stable"
                || health.trend_direction == "declining"
        );
    }

    // ── Recurring pattern tests ──

    #[test]
    fn test_recurring_pattern_detection() {
        let mut engine = MetaReflectionEngine::new();
        for i in 0..4 {
            let mut data = failing_data(i);
            data.cycle_number = i;
            engine.reflect_on_cycle(&data);
        }

        let patterns = engine.recurring_patterns(3);
        assert!(
            !patterns.is_empty(),
            "should detect patterns from 4 failing cycles"
        );
        for p in &patterns {
            assert!(p.occurrences >= 3);
        }
    }

    #[test]
    fn test_recurring_pattern_min_occurrences() {
        let engine = MetaReflectionEngine::new();
        let patterns = engine.recurring_patterns(5);
        assert!(
            patterns.is_empty(),
            "no patterns should match with empty history"
        );
    }

    // ── Improvement trend tests ──

    #[test]
    fn test_improvement_trend() {
        let mut engine = MetaReflectionEngine::new();
        let data = sample_data(1, Some(CycleOutcome::Success));
        engine.reflect_on_cycle(&data);

        let trend = engine.improvement_trend(5);
        assert!(!trend.is_empty());
    }

    #[test]
    fn test_improvement_trend_empty() {
        let engine = MetaReflectionEngine::new();
        let trend = engine.improvement_trend(10);
        assert!(trend.is_empty());
    }

    // ── Severity tests ──

    #[test]
    fn test_severity_ordering() {
        assert!(ReflectionSeverity::Info < ReflectionSeverity::Warning);
        assert!(ReflectionSeverity::Warning < ReflectionSeverity::Critical);
        assert!(ReflectionSeverity::Critical < ReflectionSeverity::Catastrophic);
    }

    #[test]
    fn test_severity_from_score() {
        assert_eq!(
            ReflectionSeverity::from_score(0.9),
            ReflectionSeverity::Info
        );
        assert_eq!(
            ReflectionSeverity::from_score(0.6),
            ReflectionSeverity::Warning
        );
        assert_eq!(
            ReflectionSeverity::from_score(0.3),
            ReflectionSeverity::Critical
        );
        assert_eq!(
            ReflectionSeverity::from_score(0.1),
            ReflectionSeverity::Catastrophic
        );
    }

    #[test]
    fn test_severity_label() {
        assert_eq!(ReflectionSeverity::Info.label(), "info");
        assert_eq!(ReflectionSeverity::Warning.label(), "warning");
        assert_eq!(ReflectionSeverity::Critical.label(), "critical");
        assert_eq!(ReflectionSeverity::Catastrophic.label(), "catastrophic");
    }

    // ── ReflectionResult construction tests ──

    #[test]
    fn test_reflection_result_construction() {
        let trace = ReflectionTrace::new(1, "test_action");
        let result = ReflectionResult {
            dimension: ReflectionDimension::ReasoningQuality,
            score: 0.85,
            confidence: 0.9,
            observations: vec!["good reasoning".into()],
            recommendations: vec!["keep going".into()],
            severity: ReflectionSeverity::Info,
            trace,
        };
        assert_eq!(result.score, 0.85);
        assert_eq!(result.dimension, ReflectionDimension::ReasoningQuality);
        assert_eq!(result.severity, ReflectionSeverity::Info);
    }

    // ── MetaCognitionController tests ──

    #[test]
    fn test_meta_cognition_controller_intervene() {
        let mut controller = MetaCognitionController::new();
        let data = failing_data(1);
        let intervention = controller.before_action(&data);
        assert!(intervention.is_some(), "should intervene on failing data");
        let intervention = intervention.unwrap();
        assert!(
            intervention.severity == ReflectionSeverity::Critical
                || intervention.severity == ReflectionSeverity::Catastrophic
        );
        assert!(!intervention.suggested_action.is_empty());
    }

    #[test]
    fn test_meta_cognition_controller_no_intervene() {
        let mut controller = MetaCognitionController::new();
        let data = sample_data(1, Some(CycleOutcome::Success));
        let intervention = controller.before_action(&data);
        assert!(intervention.is_none(), "should not intervene on good data");
    }

    #[test]
    fn test_controller_intervention_disabled() {
        let mut controller = MetaCognitionController::new().with_intervention(false);
        let data = failing_data(1);
        let intervention = controller.before_action(&data);
        assert!(
            intervention.is_none(),
            "intervention disabled → no intervention"
        );
    }

    #[test]
    fn test_controller_after_action() {
        let mut controller = MetaCognitionController::new();
        let results = controller.after_action(CycleOutcome::Success);
        assert_eq!(results.len(), 9);
    }

    #[test]
    fn test_controller_learn_from_outcome() {
        let mut controller = MetaCognitionController::new();
        controller.learn_from_outcome("test_action", &CycleOutcome::Success);
        // should not panic
    }

    // ── Builder tests ──

    #[test]
    fn test_engine_with_dimensions() {
        let dims = vec![
            ReflectionDimension::ReasoningQuality,
            ReflectionDimension::BiasDetection,
        ];
        let engine = MetaReflectionEngine::new().with_dimensions(dims.clone());
        assert_eq!(engine.active_dimensions, dims);
    }

    #[test]
    fn test_engine_with_threshold() {
        let engine = MetaReflectionEngine::new().with_threshold(0.8);
        assert!((engine.reflection_threshold - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_engine_with_threshold_clamped() {
        let engine = MetaReflectionEngine::new().with_threshold(1.5);
        assert!((engine.reflection_threshold - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_controller_with_dimensions() {
        let dims = vec![ReflectionDimension::ReasoningQuality];
        let controller = MetaCognitionController::new().with_dimensions(dims.clone());
        assert_eq!(controller.enabled_dimensions, dims);
    }

    // ── Edge cases ──

    #[test]
    fn test_empty_reasoning_paths() {
        let engine = MetaReflectionEngine::new();
        let data = CycleReflectionData::new(1);
        let result = engine.reflect_reasoning_quality(&data);
        assert!(result.score < 0.5);
    }

    #[test]
    fn test_no_outcome_confidence() {
        let engine = MetaReflectionEngine::new();
        let data = CycleReflectionData {
            action_confidence: 0.5,
            outcome: None,
            ..CycleReflectionData::new(1)
        };
        let result = engine.reflect_confidence(&data);
        assert!((result.score - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_error_learning_no_errors() {
        let engine = MetaReflectionEngine::new();
        let data = CycleReflectionData::new(1);
        let result = engine.reflect_error_learning(&data);
        assert!(result.score >= 0.7);
    }

    #[test]
    fn test_error_learning_with_errors() {
        let engine = MetaReflectionEngine::new();
        let data = CycleReflectionData {
            errors: vec![
                "timeout during request".into(),
                "parse error in response".into(),
                "conflict with constraint".into(),
            ],
            ..CycleReflectionData::new(1)
        };
        let result = engine.reflect_error_learning(&data);
        assert!(
            result.score < 0.7,
            "expected lower score with errors, got {}",
            result.score
        );
        assert!(!result.observations.is_empty());
    }

    #[test]
    fn test_temporal_consistency_insufficient_data() {
        let engine = MetaReflectionEngine::new();
        let data = CycleReflectionData::new(1);
        let result = engine.reflect_temporal_consistency(&data);
        assert!(
            (result.score - 0.7).abs() < 0.01,
            "expected default 0.7, got {}",
            result.score
        );
    }

    #[test]
    fn test_should_intervene_returns_false_when_disabled() {
        let controller = MetaCognitionController::new().with_intervention(false);
        let results = vec![ReflectionResult {
            dimension: ReflectionDimension::ReasoningQuality,
            score: 0.1,
            confidence: 0.9,
            observations: vec![],
            recommendations: vec![],
            severity: ReflectionSeverity::Catastrophic,
            trace: ReflectionTrace::new(1, ""),
        }];
        assert!(!controller.should_intervene(&results));
    }

    #[test]
    fn test_should_intervene_returns_true_on_critical() {
        let controller = MetaCognitionController::new();
        let results = vec![ReflectionResult {
            dimension: ReflectionDimension::ReasoningQuality,
            score: 0.1,
            confidence: 0.9,
            observations: vec![],
            recommendations: vec![],
            severity: ReflectionSeverity::Critical,
            trace: ReflectionTrace::new(1, ""),
        }];
        assert!(controller.should_intervene(&results));
    }

    #[test]
    fn test_reflection_trace_fields() {
        let trace = ReflectionTrace::new(42, "test_input");
        assert_eq!(trace.cycle_number, 42);
        assert_eq!(trace.input_summary, "test_input");
        assert!(trace.timestamp.len() > 0);
    }

    #[test]
    fn test_cycle_type_debug_clone() {
        let data = sample_data(1, Some(CycleOutcome::Success));
        let cloned = data.clone();
        assert_eq!(cloned.cycle_number, data.cycle_number);
    }

    #[test]
    fn test_cycle_outcome_variants() {
        match CycleOutcome::Success {
            CycleOutcome::Success => {}
            _ => panic!("expected Success"),
        }
        match CycleOutcome::PartialSuccess("partial".into()) {
            CycleOutcome::PartialSuccess(_) => {}
            _ => panic!("expected PartialSuccess"),
        }
        match CycleOutcome::Failure("fail".into()) {
            CycleOutcome::Failure(_) => {}
            _ => panic!("expected Failure"),
        }
        match CycleOutcome::NotYetEvaluated {
            CycleOutcome::NotYetEvaluated => {}
            _ => panic!("expected NotYetEvaluated"),
        }
    }

    #[test]
    fn test_dimension_eq_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ReflectionDimension::BiasDetection);
        assert!(set.contains(&ReflectionDimension::BiasDetection));
        assert!(!set.contains(&ReflectionDimension::ReasoningQuality));
    }

    #[test]
    fn test_meta_health_with_empty_engine() {
        let engine = MetaReflectionEngine::new();
        let health = engine.meta_health();
        assert!((health.overall_score - 0.5).abs() < 1e-6);
        assert_eq!(health.warning_count, 0);
        assert_eq!(health.critical_count, 0);
    }

    #[test]
    fn test_reflection_severity_partial_ord() {
        let severities = vec![
            ReflectionSeverity::Info,
            ReflectionSeverity::Warning,
            ReflectionSeverity::Critical,
            ReflectionSeverity::Catastrophic,
        ];
        for i in 0..severities.len() - 1 {
            assert!(severities[i] < severities[i + 1]);
        }
    }

    #[test]
    fn test_before_action_with_warning_only() {
        let mut controller = MetaCognitionController::new();
        let data = CycleReflectionData {
            chosen_action: "prefer_this".into(),
            action_confidence: 0.95,
            reasoning_paths: vec![ReasoningPathData {
                path_id: 1,
                steps: 1,
                final_confidence: 0.95,
                was_selected: true,
                strengths: vec![],
                weaknesses: vec![],
            }],
            ..sample_data(1, Some(CycleOutcome::Success))
        };
        let intervention = controller.before_action(&data);
        assert!(intervention.is_some());
    }
}
