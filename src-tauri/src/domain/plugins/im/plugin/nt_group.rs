//! IM 群组/机器人管理（`nt_group`）——纯搬移，零行为变更。
//!
//! 内容来源：`plugin.rs` 原文件切片：身份/机器人间消息/会话 helpers＋机器人与群组动作＋群组调用臂。

use super::super::types::*;
use super::nt_connection::ImPlugin;
use crate::domain::app_handle::get_app_handle;
use crate::domain::{serde_json, ActionSpec, DomainError, ParamSpec};
use std::collections::HashSet;
use tauri::Emitter;

impl ImPlugin {
    /// 获取全局用户 ID（五层身份模型）
    pub(crate) async fn get_global_user_id(&self, channel: &ChannelType, channel_user_id: &str) -> String {
        let key = format!("{}:{}", channel, channel_user_id);
        let mut map = self.identity_map.lock().await;

        map.get(&key).cloned().unwrap_or_else(|| {
            let short_id = uuid::Uuid::new_v4().to_string();
            let global_id = format!("user-{}", &short_id[..8]);
            map.insert(key, global_id.clone());
            global_id
        })
    }

    /// 发送机器人间消息
    pub(crate) async fn send_bot_message(&self, message: BotMessage) -> Result<(), DomainError> {
        let mut queue = self.bot_message_queue.lock().await;
        queue.push(message);
        Ok(())
    }

    /// 接收机器人间消息
    pub(crate) async fn receive_bot_messages(&self, bot_id: &str) -> Vec<BotMessage> {
        let mut queue = self.bot_message_queue.lock().await;
        let mut received = Vec::new();
        let mut remaining = Vec::new();

        for msg in queue.drain(..) {
            if msg.to_bot_id == bot_id {
                received.push(msg);
            } else {
                remaining.push(msg);
            }
        }

        queue.extend(remaining);
        received
    }

    /// 生成会话 ID（带渠道前缀）
    pub(crate) fn make_session_id(&self, channel: &ChannelType, bot_id: &str, chat_id: &str) -> String {
        format!("{}:{}:{}", channel, bot_id, chat_id)
    }

}

