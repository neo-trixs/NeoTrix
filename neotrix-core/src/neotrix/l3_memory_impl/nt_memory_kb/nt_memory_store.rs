use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};
use uuid::Uuid;

use super::nt_memory_types::*;

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// 无事务的 nodes+nodes_fts 双写核心。调用方必须自管事务 (批量路径复用)。
pub fn insert_node_rows(conn: &Connection, node: &KnowledgeNode) -> rusqlite::Result<()> {
    let temporal_json = node.temporal.as_ref().map(|t| {
        serde_json::to_string(t).unwrap_or_else(|_| "{}".to_string())
    });
    conn.execute(
        "INSERT INTO nodes (id, node_type, title, summary, content, url, domain, language,
            confidence, importance, created_at, updated_at, access_count, metadata,
            data_tier, temporal, supersedes, source_episode, tier)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
        params![
            node.id,
            node.node_type.as_str(),
            node.title,
            node.summary,
            node.content,
            node.url,
            node.domain,
            node.language,
            node.confidence,
            node.importance,
            node.created_at,
            node.updated_at,
            node.access_count,
            node.metadata.as_ref().map(|m| m.to_string()),
            "core",
            temporal_json,
            node.supersedes,
            node.source_episode,
            "warm",
        ],
    )?;

    let summary = node.summary.as_deref().unwrap_or("");
    let content = node.content.as_deref().unwrap_or("");
    let domain = node.domain.as_deref().unwrap_or("");
    conn.execute(
        "INSERT INTO nodes_fts (rowid, title, summary, content, domain)
         VALUES (last_insert_rowid(), ?1, ?2, ?3, ?4)",
        params![node.title, summary, content, domain],
    )?;
    Ok(())
}

/// 无事务的 insert-or-get: 批量摄取路径复用 (外层事务由调用方负责)。
pub fn insert_or_get_node_rows(
    conn: &Connection,
    title: &str,
    node_type: NodeType,
    summary: Option<&str>,
    url: Option<&str>,
    domain: Option<&str>,
) -> rusqlite::Result<String> {
    if let Some(url) = url {
        if let Some(existing) = find_node_by_url(conn, url)? {
            return Ok(existing.id);
        }
    } else if let Some(existing) = find_node_by_title_and_type(conn, title, &node_type)? {
        return Ok(existing.id);
    }

    let id = Uuid::new_v4().to_string();
    let ts = now();
    let node = KnowledgeNode {
        id: id.clone(),
        node_type,
        title: title.to_string(),
        summary: summary.map(|s| s.to_string()),
        content: summary.map(|s| s.to_string()),
        url: url.map(|s| s.to_string()),
        domain: domain.map(|s| s.to_string()),
        language: "en".into(),
        confidence: 1.0,
        importance: 0.5,
        created_at: ts,
        updated_at: ts,
        access_count: 0,
        metadata: None,
        temporal: None,
        supersedes: None,
        source_episode: None,
    };
    insert_node_rows(conn, &node)?;
    Ok(id)
}

pub fn insert_node(conn: &Connection, node: &KnowledgeNode) -> rusqlite::Result<()> {
    let _temporal_json = node.temporal.as_ref().map(|t| {
        serde_json::to_string(t).unwrap_or_else(|_| "{}".to_string())
    });
    // 事务：nodes + nodes_fts 双写保持一致性，crash 不残留孤立 FTS 行
    let tx = conn.unchecked_transaction()?;
    insert_node_rows(&tx, node)?;
    tx.commit()
}

pub fn update_node(conn: &Connection, node: &KnowledgeNode) -> rusqlite::Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE nodes SET node_type=?2, title=?3, summary=?4, content=?5, url=?6,
            domain=?7, language=?8, confidence=?9, importance=?10, updated_at=?11, metadata=?12
         WHERE id=?1",
        params![
            node.id,
            node.node_type.as_str(),
            node.title,
            node.summary,
            node.content,
            node.url,
            node.domain,
            node.language,
            node.confidence,
            node.importance,
            now(),
            node.metadata.as_ref().map(|m| m.to_string()),
        ],
    )?;
    let summary = node.summary.as_deref().unwrap_or("");
    let content = node.content.as_deref().unwrap_or("");
    let domain = node.domain.as_deref().unwrap_or("");
    tx.execute(
        "UPDATE nodes_fts SET title=?2, summary=?3, content=?4, domain=?5
         WHERE rowid = (SELECT rowid FROM nodes WHERE id=?1)",
        params![node.id, node.title, summary, content, domain],
    )?;
    tx.commit()
}

