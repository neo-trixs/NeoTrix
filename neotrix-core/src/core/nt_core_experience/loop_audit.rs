use std::collections::HashMap;

/// A dimension of loop readiness
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ReadinessDimension {
    /// Clear, testable stop condition
    StopCondition,
    /// Independent verifier configured
    IndependentVerifier,
    /// State persistence
    StatePersistence,
    /// Token/cost budget set
    CostBudget,
    /// Work isolation (worktree/sandbox)
    WorkIsolation,
    /// Human gate for irreversible actions
    HumanGate,
    /// Failure mode handling
    FailureHandling,
    /// Token efficiency
    TokenEfficiency,
    /// Audit trail
    AuditTrail,
    /// Documentation
    Documentation,
}

impl ReadinessDimension {
    pub fn label(&self) -> &'static str {
        match self {
            Self::StopCondition => "clear stop condition",
            Self::IndependentVerifier => "independent verifier",
            Self::StatePersistence => "state persistence",
            Self::CostBudget => "cost budget",
            Self::WorkIsolation => "work isolation",
            Self::HumanGate => "human gate",
            Self::FailureHandling => "failure handling",
            Self::TokenEfficiency => "token efficiency",
            Self::AuditTrail => "audit trail",
            Self::Documentation => "documentation",
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            Self::StopCondition => 0.20,
            Self::IndependentVerifier => 0.15,
            Self::StatePersistence => 0.15,
            Self::CostBudget => 0.10,
            Self::WorkIsolation => 0.10,
            Self::HumanGate => 0.10,
            Self::FailureHandling => 0.08,
            Self::TokenEfficiency => 0.05,
            Self::AuditTrail => 0.04,
            Self::Documentation => 0.03,
        }
    }
}

/// The readiness level for a loop
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ReadinessLevel {
    /// Production-ready: all checks pass, has been running
    Production,
    /// Safe to run unattended for narrow tasks
    Supervised,
    /// Report-only: no autonomous action
    ReportOnly,
    /// Still in design, not ready to run
    Draft,
}

impl ReadinessLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Production => "Production (L3)",
            Self::Supervised => "Supervised (L2)",
            Self::ReportOnly => "Report Only (L1)",
            Self::Draft => "Draft (L0)",
        }
    }
}

/// Result of a readiness assessment
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReadinessReport {
    pub loop_name: String,
    pub overall_score: f64,
    pub level: ReadinessLevel,
    pub dimension_scores: HashMap<String, f64>,
    pub critical_gaps: Vec<String>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
    pub estimated_cost_per_run: Option<CostEstimate>,
}

