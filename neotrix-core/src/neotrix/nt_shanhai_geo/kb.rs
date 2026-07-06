/// Safe KB operations for Shanhai data ingestion.
/// Uses INSERT OR IGNORE so running multiple times won't fail.
use rusqlite::Connection;
use crate::neotrix::nt_memory_kb::nt_memory_types::*;

pub fn safe_insert_node(conn: &Connection, node: &KnowledgeNode) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO nodes (id, node_type, title, summary, content, url, domain, language,
            confidence, importance, created_at, updated_at, access_count, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        rusqlite::params![
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
    Ok(())
}

pub fn safe_insert_edge(conn: &Connection, edge: &KnowledgeEdge) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO edges (id, source_id, target_id, relation_type, weight, description, created_at, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
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

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
