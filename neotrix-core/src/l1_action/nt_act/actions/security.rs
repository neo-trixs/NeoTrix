//! L1 CAT-6 安全 — 安全守卫能力
//!
//! 实现统一架构: L1Capability + SecurityGuard trait
//! 类别: CapabilityCategory::Security
//! 进化: C0→C1→C2→C3→C4→C5→C6
//!
//! 适配 L3 nt_shield guard 到 L1 统一 trait

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::l1_action::traits::{
    L1Capability, SecurityGuard, CapabilityCategory, ConstellationLevel,
    CapabilityHealth, CapabilityStats, CapabilityError,
    ActionRequest, SecurityVerdict, AuditEntry,
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
        Self { entries: Vec::new(), max_entries: 10000 }
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
        self.entries.iter()
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
    fn default() -> Self { Self::new() }
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
                if request.action.contains(&rule.action_pattern)
                    && request.target.contains(&rule.target_pattern)
                {
                    return rule.verdict.clone();
                }
            }
        }
        SecurityVerdict::Allow
    }
}

impl L1Capability for SecurityGuardManager {
    fn capability_id(&self) -> &str { "security.guard" }
    fn category(&self) -> CapabilityCategory { CapabilityCategory::Security }
    fn constellation(&self) -> ConstellationLevel { ConstellationLevel::C1UnitTest }
    fn health_check(&self) -> CapabilityHealth {
        CapabilityHealth {
            healthy: true,
            latency_ms: None,
            error_rate: 0.0,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            message: Some(format!("{} policies, {} audit entries", self.policies.len(), self.audit_log.entries.len())),
        }
    }
    fn description(&self) -> &str { "Security guard with policy-based access control and audit" }
    fn stats(&self) -> CapabilityStats { self.stats.clone() }
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
    fn default() -> Self { Self::new() }
}

impl SecurityRegistry {
    pub fn new() -> Self { Self { guards: Vec::new() } }
    pub fn register(&mut self, guard: Box<dyn SecurityGuard>) { self.guards.push(guard); }
    pub fn get(&self, id: &str) -> Option<&dyn SecurityGuard> {
        self.guards.iter().find(|g| g.capability_id() == id).map(|g| g.as_ref())
    }
    pub fn health_check_all(&self) -> Vec<(String, CapabilityHealth)> {
        self.guards.iter().map(|g| (g.capability_id().to_string(), g.health_check())).collect()
    }
    pub fn optimal(&self) -> Option<&dyn SecurityGuard> {
        self.guards.iter()
            .filter(|g| g.health_check().healthy)
            .max_by(|a, b| {
                let a_s = 1.0 - a.health_check().error_rate;
                let b_s = 1.0 - b.health_check().error_rate;
                a_s.partial_cmp(&b_s).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|g| g.as_ref())
    }
}

/// 安全路由器
pub struct SecurityRouter {
    registry: SecurityRegistry,
}

impl SecurityRouter {
    pub fn new(registry: SecurityRegistry) -> Self { Self { registry } }
    pub fn route(&self, _action: &ActionRequest) -> Option<&dyn SecurityGuard> { self.registry.optimal() }
    pub fn check(&self, action: &ActionRequest) -> SecurityVerdict {
        self.registry.optimal()
            .map(|g| g.check(action))
            .unwrap_or(SecurityVerdict::Deny("No security guard".into()))
    }
}

/// 安全桥接
pub struct SecurityBridge {
    router: SecurityRouter,
}

impl SecurityBridge {
    pub fn new(router: SecurityRouter) -> Self { Self { router } }
    pub fn check(&self, action: &ActionRequest) -> SecurityVerdict { self.router.check(action) }
}

// ════════════════════════════════════════════════════════════════
// 测试
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

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
