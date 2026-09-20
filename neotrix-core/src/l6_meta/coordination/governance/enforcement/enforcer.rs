//! Enforcement engine for governance policies (R-P141, R-P142).
//!
//! Evaluates actions against configured policies and produces enforcement
//! results with violations and warnings.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::policy::{EnforcementLevel, Policy, RuleSeverity};

/// A single violation detected during enforcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// ID of the policy that was violated.
    pub policy_id: String,
    /// ID of the specific rule that was violated.
    pub rule_id: String,
    /// Human-readable violation message.
    pub message: String,
    /// Severity of the violation.
    pub severity: RuleSeverity,
}

/// Result of an enforcement check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementResult {
    /// Whether the action is allowed to proceed.
    pub allowed: bool,
    /// Violations detected (blocking or advisory).
    pub violations: Vec<Violation>,
    /// Non-blocking warnings from advisory-level policies.
    pub warnings: Vec<String>,
}

impl EnforcementResult {
    /// Create a result that allows the action with no violations.
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            violations: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a result that blocks the action.
    pub fn blocked(violations: Vec<Violation>, warnings: Vec<String>) -> Self {
        Self {
            allowed: false,
            violations,
            warnings,
        }
    }

    /// Add a warning to this result.
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Whether any blocking violations exist.
    pub fn has_blocking_violations(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity >= RuleSeverity::High)
    }

    /// Count of all violations.
    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }
}

/// Governance enforcement engine.
///
/// Evaluates actions against a set of policies and determines whether
/// they are allowed based on the configured enforcement levels.
pub struct EnforcementEngine {
    policies: Vec<Policy>,
    total_checks: u64,
    total_violations: u64,
}

impl EnforcementEngine {
    /// Create a new enforcement engine with the given policies.
    pub fn new(policies: Vec<Policy>) -> Self {
        Self {
            policies,
            total_checks: 0,
            total_violations: 0,
        }
    }

    /// Check an action against all enabled policies.
    ///
    /// The `action` is a string describing what the agent intends to do.
    /// The `context` provides additional key-value pairs for rule evaluation
    /// (e.g., `{"agent_id": "agent_1", "target": "main"}`).
    pub fn check(&mut self, action: &str, context: &HashMap<String, String>) -> EnforcementResult {
        self.total_checks += 1;

        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        for policy in &self.policies {
            if !policy.enabled {
                continue;
            }

            for rule in policy.enabled_rules() {
                if self.evaluate_rule(rule, action, context) {
                    // Rule matched — action violates this rule
                    let violation = Violation {
                        policy_id: policy.id.clone(),
                        rule_id: rule.id.clone(),
                        message: format!(
                            "Rule '{}' violated: {} (action: {})",
                            rule.id, rule.condition, rule.action
                        ),
                        severity: rule.severity,
                    };

                    match policy.enforcement_level {
                        EnforcementLevel::Blocking => {
                            self.total_violations += 1;
                            violations.push(violation);
                        }
                        EnforcementLevel::Warning => {
                            self.total_violations += 1;
                            warnings.push(violation.message);
                        }
                        EnforcementLevel::Advisory => {
                            warnings.push(format!("[advisory] {}", violation.message));
                        }
                    }
                }
            }
        }

        let allowed = violations.is_empty();

        if !allowed {
            tracing::warn!(
                action = action,
                violation_count = violations.len(),
                "Enforcement: action blocked"
            );
        }

        EnforcementResult {
            allowed,
            violations,
            warnings,
        }
    }

    /// Add a policy to the engine.
    pub fn add_policy(&mut self, policy: Policy) {
        self.policies.push(policy);
    }

    /// Get all loaded policies.
    pub fn policies(&self) -> &[Policy] {
        &self.policies
    }

    /// Get enforcement statistics.
    pub fn stats(&self) -> EnforcementStats {
        EnforcementStats {
            total_checks: self.total_checks,
            total_violations: self.total_violations,
            policy_count: self.policies.iter().filter(|p| p.enabled).count(),
        }
    }

    /// Evaluate a single rule against an action and context.
    ///
    /// Returns `true` if the rule is violated (i.e., the condition matches).
    fn evaluate_rule(
        &self,
        rule: &super::policy::PolicyRule,
        action: &str,
        context: &HashMap<String, String>,
    ) -> bool {
        // Simple condition matching: check if action or context contains the condition
        // substring. This is a basic evaluator — real implementation would use
        // structured rule evaluation (AST, predicate logic, etc.).
        let condition_lower = rule.condition.to_lowercase();
        let action_lower = action.to_lowercase();

        // Check if action matches the condition
        if action_lower.contains(&condition_lower) {
            return true;
        }

        // Check if any context value matches the condition
        for value in context.values() {
            if value.to_lowercase().contains(&condition_lower) {
                return true;
            }
        }

        false
    }
}

