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

/// 数据目录。**已与 CLI 对齐**（原 TODO 已闭合）。
///
/// 之前这里硬编码 `~/.neobot`，而 CLI 走 `NeobotConfig::from_env()`，
/// 后者认 `NEOBOT_DATA_DIR` / `NEOBOT_POLICY` / `NEOBOT_ENGINE`。
/// ⇒ 设了 `NEOBOT_DATA_DIR` 时，**CLI 和桌面端会打开两个不同的库**：
///    桌面建了会话、CLI 看不见。这类问题极难查，所以口径必须**单一**。
///
/// 现在两处都问库要，桌面端不再自己拼路径。
pub fn data_dir() -> Result<PathBuf, String> {
    neotrix_neobot::NeobotConfig::from_env()
        .map(|c| c.data_dir)
        .map_err(|e| format!("解析数据目录失败（检查 NEOBOT_DATA_DIR 等环境变量）：{e}"))
}

/// ⭐ 开库（**桌面端唯一的开库口径**）。
///
/// ⛔ 2026-10-02 提为 `pub`：A1 要在 `main.rs` 的 `.setup` 里启动刷崩溃残留，
///    而**复制**第二份开库代码等于制造第二条 `data_dir` → `neobot.db` 路径 ——
///    两者一旦漂移，桌面端就会与 CLI 打开两个不同的库
///    （`tests/data_dir_env.rs` 记的正是那个事故：「CLI 与桌面端打开两个不同的库」）。
///    ⇒ 复用本函数，而不是新写一份。
pub fn open_store() -> Result<NeobotStore, String> {
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

/// `neobot_panel_demo_publish()` —— 演示骨架：登记一块面板并推给界面。
///
/// # 定位要说清楚，否则会被当成「已经有骨架了」
///
/// 这是**接口连通性证明**，不是产品功能。真实下发器在骨架侧
/// （`neotrix-neobot` 的运行进程），本仓 app 只是终端。
/// 它存在的唯一价值：让「publish → 事件 → 界面 → answer → 注册表校验」
/// 这条链在**没有骨架进程**时也能被人点一次。
///
/// ⛔ 它**不**绕过校验：面板照样走 `Registry::publish`，作答照样按 id 查。
///    若有人把它当成「界面可以自己造面板」的后门，那就白做了 ——
///    它在 Rust 侧，走的是和真实骨架完全相同的那条路。
#[tauri::command]
pub async fn neobot_panel_demo_publish(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<u64, String> {
    neobot_panel_publish(app, state, demo_panel()).await
}

/// 演示面板的内容。
///
/// `Mode::Sample` 而非 `Live` —— 它是合成的，标 Live 就是谎报（§2 决定 2）。
fn demo_panel() -> Panel {
    let src = |t: &str| neotrix_neobot::nt_panel::Source {
        title: t.to_owned(),
        url: format!("https://example.com/{t}"),
    };
    Panel {
        id: "p-demo".into(),
        thread_id: "demo".into(),
        turn_id: "demo-1".into(),
        candidate_set_version: 1,
        kind: neotrix_neobot::nt_panel::PanelKind::Comparison,
        title: "接口迁移方案定哪个".into(),
        mode: neotrix_neobot::nt_panel::Mode::Sample,
        options: vec![
            neotrix_neobot::nt_panel::PanelOption {
                id: "inplace".into(),
                label: "原地迁移".into(),
                details: vec!["改动集中，但回滚困难".into()],
                sources: vec![src("inplace")],
            },
            neotrix_neobot::nt_panel::PanelOption {
                id: "strangle".into(),
                label: "绞杀者模式".into(),
                details: vec!["可灰度，回滚容易".into()],
                sources: vec![src("strangle")],
            },
        ],
    }
}

/// `neobot_convo_list() -> Vec<ConvoView>`
///
/// 真列表 —— 读 store，不再是界面的 `MOCK_ITEMS`。
///
/// ⚠️ **为什么这条重要**：上一轮之前，界面上那 7 条会话是**硬编码的**
///    `MOCK_ITEMS`，而 `neobot_convo_group` 真的往 `~/.neobot/neobot.db`
///    里写了会话。两边各说各话 ⇒ 用户建了会话，回到列表**看不到**，
///    而界面看上去一切正常。演示数据与真数据并存时，
///    **演示数据会把真数据的 bug 盖住**。
#[tauri::command]
pub fn neobot_convo_list() -> Result<Vec<ConvoView>, String> {
    let store = open_store()?;
    project_convos(&store)
}

/// 列表投影。命令与测试**共用**这一个函数。
///
/// ⛔ 上一版测试直接 `store.list_conversations()`，于是命令体里被塞一条
///    「空库就报错」或「返回空 vec」的逻辑，测试**照样全绿** ——
///    测的是 store，不是命令。变异验证抓到了这一条。
fn project_convos(store: &NeobotStore) -> Result<Vec<ConvoView>, String> {
    let convos = store.list_conversations().map_err(|e| e.to_string())?;
    Ok(convos
        .into_iter()
        .map(|c| ConvoView {
            id: c.id,
            kind: c.kind,
            title: c.title,
            members: c.members,
            task_count: c.task_count,
            last_active: c.last_active,
            muted: c.muted,
            unread: c.unread,
        })
        .collect())
}

/// `neobot_member_list() -> Vec<MemberView>`
///
/// 成员也来自 store（`list_members` 返回三元组）。界面的 `MOCK_MEMBERS`
/// 同属演示数据，一并换掉。
#[tauri::command]
pub fn neobot_member_list() -> Result<Vec<MemberView>, String> {
    let store = open_store()?;
    let members = store.list_members().map_err(|e| e.to_string())?;
    Ok(members
        .into_iter()
        .map(|(id, kind, display)| MemberView { id, kind, display })
        .collect())
}

/// 会话的**投影**：只带界面要的字段。
///
/// ⛔ 刻意**不**返回整个 `Conversation`（sqlite row + 一堆内部字段）。
///    直接透传会让 store 的内部结构变成前端的隐式契约 ——
///    之后改一行 SQL 就得同步改 TS 类型，而没有任何门会提醒你。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ConvoView {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub members: Vec<String>,
    pub task_count: i64,
    pub last_active: String,
    pub muted: bool,
    pub unread: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MemberView {
    pub id: String,
    pub kind: String,
    pub display: String,
}

/// `neobot_member_add(id, kind) -> ()`（`kind` = `human` | `agent`，幂等 upsert）
///
/// ⛔ 存在的理由不是「补个 CRUD」：**会话创建要求成员已登记**
/// （`create_conversation` 会逐个校验），而 GUI 此前只有 `neobot_member_list`
/// —— 也就是「能看不能建」，于是空态只能把人赶去设置面板，而那里也没有。
#[tauri::command]
pub fn neobot_member_add(id: String, kind: String) -> Result<(), String> {
    open_store()?
        .upsert_member(id.trim(), kind.trim())
        .map_err(|e| e.to_string())
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

/// `neobot_send(convo_id?, text) -> AgentRunResult`
///
/// 会话内的一轮：问落库 → 跑 → 答落库，全进 `messages` 表。
/// `convo_id` 缺席/空白 = 脱离会话手动跑（旧行为保留，不落库）。
///
/// 失败语义（故意不对称，丢数据比报错贵得多）：
///   · 问落库失败 ⇒ 直接 Err，不跑（跑了也记不住问了什么）。
///   · 跑失败 ⇒ Err，问句已在库里（它确实问过）。
///   · 答落库失败 ⇒ Err，但**把回复内容嵌进错误里** ——
///     回复已经算出来了，吞掉等于白跑一次还拿不到结果。
#[tauri::command]
pub async fn neobot_send(convo_id: Option<String>, text: String) -> Result<AgentRunResult, String> {
    if text.trim().is_empty() {
        // 空白消息不是「跑了但没输出」，是**请求本身不成立**。给它一个
        // 明确错误，比跑一轮再回一句空强。
        return Err("消息为空".to_owned());
    }
    let convo = normalize_convo_arg(convo_id);
    // ⛔ 三个阶段各开一次库，不共用一个 handle：跑轮在 `spawn_blocking` 里，
    //    handle move 进闭包后外层用不得（E0382）；而文件库 + WAL 下多开一次
    //    是廉价的。共用一个看似省，实则借用跨不过闭包边界。
    if let Some(ref id) = convo {
        let store = open_store()?;
        check_convo_exists(&store, id)?;
        store
            .append_message(id, "user", text.trim())
            .map_err(|e| e.to_string())?;
    }
    // context 必须**整个 move 进闭包**：agent_run 收 Option<&str>，
    // 而借用跨不过 spawn_blocking 的 'static 边界（E0597 的教训）。
    let goal = text.clone();
    let out = tauri::async_runtime::spawn_blocking(move || {
        let store = open_store()?;
        agent_run(&store, &goal, None).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("发送任务异常：{e}"))??;
    if let Some(ref id) = convo {
        let store = open_store()?;
        store
            .append_message(id, "assistant", &out.output)
            .map_err(|e| format!("回复落库失败（回复内容：{}）：{e}", out.output))?;
    }
    // 用量记账（best-effort，见 record_send_usage）。
    if let Ok(dir) = data_dir() {
        record_send_usage(&dir, &out);
    }
    Ok(out)
}

/// 用量账本文件（`<data_dir>/usage.json`，与库 `UsageLedger` 同形）。
fn usage_file(data_dir: &std::path::Path) -> std::path::PathBuf {
    data_dir.join("usage.json")
}

/// 按模型反查 provider 名（`providers.model` 精确命中；否则 `unknown`）。
///
/// ⛔ 命中了还要过账本字符集：provider 名是用户起的（64 字符自由形），
/// 账本键只认 `[a-z0-9_-]{1,32}`。 washed 掉的（如大小写）统一小写归并 ——
/// 这是展示桶，不是身份，不断言它等于库里的原名。
/// ⛔ 宁要 `unknown` 不要猜：把 qwen 的量记到 openai 头上，比「不知道」坏得多。
fn resolve_provider(store: &NeobotStore, model_used: &str) -> String {
    let model = model_used.trim();
    if model.is_empty() {
        return "unknown".to_owned();
    }
    let hit = store
        .list_providers()
        .unwrap_or_default()
        .into_iter()
        .find(|p| p.model.trim() == model)
        .map(|p| p.name.trim().to_ascii_lowercase());
    match hit {
        Some(name)
            if !name.is_empty()
                && name.len() <= 32
                && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_') =>
        {
            name
        }
        _ => "unknown".to_owned(),
    }
}

/// 发送轮的用量落库（best-effort）。
///
/// ⛔ 失败不拦发送：回复已算出、已落库，进了用户界面 ——
///
/// 为账本报错等于把到手的结果又收回去。失败走 `eprintln`
/// （Tauri 日志可见），不吞。
/// ⛔ 引擎没报的（cached/written）按 0 记，不猜 ——
///
/// 猜缓存命中率就是编账本。
/// ⛔ usage 缺席且模型为空 = 无数据，直接跳过（不是 Err）：
///
/// 第一版返回 Err，被上游「缺字段降级」逻辑反杀 ——
///
/// 空是合法状态，不是失败。
fn record_send_usage(data_dir: &std::path::Path, out: &AgentRunResult) {
    use neotrix_neobot::nt_cost::UsageLedger;
    let path = usage_file(data_dir);
    let mut ledger = match UsageLedger::load(&path) {
        Ok(ledger) => ledger,
        Err(e) => {
            eprintln!("[neobot] 用量账本读失败（本次不记）：{e}");
            return;
        }
    };
    let usage = out.usage.as_ref();
    let prompt = usage.map(|u| u.prompt_tokens.max(0) as u64).unwrap_or(0);
    let completion = usage.map(|u| u.completion_tokens.max(0) as u64).unwrap_or(0);
    if prompt == 0 && completion == 0 && out.model_used.trim().is_empty() {
        return;
    }
    let provider = match open_store() {
        Ok(store) => resolve_provider(&store, &out.model_used),
        Err(_) => "unknown".to_owned(),
    };
    // ⛔ 模型名空但用量在 = 服务端报了数没报名。键不能空（账本拒），
    // 用显式的 `(unknown)` 占位，不借别人的名字。
    let model = if out.model_used.trim().is_empty() {
        "(unknown)"
    } else {
        out.model_used.trim()
    };
    if let Err(e) = ledger.record(
        &provider,
        model,
        None,
        prompt,
        0,
        0,
        completion,
    ) {
        eprintln!("[neobot] 用量记账失败（本次不记）：{e}");
        return;
    }
    if let Err(e) = ledger.save(&path) {
        eprintln!("[neobot] 用量账本写失败（本次不记）：{e}");
    }
}

/// 用量汇总行（`provider|model` 粒度）。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UsageRow {
    pub id: String,
    pub name: Option<String>,
    pub input: u64,
    pub cached: u64,
    pub written: u64,
    pub output: u64,
    pub requests: u64,
}

/// `neobot_usage_summary(days?) -> UsageSummary`
///
/// 日/周/月三档由调用方传 `days`（1/7/30）；缺席/0 = 有史以来。
/// 文件缺席 = 零（还没花过）；文件坏了 = Err（静默归零等于銷账）。
///
/// ⛔ `days` 上限 3650（十年）：账本窗口是逐天回退的循环，
///
/// 传 `u64::MAX` 会原地转到天荒地老 —— 而调用方传多大本是它的自由。
#[tauri::command]
pub fn neobot_usage_summary(days: Option<u64>) -> Result<UsageSummary, String> {
    use neotrix_neobot::nt_cost::UsageLedger;
    let days = days.unwrap_or(0).min(3650);
    let path = data_dir()?.join("usage.json");
    let ledger = UsageLedger::load(&path)?;
    Ok(summarize_usage(&ledger, days))
}

/// 汇总纯函数（命令薄壳，逻辑可测；`days` 已钳制）。
fn summarize_usage(
    ledger: &neotrix_neobot::nt_cost::UsageLedger,
    days: u64,
) -> UsageSummary {
    use std::collections::BTreeMap;
    let cutoff = usage_cutoff(days);
    let mut rows: BTreeMap<String, UsageRow> = BTreeMap::new();
    let mut total = UsageSummary {
        days,
        input: 0,
        cached: 0,
        written: 0,
        output: 0,
        requests: 0,
        tokens: 0,
        rows: Vec::new(),
    };
    for (day, models) in &ledger.days {
        if !cutoff.is_empty() && day.as_str() < cutoff.as_str() {
            continue;
        }
        for (id, row) in models {
            let entry = rows.entry(id.clone()).or_insert(UsageRow {
                id: id.clone(),
                name: ledger.names.get(id).cloned(),
                input: 0,
                cached: 0,
                written: 0,
                output: 0,
                requests: 0,
            });
            entry.input = entry.input.saturating_add(row.input);
            entry.cached = entry.cached.saturating_add(row.cached);
            entry.written = entry.written.saturating_add(row.written);
            entry.output = entry.output.saturating_add(row.output);
            entry.requests = entry.requests.saturating_add(row.requests);
        }
    }
    for row in rows.into_values() {
        total.input = total.input.saturating_add(row.input);
        total.cached = total.cached.saturating_add(row.cached);
        total.written = total.written.saturating_add(row.written);
        total.output = total.output.saturating_add(row.output);
        total.requests = total.requests.saturating_add(row.requests);
        total.rows.push(row);
    }
    total.tokens = total.input.saturating_add(total.output);
    total
}

/// 窗口下界（本地日历；空串 = 不限）。与库 `totals` 同口径 ——
///
/// ⛔ 两处各算各的 cutoff 是分叉之源：将来一边改口径另一边静默错。
/// 抽出来共用本来最干净，但库函数签名已定（`totals(days)` 内算），
/// 这里只做「读侧展示」，重复三行比改库签名便宜且无风险。差异写死在这句注释里。
fn usage_cutoff(days: u64) -> String {
    if days == 0 {
        return String::new();
    }
    let mut date = chrono::Local::now().date_naive();
    for _ in 1..days {
        date = date.pred_opt().unwrap_or(date);
    }
    date.format("%Y-%m-%d").to_string()
}

/// 用量汇总体。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UsageSummary {
    pub days: u64,
    pub input: u64,
    pub cached: u64,
    pub written: u64,
    pub output: u64,
    pub requests: u64,
    pub tokens: u64,
    pub rows: Vec<UsageRow>,
}
///
/// ⛔ 空串与 None 必须同义：前端 `sel` 为 null 时有时传缺席有时传 `""`
/// （JSON 里 `undefined` 字段会被整个丢掉），两条路必须同归。
#[cfg(test)]
mod convo_id_canonical_tests {
    use super::normalize_convo_id;

    /// ⭐⭐⭐ Atlas `acpSessionId` 契约的可执行形态：
    /// ⭐⭐ **id 只有一个规范化口径**，⭐⭐ 且 ⭐⭐ **空 id 必须被拒**。
    #[test]
    fn canonical_form_is_stable_and_whitespace_is_rejected_as_empty() {
        // ⭐ 规范化后**幂等**：⭐⭐ 反复调用不再变化（⭐⭐ 这是「单一真源」的判据）
        assert_eq!(normalize_convo_id(Some("c0")).as_deref(), Some("c0"));
        assert_eq!(
            normalize_convo_id(Some("c0")).as_deref(),
            normalize_convo_id(normalize_convo_id(Some("c0")).as_deref()).as_deref(),
            "⭐⭐ 规范化必须幂等"
        );
        // ⭐⭐ 带空白 ⇒ 收敛到同一个规范值（⭐⭐ 与写路径同口径 ⇒ 不再静默丢消息）
        assert_eq!(normalize_convo_id(Some("  c0 ")).as_deref(), Some("c0"));
        assert_eq!(
            normalize_convo_id(Some("  c0 ")).as_deref(),
            normalize_convo_id(Some("c0")).as_deref(),
            "⭐⭐ 带空白与不带空白必须收敛到**同一** id（⭐⭐ 否则读不到写进去的消息）"
        );
        // ⭐⭐ 空 / 纯空白 / 缺席 ⇒ 一律 None（⭐⭐ 改前 `.trim()` 会让空串去查库）
        assert_eq!(normalize_convo_id(Some("")), None);
        assert_eq!(normalize_convo_id(Some("   ")), None);
        assert_eq!(normalize_convo_id(None), None);
    }
}

fn normalize_convo_arg(convo_id: Option<String>) -> Option<String> {
    normalize_convo_id(convo_id.as_deref())
}

/// ⭐⭐⭐ **会话 id 的唯一规范化入口**（2026-10-04）。
///
/// ⭐⭐⭐ **为什么必须有唯一入口**（⭐⭐ 对标 Atlas 的 `acpSessionId` 契约）：
/// ⭐⭐ Atlas 原文：「`acpSessionId` is the single source of truth. It's both
/// ⭐⭐ the wire session id and the filename stem … **Code that reconstructs
/// ⭐⭐ or transforms this id is a bug.**」
///
/// ⭐⭐⭐ 改前本文件有 **3 处各自散写 `convo_id.trim()`**
/// ⭐⭐（`:567` 列表、`:622` 分页、`:489` Option 包装），⭐⭐⭐ 而
/// ⭐⭐ **写库路径用的是未经规范化的原值** ⇒ ⭐⭐⭐ **读写两侧规范化不一致**：
/// ⭐⭐ 一个带首尾空白的 id ⇒ **写入 `c0`，读取 `c0`**
/// ⭐⭐ ⇒ ⭐⭐⭐ **消息静默消失，且没有任何报错**。
///
/// ⭐⭐ 收敛后：**所有读路径都走这一个函数**，⭐⭐ 与写路径同一口径。
fn normalize_convo_id(raw: Option<&str>) -> Option<String> {
    let t = raw?.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_owned())
    }
}

