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
}
