//! Domain Tauri Commands — 统一入口桥接
//!
//! 将 DomainRegistry 暴露为 Tauri commands。

use crate::domain::{DomainCall, DomainInfo, DomainRegistry};
use crate::ipc::{self, IpcError, IpcResponse};
use std::sync::Arc;
use tauri::{command, State};
use tokio::sync::RwLock;

/// 域注册表状态
pub type DomainState = Arc<RwLock<DomainRegistry>>;

/// 统一域调用 — 前端唯一入口
#[command]
pub async fn domain_call(
    state: State<'_, DomainState>,
    domain: String,
    action: String,
    args: serde_json::Value,
) -> IpcResponse<serde_json::Value> {
    let registry = state.read().await;
    let request = DomainCall {
        domain,
        action,
        args,
    };
    let resp = registry.call(request).await;
    if resp.ok {
        IpcResponse::success(resp.data)
    } else {
        let err = resp.error.unwrap_or_else(|| crate::domain::DomainError {
            code: "UNKNOWN".into(),
            message: "Unknown domain error".into(),
            recoverable: true,
        });
        IpcResponse {
            ok: false,
            error: Some(IpcError::new(err.code, err.message)),
            data: None,
        }
    }
}

/// 列出所有已注册域
#[command]
pub async fn domain_list(state: State<'_, DomainState>) -> IpcResponse<Vec<DomainInfo>> {
    let registry = state.read().await;
    ipc::ok(registry.list())
}

/// 检查域是否存在
#[command]
pub async fn domain_has(state: State<'_, DomainState>, domain: String) -> IpcResponse<bool> {
    let registry = state.read().await;
    ipc::ok(registry.has_domain(&domain))
}

/// 获取域 action 数量
#[command]
pub async fn domain_action_count(
    state: State<'_, DomainState>,
    domain: String,
) -> IpcResponse<usize> {
    let registry = state.read().await;
    if registry.has_domain(&domain) {
        let info = registry.list().into_iter().find(|i| i.name == domain);
        ipc::ok(info.map(|i| i.actions.len()).unwrap_or(0))
    } else {
        ipc::err("DOMAIN_NOT_FOUND", format!("Domain '{}' not found", domain))
    }
}
