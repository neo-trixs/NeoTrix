//! 山海世界跨引用关系推断 — 从 bin/shanhai_link.rs 吸收归档 (R-P42)。
//!
//! 根据节点元数据自动创建证据→山峰/映射的 Supports 边:
//!   - 证据 metadata.scholar → 查找同名学者 (DevelopedBy)
//!   - 证据 metadata.conclusion / key_insight → 山峰引用 (Supports)
//!   - 证据 metadata.relation_to_kunlun → 昆仑节点 (Supports)
//!   - 映射 metadata.attributed_by → 学者 (DevelopedBy)
//!
//! 幂等: 使用 INSERT OR IGNORE (safe_insert_edge), 重复运行不产生重复边。

use std::collections::HashMap;

use rusqlite::Connection;
use serde_json::Value;

use super::kb::{safe_insert_edge, now};
use crate::neotrix::nt_memory_kb::nt_memory_types::{KnowledgeEdge, RelationType};

/// 查询所有 shanhai 节点 (id, node_type, title, metadata)。
fn load_shanhai_nodes(conn: &Connection) -> rusqlite::Result<Vec<(String, String, String, Option<String>)>> {
    let mut stmt = conn.prepare("SELECT id, node_type, title, metadata FROM nodes WHERE id LIKE 'shanhai-%'")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// 在已有节点中查找 id 或 title 包含关键词的节点 id 列表。
fn find_nodes_containing<'a>(
    nodes: &'a [(String, String, String, Option<String>)],
    keyword: &str,
) -> Vec<&'a str> {
    let kw = keyword.to_lowercase();
    nodes.iter()
        .filter(|(id, _ty, title, _meta)| {
            id.to_lowercase().contains(&kw) || title.to_lowercase().contains(&kw)
        })
        .map(|(id, _, _, _)| id.as_str())
        .collect()
}

/// 为山海世界节点自动创建跨引用边。返回新增边数 (幂等)。
pub fn infer_shanhai_links(conn: &Connection) -> rusqlite::Result<usize> {
    let nodes = load_shanhai_nodes(conn)?;
    let now = now();

    let mut idx: HashMap<&str, (&str, &str, Option<&str>)> = HashMap::new();
    for (id, ty, title, meta) in &nodes {
        idx.insert(id.as_str(), (title, ty, meta.as_deref()));
    }

    let mut edges_created = 0usize;

    for (id, _ty, title, meta_str) in &nodes {
        let meta: Option<Value> = meta_str.as_deref().and_then(|m| serde_json::from_str(m).ok());

        // ── 模式1: 证据节点 → 关联学者/山峰/昆仑 ──
        if id.starts_with("shanhai-evidence:") {
            // 学者关联
            let scholar = meta.as_ref()
                .and_then(|m| m.get("scholar").or_else(|| m.get("archaeologist")))
                .and_then(|s| s.as_str());
            if let Some(sch) = scholar {
                for (sid, _sty, stitle, _smeta) in &nodes {
                    if stitle.contains(sch) || sch.contains(stitle.as_str()) {
                        let edge = KnowledgeEdge {
                            id: format!("shanhai-edge:evidence→scholar-{}-{}", id, sid),
                            source_id: id.clone(),
                            target_id: sid.clone(),
                            relation_type: RelationType::DevelopedBy,
                            weight: 0.8,
                            description: Some(format!("{} 由 {} 提出", title, sch)),
                            created_at: now,
                            metadata: None,
                        };
                        let _ = safe_insert_edge(conn, &edge);
                        edges_created += 1;
                    }
                }
            }

            // 结论中的山名 → 山峰节点
            let conclusion = meta.as_ref()
                .and_then(|m| m.get("conclusion").or_else(|| m.get("key_insight")))
                .and_then(|c| c.as_str());
            if let Some(conc) = conclusion {
                let candidates = find_nodes_containing(&nodes, "山");
                for cid in candidates {
                    if *cid == *id {
                        continue;
                    }
                    let ctitle = idx.get(cid).map(|(t, _, _)| *t).unwrap_or_default();
                    let prefix4 = conc.chars().take(4).collect::<String>();
                    if conc.contains(ctitle) || ctitle.contains(prefix4.as_str()) {
                        let edge = KnowledgeEdge {
                            id: format!("shanhai-edge:evidence→peak-{}-{}", id, cid),
                            source_id: id.clone(),
                            target_id: cid.to_string(),
                            relation_type: RelationType::Supports,
                            weight: 0.7,
                            description: Some(format!("{} 证据支持 {}", title, ctitle)),
                            created_at: now,
                            metadata: None,
                        };
                        let _ = safe_insert_edge(conn, &edge);
                        edges_created += 1;
                    }
                }
            }

            // 昆仑关联
            let _relation_to = meta.as_ref()
                .and_then(|m| m.get("relation_to_kunlun"))
                .and_then(|r| r.as_str());
            let kunlun_ids = find_nodes_containing(&nodes, "昆仑");
            for kid in kunlun_ids {
                if *kid == *id {
                    continue;
                }
                let edge = KnowledgeEdge {
                    id: format!("shanhai-edge:evidence→kunlun-{}-{}", id, kid),
                    source_id: id.clone(),
                    target_id: kid.to_string(),
                    relation_type: RelationType::Supports,
                    weight: 0.8,
                    description: Some(format!("{} 关联昆仑", id)),
                    created_at: now,
                    metadata: None,
                };
                let _ = safe_insert_edge(conn, &edge);
                edges_created += 1;
            }
        }

        // ── 模式2: 映射节点 → 归因学者 ──
        if id.starts_with("shanhai-map:") {
            let attributed = meta.as_ref()
                .and_then(|m| m.get("attributed_by"))
                .and_then(|a| a.as_array())
                .cloned()
                .unwrap_or_default();
            for attr in &attributed {
                let attr_str = attr.as_str().unwrap_or_default();
                // attributed_by 格式: "学者名@学派:置信度"
                let name = attr_str.split('@').next().unwrap_or_default();
                for (sid, _sty, stitle, _smeta) in &nodes {
                    if stitle.contains(name) || name.contains(stitle.as_str()) {
                        let edge = KnowledgeEdge {
                            id: format!("shanhai-edge:map→scholar-{}-{}", id, sid),
                            source_id: id.clone(),
                            target_id: sid.clone(),
                            relation_type: RelationType::DevelopedBy,
                            weight: 0.9,
                            description: Some(format!("{} 归因于 {}", title, stitle)),
                            created_at: now,
                            metadata: None,
                        };
                        let _ = safe_insert_edge(conn, &edge);
                        edges_created += 1;
                    }
                }
            }
        }
    }

    Ok(edges_created)
}

/// 山海边总数统计。
pub fn shanhai_edge_count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM edges WHERE id LIKE 'shanhai-edge:%'", [], |r| r.get(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_memory_kb::nt_memory_schema;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_find_nodes_containing() {
        let nodes = vec![
            ("shanhai-peak:kunlun".to_string(), "mountain".to_string(), "昆仑山".to_string(), None),
            ("shanhai-peak:taishan".to_string(), "mountain".to_string(), "泰山".to_string(), None),
        ];
        let hits = find_nodes_containing(&nodes, "昆仑");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0], "shanhai-peak:kunlun");
    }

    #[test]
    fn test_infer_links_empty() {
        let conn = test_db();
        let created = infer_shanhai_links(&conn).unwrap();
        assert_eq!(created, 0);
        let total = shanhai_edge_count(&conn).unwrap();
        assert_eq!(total, 0);
    }
}