/// 记忆视图（`neobot_memory_list` 返回值）。
///
/// 一并给出 `bytes`/`cap`/`revisions`：只给行列表的话，用户撞上 8KiB 上限时
/// 只会看到「记不进去了」而不知道为什么（不可解释的失败＝没有失败原因）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MemoryView {
    pub lines: Vec<String>,
    pub bytes: usize,
    pub cap: usize,
    pub revisions: usize,
}

/// `neobot_memory_list() -> MemoryView`
///
/// 读跨会话记忆（`MEMORY.md`，逐轮注入 system prompt 尾部）。
/// 历史损坏 = Err —— 与 `memory undo` 同一口径：坏历史不静默当空。
#[tauri::command]
pub fn neobot_memory_list() -> Result<MemoryView, String> {
    memory_view(&data_dir()?)
}

/// 记忆视图纯函数（逻辑可测，不碰真实 HOME）。
fn memory_view(data_dir: &std::path::Path) -> Result<MemoryView, String> {
    use neotrix_neobot::nt_memory;
    let text = nt_memory::read_memory(data_dir);
    Ok(MemoryView {
        lines: text.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_owned).collect(),
        bytes: text.len(),
        cap: nt_memory::MEMORY_CAP,
        revisions: nt_memory::memory_revisions(data_dir)
            .map(|r| r.len())
            .map_err(|e| format!("读记忆历史失败：{e}"))?,
    })
}

