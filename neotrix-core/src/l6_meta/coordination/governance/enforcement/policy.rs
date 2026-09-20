//! Policy definitions for governance enforcement (R-P141, R-P142).
//!
//! Policies encode rules that agents must follow. Each policy contains
//! rules with human-readable conditions and machine-evaluable actions.
//! Enforcement levels determine how violations are handled.

use serde::{Deserialize, Serialize};

/// Enforcement severity level for a policy.
///
/// Determines how violations are surfaced and whether they block execution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementLevel {
    /// Advisory: logged but never blocks. Use for informational rules.
    Advisory = 0,
    /// Warning: logged and returned in results, but does not block.
    Warning = 1,
    /// Blocking: violations prevent the action from proceeding.
    Blocking = 2,
}

/// Severity of a specific rule violation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RuleSeverity {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// A single rule within a policy.
///
/// Each rule defines a condition to check and an action to take if the
/// condition is violated. Rules are evaluated in order within a policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Unique rule identifier (e.g., "no_unsafe_code").
    pub id: String,
    /// Human-readable description of the condition.
    pub condition: String,
    /// Action description when the condition is violated.
    pub action: String,
    /// Severity of this rule.
    pub severity: RuleSeverity,
    /// Whether this rule is currently enabled.
    pub enabled: bool,
}

/// A governance policy containing rules and enforcement configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Unique policy identifier.
    pub id: String,
    /// Human-readable policy name.
    pub name: String,
    /// Detailed description of what this policy enforces.
    pub description: String,
    /// Rules contained in this policy.
    pub rules: Vec<PolicyRule>,
    /// Enforcement level for this policy.
    pub enforcement_level: EnforcementLevel,
    /// Whether this policy is currently active.
    pub enabled: bool,
}

impl Policy {
    /// Create a new policy with the given configuration.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        enforcement_level: EnforcementLevel,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            rules: Vec::new(),
            enforcement_level,
            enabled: true,
        }
    }

    /// Add a rule to this policy. Returns `self` for chaining.
    pub fn with_rule(mut self, rule: PolicyRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Get only enabled rules.
    pub fn enabled_rules(&self) -> impl Iterator<Item = &PolicyRule> {
        self.rules.iter().filter(|r| r.enabled)
    }

    /// Get the count of enabled rules.
    pub fn rule_count(&self) -> usize {
        self.rules.iter().filter(|r| r.enabled).count()
    }
}

