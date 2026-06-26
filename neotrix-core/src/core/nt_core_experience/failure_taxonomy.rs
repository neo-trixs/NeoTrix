// REVIVED Evo 4
use std::collections::HashMap;

/// Types of failure modes tracked by the classifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum FailureModeType {
    IncorrectOutput,
    ValidationFail,
    ResourceExhaust,
    Timeout,
    LogicError,
    SyntaxError,
    KnowledgeGap,
    RepeatedFailure,
}

impl FailureModeType {
    pub fn label(&self) -> &'static str {
        match self {
            FailureModeType::IncorrectOutput => "incorrect_output",
            FailureModeType::ValidationFail => "validation_fail",
            FailureModeType::ResourceExhaust => "resource_exhaust",
            FailureModeType::Timeout => "timeout",
            FailureModeType::LogicError => "logic_error",
            FailureModeType::SyntaxError => "syntax_error",
            FailureModeType::KnowledgeGap => "knowledge_gap",
            FailureModeType::RepeatedFailure => "repeated_failure",
        }
    }

    pub fn all() -> Vec<FailureModeType> {
        vec![
            FailureModeType::IncorrectOutput,
            FailureModeType::ValidationFail,
            FailureModeType::ResourceExhaust,
            FailureModeType::Timeout,
            FailureModeType::LogicError,
            FailureModeType::SyntaxError,
            FailureModeType::KnowledgeGap,
            FailureModeType::RepeatedFailure,
        ]
    }
}

/// Per-mode tracking data.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FailureModeStats {
    pub mode: FailureModeType,
    pub count: u64,
    pub repair_success: u64,
    pub repair_attempts: u64,
    pub last_seen: u64,
}

impl FailureModeStats {
    pub fn success_rate(&self) -> f64 {
        if self.repair_attempts == 0 {
            0.0
        } else {
            self.repair_success as f64 / self.repair_attempts as f64
        }
    }
}

/// Configuration for the failure mode classifier.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ClassifierConfig {
    pub collapse_threshold: f64,
    pub window_size: usize,
}

impl Default for ClassifierConfig {
    fn default() -> Self {
        Self {
            collapse_threshold: 0.7,
            window_size: 100,
        }
    }
}

/// FailureModeClassifier — tracks per-mode failure rates, detects mode collapse,
/// and measures repair effectiveness per failure type.
///
/// Distilled from: arXiv:2606.05228 — LLM Failure Taxonomy on Competitive Programming.
/// Adapts the key finding (Wrong Answer dominates >90%; CoT can hurt) into a generic
/// failure mode classification system with mode collapse detection and remediation tracking.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FailureModeClassifier {
    modes: HashMap<FailureModeType, FailureModeStats>,
    total_failures: u64,
    config: ClassifierConfig,
    cycle: u64,
}

impl FailureModeClassifier {
    pub fn new() -> Self {
        let mut modes = HashMap::new();
        for m in FailureModeType::all() {
            modes.insert(
                m,
                FailureModeStats {
                    mode: m,
                    count: 0,
                    repair_success: 0,
                    repair_attempts: 0,
                    last_seen: 0,
                },
            );
        }
        Self {
            modes,
            total_failures: 0,
            config: ClassifierConfig::default(),
            cycle: 0,
        }
    }

    pub fn with_config(config: ClassifierConfig) -> Self {
        let mut c = Self::new();
        c.config = config;
        c
    }

    /// Record a failure of a given mode.
    pub fn record_failure(&mut self, mode: FailureModeType, repair_succeeded: bool) {
        let m = self.modes.entry(mode).or_insert(FailureModeStats {
            mode,
            count: 0,
            repair_success: 0,
            repair_attempts: 0,
            last_seen: 0,
        });
        m.count += 1;
        m.last_seen = self.cycle;
        if repair_succeeded {
            m.repair_success += 1;
        }
        m.repair_attempts += 1;
        self.total_failures += 1;
    }

    /// Record a failure without known repair outcome.
    pub fn record_failure_raw(&mut self, mode: FailureModeType) {
        let m = self.modes.entry(mode).or_insert(FailureModeStats {
            mode,
            count: 0,
            repair_success: 0,
            repair_attempts: 0,
            last_seen: 0,
        });
        m.count += 1;
        m.last_seen = self.cycle;
        self.total_failures += 1;
    }

    /// Ratio of a specific mode among all failures.
    pub fn dominance_ratio(&self, mode: FailureModeType) -> f64 {
        if self.total_failures == 0 {
            return 0.0;
        }
        self.modes
            .get(&mode)
            .map(|m| m.count as f64 / self.total_failures as f64)
            .unwrap_or(0.0)
    }

    /// Returns the dominant failure mode (highest count).
    pub fn dominant_mode(&self) -> Option<FailureModeType> {
        self.modes.values().max_by_key(|m| m.count).map(|m| m.mode)
    }

    /// Returns true when the dominant mode exceeds the collapse threshold.
    pub fn mode_collapse_detected(&self) -> bool {
        self.dominant_mode()
            .map(|mode| self.dominance_ratio(mode) >= self.config.collapse_threshold)
            .unwrap_or(false)
    }

    /// Returns the dominant mode if collapse is detected.
    pub fn collapsed_mode(&self) -> Option<FailureModeType> {
        if self.mode_collapse_detected() {
            self.dominant_mode()
        } else {
            None
        }
    }