/// Enforcement statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementStats {
    /// Total number of checks performed.
    pub total_checks: u64,
    /// Total number of violations detected.
    pub total_violations: u64,
    /// Number of enabled policies.
    pub policy_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l6_meta::coordination::governance::enforcement::policy::PolicyRule;

    fn test_policy() -> Policy {
        Policy::new(
            "no_unsafe",
            "No Unsafe Code",
            "Core code must not use unsafe",
            EnforcementLevel::Blocking,
        )
        .with_rule(PolicyRule::new(
            "no_unsafe_code",
            "unsafe",
            "Reject code containing unsafe blocks",
            RuleSeverity::Critical,
        ))
    }

    #[test]
    fn test_allowed_when_no_violations() {
        let mut engine = EnforcementEngine::new(vec![test_policy()]);
        let result = engine.check("deploy to staging", &HashMap::new());
        assert!(result.allowed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_blocked_when_violation_detected() {
        let mut engine = EnforcementEngine::new(vec![test_policy()]);
        let result = engine.check("write unsafe code", &HashMap::new());
        assert!(!result.allowed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].severity, RuleSeverity::Critical);
    }

    #[test]
    fn test_context_based_violation() {
        let mut engine = EnforcementEngine::new(vec![test_policy()]);
        let mut context = HashMap::new();
        context.insert("code".into(), "contains unsafe block".into());
        let result = engine.check("compile", &context);
        assert!(!result.allowed);
    }

    #[test]
    fn test_advisory_produces_warnings_not_blocks() {
        let policy = Policy::new(
            "style",
            "Style Guide",
            "Advisory style rules",
            EnforcementLevel::Advisory,
        )
        .with_rule(PolicyRule::new(
            "no_todos",
            "TODO",
            "Remove TODO comments",
            RuleSeverity::Low,
        ));

        let mut engine = EnforcementEngine::new(vec![policy]);
        let result = engine.check("add TODO comment", &HashMap::new());
        assert!(result.allowed, "advisory should not block");
        assert!(
            !result.warnings.is_empty(),
            "advisory should produce warnings"
        );
    }

    #[test]
    fn test_warning_level_produces_warnings_not_violations() {
        let policy = Policy::new(
            "naming",
            "Naming Convention",
            "Use snake_case",
            EnforcementLevel::Warning,
        )
        .with_rule(PolicyRule::new(
            "no_camel_case",
            "camelCase",
            "Use snake_case instead",
            RuleSeverity::Medium,
        ));

        let mut engine = EnforcementEngine::new(vec![policy]);
        let result = engine.check("write camelCase variable", &HashMap::new());
        assert!(result.allowed, "warning level should not block");
        assert!(
            !result.warnings.is_empty(),
            "warning level should produce warnings"
        );
    }

    #[test]
    fn test_disabled_policy_skipped() {
        let mut policy = test_policy();
        policy.enabled = false;
        let mut engine = EnforcementEngine::new(vec![policy]);
        let result = engine.check("write unsafe code", &HashMap::new());
        assert!(
            result.allowed,
            "disabled policy should not trigger violations"
        );
    }

    #[test]
    fn test_disabled_rule_skipped() {
        let policy =
            Policy::new("p1", "P1", "desc", EnforcementLevel::Blocking).with_rule(PolicyRule {
                id: "r1".into(),
                condition: "unsafe".into(),
                action: "block".into(),
                severity: RuleSeverity::Critical,
                enabled: false,
            });

        let mut engine = EnforcementEngine::new(vec![policy]);
        let result = engine.check("write unsafe code", &HashMap::new());
        assert!(result.allowed, "disabled rule should not trigger");
    }

    #[test]
    fn test_multiple_policies() {
        let p1 = Policy::new(
            "security",
            "Security",
            "No secrets",
            EnforcementLevel::Blocking,
        )
        .with_rule(PolicyRule::new(
            "no_secrets",
            "secret",
            "Do not hardcode secrets",
            RuleSeverity::Critical,
        ));

        let p2 = Policy::new("style", "Style", "No TODOs", EnforcementLevel::Advisory).with_rule(
            PolicyRule::new("no_todos", "TODO", "Remove TODOs", RuleSeverity::Low),
        );

        let mut engine = EnforcementEngine::new(vec![p1, p2]);
        let result = engine.check("add secret and TODO", &HashMap::new());
        assert!(!result.allowed, "blocking violation should block");
        // Both policies trigger
        assert!(result.violation_count() >= 1);
    }

    #[test]
    fn test_stats_tracking() {
        let mut engine = EnforcementEngine::new(vec![test_policy()]);
        engine.check("safe action", &HashMap::new());
        engine.check("write unsafe code", &HashMap::new());
        engine.check("another safe action", &HashMap::new());

        let stats = engine.stats();
        assert_eq!(stats.total_checks, 3);
        assert_eq!(stats.total_violations, 1);
        assert_eq!(stats.policy_count, 1);
    }

    #[test]
    fn test_enforcement_result_helpers() {
        let result = EnforcementResult::allowed();
        assert!(result.allowed);
        assert!(result.violations.is_empty());

        let result = EnforcementResult::allowed().with_warning("heads up");
        assert!(result.allowed);
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_has_blocking_violations() {
        let result = EnforcementResult {
            allowed: false,
            violations: vec![Violation {
                policy_id: "p".into(),
                rule_id: "r".into(),
                message: "m".into(),
                severity: RuleSeverity::Critical,
            }],
            warnings: vec![],
        };
        assert!(result.has_blocking_violations());

        let result = EnforcementResult {
            allowed: true,
            violations: vec![],
            warnings: vec!["advisory note".into()],
        };
        assert!(!result.has_blocking_violations());
    }

    #[test]
    fn test_add_policy_after_creation() {
        let mut engine = EnforcementEngine::new(vec![]);
        assert!(engine.check("anything", &HashMap::new()).allowed);

        engine.add_policy(test_policy());
        let result = engine.check("write unsafe code", &HashMap::new());
        assert!(!result.allowed);
    }

    #[test]
    fn test_multiple_violations_same_policy() {
        let policy = Policy::new(
            "multi",
            "Multi Rule",
            "Multiple blocking rules",
            EnforcementLevel::Blocking,
        )
        .with_rule(PolicyRule::new(
            "r1",
            "unsafe",
            "block unsafe",
            RuleSeverity::Critical,
        ))
        .with_rule(PolicyRule::new(
            "r2",
            "unsafe",
            "also block unsafe",
            RuleSeverity::High,
        ));

        let mut engine = EnforcementEngine::new(vec![policy]);
        let result = engine.check("write unsafe code", &HashMap::new());
        assert!(!result.allowed);
        assert_eq!(result.violations.len(), 2);
    }

    #[test]
    fn test_enforcement_result_blocked_factory() {
        let v = Violation {
            policy_id: "p".into(),
            rule_id: "r".into(),
            message: "msg".into(),
            severity: RuleSeverity::High,
        };
        let result = EnforcementResult::blocked(vec![v], vec!["w1".into()]);
        assert!(!result.allowed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_stats_after_no_checks() {
        let engine = EnforcementEngine::new(vec![]);
        let stats = engine.stats();
        assert_eq!(stats.total_checks, 0);
        assert_eq!(stats.total_violations, 0);
        assert_eq!(stats.policy_count, 0);
    }

    #[test]
    fn test_policies_returns_all() {
        let p1 = Policy::new("p1", "P1", "desc", EnforcementLevel::Advisory);
        let p2 = Policy::new("p2", "P2", "desc", EnforcementLevel::Blocking);
        let engine = EnforcementEngine::new(vec![p1, p2]);
        assert_eq!(engine.policies().len(), 2);
    }

    #[test]
    fn test_disabled_policy_not_counted_in_stats() {
        let mut policy = test_policy();
        policy.enabled = false;
        let engine = EnforcementEngine::new(vec![policy]);
        let stats = engine.stats();
        assert_eq!(stats.policy_count, 0);
    }

    #[test]
    fn test_enforcement_result_violation_count() {
        let result = EnforcementResult {
            allowed: false,
            violations: vec![
                Violation {
                    policy_id: "p1".into(),
                    rule_id: "r1".into(),
                    message: "m1".into(),
                    severity: RuleSeverity::Low,
                },
                Violation {
                    policy_id: "p2".into(),
                    rule_id: "r2".into(),
                    message: "m2".into(),
                    severity: RuleSeverity::Medium,
                },
            ],
            warnings: vec![],
        };
        assert_eq!(result.violation_count(), 2);
    }

    #[test]
    fn test_check_case_insensitive() {
        let mut engine = EnforcementEngine::new(vec![test_policy()]);
        let result = engine.check("WRITE UNSAFE CODE", &HashMap::new());
        assert!(!result.allowed);
    }

    #[test]
    fn test_context_values_checked() {
        let mut engine = EnforcementEngine::new(vec![test_policy()]);
        let mut ctx = HashMap::new();
        ctx.insert("file_content".into(), "contains unsafe block".into());
        let result = engine.check("compile module", &ctx);
        assert!(!result.allowed);
    }

    #[test]
    fn test_no_match_on_unrelated_action() {
        let mut engine = EnforcementEngine::new(vec![test_policy()]);
        let result = engine.check("deploy to production", &HashMap::new());
        assert!(result.allowed);
    }
}