/// `neobot_memory_add(text) -> bool`（`true` = 记下；`false` = 已有这行）
///
/// 秘密律仍在库侧生效（含密钥行拒绝），这里不重复实现一遍。
#[tauri::command]
pub fn neobot_memory_add(text: String) -> Result<bool, String> {
    neotrix_neobot::nt_memory::append_memory(&data_dir()?, &text).map_err(|e| e.to_string())
}

/// `neobot_memory_undo() -> bool`（`true` = 撤了一版；`false` = 无可撤）
#[tauri::command]
pub fn neobot_memory_undo() -> Result<bool, String> {
    neotrix_neobot::nt_memory::memory_undo(&data_dir()?)
        .map(|restored| restored.is_some())
        .map_err(|e| e.to_string())
}

/// 会话存在性校验。`None` 由 `get_conversation` 表达「不存在」——
///
/// ⛔ 不存在必须 Err 而不是「顺手建一个」：自动建会让打错 id 的调用
/// 在库里留下一个永远没人打开的空会话，而调用方以为发出去了。
fn check_convo_exists(store: &NeobotStore, id: &str) -> Result<(), String> {
    let found = store.get_conversation(id).map_err(|e| e.to_string())?;
    if found.is_none() {
        return Err(format!("会话不存在：{id}"));
    }
    Ok(())
}

