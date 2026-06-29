use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        Json, Sse,
    },
    routing::{get, post},
    Router,
};
use futures::FutureExt;
use serde_json::json;
use tokio::sync::RwLock;

use crate::card::AgentCard;
use crate::registry::RegistryClient;
use crate::task::{
    A2ATask, NegotiationOffer, NegotiationResponse, SendTaskRequest, SendTaskResponse, TaskState,
};

/// Configuration passed to the A2AServer builder.
pub struct AgentConfig {
    pub card: AgentCard,
    pub http_port: u16,
    pub registry_url: Option<String>,
}

/// The shared runtime state available inside handler functions.
pub struct AgentContext {
    pub config: AgentConfig,
    pub tasks: RwLock<HashMap<String, A2ATask>>,
    pub registry_client: Option<RegistryClient>,
}

impl AgentContext {
    pub fn new(config: AgentConfig) -> Self {
        let registry_client = config
            .registry_url
            .as_ref()
            .map(|url| RegistryClient::new(url));
        Self {
            config,
            tasks: RwLock::new(HashMap::new()),
            registry_client,
        }
    }
}

/// Start heartbeat loop: sends heartbeat every 15s to the registry.
fn start_heartbeat(ctx: Arc<AgentContext>, shutdown: Arc<AtomicBool>) {
    tokio::spawn(async move {
        if let Err(panic) = std::panic::AssertUnwindSafe(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
            while !shutdown.load(Ordering::Relaxed) {
                interval.tick().await;
                if shutdown.load(Ordering::Relaxed) { break; }
                if let Some(ref client) = ctx.registry_client {
                    let _ = client.heartbeat(&ctx.config.card.name).await;
                }
            }
        }).catch_unwind().await {
            tracing::error!("[agent-core] heartbeat loop panic: {:?}", panic);
        }
    });
}

/// Start the A2A server: register with registry, heartbeat, serve, then unregister on shutdown.
pub async fn run_server(ctx: Arc<AgentContext>) {
    let app = build_router(ctx.clone());

    // Register with central registry
    if let Some(ref client) = ctx.registry_client {
        let transport = crate::registry::TransportInfo {
            host: "0.0.0.0".to_string(),
            port: ctx.config.http_port,
            protocol: "HTTP+JSON".to_string(),
        };
        if let Err(e) = client.register(&ctx.config.card, &transport).await {
            tracing::warn!("failed to register with registry: {e}");
        } else {
            tracing::info!("registered with registry: {:?}", ctx.config.registry_url);
        }
    }

    // Start background heartbeat
    let hb_shutdown = Arc::new(AtomicBool::new(false));
    if ctx.registry_client.is_some() {
        start_heartbeat(ctx.clone(), hb_shutdown.clone());
    }

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", ctx.config.http_port))
        .await
        .expect("failed to bind A2A port");

    tracing::info!(
        "A2A agent '{}' v{} ready at http://0.0.0.0:{}",
        ctx.config.card.name,
        ctx.config.card.version,
        ctx.config.http_port
    );

    // Graceful shutdown: wait for Ctrl+C, then unregister + stop
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.expect("agent-core server: failed to wait for Ctrl+C shutdown signal");
        })
        .await
        .expect("agent-core server: HTTP server exited with error");

    // Signal heartbeat to stop
    hb_shutdown.store(true, Ordering::Relaxed);

    // Cleanup: unregister from registry
    if let Some(ref client) = ctx.registry_client {
        tracing::info!("unregistering '{}' from registry", ctx.config.card.name);
        let _ = client.unregister(&ctx.config.card.name).await;
    }
    tracing::info!("agent shut down");
}

/// Default handlers ────
async fn agent_card_handler(
    State(ctx): State<Arc<AgentContext>>,
) -> Json<AgentCard> {
    Json(ctx.config.card.clone())
}

async fn negotiate_handler(
    State(_ctx): State<Arc<AgentContext>>,
    Json(offer): Json<NegotiationOffer>,
) -> Json<NegotiationResponse> {
    let selected = offer.versions.first().cloned().unwrap_or("1.0".into());
    Json(NegotiationResponse {
        selected_version: selected,
        negotiation_id: offer.negotiation_id,
        accepted: true,
    })
}

async fn send_task_handler(
    State(ctx): State<Arc<AgentContext>>,
    Json(req): Json<SendTaskRequest>,
) -> Result<Json<SendTaskResponse>, StatusCode> {
    let task = A2ATask {
        id: req.id.clone(),
        session_id: req.session_id,
        status: TaskState::Submitted,
        messages: req.messages,
        artifacts: Vec::new(),
        metadata: req.metadata,
    };
    ctx.tasks.write().await.insert(task.id.clone(), task.clone());
    Ok(Json(SendTaskResponse { task }))
}

async fn get_task_handler(
    State(ctx): State<Arc<AgentContext>>,
    Path(task_id): Path<String>,
) -> Result<Json<A2ATask>, StatusCode> {
    let tasks = ctx.tasks.read().await;
    tasks.get(&task_id).cloned().ok_or(StatusCode::NOT_FOUND).map(Json)
}

async fn stream_task_handler(
    State(_ctx): State<Arc<AgentContext>>,
    Path(task_id): Path<String>,
) -> Sse<impl futures::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = futures::stream::once(async move {
        Ok(Event::default().data(json!({
            "event": "status",
            "taskId": task_id,
        }).to_string()))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn health_handler(
    State(ctx): State<Arc<AgentContext>>,
) -> Json<serde_json::Value> {
    let task_count = ctx.tasks.read().await.len();
    Json(json!({
        "status": "ok",
        "agent": ctx.config.card.name,
        "version": ctx.config.card.version,
        "tasks_pending": task_count,
    }))
}

/// Build the standard A2A router with all default endpoints.
pub fn build_router(ctx: Arc<AgentContext>) -> Router {
    Router::new()
        .route("/.well-known/agent-card", get(agent_card_handler))
        .route("/.well-known/negotiate", post(negotiate_handler))
        .route("/tasks/send", post(send_task_handler))
        .route("/tasks/{task_id}", get(get_task_handler))
        .route("/tasks/{task_id}/stream", get(stream_task_handler))
        .route("/health", get(health_handler))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(ctx)
}
