//! `nt_store_run_trace` — 轨迹读口（`tasks` 列表 + 每轮步数聚合）。
//!
//! # 为什么 SQL 在这里而不在 `nt_run_trace`
//!
//! `NeobotStore.conn` 是**模块私有**字段（`nt_store/mod.rs:208`）——
//! 本仓的纪律就是「SQL 只出现在 `nt_store/` 里，域模块只见读口」。
//! 域侧 [`crate::nt_run_trace`] 只见 [`RunStoreRow`]，不见 SQL。
//!
//! # 为什么自己读而不复用 `list_convo_tasks`
//!
//! 那两个读口对**认不出的 `status` 是 `continue` 静默丢行**
//! （`nt_types.rs:117` 自己把这个形态记成「比报错更难查」）。
//! 轨迹页丢一行 = 一轮真实跑过的事**从界面上消失且无解释**。
//! ⇒ 这里按 6 列 + 2 个聚合计数自取，**状态原样透传**，
//! 宁可界面显示一个它不认识的词。

use rusqlite::{params, OptionalExtension};

use super::NeobotStore;
use crate::nt_error::NtBotError;

/// `tasks` 的一行 + 该轮的步数聚合（读口原样；**状态不解析**）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunStoreRow {
    pub id: String,
    pub title: String,
    /// `tasks.status` **原样**（可能是 `TaskStatus::parse` 认不出的新值）。
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub error: Option<String>,
    /// `steps` 行数（工具步 + `reply` 步都在内）。
    pub steps: i64,
    /// 其中 `ok = 0` 的行数（**失败步数**，不是「整轮失败」）。
    pub failed_steps: i64,
}

/// 主表 8 列：6 个字段 + 2 个相关子查询。
///
/// ⚠️ 步数用**相关子查询**而非 `LEFT JOIN + GROUP BY`：后者在
/// 「任务存在但零 step」时会把任务行本身聚成 NULL 组而丢失，
/// 而那正是「刚建还没跑」的真实状态（`pending` 轮次必须出现在轨迹页）。
const RUN_COLS: &str = "t.id,t.title,t.status,t.created_at,t.updated_at,t.error,\
     (SELECT COUNT(*) FROM steps s WHERE s.task_id=t.id),\
     (SELECT COUNT(*) FROM steps s WHERE s.task_id=t.id AND s.ok=0)";

fn row_to_run(r: &rusqlite::Row<'_>) -> rusqlite::Result<RunStoreRow> {
    Ok(RunStoreRow {
        id: r.get(0)?,
        title: r.get(1)?,
        status: r.get(2)?,
        created_at: r.get(3)?,
        updated_at: r.get(4)?,
        error: r.get(5)?,
        steps: r.get(6)?,
        failed_steps: r.get(7)?,
    })
}

