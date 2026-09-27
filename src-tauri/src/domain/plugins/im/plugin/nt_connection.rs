//! IM 连接管理（`nt_connection`）——纯搬移，零行为变更。
//!
//! 内容来源：`plugin.rs` 原文件切片：
//! - `ImPlugin` 结构体＋`new`/`with_config_path`
//! - `base_dir`/`load_channels`/`save_channels`/`default_channels`
//! - `name`/`description`/`init`＋顶层 `actions`/`call` 分发（分发胶水为新增，其余逐行搬移）
//! - 渠道动作：`status`/`list_channels`/`get_channel`/`toggle_channel`

use super::super::types::*;
use crate::atomic_io;
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin, ParamSpec};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub struct ImPlugin {
    config_path: PathBuf,
    timeout_recovery: Arc<tokio::sync::Mutex<HashMap<String, TimeoutRecoveryState>>>,
    timeout_config: TimeoutRecoveryConfig,
    /// 消息处理状态（用于幂等处理）
    message_states: Arc<tokio::sync::Mutex<HashMap<String, MessageProcessingState>>>,
    /// 机器人间消息队列
    bot_message_queue: Arc<tokio::sync::Mutex<Vec<BotMessage>>>,
    /// 工作流模板
    workflow_templates: Arc<tokio::sync::Mutex<HashMap<String, WorkflowTemplate>>>,
    /// 用户身份映射（channel_user_id -> global_user_id）
    identity_map: Arc<tokio::sync::Mutex<HashMap<String, String>>>,
}

impl ImPlugin {
    pub fn new() -> Self {
        let config_path = crate::config::AppConfig::base_dir()
            .map(|h| h.join("im_channels.json"))
            .unwrap_or_else(|| PathBuf::from(".neotrix/im_channels.json"));

        Self::with_config_path(config_path)
    }

    /// 测试/嵌入式可注入配置路径（生产走 new()）。
    pub fn with_config_path(config_path: PathBuf) -> Self {
        Self {
            config_path,
            timeout_recovery: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            timeout_config: TimeoutRecoveryConfig::default(),
            message_states: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            bot_message_queue: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            workflow_templates: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            identity_map: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    pub(crate) fn base_dir(&self) -> std::path::PathBuf {
        self.config_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from(".neotrix"))
    }

    pub(crate) fn load_channels(&self) -> Result<Vec<ChannelConfig>, DomainError> {
        crate::domain::im_service::load_channels(&self.base_dir()).map_err(|e| DomainError {
            code: "CONFIG_READ_ERROR".into(),
            message: format!("读取 IM 配置失败: {}", e),
            recoverable: true,
        })
    }

    pub(crate) fn save_channels(&self, channels: &[ChannelConfig]) -> Result<(), DomainError> {
        crate::domain::im_service::save_channels(&self.base_dir(), channels).map_err(|e| {
            DomainError {
                code: "CONFIG_WRITE_ERROR".into(),
                message: format!("写入配置失败: {}", e),
                recoverable: true,
            }
        })
    }

    pub(crate) fn default_channels(&self) -> Vec<ChannelConfig> {
        crate::domain::im_service::default_channels()
    }

    pub(crate) fn actions_connection(&self) -> Vec<ActionSpec> {
        vec![
            // 渠道管理
            ActionSpec {
                name: "status".into(),
                description: "获取 IM 系统状态".into(),
                params: vec![],
                returns: "ImStatus".into(),
            },
            ActionSpec {
                name: "list_channels".into(),
                description: "获取所有渠道配置".into(),
                params: vec![],
                returns: "Vec<ChannelConfig>".into(),
            },
            ActionSpec {
                name: "get_channel".into(),
                description: "获取单个渠道配置".into(),
                params: vec![ParamSpec {
                    name: "channel".into(),
                    r#type: "string".into(),
                    description: "渠道标识".into(),
                    optional: false,
                }],
                returns: "ChannelConfig".into(),
            },
            ActionSpec {
                name: "toggle_channel".into(),
                description: "启用/禁用渠道".into(),
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
        ]
    }

    pub(crate) fn handles_connection(action: &str) -> bool {
        matches!(action, "status" | "list_channels" | "get_channel" | "toggle_channel")
    }

    pub(crate) async fn call_connection(
        &self,
        action: &str,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            // 渠道管理
            "status" => {
                let channels = self.load_channels()?;
                let total_bots: usize = channels.iter().map(|c| c.bots.len()).sum();
                let connected_bots: usize = channels
                    .iter()
                    .filter(|c| c.enabled)
                    .map(|c| c.bots.len())
                    .sum();

                Ok(serde_json::json!({
                    "channels": channels,
                    "total_bots": total_bots,
                    "connected_bots": connected_bots,
                    "dsh_market_enabled": false,
                }))
            }

            "list_channels" => {
                let channels = self.load_channels()?;
                Ok(serde_json::json!(channels))
            }

            "get_channel" => {
                let channel = args
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 channel 参数".into(),
                        recoverable: true,
                    })?;
                let channel_type = ChannelType::from_name(channel).ok_or_else(|| DomainError {
                    code: "INVALID_CHANNEL".into(),
                    message: format!("未知渠道: {}", channel),
                    recoverable: true,
                })?;
                let channels = self.load_channels()?;
                channels
                    .into_iter()
                    .find(|c| c.channel == channel_type)
                    .map(|c| serde_json::json!(c))
                    .ok_or_else(|| DomainError {
                        code: "NOT_FOUND".into(),
                        message: format!("渠道不存在: {}", channel),
                        recoverable: true,
                    })
            }

            "toggle_channel" => {
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
                        ch.enabled = enabled;
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
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("未知 action: {}", action),
                recoverable: true,
            }),
        }
    }

}

#[async_trait]
impl DomainPlugin for ImPlugin {
    fn name(&self) -> &str {
        "im"
    }

    fn description(&self) -> &str {
        "IM 多渠道管理：微信、飞书、钉钉、企业微信、QQ、Slack、Telegram、Discord、WhatsApp"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        let mut out = self.actions_connection();
        out.extend(self.actions_group());
        out.extend(self.actions_message());
        out.extend(self.actions_sync());
        out
    }

    #[tracing::instrument(skip(self, args), fields(action = %action))]
    async fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        if Self::handles_connection(action) {
            self.call_connection(action, &args).await
        } else if Self::handles_group(action) {
            self.call_group(action, &args).await
        } else if Self::handles_message(action) {
            self.call_message(action, &args).await
        } else if Self::handles_sync(action) {
            self.call_sync(action, &args).await
        } else {
            Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("未知 action: {}", action),
                recoverable: true,
            })
        }
    }

    async fn init(&mut self) -> Result<(), DomainError> {
        // 初始化配置文件
        if !self.config_path.exists() {
            let channels = self.default_channels();
            self.save_channels(&channels)?;
        }
        Ok(())
    }
}