pub fn update_node_metadata(conn: &Connection, id: &str, metadata: &serde_json::Value) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE nodes SET metadata=?1, updated_at=?2 WHERE id=?3",
        params![metadata.to_string(), now(), id],
    )?;
    Ok(())
}

pub fn get_node(conn: &Connection, id: &str) -> rusqlite::Result<Option<KnowledgeNode>> {
    let mut stmt = conn.prepare(
        "SELECT id, node_type, title, summary, content, url, domain, language,
            confidence, importance, created_at, updated_at, access_count, metadata,
            supersedes
         FROM nodes WHERE id=?1",
    )?;

    let mut rows = stmt.query(params![id])?;
    match rows.next()? {
        Some(row) => {
            conn.execute("UPDATE nodes SET access_count = access_count + 1 WHERE id=?1", params![id])?;
            Ok(Some(KnowledgeNode {
                id: row.get(0)?,
                node_type: NodeType::from_str(&row.get::<_, String>(1)?),
                title: row.get(2)?,
                summary: row.get(3)?,
                content: row.get(4)?,
                url: row.get(5)?,
                domain: row.get(6)?,
                language: row.get(7)?,
                confidence: row.get(8)?,
                importance: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
                access_count: row.get::<_, i64>(12)? + 1,
                metadata: row.get::<_, Option<String>>(13)?.and_then(|m| serde_json::from_str(&m).ok()),
                temporal: None,
                supersedes: row.get(14)?,
                source_episode: None,
            }))
        }
        None => Ok(None),
    }
}

pub fn find_node_by_title_and_type(conn: &Connection, title: &str, node_type: &NodeType) -> rusqlite::Result<Option<KnowledgeNode>> {
    let mut stmt = conn.prepare(
        "SELECT id, node_type, title, summary, content, url, domain, language,
            confidence, importance, created_at, updated_at, access_count, metadata,
            supersedes
         FROM nodes WHERE title=?1 AND node_type=?2 AND url IS NULL LIMIT 1",
    )?;
    let mut rows = stmt.query(params![title, node_type.as_str()])?;
    match rows.next()? {
        Some(row) => Ok(Some(KnowledgeNode {
            id: row.get(0)?,
            node_type: NodeType::from_str(&row.get::<_, String>(1)?),
            title: row.get(2)?,
            summary: row.get(3)?,
            content: row.get(4)?,
            url: row.get(5)?,
            domain: row.get(6)?,
            language: row.get(7)?,
            confidence: row.get(8)?,
            importance: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            access_count: row.get(12)?,
            metadata: row.get::<_, Option<String>>(13)?.and_then(|m| serde_json::from_str(&m).ok()),
            temporal: None,
            supersedes: row.get(14)?,
            source_episode: None,
        })),
        None => Ok(None),
    }
}

/// 合并相同标题的重复节点 (将指定节点的边迁移到保留节点)
pub fn merge_duplicate_nodes(conn: &Connection, keep_id: &str, remove_id: &str) -> rusqlite::Result<()> {
    // 事务：边重映射 + 节点删除必须原子，否则残留指向已删节点的边
    let tx = conn.unchecked_transaction()?;
    // Remove edges from remove_id that already exist on keep_id (avoid UNIQUE conflict)
    tx.execute(
        "DELETE FROM edges WHERE source_id=?1 AND (target_id, relation_type) IN \
         (SELECT target_id, relation_type FROM edges WHERE source_id=?2)",
        params![remove_id, keep_id],
    )?;
    tx.execute(
        "DELETE FROM edges WHERE target_id=?1 AND (source_id, relation_type) IN \
         (SELECT source_id, relation_type FROM edges WHERE target_id=?2)",
        params![remove_id, keep_id],
    )?;
    tx.execute("UPDATE edges SET source_id=?1 WHERE source_id=?2", params![keep_id, remove_id])?;
    tx.execute("UPDATE edges SET target_id=?1 WHERE target_id=?2", params![keep_id, remove_id])?;
    tx.execute("DELETE FROM nodes WHERE id=?1", params![remove_id])?;
    tx.execute("DELETE FROM nodes_fts WHERE rowid = (SELECT rowid FROM nodes WHERE id=?1)", params![remove_id])?;
    tx.commit()
}

