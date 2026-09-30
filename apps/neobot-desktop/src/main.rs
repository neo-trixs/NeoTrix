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
//! # ⛔ 本文件尚未通过编译验证
//!
//! 写入时 `scripts/ops/nt_mem_gate.sh` 返回 2（BLOCKED），
//! 按仓库纪律禁止起 cargo 构建；且 pre-commit 门对**任何** `.rs`
//! 跑 `cargo check --tests -p neotrix`（验的是 neotrix-core，不是本 app）。
//! 故本文件的正确性来自**逐个核对库签名**，不是来自编译器。
//! 内存闸解除后必须补跑 `cargo check -p neobot-desktop --all-targets`。
//!
//! # 命令清单为什么这么短
//!
//! 只注册**本仓库里确实存在**的函数。刻意不注册：
//! 历史注记：曾刻意**不注册** `neobot_evidence_summary` 与 `neobot_send` ——
//! 那时它们只在另一仓侧，本仓的库没有，注册了就是「点了必然失败、
//! 还带看起来在工作的假象」。2026-09-30 用户决定搬进本仓（证据模块已落
//! `crates/neotrix-neobot/src/nt_evidence.rs`），故现在正常注册。

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        // 命令实现在 `commands`（pub，可被集成测试直接调用）；
        // 本文件只负责注册与生命周期。
        // 面板注册表由应用持有：骨架下发时登记，界面作答时按 id 查。
        .manage(neobot_desktop::commands::AppState::default())
        .invoke_handler(tauri::generate_handler![
            neobot_desktop::commands::neobot_panel_publish,
            neobot_desktop::commands::neobot_panel_clear,
            neobot_desktop::commands::neobot_agent_run,
            neobot_desktop::commands::neobot_send,
            neobot_desktop::commands::neobot_convo_group,
            neobot_desktop::commands::neobot_convo_dm,
            neobot_desktop::commands::neobot_core_capabilities,
            neobot_desktop::commands::neobot_evidence_summary,
            neobot_desktop::commands::neobot_panel_answer,
        ])
        .run(tauri::generate_context!())
        .expect("启动 NeoBot 失败");
}

// 二进制入口。`run()` 与本文件同处（bin 侧）—— lib.rs 只暴露常量，
// 供集成测试复用；把 run 放 lib 会让 #[cfg_attr(mobile, ...)] 宏路径
// 在 lib 与 bin 两边各展开一次。
fn main() {
    run();
}