/// Cost estimate for a loop run
#[derive(Debug, Clone, serde::Serialize)]
pub struct CostEstimate {
    pub estimated_tokens: u64,
    pub estimated_cost_usd: f64,
    pub risk_level: CostRiskLevel,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum CostRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Failure pattern detected in loop execution history
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FailurePattern {
    pub pattern_name: String,
    pub description: String,
    pub occurrence_count: u64,
    pub severity: FailureSeverity,
    pub mitigation: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FailureSeverity {
    Warning,
    Critical,
    Blocking,
}

/// The LoopAudit implements the **readiness scoring and safety evaluation**
/// primitive of loop engineering.
///
/// Inspired by:
/// - cobusgreyling/loop-engineering's `loop-audit` CLI (Readiness Score 0-100)
/// - KanakMalpani's Loop Evaluation Standard (LES 1.0) — 8-dimension scoring
/// - millrace's governed loop pattern
///
/// Before a loop runs unattended, LoopAudit evaluates its safety, cost risk,
/// and readiness, returning a ReadinessLevel with actionable recommendations.
#[derive(Debug)]
pub struct LoopAudit {
    dimension_weights: HashMap<ReadinessDimension, f64>,
    failure_patterns: Vec<FailurePattern>,
    token_cost_per_token: f64,
}

impl LoopAudit {
    pub fn new() -> Self {
        let weights = HashMap::from([
            (ReadinessDimension::StopCondition, 0.20),
            (ReadinessDimension::IndependentVerifier, 0.15),
            (ReadinessDimension::StatePersistence, 0.15),
            (ReadinessDimension::CostBudget, 0.10),
            (ReadinessDimension::WorkIsolation, 0.10),
            (ReadinessDimension::HumanGate, 0.10),
            (ReadinessDimension::FailureHandling, 0.08),
            (ReadinessDimension::TokenEfficiency, 0.05),
            (ReadinessDimension::AuditTrail, 0.04),
            (ReadinessDimension::Documentation, 0.03),
        ]);

        Self {
            dimension_weights: weights,
            failure_patterns: Vec::new(),
            token_cost_per_token: 0.000015,
        }
    }

    /// Assess readiness of a loop given its characteristics
    pub fn assess_readiness(
        &self,
        loop_name: &str,
        scores: Vec<(ReadinessDimension, f64)>,
        estimated_steps: u64,
        estimated_tokens_per_step: u64,
    ) -> ReadinessReport {
        let mut dimension_scores = HashMap::new();
        let mut weighted_total = 0.0;
        let mut weight_sum = 0.0;
        let mut critical_gaps = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        for (dim, score) in &scores {
            let score = score.min(1.0).max(0.0);
            let dim_name = dim.label().to_string();
            dimension_scores.insert(dim_name.clone(), score);
            let weight = dim.weight();
            weighted_total += weight * score;
            weight_sum += weight;

            if score < 0.3 {
                critical_gaps.push(format!("{} ({:.0}%)", dim.label(), score * 100.0));
                recommendations.push(format!(
                    "improve {} — current score {:.0}%",
                    dim.label(),
                    score * 100.0
                ));
            } else if score < 0.6 {
                warnings.push(format!("{} score {:.0}%", dim.label(), score * 100.0));
            }
        }

        let overall_score = if weight_sum > 0.0 {
            weighted_total / weight_sum
        } else {
            0.0
        };

        let level = if overall_score >= 0.8 && critical_gaps.is_empty() {
            ReadinessLevel::Production
        } else if overall_score >= 0.5 {
            ReadinessLevel::Supervised
        } else if overall_score >= 0.3 {
            ReadinessLevel::ReportOnly
        } else {
            ReadinessLevel::Draft
        };

        let cost_estimate = if estimated_steps > 0 && estimated_tokens_per_step > 0 {
            let total_tokens = estimated_steps * estimated_tokens_per_step;
            let cost = total_tokens as f64 * self.token_cost_per_token;
            let risk = if cost > 10.0 {
                CostRiskLevel::Critical
            } else if cost > 5.0 {
                CostRiskLevel::High
            } else if cost > 1.0 {
                CostRiskLevel::Medium
            } else {
                CostRiskLevel::Low
            };
            Some(CostEstimate {
                estimated_tokens: total_tokens,
                estimated_cost_usd: cost,
                risk_level: risk,
            })
        } else {
            None
        };

        ReadinessReport {
            loop_name: loop_name.to_string(),
            overall_score,
            level,
            dimension_scores,
            critical_gaps,
            warnings,
            recommendations,
            estimated_cost_per_run: cost_estimate,
        }
    }

    pub fn register_failure_pattern(
        &mut self,
        name: &str,
        description: &str,
        severity: FailureSeverity,
        mitigation: &str,
    ) {
        self.failure_patterns.push(FailurePattern {
            pattern_name: name.to_string(),
            description: description.to_string(),
            occurrence_count: 0,
            severity,
            mitigation: mitigation.to_string(),
        });
    }

    pub fn record_failure(&mut self, pattern_name: &str) {
        if let Some(p) = self
            .failure_patterns
            .iter_mut()
            .find(|fp| fp.pattern_name == pattern_name)
        {
            p.occurrence_count += 1;
        }
    }

    pub fn failure_report(&self) -> Vec<&FailurePattern> {
        self.failure_patterns
            .iter()
            .filter(|fp| fp.occurrence_count > 0)
            .collect()
    }

    pub fn known_failure_patterns(&self) -> &[FailurePattern] {
        &self.failure_patterns
    }

    pub fn stats(&self) -> AuditStats {
        AuditStats {
            known_patterns: self.failure_patterns.len(),
            active_failures: self
                .failure_patterns
                .iter()
                .filter(|p| p.occurrence_count > 0)
                .count(),
            total_occurrences: self
                .failure_patterns
                .iter()
                .map(|p| p.occurrence_count)
                .sum(),
            token_cost_per_token: self.token_cost_per_token,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AuditStats {
    pub known_patterns: usize,
    pub active_failures: usize,
    pub total_occurrences: u64,
    pub token_cost_per_token: f64,
}

impl Default for LoopAudit {
    fn default() -> Self {
        let mut audit = Self::new();
        audit.register_failure_pattern(
            "infinite_fix_loop",
            "Agent repeatedly attempts the same fix without progress",
            FailureSeverity::Critical,
            "Add no-progress detection: if 3 consecutive attempts show no score improvement, halt",
        );
        audit.register_failure_pattern(
            "verifier_theater",
            "Verifier passes but output is actually wrong (rubric too weak)",
            FailureSeverity::Critical,
            "Strengthen the rubric with more specific criteria, use separate model for verification",
        );
        audit.register_failure_pattern(
            "token_furnace",
            "Loop consumes excessive tokens without producing useful output",
            FailureSeverity::Warning,
            "Set hard token budget per run and per day; measure token-per-useful-outcome ratio",
        );
        audit.register_failure_pattern(
            "context_collapse",
            "Agent forgets initial goal after many iterations",
            FailureSeverity::Warning,
            "Externalize state to disk; re-read goal statement every N iterations",
        );
        audit.register_failure_pattern(
            "scope_creep",
            "Loop expands beyond original objective",
            FailureSeverity::Warning,
            "Define scope boundaries in loop objective; halt on out-of-scope detection",
        );
        audit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_new() {
        let a = LoopAudit::new();
        assert_eq!(a.known_failure_patterns().len(), 5);
    }

    #[test]
    fn test_audit_assess_production() {
        let a = LoopAudit::new();
        let scores = vec![
            (ReadinessDimension::StopCondition, 1.0),
            (ReadinessDimension::IndependentVerifier, 0.9),
            (ReadinessDimension::StatePersistence, 1.0),
            (ReadinessDimension::CostBudget, 0.8),
            (ReadinessDimension::WorkIsolation, 0.9),
            (ReadinessDimension::HumanGate, 0.8),
            (ReadinessDimension::FailureHandling, 0.9),
            (ReadinessDimension::TokenEfficiency, 0.8),
            (ReadinessDimension::AuditTrail, 0.9),
            (ReadinessDimension::Documentation, 0.7),
        ];
        let report = a.assess_readiness("test_loop", scores, 10, 5000);
        assert_eq!(report.level, ReadinessLevel::Production);
        assert!(report.overall_score >= 0.8);
    }

    #[test]
    fn test_audit_assess_draft() {
        let a = LoopAudit::new();
        let scores = vec![
            (ReadinessDimension::StopCondition, 0.1),
            (ReadinessDimension::IndependentVerifier, 0.0),
            (ReadinessDimension::StatePersistence, 0.2),
        ];
        let report = a.assess_readiness("draft_loop", scores, 0, 0);
        assert_eq!(report.level, ReadinessLevel::Draft);
    }

    #[test]
    fn test_audit_assess_supervised() {
        let a = LoopAudit::new();
        let scores = vec![
            (ReadinessDimension::StopCondition, 0.7),
            (ReadinessDimension::IndependentVerifier, 0.4),
            (ReadinessDimension::StatePersistence, 0.6),
            (ReadinessDimension::CostBudget, 0.3),
            (ReadinessDimension::WorkIsolation, 0.0),
        ];
        let report = a.assess_readiness("mid_loop", scores, 5, 2000);
        assert_eq!(report.level, ReadinessLevel::Supervised);
        assert!(report.estimated_cost_per_run.is_some());
    }

    #[test]
    fn test_audit_assess_report_only() {
        let a = LoopAudit::new();
        let scores = vec![
            (ReadinessDimension::StopCondition, 0.4),
            (ReadinessDimension::IndependentVerifier, 0.2),
        ];
        let report = a.assess_readiness("report_only", scores, 0, 0);
        assert_eq!(report.level, ReadinessLevel::ReportOnly);
    }

    #[test]
    fn test_audit_register_failure_pattern() {
        let mut a = LoopAudit::new();
        a.register_failure_pattern(
            "test_pattern",
            "test description",
            FailureSeverity::Warning,
            "test mitigation",
        );
        assert_eq!(a.known_failure_patterns().len(), 6);
    }

    #[test]
    fn test_audit_record_failure() {
        let mut a = LoopAudit::default();
        a.record_failure("infinite_fix_loop");
        a.record_failure("infinite_fix_loop");
        let failures = a.failure_report();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].occurrence_count, 2);
    }

    #[test]
    fn test_audit_cost_estimate() {
        let a = LoopAudit::new();
        let scores = vec![(ReadinessDimension::StopCondition, 0.5)];
        let report = a.assess_readiness("costly_loop", scores, 100, 10000);
        let cost = report.estimated_cost_per_run.unwrap();
        assert_eq!(cost.estimated_tokens, 1_000_000);
        assert!((cost.estimated_cost_usd - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_audit_critical_gaps() {
        let a = LoopAudit::new();
        let scores = vec![
            (ReadinessDimension::StopCondition, 0.1),
            (ReadinessDimension::IndependentVerifier, 0.9),
            (ReadinessDimension::HumanGate, 0.2),
        ];
        let report = a.assess_readiness("gappy_loop", scores, 0, 0);
        assert!(!report.critical_gaps.is_empty());
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_audit_stats() {
        let mut a = LoopAudit::default();
        a.record_failure("infinite_fix_loop");
        let stats = a.stats();
        assert_eq!(stats.known_patterns, 5);
        assert_eq!(stats.active_failures, 1);
        assert_eq!(stats.total_occurrences, 1);
    }

    #[test]
    fn test_readiness_dimension_labels() {
        assert_eq!(
            ReadinessDimension::StopCondition.label(),
            "clear stop condition"
        );
        assert_eq!(
            ReadinessDimension::IndependentVerifier.label(),
            "independent verifier"
        );
        assert!(!ReadinessDimension::CostBudget.label().is_empty());
    }

    #[test]
    fn test_readiness_level_labels() {
        assert_eq!(ReadinessLevel::Production.label(), "Production (L3)");
        assert_eq!(ReadinessLevel::Draft.label(), "Draft (L0)");
    }
}