impl PolicyRule {
    /// Create a new policy rule.
    pub fn new(
        id: impl Into<String>,
        condition: impl Into<String>,
        action: impl Into<String>,
        severity: RuleSeverity,
    ) -> Self {
        Self {
            id: id.into(),
            condition: condition.into(),
            action: action.into(),
            severity,
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_new() {
        let policy = Policy::new(
            "test_policy",
            "Test Policy",
            "A test policy",
            EnforcementLevel::Warning,
        );
        assert_eq!(policy.id, "test_policy");
        assert_eq!(policy.name, "Test Policy");
        assert_eq!(policy.enforcement_level, EnforcementLevel::Warning);
        assert!(policy.enabled);
        assert!(policy.rules.is_empty());
    }

    #[test]
    fn test_policy_with_rule_chaining() {
        let policy = Policy::new("p1", "P1", "desc", EnforcementLevel::Blocking)
            .with_rule(PolicyRule::new(
                "r1",
                "condition1",
                "action1",
                RuleSeverity::High,
            ))
            .with_rule(PolicyRule::new(
                "r2",
                "condition2",
                "action2",
                RuleSeverity::Low,
            ));

        assert_eq!(policy.rules.len(), 2);
        assert_eq!(policy.rule_count(), 2);
    }

    #[test]
    fn test_disabled_rule_excluded_from_count() {
        let rule = PolicyRule {
            id: "r1".into(),
            condition: "c".into(),
            action: "a".into(),
            severity: RuleSeverity::Medium,
            enabled: false,
        };
        let policy = Policy::new("p1", "P1", "desc", EnforcementLevel::Advisory).with_rule(rule);

        assert_eq!(policy.rule_count(), 0);
        assert!(policy.enabled_rules().next().is_none());
    }

    #[test]
    fn test_rule_severity_ordering() {
        assert!(RuleSeverity::Low < RuleSeverity::Critical);
        assert!(RuleSeverity::Medium < RuleSeverity::High);
    }

    #[test]
    fn test_enforcement_level_ordering() {
        assert!(EnforcementLevel::Advisory < EnforcementLevel::Blocking);
        assert!(EnforcementLevel::Warning < EnforcementLevel::Blocking);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let policy = Policy::new("p1", "P1", "desc", EnforcementLevel::Warning)
            .with_rule(PolicyRule::new("r1", "cond", "act", RuleSeverity::High));

        let json = serde_json::to_string(&policy).unwrap();
        let deserialized: Policy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "p1");
        assert_eq!(deserialized.rules.len(), 1);
        assert_eq!(deserialized.rules[0].severity, RuleSeverity::High);
    }

    #[test]
    fn test_policy_enabled_default() {
        let policy = Policy::new("p1", "P1", "desc", EnforcementLevel::Advisory);
        assert!(policy.enabled);
    }

    #[test]
    fn test_policy_disabled_rules_not_counted() {
        let policy = Policy::new("p1", "P1", "desc", EnforcementLevel::Advisory)
            .with_rule(PolicyRule {
                id: "r1".into(),
                condition: "c".into(),
                action: "a".into(),
                severity: RuleSeverity::High,
                enabled: false,
            })
            .with_rule(PolicyRule::new("r2", "c2", "a2", RuleSeverity::Low));

        assert_eq!(policy.rule_count(), 1);
        let enabled: Vec<_> = policy.enabled_rules().collect();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].id, "r2");
    }

    #[test]
    fn test_policy_chain_multiple_rules() {
        let policy = Policy::new("p1", "P1", "desc", EnforcementLevel::Blocking)
            .with_rule(PolicyRule::new("r1", "c1", "a1", RuleSeverity::Critical))
            .with_rule(PolicyRule::new("r2", "c2", "a2", RuleSeverity::High))
            .with_rule(PolicyRule::new("r3", "c3", "a3", RuleSeverity::Medium))
            .with_rule(PolicyRule::new("r4", "c4", "a4", RuleSeverity::Low));

        assert_eq!(policy.rules.len(), 4);
        assert_eq!(policy.rule_count(), 4);
    }

    #[test]
    fn test_enforcement_level_advisory_is_lowest() {
        assert_eq!(EnforcementLevel::Advisory as i32, 0);
        assert_eq!(EnforcementLevel::Warning as i32, 1);
        assert_eq!(EnforcementLevel::Blocking as i32, 2);
    }

    #[test]
    fn test_rule_severity_ordering_full() {
        assert!(RuleSeverity::Low < RuleSeverity::Medium);
        assert!(RuleSeverity::Medium < RuleSeverity::High);
        assert!(RuleSeverity::High < RuleSeverity::Critical);
    }

    #[test]
    fn test_policy_serialization_json_format() {
        let policy = Policy::new("test", "Test", "desc", EnforcementLevel::Blocking);
        let json = serde_json::to_string(&policy).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("blocking"));
    }

    #[test]
    fn test_rule_new_default_enabled() {
        let rule = PolicyRule::new("r1", "cond", "act", RuleSeverity::Medium);
        assert!(rule.enabled);
    }

    #[test]
    fn test_policy_empty_rules_zero_count() {
        let policy = Policy::new("p1", "P1", "desc", EnforcementLevel::Warning);
        assert_eq!(policy.rule_count(), 0);
        assert!(policy.enabled_rules().next().is_none());
    }
}
