//! IM 同步/超时/DSH 市场/工作流（`nt_sync`）——纯搬移，零行为变更。
//!
//! 内容来源：`plugin.rs` 原文件切片：工作流/超时/DSH helpers＋同步动作＋同步调用臂。

use super::super::types::*;
use super::nt_connection::ImPlugin;
use crate::domain::app_handle::get_app_handle;
use crate::domain::{serde_json, ActionSpec, DomainError, ParamSpec};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::Emitter;

impl ImPlugin {
    /// 保存工作流模板
    pub(crate) async fn save_workflow_template(&self, template: WorkflowTemplate) -> Result<(), DomainError> {
        let mut templates = self.workflow_templates.lock().await;
        templates.insert(template.id.clone(), template);
        Ok(())
    }

    /// 获取工作流模板
    pub(crate) async fn get_workflow_template(&self, template_id: &str) -> Option<WorkflowTemplate> {
        let templates = self.workflow_templates.lock().await;
        templates.get(template_id).cloned()
    }

    /// 列出所有工作流模板
    pub(crate) async fn list_workflow_templates(&self) -> Vec<WorkflowTemplate> {
        let templates = self.workflow_templates.lock().await;
        templates.values().cloned().collect()
    }

    /// 处理超时恢复
    pub(crate) async fn handle_timeout_recovery(&self, bot_id: &str, message_id: &str) -> Result<(), DomainError> {
        let mut state = self.timeout_recovery.lock().await;

        let entry = state
            .entry(message_id.to_string())
            .or_insert_with(|| TimeoutRecoveryState {
                bot_id: bot_id.to_string(),
                message_id: message_id.to_string(),
                retry_count: 0,
                last_attempt: 0,
                status: TimeoutRecoveryStatus::Pending,
            });

        if entry.retry_count >= self.timeout_config.max_retries {
            entry.status = TimeoutRecoveryStatus::Failed;
            return Ok(());
        }

        entry.retry_count += 1;
        entry.status = TimeoutRecoveryStatus::Retrying;
        entry.last_attempt = chrono::Utc::now().timestamp() as u64;

        Ok(())
    }

    /// DSH 市场配置路径
    pub(crate) fn dsh_market_path(&self) -> PathBuf {
        crate::domain::im_service::dsh_market_path()
    }

    /// 加载 DSH 市场配置
    pub(crate) fn load_dsh_market(&self) -> Result<DshMarketConfig, DomainError> {
        crate::domain::im_service::load_dsh_market(&self.base_dir()).map_err(|e| DomainError {
            code: "CONFIG_READ_ERROR".into(),
            message: format!("读取 DSH 市场配置失败: {}", e),
            recoverable: true,
        })
    }

    /// 保存 DSH 市场配置
    pub(crate) fn save_dsh_market(&self, config: &DshMarketConfig) -> Result<(), DomainError> {
        crate::domain::im_service::save_dsh_market(&self.base_dir(), config).map_err(|e| {
            DomainError {
                code: "CONFIG_WRITE_ERROR".into(),
                message: format!("写入 DSH 市场配置失败: {}", e),
                recoverable: true,
            }
        })
    }

}

