//! NeoTrix Tauri V2 Desktop
//!
//! V2 架构: 完整的桌面端入口 (Brain + PTY 终端)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

use clap::Parser;
use tauri::{Manager, State, Emitter, Listener};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
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
fn pty_spawn(state: State<'_, Arc<commands::pty::PtyManager>>, cols: u16, rows: u16) -> Result<String, NeoTrixError> {
    let session_id = format!("pty-{}", uuid::Uuid::new_v4());
    state.spawn(&session_id, cols, rows).map_err(|e| NeoTrixError::Io(e.to_string()))?;
    Ok(session_id)
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

fn main() {
    // MCP stdio 子进程入口: 父进程 mcp_host_start 以 NEOTRIX_MCP_STDIO=1 拉起本进程,
    // 必须最先拦截, 避免 clap 解析 / GUI 启动。
    if std::env::var("NEOTRIX_MCP_STDIO").as_deref() == Ok("1") {
        commands::mcp_host_cmds::run_mcp_stdio();
        return;
    }

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
            let (pty_manager, pty_rx) = commands::pty::PtyManager::new();
            let pty_manager = Arc::new(pty_manager);

            // 权限管理器
            let permission_manager = Arc::new(PermissionManager::new());

            // 用户画像蒸馏引擎
            let distillation_engine = Mutex::new(DistillationEngine::new());

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
                .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.set_focus();
                    }
                }))
                .plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(|app, shortcut, event| {
                            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                                // C10: 全局呼出 — 显示并聚焦主窗口, 再通知前端聚焦输入框
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.unminimize();
                                    let _ = window.set_focus();
                                }
                                let _ = app.emit("neotrix-global-shortcut", shortcut.to_string());
                            }
                        })
                        .build(),
                )
                .manage(reasoning_bank)
                .manage(reasoning_brain)
                .manage(pty_manager.clone())
                .manage(permission_manager)
                .manage(distillation_engine)
                .invoke_handler(tauri::generate_handler![
                    commands::get_brain_stats, commands::absorb_source,
                    commands::session_list, commands::session_create,
                    commands::agent_reason,
                    commands::read_dir_recursive, commands::read_file, commands::write_file, commands::detect_project,
                    commands::cmd_project_open, commands::cmd_scan_files,
                    pty_spawn, pty_write, pty_resize, pty_close,
                    permission_dialog::request_permission,
                    permission_dialog::respond_permission,
                    permission_dialog::get_pending_permissions,
                    permission_dialog::get_permission_audit_log,
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
                    commands::cmd_diff_changed_files,
                    commands::cmd_diff_file,
                    commands::cmd_diff_stage,
                    commands::cmd_diff_unstage,
                    commands::cmd_diff_restore,
                    commands::cmd_diff_commit,
                    commands::cmd_diff_base,
                    commands::cmd_diff_base_files,
                    commands::cmd_diff_review,
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
                    // Project management commands
                    commands::project_list,
                    commands::project_create,
                    commands::project_get,
                    commands::project_update,
                    commands::project_delete,
                    commands::project_chat_list,
                    commands::project_chat_create,
                    commands::project_chat_update,
                    commands::project_chat_delete,
                    commands::project_source_list,
                    commands::project_source_add,
                    commands::project_source_update,
                    commands::project_source_delete,
                    commands::project_instruction_list,
                    commands::project_instruction_add,
                    commands::project_instruction_update,
                    commands::project_instruction_delete,
                    commands::project_scan_directory,
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
                    // Remote bridge (phone/browser pairing)
                    commands::remote_bridge_status,
                    commands::remote_bridge_pair,
                    commands::remote_bridge_connect,
                    commands::remote_bridge_disconnect,
                    commands::remote_bridge_send,
                    commands::remote_bridge_broadcast,
                    commands::remote_bridge_poll,
                    commands::remote_bridge_devices,
                    commands::remote_bridge_history,
                    commands::provider_status,
                    commands::save_api_key,
                    commands::has_api_key,
                    commands::delete_api_key,
                     commands::send_message,
                     commands::stop_generation,
                     commands::list_conversations,
                     commands::neocodex_send_message_stream,
                     commands::neocodex_health_report,
                     commands::neocodex_agent_status,
                     commands::neocodex_set_mode,
                     commands::neocodex_provider_config,
                     commands::neocodex_set_provider,
                     commands::neocodex_list_sessions,
                     commands::neocodex_search_sessions,
                      commands::neocodex_mcp_register,
                      commands::neocodex_mcp_list,
                      commands::neocodex_mcp_tools,
                      commands::neocodex_create_session,
                      commands::neocodex_get_session_messages,
                      commands::neocodex_switch_session,
                      commands::neocodex_delete_session,
                      commands::neocodex_rename_session,
                      commands::neocodex_send_side_chat,
                      commands::neocodex_get_side_chat,
                      commands::neocodex_init_project,
                      commands::neocodex_export_session,
                      commands::neocodex_clear_session,
                      commands::neocodex_feedback,
                      commands::neocodex_app_version,
                      commands::neocodex_check_update,
                      commands::neocodex_download_update,
                      commands::neocodex_git_status,
                      commands::neocodex_open_file,
                      commands::neocodex_open_external,
                      commands::neocodex_git_file_status,
                      commands::neocodex_file_operation,
                      commands::neocodex_get_diff,
                      commands::neocodex_apply_diff,
                      commands::neocodex_edit_message,
                      commands::neocodex_delete_message,
                      commands::neocodex_regenerate,
                      commands::neocodex_compact_session,
                      commands::neocodex_search_files,
                      commands::neocodex_stop_stream,
                      commands::neocodex_archive_session,
                      commands::neocodex_restore_session,
                      commands::neocodex_list_archived,
                      commands::neocodex_checkpoint_list,
                      commands::neocodex_checkpoint_restore,
commands::neocodex_set_project,
                       commands::neocodex_get_project,
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
                    commands::computer_cmds::capture_screen,
                    commands::computer_cmds::get_window_list,
                    commands::computer_cmds::get_frontmost_app,
                    // Desktop helpers (clipboard / image / app-switch / ultra review)
                    commands::desktop_cmds::read_clipboard,
                    commands::desktop_cmds::write_clipboard,
                    commands::desktop_cmds::image_generate,
                    commands::desktop_cmds::switch_app,
                    commands::desktop_cmds::ultra_review,
                    commands::desktop_cmds::window_minimize,
                    commands::desktop_cmds::window_maximize,
                    commands::desktop_cmds::window_close,
                    commands::desktop_cmds::window_is_maximized,
                    // Computer interactive (screen capture + mouse/keyboard)
                    commands::computer_screen_capture,
                    commands::computer_screen_list,
                    commands::computer_get_frontmost_app,
                    commands::computer_get_window_list,
                    commands::computer_mouse_move,
                    commands::computer_mouse_click,
                    commands::computer_mouse_position,
                    commands::computer_keyboard_type,
                    commands::computer_keyboard_press,
                    commands::computer_screenshot_and_save,
                    commands::background_cmds::list_background_tasks,
                    commands::background_cmds::create_background_task,
                    commands::background_cmds::pause_background_task,
                    commands::background_cmds::resume_background_task,
                    commands::background_cmds::delete_background_task,
                    commands::background_cmds::run_background_task_now,
                    commands::background_cmds::get_background_task_log,
                    commands::remote_cmds::list_remote_hosts,
                    commands::remote_cmds::add_remote_host,
                    commands::remote_cmds::remove_remote_host,
                    commands::remote_cmds::test_remote_connection,
                    commands::remote_cmds::execute_remote,
                    // Coordinator mode (multi-agent)
                    commands::coordinator_spawn,
                    commands::coordinator_list,
                    commands::coordinator_update,
                    commands::coordinator_remove,
                    commands::coordinator_set_max_workers,
                    commands::coordinator_set_strategy,
                    // PR Review pipeline
                    commands::review_diff,
                    commands::review_get_issue_detail,
                    // Skill management
                    commands::skill_list,
                    commands::skill_get,
                    commands::skill_read,
                    commands::skill_search,
                    // Buddy companion (AI pet)
                    commands::buddy_status,
                    commands::buddy_pet,
                    commands::buddy_feed,
                    commands::buddy_train,
                    commands::buddy_rest,
                    commands::buddy_achievements,
                    commands::buddy_idle_tick,
                    commands::buddy_log,
                    // KAIROS daemon (background file watcher + auto-fix)
                    commands::daemon_start,
                    commands::daemon_stop,
                    commands::daemon_status,
                    commands::daemon_log,
                    commands::daemon_auto_fix,
                    // Auto-Dream memory consolidation
                    commands::dream_start,
                    commands::dream_stop,
                    commands::dream_status,
                    commands::dream_entries,
                    commands::dream_consolidate_now,
                    // Self-checking gate (pre-submit verification)
                    commands::gate_run_check,
                    commands::gate_set_config,
                    commands::gate_get_config,
                    commands::gate_set_policy,
                    commands::gate_get_policy,
                    commands::gate_approve,
                    commands::gate_audit_log,
                    // Session summary pane (plans/artifacts/sources)
                    commands::summary_active,
                    commands::summary_start,
                    commands::summary_pause,
                    commands::summary_resume,
                    commands::summary_plans,
                    commands::summary_artifacts,
                    commands::summary_sources,
                    commands::summary_add_artifact,
                    commands::summary_add_source,
                    commands::summary_add_plan,
                    // Undercover mode (git identity management)
                    commands::undercover_status,
                    commands::undercover_set_profile,
                    commands::undercover_get_profiles,
                    commands::undercover_activate_profile,
                    commands::undercover_strip_metadata,
                    commands::undercover_commit_log,
                    commands::undercover_verify_anonymity,
                    // MCP Server Hosting
                    commands::mcp_host_start,
                    commands::mcp_host_stop,
                    commands::mcp_host_status,
                    commands::mcp_host_ping,
                    commands::mcp_host_list_endpoints,
                    commands::mcp_host_register_endpoint,
                    commands::mcp_host_unregister_endpoint,
                    commands::mcp_host_sessions,
                    commands::mcp_host_log,
                    // Plugin System
                    commands::plugin_list,
                    commands::plugin_install,
                    commands::plugin_uninstall,
                    commands::plugin_enable,
                    commands::plugin_disable,
                    commands::plugin_get,
                    commands::plugin_config,
                    commands::plugin_set_config,
                    commands::plugin_event_log,
                    commands::plugin_run,
                    // Plugin Marketplace
                    commands::marketplace_list,
                    commands::marketplace_search,
                    commands::marketplace_get,
                    commands::marketplace_install,
                    commands::marketplace_uninstall,
                    commands::marketplace_update,
                    commands::marketplace_check_updates,
                    commands::marketplace_update_all,
                    commands::marketplace_reviews,
                    commands::marketplace_submit_review,
                    commands::marketplace_categories,
                    commands::marketplace_stats,
                    commands::marketplace_config,
                    commands::marketplace_set_config,
                    commands::marketplace_featured,
                    // Web Search + Agent SDK
                    commands::web_search,
                    commands::web_search_config,
                    commands::web_search_set_config,
                    commands::agent_sdk_create_blueprint,
                    commands::agent_sdk_list_blueprints,
                    commands::agent_sdk_get_blueprint,
                    commands::agent_sdk_delete_blueprint,
                    commands::agent_sdk_run,
                    commands::agent_sdk_list_instances,
                    commands::agent_sdk_get_result,
                    // Channels (Telegram/Discord/Webhook/Slack)
                    commands::channels_list,
                    commands::channels_add,
                    commands::channels_remove,
                    commands::channels_enable,
                    commands::channels_disable,
                    commands::channels_send,
                    commands::channels_receive,
                    commands::channels_link_session,
                    commands::channels_unlink_session,
                    commands::channels_history,
                    commands::slack_config,
                    commands::slack_configure,
                    commands::slack_send,
                    commands::slack_status,
                    // App Previews + Chrome Debug
                    commands::preview_start,
                    commands::preview_stop,
                    commands::preview_list,
                    commands::preview_navigate,
                    commands::preview_reload,
                    commands::preview_screenshot,
                    commands::preview_config,
                    commands::preview_set_config,
                    commands::chrome_debug_connect,
                    commands::chrome_debug_disconnect,
                    commands::chrome_debug_status,
                    commands::chrome_debug_list_targets,
                    commands::chrome_debug_navigate,
                    commands::chrome_debug_reload,
                    commands::chrome_debug_evaluate,
                    commands::chrome_debug_get_console_logs,
                    commands::chrome_debug_clear_console_logs,
                    commands::chrome_debug_capture_screenshot,
                    // Teleport (cross-surface session migration)
                    commands::teleport_create,
                    commands::teleport_claim,
                    commands::teleport_list,
                    commands::teleport_config,
                    commands::teleport_set_config,
                    commands::teleport_revoke,
                    // Agent Teams
                    commands::agent_team_create,
                    commands::agent_team_list,
                    commands::agent_team_get,
                    commands::agent_team_add_member,
                    commands::agent_team_remove_member,
                    commands::agent_team_start,
                    commands::agent_team_complete_member,
                    commands::agent_team_fail_member,
                    commands::agent_team_status,
                    commands::agent_team_messages,
                    commands::agent_team_send_message,
                    commands::agent_team_result,
                    // Dynamic Workflows
                    commands::workflow_create,
                    commands::workflow_list,
                    commands::workflow_get,
                    commands::workflow_update,
                    commands::workflow_delete,
                    commands::workflow_run,
                    commands::workflow_run_status,
                    commands::workflow_run_list,
                    commands::workflow_run_steps,
                    commands::workflow_run_cancel,
                    commands::workflow_schedule_create,
                    commands::workflow_schedule_list,
                    commands::workflow_schedule_delete,
                    commands::workflow_import_from_json,
                    // Enterprise Configuration + Real API
                    commands::enterprise_status,
                    commands::enterprise_list_policies,
                    commands::enterprise_set_policy,
                    commands::enterprise_delete_policy,
                    commands::enterprise_audit_log,
                    commands::enterprise_audit_log_action,
                    commands::enterprise_compliance_check,
                    commands::enterprise_license_info,
                    commands::api_register,
                    commands::api_list,
                    commands::api_test,
                    commands::api_delete,
                    commands::api_config,
                    commands::api_set_config,
                    commands::api_call,
                    // Agent View Dashboard
                    commands::agent_view_summary,
                    commands::agent_view_list,
                    commands::agent_view_get,
                    commands::agent_view_events,
                    commands::agent_view_pause,
                    commands::agent_view_resume,
                    commands::agent_view_cancel,
                    commands::agent_view_config,
                    commands::agent_view_set_config,
                    commands::agent_view_tick,
                    // Scheduled Loop (cron-style recurring execution)
                    commands::loop_create,
                    commands::loop_list,
                    commands::loop_get,
                    commands::loop_update,
                    commands::loop_delete,
                    commands::loop_enable,
                    commands::loop_disable,
                    commands::loop_execute_now,
                    commands::loop_execution_history,
                    commands::loop_stats,
                    commands::loop_next_scheduled,
                    commands::loop_validate_cron,
                    commands::loop_tick,
                    // Memory Management
                    commands::memory_list,
                    commands::memory_get,
                    commands::memory_search,
                    commands::memory_create,
                    commands::memory_update,
                    commands::memory_delete,
                    commands::memory_pin,
                    commands::memory_unpin,
                    commands::memory_categories,
                    commands::memory_stats,
                    commands::memory_timeline,
                    commands::memory_consolidate_now,
                    commands::memory_clear,
                    commands::memory_export,
                    commands::memory_import,
                    commands::memory_config,
                    commands::memory_set_config,
                    // Browser Page Annotations
                    commands::annotation_create,
                    commands::annotation_list,
                    commands::annotation_get,
                    commands::annotation_update,
                    commands::annotation_delete,
                    commands::annotation_resolve,
                    commands::annotation_unresolve,
                    commands::annotation_collection_create,
                    commands::annotation_collection_get,
                    commands::annotation_collection_list,
                    commands::annotation_collection_delete,
                    commands::annotation_stats,
                    commands::annotation_config,
                    commands::annotation_set_config,
                    commands::annotation_get_for_url,
                    commands::annotation_search,
                    // Voice Mode
                    commands::voice_start_session,
                    commands::voice_stop_session,
                    commands::voice_session_status,
                    commands::voice_send_audio,
                    commands::voice_get_transcription,
                    commands::voice_list_sessions,
                    commands::voice_session_history,
                    commands::voice_synthesize,
                    commands::voice_config,
                    commands::voice_set_config,
                    commands::voice_test_microphone,
                    commands::voice_stats,
                    commands::voice_execute_command,
                    // Security Scanning
                    commands::security_scan_start,
                    commands::security_scan_status,
                    commands::security_scan_list,
                    commands::security_scan_findings,
                    commands::security_scan_finding_detail,
                    commands::security_scan_apply_patch,
                    commands::security_scan_mark_status,
                    commands::security_scan_config,
                    commands::security_scan_set_config,
                    commands::security_scan_summary,
                    commands::security_scan_quick_check,
                    commands::security_scan_fix_all,
                    // Context Compaction
                    commands::context_analyze,
                    commands::context_compact,
                    commands::context_get_segments,
                    commands::context_get_segment,
                    commands::context_expand,
                    commands::context_config,
                    commands::context_set_config,
                    commands::context_stats,
                    commands::context_summarize,
                    commands::context_extract_decisions,
                    commands::context_check_threshold,
                    // Profile System (named configuration profiles)
                    commands::profile_create,
                    commands::profile_list,
                    commands::profile_get,
                    commands::profile_update,
                    commands::profile_delete,
                    commands::profile_activate,
                    commands::profile_duplicate,
                    commands::profile_reset,
                    commands::profile_export,
                    commands::profile_import,
                    commands::profile_summary,
                    commands::profile_templates,
                    // Activity Insights & Usage Cards
                    commands::insights_record_event,
                    commands::insights_daily,
                    commands::insights_weekly,
                    commands::insights_insights,
                    commands::insights_generate_card,
                    commands::insights_card_list,
                    commands::insights_card_get,
                    commands::insights_card_share,
                    commands::insights_trend,
                    commands::insights_config,
                    commands::insights_set_config,
                    commands::insights_stats,
                    commands::insights_reset,
                    // Multi-Terminal Tabs per Thread
                    commands::term_tabs_create,
                    commands::term_tabs_list,
                    commands::term_tabs_get,
                    commands::term_tabs_rename,
                    commands::term_tabs_close,
                    commands::term_tabs_activate,
                    commands::term_tabs_reorder,
                    commands::term_tabs_set_color,
                    commands::term_tabs_layout,
                    commands::term_tabs_set_layout,
                    commands::term_tabs_group_create,
                    commands::term_tabs_group_list,
                    commands::term_tabs_group_delete,
                    commands::term_tabs_config,
                    commands::term_tabs_set_config,
                    commands::term_tabs_stats,
                    // Cowork file/folder productivity mode
                    commands::cowork_start,
                    commands::cowork_list,
                    commands::cowork_get,
                    commands::cowork_status,
                    commands::cowork_pause,
                    commands::cowork_resume,
                    commands::cowork_stop,
                    commands::cowork_scan_files,
                    commands::cowork_read_file,
                    commands::cowork_write_file,
                    commands::cowork_delete_file,
                    commands::cowork_list_deliverables,
                    commands::cowork_get_deliverable,
                    commands::cowork_templates,
                    commands::cowork_apply_template,
                    commands::cowork_actions,
                    commands::cowork_config,
                    commands::cowork_set_config,
                    commands::cowork_stats,
                    commands::cowork_export_session,
                    // Unified Session Overview
                    commands::unified_session_list,
                    commands::unified_session_get,
                    commands::unified_session_summary,
                    commands::unified_session_group_by,
                    commands::unified_session_search,
                    commands::unified_session_stats,
                    commands::unified_session_connect,
                    commands::unified_session_disconnect,
                    commands::unified_session_tag,
                    commands::unified_session_untag,
                    commands::unified_session_export,
                    commands::unified_session_import,
                    commands::unified_session_refresh,
                    // Unified Command Bridge (CLI ↔ NoeCodex)
                    commands::unified_command_catalog,
                    commands::unified_cli_execute,
                    commands::unified_cli_list,
                    commands::unified_cli_lookup,
                    commands::unified_tauri_list,
                ])
                .setup(move |app| {
                    if let Err(e) = neotrix_tauri::setup_tray(app) {
                        log::warn!("failed to setup tray: {}", e);
                    }
                    let _ = neotrix_tauri::setup_menu(app);

                    if let Err(e) = app.global_shortcut().register("CommandOrControl+Shift+Space") {
                        log::warn!("failed to register global shortcut: {}", e);
                    }

                    // PTY 事件转发: mpsc → Tauri 事件 (pty-output-{id} / pty-exit-{id})
                    let pty_handle = app.handle().clone();
                    tauri::async_runtime::spawn(async move {
                        let mut rx = pty_rx;
                        while let Some(evt) = rx.recv().await {
                            match evt.event_type {
                                commands::pty::PtyEventType::Output => {
                                    let _ = pty_handle.emit(&format!("pty-output-{}", evt.session_id), &evt.data);
                                }
                                commands::pty::PtyEventType::Exit(code) => {
                                    let _ = pty_handle.emit(&format!("pty-exit-{}", evt.session_id), &code);
                                }
                            }
                        }
                    });
                    let _ = commands::insights_record_event(
                        "session_start".to_string(),
                        format!("NeoTrix Desktop session started (v{})", env!("CARGO_PKG_VERSION")),
                        None,
                        None,
                        None,
                    );
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

                    Ok(())
                })
                .build(tauri::generate_context!())
                .expect("error while building tauri application")
                .run(|_app, event| {
                    if let tauri::RunEvent::ExitRequested { .. } = event {
                        let _ = commands::insights_record_event(
                            "session_end".to_string(),
                            "NeoTrix Desktop session ended".to_string(),
                            None,
                            None,
                            None,
                        );
                    }
                });
        }
        Some(Commands::Headless) => {
            println!("NeoTrix headless mode - not yet implemented");
        }
        Some(Commands::Reason { prompt: _prompt }) => {
            println!("Reasoning not yet implemented in headless mode");
        }
    }
}
