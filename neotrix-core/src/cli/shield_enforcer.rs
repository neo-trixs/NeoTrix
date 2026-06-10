use std::sync::OnceLock;
use std::sync::Mutex;

use crate::cli::approval::{ActionType, ApprovalEngine, ApprovalMode};
use crate::cli::sandbox::{SandboxEnforcer, SandboxMode};
use crate::cli::laws::{LawViolation, ProjectLaws};
use crate::neotrix::nt_shield::guard::{GuardDecision, SecurityGuard};
use crate::neotrix::nt_shield::guardrails::{GuardrailConfig, GuardrailSystem};
use crate::neotrix::nt_shield::policy::{ActionPolicy, PolicyDecision};

pub struct ShieldEnforcer {
    pub guard: SecurityGuard,
    pub policy: ActionPolicy,
    pub guardrails: GuardrailSystem,
    pub sandbox: SandboxEnforcer,
    pub approval: ApprovalEngine,
}

#[derive(Debug)]
pub enum ShieldDecision {
    Allow,
    Block(String),
    RequireApproval(String),
    Violation(Vec<LawViolation>),
}

impl ShieldEnforcer {
    pub fn new() -> Self {
        Self {
            guard: SecurityGuard::new(),
            policy: ActionPolicy::new(),
            guardrails: GuardrailSystem::new(GuardrailConfig::default()),
            sandbox: SandboxEnforcer::new(SandboxMode::Disabled),
            approval: ApprovalEngine::new(ApprovalMode::Suggest),
        }
    }

    pub fn with_mode(mode: ApprovalMode) -> Self {
        Self {
            approval: ApprovalEngine::new(mode),
            ..Self::new()
        }
    }

    /// Full short-circuit check chain.
    /// Returns Ok(()) if all pass, or the first blocking decision.
    pub fn check_all(
        &self,
        action: &str,
        target: &str,
        guardrail_input: Option<&str>,
        approval_action: Option<&ActionType>,
    ) -> Result<(), ShieldDecision> {
        // 1. SecurityGuard (denylist + session memory)
        match self.guard.check(action, target) {
            Ok(true) => {}
            Ok(false) => {
                return Err(ShieldDecision::Block(format!(
                    "SecurityGuard denied: {} on {} (denylist or session memory)",
                    action, target
                )));
            }
            Err(req) => {
                return Err(ShieldDecision::RequireApproval(format!(
                    "SecurityGuard needs approval for {} on {}: {}",
                    action, target, req.reason
                )));
            }
        }

        // 2. ActionPolicy (profile-based rules)
        match self.policy.decide(action) {
            PolicyDecision::Allow => {}
            PolicyDecision::RequireConfirmation => {
                return Err(ShieldDecision::RequireApproval(format!(
                    "ActionPolicy requires confirmation for {}",
                    action
                )));
            }
            PolicyDecision::Deny => {
                return Err(ShieldDecision::Block(format!(
                    "ActionPolicy denies {} (profile: {})",
                    action, self.policy.profile
                )));
            }
        }

        // 3. GuardrailSystem (input validation)
        if let Some(input) = guardrail_input {
            let guardrail_result = self.guardrails.check_tool_call(action, input, None);
            if !guardrail_result.passed {
                let details: Vec<String> = guardrail_result
                    .violations
                    .iter()
                    .map(|v| format!("[{}] {}: {}", v.severity as u8, v.rule, v.detail))
                    .collect();
                return Err(ShieldDecision::Block(format!(
                    "Guardrail blocked {}: {}",
                    action,
                    details.join("; ")
                )));
            }
        }

        // 4. SandboxEnforcer (read-only mode)
        if self.sandbox.is_read_only() {
            return Err(ShieldDecision::Block(
                "Sandbox is read-only — this operation is blocked".to_string(),
            ));
        }

        // 4.5 ProxyAllowlist (network access)
        if let Some(block) = self.sandbox.check_network_access(target) {
            return Err(ShieldDecision::Block(block.message));
        }

        // 5. ApprovalEngine
        if let Some(act) = approval_action {
            if self.approval.require_approval(act) {
                return Err(ShieldDecision::RequireApproval(format!(
                    "ApprovalEngine needs approval for {:?}",
                    act
                )));
            }
        }

        Ok(())
    }

    /// Check project laws against file content. Returns violations (non-blocking by default).
    pub fn check_laws(&self, content: &str, file_path: Option<&str>) -> Vec<LawViolation> {
        ProjectLaws::check_all(content, file_path)
    }

