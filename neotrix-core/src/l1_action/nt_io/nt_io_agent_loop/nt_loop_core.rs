//! NT-IO AgentLoop core — 主循环（纯搬移，行为零变更）。
//!
//! 内容：`turn`（非流式）+ `turn_stream`（流式）+
//! `turn_stream_with_approval`（带审批门槛流式）三长函数。

use serde_json::Value;

use super::nt_loop_types::{AgentLoop, ToolInvocation};
use crate::l0_substrate::nt_core_traits::ToolOutput;
use crate::l1_action::nt_io::nt_io_provider::types::{FinishReason, LlmError, Message, Role, ToolCallInfo};
use crate::l1_action::nt_action_facade::PendingAction;

impl AgentLoop {
    /// 执行一轮对话：用户输入 → (可能的多次工具调用) → 最终回答。
    pub async fn turn(&mut self, user_input: &str) -> Result<String, LlmError> {
        let user_input = if let Some(stage) = &self.multimodal {
            // 多模态→文本降维 (pi-deepseek-vision 模式): 目标模型保持 text-only。
            // 意图感知 (agent-vision-toolkit 吸收): 把当前任务意图传给视觉层。
            stage.transform_input_with_intent(user_input, &self.task_intent())
        } else {
            user_input.to_string()
        };
        self.messages.push(Message::new(Role::User, &user_input));
        // P1-B2: 高水位先摘要压缩 (救语义), 再驱逐 (兜底)。
        self.maybe_compact_context().await;
        self.trim_history();

        for _round in 0..self.max_tool_rounds {
            let request = self.build_request();
            let response = self.backend.complete(&request).await?;
            self.last_usage = Some(response.usage.clone());

            match response.finish_reason {
                FinishReason::Tool => {
                    if let Some(calls) = response.tool_calls {
                        if calls.is_empty() {
                            // 模型声明需要工具但没给出调用 → 视为停止，避免死循环。
                            return self.emit_final(&response.content);
                        }
                        // 记录 assistant 的 tool_calls，供 API 语义配对。
                        let assistant_calls: Vec<ToolCallInfo> = calls.clone();
                        let _ = self.execute_tools(&assistant_calls).await;
                        continue;
                    }
                    // finish=Tool 但无 tool_calls → 直接返回已有文本。
                    return self.emit_final(&response.content);
                }
                _ => {
                    return self.emit_final(&response.content);
                }
            }
        }

        // 达到工具轮数上限 — 返回当前上下文摘要作为兜底。
        Err(LlmError::Server(
            "AgentLoop: max_tool_rounds exceeded".to_string(),
        ))
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
        // P1-B2: 高水位先摘要压缩 (救语义), 再驱逐 (兜底)。
        self.maybe_compact_context().await;
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
                        self.last_usage = Some(resp.usage.clone());
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
                    self.messages
                        .push(Message::assistant_with_calls("", assistant_calls));
                    for call in &response_tool_calls {
                        let args: Value =
                            serde_json::from_str(&call.arguments).unwrap_or(Value::Null);
                        let result = self.call_tool(&call.name, &args);
                        if let Ok(output) = &result {
                            on_tool(call, output);
                        }
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
                            name: call.name.clone(),
                            arguments: call.arguments.clone(),
                            success: result.is_ok(),
                            output: content.clone(),
                        });
                        let history_content = self.trim_tool_output(&content);
                        self.messages.push(Message::tool(&history_content, &call.id));
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

        Err(LlmError::Server(
            "AgentLoop: max_tool_rounds exceeded".to_string(),
        ))
    }

    /// 流式对话轮（带审批门槛版本，P0 权限审批接线）。
    ///
    /// 与 [`turn_stream`] 相同的决策循环，额外支持：
    ///   - `on_tool_start`：工具执行前回调 `(name, args)`，返回 `false` 取消本轮生成；
    ///   - `on_tool`：工具执行后回调 `(name, args, result, duration_ms, success)`，
    ///     返回 `false` 取消本轮生成（参考 nt_io_neocodex.rs `react_loop_stream` 签名）；
    ///   - `on_approval`：审批回调。`Some(cb)` 时启用审批门槛：每个工具执行前经
    ///     `crate::l1_action::nt_action_facade::nt_approval::global_approval()` 检查，`require_approval` 为 true 则
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
        // P1-B2: 高水位先摘要压缩 (救语义), 再驱逐 (兜底)。
        self.maybe_compact_context().await;
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
                        self.last_usage = Some(resp.usage.clone());
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
                    self.messages
                        .push(Message::assistant_with_calls("", assistant_calls));
                    for call in &response_tool_calls {
                        let args: Value =
                            serde_json::from_str(&call.arguments).unwrap_or(Value::Null);
                        let name = call.name.clone();
                        let args_str = call.arguments.clone();

                        // P0 审批门槛：需审批且被拒绝 → 跳过工具，模型收到明确错误。
                        if let Err(approval_err) =
                            Self::check_tool_approval(&name, &args, on_approval.as_deref())
                        {
                            let content = format!("TOOL_ERROR: {}", approval_err);
                            on_tool(&name, &args_str, &content, 0, false);
                            self.tool_log.push(ToolInvocation {
                                name: name.clone(),
                                arguments: args_str.clone(),
                                success: false,
                                output: content.clone(),
                            });
                            let history_content = self.trim_tool_output(&content);
                            self.messages.push(Message::tool(&history_content, &call.id));
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
                                if *success {
                                    content.clone()
                                } else {
                                    format!("TOOL_ERROR: {}", content)
                                },
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
                        let history_content = self.trim_tool_output(&content);
                        self.messages.push(Message::tool(&history_content, &call.id));
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

        Err(LlmError::Server(
            "AgentLoop: max_tool_rounds exceeded".to_string(),
        ))
    }
}
