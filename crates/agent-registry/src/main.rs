use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use agent_core::registry::{AgentRegistration, AgentSearchResult};
use agent_registry::{RegistryState, RegistryStats, SearchQuery};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use futures::FutureExt;
use tower_http::cors::CorsLayer;

// ── Handlers ──

async fn register_handler(
    State(state): State<Arc<RegistryState>>,
    Json(reg): Json<AgentRegistration>,
) -> StatusCode {
    state.register(reg).await;
    StatusCode::OK
}

async fn unregister_handler(
    State(state): State<Arc<RegistryState>>,
    Path(name): Path<String>,
) -> StatusCode {
    if state.unregister(&name).await {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn list_agents_handler(
    State(state): State<Arc<RegistryState>>,
) -> Json<AgentSearchResult> {
    let agents = state.list_all().await;
    Json(AgentSearchResult { agents })
}

async fn search_handler(
    State(state): State<Arc<RegistryState>>,
    Query(query): Query<SearchQuery>,
) -> Json<AgentSearchResult> {
    let agents = if let Some(skill) = &query.skill {
        state.search_by_skill(skill).await
    } else if let Some(tag) = &query.tag {
        state.search_by_tag(tag).await
    } else if let Some(text) = &query.text {
        state.search_by_text(text).await
    } else {
        state.list_all().await
    };
    Json(AgentSearchResult { agents })
}

async fn heartbeat_handler(
    State(state): State<Arc<RegistryState>>,
    Path(name): Path<String>,
) -> StatusCode {
    if state.heartbeat(&name).await {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

async fn stats_handler(
    State(state): State<Arc<RegistryState>>,
) -> Json<RegistryStats> {
    Json(state.stats().await)
}

async fn agent_detail_handler(
    State(state): State<Arc<RegistryState>>,
    Path(name): Path<String>,
) -> Result<Json<AgentRegistration>, StatusCode> {
    let agents = state.agents.read().await;
    agents
        .get(&name)
        .map(|a| {
            Json(AgentRegistration {
                card: a.card.clone(),
                transport: a.transport.clone(),
            })
        })
        .ok_or(StatusCode::NOT_FOUND)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let port = std::env::var("AGENT_REGISTRY_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(42070);

    let state = Arc::new(RegistryState::new());

    // Background stale agent cleanup every 60s
    let cleanup_shutdown = Arc::new(AtomicBool::new(false));
    {
        let state = state.clone();
        let sd = cleanup_shutdown.clone();
        tokio::spawn(async move {
            if let Err(panic) = std::panic::AssertUnwindSafe(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
                while !sd.load(Ordering::Relaxed) {
                    interval.tick().await;
                    if sd.load(Ordering::Relaxed) { break; }
                    let cleaned = state.cleanup_dead().await;
                    if cleaned > 0 {
                        tracing::info!("registry cleanup: removed {cleaned} stale agents");
                    }
                }
            }).catch_unwind().await {
                tracing::error!("[registry-cleanup] background loop panic: {:?}", panic);
            }
        });
    }

    let app = Router::new()
        // Agent registration
        .route("/agents/register", post(register_handler))
        .route("/agents/unregister/{name}", post(unregister_handler))
        // Discovery
        .route("/agents", get(list_agents_handler))
        .route("/agents/search", get(search_handler))
        .route("/agents/{name}", get(agent_detail_handler))
        // Health
        .route("/agents/{name}/heartbeat", post(heartbeat_handler))
        .route("/stats", get(stats_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("failed to bind registry port");

    tracing::info!("Agent Registry v0.1 listening on :{port}");

    axum::serve(listener, app).await.expect("server error");
}
