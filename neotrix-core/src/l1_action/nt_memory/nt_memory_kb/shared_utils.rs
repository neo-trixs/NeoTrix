use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Row;

use crate::core::nt_core_kb_types::{KnowledgeNode, NodeType};

/// 统一时间戳函数 (秒级)
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// 标准 SELECT 列索引: id, node_type, title, summary, content, url, domain, language,
/// confidence, importance, recall_weight, created_at, updated_at, access_count, metadata,
/// supersedes, parent_id, depth, cluster_id
///
/// 从数据库行映射到 KnowledgeNode (标准 19 列模式)
pub fn row_to_knowledge_node(row: &Row) -> rusqlite::Result<KnowledgeNode> {
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
        recall_weight: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
        access_count: row.get(13)?,
        metadata: row.get::<_, Option<String>>(14)?.and_then(|m| serde_json::from_str(&m).ok()),
        temporal: None,
        supersedes: row.get(15)?,
        source_episode: None,
        parent_id: row.get(16)?,
        depth: row.get(17)?,
        cluster_id: row.get(18)?,
    })
}

/// 标准 SELECT 列名 (用于 SQL 拼接)
pub const NODE_COLUMNS: &str = "id, node_type, title, summary, content, url, domain, language, \
    confidence, importance, recall_weight, created_at, updated_at, access_count, metadata, \
    supersedes, parent_id, depth, cluster_id";
