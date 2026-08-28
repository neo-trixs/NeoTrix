//! L6 / NT-ACT — rustdesk (github.com/rustdesk/rustdesk) 吸收节点 (C1)。
//!
//! 源: rustdesk — 开源远程桌面, 跨平台会话控制 + 编排 API。NeoTrix 视角: 远程
//! 桌面会话编排器, 用 SQLite 记录会话/连接状态, `PRAGMA busy_timeout=5000` 防
//! 并发写。C1: 会话注册 + 状态查询 trait (stub, 无真实网络)。

use crate::core::nt_core_self_test::SelfTest;
use rusqlite::Connection;

/// 会话状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    Connected,
    Disconnected,
}

/// 远程桌面会话。
#[derive(Debug, Clone)]
pub struct RemoteSession {
    pub id: String,
    pub peer: String,
    pub state: SessionState,
}

/// 会话编排 API。
pub trait SessionOrchestrator {
    fn register(&self, s: &RemoteSession) -> rusqlite::Result<()>;
    fn state_of(&self, id: &str) -> rusqlite::Result<Option<SessionState>>;
}

pub struct RustdeskOrchestrator {
    conn: Connection,
}

/// 打开 (或创建) SQLite 会话库: 设 busy_timeout 防并发写 + 建 sessions 表。
pub fn open_db(path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA busy_timeout=5000;")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            peer TEXT NOT NULL,
            state TEXT NOT NULL
        );",
    )?;
    Ok(conn)
}

impl RustdeskOrchestrator {
    pub fn in_memory() -> rusqlite::Result<Self> {
        Ok(Self::open(open_db(":memory:")?))
    }

    /// 打开 (或创建) 文件型 SQLite 会话库。
    pub fn open(conn: Connection) -> Self {
        Self { conn }
    }
}

impl SessionOrchestrator for RustdeskOrchestrator {
    fn register(&self, s: &RemoteSession) -> rusqlite::Result<()> {
        let st = match s.state {
            SessionState::Connected => "connected",
            SessionState::Disconnected => "disconnected",
        };
        self.conn.execute(
            "INSERT OR REPLACE INTO sessions (id, peer, state) VALUES (?1, ?2, ?3)",
            rusqlite::params![s.id, s.peer, st],
        )?;
        Ok(())
    }

    fn state_of(&self, id: &str) -> rusqlite::Result<Option<SessionState>> {
        let mut stmt = self.conn.prepare("SELECT state FROM sessions WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |r| {
            let s: String = r.get(0)?;
            Ok(if s == "connected" {
                SessionState::Connected
            } else {
                SessionState::Disconnected
            })
        })?;
        match rows.next() {
            Some(Ok(st)) => Ok(Some(st)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

#[derive(Default)]
pub struct RustdeskSelfTest;

impl SelfTest for RustdeskSelfTest {
    fn name(&self) -> &str {
        "nt_act_rustdesk"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let o = match RustdeskOrchestrator::in_memory() {
            Ok(o) => o,
            Err(e) => return Err(vec![format!("rustdesk: init failed: {e}")]),
        };
        let mut errs = Vec::new();
        let s = RemoteSession {
            id: "s1".into(),
            peer: "peer-a".into(),
            state: SessionState::Connected,
        };
        if let Err(e) = o.register(&s) {
            errs.push(format!("rustdesk: register failed: {e}"));
        }
        match o.state_of("s1") {
            Ok(Some(SessionState::Connected)) => {}
            Ok(other) => errs.push(format!("rustdesk: unexpected state {:?}", other)),
            Err(e) => errs.push(format!("rustdesk: state_of failed: {e}")),
        }
        if let Ok(Some(_)) = o.state_of("missing") {
            errs.push("rustdesk: missing session must return None".into());
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_query() {
        let o = RustdeskOrchestrator::in_memory().unwrap();
        o.register(&RemoteSession {
            id: "x".into(),
            peer: "p".into(),
            state: SessionState::Connected,
        })
        .unwrap();
        assert_eq!(o.state_of("x").unwrap(), Some(SessionState::Connected));
    }

    #[test]
    fn missing_returns_none() {
        let o = RustdeskOrchestrator::in_memory().unwrap();
        assert_eq!(o.state_of("nope").unwrap(), None);
    }

    #[test]
    fn replace_updates_state() {
        let o = RustdeskOrchestrator::in_memory().unwrap();
        o.register(&RemoteSession {
            id: "x".into(),
            peer: "p".into(),
            state: SessionState::Connected,
        })
        .unwrap();
        o.register(&RemoteSession {
            id: "x".into(),
            peer: "p".into(),
            state: SessionState::Disconnected,
        })
        .unwrap();
        assert_eq!(o.state_of("x").unwrap(), Some(SessionState::Disconnected));
    }

    #[test]
    fn file_backed_open_db_roundtrip() {
        let path = std::env::temp_dir().join(format!("neotrix_rustdesk_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        {
            let o = RustdeskOrchestrator::open(open_db(path.to_str().unwrap()).unwrap());
            o.register(&RemoteSession {
                id: "s1".into(),
                peer: "peer-a".into(),
                state: SessionState::Connected,
            })
            .unwrap();
        }
        {
            let o = RustdeskOrchestrator::open(open_db(path.to_str().unwrap()).unwrap());
            assert_eq!(o.state_of("s1").unwrap(), Some(SessionState::Connected));
        }
        let _ = std::fs::remove_file(&path);
    }
}