impl ImPlugin {
    pub(crate) fn actions_sync(&self) -> Vec<ActionSpec> {
        vec![
            // 超时恢复
            ActionSpec {
                name: "timeout_recovery_status".into(),
                description: "获取超时恢复状态".into(),
                params: vec![],
                returns: "Vec<TimeoutRecoveryState>".into(),
            },
            ActionSpec {
                name: "retry_timeout".into(),
                description: "重试超时消息".into(),
                params: vec![ParamSpec {
                    name: "message_id".into(),
                    r#type: "string".into(),
                    description: "消息 ID".into(),
                    optional: false,
                }],
                returns: "TimeoutRecoveryState".into(),
            },
            // DSH 市场
            ActionSpec {
                name: "dsh_market_status".into(),
                description: "获取 DSH 市场配置".into(),
                params: vec![],
                returns: "DshMarketConfig".into(),
            },
            ActionSpec {
                name: "dsh_market_toggle".into(),
                description: "启用/禁用 DSH 市场".into(),
                params: vec![ParamSpec {
                    name: "enabled".into(),
                    r#type: "bool".into(),
                    description: "是否启用".into(),
                    optional: false,
                }],
                returns: "DshMarketConfig".into(),
            },
            ActionSpec {
                name: "dsh_market_config".into(),
                description: "更新 DSH 市场配置".into(),
                params: vec![
                    ParamSpec {
                        name: "api_endpoint".into(),
                        r#type: "string".into(),
                        description: "API 端点".into(),
                        optional: true,
                    },
                    ParamSpec {
                        name: "auth_token".into(),
                        r#type: "string".into(),
                        description: "认证令牌".into(),
                        optional: true,
                    },
                    ParamSpec {
                        name: "sync_enabled".into(),
                        r#type: "bool".into(),
                        description: "是否启用同步".into(),
                        optional: true,
                    },
                ],
                returns: "DshMarketConfig".into(),
            },
            ActionSpec {
                name: "dsh_market_sync".into(),
                description: "从 DSH 市场同步插件".into(),
                params: vec![],
                returns: "HashMap<String, String>".into(),
            },
        ]
    }

    pub(crate) fn handles_sync(action: &str) -> bool {
        matches!(
            action,
            "timeout_recovery_status"
                | "retry_timeout"
                | "dsh_market_status"
                | "dsh_market_toggle"
                | "dsh_market_config"
                | "dsh_market_sync"
                | "save_workflow"
                | "get_workflow"
                | "list_workflows"
                | "execute_workflow"
        )
    }

    pub(crate) async fn call_sync(
        &self,
        action: &str,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            // 超时恢复
            "timeout_recovery_status" => {
                let state = self.timeout_recovery.lock().await;
                let states: Vec<&TimeoutRecoveryState> = state.values().collect();
                Ok(serde_json::json!(states))
            }

            "retry_timeout" => {
                let bot_id =
                    args.get("bot_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 bot_id 参数".into(),
                            recoverable: true,
                        })?;
                let message_id =
                    args.get("message_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 message_id 参数".into(),
                            recoverable: true,
                        })?;

                self.handle_timeout_recovery(bot_id, message_id).await?;

                let state = self.timeout_recovery.lock().await;

