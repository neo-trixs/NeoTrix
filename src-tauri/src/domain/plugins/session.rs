use crate::domain::{ActionSpec, DomainError, DomainPlugin, ParamSpec};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ========== Types ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub message_count: usize,
    pub created_at: i64,
    pub updated_at: i64,
    pub project: String,
    pub sort_order: i64,
}

// ========== Plugin ==========

pub struct SessionPlugin {
    db_pool: Arc<crate::db_pool::DbPool>,
}

impl SessionPlugin {
    pub fn new(db_pool: Arc<crate::db_pool::DbPool>) -> Self {
        if let Err(e) = db_pool.init_schema(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                messages TEXT NOT NULL DEFAULT '[]',
                project TEXT NOT NULL DEFAULT '',
                sort_order INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS app_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        ) {
            tracing::warn!("schema init: {e}");
        }
        // backward-compat columns
        if let Err(e) =
            db_pool.init_schema("ALTER TABLE sessions ADD COLUMN project TEXT NOT NULL DEFAULT ''")
        {
            tracing::warn!("schema init: {e}");
        }
        if let Err(e) = db_pool
            .init_schema("ALTER TABLE sessions ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0")
        {
            tracing::warn!("schema init: {e}");
        }
        Self { db_pool }
    }

    fn list_all(&self) -> Result<Vec<SessionInfo>, DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for list_all")))?;
        let mut stmt = conn
            .prepare("SELECT id, name, created_at, updated_at, messages, project, sort_order FROM sessions ORDER BY sort_order ASC, updated_at DESC")
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("preparing list_all query")))?;
        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let created: i64 = row.get(2)?;
                let updated: i64 = row.get(3)?;
                let messages: String = row.get(4)?;
                let project: String = row.get(5)?;
                let sort_order: i64 = row.get(6)?;
                let message_count = serde_json::from_str::<serde_json::Value>(&messages)
                    .map(|v| v.as_array().map(|a| a.len()).unwrap_or(0))
                    .unwrap_or(0);
                Ok(SessionInfo {
                    id,
                    name,
                    message_count,
                    created_at: created,
                    updated_at: updated,
                    project,
                    sort_order,
                })
            })
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("executing list_all query")))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| DomainError::from(anyhow::Error::from(e).context("parsing session row in list_all")))?);
        }
        Ok(out)
    }

    fn create(&self, name: &str) -> Result<String, DomainError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for create")))?;
        conn.execute(
            "INSERT INTO sessions (id, name, created_at, updated_at, messages) VALUES (?1, ?2, ?3, ?3, '[]')",
            rusqlite::params![id, name, now],
        )
        .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("inserting new session '{}'", name))))?;
        Ok(id)
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for delete")))?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("deleting session {}", id))))?;
        if let Err(e) = conn.execute(
            "DELETE FROM app_state WHERE key = 'active_session_id' AND value = ?1",
            rusqlite::params![id],
        ) {
            tracing::warn!("db: {e}");
        }
        Ok(())
    }

    fn switch_to(&self, id: &str) -> Result<(), DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for switch_to")))?;
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sessions WHERE id = ?1)",
                rusqlite::params![id],
                |r| r.get(0),
            )
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("checking existence of session {}", id))))?;
        if !exists {
            return Err(DomainError {
                code: "NOT_FOUND".into(),
                message: format!("Session not found: {}", id),
                recoverable: true,
            });
        }
        conn.execute(
            "INSERT INTO app_state (key, value) VALUES ('active_session_id', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![id],
        )
        .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("switching to session {}", id))))?;
        Ok(())
    }

    fn reorder(&self, ids: &[String]) -> Result<(), DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for reorder")))?;
        let tx = conn.unchecked_transaction()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("starting reorder transaction")))?;
        for (i, id) in ids.iter().enumerate() {
            tx.execute(
                "UPDATE sessions SET sort_order = ?1 WHERE id = ?2",
                rusqlite::params![i as i64, id],
            )
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("updating sort_order for session {}", id))))?;
        }
        tx.commit()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("committing reorder transaction")))?;
        Ok(())
    }

    fn set_project(&self, id: &str, project: &str) -> Result<(), DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for set_project")))?;
        let n = conn
            .execute(
                "UPDATE sessions SET project = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![project, chrono::Utc::now().timestamp(), id],
            )
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("setting project for session {}", id))))?;
        if n == 0 {
            return Err(DomainError {
                code: "NOT_FOUND".into(),
                message: format!("Session not found: {}", id),
                recoverable: true,
            });
        }
        Ok(())
    }

    fn fork(&self, from_id: &str, up_to: Option<usize>) -> Result<String, DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for fork")))?;
        let messages: String = conn
            .query_row(
                "SELECT messages FROM sessions WHERE id = ?1",
                rusqlite::params![from_id],
                |r| r.get::<_, String>(0),
            )
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("reading source session {} for fork", from_id))))?;
        let sliced = up_to
            .and_then(|n| {
                serde_json::from_str::<serde_json::Value>(&messages)
                    .ok()
                    .and_then(|v| {
                        v.as_array()
                            .map(|a| a.iter().take(n).cloned().collect::<Vec<_>>())
                    })
                    .and_then(|a| serde_json::to_string(&a).ok())
            })
            .unwrap_or(messages);
        let new_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO sessions (id, name, created_at, updated_at, messages, project, sort_order) VALUES (?1, ?2, ?3, ?3, ?4, '', 0)",
            rusqlite::params![new_id, "分支会话", now, sliced],
        )
        .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("inserting forked session from {}", from_id))))?;
        Ok(new_id)
    }

    fn search(&self, query: &str) -> Result<Vec<SessionInfo>, DomainError> {
        let all = self.list_all()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("listing sessions for search")))?;
        let q = query.to_lowercase();
        Ok(all
            .into_iter()
            .filter(|s| s.name.to_lowercase().contains(&q) || s.id.to_lowercase().contains(&q))
            .collect())
    }

    fn rename(&self, id: &str, name: &str) -> Result<(), DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for rename")))?;
        let n = conn
            .execute(
                "UPDATE sessions SET name = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![name, chrono::Utc::now().timestamp(), id],
            )
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("renaming session {} to '{}'", id, name))))?;
        if n == 0 {
            return Err(DomainError {
                code: "NOT_FOUND".into(),
                message: format!("Session not found: {}", id),
                recoverable: true,
            });
        }
        Ok(())
    }

    fn archive(&self, id: &str) -> Result<(), DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for archive")))?;
        // Move to archived_sessions table
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS archived_sessions (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL, messages TEXT NOT NULL DEFAULT '[]',
                project TEXT NOT NULL DEFAULT '', sort_order INTEGER NOT NULL DEFAULT 0
            );",
        )
        .map_err(|e| DomainError::from(anyhow::Error::from(e).context("creating archived_sessions table")))?;
        conn.execute(
            "INSERT OR REPLACE INTO archived_sessions SELECT * FROM sessions WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("archiving session {}", id))))?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("deleting session {} after archive", id))))?;
        Ok(())
    }

    fn restore(&self, id: &str) -> Result<(), DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for restore")))?;
        conn.execute(
            "INSERT OR REPLACE INTO sessions SELECT * FROM archived_sessions WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("restoring session {} from archive", id))))?;
        conn.execute(
            "DELETE FROM archived_sessions WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("deleting session {} from archive after restore", id))))?;
        Ok(())
    }

    fn list_archived(&self) -> Result<Vec<SessionInfo>, DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for list_archived")))?;
        if let Err(e) = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS archived_sessions (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL, messages TEXT NOT NULL DEFAULT '[]',
                project TEXT NOT NULL DEFAULT '', sort_order INTEGER NOT NULL DEFAULT 0
            );",
        ) {
            tracing::warn!("db: {e}");
        }
        let mut stmt = conn
            .prepare("SELECT id, name, created_at, updated_at, messages, project, sort_order FROM archived_sessions ORDER BY updated_at DESC")
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("preparing list_archived query")))?;
        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let created: i64 = row.get(2)?;
                let updated: i64 = row.get(3)?;
                let messages: String = row.get(4)?;
                let project: String = row.get(5)?;
                let sort_order: i64 = row.get(6)?;
                let message_count = serde_json::from_str::<serde_json::Value>(&messages)
                    .map(|v| v.as_array().map(|a| a.len()).unwrap_or(0))
                    .unwrap_or(0);
                Ok(SessionInfo {
                    id,
                    name,
                    message_count,
                    created_at: created,
                    updated_at: updated,
                    project,
                    sort_order,
                })
            })
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("executing list_archived query")))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| DomainError::from(anyhow::Error::from(e).context("parsing archived session row")))?);
        }
        Ok(out)
    }

    fn tag(&self, id: &str, tag: &str) -> Result<Vec<String>, DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for tag")))?;
        // Store tags in app_state as JSON array
        let key = format!("session_tags:{}", id);
        let existing: String = conn
            .query_row(
                "SELECT value FROM app_state WHERE key = ?1",
                rusqlite::params![key],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "[]".to_string());
        let mut tags: Vec<String> = serde_json::from_str(&existing).unwrap_or_default();
        if !tags.contains(&tag.to_string()) {
            tags.push(tag.to_string());
        }
        let json = serde_json::to_string(&tags).unwrap_or_default();
        conn.execute(
            "INSERT INTO app_state (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![key, json],
        )
        .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("saving tag '{}' for session {}", tag, id))))?;
        Ok(tags)
    }

    fn untag(&self, id: &str, tag: &str) -> Result<Vec<String>, DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for untag")))?;
        let key = format!("session_tags:{}", id);
        let existing: String = conn
            .query_row(
                "SELECT value FROM app_state WHERE key = ?1",
                rusqlite::params![key],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "[]".to_string());
        let mut tags: Vec<String> = serde_json::from_str(&existing).unwrap_or_default();
        tags.retain(|t| t != tag);
        let json = serde_json::to_string(&tags).unwrap_or_default();
        conn.execute(
            "INSERT INTO app_state (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![key, json],
        )
        .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("removing tag '{}' from session {}", tag, id))))?;
        Ok(tags)
    }

    fn clear(&self, id: &str) -> Result<(), DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for clear")))?;
        conn.execute(
            "UPDATE sessions SET messages = '[]' WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("clearing messages for session {}", id))))?;
        Ok(())
    }
}

