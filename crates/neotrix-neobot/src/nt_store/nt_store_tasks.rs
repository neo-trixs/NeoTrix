//! `nt_store_tasks` — 任务 insert/status-CAS/认领租约/租约回收（`NeobotStore` 的 impl 片段）。

use super::{LedgerEntry, NeobotStore};
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

/// `steps` 里某一步的取值（[`NeobotStore::last_step`] 返回）。
///
/// 早先是 `Option<(bool, String)>` —— 位置性返回，调用方只能靠 `marker.0` /
/// `marker.1` 猜哪个是 `ok`、哪个是 `output`；而 SQL 里的 `tool` 是**查询条件**
/// （形参），**不在返回值里**，极易被误当成第二个字段的名字。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastStep {
    /// `steps.ok`：这一步**本身**成功没有。
    pub ok: bool,
    /// `steps.output`：输出正文。
    pub output: String,
    /// `steps.tool_call_id`：配平键（工具调用 ↔ 工具结果用同一把钥匙）。
    ///
    /// ⛔ `None` = **本列存在之前写入的旧行**，**不是**「这一步没调工具」。
    /// 判定方必须自己分辨这两种情况：把旧行当成「无对应调用」会把**每一行
    /// 存量账**报成孤儿结果（假阳性）。加字段是**加法**（老调用点仍只读
    /// `.ok` / `.output`，编译不受影响）。
    pub tool_call_id: Option<String>,
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

    /// 某任务**最后一条**指定工具的 step 行（`{ok, output, tool_call_id}`，
    /// 按写入序）。
    ///
    /// 与 `nt_store_reply_tag::last_step_output` 互补：那个只取 `output`
    /// （给侧聊继承摘要），这个把 `ok` 一起带出来 —— 取消落库要能自证
    /// 「那行 `cancelled:tool_calls` 的 `ok` 是 0」，不能只凭工具名猜。
    /// 再把 `tool_call_id` 带出来 —— 配平检查要靠它把结果接回调用。
    /// 没有匹配行即 `None`。
    pub fn last_step(&self, task_id: &str, tool: &str) -> Result<Option<LastStep>, NtBotError> {
        Ok(self
            .conn
            .query_row(
                "SELECT ok, output, tool_call_id FROM steps
                 WHERE task_id=?1 AND tool=?2 ORDER BY id DESC LIMIT 1",
                params![task_id, tool],
                |r| {
                    Ok(LastStep {
                        ok: r.get::<_, i64>(0)? != 0,
                        output: r.get::<_, String>(1)?,
                        tool_call_id: r.get::<_, Option<String>>(2)?,
                    })
                },
            )
            .ok())
    }

    /// 续租：把 `lease_until` 推后，**仅限仍持有这把钥匙的那一轮**。
    ///
    /// 两个条件缺一不可（`lease_id` 相等 **且** 仍 `running`）：
    /// - 少了 `lease_id` 相等 → 一台慢机器上的僵尸跑轮能把**别人已经接管**
    ///   的行改回自己手里（抢别人正在写的行 = 数据损坏）。
    /// - 少了 `status='running'` → 已经落终态的行会被心跳重新拉回「活着」。
    ///
    /// 返回受影响行数：`0` = **不再持有**（已被 `recover_stale_running`
    /// 回收或被他人接管）。调用方据此**不再重试** —— 拿不回钥匙就不该
    /// 假装还拿着。
    ///
    /// `lease_until` 由调用方按**滚动窗口**给（`now + LEASE_SECS`），不是
    /// 在旧值上累加。这一点是回收保证的全部：进程一死，最坏仍只挂一个
    /// `LEASE_SECS`，绝不会因为续过 N 次而挂 N 倍。
    pub fn renew_lease(
        &self,
        id: &str,
        lease_id: &str,
        lease_until: &str,
    ) -> Result<usize, NtBotError> {
        let n = self.conn.execute(
            "UPDATE tasks SET lease_until=?1, updated_at=?1
             WHERE id=?2 AND lease_id=?3 AND status='running'",
            params![lease_until, id, lease_id],
        )?;
        Ok(n)
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
    /// running 且租约过期（或无租约）→ **`outcome_unknown`** + error 注记，返回回收条数。
    /// 调用方在每次 run 起点调一次即可（单机，代价一次 UPDATE）。
    ///
    /// **2026-10-02 语义变更：不再打回 `pending`。**
    /// 原实现 `SET status='pending'` 隐含一个假设：**重跑是安全的**。
    /// ⛔ 但崩溃窗口里存在「**外部副作用已落地、结果未落库**」的缝隙
    /// （例：`neobot_send` 已把消息发出去、还没写进 `messages`），
    /// 此时打回 `pending` ⇒ 重跑**二次执行**该副作用，而界面完全看不出异常。
    ///
    /// ⇒ 改为落到 `TaskStatus::OutcomeUnknown`：**不自动重试，等人工裁决**。
    /// ⓘ `lease_id/lease_until` 仍清空 —— 不清的话 `renew_lease`（`:173`）
    /// 会把已死进程的租约续活，那才是真正的泄漏。
    /// ⛔ `error` 仍是**硬编码覆盖**（原值丢失）：这是既有性质，本笔不改，
    ///    但它是下一个可查的缺陷（真实原因被这句常量吃掉）。
    pub fn recover_stale_running(&self, now: &str) -> Result<usize, NtBotError> {
        self.mark_outcome_unknown(now, "lease expired (crash recovery)")
    }

    /// 把 `running` 且租约过期（或无租约）的行刷成 `outcome_unknown`。
    ///
    /// 这是「启动刷残留」的**唯一**入口：`bin/neobot.rs` 的 `cmd_doctor` 与
    /// `cmd_channel_serve` 各调一次（真·启动一次，不是每轮）。
    /// ⛔ 刻意**不挂** `lib.rs:76 open_store()` —— 那会让 `task list` 这类纯读命令
    /// 也触发一次 UPDATE，「启动一次」的语义名不副实。
    /// ⛔ 刻意**不复用** `nt_stale_guard.rs:43 recover_stale_best_effort` ——
    ///    实测它**零调用方**，是死代码；挂上去等于让死代码复活，那是「新增接线」
    ///    不是「复用」，得让属主知道。
    pub fn mark_outcome_unknown(&self, now: &str, note: &str) -> Result<usize, NtBotError> {
        let n = self.conn.execute(
            "UPDATE tasks SET status='outcome_unknown', lease_id=NULL, lease_until=NULL,
              error=?2, updated_at=?1
             WHERE status='running' AND (lease_until IS NULL OR lease_until < ?1)",
            params![now, note],
        )?;
        // 恢复事实必须可观测：每次真回收落一行 ledger
        // （status='outcome_unknown'，token/cost 全 0，`measured=false` 按「永不猜测」律）。
        // 失败不挡恢复（best-effort，与降级记账同口径）。
        if n > 0 {
            let _ = self.record_ledger(&LedgerEntry {
                id: uuid::Uuid::new_v4().to_string(),
                at: now.to_owned(),
                engine: "tasks-recovery".to_owned(),
                model: String::new(),
                actor: "system".to_owned(),
                purpose: "crash-recovery".to_owned(),
                in_tokens: 0,
                out_tokens: 0,
                cost_usd: 0.0,
                measured: false,
                status: "outcome_unknown".to_owned(),
                latency_ms: 0,
                error: Some(note.to_owned()),
                session_id: None,
                key_env: None,
            });
        }
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
        // A1 语义变更（2026-10-02）：租约过期 → **`outcome_unknown`**，不再 pending。
        //    理由：崩溃窗口里可能「外部副作用已落地、结果未落库」，
        //    打回 pending 会让重跑**二次执行**该副作用。
        let recovered = store
            .recover_stale_running("2026-09-24T01:00:00Z")
            .expect("recover");
        assert_eq!(recovered, 1);
        let got = store.get_task("a").expect("get").expect("exists");
        assert_eq!(
            got.status,
            TaskStatus::OutcomeUnknown,
            "租约过期的任务必须落 outcome_unknown（不可自动重试），不能是 Pending"
        );
        assert!(got.lease_id.is_none());
        assert!(got.error.is_some());
        // A1：`outcome_unknown` 是**终态** —— cancel 与 retry 皆不接它。
        //    这是有意的：「等人工裁决」意味着调度器不会碰它，但也不假装它被取消过。
        assert!(
            store.cancel_task("a").is_err(),
            "outcome_unknown 不该被 cancel 悄悄改成 Cancelled（那会丢掉『副作用未知』这个事实）"
        );
        assert!(
            store.retry_task("a").is_err(),
            "outcome_unknown 绝不可被 retry —— 那正是 A1 要防的二次执行"
        );
        // cancel / retry 的原有口径改用另一个 pending 任务验，避免与 A1 语义耦合
        let mut p2 = mk("b");
        p2.lease_until = None;
        store.save_task(&p2).expect("save pending");
        store.cancel_task("b").expect("cancel pending");
        assert_eq!(
            store.get_task("b").expect("get").expect("exists").status,
            TaskStatus::Cancelled
        );
        assert!(store.cancel_task("b").is_err());
        store.retry_task("b").expect("retry cancelled");
        assert_eq!(
            store.get_task("b").expect("get").expect("exists").status,
            TaskStatus::Pending
        );
        assert!(store.retry_task("b").is_err());
    }

    /// A1 的核心回归：租约过期**不再**自动重排队。
    ///
    /// ⛔ 这条测试的存在理由是「防止有人把 `mark_outcome_unknown` 改回 `pending`」——
    /// 那不会让任何现有测试变红（它们断言的是 lease/error，不是终态），
    /// 但会把二次执行的 bug 原样放回来。
    #[test]
    fn 租约过期不自动重排队而是落终态() {
        let store = NeobotStore::open(":memory:").expect("open");
        let task = |id: &str, status: TaskStatus| AgentTask {
            id: id.to_owned(),
            title: id.to_owned(),
            status,
            created_at: "2026-10-02T00:00:00Z".to_owned(),
            updated_at: "2026-10-02T00:00:00Z".to_owned(),
            claimed_by: None,
            claimed_at: None,
            visibility: crate::nt_types::default_visibility(),
            lease_id: Some("lease".to_owned()),
            lease_until: Some("2026-10-02T00:00:00Z".to_owned()),
            attempts: 1,
            error: None,
            conversation_id: None,
        };
        store.save_task(&task("x", TaskStatus::Running)).expect("save running");
        let n = store
            .mark_outcome_unknown("2026-10-02T01:00:00Z", "test note")
            .expect("sweep");
        assert_eq!(n, 1);
        let got = store.get_task("x").expect("get").expect("exists");
        assert_eq!(got.status, TaskStatus::OutcomeUnknown);
        assert_eq!(got.error.as_deref(), Some("test note"), "note 应可自定义");
        // 幂等：第二次扫不应再命中（已不是 running）
        assert_eq!(
            store
                .mark_outcome_unknown("2026-10-02T02:00:00Z", "again")
                .expect("sweep2"),
            0,
            "已落 outcome_unknown 的行不该被再次扫中"
        );
        // 读回不能丢行：`parse` 漏了 "outcome_unknown" 会让 list_* 静默丢行。
        let all = store.list_tasks(10).expect("list");
        assert!(
            all.iter().any(|t| t.id == "x"),
            "outcome_unknown 的任务必须在 list_tasks 里可见（parse 漏分支会静默丢行）"
        );
    }


    #[test]
    // 2026-10-08: 中文测试名违反 Rust 标识符语法（空格）。
    fn mark_outcome_unknown_lands_in_ledger() {
        let store = NeobotStore::open(":memory:").expect("open");
        let task = AgentTask {
            id: "r1".to_owned(),
            title: "old".to_owned(),
            status: TaskStatus::Running,
            created_at: "2026-10-02T00:00:00Z".to_owned(),
            updated_at: "2026-10-02T00:00:00Z".to_owned(),
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
        assert_eq!(
            store
                .mark_outcome_unknown("2026-10-08T00:00:00Z", "crash while running")
                .expect("sweep"),
            1
        );
        // 恢复事实必须在 ledger 上可观测（永不为空）。
        assert_eq!(
            store
                .ledger_count_by_status("outcome_unknown")
                .expect("count"),
            1,
            "每次真回收都要落一行 outcome_unknown 账本"
        );
        assert_eq!(
            store.ledger_count_by_status("ok").expect("count2"),
            0,
            "恢复事件不能算正常调用"
        );
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

    /// 续租的**有界**与**归属**两律。
    ///
    /// 钉三件事：① 只有持钥匙者能续；② 窗口是**滚动**的（不是累加，
    /// 否则续 N 次就把崩溃恢复窗口放大成 N 倍）；③ 落终态后不许被
    /// 心跳拉回「活着」。
    #[test]
    fn lease_renewal_is_bounded_and_owner_only() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        let task = AgentTask {
            id: "r1".to_owned(),
            title: "r".to_owned(),
            status: TaskStatus::Running,
            created_at: "2026-09-24T00:00:00Z".to_owned(),
            updated_at: "2026-09-24T00:00:00Z".to_owned(),
            claimed_by: None,
            claimed_at: None,
            visibility: crate::nt_types::default_visibility(),
            lease_id: Some("lease-a".to_owned()),
            lease_until: Some("2026-09-24T00:10:00Z".to_owned()),
            attempts: 1,
            error: None,
            conversation_id: None,
        };
        store.save_task(&task).expect("save");
        // 别人的钥匙续不动（0 行 = 不再持有）。
        assert_eq!(
            store
                .renew_lease("r1", "lease-b", "2026-09-24T00:20:00Z")
                .expect("renew"),
            0
        );
        // 自己的钥匙能续，且窗口由调用方给（滚动：now + LEASE_SECS）。
        assert_eq!(
            store
                .renew_lease("r1", "lease-a", "2026-09-24T00:20:00Z")
                .expect("renew"),
            1
        );
        let got = store.get_task("r1").expect("get").expect("exists");
        assert_eq!(got.lease_until.as_deref(), Some("2026-09-24T00:20:00Z"));
        assert_eq!(got.lease_id.as_deref(), Some("lease-a"), "钥匙不许被换掉");
        // 续租**不改**状态：它只管「还活着吗」，不管「活成什么样」。
        assert_eq!(got.status, TaskStatus::Running);
        // 落终态后心跳失效：否则已完成的轮次会被拉回 running。
        let done = AgentTask {
            status: TaskStatus::Done,
            lease_until: Some("2026-09-24T00:30:00Z".to_owned()),
            ..task.clone()
        };
        store.save_task(&done).expect("save done");
        assert_eq!(
            store
                .renew_lease("r1", "lease-a", "2026-09-24T00:40:00Z")
                .expect("renew"),
            0
        );
        // 续租过的行仍可被租约回收（进程不再持有 → 窗口一过就回收）。
        let running = AgentTask {
            status: TaskStatus::Running,
            ..task
        };
        store.save_task(&running).expect("save running");
        assert_eq!(
            store
                .renew_lease("r1", "lease-a", "2026-09-24T00:50:00Z")
                .expect("renew"),
            1
        );
        assert_eq!(
            store
                .recover_stale_running("2026-09-24T00:45:00Z")
                .expect("recover"),
            0,
            "窗口未到不该动"
        );
        assert_eq!(
            store
                .recover_stale_running("2026-09-24T01:05:00Z")
                .expect("recover"),
            1,
            "进程不再持有 → 窗口一过就回收，不泄漏行"
        );
    }

    /// `last_step` 把 `ok` 一起带出来（取消落库要自证 `ok=0`）。
    #[test]
    fn last_step_reports_the_ok_flag() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        store
            .add_step("t1", 0, "cancelled:tool_calls", false, "web_search")
            .expect("step");
        let got = store
            .last_step("t1", "cancelled:tool_calls")
            .expect("last")
            .expect("row exists");
        assert!(!got.ok, "取消汇总行的 ok 必须是 0");
        assert_eq!(got.output, "web_search");
        assert!(
            store.last_step("t1", "nope").expect("last").is_none(),
            "无匹配行即 None，不编"
        );
    }
}
