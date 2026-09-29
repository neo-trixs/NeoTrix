// ── NeoCodex agent exec: process / exec_shell,plan,agent / react loops / tools (split from agent.rs, behavior unchanged) ──

use std::collections::HashMap;
use std::time::Instant;

use base64::Engine as _;
use crate::l1_action::nt_io::nt_io_provider::context_budget::apply_context_budget;
use crate::l1_action::nt_io::nt_io_provider::context_budget::estimate_messages_tokens;
use crate::l1_action::nt_io::nt_io_provider::types::{LlmRequest, Message, Role, Tool};

use super::{NeoCodexAgent, StreamOutcome};
use super::super::evolution::{EvolutionLoop, NeoCodexHealthReport};
use super::super::hooks::{HookDecision, ToolCallContext};
use super::super::provider::{ModelCapability, NeoCodexMode};
use super::super::wire::WireEvent;

impl NeoCodexAgent {
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
    pub(crate) async fn react_loop(&mut self, input: &str, max_steps: usize) -> Option<String> {
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
                        vec![crate::l1_action::nt_io::nt_io_provider::types::ToolCallInfo {
                            id: format!("call-{}", step),
                            name: name.clone(),
                            arguments: args.clone(),
                            call_type: Some("function".into()),
                            function: Some(crate::l1_action::nt_io::nt_io_provider::types::ToolCallFunction {
                                name: name.clone(),
                                arguments: args.clone(),
                            }),
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
    ) -> StreamOutcome
    where
        F: FnMut(&str) -> bool + Send + Sync,
        G: FnMut(&str, &str, &str, u64, bool) -> bool + Send + Sync,
    {
        let provider = match self.provider.to_llm_provider() {
            Some(p) => p,
            None => {
                return StreamOutcome {
                    content: None,
                    error: Some("provider 未配置，无法开始生成".to_string()),
                }
            }
        };

        let mut messages = self.build_messages(input);
        let mut step = 0;
        let mut final_answer: Option<String> = None;
        let mut accumulated = String::new();
        let mut cancelled = false;

        while step < max_steps && !cancelled {
            Self::budget_react_messages(&mut messages, self.context.max_tokens);
            let request = match self.build_request(messages.clone()) {
                Some(r) => r,
                None => {
                    return StreamOutcome {
                        content: if accumulated.is_empty() { None } else { Some(accumulated) },
                        error: Some("请求构建失败（provider 参数无效）".to_string()),
                    }
                }
            };

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
                    // F1: 保留已累积的 partial token，错误由调用方经事件呈现，
                    // 不再把 "[provider error] …" 当作正常回答返回
                    return StreamOutcome {
                        content: if accumulated.is_empty() { None } else { Some(accumulated) },
                        error: Some(e.to_string()),
                    };
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
                        return StreamOutcome {
                            content: if accumulated.is_empty() { None } else { Some(accumulated) },
                            error: Some(e.to_string()),
                        };
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
                return StreamOutcome {
                    content: Some(response_content),
                    error: None,
                };
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
                        vec![crate::l1_action::nt_io::nt_io_provider::types::ToolCallInfo {
                            id: format!("call-{}", step),
                            name: name.clone(),
                            arguments: args.clone(),
                            call_type: Some("function".into()),
                            function: Some(crate::l1_action::nt_io::nt_io_provider::types::ToolCallFunction {
                                name: name.clone(),
                                arguments: args.clone(),
                            }),
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
            StreamOutcome {
                content: Some(accumulated),
                error: None,
            }
        } else {
            StreamOutcome {
                content: final_answer,
                error: None,
            }
        }
    }

    /// Build system + history + current user messages from the context pipeline.
    pub(crate) fn build_messages(&self, input: &str) -> Vec<Message> {
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
    pub(crate) fn budget_react_messages(messages: &mut Vec<Message>, max_tokens: usize) {
        apply_context_budget(messages, max_tokens);
    }

    /// Build an LlmRequest from the current catalog's active provider.
    pub(crate) fn build_request(&self, messages: Vec<Message>) -> Option<LlmRequest> {
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
            || crate::l1_action::nt_action_facade::model_supports_vision(&active_model);
        let mut messages = messages;
        let mut image_data = image_data;
        if image_data.is_some() && !active_has_vision {
            if let Some(b64) = image_data.take() {
                if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&b64) {
                    if let Ok((evidence, _feat)) =
                        crate::l1_action::nt_action_facade::VisionBridge::analyze_cached(&bytes)
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
                req = req.with_image_b64(raw);
            }
        }
        Some(req)
    }

    /// Parse a `<tool name="...">args</tool>` block from the model output.
    pub(crate) fn extract_tool_call(content: &str) -> Option<(String, String)> {
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
            // P2-5: MCP tool call (MCP registry removed — stub)
            "mcp_call" => {
                "[mcp_call error] MCP registry not available".to_string()
            }
            "mcp_list" => {
                "[mcp_list] MCP registry not available".to_string()
            }
            _ => format!("Unknown tool: {}", name),
        }
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
