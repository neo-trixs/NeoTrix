use std::path::Path;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::Mutex;

use neotrix::core::nt_core_shutdown::ShutdownSignal;
use neotrix_bridge::{BridgeRegistry, Domain, IntentionVsa, VsaOrigin, WorldEffect};
use neotrix_bridge::evolution::EvolutionCore;
use neotrix_evolution::{format_timestamp, socket_path};

const DEFAULT_TICK_SECS: u64 = 60;

struct AppState {
    registry: BridgeRegistry,
    core: EvolutionCore,
    tick_count: u64,
}

#[tokio::main]
async fn main() {
    log::info!("neotrix-evolution v0.18.0 starting...");

    let socket_path_str = socket_path();
    let socket_path_var = Path::new(&socket_path_str);
    if socket_path_var.exists() {
        if let Err(e) = std::fs::remove_file(socket_path_var) {
            log::error!("warning: failed to remove old socket: {}", e);
        }
    }

    let tick_secs = std::env::var("NEOTRIX_EVOLUTION_TICK_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_TICK_SECS);

    let state = Arc::new(Mutex::new(AppState {
        registry: BridgeRegistry::new(),
        core: EvolutionCore::new(),
        tick_count: 0,
    }));

    let listener = UnixListener::bind(socket_path()).unwrap_or_else(|e| {
        log::error!("fatal: cannot bind to {}: {}", socket_path(), e);
        std::process::exit(1);
    });

    log::info!("listening on {}", socket_path_var.display());

    let shutdown = ShutdownSignal::new();

    // --- Main tick loop ---
    let tick_state = state.clone();
    let tick_shutdown = shutdown.clone();
    let tick_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(tick_secs));
        loop {
            tokio::select! {
                biased;
                _ = tick_shutdown.wait_shutdown() => {
                    log::info!("[evolution] tick loop: shutdown signal received, exiting");
                    break;
                }
                _ = interval.tick() => {}
            }

            let mut s = tick_state.lock().await;

            let events = s.registry.sense_all();

            for event in &events {
                let domain = match &event.origin {
                    VsaOrigin::Bridge(d) => *d,
                    _ => Domain::System,
                };

                let intention = IntentionVsa {
                    domain,
                    action: "sense".into(),
                    parameters: serde_json::json!({"negentropy": event.negentropy_contribution}),
                    confidence: 0.5,
                    urgency: 0.3,
                };

                let effect = WorldEffect {
                    domain,
                    description: format!("sense: neg={:.4}", event.negentropy_contribution),
                    success: true,
                    latency_ms: 0,
                };

                s.core.trace.record_step(&intention, &effect, 0);
                s.core.skill.observe_actuation(&intention, &effect, 0);
                s.core.harness.record_outcome(&intention, &effect, 0, false);
                s.core.co_evolution.reflect(
                    domain,
                    "sense".into(),
                    0.5,
                    event.negentropy_contribution,
                );
            }

            let signals = s.core.heartbeat_tick();
            s.tick_count += 1;

            let ts = format_timestamp();
            log::info!(
                "[{}] tick {}: {} traces, {} skills, {} insights, {} interceptions",
                ts,
                s.tick_count,
                s.core.trace.total_recorded,
                s.core.skill.skill_count(),
                s.core.co_evolution.shared_memory.len(),
                s.core.harness.total_interceptions,
            );

            for sig in &signals {
                log::info!("  signal: {}", sig);
            }
        }
    });

    // --- Socket accept loop ---
    let sock_state = state.clone();
    let sock_shutdown = shutdown.clone();
    let sock_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                _ = sock_shutdown.wait_shutdown() => {
                    log::info!("[evolution] socket loop: shutdown signal received, exiting");
                    break;
                }
                result = listener.accept() => {
                    match result {
                        Ok((stream, _)) => {
                            let state = sock_state.clone();
                            tokio::spawn(async move {
                                let (rd, mut wr) = stream.into_split();
                                let mut buf_reader = BufReader::new(rd);
                                let mut line = String::new();

                                if buf_reader.read_line(&mut line).await.is_err() {
                                    return;
                                }

                                let trimmed = line.trim();
                                let response = match trimmed {
                                    "status" => {
                                        let s = state.lock().await;
                                        format!("{}\n", s.core.stats_summary())
                                    }
                                    "summary" => {
                                        let s = state.lock().await;
                                        let mut out = String::from("=== Evolution Summary ===\n");
                                        out.push_str(&s.core.stats_summary());
                                        out.push('\n');
                                        for h in s.registry.health_all() {
                                            out.push_str(&format!(
                                                "  {:8} available={:5} errors={:3} actuations={}\n",
                                                h.domain.as_str(),
                                                h.available,
                                                h.error_count,
                                                h.total_actuations,
                                            ));
                                        }
                                        out.push_str(&format!(
                                            "  cross-domain links: {}\n",
                                            s.core.co_evolution.links.len()
                                        ));
                                        let skill_map = s.core.skill.summary();
                                        for (domain, count) in &skill_map {
                                            out.push_str(&format!(
                                                "  {:8} skills: {}\n",
                                                domain.as_str(),
                                                count
                                            ));
                                        }
                                        out
                                    }
                                    "health" => {
                                        let s = state.lock().await;
                                        let mut out = String::from("=== Bridge Health ===\n");
                                        for h in s.registry.health_all() {
                                            out.push_str(&format!(
                                                "  {:8} available={:5} errors={:3} actuations={}\n",
                                                h.domain.as_str(),
                                                h.available,
                                                h.error_count,
                                                h.total_actuations,
                                            ));
                                        }
                                        out
                                    }
                                    "reset" => {
                                        let mut s = state.lock().await;
                                        s.core = EvolutionCore::new();
                                        "evolution state reset\n".to_string()
                                    }
                                    _ => {
                                        format!(
                                            "unknown command: {}\navailable: status, summary, health, reset\n",
                                            trimmed
                                        )
                                    }
                                };

                                let _ = wr.write_all(response.as_bytes()).await;
                            });
                        }
                        Err(e) => {
                            log::error!("socket accept error: {}", e);
                        }
                    }
                }
            }
        }
    });

    // --- Wait for Ctrl+C ---
    tokio::select! {
        _ = tick_handle => {}
        _ = sock_handle => {}
        _ = tokio::signal::ctrl_c() => {
            log::info!("\nneotrix-evolution shutting down");
            shutdown.trigger("ctrl-c received");
        }
    }

    let _ = std::fs::remove_file(socket_path());
}
