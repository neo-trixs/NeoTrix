//! NeoTrix Tauri Desktop - Domain Plugin Architecture
//!
//! 基于 DeepSeek Harness 架构理念：一切皆插件，能力缝可替换。
//! 通过 DomainRegistry 统一管理 12 个功能域插件。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

use clap::Parser;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::RwLock;

mod commands;
mod domain;
pub mod market;
mod stub;

use crate::stub::{UnifiedApi as _, UnifiedApiImpl};
use commands::domain_cmd::{
    domain_action_count, domain_call, domain_has, domain_list, DomainState,
};
use commands::unified::{
    unified_chat, unified_chat_stream, unified_cli_list, unified_create_session,
    unified_delete_session, unified_exec_cli, unified_init, unified_list_sessions,
    unified_system_state, UnifiedApiState,
};
use domain::{plugins::*, DomainRegistry};
use market::commands::*;

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

    let _sentry_guard = crate::stub::init_sentry();
    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Desktop) => {
            // 创建域注册表并注册 12 个插件
            let mut registry = DomainRegistry::new();
            registry
                .register(Box::new(SessionPlugin::new()))
                .expect("failed to register session");
            registry
                .register(Box::new(AgentPlugin))
                .expect("failed to register agent");
            registry
                .register(Box::new(KbPlugin::new()))
                .expect("failed to register kb");
            registry
                .register(Box::new(FilePlugin))
                .expect("failed to register file");
            registry
                .register(Box::new(PluginPlugin))
                .expect("failed to register plugin");
            registry
                .register(Box::new(WorkflowPluginImpl::new()))
                .expect("failed to register workflow");
            registry
                .register(Box::new(ToolPlugin))
                .expect("failed to register tool");
            registry
                .register(Box::new(SystemPlugin))
                .expect("failed to register system");
            registry
                .register(Box::new(SecurityPlugin))
                .expect("failed to register security");
            registry
                .register(Box::new(MemoryPlugin::new()))
                .expect("failed to register memory");
            registry
                .register(Box::new(ExtPlugin))
                .expect("failed to register ext");
            registry
                .register(Box::new(LlamacppPlugin::new()))
                .expect("failed to register llamacpp");
            registry
                .register(Box::new(GitPlugin))
                .expect("failed to register git");
            registry
                .register(Box::new(CliPlugin))
                .expect("failed to register cli");
            registry
                .register(Box::new(WorldPlugin))
                .expect("failed to register world");
            registry
                .register(Box::new(ContextPlugin))
                .expect("failed to register context");

            println!("🔌 已注册 {} 个域插件", registry.plugin_count());
            for info in registry.list() {
                println!(
                    "   {} — {} ({} actions)",
                    info.name,
                    info.description,
                    info.actions.len()
                );
            }

            let domain_state: DomainState = Arc::new(RwLock::new(registry));

            // 创建 chat plugin 并注册到 domain_state
            let chat_plugin = ChatPlugin::new(domain_state.clone());
            {
                let mut registry = domain_state.blocking_write();
                registry
                    .register(Box::new(chat_plugin))
                    .expect("failed to register chat");
            }
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
                .setup(|app| {
                    // 设置 chat plugin 的 app handle 以支持事件发射
                    set_app_handle(app.handle().clone());
                    Ok(())
                })
                .plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(|app, shortcut, event| {
                            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed
                            {
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
                // 域插件状态
                .manage(domain_state)
                // 统一 API 状态
                .manage(unified_api)
                // PTY 状态
                .manage(pty_manager)
                .invoke_handler(tauri::generate_handler![
                    // ===== 域插件统一入口 (3 个命令覆盖 12 域 × ~8 actions) =====
                    domain_call,
                    domain_list,
                    domain_has,
                    domain_action_count,
                    // ===== Unified API (保留用于高级对话) =====
                    unified_init,
                    unified_chat,
                    unified_chat_stream,
                    unified_system_state,
                    unified_create_session,
                    unified_list_sessions,
                    unified_delete_session,
                    unified_exec_cli,
                    unified_cli_list,
                    // ===== PTY (硬件级接口) =====
                    crate::commands::pty::pty_spawn,
                    crate::commands::pty::pty_write,
                    crate::commands::pty::pty_resize,
                    crate::commands::pty::pty_close,
                    // ===== Model Pool (模型池) =====
                    crate::commands::model_pool::model_pool_status,
                    crate::commands::model_pool::model_pool_add,
                    crate::commands::model_pool::model_pool_remove,
                    crate::commands::model_pool::model_pool_update_key,
                    crate::commands::model_pool::model_pool_check,
                    // ===== Proxy Pool (代理池) =====
                    crate::commands::proxy_pool::proxy_pool_status,
                    crate::commands::proxy_pool::proxy_pool_snapshot,
                    crate::commands::proxy_pool::proxy_pool_add,
                    crate::commands::proxy_pool::proxy_pool_remove,
                    crate::commands::proxy_pool::proxy_pool_add_subscription,
                    crate::commands::proxy_pool::proxy_pool_remove_subscription,
                    crate::commands::proxy_pool::proxy_pool_set_strategy,
                    crate::commands::proxy_pool::proxy_pool_list_strategies,
                    // ===== IM Channel (即时通讯) =====
                    crate::commands::im::im_status,
                    crate::commands::im::im_list_channels,
                    crate::commands::im::im_get_channel,
                    crate::commands::im::im_toggle_channel,
                    crate::commands::im::im_add_bot,
                    crate::commands::im::im_remove_bot,
                    crate::commands::im::im_update_bot,
                    crate::commands::im::im_set_context_enhancement,
                    crate::commands::im::im_set_proactive_delivery,
                    // ===== DSH 市场模式 =====
                    crate::commands::im::im_dsh_market_status,
                    crate::commands::im::im_dsh_market_toggle,
                    crate::commands::im::im_dsh_market_config,
                    crate::commands::im::im_dsh_market_sync,
                    // ===== Market (市场发现引擎) =====
                    market_status,
                    market_search,
                    market_get_detail,
                    market_download,
                    market_install,
                    market_uninstall,
                    market_list_installed,
                    market_check_updates,
                    market_config,
                ])
                .setup(move |app| {
                    // PTY 事件转发
                    let pty_handle = app.handle().clone();
                    tauri::async_runtime::spawn(async move {
                        let mut rx = pty_rx;
                        while let Some(evt) = rx.recv().await {
                            match evt.event_type {
                                crate::commands::pty::PtyEventType::Output => {
                                    let _ = pty_handle
                                        .emit(&format!("pty-output-{}", evt.session_id), &evt.data);
                                }
                                crate::commands::pty::PtyEventType::Exit(code) => {
                                    let _ = pty_handle
                                        .emit(&format!("pty-exit-{}", evt.session_id), &code);
                                }
                            }
                        }
                    });

                    #[cfg(debug_assertions)]
                    {
                        if let Some(window) = app.get_webview_window("main") {
                            window.open_devtools();
                        }
                    }

                    println!("✅ NeoTrix V2 Desktop ready (domain plugin architecture)");
                    println!("   域调用: domain_call(domain, action, args)");
                    println!("   域列表: domain_list()");
                    println!("   PTY: 就绪");

                    Ok(())
                })
                .build(tauri::generate_context!())
                .expect("error while building tauri application")
                .run(
                    |_app, event| {
                        if let tauri::RunEvent::ExitRequested { .. } = event {}
                    },
                );
        }
        Some(Commands::Headless) => {
            println!("NeoTrix headless mode starting...");
            let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
            rt.block_on(async {
                let api = UnifiedApiImpl::new();
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    if let Ok(state) = api.get_system_state().await {
                        println!(
                            "  [tick] phi={:.3} coherence={:.3}",
                            state.metadata.consciousness_state.phi,
                            state.metadata.consciousness_state.coherence
                        );
                    }
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