    /// Resolve a pending SecurityGuard request.
    pub fn resolve_guard_request(&self, id: &str, decision: GuardDecision) -> bool {
        self.guard.resolve(id, decision)
    }

    /// Get pending guard requests.
    pub fn pending_guard_requests(&self) -> Vec<crate::neotrix::nt_shield::guard::GuardRequest> {
        self.guard.pending_requests()
    }

    /// Set approval mode.
    pub fn set_approval_mode(&mut self, mode: ApprovalMode) {
        self.approval.set_mode(mode);
    }

    /// Set sandbox mode.
    pub fn set_sandbox_mode(&mut self, mode: SandboxMode) {
        self.sandbox.set_mode(mode);
    }

    /// Set action policy profile.
    pub fn set_policy_profile(&mut self, profile: &str) {
        self.policy.set_profile(profile);
    }

    /// Set project root on SecurityGuard (auto-allows reads within project).
    pub fn set_project_root(&self, root: &str) {
        self.guard.set_project_root(root);
    }

    /// Lightweight check for CLI command dispatch: only enforces hard blocks
    /// (denylist, guardrail violations). ActionPolicy and ApprovalEngine are
    /// handled by the individual command handlers.
    pub fn check_cli_command(&self, action: &str, target: &str) -> Result<(), ShieldDecision> {
        // 1. SecurityGuard (denylist)
        match self.guard.check(action, target) {
            Ok(true) => {}
            Ok(false) => {
                return Err(ShieldDecision::Block(format!(
                    "SecurityGuard denied: {} on {}", action, target
                )));
            }
            Err(_) => {} // Approval-based — let command handler deal with it
        }

        // 2. GuardrailSystem (input validation) — only for guardrail violations
        // Skip for CLI commands (no guardrail input to check)

        // 3. SandboxEnforcer
        if self.sandbox.is_read_only() && is_write_action(action) {
            return Err(ShieldDecision::Block(
                "Sandbox is read-only — this operation is blocked".to_string(),
            ));
        }

        Ok(())
    }

    /// Summary of current shield state.
    pub fn summary(&self) -> String {
        format!(
            "ShieldEnforcer: policy={} sandbox={:?} approval={:?} guardrails=max_calls={}",
            self.policy.profile,
            self.sandbox.mode(),
            self.approval.mode(),
            self.guardrails.config.max_tool_calls,
        )
    }
}

/// Returns true for actions that modify state (used by sandbox read-only check).
fn is_write_action(action: &str) -> bool {
    matches!(action,
        "write_file" | "delete_file" | "file_write" | "file_delete"
        | "git_push" | "git_force_push"
        | "execute_command" | "command_exec"
        | "modify_dependency" | "seal_iterate"
    )
}

impl Default for ShieldEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

/// Global ShieldEnforcer singleton.
static GLOBAL_SHIELD: OnceLock<Mutex<ShieldEnforcer>> = OnceLock::new();

pub fn global_shield() -> &'static Mutex<ShieldEnforcer> {
    GLOBAL_SHIELD.get_or_init(|| Mutex::new(ShieldEnforcer::new()))
}