#[async_trait]
impl DomainPlugin for SessionPlugin {
    fn name(&self) -> &str {
        "session"
    }
    fn description(&self) -> &str {
        "会话管理：CRUD、切换、排序、搜索"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "list".into(),
                description: "列出所有会话".into(),
                ..Default::default()
            },
            ActionSpec {
                name: "create".into(),
                description: "创建新会话".into(),
                params: vec![ParamSpec {
                    name: "name".into(),
                    r#type: "string".into(),
                    description: "会话名称".into(),
                    optional: true,
                }],
                ..Default::default()
            },
            ActionSpec {
                name: "delete".into(),
                description: "删除会话".into(),
                params: vec![ParamSpec {
                    name: "id".into(),
                    r#type: "string".into(),
                    description: "会话 ID".into(),
                    optional: false,
                }],
                ..Default::default()
            },
            ActionSpec {
                name: "switch".into(),
                description: "切换当前会话".into(),
                params: vec![ParamSpec {
                    name: "id".into(),
                    r#type: "string".into(),
                    description: "会话 ID".into(),
                    optional: false,
                }],
                ..Default::default()
            },
            ActionSpec {
                name: "reorder".into(),
                description: "拖拽排序".into(),
                params: vec![ParamSpec {
                    name: "ids".into(),
                    r#type: "array".into(),
                    description: "排序后的 ID 列表".into(),
                    optional: false,
                }],
                ..Default::default()
            },
            ActionSpec {
                name: "set_project".into(),
                description: "设置会话所属项目".into(),
                params: vec![
                    ParamSpec {
                        name: "id".into(),
                        r#type: "string".into(),
                        description: "会话 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "project".into(),
                        r#type: "string".into(),
                        description: "项目路径".into(),
                        optional: false,
                    },
                ],
                ..Default::default()
            },
            ActionSpec {
                name: "fork".into(),
                description: "分支新话题".into(),
                params: vec![
                    ParamSpec {
                        name: "from_id".into(),
                        r#type: "string".into(),
                        description: "源会话 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "up_to".into(),
                        r#type: "number".into(),
                        description: "截取前 N 条消息".into(),
                        optional: true,
                    },
                ],
                ..Default::default()
            },
            ActionSpec {
                name: "search".into(),
                description: "搜索会话".into(),
                params: vec![ParamSpec {
                    name: "query".into(),
                    r#type: "string".into(),
                    description: "搜索关键词".into(),
                    optional: false,
                }],
                ..Default::default()
            },
            ActionSpec {
                name: "tag".into(),
                description: "给会话打标".into(),
                params: vec![
                    ParamSpec {
                        name: "id".into(),
                        r#type: "string".into(),
                        description: "会话 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "tag".into(),
                        r#type: "string".into(),
                        description: "标签名".into(),
                        optional: false,
                    },
                ],
                ..Default::default()
            },
            ActionSpec {
                name: "untag".into(),
                description: "移除会话标签".into(),
                params: vec![
                    ParamSpec {
                        name: "id".into(),
                        r#type: "string".into(),
                        description: "会话 ID".into(),
                        optional: false,
                    },
                    ParamSpec {
                        name: "tag".into(),
                        r#type: "string".into(),
                        description: "标签名".into(),
                        optional: false,
                    },
                ],
                ..Default::default()
            },
        ]
    }

    #[tracing::instrument(skip(self, args), fields(action = %action))]
    async fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "list" => {
                let sessions = self.list_all()
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context("listing all sessions")))?;
                Ok(serde_json::to_value(sessions).unwrap_or_default())
            }
            "create" => {
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("新会话");
                let id = self.create(name)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("creating session '{}'", name))))?;
                Ok(serde_json::json!({ "id": id, "name": name }))
            }
            "delete" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'id'".into(),
                        recoverable: true,
                    })?;
                self.delete(id)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("deleting session {}", id))))?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "switch" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'id'".into(),
                        recoverable: true,
                    })?;
                self.switch_to(id)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("switching to session {}", id))))?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "reorder" => {
                let ids: Vec<String> = args
                    .get("ids")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'ids' array".into(),
                        recoverable: true,
                    })?;
                self.reorder(&ids)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context("reordering sessions")))?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "set_project" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'id'".into(),
                        recoverable: true,
                    })?;
                let project = args
                    .get("project")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'project'".into(),
                        recoverable: true,
                    })?;
                self.set_project(id, project)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("setting project for session {}", id))))?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "fork" => {
                let from_id = args
                    .get("from_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'from_id'".into(),
                        recoverable: true,
                    })?;
                let up_to = args
                    .get("up_to")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize);
                let new_id = self.fork(from_id, up_to)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("forking from session {}", from_id))))?;
                Ok(serde_json::json!({ "id": new_id }))
            }
            "search" => {
                let query =
                    args.get("query")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "missing 'query'".into(),
                            recoverable: true,
                        })?;
                let results = self.search(query)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("searching sessions for '{}'", query))))?;
                Ok(serde_json::to_value(results).unwrap_or_default())
            }
            "tag" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'id'".into(),
                        recoverable: true,
                    })?;
                let tag = args
                    .get("tag")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'tag'".into(),
                        recoverable: true,
                    })?;
                let tags = self.tag(id, tag)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("tagging session {} with '{}'", id, tag))))?;
                Ok(serde_json::json!({ "ok": true, "tags": tags }))
            }
            "untag" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'id'".into(),
                        recoverable: true,
                    })?;
                let tag = args
                    .get("tag")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'tag'".into(),
                        recoverable: true,
                    })?;
                let tags = self.untag(id, tag)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("untagging session {} from '{}'", id, tag))))?;
                Ok(serde_json::json!({ "ok": true, "tags": tags }))
            }
            "archive" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'id'".into(),
                        recoverable: true,
                    })?;
                self.archive(id)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("archiving session {}", id))))?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "restore" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'id'".into(),
                        recoverable: true,
                    })?;
                self.restore(id)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("restoring session {}", id))))?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "list_archived" => {
                let sessions = self.list_archived()
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context("listing archived sessions")))?;
                Ok(serde_json::json!(sessions))
            }
            "rename" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'id'".into(),
                        recoverable: true,
                    })?;
                let name =
                    args.get("name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "missing 'name'".into(),
                            recoverable: true,
                        })?;
                self.rename(id, name)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("renaming session {} to '{}'", id, name))))?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "clear" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'id'".into(),
                        recoverable: true,
                    })?;
                self.clear(id)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("clearing session {}", id))))?;
                Ok(serde_json::json!({ "ok": true }))
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}
