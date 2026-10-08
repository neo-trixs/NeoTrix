//! `nt_store_routes` — 路由组注册表（`route_groups` 表 1:1）。

use super::NeobotStore;
use crate::nt_error::NtBotError;
use crate::nt_routing::{RouteGroup, RouteMode};
use rusqlite::{OptionalExtension, params};

impl NeobotStore {
    pub fn upsert_route_group(&self, group: &RouteGroup) -> Result<(), NtBotError> {
        let name = group.name.trim();
        if name.is_empty() {
            return Err(NtBotError::Invalid("route group name is empty".to_owned()));
        }
        if group.members.is_empty() {
            return Err(NtBotError::Invalid(format!(
                "route group '{name}' must have at least one member"
            )));
        }
        RouteMode::parse(&group.mode)?; // 非法模式入库即拒
        let now = chrono::Utc::now().to_rfc3339();
        let members = serde_json::to_string(&group.members)
            .map_err(|e| NtBotError::Invalid(format!("members json: {e}")))?;
        self.conn.execute(
            "INSERT INTO route_groups(name,members,mode,enabled,created_at)
             VALUES(?1,?2,?3,?4,?5)
             ON CONFLICT(name) DO UPDATE SET members=excluded.members,
               mode=excluded.mode, enabled=excluded.enabled",
            params![name, members, group.mode.trim(), i64::from(group.enabled), now],
        )?;
        Ok(())
    }

    pub fn get_route_group(&self, name: &str) -> Result<Option<RouteGroup>, NtBotError> {
        Ok(self.list_route_groups()?.into_iter().find(|g| g.name == name.trim()))
    }

    pub fn list_route_groups(&self) -> Result<Vec<RouteGroup>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT name,members,mode,enabled FROM route_groups ORDER BY name",
        )?;
        let rows = stmt.query_map([], |r| {
            let members_raw: String = r.get(1)?;
            let members: Vec<String> =
                serde_json::from_str(&members_raw).unwrap_or_default();
            Ok(RouteGroup {
                name: r.get(0)?,
                members,
                mode: r.get(2)?,
                enabled: r.get::<_, i64>(3)? != 0,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn set_route_group_enabled(&self, name: &str, enabled: bool) -> Result<(), NtBotError> {
        let n = self.conn.execute(
            "UPDATE route_groups SET enabled=?1 WHERE name=?2",
            params![i64::from(enabled), name.trim()],
        )?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such route group '{name}'")));
        }
        Ok(())
    }

    pub fn remove_route_group(&self, name: &str) -> Result<(), NtBotError> {
        let n = self
            .conn
            .execute("DELETE FROM route_groups WHERE name=?1", params![name.trim()])?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such route group '{name}'")));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::nt_store::NeobotStore;
    use crate::nt_routing::RouteGroup;

    #[test]
    fn route_group_roundtrip() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        let group = RouteGroup {
            name: "daily".to_owned(),
            members: vec!["deepseek".to_owned(), "ollama".to_owned()],
            mode: "order".to_owned(),
            enabled: true,
        };
        store.upsert_route_group(&group).expect("upsert");
        let got = store.get_route_group("daily").expect("get").expect("some");
        assert_eq!(got.members, ["deepseek", "ollama"]);
        assert_eq!(got.mode, "order");
        // 非法模式拒绝入库
        let mut bad = group.clone();
        bad.mode = "smart".to_owned();
        assert!(store.upsert_route_group(&bad).is_err());
        // 空成员拒绝入库
        let mut empty = group.clone();
        empty.members = vec![];
        assert!(store.upsert_route_group(&empty).is_err());
        store.set_route_group_enabled("daily", false).expect("off");
        assert!(!store.get_route_group("daily").expect("get").expect("some").enabled);
        store.remove_route_group("daily").expect("remove");
        assert!(store.get_route_group("daily").expect("get").is_none());
    }
}
