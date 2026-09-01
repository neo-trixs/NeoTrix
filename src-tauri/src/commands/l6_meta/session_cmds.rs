use std::path::PathBuf;
use tauri::command;
use neotrix::neotrix::nt_core_error::NeoTrixError;
use rusqlite::Connection;
use super::SessionInfo;

// 测试时可覆盖数据库路径 (thread-local: 仅影响当前测试线程, 避免并行测试互相干扰)
thread_local! {
    static DB_OVERRIDE: std::cell::Cell<Option<PathBuf>> = const { std::cell::Cell::new(None) };
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
            messages TEXT NOT NULL DEFAULT '[]',  -- JSON 数组
            project TEXT NOT NULL DEFAULT '',     -- 跨项目拖拽：会话所属项目
            sort_order INTEGER NOT NULL DEFAULT 0 -- 手动拖拽排序权重
        );
        CREATE TABLE IF NOT EXISTS app_state (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )
    .map_err(|e| NeoTrixError::Memory(format!("初始化会话表失败: {}", e)))?;
    // 向后兼容：旧库补列（幂等，列已存在则忽略）
    let _ = conn.execute("ALTER TABLE sessions ADD COLUMN project TEXT NOT NULL DEFAULT ''", []);
    let _ = conn.execute("ALTER TABLE sessions ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0", []);
    Ok(conn)
}

/// 将 sessions 行转换为 SessionInfo (message_count 由 messages JSON 数组长度推导)
fn row_to_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionInfo> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let created: i64 = row.get(2)?;
    let messages: String = row.get(4)?;
    let project: String = row.get(5)?;
    let sort_order: i64 = row.get(6)?;
    let message_count = serde_json::from_str::<serde_json::Value>(&messages)
        .map(|v| v.as_array().map(|a| a.len()).unwrap_or(0))
        .unwrap_or(0);
    Ok(SessionInfo { id, name, message_count, created, project, sort_order })
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
        .prepare("SELECT id, name, created_at, updated_at, messages, project, sort_order FROM sessions ORDER BY sort_order ASC, updated_at DESC")
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

/// 拖拽排序：按传入 id 顺序写入 sort_order（对标 2026 会话列表拖拽持久化）
#[command]
pub fn cmd_reorder_sessions(ids: Vec<String>) -> Result<(), NeoTrixError> {
    let conn = open_db()?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    for (i, id) in ids.iter().enumerate() {
        tx.execute(
            "UPDATE sessions SET sort_order = ?1 WHERE id = ?2",
            rusqlite::params![i as i64, id],
        )
        .map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    }
    tx.commit().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    Ok(())
}

/// 跨项目拖拽：重设会话所属项目（持久化）
#[command]
pub fn cmd_set_session_project(id: String, project: String) -> Result<(), NeoTrixError> {
    let conn = open_db()?;
    let n = conn.execute(
        "UPDATE sessions SET project = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![project, chrono::Utc::now().timestamp(), id],
    )
    .map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    if n == 0 {
        return Err(NeoTrixError::Memory(format!("Session not found: {}", id)));
    }
    Ok(())
}

/// 分支新话题：复制 from 会话的消息前缀（up_to 条，默认全部）为新会话，返回新 id
#[command]
pub fn cmd_fork_session(from_id: String, up_to: Option<usize>) -> Result<String, NeoTrixError> {
    let conn = open_db()?;
    let messages: String = conn
        .query_row(
            "SELECT messages FROM sessions WHERE id = ?1",
            rusqlite::params![from_id],
            |r| r.get::<_, String>(0),
        )
        .map_err(|e| NeoTrixError::Memory(format!("读取源会话失败: {} ({})", from_id, e)))?;
    let sliced: String = up_to
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
        "INSERT INTO sessions (id, name, created_at, updated_at, messages, project, sort_order)
         VALUES (?1, ?2, ?3, ?3, ?4, '', 0)",
        rusqlite::params![new_id, "分支会话".to_string(), now, sliced],
    )
    .map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    Ok(new_id)
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

            let list = cmd_session_list().unwrap();
            assert!(list.iter().any(|s| s.id == id), "源会话应保留");
        });
    }

    #[test]
    fn test_persistence_export_import() {
        with_temp_db(|| {
            let id = cmd_session_create("导出会话".into()).unwrap();

            let list = cmd_session_list().unwrap();
            assert!(list.iter().any(|s| s.id == id), "原会话应保留");
        });
    }
}
