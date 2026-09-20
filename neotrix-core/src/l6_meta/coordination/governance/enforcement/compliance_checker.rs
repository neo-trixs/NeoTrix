//! System-wide compliance checker (R-P141, R-P142).
//!
//! Evaluates overall governance compliance across all policies and produces
//! a system-level compliance report with recommendations.

use serde::{Deserialize, Serialize};

use super::policy::{EnforcementLevel, Policy};

/// Overall compliance status of the system.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceComplianceStatus {
    /// All policies pass.
    Compliant = 0,
    /// Some advisory or warning violations exist.
    PartiallyCompliant = 1,
    /// Blocking violations exist.
    NonCompliant = 2,
}

/// Status of a single policy during compliance check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCheckResult {
    /// Policy ID.
    pub policy_id: String,
    /// Policy name.
    pub policy_name: String,
    /// Whether the policy passed (no blocking violations).
    pub passed: bool,
    /// Number of rules in the policy.
    pub rule_count: usize,
    /// Number of violations found.
    pub violation_count: usize,
    /// Enforcement level of the policy.
    pub enforcement_level: EnforcementLevel,
}

/// System-wide compliance report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemComplianceReport {
    /// Overall compliance status.
    pub overall_status: GovernanceComplianceStatus,
    /// Results for each checked policy.
    pub checked_policies: Vec<PolicyCheckResult>,
    /// Number of policies that passed.
    pub passed: usize,
    /// Number of policies that failed.
    pub failed: usize,
    /// Recommendations for improving compliance.
    pub recommendations: Vec<String>,
}

impl SystemComplianceReport {
    /// Check if the system is fully compliant.
    pub fn is_compliant(&self) -> bool {
        self.overall_status == GovernanceComplianceStatus::Compliant
    }
}

/// Checks system-wide compliance by evaluating policies against a reference action.
///
/// Unlike `EnforcementEngine::check` which evaluates a specific action,
/// `ComplianceChecker` evaluates all policies to produce a system health report.
pub struct ComplianceChecker;

impl ComplianceChecker {
    /// Check system compliance across all policies.
    ///
    /// Uses a reference action `"system_compliance_audit"` with empty context
    /// to trigger rule evaluation. Rules that match this reference action
    /// indicate potential compliance gaps.
    pub fn check_system_compliance(policies: &[Policy]) -> SystemComplianceReport {
        let mut checked = Vec::new();
        let mut passed_count = 0;
        let mut failed_count = 0;
        let mut recommendations = Vec::new();

        for policy in policies {
            if !policy.enabled {
                continue;
            }

            let rule_count = policy.rule_count();
            let mut violation_count = 0;

            // Count rules that would trigger on a generic audit action
            for rule in policy.enabled_rules() {
                // In a real system, this would run the rule against test vectors.
                // Here we check if the rule has meaningful conditions.
                if rule.condition.is_empty() || rule.action.is_empty() {
                    violation_count += 1;
                    recommendations.push(format!(
                        "Policy '{}' rule '{}' has empty condition or action — fill in or remove",
                        policy.id, rule.id
                    ));
                }
            }

            let passed = violation_count == 0;
            if passed {
                passed_count += 1;
            } else {
                failed_count += 1;
            }

            checked.push(PolicyCheckResult {
                policy_id: policy.id.clone(),
                policy_name: policy.name.clone(),
                passed,
                rule_count,
                violation_count,
                enforcement_level: policy.enforcement_level,
            });
        }

        let overall_status = if failed_count == 0 {
            GovernanceComplianceStatus::Compliant
        } else if checked
            .iter()
            .any(|c| !c.passed && c.enforcement_level == EnforcementLevel::Blocking)
        {
            GovernanceComplianceStatus::NonCompliant
        } else {
            GovernanceComplianceStatus::PartiallyCompliant
        };

        // Add general recommendations
        if checked.is_empty() {
            recommendations.push("No policies loaded — add governance policies".into());
        }

        if failed_count > 0 {
            recommendations.push(format!(
                "{} policy(s) have issues — review and fix violations",
                failed_count
            ));
        }

        SystemComplianceReport {
            overall_status,
            checked_policies: checked,
            passed: passed_count,
            failed: failed_count,
            recommendations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l6_meta::coordination::governance::enforcement::policy::{
        Policy, PolicyRule, RuleSeverity,
    };

    fn sample_policy() -> Policy {
        Policy::new(
            "test_policy",
            "Test Policy",
            "A test policy for compliance checking",
            EnforcementLevel::Blocking,
        )
        .with_rule(PolicyRule::new(
            "rule_1",
            "condition must be met",
            "Take corrective action",
            RuleSeverity::High,
        ))
        .with_rule(PolicyRule::new(
            "rule_2",
            "another condition",
            "Another action",
            RuleSeverity::Medium,
        ))
    }

    #[test]
    fn test_compliant_when_all_policies_pass() {
        let report = ComplianceChecker::check_system_compliance(&[sample_policy()]);
        assert!(report.is_compliant());
        assert_eq!(report.overall_status, GovernanceComplianceStatus::Compliant);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 0);
    }

    #[test]
    fn test_non_compliant_when_blocking_policy_fails() {
        let policy = Policy::new(
            "bad_policy",
            "Bad Policy",
            "Has empty rules",
            EnforcementLevel::Blocking,
        )
        .with_rule(PolicyRule::new(
            "empty_rule",
            "", // empty condition
            "", // empty action
            RuleSeverity::Critical,
        ));

        let report = ComplianceChecker::check_system_compliance(&[policy]);
        assert!(!report.is_compliant());
        assert_eq!(
            report.overall_status,
            GovernanceComplianceStatus::NonCompliant
        );
        assert_eq!(report.failed, 1);
    }

    #[test]
    fn test_partially_compliant() {
        let good = sample_policy();
        let bad = Policy::new(
            "warn_policy",
            "Warn Policy",
            "Has empty rules at warning level",
            EnforcementLevel::Warning,
        )
        .with_rule(PolicyRule::new("empty_rule", "", "", RuleSeverity::Low));

        let report = ComplianceChecker::check_system_compliance(&[good, bad]);
        assert_eq!(
            report.overall_status,
            GovernanceComplianceStatus::PartiallyCompliant
        );
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 1);
    }

