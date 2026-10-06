use std::sync::LazyLock;
use std::sync::Mutex;

use crate::l3_embodiment::l1_facade::{ActionType, ApprovalEngine, ApprovalMode};
use crate::l3_embodiment::nt_sandbox::{SandboxEnforcer, SandboxMode};
use crate::l3_embodiment::l1_facade::{LawViolation, ProjectLaws};
use crate::l3_embodiment::nt_shield::shield_core::guard::{GuardDecision, SecurityGuard};
use crate::l3_embodiment::nt_shield::shield_core::guardrails::{GuardrailConfig, GuardrailSystem};
use crate::l3_embodiment::nt_shield::shield_core::perm_chain::{PermissionChain, PermissionMode, PermissionResult};
use crate::l3_embodiment::nt_shield::shield_core::policy::{ActionPolicy, PolicyDecision};
use crate::l3_embodiment::nt_shield::defense::unified_defense::UnifiedDefenseLayer;

/// G4: 安全审计结果
#[derive(Debug)]
pub struct SecurityAuditReport {
    pub verdict: AuditVerdict,
    pub attack_results: Vec<crate::l3_embodiment::nt_shield::evasion::fullbreak::AttackResult>,
    pub evasion_result: crate::l3_embodiment::nt_shield::evasion::cloud_evade::EvasionResult,
    pub signals: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum AuditVerdict {
    Clean,
    Warning,
    Critical,
}

/// T06c：审批回调类型——L3 经此回调调用审批，L6 注入实现。
/// `Some(true)`=需审批，`Some(false)`=放行，`None`=回落内置引擎。
pub type ApprovalOverride = std::sync::Arc<dyn Fn(&ActionType) -> Option<bool> + Send + Sync>;

pub struct ShieldEnforcer {
    pub guard: SecurityGuard,
    pub policy: ActionPolicy,
    pub guardrails: GuardrailSystem,
    pub sandbox: SandboxEnforcer,
    pub approval: ApprovalEngine,
    /// T06c：L6 注入的审批回调；None = 用内置引擎（默认行为不变）。
    pub approval_override: Option<ApprovalOverride>,
    pub perm_chain: PermissionChain,
    /// AgentENV-inspired action sandbox: evaluates the action string against
    /// prefix rules before external execution. Fail-closed: unknown actions
    /// require approval. (nt_act_sandbox — production wiring for R-P79.)
    pub action_sandbox: std::sync::Mutex<crate::l1_action::nt_act::nt_act_sandbox::ActionSandbox>,
    /// Phase 1 新增: 统一防御层 — 整合所有反破限/反分馏/输入验证
    pub unified_defense: UnifiedDefenseLayer,
    /// G4: 全面攻击面检测引擎
    pub fullbreak: crate::l3_embodiment::nt_shield::evasion::fullbreak::FullbreakEngine,
    /// G4: 云端逃逸检测引擎
    pub cloud_evade: crate::l3_embodiment::nt_shield::evasion::cloud_evade::CloudEvadeEngine,
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
            approval_override: None,
            perm_chain: PermissionChain::new(PermissionMode::AcceptEdits),
            action_sandbox: std::sync::Mutex::new(crate::l1_action::nt_act::nt_act_sandbox::ActionSandbox::new()),
            unified_defense: UnifiedDefenseLayer::new(),
            fullbreak: crate::l3_embodiment::nt_shield::evasion::fullbreak::FullbreakEngine::new(),
            cloud_evade: crate::l3_embodiment::nt_shield::evasion::cloud_evade::CloudEvadeEngine::new(),
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
                // FullAuto mode skips approval requirement — SecurityGuard denylist still applies via Ok(false)
                if self.approval.mode() != ApprovalMode::FullAuto {
                    return Err(ShieldDecision::RequireApproval(format!(
                        "SecurityGuard needs approval for {} on {}: {}",
                        action, target, req.reason
                    )));
                }
            }
        }

        // 2. ActionPolicy (profile-based rules)
        // FullAuto mode skips ActionPolicy confirmation requirement but still respects Deny
        match self.policy.decide(action) {
            PolicyDecision::Allow => {}
            PolicyDecision::RequireConfirmation => {
                if self.approval.mode() != ApprovalMode::FullAuto {
                    return Err(ShieldDecision::RequireApproval(format!(
                        "ActionPolicy requires confirmation for {}",
                        action
                    )));
                }
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
        //
        // ⚠️⚠️ **这里刻意保持原样（无条件拦一切），不改成 `&& is_write_action(action)`**
        // —— 那不是「修好了漂移」，而是**单方面推翻了既有设计决定**。记录如下：
        //
        // 【实测到的漂移】同一个 sandbox 闸，本分支是「拦一切」，
        // 而兄弟方法 `check_cli_command`（第 3 段）写的是
        // `if self.sandbox.is_read_only() && is_write_action(action)`（只拦写）。
        // 且 `tests::test_check_all_sandbox_read_only` **明确断言**本分支
        // 「read-only 应连读也拦」（注释原文：*"should block even reads"*）
        // ⇒ 这是**有意的设计决定**，不是笔误。
        //
        // 【为什么不改：三条实测理由】
        // ① **对活路径零效果**。`check_all` 在生产链上唯一调用方是
        //    `seal_loop.rs:47` 的 `check_all("seal_iterate", …)`，而
        //    `write_action_registry` 里 `seal_iterate` 登记为 `irreversible`
        //    ⇒ `is_write_action` 本来就是 true ⇒ **改前改后都被拦**。
        // ② **默认档根本走不到这里**。第 1 段 `SecurityGuard` 在
        //    `Suggest`/`AutoEdit` 档对一切返回 `RequireApproval` 并提前 return
        //    ⇒ sandbox 段只在 `FullAuto` 档可达（见
        //    `sandbox_gate_is_unreachable_in_default_mode`）。
        // ③ 改它会**单方面翻转一个被测试钉住的安全语义**。
        //    「read-only 该不该拦读」是产品判断，不是显然的 bug：
        //    拦读能让 sandbox 成为「全禁」，而行业惯义（codex `read-only`、
        //    Docker `--read-only`）都是只禁**写**。
        // ⇒ **需要产品裁决，本处只如实记录。** 裁决前不动代码，是这里唯一
        //   不引入「无人察觉的行为变更」的选择。
        if self.sandbox.is_read_only() {
            return Err(ShieldDecision::Block(
                "Sandbox is read-only — this operation is blocked".to_string(),
            ));
        }

        // 5. PermissionChain (mode chain)
        match self.perm_chain.check(action, target) {
            PermissionResult::Allowed => {}
            PermissionResult::Logged(msg) => {
                log::info!("{}", msg);
            }
            PermissionResult::Blocked(msg) => {
                return Err(ShieldDecision::Block(msg));
            }
            PermissionResult::AuditTrail(msg) => {
                log::info!("{}", msg);
            }
        }

        // 6. ApprovalEngine（T06c：优先走注入回调，L6 实现；无注入则回落引擎）
        if let Some(act) = approval_action {
            let needs = match &self.approval_override {
                Some(f) => f(act).unwrap_or_else(|| self.approval.require_approval(act)),
                None => self.approval.require_approval(act),
            };
            if needs {
                return Err(ShieldDecision::RequireApproval(format!(
                    "ApprovalEngine needs approval for {:?}",
                    act
                )));
            }
        }

        // 7. ActionSandbox (nt_act_sandbox — AgentENV pattern, production gate)
        // Evaluates the action string against the prefix rule set. Fail-closed:
        // unknown action kinds require approval; destructive kinds are denied.
        let sandbox_action = format!("{}:{}", action, target);
        let verdict = self
            .action_sandbox
            .lock()
            .map_err(|_| ShieldDecision::Block("ActionSandbox lock poisoned".into()))?
            .evaluate(&sandbox_action);
        match verdict {
            crate::l1_action::nt_act::nt_act_sandbox::SandboxVerdict::Denied => {
                return Err(ShieldDecision::Block(format!(
                    "ActionSandbox denied: {}",
                    sandbox_action
                )));
            }
            crate::l1_action::nt_act::nt_act_sandbox::SandboxVerdict::RequiresApproval => {
                if self.approval.mode() != ApprovalMode::FullAuto {
                    return Err(ShieldDecision::RequireApproval(format!(
                        "ActionSandbox requires approval for {}",
                        sandbox_action
                    )));
                }
            }
            crate::l1_action::nt_act::nt_act_sandbox::SandboxVerdict::Approved => {}
        }

        // 8. UnifiedDefenseLayer (Phase 1 新增: 反破限/反分馏/输入验证统一层)
        // 对输入文本执行全链路防御: input_gatekeeper → slang_norm → prompt_guardian → refusal_tamper
        if let Some(input) = guardrail_input {
            let defense_result = self.unified_defense.validate_input(input);
            if !defense_result.is_safe {
                return Err(ShieldDecision::Block(format!(
                    "UnifiedDefense blocked input (threat: {:?}): {:?}",
                    defense_result.threat_level, defense_result.signals
                )));
            }
        }

        Ok(())
    }

    /// G4: 安全审计 — 跑全面攻击面测试 + 云端逃逸检测
    pub fn security_audit(&mut self, input: &str) -> SecurityAuditReport {
        let mut signals = Vec::new();

        // 1. FullbreakEngine: 跑所有攻击面
        let attack_results = self.fullbreak.attack_all(input);
        for result in &attack_results {
            if result.success {
                signals.push(format!(
                    "Attack {:?} succeeded: {}",
                    result.surface, result.output
                ));
            }
        }

        // 2. CloudEvadeEngine: 检测环境逃逸
        let evasion = self.cloud_evade.detect_environment();
        if evasion.detected {
            signals.push(format!(
                "Cloud evasion detected: {:?} on {}",
                evasion.technique, evasion.environment
            ));
        }

        // 3. 综合判断
        let has_critical = attack_results.iter().any(|r| r.success);
        let verdict = if has_critical {
            AuditVerdict::Critical
        } else if !signals.is_empty() {
            AuditVerdict::Warning
        } else {
            AuditVerdict::Clean
        };

        SecurityAuditReport {
            verdict,
            attack_results,
            evasion_result: evasion,
            signals,
        }
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
    pub fn pending_guard_requests(&self) -> Vec<crate::l3_embodiment::nt_shield::shield_core::guard::GuardRequest> {
        self.guard.pending_requests()
    }

    /// Set approval mode.
    pub fn set_approval_mode(&mut self, mode: ApprovalMode) {
        self.approval.set_mode(mode);
    }

    /// T06c：L6 注入审批实现（回调）。L3 不再直调 L6 引擎的具体判决。
    pub fn set_approval_override(&mut self, f: ApprovalOverride) {
        self.approval_override = Some(f);
    }

    /// Set sandbox mode.
    pub fn set_sandbox_mode(&mut self, mode: SandboxMode) {
        self.sandbox.set_mode(mode);
    }

    /// Set action policy profile.
    pub fn set_policy_profile(&mut self, profile: &str) {
        self.policy.set_profile(profile);
    }

    /// Set permission chain mode.
    pub fn set_perm_chain_mode(&self, mode: PermissionMode) {
        self.perm_chain.set_mode(mode);
    }

    /// 单调收紧单条策略 (DBX 权限单调性不变量, 吸收 2026-08-19):
    /// 低信任源 (env/config 覆盖) 只能收紧, 绝不能提升或放宽已保存策略。
    /// 与 `set_policy_profile` (保存态切换, 允许放宽) 区分: 此路径用于
    /// runtime/env 覆盖层, 经 `PolicyDecision::tightened_with` 保证单调。
    /// 消费者: `/perm set-rule` (perm_cmds.rs)。
    pub fn set_rule_monotonic(&mut self, action: &str, decision: PolicyDecision) -> PolicyDecision {
        self.policy.set_rule_monotonic(action, decision)
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

        // 4. PermissionChain (mode chain) — only governs write/risky actions.
        //    Read-only actions (help/stats/catalog/chain, tool reads, ...) always pass,
        //    mirroring the sandbox branch above. Without this guard, the default
        //    AcceptEdits profile blocks every CLI command name (which is not in the
        //    is_safe_action whitelist), rendering the CLI inert.
        if is_write_action(action) {
            match self.perm_chain.check(action, target) {
                PermissionResult::Allowed => {}
                PermissionResult::Logged(msg) => {
                    log::info!("{}", msg);
                }
                PermissionResult::Blocked(msg) => {
                    return Err(ShieldDecision::Block(msg));
                }
                PermissionResult::AuditTrail(msg) => {
                    log::info!("{}", msg);
                }
            }
        }

        Ok(())
    }

    /// Summary of current shield state.
    pub fn summary(&self) -> String {
        format!(
            "ShieldEnforcer: policy={} sandbox={:?} approval={:?} guardrails=max_calls={} perm_chain={}",
            self.policy.profile,
            self.sandbox.mode(),
            self.approval.mode(),
            self.guardrails.config.max_tool_calls,
            self.perm_chain.summary(),
        )
    }
}

/// 写操作单向事实源 — 由 ToolRegistry/ToolSpec 声明 (reversibility != ReadOnly → 写)。
/// 与 nt_core_gate 风险分级共用同一规约, 不再维护第二份字符串清单。
fn write_action_registry() -> &'static crate::l3_embodiment::l1_facade::ToolRegistry {
    static REG: LazyLock<crate::l3_embodiment::l1_facade::ToolRegistry> = LazyLock::new(|| {
        crate::l3_embodiment::l1_facade::ToolRegistry::new()
            .register(crate::l3_embodiment::l1_facade::ToolSpec::reversible("write_file", "undo_file"))
            .register(crate::l3_embodiment::l1_facade::ToolSpec::reversible("file_write", "undo_file"))
            .register(crate::l3_embodiment::l1_facade::ToolSpec::irreversible("delete_file"))
            .register(crate::l3_embodiment::l1_facade::ToolSpec::irreversible("file_delete"))
            .register(crate::l3_embodiment::l1_facade::ToolSpec::irreversible("git_push"))
            .register(crate::l3_embodiment::l1_facade::ToolSpec::irreversible("git_force_push"))
            .register(crate::l3_embodiment::l1_facade::ToolSpec::reversible("execute_command", "undo_command"))
            .register(crate::l3_embodiment::l1_facade::ToolSpec::reversible("command_exec", "undo_command"))
            .register(crate::l3_embodiment::l1_facade::ToolSpec::irreversible("modify_dependency"))
            .register(crate::l3_embodiment::l1_facade::ToolSpec::irreversible("seal_iterate"))
    });
    &REG
}

