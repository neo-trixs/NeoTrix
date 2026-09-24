//! daemon_common — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::sync::Arc;

use super::{info, init_brain, tokio_runtime};
use neotrix::l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain;
use neotrix::l5_cognition::nt_mind::nt_mind::panorama_pipeline::PanoramaPipeline;
use neotrix::l5_cognition::nt_mind::nt_mind_background_loop::BackgroundLoop;
use neotrix::l2_perception::nt_world::nt_world_model_v2::WorldModelV2;

/// Public entry point for clap-based CLI dispatch: runs background loop daemon.
/// Named "daemon" to distinguish from the actual HTTP server in server.rs.
pub(crate) fn run_background_daemon(_addr: &str, profile: &str) {
    println!("{} v{}", info("NeoTrix Server"), env!("CARGO_PKG_VERSION"));
    println!(
        "{}",
        info("Starting background services... Press Ctrl+C to stop.")
    );
    let server_rt = tokio_runtime();
    server_rt.block_on(async {
        let (brain, bank) = init_brain(profile);
        let mut agent = SelfIteratingBrain::new();
        agent.brain = brain;
        agent.reasoning_bank = bank;
        let bg_agent = Arc::new(tokio::sync::RwLock::new(agent));
        let mut bg = BackgroundLoop::new(bg_agent.clone());
        bg.goal_loop = neotrix::l5_cognition::nt_mind::nt_mind::GoalLoop::new();
        bg.nt_world_model = Some(WorldModelV2::new(8, 64));
        let mut panorama = PanoramaPipeline::new();
        if let Ok(kb) = neotrix::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(None) {
            let kb = std::sync::Arc::new(kb);
            panorama.attach_kb(kb.clone());
            bg.kb = Some(kb);
        }
        bg = bg.with_panorama(panorama);
        #[cfg(feature = "stealth-net")]
        {
            bg = bg.with_world_consciousness();
        }
        println!("{}", info("[server] all services initialized."));
        // 并行启动 HTTP API server（独立线程+独立 runtime，避免被 bg.start 阻塞）
        // 修复: 此前 serve 命令只跑 BackgroundLoop, HTTP server 从未启动（契约断裂）
        let http_port = parse_http_port(_addr);
        let http_handle = std::thread::spawn(move || {
            // D5: runtime 创建失败不 panic — HTTP 服务降级为日志告警, 主进程继续运行
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("[server] HTTP runtime 创建失败, 服务不可用: {}", e);
                    return;
                }
            };
            rt.block_on(async {
                // L1 层禁止直接依赖 L8 (层边界守卫 arch_fitness_layer_boundary):
                // 在 entry (bin 层) 构造 ReasoningBrain 后经 start_server_with 注入。
                neotrix::l1_action::nt_io::nt_io_web::server::start_server_with(
                    http_port,
                    Box::new(neotrix::l5_cognition::nt_mind::nt_mind::ReasoningBrain::new()),
                    neotrix::l1_action::nt_core_bank::bank::ReasoningBank::new(10000),
                )
                .await;
            });
        });
        // G3: SIGHUP → 配置热重载（kill -HUP <pid> 不重启即刷新 stealth-net 配置）
        let sighup_handle = spawn_sighup_reload();
        bg.start().await;
        tokio::signal::ctrl_c().await.unwrap_or_default();
        println!("\n{}", info("[server] shutting down..."));
        sighup_handle.abort();
        // Persist E8 state on graceful shutdown (SIGTERM/Ctrl+C)
        // Without this hook, up to 5 iterations of transition matrix learning can be lost.
        if let Ok(brain_guard) = bg.brain.try_read() {
            brain_guard.shutdown_save_e8();
        }
        bg.shutdown().await;
        // HTTP server thread will be terminated when process exits
        let _ = http_handle;
    });
}

/// 注册 SIGHUP → 热重载接线（G3: 补齐"信号重载"缺失链路）。
/// 第三方 CLI / 运维可用 `kill -HUP <pid>` 触发配置热重载，无需重启 daemon。
/// 返回 () — 由调用方决定是否 join。
pub(crate) fn spawn_sighup_reload() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup()) {
                Ok(mut sig) => {
                    sig.recv().await;
                    #[cfg(feature = "stealth-net")]
                    {
                        match neotrix::l3_embodiment::nt_shield::nt_shield_stealth_net::config::reload() {
                            Ok(_) => log::info!("[hotreload] SIGHUP: stealth-net config reloaded"),
                            Err(e) => log::warn!("[hotreload] SIGHUP reload failed: {}", e),
                        }
                    }
                    #[cfg(not(feature = "stealth-net"))]
                    log::info!(
                        "[hotreload] SIGHUP received (stealth-net feature off, nothing to reload)"
                    );
                }
                Err(e) => {
                    log::warn!("[hotreload] failed to register SIGHUP handler: {}", e);
                    break;
                }
            }
        }
    })
}

/// 从 --addr 参数解析 HTTP 端口（默认 3000）
pub(crate) fn parse_http_port(addr: &str) -> u16 {
    addr.rsplit(':')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000)
}
