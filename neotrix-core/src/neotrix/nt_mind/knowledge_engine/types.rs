use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::nt_core_hcube::VsaVector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSourceType {
    Wikipedia,
    ArXiv,
    SemanticScholar,
    GitHub,
    Book,
    WebPage,
    KnowledgeBase,
    UserInput,
    Inferred,
    PdfLocal,
}

impl KnowledgeSourceType {
    pub fn name(&self) -> &'static str {
        match self {
            KnowledgeSourceType::Wikipedia => "wikipedia",
            KnowledgeSourceType::ArXiv => "arxiv",
            KnowledgeSourceType::SemanticScholar => "semantic-scholar",
            KnowledgeSourceType::GitHub => "github",
            KnowledgeSourceType::Book => "book",
            KnowledgeSourceType::WebPage => "web",
            KnowledgeSourceType::KnowledgeBase => "kb",
            KnowledgeSourceType::UserInput => "user",
            KnowledgeSourceType::Inferred => "inferred",
            KnowledgeSourceType::PdfLocal => "pdf-local",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: String,
    pub title: String,
    pub body: String,
    pub summary: String,
    pub source: KnowledgeSourceType,
    pub source_url: String,
    pub tags: Vec<String>,
    pub dimensions: Vec<String>,
    pub embedding: Option<Vec<f64>>,
    pub vsa: Option<VsaVector<4096>>,
    pub confidence: f64,
    pub importance: f64,
    pub created_at: i64,
    pub updated_at: i64,
    pub access_count: u64,
    pub related_ids: Vec<String>,
    /// SHA-256 of (source_url || quotation || timestamp),
    /// enabling independent provenance verification (N11 — Knowledge Value Provable).
    pub provenance_hash: Option<[u8; 32]>,
    /// Cross-reference proofs: (entity_id, entity_provenance_hash) for each
    /// referenced entry. verify_cross_references() checks every reference
    /// against the current state of the referenced entry's provenance hash.
    pub cross_references: Vec<(String, [u8; 32])>,
    /// Evidence record IDs linking this entry to verifiable sources
    pub evidence_ids: Vec<u64>,
}

impl KnowledgeEntry {
    pub fn new(title: &str, body: &str, source: KnowledgeSourceType, source_url: &str) -> Self {
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
            vsa: None,
            confidence: 0.7,
            importance: 0.5,
            created_at: now,
            updated_at: now,
            access_count: 0,
            related_ids: Vec::new(),
            provenance_hash: None,
            cross_references: Vec::new(),
            evidence_ids: Vec::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_dimensions(mut self, dims: Vec<String>) -> Self {
        self.dimensions = dims;
        self
    }

    pub fn with_confidence(mut self, c: f64) -> Self {
        self.confidence = c;
        self
    }

    pub fn with_importance(mut self, i: f64) -> Self {
        self.importance = i;
        self
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

/// Compute a deterministic provenance hash from source_url, quotation, and timestamp.
///
/// Uses SHA-256 over `source_url || "||" || quotation || "||" || timestamp_le_bytes`.
/// The result is a 32-byte digest that can be independently recomputed to verify
/// that a knowledge entry's source material has not been tampered with.
pub fn compute_provenance_hash(source_url: &str, quotation: &str, timestamp_ns: u64) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(source_url.as_bytes());
    hasher.update(b"||");
    hasher.update(quotation.as_bytes());
    hasher.update(b"||");
    hasher.update(timestamp_ns.to_le_bytes());
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Format a 32-byte hash as a lowercase hex string.
pub fn format_provenance_hash(hash: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for byte in hash {
        s.push_str(&format!("{:02x}", byte));
    }
    s
}

pub(crate) fn urlencoding(s: &str) -> String {
    s.replace(' ', "+")
}

pub(crate) fn extract_xml(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    xml.find(&open).and_then(|start| {
        let cs = start + open.len();
        xml[cs..]
            .find(&close)
            .map(|end| xml[cs..cs + end].trim().to_string())
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
        } else {
            break;
        }
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
        } else {
            break;
        }
    }
    cats
}

/// A knowledge entry with its linked evidence and a combined provenance-weighted score.
#[derive(Debug, Clone)]
pub struct KnowledgeEvidenceResult {
    pub entry: KnowledgeEntry,
    pub evidence: Vec<crate::core::nt_core_knowledge::evidence::EvidenceRecord>,
    pub combined_score: f64,
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