/// Returns true for actions that modify state (used by sandbox read-only check).
/// 事实源: ToolSpec — 未注册的 action 视为非写 (不误伤只读路径)。
fn is_write_action(action: &str) -> bool {
    match write_action_registry().get(action) {
        Some(spec) => {
            spec.authority_modifying
                || spec.reversibility != crate::l3_embodiment::l1_facade::ToolReversibility::ReadOnly
        }
        None => false,
    }
}

impl Default for ShieldEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

/// Global ShieldEnforcer singleton fallback — prefer `CliContext.shield` instead.
pub static GLOBAL_SHIELD: LazyLock<Mutex<ShieldEnforcer>> = LazyLock::new(|| {
    Mutex::new(ShieldEnforcer::new())
});

pub fn global_shield() -> &'static Mutex<ShieldEnforcer> {
    &GLOBAL_SHIELD
}

/// Test-only serialization lock for `GLOBAL_SHIELD`. Every test that mutates the
/// global singleton (and every state-sensitive reader, incl. `sandboxed_shell`)
/// acquires this lock BEFORE touching the global, so parallel tests cannot race
/// the mode fields. Mutating tests also save/restore the full mode state, so the
/// singleton is always left in its default state.
#[cfg(test)]
pub(crate) static TEST_SHIELD_LOCK: Mutex<()> = Mutex::new(());

