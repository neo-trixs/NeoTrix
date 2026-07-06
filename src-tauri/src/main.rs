//! NeoTrix Tauri V2 Desktop
//!
//! V2 架构: 完整的桌面端入口 (Brain + PTY 终端)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

use clap::Parser;
use tauri::{Manager, State, Emitter, Listener};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use neotrix::neotrix::nt_core_error::NeoTrixError;
use neotrix::neotrix::nt_mind::{ReasoningBank, ReasoningBrain};
use neotrix::neotrix::nt_shield::permissions::PermissionManager;
use neotrix::neotrix::nt_io_user_avatar::DistillationEngine;

mod commands;
mod permission_dialog;
mod anthropic;

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
fn pty_spawn(state: State<'_, Arc<commands::pty::PtyManager>>, session_id: String, cols: u16, rows: u16) -> Result<(), NeoTrixError> {
    state.spawn(&session_id, cols, rows).map_err(|e| NeoTrixError::Io(e.to_string()))
}

#[tauri::command]
fn pty_write(state: State<'_, Arc<commands::pty::PtyManager>>, session_id: String, data: String) -> Result<(), NeoTrixError> {
    state.write(&session_id, &data).map_err(|e| NeoTrixError::Io(e.to_string()))
}

#[tauri::command]
fn pty_resize(state: State<'_, Arc<commands::pty::PtyManager>>, session_id: String, cols: u16, rows: u16) -> Result<(), NeoTrixError> {
    state.resize(&session_id, cols, rows).map_err(|e| NeoTrixError::Io(e.to_string()))
}

#[tauri::command]
fn pty_close(state: State<'_, Arc<commands::pty::PtyManager>>, session_id: String) -> Result<(), NeoTrixError> {
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
            let _ = tray.set_tooltip(Some(format!(
                "NeoTrix Desktop — Last sync: {}",
                now.format("%H:%M:%S")
            )));
    }
}