                state
                    .get(message_id)
                    .map(|entry| Ok(serde_json::json!(entry)))
                    .unwrap_or_else(|| {
                        Err(DomainError {
                            code: "NOT_FOUND".into(),
                            message: format!("消息不存在: {}", message_id),
                            recoverable: true,
                        })
                    })
            }

            // DSH 市场
            "dsh_market_status" => {
                let config = self.load_dsh_market()?;
                Ok(serde_json::json!(config))
            }

            "dsh_market_toggle" => {
                let enabled = args
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 enabled 参数".into(),
                        recoverable: true,
                    })?;
                let mut config = self.load_dsh_market()?;
                config.enabled = enabled;
                self.save_dsh_market(&config)?;
                Ok(serde_json::json!(config))
            }

            "dsh_market_config" => {
                let mut config = self.load_dsh_market()?;
                if let Some(ep) = args.get("api_endpoint").and_then(|v| v.as_str()) {
                    config.api_endpoint = ep.to_string();
                }
                if let Some(token) = args.get("auth_token").and_then(|v| v.as_str()) {
                    config.auth_token = Some(token.to_string());
                }
                if let Some(sync) = args.get("sync_enabled").and_then(|v| v.as_bool()) {
                    config.sync_enabled = sync;
                }
                self.save_dsh_market(&config)?;
                Ok(serde_json::json!(config))
            }

            "dsh_market_sync" => {
                let config = self.load_dsh_market()?;
                if !config.enabled {
                    return Err(DomainError {
                        code: "DSH_MARKET_DISABLED".into(),
                        message: "DSH market is not enabled".into(),
                        recoverable: true,
                    });
                }

                // TODO: 实现实际的 DSH 市场 API 调用
                // 目前返回模拟数据
                let mut plugins: HashMap<String, String> = HashMap::new();
                plugins.insert("dsh-im-core".into(), "1.0.0".into());
                plugins.insert("dsh-im-wechat".into(), "1.0.0".into());
                plugins.insert("dsh-im-feishu".into(), "1.0.0".into());
                plugins.insert("dsh-im-telegram".into(), "1.0.0".into());

                Ok(serde_json::json!(plugins))
            }

            // ═══════════════════════════════════════════════
            // Workflow Learning (from Grok Bot)
            // ═══════════════════════════════════════════════
            "save_workflow" => {
                let name =
                    args.get("name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 name 参数".into(),
                            recoverable: true,
                        })?;
                let description = args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 description 参数".into(),
                        recoverable: true,
                    })?;
                let steps: Vec<WorkflowStep> = serde_json::from_value(
                    args.get("steps")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                )
                .map_err(|e| DomainError {
                    code: "INVALID_STEPS".into(),
                    message: format!("无效的步骤列表: {}", e),
                    recoverable: true,
                })?;
                let trigger: WorkflowTrigger = serde_json::from_value(
                    args.get("trigger")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                )
                .map_err(|e| DomainError {
                    code: "INVALID_TRIGGER".into(),
                    message: format!("无效的触发条件: {}", e),
                    recoverable: true,
                })?;

                let short_id = uuid::Uuid::new_v4().to_string();
                let template = WorkflowTemplate {
                    id: format!("wf-{}", &short_id[..8]),
                    name: name.to_string(),
                    description: description.to_string(),
                    steps,
                    trigger,
                    created_at: chrono::Utc::now().timestamp() as u64,
                    last_used: None,
                    use_count: 0,
                };

                self.save_workflow_template(template.clone()).await?;

                Ok(serde_json::json!(template))
            }

            "get_workflow" => {
                let workflow_id = args
                    .get("workflow_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 workflow_id 参数".into(),
                        recoverable: true,
                    })?;

                self.get_workflow_template(workflow_id)
                    .await
                    .map(|t| Ok(serde_json::json!(t)))
                    .unwrap_or_else(|| {
                        Err(DomainError {
                            code: "NOT_FOUND".into(),
                            message: format!("工作流不存在: {}", workflow_id),
                            recoverable: true,
                        })
                    })
            }

            "list_workflows" => {
                let templates = self.list_workflow_templates().await;
                Ok(serde_json::json!(templates))
            }

            "execute_workflow" => {
                let workflow_id = args
                    .get("workflow_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 workflow_id 参数".into(),
                        recoverable: true,
                    })?;

                let mut template =
                    self.get_workflow_template(workflow_id)
                        .await
                        .ok_or_else(|| DomainError {
                            code: "NOT_FOUND".into(),
                            message: format!("工作流不存在: {}", workflow_id),
                            recoverable: true,
                        })?;

                // 更新使用统计
                template.last_used = Some(chrono::Utc::now().timestamp() as u64);
                template.use_count += 1;
                self.save_workflow_template(template.clone()).await?;

                // 发射事件到前端
                if let Some(app) = get_app_handle() {
                    if let Err(e) = app.emit(
                        "im_workflow_executed",
                        serde_json::json!({
                            "workflow_id": workflow_id,
                            "workflow_name": template.name,
                        }),
                    ) {
                        tracing::warn!("Failed to emit im_workflow_executed: {e}");
                    }
                }

                Ok(serde_json::json!({
                    "status": "executed",
                    "workflow_id": workflow_id,
                    "workflow_name": template.name,
                    "steps_count": template.steps.len(),
                }))
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("未知 action: {}", action),
                recoverable: true,
            }),
        }
    }
}

