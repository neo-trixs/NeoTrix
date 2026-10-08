//! `nt_gateway` — HTTP 传输壳（对外 JSON API 面）。
//!
//! Module gated: `#[cfg(feature = "gateway-http")]`。默认不编译。
//!
//! v0 翻译层真实层（N6.3 P2 推进）：
//! - `GET  /health`                 → `{ok:true}`
//! - `GET  /v1/models`              → `{object:"list", data:[...]}`（启动时`list_route_groups()` 快照）
//! - `POST /v1/chat/completions`    → OpenAI 形状cextract last user content →
//!   `LocalEchoEngine::run_turn` → 回写 OpenAI `chat.completion` 形状。
//!
//! **只接 echo**：复杂路由（route group / provider name / failover / 流式 SSE）
//! 是显式后置 follow-up —— 不在这里悄悄伪装已经在跑真模型。
//!
//! 总纲见 `docs/architecture/N6-3-GATEWAY-BOUNDARY-2026-10-08.md`。

#![cfg(feature = "gateway-http")]

use axum::response::IntoResponse;
use crate::nt_engine::{EngineAdapter, LocalEchoEngine};
use crate::nt_error::NtBotError;
use crate::nt_store::NeobotStore;

pub struct GatewayConfig {
    pub addr: String,
    pub cfg: crate::NeobotConfig,
    pub store: NeobotStore,
}

struct GatewayState {
    cfg: crate::NeobotConfig,
    store: std::sync::Arc<std::sync::Mutex<NeobotStore>>,
    routes: std::sync::Arc<Vec<serde_json::Value>>,
}

/// 最小启动入口（bin `gateway serve` 调用）。
pub async fn run_server(conf: GatewayConfig) -> Result<(), NtBotError> {
    use axum::{routing::get, Router};

    let routes: Vec<serde_json::Value> = match conf.store.list_route_groups() {
        Ok(routes) => routes
            .into_iter()
            .map(|r| serde_json::json!({"id": r.name, "object": "model", "owned_by": "neotrix"}))
            .collect(),
        Err(err) => {
            eprintln!("[nt_gateway] list_route_groups: {err}");
            Vec::new()
        }
    };

    let state = std::sync::Arc::new(GatewayState {
        cfg: conf.cfg,
        store: std::sync::Arc::new(std::sync::Mutex::new(conf.store)),
        routes: std::sync::Arc::new(routes),
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(models_handler))
        .route(
            "/v1/chat/completions",
            axum::routing::post(chat_completions),
        )
        .with_state(state);

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
    axum::extract::State(state): axum::extract::State<std::sync::Arc<GatewayState>>,
) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::OK,
        axum::Json(serde_json::json!({"object":"list","data":(*state.routes).clone()})),
    )
}

/// `POST /v1/chat/completions` —— v0 真实层：OpenAI 形状 → echo run_turn → chat.completion.
async fn chat_completions(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<GatewayState>>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> impl axum::response::IntoResponse {
    let Some(content) = last_user_content(&body) else {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({
                "error": {"message": "messages[] required, last user content missing", "type": "invalid_request_error", "code": "messages_required"}
            })),
        )
            .into_response();
    };

    let model = body["model"].as_str().unwrap_or("echo");

    let _guard = match state.store.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                axum::Json(serde_json::json!({"error":{"message":"store lock poisoned","type":"upstream_error","code":"store_lock"}})),
            )
                .into_response();
        }
    };

    let memory = crate::nt_memory::memory_for_config(&state.cfg);
    let engine_result = crate::build_engine_by_name(&_guard, model, Some(model), memory);
    let turn_outcome = match engine_result {
        Ok(Some(engine)) => engine.run_turn(&content, &[]),
        Ok(None) => LocalEchoEngine.run_turn(&content, &[]),
        Err(err) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                axum::Json(serde_json::json!({"error":{"message": format!("{err}"), "type":"upstream_error", "code":"engine_build_failed"}})),
            )
                .into_response();
        }
    };
    let turn = match turn_outcome {
        Ok(turn) => turn,
        Err(err) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                axum::Json(serde_json::json!({"error":{"message": format!("{err}"), "type":"upstream_error", "code":"engine_turn_failed"}})),
            )
                .into_response();
        }
    };
    (
        axum::http::StatusCode::OK,
        axum::Json(serde_json::json!({
            "id": format!("chatcmpl-{}", uuid::Uuid::new_v4().simple()),
            "object": "chat.completion",
            "created": chrono::Utc::now().timestamp(),
            "model": model,
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": turn.assistant_text},
                "finish_reason": "stop",
            }],
            "usage": {"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0},
        })),
    )
        .into_response()
}

/// 取 `messages` 里最后一条 `role=user` 的内容；兼容 `content` 为字符串或 parts 数组的形状。
fn last_user_content(body: &serde_json::Value) -> Option<String> {
    let messages = body.get("messages")?.as_array()?;
    for msg in messages.iter().rev() {
        if msg.get("role")?.as_str()? == "user" {
            match msg.get("content")? {
                serde_json::Value::String(s) => return Some(s.clone()),
                serde_json::Value::Array(parts) => {
                    for part in parts.iter() {
                        if part.get("type").and_then(|v| v.as_str()) == Some("text") {
                            if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                                return Some(text.to_owned());
                            }
                        }
                        // 兼容parts直接是string的数组或只有text字段的简形
                        if let Some(text) = part.as_str() {
                            return Some(text.to_owned());
                        }
                    }
                }
                _ => {}
            }
        }
    }
    None
}