/// 查找并合并所有重复标题的节点 (仅无 URL 节点)
pub fn dedup_nodes(conn: &Connection) -> rusqlite::Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT id, title, node_type FROM nodes WHERE url IS NULL ORDER BY title"
    )?;
    let rows: Vec<(String, String, String)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?.filter_map(|r| r.ok()).collect();

    let mut merged = 0usize;
    let mut seen: std::collections::HashMap<(String, String), String> = std::collections::HashMap::new();
    for (id, title, ntype) in &rows {
        let key = (title.to_lowercase(), ntype.clone());
        if let Some(existing_id) = seen.get(&key) {
            merge_duplicate_nodes(conn, existing_id, id)?;
            merged += 1;
        } else {
            seen.insert(key, id.clone());
        }
    }
    Ok(merged)
}

pub fn find_node_by_url(conn: &Connection, url: &str) -> rusqlite::Result<Option<KnowledgeNode>> {
    let mut stmt = conn.prepare(
        "SELECT id, node_type, title, summary, content, url, domain, language,
            confidence, importance, created_at, updated_at, access_count, metadata,
            supersedes
         FROM nodes WHERE url=?1 LIMIT 1",
    )?;
    let mut rows = stmt.query(params![url])?;
    match rows.next()? {
        Some(row) => Ok(Some(KnowledgeNode {
            id: row.get(0)?,
            node_type: NodeType::from_str(&row.get::<_, String>(1)?),
            title: row.get(2)?,
            summary: row.get(3)?,
            content: row.get(4)?,
            url: row.get(5)?,
            domain: row.get(6)?,
            language: row.get(7)?,
            confidence: row.get(8)?,
            importance: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            access_count: row.get(12)?,
            metadata: row.get::<_, Option<String>>(13)?.and_then(|m| serde_json::from_str(&m).ok()),
            temporal: None,
            supersedes: row.get(14)?,
            source_episode: None,
        })),
        None => Ok(None),
    }
}

pub fn delete_node(conn: &Connection, id: &str) -> rusqlite::Result<bool> {
    // 事务：先删 FTS 行再删节点行，crash 不留孤立 FTS 行
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM nodes_fts WHERE rowid = (SELECT rowid FROM nodes WHERE id=?1)", params![id])?;
    let affected = tx.execute("DELETE FROM nodes WHERE id=?1", params![id])?;
    tx.commit()?;
    Ok(affected > 0)
}

pub fn delete_edge(conn: &Connection, id: &str) -> rusqlite::Result<bool> {
    let affected = conn.execute("DELETE FROM edges WHERE id=?1", params![id])?;
    Ok(affected > 0)
}

pub fn insert_edge(conn: &Connection, edge: &KnowledgeEdge) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO edges (id, source_id, target_id, relation_type, weight, description, created_at, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            edge.id,
            edge.source_id,
            edge.target_id,
            edge.relation_type.as_str(),
            edge.weight,
            edge.description,
            edge.created_at,
            edge.metadata.as_ref().map(|m| m.to_string()),
        ],
    )?;
    Ok(())
}

pub fn get_edges_for_node(conn: &Connection, node_id: &str) -> rusqlite::Result<Vec<KnowledgeEdge>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_id, target_id, relation_type, weight, description, created_at, metadata
         FROM edges WHERE source_id=?1 OR target_id=?1",
    )?;
    let rows = stmt.query_map(params![node_id], |row| {
        Ok(KnowledgeEdge {
            id: row.get(0)?,
            source_id: row.get(1)?,
            target_id: row.get(2)?,
            relation_type: RelationType::from_str(&row.get::<_, String>(3)?),
            weight: row.get(4)?,
            description: row.get(5)?,
            created_at: row.get(6)?,
            metadata: row.get::<_, Option<String>>(7)?.and_then(|m| serde_json::from_str(&m).ok()),
        })
    })?;
    rows.collect()
}

