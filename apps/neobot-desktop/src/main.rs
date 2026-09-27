//! NeoBot — 独立本地 agent 桌面 App.
//!
//! 瘦壳: 唯一依赖是 `neotrix-neobot` (SQLite + fail-closed 网关 +
//! EngineAdapter). 不链接 `neotrix` 大内核, 不含 domain 插件/pty/模型池.
//! 数据与 CLI 同源 (`~/.neobot/neobot.db`), 桌面+CLI 可并用.

#![forbid(unsafe_code)]

mod nt_commands;

use nt_commands::{DaemonMap, PresenceMap};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            use tauri::Manager as _;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .manage(PresenceMap::default())
        .manage(DaemonMap::default())
        .invoke_handler(tauri::generate_handler![
            nt_commands::nt_cmd_sys::neobot_doctor,
            nt_commands::nt_cmd_run::neobot_run,
            nt_commands::nt_cmd_run::neobot_run_stream,
            nt_commands::nt_cmd_tasks::neobot_tasks,
            nt_commands::nt_cmd_tasks::neobot_audit,
            nt_commands::nt_cmd_sys::neobot_models,
            nt_commands::nt_cmd_sys::neobot_providers,
            nt_commands::nt_cmd_sys::neobot_provider_add,
            nt_commands::nt_cmd_sys::neobot_provider_remove,
            nt_commands::nt_cmd_sys::neobot_provider_toggle,
            nt_commands::nt_cmd_tasks::neobot_task_claim,
            nt_commands::nt_cmd_tasks::neobot_task_release,
            nt_commands::nt_cmd_tasks::neobot_task_visibility,
            nt_commands::nt_cmd_tasks::neobot_task_cancel,
            nt_commands::nt_cmd_tasks::neobot_task_retry,
            nt_commands::nt_cmd_tasks::neobot_task_rename,
            nt_commands::nt_cmd_tasks::neobot_task_delete,
            nt_commands::nt_cmd_convo::neobot_convos,
            nt_commands::nt_cmd_convo::neobot_convo_group,
            nt_commands::nt_cmd_convo::neobot_convo_dm,
            nt_commands::nt_cmd_convo::neobot_convo_ensure_default,
            nt_commands::nt_cmd_convo::neobot_convo_rename,
            nt_commands::nt_cmd_convo::neobot_convo_delete,
            nt_commands::nt_cmd_convo::neobot_convo_mute,
            nt_commands::nt_cmd_convo::neobot_convo_mark_read,
            nt_commands::nt_cmd_convo::neobot_convo_add_member,
            nt_commands::nt_cmd_convo::neobot_convo_rm_member,
            nt_commands::nt_cmd_convo::neobot_attach_add,
            nt_commands::nt_cmd_convo::neobot_attachments,
            nt_commands::nt_cmd_convo::neobot_attach_remove,
            nt_commands::nt_cmd_sys::neobot_memory_get,
            nt_commands::nt_cmd_sys::neobot_memory_set,
            nt_commands::nt_cmd_tasks::neobot_cost_ledger,
            nt_commands::nt_cmd_tasks::neobot_cost_ledger_by_actor,
            nt_commands::nt_cmd_core::neobot_core_status,
            nt_commands::nt_cmd_core::neobot_core_reload,
            nt_commands::nt_cmd_core::neobot_core_capabilities,
            nt_commands::nt_cmd_core::neobot_agent_run,
            nt_commands::nt_cmd_core::neobot_core_free,
            nt_commands::nt_cmd_core::neobot_core_pair_free,
            nt_commands::nt_cmd_sys::neobot_routine_list,
            nt_commands::nt_cmd_sys::neobot_routine_fire,
            nt_commands::nt_cmd_sys::neobot_routine_sweep,
            nt_commands::nt_cmd_sys::neobot_skills,
            nt_commands::nt_cmd_convo::neobot_members,
            nt_commands::nt_cmd_convo::neobot_member_add,
            nt_commands::nt_cmd_convo::neobot_member_remove,
            nt_commands::nt_cmd_sys::neobot_control_status,
            nt_commands::nt_cmd_sys::neobot_control_take,
            nt_commands::nt_cmd_sys::neobot_control_release,
            nt_commands::nt_cmd_sys::neobot_settings_window,
            nt_commands::nt_cmd_sys::neobot_roster_heartbeat,
            nt_commands::nt_cmd_sys::neobot_roster_list,
        ])
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .unwrap_or_else(|err| {
            eprintln!("NeoBot 启动失败: {err}");
            std::process::exit(1);
        })
        .run(|app, event| {
            // Cmd-Q/Dock Quit 转 Hide（macOS 惯例；优雅退出只走菜单外显路径）。
            // 自退 hunting 结论：未拦截 ExitRequested 是连环退主因（关窗已 Hide 化）。
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                use tauri::Manager as _;
                api.prevent_exit();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        });
}
