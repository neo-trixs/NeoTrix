use std::collections::HashMap;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    Wikipedia,
    ArXiv,
    SemanticScholar,
    GitHub,
    Book,
    WebPage,
    KnowledgeBase,
    UserInput,
    Inferred,
}

impl SourceType {
    pub fn name(&self) -> &'static str {
        match self {
            SourceType::Wikipedia => "wikipedia",
            SourceType::ArXiv => "arxiv",
            SourceType::SemanticScholar => "semantic-scholar",
            SourceType::GitHub => "github",
            SourceType::Book => "book",
            SourceType::WebPage => "web",
            SourceType::KnowledgeBase => "kb",
            SourceType::UserInput => "user",
            SourceType::Inferred => "inferred",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: String,
    pub title: String,
    pub body: String,
    pub summary: String,
    pub source: SourceType,
    pub source_url: String,
    pub tags: Vec<String>,
    pub dimensions: Vec<String>,
    pub embedding: Option<Vec<f64>>,
    pub confidence: f64,
    pub importance: f64,
    pub created_at: i64,
    pub updated_at: i64,
    pub access_count: u64,
    pub related_ids: Vec<String>,
}

impl KnowledgeEntry {
    pub fn new(title: &str, body: &str, source: SourceType, source_url: &str) -> Self {
        let now = Utc::now().timestamp();
        let summary = body.chars().take(300).collect();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            body: body.to_string(),
            summary,
            source,
            source_url: source_url.to_string(),
            tags: Vec::new(),
            dimensions: Vec::new(),
            embedding: None,
            confidence: 0.7,
            importance: 0.5,
            created_at: now,
            updated_at: now,
            access_count: 0,
            related_ids: Vec::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags; self
    }

    pub fn with_dimensions(mut self, dims: Vec<String>) -> Self {
        self.dimensions = dims; self
    }

    pub fn with_confidence(mut self, c: f64) -> Self {
        self.confidence = c; self
    }

    pub fn with_importance(mut self, i: f64) -> Self {
        self.importance = i; self
    }

    pub fn estimate_importance(&self) -> f64 {
        let base = 0.3;
        let title_len = self.title.len().min(100) as f64 / 100.0 * 0.2;
        let body_len = self.body.len().min(10000) as f64 / 10000.0 * 0.2;
        let tag_bonus = (self.tags.len() as f64).min(10.0) / 10.0 * 0.2;
        let conf = self.confidence * 0.2;
        (base + title_len + body_len + tag_bonus + conf).min(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeRelation {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub relation_type: RelationType,
    pub weight: f64,
    pub description: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationType {
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
pub struct KnowledgeEngineStats {
    pub total_entries: usize,
    pub total_relations: usize,
    pub max_entries: usize,
    pub per_source: HashMap<String, usize>,
}

pub(crate) fn urlencoding(s: &str) -> String {
    s.replace(' ', "+")
}

pub(crate) fn extract_xml(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    xml.find(&open).and_then(|start| {
        let cs = start + open.len();
        xml[cs..].find(&close).map(|end| {
            xml[cs..cs + end].trim().to_string()
        })
    })
}

pub(crate) fn extract_authors(xml: &str) -> Vec<String> {
    let mut authors = Vec::new();
    let mut pos = 0;
    while let Some(s) = xml[pos..].find("<author>") {
        let start = pos + s;
        if let Some(e) = xml[start..].find("</author>") {
            let block = &xml[start..start + e];
            if let Some(name) = extract_xml(block, "name") {
                authors.push(name);
            }
            pos = start + e + 9;
        } else { break; }
    }
    authors
}

pub(crate) fn extract_categories(xml: &str) -> Vec<String> {
    let mut cats = Vec::new();
    let mut pos = 0;
    while let Some(s) = xml[pos..].find("term=\"") {
        let start = pos + s + 6;
        let rest = &xml[start..];
        if let Some(end) = rest.find('\"') {
            cats.push(rest[..end].to_string());
            pos = start + end + 1;
        } else { break; }
    }
    cats
}

pub(crate) fn strip_html(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}
