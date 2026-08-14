pub mod audit;
pub mod guard;
pub mod guard_chain;
pub mod key_encryption;
pub mod perm_chain;
pub mod permissions;
pub mod policy;
pub mod guardrails;
pub mod tool_permissions;
pub mod tool_inspection_stack;
#[cfg(feature = "sandbox")]
pub mod keyvault;
#[cfg(feature = "sandbox")]
pub mod vault;
pub mod cvss;
pub mod nt_shield_mcp_security;
pub mod safety_kernel;
pub mod check_registry;
pub mod http_proxy;
pub mod poc_engine;
pub mod browser_security;
pub mod redaction;
pub mod nt_shield_secret_collector;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use self::nt_shield_mcp_security::SecurityMcpToolRegistry;
use self::safety_kernel::{SafetyKernel, ActionRequest, ActionType, SafetyDecision};
use self::tool_inspection_stack::{ToolInspectionStack, InspectionResult};
use self::check_registry::{CheckRegistry, CheckVerdict, ToolSource};
use self::browser_security::{BrowserSecurityScanner, BrowserSecurityConfig};

/// 安全管理器 — 门控所有安全敏感操作
///
/// 对标 Grippy 确定性规则引擎 + OWASP Top 10:2025
/// 集成 Execution-Time AI Alignment SafetyKernel (P0)
pub struct SecurityManager {
    pub audit: audit::SecurityAudit,
    pub policy: policy::ActionPolicy,
    enabled: Arc<AtomicBool>,
    pub execution_guard: Option<Arc<SafetyKernel>>,
    pub tool_inspector: Arc<ToolInspectionStack>,
    pub check_registry: Arc<CheckRegistry>,
    pub browser_scanner: Option<Arc<BrowserSecurityScanner>>,
    pub mcp_security: SecurityMcpToolRegistry,
}

impl SecurityManager {
    pub fn new() -> Self {
        let mut browser_scanner = BrowserSecurityScanner::new(BrowserSecurityConfig::default());
        browser_scanner.register_default_checks();

        let mut mcp_security = SecurityMcpToolRegistry::new();
        mcp_security.register_defaults();

        Self {
            audit: audit::SecurityAudit::new(),
            policy: policy::ActionPolicy::new(),
            enabled: Arc::new(AtomicBool::new(true)),
            execution_guard: None,
            tool_inspector: Arc::new(ToolInspectionStack::with_defaults()),
            check_registry: Arc::new(CheckRegistry::new()),
            browser_scanner: Some(Arc::new(browser_scanner)),
            mcp_security,
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    pub fn check_action(&self, action: &str) -> bool {
        if !self.enabled.load(Ordering::SeqCst) {
            return false;
        }

        if let Some(ref kernel) = self.execution_guard {
            let action_type = match action {
                "read_file" | "file_read" => ActionType::FileRead,
                "write_file" | "file_write" => ActionType::FileWrite,
                "delete_file" | "file_delete" => ActionType::FileDelete,
                "network_request" | "network" => ActionType::NetworkRequest,
                "execute_command" | "exec" => ActionType::SubprocessExec,
                "code_eval" | "eval" => ActionType::CodeEval,
                "data_export" | "export" => ActionType::DataExport,
                "model_access" | "model" => ActionType::ModelAccess,
                "tool_call" | "tool" => ActionType::ToolCall,
                _ => ActionType::Custom(action.to_string()),
            };

            let request = ActionRequest {
                action_id: format!("check_action:{}", action),
                action_type,
                target: String::new(),
                args: std::collections::HashMap::new(),
                context: std::collections::HashMap::new(),
                risk_score: 0.0,
            };

            let evidence = kernel.check(&request);
            match evidence.decision {
                SafetyDecision::Allowed { .. } => {
                    self.policy.evaluate(action)
                }
                SafetyDecision::Denied { .. } | SafetyDecision::RequiresApproval { .. } => {
                    false
                }
            }
        } else {
            self.policy.evaluate(action)
        }
    }

    pub fn attach_safety_kernel(&mut self, kernel: Arc<SafetyKernel>) {
        self.execution_guard = Some(kernel);
    }

    pub fn detach_safety_kernel(&mut self) {
        self.execution_guard = None;
    }

    /// Run the full security inspection stack before a tool call
    ///
    /// Returns Ok(()) if all checks pass, or the first blocking result.
    pub fn inspect_tool(&self, tool_name: &str, args: &serde_json::Value) -> Result<(), String> {
        if !self.enabled.load(Ordering::SeqCst) {
            return Ok(());
        }

        // 1. Tool inspection stack (5 layers): security, egress, permission, repetition, build
        let results = self.tool_inspector.inspect(tool_name, args);
        for (inspector_name, result) in &results {
            match result {
                InspectionResult::Deny(reason) => {
                    return Err(format!("[{}] Denied: {}", inspector_name, reason));
                }
                InspectionResult::RequireApproval(reason) => {
                    return Err(format!("[{}] Requires approval: {}", inspector_name, reason));
                }
                InspectionResult::Allow => {}
            }
        }

        // 2. Check registry (22 security rules)
        let verdicts = self.check_registry.evaluate(tool_name, args, &ToolSource::Consciousness);
        for (check_id, verdict) in &verdicts {
            match verdict {
                CheckVerdict::Fail(reason) => {
                    return Err(format!("[{}] Check failed: {}", check_id, reason));
                }
                CheckVerdict::Warn(reason) => {
                    // Warnings are informational — don't block
                    log::warn!("[{}] Warning: {}", check_id, reason);
                }
                CheckVerdict::Pass => {}
            }
        }

        Ok(())
    }

    pub fn audit_project(&self, path: &str) -> Vec<audit::SecurityFinding> {
        // .rs 安全规则扫描 + .md 引用真实性审计 (P1-16: academic-research-skills)
        let mut findings = self.audit.scan_directory(path);
        findings.extend(self.audit.scan_documents(path));
        findings
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}
