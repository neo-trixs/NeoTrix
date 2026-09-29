//! L1 CAT-6 安全 — 安全守卫能力
//!
//! 实现统一架构: L1Capability + SecurityGuard trait
//! 类别: CapabilityCategory::Security
//! 进化: C0→C1→C2→C3→C4→C5→C6
//!
//! 适配 L3 nt_shield guard 到 L1 统一 trait

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::l1_action::traits::{
    ActionRequest, AuditEntry, CapabilityCategory, CapabilityError, CapabilityHealth,
    CapabilityStats, ConstellationLevel, L1Capability, SecurityGuard, SecurityVerdict,
};

// ════════════════════════════════════════════════════════════════
// 安全策略
// ════════════════════════════════════════════════════════════════

/// 安全策略规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub id: String,
    pub name: String,
    pub rules: Vec<SecurityRule>,
    pub default_verdict: SecurityVerdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRule {
    pub action_pattern: String,
    pub target_pattern: String,
    pub verdict: SecurityVerdict,
    pub priority: u32,
}

/// 安全审计记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditLog {
    pub entries: Vec<AuditEntry>,
    pub max_entries: usize,
}

impl Default for SecurityAuditLog {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 10000,
        }
    }
}

impl SecurityAuditLog {
    pub fn record(&mut self, entry: AuditEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn query(&self, action: &str, limit: usize) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.action.contains(action))
            .rev()
            .take(limit)
            .collect()
    }
}

// ════════════════════════════════════════════════════════════════
// L1Capability + SecurityGuard 实现
// ════════════════════════════════════════════════════════════════

/// 安全守卫管理器
pub struct SecurityGuardManager {
    policies: Vec<SecurityPolicy>,
    audit_log: SecurityAuditLog,
    stats: CapabilityStats,
}

impl Default for SecurityGuardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityGuardManager {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
            audit_log: SecurityAuditLog::default(),
            stats: CapabilityStats::default(),
        }
    }

    pub fn add_policy(&mut self, policy: SecurityPolicy) {
        self.policies.push(policy);
    }

    pub fn evaluate(&self, request: &ActionRequest) -> SecurityVerdict {
        for policy in &self.policies {
            for rule in &policy.rules {
                // 2026-09-27 修复: target_pattern 是通配模式, 原用 `contains` 字面量比对
                // → `"*"` 永远匹配不到, 任何通配安全规则都无法 deny (fail-open)。
                let target_ok = rule.target_pattern == "*"
                    || rule.target_pattern.contains('*')
                    || request.target.contains(&rule.target_pattern);
                if request.action.contains(&rule.action_pattern) && target_ok {
                    return rule.verdict.clone();
                }
            }
        }
        SecurityVerdict::Allow
    }
}

impl L1Capability for SecurityGuardManager {
    fn capability_id(&self) -> &str {
        "security.guard"
    }
    fn category(&self) -> CapabilityCategory {
        CapabilityCategory::Security
    }
    fn constellation(&self) -> ConstellationLevel {
        ConstellationLevel::C1UnitTest
    }
    fn health_check(&self) -> CapabilityHealth {
        CapabilityHealth {
            healthy: true,
            latency_ms: None,
            error_rate: 0.0,
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            message: Some(format!(
                "{} policies, {} audit entries",
                self.policies.len(),
                self.audit_log.entries.len()
            )),
        }
    }
    fn description(&self) -> &str {
        "Security guard with policy-based access control and audit"
    }
    fn stats(&self) -> CapabilityStats {
        self.stats.clone()
    }
}

impl SecurityGuard for SecurityGuardManager {
    fn check(&self, action: &ActionRequest) -> SecurityVerdict {
        self.evaluate(action)
    }

    fn audit(&self, _entry: &AuditEntry) -> Result<(), CapabilityError> {
        // Audit is read-only in check; recording happens externally
        Ok(())
    }
}

// ════════════════════════════════════════════════════════════════
// Registry + Router + Bridge
// ════════════════════════════════════════════════════════════════

/// 安全能力注册中心
pub struct SecurityRegistry {
    guards: Vec<Box<dyn SecurityGuard>>,
}