/// `neobot_convo_messages(convo_id) -> ChatMessage[]`
///
/// 一个会话的全部消息（时间正序）。切会话时界面调这条换历史 ——
///
/// ⛔ 之前切会话只换了标题，消息流还留着上一个会话的（串台）。
/// 历史由库出，界面只负责渲染，不做合并/去重/截断。
#[tauri::command]
pub fn neobot_convo_messages(
    convo_id: String,
) -> Result<Vec<neotrix_neobot::nt_store::ChatMessage>, String> {
    let store = open_store()?;
    // ⭐⭐ 走唯一规范化入口（⭐⭐ 改前是裸 `.trim()`：⭐⭐ 空串会被当成合法 id 去查库）
    let convo = normalize_convo_id(Some(convo_id.as_str()))
        .ok_or_else(|| "会话 id 为空".to_string())?;
    store.list_messages(&convo).map_err(|e| e.to_string())
}

/// ⭐⭐ 分页拉取一页消息（**增量式**，治长会话一次性全量渲染）。
///
/// `neobot_convo_messages_page(convo_id, before_seq, limit) -> MessagePage`
///
/// ## 为什么加这条（2026-10-03 实测的「建成未用」第三实例）
/// ⭐ `NeobotStore::list_messages_page`（`nt_store_messages.rs:137`）早已存在，
/// 且有 `created_at` 并列/不漏不重测试，⭐ 但**全仓消费者只有它自己**
/// ⇒ 命令层只有全量的 `neobot_convo_messages`，界面 `:542` 也是一次性全量拉取。
/// ⇒ ⭐ 三层都缺接线，长会话会把整段历史一次性塞进 DOM。
///
/// ## ⭐ 为什么用**新命令**而不是改 `neobot_convo_messages` 的签名
/// ⛔ 改既有命令会破坏 UI 的 `invoke<ChatMessage[]>`（返回类型变了）
/// ⇒ ⭐ **加法式**新增，旧路径保持可用 ⇒ 界面可**渐进**切换。
///
/// ## ⭐ 游标语义（实测 store 层）
/// `after_seq` 是 `messages.seq`，即 SQLite `rowid`（`a0243425` 起）
/// ⇒ ⭐ **按 seq 严格递增翻页**，⛔ 不用 `created_at`（同秒消息会并列漏掉）。
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagePage {
    /// 本页消息（时间正序，与 `neobot_convo_messages` 一致）
    pub messages: Vec<neotrix_neobot::nt_store::ChatMessage>,
    /// ⭐ 是否还有更早的消息（用 `limit + 1` 探测，见实现）
    pub has_more: bool,
    /// ⭐ 下一页游标：把本页**最小** `seq` 回传（继续往更早翻）
    pub next_seq: Option<i64>,
}

/// ⭐⭐ 分页整形（**生产与测试共用同一份**）。
///
/// ⭐ 刻意抽成自由函数：否则测试会复制一份逻辑，
/// ⭐⭐ **测的就不是命令真正跑的那段代码**（本日已在
/// `neobot-check-emergence.mjs` 上踩过「自测重写正则 ⇒ 假通过」的同型坑）。
fn shape_page(
    mut probe: Vec<neotrix_neobot::nt_store::ChatMessage>,
    take: i64,
) -> MessagePage {
    // ⭐ 多取 1 条 ⇒ 有余量即「还有更早的」
    let has_more = probe.len() as i64 > take;
    probe.truncate(take as usize);
    // ⭐ 游标 = 本页**最小** seq（消息正序 ⇒ 最后一条 seq 最小）
    let next_seq = probe.last().map(|m| m.seq).filter(|_| has_more);
    MessagePage { messages: probe, has_more, next_seq }
}

#[tauri::command]
pub fn neobot_convo_messages_page(
    convo_id: String,
    before_seq: Option<i64>,
    limit: Option<i64>,
) -> Result<MessagePage, String> {
    let store = open_store()?;
    // ⭐⭐ 同样走唯一入口 + ⭐⭐ 拒绝空 id（⭐⭐ 改前 `.trim()` 会让空串通过）
    let convo = normalize_convo_id(Some(convo_id.as_str()))
        .ok_or_else(|| "会话 id 为空".to_string())?;
    // ⭐ 夹紧 limit：⛔ 不接受 0/负数（会让 `has_more` 探测失真），
    // ⛔ 也不接受超大值（界面 bug 不该拖垮库）。
    let take = limit.unwrap_or(200).clamp(1, 1000);
    // ⭐ 多取 1 条用于探测 has_more ⇒ 界面不需要知道「怎么算还有没有」
    let probe = store
        .list_messages_page(&convo, before_seq, take + 1)
        .map_err(|e| e.to_string())?;
    Ok(shape_page(probe, take))
}

