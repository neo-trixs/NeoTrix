//! `nt_store_upkeep` — 接管（take-the-wheel）+ 留存清扫。

use super::NeobotStore;
use crate::nt_error::NtBotError;
use rusqlite::{OptionalExtension, params};
use crate::nt_audit::{AuditDecision, AuditEvent};

impl NeobotStore {
    // ---- control（take-the-wheel：具名接管 + 交接审计） ----

    /// 接管：无 holder 或 holder 即本人 → 成功（幂等）；他人持有 → 具名拒绝。
    /// 交接事件写审计（记名、可查）。
    pub fn take_control(&self, holder: &str) -> Result<(), NtBotError> {
        if holder.trim().is_empty() {
            return Err(NtBotError::Invalid("control holder is empty".to_owned()));
        }
        let now = chrono::Utc::now().to_rfc3339();
        let current = self.control_holder()?;
        if let Some(name) = current {
            if name != holder {
                return Err(NtBotError::Store(format!("control held by '{name}'")));
            }
            return Ok(());
        }
        self.conn.execute(
            "INSERT INTO control(id,holder,updated_at) VALUES(1,?1,?2)",
            params![holder.trim(), now],
        )?;
        let event = AuditEvent::new(
            holder.trim(),
            "control",
            AuditDecision::Allow,
            Some("take-the-wheel".to_owned()),
            &format!("control taken by {holder}"),
        );
        self.record_audit(&event)?;
        Ok(())
    }

    /// 交棒：holder 本人可放；他人持有拒绝（陌生人不能替人交棒）。
    pub fn release_control(&self, holder: &str) -> Result<(), NtBotError> {
        let current = self.control_holder()?;
        match current {
            None => Ok(()),
            Some(name) if name == holder => {
                self.conn.execute("DELETE FROM control WHERE id=1", [])?;
                let event = AuditEvent::new(
                    holder,
                    "control",
                    AuditDecision::Allow,
                    Some("take-the-wheel".to_owned()),
                    &format!("control released by {holder}"),
                );
                self.record_audit(&event)?;
                Ok(())
            }
            Some(name) => Err(NtBotError::Store(format!("control held by '{name}'"))),
        }
    }

    pub fn control_holder(&self) -> Result<Option<String>, NtBotError> {
        let holder: Option<String> = self
            .conn
            .query_row("SELECT holder FROM control WHERE id=1", [], |r| r.get(0))
            .optional()?;
        Ok(holder)
    }
}
