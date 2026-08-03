//! Hook 生命周期系统 — 25+ 事件点 × before/after
//! 集成到 SEAL pipeline、CLI 命令、GWT 广播的全生命周期

use std::collections::HashMap;
use crate::neotrix::l8_autonomic_impl::nt_mind::self_iterating::SelfIteratingBrain;

/// 钩子事件枚举 — 覆盖 SEAL 管线、CLI 命令、E8 推理、GWT 广播
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum HookEvent {
    // ── SEAL Pipeline 事件 (34 stages + 3 lifecycle) ──
    SealBeforeStage(String),
    SealAfterStage(String),
    SealBeforePipeline,
    SealAfterPipeline,
    SealCheckpoint,
    SealRollback,

    // ── CLI 命令事件 ──
    CliBeforeExecute(String),
    CliAfterExecute(String),
    CliCommandNotFound(String),

    // ── E8 推理事件 ──
    E8ReasoningStart,
    E8ReasoningComplete,
    E8ModeTransition(u8, u8),
    E8PolicyUpdate,

    // ── GWT 意识事件 ──
    GwtBroadcast,
    GwtResonanceCycle,
    GwtCompetitionWinner(String),
    GwtCompaction,

    // ── 子代理事件 ──
    SubagentSpawned(String),
    SubagentMessage(String, String),
    SubagentCompleted(String),

    // ── 计划事件 ──
    PlanCreated(String),
    PlanStepCompleted(String, usize),
    PlanFailed(String),

    // ── 会话事件 ──
    SessionStart,
    SessionEnd,
    SessionRecovered,

    // ── KB 事件 ──
    KbNodeCreated,
    KbSearch,

    // ── 学习事件 ──
    ConversationDistilled,
    EvolutionRecordCreated,

    // ── 技能引擎事件 ──
    SkillLoaded,
    SkillUnloaded,
}

/// 钩子上下文 — 传递给 HookAction 的运行时信息
#[derive(Debug, Clone)]
pub struct HookContext {
    pub event: HookEvent,
    pub message: String,
    pub brain_state: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub timestamp: u64,
}

impl HookContext {
    pub fn new(event: HookEvent, message: &str) -> Self {
        Self {
            event,
            message: message.to_string(),
            brain_state: None,
            payload: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
        }
    }

    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn with_brain(mut self, brain: &SelfIteratingBrain) -> Self {
        self.brain_state = Some(format!("iter={}, champion={:?}", brain.iteration, brain.champion.is_some()));
        self
    }
}

/// 钩子动作 trait — 用户可以注册自己的 hook action
pub trait HookAction: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, ctx: &HookContext) -> HookResult;
}

#[derive(Debug, Clone)]
pub struct HookResult {
    pub success: bool,
    pub message: String,
    pub effects: Vec<String>,
}

impl HookResult {
    pub fn ok(msg: &str) -> Self {
        Self { success: true, message: msg.to_string(), effects: vec![] }
    }

    pub fn err(msg: &str) -> Self {
        Self { success: false, message: msg.to_string(), effects: vec![] }
    }

    pub fn with_effect(mut self, effect: &str) -> Self {
        self.effects.push(effect.to_string());
        self
    }
}

/// 钩子注册表 — 管理所有已注册的 hook
pub struct MindHookRegistry {
    hooks: HashMap<HookEvent, Vec<Box<dyn HookAction>>>,
    execution_log: Vec<HookLogEntry>,
    max_log: usize,
}

struct HookLogEntry {
    event: HookEvent,
    action: String,
    success: bool,
    #[allow(dead_code)]
    message: String,
    timestamp: u64,
}

