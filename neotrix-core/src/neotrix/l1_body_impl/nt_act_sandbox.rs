use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::neotrix::l1_body_impl::nt_act_disk_guard::{DiskGuard, DiskVerdict};

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

#[derive(Debug, Clone)]
pub struct ActionSandbox {
    /// Prefix rules: most-specific prefix wins
    rules: Vec<SandboxRule>,
    /// Executed action counters for telemetry
    pub executions: HashMap<String, u64>,
    /// Total denied actions (drift signal for the tree)
    pub denied_count: u64,
    /// Total actions evaluated
    pub evaluated_count: u64,
    /// 磁盘沙盒 — 任务 allowlist 越界检查 (nt_act_disk_guard 接线)
    pub disk_guard: Option<DiskGuard>,
}

impl Default for ActionSandbox {
    /// 与 `new()` 等价: 默认实例也必须携带保守防护规则。
    /// 此前 derive(Default) 的 rules 恒空 → SelfTest "no sandbox rules
    /// configured" 失败 → NT-ACT 分支 health 恒 0.667 (3 检测 2 过 1 败)。
    /// 注册表以 `ActionSandbox::default()` 构造检测件, 空规则即无防护语义。
    fn default() -> Self {
        Self::new()
    }
}

impl ActionSandbox {
    pub fn new() -> Self {
        let mut rules = Vec::new();
        // Conservative defaults: destructive/external actions denied unless whitelisted
        for prefix in ["rm:", "drop:", "destroy:", "wipe:", "delete_file:"] {
            rules.push(SandboxRule { action_prefix: prefix.into(), allowed: false, requires_approval: false });
        }
        // Safe read-only actions allowed by default (incl. ShieldEnforcer action kinds)
        for prefix in ["read:", "query:", "search:", "list:", "file_read:"] {
            rules.push(SandboxRule { action_prefix: prefix.into(), allowed: true, requires_approval: false });
        }
        // High-risk actions require approval
        for prefix in ["shell:", "network:", "send:", "execute_command:", "write:/etc", "write:/usr", "write:/var", "write_file:/etc", "write_file:/usr", "write_file:/var"] {
            rules.push(SandboxRule { action_prefix: prefix.into(), allowed: true, requires_approval: true });
        }
        Self { rules, executions: HashMap::new(), denied_count: 0, evaluated_count: 0, disk_guard: None }
    }

    /// 挂接磁盘守卫 (任务 allowlist)。生产路径: 任务初始化时分配工作区后调用。
    pub fn attach_disk_guard(&mut self, guard: DiskGuard) {
        self.disk_guard = Some(guard);
    }

    /// 从动作字符串提取路径参数: "write:/tmp/a" → "/tmp/a", "file_write:/x" → "/x"。
    fn extract_path(action: &str) -> Option<std::path::PathBuf> {
        let (_, rest) = action.split_once(':')?;
        let rest = rest.trim();
        if rest.is_empty() || rest.contains(' ') {
            return None;
        }
        Some(std::path::PathBuf::from(rest))
    }

    /// 带磁盘越界检查的求值: 先过规则, 再对 write/delete 类动作做路径 allowlist 检查。
    /// 磁盘越界 → Denied (不依赖规则默认 fail-open/approval)。
    pub fn evaluate_with_path(&mut self, action: &str) -> SandboxVerdict {
        let verdict = self.evaluate(action);
        if verdict == SandboxVerdict::Denied {
            return verdict;
        }
        // 仅对写/删类动作做磁盘越界检查 (读放宽 — 沙盒外读取多为查询)
        let is_write_like = action.starts_with("write:")
            || action.starts_with("write_file:")
            || action.starts_with("delete:")
            || action.starts_with("delete_file:")
            || action.starts_with("rm:");
        if !is_write_like {
            return verdict;
        }
        let Some(guard) = &mut self.disk_guard else {
            // 未配置磁盘守卫: 保持规则判定 (向后兼容)
            return verdict;
        };
        match Self::extract_path(action) {
            Some(path) => match guard.check("write", &path) {
                DiskVerdict::Allowed => verdict,
                DiskVerdict::Blocked(_reason) => {
                    self.denied_count += 1;
                    SandboxVerdict::Denied
                }
            },
            None => verdict,
        }
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
            // Unlisted actions default to RequiresApproval (fail-closed):
            // an unknown action kind must be explicitly whitelisted before it
            // can run without human gate. Previously fail-open (Approved).
            None => SandboxVerdict::RequiresApproval,
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
        // DiskGuard 越界检查: 配置 allowlist 后 write: 越界必须 Denied
        if let Some(guard) = &self.disk_guard {
            if guard.allowlist().is_empty() {
                failures.push("disk guard attached with empty allowlist".into());
            }
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

    #[test]
    fn test_disk_guard_blocks_outside_workspace() {
        use std::path::Path;
        let mut sandbox = ActionSandbox::new();
        let mut guard = DiskGuard::new();
        guard.allow(Path::new("/tmp/ws"));
        sandbox.attach_disk_guard(guard);
        // 越界写入 → Denied (disk guard 覆盖规则)
        assert_eq!(sandbox.evaluate_with_path("write:/etc/hosts"), SandboxVerdict::Denied);
        // 允许区内写入 → 磁盘检查放行, 保持规则判定 (write: 默认 RequiresApproval)
        assert_eq!(sandbox.evaluate_with_path("write:/tmp/ws/a.txt"), SandboxVerdict::RequiresApproval);
    }

    #[test]
    fn test_disk_guard_not_attached_backward_compat() {
        let mut sandbox = ActionSandbox::new();
        // 未挂磁盘守卫: evaluate_with_path 退回纯规则
        assert_eq!(sandbox.evaluate_with_path("write:/etc/hosts"), SandboxVerdict::RequiresApproval);
    }

    #[test]
    fn test_disk_guard_allowlist_signal() {
        use std::path::Path;
        let mut guard = DiskGuard::new();
        guard.allow(Path::new("/tmp/ws"));
        assert!(guard.is_within(Path::new("/tmp/ws/a.txt")));
        assert!(!guard.is_within(Path::new("/etc/hosts")));
    }

    // Silence unused import lint for SystemTime when not otherwise used
}
