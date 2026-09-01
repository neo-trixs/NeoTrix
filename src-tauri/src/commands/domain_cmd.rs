//! Domain Tauri Commands — 统一入口桥接
//!
//! 将 DomainRegistry 暴露为 Tauri commands。

use tauri::{command, State};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::domain::{DomainCall, DomainResponse, DomainInfo, DomainRegistry};

/// 域注册表状态
pub type DomainState = Arc<RwLock<DomainRegistry>>;

/// 统一域调用 — 前端唯一入口
#[command]
pub async fn domain_call(
    state: State<'_, DomainState>,
    domain: String,
    action: String,
    args: serde_json::Value,
) -> Result<DomainResponse, String> {
    let registry = state.read().await;
    let request = DomainCall { domain, action, args };
    Ok(registry.call(request))
}

/// 列出所有已注册域
#[command]
pub async fn domain_list(
    state: State<'_, DomainState>,
) -> Result<Vec<DomainInfo>, String> {
    let registry = state.read().await;
    Ok(registry.list())
}

/// 检查域是否存在
#[command]
pub async fn domain_has(
    state: State<'_, DomainState>,
    domain: String,
) -> Result<bool, String> {
    let registry = state.read().await;
    Ok(registry.has_domain(&domain))
}

/// 获取域 action 数量
#[command]
pub async fn domain_action_count(
    state: State<'_, DomainState>,
    domain: String,
) -> Result<usize, String> {
    let registry = state.read().await;
    if registry.has_domain(&domain) {
        let info = registry.list().into_iter().find(|i| i.name == domain);
        Ok(info.map(|i| i.actions.len()).unwrap_or(0))
    } else {
        Err(format!("Domain '{}' not found", domain))
    }
}
