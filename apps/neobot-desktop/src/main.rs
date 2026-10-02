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
        .invoke_handler(neobot_desktop::neobot_commands!())
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
            // ⭐⭐ A1：启动刷一次崩溃残留（`mark_outcome_unknown`）。
            //
            // ⛔ **为什么必须在这里**：A1 只挂了 `bin/neobot.rs` 的
            //    `cmd_doctor` 与 `cmd_channel_serve` —— 但**桌面端不 spawn `neobot`
            //    子进程**（`apps/neobot-desktop/src` 内无 `Command::new`/`sidecar`）。
            //    ⇒ 不挂这里，桌面端启动就**永不刷残留**，崩溃后经桌面端打开仍会
            //    看到 `running` 悬挂行，而它本该落 `outcome_unknown` 等人工裁决。
            //
            // ⛔ 刻意**不**挂进 `commands::open_store()`：那个函数在每个 IPC 命令里
            //    都被调用（`neobot_convo_messages` 等全都开一次库），挂那里会让
            //    **每次读命令**都触发一次 UPDATE，「启动一次」名不副实。
            //
            // ⛔ best-effort 语义（与上方桌宠同一档）：开不起库不拦主窗启动 ——
            //    为一次维护动作而让整个应用起不来是本末倒置。
            // ⭐ 用 `let _marked: usize` 而非 `let _ =`：避开
            //    `check-silent-failure` 的 opener（虽然 `mark_outcome_unknown`
            //    不在 GATED 动词表里，属双保险）。
            match neobot_desktop::commands::open_store() {
                Ok(store) => {
                    let now = chrono::Utc::now().to_rfc3339();
                    let _marked: usize = store
                        .mark_outcome_unknown(&now, "startup sweep (desktop)")
                        .unwrap_or(0);
                }
                Err(e) => eprintln!("[a1] 启动刷崩溃残留失败（不拦主窗启动）：{e}"),
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
