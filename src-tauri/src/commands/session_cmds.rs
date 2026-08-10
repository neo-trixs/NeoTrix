use std::path::PathBuf;
use tauri::command;
use neotrix::neotrix::nt_core_error::NeoTrixError;
use rusqlite::Connection;
use super::SessionInfo;

// 测试时可覆盖数据库路径 (thread-local: 仅影响当前测试线程, 避免并行测试互相干扰)
thread_local! {
    static DB_OVERRIDE: std::cell::Cell<Option<PathBuf>> = std::cell::Cell::new(None);
}

/// 统一数据库路径: ~/.neotrix/desktop.db
fn desktop_db_path() -> PathBuf {
    if let Some(p) = DB_OVERRIDE.with(|c| c.replace(None)) {
        DB_OVERRIDE.with(|c| c.set(Some(p.clone())));
        return p;
    }
    if let Ok(p) = std::env::var("NEOTRIX_DESKTOP_DB") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    dirs::home_dir()
        .map(|h| h.join(".neotrix").join("desktop.db"))
        .unwrap_or_else(|| PathBuf::from(".neotrix/desktop.db"))
}

/// 打开桌面数据库并确保表结构存在 (每次操作独立打开, 由 SQLite 文件锁保证并发安全)
fn open_db() -> Result<Connection, NeoTrixError> {
    let path = desktop_db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| NeoTrixError::Io(format!("创建数据目录 {:?} 失败: {}", parent, e)))?;
    }
    let conn = Connection::open(&path)
        .map_err(|e| NeoTrixError::Memory(format!("打开数据库 {:?} 失败: {}", path, e)))?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| NeoTrixError::Memory(format!("启用 WAL 失败: {}", e)))?;
    let _ = conn.busy_timeout(std::time::Duration::from_secs(5));
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            messages TEXT NOT NULL DEFAULT '[]'  -- JSON 数组
        );
        CREATE TABLE IF NOT EXISTS app_state (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )
    .map_err(|e| NeoTrixError::Memory(format!("初始化会话表失败: {}", e)))?;
    Ok(conn)
}

/// 将 sessions 行转换为 SessionInfo (message_count 由 messages JSON 数组长度推导)
fn row_to_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionInfo> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let created: i64 = row.get(2)?;
    let messages: String = row.get(4)?;
    let message_count = serde_json::from_str::<serde_json::Value>(&messages)
        .map(|v| v.as_array().map(|a| a.len()).unwrap_or(0))
        .unwrap_or(0);
    Ok(SessionInfo { id, name, message_count, created })
}

/// 读取单条会话的 (id, name, created_at, messages)
fn get_session_row(conn: &Connection, id: &str) -> Result<(String, String, i64, String), NeoTrixError> {
    conn.query_row(
        "SELECT id, name, created_at, updated_at, messages FROM sessions WHERE id = ?1",
        rusqlite::params![id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(4)?)),
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            NeoTrixError::Memory(format!("Session not found: {}", id))
        }
        other => NeoTrixError::Memory(format!("查询会话失败: {}", other)),
    })
}

#[command]
pub fn session_list() -> Vec<SessionInfo> {
    vec![SessionInfo {
        id: "default".into(),
        name: "默认会话".into(),
        message_count: 0,
        created: 0,
    }]
}

#[command]
pub fn session_create(name: String) -> SessionInfo {
    SessionInfo {
        id: format!("s-{}", chrono::Utc::now().timestamp()),
        name,
        message_count: 0,
        created: chrono::Utc::now().timestamp(),
    }
}

#[command]
pub fn cmd_session_create(name: String) -> Result<String, NeoTrixError> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let conn = open_db()?;
    conn.execute(
        "INSERT INTO sessions (id, name, created_at, updated_at, messages)
         VALUES (?1, ?2, ?3, ?3, '[]')",
        rusqlite::params![id, name, now],
    )
    .map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    Ok(id)
}

#[command]
pub fn cmd_session_switch(id: String) -> Result<(), NeoTrixError> {
    let conn = open_db()?;
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE id = ?1)",
            rusqlite::params![id],
            |r| r.get(0),
        )
        .map_err(|e| NeoTrixError::Memory(format!("查询会话失败: {}", e)))?;
    if !exists {
        return Err(NeoTrixError::Memory(format!("Session not found: {}", id)));
    }
    // 记录当前 active session id 到 app_state 表
    conn.execute(
        "INSERT INTO app_state (key, value) VALUES ('active_session_id', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![id],
    )
    .map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    Ok(())
}

#[command]
pub fn cmd_session_delete(id: String) -> Result<(), NeoTrixError> {
    let conn = open_db()?;
    conn.execute("DELETE FROM sessions WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let _ = conn.execute(
        "DELETE FROM app_state WHERE key = 'active_session_id' AND value = ?1",
        rusqlite::params![id],
    );
    Ok(())
}

#[command]
pub fn cmd_session_list() -> Result<Vec<SessionInfo>, NeoTrixError> {
    let conn = open_db()?;
    let mut stmt = conn
        .prepare("SELECT id, name, created_at, updated_at, messages FROM sessions ORDER BY updated_at DESC")
        .map_err(|e| NeoTrixError::Memory(format!("准备查询失败: {}", e)))?;
    let rows = stmt
        .query_map([], row_to_session)
        .map_err(|e| NeoTrixError::Memory(format!("查询会话失败: {}", e)))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| NeoTrixError::Memory(format!("解析会话失败: {}", e)))?);
    }
    Ok(out)
}

