//! `nt_gateway` — HTTP 传输壳（对外 JSON API 面）。
//!
//! Module gated: `#[cfg(feature = "gateway-http")]`。默认不编译。
//!
//! 本期只落「接线壳」，不造概念层（CostGate/FallbackChain/quota 复用 `neotrix-gateway`/neobot 已有路由）：
//! - `GET  /health`                 → `{ok:true}`
//! - `GET  /v1/models`              → `{object:"list", data:[...]}`（`list_route_groups()`）
//! - `POST /v1/chat/completions`    → 501 OpenAI 形状的 `not_implemented_error`
//!
//! 待期：翻译矩阵 6 端点 metà、SSE 粗粒度流式、CostGate 接线、dispatch 进账本。
//!
//! 为什么 501 而不是假实现：翻译层缺路由名与形状校验时**必须如实报不实现**，
//! 伪造 200=「通真等于通不可眼悍横」（Captain 形状错误映射表的同源纪律）。

#![cfg(feature = "gateway-http")]

use crate::nt_error::NtBotError;
use crate::nt_store::NeobotStore;

pub struct GatewayConfig {
    pub addr: String,
    pub store: NeobotStore,
}

/// 最小启动入口（bin `gateway serve` 调用）。
pub async fn run_server(conf: GatewayConfig) -> Result<(), NtBotError> {
    use axum::{routing::get, Router};

    let models: Vec<serde_json::Value> = match conf.store.list_route_groups() {
        Ok(routes) => routes
            .into_iter()
            .map(|r| serde_json::json!({"id": r.name, "object": "model", "owned_by": "neotrix"}))
            .collect(),
        Err(err) => {
            eprintln!("[nt_gateway] list_route_groups: {err}");
            Vec::new()
        }
    };
    let models = std::sync::Arc::new(models);

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(models_handler))
        .route(
            "/v1/chat/completions",
            axum::routing::post(chat_completions),
        )
        .with_state(models);

    let listener = tokio::net::TcpListener::bind(&conf.addr)
        .await
        .map_err(|e| NtBotError::Io(format!("bind {}: {e}", &conf.addr)))?;
    axum::serve(listener, app)
        .await
        .map_err(|e| NtBotError::Io(format!("serve: {e}")))?;
    Ok(())
}

async fn health() -> impl axum::response::IntoResponse {
    axum::Json(serde_json::json!({"ok": true}))
}

/// `GET /v1/models` —— 启动时快照路由组列出系我们有的模型簇。
async fn models_handler(
    axum::extract::State(routes): axum::extract::State<std::sync::Arc<Vec<serde_json::Value>>>,
) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::OK,
        axum::Json(serde_json::json!({"object":"list","data":(*routes).clone()})),
    )
}

/// `POST /v1/chat/completions` —— 本期 501；形状未校验前不伪造响应。
async fn chat_completions() -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::NOT_IMPLEMENTED,
        axum::Json(serde_json::json!({
            "error": {
                "message": "nt_gateway_http is a skeleton: translation layer lands in follow-up PR (N6.3 P2)",
                "type": "not_implemented_error",
                "code": "not_implemented",
            }
        })),
    )
}
