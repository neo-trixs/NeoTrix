mod a2a;
mod config;
mod engine;
mod scheduler;
mod state;

use std::sync::Arc;

use futures::FutureExt;
use neotrix::core::nt_core_shutdown::ShutdownSignal;
use a2a::AppContext;
use agent_core::registry::{RegistryClient, TransportInfo};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,ghost_mvp_agent=debug")),
        )
        .init();

    let cfg = config::Config::load();
    tracing::info!("GhostMVP v{} starting", cfg.agent_version);
    tracing::info!("HTTP server on port {}", cfg.http_port);
    tracing::info!("Data directory: {}", cfg.data_dir.display());

    let state_store = state::Store::new(&cfg.data_dir);
    let http_client = reqwest::Client::new();
    let registry_client = cfg.registry_url.as_ref().map(|url| RegistryClient::new(url));

    let ctx = Arc::new(AppContext {
        config: cfg.clone(),
        state_store,
        tasks: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        registry_client,
        http_client,
    });

    let shutdown = ShutdownSignal::new();

    // Register with agent registry
    if let Some(ref client) = ctx.registry_client {
        let transport = TransportInfo {
            host: "0.0.0.0".to_string(),
            port: cfg.http_port,
            protocol: "HTTP+JSON".to_string(),
        };
        let card = a2a::build_agent_card(&cfg);
        if let Err(e) = client.register(&card, &transport).await {
            tracing::warn!("failed to register with registry: {e}");
        } else {
            tracing::info!("registered with registry at {:?}", cfg.registry_url);
        }
    }

    // Start background heartbeat
    if ctx.registry_client.is_some() {
        let ctx_clone = ctx.clone();
        let sig = shutdown.clone();
        tokio::spawn(async move {
            if let Err(panic) = std::panic::AssertUnwindSafe(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
                loop {
                    tokio::select! {
                        biased;
                        _ = sig.wait_shutdown() => {
                            tracing::info!("[ghost] heartbeat loop: shutdown received, exiting");
                            break;
                        }
                        _ = interval.tick() => {}
                    }
                    if let Some(ref client) = ctx_clone.registry_client {
                        let _ = client.heartbeat(&ctx_clone.config.agent_name).await;
                    }
                }
            }).catch_unwind().await {
                tracing::error!("[ghost-mvp] heartbeat panic: {:?}", panic);
            }
        });
    }

    // Start background scheduler
    scheduler::start(ctx.clone(), shutdown.clone()).await;

    // Build and serve A2A HTTP server
    let app = a2a::build_router(ctx.clone());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cfg.http_port))
        .await
        .expect("failed to bind port");

    tracing::info!("GhostMVP agent ready at http://0.0.0.0:{}", cfg.http_port);
    tracing::info!("Agent card: http://0.0.0.0:{}/.well-known/agent-card", cfg.http_port);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.expect("ghost-mvp agent: failed to wait for Ctrl+C shutdown signal");
        })
        .await
        .expect("ghost-mvp agent: HTTP server exited with error");

    // Graceful shutdown
    tracing::info!("shutting down GhostMVP");
    shutdown.trigger("server stopped");
    if let Some(ref client) = ctx.registry_client {
        let _ = client.unregister(&cfg.agent_name).await;
    }
    ctx.state_store.persist().await;
    tracing::info!("GhostMVP shut down");
}
