use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};
use uuid::Uuid;

use super::nt_memory_types::*;

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn insert_node(conn: &Connection, node: &KnowledgeNode) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO nodes (id, node_type, title, summary, content, url, domain, language,
            confidence, importance, created_at, updated_at, access_count, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
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
        ],
    )?;

    if let Some(summary) = &node.summary {
        let content = node.content.as_deref().unwrap_or("");
        let domain = node.domain.as_deref().unwrap_or("");
        conn.execute(
            "INSERT INTO nodes_fts (rowid, title, summary, content, domain)
             VALUES (last_insert_rowid(), ?1, ?2, ?3, ?4)",
            params![node.title, summary, content, domain],
        )?;
    }

    Ok(())
}

pub fn update_node(conn: &Connection, node: &KnowledgeNode) -> rusqlite::Result<()> {
    conn.execute(
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
    Ok(())
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
            confidence, importance, created_at, updated_at, access_count, metadata
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
            }))
        }
        None => Ok(None),
    }
}

pub fn find_node_by_title_and_type(conn: &Connection, title: &str, node_type: &NodeType) -> rusqlite::Result<Option<KnowledgeNode>> {
    let mut stmt = conn.prepare(
        "SELECT id, node_type, title, summary, content, url, domain, language,
            confidence, importance, created_at, updated_at, access_count, metadata
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
        })),
        None => Ok(None),
    }
}

/// 合并相同标题的重复节点 (将指定节点的边迁移到保留节点)
pub fn merge_duplicate_nodes(conn: &Connection, keep_id: &str, remove_id: &str) -> rusqlite::Result<()> {
    // Remove edges from remove_id that already exist on keep_id (avoid UNIQUE conflict)
    conn.execute(
        "DELETE FROM edges WHERE source_id=?1 AND (target_id, relation_type) IN \
         (SELECT target_id, relation_type FROM edges WHERE source_id=?2)",
        params![remove_id, keep_id],
    )?;
    conn.execute(
        "DELETE FROM edges WHERE target_id=?1 AND (source_id, relation_type) IN \
         (SELECT source_id, relation_type FROM edges WHERE target_id=?2)",
        params![remove_id, keep_id],
    )?;
    conn.execute("UPDATE edges SET source_id=?1 WHERE source_id=?2", params![keep_id, remove_id])?;
    conn.execute("UPDATE edges SET target_id=?1 WHERE target_id=?2", params![keep_id, remove_id])?;
    conn.execute("DELETE FROM nodes WHERE id=?1", params![remove_id])?;
    conn.execute("DELETE FROM nodes_fts WHERE rowid = (SELECT rowid FROM nodes WHERE id=?1)", params![remove_id])?;
    Ok(())
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
            confidence, importance, created_at, updated_at, access_count, metadata
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
        })),
        None => Ok(None),
    }
}

pub fn delete_node(conn: &Connection, id: &str) -> rusqlite::Result<bool> {
    let affected = conn.execute("DELETE FROM nodes WHERE id=?1", params![id])?;
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

pub fn claim_next_crawl_url(conn: &Connection) -> rusqlite::Result<Option<CrawlQueueItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, url, depth, domain, priority, status, discovered_at, last_attempt, retry_count, error_message
         FROM crawl_queue
         WHERE status='pending'
         ORDER BY priority DESC, discovered_at ASC
         LIMIT 1",
    )?;
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
        content: None,
        url: url.map(|s| s.to_string()),
        domain: domain.map(|s| s.to_string()),
        language: "en".into(),
        confidence: 1.0,
        importance: 0.5,
        created_at: ts,
        updated_at: ts,
        access_count: 0,
        metadata: None,
    };
    insert_node(conn, &node)?;
    Ok(id)
}

pub fn upsert_edge(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
    relation_type: RelationType,
    weight: f64,
    description: Option<&str>,
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
        metadata: None,
    };
    insert_edge(conn, &edge)
}

pub fn get_all_nodes(conn: &Connection) -> rusqlite::Result<Vec<KnowledgeNode>> {
    let mut stmt = conn.prepare(
        "SELECT id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at, access_count, metadata FROM nodes"
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
        })
    })?;
    let mut nodes = Vec::new();
    for row in rows {
        nodes.push(row?);
    }
    Ok(nodes)
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
