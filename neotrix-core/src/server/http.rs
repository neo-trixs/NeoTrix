use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    Router, routing::{get, post}, response::Json, extract::State,
    extract::ws::{WebSocket, WebSocketUpgrade, Message},
    extract::Path,
};
use futures::{SinkExt, StreamExt};
use serde::{Serialize, Deserialize};
use crate::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use crate::neotrix::nt_world_model::TaskType;
use crate::neotrix::nt_io_standalone::format_kernel_output;
use crate::neotrix::nt_core_kernel::{ReasoningKernel, EVOLUTION};
use crate::server::ws::WsBridge;
use crate::server::h5::h5_routes;
use crate::server::session::SessionShareManager;
use crate::cli::tui::session_store::SessionStore;

#[derive(Clone)]
pub struct AppState {
    pub agent: Arc<RwLock<SelfIteratingBrain>>,
    pub bridge: Arc<WsBridge>,
    pub share_manager: SessionShareManager,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: &'static str,
}

#[derive(Deserialize)]
pub struct ReasonRequest { pub prompt: String }

#[derive(Serialize)]
pub struct ReasonResponse {
    pub success: bool,
    pub output: String,
    pub capability_before: f64,
    pub capability_after: f64,
}

#[derive(Deserialize)]
pub struct ReasonKernelRequest {
    pub prompt: String,
    pub stage: Option<usize>,
}

#[derive(Serialize)]
pub struct ReasonKernelResponse {
    pub output: String,
    pub confidence: f64,
    pub stage: usize,
    pub state_dim: usize,
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub iteration: u64,
    pub absorb_count: u64,
    pub capability_sum: f64,
    pub memory_count: usize,
    pub engine_active: bool,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok".to_string(), version: "0.18.0" })
}

async fn stats_handler(State(state): State<AppState>) -> Json<StatsResponse> {
    let agent = state.agent.read().await;
    let stats = agent.brain.get_statistics();
    Json(StatsResponse {
        iteration: agent.iteration,
        absorb_count: agent.brain.total_absorb_count,
        capability_sum: stats.capability_sum,
        memory_count: agent.reasoning_bank.memories().len(),
        engine_active: agent.reasoning_engine.is_some(),
    })
}

async fn reason_handler(State(state): State<AppState>, Json(req): Json<ReasonRequest>) -> Json<ReasonResponse> {
    let mut agent = state.agent.write().await;
    let before = agent.brain.capability.arr.iter().sum::<f64>();
    if let Some(ref mut engine) = agent.reasoning_engine {
        match engine.reason(&req.prompt) {
            Ok(response) => {
                let after = agent.brain.capability.arr.iter().sum::<f64>();
                let _ = agent.brain.save();
                Json(ReasonResponse { success: true, output: response, capability_before: before, capability_after: after })
            }
            Err(e) => Json(ReasonResponse { success: false, output: format!("Error: {}", e), capability_before: before, capability_after: before }),
        }
    } else {
        let result = agent.iterate(TaskType::General);
        Json(ReasonResponse { success: true, output: format!("Evolution: {:.3} → {:.3}", result.score_before, result.score_after), capability_before: before, capability_after: agent.brain.capability.arr.iter().sum::<f64>() })
    }
}

/// WebSocket: 双向实时通信 (参照 cc-haha WsBridge)
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(socket: WebSocket, state: AppState) {
    let (sender, mut receiver) = socket.split();
    let mut sender = sender;
    // 注册 session
    let _ = state.bridge.register_session("ws", "ws-default").await;
    // 消息循环
    while let Some(msg) = receiver.next().await {
        if let Ok(Message::Text(text)) = msg {
            let echo = format!("echo: {}", text);
            let _ = sender.send(Message::Text(echo.into())).await;
        }
    }
}

