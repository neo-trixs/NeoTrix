//! Hook 系统 — ECC 生命周期事件驱动自动化
//!
//! 参照 Everythind Claude Code (ECC) hook 架构：
//! - PreToolUse: 工具执行前触发，可阻断
//! - PostToolUse: 工具执行后触发，分析输出
//! - Stop: 每次响应后触发
//! - SessionStart/SessionEnd: 会话生命周期边界
//! - PreCompact: 上下文压缩前触发

use std::collections::HashMap;
use std::time::Instant;

/// Hook 事件类型（ECC 完整生命周期）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HookEvent {
    SessionStart,
    SessionEnd,
    PreToolUse,
    PostToolUse,
    Stop,
    PreCompact,
}

impl HookEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookEvent::SessionStart => "SessionStart",
            HookEvent::SessionEnd => "SessionEnd",
            HookEvent::PreToolUse => "PreToolUse",
            HookEvent::PostToolUse => "PostToolUse",
            HookEvent::Stop => "Stop",
            HookEvent::PreCompact => "PreCompact",
        }
    }
}

/// Hook 操作结果
#[derive(Debug, Clone)]
pub enum HookAction {
    /// 继续执行（exit code 0）
    Continue,
    /// 阻断工具执行（exit code 2，仅 PreToolUse）
    Block(String),
    /// 警告但不阻断（stderr）
    Warn(String),
}

/// Hook 上下文
#[derive(Debug, Clone)]
pub struct HookContext {
    pub event: HookEvent,
    pub tool_name: Option<String>,
    pub tool_input: Option<String>,
    pub tool_output: Option<String>,
    pub file_path: Option<String>,
    pub session_id: Option<String>,
    pub timestamp: Instant,
}

impl HookContext {
    pub fn new(event: HookEvent) -> Self {
        Self {
            event,
            tool_name: None,
            tool_input: None,
            tool_output: None,
            file_path: None,
            session_id: None,
            timestamp: Instant::now(),
        }
    }
}

/// Hook trait — 所有 Hook 需实现
pub trait Hook: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn events(&self) -> Vec<HookEvent>;
    /// 执行 Hook 逻辑
    fn execute(&self, ctx: &HookContext) -> HookAction;
    /// 是否为异步（非阻断）Hook
    fn is_async(&self) -> bool {
        false
    }
}

// ========== 默认 Hook 实现 ==========

/// 会话持久化 Hook（ECC SessionStart 风格）
pub struct SessionPersistenceHook;

impl Hook for SessionPersistenceHook {
    fn name(&self) -> &'static str {
        "session-persistence"
    }
    fn description(&self) -> &'static str {
        "保存/恢复会话上下文"
    }
    fn events(&self) -> Vec<HookEvent> {
        vec![
            HookEvent::SessionStart,
            HookEvent::SessionEnd,
            HookEvent::PreCompact,
        ]
    }
    fn execute(&self, ctx: &HookContext) -> HookAction {
        match ctx.event {
            HookEvent::SessionStart => HookAction::Continue,
            HookEvent::PreCompact => HookAction::Continue,
            HookEvent::SessionEnd => HookAction::Continue,
            _ => HookAction::Continue,
        }
    }
}

/// 质量门控 Hook（ECC PreToolUse 风格 — 检查大文件创建）
pub struct QualityGateHook;

impl Hook for QualityGateHook {
    fn name(&self) -> &'static str {
        "quality-gate"
    }
    fn description(&self) -> &'static str {
        "阻止创建超 800 行的文件"
    }
    fn events(&self) -> Vec<HookEvent> {
        vec![HookEvent::PreToolUse]
    }
    fn execute(&self, ctx: &HookContext) -> HookAction {
        if let Some(ref input) = ctx.tool_input {
            let lines = input.lines().count();
            if lines > 800 {
                return HookAction::Block(format!(
                    "File exceeds 800 lines ({} lines). Split into smaller modules.",
                    lines
                ));
            }
        }
        HookAction::Continue
    }
}

/// 安全检查 Hook（检测 TODO/FIXME/HACK 注释）
pub struct TodoWarningHook;

impl Hook for TodoWarningHook {
    fn name(&self) -> &'static str {
        "todo-warning"
    }
    fn description(&self) -> &'static str {
        "警告新增 TODO/FIXME/HACK 注释"
    }
    fn events(&self) -> Vec<HookEvent> {
        vec![HookEvent::PostToolUse]
    }
    fn is_async(&self) -> bool {
        true
    }
    fn execute(&self, ctx: &HookContext) -> HookAction {
        if let Some(ref output) = ctx.tool_output {
            if output.contains("TODO") || output.contains("FIXME") || output.contains("HACK") {
                return HookAction::Warn(
                    "New TODO/FIXME/HACK found — consider creating an issue".into(),
                );
            }
        }
        HookAction::Continue
    }
}