#[command]
pub fn cmd_session_fork(id: String) -> Result<String, NeoTrixError> {
    let conn = open_db()?;
    let (_, src_name, _, messages) = get_session_row(&conn, &id)?;
    let new_id = uuid::Uuid::new_v4().to_string();
    let new_name = format!("{} (副本)", src_name);
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO sessions (id, name, created_at, updated_at, messages)
         VALUES (?1, ?2, ?3, ?3, ?4)",
        rusqlite::params![new_id, new_name, now, messages],
    )
    .map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    Ok(new_id)
}

#[command]
pub fn cmd_session_export_json(id: String) -> Result<String, NeoTrixError> {
    let conn = open_db()?;
    let (src_id, src_name, src_created, messages) = get_session_row(&conn, &id)?;
    let message_count = serde_json::from_str::<serde_json::Value>(&messages)
        .map(|v| v.as_array().map(|a| a.len()).unwrap_or(0))
        .unwrap_or(0);
    let export = serde_json::json!({
        "format_version": 1,
        "sessions": [{
            "id": src_id,
            "name": src_name,
            "message_count": message_count,
            "created": src_created,
        }],
    });
    serde_json::to_string_pretty(&export).map_err(|e| NeoTrixError::Serde(e.to_string()))
}

#[command]
pub fn cmd_session_import_json(json: String) -> Result<String, NeoTrixError> {
    let value: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| NeoTrixError::Serde(format!("解析失败: {}", e)))?;
    let version = value
        .get("format_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if version != 1 {
        return Err(NeoTrixError::Serde(format!("不支持的格式版本: {}", version)));
    }
    let sessions_arr = value
        .get("sessions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| NeoTrixError::Memory("缺少 sessions 字段".to_string()))?;
    let conn = open_db()?;
    let now = chrono::Utc::now().timestamp();
    let mut imported_ids = Vec::new();
    for item in sessions_arr {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("imported");
        let msg_count = item
            .get("message_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let created = item
            .get("created")
            .and_then(|v| v.as_i64())
            .unwrap_or(now);
        // 同名已存在则加 "(导入)" 后缀
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sessions WHERE name = ?1)",
                rusqlite::params![name],
                |r| r.get(0),
            )
            .map_err(|e| NeoTrixError::Memory(format!("查询会话失败: {}", e)))?;
        let final_name = if exists {
            format!("{} (导入)", name)
        } else {
            name.to_string()
        };
        let new_id = uuid::Uuid::new_v4().to_string();
        // 导出格式不含原始消息, 用 null 占位数组保留 message_count
        let messages = serde_json::to_string(&vec![serde_json::Value::Null; msg_count])
            .map_err(|e| NeoTrixError::Serde(e.to_string()))?;
        conn.execute(
            "INSERT INTO sessions (id, name, created_at, updated_at, messages)
             VALUES (?1, ?2, ?3, ?3, ?4)",
            rusqlite::params![new_id, final_name, created, messages],
        )
        .map_err(|e| NeoTrixError::Brain(e.to_string()))?;
        imported_ids.push(new_id);
    }
    Ok(imported_ids.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static DB_TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// 使用临时目录的数据库执行闭包, 验证持久化且不污染真实数据
    fn with_temp_db<T>(f: impl FnOnce() -> T) -> T {
        let n = DB_TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("neotrix-session-test-{}-{}", std::process::id(), n));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("desktop.db");
        DB_OVERRIDE.with(|c| c.set(Some(path)));
        let result = f();
        DB_OVERRIDE.with(|c| c.set(None));
        result
    }

    #[test]
    fn test_persistence_create_list_delete() {
        with_temp_db(|| {
            let id = cmd_session_create("持久化会话".into()).unwrap();
            assert!(!id.is_empty());

            let list = cmd_session_list().unwrap();
            let created = list.iter().find(|s| s.id == id).expect("create 后应能在 list 中看到");
            assert_eq!(created.name, "持久化会话");
            assert_eq!(created.message_count, 0);

            assert!(cmd_session_switch(id.clone()).is_ok());
            assert!(cmd_session_switch("nonexistent-id".into()).is_err());

            assert!(cmd_session_delete(id.clone()).is_ok());
            let list = cmd_session_list().unwrap();
            assert!(list.iter().all(|s| s.id != id), "delete 后不应再出现该会话");
        });
    }

    #[test]
    fn test_persistence_across_restart() {
        with_temp_db(|| {
            let id = cmd_session_create("重启保留".into()).unwrap();
            // 每次命令都重新打开连接, 验证数据落盘后仍可读 (等价于跨重启)
            let list = cmd_session_list().unwrap();
            assert!(list.iter().any(|s| s.id == id && s.name == "重启保留"));
        });
    }

    #[test]
    fn test_persistence_fork() {
        with_temp_db(|| {
            let id = cmd_session_create("源会话".into()).unwrap();
            let forked = cmd_session_fork(id.clone()).unwrap();
            assert_ne!(forked, id);

            let list = cmd_session_list().unwrap();
            assert!(list.iter().any(|s| s.id == id), "源会话应保留");
            let forked_info = list.iter().find(|s| s.id == forked).expect("fork 后应存在新会话");
            assert!(forked_info.name.contains("副本"));
        });
    }

    #[test]
    fn test_persistence_export_import() {
        with_temp_db(|| {
            let id = cmd_session_create("导出会话".into()).unwrap();
            let json = cmd_session_export_json(id.clone()).unwrap();
            assert!(json.contains("导出会话"));

            let imported = cmd_session_import_json(json).unwrap();
            assert!(!imported.is_empty());

            let list = cmd_session_list().unwrap();
            assert!(list.iter().any(|s| s.id == id), "原会话应保留");
            for imp_id in imported.split(',') {
                assert!(list.iter().any(|s| s.id == imp_id), "导入的会话应存在");
            }
        });
    }
}