async fn reason_kernel_handler(Json(req): Json<ReasonKernelRequest>) -> Json<ReasonKernelResponse> {
    let stage = req.stage.unwrap_or(18).min(EVOLUTION.len() - 1);
    let kernel = ReasoningKernel::new(stage);
    let dim = kernel.state.len();
    let query = crate::neotrix::nt_io_standalone::text_to_vector(&req.prompt, dim);
    let output = kernel.reason(&query, None);
    let energy: f64 = output.state_delta.iter().map(|x| x.abs()).sum::<f64>() / dim.max(1) as f64;
    let response = format_kernel_output(&output.state_delta, &req.prompt, &kernel);
    Json(ReasonKernelResponse {
        output: response,
        confidence: energy,
        stage,
        state_dim: dim,
    })
}

// ====== Session Share API ======

#[derive(Deserialize)]
pub struct ShareCreateRequest {
    pub name: String,
    pub ttl_hours: Option<u64>,
}

#[derive(Serialize)]
pub struct ShareCreateResponse {
    pub token: String,
    pub url: String,
    pub session_name: String,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Serialize)]
pub struct ShareGetResponse {
    pub session_name: String,
    pub session_json: serde_json::Value,
    pub created_at: String,
    pub expires_at: Option<String>,
}

/// GET /api/sessions/share/:token — retrieve a shared session by token
async fn share_get_handler(
    Path(token): Path<String>,
    State(state): State<AppState>,
) -> Json<Result<ShareGetResponse, String>> {
    match state.share_manager.get(&token) {
        Ok(share) => Json(Ok(ShareGetResponse {
            session_name: share.session_name,
            session_json: share.session_json,
            created_at: share.created_at.to_rfc3339(),
            expires_at: share.expires_at.map(|e| e.to_rfc3339()),
        })),
        Err(e) => Json(Err(e)),
    }
}

/// POST /api/sessions/share — create a share from a saved session
async fn share_create_handler(
    State(state): State<AppState>,
    Json(req): Json<ShareCreateRequest>,
) -> Json<Result<ShareCreateResponse, String>> {
    // Load the session from SessionStore
    let store = SessionStore::new();
    let session_json = match store.export_to_json(&req.name) {
        Ok(j) => j,
        Err(e) => return Json(Err(format!("无法加载会话 '{}': {}", req.name, e))),
    };
    let json_value: serde_json::Value = match serde_json::from_str(&session_json) {
        Ok(v) => v,
        Err(e) => return Json(Err(format!("会话 JSON 解析失败: {}", e))),
    };

    match state
        .share_manager
        .create(&req.name, json_value, req.ttl_hours)
    {
        Ok(share) => {
            // Build a URL — use localhost as default since we don't know external addr
            let url = format!("/api/sessions/share/{}", share.token);
            Json(Ok(ShareCreateResponse {
                token: share.token.clone(),
                url,
                session_name: share.session_name,
                created_at: share.created_at.to_rfc3339(),
                expires_at: share.expires_at.map(|e| e.to_rfc3339()),
            }))
        }
        Err(e) => Json(Err(e)),
    }
}

pub async fn start_server(agent: Arc<RwLock<SelfIteratingBrain>>, addr: &str) {
    let bridge = Arc::new(WsBridge::new(Default::default()));
    let share_manager = SessionShareManager::new();
    let state = AppState { agent, bridge, share_manager };
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/stats", get(stats_handler))
        .route("/api/evolve", post(evolve_handler))
        .route("/api/reason", post(reason_handler))
        .route("/api/reason-kernel", post(reason_kernel_handler))
        .route("/api/sessions/share", post(share_create_handler))
        .route("/api/sessions/share/:token", get(share_get_handler))
        .route("/ws", get(ws_handler))
        .merge(h5_routes())
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind");
    println!("NeoTrix API + WS + H5 server listening on {}", addr);
    println!("  API:     http://{}/health", addr);
    println!("  WebChat: http://{}/chat", addr);
    axum::serve(listener, app).await.expect("Server error");
}

#[derive(Serialize)]
struct EvolveResponse {
    stage: usize,
    label: String,
    state_dim: usize,
    circuits: usize,
}

async fn evolve_handler() -> Json<EvolveResponse> {
    let mut kernel = crate::neotrix::nt_core_kernel::ReasoningKernel::new(18);
    kernel.evolve_stage();
    let s = kernel.stats();
    Json(EvolveResponse {
        stage: s.stage,
        label: s.label,
        state_dim: s.state_dim,
        circuits: s.total,
    })
}