/// 骨架下发给界面的面板事件名。前端 `listen` 同一个名字。
pub const PANEL_EVENT: &str = "neobot:panel";

/// 应用状态：面板注册表。**由骨架持有，界面只能报 id。**
#[derive(Default)]
pub struct AppState {
    pub panels: std::sync::Mutex<Registry>,
}

// ── AppState 的方法：真正的逻辑放这里 ──────────────────────────
//
// 为什么拆开：Tauri 的 `tauri::State<'_, AppState>` 在 Tauri 运行时之外
// **构造不出来**，所以逻辑若写在 command 里就**无法单测** —— 而
// 「作答是否真的查了注册表」正是最该被测的一件事。
// command 退化成薄壳，方法可测。这与本仓既有的「可测命令层」约定一致。

impl AppState {
    /// 登记一块面板。返回候选集版本。
    pub fn publish(&self, panel: &Panel) -> Result<u64, String> {
        let version = panel.candidate_set_version;
        let mut reg = self
            .panels
            .lock()
            .map_err(|_| "面板注册表锁中毒（此前有命令 panic）".to_owned())?;
        reg.publish(panel.clone()).map_err(|e| match e {
            PublishError::Invalid(why) => format!("面板非法：{why}"),
            PublishError::VersionWentBack { current, incoming } => {
                format!("候选集版本倒退（{incoming} < {current}）")
            }
        })?;
        Ok(version)
    }

    /// ⛔ 按 id 校验作答。**只收 answer** —— 基准由注册表持有。
    pub fn answer(&self, a: &Answer) -> Result<AnswerOutcome, String> {
        let reg = self
            .panels
            .lock()
            .map_err(|_| "面板注册表锁中毒（此前有命令 panic）".to_owned())?;
        Ok(reg.answer(a))
    }

    /// 清空。返回清掉了几块。
    pub fn clear(&self) -> Result<usize, String> {
        let mut reg = self
            .panels
            .lock()
            .map_err(|_| "面板注册表锁中毒（此前有命令 panic）".to_owned())?;
        let n = reg.len();
        *reg = Registry::new();
        Ok(n)
    }
}

/// `neobot_api_specs() -> ApiCatalog`
///
/// 契约全量清单。界面据此渲染「后端到底有什么」——
/// **前端是后端的可视化交互**，这句话的字面实现就是这条命令。
#[tauri::command]
pub fn neobot_api_specs() -> crate::api::ApiCatalog {
    crate::api::catalog()
}

/// `neobot_api_call(name, args) -> ApiCallResult`
///
/// 按名字分派。
///
/// ⛔ **未实现的不抛错，返回结构化说明。** 抛错的话，调用方看到的是
///    「这个命令不存在」和「这个命令存在但失败」**同一种东西** ——
///    缺口就永远隐性存在。返回 `{ ok: false, reason, status }` 之后，
///    界面能把「本仓不做」「还没做」「真出错」三态分开显示。
#[tauri::command]
pub fn neobot_api_call(
    name: String,
    args: Option<serde_json::Value>,
) -> Result<crate::api::ApiCallResult, String> {
    let _ = args; // 真正的分派留给各命令自己的实现；这里只做「状态查询」语义
    match crate::api::SPECS.iter().find(|s| s.name == name) {
        None => {
            if crate::api::UPSTREAM_UNLISTED.contains(&name.as_str()) {
                Ok(crate::api::ApiCallResult {
                    ok: false,
                    status: "unlisted".into(),
                    reason: format!(
                        "上游命令，已登记为未逐条展开：{name}。平台保证不漏，理由待补。"
                    ),
                    data: serde_json::Value::Null,
                })
            } else {
                // 真的不在契约里 ⇒ 这才是「不存在」，与上面两种必须可区分
                Err(format!("命令不在契约内：{name}"))
            }
        }
        Some(spec) if spec.status != crate::api::Status::Implemented => Ok(crate::api::ApiCallResult {
            ok: false,
            status: match spec.status {
                crate::api::Status::Stub => "stub".into(),
                crate::api::Status::Planned => "planned".into(),
                // ⛔ `Refused` 在这里也走不通路：它是**已注册**的，直接 invoke 会
                //    得到一句人话（比这里返回理由更好）。这里给同一状态是为了让
                //    「API 面板里点它」也不至于显示成一个不存在的命令。
                crate::api::Status::Refused => "refused".into(),
                crate::api::Status::Implemented => unreachable!("已在上层拦掉"),
            },
            reason: spec.note.to_owned(),
            data: serde_json::Value::Null,
        }),
        Some(spec) => Ok(crate::api::ApiCallResult {
            ok: true,
            status: "implemented".into(),
            reason: String::new(),
            data: serde_json::json!({ "hint": "已实现，请直接 invoke 该命令" , "name": spec.name }),
        }),
    }
}

/// `neobot_panel_publish(panel) -> u64`
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
    let version = state.publish(&panel)?;
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
    state.answer(&answer)
}