impl Default for SecurityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityRegistry {
    pub fn new() -> Self {
        Self { guards: Vec::new() }
    }
    pub fn register(&mut self, guard: Box<dyn SecurityGuard>) {
        self.guards.push(guard);
    }
    pub fn get(&self, id: &str) -> Option<&dyn SecurityGuard> {
        self.guards
            .iter()
            .find(|g| g.capability_id() == id)
            .map(|g| g.as_ref())
    }
    pub fn health_check_all(&self) -> Vec<(String, CapabilityHealth)> {
        self.guards
            .iter()
            .map(|g| (g.capability_id().to_string(), g.health_check()))
            .collect()
    }
    /// 最严否决：遍历**所有健康 guard**，取严格程度最高的那次判决。
    ///
    /// ## 为什么不取「最健康的那个」（2026-09-29 修）
    ///
    /// 原实现返回 `optimal()` = **单个** `1.0 - error_rate` 最高的 guard。
    /// 那个标准回答的是「哪个 guard 更**可靠**」，
    /// 而路由需要回答的是「这个动作**允不允许**」——
    /// **两者不是同一个问题**。
    ///
    /// ⇒ 后果实测（`deny_in_any_guard_wins_over_healthier_allow` 为证）：
    /// 一个 `error_rate=0.0` 的宽松 `Allow` guard 会**静默屏蔽**
    /// `error_rate=0.5` 但判 `Deny` 的 guard ⇒ **fail-open**。
    ///
    /// ⇒ 正确语义 = 集合上的「最严否决」：
    /// 任一健康 guard 判 `Deny` ⇒ 整体 `Deny`；
    /// 否则任一判 `RequireApproval` ⇒ 整体 `RequireApproval`；
    /// 否则 `Allow`。
    ///
    /// ⛔ 不健康 guard 仍被跳过 —— 那部分语义不变
    ///   （宁可少一道 guard 也不要用一个正在误报的）。
    pub fn strictest_verdict(&self, action: &ActionRequest) -> Option<SecurityVerdict> {
        let mut seen_any = false;
        let mut saw_approval = false;
        for g in self.guards.iter().filter(|g| g.health_check().healthy) {
            seen_any = true;
            match g.check(action) {
                SecurityVerdict::Deny(_) => return Some(SecurityVerdict::Deny(
                    format!("guard `{}` 判拒绝", g.capability_id()),
                )),
                SecurityVerdict::RequireApproval(_) => saw_approval = true,
                SecurityVerdict::Allow => {}
            }
        }
        if !seen_any {
            return None;
        }
        if saw_approval {
            return Some(SecurityVerdict::RequireApproval(
                "至少一个 guard 要求人工批准".into(),
            ));
        }
        Some(SecurityVerdict::Allow)
    }
}

/// 安全路由器
pub struct SecurityRouter {
    registry: SecurityRegistry,
}

impl SecurityRouter {
    pub fn new(registry: SecurityRegistry) -> Self {
        Self { registry }
    }
    /// 路由判定：遍历**所有**健康 guard，取最严的那次判决。
    ///
    /// ## 语义变更（2026-09-29）
    ///
    /// 旧实现走 `registry.optimal()` = **单个** `1.0 - error_rate` 最高的 guard。
    /// 那个标准回答「哪个 guard 更**可靠**」，
    /// 而路由要回答「这个动作**允不允许**」—— 两者不是同一个问题。
    /// ⇒ 实测 fail-open：`error_rate=0.0` 的宽松 Allow 会静默屏蔽
    ///   `error_rate=0.5` 的 Deny（见 `deny_in_any_guard_wins_over_healthier_allow`）。
    ///
    /// ⛔ 同时**删除了 `route()`**（返回单个 guard 引用）与 `optimal()`
    ///   （按健康度选单个）：两者零外部消费者，而这个「挑一个 guard」
    ///   的形状本身会**诱导调用方重新引入 fail-open** ——
    ///   留着它等于把已修的缺陷留一个后门。
    ///   需要**枚举** guard 用 `health_check_all()` / `get(id)`。
    pub fn check(&self, action: &ActionRequest) -> SecurityVerdict {
        self.registry
            .strictest_verdict(action)
            .unwrap_or(SecurityVerdict::Deny("No security guard".into()))
    }
}

/// 安全桥接
pub struct SecurityBridge {
    router: SecurityRouter,
}

impl SecurityBridge {
    pub fn new(router: SecurityRouter) -> Self {
        Self { router }
    }
    pub fn check(&self, action: &ActionRequest) -> SecurityVerdict {
        self.router.check(action)
    }
}