impl MindHookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
            execution_log: Vec::new(),
            max_log: 1000,
        }
    }

    pub fn register(&mut self, event: HookEvent, action: Box<dyn HookAction>) {
        self.hooks.entry(event).or_default().push(action);
    }

    pub fn unregister(&mut self, event: &HookEvent, name: &str) {
        if let Some(actions) = self.hooks.get_mut(event) {
            actions.retain(|a| a.name() != name);
        }
    }

    pub fn trigger(&mut self, ctx: &HookContext) -> Vec<HookResult> {
        let event = ctx.event.clone();
        let mut results = Vec::new();
        if let Some(actions) = self.hooks.get(&event) {
            for action in actions {
                let result = action.execute(ctx);
                if self.execution_log.len() >= self.max_log {
                    self.execution_log.remove(0);
                }
                self.execution_log.push(HookLogEntry {
                    event: event.clone(),
                    action: action.name().to_string(),
                    success: result.success,
                    message: result.message.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
                });
                results.push(result);
            }
        }
        results
    }

    pub fn hook_count(&self) -> usize {
        self.hooks.values().map(|v| v.len()).sum()
    }

    pub fn event_count(&self) -> usize {
        self.hooks.len()
    }

    pub fn recent_log(&self, limit: usize) -> Vec<String> {
        self.execution_log.iter().rev().take(limit).map(|e| {
            format!("[{}] {:?} -> {}: {}", e.timestamp, e.event, e.action, if e.success { "OK" } else { "FAIL" })
        }).collect()
    }

    pub fn clear_log(&mut self) {
        self.execution_log.clear();
    }
}

impl Default for MindHookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 日志钩子 — 将所有事件记录到日志
pub struct LogHook {
    pub prefix: String,
}

impl LogHook {
    pub fn new(prefix: &str) -> Self {
        Self { prefix: prefix.to_string() }
    }
}

impl HookAction for LogHook {
    fn name(&self) -> &str {
        "log_hook"
    }

    fn execute(&self, ctx: &HookContext) -> HookResult {
        log::info!("[{}] Event: {:?} — {}", self.prefix, ctx.event, ctx.message);
        HookResult::ok("logged")
    }
}

/// 提供所有预定义的 HookEvent 常量名
pub mod events {
    pub const ALL_HOOKS: &[&str] = &[
        "seal_before_each", "seal_after_each", "seal_before_pipeline", "seal_after_pipeline",
        "cli_before_execute", "cli_after_execute",
        "e8_reasoning_start", "e8_reasoning_complete",
        "gwt_broadcast", "gwt_resonance",
        "subagent_spawned", "subagent_message", "subagent_completed",
        "plan_created", "plan_step_completed", "plan_failed",
        "session_start", "session_end", "session_recovered",
        "kb_node_created", "kb_search",
        "conversation_distilled", "evolution_created",
        "skill_loaded", "skill_unloaded",
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHook;
    impl HookAction for TestHook {
        fn name(&self) -> &str { "test_hook" }
        fn execute(&self, ctx: &HookContext) -> HookResult {
            HookResult::ok(&format!("handled: {}", ctx.message))
        }
    }

    #[test]
    fn test_register_and_trigger() {
        let mut reg = MindHookRegistry::new();
        reg.register(HookEvent::SealBeforePipeline, Box::new(TestHook));
        let ctx = HookContext::new(HookEvent::SealBeforePipeline, "checkpoint");
        let results = reg.trigger(&ctx);
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }

    #[test]
    fn test_unregister() {
        let mut reg = MindHookRegistry::new();
        reg.register(HookEvent::E8ReasoningStart, Box::new(TestHook));
        assert_eq!(reg.hook_count(), 1);
        reg.unregister(&HookEvent::E8ReasoningStart, "test_hook");
        assert_eq!(reg.hook_count(), 0);
    }

    #[test]
    fn test_no_hooks_for_event() {
        let mut reg = MindHookRegistry::new();
        let ctx = HookContext::new(HookEvent::SealAfterPipeline, "nobody listening");
        let results = reg.trigger(&ctx);
        assert!(results.is_empty());
    }

    #[test]
    fn test_log_hook() {
        let mut reg = MindHookRegistry::new();
        reg.register(HookEvent::GwtBroadcast, Box::new(LogHook::new("test")));
        let ctx = HookContext::new(HookEvent::GwtBroadcast, "broadcast message");
        let results = reg.trigger(&ctx);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_hook_context_with_payload() {
        let ctx = HookContext::new(HookEvent::PlanCreated("test".into()), "new plan")
            .with_payload(serde_json::json!({"steps": 5}));
        assert!(ctx.payload.is_some());
        assert_eq!(ctx.payload.unwrap()["steps"], 5);
    }

    #[test]
    fn test_recent_log() {
        let mut reg = MindHookRegistry::new();
        reg.register(HookEvent::SessionStart, Box::new(TestHook));
        let ctx = HookContext::new(HookEvent::SessionStart, "started");
        reg.trigger(&ctx);
        let log = reg.recent_log(10);
        assert_eq!(log.len(), 1);
        assert!(log[0].contains("OK"));
    }
}
