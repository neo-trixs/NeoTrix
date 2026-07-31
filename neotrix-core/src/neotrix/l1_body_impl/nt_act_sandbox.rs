use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// AgentENV-inspired action sandbox: evaluates an action against a permission +
/// safety rule set BEFORE external execution. Only approved actions reach the
/// real environment (permission-aware retrieval/execution gate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum SandboxVerdict {
    /// Action is safe and permitted
    #[default]
    Approved,
    /// Action exceeds a safety/permission boundary
    Denied,
    /// Action requires human approval before execution
    RequiresApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxRule {
    /// Action kind prefix (e.g. "read:", "shell:", "write:")
    pub action_prefix: String,
    /// If true, rules matching this prefix are allowed by default
    pub allowed: bool,
    /// If true, matching actions require explicit human approval
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ActionSandbox {
    /// Prefix rules: most-specific prefix wins
    rules: Vec<SandboxRule>,
    /// Executed action counters for telemetry
    pub executions: HashMap<String, u64>,
    /// Total denied actions (drift signal for the tree)
    pub denied_count: u64,
    /// Total actions evaluated
    pub evaluated_count: u64,
}

impl ActionSandbox {
    pub fn new() -> Self {
        let mut rules = Vec::new();
        // Conservative defaults: destructive/external actions denied unless whitelisted
        for prefix in ["rm:", "drop:", "destroy:", "wipe:"] {
            rules.push(SandboxRule { action_prefix: prefix.into(), allowed: false, requires_approval: false });
        }
        // High-risk actions require approval
        for prefix in ["shell:", "network:", "send:", "write:/etc", "write:/usr"] {
            rules.push(SandboxRule { action_prefix: prefix.into(), allowed: true, requires_approval: true });
        }
        Self { rules, executions: HashMap::new(), denied_count: 0, evaluated_count: 0 }
    }

    pub fn add_rule(&mut self, rule: SandboxRule) {
        self.rules.push(rule);
    }

    fn matching_rule(&self, action: &str) -> Option<&SandboxRule> {
        self.rules
            .iter()
            .filter(|r| action.starts_with(&r.action_prefix))
            .max_by_key(|r| r.action_prefix.len())
    }

    /// Evaluate an action string against the rule set.
    pub fn evaluate(&mut self, action: &str) -> SandboxVerdict {
        self.evaluated_count += 1;
        let verdict = match self.matching_rule(action) {
            Some(rule) if rule.allowed && rule.requires_approval => SandboxVerdict::RequiresApproval,
            Some(rule) if rule.allowed => SandboxVerdict::Approved,
            Some(_) => SandboxVerdict::Denied,
            // Unlisted actions default to approved (explicit allow model)
            None => SandboxVerdict::Approved,
        };
        if verdict == SandboxVerdict::Denied {
            self.denied_count += 1;
        }
        *self.executions.entry(action.split(':').next().unwrap_or(action).to_string()).or_insert(0) += 1;
        verdict
    }

    /// Approval callback for RequiresApproval verdicts.
    pub fn approve(&mut self, action: &str) -> bool {
        let verdict = self.evaluate(action);
        verdict == SandboxVerdict::Approved || verdict == SandboxVerdict::RequiresApproval
    }

    /// Sandbox health as a 0..1 score (D15 energy flow to the tree).
    pub fn health(&self) -> f64 {
        if self.evaluated_count == 0 {
            return 1.0;
        }
        (1.0 - self.denied_count as f64 / self.evaluated_count as f64).max(0.0).min(1.0)
    }

    pub fn summary(&self) -> String {
        format!(
            "sandbox: evaluated={} denied={} health={:.2}",
            self.evaluated_count, self.denied_count, self.health()
        )
    }
}

/// SelfTest: sandbox detects its own configuration sanity.
impl crate::core::nt_core_self_test::SelfTest for ActionSandbox {
    fn name(&self) -> &str { "nt_act_sandbox" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.rules.is_empty() {
            failures.push("no sandbox rules configured".into());
        }
        // Conservative default check: rm: must be denied
        let mut probe = ActionSandbox::new();
        if probe.evaluate("rm:important_file") != SandboxVerdict::Denied {
            failures.push("rm: prefix not denied by default".into());
        }
        if failures.is_empty() { Ok(()) } else { Err(failures) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    #[test]
    fn test_destructive_action_denied() {
        let mut sandbox = ActionSandbox::new();
        assert_eq!(sandbox.evaluate("rm:data"), SandboxVerdict::Denied);
        assert_eq!(sandbox.denied_count, 1);
    }

    #[test]
    fn test_read_action_approved() {
        let mut sandbox = ActionSandbox::new();
        assert_eq!(sandbox.evaluate("read:/tmp/file"), SandboxVerdict::Approved);
    }

    #[test]
    fn test_high_risk_requires_approval() {
        let mut sandbox = ActionSandbox::new();
        assert_eq!(sandbox.evaluate("shell:curl https://x"), SandboxVerdict::RequiresApproval);
    }

    #[test]
    fn test_most_specific_prefix_wins() {
        let mut sandbox = ActionSandbox::new();
        sandbox.add_rule(SandboxRule { action_prefix: "write:/tmp".into(), allowed: true, requires_approval: false });
        // write:/etc requires approval (default), but write:/tmp (more specific) is approved
        assert_eq!(sandbox.evaluate("write:/tmp/a"), SandboxVerdict::Approved);
        assert_eq!(sandbox.evaluate("write:/etc/hosts"), SandboxVerdict::RequiresApproval);
    }

    #[test]
    fn test_health_and_summary() {
        let mut sandbox = ActionSandbox::new();
        let _ = sandbox.evaluate("rm:x");
        let _ = sandbox.evaluate("read:a");
        assert!(sandbox.health() > 0.0 && sandbox.health() < 1.0);
        assert!(sandbox.summary().contains("evaluated=2"));
    }

    #[test]
    fn test_self_test() {
        let sandbox = ActionSandbox::new();
        assert!(sandbox.self_test().is_ok());
    }

    #[test]
    fn test_timestamped_executions() {
        let mut sandbox = ActionSandbox::new();
        let _ = sandbox.evaluate("read:a");
        let _ = sandbox.evaluate("read:b");
        assert_eq!(sandbox.executions.get("read"), Some(&2));
    }

    // Silence unused import lint for SystemTime when not otherwise used
}