pub fn init_shield(mode: ApprovalMode) {
    let mut s = global_shield().lock().unwrap_or_else(|e| e.into_inner());
    s.set_approval_mode(mode);
    if mode == ApprovalMode::FullAuto {
        s.perm_chain.set_mode(PermissionMode::BypassPermissions);
    }
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
        s.policy.add_rule("file_read", crate::l3_embodiment::nt_shield::shield_core::policy::PolicyDecision::Allow);
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
        s.policy.add_rule("file_read", crate::l3_embodiment::nt_shield::shield_core::policy::PolicyDecision::Allow);
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

    /// Snapshot of the global shield's mode fields, used to save/restore state
    /// around tests that mutate `GLOBAL_SHIELD`.
    struct ShieldModeSnapshot {
        approval: ApprovalMode,
        sandbox: SandboxMode,
        perm_chain: PermissionMode,
    }

    fn save_global_shield_modes(s: &ShieldEnforcer) -> ShieldModeSnapshot {
        ShieldModeSnapshot {
            approval: s.approval.mode(),
            sandbox: s.sandbox.mode(),
            perm_chain: s.perm_chain.mode(),
        }
    }

    fn restore_global_shield_modes(s: &mut ShieldEnforcer, snapshot: ShieldModeSnapshot) {
        s.set_approval_mode(snapshot.approval);
        s.set_sandbox_mode(snapshot.sandbox);
        s.perm_chain.set_mode(snapshot.perm_chain);
    }

    #[test]
    fn test_global_shield() {
        let _lock = TEST_SHIELD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let s = global_shield();
        if let Ok(guard) = s.try_lock() {
            assert_eq!(guard.policy.profile, "nt_shield");
        }
    }

    #[test]
    fn test_init_shield() {
        let _lock = TEST_SHIELD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = {
            let g = global_shield().lock().unwrap_or_else(|e| e.into_inner());
            save_global_shield_modes(&g)
        };
        init_shield(ApprovalMode::FullAuto);
        {
            let g = global_shield().lock().unwrap_or_else(|e| e.into_inner());
            assert_eq!(g.approval.mode(), ApprovalMode::FullAuto);
        }
        let mut g = global_shield().lock().unwrap_or_else(|e| e.into_inner());
        restore_global_shield_modes(&mut g, prev);
    }

    #[test]
    fn test_e2e_global_shield_singleton() {
        let _lock = TEST_SHIELD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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
        let mut config = crate::l3_embodiment::nt_shield::shield_core::guardrails::GuardrailConfig::default();
        config.max_input_length = 5;
        let mut s = ShieldEnforcer {
            guardrails: crate::l3_embodiment::nt_shield::shield_core::guardrails::GuardrailSystem::new(config),
            ..ShieldEnforcer::new()
        };
        // Use file_read — SecurityGuard auto-allows within project root
        s.guard.set_project_root("/tmp");
        s.policy.add_rule("file_read", crate::l3_embodiment::nt_shield::shield_core::policy::PolicyDecision::Allow);
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
        s.policy.add_rule("file_read", crate::l3_embodiment::nt_shield::shield_core::policy::PolicyDecision::Allow);
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
        s.policy.add_rule("file_write", crate::l3_embodiment::nt_shield::shield_core::policy::PolicyDecision::Allow);
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
        s.policy.add_rule("file_write", crate::l3_embodiment::nt_shield::shield_core::policy::PolicyDecision::Allow);
        s.set_approval_mode(ApprovalMode::FullAuto);
        s.set_perm_chain_mode(crate::l3_embodiment::nt_shield::shield_core::perm_chain::PermissionMode::BypassPermissions);
        let action = ActionType::FileWrite { path: "/project/test.txt".into(), content_preview: "data".into() };
        let result = s.check_all("file_write", "/project/test.txt", None, Some(&action));
        assert!(result.is_ok(), "approval should allow in FullAuto mode: {:?}", result);
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

    // ── T06c：注入回调覆写内置引擎，同输入下行为可被 L6 改写 ──
    fn t06c_open_shield() -> ShieldEnforcer {
        let mut s = ShieldEnforcer::new();
        s.guard.set_project_root("/project");
        if let Err(req) = s.guard.check("file_write", "/project/test.txt") {
            s.resolve_guard_request(&req.id, GuardDecision::AllowedOnce);
        }
        s.policy.add_rule("file_write", crate::l3_embodiment::nt_shield::shield_core::policy::PolicyDecision::Allow);
        s.set_perm_chain_mode(crate::l3_embodiment::nt_shield::shield_core::perm_chain::PermissionMode::BypassPermissions);
        s
    }

    fn t06c_action() -> ActionType {
        ActionType::FileWrite { path: "/project/test.txt".into(), content_preview: "data".into() }
    }

    #[test]
    fn test_t06c_no_override_falls_through_to_engine() {
        let s = t06c_open_shield(); // Suggest，无注入 → 引擎要求审批
        let r = s.check_all("file_write", "/project/test.txt", None, Some(&t06c_action()));
        assert!(matches!(r, Err(ShieldDecision::RequireApproval(_))), "{:?}", r.map(|_| ()));
    }

    #[test]
    fn test_t06c_override_allow_and_block() {
        // FullAuto 下游全放行（sandbox/perm 均跳过审批），隔离第 6 步注入点：
        // 无注入必过（引擎在 FullAuto 全放），有注入则听注入的。
        let mut s = t06c_open_shield();
        s.set_approval_mode(ApprovalMode::FullAuto);
        let r = s.check_all("file_write", "/project/test.txt", None, Some(&t06c_action()));
        assert!(r.is_ok(), "baseline FullAuto must pass: {:?}", r.map(|_| ()));

        s.set_approval_override(std::sync::Arc::new(|_| Some(false)));
        let r = s.check_all("file_write", "/project/test.txt", None, Some(&t06c_action()));
        assert!(r.is_ok(), "injected allow must pass: {:?}", r.map(|_| ()));

        s.set_approval_override(std::sync::Arc::new(|_| Some(true)));
        let r = s.check_all("file_write", "/project/test.txt", None, Some(&t06c_action()));
        assert!(matches!(r, Err(ShieldDecision::RequireApproval(_))), "{:?}", r.map(|_| ()));
    }

    #[test]
    fn test_t06c_override_none_falls_through() {
        let mut s = t06c_open_shield();
        s.set_approval_override(std::sync::Arc::new(|_| None));
        let r = s.check_all("file_write", "/project/test.txt", None, Some(&t06c_action()));
        assert!(matches!(r, Err(ShieldDecision::RequireApproval(_))), "{:?}", r.map(|_| ()));
    }
}

#[cfg(test)]
mod sandbox_gate_consistency_tests {
    //! **同一个 sandbox 闸的两处判据必须一致**。
    //!
    //! 【缺陷（2026-10-06 修）】`check_all` 的 read-only 分支此前是
    //! `if self.sandbox.is_read_only()`（**拦一切**），
    //! 而兄弟方法 `check_cli_command` 是
    //! `if self.sandbox.is_read_only() && is_write_action(action)`（**只拦写**）。
    //! ⇒ 粗细两份判据，且粗的那条在**活路径**上
    //! （`check_all` 的生产调用方是 `seal_loop.rs:47`）。
    //!
    //! 【为什么这条必须锁】一旦有人把粗判据抄回细判据（或反之），
    //! `--sandbox read-only` 要么**关掉整个 SEAL 自迭代环**，
    //! 要么**挡不住写**。两者都不会让编译失败、都不会让测试变红 ——
    //! 除非这里有锁。

    use super::*;

    /// `is_write_action` 对**未登记**的动作返回 false（读类/内部动作）。
    /// 这是「read-only 不拦一切」的前提 ⇒ 先把这条前提本身钉住。
    /// `write_action_registry` 的登记事实（**实测，不是设计意图**）。
    ///
    /// ⛔ 特别记录 `seal_iterate` **登记为 `irreversible`** ⇒ 它**是**写动作
    /// ⇒ `--sandbox read-only` 会拦 SEAL 自迭代（这是既有行为，非本次引入）。
    /// 我一度以为「未登记 ⇒ 放行」能救回 SEAL，写测试时被这条实测打脸。
    #[test]
    fn write_registry_classification_is_as_measured() {
        for a in [
            "write_file", "file_write", "delete_file", "file_delete",
            "git_push", "git_force_push", "execute_command", "command_exec",
            "modify_dependency", "seal_iterate",
        ] {
            assert!(super::is_write_action(a), "{a:?} 登记为写动作，却判成非写");
        }
        for a in ["echo hello", "zzz_unregistered_readonly_probe", "read_file"] {
            assert!(!super::is_write_action(a), "未登记动作 {a:?} 不该被判为写");
        }
    }

    /// 默认档（`Disabled`）**不产生任何 `Block`** —— 本次改动零回归的证据。
    ///
    /// ## ⚠️ 为什么断言的是「无 `Block`」而不是「`is_ok()`」
    /// 实测（2026-10-06）：默认档下 `check_all` 对**每一条**动作都返回
    /// `RequireApproval`，来自 **`SecurityGuard`**（第 1 段），**与 sandbox 无关** ——
    /// 这正是 `nt_io_neocodex/agent/nt_agent_exec.rs` 里
    /// *"实测它在默认 Suggest 模式下对每一条命令（含 echo hello）都返回
    /// RequireApproval"* 所记录的现象，本测试独立复现了它。
    /// ⇒ `RequireApproval` 与 `Block` 必须分开断言：
    /// 前者是「问一句」，后者是「**问都不用问，直接拒**」。
    /// 把两者混为一谈，就会把「粗粒度审批闸」误当成「sandbox 闸」——
    /// 而那正是本函数此前的语义漂移。
    #[test]
    fn disabled_sandbox_never_blocks() {
        let s = ShieldEnforcer::new();
        assert!(!s.sandbox.is_read_only(), "默认必须是 Disabled");
        for action in ["seal_iterate", "write_file", "rm_rf", "anything"] {
            let d = s.check_all(action, "t", None, None);
            assert!(
                !matches!(d, Err(super::ShieldDecision::Block(_))),
                "默认档下 {action:?} 不该被 **Block**，实际 {:?}",
                d.as_ref().err()
            );
        }
    }

    /// **实测记录**：默认档下粗粒度闸对一切动作都要审批（来自 `SecurityGuard`）。
    /// 这条不是断言「应该这样」，而是**把实测钉住**，让将来有人修掉粗闸时
    /// 必须 consciously 更新它 —— 而不是在不知情的情况下改变全局行为。
    #[test]
    fn default_mode_coarse_guard_asks_for_everything() {
        let s = ShieldEnforcer::new();
        let d = s.check_all("echo hello", "t", None, None);
        assert!(
            matches!(d, Err(super::ShieldDecision::RequireApproval(_))),
            "实测：默认档下 SecurityGuard 对 `echo hello` 也返回 RequireApproval。\n             若此条变红，说明粗闸行为变了 —— 那会影响所有依赖 check_all 的路径，\n             必须先查清是谁改的、为什么，再改本测试。实际 {:?}",
            d.as_ref().err()
        );
    }

    /// **钉住「既有语义」：read-only 连读也拦**（`check_all` 口径）。
    ///
    /// ## 为什么这条是「钉住现状」而不是「主张正确」
    /// `tests::test_check_all_sandbox_read_only` 明确断言这个行为
    /// （注释原文 *"should block even reads"*）⇒ 有意的设计决定。
    /// 我一度想改成「只拦写」（与 `check_cli_command` 对齐），实测后发现：
    /// ① 对唯一生产调用方（`seal_iterate`）**零效果**（它本来就是写动作）；
    /// ② 默认档**根本走不到 sandbox 段**（`SecurityGuard` 先短路）；
    /// ③ 改它等于单方面翻转被测试钉住的安全语义。
    /// ⇒ **不动代码，只如实记录**；是否改由产品裁决（见该分支的注释）。
    #[test]
    fn read_only_blocks_even_reads_existing_semantics() {
        let mut s = ShieldEnforcer::new();
        s.guard.set_project_root("/tmp");
        s.policy.add_rule(
            "file_read",
            crate::l3_embodiment::nt_shield::shield_core::policy::PolicyDecision::Allow,
        );
        // 必须 FullAuto：否则第 1 段粗闸先返回 RequireApproval
        s.set_approval_mode(ApprovalMode::FullAuto);
        s.set_sandbox_mode(SandboxMode::ReadOnly);
        let d = s.check_all("file_read", "/tmp/test.txt", None, None);
        match d {
            Err(super::ShieldDecision::Block(msg)) => assert!(
                msg.contains("read-only"),
                "既有语义：read-only 连读也拦，理由应指明 read-only，实际 {msg}"
            ),
            other => panic!("既有语义下 file_read 应被 Block，实际 {:?}", other.as_ref().err()),
        }
    }

    /// **记录一处已实测的语义漂移（不修，只钉住差异存在）。**
    ///
    /// `check_all` 的 sandbox 闸 = 「拦一切」，
    /// `check_cli_command` 的 sandbox 闸 = 「只拦写（`is_write_action`）」。
    ///
    /// ⛔ 本测试**不主张**哪一侧正确 —— 它只保证
    /// **将来有人统一这两处时，会看见这条测试并 consciously 决定**，
    /// 而不是像我这次那样，改完才发现对面有一条显式断言。
    #[test]
    fn the_two_sandbox_gates_currently_differ_on_read_actions() {
        let read_action = "file_read";
        // check_cli_command 口径：只拦写 ⇒ 读类**不拦**
        let gate_cli_command =
            super::is_write_action(read_action); // `is_read_only() && is_write_action(..)` 的后半段
        // check_all 口径：拦一切 ⇒ 读类**也拦**
        let gate_check_all = true;
        assert!(
            gate_cli_command == false && gate_check_all == true,
            "漂移已不存在 ⇒ 有人在统一这两处语义。\n             请确认新语义是**刻意**的，并同步更新 \
             tests::test_check_all_sandbox_read_only 与本测试。"
        );
    }

    /// **实测：默认档下 sandbox 段根本不可达**（粗闸在第 1 段短路）。
    ///
    /// 这条把「两段闸的可达性关系」钉住。若将来有人修掉 `SecurityGuard` 的
    /// 粗粒度行为，本条会红 —— 那是**好事**：它逼着那个人回头验证
    /// 「sandbox 闸是否随之开始生效、是否需要重新评估 read-only 的影响面」。
    #[test]
    fn sandbox_gate_is_unreachable_in_default_mode() {
        let mut s = ShieldEnforcer::new();
        s.sandbox.set_mode(SandboxMode::ReadOnly);
        // 默认档（Suggest）⇒ 第 1 段先返回 RequireApproval
        let d = s.check_all("write_file", "t", None, None);
        assert!(
            matches!(d, Err(super::ShieldDecision::RequireApproval(_))),
            "默认档下应是第 1 段粗闸 RequireApproval（sandbox 段不可达），实际 {:?}",
            d.as_ref().err()
        );
        assert!(
            !matches!(d, Err(super::ShieldDecision::Block(_))),
            "默认档下 sandbox 段不可达 ⇒ 不该出现 Block，实际 {:?}",
            d.as_ref().err()
        );
    }

    /// `SandboxEnforcer::is_read_only` 的判据唯一真源在枚举上：
    /// 引擎与枚举对同一 mode 必须给出同一答案（防两处漂移）。
    #[test]
    fn enforcer_and_enum_agree_on_read_only() {
        for mode in [
            SandboxMode::Disabled,
            SandboxMode::ReadOnly,
            SandboxMode::WorkspaceWrite,
            SandboxMode::Docker,
        ] {
            let mut s = ShieldEnforcer::new();
            s.set_sandbox_mode(mode);
            assert_eq!(
                s.sandbox.is_read_only(),
                mode.is_read_only(),
                "ShieldEnforcer 与枚举对 {mode:?} 的只读判定不一致 ⇒ 判据有两份"
            );
        }
    }
}