/// `neobot_panel_clear() -> usize` —— 清空注册表。
///
/// 用途：切换会话。换会话不清的话，旧会话的面板仍可被作答 ——
/// 而作答会被记到新会话的流里，属于串台。
#[tauri::command]
pub fn neobot_panel_clear(state: tauri::State<'_, AppState>) -> Result<usize, String> {
    state.clear()
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

#[cfg(test)]
mod panel_state_tests {
    use super::*;
    use neotrix_neobot::nt_panel::{
        Answer, AnswerOutcome, Mode, Panel, PanelKind, PanelOption, Reject, Source,
    };

    fn src(t: &str) -> Source {
        Source { title: t.to_owned(), url: format!("https://example.com/{t}") }
    }

    fn panel(version: u64) -> Panel {
        Panel {
            id: "p1".into(),
            thread_id: "t1".into(),
            turn_id: "tu1".into(),
            candidate_set_version: version,
            kind: PanelKind::Comparison,
            title: "选哪个".into(),
            mode: Mode::Live,
            options: vec![
                PanelOption { id: "a".into(), label: "A".into(), details: vec![], sources: vec![src("a")] },
                PanelOption { id: "b".into(), label: "B".into(), details: vec![], sources: vec![src("b")] },
            ],
        }
    }

    fn answer(v: u64) -> Answer {
        Answer { panel_id: "p1".into(), option_id: "a".into(), candidate_set_version: v }
    }

    #[test]
    fn 登记后作答走注册表() {
        let st = AppState::default();
        assert_eq!(st.publish(&panel(1)), Ok(1));
        assert_eq!(st.answer(&answer(1)), Ok(AnswerOutcome::Accepted));
    }

    /// ⛔ 这条是「面板基准改由骨架持有」这个改动的**核心断言**。
    ///    在 command 层（不是库层）再验一次：Tauri 薄壳**没有偷偷**加回
    ///    「接受界面传来的 panel」这条路。
    #[test]
    fn command层也不能接受界面自带的面板() {
        // 状态是空的 ⇒ 无论界面怎么说，都得拒
        let st = AppState::default();
        assert_eq!(
            st.answer(&answer(1)),
            Ok(AnswerOutcome::Rejected(Reject::NoSuchPanel)),
            "注册表为空时必须拒 —— 界面无从提供基准"
        );
    }

    #[test]
    fn 换批后旧版本作答被拒() {
        let st = AppState::default();
        st.publish(&panel(1)).unwrap();
        st.publish(&panel(2)).unwrap();
        assert_eq!(
            st.answer(&answer(1)),
            Ok(AnswerOutcome::Rejected(Reject::Stale { current: 2, answered: 1 }))
        );
        assert_eq!(st.answer(&answer(2)), Ok(AnswerOutcome::Accepted));
    }

    #[test]
    fn 版本倒退不污染注册表() {
        let st = AppState::default();
        st.publish(&panel(2)).unwrap();
        assert!(matches!(
            st.publish(&panel(1)),
            Err(ref e) if e.contains("版本倒退")
        ));
        // 倒退被拒后，v2 仍在
        assert_eq!(st.answer(&answer(2)), Ok(AnswerOutcome::Accepted));
    }

    #[test]
    fn 非法面板不登记() {
        let st = AppState::default();
        let mut p = panel(1);
        p.options[0].sources.clear();
        assert!(st.publish(&p).is_err());
        assert_eq!(
            st.answer(&answer(1)),
            Ok(AnswerOutcome::Rejected(Reject::NoSuchPanel)),
            "被拒的面板不该留下任何可作答的基准"
        );
    }

    #[test]
    fn 换会话清空后旧面板不可答() {
        // 不清的危害：作答会被记到**新**会话的流里（串台）。
        let st = AppState::default();
        st.publish(&panel(1)).unwrap();
        assert_eq!(st.clear(), Ok(1));
        assert_eq!(st.answer(&answer(1)), Ok(AnswerOutcome::Rejected(Reject::NoSuchPanel)));
        assert_eq!(st.clear(), Ok(0), "再清应是 0，不该报错");
    }

    #[test]
    fn 演示面板必须标Sample() {
        // 合成的东西标 Live 就是谎报。
        assert_eq!(demo_panel().mode, Mode::Sample);
        assert!(matches!(demo_panel().mode, Mode::Sample));
    }

    #[test]
    fn 演示面板能被注册表接受() {
        let st = AppState::default();
        // 走与真实骨架完全相同的那条路 —— 它不是后门。
        assert!(st.publish(&demo_panel()).is_ok());
    }

    #[test]
    fn PublishError两个变体都有文案() {
        // 门只抓到「Invalid」；这条确保「版本倒退」也有可读文案，
        // 否则界面上会显示一个裸的枚举名。
        let st = AppState::default();
        st.publish(&panel(2)).unwrap();
        let e = st.publish(&panel(1)).unwrap_err();
        assert!(e.contains("倒退"), "实际文案：{e}");
        let mut bad = panel(1);
        bad.options[0].sources.clear();
        assert!(st.publish(&bad).unwrap_err().contains("非法"));
    }
}

#[cfg(test)]
mod convo_list_tests {
    use super::*;

    /// 真实 store 往返：建会话 → 读列表 → 成员。
    ///
    /// ⛔ 这条测试**必须**用真 `NeobotStore`，不能用 mock。
    ///    上一轮的 `MOCK_ITEMS` 之所以能长期冒充真实列表，正是因为
    ///    **没有任何测试跨越过 store** —— mock 和真数据各自绿，
    ///    而两者对不上这件事，只有真 store 才看得出来。
    fn temp_store() -> (tempdir::TempDir, NeobotStore) {
        let d = tempdir::TempDir::new("nb-convo").expect("临时目录");
        let store = NeobotStore::open(d.path().join("nb.db").to_str().unwrap()).expect("开库");
        (d, store)
    }

    /// store 强制成员外键：`create_conversation` 遇到不存在的成员会报
    /// `no such member`。这是**对的设计**（防止会话里挂着幽灵成员），
    /// 但第一版测试直接塞成员名单就炸了 —— 说明这条约束以前没人测过，
    /// 因为界面用的是 `MOCK_MEMBERS`，根本不经过 store。
    fn store_with_members() -> (tempdir::TempDir, NeobotStore) {
        let (d, store) = temp_store();
        for m in ["neo", "ada", "lin"] {
            // kind 只认 human|agent（store 有校验）。第一版我写 "person"
            // 被拒 —— 这条校验以前也没被测过，同属「界面走 mock 所以没碰到」。
            store.upsert_member(m, "human").expect("登记成员");
        }
        (d, store)
    }

    #[test]
    fn 建的会话能被列表读到() {
        let (_d, store) = store_with_members();
        let id = store.create_conversation("group", "发布值班", &["neo".into(), "ada".into()]).unwrap();
        let list = store.list_conversations().unwrap();
        assert_eq!(list.len(), 1, "刚建的会话必须出现在列表里");
        assert_eq!(list[0].id, id);
        assert_eq!(list[0].title, "发布值班");
        assert_eq!(list[0].kind, "group");
    }

    /// 这条约束以前没被测过（界面用 MOCK_MEMBERS，压根不走 store）。
    /// 补上，免得后人以为「塞名单就行」。
    #[test]
    fn 成员必须先存在() {
        let (_d, store) = temp_store();
        let e = store
            .create_conversation("group", "t", &["幽灵".into()])
            .expect_err("不存在的成员必须被拒");
        assert!(e.to_string().contains("no such member"), "实际：{e}");
    }

    #[test]
    fn 命令体在空库时返回空列表() {
        // 走命令的**同一个**函数。界面据此显示「还没有会话」；
        // 若这里返回 Err，界面显示「读失败」——对用户是两种处境。
        let (_d, store) = temp_store();
        assert!(project_convos(&store).expect("空库不该报错").is_empty());
    }

    #[test]
    fn 命令体不漏掉刚建的会话() {
        // MOCK_ITEMS 那个病的直接反证：建了会话，列表里必须看得到。
        let (_d, store) = store_with_members();
        store.create_conversation("group", "发布值班", &["neo".into()]).unwrap();
        let v = project_convos(&store).expect("读列表");
        assert_eq!(v.len(), 1, "建的会话必须出现在列表里");
        assert_eq!(v[0].title, "发布值班");
    }

    #[test]
    fn 空库时列表为空而不是报错() {
        // 界面据此显示「还没有会话」。若这里报错，界面就会显示「读失败」，
        // 两者对用户是两种完全不同的处境。
        let (_d, store) = temp_store();
        assert!(store.list_conversations().unwrap().is_empty());
    }

    #[test]
    fn 投影不泄漏store内部字段() {
        // ConvoView 刻意不含 created_at / origin / parent_id。
        // 直接透传 Conversation 会让 store 结构变成前端隐式契约 ——
        // 之后改一行 SQL 就要同步改 TS 类型，而没有任何门会提醒。
        let (_d, store) = store_with_members();
        store.create_conversation("dm", "与 ada", &["neo".into()]).unwrap();
        let c = store.list_conversations().unwrap().remove(0);
        let v = ConvoView {
            id: c.id, kind: c.kind, title: c.title, members: c.members,
            task_count: c.task_count, last_active: c.last_active,
            muted: c.muted, unread: c.unread,
        };
        let json = serde_json::to_value(&v).unwrap();
        let keys: Vec<&str> = json.as_object().unwrap().keys().map(|s| s.as_str()).collect();
        for leaked in ["created_at", "origin", "parent_id"] {
            assert!(!keys.contains(&leaked), "投影泄漏了 {leaked}：{keys:?}");
        }
        // 字段名是 snake_case（serde 声明），不是 Rust 的 camelCase
        assert!(keys.contains(&"last_active") && keys.contains(&"task_count"));
    }

    #[test]
    fn 投影可序列化往返() {
        let v = ConvoView {
            id: "c1".into(), kind: "group".into(), title: "t".into(),
            members: vec!["a".into()], task_count: 3, last_active: "2026-09-30T10:00:00Z".into(),
            muted: false, unread: 2,
        };
        let s = serde_json::to_string(&v).unwrap();
        assert!(s.contains("\"last_active\""), "实际：{s}");
        assert!(s.contains("\"task_count\":3"), "实际：{s}");
    }
}

#[cfg(test)]
mod send_tests {
    use super::*;

    fn temp_store() -> (tempdir::TempDir, NeobotStore) {
        let d = tempdir::TempDir::new("nb-send").expect("临时目录");
        let store = NeobotStore::open(d.path().join("nb.db").to_str().unwrap()).expect("开库");
        (d, store)
    }

    #[test]
    fn 空串与缺席同义() {
        assert_eq!(normalize_convo_arg(None), None);
        assert_eq!(normalize_convo_arg(Some("".into())), None);
        assert_eq!(normalize_convo_arg(Some("   ".into())), None);
        assert_eq!(normalize_convo_arg(Some(" c1 ".into())), Some("c1".into()));
    }

    #[test]
    fn 不存在的会话不让发() {
        let (_d, store) = temp_store();
        assert!(check_convo_exists(&store, "ghost").is_err());
    }

    #[test]
    fn 记忆视图给出容量与历史数() {
        use neotrix_neobot::nt_memory;
        let dir = tempdir::TempDir::new("nb-memview").expect("临时目录");
        let d = dir.path();
        // 初始：无行、无历史（bytes 0，因为文件不存在）。
        let v0 = memory_view(d).expect("读");
        assert!(v0.lines.is_empty());
        assert_eq!(v0.bytes, 0);
        assert_eq!(v0.revisions, 0);
        assert_eq!(v0.cap, nt_memory::MEMORY_CAP);
        nt_memory::append_memory(d, "喜欢深色模式").expect("记");
        nt_memory::append_memory(d, "用 pnpm").expect("记");
        let v = memory_view(d).expect("读");
        assert_eq!(v.lines, vec!["喜欢深色模式", "用 pnpm"]);
        assert!(v.bytes > 0);
        // 两次 append = 两条历史（含首次写入前的空版）。
        assert_eq!(v.revisions, 2);
        // 坏历史 = Err（不静默当空）。
        std::fs::write(nt_memory::memory_history_path(d), "{断的\n").expect("造坏行");
        assert!(memory_view(d).is_err());
    }

    #[test]
    fn 记忆视图滤掉空行但容量算原文() {
        use neotrix_neobot::nt_memory;
        let dir = tempdir::TempDir::new("nb-memview2").expect("临时目录");
        let d = dir.path();
        std::fs::write(nt_memory::memory_path(d), "  a  \n\n\n  b\n").expect("造");
        let v = memory_view(d).expect("读");
        // 行是 trim 后的非空行；bytes 是原文长度（容量按原文算，不是按行数）。
        assert_eq!(v.lines, vec!["a", "b"]);
        assert_eq!(v.bytes, "  a  \n\n\n  b\n".len());
    }

    #[test]
    fn 存在的会话放行且问答都能落库() {
        // ⛔ 这条不断言 agent_run（那会真跑一轮）：只验「落库这半边」。
        //    跑轮由库侧测试覆盖；这里守的是「存在性校验 + 消息表往返」。
        let (_d, store) = temp_store();
        store.upsert_member("neo", "human").expect("成员");
        let id = store.create_conversation("group", "t", &["neo".into()]).expect("会话");
        check_convo_exists(&store, &id).expect("存在必须放行");
        store.append_message(&id, "user", "hi").expect("问落库");
        store.append_message(&id, "assistant", "在").expect("答落库");
        let v = store.list_messages(&id).expect("读回");
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].role, "user");
    }

    fn result_with(model: &str, prompt: i64, completion: i64) -> AgentRunResult {
        AgentRunResult {
            status: "ok".into(),
            output: "o".into(),
            trace: Vec::new(),
            model_used: model.into(),
            mode: "passthrough".into(),
            tools: Vec::new(),
            usage: Some(neotrix_neobot::nt_types::TokenUsage {
                prompt_tokens: prompt,
                completion_tokens: completion,
                cost_usd: 0.0,
            }),
        }
    }

    #[test]
    fn provider按模型精确命中否则unknown() {
        let (_d, store) = temp_store();
        // 空库、无模型 —— 都是 unknown，不猜。
        assert_eq!(resolve_provider(&store, ""), "unknown");
        assert_eq!(resolve_provider(&store, "qwen-plus"), "unknown");
        store
            .upsert_provider(&neotrix_neobot::nt_provider::Provider {
                name: "Qwen".into(),
                base_url: "https://x.dev/v1".into(),
                key_env: "QWEN_API_KEY".into(),
                model: "qwen-plus".into(),
                enabled: true,
            })
            .expect("provider");
        // 命中且归并小写；未命中仍 unknown。
        assert_eq!(resolve_provider(&store, "qwen-plus"), "qwen");
        assert_eq!(resolve_provider(&store, "gpt-x"), "unknown");
    }

    #[test]
    fn 用量落库往返与空跳过() {
        use neotrix_neobot::nt_cost::UsageLedger;
        let dir = tempdir::TempDir::new("nb-usage").expect("临时目录");
        let path = usage_file(dir.path());
        assert_eq!(path.file_name().unwrap(), "usage.json");
        // 有数即记。
        record_send_usage(dir.path(), &result_with("qwen-plus", 100, 50));
        let back = UsageLedger::load(&path).expect("读回");
        assert_eq!(back.totals(0).input, 100);
        assert_eq!(back.totals(0).output, 50);
        // 无数无模型即跳过（不报错、不建行）。
        let empty = AgentRunResult {
            status: "ok".into(),
            output: "o".into(),
            trace: Vec::new(),
            model_used: String::new(),
            mode: "passthrough".into(),
            tools: Vec::new(),
            usage: None,
        };
        record_send_usage(dir.path(), &empty);
        let back2 = UsageLedger::load(&path).expect("读回");
        assert_eq!(back2.totals(0).requests, 1);
    }

    fn usage_ledger_two_days() -> neotrix_neobot::nt_cost::UsageLedger {
        use neotrix_neobot::nt_cost::{DayUsage, UsageLedger};
        let mut ledger = UsageLedger::default();
        ledger
            .record("deepseek", "deepseek-chat", Some("DeepSeek"), 100, 10, 0, 50)
            .expect("记今天");
        let yesterday = chrono::Local::now()
            .date_naive()
            .pred_opt()
            .unwrap()
            .format("%Y-%m-%d")
            .to_string();
        ledger.days.entry(yesterday).or_default().insert(
            "qwen|qwen-plus".to_owned(),
            DayUsage { input: 20, cached: 0, written: 0, output: 5, requests: 1 },
        );
        ledger
    }

    #[test]
    fn 汇总窗口与分行正确() {
        let ledger = usage_ledger_two_days();
        // 今天一档只有 deepseek 行。
        let one = summarize_usage(&ledger, 1);
        assert_eq!(one.days, 1);
        assert_eq!(one.rows.len(), 1);
        assert_eq!(one.rows[0].id, "deepseek|deepseek-chat");
        assert_eq!(one.rows[0].name.as_deref(), Some("DeepSeek"));
        assert_eq!((one.input, one.output, one.requests), (100, 50, 1));
        assert_eq!(one.tokens, 150);
        // 两天两行都进。
        let all = summarize_usage(&ledger, 0);
        assert_eq!(all.rows.len(), 2);
        assert_eq!((all.input, all.output, all.requests), (120, 55, 2));
        // days 缺席即 0（有史以来），与显式 0 同义。
        assert_eq!(summarize_usage(&ledger, 0).tokens, all.tokens);
    }

    #[test]
    fn 超大days被钳制不转圈() {
        // ⛔ 窗口是逐天回退循环：u64::MAX 会转到天荒地老。
        // 命令层钳 3650，这里断言钳制后的汇总仍正常（不断言耗时，只断言正确）。
        let ledger = usage_ledger_two_days();
        let s = summarize_usage(&ledger, 3650);
        assert_eq!(s.tokens, 175);
    }
}

