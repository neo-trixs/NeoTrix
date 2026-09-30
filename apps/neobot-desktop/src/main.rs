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

use std::path::PathBuf;

use neotrix_neobot::nt_core::{agent_run, capabilities_or_default, CapabilitiesInfo, AgentRunResult};
use neotrix_neobot::nt_evidence::{audit, EvidenceReport};
use neotrix_neobot::nt_store::NeobotStore;
use serde::Serialize;
use tauri::Manager;

/// 数据目录。**待与 CLI 对齐**（见下方 TODO）。
///
/// CLI 侧的实际解析逻辑我没在本次核对中确认，所以这里只写约定
/// `~/.neobot` 并显式标注，而不是假装已经一致 —— 两处不一致会导致
/// 「桌面建了会话、CLI 看不见」这类极难查的问题。
fn data_dir() -> Result<PathBuf, String> {
    // TODO(2026-09-30): 与 crates/neotrix-neobot/src/bin/neobot.rs 的
    // 数据目录解析对齐（含 `NEOTRIX_HOME` / `NEOBOT_HOME` 之类覆盖变量）。
    // 未对齐前不要当成已验证行为。
    let home = std::env::var_os("HOME").ok_or("HOME 未设置")?;
    Ok(PathBuf::from(home).join(".neobot"))
}

fn open_store() -> Result<NeobotStore, String> {
    let dir = data_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建数据目录失败：{e}"))?;
    let db = dir.join("neobot.db");
    NeobotStore::open(db.to_str().ok_or("数据目录不是合法 UTF-8")?).map_err(|e| e.to_string())
}

/// `neobot_agent_run(goal, context?) -> AgentRunResult`
#[tauri::command]
async fn neobot_agent_run(goal: String, context: Option<String>) -> Result<AgentRunResult, String> {
    let store = open_store()?;
    // 同步跑轮放在 blocking 里：agent_run 是阻塞 IO，挂在 async 命令上
    // 会占住 Tauri 的异步线程池。
    //
    // ⚠️ context 必须**整个 move 进闭包**：agent_run 收 Option<&str>，
    //    而借用跨不过 spawn_blocking 的 'static 边界（首版就是漏了 move，
    //    编译报 E0597 —— 这次能发现全靠真的编了一遍）。
    tauri::async_runtime::spawn_blocking(move || {
        agent_run(&store, &goal, context.as_deref()).map_err(|e| e.to_string())
    })
        .await
        .map_err(|e| format!("跑轮任务异常：{e}"))?
}

/// `neobot_convo_group(title, members) -> convo_id`
#[tauri::command]
fn neobot_convo_group(title: String, members: Vec<String>) -> Result<String, String> {
    let store = open_store()?;
    store
        .create_conversation("group", &title, &members)
        .map_err(|e| e.to_string())
}

/// `neobot_convo_dm(me, peer) -> convo_id`
///
/// 与 group 同一套：前端不该为「私聊/群」各写一条调用路径。
#[tauri::command]
fn neobot_convo_dm(me: String, peer: String) -> Result<String, String> {
    let store = open_store()?;
    store.get_or_create_dm(&me, &peer).map_err(|e| e.to_string())
}

/// `neobot_evidence_summary(text) -> EvidenceReport`
///
/// 用户已决定把该能力搬进本仓（原先只在另一仓侧，本仓无 → 界面点了必然失败）。
/// 返回**完整报告**而非只有一句话：前端的证据块要能列出每一条问题，
/// 只回一句「证据不足」等于把「哪一句有问题」藏起来。
#[tauri::command]
fn neobot_evidence_summary(text: String) -> EvidenceReport {
    audit(&text)
}

/// `neobot_send(text) -> AgentRunResult`
///
/// 前端按 Enter 发送时走这条。注意它与 `neobot_agent_run` 的区别：
/// 前者是**会话内**的一轮（带 convo_id 上下文），后者是脱离会话手动跑。
/// 两者曾在前端被混用（都落到同一个「跑一轮」），这里保持分离。
#[tauri::command]
async fn neobot_send(text: String) -> Result<AgentRunResult, String> {
    if text.trim().is_empty() {
        // 空白消息不是「跑了但没输出」，是**请求本身不成立**。给它一个
        // 明确错误，比跑一轮再回一句空强。
        return Err("消息为空".to_owned());
    }
    let store = open_store()?;
    tauri::async_runtime::spawn_blocking(move || {
        agent_run(&store, &text, None).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("发送任务异常：{e}"))?
}

/// 能力快照 —— 前端能力矩阵的**真源**。
///
/// 前端 `CapabilityRegistry` 目前先跑一份默认矩阵（`defaultCapabilities`）。
/// 这条命令存在的意义是：等它接上后，前端应改为**以此为准**，
/// 于是「界面为什么没有某个功能」有了单一答案，而不是两处默认值在猜。
#[derive(Serialize)]
struct CapabilitySnapshot {
    crystal_version: String,
    tool_count: usize,
    model: String,
    /// 前端要展示的完整路径，便于界面上「查看详情」。
    model_source: String,
}

#[tauri::command]
fn neobot_core_capabilities() -> Result<CapabilitySnapshot, String> {
    // base_url / token_env 走与 CLI 相同的解析；此处先走默认（空），
    // 等把 nt_config 的解析函数接进来再改。**已标注，不假装已对齐。**
    let info: CapabilitiesInfo = capabilities_or_default("", "");
    Ok(CapabilitySnapshot {
        crystal_version: info.crystal_version,
        tool_count: info.tool_count,
        model: info.model.clone(),
        model_source: info.model,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 把数据目录暴露给前端：前端要显示「本地存储在哪」，
            // 不该自己猜 HOME 拼路径（那是宿主职责，不是界面职责）。
            if let Ok(dir) = data_dir() {
                let _ = app.handle().path();
                let _ = dir;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            neobot_agent_run,
            neobot_send,
            neobot_convo_group,
            neobot_convo_dm,
            neobot_core_capabilities,
            neobot_evidence_summary,
        ])
        .run(tauri::generate_context!())
        .expect("启动 NeoBot 失败");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_dir_is_absolute() {
        // 只断言形状，不碰真实 HOME：CI 里 HOME 可能是别的值。
        let dir = data_dir().expect("data_dir");
        assert!(dir.is_absolute(), "数据目录必须是绝对路径");
        assert!(dir.ends_with(".neobot"), "数据目录名不对：{dir:?}");
    }

    #[test]
    fn group_creation_rejects_empty_title() {
        // 库的 create_conversation 自带这条校验；这里确认错误能变成
        // **可展示的中文**而不是裸的 Err Debug 输出。
        let store = NeobotStore::open(":memory:").expect("内存库");
        let err = store
            .create_conversation("group", "  ", &[])
            .expect_err("空标题必须被拒");
        let msg = err.to_string();
        assert!(!msg.is_empty(), "错误不能是空串");
    }

    #[test]
    fn group_creation_rejects_bad_kind() {
        let store = NeobotStore::open(":memory:").expect("内存库");
        assert!(
            store.create_conversation("party", "x", &[]).is_err(),
            "未知 kind 必须被拒"
        );
    }
}

// 二进制入口。`run()` 与本文件同处（bin 侧）—— lib.rs 只暴露常量，
// 供集成测试复用；把 run 放 lib 会让 #[cfg_attr(mobile, ...)] 宏路径
// 在 lib 与 bin 两边各展开一次。
fn main() {
    run();
}
