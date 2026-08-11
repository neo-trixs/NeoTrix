use axum::{
    extract::{DefaultBodyLimit, Request},
    http::StatusCode,
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

use super::api;
use super::AppState;

const FRONTEND_HTML: &str = include_str!("frontend.html");

pub async fn handle_frontend() -> impl IntoResponse {
    Html(FRONTEND_HTML)
}

pub async fn not_found_handler() -> impl IntoResponse {
    axum::response::Json(serde_json::json!({
        "error": "not_found",
        "message": "Endpoint not found"
    }))
}

async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    if let Some(expected) = &state.api_token {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let provided = auth_header.strip_prefix("Bearer ").unwrap_or("");
        if provided != expected {
            return (StatusCode::UNAUTHORIZED, axum::response::Json(serde_json::json!({
                "error": "unauthorized",
                "message": "Invalid or missing API token. Provide via Authorization: Bearer <token>"
            }))).into_response();
        }
    }
    next.run(req).await
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Brain
        .route("/api/brain/stats", get(api::brain_stats_handler))
        .route("/api/brain/absorb", post(api::absorb_source_handler))
        .route("/api/brain/knowledge/search", get(api::search_knowledge_handler))
        .route("/api/brain/reason", post(api::reason_handler))
        // Sessions
        .route("/api/sessions", get(api::session_list_handler).post(api::session_create_handler))
        .route(
            "/api/sessions/{id}/switch",
            post(api::session_switch_handler),
        )
        .route(
            "/api/sessions/{id}",
            delete(api::session_delete_handler),
        )
        .route("/api/sessions/{id}/fork", post(api::session_fork_handler))
        .route(
            "/api/sessions/{id}/export",
            get(api::session_export_handler),
        )
        .route("/api/sessions/import", post(api::session_import_handler))
        // Agent
        .route(
            "/api/agent/status",
            get(api::agent_status_handler),
        )
        .route("/api/agent/start", post(api::agent_start_handler))
        .route("/api/agent/stop", post(api::agent_stop_handler))
        .route(
            "/api/agent/reason-stream",
            get(api::agent_reason_stream_handler),
        )
        // Project
        .route("/api/project/tree", get(api::file_tree_handler))
        .route("/api/project/file", get(api::read_file_handler).post(api::write_file_handler))
        .route("/api/project/detect", get(api::detect_project_handler))
        // Diff
        .route("/api/diff/staged", get(api::diff_staged_handler))
        .route("/api/diff/unstaged", get(api::diff_unstaged_handler))
        .route("/api/diff/file", get(api::diff_file_handler))
        // Permissions
        .route(
            "/api/permissions/pending",
            get(api::pending_permissions_handler),
        )
        .route(
            "/api/permissions/request",
            post(api::permission_request_handler),
        )
        .route(
            "/api/permissions/{id}/approve",
            post(api::permission_approve_handler),
        )
        .route(
            "/api/permissions/{id}/deny",
            post(api::permission_deny_handler),
        )
        // MCP / provider
        .route(
            "/api/mcp/test-provider",
            post(api::test_provider_handler),
        )
        .route(
            "/api/mcp/save-provider",
            post(api::save_provider_handler),
        )
        .route("/api/mcp/command", post(api::cli_command_handler))
        // Session share (从 server/http.rs 融合)
        .route("/api/sessions/share", post(api::share_create_handler))
        .route(
            "/api/sessions/share/{token}",
            get(api::share_get_handler),
        )
        // H5 远程聊天 (从 server/h5.rs 融合)
        .route("/chat", get(api::h5_page))
        // WebSocket echo (从 server/http.rs ws_handler 融合)
        .route("/ws", get(ws_echo_handler))
        // Frontend + fallback
        .route("/", get(handle_frontend))
        .fallback(not_found_handler)
        // NOTE: auth middleware 不在 build_router 内套用——merge 进来的
        // KB/EWHR 路由不会继承 base router 的 route_layer（axum 语义）。
        // 统一在 start_server_with 对合并后的完整 router 套 auth（见下）。
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .with_state(state)
}

/// WebSocket echo — 从 server/http.rs 拆解融合
pub async fn ws_echo_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(|socket| ws_echo_loop(socket))
}

