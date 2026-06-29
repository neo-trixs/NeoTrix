pub mod agent_anomaly;
pub(crate) mod audit;
pub(crate) mod guard;
pub(crate) mod guardrails;
pub mod permissions; // re-enabled — PermissionManager used by src-tauri
pub(crate) mod policy;
pub(crate) mod tool_permissions;
// keyvault & vault — gated behind sandbox feature, but feature not in Cargo.toml
// Dependencies (aes-gcm, sha2, rand) exist, so compile them as pub(crate)
pub(crate) mod keyvault;
pub(crate) mod vault;
// CSS/UI design auditor — DIFFERENT from core::nt_core_consciousness::inner_critic (VSA thought quality gate)
// DESIGN REVIEW — belongs in nt_io_design_review (copy exists at neotrix/nt_io_design_review/)
pub(crate) mod inner_critic;
pub(crate) mod tool_inspection_stack;
pub use tool_inspection_stack::*;
pub(crate) mod check_registry;
pub(crate) mod network_egress;
pub(crate) mod network_enforcer;
pub use check_registry::*;
pub use network_egress::NetworkEgressPolicy;
pub use ast_safety_gate::{AstSafetyGate, SafetyRule, SafetyVerdict, SafetyViolation, Permission, PermissionTier, AstNodeType, RuleAction, SeverityLevel};
pub mod input_sanitizer;
pub mod static_code_detector; // was DEAD — re-enabled, referenced by check_registry
pub(crate) mod vsa_guard;
pub mod vulnerability_pipeline;
pub mod ast_safety_gate;
// pub mod hash_chain_audit; // DEAD — orphan, 0 consumers

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