pub fn upsert_crawl_queue(conn: &Connection, url: &str, depth: i64, domain: &str, priority: i64, discovered_at: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO crawl_queue (id, url, depth, domain, priority, status, discovered_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6)",
        params![Uuid::new_v4().to_string(), url, depth, domain, priority, discovered_at],
    )?;
    Ok(())
}

pub fn claim_hf_pending_url(conn: &Connection) -> rusqlite::Result<Option<CrawlQueueItem>> {
    claim_pending_url_where(conn, "domain='huggingface.co' AND status='pending'")
}

pub fn claim_next_crawl_url(conn: &Connection) -> rusqlite::Result<Option<CrawlQueueItem>> {
    claim_pending_url_where(conn, "status='pending'")
}

fn claim_pending_url_where(conn: &Connection, where_clause: &str) -> rusqlite::Result<Option<CrawlQueueItem>> {
    let sql = format!(
        "SELECT id, url, depth, domain, priority, status, discovered_at, last_attempt, retry_count, error_message
         FROM crawl_queue
         WHERE {}
         ORDER BY priority DESC, discovered_at ASC
         LIMIT 1",
        where_clause
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query([])?;
    match rows.next()? {
        Some(row) => {
            let item = CrawlQueueItem {
                id: row.get(0)?,
                url: row.get(1)?,
                depth: row.get(2)?,
                domain: row.get(3)?,
                priority: row.get(4)?,
                status: row.get(5)?,
                discovered_at: row.get(6)?,
                last_attempt: row.get(7)?,
                retry_count: row.get(8)?,
                error_message: row.get(9)?,
            };
            conn.execute(
                "UPDATE crawl_queue SET status='processing', last_attempt=?1 WHERE id=?2",
                params![now(), item.id],
            )?;
            Ok(Some(item))
        }
        None => Ok(None),
    }
}

pub fn mark_crawl_complete(conn: &Connection, id: &str, success: bool, error: Option<&str>) -> rusqlite::Result<()> {
    if success {
        conn.execute(
            "UPDATE crawl_queue SET status='completed' WHERE id=?1",
            params![id],
        )?;
    } else {
        conn.execute(
            "UPDATE crawl_queue SET status='failed', retry_count=retry_count+1, error_message=?2 WHERE id=?1",
            params![id, error],
        )?;
    }
    Ok(())
}

pub fn get_stats(conn: &Connection) -> Result<KnowledgeStats, rusqlite::Error> {
    let total_nodes: i64 = conn.query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))?;
    let total_edges: i64 = conn.query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0))?;

    let mut type_stmt = conn.prepare("SELECT node_type, COUNT(*) FROM nodes GROUP BY node_type ORDER BY COUNT(*) DESC")?;
    let by_type: Vec<(String, i64)> = {
        let rows = type_stmt.query_map([], |r| {
            let t: String = r.get(0)?;
            let c: i64 = r.get(1)?;
            Ok((t, c))
        })?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let mut domain_stmt = conn.prepare("SELECT domain, COUNT(*) FROM nodes WHERE domain IS NOT NULL GROUP BY domain ORDER BY COUNT(*) DESC LIMIT 20")?;
    let by_domain: Vec<(String, i64)> = {
        let rows = domain_stmt.query_map([], |r| {
            let t: String = r.get(0)?;
            let c: i64 = r.get(1)?;
            Ok((t, c))
        })?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let crawl_pending: i64 = conn.query_row("SELECT COUNT(*) FROM crawl_queue WHERE status='pending'", [], |r| r.get(0))?;
    let crawl_completed: i64 = conn.query_row("SELECT COUNT(*) FROM crawl_queue WHERE status='completed'", [], |r| r.get(0))?;

    let db_size: i64 = conn
        .query_row("SELECT COALESCE(SUM(pgsize), 0) FROM dbstat WHERE name LIKE 'knowledge_%'", [], |r| r.get(0))
        .unwrap_or(0);

    Ok(KnowledgeStats {
        total_nodes,
        total_edges,
        by_type,
        by_domain,
        crawl_pending,
        crawl_completed,
        db_size_bytes: db_size,
    })
}

pub fn insert_or_get_node(
    conn: &Connection,
    title: &str,
    node_type: NodeType,
    summary: Option<&str>,
    url: Option<&str>,
    domain: Option<&str>,
) -> rusqlite::Result<String> {
    if let Some(url) = url {
        if let Some(existing) = find_node_by_url(conn, url)? {
            return Ok(existing.id);
        }
    } else if let Some(existing) = find_node_by_title_and_type(conn, title, &node_type)? {
        return Ok(existing.id);
    }

    let id = Uuid::new_v4().to_string();
    let ts = now();
    let node = KnowledgeNode {
        id: id.clone(),
        node_type,
        title: title.to_string(),
        summary: summary.map(|s| s.to_string()),
        // P0 根治 (content/summary 双列分裂): write_memory_entry 把正文作为
        // summary 传入, 此前 content 硬编码 None → 所有读 content 列的下游
        // (conflict_detect / crawl 回填 / absorb_mapper / search 返回) 全部漏掉。
        // 镜像写入两列, 一处修复覆盖所有下游。
        content: summary.map(|s| s.to_string()),
        url: url.map(|s| s.to_string()),
        domain: domain.map(|s| s.to_string()),
        language: "en".into(),
        confidence: 1.0,
        importance: 0.5,
        created_at: ts,
        updated_at: ts,
        access_count: 0,
        metadata: None,
        temporal: None,
        supersedes: None,
        source_episode: None,
    };
    insert_node(conn, &node)?;
    Ok(id)
}

/// upsert_edge 的 metadata 增强版 (T0.1 类型化边, 来源: codebase-memory-mcp 类型化边
/// + semantica PROV-O 溯源)。metadata 承载结构化溯源 (evidence/source/extractor),
///
/// 使 edges 具备"证据优先"的机器可查版本, 而非只塞进 description。
pub fn upsert_edge_full(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
    relation_type: RelationType,
    weight: f64,
    description: Option<&str>,
    metadata: Option<serde_json::Value>,
) -> rusqlite::Result<()> {
    let id = Uuid::new_v4().to_string();
    let edge = KnowledgeEdge {
        id,
        source_id: source_id.to_string(),
        target_id: target_id.to_string(),
        relation_type,
        weight,
        description: description.map(|s| s.to_string()),
        created_at: now(),
        metadata,
    };
    insert_edge(conn, &edge)
}

pub fn upsert_edge(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
    relation_type: RelationType,
    weight: f64,
    description: Option<&str>,
) -> rusqlite::Result<()> {
    upsert_edge_full(conn, source_id, target_id, relation_type, weight, description, None)
}

// D3 架构倒置: 基础 count 原语下沉至 core (nt_core_kb_primitives), re-export
// 保持 `nt_memory_store::count_nodes/count_edges` 调用方路径不变。
// 域级聚合 count (by_type/by_domain/...) 保留本模块 (依赖 nt_memory_types)。
pub use crate::core::nt_core_kb_primitives::{count_edges, count_nodes};

pub fn count_nodes_by_type(conn: &Connection, node_type: &str) -> rusqlite::Result<usize> {
    conn.query_row(
        "SELECT COUNT(*) FROM nodes WHERE node_type=?1",
        params![node_type],
        |row| row.get(0),
    )
}

pub fn count_nodes_by_type_map(conn: &Connection) -> rusqlite::Result<HashMap<String, usize>> {
    let mut stmt = conn.prepare("SELECT node_type, COUNT(*) FROM nodes GROUP BY node_type")?;
    let rows = stmt.query_map([], |row| {
        let node_type: String = row.get(0)?;
        let count: usize = row.get(1)?;
        Ok((node_type, count))
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let (k, v) = row?;
        map.insert(k, v);
    }
    Ok(map)
}

pub fn count_nodes_by_domain(conn: &Connection) -> rusqlite::Result<HashMap<String, usize>> {
    let mut stmt = conn.prepare("SELECT COALESCE(domain,'unknown'), COUNT(*) FROM nodes GROUP BY domain")?;
    let rows = stmt.query_map([], |row| {
        let domain: String = row.get(0)?;
        let count: usize = row.get(1)?;
        Ok((domain, count))
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let (k, v) = row?;
        map.insert(k, v);
    }
    Ok(map)
}

pub fn count_edges_by_type(conn: &Connection) -> rusqlite::Result<HashMap<String, usize>> {
    let mut stmt = conn.prepare("SELECT relation_type, COUNT(*) FROM edges GROUP BY relation_type")?;
    let rows = stmt.query_map([], |row| {
        let rel_type: String = row.get(0)?;
        let count: usize = row.get(1)?;
        Ok((rel_type, count))
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let (k, v) = row?;
        map.insert(k, v);
    }
    Ok(map)
}

pub fn count_nodes_by_domain_and_type(conn: &Connection) -> rusqlite::Result<Vec<(String, String, usize)>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(domain,'unknown'), node_type, COUNT(*) FROM nodes GROUP BY domain, node_type ORDER BY domain"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, usize>(2)?))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn get_stale_node_count(conn: &Connection, older_than_days: i64) -> rusqlite::Result<usize> {
    let cutoff = now() - older_than_days * 86400;
    conn.query_row(
        "SELECT COUNT(*) FROM nodes WHERE updated_at < ?1",
        params![cutoff],
        |row| row.get(0),
    )
}

pub fn get_nodes_page(conn: &Connection, offset: usize, limit: usize) -> rusqlite::Result<Vec<KnowledgeNode>> {
    let mut stmt = conn.prepare(
        "SELECT id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at, access_count, metadata, supersedes FROM nodes ORDER BY rowid LIMIT ?1 OFFSET ?2"
    )?;
    let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
        Ok(KnowledgeNode {
            id: row.get(0)?,
            node_type: NodeType::from_str(&row.get::<_, String>(1)?),
            title: row.get(2)?,
            summary: row.get(3)?,
            content: row.get(4)?,
            url: row.get(5)?,
            domain: row.get(6)?,
            language: row.get(7)?,
            confidence: row.get(8)?,
            importance: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            access_count: row.get(12)?,
            metadata: row.get::<_, Option<String>>(13)?.and_then(|m| serde_json::from_str(&m).ok()),
            temporal: None,
            supersedes: row.get(14)?,
            source_episode: None,
        })
    })?;
    let mut nodes = Vec::with_capacity(limit.min(4096));
    for row in rows {
        nodes.push(row?);
    }
    Ok(nodes)
}

pub fn get_edges_page(conn: &Connection, offset: usize, limit: usize) -> rusqlite::Result<Vec<KnowledgeEdge>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_id, target_id, relation_type, weight, description, created_at, metadata FROM edges ORDER BY rowid LIMIT ?1 OFFSET ?2"
    )?;
    let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
        Ok(KnowledgeEdge {
            id: row.get(0)?,
            source_id: row.get(1)?,
            target_id: row.get(2)?,
            relation_type: RelationType::from_str(&row.get::<_, String>(3)?),
            weight: row.get(4)?,
            description: row.get(5)?,
            created_at: row.get(6)?,
            metadata: row.get::<_, Option<String>>(7)?.and_then(|m| serde_json::from_str(&m).ok()),
        })
    })?;
    let mut edges = Vec::with_capacity(limit.min(4096));
    for row in rows {
        edges.push(row?);
    }
    Ok(edges)
}