/// 会话摘要 Hook（ECC Stop 风格）
pub struct SessionSummaryHook;

impl Hook for SessionSummaryHook {
    fn name(&self) -> &'static str {
        "session-summary"
    }
    fn description(&self) -> &'static str {
        "在 Stop 事件时记录会话摘要"
    }
    fn events(&self) -> Vec<HookEvent> {
        vec![HookEvent::Stop]
    }
    fn is_async(&self) -> bool {
        true
    }
    fn execute(&self, _ctx: &HookContext) -> HookAction {
        HookAction::Continue
    }
}

// ========== Hook 注册表 ==========

const MAX_HOOKS: usize = 1000;

/// Hook 注册表 — 管理所有注册的 Hook
pub struct AgentHookRegistry {
    hooks: Vec<Box<dyn Hook>>,
    event_index: HashMap<HookEvent, Vec<usize>>,
    profile: HookProfile,
    disabled_hooks: Vec<String>,
}

/// Hook 执行模式（ECC 兼容）
#[derive(Debug, Clone)]
pub enum HookProfile {
    Minimal,
    Standard,
    Strict,
}

impl AgentHookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
            event_index: HashMap::new(),
            profile: HookProfile::Standard,
            disabled_hooks: Vec::new(),
        }
    }

    /// 注册一个 Hook
    pub fn register(&mut self, hook: Box<dyn Hook>) {
        if self.hooks.len() >= MAX_HOOKS {
            return;
        }
        let idx = self.hooks.len();
        for event in hook.events() {
            let entries = self.event_index.entry(event).or_default();
            if entries.len() >= MAX_HOOKS {
                continue;
            }
            entries.push(idx);
        }
        self.hooks.push(hook);
    }

    /// 注册默认 Hooks（ECC 标准配置文件）
    pub fn register_defaults(&mut self) {
        self.register(Box::new(SessionPersistenceHook));
        self.register(Box::new(QualityGateHook));
        self.register(Box::new(TodoWarningHook));
        self.register(Box::new(SessionSummaryHook));
    }

    /// 设置 Hook 执行模式
    pub fn set_profile(&mut self, profile: HookProfile) {
        self.profile = profile;
    }

    /// 禁用特定的 Hook（ECC ECC_DISABLED_HOOKS 兼容）
    pub fn disable_hook(&mut self, name: &str) {
        if !self.disabled_hooks.contains(&name.to_string()) {
            self.disabled_hooks.push(name.to_string());
        }
    }

    /// 为指定事件执行所有匹配的 Hook
    pub fn execute_event(&self, ctx: &HookContext) -> Vec<HookAction> {
        let mut actions = Vec::new();

        // 按 profile 过滤
        let allowed = match self.profile {
            HookProfile::Minimal => self.minimal_hooks(),
            HookProfile::Standard => self.standard_hooks(),
            HookProfile::Strict => self.strict_hooks(),
        };

        if let Some(indices) = self.event_index.get(&ctx.event) {
            for &idx in indices {
                if !allowed.contains(&idx) {
                    continue;
                }
                if self
                    .disabled_hooks
                    .contains(&self.hooks[idx].name().to_string())
                {
                    continue;
                }

                let hook = &self.hooks[idx];
                let action = hook.execute(ctx);
                actions.push(action);
            }
        }

        actions
    }

    /// 检查是否有阻断性 Hook 结果
    pub fn check_blocked(actions: &[HookAction]) -> Option<String> {
        for action in actions {
            if let HookAction::Block(msg) = action {
                return Some(msg.clone());
            }
        }
        None
    }

    fn minimal_hooks(&self) -> Vec<usize> {
        (0..self.hooks.len())
            .filter(|&i| {
                let h = &self.hooks[i];
                matches!(h.name(), "session-persistence")
            })
            .collect()
    }

    fn standard_hooks(&self) -> Vec<usize> {
        (0..self.hooks.len()).collect()
    }

    fn strict_hooks(&self) -> Vec<usize> {
        (0..self.hooks.len()).collect()
    }

    pub fn hook_count(&self) -> usize {
        self.hooks.len()
    }

    pub fn list_hooks(&self) -> Vec<(&str, &str)> {
        self.hooks
            .iter()
            .map(|h| (h.name(), h.description()))
            .collect()
    }
}

/// Backward-compatible alias
pub type HookRegistry = AgentHookRegistry;

