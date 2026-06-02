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
        // Frontend + fallback
        .route("/", get(handle_frontend))
        .fallback(not_found_handler)
        // Auth middleware on API routes
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .with_state(state)
}

pub async fn start_server(port: u16) {
    let brain = crate::neotrix::nt_mind::ReasoningBrain::new();
    let bank = crate::neotrix::nt_mind::ReasoningBank::new(10000);

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

    let mut app = build_router(state);

    // Merge KB API routes if KnowledgeBase can be opened
    if let Some(kb_state) = crate::neotrix::nt_memory_kb::nt_memory_api::KbApiState::try_open_default() {
        let kb_router = crate::neotrix::nt_memory_kb::nt_memory_api::build_kb_router(kb_state);
        app = app.merge(kb_router);
    }

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    println!("╔══════════════════════════════════════════════╗");
    println!("║     NeoTrix Web UI                          ║");
    println!("║     Listening on http://{}               ║", addr);
    println!("╚══════════════════════════════════════════════╝");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

use super::{AgentStatus, SessionInfo};
