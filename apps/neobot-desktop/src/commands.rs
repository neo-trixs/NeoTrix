//! Tauri 命令层 —— **只做接线**。
//!
//! ⛔ 刻意从 `main.rs` 抽出来成独立模块并标 `pub`，原因很实际：
//!    命令函数写在 `main.rs` 里是私有的，**集成测试够不着**
//!    （`tests/` 只能访问 lib 的 pub API）。
//!    于是「从命令名到函数到库」这段接线在发布前**无人能测** ——
//!    而这段接线正是最容易错的地方（参数名、返回形状、库函数选错）。
//!    抽出来之后 `tests/commands.rs` 可以直接逐个调用。

use std::path::PathBuf;

use neotrix_neobot::nt_core::{agent_run, capabilities_or_default, CapabilitiesInfo, AgentRunResult};
use neotrix_neobot::nt_evidence::{audit, EvidenceReport};
use neotrix_neobot::nt_panel::{Answer, AnswerOutcome, Panel, PublishError, Registry};
use neotrix_neobot::nt_store::NeobotStore;
use serde::Serialize;

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
pub async fn neobot_agent_run(goal: String, context: Option<String>) -> Result<AgentRunResult, String> {
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
pub fn neobot_convo_group(title: String, members: Vec<String>) -> Result<String, String> {
    let store = open_store()?;
    store
        .create_conversation("group", &title, &members)
        .map_err(|e| e.to_string())
}

/// `neobot_convo_dm(me, peer) -> convo_id`
///
/// 与 group 同一套：前端不该为「私聊/群」各写一条调用路径。
#[tauri::command]
pub fn neobot_convo_dm(me: String, peer: String) -> Result<String, String> {
    let store = open_store()?;
    store.get_or_create_dm(&me, &peer).map_err(|e| e.to_string())
}

/// `neobot_evidence_summary(text) -> EvidenceReport`
///
/// 用户已决定把该能力搬进本仓（原先只在另一仓侧，本仓无 → 界面点了必然失败）。
/// 返回**完整报告**而非只有一句话：前端的证据块要能列出每一条问题，
/// 只回一句「证据不足」等于把「哪一句有问题」藏起来。
#[tauri::command]
pub fn neobot_evidence_summary(text: String) -> EvidenceReport {
    audit(&text)
}

/// `neobot_send(text) -> AgentRunResult`
///
/// 前端按 Enter 发送时走这条。注意它与 `neobot_agent_run` 的区别：
/// 前者是**会话内**的一轮（带 convo_id 上下文），后者是脱离会话手动跑。
/// 两者曾在前端被混用（都落到同一个「跑一轮」），这里保持分离。
#[tauri::command]
pub async fn neobot_send(text: String) -> Result<AgentRunResult, String> {
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

/// 骨架下发给界面的面板事件名。前端 `listen` 同一个名字。
pub const PANEL_EVENT: &str = "neobot:panel";

/// 应用状态：面板注册表。**由骨架持有，界面只能报 id。**
#[derive(Default)]
pub struct AppState {
    pub panels: std::sync::Mutex<Registry>,
}

/// `neobot_panel_publish(panel) -> ()`
///
/// 骨架下发一个决策面板：校验 → 登记 → 推给界面。
///
/// ⛔ 登记失败（面板非法/版本倒退）时**返回错误而不静默丢弃** ——
///    静默丢弃等于骨架以为自己发出了、界面永远等不到，表现为「卡住不动」，
///    而两端都没有报错。
#[tauri::command]
pub async fn neobot_panel_publish(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    panel: Panel,
) -> Result<u64, String> {
    let version = panel.candidate_set_version;
    {
        let mut reg = state
            .panels
            .lock()
            .map_err(|_| "面板注册表锁中毒（此前有命令 panic）".to_owned())?;
        reg.publish(panel.clone()).map_err(|e| match e {
            PublishError::Invalid(why) => format!("面板非法：{why}"),
            PublishError::VersionWentBack { current, incoming } => {
                format!("候选集版本倒退（{incoming} < {current}）")
            }
        })?;
    }
    // 推给界面。发不出去就报 —— 界面收不到与「面板没发」同样表现为无响应。
    tauri::Emitter::emit(&app, PANEL_EVENT, &panel)
        .map_err(|e| format!("推送面板到界面失败：{e}"))?;
    Ok(version)
}

/// `neobot_panel_answer(answer) -> AnswerOutcome`
///
/// ⚠️ **只收 answer，不收 panel。** 基准由 `AppState.panels` 持有。
///    旧签名让调用方一并传 panel，等于「被判定的一方自带基准」——
///    界面伪造一个同 id 的面板就能通过校验。这与 `ActorContext.resolvedBy`
///    是同一条纪律：**基准不能由被判定方提供。**
#[tauri::command]
pub fn neobot_panel_answer(
    state: tauri::State<'_, AppState>,
    answer: Answer,
) -> Result<AnswerOutcome, String> {
    let reg = state
        .panels
        .lock()
        .map_err(|_| "面板注册表锁中毒（此前有命令 panic）".to_owned())?;
    Ok(reg.answer(&answer))
}

/// `neobot_panel_clear()` —— 清空注册表。
///
/// 用途：切换会话。换会话不清的话，旧会话的面板仍可被作答 ——
/// 而作答会被记到新会话的流里，属于串台。
#[tauri::command]
pub fn neobot_panel_clear(state: tauri::State<'_, AppState>) -> Result<usize, String> {
    let mut reg = state
        .panels
        .lock()
        .map_err(|_| "面板注册表锁中毒（此前有命令 panic）".to_owned())?;
    let n = reg.len();
    *reg = Registry::new();
    Ok(n)
}

/// 能力快照 —— 前端能力矩阵的**真源**。
///
/// 前端 `CapabilityRegistry` 目前先跑一份默认矩阵（`defaultCapabilities`）。
/// 这条命令存在的意义是：等它接上后，前端应改为**以此为准**，
/// 于是「界面为什么没有某个功能」有了单一答案，而不是两处默认值在猜。
#[derive(Serialize)]
pub struct CapabilitySnapshot {
    crystal_version: String,
    tool_count: usize,
    model: String,
    /// 前端要展示的完整路径，便于界面上「查看详情」。
    model_source: String,
}

#[tauri::command]
pub fn neobot_core_capabilities() -> Result<CapabilitySnapshot, String> {
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
