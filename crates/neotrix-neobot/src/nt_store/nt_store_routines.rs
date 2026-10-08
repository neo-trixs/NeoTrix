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
    /// 按 key_env 聚合总成本/用量（quota 窗口切片；COALESCE 空串为 '-'）。
    pub fn ledger_sums_by_key_env(
        &self,
    ) -> Result<Vec<(String, String, String, i64, i64, f64)>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT engine, model, COALESCE(NULLIF(key_env, ''), '-') AS ke,
              COALESCE(CAST(SUM(in_tokens) AS INTEGER),0),
              COALESCE(CAST(SUM(out_tokens) AS INTEGER),0),
              COALESCE(SUM(cost_usd),0.0)
             FROM ledger GROUP BY engine, model, ke",
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

    /// 记一条 step（**不**带配平键；老签名保持不变）。
    ///
    /// ⛔ 这里**故意**不删不改签名：`add_step` 除 `nt_agent` 跑轮外还有 9 处测试
    /// 调用点，且 `nt_agent` 那一处正在被别人改 —— 新语义一律走
    /// [`NeobotStore::add_step_with_call`]（本函数即它的 `None` 包装）。
    pub fn add_step(
        &self,
        task_id: &str,
        n: i64,
        tool: &str,
        ok: bool,
        output: &str,
    ) -> Result<(), NtBotError> {
        self.add_step_with_call(task_id, n, tool, ok, output, None)
    }

    /// 记一条 step，并把**工具调用 id**（配平键）一起落库。
    ///
    /// ## 为什么需要这一列
    ///
    /// 「工具调用 ↔ 工具结果」这条不变量在 `nt_agent` 里被当成规则写着
    /// （汇总 steps 行就是为了「不留悬空的 tool_call id」），但**两侧用的是
    /// 不同键**：transcript/history 侧有 `TranscriptItem.tool_call_id`，
    /// `steps` 侧原先**什么都没有** ⇒ join 对真实落库数据**根本无法表达**，
    /// 不变量检查只能诚实回 `NOT_EVALUABLE`。补这一列是把不变量从
    /// 「写在注释里的愿望」变成「可判定」的**最小**改动（不是重写）。
    ///
    /// ## `tool_call_id = None` 是什么意思
    ///
    /// ⛔ **`None` = 这一行是本列存在之前写入的旧行**（或调用方走
    /// [`NeobotStore::add_step`] 没给键），**不是**「这次没发生工具调用」。
    /// 两者混同的后果是实打实的：配平检查会拿 `None` 当「有个结果没有对应
    /// 调用」⇒ **每一行存量账都被报成孤儿结果**（假阳性），而假阳性足以让人
    /// 把检查关掉。所以读侧必须原样保留 `NULL`、不许折成空串或 `Some("")`。
    ///
    /// 非工具步骤（`reply` / `side-effect:*` / `cancelled:tool_calls` 汇总行）
    /// 没有可配的调用 id，一律传 `None` —— 它们**本就不该**参与配平。
    pub fn add_step_with_call(
        &self,
        task_id: &str,
        n: i64,
        tool: &str,
        ok: bool,
        output: &str,
        tool_call_id: Option<&str>,
    ) -> Result<(), NtBotError> {
        self.conn.execute(
            "INSERT INTO steps(task_id,n,tool,ok,output,tool_call_id)
             VALUES(?1,?2,?3,?4,?5,?6)",
            params![task_id, n, tool, i64::from(ok), output, tool_call_id],
        )?;
        Ok(())
    }

    /// 某任务的全部 step 行（**按写入序**），含配平键 `tool_call_id`。
    ///
    /// 这是配平检查的读口：[`NeobotStore::last_step`] 只给「某工具名的最后
    /// 一行」，而 join 要的是**全集** —— 一个配平键出现两次（一 `Call` 一
    /// `Result`）才是正常配平，按工具名投影会把配平对切碎。
    ///
    /// `tool_call_id` 原样透传 `NULL`（见 [`NeobotStore::add_step_with_call`]
    /// 的 `None` 语义）：**判定方必须自己区分「旧行」与「无调用」**，
    /// 读口替它猜就是造假。
    pub fn list_steps(&self, task_id: &str) -> Result<Vec<StepRow>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, n, tool, ok, output, tool_call_id FROM steps
             WHERE task_id=?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![task_id], |r| {
            Ok(StepRow {
                id: r.get(0)?,
                n: r.get(1)?,
                tool: r.get(2)?,
                ok: r.get::<_, i64>(3)? != 0,
                output: r.get(4)?,
                tool_call_id: r.get(5)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }
}

/// `steps` 一行（[`NeobotStore::list_steps`] 返回）。
///
/// ⛔ `tool_call_id` 是 `Option<String>` 且**空串与 `NULL` 必须可区分** ——
/// 空串是**非法**的调用 id（会与「没有键」撞成假配平），`NULL` 是**合法**的
/// 存量状态（见 [`NeobotStore::add_step_with_call`]）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRow {
    /// `steps.id`（`AUTOINCREMENT` 行号；写入序的判据）。
    pub id: i64,
    /// 第几跳（`0` 起）。
    pub n: i64,
    /// 工具名（或 `reply` / `side-effect:*` / `cancelled:tool_calls`）。
    pub tool: String,
    /// 这一步本身成功没有。
    pub ok: bool,
    /// 输出正文。
    pub output: String,
    /// 配平键；`None` = **本列存在之前写入的旧行**（不是「没调工具」）。
    pub tool_call_id: Option<String>,
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

    // ══════════════════════════════════════════════════════════════════
    // `steps.tool_call_id`：配平键的落库 / 读回 / 迁移
    //
    // 这几条测试守的是**一条已上线的不变量**：工具调用与工具结果必须能用
    // **同一把钥匙**接起来（`nt_core_artifact_verdict::ToolCallJoinCheck`
    // 据此判孤儿调用/孤儿结果）。缺列时它只能诚实回 `NOT_EVALUABLE`。
    // ══════════════════════════════════════════════════════════════════

    /// 一个新库专属的 db 文件路径（每次调用都不同 ⇒ 并发跑不互擦）。
    fn temp_db(tag: &str) -> std::path::PathBuf {
        let dir = crate::nt_testutil::temp_dir(tag);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir.join("store.sqlite")
    }

    /// 造一个**存量形态**的库：`steps` 只有补列前的六列，且已有数据行。
    /// ⛔ 直接用 `rusqlite` 手搓，不用 `NeobotStore::open` —— 后者会自己
    ///    补列，那样就测不到「ALTER 真的能救活旧库」了。
    fn legacy_steps_db(path: &std::path::Path) {
        let conn = rusqlite::Connection::open(path).expect("open legacy db");
        conn.execute_batch(
            "CREATE TABLE steps(
               id INTEGER PRIMARY KEY AUTOINCREMENT, task_id TEXT NOT NULL,
               n INTEGER NOT NULL, tool TEXT NOT NULL, ok INTEGER NOT NULL,
               output TEXT NOT NULL);
             CREATE TABLE tasks(
               id TEXT PRIMARY KEY, title TEXT NOT NULL, status TEXT NOT NULL,
               created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
               claimed_by TEXT, claimed_at TEXT,
               visibility TEXT NOT NULL DEFAULT 'team',
               lease_id TEXT, lease_until TEXT,
               attempts INTEGER NOT NULL DEFAULT 0, error TEXT);
             INSERT INTO steps(task_id,n,tool,ok,output)
               VALUES('legacy-task',0,'bash',1,'old output');
             INSERT INTO tasks(id,title,status,created_at,updated_at)
               VALUES('legacy-task','legacy','done','1970-01-01T00:00:00Z','1970-01-01T00:00:00Z');",
        )
        .expect("seed legacy db");
    }

    /// 带 id 写入 ⇒ 原样读回；且读口必须能**枚举全集**（join 要的是全集，
    /// 不是「某工具名的最后一行」）。
    #[test]
    fn step_with_call_id_roundtrips() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        store
            .add_step_with_call("t1", 0, "bash", true, "out-1", Some("call_a"))
            .expect("step with id");
        let rows = store.list_steps("t1").expect("list");
        assert_eq!(rows.len(), 1, "写一行读一行");
        assert_eq!(rows[0].tool, "bash");
        assert!(rows[0].ok);
        assert_eq!(rows[0].output, "out-1");
        assert_eq!(
            rows[0].tool_call_id.as_deref(),
            Some("call_a"),
            "配平键必须原样读回"
        );
        // 未知任务回空，不炸。
        assert!(store.list_steps("nope").expect("list").is_empty());
    }

    /// ⛔ `None` 与空串**必须可区分**：`None` = 旧行（不可判定），
    /// 空串 = 有一个（非法）键。折成同一个值就会造出假配平。
    #[test]
    fn null_id_is_distinguishable_from_empty_string() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        store
            .add_step("t1", 0, "reply", true, "no key")
            .expect("no key");
        store
            .add_step_with_call("t1", 1, "bash", true, "empty key", Some(""))
            .expect("empty key");
        let rows = store.list_steps("t1").expect("list");
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0].tool_call_id, None,
            "老 API 不给键 ⇒ 必须落成真 NULL（SQL NULL，不是空串）"
        );
        assert_eq!(
            rows[1].tool_call_id.as_deref(),
            Some(""),
            "显式给空串 ⇒ 必须与 NULL 读回不同值"
        );
        assert_ne!(
            rows[0].tool_call_id, rows[1].tool_call_id,
            "NULL 与空串混同 ⇒ 配平检查会把旧行读成有键（或反之）"
        );
    }

    /// **迁移幂等**：同一个库开两次都不得报错、且数据存活。
    ///
    /// 这条是本组最要紧的一条：`migrate()` 的 `ALTER` 列表是
    /// **无条件逐条执行**的（SQLite 不支持 `ADD COLUMN IF NOT EXISTS`），
    /// 第二次跑必然报 `duplicate column name` ⇒ 幂等**只**由
    /// `self.conn.execute(alter, []).ok()` 吞错这一句保证。若那行被删/被改成
    /// `?`，所有存量用户**开库即炸** —— 而这种炸法只有真跑一遍旧库才看得见。
    #[test]
    fn migration_is_idempotent_across_repeated_opens() {
        let path = temp_db("steps-migrate-twice");
        let p = path.to_str().expect("utf8 path");

        let first = NeobotStore::open(p).expect("第一次 open（建表 + 补列）");
        first
            .add_step_with_call("t1", 0, "bash", true, "survivor", Some("call_a"))
            .expect("write before reopen");
        drop(first);

        // 第二次、第三次 open：migrate 整套重跑，ALTER 必然撞 duplicate column。
        for round in 2..=3 {
            let store = NeobotStore::open(p)
                .unwrap_or_else(|e| panic!("第 {round} 次 open 不得报错（幂等）：{e}"));
            let rows = store.list_steps("t1").expect("list after reopen");
            assert_eq!(rows.len(), 1, "第 {round} 次 open 后数据必须存活");
            assert_eq!(
                rows[0].tool_call_id.as_deref(),
                Some("call_a"),
                "第 {round} 次 open 后配平键必须存活"
            );
        }
        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }

    /// **存量库迁移**：表里没有该列的旧库必须能开开，且旧行读回 `NULL`
    /// （**不是**被编造一个 id、**不是**空串）。
    #[test]
    fn legacy_db_without_column_migrates_and_keeps_rows() {
        let path = temp_db("steps-legacy");
        legacy_steps_db(&path);

        // 开之前先确认「列真的不在」—— 否则这条测试可能因为建表语句已被
        // 改动而假绿（这就是它必须自己手搓旧 schema 的原因）。
        {
            let conn = rusqlite::Connection::open(&path).expect("reopen legacy");
            let has: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM pragma_table_info('steps') WHERE name='tool_call_id'",
                    [],
                    |r| r.get(0),
                )
                .expect("pragma");
            assert_eq!(has, 0, "夹具前提：旧库不该有该列");
        }

        let store = NeobotStore::open(path.to_str().expect("utf8 path"))
            .expect("旧库必须能开（ALTER 补列不炸）");
        let rows = store.list_steps("legacy-task").expect("list legacy");
        assert_eq!(rows.len(), 1, "旧行必须存活");
        assert_eq!(rows[0].tool, "bash");
        assert_eq!(rows[0].output, "old output");
        assert_eq!(
            rows[0].tool_call_id, None,
            "旧行的配平键必须是 NULL（不可判定），绝不能被 DEFAULT 编造出来"
        );
        // 旧库里别的表也不能被迁移弄坏。
        let tasks = store.list_tasks(10).expect("tasks still readable");
        assert_eq!(tasks.len(), 1, "旧 tasks 行也必须存活");
        // 迁移后新写入仍能带上键（证明补列真的落到表上了，不是只在读口糊弄）。
        store
            .add_step_with_call("legacy-task", 1, "bash", true, "new", Some("call_z"))
            .expect("write after migration");
        let rows = store.list_steps("legacy-task").expect("list after write");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].tool_call_id, None, "旧行仍为 NULL");
        assert_eq!(rows[1].tool_call_id.as_deref(), Some("call_z"));
        drop(store);
        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }

    /// 读口一致性：三个 step 读口都必须把配平键原样透出（同一条 SQL 语义，
    /// 不许有一个把它折掉）。
    #[test]
    fn all_step_reads_surface_the_join_key() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        store
            .add_step_with_call("t1", 0, "bash", true, "kept", Some("call_a"))
            .expect("step");
        assert_eq!(
            store
                .last_step("t1", "bash")
                .expect("last")
                .expect("row")
                .tool_call_id,
            Some("call_a".to_owned()),
            "last_step 必须带出配平键"
        );
        assert_eq!(
            store.list_step_tools("t1").expect("tools"),
            vec!["bash".to_owned()],
            "list_step_tools 不该受影响"
        );
        assert_eq!(
            store
                .last_step_output("t1", "bash")
                .expect("out")
                .as_deref(),
            Some("kept"),
            "last_step_output 不该受影响"
        );
    }
}
