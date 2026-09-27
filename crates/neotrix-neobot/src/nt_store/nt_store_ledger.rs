//! `nt_store_ledger` — 审计 append-only + 调用账本 + outbox。

use super::{LedgerEntry, NeobotStore};
use crate::nt_audit::{AuditDecision, AuditEvent};
use crate::nt_error::NtBotError;
use rusqlite::params;

impl NeobotStore {
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

    // ---- ledger（调用账本：purpose + status + measured） ----

    /// 记账一行。`purpose` 为业务用途（默认 agent-turn）；`measured=false`
    /// 表示用量缺失、费用未估（“永不猜测”律：此时 cost_usd 强制为 0）。
    pub fn record_ledger(&self, entry: &LedgerEntry) -> Result<(), NtBotError> {
        let cost = if entry.measured {
            entry.cost_usd.max(0.0)
        } else {
            0.0
        };
        self.conn.execute(
            "INSERT INTO ledger(id,at,engine,model,actor,purpose,in_tokens,out_tokens,
              cost_usd,measured,status,latency_ms,error)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            params![
                entry.id,
                entry.at,
                entry.engine,
                entry.model,
                entry.actor,
                entry.purpose,
                entry.in_tokens.max(0),
                entry.out_tokens.max(0),
                cost,
                i64::from(entry.measured),
                entry.status,
                entry.latency_ms.max(0),
                entry.error
            ],
        )?;
        Ok(())
    }

    // ---- outbox（实时 outbox：attempts + available_at 退避） ----

    pub fn enqueue_outbox(&self, id: &str, topic: &str, payload: &str) -> Result<(), NtBotError> {
        self.conn.execute(
            "INSERT INTO outbox(id,topic,payload,claimed) VALUES(?1,?2,?3,0)",
            params![id, topic, payload],
        )?;
        Ok(())
    }

    /// 取出到期未领行（available_at<=now），顺手认领、attempts 加一并
    /// 把 available_at 记为 now（认领时刻；`prune_outbox` 凭它删已交付旧行）。
    /// 调用方投递失败时调 `fail_outbox` 推迟重试（指数退避由调用方算）。
    pub fn drain_outbox(
        &self,
        limit: i64,
        now: &str,
    ) -> Result<Vec<(String, String, String)>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT id,topic,payload FROM outbox
             WHERE claimed=0 AND available_at <= ?1 ORDER BY available_at LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![now, limit.max(1)], |r| {
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
            self.conn.execute(
                "UPDATE outbox SET claimed=1, attempts=attempts+1, available_at=?1 WHERE id=?2",
                params![now, id],
            )?;
        }
        Ok(out)
    }

    /// 投递失败：放回队列并推迟到 retry_at（RFC3339），attempts 保留供观测。
    pub fn fail_outbox(&self, id: &str, retry_at: &str) -> Result<(), NtBotError> {
        self.conn.execute(
            "UPDATE outbox SET claimed=0, available_at=?1 WHERE id=?2",
            params![retry_at, id],
        )?;
        Ok(())
    }

    /// 清理已交付旧行：删 claimed=1 且认领时刻早于 cutoff 的（纯时间水位）。
    pub fn prune_outbox(&self, cutoff: &str) -> Result<usize, NtBotError> {
        let n = self.conn.execute(
            "DELETE FROM outbox WHERE claimed=1 AND available_at <= ?1",
            params![cutoff],
        )?;
        Ok(n)
    }

    /// 审计留存清扫：删早于 cutoff 的行。
    pub fn prune_audit(&self, cutoff: &str) -> Result<usize, NtBotError> {
        let n = self.conn.execute("DELETE FROM audit WHERE at < ?1", params![cutoff])?;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use crate::nt_audit::{AuditDecision, AuditEvent};
    use crate::nt_store::NeobotStore;

    #[test]
    fn outbox_retry_and_prune() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        let now = "2026-09-24T00:00:00Z";
        store.enqueue_outbox("o1", "T", "{}").expect("enqueue");
        // 推迟到未来 → drain 为空；到期后可见
        store
            .fail_outbox("o1", "2026-09-24T01:00:00Z")
            .expect("fail");
        assert!(store.drain_outbox(10, now).expect("drain").is_empty());
        assert_eq!(
            store
                .drain_outbox(10, "2026-09-24T02:00:00Z")
                .expect("drain")
                .len(),
            1
        );
        // 已交付（认领时刻 02:00）+ 早于次日水位 → prune 删
        assert_eq!(store.prune_outbox("2026-09-25T00:00:00Z").expect("prune"), 1);
    }


    #[test]
    fn ledger_actor_sums() {
        use super::LedgerEntry;
        let store = NeobotStore::open(":memory:").expect("open memory db");
        let at = "2026-09-24T00:00:00Z";
        for (id, actor, cost) in [("l1", "alice", 0.01), ("l2", "bot", 0.0)] {
            store
                .record_ledger(&LedgerEntry {
                    id: id.to_owned(),
                    at: at.to_owned(),
                    engine: "http".to_owned(),
                    model: "m".to_owned(),
                    actor: actor.to_owned(),
                    purpose: "agent-turn".to_owned(),
                    in_tokens: 10,
                    out_tokens: 5,
                    cost_usd: cost,
                    measured: true,
                    status: "ok".to_owned(),
                    latency_ms: 12,
                    error: None,
                })
                .expect("record");
        }
        // 未计量行费用归零（永不猜测律）
        store
            .record_ledger(&LedgerEntry {
                id: "l3".to_owned(),
                at: at.to_owned(),
                engine: "http".to_owned(),
                model: "m".to_owned(),
                actor: "bot".to_owned(),
                purpose: "agent-turn".to_owned(),
                in_tokens: 10,
                out_tokens: 5,
                cost_usd: 9.99,
                measured: false,
                status: "ok".to_owned(),
                latency_ms: 1,
                error: None,
            })
            .expect("record");
        let sums = store.ledger_sums().expect("sums");
        assert_eq!(sums.len(), 1);
        let (_, _, _, _, cost) = sums.first().expect("one sum row");
        assert!((cost - 0.01).abs() < 1e-9);
        let by_actor = store.ledger_sums_by_actor().expect("by actor");
        assert_eq!(by_actor.len(), 2);
    }


    #[test]
    fn audit_append_only() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        let event = AuditEvent::new("bot", "bash", AuditDecision::Deny, Some("workspace-jail".to_owned()), "x");
        store.record_audit(&event).expect("record");
        assert_eq!(store.list_audit(10).expect("list").len(), 1);
    }
}
