//! NeoBot 桌面端 —— Tauri 薄壳。
//!
//! # 职责边界（唯一一条，别越）
//!
//! 本文件只做**接线**：
//!   ① 把前端的 `invoke(cmd, args)` 映射到 `crates/neotrix-neobot` 的库函数
//!   ② 把库返回的领域类型转成前端要的形状
//!
//! **不含任何业务逻辑**。依据 d5413335：桌面端曾是两份同源物，
//! 删除内嵌 app 就是为了消除那个问题。这次重建若在这里重写一遍
//! 逻辑，等于把它原样造回来。
//!
//! # 编译状态
//!
//! `cargo check -p neobot-desktop --all-targets` 0 error，
//! `cargo test -p neobot-desktop` 52+3 全绿（2026-09-30）。
//! pre-commit 门对**任何** `.rs` 跑的是 `cargo check --tests -p neotrix`
//!（验的是 neotrix-core，不是本 app）—— 故本 app 的编译正确性
//! 由上两条命令保证，改完 `.rs` 必跑。
//!
//! # 命令清单为什么这么短
//!
//! 只注册**本仓库里确实存在**的函数。刻意不注册：
//! 历史注记：曾刻意**不注册** `neobot_evidence_summary` 与 `neobot_send` ——
//! 那时它们只在另一仓侧，本仓的库没有，注册了就是「点了必然失败、
//! 还带看起来在工作的假象」。2026-09-30 用户决定搬进本仓（证据模块已落
//! `crates/neotrix-neobot/src/nt_evidence.rs`），故现在正常注册。

