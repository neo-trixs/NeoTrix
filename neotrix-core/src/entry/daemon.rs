use std::sync::Arc;

use tokio::sync::RwLock;

use neotrix::neotrix::nt_expert_routing::WorldModelV2;
use neotrix::neotrix::nt_mind::self_iterating::SelfIteratingBrain;
use neotrix::neotrix::nt_mind_background_loop::{BackgroundLoop, ConsciousnessIntegration};

use super::{info, init_brain};
use neotrix::core::nt_core_shutdown::ShutdownSignal;
use neotrix::core::nt_core_util::A2A_DEFAULT_PORT;

pub fn run_daemon(profile: &str) {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("failed to create tokio runtime: {}", e);
            return;
        }
    };
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);
        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        let bg_agent = Arc::new(RwLock::new(agent));
        let mut bg = BackgroundLoop::new(bg_agent.clone())
            .with_consciousness(ConsciousnessIntegration::new())
            .with_a2a_server_default(A2A_DEFAULT_PORT)
            .with_adaptive_controller_default();
        bg.goal_loop = neotrix::neotrix::nt_mind::goal_loop::GoalLoop::new();
        bg.nt_world_model = Some(WorldModelV2::new(8, 64));
        #[cfg(feature = "stealth-net")]
        {
            bg = bg.with_world_consciousness();
        }
        log::info!(
            "{} {}",
            info("[daemon]"),
            info("NeoTrix background daemon started")
        );
        let mut sigterm = None;
        if cfg!(unix) {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(s) => sigterm = Some(s),
                Err(e) => log::error!("failed to set up SIGTERM handler: {}", e),
            }
        }
        tokio::select! {
            _ = bg.start() => {},
            _ = async {
                if let Some(ref mut s) = sigterm {
                    s.recv().await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => {
                log::info!("\n{}", info("[daemon] SIGTERM received, shutting down..."));
            }
        }
    });
}

pub fn run_daemon_evolution(profile: &str) {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("failed to create tokio runtime: {}", e);
            return;
        }
    };
    rt.block_on(async {
        let (brain, bank) = init_brain(profile);
        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        let bg_agent = Arc::new(RwLock::new(agent));
        let mut bg = BackgroundLoop::new(bg_agent.clone())
            .with_consciousness(ConsciousnessIntegration::new())
            .with_a2a_server_default(A2A_DEFAULT_PORT)
            .with_adaptive_controller_default();
        bg.goal_loop = neotrix::neotrix::nt_mind::goal_loop::GoalLoop::new();
        bg.nt_world_model = Some(WorldModelV2::new(8, 64));
        #[cfg(feature = "stealth-net")]
        {
            bg = bg.with_world_consciousness();
        }
        log::info!(
            "{} {}",
            info("[daemon]"),
            info("NeoTrix evolution daemon started")
        );
        let daemon_handle = bg.start();
        let daemon = std::sync::Arc::new(std::sync::Mutex::new(
            neotrix::neotrix::nt_mind_evolution_daemon::EvolutionDaemon::default(),
        ));
        let daemon_clone = daemon.clone();
        let evolution_shutdown = ShutdownSignal::new();
        let evolution_shutdown_task = evolution_shutdown.clone();
        let evolution_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {}
                    _ = evolution_shutdown_task.wait_shutdown() => {
                        log::info!("[evolution] daemon evolution loop shutting down");
                        break;
                    }
                }
                let mut d = daemon_clone.lock().unwrap_or_else(|e| e.into_inner());
                let report = d.run_cycle_goal();
                if report.fixes_applied > 0 {
                    log::info!(
                        "[evolution] 🔧 {} fixes applied (cycle {})",
                        report.fixes_applied,
                        report.cycle
                    );
                }
            }
        });
        let mut sigterm = None;
        if cfg!(unix) {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(s) => sigterm = Some(s),
                Err(e) => log::error!("failed to set up SIGTERM handler: {}", e),
            }
        }
        tokio::select! {
            _ = daemon_handle => {},
            _ = evolution_handle => {
                log::info!("\n{}", info("[daemon] evolution loop exited"));
            }
            _ = tokio::signal::ctrl_c() => {
                log::info!("\n{}", info("[daemon] shutting down..."));
                evolution_shutdown.trigger("ctrl-c");
            }
            _ = async {
                if let Some(ref mut s) = sigterm {
                    s.recv().await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => {
                log::info!("\n{}", info("[daemon] SIGTERM received, shutting down..."));
                evolution_shutdown.trigger("SIGTERM");
            }
        }
    });
}