impl NeobotStore {
    /// 列轨迹行（`convo_id` 缺席 = 全部会话；`created_at` 倒序，
    /// 同秒用 `rowid` 兜底 —— `created_at` 是秒级精度，同秒并列很常见，
    /// 只按它排序会让「哪一轮在后」变得不确定）。
    pub fn list_run_rows(
        &self,
        convo_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<RunStoreRow>, NtBotError> {
        let capped = limit.max(1);
        let mut stmt = match convo_id {
            Some(_) => self.conn.prepare(&format!(
                "SELECT {RUN_COLS} FROM tasks t WHERE t.conversation_id=?1 \
                 ORDER BY t.created_at DESC, t.rowid DESC LIMIT ?2"
            ))?,
            None => self.conn.prepare(&format!(
                "SELECT {RUN_COLS} FROM tasks t \
                 ORDER BY t.created_at DESC, t.rowid DESC LIMIT ?1"
            ))?,
        };
        let rows = match convo_id {
            Some(convo) => stmt.query_map(params![convo, capped], row_to_run)?,
            None => stmt.query_map(params![capped], row_to_run)?,
        };
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// 取单轮轨迹行（`None` = 这一轮不存在）。
    ///
    /// ⛔ 与 `get_task`（`nt_store_routines.rs:245`）**故意不同**：
    ///   那个遇到认不出的状态返回 `Err`，这个**原样透传**。
    ///   域层据此区分「不存在」与「存在但状态是新的」，两者都不是「空轨迹页」。
    pub fn get_run_row(&self, task_id: &str) -> Result<Option<RunStoreRow>, NtBotError> {
        let mut stmt = self
            .conn
            .prepare(&format!("SELECT {RUN_COLS} FROM tasks t WHERE t.id=?1"))?;
        // `optional()` 把「无此行」变成 `None`（而不是 `QueryReturnedNoRows` 错误）——
        // 域层据此区分「这一轮不存在」与「读失败」，见 `nt_run_trace::run_trace`。
        Ok(stmt
            .query_row(params![task_id], row_to_run)
            .optional()?)
    }
}

#[cfg(test)]
mod tests {
    use crate::nt_types::{AgentTask, TaskStatus};
    use crate::nt_store::NeobotStore;

    fn seed(store: &NeobotStore, id: &str, convo: Option<&str>, created: &str) {
        store
            .save_task(&AgentTask {
                id: id.to_owned(),
                title: format!("轮次 {id}"),
                status: TaskStatus::Done,
                created_at: created.to_owned(),
                updated_at: created.to_owned(),
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
    fn unknown_status_is_kept_not_dropped() {
        let store = NeobotStore::open(":memory:").expect("open");
        seed(&store, "a", Some("c1"), "2026-10-07T00:00:00Z");
        // ⛔ 直接写库绕过 `TaskStatus`：模拟「新加状态忘了同步 parse」。
        //    此时 `list_convo_tasks` 会静默丢行（nt_types.rs:117 记过这个坑），
        //    而本读口必须**原样透传**。
        //    `conn` 是 `nt_store` 的私有字段，但本测试住在它的子模块里 ⇒ 可访问。
        store
            .conn
            .execute("UPDATE tasks SET status='quantum' WHERE id='a'", [])
            .expect("poke status");
        let got = store.list_run_rows(Some("c1"), 10).expect("list");
        assert_eq!(got.len(), 1, "认不出的状态不得让轮次从列表消失");
        assert_eq!(got[0].status, "quantum");
        // 单轮读口同样透传（不像 get_task 那样 Err）。
        assert_eq!(
            store.get_run_row("a").expect("get").expect("some").status,
            "quantum"
        );
    }

    #[test]
    fn zero_step_task_is_still_listed() {
        // ⛔ 这行正是「相关子查询而非 LEFT JOIN+GROUP BY」保住的：
        //    「刚建还没跑」的任务必须出现（status=pending）。
        let store = NeobotStore::open(":memory:").expect("open");
        seed(&store, "a", Some("c1"), "2026-10-07T00:00:00Z");
        let got = store.list_run_rows(Some("c1"), 10).expect("list");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].steps, 0);
        assert_eq!(got[0].failed_steps, 0);
    }

    #[test]
    fn scope_counts_and_ordering() {
        let store = NeobotStore::open(":memory:").expect("open");
        seed(&store, "old", Some("c1"), "2026-10-05T00:00:00Z");
        seed(&store, "new", Some("c1"), "2026-10-07T00:00:00Z");
        seed(&store, "other", Some("c2"), "2026-10-06T00:00:00Z");
        store.add_step("new", 0, "bash", true, "ok").expect("step");
        store.add_step("new", 1, "bash", false, "boom").expect("step");

        let c1 = store.list_run_rows(Some("c1"), 10).expect("list");
        assert_eq!(
            c1.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
            vec!["new", "old"],
            "c1 内按 created_at 倒序，且 c2 的轮次不得混入"
        );
        assert_eq!(c1[0].steps, 2);
        assert_eq!(c1[0].failed_steps, 1, "失败步数 ≠ 失败轮数");

        assert_eq!(store.list_run_rows(None, 10).expect("list").len(), 3);
        assert_eq!(
            store.list_run_rows(Some("ghost"), 10).expect("list").len(),
            0,
            "范围不存在 ⇒ 空列表（不是 Err，那由域层区分读失败）"
        );
        assert!(store.get_run_row("ghost").expect("get").is_none());
    }

    #[test]
    fn limit_is_honored() {
        let store = NeobotStore::open(":memory:").expect("open");
        for i in 0..5 {
            seed(&store, &format!("t{i}"), Some("c1"), "2026-10-07T00:00:00Z");
        }
        assert_eq!(store.list_run_rows(Some("c1"), 2).expect("list").len(), 2);
        // 非正数钳到 1（不让 `LIMIT 0` 被当成「全部」的相反面 —— 那会读空）。
        assert_eq!(store.list_run_rows(Some("c1"), 0).expect("list").len(), 1);
    }
}
