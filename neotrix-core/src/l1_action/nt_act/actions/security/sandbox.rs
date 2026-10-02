use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::l1_action::nt_act::nt_act_disk_guard::{DiskGuard, DiskVerdict};

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

    /// 找出最匹配的规则：**最长前缀优先**。
    ///
    /// ⚠️ 2026-09-30 修正：原先用 `max_by_key(|r| r.action_prefix.len())`。
    /// `max_by_key` **签名里没有 tie-break 的位置** ⇒ 前缀长度并列时
    /// 返回**任意一条**。而 `add_rule` **不做去重** ⇒ 完全可以注册
    /// 两条**同前缀**规则（一条 `allowed=true`、一条 `allowed=false`）。
    ///
    /// ⛔ 这是**安全路径**：命中哪条决定沙箱裁决
    /// （`Approved` / `RequiresApproval` / `Denied`）
    /// ⇒ 并列时**裁决结果随机**，同一份规则集会时而放行时而拒绝。
    /// 且本模块**无任何测试覆盖重复前缀**。
    ///
    /// ⇒ 修法：`max_by` + 确定性 tie-break ——
    ///   ① 前缀长度降序（保留原有「最长前缀优先」语义）；
    ///   ② 长度并列时按前缀字典序取小者，使结果可复现。
    ///   （语义上「后注册的规则覆盖先注册的」也说得通，但仓库里
    ///   没有这种约定，且字典序不依赖注册顺序、更可测。）
    fn matching_rule(&self, action: &str) -> Option<&SandboxRule> {
        self.rules
            .iter()
            .filter(|r| action.starts_with(&r.action_prefix))
            // ⚠️ tie-break 键必须是**能区分两条规则**的量。
            // 第一版我用了 `action_prefix` 字典序 —— 但重复前缀场景下
            // 两条规则的 prefix **完全相同** ⇒ tie-break 恒等 ⇒
            // 裁决**仍然随机**（实测一 Denied 一 Approved）。
            //
            // ⇒ 改用**注册顺序**：靠 `Vec` 的下标天然区分，
            // 语义为「**后注册的规则覆盖先注册的**」（同长度时后者胜），
            // 这也与「后加的策略覆盖默认策略」的一般直觉一致，
            // 且**可复现**（顺序是确定的）。
            .enumerate()
            .max_by(|a, b| {
                // `enumerate()` 的首元是 `usize`（已复制），**不是** `&usize`
                // ⇒ 不能 `*a.0`（第一版这么写触发 E0614）。
                let (ia, ra) = (a.0, a.1);
                let (ib, rb) = (b.0, b.1);
                ra.action_prefix
                    .len()
                    .cmp(&rb.action_prefix.len())
                    .then_with(|| ia.cmp(&ib))
            })
            .map(|(_, r)| r)
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
impl crate::l0_substrate::nt_core_self_test::SelfTest for ActionSandbox {
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
    use crate::l0_substrate::nt_core_self_test::SelfTest;

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

    /// 回归（2026-09-30）：**重复前缀**下裁决必须可复现。
    ///
    /// `add_rule` 不做去重 ⇒ 可注册两条同前缀规则而 `allowed` 相反。
    /// 原实现 `max_by_key(len)` 在长度并列时返回任意一条
    /// ⇒ 沙箱裁决（`Approved` / `Denied`）在**同一份规则集**上随机。
    /// 这是安全路径，故必须固定。
    #[test]
    fn test_duplicate_prefix_verdict_is_reproducible() {
        // 构造两份规则集：同样两条同前缀规则、顺序相反。
        let mk = |rules: Vec<SandboxRule>| {
            let mut sb = ActionSandbox::new();
            for r in rules {
                sb.add_rule(r);
            }
            sb
        };
        let mut allow_first = mk(vec![
            SandboxRule { action_prefix: "git".into(), allowed: true, requires_approval: false },
            SandboxRule { action_prefix: "git".into(), allowed: false, requires_approval: false },
        ]);
        let mut deny_first = mk(vec![
            SandboxRule { action_prefix: "git".into(), allowed: false, requires_approval: false },
            SandboxRule { action_prefix: "git".into(), allowed: true, requires_approval: false },
        ]);
        // ⚠️ tie-break 采用「后注册者优先」⇒ 顺序不同结果**本就应当不同**
        //   （allow_first 的最后一条是 denied ⇒ Denied；
        //     deny_first 的最后一条是 allowed ⇒ Approved）。
        // ⇒ 真正要断言的是：**同一份规则集重复求值结果一致**（可复现）。
        assert_eq!(allow_first.evaluate("git status"), SandboxVerdict::Denied);
        assert_eq!(allow_first.evaluate("git status"), SandboxVerdict::Denied);
        assert_eq!(deny_first.evaluate("git status"), SandboxVerdict::Approved);
        assert_eq!(deny_first.evaluate("git status"), SandboxVerdict::Approved);
    }

    /// 回归（2026-09-30）：「最长前缀优先」语义不得被 tie-break 破坏。
    #[test]
    fn test_longest_prefix_still_wins_over_tiebreak() {
        let mut sb = ActionSandbox::new();
        // "git" 放行；"git push" 拒绝 ⇒ 更长前缀必须胜出
        sb.add_rule(SandboxRule { action_prefix: "git".into(), allowed: true, requires_approval: false });
        sb.add_rule(SandboxRule { action_prefix: "git push".into(), allowed: false, requires_approval: false });
        assert_eq!(sb.evaluate("git push origin"), SandboxVerdict::Denied);
        assert_eq!(sb.evaluate("git status"), SandboxVerdict::Approved);
    }
}
