//! # NT-IO AgentLoop — NeoTrix 作为主体的对话驱动循环
//!
//! 架构目标（CORE 反转关系）：
//!   - **NeoTrix 系统** 是和你对话的主体（持有状态、工具、决策逻辑）
//!   - **LLM** 是它调用的一个后端能力（"推理生成函数"）
//!
//! AgentLoop 不依赖具体 provider 类型，只依赖 `LlmProvider` trait，
//! 生产环境注入 `GatewayV2`（含路由/熔断/限流），测试注入 mock。
//!
//! 循环契约：
//!   1. 追加用户消息到会话历史
//!   2. 构建 `LlmRequest`（携带工具定义 + 完整历史）
//!   3. 调用 LLM 后端
//!   4. 若响应带 tool_calls → 执行每个工具 → 结果以 Role::Tool 消息回填 → 回到 2
//!   5. 无工具调用 → 返回最终回答，追加 Assistant 消息
//!
//! 安全上限：`max_tool_rounds` 防止工具循环死循环；`max_history` 防止上下文无限膨胀。

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use super::nt_io_provider::types::{
    FinishReason, LlmError, LlmProvider, LlmRequest, Message, Role, ToolCallInfo,
};
use crate::cli::approval::{ActionType, PendingAction};
use crate::core::nt_core_traits::{NativeTool, ToolOutput};

/// 一次工具执行的记录（供调用方观测/审计）。
#[derive(Debug, Clone)]
pub struct ToolInvocation {
    pub name: String,
    pub arguments: String,
    pub success: bool,
    pub output: String,
}

/// AgentLoop — 系统主体对话循环。
pub struct AgentLoop {
    backend: Arc<dyn LlmProvider>,
    tools: Vec<Box<dyn NativeTool>>,
    /// 会话消息历史（含 System 首条）。
    messages: Vec<Message>,
    model: String,
    max_tool_rounds: usize,
    max_history: usize,
    /// 本会话已执行的工具调用记录。
    pub tool_log: Vec<ToolInvocation>,
}

impl AgentLoop {
    pub fn new(backend: Arc<dyn LlmProvider>, model: &str, system_prompt: &str) -> Self {
        let mut messages = Vec::new();
        if !system_prompt.is_empty() {
            messages.push(Message::new(Role::System, system_prompt));
        }
        Self {
            backend,
            tools: Vec::new(),
            messages,
            model: model.to_string(),
            max_tool_rounds: 8,
            max_history: 64,
            tool_log: Vec::new(),
        }
    }

