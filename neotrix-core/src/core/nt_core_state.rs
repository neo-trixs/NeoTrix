// ═══════════════════════════════════════════════════════════════════
// nt_core_state — 内部状态统一落 KB 构造 (Phase 2)
//
// 目的: 逐步淘汰内部状态本地 JSON 文件, 统一直写 KB kv_store `state`
// 命名空间。过渡期 dual-write 保留 legacy 文件; 切换点由 DUAL_WRITE_FILE
// 控制 (false 后不再写文件, 仅读文件作迁移 fallback)。
//
// 分层: 位于 core 层, 仅依赖 nt_core_kb_primitives (core 内), 避免
// core→neotrix 层违规 (nt_core_self_review 会审计)。
// ═══════════════════════════════════════════════════════════════════

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use neotrix_types::core::fs_util::atomic_write;
use rusqlite::Connection;

use super::nt_core_kb_primitives as kv;

/// state 统一命名空间 (kv_store.namespace)。
pub const NS: &str = "state";

/// 过渡开关: true = 写 KB 同时写 legacy 文件; 翻 false 后停止写文件。
/// 注意: 该值是编译期常量, 翻转需重新编译并测试 (R-P35)。
pub const DUAL_WRITE_FILE: bool = true;

/// 全局懒加载连接 (生产路径): ~/.neotrix/knowledge.db, WAL + schema 初始化一次。
static CONN: OnceLock<Mutex<Connection>> = OnceLock::new();

fn global_conn() -> &'static Mutex<Connection> {
    CONN.get_or_init(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let db_path = PathBuf::from(home).join(".neotrix").join("knowledge.db");
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(&db_path).expect("open KB state connection");
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        kv::schema_initialize(&conn).expect("initialize KB schema for state");
        Mutex::new(conn)
    })
}

/// legacy 文件路径 (迁移前布局): ~/.neotrix/{name}.json。
pub fn legacy_path(name: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".neotrix").join(format!("{name}.json"))
}

fn file_load(name: &str) -> Option<String> {
    std::fs::read_to_string(legacy_path(name)).ok()
}

fn file_save(name: &str, json: &str) -> Result<(), String> {
    let path = legacy_path(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create_dir: {e}"))?;
    }
    atomic_write(&path, json.as_bytes()).map_err(|e| format!("file dual-write: {e}"))
}

/// 生产路径: 写 KB + (DUAL_WRITE_FILE) legacy 文件。使用全局连接。
pub fn save(name: &str, json: &str) -> Result<(), String> {
    let guard = global_conn()
        .lock()
        .map_err(|_| "state conn poisoned".to_string())?;
    save_with(&guard, name, json)
}

/// 可注入连接变体 (测试用内存连接 / 复用外部连接)。
pub fn save_with(conn: &Connection, name: &str, json: &str) -> Result<(), String> {
    kv::kv_set(conn, NS, name, json)?;
    if DUAL_WRITE_FILE {
        file_save(name, json)?;
    }
    Ok(())
}

/// 生产路径: KB 优先, 未命中回退 legacy 文件。
pub fn load(name: &str) -> Option<String> {
    match global_conn().lock() {
        Ok(guard) => load_with(&guard, name),
        Err(_) => file_load(name),
    }
}

/// 可注入连接变体: KB 优先, 未命中回退 legacy 文件。
pub fn load_with(conn: &Connection, name: &str) -> Option<String> {
    match kv::kv_get(conn, NS, name) {
        Ok(Some(v)) => Some(v),
        Ok(None) => file_load(name),
        Err(e) => {
            log::warn!("state kv_get({name}) failed: {e}; falling back to file");
            file_load(name)
        }
    }
}

/// 生产路径: 删除 KB 状态 (翻转期/测试清理)。
pub fn delete(name: &str) -> Result<bool, String> {
    let guard = global_conn()
        .lock()
        .map_err(|_| "state conn poisoned".to_string())?;
    kv::kv_delete(&guard, NS, name)
}

/// 可注入连接变体。
pub fn delete_with(conn: &Connection, name: &str) -> Result<bool, String> {
    kv::kv_delete(conn, NS, name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        kv::schema_initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_legacy_path_shape() {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let expected = PathBuf::from(home)
            .join(".neotrix")
            .join("workspaces.json");
        assert_eq!(legacy_path("workspaces"), expected);
    }

    #[test]
    fn test_save_load_kb_roundtrip() {
        let conn = mem_conn();
        save_with(&conn, "workspaces", r#"{"a":1}"#).unwrap();
        assert_eq!(
            load_with(&conn, "workspaces").as_deref(),
            Some(r#"{"a":1}"#)
        );
    }

    #[test]
    fn test_kb_priority_over_file() {
        let conn = mem_conn();
        let name = "kb_priority_test";
        let legacy = legacy_path(name);
        file_save(name, "old-file").unwrap();
        save_with(&conn, name, "kb-value").unwrap();
        assert_eq!(load_with(&conn, name).as_deref(), Some("kb-value"));
        let _ = std::fs::remove_file(&legacy);
    }
}