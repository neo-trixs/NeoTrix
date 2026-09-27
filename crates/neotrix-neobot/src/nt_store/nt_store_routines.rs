//! `nt_store_routines` — 定时例行（15min 地板 + 10 连败自停）。

use super::{LedgerActorSum, LedgerSum, NeobotStore, TaskRow};
use crate::nt_types::{AgentTask, TaskStatus};
use crate::nt_error::NtBotError;
use rusqlite::{OptionalExtension, params};
use crate::nt_audit::{AuditDecision, AuditEvent};

impl NeobotStore {
    // ---- routines（定时例行：15min 地板 + 10 连败自停） ----

    /// 注册例行：interval < 900s 拒；instruction 超 2000 字拒；
    /// 同名存在拒（先 remove 再 add 即改）。next_run_at=now（注册即到期）。
    pub fn register_routine(
        &self,
        name: &str,
        interval_secs: i64,
        instruction: &str,
        owner: &str,
        now_epoch: i64,
    ) -> Result<(), NtBotError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(NtBotError::Invalid("routine name is empty".to_owned()));
        }
        if interval_secs < crate::nt_routine::MIN_INTERVAL_SECS {
            return Err(NtBotError::Invalid(format!(
                "routines may run at most every {} minutes",
                crate::nt_routine::MIN_INTERVAL_SECS / 60
            )));
        }
        if instruction.chars().count() > crate::nt_routine::MAX_INSTRUCTION_CHARS {
            return Err(NtBotError::Invalid(format!(
                "instruction exceeds {} chars",
                crate::nt_routine::MAX_INSTRUCTION_CHARS
            )));
        }
        let enabled = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM routines WHERE disabled=0",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0);
        if enabled >= crate::nt_routine::MAX_ENABLED_ROUTINES {
            return Err(NtBotError::Invalid(format!(
                "too many enabled routines (max {})",
                crate::nt_routine::MAX_ENABLED_ROUTINES
            )));
        }
        let exists: Option<String> = self
            .conn
            .query_row("SELECT name FROM routines WHERE name=?1", params![name], |r| {
                r.get(0)
            })
            .optional()?;
        if exists.is_some() {
            return Err(NtBotError::Store(format!("routine '{name}' already exists")));
        }
        self.conn.execute(
            "INSERT INTO routines(name,interval_secs,instruction,owner,
              failures,disabled,next_run_at,last_run_at,last_error)
             VALUES(?1,?2,?3,?4,0,0,?5,0,NULL)",
            params![name, interval_secs, instruction, owner, now_epoch],
        )?;
        Ok(())
    }

    pub fn remove_routine(&self, name: &str) -> Result<(), NtBotError> {
        let n = self
            .conn
            .execute("DELETE FROM routines WHERE name=?1", params![name])?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such routine '{name}'")));
        }
        Ok(())
    }

    pub fn list_routines(&self) -> Result<Vec<crate::nt_routine::Routine>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT name,interval_secs,instruction,owner,failures,disabled,
              next_run_at,last_run_at,last_error FROM routines ORDER BY name",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(crate::nt_routine::Routine {
                name: r.get(0)?,
                interval_secs: r.get(1)?,
                instruction: r.get(2)?,
                owner: r.get(3)?,
                failures: r.get(4)?,
                disabled: r.get(5)?,
                next_run_at: r.get(6)?,
                last_run_at: r.get(7)?,
                last_error: r.get(8)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// 到期且未禁用的例行（sweep 用；now 为 epoch 秒）。
    pub fn due_routines(
        &self,
        now_epoch: i64,
    ) -> Result<Vec<crate::nt_routine::Routine>, NtBotError> {
        Ok(self
            .list_routines()?
            .into_iter()
            .filter(|r| r.disabled == 0 && r.next_run_at <= now_epoch)
            .collect())
    }

    /// 记录一次 firing 结果：成功清零 failures；失败 +1，
    /// 触疲劳上限（10 连败）即禁用 + 审计注记。
    /// next_run 恒按 interval 推进（成功失败都推进，不补跑）。
    pub fn record_routine_run(
        &self,
        name: &str,
        now_epoch: i64,
        ok: bool,
        error: Option<&str>,
    ) -> Result<(), NtBotError> {
        let routine = self
            .list_routines()?
            .into_iter()
            .find(|r| r.name == name)
            .ok_or_else(|| NtBotError::Store(format!("no such routine '{name}'")))?;
        let failures = if ok { 0 } else { routine.failures + 1 };
        let disabled = i64::from(failures >= crate::nt_routine::FATIGUE_LIMIT);
        let next = now_epoch + routine.interval_secs.max(60);
        self.conn.execute(
            "UPDATE routines SET failures=?1, disabled=?2, next_run_at=?3,
              last_run_at=?4, last_error=?5 WHERE name=?6",
            params![failures, disabled, next, now_epoch, error, name],
        )?;
        if disabled == 1 && routine.disabled == 0 {
            let event = AuditEvent::new(
                &routine.owner,
                "routine",
                AuditDecision::Deny,
                Some("routine-fatigue".to_owned()),
                &format!("routine '{name}' failed {failures} times in a row; switched off"),
            );
            self.record_audit(&event)?;
        }
        Ok(())
    }

    /// 成本账聚合（按 engine+model；actor 维度见 `ledger_sums_by_actor`）。
    /// `SUM()` 遇 REAL 行回 REAL、空表回 NULL——一律 `COALESCE(CAST…INTEGER)` 收敛，
    /// 否则 `Invalid column type Real/Null` 炸聚合（2026-09-26 ledger 实锤）。
    pub fn ledger_sums(&self) -> Result<Vec<LedgerSum>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT engine, model, COALESCE(CAST(SUM(in_tokens) AS INTEGER),0), COALESCE(CAST(SUM(out_tokens) AS INTEGER),0), COALESCE(SUM(cost_usd),0.0)
             FROM ledger GROUP BY engine, model",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, f64>(4)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// 成本账聚合（按 engine+model+actor 切账；SUM 收敛同上）。
    pub fn ledger_sums_by_actor(&self) -> Result<Vec<LedgerActorSum>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT engine, model, actor, COALESCE(CAST(SUM(in_tokens) AS INTEGER),0), COALESCE(CAST(SUM(out_tokens) AS INTEGER),0), COALESCE(SUM(cost_usd),0.0)
             FROM ledger GROUP BY engine, model, actor",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, f64>(5)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn get_task(&self, id: &str) -> Result<Option<AgentTask>, NtBotError> {
        let row: Option<TaskRow> = self
            .conn
            .query_row(
                "SELECT id,title,status,created_at,updated_at,claimed_by,claimed_at,
                  visibility,lease_id,lease_until,attempts,error,conversation_id
                 FROM tasks WHERE id=?1",
                params![id],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                        r.get(7)?,
                        r.get(8)?,
                        r.get(9)?,
                        r.get(10)?,
                        r.get(11)?,
                        r.get(12)?,
                    ))
                },
            )
            .optional()?;
        let Some((
            id,
            title,
            status_raw,
            created_at,
            updated_at,
            claimed_by,
            claimed_at,
            visibility,
            lease_id,
            lease_until,
            attempts,
            error,
            conversation_id,
        )) = row
        else {
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
            claimed_by,
            claimed_at,
            visibility,
            lease_id,
            lease_until,
            attempts,
            error,
            conversation_id,
        }))
    }

    pub fn list_tasks(&self, limit: i64) -> Result<Vec<AgentTask>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT id,title,status,created_at,updated_at,claimed_by,claimed_at,
              visibility,lease_id,lease_until,attempts,error,conversation_id FROM tasks
             ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit.max(1)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, Option<String>>(8)?,
                r.get::<_, Option<String>>(9)?,
                r.get::<_, i64>(10)?,
                r.get::<_, Option<String>>(11)?,
                r.get::<_, Option<String>>(12)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (
                id,
                title,
                status_raw,
                created_at,
                updated_at,
                claimed_by,
                claimed_at,
                visibility,
                lease_id,
                lease_until,
                attempts,
                error,
                conversation_id,
            ) = row?;
            let Some(status) = TaskStatus::parse(&status_raw) else {
                continue;
            };
            out.push(AgentTask {
                id,
                title,
                status,
                created_at,
                updated_at,
                claimed_by,
                claimed_at,
                visibility,
                lease_id,
                lease_until,
                attempts,
                error,
                conversation_id,
            });
        }
        Ok(out)
    }

    /// 按会话列任务（会话 thread 用）.
    pub fn list_convo_tasks(
        &self,
        convo_id: &str,
        limit: i64,
    ) -> Result<Vec<AgentTask>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT id,title,status,created_at,updated_at,claimed_by,claimed_at,
              visibility,lease_id,lease_until,attempts,error,conversation_id FROM tasks
             WHERE conversation_id=?1 ORDER BY created_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![convo_id, limit.max(1)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, Option<String>>(8)?,
                r.get::<_, Option<String>>(9)?,
                r.get::<_, i64>(10)?,
                r.get::<_, Option<String>>(11)?,
                r.get::<_, Option<String>>(12)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (
                id,
                title,
                status_raw,
                created_at,
                updated_at,
                claimed_by,
                claimed_at,
                visibility,
                lease_id,
                lease_until,
                attempts,
                error,
                conversation_id,
            ) = row?;
            let Some(status) = TaskStatus::parse(&status_raw) else {
                continue;
            };
            out.push(AgentTask {
                id,
                title,
                status,
                created_at,
                updated_at,
                claimed_by,
                claimed_at,
                visibility,
                lease_id,
                lease_until,
                attempts,
                error,
                conversation_id,
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
}

#[cfg(test)]
mod tests {
    use crate::nt_store::NeobotStore;

    #[test]
    fn routine_guards_and_fatigue() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        // 地板：900s 以下拒
        assert!(store
            .register_routine("fast", 60, "do x", "alice", 1000)
            .is_err());
        // 指令超长拒
        assert!(store
            .register_routine("long", 900, &"x".repeat(2001), "alice", 1000)
            .is_err());
        store
            .register_routine("daily", 900, "check inbox", "alice", 1000)
            .expect("register");
        assert!(store
            .register_routine("daily", 900, "dup", "alice", 1000)
            .is_err());
        // 注册即到期
        assert_eq!(store.due_routines(1000).expect("due").len(), 1);
        assert!(store.due_routines(999).expect("due").is_empty());
        // 9 连败不禁用，第 10 次禁用 + 审计
        for _ in 0..9 {
            store
                .record_routine_run("daily", 1000, false, Some("boom"))
                .expect("record");
        }
        let mid = store.list_routines().expect("list");
        assert_eq!(mid[0].failures, 9);
        assert_eq!(mid[0].disabled, 0);
        store
            .record_routine_run("daily", 2000, false, Some("boom"))
            .expect("record");
        let last = store.list_routines().expect("list");
        assert_eq!(last[0].failures, 10);
        assert_eq!(last[0].disabled, 1);
        assert!(store.due_routines(99999).expect("due").is_empty());
        let audits = store.list_audit(10).expect("audits");
        assert!(audits
            .iter()
            .any(|e| e.rule.as_deref() == Some("routine-fatigue")));
        // 成功清零
        store
            .record_routine_run("daily", 3000, true, None)
            .expect("record");
        assert_eq!(store.list_routines().expect("list")[0].failures, 0);
        store.remove_routine("daily").expect("remove");
        assert!(store.remove_routine("daily").is_err());
    }

}