pub fn get_all_nodes(conn: &Connection) -> rusqlite::Result<Vec<KnowledgeNode>> {
    let mut stmt = conn.prepare(
        "SELECT id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at, access_count, metadata, supersedes FROM nodes"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(KnowledgeNode {
            id: row.get(0)?,
            node_type: NodeType::from_str(&row.get::<_, String>(1)?),
            title: row.get(2)?,
            summary: row.get(3)?,
            content: row.get(4)?,
            url: row.get(5)?,
            domain: row.get(6)?,
            language: row.get(7)?,
            confidence: row.get(8)?,
            importance: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            access_count: row.get(12)?,
            metadata: row.get::<_, Option<String>>(13)?.and_then(|m| serde_json::from_str(&m).ok()),
            temporal: None,
            supersedes: row.get(14)?,
            source_episode: None,
        })
    })?;
    let mut nodes = Vec::new();
    for row in rows {
        nodes.push(row?);
    }
    Ok(nodes)
}

pub fn get_all_edges(conn: &Connection) -> rusqlite::Result<Vec<KnowledgeEdge>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_id, target_id, relation_type, weight, description, created_at, metadata FROM edges"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(KnowledgeEdge {
            id: row.get(0)?,
            source_id: row.get(1)?,
            target_id: row.get(2)?,
            relation_type: RelationType::from_str(&row.get::<_, String>(3)?),
            weight: row.get(4)?,
            description: row.get(5)?,
            created_at: row.get(6)?,
            metadata: row.get::<_, Option<String>>(7)?.and_then(|m| serde_json::from_str(&m).ok()),
        })
    })?;
    let mut edges = Vec::new();
    for row in rows {
        edges.push(row?);
    }
    Ok(edges)
}


