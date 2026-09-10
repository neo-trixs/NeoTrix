use crate::domain::{DomainPlugin, ActionSpec, DomainError, serde_json};
use std::path::PathBuf;
use std::sync::Mutex;
use rusqlite::Connection;

pub struct KbPlugin {
    db_path: PathBuf,
    _db: Mutex<()>,
}

impl KbPlugin {
    pub fn new() -> Self {
        let db_path = dirs::home_dir()
            .map(|h| h.join(".neotrix").join("knowledge.db"))
            .unwrap_or_else(|| PathBuf::from(".neotrix/knowledge.db"));
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
            "CREATE TABLE IF NOT EXISTS kv_store (
                namespace TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                PRIMARY KEY (namespace, key)
            );
            CREATE TABLE IF NOT EXISTS nodes (
                id TEXT PRIMARY KEY,
                label TEXT NOT NULL,
                kind TEXT NOT NULL,
                data TEXT,
                library TEXT NOT NULL DEFAULT 'default',
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );
            CREATE TABLE IF NOT EXISTS edges (
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                relation TEXT NOT NULL,
                data TEXT,
                PRIMARY KEY (source, target, relation)
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(label, kind, content=nodes, content_rowid=rowid);
            CREATE TABLE IF NOT EXISTS kb_libraries (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );",
        )
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("初始化表失败: {}", e), recoverable: true })?;
        Ok(conn)
    }
}

