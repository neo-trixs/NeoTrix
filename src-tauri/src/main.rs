//! NeoTrix Tauri V2 Desktop
//!
//! V2 架构: 完整的桌面端入口 (Brain + PTY 终端)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

use clap::Parser;
use tauri::{Manager, State, Emitter, Listener};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;
use neotrix::SelfIteratingBrain;
use neotrix::neotrix::nt_mind_background_loop::{BackgroundLoop, ConsciousnessIntegration, ExperienceStats};
use neotrix::neotrix::nt_mind::goal_loop::GoalLoop;
use neotrix::neotrix::nt_shield::permissions::PermissionManager;
use neotrix_types::core::node_canvas::CanvasProject;

mod browser_host;
mod commands;
// mod consciousness_bridge;  // removed — inlined into setup()
mod permission_dialog;
#[path = "../plugins/mod.rs"]
pub mod plugins;

#[derive(Parser)]
#[clap(name = "neotrix-tauri", version)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    #[clap(name = "desktop")]
    Desktop,
    #[clap(name = "headless")]
    Headless,
    #[clap(name = "reason")]
    Reason { prompt: String },
}

/// Tauri 命令: PTY 终端管理
#[tauri::command]
fn pty_spawn(state: State<'_, Arc<commands::pty::PtyManager>>, session_id: String, cols: u16, rows: u16) -> Result<(), String> {
    state.spawn(&session_id, cols, rows)
}

#[tauri::command]
fn pty_write(state: State<'_, Arc<commands::pty::PtyManager>>, session_id: String, data: String) -> Result<(), String> {
    state.write(&session_id, &data)
}

#[tauri::command]
fn pty_resize(state: State<'_, Arc<commands::pty::PtyManager>>, session_id: String, cols: u16, rows: u16) -> Result<(), String> {
    state.resize(&session_id, cols, rows)
}

#[tauri::command]
fn pty_close(state: State<'_, Arc<commands::pty::PtyManager>>, session_id: String) -> Result<(), String> {
    state.close(&session_id);
    Ok(())
}

/// Run one auto-sync cycle: for each pair, diff and transfer, emit result event
fn auto_sync_cycle(sync_state: &commands::SyncState, handle: &tauri::AppHandle) {
    let start = std::time::Instant::now();

    let peer_ids = {
        let guard = match sync_state.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let sync = match guard.as_ref() {
            Some(s) => s,
            None => return,
        };
        let ids: Vec<String> = sync.pairs().iter().map(|p| p.peer_id.clone()).collect();
        if ids.is_empty() {
            return;
        }
        ids
    };

    let mut total_files: usize = 0;
    let mut has_error = false;

    for peer_id in &peer_ids {
        let count = {
            let mut guard = match sync_state.lock() {
                Ok(g) => g,
                Err(_) => {
                    has_error = true;
                    continue;
                }
            };
            let sync = match guard.as_mut() {
                Some(s) => s,
                None => {
                    has_error = true;
                    continue;
                }
            };
            match sync.compute_diff(peer_id) {
                Ok((_, _, diff)) => diff.to_send.len() + diff.to_receive.len(),
                Err(e) => {
                    log::warn!("[auto-sync] diff {}: {}", peer_id, e);
                    has_error = true;
                    continue;
                }
            }
        };

        {
            let mut guard = match sync_state.lock() {
                Ok(g) => g,
                Err(_) => {
                    has_error = true;
                    continue;
                }
            };
            let sync = match guard.as_mut() {
                Some(s) => s,
                None => {
                    has_error = true;
                    continue;
                }
            };
            match sync.execute_sync(peer_id) {
                Ok(_) => total_files += count,
                Err(e) => {
                    log::warn!("[auto-sync] sync {}: {}", peer_id, e);
                    has_error = true;
                }
            }
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let now = chrono::Local::now();
    let timestamp = now.format("%Y-%m-%dT%H:%M:%S%.3f%z").to_string();

    let payload = serde_json::json!({
        "status": if has_error { "error" } else { "ok" },
        "files_synced": total_files,
        "duration_ms": duration_ms,
        "timestamp": timestamp,
    });
    let _ = handle.emit("sync-complete", payload);

    if let Some(tray) = handle.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(&format!(
            "NeoTrix Desktop — Last sync: {}",
            now.format("%H:%M:%S")
        )));
    }
}

