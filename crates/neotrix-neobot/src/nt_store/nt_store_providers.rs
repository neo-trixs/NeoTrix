//! `nt_store_providers` — 第三方端点注册表 + 晶体配对单行表。

use super::{CorePair, NeobotStore};
use crate::nt_error::NtBotError;
use rusqlite::{OptionalExtension, params};

impl NeobotStore {
    // ---- providers（第三方端点注册表；密钥只存变量名） ----

    pub fn upsert_provider(&self, provider: &crate::nt_provider::Provider) -> Result<(), NtBotError> {
        provider.validate()?;
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO providers(name,base_url,key_env,model,enabled,created_at)
             VALUES(?1,?2,?3,?4,?5,?6)
             ON CONFLICT(name) DO UPDATE SET base_url=excluded.base_url,
               key_env=excluded.key_env, model=excluded.model, enabled=excluded.enabled",
            params![
                provider.name.trim(),
                provider.base_url.trim(),
                provider.key_env.trim(),
                provider.model.trim(),
                i64::from(provider.enabled),
                now
            ],
        )?;
        Ok(())
    }

    pub fn remove_provider(&self, name: &str) -> Result<(), NtBotError> {
        let n = self
            .conn
            .execute("DELETE FROM providers WHERE name=?1", params![name.trim()])?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such provider '{name}'")));
        }
        Ok(())
    }

    // ---- core_pair（晶体核心配对：单行；配对即灵魂嵌入，删除即摘除） ----

    /// 读配对（None = 未嵌入，纯本地 App）。
    pub fn get_core_pair(&self) -> Result<Option<CorePair>, NtBotError> {
        self.conn
            .query_row(
                "SELECT base_url,model,paired_at,last_ok_at,last_latency_ms,
                   COALESCE(via,'http'),COALESCE(token_env,'CRYSTAL_TOKEN') FROM core_pair WHERE id=1",
                [],
                |r| {
                    Ok(CorePair {
                        base_url: r.get(0)?,
                        model: r.get(1)?,
                        paired_at: r.get(2)?,
                        last_ok_at: r.get(3)?,
                        last_latency_ms: r.get(4)?,
                        via: r.get(5)?,
                        token_env: r.get(6)?,
                    })
                },
            )
            .optional()?
            .map(Ok)
            .transpose()
    }

    /// 写配对（幂等覆盖；调用方须先探活，死端点不配对）。
    /// `token_env` 只存变量名（空即缺省 `CRYSTAL_TOKEN`），永不存 token 值。
    pub fn set_core_pair(
        &self,
        base_url: &str,
        model: &str,
        latency_ms: i64,
        via: &str,
        token_env: &str,
    ) -> Result<CorePair, NtBotError> {
        let now = chrono::Utc::now().to_rfc3339();
        let token_name = {
            let t = token_env.trim();
            if t.is_empty() {
                "CRYSTAL_TOKEN".to_owned()
            } else {
                t.to_owned()
            }
        };
        self.conn.execute(
            "INSERT INTO core_pair(id,base_url,model,paired_at,last_ok_at,last_latency_ms,via,token_env)
             VALUES(1,?1,?2,?3,?3,?4,?5,?6)
             ON CONFLICT(id) DO UPDATE SET base_url=excluded.base_url,
               model=excluded.model, paired_at=excluded.paired_at,
               last_ok_at=excluded.last_ok_at, last_latency_ms=excluded.last_latency_ms,
               via=excluded.via, token_env=excluded.token_env",
            params![base_url, model, now, latency_ms, via, token_name],
        )?;
        self.get_core_pair()?
            .ok_or_else(|| NtBotError::Store("core_pair write lost".to_owned()))
    }

    /// 摘除配对（灵魂摘除；provider 记录保留，池子自动跳过死端点）。
    pub fn clear_core_pair(&self) -> Result<bool, NtBotError> {
        let n = self.conn.execute("DELETE FROM core_pair WHERE id=1", [])?;
        Ok(n > 0)
    }

    pub fn set_provider_enabled(&self, name: &str, enabled: bool) -> Result<(), NtBotError> {
        let n = self.conn.execute(
            "UPDATE providers SET enabled=?1 WHERE name=?2",
            params![i64::from(enabled), name.trim()],
        )?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such provider '{name}'")));
        }
        Ok(())
    }

    pub fn list_providers(&self) -> Result<Vec<crate::nt_provider::Provider>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT name,base_url,key_env,model,enabled FROM providers ORDER BY name",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(crate::nt_provider::Provider {
                name: r.get(0)?,
                base_url: r.get(1)?,
                key_env: r.get(2)?,
                model: r.get(3)?,
                enabled: r.get::<_, i64>(4)? != 0,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn get_provider(&self, name: &str) -> Result<Option<crate::nt_provider::Provider>, NtBotError> {
        Ok(self
            .list_providers()?
            .into_iter()
            .find(|p| p.name == name.trim()))
    }
}

#[cfg(test)]
mod tests {
    use crate::nt_store::NeobotStore;

    #[test]
    fn provider_registry_roundtrip() {
        use crate::nt_provider::Provider;
        let store = NeobotStore::open(":memory:").expect("open memory db");
        // migrate 自举 neotrix 内置端点（不可达则列表跳过，不影响本地）。
        let seeded = store.list_providers().expect("list");
        assert_eq!(seeded.len(), 1);
        assert_eq!(seeded[0].name, "neotrix");
        let mk = |name: &str| Provider {
            name: name.to_owned(),
            base_url: "http://127.0.0.1:11434/v1".to_owned(),
            key_env: String::new(),
            model: "qwen".to_owned(),
            enabled: true,
        };
        assert!(store.upsert_provider(&mk("")).is_err());
        store.upsert_provider(&mk("ollama")).expect("add");
        store.set_provider_enabled("ollama", false).expect("off");
        assert!(store.set_provider_enabled("nope", true).is_err());
        let list = store.list_providers().expect("list");
        assert_eq!(list.len(), 2);
        let ollama = list.iter().find(|p| p.name == "ollama").expect("ollama row");
        assert!(!ollama.enabled);
        assert!(store.get_provider("ollama").expect("get").is_some());
        assert!(store.remove_provider("nope").is_err());
        store.remove_provider("ollama").expect("remove");
        // 删完自建后只剩自举的 neotrix
        let rest = store.list_providers().expect("list");
        assert_eq!(rest.len(), 1);
        assert_eq!(rest[0].name, "neotrix");
    }

}
