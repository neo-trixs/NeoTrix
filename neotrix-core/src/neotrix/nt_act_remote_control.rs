use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    Router, routing::get, extract::ws::{WebSocket, WebSocketUpgrade, Message},
    response::Json, extract::Query, extract::State,
};

/// Shared brain state accessible to the remote control server
pub struct RemoteBrainState {
    pub pipeline_status: String,
    pub current_task: String,
    pub iteration: u64,
    pub reward: f64,
}

impl RemoteBrainState {
    pub fn new() -> Self {
        Self {
            pipeline_status: "idle".into(),
            current_task: String::new(),
            iteration: 0,
            reward: 0.0,
        }
    }
}

#[derive(serde::Serialize)]
struct StatusResponse {
    status: String,
    task: String,
    iteration: u64,
    reward: f64,
}

#[derive(serde::Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_k")]
    k: usize,
}

fn default_k() -> usize { 5 }

#[derive(Clone)]
struct AppState {
    brain: Arc<RwLock<RemoteBrainState>>,
}

/// Lightweight HTTP+WebSocket remote control server for NeoTrix.
///
/// Endpoints:
/// - GET  /status     → pipeline status JSON
/// - GET  /journal/search?q=...&k=5 → journal search results
/// - WS   /ws         → bidirectional command/response channel
pub struct RemoteControlServer {
    port: u16,
    state: Arc<RwLock<RemoteBrainState>>,
}

impl Default for RemoteControlServer {
    fn default() -> Self {
        Self::new(9876)
    }
}

impl RemoteControlServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            state: Arc::new(RwLock::new(RemoteBrainState::new())),
        }
    }

    pub fn brain_state(&self) -> Arc<RwLock<RemoteBrainState>> {
        self.state.clone()
    }

    pub async fn start(self) -> Result<(), String> {
        let app_state = AppState { brain: self.state.clone() };

        let app = Router::new()
            .route("/status", get(status_handler))
            .route("/journal/search", get(journal_search_handler))
            .route("/ws", get(ws_handler))
            .with_state(app_state);

        let addr = format!("127.0.0.1:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr).await
            .map_err(|e| format!("cannot bind remote control server: {}", e))?;

        log::info!("[remote] control server listening on http://{}", addr);
        axum::serve(listener, app).await
            .map_err(|e| format!("server error: {}", e))?;
        Ok(())
    }
}

async fn status_handler(
    State(state): State<AppState>,
) -> Json<StatusResponse> {
    let brain = state.brain.read().await;
    Json(StatusResponse {
        status: brain.pipeline_status.clone(),
        task: brain.current_task.clone(),
        iteration: brain.iteration,
        reward: brain.reward,
    })
}

async fn journal_search_handler(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Json<Vec<serde_json::Value>> {
    let _brain = state.brain.read().await;
    let idx = match crate::neotrix::nt_world_journal_index::JournalIndex::open() {
        Ok(idx) => idx,
        Err(_) => return Json(vec![]),
    };
    let results = match idx.search(&params.q, params.k) {
        Ok(r) => r,
        Err(_) => return Json(vec![]),
    };
    let entries: Vec<serde_json::Value> = results.into_iter().map(|(entry, score)| {
        serde_json::json!({
            "id": entry.id,
            "goal": entry.goal_text,
            "timestamp": entry.timestamp,
            "score": score,
            "evidence_count": entry.evidence_count,
            "success": entry.success,
        })
    }).collect();
    Json(entries)
}

async fn ws_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    log::info!("[remote] WebSocket connected");
    let _ = socket.send(Message::Text("{\"type\":\"connected\"}".into())).await;

    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(Message::Text(t)) => t,
            Ok(Message::Close(_)) => break,
            _ => continue,
        };

        let response = match msg.as_str() {
            "ping" => "{\"type\":\"pong\"}".to_string(),
            "status" => {
                let brain = state.brain.read().await;
                serde_json::to_string(&StatusResponse {
                    status: brain.pipeline_status.clone(),
                    task: brain.current_task.clone(),
                    iteration: brain.iteration,
                    reward: brain.reward,
                }).unwrap_or_else(|_| "{}".into())
            }
            other if other.starts_with("search:") => {
                let query = other.trim_start_matches("search:");
                let idx = crate::neotrix::nt_world_journal_index::JournalIndex::open().ok();
                match idx.and_then(|i| i.search(query, 5).ok()) {
                    Some(results) => {
                        let entries: Vec<serde_json::Value> = results.into_iter().map(|(e, s)| {
                            serde_json::json!({"id": e.id, "goal": e.goal_text, "score": s})
                        }).collect();
                        serde_json::to_string(&entries).unwrap_or_else(|_| "[]".into())
                    }
                    None => "[]".into()
                }
            }
            _ => {
                serde_json::json!({"type": "error", "message": format!("unknown command: {}", msg)}).to_string()
            }
        };

        if socket.send(Message::Text(response.into())).await.is_err() {
            break;
        }
    }
    log::info!("[remote] WebSocket disconnected");
}
