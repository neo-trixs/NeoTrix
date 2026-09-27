use std::collections::HashMap;

use rusqlite::Connection;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::nt_unify_config::now;

// ─── Session Logs ───────────────────────────────────────────────────────────

pub fn session_log_append(
    conn: &Connection,
    session_id: &str,
    content: &str,
    content_type: &str,
    metadata: Option<&serde_json::Value>,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let ts = now();
    let seq = conn
        .query_row(
            "SELECT COALESCE(MAX(sequence), 0) + 1 FROM session_logs WHERE session_id=?1",
            rusqlite::params![session_id],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(1);
    let meta_str = metadata.map(|m| m.to_string());
    conn.execute(
        "INSERT INTO session_logs (id, session_id, sequence, content, content_type, created_at, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, session_id, seq, content, content_type, ts, meta_str],
    )
    .map_err(|e| format!("session_log_append: {}", e))?;
    Ok(id)
}

pub fn session_log_get(
    conn: &Connection,
    session_id: &str,
    limit: usize,
    offset: usize,
) -> Result<Vec<(i64, String, String, String, Option<String>)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT sequence, content, content_type, created_at, metadata
             FROM session_logs WHERE session_id=?1
             ORDER BY sequence DESC LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| format!("session_log_get prepare: {}", e))?;
    let rows = stmt
        .query_map(
            rusqlite::params![session_id, limit as i64, offset as i64],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?.to_string(),
                    row.get::<_, Option<String>>(4)?,
                ))
            },
        )
        .map_err(|e| format!("session_log_get query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("session_log_get row: {}", e))?);
    }
    Ok(results)
}

pub fn session_log_list_sessions(conn: &Connection) -> Result<Vec<(String, i64, i64)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT session_id, COUNT(*), MAX(created_at)
             FROM session_logs GROUP BY session_id ORDER BY MAX(created_at) DESC",
        )
        .map_err(|e| format!("session_log_list_sessions prepare: {}", e))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| format!("session_log_list_sessions query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("session_log_list_sessions row: {}", e))?);
    }
    Ok(results)
}

// ─── Cookies ────────────────────────────────────────────────────────────────

pub fn cookie_set(
    conn: &Connection,
    domain: &str,
    name: &str,
    value: &str,
    path: &str,
    secure: bool,
    http_only: bool,
    expiry: Option<i64>,
) -> Result<(), String> {
    let ts = now();
    conn.execute(
        "INSERT INTO cookies (domain, name, value, path, secure, http_only, expiry, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(domain, name, path) DO UPDATE SET
           value=excluded.value, secure=excluded.secure, http_only=excluded.http_only,
           expiry=excluded.expiry, updated_at=excluded.updated_at",
        rusqlite::params![
            domain, name, value, path, secure as i32, http_only as i32, expiry, ts, ts
        ],
    )
    .map_err(|e| format!("cookie_set: {}", e))?;
    Ok(())
}

pub fn cookie_get(
    conn: &Connection,
    domain: &str,
    name: &str,
    path: &str,
) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT value FROM cookies WHERE domain=?1 AND name=?2 AND path=?3")
        .map_err(|e| format!("cookie_get prepare: {}", e))?;
    let result = stmt
        .query_row(rusqlite::params![domain, name, path], |row| {
            row.get::<_, String>(0)
        })
        .ok();
    Ok(result)
}

pub fn cookie_list_domain(
    conn: &Connection,
    domain: &str,
) -> Result<Vec<(String, String, String, bool, bool, Option<i64>)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT name, value, path, secure, http_only, expiry
             FROM cookies WHERE domain=?1 ORDER BY name",
        )
        .map_err(|e| format!("cookie_list_domain prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![domain], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)? != 0,
                row.get::<_, i32>(4)? != 0,
                row.get::<_, Option<i64>>(5)?,
            ))
        })
        .map_err(|e| format!("cookie_list_domain query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("cookie_list_domain row: {}", e))?);
    }
    Ok(results)
}

pub fn cookie_delete(
    conn: &Connection,
    domain: &str,
    name: &str,
    path: &str,
) -> Result<bool, String> {
    let rows = conn
        .execute(
            "DELETE FROM cookies WHERE domain=?1 AND name=?2 AND path=?3",
            rusqlite::params![domain, name, path],
        )
        .map_err(|e| format!("cookie_delete: {}", e))?;
    Ok(rows > 0)
}

pub fn cookie_purge_expired(conn: &Connection) -> Result<usize, String> {
    let now_ts = now();
    let rows = conn
        .execute(
            "DELETE FROM cookies WHERE expiry IS NOT NULL AND expiry < ?1",
            rusqlite::params![now_ts],
        )
        .map_err(|e| format!("cookie_purge_expired: {}", e))?;
    Ok(rows)
}

// ─── Binary Assets ──────────────────────────────────────────────────────────