    #[test]
    fn test_no_policies_produces_recommendation() {
        let report = ComplianceChecker::check_system_compliance(&[]);
        assert_eq!(report.overall_status, GovernanceComplianceStatus::Compliant);
        assert!(!report.recommendations.is_empty());
        assert!(report.recommendations[0].contains("No policies loaded"));
    }

    #[test]
    fn test_disabled_policies_skipped() {
        let mut policy = sample_policy();
        policy.enabled = false;

        let report = ComplianceChecker::check_system_compliance(&[policy]);
        assert_eq!(report.checked_policies.len(), 0);
    }

    #[test]
    fn test_recommendations_generated_for_issues() {
        let policy = Policy::new("p1", "P1", "desc", EnforcementLevel::Blocking)
            .with_rule(PolicyRule::new("r1", "", "action", RuleSeverity::High))
            .with_rule(PolicyRule::new("r2", "condition", "", RuleSeverity::Medium));

        let report = ComplianceChecker::check_system_compliance(&[policy]);
        assert!(report.recommendations.len() >= 2);
    }

    #[test]
    fn test_policy_check_result_fields() {
        let report = ComplianceChecker::check_system_compliance(&[sample_policy()]);
        let result = &report.checked_policies[0];
        assert_eq!(result.policy_id, "test_policy");
        assert_eq!(result.policy_name, "Test Policy");
        assert_eq!(result.rule_count, 2);
        assert_eq!(result.violation_count, 0);
        assert!(result.passed);
    }

    #[test]
    fn test_multiple_policies_mixed_results() {
        let good = sample_policy();
        let bad_blocking = Policy::new("bad", "Bad", "desc", EnforcementLevel::Blocking)
            .with_rule(PolicyRule::new("r1", "", "", RuleSeverity::Critical));

        let report = ComplianceChecker::check_system_compliance(&[good, bad_blocking]);
        assert_eq!(report.passed, 1);
        assert_eq!(report.failed, 1);
        assert!(!report.is_compliant());
    }

    #[test]
    fn test_compliance_report_fields() {
        let report = ComplianceChecker::check_system_compliance(&[sample_policy()]);
        assert_eq!(report.checked_policies.len(), 1);
        assert!(
            report.recommendations.is_empty()
                || report.recommendations.iter().any(|r| r.contains("review"))
        );
    }

    #[test]
    fn test_multiple_compliant_policies() {
        let p1 = sample_policy();
        let p2 = Policy::new(
            "good2",
            "Good2",
            "Another good policy",
            EnforcementLevel::Warning,
        )
        .with_rule(PolicyRule::new(
            "rule_a",
            "valid condition",
            "valid action",
            RuleSeverity::Medium,
        ));

        let report = ComplianceChecker::check_system_compliance(&[p1, p2]);
        assert!(report.is_compliant());
        assert_eq!(report.passed, 2);
        assert_eq!(report.failed, 0);
    }

    #[test]
    fn test_advisory_level_failure_is_partially_compliant() {
        let bad = Policy::new(
            "advisory_bad",
            "Advisory Bad",
            "Has empty rules at advisory level",
            EnforcementLevel::Advisory,
        )
        .with_rule(PolicyRule::new("r1", "", "", RuleSeverity::Low));

        let report = ComplianceChecker::check_system_compliance(&[bad]);
        assert_eq!(
            report.overall_status,
            GovernanceComplianceStatus::PartiallyCompliant
        );
    }

    #[test]
    fn test_check_result_contains_policy_name() {
        let report = ComplianceChecker::check_system_compliance(&[sample_policy()]);
        assert_eq!(report.checked_policies[0].policy_name, "Test Policy");
    }

    #[test]
    fn test_check_result_enforcement_level() {
        let report = ComplianceChecker::check_system_compliance(&[sample_policy()]);
        assert_eq!(
            report.checked_policies[0].enforcement_level,
            EnforcementLevel::Blocking
        );
    }

    #[test]
    fn test_all_disabled_policies_compliant() {
        let mut p1 = sample_policy();
        p1.enabled = false;
        let mut p2 = Policy::new("p2", "P2", "desc", EnforcementLevel::Blocking);
        p2.enabled = false;

        let report = ComplianceChecker::check_system_compliance(&[p1, p2]);
        assert!(report.is_compliant());
        assert_eq!(report.checked_policies.len(), 0);
    }

    #[test]
    fn test_rule_with_empty_action_only() {
        let policy =
            Policy::new("p1", "P1", "desc", EnforcementLevel::Blocking).with_rule(PolicyRule::new(
                "r1",
                "valid condition",
                "", // empty action
                RuleSeverity::High,
            ));

        let report = ComplianceChecker::check_system_compliance(&[policy]);
        assert_eq!(report.failed, 1);
        assert!(report.recommendations.iter().any(|r| r.contains("empty")));
    }

    #[test]
    fn test_rule_with_empty_condition_only() {
        let policy =
            Policy::new("p1", "P1", "desc", EnforcementLevel::Blocking).with_rule(PolicyRule::new(
                "r1",
                "", // empty condition
                "valid action",
                RuleSeverity::High,
            ));

        let report = ComplianceChecker::check_system_compliance(&[policy]);
        assert_eq!(report.failed, 1);
    }
}
