//! NeoBot 桌面端 —— **薄壳**。
//!
//! 这里的哲学只有一句：**逻辑不搬进 app**。
//! 能力实现全在同仓 `crates/neotrix-neobot`，本 crate 只做
//! 「Tauri 命令注册」+「前端资源装配」两件接线活。
//!
//! 依据：d5413335 删除内嵌 app 的理由之一是「桌面端曾是两份同源物」。
//! 这次重建若把库的代码复制进 app，那个问题就原样回来了。

/// 前端资源目录（由 vite build 产出）。
pub const FRONTEND_DIST: &str = "../frontend/dist";

/// Tauri 命令层。`pub` 是为了**让集成测试够得着** ——
/// 命令函数留在 `main.rs` 里的话，`tests/` 访问不到，
/// 「命令名 → 函数 → 库」这段接线就只能在发布后才第一次被执行。
pub mod api;
pub mod commands;
pub mod core;
pub mod desktop;
pub mod menu;
pub mod pet;
pub mod platform;

/// **命令注册表（唯一真源）** —— `main.rs` 与集成测试**共用**。
///
/// ## 为什么必须共用
///
/// ⛔ 2026-10-02 的 P0 复盘教了一件事：**「命令名 → 函数 → 库」这段接线，
///    在此之前从未被任何自动化测试执行过** —— 74 个命令的 IPC 往返覆盖率 = 0%，
///    已有 74 个测试全都落在私有 helper / store / state 层。
///    注册表若只留在 `main.rs`（bin 侧），`tests/` 根本够不着它，
///    于是「测试能跑」与「命令真被注册」之间永远隔着一条没人走过的路
///    —— 漏注册不会有任何信号。⇒ 注册表进 lib、两侧展开同一个宏
///    ⇒ **漏注册，测试立刻红**。
///
/// ⛔ 刻意**不**把 `build_app` 整个泛型化搬进 lib：那会把 4 个真实插件
///    （autostart / clipboard / opener / notification）带进测试，而它们会
///    **写真实 LaunchAgent plist、改用户真实剪贴板、真开浏览器、真弹通知**。
///    这里只共享**命令清单**；插件装配仍归 `main.rs`。
#[macro_export]
macro_rules! neobot_commands {
    () => {
        tauri::generate_handler![

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
            // 2026-10-04：**已删除** `neobot_convo_messages`（全量）注册。
            //
            // ⛔ 删前这里写着「⛔ 不改 neobot_convo_messages 的签名 ⇒ 旧路径仍可用」
            // —— **那正是分叉的根源**：两条读历史的路径并存
            // （「同一语义两个真源」），而本仓已**零调用方**用它
            // （自持树只在注释里提到）。⇒ **唯一真源 = `_page`**。
            // 详见 `commands.rs` 里同名的删除说明 + `api.rs` 的契约条目。
            neobot_desktop::commands::neobot_convo_messages_page,
            // 轨迹读口（2026-10-07）：跑过什么此前在库里、界面上看不见。
            // ⛔ 与 `neobot_convo_messages_page` 的差别**故意**：那条必须给会话，
            //    这条允许缺席（= 全部会话）—— 理由见 `api.rs` 同名条目。
            neobot_desktop::commands::neobot_run_list,
            neobot_desktop::commands::neobot_run_trace,
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
        ]
    };
}
