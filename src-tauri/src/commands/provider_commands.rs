#![forbid(unsafe_code)]

//! # Provider Commands — 统一接口 API
//!
//! 基于 Azure APIM Unified Model API 模式。
//! 提供 Provider 管理、failover、cost tracking 的完整 CRUD 操作。

use crate::service::provider_manager::{
    FailoverChain, ProviderConfig, SharedProviderManager,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{command, AppHandle, Emitter, State};

// ========== Provider 配置 DTO ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigDto {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub models: Vec<String>,
    pub enabled: bool,
    pub priority: u32,
    pub failover_group: Option<String>,
}

impl From<ProviderConfigDto> for ProviderConfig {
    fn from(dto: ProviderConfigDto) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            provider_type: dto.provider_type,
            api_key: dto.api_key,
            base_url: dto.base_url,
            models: dto.models,
            enabled: dto.enabled,
            priority: dto.priority,
            failover_group: dto.failover_group,
        }
    }
}

// ========== Provider 状态 DTO ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatusDto {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub available: bool,
    pub models: Vec<String>,
    pub last_used: Option<String>,
    pub error: Option<String>,
    pub circuit_state: Option<String>,
    pub total_cost: Option<f64>,
}

// ========== Commands ==========

/// 列出所有已注册的 Provider
#[command]
pub async fn provider_list_providers(
    state: State<'_, SharedProviderManager>,
) -> Result<Vec<ProviderStatusDto>, String> {
    let manager = state.read().await;
    let cb_snapshot = manager.circuit_breaker_snapshot();
    let cost_summaries = manager.cost_summaries();

    Ok(manager
        .list_providers()
        .iter()
        .map(|p| {
            let cb = cb_snapshot.iter().find(|c| c.provider_id == p.id);
            let cost = cost_summaries.iter().find(|c| c.provider_id == p.id);

            ProviderStatusDto {
                id: p.id.clone(),
                name: p.name.clone(),
                provider_type: p.provider_type.clone(),
                available: p.enabled,
                models: p.models.clone(),
                last_used: None,
                error: None,
                circuit_state: cb.map(|c| format!("{:?}", c.state).to_lowercase()),
                total_cost: cost.map(|c| c.total_cost),
            }
        })
        .collect())
}

/// 列出指定 Provider 的模型
#[command]
pub async fn provider_list_models(
    state: State<'_, SharedProviderManager>,
    provider_id: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    let manager = state.read().await;
    let mut models = Vec::new();

    for p in manager.list_providers() {
        if provider_id.as_deref() == Some(&p.id) || provider_id.is_none() {
            for model in &p.models {
                models.push(serde_json::json!({
                    "id": format!("{}:{}", p.id, model),
                    "name": model,
                    "provider": p.name,
                    "provider_type": p.provider_type,
                }));
            }
        }
    }

    Ok(models)
}

