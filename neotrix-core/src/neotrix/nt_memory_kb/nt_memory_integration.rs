use rusqlite::Connection;

use super::nt_memory_store as store;
use super::nt_memory_types::{NodeType, RelationType};

/// Bundled request parameters for persist_mined_knowledge.
pub struct PersistKnowledgeRequest {
    pub title: String,
    pub summary: String,
    pub url: String,
    pub source_type: String,
    pub confidence: f64,
    pub edits: Vec<(String, f64)>,
    pub insights: Vec<String>,
}

/// 将 WebMiner 挖掘结果持久化到 KnowledgeBase
pub fn persist_mined_knowledge(
    conn: &Connection,
    req: &PersistKnowledgeRequest,
) -> Result<String, String> {
    let domain = extract_domain(&req.url);
    let node_type = match req.source_type.as_str() {
        "Wikipedia" => NodeType::Concept,
        "ArXiv" => NodeType::Paper,
        "GitHub" => NodeType::Repository,
        _ => NodeType::Article,
    };

    let summary_short = if req.summary.len() > 2000 {
        &req.summary[..2000]
    } else {
        &req.summary
    };

    let node_id = store::insert_or_get_node(
        conn,
        &req.title,
        node_type,
        Some(summary_short),
        Some(&req.url),
        Some(&domain),
    )
    .map_err(|e| format!("KB insert node: {}", e))?;

    for insight in &req.insights {
        let insight_id =
            store::insert_or_get_node(conn, insight, NodeType::Insight, None, None, None)
                .map_err(|e| format!("KB insert insight: {}", e))?;
        store::upsert_edge(
            conn,
            &node_id,
            &insight_id,
            RelationType::Related,
            0.7,
            Some("Mined insight"),
        )
        .map_err(|e| format!("KB upsert edge: {}", e))?;
    }

    Ok(node_id)
}

/// 将 KnowledgeEngine 全量条目导入 KnowledgeBase
pub fn import_from_knowledge_engine(
    conn: &Connection,
    entries: &[super::super::nt_mind::knowledge_engine::KnowledgeEntry],
    relations: &[super::super::nt_mind::knowledge_engine::KnowledgeRelation],
) -> Result<(usize, usize), String> {
    let mut nodes = 0;
    let mut edges = 0;

    for entry in entries {
        let node_type = match entry.source {
            super::super::nt_mind::knowledge_engine::KnowledgeSourceType::Wikipedia => {
                NodeType::Concept
            }
            super::super::nt_mind::knowledge_engine::KnowledgeSourceType::ArXiv => NodeType::Paper,
            super::super::nt_mind::knowledge_engine::KnowledgeSourceType::GitHub => {
                NodeType::Repository
            }
            super::super::nt_mind::knowledge_engine::KnowledgeSourceType::Book => NodeType::Article,
            super::super::nt_mind::knowledge_engine::KnowledgeSourceType::WebPage => {
                NodeType::Article
            }
            super::super::nt_mind::knowledge_engine::KnowledgeSourceType::SemanticScholar => {
                NodeType::Paper
            }
            _ => NodeType::Concept,
        };
        let summary = if entry.summary.is_empty() {
            &entry.body
        } else {
            &entry.summary
        };
        let summary_short = if summary.len() > 2000 {
            &summary[..2000]
        } else {
            summary
        };
        let domain = extract_domain(&entry.source_url);

        let node_id = store::insert_or_get_node(
            conn,
            &entry.title,
            node_type,
            Some(summary_short),
            Some(&entry.source_url),
            Some(&domain),
        )
        .map_err(|e| format!("KB insert: {}", e))?;

        let mut meta = serde_json::Map::new();
        meta.insert(
            "tags".into(),
            serde_json::Value::Array(
                entry
                    .tags
                    .iter()
                    .map(|t| serde_json::Value::String(t.clone()))
                    .collect(),
            ),
        );
        meta.insert(
            "confidence".into(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(entry.confidence)
                    .unwrap_or(serde_json::Number::from(0)),
            ),
        );
        if !entry.dimensions.is_empty() {
            meta.insert(
                "dimensions".into(),
                serde_json::Value::Array(
                    entry
                        .dimensions
                        .iter()
                        .map(|d| serde_json::Value::String(d.clone()))
                        .collect(),
                ),
            );
        }

        store::update_node_metadata(conn, &node_id, &serde_json::Value::Object(meta))
            .map_err(|e| format!("KB update metadata: {}", e))?;

        nodes += 1;
    }

    for rel in relations {
        let relation_type = match rel.relation_type {
            super::super::nt_mind::knowledge_engine::RelationType::References => {
                RelationType::References
            }
            super::super::nt_mind::knowledge_engine::RelationType::SubclassOf => {
                RelationType::SubclassOf
            }
            super::super::nt_mind::knowledge_engine::RelationType::InstanceOf => {
                RelationType::InstanceOf
            }
            super::super::nt_mind::knowledge_engine::RelationType::Causes => RelationType::Causes,
            super::super::nt_mind::knowledge_engine::RelationType::PrerequisiteOf => {
                RelationType::PrerequisiteOf
            }
            super::super::nt_mind::knowledge_engine::RelationType::Contradicts => {
                RelationType::Contradicts
            }
            super::super::nt_mind::knowledge_engine::RelationType::Supports => {
                RelationType::Supports
            }
            super::super::nt_mind::knowledge_engine::RelationType::BeforeInTime => {
                RelationType::BeforeInTime
            }
            super::super::nt_mind::knowledge_engine::RelationType::Related => RelationType::Related,
        };

        store::upsert_edge(
            conn,
            &rel.from_id,
            &rel.to_id,
            relation_type,
            rel.weight,
            Some(&rel.description),
        )
        .map_err(|e| format!("KB upsert edge: {}", e))?;
        edges += 1;
    }

    Ok((nodes, edges))
}

fn extract_domain(url_str: &str) -> String {
    url_str
        .split('/')
        .nth(2)
        .unwrap_or("unknown")
        .trim_start_matches("www.")
        .to_string()
}
