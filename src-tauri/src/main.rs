//! NeoTrix Tauri Desktop - Domain Plugin Architecture
//!
//! 基于 DeepSeek Harness 架构理念：一切皆插件，能力缝可替换。
//! 通过 DomainRegistry 统一管理 12 个功能域插件。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(clippy::all, clippy::pedantic, clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    reason = "pedantic false-positives in Tauri plugin boilerplate"
)]

use clap::Parser;
use std::process;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

use neotrix_tauri::commands::chat::{chat_send, chat_help};
use neotrix_tauri::commands::domain_cmd::{
    domain_action_count, domain_call, domain_has, domain_list, DomainState,
};
use neotrix_tauri::domain::{plugins::*, DomainRegistry};
use neotrix_tauri::stub::{UnifiedApi as _, UnifiedApiImpl};

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

fn register_plugins(
    registry: &mut DomainRegistry,
    db_pool: &Arc<neotrix_tauri::db_pool::DbPool>,
) -> Result<(), String> {
    let registrations: Vec<(
        &str,
        Box<dyn neotrix_tauri::domain::DomainPlugin + Send + Sync>,
    )> = vec![
        ("session", Box::new(SessionPlugin::new(db_pool.clone()))),
        ("agent", Box::new(AgentPlugin)),
        ("kb", Box::new(KbPlugin::new(db_pool.clone()))),
        ("file", Box::new(FilePlugin)),
        ("plugin", Box::new(PluginPlugin)),
        ("workflow", Box::new(WorkflowPluginImpl::new())),
        ("tool", Box::new(ToolPlugin)),
        ("system", Box::new(SystemPlugin)),
        ("security", Box::new(SecurityPlugin)),
        ("memory", Box::new(MemoryPlugin::new(db_pool.clone()))),
        ("proxy_pool", Box::new(ProxyPoolPlugin::new())),
        ("model_pool", Box::new(ModelPoolPlugin::new())),
        ("autostart", Box::new(AutostartPlugin::new())),
        ("ext", Box::new(ExtPlugin)),
        ("llamacpp", Box::new(LlamacppPlugin::new())),
        ("git", Box::new(GitPlugin)),
        ("cli", Box::new(CliPlugin)),
        ("world", Box::new(WorldPlugin)),
        ("context", Box::new(ContextPlugin)),
        ("ai_orchestration", Box::new(AiOrchestrationPlugin::new())),
        (
            "folder_instructions",
            Box::new(FolderInstructionsPlugin::new()),
        ),
        ("im", Box::new(ImPlugin::new())),
        ("mcp_extension", Box::new(McpExtensionPlugin::new())),
        ("session_sync", Box::new(SessionSyncPlugin::new())),
        ("unified_surface", Box::new(UnifiedSurfacePlugin::new())),
        ("voice_agent", Box::new(VoiceAgentPlugin::new())),
    ];
    for (name, plugin) in registrations {
        registry
            .register(plugin)
            .map_err(|e| format!("failed to register {name} plugin: {e}"))?;
    }
    Ok(())
}

