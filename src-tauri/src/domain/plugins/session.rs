use crate::domain::{ActionSpec, DomainError, DomainPlugin, ParamSpec};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

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
    db_path: PathBuf,
    _db: Mutex<()>,
}

impl SessionPlugin {
    pub fn new() -> Self {
        let db_path = dirs::home_dir()
            .map(|h| h.join(".neotrix").join("desktop.db"))
            .unwrap_or_else(|| PathBuf::from(".neotrix/desktop.db"));
        Self {
            db_path,
            _db: Mutex::new(()),
        }
    }

    fn open_db(&self) -> Result<Connection, DomainError> {
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("创建数据目录失败: {}", e), recoverable: true })?;
        }
        let conn = Connection::open(&self.db_path)
            .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("打开数据库失败: {}", e), recoverable: true })?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("启用 WAL 失败: {}", e), recoverable: true })?;
        let _ = conn.busy_timeout(std::time::Duration::from_secs(5));
        conn.execute_batch(
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
        )
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("初始化表失败: {}", e), recoverable: true })?;
        // 向后兼容
        let _ = conn.execute("ALTER TABLE sessions ADD COLUMN project TEXT NOT NULL DEFAULT ''", []);
        let _ = conn.execute("ALTER TABLE sessions ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0", []);
        Ok(conn)
    }

    fn list_all(&self) -> Result<Vec<SessionInfo>, DomainError> {
        let conn = self.open_db()?;
        let mut stmt = conn
            .prepare("SELECT id, name, created_at, updated_at, messages, project, sort_order FROM sessions ORDER BY sort_order ASC, updated_at DESC")
            .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("准备查询失败: {}", e), recoverable: true })?;
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
                Ok(SessionInfo { id, name, message_count, created_at: created, updated_at: updated, project, sort_order })
            })
            .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("解析行失败: {}", e), recoverable: true })?);
        }
        Ok(out)
    }

    fn create(&self, name: &str) -> Result<String, DomainError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let conn = self.open_db()?;
        conn.execute(
            "INSERT INTO sessions (id, name, created_at, updated_at, messages) VALUES (?1, ?2, ?3, ?3, '[]')",
            rusqlite::params![id, name, now],
        )
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("插入失败: {}", e), recoverable: true })?;
        Ok(id)
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let conn = self.open_db()?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("删除失败: {}", e), recoverable: true })?;
        let _ = conn.execute(
            "DELETE FROM app_state WHERE key = 'active_session_id' AND value = ?1",
            rusqlite::params![id],
        );
        Ok(())
    }

    fn switch_to(&self, id: &str) -> Result<(), DomainError> {
        let conn = self.open_db()?;
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sessions WHERE id = ?1)",
                rusqlite::params![id],
                |r| r.get(0),
            )
            .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?;
        if !exists {
            return Err(DomainError { code: "NOT_FOUND".into(), message: format!("Session not found: {}", id), recoverable: true });
        }
        conn.execute(
            "INSERT INTO app_state (key, value) VALUES ('active_session_id', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![id],
        )
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("切换失败: {}", e), recoverable: true })?;
        Ok(())
    }

    fn reorder(&self, ids: &[String]) -> Result<(), DomainError> {
        let conn = self.open_db()?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("事务失败: {}", e), recoverable: true })?;
        for (i, id) in ids.iter().enumerate() {
            tx.execute(
                "UPDATE sessions SET sort_order = ?1 WHERE id = ?2",
                rusqlite::params![i as i64, id],
            )
            .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("更新排序失败: {}", e), recoverable: true })?;
        }
        tx.commit().map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("提交失败: {}", e), recoverable: true })?;
        Ok(())
    }

    fn set_project(&self, id: &str, project: &str) -> Result<(), DomainError> {
        let conn = self.open_db()?;
        let n = conn.execute(
            "UPDATE sessions SET project = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![project, chrono::Utc::now().timestamp(), id],
        )
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("更新失败: {}", e), recoverable: true })?;
        if n == 0 {
            return Err(DomainError { code: "NOT_FOUND".into(), message: format!("Session not found: {}", id), recoverable: true });
        }
        Ok(())
    }

    fn fork(&self, from_id: &str, up_to: Option<usize>) -> Result<String, DomainError> {
        let conn = self.open_db()?;
        let messages: String = conn
            .query_row(
                "SELECT messages FROM sessions WHERE id = ?1",
                rusqlite::params![from_id],
                |r| r.get::<_, String>(0),
            )
            .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("读取源会话失败: {}", e), recoverable: true })?;
        let sliced = up_to
            .and_then(|n| {
                serde_json::from_str::<serde_json::Value>(&messages)
                    .ok()
                    .and_then(|v| v.as_array().map(|a| a.iter().take(n).cloned().collect::<Vec<_>>()))
                    .and_then(|a| serde_json::to_string(&a).ok())
            })
            .unwrap_or(messages);
        let new_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO sessions (id, name, created_at, updated_at, messages, project, sort_order) VALUES (?1, ?2, ?3, ?3, ?4, '', 0)",
            rusqlite::params![new_id, "分支会话", now, sliced],
        )
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("插入失败: {}", e), recoverable: true })?;
        Ok(new_id)
    }

    fn search(&self, query: &str) -> Result<Vec<SessionInfo>, DomainError> {
        let all = self.list_all()?;
        let q = query.to_lowercase();
        Ok(all.into_iter()
            .filter(|s| s.name.to_lowercase().contains(&q) || s.id.to_lowercase().contains(&q))
            .collect())
    }

    fn rename(&self, id: &str, name: &str) -> Result<(), DomainError> {
        let conn = self.open_db()?;
        let n = conn.execute(
            "UPDATE sessions SET name = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![name, chrono::Utc::now().timestamp(), id],
        )
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("重命名失败: {}", e), recoverable: true })?;
        if n == 0 {
            return Err(DomainError { code: "NOT_FOUND".into(), message: format!("Session not found: {}", id), recoverable: true });
        }
        Ok(())
    }

    fn archive(&self, id: &str) -> Result<(), DomainError> {
        let conn = self.open_db()?;
        // Move to archived_sessions table
        conn.execute_batch(&format!(
            "CREATE TABLE IF NOT EXISTS archived_sessions (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL, messages TEXT NOT NULL DEFAULT '[]',
                project TEXT NOT NULL DEFAULT '', sort_order INTEGER NOT NULL DEFAULT 0
            );
            INSERT OR REPLACE INTO archived_sessions SELECT * FROM sessions WHERE id = '{}';
            DELETE FROM sessions WHERE id = '{}';",
            id, id
        ))
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("归档失败: {}", e), recoverable: true })?;
        Ok(())
    }

    fn restore(&self, id: &str) -> Result<(), DomainError> {
        let conn = self.open_db()?;
        conn.execute_batch(&format!(
            "INSERT OR REPLACE INTO sessions SELECT * FROM archived_sessions WHERE id = '{}';
            DELETE FROM archived_sessions WHERE id = '{}';",
            id, id
        ))
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("恢复失败: {}", e), recoverable: true })?;
        Ok(())
    }

    fn list_archived(&self) -> Result<Vec<SessionInfo>, DomainError> {
        let conn = self.open_db()?;
        let _ = conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS archived_sessions (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL, messages TEXT NOT NULL DEFAULT '[]',
                project TEXT NOT NULL DEFAULT '', sort_order INTEGER NOT NULL DEFAULT 0
            );"
        );
        let mut stmt = conn
            .prepare("SELECT id, name, created_at, updated_at, messages, project, sort_order FROM archived_sessions ORDER BY updated_at DESC")
            .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?;
        let rows = stmt.query_map([], |row| {
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
            Ok(SessionInfo { id, name, message_count, created_at: created, updated_at: updated, project, sort_order })
        })
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("解析行失败: {}", e), recoverable: true })?);
        }
        Ok(out)
    }

    fn tag(&self, id: &str, tag: &str) -> Result<Vec<String>, DomainError> {
        let conn = self.open_db()?;
        // Store tags in app_state as JSON array
        let key = format!("session_tags:{}", id);
        let existing: String = conn.query_row(
            "SELECT value FROM app_state WHERE key = ?1",
            rusqlite::params![key],
            |r| r.get(0),
        ).unwrap_or_else(|_| "[]".to_string());
        let mut tags: Vec<String> = serde_json::from_str(&existing).unwrap_or_default();
        if !tags.contains(&tag.to_string()) {
            tags.push(tag.to_string());
        }
        let json = serde_json::to_string(&tags).unwrap_or_default();
        conn.execute(
            "INSERT INTO app_state (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![key, json],
        )
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("打标失败: {}", e), recoverable: true })?;
        Ok(tags)
    }

    fn untag(&self, id: &str, tag: &str) -> Result<Vec<String>, DomainError> {
        let conn = self.open_db()?;
        let key = format!("session_tags:{}", id);
        let existing: String = conn.query_row(
            "SELECT value FROM app_state WHERE key = ?1",
            rusqlite::params![key],
            |r| r.get(0),
        ).unwrap_or_else(|_| "[]".to_string());
        let mut tags: Vec<String> = serde_json::from_str(&existing).unwrap_or_default();
        tags.retain(|t| t != tag);
        let json = serde_json::to_string(&tags).unwrap_or_default();
        conn.execute(
            "INSERT INTO app_state (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![key, json],
        )
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("取消打标失败: {}", e), recoverable: true })?;
        Ok(tags)
    }

    fn clear(&self, id: &str) -> Result<(), DomainError> {
        let conn = self.open_db()?;
        conn.execute(
            "UPDATE sessions SET messages = '[]' WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("清空消息失败: {}", e), recoverable: true })?;
        Ok(())
    }
}

