//! 单调授权守卫链 — 吸收自 cordiverse/paper §5.1.2 (compositional hardening:
//! 安全层组合而非替换) 与 deepseek-harness 逆向报告 (guard 判定单调收敛,
//! 只许降级不许回放), 语义对齐 dsh-grant 单调授权 (一次性 → 会话 → 永久)。
//!
//! 与现有 `SecurityGuard` (交互式 ask) / `PermissionChain` (模式切换) /
//! `tool_permissions` (静态权限集) 的差异: 本模块是**纯判定聚合器** —
//! 一组守卫单调收敛到单个裁决 (Deny > Ask > Allow), 无 I/O, 无会话状态,
//! 供 McpServer 工具调用路径作为第一道闸 (R-P79 生产接线)。

use serde_json::Value;

/// 守卫裁决 — 单调递进: Allow < Ask < Deny (只允许降级, 不允许回放)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GuardVerdict {
    Allow,
    Ask,
    Deny,
}

impl GuardVerdict {
    pub fn name(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Ask => "ask",
            Self::Deny => "deny",
        }
    }

    /// 是否放行 (仅 Allow 放行; Ask/Deny 均拦截)。
    pub fn is_allowed(self) -> bool {
        matches!(self, Self::Allow)
    }
}

/// 单调授权链: 起始 Allow, 每个守卫只能把裁决推向更严 (Ask/Deny)。
/// 聚合规则: `max` 语义 — Deny 压过 Ask 压过 Allow; 命中 Deny 即短路。
pub struct GuardChain {
    guards: Vec<(String, Box<dyn Fn(&str, &Value) -> GuardVerdict + Send + Sync>)>,
}

impl std::fmt::Debug for GuardChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GuardChain")
            .field("guard_count", &self.guards.len())
            .field("guards", &self.guards.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>())
            .finish()
    }
}

impl Clone for GuardChain {
    fn clone(&self) -> Self {
        // 守卫闭包不可克隆 — 克隆为空链 (守卫是判定策略, 不随快照复制;
        // 语义同 `serde(skip)` 的 `vector_index`, 需由调用方重建)。
        Self::new()
    }
}

impl Default for GuardChain {
    fn default() -> Self {
        Self::new()
    }
}

