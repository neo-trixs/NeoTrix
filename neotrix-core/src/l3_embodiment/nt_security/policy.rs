//! Security policy definition and enforcement.

/// A security policy rule
#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub name: String,
    pub description: String,
    pub severity_threshold: super::scanner::Severity,
    pub action: PolicyAction,
}

/// Action to take when a policy is violated
#[derive(Debug, Clone)]
pub enum PolicyAction {
    Block,
    Warn,
    Escalate,
    Log,
}

/// A security policy containing rules
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub name: String,
    pub rules: Vec<PolicyRule>,
}

impl SecurityPolicy {
    pub fn new(name: &str) -> Self {
        Self { name: name.into(), rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
    }

    pub fn evaluate(&self, finding: &super::scanner::SecurityFinding) -> PolicyAction {
        for rule in &self.rules {
            if self.severity_meets_threshold(&finding.severity, &rule.severity_threshold) {
                return rule.action.clone();
            }
        }
        PolicyAction::Log
    }

    fn severity_meets_threshold(&self, actual: &super::scanner::Severity, threshold: &super::scanner::Severity) -> bool {
        let order = |s: &super::scanner::Severity| match s {
            super::scanner::Severity::Critical => 4,
            super::scanner::Severity::High => 3,
            super::scanner::Severity::Medium => 2,
            super::scanner::Severity::Low => 1,
            super::scanner::Severity::Informational => 0,
        };
        order(actual) >= order(threshold)
    }
}