pub fn store_procedural_memory(conn: &Connection, record: &ProceduralMemoryRecord) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO procedural_memory
         (id, skill_id, name, description, e8_sequence, trigger_pattern,
          success_rate, execution_count, avg_reward, created_at, updated_at, tags)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            record.id,
            record.skill_id,
            record.name,
            record.description,
            serde_json::to_string(&record.e8_sequence).unwrap_or_default(),
            serde_json::to_string(&record.trigger_pattern).unwrap_or_default(),
            record.success_rate,
            record.execution_count as i64,
            record.avg_reward,
            record.created_at,
            record.updated_at,
            serde_json::to_string(&record.tags).unwrap_or_default(),
        ],
    )?;
    Ok(())
}

pub fn get_procedural_memory(conn: &Connection, skill_id: &str) -> rusqlite::Result<Option<ProceduralMemoryRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, skill_id, name, description, e8_sequence, trigger_pattern,
                success_rate, execution_count, avg_reward, created_at, updated_at, tags
         FROM procedural_memory WHERE skill_id=?1"
    )?;
    let mut rows = stmt.query(params![skill_id])?;
    match rows.next()? {
        Some(row) => Ok(Some(ProceduralMemoryRecord {
            id: row.get(0)?,
            skill_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            e8_sequence: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
            trigger_pattern: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
            success_rate: row.get(6)?,
            execution_count: row.get::<_, i64>(7)? as u64,
            avg_reward: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
            tags: serde_json::from_str(&row.get::<_, String>(11)?).unwrap_or_default(),
        })),
        None => Ok(None),
    }
}

