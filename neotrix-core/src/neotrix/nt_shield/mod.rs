pub mod audit;
pub mod guard;
pub mod permissions;
pub mod policy;
pub mod guardrails;
pub mod tool_permissions;
#[cfg(feature = "sandbox")]
pub mod keyvault;
#[cfg(feature = "sandbox")]
pub mod vault;
pub mod cvss;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// 安全管理器 — 门控所有安全敏感操作
///
/// 对标 Grippy 确定性规则引擎 + OWASP Top 10:2025
pub struct SecurityManager {
    pub audit: audit::SecurityAudit,
    pub policy: policy::ActionPolicy,
    enabled: Arc<AtomicBool>,
}

impl SecurityManager {
    pub fn new() -> Self {
        Self {
            audit: audit::SecurityAudit::new(),
            policy: policy::ActionPolicy::new(),
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    /// 放行/禁止所有安全敏感操作
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    /// 检查操作是否允许执行
    pub fn check_action(&self, action: &str) -> bool {
        if !self.enabled.load(Ordering::SeqCst) {
            return false;
        }
        self.policy.evaluate(action)
    }

    /// 对整个项目路径执行安全审计
    pub fn audit_project(&self, path: &str) -> Vec<audit::SecurityFinding> {
        self.audit.scan_directory(path)
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}
