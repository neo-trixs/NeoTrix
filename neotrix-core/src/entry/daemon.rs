//! daemon — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::path::PathBuf;
use std::sync::Arc;

use super::{err, info, init_brain, tokio_runtime};
use neotrix::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain;
use neotrix::l5_cognition::nt_mind::nt_mind::panorama_pipeline::PanoramaPipeline;
use neotrix::l5_cognition::nt_mind::nt_mind_background_loop::BackgroundLoop;
use neotrix::l2_perception::nt_world::nt_world_model_v2::WorldModelV2;

pub fn run_daemon(profile: &str) {
    let rt = tokio_runtime();
    rt.block_on(async {
        // ── Supervisor Loop: 完全自守护, 不依赖 launchd ──
        // 崩溃自动重启 + PID 文件 + 心跳 + 恢复日志
        let pid_path = dirs_home().join(".neotrix/daemon.pid");
        let heartbeat_path = dirs_home().join(".neotrix/daemon.heartbeat");
        let recovery_log = dirs_home().join(".neotrix/daemon_recovery.log");

        // 写入 PID
        let _ = std::fs::create_dir_all(pid_path.parent().unwrap_or(&pid_path));
        let _ = std::fs::write(&pid_path, std::process::id().to_string());

        println!(
            "{} {}",
            info("[daemon-supervisor]"),
            info("NeoTrix self-guardian daemon started (no launchd dependency)")
        );
        log_recovery(&recovery_log, "supervisor启动", "PID文件已写入");

        let mut restart_count: u32 = 0;
        const MAX_RESTARTS: u32 = 10;
        const BASE_BACKOFF_SECS: u64 = 2;
        const MAX_BACKOFF_SECS: u64 = 120;

        // 用于通知 supervisor 循环退出的 channel
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // 注册 SIGTERM 处理
        let shutdown_tx_term = std::sync::Mutex::new(Some(shutdown_tx));
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let _ = signal(SignalKind::terminate()).map(|mut s| {
                tokio::spawn(async move {
                    s.recv().await;
                    log::info!("[daemon-supervisor] SIGTERM received, shutting down");
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
                // 没有 KB, 所有吸收 handler (crawl_queue/exploration/knowledge_chain)
                // 都会在 "kb not attached" 处直接 return, 晶体无法吸收外部数据。
                if let Ok(kb) =
                    neotrix::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None)
                {
                    let kb = std::sync::Arc::new(kb);
                    bg.kb = Some(kb.clone());
                    log_recovery(
                        &recovery,
                        "KB已打开",
                        &format!("attached to BackgroundLoop"),
                    );
                    // 同时附加到 panorama
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

                // 心跳任务: 每 60s 写入时间戳
                let heartbeat_clone = heartbeat.clone();
                let heartbeat_task = tokio::spawn(async move {
                    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60));
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

                log_recovery(
                    &recovery,
                    "子进程启动",
                    &format!("PID={}", std::process::id()),
                );

                bg.start().await;
                tokio::signal::ctrl_c().await.unwrap_or_default();
                println!("\n{}", info("[daemon-supervisor] shutting down..."));
                // Persist E8 state on graceful shutdown
                if let Ok(brain_guard) = bg.brain.try_read() {
                    brain_guard.shutdown_save_e8();
                }
                bg.shutdown().await;
                heartbeat_task.abort();
                let _ = heartbeat_task.await;
            })
            .await;

            // 检查是否收到 SIGTERM/Ctrl+C
            if shutdown_rx.try_recv().is_ok() {
                log_recovery(&recovery_log, "supervisor退出", "收到关闭信号");
                break;
            }

            match result {
                Ok(()) => {
                    // 正常退出 (Ctrl+C)
                    log_recovery(&recovery_log, "supervisor退出", "正常关闭");
                    break;
                }
                Err(e) => {
                    // 子任务 panic 或 JoinError
                    restart_count += 1;
                    let backoff = std::cmp::min(
                        BASE_BACKOFF_SECS * 2u64.pow(restart_count - 1),
                        MAX_BACKOFF_SECS,
                    );
                    let msg = format!(
                        "子进程异常({}), 第{}次重启, 等待{}秒",
                        e, restart_count, backoff
                    );
                    log_recovery(&recovery_log, "崩溃恢复", &msg);
                    eprintln!("{} {}", err("[daemon-supervisor]"), err(&msg));

                    if restart_count >= MAX_RESTARTS {
                        let msg = format!("连续重启{}次, 超过上限, 停止守护", MAX_RESTARTS);
                        log_recovery(&recovery_log, "守护终止", &msg);
                        eprintln!("{} {}", err("[daemon-supervisor]"), err(&msg));
                        break;
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                }
            }
        }

        // 清理 PID 文件
        let _ = std::fs::remove_file(&pid_path);
        println!(
            "{} {}",
            info("[daemon-supervisor]"),
            info("NeoTrix self-guardian daemon stopped")
        );
    });
}

/// 写入守护进程恢复日志 (追加模式, 无外部依赖)
pub(crate) fn log_recovery(log_path: &std::path::Path, event: &str, detail: &str) {
    use std::io::Write;
    let ts = chrono_now();
    let line = format!("[{}] {} — {}\n", ts, event, detail);
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        let _ = f.write_all(line.as_bytes());
    }
    // 同时输出到 stderr
    log::info!("[daemon-supervisor] {} — {}", event, detail);
}

/// 简易时间戳 (无 chrono 依赖)
pub(crate) fn chrono_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // 简单格式: epoch seconds (足够用于日志)
    format!("{}", secs)
}

/// 获取用户主目录
pub(crate) fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}
