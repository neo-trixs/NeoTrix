//! nt_data_gateway — EVO-08 统一数据面网关（dbx 多后端抽象思想）。
//!
//! 后端种类＋端点校验＋内存注册表（上限 [`MAX_ENDPOINTS`]）＋DSN 脱敏＋
//! 查询权限（默认拒绝写）。只做类型与纯逻辑，不做真实连接 / IO。

use serde::{Deserialize, Serialize};

/// 注册表上限。
pub const MAX_ENDPOINTS: usize = 16;

/// 后端种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DbKind {
    Sqlite,
    Postgres,
    Mysql,
    Redis,
    Mongo,
    Duckdb,
}

/// 网关错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayError {
    EmptyHost,
    EmptyDb,
    InvalidPort,
    Duplicate,
    RegistryFull,
    WriteDenied,
}

/// 数据端点。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbEndpoint {
    pub kind: DbKind,
    pub host: String,
    pub port: u16,
    pub db_name: String,
}

impl DbEndpoint {
    pub fn validate(&self) -> Result<(), GatewayError> {
        if self.host.is_empty() {
            return Err(GatewayError::EmptyHost);
        }
        if self.db_name.is_empty() {
            return Err(GatewayError::EmptyDb);
        }
        if self.port == 0 {
            return Err(GatewayError::InvalidPort);
        }
        Ok(())
    }
}

/// 内存注册表。
#[derive(Debug, Default, Clone)]
pub struct GatewayRegistry {
    entries: Vec<(String, DbEndpoint)>,
}

impl GatewayRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 注册（非法端点拒绝；同名拒绝；超限拒绝）。
    pub fn register(&mut self, name: impl Into<String>, ep: DbEndpoint) -> Result<(), GatewayError> {
        let name = name.into();
        if name.is_empty() {
            return Err(GatewayError::Duplicate);
        }
        ep.validate()?;
        if self.entries.iter().any(|(n, _)| *n == name) {
            return Err(GatewayError::Duplicate);
        }
        if self.entries.len() >= MAX_ENDPOINTS {
            return Err(GatewayError::RegistryFull);
        }
        self.entries.push((name, ep));
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&DbEndpoint> {
        self.entries.iter().find(|(n, _)| n == name).map(|(_, e)| e)
    }

    pub fn list_names(&self) -> Vec<&str> {
        self.entries.iter().map(|(n, _)| n.as_str()).collect()
    }
}

/// DSN 脱敏：遮蔽 `user:pass@` 中的密码段（无密码原样返回；先剥 scheme 再找 userinfo）。
pub fn mask_dsn(dsn: &str) -> String {
    // 先剥 scheme（如 `postgres://`），避免 scheme 内的 `:` 被误判为 userinfo 分隔。
    let (scheme, rest) = match dsn.find("://") {
        Some(i) => dsn.split_at(i + 3),
        None => ("", dsn),
    };
    let at = rest.find('@');
    match at {
        None => dsn.to_owned(),
        Some(i) => {
            let (head, tail) = rest.split_at(i);
            match head.find(':') {
                None => dsn.to_owned(),
                Some(c) => {
                    let (user, _) = head.split_at(c);
                    format!("{scheme}{user}:***{tail}")
                }
            }
        }
    }
}

/// 查询权限（默认拒绝写）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub struct QueryPolicy {
    pub allow_write: bool,
}


impl QueryPolicy {
    pub fn check_write(&self) -> Result<(), GatewayError> {
        if self.allow_write {
            Ok(())
        } else {
            Err(GatewayError::WriteDenied)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ep() -> DbEndpoint {
        DbEndpoint {
            kind: DbKind::Sqlite,
            host: "127.0.0.1".to_owned(),
            port: 5432,
            db_name: "nt".to_owned(),
        }
    }

    #[test]
    fn validate_edges() {
        assert!(ep().validate().is_ok());
        let mut bad = ep();
        bad.host.clear();
        assert_eq!(bad.validate(), Err(GatewayError::EmptyHost));
        let mut bad = ep();
        bad.db_name.clear();
        assert_eq!(bad.validate(), Err(GatewayError::EmptyDb));
        let mut bad = ep();
        bad.port = 0;
        assert_eq!(bad.validate(), Err(GatewayError::InvalidPort));
    }

    #[test]
    fn registry_dup_and_full() {
        let mut r = GatewayRegistry::new();
        assert!(r.is_empty());
        assert!(r.register("a", ep()).is_ok());
        assert_eq!(r.register("a", ep()), Err(GatewayError::Duplicate));
        for i in 0..15 {
            r.register(format!("n{i}"), ep()).ok();
        }
        assert_eq!(r.len(), MAX_ENDPOINTS);
        assert_eq!(r.register("over", ep()), Err(GatewayError::RegistryFull));
        assert_eq!(r.get("a"), Some(&ep()));
    }

    #[test]
    fn mask_dsn_hides_password() {
        assert_eq!(mask_dsn("postgres://u:p@h:5432/db"), "postgres://u:***@h:5432/db");
        assert_eq!(mask_dsn("sqlite:///x.db"), "sqlite:///x.db");
        assert_eq!(mask_dsn("redis://h:6379"), "redis://h:6379");
    }

    #[test]
    fn write_denied_by_default() {
        assert_eq!(QueryPolicy::default().check_write(), Err(GatewayError::WriteDenied));
        assert!(QueryPolicy { allow_write: true }.check_write().is_ok());
    }
}
