use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct MemoryPlugin {
    db_path: PathBuf,
    _db: Mutex<()>,
}

impl MemoryPlugin {
    pub fn new() -> Self {
        let db_path = dirs::home_dir()
            .map(|h| h.join(".neotrix").join("memory.db"))
            .unwrap_or_else(|| PathBuf::from(".neotrix/memory.db"));
        Self {
            db_path,
            _db: Mutex::new(()),
        }
    }

    fn open_db(&self) -> Result<Connection, DomainError> {
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| DomainError {
                code: "DB_ERROR".into(),
                message: format!("创建数据目录失败: {}", e),
                recoverable: true,
            })?;
        }
        let conn = Connection::open(&self.db_path).map_err(|e| DomainError {
            code: "DB_ERROR".into(),
            message: format!("打开数据库失败: {}", e),
            recoverable: true,
        })?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| DomainError {
                code: "DB_ERROR".into(),
                message: format!("启用 WAL 失败: {}", e),
                recoverable: true,
            })?;
        let _ = conn.busy_timeout(std::time::Duration::from_secs(5));
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT,
                confidence REAL NOT NULL DEFAULT 0.5,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                accessed_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );
            CREATE INDEX IF NOT EXISTS idx_memories_kind ON memories(kind);
            CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at);",
        )
        .map_err(|e| DomainError {
            code: "DB_ERROR".into(),
            message: format!("初始化表失败: {}", e),
            recoverable: true,
        })?;
        Ok(conn)
    }
}