pub fn init_shield(mode: ApprovalMode) {
    let mut s = global_shield().lock().expect("global_shield lock");
    s.set_approval_mode(mode);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shield_enforcer_new() {
        let s = ShieldEnforcer::new();
        assert_eq!(s.policy.profile, "nt_shield");
        assert_eq!(s.sandbox.mode(), SandboxMode::Disabled);
        assert_eq!(s.approval.mode(), ApprovalMode::Suggest);
    }

    #[test]
    fn test_shield_summary() {
        let s = ShieldEnforcer::new();
        let summary = s.summary();
        assert!(summary.contains("nt_shield"));
        assert!(summary.contains("Disabled"));
    }

    #[test]
    fn test_check_all_allows_read() {
        let mut s = ShieldEnforcer::new();
        s.guard.set_project_root("/tmp");
        // Pre-resolve SecurityGuard for file_read
        if let Err(req) = s.guard.check("file_read", "/tmp/test.txt") {
            s.resolve_guard_request(&req.id, GuardDecision::AllowedOnce);
        }
        // Add policy rule for file_read
        s.policy.add_rule("file_read", crate::neotrix::nt_shield::policy::PolicyDecision::Allow);
        let result = s.check_all("file_read", "/tmp/test.txt", None, None);
        assert!(result.is_ok(), "read within project should be allowed");
    }

    #[test]
    fn test_check_all_blocks_denylist() {
        let s = ShieldEnforcer::new();
        let result = s.check_all("file_write", "/etc/passwd", None, None);
        assert!(result.is_err(), "write to blocked path should be denied");
    }

    #[test]
    fn test_check_all_sandbox_read_only() {
        let mut s = ShieldEnforcer::new();
        // Use file_read — SecurityGuard auto-allows within project root
        s.guard.set_project_root("/tmp");
        // Add policy rule for file_read
        s.policy.add_rule("file_read", crate::neotrix::nt_shield::policy::PolicyDecision::Allow);
        // Bypass ApprovalEngine
        s.set_approval_mode(ApprovalMode::FullAuto);
        // Enable read-only sandbox — should block even reads
        s.set_sandbox_mode(SandboxMode::ReadOnly);
        let result = s.check_all("file_read", "/tmp/test.txt", None, None);
        assert!(result.is_err(), "sandbox should block in read-only mode");
        match result.unwrap_err() {
            ShieldDecision::Block(msg) => assert!(msg.contains("read-only")),
            _ => panic!("expected Block"),
        }
    }

    #[test]
    fn test_check_laws_violations() {
        let s = ShieldEnforcer::new();
        let content = r#"let api_key = "sk-1234567890abcdef1234567890abcdef";"#;
        let violations = s.check_laws(content, Some("src/main.rs"));
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.code == "L001"));
    }

    #[test]
    fn test_global_shield() {
        let s = global_shield();
        if let Ok(guard) = s.try_lock() {
            assert_eq!(guard.policy.profile, "nt_shield");
        }
    }

    #[test]
    fn test_init_shield() {
        init_shield(ApprovalMode::FullAuto);
        let s = global_shield();
        if let Ok(guard) = s.try_lock() {
            assert_eq!(guard.approval.mode(), ApprovalMode::FullAuto);
            // Reset for other tests
            drop(guard);
            if let Ok(mut g) = s.try_lock() {
                g.set_approval_mode(ApprovalMode::Suggest);
            }
        }
    }

    #[test]
    fn test_e2e_global_shield_singleton() {
        let s1 = global_shield();
        if let Ok(g1) = s1.try_lock() {
            let profile = g1.policy.profile.clone();
            drop(g1);
            let s2 = global_shield();
            if let Ok(g2) = s2.try_lock() {
                assert_eq!(g2.policy.profile, profile);
            }
        }
    }

    #[test]
    fn test_resolve_guard_request() {
        let s = ShieldEnforcer::new();
        s.guard.set_project_root("/p");
        let result = s.guard.check("file_write", "/p/test.txt");
        assert!(result.is_err());
        let req = result.unwrap_err();
        assert!(s.resolve_guard_request(&req.id, GuardDecision::AllowedOnce));
        assert_eq!(s.guard.check("file_write", "/p/test.txt"), Ok(true));
    }

    #[test]
    fn test_set_policy_profile() {
        let mut s = ShieldEnforcer::new();
        s.set_policy_profile("general");
        assert_eq!(s.policy.profile, "general");
    }

    #[test]
    fn test_set_project_root() {
        let s = ShieldEnforcer::new();
        s.set_project_root("/project");
        assert!(s.guard.check("file_read", "/project/src/lib.rs").unwrap_or(false));
    }

    #[test]
    fn test_check_all_policy_deny() {
        let mut s = ShieldEnforcer::new();
        // Bypass SecurityGuard
        if let Err(req) = s.guard.check("read_secrets", "/tmp/x") {
            s.resolve_guard_request(&req.id, GuardDecision::AllowedOnce);
        }
        // ApprovalEngine would also block — bypass
        s.approval.set_mode(ApprovalMode::FullAuto);
        // Policy should deny read_secrets (default rule)
        let result = s.check_all("read_secrets", "/tmp/x", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_guardrail_blocks_long_input() {
        let mut config = crate::neotrix::nt_shield::guardrails::GuardrailConfig::default();
        config.max_input_length = 5;
        let mut s = ShieldEnforcer {
            guardrails: crate::neotrix::nt_shield::guardrails::GuardrailSystem::new(config),
            ..ShieldEnforcer::new()
        };
        // Use file_read — SecurityGuard auto-allows within project root
        s.guard.set_project_root("/tmp");
        s.policy.add_rule("file_read", crate::neotrix::nt_shield::policy::PolicyDecision::Allow);
        s.set_approval_mode(ApprovalMode::FullAuto);
        // Guardrail blocks long input
        let result = s.check_all("file_read", "/tmp/test.txt", Some("very long input that exceeds the limit"), None);
        assert!(result.is_err());
    }

    // === R9: E2E integration tests ===

    #[test]
    fn test_e2e_full_chain_allows_clean_read() {
        let mut s = ShieldEnforcer::new();
        s.guard.set_project_root("/project");
        if let Err(req) = s.guard.check("file_read", "/project/src/lib.rs") {
            s.resolve_guard_request(&req.id, GuardDecision::AllowedOnce);
        }
        s.policy.add_rule("file_read", crate::neotrix::nt_shield::policy::PolicyDecision::Allow);
        s.set_approval_mode(ApprovalMode::FullAuto);
        let result = s.check_all("file_read", "/project/src/lib.rs", None, None);
        assert!(result.is_ok(), "full chain should allow clean read");
    }

    #[test]
    fn test_e2e_full_chain_blocks_denylist_path() {
        let s = ShieldEnforcer::new();
        let result = s.check_all("file_write", "/etc/passwd", None, None);
        assert!(result.is_err(), "full chain should block /etc/passwd");
        match result.unwrap_err() {
            ShieldDecision::Block(_) => {}
            other => panic!("expected Block, got {:?}", other),
        }
    }

    #[test]
    fn test_e2e_approval_engine_blocks_in_suggest_mode() {
        let mut s = ShieldEnforcer::new();
        s.guard.set_project_root("/project");
        if let Err(req) = s.guard.check("file_write", "/project/test.txt") {
            s.resolve_guard_request(&req.id, GuardDecision::AllowedOnce);
        }
        s.policy.add_rule("file_write", crate::neotrix::nt_shield::policy::PolicyDecision::Allow);
        let action = ActionType::FileWrite { path: "/project/test.txt".into(), content_preview: "data".into() };
        let result = s.check_all("file_write", "/project/test.txt", None, Some(&action));
        assert!(result.is_err(), "approval should block in suggest mode");
    }

    #[test]
    fn test_e2e_approval_engine_allows_in_full_auto() {
        let mut s = ShieldEnforcer::new();
        s.guard.set_project_root("/project");
        if let Err(req) = s.guard.check("file_write", "/project/test.txt") {
            s.resolve_guard_request(&req.id, GuardDecision::AllowedOnce);
        }
        s.policy.add_rule("file_write", crate::neotrix::nt_shield::policy::PolicyDecision::Allow);
        s.set_approval_mode(ApprovalMode::FullAuto);
        let action = ActionType::FileWrite { path: "/project/test.txt".into(), content_preview: "data".into() };
        let result = s.check_all("file_write", "/project/test.txt", None, Some(&action));
        assert!(result.is_ok(), "approval should allow in FullAuto mode");
    }

    #[test]
    fn test_e2e_check_laws_integration() {
        let s = ShieldEnforcer::new();
        let content = r#"
            let api_key = "sk-1234567890abcdef1234567890abcdef";
            unsafe { transmute(x) }
            let x = val.unwrap();
        "#;
        let violations = s.check_laws(content, Some("src/main.rs"));
        assert!(violations.iter().any(|v| v.code == "L001"), "should detect L001");
        assert!(violations.iter().any(|v| v.code == "L002"), "should detect L002");
        assert!(violations.iter().any(|v| v.code == "L003"), "should detect L003");
    }

    #[test]
    fn test_e2e_policy_profile_switch_reconfigures_chain() {
        let mut s = ShieldEnforcer::new();
        assert_eq!(s.policy.profile, "nt_shield");
        s.set_policy_profile("general");
        assert_eq!(s.policy.profile, "general");
        s.set_policy_profile("strict-nt_shield");
        assert_eq!(s.policy.profile, "strict-nt_shield");
        s.set_sandbox_mode(SandboxMode::ReadOnly);
        assert_eq!(s.sandbox.mode(), SandboxMode::ReadOnly);
    }

    #[test]
    fn test_e2e_project_laws_describe_all() {
        for code in &["L001", "L002", "L003", "L004", "L005", "L006", "L007", "L008", "L009", "L010"] {
            assert!(ProjectLaws::describe(code).is_some(), "{} should have description", code);
        }
    }
}
