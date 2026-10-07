//! `nt_run_trace` — 「跑过什么」的可观测视图（**轨迹页的唯一数据源**）。
//!
//! # 为什么有这个模块
//!
//! 跑轮把执行痕迹写进了 `tasks` / `steps` / `file_changes` 三张表，
//! 而**没有任何读口把它们端到端取出来**。后果不是「暂时没做」，
//! 是**数据在库里、接口不存在**：界面看不见「这一轮用了哪些工具、
//! 第几步失败、改了哪些文件」，用户只能对着一个「已完成」猜它干了什么。
//!
//! ⛔ `neobot_send` 的返回里确实带 `AgentRunResult.trace`
//! （`nt_core::TraceRow{kind,detail}`），但那是**服务端这一次**的摘要，
//! **既不落库、也不覆盖历史轮** —— 只看它等于「只看得见最后一眼」。
//! 两者**互补**：本模块负责历史，trace 负责当次。
//!
//! # 三条不变量（全部来自本仓既有纪律，不是新发明的）
//!
//! 1. **未知状态不丢行、也不猜。** `TaskStatus::parse` 认不出的状态，
//!    既有 `list_tasks` / `list_convo_tasks` 是 `continue` **静默丢行**
//!    （`nt_types.rs:117` 自己把这个形态记成「比报错更难查」）。
//!    ⇒ 本模块的读口按 6 列自取（[`crate::nt_store::nt_store_run_trace`]）
//!    并把状态串**原样透传**。宁可界面显示一个它不认识的词，
//!    也不要让一轮跑过的事**从列表里消失**。
//! 2. **不把用量摊到某一轮。** `ledger` 表按 `(at, actor, purpose)` 记，
//!    **不以 `task_id` 为键**（见 `nt_store/mod.rs` 建表语句）。
//!    按时间窗把账摊到某一轮上是**猜** ⇒ 本模块**不产出每轮费用/耗时**。
//!    界面要数字就走 `neobot_usage_summary`（`usage.json` 的按天账），
//!    两边各有各的口径，不混。
//! 3. **`tool_call_id` 的 `None` 与空串保持可区分**（沿用 `StepRow` 的
//!    纪律：`""` 是**非法**调用 id，`NULL` 是合法的存量状态，
//!    折成空串就是把「没键」说成「键是空的」）。
//!
//! # 分层
//!
//! 本模块**只做 DTO 映射**（store 行 → 契约形状）。SQL 在
//! `nt_store/nt_store_run_trace.rs`（`conn` 是 `nt_store` 私有字段）。

use serde::{Deserialize, Serialize};

use crate::nt_error::NtBotError;
use crate::nt_store::{FileChange, NeobotStore, RunStoreRow};

/// 列表条数缺省。⛔ **有界** —— 「全部」在长会话上是把整段历史
/// 一次塞进界面，与 `neobot_convo_messages_page` 同一条纪律。
pub const RUN_LIST_DEFAULT_LIMIT: i64 = 50;
/// 列表条数上限（钳位）。
pub const RUN_LIST_MAX_LIMIT: i64 = 200;
/// 单轮文件改动行数上限（钳位）。前后内容**不在本视图内**
/// （重物，按需走 `get_change` 单独取，见 `nt_store::FileChangeView`）。
pub const RUN_CHANGES_LIMIT: i64 = 200;

/// 一轮（= 一个 `tasks` 行）的轨迹行。
///
/// ⛔ **刻意不是 `AgentTask`**：后者带 `lease_id` / `lease_until` /
/// `claimed_by` 等**运行租约内部字段** —— 那是崩溃恢复凭据，
/// 端到端给界面等于把内部协议与凭据面一起暴露。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunRow {
    pub id: String,
    pub title: String,
    /// `tasks.status` **原样**（不解析、不丢行；见模块头不变量 1）。
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    /// `tasks.error`（认领/租约/终态原因；`None` = 没记原因）。
    ///
    /// ⛔ **不要**用 `Option<String>` 的缺席推导「成功」：没记原因与
    /// 没失败是两句话。
    pub error: Option<String>,
    /// `steps` 行数（工具步 + `reply` 步都在内 —— `reply` 也是一步）。
    pub steps: i64,
    /// 其中 `ok = 0` 的行数（**失败步数**，不是「整轮失败」）。
    pub failed_steps: i64,
}