pub fn asset_store(
    conn: &Connection,
    namespace: &str,
    name: &str,
    data: &[u8],
    mime_type: Option<&str>,
    metadata: Option<&serde_json::Value>,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let ts = now();
    let checksum = {
        let hash = Sha256::digest(data);
        hex::encode(hash)
    };
    let size = data.len() as i64;
    let meta_str = metadata.map(|m| m.to_string());
    conn.execute(
        "INSERT INTO binary_assets (id, namespace, name, data, mime_type, size, checksum, created_at, updated_at, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![id, namespace, name, data, mime_type, size, checksum, ts, ts, meta_str],
    )
    .map_err(|e| format!("asset_store: {}", e))?;
    Ok(id)
}

pub fn asset_load(
    conn: &Connection,
    id: &str,
) -> Result<Option<(Vec<u8>, String, String, Option<String>, i64, Option<String>)>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT data, namespace, name, mime_type, size, checksum FROM binary_assets WHERE id=?1",
        )
        .map_err(|e| format!("asset_load prepare: {}", e))?;
    let result = stmt
        .query_row(rusqlite::params![id], |row| {
            Ok((
                row.get::<_, Vec<u8>>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })
        .ok();
    Ok(result)
}

pub fn asset_list(
    conn: &Connection,
    namespace: &str,
) -> Result<
    Vec<(
        String,
        String,
        Option<String>,
        Option<String>,
        i64,
        Option<String>,
    )>,
    String,
> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, mime_type, checksum, size, metadata
             FROM binary_assets WHERE namespace=?1 ORDER BY name",
        )
        .map_err(|e| format!("asset_list prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![namespace], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })
        .map_err(|e| format!("asset_list query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("asset_list row: {}", e))?);
    }
    Ok(results)
}

pub fn asset_delete(conn: &Connection, id: &str) -> Result<bool, String> {
    let rows = conn
        .execute(
            "DELETE FROM binary_assets WHERE id=?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("asset_delete: {}", e))?;
    Ok(rows > 0)
}

// ─── Rkyv Blobs ─────────────────────────────────────────────────────────────

pub fn rkyv_store(conn: &Connection, namespace: &str, data: &[u8]) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let ts = now();
    let checksum = {
        let hash = Sha256::digest(data);
        hex::encode(hash)
    };
    conn.execute(
        "INSERT INTO rkyv_blobs (id, namespace, data, checksum, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, namespace, data, checksum, ts],
    )
    .map_err(|e| format!("rkyv_store: {}", e))?;
    Ok(id)
}

pub fn rkyv_load(conn: &Connection, id: &str) -> Result<Option<(Vec<u8>, String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT data, namespace, checksum FROM rkyv_blobs WHERE id=?1")
        .map_err(|e| format!("rkyv_load prepare: {}", e))?;
    let result = stmt
        .query_row(rusqlite::params![id], |row| {
            Ok((
                row.get::<_, Vec<u8>>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .ok();
    Ok(result)
}

pub fn rkyv_list(conn: &Connection, namespace: &str) -> Result<Vec<(String, String, i64)>, String> {
    let mut stmt = conn
        .prepare("SELECT id, checksum, created_at FROM rkyv_blobs WHERE namespace=?1 ORDER BY created_at DESC")
        .map_err(|e| format!("rkyv_list prepare: {}", e))?;
    let rows = stmt
        .query_map(rusqlite::params![namespace], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| format!("rkyv_list query: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("rkyv_list row: {}", e))?);
    }
    Ok(results)
}

pub fn rkyv_delete(conn: &Connection, id: &str) -> Result<bool, String> {
    let rows = conn
        .execute("DELETE FROM rkyv_blobs WHERE id=?1", rusqlite::params![id])
        .map_err(|e| format!("rkyv_delete: {}", e))?;
    Ok(rows > 0)
}

// ─── Cleanup helpers ────────────────────────────────────────────────────────
// kv_purge_namespace 已下沉至 core (nt_core_kb_primitives), 经 nt_unify_config re-export。

/// Get total size stats for the unified store
pub fn store_stats(conn: &Connection) -> Result<HashMap<String, usize>, String> {
    let tables = [
        "kv_store",
        "config_entries",
        "secrets",
        "session_logs",
        "cookies",
        "binary_assets",
        "skills_index",
        "rkyv_blobs",
    ];
    let mut stats = HashMap::new();
    for table in &tables {
        let sql = format!("SELECT COUNT(*) FROM {}", table);
        if let Ok(count) = conn.query_row(&sql, [], |row| row.get::<_, i64>(0)) {
            stats.insert(table.to_string(), count as usize);
        }
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::nt_unify_config::{config_set, kv_set};
    use crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_schema;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    fn mime_for_extension(ext: Option<&str>) -> Option<String> {
        let mime = match ext.unwrap_or("") {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "json" => "application/json",
            "xml" => "application/xml",
            "yaml" | "yml" => "application/yaml",
            "pdf" => "application/pdf",
            "md" | "markdown" => "text/markdown",
            "txt" => "text/plain",
            "html" | "htm" => "text/html",
            "csv" => "text/csv",
            "toml" => "application/toml",
            "rs" => "text/x-rust",
            "py" => "text/x-python",
            "js" => "application/javascript",
            "ts" => "application/typescript",
            _ => "application/octet-stream",
        };
        Some(mime.to_string())
    }

    #[test]
    fn test_session_log_append_and_retrieve() {
        let conn = test_conn();
        let id1 = session_log_append(&conn, "session1", "hello", "markdown", None).unwrap();
        let id2 = session_log_append(&conn, "session1", "world", "markdown", None).unwrap();
        assert_ne!(id1, id2);
        let logs = session_log_get(&conn, "session1", 10, 0).unwrap();
        assert_eq!(logs.len(), 2);
        // Returns in DESC sequence, so "world" first
        assert!(logs[0].1.contains("world"));
        assert!(logs[1].1.contains("hello"));
    }

    #[test]
    fn test_session_log_list_sessions() {
        let conn = test_conn();
        session_log_append(&conn, "s1", "a", "text", None).unwrap();
        session_log_append(&conn, "s2", "b", "text", None).unwrap();
        let sessions = session_log_list_sessions(&conn).unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_cookie_roundtrip() {
        let conn = test_conn();
        cookie_set(
            &conn,
            "example.com",
            "session",
            "abc123",
            "/",
            true,
            true,
            None,
        )
        .unwrap();
        assert_eq!(
            cookie_get(&conn, "example.com", "session", "/").unwrap(),
            Some("abc123".into())
        );
    }

    #[test]
    fn test_cookie_list_domain() {
        let conn = test_conn();
        cookie_set(&conn, "example.com", "a", "1", "/", false, false, None).unwrap();
        cookie_set(&conn, "example.com", "b", "2", "/", false, false, None).unwrap();
        let cookies = cookie_list_domain(&conn, "example.com").unwrap();
        assert_eq!(cookies.len(), 2);
    }

    #[test]
    fn test_cookie_delete() {
        let conn = test_conn();
        cookie_set(&conn, "ex.com", "k", "v", "/", false, false, None).unwrap();
        assert!(cookie_delete(&conn, "ex.com", "k", "/").unwrap());
        assert_eq!(cookie_get(&conn, "ex.com", "k", "/").unwrap(), None);
    }

    #[test]
    fn test_cookie_purge_expired() {
        let conn = test_conn();
        cookie_set(
            &conn,
            "ex.com",
            "valid",
            "ok",
            "/",
            false,
            false,
            Some(now() + 86400),
        )
        .unwrap();
        cookie_set(
            &conn,
            "ex.com",
            "expired",
            "old",
            "/",
            false,
            false,
            Some(now() - 86400),
        )
        .unwrap();
        let purged = cookie_purge_expired(&conn).unwrap();
        assert_eq!(purged, 1);
    }

    #[test]
    fn test_asset_store_load_delete() {
        let conn = test_conn();
        let data = b"hello world";
        let id = asset_store(&conn, "test", "hello.txt", data, Some("text/plain"), None).unwrap();
        let loaded = asset_load(&conn, &id).unwrap();
        assert!(loaded.is_some());
        let (loaded_data, ns, name, mime, size, _) = loaded.unwrap();
        assert_eq!(loaded_data, data);
        assert_eq!(ns, "test");
        assert_eq!(name, "hello.txt");
        assert_eq!(mime, Some("text/plain".to_string()));
        assert_eq!(size, 11);
        assert!(asset_delete(&conn, &id).unwrap());
        assert!(asset_load(&conn, &id).unwrap().is_none());
    }

    #[test]
    fn test_rkyv_store_load() {
        let conn = test_conn();
        let data = b"binary data for rkyv";
        let id = rkyv_store(&conn, "test_ns", data).unwrap();
        let loaded = rkyv_load(&conn, &id).unwrap();
        assert!(loaded.is_some());
        let (loaded_data, ns, _) = loaded.unwrap();
        assert_eq!(loaded_data, data);
        assert_eq!(ns, "test_ns");
    }

    #[test]
    fn test_rkyv_list() {
        let conn = test_conn();
        rkyv_store(&conn, "ns1", b"a").unwrap();
        rkyv_store(&conn, "ns1", b"b").unwrap();
        let entries = rkyv_list(&conn, "ns1").unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_store_stats() {
        let conn = test_conn();
        kv_set(&conn, "ns", "k", "v").unwrap();
        config_set(&conn, "sec", "k", "v", false).unwrap();
        let stats = store_stats(&conn).unwrap();
        assert_eq!(*stats.get("kv_store").unwrap_or(&0), 1);
        assert_eq!(*stats.get("config_entries").unwrap_or(&0), 1);
    }

    #[test]
    fn test_mime_for_extension() {
        assert_eq!(mime_for_extension(Some("png")), Some("image/png".into()));
        assert_eq!(
            mime_for_extension(Some("json")),
            Some("application/json".into())
        );
        assert_eq!(
            mime_for_extension(Some("unknown_ext")),
            Some("application/octet-stream".into())
        );
        assert_eq!(
            mime_for_extension(None),
            Some("application/octet-stream".into())
        );
    }
}