fn main() {
    let _sentry_guard = neotrix::neotrix::nt_shield_sentry::init_sentry();
    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Desktop) => {
            let mut reasoning_bank = ReasoningBank::new(1000);
            let mut reasoning_brain = ReasoningBrain::new();
            reasoning_brain.initialize_with_design_knowledge(&mut reasoning_bank);

            let reasoning_bank = Mutex::new(reasoning_bank);
            let reasoning_brain = Mutex::new(reasoning_brain);

            // PTY 管理器
            let (pty_manager, _pty_rx) = commands::pty::PtyManager::new();
            let pty_manager = Arc::new(pty_manager);

            // 权限管理器
            let permission_manager = Arc::new(PermissionManager::new());

            // 用户画像蒸馏引擎
            let distillation_engine = Mutex::new(DistillationEngine::new());

            // 文件同步状态
            let sync_state: commands::SyncState = Arc::new(Mutex::new(None));
            let sync_state_bg = sync_state.clone();

            // LLM 提供者统一网关
            let _gateway = commands::provider_cmds::init_gateway();

            tauri::Builder::default()
                .plugin(tauri_plugin_shell::init())
                .plugin(tauri_plugin_dialog::init())
                .plugin(tauri_plugin_deep_link::init())
                .plugin(tauri_plugin_notification::init())
                .plugin(tauri_plugin_http::init())
                .plugin(tauri_plugin_fs::init())
                .plugin(tauri_plugin_updater::Builder::new().build::<tauri::Wry>())
                .manage(reasoning_bank)
                .manage(reasoning_brain)
                .manage(pty_manager.clone())
                .manage(permission_manager)
                .manage(distillation_engine)
                .manage(sync_state)
                .manage(commands::browser_cmds::WebAppState::new())
                .manage(commands::browser_cmds::XAutoScrollState::new())
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
                    commands::test_provider,
                    commands::save_provider_config,
                    commands::search_knowledge,
                    commands::get_knowledge_graph,
                    commands::get_knowledge_stats,
                    commands::kb_search,
                    commands::kb_get_node,
                    commands::kb_get_related,
                    commands::kb_feed,
                    commands::send_notification,
                    commands::get_user_avatar,
                    commands::get_distillation_flow,
                    commands::distill_message,
                    commands::set_user_identity,
                    commands::get_identity,
                    commands::get_chain_stats,
                    commands::brain_write_back,
                    commands::auto_distill,
                    commands::execute_terminal_command,
                    commands::cli_command,
                    commands::request_capability,
                    commands::check_auth,
                    commands::grant_capability,
                    commands::revoke_capability,
                    commands::proxy_status,
                    commands::proxy_set_mode,
                    commands::proxy_start_daemon,
                    commands::proxy_stop_daemon,
                    commands::proxy_source_status,
                    commands::proxy_connectivity,
                    commands::proxy_trigger_fetch,
                    commands::proxy_sub_list,
                    commands::proxy_sub_add,
                    commands::proxy_sub_remove,
                    commands::proxy_pool_nodes,
                    commands::proxy_config_get,
                    commands::proxy_config_set,
                    commands::sync_init,
                    commands::sync_discover,
                    commands::sync_add_pair,
                    commands::sync_remove_pair,
                    commands::sync_list_pairs,
                    commands::sync_preview,
                    commands::sync_start,
                    commands::sync_status,
                    commands::provider_status,
                    commands::save_api_key,
                    commands::has_api_key,
                    commands::delete_api_key,
                    commands::send_message,
                    commands::stop_generation,
                    commands::list_conversations,
                    commands::browser_cmds::browser_open,
                    commands::browser_cmds::browser_navigate,
                    commands::browser_cmds::browser_back,
                    commands::browser_cmds::browser_forward,
                    commands::browser_cmds::browser_reload,
                    commands::browser_cmds::browser_close,
                    commands::browser_cmds::browser_execute_js,
                    commands::browser_cmds::browser_agent_list,
                    commands::browser_cmds::browser_agent_detect,
                    commands::browser_cmds::browser_agent_execute,
                    commands::browser_cmds::browser_extract_content,
                    commands::browser_cmds::browser_ingest_content,
                    commands::browser_cmds::browser_collected_knowledge,
                    commands::browser_cmds::browser_x_start_session,
                    commands::browser_cmds::browser_x_login,
                    commands::browser_cmds::browser_x_auto_scroll,
                    commands::browser_cmds::browser_x_human_scroll,
                    commands::browser_cmds::browser_x_stop_session,
                    commands::browser_cmds::browser_x_status,
                    commands::browser_cmds::browser_x_human_profile,
                    commands::pet_cmds::get_pet_state,
                    commands::pet_cmds::feed_pet_conversation,
                    commands::pet_cmds::sync_pet_consciousness,
                    commands::tool_cmds::tool_execute,
                    commands::tool_cmds::tool_search,
                    commands::plan_cmds::plan_list,
                    commands::plan_cmds::plan_create,
                    commands::plan_cmds::plan_steps,
                    commands::plan_cmds::plan_step,
                    commands::plan_cmds::plan_complete,
                ])
                .setup(move |app| {
                    neotrix_tauri::setup_tray(app).expect("failed to setup tray");
                    #[cfg(debug_assertions)]
                    {
                        if let Some(window) = app.get_webview_window("main") { window.open_devtools(); }
                    }
                    let bank = app.state::<Mutex<ReasoningBank>>();
                    let brain = app.state::<Mutex<ReasoningBrain>>();
                    let bank = bank.lock().unwrap_or_else(|e| { log::warn!("ReasoningBank mutex poisoned, recovering"); e.into_inner() });
                    let brain = brain.lock().unwrap_or_else(|e| { log::warn!("ReasoningBrain mutex poisoned, recovering"); e.into_inner() });
                    log::info!("通知插件已就绪 (feature: notification)");
                    println!("✅ NeoTrix V2 Desktop ready (v0.18.0)");
                    println!("   记忆: {} | 能力: {} 维 | PTY: 就绪",
                        bank.stats().total_memories, brain.capability.total_dim());
                    drop(bank);
                    drop(brain);

                    // 分身定时自蒸馏（60-300s 随机间隔）
                    let handle = app.handle().clone();
                    std::thread::spawn(move || {
                        let mut rng_seed = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        loop {
                            let delay = 60 + (rng_seed % 241);
                            rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                            std::thread::sleep(std::time::Duration::from_secs(delay));
                            if let Some(engine) = handle.try_state::<Mutex<DistillationEngine>>() {
                                if let Ok(mut eng) = engine.lock() {
                                    eng.auto_distill();
                                    let _ = handle.emit("distillation-update", &eng.get_flow());
                                }
                            }
                        }
                    });
 
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
                    app.handle().listen("sync-trigger", move |_| {
                        let state = sync_state_trigger.clone();
                        let handle = handle_trigger.clone();
                        std::thread::spawn(move || {
                            auto_sync_cycle(&state, &handle);
                        });
                    });
 
                    Ok(())
                })
                .run(tauri::generate_context!())
                .expect("error while running tauri application");
        }
        Some(Commands::Headless) => {
            println!("NeoTrix headless mode - not yet implemented");
        }
        Some(Commands::Reason { prompt: _prompt }) => {
            println!("Reasoning not yet implemented in headless mode");
        }
    }
}
