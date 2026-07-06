//! neotrix-shanhai-link — 跨引用关系推断
//!
//! 根据节点元数据自动创建证据→山峰/映射的 Supports 边:
//!   - 证据 metadata.scholar → 查找同名学者
//!   - 证据 metadata.relation_to_kunlun → 昆仑节点
//!   - 映射 metadata.attributed_by → 学者→理论边
//!   - 证据 metadata.conclusion 中的山名 → 同名校对
//!
//! Usage: cargo run -p neotrix --bin neotrix-shanhai-link

use neotrix::neotrix::nt_memory_kb::nt_memory_types::*;
use neotrix::neotrix::nt_memory_kb::nt_memory_schema;
use neotrix::neotrix::nt_shanhai_geo::*;
use rusqlite::Connection;

fn open_kb() -> Connection {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.neotrix/knowledge.db", home);
    let conn = Connection::open(&db_path).expect("Failed to open KB");
    nt_memory_schema::initialize(&conn).expect("Failed to init schema");
    conn
}

/// 查询所有 shanhai 节点
fn load_shanhai_nodes(conn: &Connection) -> Vec<(String, String, String, Option<String>)> {
    let mut stmt = conn
        .prepare("SELECT id, node_type, title, metadata FROM nodes WHERE id LIKE 'shanhai-%'")
        .expect("load_shanhai_nodes: SQL prepare");
    stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
        ))
    })
    .expect("load_shanhai_nodes: query_map")
    .filter_map(|r| r.ok())
    .collect()
}

fn parse_meta(s: Option<&String>) -> Option<serde_json::Value> {
    s.and_then(|m| serde_json::from_str::<serde_json::Value>(m).ok())
}

/// 在已有节点中查找包含关键词的
fn find_nodes_containing<'a>(nodes: &'a [(String, String, String, Option<String>)], keyword: &str) -> Vec<&'a str> {
    let kw = keyword.to_lowercase();
    nodes.iter()
        .filter(|(id, _ty, title, _meta)| {
            id.to_lowercase().contains(&kw) || title.to_lowercase().contains(&kw)
        })
        .map(|(id, _, _, _)| id.as_str())
        .collect()
}

fn main() {
    let conn = open_kb();
    let nodes = load_shanhai_nodes(&conn);
    let now = now();

    println!("🔗 山海经关系链接器");
    println!("   节点总数: {}", nodes.len());

    // 建立快速索引: id → (title, node_type, metadata)
    let mut idx: std::collections::HashMap<&str, (&str, &str, Option<&str>)> = std::collections::HashMap::new();
    for (id, ty, title, meta) in &nodes {
        idx.insert(id.as_str(), (title, ty, meta.as_deref()));
    }

    let mut edges_created = 0;

    for (id, _ty, title, meta_str) in &nodes {
        let meta = parse_meta(meta_str.as_ref());

        // --- 模式1: 证据节点 → 关联山峰/映射 ---
        if id.starts_with("shanhai-evidence:") {
            let _ev_type = meta.as_ref().and_then(|m| m.get("type")).and_then(|t| t.as_str());
            let scholar = meta.as_ref()
                .and_then(|m| m.get("scholar").or_else(|| m.get("archaeologist")))
                .and_then(|s| s.as_str());

            // 查找关联的学者节点
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
                        let _ = safe_insert_edge(&conn, &edge);
                        edges_created += 1;
                    }
                }
            }

            // 查找证据中提到的山名 → 山峰节点
            let conclusion = meta.as_ref()
                .and_then(|m| m.get("conclusion").or_else(|| m.get("key_insight")))
                .and_then(|c| c.as_str());
            if let Some(conc) = conclusion {
                // 提取可能的山峰名
                let candidates = find_nodes_containing(&nodes, "山");
                for cid in candidates {
                    if *cid == *id { continue; }
                    let ctitle = idx.get(cid).map(|(t, _, _)| *t).unwrap_or_default();
                    if conc.contains(ctitle) || ctitle.contains(conc.chars().take(4).collect::<String>().as_str()) {
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
                        let _ = safe_insert_edge(&conn, &edge);
                        edges_created += 1;
                    }
                }
            }

            // 关系→昆仑引用
            let relation_to = meta.as_ref()
                .and_then(|m| m.get("relation_to_kunlun"))
                .and_then(|r| r.as_str());
            if let Some(_rel) = relation_to {
                // 查找昆仑节点
                let kunlun_ids = find_nodes_containing(&nodes, "昆仑");
                for kid in kunlun_ids {
                    if *kid == *id { continue; }
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
                    let _ = safe_insert_edge(&conn, &edge);
                    edges_created += 1;
                }
            }
        }

        // --- 模式2: 映射节点 → 归因学者/学派 ---
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
                        let _ = safe_insert_edge(&conn, &edge);
                        edges_created += 1;
                    }
                }
            }
        }
    }

    println!("\n✅ 关系链接完成!");
    println!("   新增边: {} 条", edges_created);

    // 统计
    let total_edges: i64 = conn
        .query_row("SELECT COUNT(*) FROM edges WHERE id LIKE 'shanhai-edge:%'", [], |r| r.get(0))
        .unwrap_or(0);
    println!("   山海边总数: {}", total_edges);
}