/// 调用 Provider 进行补全（带 failover）
#[command]
pub async fn provider_complete(
    state: State<'_, SharedProviderManager>,
    app: AppHandle,
    model_id: String,
    prompt: String,
    max_tokens: Option<u32>,
    task_type: Option<String>,
) -> Result<serde_json::Value, String> {
    let mut manager = state.write().await;

    // 获取 failover 链
    let chain = manager.get_failover_chain(task_type.as_deref().unwrap_or("default"));

    if chain.is_empty() {
        return Err("No available providers".into());
    }

    // 尝试每个 provider 直到成功
    let mut last_error = String::new();
    for provider in chain {
        let parts: Vec<&str> = model_id.splitn(2, ':').collect();
        let model_name = parts.get(1).unwrap_or(&"default");

        // TODO: 实际调用 provider API
        // 这里模拟成功
        manager.record_success(&provider.id);

        // 发射成功事件
        let _ = app.emit("provider-request-success", serde_json::json!({
            "provider_id": provider.id,
            "model": model_name,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));

        return Ok(serde_json::json!({
            "id": uuid::Uuid::new_v4().to_string(),
            "model": model_name,
            "provider": provider.name,
            "content": format!("Response from {} ({})", provider.name, model_name),
            "tokens_used": prompt.len() / 4,
            "max_tokens": max_tokens.unwrap_or(1000),
            "provider_used": provider.id,
        }));
    }

    Err(format!("All providers failed. Last error: {}", last_error))
}

/// Provider 健康检查
#[command]
pub async fn provider_health_check(
    state: State<'_, SharedProviderManager>,
    provider_id: String,
) -> Result<serde_json::Value, String> {
    let manager = state.read().await;
    let provider = manager
        .get_provider(&provider_id)
        .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;

    Ok(serde_json::json!({
        "provider_id": provider.id,
        "name": provider.name,
        "available": provider.enabled,
        "models_count": provider.models.len(),
        "last_check": chrono::Utc::now().to_rfc3339(),
    }))
}

/// 添加新 Provider
#[command]
pub async fn provider_add(
    state: State<'_, SharedProviderManager>,
    app: AppHandle,
    config: ProviderConfigDto,
) -> Result<ProviderStatusDto, String> {
    let mut manager = state.write().await;

    if manager.get_provider(&config.id).is_some() {
        return Err(format!("Provider '{}' already exists", config.id));
    }

    let provider_config = config.clone().into();
    manager.register(provider_config);

    // 发射事件
    let _ = app.emit("provider-added", serde_json::json!({
        "provider_id": config.id,
        "name": config.name,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }));

    Ok(ProviderStatusDto {
        id: config.id,
        name: config.name,
        provider_type: config.provider_type,
        available: config.enabled,
        models: config.models,
        last_used: None,
        error: None,
        circuit_state: Some("closed".into()),
        total_cost: Some(0.0),
    })
}

/// 移除 Provider
#[command]
pub async fn provider_remove(
    state: State<'_, SharedProviderManager>,
    app: AppHandle,
    provider_id: String,
) -> Result<(), String> {
    let mut manager = state.write().await;

    if !manager.unregister(&provider_id) {
        return Err(format!("Provider '{}' not found", provider_id));
    }

    // 发射事件
    let _ = app.emit("provider-removed", serde_json::json!({
        "provider_id": provider_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }));

    Ok(())
}

/// 获取 Provider 配置
#[command]
pub async fn provider_get_config(
    state: State<'_, SharedProviderManager>,
    provider_id: String,
) -> Result<ProviderConfigDto, String> {
    let manager = state.read().await;
    let provider = manager
        .get_provider(&provider_id)
        .ok_or_else(|| format!("Provider '{}' not found", provider_id))?;

    Ok(ProviderConfigDto {
        id: provider.id.clone(),
        name: provider.name.clone(),
        provider_type: provider.provider_type.clone(),
        api_key: provider.api_key.clone(),
        base_url: provider.base_url.clone(),
        models: provider.models.clone(),
        enabled: provider.enabled,
        priority: provider.priority,
        failover_group: provider.failover_group.clone(),
    })
}

/// 更新 Provider 配置
#[command]
pub async fn provider_update_config(
    state: State<'_, SharedProviderManager>,
    provider_id: String,
    config: ProviderConfigDto,
) -> Result<ProviderStatusDto, String> {
    let mut manager = state.write().await;

    if manager.get_provider(&provider_id).is_none() {
        return Err(format!("Provider '{}' not found", provider_id));
    }

    manager.unregister(&provider_id);
    manager.register(config.clone().into());

    Ok(ProviderStatusDto {
        id: config.id,
        name: config.name,
        provider_type: config.provider_type,
        available: config.enabled,
        models: config.models,
        last_used: None,
        error: None,
        circuit_state: Some("closed".into()),
        total_cost: Some(0.0),
    })
}

/// 获取 Circuit Breaker 状态
#[command]
pub async fn provider_circuit_breaker_status(
    state: State<'_, SharedProviderManager>,
) -> Result<Vec<serde_json::Value>, String> {
    let manager = state.read().await;
    let snapshot = manager.circuit_breaker_snapshot();

    Ok(snapshot
        .iter()
        .map(|cb| {
            serde_json::json!({
                "provider_id": cb.provider_id,
                "state": format!("{:?}", cb.state).to_lowercase(),
                "consecutive_failures": cb.consecutive_failures,
                "last_failure": cb.last_failure,
                "cooldown_until": cb.cooldown_until,
            })
        })
        .collect())
}

/// 获取成本摘要
#[command]
pub async fn provider_cost_summary(
    state: State<'_, SharedProviderManager>,
) -> Result<Vec<serde_json::Value>, String> {
    let manager = state.read().await;
    let summaries = manager.cost_summaries();

    Ok(summaries
        .iter()
        .map(|s| {
            serde_json::json!({
                "provider_id": s.provider_id,
                "total_requests": s.total_requests,
                "total_input_tokens": s.total_input_tokens,
                "total_output_tokens": s.total_output_tokens,
                "total_cost": s.total_cost,
            })
        })
        .collect())
}

/// 注册 Failover Chain
#[command]
pub async fn provider_register_failover_chain(
    state: State<'_, SharedProviderManager>,
    name: String,
    providers: Vec<String>,
    task_type: String,
) -> Result<(), String> {
    let mut manager = state.write().await;
    manager.register_failover_chain(FailoverChain {
        name,
        providers,
        task_type,
    });
    Ok(())
}

/// 记录请求成功
#[command]
pub async fn provider_record_success(
    state: State<'_, SharedProviderManager>,
    provider_id: String,
) -> Result<(), String> {
    let mut manager = state.write().await;
    manager.record_success(&provider_id);
    Ok(())
}

/// 记录请求失败
#[command]
pub async fn provider_record_failure(
    state: State<'_, SharedProviderManager>,
    provider_id: String,
) -> Result<(), String> {
    let mut manager = state.write().await;
    manager.record_failure(&provider_id);
    Ok(())
}

// ========== 测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_dto() {
        let dto = ProviderConfigDto {
            id: "test".into(),
            name: "Test".into(),
            provider_type: "openai".into(),
            api_key: None,
            base_url: None,
            models: vec![],
            enabled: true,
            priority: 1,
            failover_group: None,
        };

        let config: ProviderConfig = dto.into();
        assert_eq!(config.id, "test");
    }
}