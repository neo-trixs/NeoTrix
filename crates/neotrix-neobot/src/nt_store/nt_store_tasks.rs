//! `nt_store_tasks` — 任务 insert/status-CAS/认领租约/租约回收（`NeobotStore` 的 impl 片段）。

use super::NeobotStore;
use crate::nt_error::NtBotError;
use crate::nt_types::AgentTask;
use rusqlite::params;

/// RFC3339 回退 N 秒（解析失败回原串，保证调用方永不崩）。
fn rfc3339_minus_secs(now: &str, secs: i64) -> String {
    let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(now) else {
        return now.to_owned();
    };
    (parsed - chrono::Duration::seconds(secs.max(0))).to_rfc3339()
}

impl NeobotStore {
    // ---- tasks (durable TaskWorker 简化: insert + status CAS) ----

    pub fn save_task(&self, task: &AgentTask) -> Result<(), NtBotError> {
        self.conn.execute(
            "INSERT INTO tasks(id,title,status,created_at,updated_at,claimed_by,claimed_at,
               visibility,lease_id,lease_until,attempts,error,conversation_id)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
             ON CONFLICT(id) DO UPDATE SET title=excluded.title,
               status=excluded.status, updated_at=excluded.updated_at,
               claimed_by=excluded.claimed_by, claimed_at=excluded.claimed_at,
               visibility=excluded.visibility, lease_id=excluded.lease_id,
               lease_until=excluded.lease_until, attempts=excluded.attempts,
               error=excluded.error, conversation_id=excluded.conversation_id",
            params![
                task.id,
                task.title,
                task.status.as_str(),
                task.created_at,
                task.updated_at,
                task.claimed_by,
                task.claimed_at,
                task.visibility,
                task.lease_id,
                task.lease_until,
                task.attempts,
                task.error,
                task.conversation_id
            ],
        )?;
        Ok(())
    }

    /// 原子认领：仅未认领可认领；已被认领返回 holder（调用方展示“XX 认领中”）。
    /// 认领带时刻（认领租约）：`sweep_stale_claims` 可扫回过期认领。
    pub fn claim_task(&self, id: &str, actor: &str) -> Result<(), NtBotError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.claim_task_at(id, actor, &now)
    }

    pub fn claim_task_at(&self, id: &str, actor: &str, now: &str) -> Result<(), NtBotError> {
        let n = self.conn.execute(
            "UPDATE tasks SET claimed_by=?1, claimed_at=?2, updated_at=?2
             WHERE id=?3 AND claimed_by IS NULL",
            params![actor, now, id],
        )?;
        if n == 0 {
            let holder: Option<String> = self
                .conn
                .query_row("SELECT claimed_by FROM tasks WHERE id=?1", params![id], |r| {
                    r.get(0)
                })
                .unwrap_or(None);
            return Err(NtBotError::Store(format!(
                "task {id} already claimed by {}",
                holder.as_deref().unwrap_or("?")
            )));
        }
        Ok(())
    }

    /// 释放：holder 本人或空认领可放；他人持有拒绝。
    pub fn release_task(&self, id: &str, actor: &str) -> Result<(), NtBotError> {
        let now = chrono::Utc::now().to_rfc3339();
        let n = self.conn.execute(
            "UPDATE tasks SET claimed_by=NULL, claimed_at=NULL, updated_at=?1
             WHERE id=?2 AND (claimed_by IS NULL OR claimed_by=?3)",
            params![now, id, actor],
        )?;
        if n == 0 {
            return Err(NtBotError::Store(format!("task {id} held by another actor")));
        }
        Ok(())
    }

    /// 重命名（1..=160 字，空拒）。
    pub fn rename_task(&self, id: &str, title: &str) -> Result<(), NtBotError> {
        let title = title.trim();
        if title.is_empty() {
            return Err(NtBotError::Invalid("task title is empty".to_owned()));
        }
        if title.chars().count() > 160 {
            return Err(NtBotError::Invalid("task title exceeds 160 chars".to_owned()));
        }
        let now = chrono::Utc::now().to_rfc3339();
        let n = self.conn.execute(
            "UPDATE tasks SET title=?1, updated_at=?2 WHERE id=?3",
            params![title, now, id],
        )?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such task '{id}'")));
        }
        Ok(())
    }

    /// 删除（含 steps 行；不存在拒）。
    pub fn delete_task(&self, id: &str) -> Result<(), NtBotError> {
        self.conn
            .execute("DELETE FROM steps WHERE task_id=?1", params![id])?;
        let n = self
            .conn
            .execute("DELETE FROM tasks WHERE id=?1", params![id])?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such task '{id}'")));
        }
        Ok(())
    }

    /// 取消运行中/排队任务（仅 pending/running 可取消，顺带清租约）。
    pub fn cancel_task(&self, id: &str) -> Result<(), NtBotError> {
        let now = chrono::Utc::now().to_rfc3339();
        let n = self.conn.execute(
            "UPDATE tasks SET status='cancelled', lease_id=NULL, lease_until=NULL,
              updated_at=?1 WHERE id=?2 AND status IN ('pending','running')",
            params![now, id],
        )?;
        if n == 0 {
            return Err(NtBotError::Store(format!("task {id} is not cancellable")));
        }
        Ok(())
    }

    /// 失败/取消任务重跑（回 pending，保留 attempts 计数）。
    pub fn retry_task(&self, id: &str) -> Result<(), NtBotError> {
        let now = chrono::Utc::now().to_rfc3339();
        let n = self.conn.execute(
            "UPDATE tasks SET status='pending', error=NULL, updated_at=?1
             WHERE id=?2 AND status IN ('failed','cancelled')",
            params![now, id],
        )?;
        if n == 0 {
            return Err(NtBotError::Store(format!("task {id} is not retryable")));
        }
        Ok(())
    }

    /// 崩溃恢复（租约过期回收）：
    /// running 且租约过期（或无租约）→ pending + error 注记，返回恢复条数。
    /// 调用方在每次 run 起点调一次即可（单机，代价一次 UPDATE）。
    pub fn recover_stale_running(&self, now: &str) -> Result<usize, NtBotError> {
        let n = self.conn.execute(
            "UPDATE tasks SET status='pending', lease_id=NULL, lease_until=NULL,
              error='lease expired (crash recovery)', updated_at=?1
             WHERE status='running' AND (lease_until IS NULL OR lease_until < ?1)",
            params![now],
        )?;
        Ok(n)
    }

    /// 扫回过期认领（认领超 TTL 自动过期，默认见 `CLAIM_TTL_SECS`）。
    /// 返回释放条数。
    pub fn sweep_stale_claims(&self, now: &str, ttl_secs: i64) -> Result<usize, NtBotError> {
        let cutoff = rfc3339_minus_secs(now, ttl_secs);
        let n = self.conn.execute(
            "UPDATE tasks SET claimed_by=NULL, claimed_at=NULL, updated_at=?1
             WHERE claimed_by IS NOT NULL AND claimed_at IS NOT NULL AND claimed_at < ?2",
            params![now, cutoff],
        )?;
        Ok(n)
    }

    /// 可见性切换（team/private；默认 team）。
    pub fn set_task_visibility(&self, id: &str, visibility: &str) -> Result<(), NtBotError> {
        if visibility != "team" && visibility != "private" {
            return Err(NtBotError::Store(format!("bad visibility '{visibility}'")));
        }
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE tasks SET visibility=?1, updated_at=?2 WHERE id=?3",
            params![visibility, now, id],
        )?;
        Ok(())
    }

    /// 带 owner 门的可见性切换：转 private 必须是 owner（单管理员座位；
    /// 无成员/无 owner 配置时放行，保证冷启动可用）。
    pub fn set_task_visibility_as(
        &self,
        id: &str,
        visibility: &str,
        actor: &str,
    ) -> Result<(), NtBotError> {
        if visibility == "private" {
            let owner = self.owner_name()?;
            if let Some(name) = owner {
                if name != actor {
                    return Err(NtBotError::Store(format!(
                        "private visibility requires owner '{name}'"
                    )));
                }
            }
        }
        self.set_task_visibility(id, visibility)
    }
}

