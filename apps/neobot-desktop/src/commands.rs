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
        Answer, AnswerOutcome, Mode, Panel, PanelKind, PanelOption, PublishError, Reject, Source,
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
}