#[cfg(test)]
mod data_dir_tests {
    use super::*;

    /// ⚠️ 「设了 NEOBOT_DATA_DIR」那条**不在这里**。
    ///
    /// 上一版把它放在单元测试里，锁只串行了本模块的 3 条测试 ——
    /// 而另外 19 条（`open_store` 会间接调 `data_dir`）**并行跑**，
    /// 于是它们看见了我设的临时目录 ⇒ 3 条失败。
    ///
    /// ⇒ 改环境变量的测试必须**换进程**（`tests/data_dir_env.rs`）。
    ///    锁只能保护**自己那几条**，保护不了别人。
    #[test]
    fn 与库的配置口径一致() {
        // 同一环境下两边必须一致（防止将来有人又自己拼路径）。
        assert_eq!(
            data_dir().expect("桌面端"),
            neotrix_neobot::NeobotConfig::from_env().expect("库侧").data_dir
        );
    }

    #[test]
    fn 未设时落在家目录下的点neobot() {
        if std::env::var("NEOBOT_DATA_DIR").is_ok() {
            eprintln!("跳过：外部已设 NEOBOT_DATA_DIR");
            return;
        }
        let d = data_dir().expect("解析");
        assert!(d.ends_with(".neobot"), "实际：{d:?}");
    }