/// 组装 Tauri 应用（与 [`run`] 分离，好让 `run` 能用 `if let` 而不必 panic）。
fn build_app() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        // 壳启动必用。⛔ 少注册哪个都会报「not allowed by ACL」——
        //    而「ACL 不允许」与「命令不存在」是两个完全不同的病因，
        //    前者去查 capabilities，后者去查命令注册表。
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        // 命令实现在 `commands`（pub，可被集成测试直接调用）；
        // 本文件只负责注册与生命周期。
        // 面板注册表由应用持有：骨架下发时登记，界面作答时按 id 查。
        .manage(neobot_desktop::commands::AppState::default())
        .invoke_handler(tauri::generate_handler![
            neobot_desktop::core::get_cores,
            neobot_desktop::core::set_active_core,
            neobot_desktop::core::remove_core,
            neobot_desktop::core::list_backups,
            neobot_desktop::core::delete_backup,
            neobot_desktop::core::update_app_config,
            neobot_desktop::core::get_app_config,
            neobot_desktop::platform::open_external_url,
            neobot_desktop::platform::write_clipboard_text,
            neobot_desktop::platform::read_clipboard_image,
            neobot_desktop::platform::show_native_notification,
            neobot_desktop::platform::get_launch_on_login,
            neobot_desktop::platform::set_launch_on_login,
            neobot_desktop::platform::create_app_window,
            neobot_desktop::platform::remote_open_window,
            neobot_desktop::pet::move_pet_window,
            neobot_desktop::pet::list_pets,
            neobot_desktop::pet::import_pet,
            neobot_desktop::pet::get_pet_asset,
            neobot_desktop::pet::list_preset_pets,
            neobot_desktop::platform::quit_app,
            neobot_desktop::platform::reveal_data_dir,
            neobot_desktop::platform::reveal_in_folder,
            neobot_desktop::platform::open_dir,
            neobot_desktop::pet::get_pet_status,
            neobot_desktop::pet::set_pet_enabled,
            neobot_desktop::pet::set_active_pet,
            neobot_desktop::pet::set_pet_size,
            neobot_desktop::pet::get_pet_overlay_supported,
            neobot_desktop::pet::get_force_xwayland,
            neobot_desktop::pet::set_force_xwayland,
            neobot_desktop::pet::set_pet_ignore_cursor_events,
            neobot_desktop::desktop::get_runtime_info,
            neobot_desktop::desktop::runtime_ready,
            neobot_desktop::desktop::install_dependencies,
            neobot_desktop::desktop::set_language,
            neobot_desktop::desktop::get_dsh_theme,
            neobot_desktop::desktop::is_dev_build,
            neobot_desktop::desktop::log_frontend,
            neobot_desktop::desktop::read_run_logs,
            neobot_desktop::desktop::get_desktop_about,
            neobot_desktop::commands::neobot_api_specs,
            neobot_desktop::commands::neobot_api_call,
            neobot_desktop::commands::neobot_panel_publish,
            neobot_desktop::commands::neobot_panel_clear,
            neobot_desktop::commands::neobot_panel_demo_publish,
            neobot_desktop::commands::neobot_agent_run,
            neobot_desktop::commands::neobot_send,
            neobot_desktop::commands::neobot_usage_summary,
            neobot_desktop::commands::neobot_memory_list,
            neobot_desktop::commands::neobot_memory_add,
            neobot_desktop::commands::neobot_memory_undo,
            neobot_desktop::commands::neobot_convo_messages,
            neobot_desktop::commands::neobot_convo_list,
            neobot_desktop::commands::neobot_member_list,
            neobot_desktop::commands::neobot_member_add,
            neobot_desktop::commands::neobot_convo_group,
            neobot_desktop::commands::neobot_convo_dm,
            neobot_desktop::commands::neobot_core_capabilities,
            neobot_desktop::commands::neobot_evidence_summary,
            neobot_desktop::commands::neobot_panel_answer,
            neobot_desktop::core::neobot_skill_list,
            neobot_desktop::core::neobot_skill_install,
            // 「本仓不提供」的显式声明（理由见 desktop 模块段头注释）。
            neobot_desktop::desktop::get_dsh_plugins,
            neobot_desktop::desktop::get_cli_link_status,
            neobot_desktop::desktop::copy_service_url,
            neobot_desktop::desktop::get_plugin_backup,
            neobot_desktop::desktop::download_core,
            neobot_desktop::desktop::update_local_core,
            neobot_desktop::desktop::report_plugin_error,
            neobot_desktop::desktop::open_preinstall_repo,
            neobot_desktop::desktop::get_profiles,
            neobot_desktop::desktop::create_profile,
            neobot_desktop::desktop::set_active_profile,
        ])
        .setup(|app| {
            // macOS 原生菜单栏。⛔ 不是可选项：
            //    `navbar.tsx`（逐字未改）在 macOS 上把「文件/运行/帮助」整组隐藏，
            //    假定它们由原生菜单承载。不装这一段，配置/关于/日志/更新/文档/
            //    新建窗口就全都没有入口，而前端那段 `macos-menu-action` 分发是死代码。
            if let Err(e) = neobot_desktop::menu::install(app.handle()) {
                eprintln!("[menu] 安装 macOS 原生菜单失败：{e}");
            }
            // 桌宠窗口恢复：上次开着，这次接着开。
            // ⛔ 建不起不拦主窗启动（`let _`）：桌宠是点缀，主窗是正餐；
            // 为点缀失败而让整个应用起不来是本末倒置。失败时状态保持 enabled，
            // 用户在设置页开关一次即重建。
            if neobot_desktop::pet::get_pet_status().enabled {
                if let Ok(w) = neobot_desktop::pet::ensure_pet_window(app.handle()) {
                    let _ = w.show();
                }
            }
            Ok(())
        })
}

/// 启动应用。
///
/// ⛔⛔ **签名必须是 `fn run() -> ()`**：上面的
/// `#[cfg_attr(mobile, tauri::mobile_entry_point)]` 要求这一点，
/// 改成 `-> Result` 在 mobile 下直接破。
/// ⛔ 原先这里是 `.expect("启动 NeoBot 失败")`。现在改成
/// **报错 + 退出**：保留「启动失败必须致命」的语义，去掉 unwinding
/// （panic 会往 stdout/stderr 打一条 Rust backtrace，对用户是噪声）。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = build_app().run(tauri::generate_context!()) {
        eprintln!("启动 NeoBot 失败：{e}");
        std::process::exit(1);
    }
}

// 二进制入口。`run()` 与本文件同处（bin 侧）—— lib.rs 只暴露常量，
// 供集成测试复用；把 run 放 lib 会让 #[cfg_attr(mobile, ...)] 宏路径
// 在 lib 与 bin 两边各展开一次。
fn main() {
    run();
}