fn main() {
    #[cfg(feature = "telemetry")]
    let _sentry_guard = neotrix::neotrix::nt_shield_sentry::init_sentry();
    #[cfg(not(feature = "telemetry"))]
    let _sentry_guard = None::<()>;
    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Desktop) => {
            // ── Initialize SelfIteratingBrain ──
            let mut agent = SelfIteratingBrain::new();
            agent.load_cortex();
            agent.init_reasoning_engine();
            agent.quality_threshold = 0.7;
            agent.auto_absorb = true;
            agent.auto_memory_iteration = true;
            agent.memory_iteration_interval = 5;

            let agent = Arc::new(RwLock::new(agent));

            // ── Shared consciousness stats & input ──
            let ci_pending_input: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
            let ci_response_output: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
            let stats_snapshot: Arc<std::sync::RwLock<ExperienceStats>> = Arc::new(std::sync::RwLock::new(ExperienceStats {
                c_score: 0.0, sp_coherence: 0.0, nm_da: 0.0, nm_ne: 0.0, nm_ht: 0.0,
                nm_ach: 0.0, critic_pass_rate: 0.0, load_mode: 0, vsa_buffer_size: 0,
                text_feed_total: 0, reflexivity: 0.0, emotion: "init".to_string(),
                critic_issued: 0, cycle: 0,
                last_critique: neotrix::core::nt_core_consciousness::CritiqueResult::perfect(),
            }));

            // ── Shared canvas snapshot for E8 reasoning graph ──
            let canvas_snapshot: Arc<std::sync::RwLock<CanvasProject>> = Arc::new(std::sync::RwLock::new(CanvasProject::new("E8 Reasoning Graph")));

            // ── Spawn BackgroundLoop with shared CI and snapshot ──
            let bg_agent = agent.clone();
            let bg_pending = ci_pending_input.clone();
            let bg_output = ci_response_output.clone();
            let bg_snapshot = stats_snapshot.clone();
            let bg_canvas = canvas_snapshot.clone();
            std::thread::spawn(move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        log::error!("[bg] failed to create tokio runtime: {}", e);
                        return;
                    }
                };
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    rt.block_on(async {
                        let bg_ci = ConsciousnessIntegration::new();
                        let mut bg = BackgroundLoop::new(bg_agent)
                            .with_consciousness(bg_ci)
                            .with_goal_loop(GoalLoop::new())
                            .with_stats_snapshot(bg_snapshot)
                            .with_canvas_snapshot(bg_canvas)
                            .with_adaptive_controller_default();
                        // Share the ci_pending_input and ci_response_output
                        bg.ci_pending_input = bg_pending;
                        bg.ci_response_output = bg_output;
                        bg.start().await;
                    });
                }));
                if let Err(e) = result {
                    log::error!("[bg] background loop panicked: {:?}", e);
                }
            });

            // PTY 管理器
            let (pty_manager, _pty_rx) = commands::pty::PtyManager::new();
            let pty_manager = Arc::new(pty_manager);

            // 权限管理器
            let permission_manager = Arc::new(PermissionManager::new());

            // 文件同步状态
            let sync_state: commands::SyncState = Arc::new(Mutex::new(None));
            let sync_state_bg = sync_state.clone();

            // X 自动浏览状态
            let x_scroll_state = commands::XAutoScrollState::new();

            // 凭据管理器（AES-256-GCM 加密 + 空闲锁）
            let credential_state = commands::CredentialState::new();

            // 插件管理器（lazy-init, 首次开关触发 discover）
            let plugin_manager = Mutex::new(crate::plugins::manager::PluginManager::new());

            tauri::Builder::default()
                .plugin(tauri_plugin_shell::init())
                .plugin(tauri_plugin_dialog::init())
                .plugin(tauri_plugin_deep_link::init())
                .plugin(tauri_plugin_notification::init())
                .plugin(tauri_plugin_http::init())
                .plugin(tauri_plugin_fs::init())
                .plugin(tauri_plugin_updater::Builder::new().build::<tauri::Wry>())
                .manage(agent.clone())
                .manage(stats_snapshot.clone())
                .manage(ci_pending_input.clone())
                .manage(ci_response_output.clone())
                .manage(pty_manager.clone())
                .manage(permission_manager)
                .manage(sync_state)
                .manage(x_scroll_state)
                .manage(credential_state)
                .manage(plugin_manager)
                .invoke_handler(tauri::generate_handler![
                    commands::get_brain_stats, commands::absorb_source,
                    commands::session_list, commands::session_create,
                    commands::agent_reason,
                    commands::read_dir_recursive, commands::read_file, commands::write_file, commands::detect_project,
                    pty_spawn, pty_write, pty_resize, pty_close,
                    permission_dialog::request_permission,
                    permission_dialog::respond_permission,
                    permission_dialog::get_pending_permissions,
                    permission_dialog::get_permission_audit_log,
                    commands::cmd_project_open,
                    commands::cmd_scan_files,
                    commands::cmd_session_create,
                    commands::cmd_session_list,
                    commands::cmd_session_switch,
                    commands::cmd_session_delete,
                    commands::cmd_session_fork,
                    commands::cmd_session_export_json,
                    commands::cmd_session_import_json,
                    commands::cmd_agent_start,
                    commands::cmd_agent_stop,
                    commands::cmd_agent_status,
                    commands::cmd_diff_staged,
                    commands::cmd_diff_unstaged,
                    commands::cmd_diff_file,
                    commands::cmd_permission_request,
                    commands::cmd_permission_approve,
                    commands::cmd_permission_deny,
                    commands::brain_stats,
                    commands::brain_write_back,
                    commands::read_consciousness_response,
                    commands::search_knowledge,
                    #[cfg(feature = "stealth-net")]
                    commands::proxy_status,
                    #[cfg(feature = "stealth-net")]
                    commands::proxy_set_mode,
                    #[cfg(feature = "stealth-net")]
                    commands::proxy_start_daemon,
                    #[cfg(feature = "stealth-net")]
                    commands::proxy_stop_daemon,
                    commands::sync_init,
                    commands::sync_discover,
                    commands::sync_add_pair,
                    commands::sync_remove_pair,
                    commands::sync_list_pairs,
                    commands::sync_preview,
                    commands::sync_start,
                    commands::sync_status,
                    commands::browser_credential_store,
                    commands::browser_credential_list,
                    commands::browser_credential_remove,
                    commands::browser_credential_autofill,
                    commands::browser_credential_lock,
                    commands::browser_credential_unlock,
                    commands::browser_credential_health_check,
                    commands::browser_credential_audit_log,
                    commands::browser_credential_clear_audit_log,
                    commands::browser_credential_rotate_key,
                    commands::browser_x_start_session,
                    commands::browser_x_login,
                    commands::browser_x_human_scroll,
                    commands::browser_x_stop_session,
                    commands::browser_x_status,
                    commands::browser_x_human_profile,
                    commands::plugin_list,
                    commands::plugin_load,
                    commands::plugin_unload,
                    commands::plugin_uninstall,
                    commands::plugin_install_from_zip,
                    commands::plugin_get_info,
                    commands::plugin_write_data,
                    commands::plugin_read_data,
                    commands::save_provider_config,
                    commands::get_current_provider,
                    commands::test_provider,
                    #[cfg(feature = "extra-commands")]
                    commands::feed_refresh,
                    #[cfg(feature = "extra-commands")]
                    commands::feed_search,
                    #[cfg(feature = "extra-commands")]
                    commands::feed_insight,
                    #[cfg(feature = "extra-commands")]
                    commands::feed_timeline_summary,
                    #[cfg(feature = "extra-commands")]
                    commands::feed_stream_start,
                    #[cfg(feature = "extra-commands")]
                    commands::feed_stream_stop,
                    #[cfg(feature = "extra-commands")]
                    commands::feed_get_tags,
                    #[cfg(feature = "extra-commands")]
                    commands::im_list_adapters,
                    #[cfg(feature = "extra-commands")]
                    commands::im_get_adapter,
                    #[cfg(feature = "extra-commands")]
                    commands::im_save_adapter,
                    #[cfg(feature = "extra-commands")]
                    commands::im_delete_adapter,
                    #[cfg(feature = "extra-commands")]
                    commands::im_toggle_adapter,
                    #[cfg(feature = "extra-commands")]
                    commands::im_connect_adapter,
                    #[cfg(feature = "extra-commands")]
                    commands::im_disconnect_adapter,
                    #[cfg(feature = "extra-commands")]
                    commands::remote_start,
                    #[cfg(feature = "extra-commands")]
                    commands::remote_get_qr,
                    #[cfg(feature = "extra-commands")]
                    commands::remote_status,
                    #[cfg(feature = "extra-commands")]
                    commands::remote_poll,
                    #[cfg(feature = "extra-commands")]
                    commands::remote_send,
                    #[cfg(feature = "extra-commands")]
                    commands::remote_stop,
                    #[cfg(feature = "extra-commands")]
                    commands::sandbox_execute,
                    #[cfg(feature = "extra-commands")]
                    commands::sandbox_status,
                    commands::get_consciousness_dashboard,
                    commands::get_consciousness_full,
                    commands::get_e8_attention,
                    #[cfg(feature = "extra-commands")]
                    commands::tool_execute,
                    #[cfg(feature = "extra-commands")]
                    commands::tool_search,
                ])
                .setup(move |app| {
                    neotrix_tauri::setup_tray(app).expect("failed to setup tray");
                    #[cfg(debug_assertions)]
                    {
                        if let Some(window) = app.get_webview_window("main") { window.open_devtools(); }
                    }
                    // Start consciousness stats streaming
                    let streaming_snapshot = stats_snapshot.clone();
                    let streaming_running = Arc::new(AtomicBool::new(true));
                    let streaming_handle = app.handle().clone();
                    std::thread::spawn(move || {
                        let rt = match tokio::runtime::Runtime::new() {
                            Ok(rt) => rt,
                            Err(e) => { log::error!("[stream] tokio: {}", e); return; }
                        };
                        rt.block_on(async {
                            let mut ticker = tokio::time::interval(Duration::from_secs(10));
                            ticker.tick().await;
                            while streaming_running.load(Ordering::SeqCst) {
                                ticker.tick().await;
                                let s = match streaming_snapshot.read() {
                                    Ok(g) => g.clone(),
                                    Err(_) => continue,
                                };
                                let event = serde_json::json!({
                                    "cycle": s.cycle, "c_score": s.c_score,
                                    "coherence": s.sp_coherence, "emotion": s.emotion,
                                    "reflexivity": s.reflexivity,
                                    "vsa_buffer_size": s.vsa_buffer_size,
                                    "load_mode": match s.load_mode { 0 => "idle", _ => "active" },
                                    "critic_pass_rate": s.critic_pass_rate,
                                });
                                let _ = streaming_handle.emit("consciousness-tick", event);
                            }
                        });
                    });
                    // Start canvas snapshot streaming (only emits when there are nodes)
                    let canvas_handle = app.handle().clone();
                    let canvas_snapshot_stream = canvas_snapshot.clone();
                    std::thread::spawn(move || {
                        let rt = match tokio::runtime::Runtime::new() {
                            Ok(rt) => rt,
                            Err(e) => { log::error!("[canvas] tokio: {}", e); return; }
                        };
                        rt.block_on(async {
                            let mut ticker = tokio::time::interval(Duration::from_secs(5));
                            ticker.tick().await;
                            loop {
                                ticker.tick().await;
                                let proj = match canvas_snapshot_stream.read() {
                                    Ok(g) => g.clone(),
                                    Err(_) => continue,
                                };
                                if proj.nodes.is_empty() { continue; }
                                let event = serde_json::json!({
                                    "nodes": proj.nodes,
                                    "edges": proj.edges,
                                    "node_count": proj.nodes.len(),
                                    "edge_count": proj.edges.len(),
                                });
                                let _ = canvas_handle.emit("canvas-update", event);
                            }
                        });
                    });
                    let agent = app.state::<Arc<RwLock<SelfIteratingBrain>>>();
                    let agent_guard = agent.blocking_read();
                    log::info!("通知插件已就绪 (feature: notification)");
                    log::info!("✅ NeoTrix V2 Desktop ready (v0.18.0)");
                    log::info!("   记忆: {} | 能力: {} 维 | PTY: 就绪",
                        agent_guard.reasoning_bank.memories().len(),
                        agent_guard.brain.capability.total_dim());
                    drop(agent_guard);
 
                    // 自动文件同步后台线程（120 秒周期）
                    let sync_state_auto = sync_state_bg.clone();
                    let handle_auto = app.handle().clone();
                    std::thread::spawn(move || {
                        loop {
                            std::thread::sleep(Duration::from_secs(120));
                            auto_sync_cycle(&sync_state_auto, &handle_auto);
                        }
                    });
 
                    // 监听系统托盘 "Sync Now" 事件
                    let sync_state_trigger = sync_state_bg.clone();
                    let handle_trigger = app.handle().clone();
                    let handle_trigger_listener = handle_trigger.clone();
                    handle_trigger_listener.listen("sync-trigger", move |_| {
                        let state = sync_state_trigger.clone();
                        let handle = handle_trigger.clone();
                        std::thread::spawn(move || {
                            auto_sync_cycle(&state, &handle);
                        });
                    });
 
                    Ok(())
                })
                .run(tauri::generate_context!())
                .expect("tauri::Builder::run failed on second entry point");
        }
        Some(Commands::Headless) => {
            log::info!("NeoTrix headless mode - not yet implemented");
        }
        Some(Commands::Reason { prompt: _prompt }) => {
            log::info!("Reasoning not yet implemented in headless mode");
        }
    }
}