impl DomainPlugin for MemoryPlugin {
    fn name(&self) -> &str {
        "memory"
    }
    fn description(&self) -> &str {
        "记忆：管理、insights、context"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "list".into(),
                description: "列出记忆".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "search".into(),
                description: "搜索记忆".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "clear".into(),
                description: "清空记忆".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "stats".into(),
                description: "统计信息".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "timeline".into(),
                description: "时间线".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "export".into(),
                description: "导出记忆".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "import".into(),
                description: "导入记忆".into(),
                params: vec![],
                returns: "Value".into(),
            },
        ]
    }

    fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        let conn = self.open_db()?;

        match action {
            "list" => {
                let kind = args.get("kind").and_then(|v| v.as_str());
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;

                let (query, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(
                    k,
                ) = kind
                {
                    ("SELECT id, kind, content, confidence, created_at FROM memories WHERE kind = ?1 ORDER BY created_at DESC LIMIT ?2".into(),
                     vec![Box::new(k.to_string()), Box::new(limit as i64)])
                } else {
                    ("SELECT id, kind, content, confidence, created_at FROM memories ORDER BY created_at DESC LIMIT ?1".into(),
                     vec![Box::new(limit as i64)])
                };

                let mut stmt = conn.prepare(&query).map_err(|e| DomainError {
                    code: "DB_ERROR".into(),
                    message: format!("准备查询失败: {}", e),
                    recoverable: true,
                })?;

                let params_refs: Vec<&dyn rusqlite::types::ToSql> =
                    params.iter().map(|p| p.as_ref()).collect();
                let rows = stmt
                    .query_map(params_refs.as_slice(), |row| {
                        Ok(serde_json::json!({
                            "id": row.get::<_, String>(0)?,
                            "kind": row.get::<_, String>(1)?,
                            "content": row.get::<_, String>(2)?,
                            "confidence": row.get::<_, f64>(3)?,
                            "created_at": row.get::<_, i64>(4)?,
                        }))
                    })
                    .map_err(|e| DomainError {
                        code: "DB_ERROR".into(),
                        message: format!("查询失败: {}", e),
                        recoverable: true,
                    })?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();

                Ok(serde_json::json!({ "ok": true, "memories": rows }))
            }
            "search" => {
                let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

                let pattern = format!("%{}%", query);
                let mut stmt = conn
                    .prepare("SELECT id, kind, content, confidence, created_at FROM memories WHERE content LIKE ?1 ORDER BY confidence DESC LIMIT ?2")
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("准备查询失败: {}", e), recoverable: true })?;

                let rows = stmt
                    .query_map(rusqlite::params![pattern, limit as i64], |row| {
                        Ok(serde_json::json!({
                            "id": row.get::<_, String>(0)?,
                            "kind": row.get::<_, String>(1)?,
                            "content": row.get::<_, String>(2)?,
                            "confidence": row.get::<_, f64>(3)?,
                            "created_at": row.get::<_, i64>(4)?,
                        }))
                    })
                    .map_err(|e| DomainError {
                        code: "DB_ERROR".into(),
                        message: format!("查询失败: {}", e),
                        recoverable: true,
                    })?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();

                Ok(serde_json::json!({ "ok": true, "memories": rows }))
            }
            "clear" => {
                let kind = args.get("kind").and_then(|v| v.as_str());
                let count = if let Some(k) = kind {
                    conn.execute("DELETE FROM memories WHERE kind = ?1", [k])
                        .map_err(|e| DomainError {
                            code: "DB_ERROR".into(),
                            message: format!("清空失败: {}", e),
                            recoverable: true,
                        })?
                } else {
                    conn.execute("DELETE FROM memories", [])
                        .map_err(|e| DomainError {
                            code: "DB_ERROR".into(),
                            message: format!("清空失败: {}", e),
                            recoverable: true,
                        })?
                };
                Ok(serde_json::json!({ "ok": true, "deleted": count }))
            }
            "stats" => {
                let total: i64 = conn
                    .query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
                    .unwrap_or(0);
                let by_kind: Vec<(String, i64)> = conn
                    .prepare("SELECT kind, COUNT(*) FROM memories GROUP BY kind")
                    .map_err(|e| DomainError {
                        code: "DB_ERROR".into(),
                        message: format!("查询失败: {}", e),
                        recoverable: true,
                    })?
                    .query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                    })
                    .map_err(|e| DomainError {
                        code: "DB_ERROR".into(),
                        message: format!("查询失败: {}", e),
                        recoverable: true,
                    })?
                    .filter_map(|r| r.ok())
                    .collect();

                let avg_confidence: f64 = conn
                    .query_row("SELECT AVG(confidence) FROM memories", [], |row| row.get(0))
                    .unwrap_or(0.0);

                let total_size: i64 = conn
                    .query_row("SELECT SUM(LENGTH(content)) FROM memories", [], |row| {
                        row.get(0)
                    })
                    .unwrap_or(0);

                Ok(serde_json::json!({
                    "ok": true,
                    "total_entries": total,
                    "total_categories": by_kind.len(),
                    "by_kind": by_kind.into_iter().collect::<std::collections::HashMap<_, _>>(),
                    "avg_confidence": avg_confidence,
                    "memory_usage_bytes": total_size,
                }))
            }
            "timeline" => {
                let days = args.get("days").and_then(|v| v.as_u64()).unwrap_or(7) as i64;
                let cutoff = chrono::Utc::now().timestamp() - (days * 86400);

                let rows: Vec<(String, i64)> = conn
                    .prepare("SELECT DATE(created_at, 'unixepoch') as day, COUNT(*) FROM memories WHERE created_at > ?1 GROUP BY day ORDER BY day")
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .query_map([cutoff], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .filter_map(|r| r.ok())
                    .collect();

                Ok(serde_json::json!({
                    "ok": true,
                    "timeline": rows.into_iter().map(|(d, c)| serde_json::json!({ "date": d, "count": c })).collect::<Vec<_>>(),
                }))
            }
            "export" => {
                let format = args
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("json");
                let mut stmt = conn
                    .prepare("SELECT id, kind, content, confidence, created_at FROM memories ORDER BY created_at DESC")
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("准备查询失败: {}", e), recoverable: true })?;

                let memories: Vec<serde_json::Value> = stmt
                    .query_map([], |row| {
                        Ok(serde_json::json!({
                            "id": row.get::<_, String>(0)?,
                            "kind": row.get::<_, String>(1)?,
                            "content": row.get::<_, String>(2)?,
                            "confidence": row.get::<_, f64>(3)?,
                            "created_at": row.get::<_, i64>(4)?,
                        }))
                    })
                    .map_err(|e| DomainError {
                        code: "DB_ERROR".into(),
                        message: format!("查询失败: {}", e),
                        recoverable: true,
                    })?
                    .filter_map(|r| r.ok())
                    .collect();

                if format == "json" {
                    Ok(serde_json::json!({
                        "ok": true,
                        "data": serde_json::json!({
                            "memories": memories,
                            "exported_at": chrono::Utc::now().to_rfc3339(),
                            "count": memories.len(),
                        }),
                    }))
                } else {
                    Ok(serde_json::json!({
                        "ok": true,
                        "data": serde_json::to_string(&serde_json::json!({
                            "memories": memories,
                            "exported_at": chrono::Utc::now().to_rfc3339(),
                            "count": memories.len(),
                        })).unwrap_or_default(),
                    }))
                }
            }
            "import" => {
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'content'".into(),
                        recoverable: true,
                    })?;

                let data: serde_json::Value =
                    serde_json::from_str(content).map_err(|e| DomainError {
                        code: "PARSE_ERROR".into(),
                        message: format!("JSON 解析失败: {}", e),
                        recoverable: true,
                    })?;

                let memories =
                    data.get("memories")
                        .and_then(|v| v.as_array())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_DATA".into(),
                            message: "missing 'memories' array".into(),
                            recoverable: true,
                        })?;

                let mut imported = 0;
                for mem in memories {
                    let default_id = uuid::Uuid::new_v4().to_string();
                    let id = mem
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&default_id);
                    let kind = mem
                        .get("kind")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let content = mem.get("content").and_then(|v| v.as_str()).unwrap_or("");
                    let confidence = mem
                        .get("confidence")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.5);

                    conn.execute(
                        "INSERT OR REPLACE INTO memories (id, kind, content, confidence, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                        rusqlite::params![id, kind, content, confidence, chrono::Utc::now().timestamp()],
                    )
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("插入失败: {}", e), recoverable: true })?;
                    imported += 1;
                }

                Ok(serde_json::json!({
                    "ok": true,
                    "imported": imported,
                }))
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}
