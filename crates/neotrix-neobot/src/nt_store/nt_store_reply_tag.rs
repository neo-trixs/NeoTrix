//! `nt_store_reply_tag` — 回复标签的 store 读口（`steps` 工具列按写入序）。
//!
//! 新文件：`nt_agent` 跑轮写 `steps`，跑轮命令经此读回本轮工具名，
//! 再由 `nt_reply_tag::normalize_tools` 过滤去重；无行回空（纯回复轮）。

use super::NeobotStore;
use crate::nt_error::NtBotError;

impl NeobotStore {
    /// 本轮步骤工具名（按写入序；调用方经 `normalize_tools` 过滤）。
    /// 失败由调用方按空工具降级，不挡主流程。
    pub fn list_step_tools(&self, task_id: &str) -> Result<Vec<String>, NtBotError> {
        let mut stmt = self
            .conn
            .prepare("SELECT tool FROM steps WHERE task_id=?1 ORDER BY id")?;
        let rows = stmt.query_map(rusqlite::params![task_id], |r| {
            r.get::<_, String>(0)
        })?;
        let mut out = Vec::new();
        for row in rows {
            match row {
                Ok(tool) => out.push(tool),
                Err(_) => continue,
            }
        }
        Ok(out)
    }

    /// 某任务**最后一条**指定工具的输出（`ORDER BY id DESC LIMIT 1`）。
    ///
    /// 侧聊继承摘要用它取「末次助手回复」（`nt_side_chat`）。
    /// 没有匹配的步骤行即 `None` —— 那就诚实地没有回复，不编。
    pub fn last_step_output(
        &self,
        task_id: &str,
        tool: &str,
    ) -> Result<Option<String>, NtBotError> {
        Ok(self
            .conn
            .query_row(
                "SELECT output FROM steps WHERE task_id=?1 AND tool=?2 ORDER BY id DESC LIMIT 1",
                rusqlite::params![task_id, tool],
                |r| r.get::<_, String>(0),
            )
            .ok())
    }
}

#[cfg(test)]
mod tests {
    use crate::nt_store::NeobotStore;

    #[test]
    fn step_tools_roundtrip_in_write_order() {
        let store = NeobotStore::open(":memory:").expect("open");
        store.add_step("t1", 0, "bash", true, "ok").expect("step");
        store.add_step("t1", 1, "reply", true, "hi").expect("step");
        let got = store.list_step_tools("t1").expect("list");
        assert_eq!(got, vec!["bash".to_owned(), "reply".to_owned()]);
        // 未知任务回空，不炸。
        assert!(store.list_step_tools("missing").expect("list").is_empty());
    }

    #[test]
    fn last_step_output_picks_the_newest_match() {
        let store = NeobotStore::open(":memory:").expect("open");
        store.add_step("t1", 0, "reply", true, "first").expect("step");
        store.add_step("t1", 1, "bash", true, "cmd").expect("step");
        store.add_step("t1", 2, "reply", true, "second").expect("step");
        assert_eq!(
            store.last_step_output("t1", "reply").expect("out").as_deref(),
            Some("second")
        );
        // 没有该工具的行 → None（不是空串，避免和「空回复」混淆）。
        assert!(store.last_step_output("t1", "web_search").expect("out").is_none());
        // 未知任务同样 None。
        assert!(store.last_step_output("nope", "reply").expect("out").is_none());
    }
}
