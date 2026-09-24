//! daemon_evolution — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::sync::Arc;

use super::{dirs_home, err, info, init_brain, log_recovery, tokio_runtime};
use neotrix::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain;
use neotrix::l5_cognition::nt_mind::nt_mind::panorama_pipeline::PanoramaPipeline;
use neotrix::l5_cognition::nt_mind::nt_mind_background_loop::BackgroundLoop;
use neotrix::l2_perception::nt_world::nt_world_model_v2::WorldModelV2;

pub fn run_daemon_evolution(profile: &str) {
    let rt = tokio_runtime();
    rt.block_on(async {
        // ── Evolution Supervisor: 自守护 + 持续自进化 ──
        let pid_path = dirs_home().join(".neotrix/daemon.pid");
        let heartbeat_path = dirs_home().join(".neotrix/daemon.heartbeat");
        let recovery_log = dirs_home().join(".neotrix/daemon_recovery.log");

        let _ = std::fs::create_dir_all(pid_path.parent().unwrap_or(&pid_path));
        let _ = std::fs::write(&pid_path, std::process::id().to_string());

        println!(
            "{} {}",
            info("[evolution-supervisor]"),
            info("NeoTrix evolution self-guardian started")
        );
        log_recovery(&recovery_log, "evolution-supervisor启动", "PID文件已写入");

        let mut restart_count: u32 = 0;
        const MAX_RESTARTS: u32 = 10;
        const BASE_BACKOFF_SECS: u64 = 2;
        const MAX_BACKOFF_SECS: u64 = 120;

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let shutdown_tx_term = std::sync::Mutex::new(Some(shutdown_tx));
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let _ = signal(SignalKind::terminate()).map(|mut s| {
                tokio::spawn(async move {
                    s.recv().await;
                    log::info!("[evolution-supervisor] SIGTERM received");
                    if let Ok(mut tx) = shutdown_tx_term.lock() {
                        let _ = tx.take().map(|tx| tx.send(()));
                    }
                });
            });
        }

        loop {
            let profile = profile.to_string();
            let heartbeat = heartbeat_path.clone();
            let recovery = recovery_log.clone();

            let result = tokio::spawn(async move {
                let (brain, bank) = init_brain(&profile);
                let mut agent = SelfIteratingBrain::new();
                agent.brain = brain;
                agent.reasoning_bank = bank;
                let bg_agent = Arc::new(tokio::sync::RwLock::new(agent));
                let mut bg = BackgroundLoop::new(bg_agent.clone());
                bg.goal_loop = neotrix::l5_cognition::nt_mind::nt_mind::GoalLoop::new();
                bg.nt_world_model = Some(WorldModelV2::new(8, 64));
                // ── 关键: 打开 KB 并附加到 BackgroundLoop ──
                if let Ok(kb) = neotrix::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None) {
                    let kb = std::sync::Arc::new(kb);
                    bg.kb = Some(kb.clone());
                    log_recovery(&recovery, "KB已打开", &format!("attached to evolution daemon"));
                    let mut panorama = PanoramaPipeline::new();
                    panorama.attach_kb(kb);
                    bg = bg.with_panorama(panorama);
                } else {
                    log_recovery(&recovery, "KB打开失败", "吸收功能将不可用");
                }
                #[cfg(feature = "stealth-net")]
                {
                    bg = bg.with_world_consciousness();
                }

                // 心跳任务
                let heartbeat_clone = heartbeat.clone();
                let heartbeat_task = tokio::spawn(async move {
                    let mut ticker =
                        tokio::time::interval(std::time::Duration::from_secs(60));
                    loop {
                        ticker.tick().await;
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let _ = std::fs::write(
                            &heartbeat_clone,
                            format!("{}\n{}", ts, std::process::id()),
                        );
                    }
                });

                log_recovery(&recovery, "evolution子进程启动", &format!("PID={}", std::process::id()));

                // Evolution 后台任务
                let daemon = std::sync::Arc::new(std::sync::Mutex::new(
                    neotrix::l5_cognition::nt_mind::evolution::evolution_daemon::EvolutionDaemon::default()
                ));
                let daemon_clone = daemon.clone();
                let evolution_task = tokio::spawn(async move {
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                        let mut d = daemon_clone.lock().unwrap_or_else(|e| e.into_inner());
                        let report = d.run_cycle_goal();
                        if report.fixes_applied > 0 {
                            println!("[evolution] 🔧 {} fixes applied (cycle {})", report.fixes_applied, report.cycle);
                        }
                    }
                });

                bg.start().await;
                tokio::signal::ctrl_c().await.unwrap_or_default();
                println!("\n{}", info("[evolution-supervisor] shutting down..."));
                if let Ok(brain_guard) = bg.brain.try_read() {
                    brain_guard.shutdown_save_e8();
                }
                evolution_task.abort();
                let _ = evolution_task.await;
                bg.shutdown().await;
                heartbeat_task.abort();
                let _ = heartbeat_task.await;
            })
            .await;

            if shutdown_rx.try_recv().is_ok() {
                log_recovery(&recovery_log, "evolution-supervisor退出", "收到关闭信号");
                break;
            }

            match result {
                Ok(()) => {
                    log_recovery(&recovery_log, "evolution-supervisor退出", "正常关闭");
                    break;
                }
                Err(e) => {
                    restart_count += 1;
                    let backoff = std::cmp::min(
                        BASE_BACKOFF_SECS * 2u64.pow(restart_count - 1),
                        MAX_BACKOFF_SECS,
                    );
                    let msg = format!(
                        "evolution子进程异常({}), 第{}次重启, 等待{}秒",
                        e, restart_count, backoff
                    );
                    log_recovery(&recovery_log, "evolution崩溃恢复", &msg);
                    eprintln!("{} {}", err("[evolution-supervisor]"), err(&msg));

                    if restart_count >= MAX_RESTARTS {
                        let msg = format!("连续重启{}次, 超过上限, 停止守护", MAX_RESTARTS);
                        log_recovery(&recovery_log, "evolution守护终止", &msg);
                        eprintln!("{} {}", err("[evolution-supervisor]"), err(&msg));
                        break;
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                }
            }
        }

        let _ = std::fs::remove_file(&pid_path);
        println!(
            "{} {}",
            info("[evolution-supervisor]"),
            info("NeoTrix evolution self-guardian stopped")
        );
    });
}