    // ════════════════════════════════════════════════════════════════
    // ⭐⭐ 分页命令的**纯逻辑**测试（2026-10-03）
    // ⭐ 刻意**不**调 `open_store()`（它走全局 HOME）⇒ 测纯函数化后的分页整形。
    // ════════════════════════════════════════════════════════════════

    fn msg(seq: i64) -> neotrix_neobot::nt_store::ChatMessage {
        neotrix_neobot::nt_store::ChatMessage {
            seq,
            id: format!("m{seq}"),
            convo_id: "c1".into(),
            role: "assistant".into(),
            text: format!("body {seq}"),
            created_at: "2026-10-03T00:00:00Z".into(),
        }
    }

    /// ⭐⭐ `has_more` 探测正确：多取 1 条 ⇒ 有余量即 true
    #[test]
    fn 分页探测出还有更多() {
        let p = shape_page((1..=6).map(msg).collect(), 5);
        assert!(p.has_more, "6 条取 5 条 ⇒ 必须报告还有更多");
        assert_eq!(p.messages.len(), 5, "实际返回条数必须等于 limit");
    }

    /// ⭐⭐ 最后一页：`has_more=false` 且 ⭐ **`next_seq` 必须为 None**
    /// （否则界面会拿着陈旧游标反复请求同一页 ⇒ 死循环）
    #[test]
    fn 最后一页无更多且游标为空() {
        let p = shape_page((1..=3).map(msg).collect(), 5);
        assert!(!p.has_more, "3 条取 5 条 ⇒ 没有更多");
        assert_eq!(p.next_seq, None, "无更多时不得回传游标，否则界面会死循环");
    }

    /// ⭐⭐ 游标 = 本页**最小** seq（正序 ⇒ 最后一条），⭐ 不是最大
    /// —— 用错方向会让「加载更早」变成「加载更晚」，静默错误。
    #[test]
    fn 游标取本页最小seq() {
        let p = shape_page((1..=6).map(msg).collect(), 5);
        assert_eq!(p.next_seq, Some(5), "本页是 seq 1..5 ⇒ 游标应是 5（最小值）");
    }
}