    /// Repair effectiveness for a specific mode.
    pub fn repair_effectiveness(&self, mode: FailureModeType) -> f64 {
        self.modes
            .get(&mode)
            .map(|m| m.success_rate())
            .unwrap_or(0.0)
    }

    /// Overall repair effectiveness across all modes.
    pub fn overall_repair_rate(&self) -> f64 {
        let total_attempts: u64 = self.modes.values().map(|m| m.repair_attempts).sum();
        let total_success: u64 = self.modes.values().map(|m| m.repair_success).sum();
        if total_attempts == 0 {
            0.0
        } else {
            total_success as f64 / total_attempts as f64
        }
    }

    /// Per-type breakdown.
    pub fn mode_breakdown(&self) -> Vec<&FailureModeStats> {
        let mut v: Vec<_> = self.modes.values().collect();
        v.sort_by(|a, b| b.count.cmp(&a.count));
        v
    }

    /// Get stats for a specific mode.
    pub fn stats_for(&self, mode: FailureModeType) -> Option<&FailureModeStats> {
        self.modes.get(&mode)
    }

    pub fn advance_cycle(&mut self) {
        self.cycle += 1;
    }

    pub fn stats(&self) -> ClassifierStats {
        ClassifierStats {
            total_failures: self.total_failures,
            dominant_mode: self.dominant_mode().map(|m| m.label().to_string()),
            mode_collapse: self.mode_collapse_detected(),
            collapsed_mode: self.collapsed_mode().map(|m| m.label().to_string()),
            overall_repair_rate: self.overall_repair_rate(),
            mode_count: self.modes.len(),
        }
    }
}

impl Default for FailureModeClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ClassifierStats {
    pub total_failures: u64,
    pub dominant_mode: Option<String>,
    pub mode_collapse: bool,
    pub collapsed_mode: Option<String>,
    pub overall_repair_rate: f64,
    pub mode_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_failure_increases_count() {
        let mut c = FailureModeClassifier::new();
        c.record_failure(FailureModeType::IncorrectOutput, true);
        assert_eq!(c.total_failures, 1);
        assert_eq!(
            c.stats_for(FailureModeType::IncorrectOutput).unwrap().count,
            1
        );
    }

    #[test]
    fn test_dominance_ratio() {
        let mut c = FailureModeClassifier::new();
        c.record_failure_raw(FailureModeType::IncorrectOutput);
        c.record_failure_raw(FailureModeType::IncorrectOutput);
        c.record_failure_raw(FailureModeType::KnowledgeGap);
        assert!((c.dominance_ratio(FailureModeType::IncorrectOutput) - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_mode_collapse_detected() {
        let mut c = FailureModeClassifier::with_config(ClassifierConfig {
            collapse_threshold: 0.5,
            ..ClassifierConfig::default()
        });
        c.record_failure_raw(FailureModeType::LogicError);
        c.record_failure_raw(FailureModeType::LogicError);
        c.record_failure_raw(FailureModeType::LogicError);
        c.record_failure_raw(FailureModeType::Timeout);
        assert!(c.mode_collapse_detected());
        assert_eq!(c.collapsed_mode(), Some(FailureModeType::LogicError));
    }

    #[test]
    fn test_no_collapse_when_balanced() {
        let mut c = FailureModeClassifier::new();
        c.record_failure_raw(FailureModeType::IncorrectOutput);
        c.record_failure_raw(FailureModeType::Timeout);
        c.record_failure_raw(FailureModeType::KnowledgeGap);
        assert!(!c.mode_collapse_detected());
    }

    #[test]
    fn test_repair_effectiveness() {
        let mut c = FailureModeClassifier::new();
        c.record_failure(FailureModeType::SyntaxError, true);
        c.record_failure(FailureModeType::SyntaxError, false);
        c.record_failure(FailureModeType::SyntaxError, true);
        let eff = c.repair_effectiveness(FailureModeType::SyntaxError);
        assert!((eff - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_overall_repair_rate() {
        let mut c = FailureModeClassifier::new();
        c.record_failure(FailureModeType::IncorrectOutput, true);
        c.record_failure(FailureModeType::Timeout, false);
        c.record_failure(FailureModeType::SyntaxError, true);
        c.record_failure(FailureModeType::KnowledgeGap, false);
        assert!((c.overall_repair_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_mode_breakdown_order() {
        let mut c = FailureModeClassifier::new();
        c.record_failure_raw(FailureModeType::LogicError);
        c.record_failure_raw(FailureModeType::LogicError);
        c.record_failure_raw(FailureModeType::IncorrectOutput);
        let breakdown = c.mode_breakdown();
        assert_eq!(breakdown[0].mode, FailureModeType::LogicError);
        assert_eq!(breakdown[1].mode, FailureModeType::IncorrectOutput);
    }

    #[test]
    fn test_advance_cycle() {
        let mut c = FailureModeClassifier::new();
        assert_eq!(c.stats().total_failures, 0);
        c.record_failure_raw(FailureModeType::Timeout);
        c.advance_cycle();
        assert_eq!(c.stats_for(FailureModeType::Timeout).unwrap().last_seen, 0);
    }

    #[test]
    fn test_dominant_mode_empty() {
        let c = FailureModeClassifier::new();
        assert!(c.dominant_mode().is_none());
        assert!(!c.mode_collapse_detected());
    }

    #[test]
    fn test_all_failure_types_registered() {
        let c = FailureModeClassifier::new();
        // All 8 types should have entries even with zero failures
        for m in FailureModeType::all() {
            assert!(c.modes.contains_key(&m));
        }
    }
}
