use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Row;
use serde::{Serialize, de::DeserializeOwned};

use crate::core::nt_core_kb_types::{KnowledgeNode, NodeType};

/// 统一时间戳函数 (秒级)
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// 向后兼容别名
pub use now as now_ts;

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

/// 泛型 KV 状态保存: 将任意 Serialize 类型序列化为 JSON 存入 KB kv_store。
pub fn save_kv_state<T: Serialize>(kb: &super::KnowledgeBase, ns: &str, key: &str, data: &T) -> Result<(), String> {
    let json = serde_json::to_string(data).map_err(|e| format!("serde: {}", e))?;
    let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
    super::nt_memory_unify::kv_set(&conn, ns, key, &json)
}

/// 泛型 KV 状态加载: 从 KB kv_store 读取 JSON 并反序列化为任意 DeserializeOwned 类型。
pub fn load_kv_state<T: DeserializeOwned>(kb: &super::KnowledgeBase, ns: &str, key: &str) -> Result<Option<T>, String> {
    let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
    let json = super::nt_memory_unify::kv_get(&conn, ns, key)?;
    match json {
        Some(data) => Ok(Some(serde_json::from_str(&data).map_err(|e| format!("deser: {}", e))?)),
        None => Ok(None),
    }
}
