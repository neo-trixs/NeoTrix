use rusqlite::Connection;

use super::nt_memory_store as store;
use super::nt_memory_types::{NodeType, RelationType};

/// Local types to break L3→L8 upward dependency on nt_mind::knowledge_engine
#[derive(Debug, Clone)]
pub enum KnowledgeSourceType {
    Wikipedia,
    ArXiv,
    GitHub,
    Book,
    WebPage,
    SemanticScholar,
}

#[derive(Debug, Clone)]
pub enum KnowledgeRelationType {
    References,
    SubclassOf,
    InstanceOf,
    Causes,
    PrerequisiteOf,
    Contradicts,
    Supports,
    BeforeInTime,
    Related,
}

#[derive(Debug, Clone)]
pub struct KnowledgeEntry {
    pub source: KnowledgeSourceType,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub source_url: String,
    pub tags: Vec<String>,
    pub confidence: f64,
    pub dimensions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct KnowledgeRelation {
    pub from_id: String,
    pub to_id: String,
    pub relation_type: KnowledgeRelationType,
    pub weight: f64,
    pub description: String,
}

/// 将 WebMiner 挖掘结果持久化到 KnowledgeBase
pub fn persist_mined_knowledge(conn: &Connection, title: &str, summary: &str, url: &str, source_type: &str, _confidence: f64, _edits: &[(String, f64)], insights: &[String]) -> Result<String, String> {
    let domain = extract_domain(url);
    let node_type = match source_type {
        "Wikipedia" => NodeType::Concept,
        "ArXiv" => NodeType::Paper,
        "GitHub" => NodeType::Repository,
        _ => NodeType::Article,
    };

    let summary_short = if summary.len() > 2000 { &summary[..2000] } else { summary };

    let node_id = store::insert_or_get_node(conn, title, node_type, Some(summary_short), Some(url), Some(&domain))
        .map_err(|e| format!("KB insert node: {}", e))?;

    for insight in insights {
        let insight_id = store::insert_or_get_node(conn, insight, NodeType::Insight, None, None, None)
            .map_err(|e| format!("KB insert insight: {}", e))?;
        store::upsert_edge(conn, &node_id, &insight_id, RelationType::Related, 0.7, Some("Mined insight"))
            .map_err(|e| format!("KB upsert edge: {}", e))?;
    }

    Ok(node_id)
}

/// 将 KnowledgeEngine 全量条目导入 KnowledgeBase
pub fn import_from_knowledge_engine(conn: &Connection, entries: &[KnowledgeEntry], relations: &[KnowledgeRelation]) -> Result<(usize, usize), String> {
    let mut nodes = 0;
    let mut edges = 0;

    for entry in entries {
        let node_type = match entry.source {
            KnowledgeSourceType::Wikipedia => NodeType::Concept,
            KnowledgeSourceType::ArXiv => NodeType::Paper,
            KnowledgeSourceType::GitHub => NodeType::Repository,
            KnowledgeSourceType::Book => NodeType::Article,
            KnowledgeSourceType::WebPage => NodeType::Article,
            KnowledgeSourceType::SemanticScholar => NodeType::Paper,
        };
        let summary = if entry.summary.is_empty() { &entry.body } else { &entry.summary };
        let summary_short = if summary.len() > 2000 { &summary[..2000] } else { summary };
        let domain = extract_domain(&entry.source_url);

        let node_id = store::insert_or_get_node(conn, &entry.title, node_type, Some(summary_short), Some(&entry.source_url), Some(&domain))
            .map_err(|e| format!("KB insert: {}", e))?;

        let mut meta = serde_json::Map::new();
        meta.insert("tags".into(), serde_json::Value::Array(entry.tags.iter().map(|t| serde_json::Value::String(t.clone())).collect()));
        meta.insert("confidence".into(), serde_json::Value::Number(serde_json::Number::from_f64(entry.confidence).unwrap_or(serde_json::Number::from(0))));
        if !entry.dimensions.is_empty() {
            meta.insert("dimensions".into(), serde_json::Value::Array(entry.dimensions.iter().map(|d| serde_json::Value::String(d.clone())).collect()));
        }

        store::update_node_metadata(conn, &node_id, &serde_json::Value::Object(meta))
            .map_err(|e| format!("KB update metadata: {}", e))?;

        nodes += 1;
    }

    for rel in relations {
        let relation_type = match rel.relation_type {
            KnowledgeRelationType::References => RelationType::References,
            KnowledgeRelationType::SubclassOf => RelationType::SubclassOf,
            KnowledgeRelationType::InstanceOf => RelationType::InstanceOf,
            KnowledgeRelationType::Causes => RelationType::Causes,
            KnowledgeRelationType::PrerequisiteOf => RelationType::PrerequisiteOf,
            KnowledgeRelationType::Contradicts => RelationType::Contradicts,
            KnowledgeRelationType::Supports => RelationType::Supports,
            KnowledgeRelationType::BeforeInTime => RelationType::BeforeInTime,
            KnowledgeRelationType::Related => RelationType::Related,
        };

        store::upsert_edge(conn, &rel.from_id, &rel.to_id, relation_type, rel.weight, Some(&rel.description))
            .map_err(|e| format!("KB upsert edge: {}", e))?;
        edges += 1;
    }

    Ok((nodes, edges))
}

fn extract_domain(url_str: &str) -> String {
    url_str.split('/')
        .nth(2)
        .unwrap_or("unknown")
        .trim_start_matches("www.")
        .to_string()
}
