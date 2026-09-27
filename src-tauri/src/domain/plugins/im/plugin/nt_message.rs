//! IM 消息处理（`nt_message`）——纯搬移，零行为变更。
//!
//! 内容来源：`plugin.rs` 原文件切片：幂等/发送/流式/响应策略 helpers＋消息动作＋消息调用臂。

use super::super::types::*;
use super::nt_connection::ImPlugin;
use crate::domain::app_handle::get_app_handle;
use crate::domain::{serde_json, ActionSpec, DomainError, ParamSpec};
use tauri::Emitter;

impl ImPlugin {
    /// 检查消息是否已处理（幂等性检查）
    pub(crate) async fn is_message_processed(&self, message_id: &str) -> bool {
        let states = self.message_states.lock().await;
        states
            .get(message_id)
            .map(|s| s.status == ProcessingStatus::Completed)
            .unwrap_or(false)
    }

    /// 记录消息处理状态
    pub(crate) async fn record_message_state(&self, message_id: &str, status: ProcessingStatus) {
        let mut states = self.message_states.lock().await;
        let now = chrono::Utc::now().timestamp() as u64;

        let state =
            states
                .entry(message_id.to_string())
                .or_insert_with(|| MessageProcessingState {
                    message_id: message_id.to_string(),
                    status: ProcessingStatus::Pending,
                    started_at: now,
                    completed_at: None,
                    result: None,
                    error: None,
                    retry_count: 0,
                });

        state.status = status;
        if state.status == ProcessingStatus::Completed || state.status == ProcessingStatus::Failed {
            state.completed_at = Some(now);
        }
    }

    /// 检查是否应该响应消息
    pub(crate) fn should_respond(&self, bot: &BotConfig, chat_id: &str, sender_id: &str, text: &str) -> bool {
        match &bot.response_mode {
            ResponseMode::Private => {
                // 私聊模式：检查白名单
                if bot.whitelist.is_empty() {
                    true // 白名单为空时允许所有
                } else {
                    bot.whitelist.contains(sender_id)
                }
            }
            ResponseMode::GroupInvite => {
                // 群聊中需要被 @
                text.contains(&format!("@{}", bot.name))
            }
            ResponseMode::GroupKeyword { keyword } => {
                // 群聊中需要关键词触发
                text.contains(keyword)
            }
            ResponseMode::GroupAll => {
                // 群聊中响应所有消息
                true
            }
        }
    }

    /// 发送消息到渠道（模拟）
    #[tracing::instrument(skip(self, content), fields(channel = %channel, bot_id = %bot_id, chat_id = %chat_id))]
    pub(crate) async fn send_to_channel(
        &self,
        channel: &ChannelType,
        bot_id: &str,
        chat_id: &str,
        content: &str,
    ) -> Result<String, DomainError> {
        // 模拟发送消息
        let message_id = format!("msg-{}", chrono::Utc::now().timestamp_millis());

        // 发射事件到前端
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit(
                "im_message_sent",
                serde_json::json!({
                    "channel": channel.to_string(),
                    "bot_id": bot_id,
                    "chat_id": chat_id,
                    "message_id": message_id,
                    "content": content,
                }),
            ) {
                tracing::warn!("Failed to emit im_message_sent: {e}");
            }
        }

        Ok(message_id)
    }

    /// 流式发送消息
    pub(crate) async fn send_streaming(
        &self,
        channel: &ChannelType,
        bot_id: &str,
        chat_id: &str,
        content: &str,
    ) -> Result<String, DomainError> {
        // 模拟流式发送
        let message_id = format!("msg-{}", chrono::Utc::now().timestamp_millis());

        // 发射流式事件
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit(
                "im_stream_start",
                serde_json::json!({
                    "channel": channel.to_string(),
                    "bot_id": bot_id,
                    "chat_id": chat_id,
                    "message_id": message_id,
                }),
            ) {
                tracing::warn!("Failed to emit im_stream_start: {e}");
            }

            // 模拟流式 token
            for token in content.chars() {
                if let Err(e) = app.emit(
                    "im_stream_token",
                    serde_json::json!({
                        "message_id": message_id,
                        "token": token.to_string(),
                    }),
                ) {
                    tracing::warn!("Failed to emit im_stream_token: {e}");
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }

            if let Err(e) = app.emit(
                "im_stream_end",
                serde_json::json!({
                    "message_id": message_id,
                    "content": content,
                }),
            ) {
                tracing::warn!("Failed to emit im_stream_end: {e}");
            }
        }

        Ok(message_id)
    }

}