impl DomainPlugin for KbPlugin {
    fn name(&self) -> &str { "kb" }
    fn description(&self) -> &str { "知识库：搜索、图谱、KV、地理" }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec { name: "search".into(), description: "搜索知识库".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "get".into(), description: "获取节点".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "graph".into(), description: "获取图谱".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "stats".into(), description: "统计信息".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "kv_set".into(), description: "设置KV".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "kv_get".into(), description: "获取KV".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "kv_list".into(), description: "列出KV".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "doc_ingest".into(), description: "导入文档".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "doc_list".into(), description: "列出文档".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "doc_delete".into(), description: "删除文档".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "doc_reindex".into(), description: "重建索引".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "library_create".into(), description: "创建知识库".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "library_list".into(), description: "列出知识库".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "library_rename".into(), description: "重命名知识库".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "library_delete".into(), description: "删除知识库".into(), params: vec![], returns: "Value".into() },
        ]
    }

    fn call(&self, action: &str, args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        let conn = self.open_db()?;
        
        match action {
            "search" => {
                let query = args.get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let limit = args.get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(20) as usize;
                
                let mut stmt = conn
                    .prepare("SELECT id, label, kind FROM nodes WHERE label LIKE ?1 OR kind LIKE ?1 LIMIT ?2")
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("准备查询失败: {}", e), recoverable: true })?;
                
                let pattern = format!("%{}%", query);
                let rows = stmt.query_map(rusqlite::params![pattern, limit as i64], |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, String>(0)?,
                        "label": row.get::<_, String>(1)?,
                        "kind": row.get::<_, String>(2)?,
                    }))
                })
                .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                .filter_map(|r| r.ok())
                .collect::<Vec<_>>();
                
                Ok(serde_json::json!({ "ok": true, "results": rows }))
            }
            "get" => {
                let id = args.get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "缺少 id 参数".into(), recoverable: true })?;
                
                let result = conn.query_row(
                    "SELECT id, label, kind, data FROM nodes WHERE id = ?1",
                    [id],
                    |row| {
                        Ok(serde_json::json!({
                            "id": row.get::<_, String>(0)?,
                            "label": row.get::<_, String>(1)?,
                            "kind": row.get::<_, String>(2)?,
                            "data": row.get::<_, Option<String>>(3)?.and_then(|s| {
                                serde_json::from_str::<serde_json::Value>(&s).ok()
                            }),
                        }))
                    },
                );
                
                match result {
                    Ok(node) => Ok(serde_json::json!({ "ok": true, "node": node })),
                    Err(_) => Ok(serde_json::json!({ "ok": true, "node": null })),
                }
            }
            "graph" => {
                let nodes: Vec<serde_json::Value> = conn
                    .prepare("SELECT id, label, kind FROM nodes LIMIT 100")
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .query_map([], |row| {
                        Ok(serde_json::json!({
                            "id": row.get::<_, String>(0)?,
                            "label": row.get::<_, String>(1)?,
                            "kind": row.get::<_, String>(2)?,
                        }))
                    })
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .filter_map(|r| r.ok())
                    .collect();
                
                let edges: Vec<serde_json::Value> = conn
                    .prepare("SELECT source, target, relation FROM edges LIMIT 200")
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .query_map([], |row| {
                        Ok(serde_json::json!({
                            "source": row.get::<_, String>(0)?,
                            "target": row.get::<_, String>(1)?,
                            "relation": row.get::<_, String>(2)?,
                        }))
                    })
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .filter_map(|r| r.ok())
                    .collect();
                
                Ok(serde_json::json!({ "ok": true, "nodes": nodes, "edges": edges }))
            }
            "stats" => {
                let node_count: i64 = conn.query_row("SELECT COUNT(*) FROM nodes", [], |row| row.get(0))
                    .unwrap_or(0);
                let edge_count: i64 = conn.query_row("SELECT COUNT(*) FROM edges", [], |row| row.get(0))
                    .unwrap_or(0);
                let kv_count: i64 = conn.query_row("SELECT COUNT(*) FROM kv_store", [], |row| row.get(0))
                    .unwrap_or(0);
                
                Ok(serde_json::json!({
                    "ok": true,
                    "node_count": node_count,
                    "edge_count": edge_count,
                    "kv_count": kv_count,
                }))
            }
            "kv_set" => {
                let namespace = args.get("namespace")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "缺少 namespace 参数".into(), recoverable: true })?;
                let key = args.get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "缺少 key 参数".into(), recoverable: true })?;
                let value = args.get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                conn.execute(
                    "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, strftime('%s', 'now'))",
                    rusqlite::params![namespace, key, value],
                )
                .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("设置KV失败: {}", e), recoverable: true })?;
                
                Ok(serde_json::json!({ "ok": true }))
            }
            "kv_get" => {
                let namespace = args.get("namespace")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "缺少 namespace 参数".into(), recoverable: true })?;
                let key = args.get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "缺少 key 参数".into(), recoverable: true })?;
                
                let result = conn.query_row(
                    "SELECT value FROM kv_store WHERE namespace = ?1 AND key = ?2",
                    rusqlite::params![namespace, key],
                    |row| row.get::<_, String>(0),
                );
                
                match result {
                    Ok(value) => Ok(serde_json::json!({ "ok": true, "value": value })),
                    Err(_) => Ok(serde_json::json!({ "ok": true, "value": null })),
                }
            }
            "kv_list" => {
                let namespace = args.get("namespace")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                let pattern = format!("%{}%", namespace);
                let rows: Vec<(String, String)> = conn
                    .prepare("SELECT key, value FROM kv_store WHERE namespace LIKE ?1")
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .query_map([pattern], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    })
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .filter_map(|r| r.ok())
                    .collect();
                
                Ok(serde_json::json!({ "ok": true, "items": rows }))
            }
            "doc_ingest" => {
                let id = args.get("id").and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                let label = args.get("label").and_then(|v| v.as_str()).unwrap_or("");
                let kind = args.get("kind").and_then(|v| v.as_str()).unwrap_or("document");
                let data = args.get("data").and_then(|v| v.as_str()).unwrap_or("");
                let library = args.get("library").and_then(|v| v.as_str()).unwrap_or("default");
                conn.execute(
                    "INSERT OR REPLACE INTO nodes (id, label, kind, data, library) VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![id, label, kind, data, library],
                ).map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("导入失败: {}", e), recoverable: true })?;
                Ok(serde_json::json!({ "ok": true, "id": id }))
            }
            "doc_list" => {
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
                let rows: Vec<serde_json::Value> = conn
                    .prepare("SELECT id, label, kind, created_at FROM nodes ORDER BY created_at DESC LIMIT ?1")
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .query_map([limit as i64], |row| {
                        Ok(serde_json::json!({
                            "id": row.get::<_, String>(0)?,
                            "label": row.get::<_, String>(1)?,
                            "kind": row.get::<_, String>(2)?,
                            "created_at": row.get::<_, i64>(3)?,
                        }))
                    })
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .filter_map(|r| r.ok())
                    .collect();
                Ok(serde_json::json!({ "ok": true, "items": rows }))
            }
            "doc_delete" => {
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "缺少 id 参数".into(), recoverable: true })?;
                conn.execute("DELETE FROM nodes WHERE id = ?1", [id])
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("删除失败: {}", e), recoverable: true })?;
                conn.execute("DELETE FROM edges WHERE source = ?1 OR target = ?1", [id]).ok();
                Ok(serde_json::json!({ "ok": true }))
            }
            "doc_reindex" => {
                // Rebuild FTS index by re-inserting all nodes
                let nodes: Vec<(String, String, String)> = {
                    let mut stmt = conn.prepare("SELECT id, label, kind FROM nodes")
                        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?;
                    stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)))
                        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                        .filter_map(|r| r.ok())
                        .collect()
                };
                conn.execute("DELETE FROM nodes_fts", []).ok();
                let mut reindexed = 0;
                for (id, label, kind) in &nodes {
                    conn.execute(
                        "INSERT INTO nodes_fts (rowid, label, kind) SELECT rowid, ?1, ?2 FROM nodes WHERE id = ?3",
                        rusqlite::params![label, kind, id],
                    ).ok();
                    reindexed += 1;
                }
                Ok(serde_json::json!({ "ok": true, "reindexed": reindexed }))
            }
            "library_create" => {
                let name = args.get("name").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "缺少 name 参数".into(), recoverable: true })?;
                let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("");
                let id = args.get("id").and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                let now = chrono::Utc::now().timestamp();
                conn.execute(
                    "INSERT INTO kb_libraries (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![id, name, description, now, now],
                ).map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("创建知识库失败: {}", e), recoverable: true })?;
                Ok(serde_json::json!({ "ok": true, "id": id }))
            }
            "library_list" => {
                let rows: Vec<serde_json::Value> = conn
                    .prepare("SELECT id, name, description, created_at, updated_at FROM kb_libraries ORDER BY updated_at DESC")
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .query_map([], |row| {
                        let id: String = row.get(0)?;
                        let name: String = row.get(1)?;
                        let description: String = row.get(2)?;
                        let created_at: i64 = row.get(3)?;
                        let updated_at: i64 = row.get(4)?;
                        // Count documents in this library
                        Ok(serde_json::json!({
                            "id": id,
                            "name": name,
                            "description": description,
                            "created_at": created_at,
                            "updated_at": updated_at,
                        }))
                    })
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .filter_map(|r| r.ok())
                    .collect();
                // Enrich with doc counts
                let mut enriched = Vec::new();
                for mut lib in rows {
                    let lib_id = lib["id"].as_str().unwrap_or("");
                    let lib_name = lib["name"].as_str().unwrap_or("");
                    let doc_count: i64 = conn.query_row(
                        "SELECT COUNT(*) FROM nodes WHERE library = ?1",
                        [lib_name],
                        |r| r.get(0),
                    ).unwrap_or(0);
                    let chunk_count: i64 = conn.query_row(
                        "SELECT COALESCE(SUM(LENGTH(data)), 0) FROM nodes WHERE library = ?1",
                        [lib_name],
                        |r| r.get(0),
                    ).unwrap_or(0);
                    lib["doc_count"] = serde_json::json!(doc_count);
                    lib["chunk_count"] = serde_json::json!(chunk_count);
                    enriched.push(lib);
                }
                Ok(serde_json::json!({ "ok": true, "libraries": enriched }))
            }
            "library_rename" => {
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "缺少 id 参数".into(), recoverable: true })?;
                let name = args.get("name").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "缺少 name 参数".into(), recoverable: true })?;
                let now = chrono::Utc::now().timestamp();
                conn.execute(
                    "UPDATE kb_libraries SET name = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![name, now, id],
                ).map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("重命名失败: {}", e), recoverable: true })?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "library_delete" => {
                let id = args.get("id").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "缺少 id 参数".into(), recoverable: true })?;
                // Get library name before deleting
                let lib_name: String = conn.query_row(
                    "SELECT name FROM kb_libraries WHERE id = ?1",
                    [id],
                    |r| r.get(0),
                ).map_err(|e| DomainError { code: "NOT_FOUND".into(), message: format!("知识库不存在: {}", e), recoverable: true })?;
                // Delete documents in this library
                conn.execute("DELETE FROM nodes WHERE library = ?1", [&lib_name])
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("删除文档失败: {}", e), recoverable: true })?;
                // Delete library
                conn.execute("DELETE FROM kb_libraries WHERE id = ?1", [id])
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("删除知识库失败: {}", e), recoverable: true })?;
                Ok(serde_json::json!({ "ok": true }))
            }
            _ => Err(DomainError { code: "UNKNOWN_ACTION".into(), message: format!("Unknown action: {}", action), recoverable: true }),
        }
    }
}
