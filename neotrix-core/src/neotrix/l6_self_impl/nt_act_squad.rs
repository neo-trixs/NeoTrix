//! L6 / NT-ACT — squad (github.com/mco-org/squad) 吸收节点 (C1)。
//!
//! 源: squad — 基于 SQLite 的多代理终端协作基础设施。多个 agent 通过共享 SQLite
//! 库交换消息/任务/结果, 实现终端内协作。NeoTrix 视角: SQLite-backed 协作总线
//! (消息表 + 任务表), `PRAGMA busy_timeout=5000` 防并发写锁。

use crate::core::nt_core_self_test::SelfTest;
use rusqlite::Connection;

/// 协作消息。
#[derive(Debug, Clone, PartialEq)]
pub struct CollabMessage {
    pub sender: String,
    pub body: String,
}

/// SQLite 多代理协作总线。
pub trait CollabBus {
    fn post(&self, msg: &CollabMessage) -> rusqlite::Result<()>;
    fn inbox(&self, agent: &str) -> rusqlite::Result<Vec<CollabMessage>>;
}

pub struct SquadBus {
    conn: Connection,
}

/// 打开 (或创建) SQLite 协作库: 设 busy_timeout 防并发写锁, 再建角色表 + 消息表。
pub fn open_db(path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA busy_timeout=5000;")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS agents (
            name TEXT PRIMARY KEY,
            role TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            recipient TEXT NOT NULL,
            sender TEXT NOT NULL,
            body TEXT NOT NULL
        );",
    )?;
    Ok(conn)
}

impl SquadBus {
    /// 用内存库初始化协作总线 (建表 + busy_timeout)。
    pub fn in_memory() -> rusqlite::Result<Self> {
        Ok(Self::open(open_db(":memory:")?))
    }

    /// 打开 (或创建) 文件型 SQLite 协作总线。
    pub fn open(conn: Connection) -> Self {
        Self { conn }
    }

    /// 持久化一个 agent 角色 (INSERT OR REPLACE)。
    pub fn register_agent(&self, name: &str, role: &str) -> rusqlite::Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO agents (name, role) VALUES (?1, ?2)",
                rusqlite::params![name, role],
            )?;
        Ok(())
    }

    /// 查询某 agent 的角色。
    pub fn role_of(&self, name: &str) -> rusqlite::Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT role FROM agents WHERE name = ?1")?;
        let mut rows = stmt.query_map([name], |r| r.get::<_, String>(0))?;
        match rows.next() {
            Some(Ok(role)) => Ok(Some(role)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

impl CollabBus for SquadBus {
    fn post(&self, msg: &CollabMessage) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO messages (recipient, sender, body) VALUES ('_broadcast', ?1, ?2)",
            rusqlite::params![msg.sender, msg.body],
        )?;
        Ok(())
    }

    fn inbox(&self, agent: &str) -> rusqlite::Result<Vec<CollabMessage>> {
        let mut stmt = self
            .conn
            .prepare("SELECT sender, body FROM messages WHERE recipient = ?1 OR recipient = '_broadcast'")?;
        let rows = stmt.query_map([agent], |r| {
            Ok(CollabMessage {
                sender: r.get(0)?,
                body: r.get(1)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
    }
}

#[derive(Default)]
pub struct SquadSelfTest;

impl SelfTest for SquadSelfTest {
    fn name(&self) -> &str {
        "nt_act_squad"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let bus = match SquadBus::in_memory() {
            Ok(b) => b,
            Err(e) => return Err(vec![format!("squad: init failed: {e}")]),
        };
        let mut errs = Vec::new();
        if let Err(e) = bus.post(&CollabMessage {
            sender: "a1".into(),
            body: "hello".into(),
        }) {
            errs.push(format!("squad: post failed: {e}"));
        }
        match bus.inbox("a2") {
            Ok(msgs) => {
                if msgs.len() != 1 {
                    errs.push(format!("squad: expected 1 broadcast message, got {}", msgs.len()));
                }
            }
            Err(e) => errs.push(format!("squad: inbox failed: {e}")),
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
    fn broadcast_visible_to_all() {
        let bus = SquadBus::in_memory().unwrap();
        bus.post(&CollabMessage {
            sender: "a1".into(),
            body: "x".into(),
        })
        .unwrap();
        assert_eq!(bus.inbox("anyone").unwrap().len(), 1);
    }

    #[test]
    fn empty_inbox_no_messages() {
        let bus = SquadBus::in_memory().unwrap();
        assert!(bus.inbox("lonely").unwrap().is_empty());
    }

    #[test]
    fn file_backed_open_db_roundtrip() {
        let path = std::env::temp_dir().join(format!("neotrix_squad_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        {
            let bus = SquadBus::open(open_db(path.to_str().unwrap()).unwrap());
            bus.register_agent("a1", "planner").unwrap();
            bus.post(&CollabMessage {
                sender: "a1".into(),
                body: "x".into(),
            })
            .unwrap();
            assert_eq!(bus.role_of("a1").unwrap(), Some("planner".into()));
            assert_eq!(bus.inbox("a2").unwrap().len(), 1);
        }
        {
            let bus = SquadBus::open(open_db(path.to_str().unwrap()).unwrap());
            assert_eq!(bus.role_of("a1").unwrap(), Some("planner".into()));
            assert_eq!(bus.inbox("a2").unwrap().len(), 1);
        }
        let _ = std::fs::remove_file(&path);
    }
}