impl ImPlugin {
    pub(crate) fn actions_message(&self) -> Vec<ActionSpec> {
        vec![
            // 消息处理
            ActionSpec {
                name: "send_message".into(),
                description: "发送消息到渠道".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "bot_id".into(),
                        r#type: "string".into(),
                        description: "机器人 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "chat_id".into(),
                        r#type: "string".into(),
                        description: "聊天会话 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "content".into(),
                        r#type: "string".into(),
                        description: "消息内容".into(),
                        optional: false,
                    },
                ],
                returns: "String".into(),
            },
            ActionSpec {
                name: "send_streaming".into(),
                description: "流式发送消息到渠道".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "bot_id".into(),
                        r#type: "string".into(),
                        description: "机器人 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "chat_id".into(),
                        r#type: "string".into(),
                        description: "聊天会话 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "content".into(),
                        r#type: "string".into(),
                        description: "消息内容".into(),
                        optional: false,
                    },
                ],
                returns: "String".into(),
            },
            ActionSpec {
                name: "should_respond".into(),
                description: "检查是否应该响应消息".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "bot_id".into(),
                        r#type: "string".into(),
                        description: "机器人 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "chat_id".into(),
                        r#type: "string".into(),
                        description: "聊天会话 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "sender_id".into(),
                        r#type: "string".into(),
                        description: "发送者 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "text".into(),
                        r#type: "string".into(),
                        description: "消息文本".into(),
                        optional: false,
                    },
                ],
                returns: "bool".into(),
            },
        ]
    }

    pub(crate) fn handles_message(action: &str) -> bool {
        matches!(
            action,
            "send_message" | "send_streaming" | "should_respond" | "receive_message" | "check_idempotency"
        )
    }

    pub(crate) async fn call_message(
        &self,
        action: &str,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            // 消息处理
            "send_message" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let chat_id = args
                    .get("chat_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 chat_id 参数".into(),
                        recoverable: true,
                    })?;
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 content 参数".into(),
                        recoverable: true,
                    })?;

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let message_id = self.send_to_channel(&channel_type, bot_id, chat_id, content)
                    .await?;

                Ok(serde_json::json!({
                    "message_id": message_id,
                    "channel": channel,
                    "bot_id": bot_id,
                    "chat_id": chat_id,
                }))
            }

            "send_streaming" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let chat_id = args
                    .get("chat_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 chat_id 参数".into(),
                        recoverable: true,
                    })?;
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 content 参数".into(),
                        recoverable: true,
                    })?;

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let message_id = self.send_streaming(&channel_type, bot_id, chat_id, content)
                    .await?;

                Ok(serde_json::json!({
                    "message_id": message_id,
                    "channel": channel,
                    "bot_id": bot_id,
                    "chat_id": chat_id,
                }))
            }

            "should_respond" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let chat_id = args
                    .get("chat_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 chat_id 参数".into(),
                        recoverable: true,
                    })?;
                let sender_id =
                    args.get("sender_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 sender_id 参数".into(),
                            recoverable: true,
                        })?;
                let text =
                    args.get("text")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 text 参数".into(),
                            recoverable: true,
                        })?;

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let channels = self.load_channels()?;
                for ch in &channels {
                    if ch.channel == channel_type {
                        for bot in &ch.bots {
                            if bot.id == bot_id {
                                let should = self.should_respond(bot, chat_id, sender_id, text);
                                return Ok(serde_json::json!(should));
                            }
                        }
                    }
                }
                Err(DomainError {
                    code: "NOT_FOUND".into(),
                    message: format!("机器人不存在: {} / {}", channel, bot_id),
                    recoverable: true,
                })
            }

            // ═══════════════════════════════════════════════
            // Single Inbox Pattern (from Multi-Channel Architecture)
            // ═══════════════════════════════════════════════
            "receive_message" => {
                let envelope: MessageEnvelope = serde_json::from_value(
                    args.get("envelope")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                )
                .map_err(|e| DomainError {
                    code: "INVALID_ENVELOPE".into(),
                    message: format!("无效的消息信封: {}", e),
                    recoverable: true,
                })?;

                // 幂等性检查
                if self.is_message_processed(&envelope.id).await {
                    return Ok(serde_json::json!({
                        "status": "already_processed",
                        "message_id": envelope.id,
                    }));
                }

                // 记录处理状态
                self.record_message_state(&envelope.id, ProcessingStatus::Processing);

                // 发射事件到前端
                if let Some(app) = get_app_handle() {
                    if let Err(e) = app.emit(
                        "im_message_received",
                        serde_json::json!({
                            "envelope": envelope,
                        }),
                    ) {
                        tracing::warn!("Failed to emit im_message_received: {e}");
                    }
                }

                // 标记完成
                self.record_message_state(&envelope.id, ProcessingStatus::Completed);

                Ok(serde_json::json!({
                    "status": "processed",
                    "message_id": envelope.id,
                }))
            }

            "check_idempotency" => {
                let message_id =
                    args.get("message_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 message_id 参数".into(),
                            recoverable: true,
                        })?;

                let processed = self.is_message_processed(message_id).await;
                Ok(serde_json::json!(processed))
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("未知 action: {}", action),
                recoverable: true,
            }),
        }
    }
}