fn main() {
    if std::env::var("NEOTRIX_MCP_STDIO").as_deref() == Ok("1") {
        return;
    }

    if let Err(e) = color_eyre::install() {
        tracing::warn!("Failed to install color-eyre: {e}");
    }

    // Fail-fast config validation — exits before anything else starts
    let config = neotrix_tauri::config::AppConfig::load().unwrap_or_else(|e| {
        tracing::error!("FATAL: Configuration error: {e}");
        tracing::error!("Fix your config or set environment variables (NEOTRIX_*)");
        std::process::exit(1);
    });

    let _sentry_guard = neotrix_tauri::stub::init_sentry();
    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Desktop) => {
            // 创建域注册表并注册 12 个插件
            let mut registry = DomainRegistry::new();
            let db_pool = Arc::new(
                neotrix_tauri::db_pool::DbPool::new(&config.data_dir.join("neotrix.db"))
                    .unwrap_or_else(|e| {
                        tracing::error!("FATAL: Failed to create DB pool: {e}");
                        process::exit(1);
                    }),
            );
            register_plugins(&mut registry, &db_pool).unwrap_or_else(|e| {
                tracing::error!("FATAL: {e}");
                process::exit(1);
            });

            tracing::info!("已注册 {} 个域插件", registry.plugin_count());
            for info in registry.list() {
                tracing::info!(
                    "   {} — {} ({} actions)",
                    info.name,
                    info.description,
                    info.actions.len()
                );
            }

            let domain_state: DomainState = Arc::new(RwLock::new(registry));

            // 创建 chat plugin 并注册到 domain_state
            let chat_plugin = ChatPlugin::new(db_pool.clone(), domain_state.clone());
            {
                let mut registry = domain_state.blocking_write();
                registry
                    .register(Box::new(chat_plugin))
                    .unwrap_or_else(|e| {
                        tracing::error!("FATAL: Failed to register chat plugin: {e}");
                        process::exit(1);
                    });
            }

            let (pty_manager, pty_rx) = neotrix_tauri::commands::pty::PtyManager::new();
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
                tracing::info!("[boundary] updater disabled (NEOTRIX_UPDATER unset + debug build)");
                builder
            };

            builder
                .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.set_focus();
                    }
                }))
                .plugin(tauri_plugin_autostart::init(
                    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                    None,
                ))

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
                                let _ = app.emit("neotrix_global_shortcut", shortcut.to_string());
                            }
                        })
                        .build(),
                )
                // 域插件状态
                .manage(domain_state)
                // 分层配置
                .manage(config)
                // PTY 状态
                .manage(pty_manager)
                .invoke_handler(tauri::generate_handler![
                    // ===== Natural Language Chat (single entry point) =====
                    chat_send,
                    chat_help,
                    // ===== Domain Plugin API (程序化入口) =====
                    domain_call,
                    domain_list,
                    domain_has,
                    domain_action_count,
                    // ===== PTY (硬件级接口) =====
                    neotrix_tauri::commands::pty::pty_spawn,
                    neotrix_tauri::commands::pty::pty_write,
                    neotrix_tauri::commands::pty::pty_resize,
                    neotrix_tauri::commands::pty::pty_close,
                ])
                .setup(move |app| {
                    neotrix_tauri::logger::init_logging(app.handle());
                    neotrix_tauri::domain::app_handle::set_app_handle(app.handle().clone());
                    neotrix_tauri::service::start(app.handle());

                    let notification_manager =
                        neotrix_tauri::notifications::NotificationManager::new(
                            app.handle().clone(),
                        );
                    app.manage(notification_manager);

                    let autostart_manager =
                        neotrix_tauri::autostart::AutoStartManager::new(app.handle().clone());
                    app.manage(autostart_manager);

                    neotrix_tauri::setup_menu(app)?;
                    neotrix_tauri::setup_tray(app)?;

                    let pty_handle = app.handle().clone();
                    tauri::async_runtime::spawn(async move {
                        let mut rx = pty_rx;
                        while let Some(evt) = rx.recv().await {
                            match evt.event_type {
                                neotrix_tauri::commands::pty::PtyEventType::Output => {
                                    let _ = pty_handle
                                        .emit(&format!("pty_output_{}", evt.session_id), &evt.data);
                                }
                                neotrix_tauri::commands::pty::PtyEventType::Exit(code) => {
                                    let _ = pty_handle
                                        .emit(&format!("pty_exit_{}", evt.session_id), &code);
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

                    tracing::info!("NeoTrix V2 Desktop ready (chat-first architecture)");
                    tracing::info!("   NL Chat: chat_send(message)");
                    tracing::info!("   Domain API: domain_call(domain, action, args)");
                    tracing::info!("   PTY: 就绪");

                    Ok(())
                })
                .build(tauri::generate_context!())
                .unwrap_or_else(|e| {
                    tracing::error!("FATAL: Failed to build Tauri application: {e}");
                    process::exit(1);
                })
                .run(
                    |_app, event| {
                        if let tauri::RunEvent::ExitRequested { .. } = event {}
                    },
                );
        }
        Some(Commands::Headless) => {
            tracing::info!("NeoTrix headless mode starting...");
            let rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
                tracing::error!("FATAL: Failed to create tokio runtime: {e}");
                process::exit(1);
            });
            rt.block_on(async {
                let api = UnifiedApiImpl::new();
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    if let Ok(state) = api.get_system_state().await {
                        tracing::info!(
                            "  [tick] phi={:.3} coherence={:.3}",
                            state.metadata.consciousness_state.phi,
                            state.metadata.consciousness_state.coherence
                        );
                    }
                }
            });
        }
        Some(Commands::Reason { prompt }) => {
            tracing::info!("NeoTrix reasoning: {}", prompt);
            let rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
                tracing::error!("FATAL: Failed to create tokio runtime: {e}");
                process::exit(1);
            });
            rt.block_on(async {
                let api = UnifiedApiImpl::new();
                let request = neotrix_tauri::stub::UnifiedRequest::chat(prompt);
                let response = api.handle(request).await;
                match response {
                    Ok(r) => tracing::info!("Response: {}", r.content),
                    Err(e) => tracing::error!("Error: {} - {}", e.code, e.message),
                }
            });
        }
    }
}