/// 轨迹列表（按 `created_at` 倒序）。
///
/// ⛔ 契约形状即语义：**读失败是 `Err`，空是 `Ok` 且 `runs` 为空**。
/// 两者对用户是两种处境，混起来会让人白折腾（本仓反复吃过的坑）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunListView {
    pub runs: Vec<RunRow>,
}

/// 轨迹页的一步（`steps` 一行的契约形状）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepView {
    /// `steps.id`（`AUTOINCREMENT` 行号；**写入序**的判据）。
    pub id: i64,
    /// 第几跳（`0` 起）。
    pub n: i64,
    /// 工具名 / `reply` / `side-effect:*` / `cancelled:tool_calls`。
    pub tool: String,
    pub ok: bool,
    pub output: String,
    /// ⛔ 保留 `None` 与 `""` 的区别（模块头不变量 3）；序列化后
    ///   `None` 是 `null`、`Some("")` 是 `""`，界面据此分别显示。
    pub tool_call_id: Option<String>,
}

/// 文件改动一行（`file_changes` 的契约形状；**不含**前后内容）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeView {
    pub id: String,
    pub at: String,
    /// 相对工作区根的路径（工作区整体搬走后历史账仍读得懂）。
    pub path: String,
    /// `read` | `write` | `edit`（见 `nt_changes::KIND_*`）。
    pub kind: String,
    pub bytes: i64,
    pub content_omitted: bool,
}

/// 一轮的完整轨迹：轮次行 + 每一步 + 改了哪些文件。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunTraceView {
    pub run: RunRow,
    pub steps: Vec<StepView>,
    pub changes: Vec<ChangeView>,
}

impl From<RunStoreRow> for RunRow {
    fn from(r: RunStoreRow) -> Self {
        Self {
            id: r.id,
            title: r.title,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
            error: r.error,
            steps: r.steps,
            failed_steps: r.failed_steps,
        }
    }
}

impl From<&FileChange> for ChangeView {
    fn from(c: &FileChange) -> Self {
        Self {
            id: c.id.clone(),
            at: c.at.clone(),
            path: c.path.clone(),
            kind: c.kind.clone(),
            bytes: c.bytes,
            content_omitted: c.content_omitted,
        }
    }
}

/// 钳 `limit`：非正数 → 缺省；超上限 → 上限。
fn clamp_limit(limit: i64, cap: i64) -> i64 {
    if limit <= 0 {
        return RUN_LIST_DEFAULT_LIMIT.min(cap);
    }
    limit.min(cap)
}

/// 列轨迹（`convo_id` 缺席 = 全部会话）。
///
/// `Err` = **读不到**（打不开库 / SQL 失败）；`Ok` 里 `runs` 为空 =
/// **真的没有**。界面上这两种必须说不同的话。
pub fn run_list(
    store: &NeobotStore,
    convo_id: Option<&str>,
    limit: i64,
) -> Result<RunListView, NtBotError> {
    let capped = clamp_limit(limit, RUN_LIST_MAX_LIMIT);
    let rows = store
        .list_run_rows(convo_id, capped)
        .map_err(|e| NtBotError::Store(format!("read run list: {e}")))?;
    Ok(RunListView {
        runs: rows.into_iter().map(RunRow::from).collect(),
    })
}