impl GuardChain {
    pub fn new() -> Self {
        Self {
            guards: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.guards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.guards.is_empty()
    }

    /// 追加守卫 (同名守卫可重复, 形成链)。
    pub fn add<F>(&mut self, name: impl Into<String>, check: F)
    where
        F: Fn(&str, &Value) -> GuardVerdict + Send + Sync + 'static,
    {
        self.guards.push((name.into(), Box::new(check)));
    }

    /// 便捷: 追加无条件放行守卫。
    pub fn allow(&mut self, name: impl Into<String>) {
        self.add(name, |_, _| GuardVerdict::Allow);
    }

    /// 便捷: 追加无条件拦截守卫。
    pub fn deny(&mut self, name: impl Into<String>) {
        self.add(name, |_, _| GuardVerdict::Deny);
    }

    /// 聚合评估: 返回 (最终裁决, 命中理由列表)。
    /// 单调性: 裁决只升不降; 命中 Deny 立即短路。
    pub fn evaluate(&self, tool: &str, args: &Value) -> (GuardVerdict, Vec<String>) {
        let mut verdict = GuardVerdict::Allow;
        let mut reasons = Vec::new();
        for (name, guard) in &self.guards {
            match guard(tool, args) {
                GuardVerdict::Deny => {
                    reasons.push(format!("[{}] deny: {}", name, tool));
                    return (GuardVerdict::Deny, reasons);
                }
                GuardVerdict::Ask => {
                    if verdict < GuardVerdict::Ask {
                        verdict = GuardVerdict::Ask;
                        reasons.push(format!("[{}] ask: {}", name, tool));
                    }
                }
                GuardVerdict::Allow => {}
            }
        }
        (verdict, reasons)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(cmd: &str) -> Value {
        serde_json::json!({ "command": cmd })
    }

    #[test]
    fn test_empty_chain_is_permissive() {
        let chain = GuardChain::new();
        let (verdict, reasons) = chain.evaluate("execute_command", &args("ls"));
        assert_eq!(verdict, GuardVerdict::Allow);
        assert!(reasons.is_empty());
        assert!(chain.is_empty());
        assert!(verdict.is_allowed());
    }

    #[test]
    fn test_deny_dominates_and_short_circuits() {
        let mut chain = GuardChain::new();
        chain.allow("first");
        chain.add("destructive", |tool, a| {
            if tool == "execute_command" {
                let cmd = a.get("command").and_then(|v| v.as_str()).unwrap_or("");
                if cmd.contains("rm -rf /") {
                    return GuardVerdict::Deny;
                }
            }
            GuardVerdict::Allow
        });
        // 第三个守卫标记为不可达 (应被短路) — 用 panic 验证短路
        chain.add("should_be_skipped", |_, _| panic!("must not run"));
        let (verdict, reasons) = chain.evaluate("execute_command", &args("rm -rf /"));
        assert_eq!(verdict, GuardVerdict::Deny);
        assert!(!verdict.is_allowed());
        assert_eq!(reasons.len(), 1);
        assert!(reasons[0].contains("destructive"));
    }

    #[test]
    fn test_ask_is_monotonic() {
        let mut chain = GuardChain::new();
        chain.allow("permissive");
        chain.add("risky", |tool, a| {
            if tool == "execute_command" {
                let cmd = a.get("command").and_then(|v| v.as_str()).unwrap_or("");
                if cmd.contains("curl") {
                    return GuardVerdict::Ask;
                }
            }
            GuardVerdict::Allow
        });
        let (verdict, reasons) = chain.evaluate("execute_command", &args("curl example.com"));
        assert_eq!(verdict, GuardVerdict::Ask);
        assert!(!verdict.is_allowed());
        assert_eq!(reasons.len(), 1);
        assert!(reasons[0].contains("risky"));
    }

    #[test]
    fn test_later_allow_cannot_upgrade_ask() {
        let mut chain = GuardChain::new();
        chain.add("risky", |tool, _| {
            if tool == "write_file" {
                GuardVerdict::Ask
            } else {
                GuardVerdict::Allow
            }
        });
        chain.allow("explicitly_permit");
        let (verdict, _) = chain.evaluate("write_file", &Value::Null);
        assert_eq!(verdict, GuardVerdict::Ask); // 单调: 后面 Allow 不能拉回
    }

    #[test]
    fn test_deny_beats_ask_regardless_of_order() {
        // ask 在 deny 之后也不影响 deny 胜出
        let mut chain = GuardChain::new();
        chain.add("first", |_, _| GuardVerdict::Deny);
        chain.add("second", |_, _| GuardVerdict::Ask);
        let (verdict, reasons) = chain.evaluate("any", &Value::Null);
        assert_eq!(verdict, GuardVerdict::Deny);
        assert_eq!(reasons.len(), 1);
    }

    #[test]
    fn test_verdict_names() {
        assert_eq!(GuardVerdict::Allow.name(), "allow");
        assert_eq!(GuardVerdict::Ask.name(), "ask");
        assert_eq!(GuardVerdict::Deny.name(), "deny");
    }

    #[test]
    fn test_multiple_ask_only_first_reason_captured() {
        let mut chain = GuardChain::new();
        chain.add("a", |_, _| GuardVerdict::Ask);
        chain.add("b", |_, _| GuardVerdict::Ask);
        let (verdict, reasons) = chain.evaluate("t", &Value::Null);
        assert_eq!(verdict, GuardVerdict::Ask);
        assert_eq!(reasons.len(), 1); // 裁决已 Ask, 后续 Ask 不重复记录
    }
}
