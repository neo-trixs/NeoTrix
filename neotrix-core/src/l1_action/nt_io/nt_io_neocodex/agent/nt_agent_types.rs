// ── NeoCodex agent types: config / state / outcome + struct (split from agent.rs, behavior unchanged) ──

use std::sync::Arc;
use std::time::Instant;

use super::super::context::ContextPipeline;
use super::super::cost::CostTracker;
use super::super::evolution::{EvolutionLoop, NeoCodexSelfAudit};
use super::super::goals::GoalQueue;
use super::super::hooks::LifecycleHookRegistry;
use super::super::markdown::StreamingMarkdown;
use super::super::permissions::PermissionSystem;
use super::super::provider::{NeoCodexMode, ProviderCatalog};
use super::super::subagent::SubagentResult;
use super::super::wire::WireSession;

#[derive(Debug, Clone)]
pub struct NeoCodexConfig {
    pub mode: NeoCodexMode,
    pub max_turn_tokens: usize,
    pub provider_name: String,
    pub auto_compact: bool,
    pub shell_available: bool,
    pub thinking_enabled: bool,
    pub goal_mode: bool,
    /// P2-1: generation parameters surfaced from the desktop settings panel.
    /// Previously hardcoded (temperature 0.3 / max_tokens 4096) so the
    /// editable Settings temperature/maxTokens fields never reached the LLM.
    pub temperature: f64,
    pub max_tokens: u32,
}

impl Default for NeoCodexConfig {
    fn default() -> Self {
        Self {
            mode: NeoCodexMode::default(),
            max_turn_tokens: 0,
            provider_name: "neotrix".to_string(),
            auto_compact: true,
            shell_available: true,
            thinking_enabled: false,
            goal_mode: false,
            temperature: 0.3,
            max_tokens: 4096,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentState {
    pub turn_count: u64,
    pub tool_call_count: u64,
    pub tokens_used: usize,
    pub mode: NeoCodexMode,
    pub mode_start: Instant,
    pub goal_active: bool,
    /// Permission policy for the streaming path (P0-2). Mirrors Claude Code
    /// Manual/AcceptEdits/Plan and Codex approval modes. Stored on the agent
    /// because the desktop UI has no blocking stdin for `interactive_check`.
    pub permission_mode: String,
}

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentState {
    pub fn new() -> Self {
        Self {
            turn_count: 0,
            tool_call_count: 0,
            tokens_used: 0,
            mode: NeoCodexMode::Agent,
            mode_start: Instant::now(),
            goal_active: false,
            permission_mode: "auto".to_string(),
        }
    }
}

/// 流式生成的最终结果。`error` 有值时表示 provider 阶段发生错误：
/// 调用方应保留 `content`（已累积的 partial token）但**不**将其落盘为
/// 合法助手消息（F1 修复：避免 `[provider error]` 污染 wire/context）。
pub struct StreamOutcome {
    /// 完整或部分生成内容
    pub content: Option<String>,
    /// provider 错误信息（None = 正常完成）
    pub error: Option<String>,
}

pub struct NeoCodexAgent {
    pub config: NeoCodexConfig,
    pub state: AgentState,
    pub context: ContextPipeline,
    pub provider: ProviderCatalog,
    pub goals: GoalQueue,
    pub wire: WireSession,
    pub markdown: StreamingMarkdown,
    pub consciousness: Option<crate::l1_action::nt_action_facade::ConsciousnessTree>,
    pub event_bus: Option<crate::neotrix::nt_core_event_bus::EventBus>,
    pub brain: Option<
        Arc<tokio::sync::RwLock<dyn crate::l0_substrate::nt_core_traits::BrainHandle>>,
    >,
    // Cycle 112b additions
    pub hooks: LifecycleHookRegistry,
    pub cost: CostTracker,
    pub permissions: PermissionSystem,
    pub subagent_results: Vec<SubagentResult>,
    // Cycle 159 additions: self-audit + evolution loop
    pub evolution: EvolutionLoop,
    pub audit: NeoCodexSelfAudit,
    // Cycle 160e: tool grounding monitor (D25 production-wired, R-P49~R-P53)
    pub tool_grounding: crate::l1_action::nt_action_facade::ToolGroundingMonitor,
    // P2-5: MCP tool registry (Codex/Claude MCP parity). When registered, the
    // agent gains a `mcp_call` tool proxying to the registry; previously the
    // MCP host existed only for CLI/headless and the NeoCodex agent could not
    // call MCP tools despite the desktop UI having zero MCP surface.
    pub mcp: Option<serde_json::Value>,
}

impl NeoCodexAgent {
    pub fn new(session_id: &str) -> Self {
        Self {
            config: NeoCodexConfig::default(),
            state: AgentState::new(),
            context: ContextPipeline::new(100_000),
            provider: ProviderCatalog::new(),
            goals: GoalQueue::new(),
            wire: WireSession::new(session_id),
            markdown: StreamingMarkdown::new(),
            consciousness: None,
            event_bus: None,
            brain: None,
            hooks: LifecycleHookRegistry::new(),
            cost: CostTracker::new(10.0),
            permissions: PermissionSystem::new(),
            subagent_results: Vec::new(),
            evolution: EvolutionLoop::new(),
            audit: NeoCodexSelfAudit::new(),
            tool_grounding: crate::l1_action::nt_action_facade::ToolGroundingMonitor::new(),
            mcp: None,
        }
    }

    /// Set budget limit (from Claude Code max_budget_usd)
    pub fn with_budget(mut self, max_budget: f64) -> Self {
        self.cost.max_budget = max_budget;
        self
    }

    /// P2-1: set generation params from the desktop settings panel. Applies on
    /// the next request built by build_request (was previously hardcoded).
    pub(crate) fn _set_generation_params(&mut self, temperature: Option<f64>, max_tokens: Option<u32>) {
        if let Some(t) = temperature {
            self.config.temperature = t.clamp(0.0, 2.0);
        }
        if let Some(m) = max_tokens {
            self.config.max_tokens = m.max(1);
        }
    }
}
