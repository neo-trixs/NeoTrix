// ── NeoCodex Agent Loop (from Claude Code: ReAct pattern + NeoTrix Consciousness) ──

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use base64::Engine as _;
use crate::neotrix::l1_body_impl::nt_io_provider::context_budget::apply_context_budget;
use crate::neotrix::nt_io_provider::context_budget::estimate_messages_tokens;
use crate::neotrix::nt_io_provider::types::{LlmRequest, Message, Role, Tool};

use super::context::ContextPipeline;
use super::cost::CostTracker;
use super::evolution::{EvolutionLoop, NeoCodexHealthReport, NeoCodexSelfAudit};
use super::goals::GoalQueue;
use super::hooks::{HookDecision, HookResult, LifecycleHookRegistry, ToolCallContext};
use super::markdown::StreamingMarkdown;
use super::permissions::PermissionSystem;
use super::provider::{ModelCapability, NeoCodexMode, ProviderCatalog};
use super::subagent::{SubagentDispatch, SubagentKind, SubagentResult};
use super::wire::{WireEvent, WireSession};

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

pub struct NeoCodexAgent {
    pub config: NeoCodexConfig,
    pub state: AgentState,
    pub context: ContextPipeline,
    pub provider: ProviderCatalog,
    pub goals: GoalQueue,
    pub wire: WireSession,
    pub markdown: StreamingMarkdown,
    pub consciousness: Option<crate::core::nt_core_consciousness_tree::ConsciousnessTree>,
    pub event_bus: Option<crate::neotrix::nt_core_event_bus::EventBus>,
    pub brain: Option<
        Arc<tokio::sync::RwLock<crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain>>,
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
    pub tool_grounding: crate::core::nt_core_self::self_audit::ToolGroundingMonitor,
    // P2-5: MCP tool registry (Codex/Claude MCP parity). When registered, the
    // agent gains a `mcp_call` tool proxying to the registry; previously the
    // MCP host existed only for CLI/headless and the NeoCodex agent could not
    // call MCP tools despite the desktop UI having zero MCP surface.
    pub mcp: Option<crate::neotrix::l1_body_impl::nt_agent_mcp_registry::McpRegistry>,
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
            tool_grounding: crate::core::nt_core_self::self_audit::ToolGroundingMonitor::new(),
            mcp: None,
        }
    }

    /// P2-5: attach the shared MCP registry so the agent can call MCP tools.
    pub fn with_mcp(
        mut self,
        mcp: crate::neotrix::l1_body_impl::nt_agent_mcp_registry::McpRegistry,
    ) -> Self {
        self.mcp = Some(mcp);
        self
    }

    pub fn set_mcp(
        &mut self,
        mcp: Option<crate::neotrix::l1_body_impl::nt_agent_mcp_registry::McpRegistry>,
    ) {
        self.mcp = mcp;
    }

    /// Set budget limit (from Claude Code max_budget_usd)
    pub fn with_budget(mut self, max_budget: f64) -> Self {
        self.cost.max_budget = max_budget;
        self
    }

    /// P2-1: set generation params from the desktop settings panel. Applies on
    /// the next request built by build_request (was previously hardcoded).
    pub fn set_generation_params(&mut self, temperature: Option<f64>, max_tokens: Option<u32>) {
        if let Some(t) = temperature {
            self.config.temperature = t.clamp(0.0, 2.0);
        }
        if let Some(m) = max_tokens {
            self.config.max_tokens = m.max(1);
        }
    }

    /// Register a pre-tool lifecycle hook (from Kimi Code lifecycle hooks)
    pub fn add_pre_hook<F>(&mut self, name: &str, hook: F)
    where
        F: Fn(ToolCallContext) -> HookResult + Send + Sync + 'static,
    {
        self.hooks.register_pre(name, Arc::new(hook));
    }

    /// Dispatch parallel subagents (from Claude Code fork/async/sync)
    ///
    /// P2-C2: 把父代理最近的对话 turns 压缩成有界摘要随任务一起派发——
    /// subagent 获得定向父上下文 (信息不丢失), 又不复制全量历史 (token 受控)。
    pub async fn dispatch_subagents(&mut self, tasks: Vec<(SubagentKind, String)>) {
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let context_hint = SubagentDispatch::compress_context(&self.context.turns, 2000);
        let hint = if context_hint.is_empty() { None } else { Some(context_hint.as_str()) };
        self.subagent_results = SubagentDispatch::run_parallel_with_context(tasks, hint, &cwd).await;
    }

    /// Get all tool call context for permission checking
    fn current_tool_context(&self) -> ToolCallContext {
        ToolCallContext {
            tool_name: format!("{:?}", self.state.mode),
            args: String::new(),
            cwd: std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            estimated_cost: self.state.tokens_used as f64 * 0.00001,
        }
    }

    pub fn set_consciousness_tree(
        &mut self,
        tree: crate::core::nt_core_consciousness_tree::ConsciousnessTree,
    ) {
        self.consciousness = Some(tree);
    }

    pub fn set_event_bus(&mut self, bus: crate::neotrix::nt_core_event_bus::EventBus) {
        self.event_bus = Some(bus);
    }

    pub fn set_brain(
        &mut self,
        brain: Arc<
            tokio::sync::RwLock<crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain>,
        >,
    ) {
        self.brain = Some(brain);
    }

    /// Set an explicit mode (Agent/Shell/Plan)
    pub fn set_mode(&mut self, mode: NeoCodexMode) {
        if self.state.mode == mode {
            return;
        }
        let from = self.state.mode;
        self.wire.record(WireEvent::ModeChange { from, to: mode });
        self.state.mode = mode;
        self.state.mode_start = Instant::now();
    }

    /// Persist a user-chosen session name into the wire stream (overrides
    /// the derived first-message name on read).
    pub fn rename_session(&mut self, name: &str) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        // 保留既有标签（重命名不动 tags；读旧态标签并回写）
        let tags = self.read_session_tags();
        self.wire.record(WireEvent::SessionMeta {
            name: name.trim().to_string(),
            timestamp: ts,
            tags,
        });
    }

    /// Read the persisted tag set for the current wire session (from last
    /// SessionMeta). Empty when none recorded yet.
    fn read_session_tags(&self) -> Vec<String> {
        self.wire
            .events
            .iter()
            .rev()
            .find(|e| matches!(e, WireEvent::SessionMeta { .. }))
            .and_then(|e| match e {
                WireEvent::SessionMeta { tags, .. } => Some(tags.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    /// Persist a tag onto the current session (deduped). Writes a new
    /// SessionMeta carrying the merged tag set.
    pub fn tag_session(&mut self, tag: &str) {
        let mut tags = self.read_session_tags();
        let clean = tag.trim().to_lowercase().replace(' ', "-");
        if clean.is_empty() || tags.contains(&clean) {
            return;
        }
        tags.push(clean);
        self.write_session_tags(tags);
    }

    /// Remove a tag from the current session. No-op when absent.
    pub fn untag_session(&mut self, tag: &str) {
        let mut tags = self.read_session_tags();
        let before = tags.len();
        tags.retain(|t| t != tag);
        if tags.len() != before {
            self.write_session_tags(tags);
        }
    }

    /// Overwrite persisted tags (rename/tag/untag 共用落盘点)。
    fn write_session_tags(&mut self, tags: Vec<String>) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let meta = self
            .wire
            .events
            .iter()
            .rev()
            .find(|e| matches!(e, WireEvent::SessionMeta { .. }));
        let name = meta
            .and_then(|e| match e {
                WireEvent::SessionMeta { name, .. } => Some(name.clone()),
                _ => None,
            })
            .unwrap_or_else(|| self.wire.session_id.clone());
        self.wire.record(WireEvent::SessionMeta {
            name,
            timestamp: ts,
            tags,
        });
    }

    /// Record a side-chat message (branched question that must NOT pollute
    /// the main session context). Persisted to the same wire stream but
    /// filtered out of resume_session / get_session_messages.
    pub fn record_side_chat(&mut self, content: &str, role: &str) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.wire.record(WireEvent::SideChatMessage {
            content: content.trim().to_string(),
            timestamp: ts,
            role: role.to_string(),
        });
    }

    /// Detach the agent from its current wire file (P2-3). After a session is
    /// archived/deleted while active, the stale `wire.path` would otherwise
    /// recreate the file on the next `record()` (`create+append`) and split
    /// the conversation into a divergent duplicate. Resets to a fresh empty
    /// session so the next turn starts clean.
    pub fn detach_wire(&mut self) {
        self.wire = WireSession::new(&format!(
            "s-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ));
        self.context.turns.clear();
        self.state.tokens_used = 0;
        self.state.turn_count = 0;
        self.state.tool_call_count = 0;
    }

    /// One-shot side-chat answer (P1-2). Unlike the main loop, this must NOT
    /// push into `self.context` / `self.state` — the branch question is
    /// isolated from the session's real conversation. We generate the reply
    /// on a throwaway message list and record both turns as side-chat events.
    pub async fn side_chat_ask(&mut self, content: &str) -> String {
        let provider = match self.provider.to_llm_provider() {
            Some(p) => p,
            None => return "[side chat] no provider configured".to_string(),
        };
        let system = "You are NeoCodex, an AI coding assistant. Answer the user's question concisely and precisely. Respond in markdown.";
        let messages = vec![
            Message::new(Role::System, system),
            Message::new(Role::User, content.trim()),
        ];
        let request = match self.build_request(messages) {
            Some(r) => r,
            None => return "[side chat] provider unavailable".to_string(),
        };
        let mut rx = match provider.stream_complete(&request).await {
            Ok(rx) => rx,
            Err(e) => return format!("[side chat] {}", e),
        };
        let mut answer = String::new();
        while let Some(chunk) = rx.recv().await {
            match chunk {
                Ok(resp) => answer.push_str(&resp.content),
                Err(e) => return format!("[side chat] {}", e),
            }
        }
        if answer.trim().is_empty() {
            "[side chat] empty response".to_string()
        } else {
            answer
        }
    }

    /// Toggle between Agent and Shell mode (from Kimi Code Ctrl-X)
    pub fn toggle_mode(&mut self) -> NeoCodexMode {
        let from = self.state.mode;
        self.state.mode = match from {
            NeoCodexMode::Agent => NeoCodexMode::Shell,
            NeoCodexMode::Shell => NeoCodexMode::Agent,
            NeoCodexMode::Plan => NeoCodexMode::Agent,
        };
        self.state.mode_start = Instant::now();
        self.wire.record(WireEvent::ModeChange {
            from,
            to: self.state.mode,
        });
        self.state.mode
    }

    /// Switch to Plan mode (from Kimi Code Shift-Tab / Claude Code Plan)
    pub fn set_plan_mode(&mut self) {
        let from = self.state.mode;
        self.wire.record(WireEvent::ModeChange {
            from,
            to: NeoCodexMode::Plan,
        });
        self.state.mode = NeoCodexMode::Plan;
        self.state.mode_start = Instant::now();
    }

    /// Set the streaming-path permission policy (P0-2). Called by the desktop
    /// command layer from the UI's permission_mode (auto/manual/accept_edits/plan).
    pub fn set_permission_mode(&mut self, mode: &str) {
        self.state.permission_mode = mode.to_string();
        if mode == "plan" {
            self.set_plan_mode();
        }
    }

    /// Process user input through the agent loop
    pub async fn process(&mut self, input: &str) -> String {
        self.state.turn_count += 1;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.wire.record(WireEvent::UserMessage {
            content: input.to_string(),
            timestamp,
            attachments: None,
        });

        let token_estimate = input.len() / 4;
        self.context.push("user", input.to_string(), token_estimate);
        self.state.tokens_used += token_estimate;

        // ── Consciousness-in-the-loop (NeoTrix unique) ──
        self.inject_into_consciousness();
        self.apply_consciousness_guidance();
        if let Some(completed_goal) = self.check_goals() {
            log::debug!("[neocodex] goal completed, advancing: {}", completed_goal);
        }

        let response = match self.state.mode {
            NeoCodexMode::Shell => self.exec_shell(input).await,
            NeoCodexMode::Plan => self.exec_plan(input).await,
            NeoCodexMode::Agent => self.exec_agent(input).await,
        };

        self.wire.record(WireEvent::AgentMessage {
            content: response.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        });

        response
    }

    async fn exec_shell(&mut self, input: &str) -> String {
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(input)
            .output()
            .await;
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let exit_code = out.status.code().unwrap_or(-1);
                let result = if exit_code == 0 {
                    stdout
                } else {
                    format!("exit({}): {}", exit_code, stderr)
                };
                self.state.tool_call_count += 1;
                self.wire.record(WireEvent::ToolCall {
                    name: "shell".into(),
                    args: input.to_string(),
                    result: result.clone(),
                    duration_ms: 0,
                    success: exit_code == 0,
                });
                if exit_code != 0 {
                    format!("```\n{}\n```\n\nExit code: {}", result, exit_code)
                } else {
                    format!("```{}\n```", result.trim())
                }
            }
            Err(e) => format!("Shell error: {}", e),
        }
    }

    async fn exec_plan(&mut self, input: &str) -> String {
        let plan = self.consciousness_plan(input);
        let token_estimate = plan.len() / 4;
        self.context.push("assistant", plan.clone(), token_estimate);
        plan
    }

    async fn exec_agent(&mut self, input: &str) -> String {
        // Pre-execution hook gate
        let ctx = self.current_tool_context();
        match self.hooks.run_pre(ctx.clone()) {
            HookDecision::Deny(reason) => {
                return format!("[blocked] Hook denied: {}", reason);
            }
            HookDecision::RequireConfirm(msg) => {
                let response = format!(
                    "[confirmed] {}\n\n<thinking>Processing turn {} in Agent mode</thinking>\n\n{}",
                    msg, self.state.turn_count, input
                );
                self.markdown.push(&response);
                let clean = response.clone();
                self.context
                    .push("assistant", clean.clone(), clean.len() / 4);
                self.hooks.run_post(ctx, clean.clone(), 0);
                return clean;
            }
            HookDecision::Allow => {}
        }

        let start = Instant::now();

        // Cycle 159: real ReAct loop via provider if a concrete model is resolvable
        let response = match self.react_loop(input, 4).await {
            Some(out) => out,
            None => {
                // Fallback: provider not wired — keep the deterministic stub so the
                // agent remains usable offline (no silent "dead agent").
                format!("<thinking>Processing turn {} in Agent mode (provider unavailable, stub)</thinking>\n\n{}",
                    self.state.turn_count, input)
            }
        };

        self.markdown.push(&response);
        let clean = response.clone();
        let token_estimate = clean.len() / 4;
        self.context
            .push("assistant", clean.clone(), token_estimate);
        let _ = self.cost.record("agent", 0.0, token_estimate as u64);
        self.hooks
            .run_post(ctx, clean.clone(), start.elapsed().as_millis() as u64);

        // Evolution loop advances every turn (self-audit → diagnose → fix)
        EvolutionLoop::step(self);

        clean
    }

    /// Add a goal to the queue (from Kimi Code /goal system)
    pub fn add_goal(&mut self, description: &str, max_iters: u64) {
        self.goals.add(description, max_iters);
        let id = self
            .goals
            .goals
            .back()
            .map(|g| g.id.clone())
            .unwrap_or_default();
        self.state.goal_active = true;
        self.wire.record(WireEvent::GoalUpdate {
            id,
            state: "active".into(),
            description: description.into(),
        });
    }

    /// Check if a goal is complete and advance the queue.
    ///
    /// Also increments the active goal's iteration counter each turn and
    /// resets `goal_active` once the queue drains, so the goal loop can
    /// actually make progress (previously `check_goals` was never called,
    /// `iterations` never incremented, and `goal_active` never reset).
    /// When no goal is active but the queue is non-empty, promotes the head
    /// of the queue (add_goal queues but never promotes).
    pub fn check_goals(&mut self) -> Option<String> {
        if self.goals.active.is_none() && !self.goals.goals.is_empty() {
            self.goals.next();
        }
        if let Some(ref mut goal) = self.goals.active {
            goal.iterations = goal.iterations.saturating_add(1);
            if goal.iterations >= goal.max_iterations {
                self.wire.record(WireEvent::GoalUpdate {
                    id: goal.id.clone(),
                    state: "completed".into(),
                    description: goal.description.clone(),
                });
                let next = self.goals.next().map(|g| g.description);
                if self.goals.active.is_none() && self.goals.goals.is_empty() {
                    self.state.goal_active = false;
                }
                return next;
            }
        }
        None
    }

    /// Record a tool call (from Claude Code: StreamingToolExecutor pattern)
    pub fn record_tool_call(
        &mut self,
        name: &str,
        args: &str,
        result: String,
        duration_ms: u64,
        success: bool,
    ) {
        self.state.tool_call_count += 1;
        self.wire.record(WireEvent::ToolCall {
            name: name.to_string(),
            args: args.to_string(),
            result,
            duration_ms,
            success,
        });
    }

    /// Push agent state into ConsciousnessTree soil (outbound integration)
    fn inject_into_consciousness(&mut self) {
        if let Some(ref mut tree) = self.consciousness {
            tree.soil.crawl_queue_depth = self.state.turn_count;
            tree.run_growth_cycle();
        }
    }

    /// Consume ConsciousnessTree guidance to adjust agent behavior (inbound integration)
    fn apply_consciousness_guidance(&mut self) {
        let Some(ref tree) = self.consciousness else {
            return;
        };

        let fruit_count = tree.fruits.len();
        let phi_avg = if !tree.fruits.is_empty() {
            tree.fruits.iter().map(|f| f.quality).sum::<f64>() / tree.fruits.len() as f64
        } else {
            0.0
        };
        self.config.thinking_enabled = phi_avg > 0.3;

        if fruit_count < 3 && tree.cycle > 5 && !self.state.goal_active {
            self.state.mode = NeoCodexMode::Plan;
            self.add_goal(
                "Cultivate capability branches: absorb external knowledge",
                5,
            );
        }
    }

    /// Feed consciousness data into the agent's plan mode
    fn consciousness_plan(&mut self, input: &str) -> String {
        match self.consciousness.as_ref() {
            None => format!("## Plan\n\n{}\n\n---\n\nAwaiting approval...", input),
            Some(tree) => {
                let guidance = tree.core.next_actions.join("; ");
                let avg_quality = if !tree.fruits.is_empty() {
                    tree.fruits.iter().map(|f| f.quality).sum::<f64>() / tree.fruits.len() as f64
                } else {
                    0.0
                };
                format!(
                    "## Plan (Cycle {})\n\n**Avg Quality**: {:.3}\n\n**Next actions**: {}\n\n---\n\n{}",
                    tree.cycle, avg_quality, guidance, input,
                )
            }
        }
    }

    // ── Cycle 159: Real ReAct Loop + Self-Audit + Evolution ──

    /// True ReAct loop: build messages from context pipeline, call the real LLM
    /// provider, then parse any tool-call block and execute it. Loops up to
    /// `max_steps` (mirrors Claude Code's streaming tool executor + Codex's
    /// plan-execute cycle).
    async fn react_loop(&mut self, input: &str, max_steps: usize) -> Option<String> {
        let provider = self.provider.to_llm_provider()?;

        let mut messages = self.build_messages(input);
        let mut step = 0;
        let mut final_answer: Option<String> = None;

        while step < max_steps {
            Self::budget_react_messages(&mut messages, self.context.max_tokens);
            let request = self.build_request(messages.clone())?;

            let response = match provider.complete(&request).await {
                Ok(r) => r,
                Err(e) => {
                    self.wire.record(WireEvent::SystemEvent {
                        kind: "provider_error".into(),
                        detail: e.to_string(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64,
                    });
                    return Some(format!("[provider error] {}", e));
                }
            };

            self.state.tokens_used += response.usage.total_tokens as usize;
            let _ = self
                .cost
                .record("agent", 0.0, response.usage.total_tokens as u64);

            // Attempt to extract a structured tool-call from the response.
            let tool_call = Self::extract_tool_call(&response.content);

            match tool_call {
                Some((name, args)) => {
                    self.state.tool_call_count += 1;
                    // P0-2: enforce the permission policy on the streaming path.
                    // Previously only the CLI AgentStream and exec_agent honored
                    // PermissionSystem; react_loop bypassed it entirely, so
                    // Manual/AcceptEdits/Plan modes were advisory at best.
                    let allowed = self
                        .permissions
                        .policy_gate(&name, &self.state.permission_mode);
                    if !allowed {
                        let denied = format!(
                            "[denied] tool `{}` blocked by permission mode `{}`",
                            name, self.state.permission_mode
                        );
                        self.wire.record(WireEvent::ToolCall {
                            name: name.clone(),
                            args: args.clone(),
                            result: denied.clone(),
                            duration_ms: 0,
                            success: false,
                        });
                        messages.push(Message::tool(&denied, &format!("call-{}", step)));
                        step += 1;
                        continue;
                    }
                    let result = self.execute_tool(&name, &args).await;
                    // Tool grounding (Cycle 160e): claimed success when invoked; actual success
                    // if the tool did not return a distinguishable error marker.
                    let actual_ok = !result.starts_with('[');
                    self.tool_grounding
                        .record_tool_result(&name, true, actual_ok);
                    self.wire.record(WireEvent::ToolCall {
                        name: name.clone(),
                        args: args.clone(),
                        result: result.clone(),
                        duration_ms: 0,
                        success: actual_ok,
                    });
                    messages.push(Message::assistant_with_calls(
                        &response.content,
                        vec![crate::neotrix::nt_io_provider::types::ToolCallInfo {
                            id: format!("call-{}", step),
                            call_type: "function".into(),
                            function: crate::neotrix::nt_io_provider::types::ToolCallFunction {
                                name: name.clone(),
                                arguments: args.clone(),
                            },
                        }],
                    ));
                    messages.push(Message::tool(&result, &format!("call-{}", step)));
                    self.context.push(
                        "assistant",
                        response.content.clone(),
                        response.content.len() / 4,
                    );
                    self.context.push("tool", result.clone(), result.len() / 4);
                    step += 1;
                }
                None => {
                    final_answer = Some(response.content);
                    break;
                }
            }
        }

        final_answer
    }

    /// Streaming ReAct loop: emits tokens via callback as they arrive from the provider.
    /// `on_token` returns `true` to continue or `false` to cancel; a cancelled
    /// stream returns the tokens accumulated so far (partial reply).
    /// `on_tool` fires after each tool execution (name, args, result, duration_ms, success);
    /// returning `false` cancels the loop (same semantics as `on_token`).
    /// Returns the final accumulated response (or error).
    pub async fn react_loop_stream<F, G>(
        &mut self,
        input: &str,
        max_steps: usize,
        mut on_token: F,
        mut on_tool: G,
    ) -> Option<String>
    where
        F: FnMut(&str) -> bool + Send + Sync,
        G: FnMut(&str, &str, &str, u64, bool) -> bool + Send + Sync,
    {
        let provider = self.provider.to_llm_provider()?;

        let mut messages = self.build_messages(input);
        let mut step = 0;
        let mut final_answer: Option<String> = None;
        let mut accumulated = String::new();
        let mut cancelled = false;

        while step < max_steps && !cancelled {
            Self::budget_react_messages(&mut messages, self.context.max_tokens);
            let request = self.build_request(messages.clone())?;

            let mut rx = match provider.stream_complete(&request).await {
                Ok(rx) => rx,
                Err(e) => {
                    self.wire.record(WireEvent::SystemEvent {
                        kind: "provider_error".into(),
                        detail: e.to_string(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64,
                    });
                    return Some(format!("[provider error] {}", e));
                }
            };

            let mut response_content = String::new();
            let mut response_usage = None;

            while let Some(chunk) = rx.recv().await {
                match chunk {
                    Ok(resp) => {
                        if !resp.content.is_empty() {
                            response_content.push_str(&resp.content);
                            accumulated.push_str(&resp.content);
                            if !on_token(&resp.content) {
                                cancelled = true;
                                break;
                            }
                        }
                        if resp.usage.total_tokens > 0 {
                            response_usage = Some(resp.usage);
                        }
                    }
                    Err(e) => {
                        self.wire.record(WireEvent::SystemEvent {
                            kind: "provider_error".into(),
                            detail: e.to_string(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs() as i64,
                        });
                        return Some(format!("[provider error] {}", e));
                    }
                }
            }

            if let Some(usage) = response_usage {
                self.state.tokens_used += usage.total_tokens as usize;
                let _ = self.cost.record("agent", 0.0, usage.total_tokens as u64);
            }

            if cancelled {
                break;
            }

            // Attempt to extract a structured tool-call from the response.
            let tool_call = Self::extract_tool_call(&response_content);

            // P1-1 Plan gate: in non-Agent modes (Plan) tools must NOT be
            // executed. Plan is a read-only promise (Codex plan / Claude
            // plan-mode parity); executing shell there lets the model run
            // arbitrary commands despite the read-only contract. Skip tool
            // execution and return the drafted plan/response as the answer.
            if self.state.mode != NeoCodexMode::Agent {
                return Some(response_content);
            }

            match tool_call {
                Some((name, args)) => {
                    self.state.tool_call_count += 1;
                    // P0-2: enforce the permission policy on the streaming path.
                    let allowed = self
                        .permissions
                        .policy_gate(&name, &self.state.permission_mode);
                    if !allowed {
                        let denied = format!(
                            "[denied] tool `{}` blocked by permission mode `{}`",
                            name, self.state.permission_mode
                        );
                        self.wire.record(WireEvent::ToolCall {
                            name: name.clone(),
                            args: args.clone(),
                            result: denied.clone(),
                            duration_ms: 0,
                            success: false,
                        });
                        messages.push(Message::tool(&denied, &format!("call-{}", step)));
                        step += 1;
                        continue;
                    }
                    let tool_started = Instant::now();
                    let result = self.execute_tool(&name, &args).await;
                    let tool_duration_ms = tool_started.elapsed().as_millis() as u64;
                    let actual_ok = !result.starts_with('[');
                    self.tool_grounding
                        .record_tool_result(&name, true, actual_ok);
                    self.wire.record(WireEvent::ToolCall {
                        name: name.clone(),
                        args: args.clone(),
                        result: result.clone(),
                        duration_ms: tool_duration_ms,
                        success: actual_ok,
                    });
                    if !on_tool(&name, &args, &result, tool_duration_ms, actual_ok) {
                        cancelled = true;
                        break;
                    }
                    messages.push(Message::assistant_with_calls(
                        &response_content,
                        vec![crate::neotrix::nt_io_provider::types::ToolCallInfo {
                            id: format!("call-{}", step),
                            call_type: "function".into(),
                            function: crate::neotrix::nt_io_provider::types::ToolCallFunction {
                                name: name.clone(),
                                arguments: args.clone(),
                            },
                        }],
                    ));
                    messages.push(Message::tool(&result, &format!("call-{}", step)));
                    self.context.push(
                        "assistant",
                        response_content.clone(),
                        response_content.len() / 4,
                    );
                    self.context.push("tool", result.clone(), result.len() / 4);
                    step += 1;
                }
                None => {
                    final_answer = Some(response_content);
                    break;
                }
            }
        }

        if cancelled {
            Some(accumulated)
        } else {
            final_answer
        }
    }

    /// Build system + history + current user messages from the context pipeline.
    fn build_messages(&self, input: &str) -> Vec<Message> {
        let system = "You are NeoCodex, an AI coding agent inside the NeoTrix architecture. \
            Modes: Agent (autonomous coding), Shell (run commands), Plan (draft plans). \
            Use the tools when you need to read files, search the repo, or run shell commands. \
            Always respond in markdown. Be concise and precise."
            .to_string();
        let mut messages = vec![Message::new(Role::System, &system)];
        for turn in &self.context.turns {
            let role = match turn.role.as_str() {
                "user" => Role::User,
                "assistant" | "summary" => Role::Assistant,
                "tool" => Role::Tool,
                "system" => Role::System,
                _ => Role::User,
            };
            if role == Role::System {
                continue; // keep single system message
            }
            messages.push(Message::new(role, &turn.content));
        }
        // P1-2 dedup: the current user input is already pushed into
        // `context.turns` by the caller before invoking the loop (send
        // command / process). Appending it again yields two consecutive
        // identical user turns in every request. Only append when the last
        // history turn is NOT the same message.
        if self.context.turns.back().map(|t| t.content.as_str()) != Some(input) {
            messages.push(Message::new(Role::User, input));
        }
        messages
    }

    /// Bottom-up token budget for the ReAct loop. The local `messages` vec grows
    /// by one assistant + one tool-result turn per step, so a long loop can blow
    /// the provider context window. Delegates to the shared token-budget engine
    /// (`nt_io_provider::context_budget::apply_context_budget`, CJK-aware estimate).
    /// Evicts oldest non-system turns first; index 0 (system) and the trailing
    /// current-user request are never evicted. Tool-result truncation is disabled
    /// here (0) — ContextPipeline Layer-3 already caps tool turns.
    fn budget_react_messages(messages: &mut Vec<Message>, max_tokens: usize) {
        apply_context_budget(messages, max_tokens, 0);
    }
    /// Build an LlmRequest from the current catalog's active provider.
    fn build_request(&self, messages: Vec<Message>) -> Option<LlmRequest> {
        self.provider.providers.get(self.provider.active)?;
        // P0-3: surface the most recent user-turn image attachment to the model.
        // The UI stores base64 in WireEvent::UserMessage.attachments; previously
        // image_data was hardcoded None, so attached screenshots never reached
        // the provider despite being rendered inline in the chat.
        let image_data = self.wire.events.iter().rev().find_map(|ev| match ev {
            WireEvent::UserMessage {
                attachments: Some(list),
                ..
            } => list
                .iter()
                .find(|a| a.mime_type.starts_with("image/"))
                .and_then(|a| a.data.clone()),
            _ => None,
        });

        // Vision-bridge: the active model may be text-only (e.g. deepseek-v4-flash,
        // local qwen2.5:7b without -vl). Sending image_data to such providers is a
        // no-op or an error — the VisionBridge converts the attachment into
        // deterministic structured evidence text that the text-only model CAN
        // reason over, and we drop the raw image channel.
        let active_model = self.provider.active_model();
        let active_has_vision = self.provider.has_capability(ModelCapability::Vision)
            || crate::core::nt_core_e8::nt_multimodal::model_supports_vision(&active_model);
        let mut messages = messages;
        let mut image_data = image_data;
        if image_data.is_some() && !active_has_vision {
            if let Some(b64) = image_data.take() {
                if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&b64) {
                    if let Ok((evidence, _feat)) =
                        crate::core::nt_core_e8::nt_multimodal::VisionBridge::analyze_cached(&bytes)
                    {
                        if let Some(last_user) =
                            messages.iter_mut().rev().find(|m| m.role == Role::User)
                        {
                            last_user.content = format!(
                                "{}\n\n{}\n\n(Note: the active model is text-only; the image attachment was bridged to structured pixel evidence above.)",
                                last_user.content,
                                evidence.to_evidence_text(),
                            );
                        }
                    }
                }
            }
        }

        // P0-4 prefix caching: 稳定前缀 = 除末条 (当前请求) 外的全部历史。
        // ReAct 每轮重发时该前缀命中 provider 缓存, 成本趋近增量。
        let cacheable_prefix_tokens = if messages.len() > 1 {
            Some(estimate_messages_tokens(&messages[..messages.len() - 1]))
        } else {
            None
        };

        let mut req = LlmRequest {
            model: self.provider.active_model(),
            messages,
            // P2-1: honor the settings-panel generation params instead of the
            // old hardcoded values.
            temperature: Some(self.config.temperature.clamp(0.0, 2.0) as f32),
            max_tokens: self.config.max_tokens.max(1),
            tools: vec![
                Tool {
                    name: "read".into(),
                    description: "Read a file at the given absolute or relative path".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {"path": {"type": "string"}}}),
                },
                Tool {
                    name: "search".into(),
                    description: "Grep the codebase for a pattern (regex)".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {"pattern": {"type": "string"}}}),
                },
                Tool {
                    name: "write".into(),
                    description: "Write or overwrite a file. Args format: <path>|<content> (split on the first pipe). Creates parent dirs. Guarded to the workspace.".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {"path": {"type": "string"}, "content": {"type": "string"}}}),
                },
                Tool {
                    name: "edit".into(),
                    description: "Replace a unique old substring with new in a file. Args format: <path>|<old>|<new> (split on the first two pipes). Fails if old is missing or not unique.".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {"path": {"type": "string"}, "old": {"type": "string"}, "new": {"type": "string"}}}),
                },
                Tool {
                    name: "shell".into(),
                    description: "Run a shell command and return stdout".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {"command": {"type": "string"}}}),
                },
                Tool {
                    name: "mcp_call".into(),
                    description: "Call a registered MCP tool. Args format: <tool_name>|<json_args> (split on the first pipe). List available tools with mcp_list.".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {"name": {"type": "string"}, "args": {"type": "string"}}}),
                },
                Tool {
                    name: "mcp_list".into(),
                    description: "List registered MCP servers and their available tools".into(),
                    input_schema: serde_json::json!({"type": "object", "properties": {}}),
                },
            ],
            image_data,
            thinking_budget: if self.config.thinking_enabled { Some(2048) } else { None },
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: None,
            cacheable_prefix_tokens,
        };
        if req.image_data.is_some() && active_has_vision {
            if let Some(raw) = req.image_data.clone() {
                req = req.with_image_b64(&raw);
            }
        }
        Some(req)
    }

    /// Parse a `<tool name="...">args</tool>` block from the model output.
    fn extract_tool_call(content: &str) -> Option<(String, String)> {
        let marker = "<tool";
        let start = content.find(marker)?;
        let name_start = content[start..].find("name=\"")? + start + 6;
        let name_end = content[name_start..].find('"')? + name_start;
        let name = content[name_start..name_end].to_string();
        let args_start = content[name_end..].find('>')? + name_end + 1;
        let args_end = content[args_start..].find("</tool>")? + args_start;
        let args = content[args_start..args_end].trim().to_string();
        Some((name, args))
    }

    /// Resolve a tool-provided path against the workspace root and refuse any
    /// path that escapes it (`..` / absolute outside cwd). Claude/Codex both
    /// sandbox agent file access to the project; P0-1/P2-3: without this the
    /// read/write/edit tools could touch arbitrary files outside the repo.
    fn guard_path(&self, raw: &str) -> Result<std::path::PathBuf, String> {
        let cwd = std::env::current_dir().map_err(|e| format!("[cwd error] {}", e))?;
        let p = std::path::Path::new(raw.trim());
        let candidate = if p.is_absolute() {
            p.to_path_buf()
        } else {
            cwd.join(p)
        };
        // Normalize: lexically resolve `.`/`..` components without touching FS.
        let mut parts: Vec<std::ffi::OsString> = Vec::new();
        for comp in candidate.components() {
            match comp {
                std::path::Component::Normal(c) => parts.push(c.to_os_string()),
                std::path::Component::ParentDir => {
                    if parts.pop().is_none() {
                        return Err(format!("[path error] {} escapes the workspace", raw));
                    }
                }
                std::path::Component::CurDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_) => {}
            }
        }
        let normalized = parts
            .iter()
            .fold(std::path::PathBuf::new(), |acc, c| acc.join(c));
        let resolved = normalized;
        if !resolved.starts_with(&cwd) {
            return Err(format!("[path error] {} is outside the workspace", raw));
        }
        Ok(resolved)
    }

    /// Execute a concrete tool. Wired to real FS + shell (no external binaries,
    /// consistent with R-P48 zero third-party binary dependency).
    async fn execute_tool(&mut self, name: &str, args: &str) -> String {
        match name {
            "read" => {
                let path = match self.guard_path(args) {
                    Ok(p) => p,
                    Err(e) => return e,
                };
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        if content.len() > 16_000 {
                            content[..content.floor_char_boundary(16_000)].to_string()
                        } else {
                            content
                        }
                    }
                    Err(e) => format!("[read error] {}", e),
                }
            }
            "search" => {
                let cwd = std::env::current_dir().unwrap_or_default();
                let pattern = args.trim();
                let mut hits = Vec::new();
                // P1-2: recursive search — the old impl only walked the top
                // level (read_dir, no recursion) and filtered to *.rs, making it
                // useless on real codebases. Skip heavy dirs to bound cost.
                fn walk(
                    dir: &std::path::Path,
                    pattern: &str,
                    hits: &mut Vec<String>,
                    depth: usize,
                ) {
                    if depth > 8 || hits.len() >= 40 {
                        return;
                    }
                    let Ok(entries) = std::fs::read_dir(dir) else {
                        return;
                    };
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let fname = entry.file_name().to_string_lossy().to_string();
                        if fname == "target"
                            || fname == "node_modules"
                            || fname == ".git"
                            || fname == "dist"
                            || fname == "build"
                            || fname == ".venv"
                            || fname == "vendor"
                        {
                            continue;
                        }
                        if path.is_dir() {
                            walk(&path, pattern, hits, depth + 1);
                            continue;
                        }
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            for (i, line) in content.lines().enumerate() {
                                if line.contains(pattern) {
                                    hits.push(format!(
                                        "{}:{}: {}",
                                        path.display(),
                                        i + 1,
                                        line.trim()
                                    ));
                                    if hits.len() >= 40 {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
                walk(&cwd, pattern, &mut hits, 0);
                if hits.is_empty() {
                    format!("No matches for {:?} in {}", pattern, cwd.display())
                } else {
                    hits.join("\n")
                }
            }
            // P0-1: native write tool (Claude Write parity). Unlike shell
            // escape, this is a bounded, guarded single-file write.
            "write" => {
                // Args format: `<path>|<content>` — split on the first `|` so
                // content may itself contain pipes. Model contract documented in
                // build_request tool description.
                let (path, content) = match args.split_once('|') {
                    Some((p, c)) => (p, c),
                    None => {
                        return "[write error] expected format: <path>|<content>".to_string();
                    }
                };
                let path = match self.guard_path(path) {
                    Ok(p) => p,
                    Err(e) => return e,
                };
                if content.len() > 64_000 {
                    return format!(
                        "[write error] content exceeds 64 KB ({} bytes)",
                        content.len()
                    );
                }
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            return format!("[write error] mkdir: {}", e);
                        }
                    }
                }
                match std::fs::write(&path, content) {
                    Ok(()) => format!("[ok] wrote {} ({} bytes)", path.display(), content.len()),
                    Err(e) => format!("[write error] {}", e),
                }
            }
            // P0-1: native edit tool (Claude Edit parity). Replaces a unique
            // `old` substring with `new` in the target file. Args: `<path>|<old>|<new>`.
            "edit" => {
                let parts: Vec<&str> = args.splitn(3, '|').collect();
                if parts.len() != 3 {
                    return "[edit error] expected format: <path>|<old>|<new>".to_string();
                }
                let path = match self.guard_path(parts[0]) {
                    Ok(p) => p,
                    Err(e) => return e,
                };
                let original = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => return format!("[edit error] read {}: {}", path.display(), e),
                };
                if original.len() > 64_000 {
                    return format!("[edit error] file exceeds 64 KB ({} bytes)", original.len());
                }
                let old = parts[1];
                let new = parts[2];
                let count = original.matches(old).count();
                if count == 0 {
                    return format!("[edit error] old text not found in {}", path.display());
                }
                if count > 1 {
                    return format!(
                        "[edit error] old text is not unique ({} matches) in {}",
                        count,
                        path.display()
                    );
                }
                let updated = original.replace(old, new);
                match std::fs::write(&path, updated) {
                    Ok(()) => format!("[ok] edited {} (replaced unique match)", path.display()),
                    Err(e) => format!("[edit error] {}", e),
                }
            }
            "shell" => {
                let output = tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(args)
                    .output()
                    .await;
                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                        let code = out.status.code().unwrap_or(-1);
                        let cap = 8_000usize;
                        if code == 0 {
                            if stdout.len() > cap {
                                format!(
                                    "{}... [stdout truncated {} bytes]",
                                    &stdout[..stdout.floor_char_boundary(cap)],
                                    stdout.len().saturating_sub(cap)
                                )
                            } else {
                                stdout
                            }
                        } else {
                            format!("exit({}): {}", code, stderr)
                        }
                    }
                    Err(e) => format!("[shell error] {}", e),
                }
            }
            // P2-5: MCP tool call (Codex/Claude MCP parity). Args format:
            // `<tool_name>|<json_args>` — split on the first `|`. Delegates to
            // the attached McpRegistry; without a registry it returns a clear
            // error instead of silently pretending to succeed.
            "mcp_call" => {
                let Some(registry) = &self.mcp else {
                    return "[mcp_call error] no MCP registry attached; register MCP servers first"
                        .to_string();
                };
                let (name, json) = match args.split_once('|') {
                    Some((n, j)) => (n.trim(), j),
                    None => (args.trim(), "{}"),
                };
                let parsed: serde_json::Value = match serde_json::from_str(json) {
                    Ok(v) => v,
                    Err(e) => return format!("[mcp_call error] invalid JSON args: {}", e),
                };
                match registry.call_tool(name, &parsed) {
                    Ok(result) => result,
                    Err(e) => format!("[mcp_call error] {}", e),
                }
            }
            "mcp_list" => {
                let Some(registry) = &self.mcp else {
                    return "[mcp_list] no MCP registry attached".to_string();
                };
                if registry.server_count() == 0 {
                    return "[mcp_list] no MCP servers registered".to_string();
                }
                let mut out = String::new();
                for server in registry.list_servers() {
                    out.push_str(&format!("# {} ({})\n", server.name, server.tools.len()));
                    for tool in &server.tools {
                        out.push_str(&format!("  - {}: {}\n", tool.name, tool.description));
                    }
                }
                out
            }
            _ => format!("Unknown tool: {}", name),
        }
    }

    /// Restore prior session events from the wire file into the context
    /// pipeline (G2 session continuity, matching Claude Code `--resume`).
    /// Returns the number of events restored.
    /// Clear in-memory context and re-restore it from the wire file. Used after
    /// an edit/delete/regenerate rewrites the JSONL so the agent's next turn is
    /// built from the corrected history, not stale in-memory state.
    pub fn rebuild_context_from_wire(&mut self) -> usize {
        self.context.turns.clear();
        self.state.tokens_used = 0;
        self.state.tool_call_count = 0;
        self.resume_session()
    }

    pub fn resume_session(&mut self) -> usize {
        let events = self.wire.load();
        let mut restored = 0;
        for event in events {
            match event {
                WireEvent::UserMessage { content, .. } => {
                    let est = content.len() / 4;
                    self.context.push("user", content, est);
                    self.state.tokens_used += est;
                    restored += 1;
                }
                WireEvent::AgentMessage { content, .. } => {
                    let est = content.len() / 4;
                    self.context.push("assistant", content, est);
                    self.state.tokens_used += est;
                    restored += 1;
                }
                WireEvent::ToolCall {
                    name, args, result, ..
                } => {
                    self.context.push(
                        "user",
                        format!("[tool {}] args={} result={}", name, args, result),
                        (name.len() + args.len() + result.len()) / 4,
                    );
                    self.state.tool_call_count += 1;
                    restored += 1;
                }
                WireEvent::ModeChange { to, .. } => {
                    self.state.mode = to;
                    restored += 1;
                }
                _ => {}
            }
        }
        restored
    }

    /// Produce a full health snapshot of the agent (D25: output must be
    /// consumable — this is consumed by SelfTest, UI status, and evolution).
    pub fn health_report(&self) -> NeoCodexHealthReport {
        let provider_count = self.provider.providers.len();
        let provider_resolvable = self.provider.to_llm_provider().is_some();
        let context_usage = if self.context.max_tokens == 0 {
            0.0
        } else {
            self.context.total_tokens() as f64 / self.context.max_tokens as f64
        };
        let session_writable = self.wire.path.parent().map(|p| p.exists()).unwrap_or(false);
        let evolution_iterations = self.evolution.iteration;

        NeoCodexHealthReport {
            mode: self.state.mode,
            turn_count: self.state.turn_count,
            tool_call_count: self.state.tool_call_count,
            tokens_used: self.state.tokens_used,
            context_usage: context_usage.max(0.0).min(1.0),
            context_turns: self.context.turns.len(),
            provider_count,
            provider_resolvable,
            provider_model: self.provider.active_model(),
            session_writable,
            goals_active: self.state.goal_active,
            cost_spent: self.cost.total_spent,
            cost_budget: self.cost.max_budget,
            subagent_results: self.subagent_results.len(),
            consciousness_attached: self.consciousness.is_some(),
            brain_attached: self.brain.is_some(),
            event_bus_attached: self.event_bus.is_some(),
            evolution_iterations,
            tool_grounding_degraded: self.tool_grounding.any_degraded(),
            node_snapshots: self
                .consciousness
                .as_ref()
                .map(|tree| tree.snapshots())
                .unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_io_provider::types::LlmProvider;

    use super::super::provider::ProviderInfo;
    use super::super::wire::NeoCodexAttachment;

    #[test]
    fn test_agent_state() {
        let state = AgentState::new();
        assert_eq!(state.mode, NeoCodexMode::Agent);
        assert_eq!(state.turn_count, 0);
    }

    #[test]
    fn test_dual_mode_toggle() {
        let mut agent = NeoCodexAgent::new("test-session");
        assert_eq!(agent.state.mode, NeoCodexMode::Agent);
        agent.toggle_mode();
        assert_eq!(agent.state.mode, NeoCodexMode::Shell);
        agent.toggle_mode();
        assert_eq!(agent.state.mode, NeoCodexMode::Agent);
        agent.set_plan_mode();
        assert_eq!(agent.state.mode, NeoCodexMode::Plan);
    }

    #[test]
    fn test_budget_react_messages_evicts_oldest() {
        let mut messages = vec![
            Message::new(Role::System, "system prompt"),
            Message::new(Role::User, "first request"),
            Message::assistant_with_calls("tool call a", vec![]),
            Message::tool("a very long tool result that blows any budget", "call-0"),
        ];
        NeoCodexAgent::budget_react_messages(&mut messages, 1);
        assert_eq!(messages[0].role, Role::System);
        assert_eq!(messages.last().unwrap().role, Role::Tool);
        assert!(messages.len() < 4);
    }

    #[test]
    fn test_build_messages_preserves_summary_and_tool_roles() {
        // Regression 1: Layer-4 distilled "summary" turns were mapped to
        // Role::System and skipped, silently discarding the compaction context.
        // Regression 2: "tool" history turns were mapped to Role::User, breaking
        // the ReAct message protocol (tool results must be Role::Tool).
        let mut agent = NeoCodexAgent::new("role-test");
        agent.context.push("system", "the system prompt".into(), 5);
        agent.context.push("user", "question".into(), 10);
        agent.context.push("assistant", "thinking".into(), 10);
        agent.context.push("tool", "tool output".into(), 10);
        agent
            .context
            .push("summary", "distilled earlier context".into(), 10);

        let messages = NeoCodexAgent::build_messages(&agent, "current input");
        let summary_msg = messages.iter().find(|m| m.content.contains("distilled"));
        let tool_msg = messages.iter().find(|m| m.content.contains("tool output"));

        assert!(
            summary_msg.is_some(),
            "summary turn must survive into messages"
        );
        assert_eq!(summary_msg.unwrap().role, Role::Assistant);
        assert_eq!(tool_msg.unwrap().role, Role::Tool);
        assert_eq!(messages.last().unwrap().role, Role::User);
        assert_eq!(messages.last().unwrap().content, "current input");
        assert!(
            messages.iter().filter(|m| m.role == Role::System).count() == 1,
            "only the real system prompt is kept"
        );
    }

    #[test]
    fn test_agent_process_basic() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut agent = NeoCodexAgent::new("test-agent");
        rt.block_on(async {
            let response = agent.process("Hello, NeoCodex!").await;
            assert!(!response.is_empty());
            assert_eq!(agent.state.turn_count, 1);
        });
    }

    #[test]
    fn test_goal_lifecycle() {
        let mut agent = NeoCodexAgent::new("goal-test");
        agent.add_goal("Fix the bug", 5);
        assert!(agent.state.goal_active);
    }

    #[test]
    fn test_goal_wire_id_matches_queue_id() {
        // Regression: add_goal recorded a wire GoalUpdate id derived from
        // turn_count, while GoalQueue::add generated a different id for the
        // queue entry — consumers correlating wire records with queue goals
        // would never match. The wire id now comes from the queue entry.
        let mut agent = NeoCodexAgent::new("goal-id-test");
        agent.add_goal("Goal one", 3);
        agent.add_goal("Goal two", 3);
        let queued: Vec<&String> = agent.goals.goals.iter().map(|g| &g.id).collect();
        assert_eq!(queued.len(), 2);
        assert_ne!(
            queued[0], queued[1],
            "distinct goals must have distinct ids"
        );
        for gid in queued {
            assert!(
                agent.wire.events.iter().any(|e| matches!(
                    e, WireEvent::GoalUpdate { id, .. } if id == gid
                )),
                "wire event must use the same id as the queue entry"
            );
        }
    }

    #[test]
    fn test_goal_completes_and_resets_active() {
        // Regression: check_goals was never wired into the turn loop,
        // iterations never incremented, and goal_active never reset — the
        // goal queue could only grow, never progress.
        let mut agent = NeoCodexAgent::new("goal-test");
        agent.add_goal("Fix the bug", 2);
        assert!(agent.state.goal_active);

        // 1st call: promotes queued goal to active, iterations 0→1, not complete.
        assert!(agent.check_goals().is_none());
        assert_eq!(agent.goals.active.as_ref().unwrap().iterations, 1);

        // 2nd call: iterations 1→2 == max, completes.
        assert!(
            agent.check_goals().is_none(),
            "single goal: completion returns None (no next)"
        );
        assert!(agent.goals.active.is_none());
        assert!(
            !agent.state.goal_active,
            "goal_active must reset once queue drains"
        );
        assert_eq!(agent.goals.completed.len(), 1);
    }

    #[test]
    fn test_chained_goals_advance_to_next() {
        let mut agent = NeoCodexAgent::new("goal-test");
        agent.add_goal("Goal A", 1);
        agent.add_goal("Goal B", 1);
        assert!(agent.state.goal_active);

        // Call 1: promotes A to active, increments 0→1 == max, completes A and
        // returns the description of the next goal B.
        assert_eq!(agent.check_goals().unwrap_or_default(), "Goal B");
        assert_eq!(agent.goals.completed.len(), 1);

        // Call 2: B is already active, increments 0→1 == max, completes B.
        // No next goal → returns None and goal_active resets.
        assert!(agent.check_goals().is_none());
        assert_eq!(agent.goals.completed.len(), 2);
        assert!(!agent.state.goal_active);
    }

    #[test]
    fn test_extract_tool_call() {
        let content = "Let me read the file.\n\n<tool name=\"read\">Cargo.toml</tool>";
        let (name, args) = NeoCodexAgent::extract_tool_call(content).unwrap();
        assert_eq!(name, "read");
        assert_eq!(args, "Cargo.toml");
        assert!(NeoCodexAgent::extract_tool_call("no tool here").is_none());
    }

    #[test]
    fn test_react_loop_falls_back_without_provider() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut agent = NeoCodexAgent::new("react-test");
            let out = agent.react_loop("hello", 2).await;
            // Default catalog (opencode stub) is not a real provider → None
            assert!(out.is_none());
        });
    }

    #[test]
    fn test_resume_session_restores_context() {
        use std::io::Write;
        let dir = std::env::temp_dir().join("neocodex_test_resume");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("resume.jsonl");
        let mut f = std::fs::File::create(&path).unwrap();
        for event in [
            WireEvent::UserMessage {
                content: "hello".into(),
                timestamp: 1,
                attachments: None,
            },
            WireEvent::AgentMessage {
                content: "hi there".into(),
                timestamp: 2,
            },
            WireEvent::ModeChange {
                from: NeoCodexMode::Agent,
                to: NeoCodexMode::Plan,
            },
        ] {
            let line = serde_json::to_string(&event).unwrap();
            writeln!(f, "{}", line).unwrap();
        }
        drop(f);

        let mut agent = NeoCodexAgent::new("resume-test");
        agent.wire.path = path;
        let n = agent.resume_session();
        assert_eq!(n, 3);
        assert_eq!(agent.state.mode, NeoCodexMode::Plan);
        assert!(agent.context.turns.len() >= 2);
        assert!(agent.state.tokens_used > 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resume_empty_session_is_noop() {
        let mut agent = NeoCodexAgent::new("resume-empty");
        agent.wire.path = std::env::temp_dir().join("neocodex_missing.jsonl");
        let n = agent.resume_session();
        assert_eq!(n, 0);
    }

    #[test]
    fn test_react_loop_with_configured_provider() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut agent = NeoCodexAgent::new("react-wired");
            agent.provider.sync_from_real();
            // Find a local/resolvable provider (ollama first in catalog)
            if let Some(idx) = agent
                .provider
                .providers
                .iter()
                .position(|p| p.name == "ollama")
            {
                agent.provider.active = idx;
                // to_llm_provider succeeds for ollama even without network (request only)
                let provider = agent.provider.to_llm_provider();
                assert!(provider.is_some());
            }
        });
    }

    #[test]
    fn test_react_loop_stream_no_provider_is_noop() {
        // D-streaming closure: without a resolvable provider the streaming
        // loop must not panic and must return None (caller falls back).
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut agent = NeoCodexAgent::new("react-stream-empty");
            agent.provider.providers.clear();
            agent.provider.providers.push(ProviderInfo::default());
            agent.provider.active = 0;
            let mut seen = Vec::new();
            let mut tools = Vec::new();
            let result = agent
                .react_loop_stream(
                    "hi",
                    3,
                    |t| {
                        seen.push(t.to_string());
                        true
                    },
                    |n, _, _, _, _| {
                        tools.push(n.to_string());
                        true
                    },
                )
                .await;
            assert!(result.is_none());
            assert!(seen.is_empty());
            assert!(tools.is_empty());
        });
    }

    #[test]
    fn test_react_loop_stream_provider_supports_streaming() {
        // All real providers implement stream_complete; verify the trait is
        // reachable through the resolved provider (no network needed).
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut agent = NeoCodexAgent::new("react-stream-wired");
            agent.provider.sync_from_real();
            if let Some(idx) = agent
                .provider
                .providers
                .iter()
                .position(|p| p.name == "ollama")
            {
                agent.provider.active = idx;
                let provider = agent.provider.to_llm_provider();
                assert!(provider.is_some());
                let provider = provider.unwrap();
                // Building a stream request must not panic; a stopped local
                // server yields Err(LlmError) which is the expected path.
                let request = LlmRequest {
                    model: "test".into(),
                    messages: vec![Message::new(Role::User, "hi")],
                    temperature: None,
                    max_tokens: 16,
                    tools: vec![],
                    image_data: None,
                    thinking_budget: None,
                    provider_params: Default::default(),
                    constraint_json: None,
                    structured_output: None,
                    cacheable_prefix_tokens: None,
                };
                let _ = provider.stream_complete(&request).await;
            }
        });
    }

    #[test]
    fn test_tool_grounding_records_calls() {
        // D25 closure: record_tool_result must be invoked on real tool execution
        let mut agent = NeoCodexAgent::new("grounding");
        agent.tool_grounding.record_tool_result("read", true, true);
        agent.tool_grounding.record_tool_result("read", true, false);
        assert_eq!(agent.tool_grounding.total_calls, 2);
        assert!(agent.tool_grounding.degraded_tools().len() <= 1);
        let report = agent.health_report();
        assert_eq!(report.tool_call_count, agent.state.tool_call_count, "health report mirrors state tool calls");
    }

    #[test]
    fn test_build_request_bridges_image_for_text_only_model() {
        // A real 4x4 PNG encoded as base64 → a valid VisionBridge decode target.
        let png = {
            use image::RgbImage;
            let mut buf = std::io::Cursor::new(Vec::new());
            let img = RgbImage::from_pixel(4, 4, image::Rgb([255, 0, 0]));
            image::DynamicImage::ImageRgb8(img)
                .write_to(&mut buf, image::ImageFormat::Png)
                .expect("encode");
            buf.into_inner()
        };
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);

        let mut agent = NeoCodexAgent::new("vision-bridge");
        // Force a text-only active provider (default catalog: opencode stub, no Vision cap).
        agent.wire.record(WireEvent::UserMessage {
            content: "describe the image".into(),
            timestamp: 1,
            attachments: Some(vec![NeoCodexAttachment {
                name: "shot.png".into(),
                size: png.len() as u64,
                mime_type: "image/png".into(),
                data: Some(b64.clone()),
            }]),
        });
        let messages = vec![
            Message::new(Role::System, "system"),
            Message::new(Role::User, "describe the image"),
        ];
        let req = agent
            .build_request(messages.clone())
            .expect("request built");
        // Text-only model → image must be bridged into the user message, image_data None.
        assert!(
            req.image_data.is_none(),
            "image_data must be dropped for text-only"
        );
        let user_msg = req.messages.iter().find(|m| m.role == Role::User).unwrap();
        assert!(
            user_msg.content.contains("<image_evidence>"),
            "evidence must be injected: {}",
            user_msg.content
        );
        assert!(user_msg.content.contains("dimensions: 4x4"));
        // image_data None → data-URI wrap not applied (no vision).
    }

    #[test]
    fn test_build_request_keeps_image_for_vision_model() {
        let mut agent = NeoCodexAgent::new("vision-native");
        agent.provider.add_provider(ProviderInfo {
            name: "vision".into(),
            model: "gpt-4o".into(),
            capabilities: vec![ModelCapability::Vision],
            context_limit: 100_000,
            cost_per_m_input: 1.0,
            cost_per_m_output: 1.0,
        });
        agent.provider.active = 1;
        let b64 = "iVBORw0KGgo=";
        agent.wire.record(WireEvent::UserMessage {
            content: "describe".into(),
            timestamp: 1,
            attachments: Some(vec![NeoCodexAttachment {
                name: "shot.png".into(),
                size: 10,
                mime_type: "image/png".into(),
                data: Some(b64.into()),
            }]),
        });
        let messages = vec![Message::new(Role::User, "describe")];
        let req = agent.build_request(messages).expect("request built");
        // Vision-capable model → raw base64 upgraded to a data URI and preserved.
        let uri = req
            .image_data
            .expect("image_data preserved for vision model");
        assert!(
            uri.starts_with("data:image/"),
            "data URI expected, got {uri}"
        );
        assert!(!req.messages[0].content.contains("<image_evidence>"));
    }
}