async fn ws_echo_loop(socket: axum::extract::ws::WebSocket) {
    use axum::extract::ws::{Message};
    use futures::{SinkExt, StreamExt};
    let (mut sender, mut receiver) = socket.split();
    while let Some(msg) = receiver.next().await {
        if let Ok(Message::Text(text)) = msg {
            let echo = format!("echo: {}", text);
            if sender.send(Message::Text(echo.into())).await.is_err() {
                break;
            }
        }
    }
}

pub async fn start_server(port: u16) {
    start_server_with(port,
        Box::new(crate::neotrix::l8_autonomic_impl::nt_mind::ReasoningBrain::new()),
        crate::core::ReasoningBank::new(10000),
    ).await;
}

/// Inner server start — accepts pre-constructed brain and bank
/// to avoid L1→L8 direct dependency. Callers from L8/binaries
/// construct the brain and pass it in.
pub async fn start_server_with(
    port: u16,
    brain: Box<dyn crate::core::nt_core_traits::BrainProvider>,
    bank: crate::core::ReasoningBank,
) {

    // Read api_token from config.toml
    let api_token = std::env::var("NEOTRIX_API_TOKEN").ok().or_else(|| {
        let config_path = dirs::home_dir()
            .unwrap_or_default()
            .join(".config")
            .join("neotrix")
            .join("config.toml");
        std::fs::read_to_string(&config_path).ok().and_then(|content| {
            content.lines().find_map(|line| {
                if line.trim().starts_with("api_token") {
                    let parts: Vec<&str> = line.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        Some(parts[1].trim().trim_matches('"').to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
    });

    // 零信任绑定策略：无 API token 时只绑定回环地址（127.0.0.1），
    // 防止局域网内未鉴权访问推理/知识库 API。配置 token 后才暴露 0.0.0.0。
    let has_token = api_token.is_some();
    let bind_host = if has_token { "0.0.0.0" } else { "127.0.0.1" };

    let state = AppState {
        brain: Arc::new(Mutex::new(brain)),
        bank: Arc::new(Mutex::new(bank)),
        sessions: Arc::new(Mutex::new(vec![SessionInfo {
            id: "default".into(),
            name: "Default Session".into(),
            message_count: 0,
            created: chrono::Utc::now().timestamp(),
        }])),
        permission_counter: Arc::new(AtomicU64::new(1)),
        pending_permissions: Arc::new(Mutex::new(Vec::new())),
        agent_running: Arc::new(Mutex::new(AgentStatus {
            running: false,
            current_task: None,
            uptime_secs: 0,
        })),
        agent_start_time: Arc::new(Mutex::new(None)),
        api_token,
    };

    let mut app = build_router(state.clone());

    // Merge KB API routes if KnowledgeBase can be opened
    if let Some(kb_state) = crate::neotrix::nt_memory_kb::nt_memory_api::KbApiState::try_open_default() {
        let kb_router = crate::neotrix::nt_memory_kb::nt_memory_api::build_kb_router(kb_state);
        app = app.merge(kb_router);
    }

    // Merge EWHR API routes if KB can be opened
    if let Some(ewhr_state) = crate::neotrix::nt_memory_historian::EvidenceApiState::try_open_default() {
        let ewhr_router = crate::neotrix::nt_memory_historian::build_ewhr_router(ewhr_state);
        app = app.merge(ewhr_router);
    }

    // Auth middleware on ALL API routes (含 merge 进来的 KB/EWHR 路由)。
    // 必须在 merge 之后套用——axum 的 route_layer 只作用于当前 router 的路由，
    // 先套再 merge 会导致 KB/EWHR 路由无鉴权（C-1 修复）。
    app = app.route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let addr = format!("{}:{}", bind_host, port);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind address {}: {}", addr, e);
            return;
        }
    };

    println!("╔══════════════════════════════════════════════╗");
    println!("║     NeoTrix Web UI                          ║");
    println!("║     Listening on http://{}               ║", addr);
    if !has_token {
        println!("║     ⚠ 无 API token — 仅绑定回环地址            ║");
    }
    println!("╚══════════════════════════════════════════════╝");

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server failed: {}", e);
    }
}

use super::{AgentStatus, SessionInfo};