// ════════════════════════════════════════════════════════════════
// 测试
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_security_guard_trait() {
        let mgr = SecurityGuardManager::new();
        assert_eq!(mgr.category(), CapabilityCategory::Security);
        assert_eq!(mgr.constellation(), ConstellationLevel::C1UnitTest);
        assert!(mgr.health_check().healthy);
    }

    #[test]
    fn test_check_allow() {
        let mgr = SecurityGuardManager::new();
        let req = ActionRequest {
            action: "read".into(),
            target: "file.txt".into(),
            parameters: HashMap::new(),
        };
        assert!(matches!(mgr.check(&req), SecurityVerdict::Allow));
    }

    #[test]
    fn test_policy_deny() {
        let mut mgr = SecurityGuardManager::new();
        mgr.add_policy(SecurityPolicy {
            id: "block_delete".into(),
            name: "Block delete".into(),
            rules: vec![SecurityRule {
                action_pattern: "delete".into(),
                target_pattern: "*".into(),
                verdict: SecurityVerdict::Deny("Delete not allowed".into()),
                priority: 1,
            }],
            default_verdict: SecurityVerdict::Allow,
        });
        let req = ActionRequest {
            action: "delete".into(),
            target: "important.db".into(),
            parameters: HashMap::new(),
        };
        assert!(matches!(mgr.check(&req), SecurityVerdict::Deny(_)));
    }

    /// 多 guard 聚合：**Deny 必须优先**，不得被「更健康」的 Allow 屏蔽。
    ///
    /// ## 针对的缺陷（2026-09-29 实测发现）
    ///
    /// `SecurityRouter::check()` 只取 `registry.optimal()` —— **单个** guard，
    /// 挑选标准是 `1.0 - error_rate`（**健康度**，与严格程度无关）。
    /// ⇒ 注册 2 个 guard 时，一个 `error_rate=0.0` 的宽松 `Allow` guard
    /// 会**静默屏蔽**掉 `error_rate=0.5` 但判 `Deny` 的 guard。
    ///
    /// ⛔ 这是 fail-open：**安全性被等同于「哪个 guard 更可靠」**。
    /// 正确语义是集合的**最严否决**：任一健康 guard 判 Deny ⇒ 整体 Deny。
    #[test]
    fn deny_in_any_guard_wins_over_healthier_allow() {
        struct LenientAllow {
            h: CapabilityHealth,
        }
        impl L1Capability for LenientAllow {
            fn capability_id(&self) -> &str {
                "lenient-allow"
            }
            fn health_check(&self) -> CapabilityHealth {
                self.h.clone()
            }
            fn category(&self) -> CapabilityCategory {
                CapabilityCategory::Security
            }
            fn constellation(&self) -> ConstellationLevel {
                ConstellationLevel::C1UnitTest
            }
            fn description(&self) -> &str {
                "lenient"
            }
        }
        impl SecurityGuard for LenientAllow {
            fn check(&self, _a: &ActionRequest) -> SecurityVerdict {
                SecurityVerdict::Allow
            }
            fn audit(&self, _e: &AuditEntry) -> Result<(), CapabilityError> {
                Ok(())
            }
        }

        struct StrictDeny {
            h: CapabilityHealth,
        }
        impl L1Capability for StrictDeny {
            fn capability_id(&self) -> &str {
                "strict-deny"
            }
            fn health_check(&self) -> CapabilityHealth {
                self.h.clone()
            }
            fn category(&self) -> CapabilityCategory {
                CapabilityCategory::Security
            }
            fn constellation(&self) -> ConstellationLevel {
                ConstellationLevel::C1UnitTest
            }
            fn description(&self) -> &str {
                "strict"
            }
        }
        impl SecurityGuard for StrictDeny {
            fn check(&self, _a: &ActionRequest) -> SecurityVerdict {
                SecurityVerdict::Deny("strict policy".into())
            }
            fn audit(&self, _e: &AuditEntry) -> Result<(), CapabilityError> {
                Ok(())
            }
        }

        let mut reg = SecurityRegistry::new();
        reg.register(Box::new(LenientAllow {
            h: CapabilityHealth {
                healthy: true,
                error_rate: 0.0, // 更"健康"
                ..Default::default()
            },
        }));
        reg.register(Box::new(StrictDeny {
            h: CapabilityHealth {
                healthy: true,
                error_rate: 0.5, // 较不"健康"
                ..Default::default()
            },
        }));

        let router = SecurityRouter::new(reg);
        let req = ActionRequest {
            action: "delete".into(),
            target: "important.db".into(),
            parameters: HashMap::new(),
        };
        assert!(
            matches!(router.check(&req), SecurityVerdict::Deny(_)),
            "任一 guard 判 Deny ⇒ 整体必须 Deny；不得被 error_rate 更低的 Allow 屏蔽"
        );
    }

    #[test]
    fn test_audit_log() {
        let mut log = SecurityAuditLog::default();
        log.record(AuditEntry {
            timestamp: 1000,
            action: "read".into(),
            actor: "user".into(),
            result: "success".into(),
        });
        assert_eq!(log.query("read", 10).len(), 1);
    }
}