impl DomainPlugin for SessionPlugin {
    fn name(&self) -> &str { "session" }
    fn description(&self) -> &str { "会话管理：CRUD、切换、排序、搜索" }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec { name: "list".into(), description: "列出所有会话".into(), ..Default::default() },
            ActionSpec { name: "create".into(), description: "创建新会话".into(),
                params: vec![ParamSpec { name: "name".into(), r#type: "string".into(), description: "会话名称".into(), optional: true }],
                ..Default::default() },
            ActionSpec { name: "delete".into(), description: "删除会话".into(),
                params: vec![ParamSpec { name: "id".into(), r#type: "string".into(), description: "会话 ID".into(), optional: false }],
                ..Default::default() },
            ActionSpec { name: "switch".into(), description: "切换当前会话".into(),
                params: vec![ParamSpec { name: "id".into(), r#type: "string".into(), description: "会话 ID".into(), optional: false }],
                ..Default::default() },
            ActionSpec { name: "reorder".into(), description: "拖拽排序".into(),
                params: vec![ParamSpec { name: "ids".into(), r#type: "array".into(), description: "排序后的 ID 列表".into(), optional: false }],
                ..Default::default() },
            ActionSpec { name: "set_project".into(), description: "设置会话所属项目".into(),
                params: vec![
                    ParamSpec { name: "id".into(), r#type: "string".into(), description: "会话 ID".into(), optional: false },
                    ParamSpec { name: "project".into(), r#type: "string".into(), description: "项目路径".into(), optional: false },
                ],
                ..Default::default() },
            ActionSpec { name: "fork".into(), description: "分支新话题".into(),
                params: vec![
                    ParamSpec { name: "from_id".into(), r#type: "string".into(), description: "源会话 ID".into(), optional: false },
                    ParamSpec { name: "up_to".into(), r#type: "number".into(), description: "截取前 N 条消息".into(), optional: true },
                ],
                ..Default::default() },
            ActionSpec { name: "search".into(), description: "搜索会话".into(),
                params: vec![ParamSpec { name: "query".into(), r#type: "string".into(), description: "搜索关键词".into(), optional: false }],
                ..Default::default() },
            ActionSpec { name: "tag".into(), description: "给会话打标".into(),
                params: vec![
                    ParamSpec { name: "id".into(), r#type: "string".into(), description: "会话 ID".into(), optional: false },
                    ParamSpec { name: "tag".into(), r#type: "string".into(), description: "标签名".into(), optional: false },
                ],
                ..Default::default() },
            ActionSpec { name: "untag".into(), description: "移除会话标签".into(),
                params: vec![
                    ParamSpec { name: "id".into(), r#type: "string".into(), description: "会话 ID".into(), optional: false },
                    ParamSpec { name: "tag".into(), r#type: "string".into(), description: "标签名".into(), optional: false },
                ],
                ..Default::default() },
        ]
    }

    fn call(&self, action: &str, args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        match action {
            "list" => {
                let sessions = self.list_all()?;
                Ok(serde_json::to_value(sessions).unwrap_or_default())
            }
            "create" => {
                let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("新会话");
                let id = self.create(name)?;
                Ok(serde_json::json!({ "id": id, "name": name }))
            }
            "delete" => {
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'id'".into(), recoverable: true })?;
                self.delete(id)?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "switch" => {
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'id'".into(), recoverable: true })?;
                self.switch_to(id)?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "reorder" => {
                let ids: Vec<String> = args.get("ids")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'ids' array".into(), recoverable: true })?;
                self.reorder(&ids)?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "set_project" => {
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'id'".into(), recoverable: true })?;
                let project = args.get("project").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'project'".into(), recoverable: true })?;
                self.set_project(id, project)?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "fork" => {
                let from_id = args.get("from_id").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'from_id'".into(), recoverable: true })?;
                let up_to = args.get("up_to").and_then(|v| v.as_u64()).map(|n| n as usize);
                let new_id = self.fork(from_id, up_to)?;
                Ok(serde_json::json!({ "id": new_id }))
            }
            "search" => {
                let query = args.get("query").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'query'".into(), recoverable: true })?;
                let results = self.search(query)?;
                Ok(serde_json::to_value(results).unwrap_or_default())
            }
            "tag" => {
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'id'".into(), recoverable: true })?;
                let tag = args.get("tag").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'tag'".into(), recoverable: true })?;
                let tags = self.tag(id, tag)?;
                Ok(serde_json::json!({ "ok": true, "tags": tags }))
            }
            "untag" => {
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'id'".into(), recoverable: true })?;
                let tag = args.get("tag").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'tag'".into(), recoverable: true })?;
                let tags = self.untag(id, tag)?;
                Ok(serde_json::json!({ "ok": true, "tags": tags }))
            }
            "archive" => {
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'id'".into(), recoverable: true })?;
                self.archive(id)?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "restore" => {
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'id'".into(), recoverable: true })?;
                self.restore(id)?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "list_archived" => {
                let sessions = self.list_archived()?;
                Ok(serde_json::json!(sessions))
            }
            "rename" => {
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'id'".into(), recoverable: true })?;
                let name = args.get("name").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'name'".into(), recoverable: true })?;
                self.rename(id, name)?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "clear" => {
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'id'".into(), recoverable: true })?;
                self.clear(id)?;
                Ok(serde_json::json!({ "ok": true }))
            }
            _ => Err(DomainError { code: "UNKNOWN_ACTION".into(), message: format!("Unknown action: {}", action), recoverable: true }),
        }
    }
}

impl Default for ActionSpec {
    fn default() -> Self {
        Self { name: String::new(), description: String::new(), params: vec![], returns: "Value".into() }
    }
}