impl ImPlugin {
    pub(crate) fn actions_group(&self) -> Vec<ActionSpec> {
        vec![
            // 机器人管理
            ActionSpec {
                name: "add_bot".into(),
                description: "添加机器人".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "name".into(),
                        r#type: "string".into(),
                        description: "机器人名称".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "credential_type".into(),
                        r#type: "string".into(),
                        description: "凭证类型".into(),
                        optional: true,
                    },
                    ParamSpec {
                        name: "workspace".into(),
                        r#type: "string".into(),
                        description: "工作空间".into(),
                        optional: true,
                    },
                    ParamSpec {
                        name: "model".into(),
                        r#type: "string".into(),
                        description: "模型标识".into(),
                        optional: true,
                    },
                ],
                returns: "BotConfig".into(),
            },
            ActionSpec {
                name: "remove_bot".into(),
                description: "删除机器人".into(),
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
                ],
                returns: "bool".into(),
            },
            ActionSpec {
                name: "update_bot".into(),
                description: "更新机器人配置".into(),
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
                        name: "name".into(),
                        r#type: "string".into(),
                        description: "机器人名称".into(),
                        optional: true,
                    },
                    ParamSpec {
                        name: "workspace".into(),
                        r#type: "string".into(),
                        description: "工作空间".into(),
                        optional: true,
                    },
                    ParamSpec {
                        name: "model".into(),
                        r#type: "string".into(),
                        description: "模型标识".into(),
                        optional: true,
                    },
                ],
                returns: "BotConfig".into(),
            },
            ActionSpec {
                name: "set_response_mode".into(),
                description: "设置机器人响应模式".into(),
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
                        name: "mode".into(),
                        r#type: "string".into(),
                        description: "响应模式".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "keyword".into(),
                        r#type: "string".into(),
                        description: "触发关键词".into(),
                        optional: true,
                    },
                ],
                returns: "BotConfig".into(),
            },
            ActionSpec {
                name: "add_whitelist".into(),
                description: "添加白名单用户".into(),
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
                        name: "user_id".into(),
                        r#type: "string".into(),
                        description: "用户 ID".into(),
                        optional: false,
                    },
                ],
                returns: "BotConfig".into(),
            },
            ActionSpec {
                name: "remove_whitelist".into(),
                description: "移除白名单用户".into(),
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
                        name: "user_id".into(),
                        r#type: "string".into(),
                        description: "用户 ID".into(),
                        optional: false,
                    },
                ],
                returns: "BotConfig".into(),
            },
            // 功能配置
            ActionSpec {
                name: "set_context_enhancement".into(),
                description: "设置上下文增强".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "enabled".into(),
                        r#type: "bool".into(),
                        description: "是否启用".into(),
                        optional: false,
                    },
                ],
                returns: "ChannelConfig".into(),
            },
            ActionSpec {
                name: "set_proactive_delivery".into(),
                description: "设置主动投递".into(),
                params: vec![
                    ParamSpec {
                        name: "channel".into(),
                        r#type: "string".into(),
                        description: "渠道标识".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "enabled".into(),
                        r#type: "bool".into(),
                        description: "是否启用".into(),
                        optional: false,
                    },
                ],
                returns: "ChannelConfig".into(),
            },
            // 会话路由
            ActionSpec {
                name: "make_session_id".into(),
                description: "生成会话 ID（带渠道前缀）".into(),
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
                ],
                returns: "String".into(),
            },
            ActionSpec {
                name: "parse_session_id".into(),
                description: "解析会话 ID".into(),
                params: vec![ParamSpec {
                    name: "session_id".into(),
                    r#type: "string".into(),
                    description: "会话 ID".into(),
                    optional: false,
                }],
                returns: "SessionChannelPrefix".into(),
            },
        ]
    }

    pub(crate) fn handles_group(action: &str) -> bool {
        matches!(
            action,
            "add_bot"
                | "remove_bot"
                | "update_bot"
                | "set_response_mode"
                | "add_whitelist"
                | "remove_whitelist"
                | "set_context_enhancement"
                | "set_proactive_delivery"
                | "make_session_id"
                | "parse_session_id"
                | "resolve_identity"
                | "send_bot_message"
                | "receive_bot_messages"
        )
    }

    pub(crate) async fn call_group(
        &self,
        action: &str,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            // 机器人管理
            "add_bot" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let name =
                    args.get("name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 name 参数".into(),
                            recoverable: true,
                        })?;
                let credential_type = args
                    .get("credential_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("token");
                let workspace = args
                    .get("workspace")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let model = args.get("model").and_then(|v| v.as_str()).map(String::from);

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let bot_id = format!(
                    "bot-{}-{}",
                    channel_type,
                    &uuid::Uuid::new_v4().to_string()[..8]
                );

                let bot = BotConfig {
                    id: bot_id,
                    channel: channel_type.clone(),
                    name: name.to_string(),
                    credential_type: credential_type.to_string(),
                    workspace,
                    model,
                    enabled: true,
                    created_at: chrono::Utc::now().timestamp() as u64,
                    response_mode: ResponseMode::default(),
                    whitelist: HashSet::new(),
                };

                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        ch.bots.push(bot.clone());
                        let out = serde_json::json!(bot);
                        self.save_channels(&channels)?;
                        return Ok(out);
                    }
                }
                Err(DomainError {
                    code: "NOT_FOUND".into(),
                    message: format!("渠道不存在: {}", channel),
                    recoverable: true,
                })
            }

            "remove_bot" => {
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
                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;
                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        let original_len = ch.bots.len();
                        ch.bots.retain(|b| b.id != bot_id);
                        if ch.bots.len() < original_len {
                            self.save_channels(&channels)?;
                            return Ok(serde_json::json!(true));
                        }
                        return Ok(serde_json::json!(false));
                    }
                }
                Ok(serde_json::json!(false))
            }

            "update_bot" => {
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
                let name = args.get("name").and_then(|v| v.as_str()).map(String::from);
                let workspace = args
                    .get("workspace")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let model = args.get("model").and_then(|v| v.as_str()).map(String::from);

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        for bot in &mut ch.bots {
                            if bot.id == bot_id {
                                if let Some(n) = name {
                                    bot.name = n;
                                }
                                if let Some(w) = workspace {
                                    bot.workspace = Some(w);
                                }
                                if let Some(m) = model {
                                    bot.model = Some(m);
                                }
                                let out = serde_json::json!(&*bot);
                                self.save_channels(&channels)?;
                                return Ok(out);
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

            "set_response_mode" => {
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
                let mode =
                    args.get("mode")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 mode 参数".into(),
                            recoverable: true,
                        })?;
                let keyword = args
                    .get("keyword")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let response_mode = match mode {
                    "group_invite" => ResponseMode::GroupInvite,
                    "group_keyword" => ResponseMode::GroupKeyword {
                        keyword: keyword.unwrap_or_default(),
                    },
                    "group_all" => ResponseMode::GroupAll,
                    "private" => ResponseMode::Private,
                    _ => {
                        return Err(DomainError {
                            code: "INVALID_MODE".into(),
                            message: format!("未知响应模式: {}", mode),
                            recoverable: true,
                        })
                    }
                };

                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        for bot in &mut ch.bots {
                            if bot.id == bot_id {
                                bot.response_mode = response_mode;
                                let out = serde_json::json!(&*bot);
                                self.save_channels(&channels)?;
                                return Ok(out);
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

            "add_whitelist" => {
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
                let user_id = args
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 user_id 参数".into(),
                        recoverable: true,
                    })?;

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        for bot in &mut ch.bots {
                            if bot.id == bot_id {
                                bot.whitelist.insert(user_id.to_string());
                                let out = serde_json::json!(&*bot);
                                self.save_channels(&channels)?;
                                return Ok(out);
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

            "remove_whitelist" => {
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
                let user_id = args
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 user_id 参数".into(),
                        recoverable: true,
                    })?;

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        for bot in &mut ch.bots {
                            if bot.id == bot_id {
                                bot.whitelist.remove(user_id);
                                let out = serde_json::json!(&*bot);
                                self.save_channels(&channels)?;
                                return Ok(out);
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

            // 功能配置
            "set_context_enhancement" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let enabled = args
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 enabled 参数".into(),
                        recoverable: true,
                    })?;
                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;
                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        ch.context_enhancement = enabled;
                        let out = serde_json::json!(&*ch);
                        self.save_channels(&channels)?;
                        return Ok(out);
                    }
                }
                Err(DomainError {
                    code: "NOT_FOUND".into(),
                    message: format!("渠道不存在: {}", channel),
                    recoverable: true,
                })
            }

            "set_proactive_delivery" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let enabled = args
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 enabled 参数".into(),
                        recoverable: true,
                    })?;
                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;
                let mut channels = self.load_channels()?;
                for ch in &mut channels {
                    if ch.channel == channel_type {
                        ch.proactive_delivery = enabled;
                        let out = serde_json::json!(&*ch);
                        self.save_channels(&channels)?;
                        return Ok(out);
                    }
                }
                Err(DomainError {
                    code: "NOT_FOUND".into(),
                    message: format!("渠道不存在: {}", channel),
                    recoverable: true,
                })
            }

            // 会话路由
            "make_session_id" => {
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

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let session_id = self.make_session_id(&channel_type, bot_id, chat_id);
                Ok(serde_json::json!(session_id))
            }

            "parse_session_id" => {
                let session_id =
                    args.get("session_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 session_id 参数".into(),
                            recoverable: true,
                        })?;

                SessionChannelPrefix::from_session_id(session_id)
                    .map(|prefix| serde_json::json!(prefix))
                    .ok_or_else(|| DomainError {
                        code: "INVALID_SESSION_ID".into(),
                        message: format!("无法解析会话 ID: {}", session_id),
                        recoverable: true,
                    })
            }

            // ═══════════════════════════════════════════════
            // Five Identity Layers (from Multi-Channel Architecture)
            // ═══════════════════════════════════════════════
            "resolve_identity" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let channel_user_id = args
                    .get("channel_user_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel_user_id 参数".into(),
                        recoverable: true,
                    })?;

                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;

                let global_user_id = self.get_global_user_id(&channel_type, channel_user_id).await;

                let identity = IdentityLayers {
                    channel_user_id: channel_user_id.to_string(),
                    global_user_id,
                    tenant_id: "default".to_string(),
                    session_id: format!("{}:{}", channel, channel_user_id),
                    roles: vec!["user".to_string()],
                };

                Ok(serde_json::json!(identity))
            }

            // ═══════════════════════════════════════════════
            // Bot-to-Bot Communication (from Grok Bot)
            // ═══════════════════════════════════════════════
            "send_bot_message" => {
                let from_bot_id = args
                    .get("from_bot_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 from_bot_id 参数".into(),
                        recoverable: true,
                    })?;
                let to_bot_id =
                    args.get("to_bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 to_bot_id 参数".into(),
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
                let message_type = args
                    .get("message_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 message_type 参数".into(),
                        recoverable: true,
                    })?;
                let context = args.get("context").cloned();

                let msg_type = match message_type {
                    "task_delegation" => BotMessageType::TaskDelegation,
                    "context_share" => BotMessageType::ContextShare,
                    "status_update" => BotMessageType::StatusUpdate,
                    "result_return" => BotMessageType::ResultReturn,
                    "heartbeat" => BotMessageType::Heartbeat,
                    _ => {
                        return Err(DomainError {
                            code: "INVALID_MESSAGE_TYPE".into(),
                            message: format!("未知消息类型: {}", message_type),
                            recoverable: true,
                        })
                    }
                };

                let message = BotMessage {
                    from_bot_id: from_bot_id.to_string(),
                    to_bot_id: to_bot_id.to_string(),
                    content: content.to_string(),
                    message_type: msg_type,
                    context,
                    timestamp: chrono::Utc::now().timestamp() as u64,
                };

                self.send_bot_message(message.clone()).await?;

                // 发射事件到前端
                if let Some(app) = get_app_handle() {
                    if let Err(e) = app.emit(
                        "im_bot_message_sent",
                        serde_json::json!({
                            "message": message,
                        }),
                    ) {
                        tracing::warn!("Failed to emit im_bot_message_sent: {e}");
                    }
                }

                Ok(serde_json::json!(message))
            }

            "receive_bot_messages" => {
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;

                let messages = self.receive_bot_messages(bot_id).await;
                Ok(serde_json::json!(messages))
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("未知 action: {}", action),
                recoverable: true,
            }),
        }
    }
}