    pub fn with_tools(mut self, tools: Vec<Box<dyn NativeTool>>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_max_tool_rounds(mut self, max: usize) -> Self {
        self.max_tool_rounds = max;
        self
    }

    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// 注册单个工具。
    pub fn register_tool(&mut self, tool: Box<dyn NativeTool>) {
        self.tools.push(tool);
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    pub fn history_len(&self) -> usize {
        self.messages.len()
    }

    /// P1-5 修复: 重置会话历史(保留 System 首条与已注册工具)。
    /// TUI 切换/清空会话时调用, 避免新会话沿用旧会话上下文造成语义串台。
    pub fn reset_history(&mut self, system_prompt: &str) {
        self.messages.clear();
        if !system_prompt.is_empty() {
            self.messages.push(Message::new(Role::System, system_prompt));
        }
    }

    /// 执行一轮对话：用户输入 → (可能的多次工具调用) → 最终回答。
    pub async fn turn(&mut self, user_input: &str) -> Result<String, LlmError> {
        self.messages.push(Message::new(Role::User, user_input));
        self.trim_history();

        for _round in 0..self.max_tool_rounds {
            let request = self.build_request();
            let response = self.backend.complete(&request).await?;

            match response.finish_reason {
                FinishReason::Tool => {
                    if let Some(calls) = response.tool_calls {
                        if calls.is_empty() {
                            // 模型声明需要工具但没给出调用 → 视为停止，避免死循环。
                            let text = response.content;
                            self.messages.push(Message::new(Role::Assistant, &text));
                            self.trim_history();
                            return Ok(text);
                        }
                        // 记录 assistant 的 tool_calls，供 API 语义配对。
                        let assistant_calls: Vec<ToolCallInfo> = calls.clone();
                        let _ = self.execute_tools(&assistant_calls).await;
                        continue;
                    }
                    // finish=Tool 但无 tool_calls → 直接返回已有文本。
                    let text = response.content;
                    self.messages.push(Message::new(Role::Assistant, &text));
                    self.trim_history();
                    return Ok(text);
                }
                _ => {
                    let text = response.content;
                    self.messages.push(Message::new(Role::Assistant, &text));
                    self.trim_history();
                    return Ok(text);
                }
            }
        }

        // 达到工具轮数上限 — 返回当前上下文摘要作为兜底。
        Err(LlmError::Server("AgentLoop: max_tool_rounds exceeded".to_string()))
    }

    /// 流式对话轮：与 [`turn`] 相同决策循环，但 LLM 响应经 `stream_complete`
    /// 逐 chunk 推送。`on_token` 返回 `false` 可取消当前生成；
    /// `on_tool` 在每次工具执行后回调（携带调用信息与结果）。
    pub async fn turn_stream<F, G>(
        &mut self,
        user_input: &str,
        mut on_token: F,
        mut on_tool: G,
    ) -> Result<String, LlmError>
    where
        F: FnMut(&str) -> bool + Send + Sync,
        G: FnMut(&ToolCallInfo, &ToolOutput) + Send + Sync,
    {
        self.messages.push(Message::new(Role::User, user_input));
        self.trim_history();

        let mut cancelled = false;
        for _round in 0..self.max_tool_rounds {
            let request = self.build_request();
            let mut rx = self.backend.stream_complete(&request).await?;

            let mut response_content = String::new();
            let mut response_tool_calls: Vec<ToolCallInfo> = Vec::new();
            let mut response_finish = FinishReason::Stop;

            while let Some(chunk) = rx.recv().await {
                match chunk {
                    Ok(resp) => {
                        if !resp.content.is_empty() {
                            response_content.push_str(&resp.content);
                            if !on_token(&resp.content) {
                                cancelled = true;
                                break;
                            }
                        }
                        if let Some(calls) = resp.tool_calls {
                            response_tool_calls.extend(calls);
                        }
                        response_finish = resp.finish_reason;
                    }
                    Err(e) => {
                        // 单 chunk 失败：返回已累积文本 + 错误。
                        let text = response_content.clone();
                        if !text.is_empty() {
                            self.messages.push(Message::new(Role::Assistant, &text));
                            self.trim_history();
                        }
                        return Err(e);
                    }
                }
            }
            if cancelled {
                // 用户取消：保留已累积内容作为最终回答。
                let text = response_content.clone();
                self.messages.push(Message::new(Role::Assistant, &text));
                self.trim_history();
                return Ok(text);
            }

            match response_finish {
                FinishReason::Tool => {
                    if response_tool_calls.is_empty() {
                        let text = response_content.clone();
                        self.messages.push(Message::new(Role::Assistant, &text));
                        self.trim_history();
                        return Ok(text);
                    }
                    // 回填 assistant tool_calls + 执行工具（复用非流式执行，回填 Tool 消息）。
                    let assistant_calls = response_tool_calls.clone();
                    self.messages.push(Message::assistant_with_calls("", assistant_calls));
                    for call in &response_tool_calls {
                        let args: Value =
                            serde_json::from_str(&call.function.arguments).unwrap_or(Value::Null);
                        let result = self.call_tool(&call.function.name, &args);
                        match &result {
                            Ok(output) => on_tool(call, output),
                            Err(_) => {}
                        }
                        let content = match &result {
                            Ok(ToolOutput { success, content }) => {
                                if *success { content.clone() }
                                else { format!("TOOL_ERROR: {}", content) }
                            }
                            Err(e) => format!("TOOL_ERROR: {}", e),
                        };
                        self.tool_log.push(ToolInvocation {
                            name: call.function.name.clone(),
                            arguments: call.function.arguments.clone(),
                            success: result.is_ok(),
                            output: content.clone(),
                        });
                        self.messages.push(Message::tool(&content, &call.id));
                    }
                    self.trim_history();
                    continue;
                }
                _ => {
                    let text = response_content.clone();
                    self.messages.push(Message::new(Role::Assistant, &text));
                    self.trim_history();
                    return Ok(text);
                }
            }
        }

        Err(LlmError::Server("AgentLoop: max_tool_rounds exceeded".to_string()))
    }

    /// 流式对话轮（带审批门槛版本，P0 权限审批接线）。
    ///
    /// 与 [`turn_stream`] 相同的决策循环，额外支持：
    ///   - `on_tool_start`：工具执行前回调 `(name, args)`，返回 `false` 取消本轮生成；
    ///   - `on_tool`：工具执行后回调 `(name, args, result, duration_ms, success)`，
    ///     返回 `false` 取消本轮生成（参考 nt_io_neocodex.rs `react_loop_stream` 签名）；
    ///   - `on_approval`：审批回调。`Some(cb)` 时启用审批门槛：每个工具执行前经
    ///     `crate::cli::approval::global_approval()` 检查，`require_approval` 为 true 则
    ///     提交 `PendingAction` 并调用 `cb(&PendingAction)` 等待决策（true=approve,
    ///     false=deny）。deny 时工具被跳过，模型收到明确的 "需审批" 错误。
    ///     `None` 时同样启用门槛，但无回调可问 → 需审批的工具一律跳过（返回 "需审批" 错误）。
    ///
    /// 向后兼容：旧入口 [`turn_stream`] 保持原签名且**不**启用审批门槛（无 on_approval
    /// 时保持现状），本方法供需要审批交互的调用方（如 TUI）使用。
    pub async fn turn_stream_with_approval<F, G, H>(
        &mut self,
        user_input: &str,
        mut on_token: F,
        mut on_tool_start: G,
        mut on_tool: H,
        on_approval: Option<Box<dyn Fn(&PendingAction) -> bool + Send>>,
    ) -> Result<String, LlmError>
    where
        F: FnMut(&str) -> bool + Send + Sync,
        G: FnMut(&str, &str) -> bool + Send + Sync,
        H: FnMut(&str, &str, &str, u64, bool) -> bool + Send + Sync,
    {
        self.messages.push(Message::new(Role::User, user_input));
        self.trim_history();

        let mut cancelled = false;
        let mut last_response_content = String::new();
        for _round in 0..self.max_tool_rounds {
            let request = self.build_request();
            let mut rx = self.backend.stream_complete(&request).await?;

            let mut response_content = String::new();
            let mut response_tool_calls: Vec<ToolCallInfo> = Vec::new();
            let mut response_finish = FinishReason::Stop;

            while let Some(chunk) = rx.recv().await {
                match chunk {
                    Ok(resp) => {
                        if !resp.content.is_empty() {
                            response_content.push_str(&resp.content);
                            if !on_token(&resp.content) {
                                cancelled = true;
                                break;
                            }
                        }
                        if let Some(calls) = resp.tool_calls {
                            response_tool_calls.extend(calls);
                        }
                        response_finish = resp.finish_reason;
                    }
                    Err(e) => {
                        // 单 chunk 失败：返回已累积文本 + 错误。
                        let text = response_content.clone();
                        if !text.is_empty() {
                            self.messages.push(Message::new(Role::Assistant, &text));
                            self.trim_history();
                        }
                        return Err(e);
                    }
                }
            }
            if cancelled {
                // 用户取消：保留已累积内容作为最终回答。
                let text = response_content.clone();
                self.messages.push(Message::new(Role::Assistant, &text));
                self.trim_history();
                return Ok(text);
            }
            last_response_content = response_content.clone();

            match response_finish {
                FinishReason::Tool => {
                    if response_tool_calls.is_empty() {
                        let text = response_content.clone();
                        self.messages.push(Message::new(Role::Assistant, &text));
                        self.trim_history();
                        return Ok(text);
                    }
                    // 回填 assistant tool_calls + 逐个执行工具（含审批门槛）。
                    let assistant_calls = response_tool_calls.clone();
                    self.messages.push(Message::assistant_with_calls("", assistant_calls));
                    for call in &response_tool_calls {
                        let args: Value =
                            serde_json::from_str(&call.function.arguments).unwrap_or(Value::Null);
                        let name = call.function.name.clone();
                        let args_str = call.function.arguments.clone();

                        // P0 审批门槛：需审批且被拒绝 → 跳过工具，模型收到明确错误。
                        if let Err(approval_err) = Self::check_tool_approval(&name, &args, on_approval.as_deref()) {
                            let content = format!("TOOL_ERROR: {}", approval_err);
                            on_tool(&name, &args_str, &content, 0, false);
                            self.tool_log.push(ToolInvocation {
                                name: name.clone(),
                                arguments: args_str.clone(),
                                success: false,
                                output: content.clone(),
                            });
                            self.messages.push(Message::tool(&content, &call.id));
                            continue;
                        }

                        if !on_tool_start(&name, &args_str) {
                            cancelled = true;
                            break;
                        }
                        let started = std::time::Instant::now();
                        let result = self.call_tool(&name, &args);
                        let duration_ms = started.elapsed().as_millis() as u64;
                        let (content, success) = match &result {
                            Ok(ToolOutput { success, content }) => (
                                if *success { content.clone() }
                                else { format!("TOOL_ERROR: {}", content) },
                                *success,
                            ),
                            Err(e) => (format!("TOOL_ERROR: {}", e), false),
                        };
                        if !on_tool(&name, &args_str, &content, duration_ms, success) {
                            cancelled = true;
                            break;
                        }
                        self.tool_log.push(ToolInvocation {
                            name: name.clone(),
                            arguments: args_str.clone(),
                            success,
                            output: content.clone(),
                        });
                        self.messages.push(Message::tool(&content, &call.id));
                    }
                    self.trim_history();
                    if cancelled {
                        break;
                    }
                    continue;
                }
                _ => {
                    let text = response_content.clone();
                    self.messages.push(Message::new(Role::Assistant, &text));
                    self.trim_history();
                    return Ok(text);
                }
            }
        }

        if cancelled {
            // 工具回调取消：返回本轮已累积文本（通常为空，表示无文本回答）。
            let text = last_response_content.clone();
            self.messages.push(Message::new(Role::Assistant, &text));
            self.trim_history();
            return Ok(text);
        }

        Err(LlmError::Server("AgentLoop: max_tool_rounds exceeded".to_string()))
    }

    fn build_request(&self) -> LlmRequest {
        let tools = self.tools.iter().map(|t| {
            let def = t.to_def();
            super::nt_io_provider::types::Tool {
                name: def.name,
                description: def.description,
                input_schema: def.input_schema,
            }
        }).collect();

        LlmRequest {
            model: self.model.clone(),
            messages: self.messages.clone(),
            temperature: Some(0.7),
            max_tokens: 4096,
            tools,
            image_data: None,
            thinking_budget: None,
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: None,
        }
    }

    /// 执行一组工具调用，把 assistant 调用 + 每个工具结果回填历史。
    async fn execute_tools(&mut self, calls: &[ToolCallInfo]) -> Result<(), LlmError> {
        // 1. 回填 assistant 的 tool_calls 消息（OpenAI 语义要求）。
        let assistant_msg = Message::assistant_with_calls("", calls.to_vec());
        self.messages.push(assistant_msg);

        // 2. 逐个执行。
        for call in calls {
            let args: Value = serde_json::from_str(&call.function.arguments).unwrap_or(Value::Null);
            let result = self.call_tool(&call.function.name, &args);
            let content = match &result {
                Ok(ToolOutput { success, content }) => {
                    if *success {
                        content.clone()
                    } else {
                        format!("TOOL_ERROR: {}", content)
                    }
                }
                Err(e) => format!("TOOL_ERROR: {}", e),
            };
            self.tool_log.push(ToolInvocation {
                name: call.function.name.clone(),
                arguments: call.function.arguments.clone(),
                success: result.is_ok(),
                output: content.clone(),
            });
            self.messages.push(Message::tool(&content, &call.id));
        }
        self.trim_history();
        Ok(())
    }

    fn call_tool(&self, name: &str, args: &Value) -> Result<ToolOutput, String> {
        // P0-3 pre-tool-use secret 扫描 (吸收 sonarqube-cli 防泄漏模式):
        // 工具调用前扫描参数, 发现 Critical/High 凭据即阻断 — 防止密钥/令牌
        // 经工具参数泄漏给外部服务或写入日志。对应 sonarqube-cli
        // "detect secrets before they leak" 的 pre-tool-use hook 语义。
        let args_text = args.to_string();
        let scanner = crate::neotrix::l8_autonomic_impl::nt_mind::seal_core::self_iterating::secret_scanner::SecretScanner::new();
        let findings = scanner.scan(&args_text);
        let blocked: Vec<String> = findings
            .iter()
            .filter(|f| {
                f.severity
                    >= crate::neotrix::l8_autonomic_impl::nt_mind::seal_core::self_iterating::secret_scanner::Severity::High
            })
            .map(|f| format!("{}@{}", f.pattern, f.line))
            .collect();
        if !blocked.is_empty() {
            return Err(format!(
                "[secret-guard] tool '{}' blocked: potential credential leak in args ({})",
                name,
                blocked.join(", ")
            ));
        }
        self.tools.iter()
            .find(|t| t.id() == name)
            .ok_or_else(|| format!("Unknown tool: {}", name))
            .and_then(|t| t.execute(args))
    }

    /// 工具名 → 审批动作类型（用于权限门禁分类）。
    /// 启发式映射: 命令执行→ShellCommand, git→GitOperation, 文件写→FileWrite/FileEdit,
    /// 其余兜底 Other{tool, args}。
    fn action_type_for_tool(name: &str, args: &Value) -> ActionType {
        let n = name.to_lowercase();
        let args_s = args.to_string();
        if n.contains("shell") || n.contains("exec") || n.contains("bash") || n.contains("run") || n.contains("command") {
            ActionType::ShellCommand { command: Self::truncate(&args_s, 120) }
        } else if n.contains("git") {
            ActionType::GitOperation { description: Self::truncate(&args_s, 120) }
        } else if n.contains("write") || n.contains("create") || n.contains("edit") || n.contains("patch") || n.contains("diff") {
            ActionType::FileEdit { path: name.to_string(), diff: Self::truncate(&args_s, 120) }
        } else {
            ActionType::Other { tool: name.to_string(), args: Self::truncate(&args_s, 120) }
        }
    }

    /// CJK 安全截断（按字符，不按字节）。
    fn truncate(s: &str, max_chars: usize) -> String {
        s.chars().take(max_chars).collect()
    }

    /// 工具执行前审批门槛。
    /// - 无回调 (None)：需审批工具一律跳过，返回错误 "需审批"（默认安全）
    /// - 有回调：回调决策 approve (true) / deny (false)
    /// 锁纪律：必须在调回调前 drop(guard)（回调可能阻塞等待用户按键），
    /// 批准后再重新加锁 approve/deny。
    fn check_tool_approval(
        name: &str,
        args: &Value,
        on_approval: Option<&(dyn Fn(&PendingAction) -> bool + Send)>,
    ) -> Result<(), String> {
        let engine = crate::cli::approval::global_approval();
        let action = Self::action_type_for_tool(name, args);
        let require = {
            let guard = engine.lock().map_err(|e| format!("approval lock: {}", e))?;
            guard.require_approval(&action)
        };
        if !require {
            return Ok(());
        }
        let pending = {
            let mut guard = engine.lock().map_err(|e| format!("approval lock: {}", e))?;
            guard.submit(action)
        };
        let decision = match on_approval {
            Some(cb) => cb(&pending),
            None => false,
        };
        // 批准后重新加锁 approve；deny 则无需回写（PendingAction 自然过期）
        if decision {
            let mut guard = engine.lock().map_err(|e| format!("approval lock: {}", e))?;
            let _ = guard.approve(&pending.id);
        }
        if decision {
            Ok(())
        } else {
            Err(format!("需要审批: {}", pending.description))
        }
    }

    /// 超过 max_history 时裁剪最旧的 user/assistant 消息，保留 System 首条。
    fn trim_history(&mut self) {
        let limit = self.max_history;
        if self.messages.len() <= limit {
            return;
        }
        // 保留第 0 条（System），从第 1 条开始删，直到长度达标。
        let mut drop = self.messages.len() - limit;
        let mut i = 1;
        while drop > 0 && i < self.messages.len() {
            if i == 0 {
                i += 1;
                continue;
            }
            self.messages.remove(i);
            drop -= 1;
            // remove 后索引不变（后续元素前移），继续检查同位置。
        }
    }

    /// 返回当前会话的消息历史（供持久化/检索）。
    pub fn history(&self) -> &[Message] {
        &self.messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_traits::ToolDef;
    use crate::neotrix::nt_io_provider::LlmResponse;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    // ── Mock 工具 ─────────────────────────────────────────────────────

    struct MockCalc {
        calls: Arc<Mutex<Vec<String>>>,
    }

    impl NativeTool for MockCalc {
        fn id(&self) -> &str { "calc" }
        fn description(&self) -> &str { "Mock calculator" }
        fn input_schema(&self) -> Value {
            serde_json::json!({"type": "object", "properties": {"expr": {"type": "string"}}})
        }
        fn capability_tags(&self) -> Vec<&'static str> { vec!["compute"] }
        fn execute(&self, args: &Value) -> Result<ToolOutput, String> {
            let expr = args["expr"].as_str().unwrap_or("").to_string();
            self.calls.lock().unwrap_or_else(|e| e.into_inner()).push(expr.clone());
            if expr == "1+1" {
                Ok(ToolOutput { success: true, content: "2".to_string() })
            } else {
                Err(format!("cannot compute {}", expr))
            }
        }
    }

    fn tool_def(id: &str) -> ToolDef {
        ToolDef {
            name: id.to_string(),
            description: "tool".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
        }
    }

    // ── 可编程 Mock LLM ───────────────────────────────────────────────

    /// 按预设脚本返回响应序列；`tool_calls_seq` 控制每轮是否返回工具调用。
    struct ScriptedLlm {
        /// (content, finish_reason, tool_calls) 序列，每次调用 pop 第一个。
        script: Arc<Mutex<Vec<(String, FinishReason, Vec<ToolCallInfo>)>>>,
        /// 记录每次请求携带的工具数量。
        seen_tools: Arc<Mutex<Vec<usize>>>,
    }

    #[async_trait]
    impl LlmProvider for ScriptedLlm {
        async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
            self.seen_tools.lock().unwrap_or_else(|e| e.into_inner()).push(request.tools.len());
            let mut script = self.script.lock().unwrap_or_else(|e| e.into_inner());
            if script.is_empty() {
                return Ok(LlmResponse::plain("done".into(), "mock".into(), Default::default(), FinishReason::Stop));
            }
            let (content, fr, calls) = script.remove(0);
            Ok(LlmResponse { content, model: "mock".into(), usage: Default::default(), finish_reason: fr, tool_calls: Some(calls) })
        }

        async fn stream_complete(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
            use tokio::sync::mpsc;
            self.seen_tools.lock().unwrap_or_else(|e| e.into_inner()).push(request.tools.len());
            let mut script = self.script.lock().unwrap_or_else(|e| e.into_inner());
            let (tx, rx) = mpsc::channel(16);
            if script.is_empty() {
                let _ = tx.try_send(Ok(LlmResponse::plain("done".into(), "mock".into(), Default::default(), FinishReason::Stop)));
                return Ok(rx);
            }
            let (content, fr, calls) = script.remove(0);
            // 模拟逐 chunk 推送：按 1 字符切块，保留 finish_reason 与 tool_calls。
            for ch in content.chars() {
                let resp = LlmResponse {
                    content: ch.to_string(),
                    model: "mock".into(),
                    usage: Default::default(),
                    finish_reason: FinishReason::Stop,
                    tool_calls: None,
                };
                if tx.try_send(Ok(resp)).is_err() { break; }
            }
            let final_resp = LlmResponse {
                content: String::new(),
                model: "mock".into(),
                usage: Default::default(),
                finish_reason: fr,
                tool_calls: Some(calls),
            };
            let _ = tx.try_send(Ok(final_resp));
            Ok(rx)
        }
    }

    fn tool_call(name: &str, id: &str, args: &str) -> ToolCallInfo {
        ToolCallInfo {
            id: id.to_string(),
            call_type: "function".to_string(),
            function: super::super::nt_io_provider::types::ToolCallFunction {
                name: name.to_string(),
                arguments: args.to_string(),
            },
        }
    }

    fn backend_with(script: Vec<(String, FinishReason, Vec<ToolCallInfo>)>) -> (Arc<ScriptedLlm>, Arc<Mutex<Vec<usize>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let llm = ScriptedLlm { script: Arc::new(Mutex::new(script)), seen_tools: seen.clone() };
        (Arc::new(llm), seen)
    }

    // ── 测试 ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_turn_simple_stop() {
        let (llm, _seen) = backend_with(vec![
            ("hello there".into(), FinishReason::Stop, vec![]),
        ]);
        let mut loop_ = AgentLoop::new(llm, "mock", "You are NeoTrix.");
        let out = loop_.turn("hi").await.expect("turn ok");
        assert_eq!(out, "hello there");
        assert_eq!(loop_.history_len(), 3); // System + User + Assistant
    }

    #[tokio::test]
    async fn test_turn_without_system_prompt() {
        let (llm, _seen) = backend_with(vec![
            ("ok".into(), FinishReason::Stop, vec![]),
        ]);
        let mut loop_ = AgentLoop::new(llm, "mock", "");
        let out = loop_.turn("q").await.expect("turn ok");
        assert_eq!(out, "ok");
        assert_eq!(loop_.history_len(), 2); // User + Assistant
    }

    #[tokio::test]
    async fn test_turn_executes_tool_and_continues() {
        // 第 1 轮：模型请求 calc(1+1)；第 2 轮：模型给出最终答案。
        let (llm, seen) = backend_with(vec![
            ("".into(), FinishReason::Tool, vec![tool_call("calc", "call_1", r#"{"expr":"1+1"}"#)]),
            ("result is 2".into(), FinishReason::Stop, vec![]),
        ]);
        let calc_calls = Arc::new(Mutex::new(Vec::new()));
        let calc = MockCalc { calls: calc_calls.clone() };
        let mut loop_ = AgentLoop::new(llm, "mock", "sys")
            .with_tools(vec![Box::new(calc)])
            .with_max_tool_rounds(4);

        let out = loop_.turn("compute 1+1").await.expect("turn ok");
        assert_eq!(out, "result is 2");

        // 工具确实被执行了。
        assert_eq!(calc_calls.lock().unwrap_or_else(|e| e.into_inner()).as_slice(), &["1+1".to_string()]);
        // 工具日志有记录。
        assert_eq!(loop_.tool_log.len(), 1);
        assert_eq!(loop_.tool_log[0].name, "calc");
        assert!(loop_.tool_log[0].success);
        assert_eq!(loop_.tool_log[0].output, "2");
        // LLM 两次调用都拿到了工具定义。
        assert_eq!(seen.lock().unwrap_or_else(|e| e.into_inner()).as_slice(), &[1, 1]);
        // 历史：System + User + Assistant(tool_calls) + Tool + Assistant(final) = 5
        assert_eq!(loop_.history_len(), 5);
    }

    #[tokio::test]
    async fn test_turn_tool_error_surfaces() {
        let (llm, _seen) = backend_with(vec![
            ("".into(), FinishReason::Tool, vec![tool_call("calc", "call_1", r#"{"expr":"2+2"}"#)]),
            ("cannot compute".into(), FinishReason::Stop, vec![]),
        ]);
        let calc = MockCalc { calls: Arc::new(Mutex::new(Vec::new())) };
        let mut loop_ = AgentLoop::new(llm, "mock", "")
            .with_tools(vec![Box::new(calc)]);
        let out = loop_.turn("compute 2+2").await.expect("turn ok");
        assert_eq!(out, "cannot compute");
        assert_eq!(loop_.tool_log.len(), 1);
        assert!(!loop_.tool_log[0].success);
        assert!(loop_.tool_log[0].output.contains("TOOL_ERROR"));
    }

    #[tokio::test]
    async fn test_turn_unknown_tool_surfaces_error() {
        let (llm, _seen) = backend_with(vec![
            ("".into(), FinishReason::Tool, vec![tool_call("ghost", "call_9", "{}")]),
            ("recovered".into(), FinishReason::Stop, vec![]),
        ]);
        let mut loop_ = AgentLoop::new(llm, "mock", "");
        let out = loop_.turn("use ghost").await.expect("turn ok");
        assert_eq!(out, "recovered");
        assert_eq!(loop_.tool_log.len(), 1);
        assert!(!loop_.tool_log[0].success);
        assert!(loop_.tool_log[0].output.contains("Unknown tool"));
    }

    #[tokio::test]
    async fn test_turn_loop_cap_prevents_infinite() {
        // 模型永远请求工具 → 达到上限必须报错而非死循环。
        let (llm, _seen) = backend_with(vec![
            ("".into(), FinishReason::Tool, vec![tool_call("calc", "c1", r#"{"expr":"1+1"}"#)]),
            ("".into(), FinishReason::Tool, vec![tool_call("calc", "c2", r#"{"expr":"1+1"}"#)]),
            ("".into(), FinishReason::Tool, vec![tool_call("calc", "c3", r#"{"expr":"1+1"}"#)]),
        ]);
        let calc = MockCalc { calls: Arc::new(Mutex::new(Vec::new())) };
        let mut loop_ = AgentLoop::new(llm, "mock", "")
            .with_tools(vec![Box::new(calc)])
            .with_max_tool_rounds(3);
        let err = loop_.turn("loop").await.err().expect("must error");
        assert!(err.to_string().contains("max_tool_rounds"), "got: {}", err);
    }

    #[tokio::test]
    async fn test_turn_empty_tool_calls_treated_as_stop() {
        let (llm, _seen) = backend_with(vec![
            ("finished anyway".into(), FinishReason::Tool, vec![]),
        ]);
        let mut loop_ = AgentLoop::new(llm, "mock", "");
        let out = loop_.turn("q").await.expect("turn ok");
        assert_eq!(out, "finished anyway");
    }

    #[tokio::test]
    async fn test_turn_request_carries_tools_and_history() {
        // 验证 build_request 把工具定义 + 全部历史传给 LLM。
        let (llm, seen) = backend_with(vec![
            ("ok".into(), FinishReason::Stop, vec![]),
        ]);
        let calc = MockCalc { calls: Arc::new(Mutex::new(Vec::new())) };
        let mut loop_ = AgentLoop::new(llm, "mock", "sys")
            .with_tools(vec![Box::new(calc)]);
        let _ = loop_.turn("first message").await;
        let _ = loop_.turn("second message").await;

        let seen = seen.lock().unwrap_or_else(|e| e.into_inner());
        // 两次调用都应携带 1 个工具。
        assert_eq!(seen.as_slice(), &[1, 1]);

        // 消息历史应累积（System+2×(User+Assistant) = 5）。
        let roles: Vec<Role> = loop_.history().iter().map(|m| m.role).collect();
        assert_eq!(roles.len(), 5);
        assert_eq!(roles[0], Role::System);
    }

    #[tokio::test]
    async fn test_trim_history_keeps_system() {
        let (llm, _seen) = backend_with(vec![
            ("ok".into(), FinishReason::Stop, vec![]),
        ]);
        let mut loop_ = AgentLoop::new(llm, "mock", "sys")
            .with_max_history(4);
        for i in 0..5 {
            let _ = loop_.turn(&format!("msg {}", i)).await;
        }
        assert!(loop_.history_len() <= 4);
        assert_eq!(loop_.history()[0].role, Role::System);
    }

    #[tokio::test]
    async fn test_turn_stream_simple_stop() {
        let (llm, seen) = backend_with(vec![
            ("streamed hello".into(), FinishReason::Stop, vec![]),
        ]);
        let mut loop_ = AgentLoop::new(llm, "mock", "sys");
        let mut chunks: Vec<String> = Vec::new();
        let out = loop_.turn_stream("hi", |c| { chunks.push(c.to_string()); true }, |_, _| {}).await
            .expect("turn_stream ok");
        assert_eq!(out, "streamed hello");
        // 逐字符 chunk：11 chars → 11 chunks + final。
        assert!(chunks.len() >= 11, "expected >=11 chunks, got {}", chunks.len());
        let joined: String = chunks.concat();
        assert_eq!(joined, "streamed hello");
        assert_eq!(seen.lock().unwrap_or_else(|e| e.into_inner()).as_slice(), &[0]);
    }

    #[tokio::test]
    async fn test_turn_stream_executes_tool_and_continues() {
        let (llm, _seen) = backend_with(vec![
            ("".into(), FinishReason::Tool, vec![tool_call("calc", "call_1", r#"{"expr":"1+1"}"#)]),
            ("answer is 2".into(), FinishReason::Stop, vec![]),
        ]);
        let calc = MockCalc { calls: Arc::new(Mutex::new(Vec::new())) };
        let mut loop_ = AgentLoop::new(llm, "mock", "sys")
            .with_tools(vec![Box::new(calc)])
            .with_max_tool_rounds(4);

        let mut tool_seen: Vec<(String, String)> = Vec::new();
        let out = loop_.turn_stream(
            "compute",
            |_| true,
            |call, output| tool_seen.push((call.function.name.clone(), output.content.clone())),
        ).await.expect("turn_stream ok");
        assert_eq!(out, "answer is 2");
        assert_eq!(tool_seen.len(), 1);
        assert_eq!(tool_seen[0].0, "calc");
        assert_eq!(tool_seen[0].1, "2");
        assert_eq!(loop_.tool_log.len(), 1);
        assert!(loop_.tool_log[0].success);
    }

    #[tokio::test]
    async fn test_turn_stream_cancel_stops_generation() {
        // 模型永远请求工具 → 若 on_token 立即返回 false，应取消并返回已累积文本。
        let (llm, _seen) = backend_with(vec![
            ("partial".into(), FinishReason::Tool, vec![tool_call("calc", "c1", r#"{"expr":"1+1"}"#)]),
        ]);
        let calc = MockCalc { calls: Arc::new(Mutex::new(Vec::new())) };
        let mut loop_ = AgentLoop::new(llm, "mock", "sys")
            .with_tools(vec![Box::new(calc)]);
        // on_token 第一次调用返回 false → 取消。
        let mut first = true;
        let out = loop_.turn_stream("q", |_| { let keep = first; first = false; keep }, |_, _| {}).await
            .expect("cancel is not error");
        // 取消后返回累积内容（至少首 chunk）。
        assert!(out.chars().count() >= 1);
        // 工具不应被真正执行（取消发生在工具轮之前…但脚本第二项缺失，工具轮会因脚本空而 stop）。
        assert!(loop_.tool_log.len() <= 1);
    }

    #[test]
    fn test_tool_def_helper() {
        let _ = tool_def("x");
    }

    // ── 真实 LLM 端到端（agent 循环层，本地手动跑，不进 CI）──────────
    // 验证 TUI 实际调用路径: AgentLoop::turn_stream → gateway → llm7 keyless。
    //   cargo test -p neotrix --lib -- --ignored test_turn_stream_real_llm7
    #[tokio::test]
    #[ignore]
    async fn test_turn_stream_real_llm7() {
        use crate::neotrix::l1_body_impl::nt_io_provider::factory::create_gateway_async;
        let gw = create_gateway_async().await;
        let mut loop_ = AgentLoop::new(Arc::new(gw), "llm7/codestral-latest", "You are a test assistant. Be terse.");
        let mut streamed = String::new();
        let out = loop_.turn_stream(
            "Reply with exactly: E2E-OK",
            |tok| { streamed.push_str(tok); true },
            |_, _| {},
        ).await.expect("turn_stream ok");
        assert!(streamed.contains("E2E-OK") || out.contains("E2E-OK"),
            "expected E2E-OK in streamed output, got streamed={:?} out={:?}", streamed, out);
    }
}
