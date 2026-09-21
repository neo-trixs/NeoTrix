use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use async_trait::async_trait;
use std::sync::Arc;

pub struct MemoryPlugin {
    db_pool: Arc<crate::db_pool::DbPool>,
}

impl MemoryPlugin {
    pub fn new(db_pool: Arc<crate::db_pool::DbPool>) -> Self {
        if let Err(e) = db_pool.init_schema(
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
        ) {
            tracing::warn!("schema init: {e}");
        }
        Self { db_pool }
    }
}

#[async_trait]
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
            ActionSpec {
                name: "save_api_key".into(),
                description: "保存 API Key（本地文件）".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "has_api_key".into(),
                description: "是否存在已保存的 API Key".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "delete_api_key".into(),
                description: "删除已保存的 API Key".into(),
                params: vec![],
                returns: "Value".into(),
            },
        ]
    }

    #[tracing::instrument(skip(self, args), fields(action = %action))]
    async fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        let conn = self.db_pool.get().map_err(DomainError::from)?;

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

                // Bare array: frontend MemoryEntry[] contract.
                Ok(serde_json::Value::Array(rows))
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

                // Bare array: frontend MemoryEntry[] contract.
                Ok(serde_json::Value::Array(rows))
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
                // Bare count: frontend expects a number.
                Ok(serde_json::json!(count))
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
                let cutoff = chrono::Utc::now().timestamp() - (days * crate::constants::SECS_PER_DAY);

                // Per-day created counts.
                let created: Vec<(String, i64)> = conn
                    .prepare("SELECT DATE(created_at, 'unixepoch') as day, COUNT(*) FROM memories WHERE created_at > ?1 GROUP BY day ORDER BY day")
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .query_map([cutoff], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .filter_map(|r| r.ok())
                    .collect();

                // Per-day access counts (accessed_at is bumped on read; rows
                // never re-read since insert share their created timestamp).
                let accessed: std::collections::HashMap<String, i64> = conn
                    .prepare("SELECT DATE(accessed_at, 'unixepoch') as day, COUNT(*) FROM memories WHERE accessed_at > ?1 GROUP BY day")
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .query_map([cutoff], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
                    .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                    .filter_map(|r| r.ok())
                    .collect();

                // Top kind per day as the "hot topic" label.
                let top_kinds: std::collections::HashMap<String, String> = {
                    let mut best: std::collections::HashMap<String, (String, i64)> =
                        std::collections::HashMap::new();
                    let rows: Vec<(String, String, i64)> = conn
                        .prepare("SELECT DATE(created_at, 'unixepoch') as day, kind, COUNT(*) FROM memories WHERE created_at > ?1 GROUP BY day, kind")
                        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                        .query_map([cutoff], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?)))
                        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("查询失败: {}", e), recoverable: true })?
                        .filter_map(|r| r.ok())
                        .collect();
                    for (day, kind, count) in rows {
                        match best.get(&day) {
                            Some((_, c)) if *c >= count => {}
                            _ => {
                                best.insert(day, (kind, count));
                            }
                        }
                    }
                    best.into_iter().map(|(d, (k, _))| (d, k)).collect()
                };

                // Bare array matching the frontend TimelineEntry contract.
                let timeline: Vec<serde_json::Value> = created
                    .into_iter()
                    .map(|(date, entries_created)| {
                        let entries_accessed =
                            accessed.get(&date).copied().unwrap_or(0);
                        let top_topic = top_kinds
                            .get(&date)
                            .cloned()
                            .unwrap_or_default();
                        serde_json::json!({
                            "date": date,
                            "entries_created": entries_created,
                            "entries_accessed": entries_accessed,
                            "top_topic": top_topic,
                        })
                    })
                    .collect();
                Ok(serde_json::Value::Array(timeline))
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

                // Both consumers (SettingsModal file save, ChatShellProto Blob
                // download) need a JSON *string*; the non-json branch keeps a
                // serialized string as well for symmetry.
                let payload = serde_json::json!({
                    "memories": memories,
                    "exported_at": chrono::Utc::now().to_rfc3339(),
                    "count": memories.len(),
                });
                if format == "json" {
                    Ok(serde_json::Value::String(
                        serde_json::to_string(&payload).unwrap_or_default(),
                    ))
                } else {
                    Ok(serde_json::Value::String(
                        serde_json::to_string(&payload).unwrap_or_default(),
                    ))
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
            // API Key persistence (local file under base_dir; never logged).
            "save_api_key" => {
                let key = args
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'key'".into(),
                        recoverable: true,
                    })?;
                let dir = crate::config::AppConfig::base_dir().ok_or_else(|| {
                    DomainError {
                        code: "CONFIG_DIR_ERROR".into(),
                        message: "Cannot determine base directory".into(),
                        recoverable: false,
                    }
                })?;
                let path = dir.join("api_key");
                crate::atomic_io::ensure_parent_dir(&path).map_err(|e| DomainError {
                    code: "IO_ERROR".into(),
                    message: format!("创建目录失败: {e}"),
                    recoverable: true,
                })?;
                crate::atomic_io::write_atomic(&path, key.as_bytes()).map_err(|e| {
                    DomainError {
                        code: "IO_ERROR".into(),
                        message: format!("写入失败: {e}"),
                        recoverable: true,
                    }
                })?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "has_api_key" => {
                let has = crate::config::AppConfig::base_dir()
                    .map(|d| d.join("api_key"))
                    .filter(|p| p.exists())
                    .and_then(|p| {
                        crate::atomic_io::read_with_fallback(&p)
                            .ok()
                            .and_then(|b| String::from_utf8(b).ok())
                    })
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false);
                // Bare bool: frontend hasApiKey() contract.
                Ok(serde_json::json!(has))
            }
            "delete_api_key" => {
                if let Some(path) = crate::config::AppConfig::base_dir().map(|d| d.join("api_key"))
                {
                    if path.exists() {
                        std::fs::remove_file(&path).map_err(|e| DomainError {
                            code: "IO_ERROR".into(),
                            message: format!("删除失败: {e}"),
                            recoverable: true,
                        })?;
                    }
                }
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