/// 取一轮的完整轨迹。
///
/// ⛔ 「这一轮不存在」是 `Err`，不是「空轨迹页」—— 空页同时意味着
/// 「存在但没记录」，而那两件事对用户是两种处境（后者读口坏了）。
pub fn run_trace(store: &NeobotStore, task_id: &str) -> Result<RunTraceView, NtBotError> {
    let id = task_id.trim();
    if id.is_empty() {
        return Err(NtBotError::Invalid("run id is empty".to_owned()));
    }
    let run = store
        .get_run_row(id)
        .map_err(|e| NtBotError::Store(format!("read run {id}: {e}")))?
        .ok_or_else(|| NtBotError::Invalid(format!("run not found: {id}")))?;
    let steps = store
        .list_steps(id)
        .map_err(|e| NtBotError::Store(format!("read steps of {id}: {e}")))?
        .into_iter()
        .map(|s| StepView {
            id: s.id,
            n: s.n,
            tool: s.tool,
            ok: s.ok,
            output: s.output,
            tool_call_id: s.tool_call_id,
        })
        .collect();
    let changes = store
        .list_changes(Some(id), RUN_CHANGES_LIMIT)
        .map_err(|e| NtBotError::Store(format!("read changes of {id}: {e}")))?
        .iter()
        .map(ChangeView::from)
        .collect();
    Ok(RunTraceView {
        run: RunRow::from(run),
        steps,
        changes,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        clamp_limit, run_list, run_trace, ChangeView, RunListView, StepView, RUN_LIST_DEFAULT_LIMIT,
        RUN_LIST_MAX_LIMIT,
    };
    use crate::nt_changes::KIND_EDIT;
    use crate::nt_store::{FileChange, NeobotStore};
    use crate::nt_types::{AgentTask, TaskStatus};

    fn seed(store: &NeobotStore, id: &str, convo: Option<&str>) {
        store
            .save_task(&AgentTask {
                id: id.to_owned(),
                title: format!("轮次 {id}"),
                status: TaskStatus::Done,
                created_at: "2026-10-07T00:00:00Z".to_owned(),
                updated_at: "2026-10-07T00:00:10Z".to_owned(),
                claimed_by: None,
                conversation_id: convo.map(str::to_owned),
                claimed_at: None,
                visibility: "team".to_owned(),
                lease_id: None,
                lease_until: None,
                attempts: 0,
                error: None,
            })
            .expect("save task");
    }

    #[test]
    fn empty_scope_is_ok_empty_not_err() {
        let store = NeobotStore::open(":memory:").expect("open");
        // 读得到、且真的没有 ⇒ Ok + 空。⛔ 不是 Err（那会被界面说成「读不到」）。
        assert_eq!(
            run_list(&store, Some("nope"), 10).expect("list"),
            RunListView { runs: Vec::new() }
        );
    }

    #[test]
    fn run_row_never_leaks_lease_internals() {
        // 契约面只暴露 8 个字段；`lease_id`/`lease_until`/`claimed_by`
        // 是崩溃恢复凭据，不得出现在给界面的 DTO 里。
        let row = super::RunRow {
            id: "a".to_owned(),
            title: "t".to_owned(),
            status: "done".to_owned(),
            created_at: "x".to_owned(),
            updated_at: "y".to_owned(),
            error: None,
            steps: 0,
            failed_steps: 0,
        };
        let json = serde_json::to_string(&row).expect("serialize");
        for banned in ["lease", "claimed", "attempts", "visibility"] {
            assert!(!json.contains(banned), "租约内部字段泄漏进契约：{json}");
        }
    }

    #[test]
    fn trace_joins_steps_and_changes_and_keeps_null_call_id() {
        let store = NeobotStore::open(":memory:").expect("open");
        seed(&store, "a", Some("c1"));
        store
            .add_step_with_call("a", 0, "bash", true, "ok", Some("c-1"))
            .expect("step");
        store.add_step("a", 1, "reply", true, "hi").expect("step");
        store
            .record_change(&FileChange {
                id: "ch1".to_owned(),
                task_id: "a".to_owned(),
                at: "2026-10-07T00:00:05Z".to_owned(),
                path: "src/x.rs".to_owned(),
                kind: KIND_EDIT.to_owned(),
                bytes: 12,
                content_omitted: false,
            },
            None,
            None,
        )
            .expect("change");

        let got = run_trace(&store, "a").expect("trace");
        assert_eq!(got.steps.len(), 2);
        assert_eq!(got.steps[0].tool_call_id.as_deref(), Some("c-1"));
        // ⛔ 第二行没有配平键 ⇒ 必须是 None（不是 Some("")）。
        assert_eq!(got.steps[1].tool_call_id, None);
        assert_eq!(
            got.changes,
            vec![ChangeView {
                id: "ch1".to_owned(),
                at: "2026-10-07T00:00:05Z".to_owned(),
                path: "src/x.rs".to_owned(),
                kind: "edit".to_owned(),
                bytes: 12,
                content_omitted: false,
            }]
        );
        // 列表与详情说的是同一轮（两个读口的口径不许分叉）。
        let listed = run_list(&store, Some("c1"), 10).expect("list").runs;
        assert_eq!(listed, vec![got.run]);
    }

    #[test]
    fn trace_of_a_fresh_run_is_empty_but_not_missing() {
        // 「存在但零步」≠「不存在」：前者要渲染出空轨迹，后者要报错。
        let store = NeobotStore::open(":memory:").expect("open");
        seed(&store, "a", Some("c1"));
        let got = run_trace(&store, "a").expect("trace");
        assert!(got.steps.is_empty() && got.changes.is_empty());
        assert_eq!(got.run.steps, 0);
    }

    #[test]
    fn missing_run_is_an_error_not_an_empty_trace() {
        let store = NeobotStore::open(":memory:").expect("open");
        let err = run_trace(&store, "ghost").expect_err("must refuse");
        assert!(err.to_string().contains("run not found"), "{err}");
        // 空 id 是**请求不成立**，不是「没查到」。
        assert!(run_trace(&store, "  ").is_err());
    }

    #[test]
    fn trace_does_not_invent_per_run_cost() {
        // 不变量 2 的守卫：`RunRow` 里**没有** cost/tokens/latency 字段。
        // 若日后有人「顺手加上」，本测试即红 —— 那正是要重新论证
        // 「账能不能归到轮次」的时刻（ledger 不以 task_id 为键）。
        let store = NeobotStore::open(":memory:").expect("open");
        seed(&store, "a", Some("c1"));
        let json = serde_json::to_string(&run_list(&store, Some("c1"), 10).expect("list"))
            .expect("serialize");
        for banned in ["cost", "token", "usd", "latency"] {
            assert!(!json.contains(banned), "轨迹行不得自带用量字段：{json}");
        }
    }

    #[test]
    fn limit_is_clamped_and_bounded() {
        assert_eq!(clamp_limit(0, RUN_LIST_MAX_LIMIT), RUN_LIST_DEFAULT_LIMIT);
        assert_eq!(clamp_limit(-5, RUN_LIST_MAX_LIMIT), RUN_LIST_DEFAULT_LIMIT);
        assert_eq!(clamp_limit(7, RUN_LIST_MAX_LIMIT), 7);
        assert_eq!(
            clamp_limit(9999, RUN_LIST_MAX_LIMIT),
            RUN_LIST_MAX_LIMIT,
            "超限必须钳住（界面一次要一万行不是「有界」）"
        );
    }

    #[test]
    fn view_types_are_serializable_contract_dtos() {
        // 契约门读的是**字段名**；这里钉住「步」这层的形状，
        // 免得日后有人加字段时界面静默少渲染。
        let step = StepView {
            id: 1,
            n: 0,
            tool: "bash".to_owned(),
            ok: true,
            output: "ok".to_owned(),
            tool_call_id: None,
        };
        let v = serde_json::to_value(&step).expect("serialize");
        assert_eq!(v["tool_call_id"], serde_json::Value::Null);
        assert!(v.get("id").is_some() && v.get("n").is_some() && v.get("ok").is_some());
    }
}