#[cfg(test)]
mod tests {
    use crate::nt_store::NeobotStore;
    use crate::nt_types::{AgentTask, TaskStatus};

    #[test]
    fn task_roundtrip_and_outbox() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        let task = AgentTask {
            id: "t1".to_owned(),
            title: "demo".to_owned(),
            status: TaskStatus::Running,
            created_at: "2026-09-23T00:00:00Z".to_owned(),
            updated_at: "2026-09-23T00:00:00Z".to_owned(),
            claimed_by: None,
            claimed_at: None,
            visibility: crate::nt_types::default_visibility(),
            lease_id: None,
            lease_until: None,
            attempts: 0,
            error: None,
            conversation_id: None,
        };
        store.save_task(&task).expect("save");
        let got = store.get_task("t1").expect("get").expect("exists");
        assert_eq!(got.title, "demo");
        assert_eq!(got.visibility, "team");
        // 原子认领：首认领成功，重认领报 holder
        store.claim_task("t1", "alice").expect("claim");
        let err = store.claim_task("t1", "bob").expect_err("re-claim must fail");
        assert!(err.to_string().contains("alice"), "holder named: {err}");
        // 非 holder 释放被拒，holder 释放成功
        assert!(store.release_task("t1", "bob").is_err());
        store.release_task("t1", "alice").expect("release");
        store.claim_task("t1", "bob").expect("claim after release");
        store
            .set_task_visibility("t1", "private")
            .expect("visibility");
        assert!(store.set_task_visibility("t1", "nope").is_err());
        store.enqueue_outbox("o1", "CH_MESSAGE_NEW", "{}").expect("enqueue");
        let now = "2026-09-24T00:00:00Z";
        let drained = store.drain_outbox(10, now).expect("drain");
        assert_eq!(drained.len(), 1);
        assert!(store.drain_outbox(10, now).expect("drain2").is_empty());
    }


    #[test]
    fn lease_recovery_and_cancel_retry() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        let mk = |id: &str| AgentTask {
            id: id.to_owned(),
            title: id.to_owned(),
            status: TaskStatus::Running,
            created_at: "2026-09-24T00:00:00Z".to_owned(),
            updated_at: "2026-09-24T00:00:00Z".to_owned(),
            claimed_by: None,
            claimed_at: None,
            visibility: crate::nt_types::default_visibility(),
            lease_id: Some("old-lease".to_owned()),
            lease_until: Some("2026-09-24T00:00:00Z".to_owned()),
            attempts: 1,
            error: None,
            conversation_id: None,
        };
        store.save_task(&mk("a")).expect("save");
        // 租约过期 → 恢复为 pending；未来租约不动
        let recovered = store
            .recover_stale_running("2026-09-24T01:00:00Z")
            .expect("recover");
        assert_eq!(recovered, 1);
        let got = store.get_task("a").expect("get").expect("exists");
        assert_eq!(got.status, TaskStatus::Pending);
        assert!(got.lease_id.is_none());
        assert!(got.error.is_some());
        // cancel 仅 pending/running；retry 仅 failed/cancelled
        store.cancel_task("a").expect("cancel");
        assert_eq!(
            store.get_task("a").expect("get").expect("exists").status,
            TaskStatus::Cancelled
        );
        assert!(store.cancel_task("a").is_err());
        store.retry_task("a").expect("retry");
        assert_eq!(
            store.get_task("a").expect("get").expect("exists").status,
            TaskStatus::Pending
        );
        assert!(store.retry_task("a").is_err());
    }


    #[test]
    fn rename_and_delete_task() {        let store = NeobotStore::open(":memory:").expect("open memory db");
        let task = AgentTask {
            id: "r1".to_owned(),
            title: "old".to_owned(),
            status: TaskStatus::Pending,
            created_at: "2026-09-24T00:00:00Z".to_owned(),
            updated_at: "2026-09-24T00:00:00Z".to_owned(),
            claimed_by: None,
            claimed_at: None,
            visibility: crate::nt_types::default_visibility(),
            lease_id: None,
            lease_until: None,
            attempts: 0,
            error: None,
            conversation_id: None,
        };
        store.save_task(&task).expect("save");
        store.add_step("r1", 0, "reply", true, "hi").expect("step");
        assert!(store.rename_task("r1", "   ").is_err());
        assert!(store.rename_task("r1", &"x".repeat(161)).is_err());
        assert!(store.rename_task("nope", "t").is_err());
        store.rename_task("r1", "new name").expect("rename");
        assert_eq!(
            store.get_task("r1").expect("get").expect("exists").title,
            "new name"
        );
        assert!(store.delete_task("nope").is_err());
        store.delete_task("r1").expect("delete");
        assert!(store.get_task("r1").expect("get").is_none());
    }


    #[test]
    fn stale_claims_swept_by_ttl() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        let task = AgentTask {
            id: "c1".to_owned(),
            title: "c".to_owned(),
            status: TaskStatus::Pending,
            created_at: "2026-09-24T00:00:00Z".to_owned(),
            updated_at: "2026-09-24T00:00:00Z".to_owned(),
            claimed_by: None,
            claimed_at: None,
            visibility: crate::nt_types::default_visibility(),
            lease_id: None,
            lease_until: None,
            attempts: 0,
            error: None,
            conversation_id: None,
        };
        store.save_task(&task).expect("save");
        store
            .claim_task_at("c1", "alice", "2026-09-24T00:00:00Z")
            .expect("claim");
        // 60s 后扫（TTL 300）：还在；400s 后扫：释放
        assert_eq!(
            store
                .sweep_stale_claims("2026-09-24T00:01:00Z", crate::nt_store::CLAIM_TTL_SECS)
                .expect("sweep"),
            0
        );
        assert_eq!(
            store
                .sweep_stale_claims("2026-09-24T00:07:00Z", crate::nt_store::CLAIM_TTL_SECS)
                .expect("sweep"),
            1
        );
        let got = store.get_task("c1").expect("get").expect("exists");
        assert!(got.claimed_by.is_none());
    }

}
