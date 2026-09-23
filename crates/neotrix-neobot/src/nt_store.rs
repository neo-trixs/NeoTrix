//! `nt_store` — SQLite 持久化.
//!
//! 移植 OpenMuse 通用 `Store{put/compareAndSwap/claim}` 思想 +
//! cumora `llm_calls` 账本 + `realtime-outbox` (事务内 enqueue +
//! leased drain), 全部收敛到单文件 SQLite, 无 Postgres/Redis.

use rusqlite::{Connection, OptionalExtension, params};

use crate::nt_audit::{AuditDecision, AuditEvent};
use crate::nt_error::NtBotError;
use crate::nt_types::{AgentTask, TaskStatus};

/// SQLite store (单文件, `bundled` 特性本地编译, 无外部服务).
pub struct NeobotStore {
    conn: Connection,
}

impl NeobotStore {
    /// 打开 (内存 `:memory:` 或文件), 幂等建表.
    pub fn open(path: &str) -> Result<Self, NtBotError> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), NtBotError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks(
               id TEXT PRIMARY KEY, title TEXT NOT NULL, status TEXT NOT NULL,
               created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS steps(
               id INTEGER PRIMARY KEY AUTOINCREMENT, task_id TEXT NOT NULL,
               n INTEGER NOT NULL, tool TEXT NOT NULL, ok INTEGER NOT NULL,
               output TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS audit(
               id TEXT PRIMARY KEY, at TEXT NOT NULL, actor TEXT NOT NULL,
               tool TEXT NOT NULL, decision TEXT NOT NULL,
               rule TEXT, detail TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS ledger(
               id TEXT PRIMARY KEY, at TEXT NOT NULL, engine TEXT NOT NULL,
               model TEXT NOT NULL, in_tokens INTEGER NOT NULL,
               out_tokens INTEGER NOT NULL, cost_usd REAL NOT NULL);
             CREATE TABLE IF NOT EXISTS outbox(
               id TEXT PRIMARY KEY, topic TEXT NOT NULL,
               payload TEXT NOT NULL, claimed INTEGER NOT NULL DEFAULT 0);",
        )?;
        Ok(())
    }

    // ---- tasks (durable TaskWorker 简化: insert + status CAS) ----

    pub fn save_task(&self, task: &AgentTask) -> Result<(), NtBotError> {
        self.conn.execute(
            "INSERT INTO tasks(id,title,status,created_at,updated_at)
             VALUES(?1,?2,?3,?4,?5)
             ON CONFLICT(id) DO UPDATE SET title=excluded.title,
               status=excluded.status, updated_at=excluded.updated_at",
            params![task.id, task.title, task.status.as_str(), task.created_at, task.updated_at],
        )?;
        Ok(())
    }

    pub fn get_task(&self, id: &str) -> Result<Option<AgentTask>, NtBotError> {
        let row: Option<(String, String, String, String, String)> = self
            .conn
            .query_row(
                "SELECT id,title,status,created_at,updated_at FROM tasks WHERE id=?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()?;
        let Some((id, title, status_raw, created_at, updated_at)) = row else {
            return Ok(None);
        };
        let status = TaskStatus::parse(&status_raw)
            .ok_or_else(|| NtBotError::Store(format!("unknown task status '{status_raw}'")))?;
        Ok(Some(AgentTask {
            id,
            title,
            status,
            created_at,
            updated_at,
        }))
    }

    pub fn list_tasks(&self, limit: i64) -> Result<Vec<AgentTask>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT id,title,status,created_at,updated_at FROM tasks
             ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit.max(1)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, title, status_raw, created_at, updated_at) = row?;
            let Some(status) = TaskStatus::parse(&status_raw) else {
                continue;
            };
            out.push(AgentTask {
                id,
                title,
                status,
                created_at,
                updated_at,
            });
        }
        Ok(out)
    }

    pub fn add_step(
        &self,
        task_id: &str,
        n: i64,
        tool: &str,
        ok: bool,
        output: &str,
    ) -> Result<(), NtBotError> {
        self.conn.execute(
            "INSERT INTO steps(task_id,n,tool,ok,output) VALUES(?1,?2,?3,?4,?5)",
            params![task_id, n, tool, i64::from(ok), output],
        )?;
        Ok(())
    }

    // ---- audit (append-only) ----

    pub fn record_audit(&self, event: &AuditEvent) -> Result<(), NtBotError> {
        self.conn.execute(
            "INSERT INTO audit(id,at,actor,tool,decision,rule,detail)
             VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![
                event.id,
                event.at,
                event.actor,
                event.tool,
                event.decision.as_str(),
                event.rule,
                event.detail
            ],
        )?;
        Ok(())
    }

    pub fn list_audit(&self, limit: i64) -> Result<Vec<AuditEvent>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT id,at,actor,tool,decision,rule,detail FROM audit
             ORDER BY at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit.max(1)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, String>(6)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, at, actor, tool, decision_raw, rule, detail) = row?;
            let decision = match decision_raw.as_str() {
                "deny" => AuditDecision::Deny,
                _ => AuditDecision::Allow,
            };
            out.push(AuditEvent {
                id,
                at,
                actor,
                tool,
                decision,
                rule,
                detail,
            });
        }
        Ok(out)
    }

    // ---- ledger (cumora llm_calls 本地子集) ----

    pub fn record_ledger(
        &self,
        id: &str,
        at: &str,
        engine: &str,
        model: &str,
        in_tokens: i64,
        out_tokens: i64,
        cost_usd: f64,
    ) -> Result<(), NtBotError> {
        self.conn.execute(
            "INSERT INTO ledger(id,at,engine,model,in_tokens,out_tokens,cost_usd)
             VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![id, at, engine, model, in_tokens, out_tokens, cost_usd],
        )?;
        Ok(())
    }

    // ---- outbox (leased drain, OpenMuse claim 语义简化) ----

    pub fn enqueue_outbox(&self, id: &str, topic: &str, payload: &str) -> Result<(), NtBotError> {
        self.conn.execute(
            "INSERT INTO outbox(id,topic,payload,claimed) VALUES(?1,?2,?3,0)",
            params![id, topic, payload],
        )?;
        Ok(())
    }

    pub fn drain_outbox(&self, limit: i64) -> Result<Vec<(String, String, String)>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT id,topic,payload FROM outbox WHERE claimed=0 LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit.max(1)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        for (id, _, _) in &out {
            self.conn.execute("UPDATE outbox SET claimed=1 WHERE id=?1", params![id])?;
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::NeobotStore;
    use crate::nt_audit::{AuditDecision, AuditEvent};
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
        };
        store.save_task(&task).expect("save");
        let got = store.get_task("t1").expect("get").expect("exists");
        assert_eq!(got.title, "demo");
        store.enqueue_outbox("o1", "CH_MESSAGE_NEW", "{}").expect("enqueue");
        let drained = store.drain_outbox(10).expect("drain");
        assert_eq!(drained.len(), 1);
        assert!(store.drain_outbox(10).expect("drain2").is_empty());
    }

    #[test]
    fn audit_append_only() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        let event = AuditEvent::new("bot", "bash", AuditDecision::Deny, Some("workspace-jail".to_owned()), "x");
        store.record_audit(&event).expect("record");
        assert_eq!(store.list_audit(10).expect("list").len(), 1);
    }
}