pub fn list_procedural_memories(conn: &Connection, top_k: usize) -> rusqlite::Result<Vec<ProceduralMemoryRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, skill_id, name, description, e8_sequence, trigger_pattern,
                success_rate, execution_count, avg_reward, created_at, updated_at, tags
         FROM procedural_memory ORDER BY success_rate DESC LIMIT ?1"
    )?;
    let rows = stmt.query_map(params![top_k as i64], |row| {
        Ok(ProceduralMemoryRecord {
            id: row.get(0)?,
            skill_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            e8_sequence: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
            trigger_pattern: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
            success_rate: row.get(6)?,
            execution_count: row.get::<_, i64>(7)? as u64,
            avg_reward: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
            tags: serde_json::from_str(&row.get::<_, String>(11)?).unwrap_or_default(),
        })
    })?;
    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

pub fn update_procedural_memory_success(conn: &Connection, skill_id: &str, reward: f64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE procedural_memory SET
            execution_count = execution_count + 1,
            avg_reward = (avg_reward * (execution_count - 1) + ?1) / execution_count,
            success_rate = COALESCE(success_rate, 0.0) * 0.9 + CASE WHEN ?1 > 0.5 THEN 0.1 ELSE 0.0 END,
            updated_at = datetime('now')
         WHERE skill_id=?2",
        params![reward, skill_id],
    )?;
    Ok(())
}

pub fn find_matching_skills(conn: &Connection, e8_state: u8) -> rusqlite::Result<Vec<ProceduralMemoryRecord>> {
    let pattern_str = format!("%{}%", e8_state);
    let mut stmt = conn.prepare(
        "SELECT id, skill_id, name, description, e8_sequence, trigger_pattern,
                success_rate, execution_count, avg_reward, created_at, updated_at, tags
         FROM procedural_memory WHERE trigger_pattern LIKE ?1 ORDER BY success_rate DESC"
    )?;
    let rows = stmt.query_map(params![pattern_str], |row| {
        Ok(ProceduralMemoryRecord {
            id: row.get(0)?,
            skill_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            e8_sequence: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
            trigger_pattern: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
            success_rate: row.get(6)?,
            execution_count: row.get::<_, i64>(7)? as u64,
            avg_reward: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
            tags: serde_json::from_str(&row.get::<_, String>(11)?).unwrap_or_default(),
        })
    })?;
    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
