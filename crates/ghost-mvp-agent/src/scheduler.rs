use std::sync::Arc;

use futures::FutureExt;
use tokio::time::{interval, Duration};

use neotrix::core::nt_core_shutdown::ShutdownSignal;
use crate::a2a::AppContext;

pub async fn start(ctx: Arc<AppContext>, signal: ShutdownSignal) {
    let analysis_interval = ctx.config.schedule_analysis_interval_hours;
    let audit_interval = ctx.config.schedule_geo_audit_interval_days;

    // Start the periodic analysis ticker
    if analysis_interval > 0 {
        let ctx_clone = ctx.clone();
        let sig = signal.clone();
        tokio::spawn(async move {
            if let Err(panic) = std::panic::AssertUnwindSafe(async move {
                let mut timer = interval(Duration::from_secs(analysis_interval * 3600));
                // Skip the first tick immediately — give the server time to start
                timer.tick().await;
                loop {
                    tokio::select! {
                        biased;
                        _ = sig.wait_shutdown() => {
                            tracing::info!("[scheduler] analysis loop: shutdown received, exiting");
                            break;
                        }
                        _ = timer.tick() => {}
                    }
                    tracing::info!("[scheduler] running scheduled analysis tick");
                    if let Err(e) = scheduled_analysis(&ctx_clone).await {
                        tracing::error!("[scheduler] analysis tick failed: {e}");
                    }
                }
            }).catch_unwind().await {
                tracing::error!("[ghost-mvp] analysis panic: {:?}", panic);
            }
        });
    }

    // Start the periodic GEO audit ticker
    if audit_interval > 0 {
        let ctx_clone = ctx.clone();
        let sig = signal.clone();
        tokio::spawn(async move {
            if let Err(panic) = std::panic::AssertUnwindSafe(async move {
                let mut timer = interval(Duration::from_secs(audit_interval * 86400));
                timer.tick().await;
                loop {
                    tokio::select! {
                        biased;
                        _ = sig.wait_shutdown() => {
                            tracing::info!("[scheduler] GEO audit loop: shutdown received, exiting");
                            break;
                        }
                        _ = timer.tick() => {}
                    }
                    tracing::info!("[scheduler] running scheduled GEO audit");
                    if let Err(e) = scheduled_geo_audit(&ctx_clone).await {
                        tracing::error!("[scheduler] GEO audit failed: {e}");
                    }
                }
            }).catch_unwind().await {
                tracing::error!("[ghost-mvp] GEO audit panic: {:?}", panic);
            }
        });
    }

    // Persistence ticker — save state every 5 minutes
    let ctx_clone = ctx.clone();
    let sig = signal.clone();
    tokio::spawn(async move {
        if let Err(panic) = std::panic::AssertUnwindSafe(async move {
            let mut timer = interval(Duration::from_secs(300));
            loop {
                tokio::select! {
                    biased;
                    _ = sig.wait_shutdown() => {
                        tracing::info!("[scheduler] persistence loop: shutdown received, exiting");
                        break;
                    }
                    _ = timer.tick() => {}
                }
                ctx_clone.state_store.persist().await;
                tracing::debug!("[scheduler] state persisted");
            }
        }).catch_unwind().await {
            tracing::error!("[ghost-mvp] persistence panic: {:?}", panic);
        }
    });
}

async fn scheduled_analysis(ctx: &Arc<AppContext>) -> Result<(), String> {
    // Read the last analyzed repo from state, or use a default
    let state = ctx.state_store.read().await;
    let recent: Vec<String> = state
        .analyses
        .iter()
        .rev()
        .take(5)
        .map(|a| a.repo_url.clone())
        .collect();
    drop(state);

    tracing::info!("[scheduler] recent analyses: {:?}", recent);
    Ok(())
}

async fn scheduled_geo_audit(ctx: &Arc<AppContext>) -> Result<(), String> {
    tracing::info!(
        "[scheduler] geo audit skipped — auto-audit coming in v1.1 (endpoint: {:?})",
        ctx.config.neotrix_a2a_endpoint
    );
    Ok(())
}
