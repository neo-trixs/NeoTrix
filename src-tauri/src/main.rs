//! NeoTrix Tauri Desktop - Unified API Version
//!
//! 简化的桌面端入口，通过统一 API 与意识核心交互
//! 当 neotrix crate 编译失败时使用 stub 类型

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

use clap::Parser;
use tauri::{Manager, State, Emitter};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::stub::UnifiedApiImpl;
use crate::commands::unified::{UnifiedApiState, unified_init, unified_chat, unified_chat_stream, unified_system_state, unified_create_session, unified_list_sessions, unified_delete_session, unified_exec_cli, unified_cli_list};

mod commands;
mod stub;

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

fn updater_enabled() -> bool {
    match std::env::var("NEOTRIX_UPDATER").as_deref() {
        Ok("0") => false,
        Ok("1") => true,
        _ => cfg!(not(debug_assertions)),
    }
}

fn main() {
    if std::env::var("NEOTRIX_MCP_STDIO").as_deref() == Ok("1") {
        return;
    }

    // stub: 不初始化 sentry
    let _sentry_guard = crate::stub::init_sentry();
    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Desktop) => {
            let unified_api: UnifiedApiState = Arc::new(RwLock::new(UnifiedApiImpl::new()));

            let (pty_manager, pty_rx) = crate::commands::pty::PtyManager::new();
            let pty_manager = Arc::new(pty_manager);

            let builder = tauri::Builder::default()
                .plugin(tauri_plugin_shell::init())
                .plugin(tauri_plugin_dialog::init())
                .plugin(tauri_plugin_deep_link::init())
                .plugin(tauri_plugin_notification::init())
                .plugin(tauri_plugin_http::init())
                .plugin(tauri_plugin_fs::init());

            let builder = if updater_enabled() {
                builder.plugin(tauri_plugin_updater::Builder::new().build::<tauri::Wry>())
            } else {
                log::info!("[boundary] updater disabled (NEOTRIX_UPDATER unset + debug build)");
                builder
            };

            builder
                .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.set_focus();
                    }
                }))
                .plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(|app, shortcut, event| {
                            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
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
                .manage(unified_api.clone())
                .manage(pty_manager.clone())
                .invoke_handler(tauri::generate_handler![
                    unified_init,
                    unified_chat,
                    unified_chat_stream,
                    unified_system_state,
                    unified_create_session,
                    unified_list_sessions,
                    unified_delete_session,
                    unified_exec_cli,
                    unified_cli_list,
                    crate::commands::pty::pty_spawn,
                    crate::commands::pty::pty_write,
                    crate::commands::pty::pty_resize,
                    crate::commands::pty::pty_close,
                ])
                .setup(move |app| {
                    // PTY 事件转发
                    let pty_handle = app.handle().clone();
                    tauri::async_runtime::spawn(async move {
                        let mut rx = pty_rx;
                        while let Some(evt) = rx.recv().await {
                            match evt.event_type {
                                crate::commands::pty::PtyEventType::Output => {
                                    let _ = pty_handle.emit(&format!("pty-output-{}", evt.session_id), &evt.data);
                                }
                                crate::commands::pty::PtyEventType::Exit(code) => {
                                    let _ = pty_handle.emit(&format!("pty-exit-{}", evt.session_id), &code);
                                }
                            }
                        }
                    });

                    #[cfg(debug_assertions)]
                    {
                        if let Some(window) = app.get_webview_window("main") { window.open_devtools(); }
                    }

                    println!("✅ NeoTrix V2 Desktop ready (unified API stub)");
                    println!("   统一接口: unified_chat / unified_chat_stream / unified_system_state");
                    println!("   PTY: 就绪");

                    Ok(())
                })
                .build(tauri::generate_context!())
                .expect("error while building tauri application")
                .run(|_app, event| {
                    if let tauri::RunEvent::ExitRequested { .. } = event {
                    }
                });
        }
        Some(Commands::Headless) => {
            println!("NeoTrix headless mode starting...");
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            rt.block_on(async {
                let api = UnifiedApiImpl::new();
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    let state = api.get_system_state().await;
                    println!("  [tick] phi={:.3} coherence={:.3}",
                        state.metadata.consciousness_state.phi,
                        state.metadata.consciousness_state.coherence);
                }
            });
        }
        Some(Commands::Reason { prompt }) => {
            println!("NeoTrix reasoning: {}", prompt);
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            rt.block_on(async {
                let api = UnifiedApiImpl::new();
                let request = crate::stub::UnifiedRequest::chat(prompt);
                let response = api.handle(request).await;
                match response {
                    Ok(r) => println!("Response: {}", r.content),
                    Err(e) => println!("Error: {} - {}", e.code, e.message),
                }
            });
        }
    }
}