impl Default for AgentHookRegistry {
    fn default() -> Self {
        let mut reg = Self::new();
        reg.register_defaults();
        reg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_events() {
        assert_eq!(HookEvent::SessionStart.as_str(), "SessionStart");
        assert_eq!(HookEvent::SessionEnd.as_str(), "SessionEnd");
        assert_eq!(HookEvent::PreToolUse.as_str(), "PreToolUse");
        assert_eq!(HookEvent::PostToolUse.as_str(), "PostToolUse");
        assert_eq!(HookEvent::Stop.as_str(), "Stop");
    }

    #[test]
    fn test_register_defaults() {
        let reg = HookRegistry::default();
        assert_eq!(reg.hook_count(), 4);
    }

    #[test]
    fn test_session_persistence_hook() {
        let hook = SessionPersistenceHook;
        let ctx = HookContext::new(HookEvent::SessionStart);
        let action = hook.execute(&ctx);
        assert!(matches!(action, HookAction::Continue));
    }

    #[test]
    fn test_quality_gate_blocks_large_input() {
        let hook = QualityGateHook;
        let mut ctx = HookContext::new(HookEvent::PreToolUse);
        ctx.tool_input = Some("line\n".repeat(900));
        let action = hook.execute(&ctx);
        assert!(matches!(action, HookAction::Block(_)));
    }

    #[test]
    fn test_quality_gate_allows_small_input() {
        let hook = QualityGateHook;
        let mut ctx = HookContext::new(HookEvent::PreToolUse);
        ctx.tool_input = Some("small content".to_string());
        let action = hook.execute(&ctx);
        assert!(matches!(action, HookAction::Continue));
    }

    #[test]
    fn test_todo_warning_hook() {
        let hook = TodoWarningHook;
        let mut ctx = HookContext::new(HookEvent::PostToolUse);
        ctx.tool_output = Some("Added new TODO: implement later".to_string());
        let action = hook.execute(&ctx);
        assert!(matches!(action, HookAction::Warn(_)));
    }

    #[test]
    fn test_todo_warning_clean() {
        let hook = TodoWarningHook;
        let mut ctx = HookContext::new(HookEvent::PostToolUse);
        ctx.tool_output = Some("Clean implementation".to_string());
        let action = hook.execute(&ctx);
        assert!(matches!(action, HookAction::Continue));
    }

    #[test]
    fn test_registry_execute_event() {
        let reg = HookRegistry::default();
        let ctx = HookContext::new(HookEvent::SessionStart);
        let actions = reg.execute_event(&ctx);
        assert_eq!(actions.len(), 1); // SessionPersistenceHook only
    }

    #[test]
    fn test_registry_check_blocked() {
        let actions = vec![
            HookAction::Continue,
            HookAction::Block("blocked!".to_string()),
        ];
        let blocked = HookRegistry::check_blocked(&actions);
        assert!(blocked.is_some());
        assert_eq!(blocked.expect("blocked should be ok in test"), "blocked!");
    }

    #[test]
    fn test_registry_no_blocked() {
        let actions = vec![HookAction::Continue, HookAction::Warn("warn".to_string())];
        let blocked = HookRegistry::check_blocked(&actions);
        assert!(blocked.is_none());
    }

    #[test]
    fn test_disable_hook() {
        let mut reg = HookRegistry::default();
        reg.disable_hook("quality-gate");
        // should not panic when checking
        let _ = reg.execute_event(&HookContext::new(HookEvent::PreToolUse));
    }

    #[test]
    fn test_hook_profile_minimal() {
        let mut reg = HookRegistry::default();
        reg.set_profile(HookProfile::Minimal);
        let ctx = HookContext::new(HookEvent::PreToolUse);
        let actions = reg.execute_event(&ctx);
        // minimal only allows session-persistence, so PreToolUse has no hooks
        assert!(actions.is_empty());
    }

    #[test]
    fn test_hook_list() {
        let reg = HookRegistry::default();
        let list = reg.list_hooks();
        assert_eq!(list.len(), 4);
        assert!(list.iter().any(|(n, _)| *n == "session-persistence"));
    }

    #[test]
    fn test_custom_hook() {
        struct TestHook;
        impl Hook for TestHook {
            fn name(&self) -> &'static str {
                "test"
            }
            fn description(&self) -> &'static str {
                "test hook"
            }
            fn events(&self) -> Vec<HookEvent> {
                vec![HookEvent::Stop]
            }
            fn execute(&self, _: &HookContext) -> HookAction {
                HookAction::Warn("test".into())
            }
        }

        let mut reg = HookRegistry::new();
        reg.register(Box::new(TestHook));
        assert_eq!(reg.hook_count(), 1);
        let ctx = HookContext::new(HookEvent::Stop);
        let actions = reg.execute_event(&ctx);
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], HookAction::Warn(_)));
    }
}
