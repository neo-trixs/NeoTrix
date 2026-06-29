pub mod file_perception;

use crate::core::nt_core_consciousness::vsa_tag::{
    SenseModality, VsaOrigin, VsaTagged, VsaWorldCategory,
};
use crate::core::nt_core_hcube::cross_modal::CrossModalAligner;

pub use file_perception::{
    DocumentFormat, DocumentPerceptionModule, LayoutType, ParseResult, PerceptionReport,
};

/// Result of parsing a document into structured perceptions
#[derive(Debug, Clone)]
pub struct DocumentPercept {
    /// Parsed fields: key → value string pairs
    pub fields: Vec<(String, String)>,
    /// Confidence in this perception (0.0–1.0)
    pub confidence: f64,
    /// Detected layout pattern
    pub layout: LayoutType,
    /// Source file path
    pub source: String,
    /// VSA-encoded vector for consciousness ingestion
    pub vsa_vector: Vec<u8>,
    /// Timestamp in ms since epoch
    pub timestamp_ms: i64,
}

/// Convert DocumentPercept → VsaTagged for the consciousness pipeline
pub fn percept_to_vsa_tagged(percept: &DocumentPercept, aligner: &CrossModalAligner) -> VsaTagged {
    // Encode all fields into a single deterministic VSA
    let mut combined = String::new();
    for (k, v) in &percept.fields {
        combined.push_str(k);
        combined.push(':');
        combined.push_str(v);
        combined.push('|');
    }
    let vsa = aligner.text_to_vsa(&combined);

    VsaTagged {
        vector: vsa,
        tag: VsaOrigin::World(VsaWorldCategory::Sensor),
        sense_modality: Some(SenseModality::Document),
        confidence: percept.confidence,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        salience: percept.confidence.max(0.5),
        provenance: Some(crate::core::nt_core_consciousness::source_hierarchy::ProvenanceChain::new(
            vec![(crate::core::nt_core_consciousness::source_hierarchy::KnowledgeLayer::Raw(
                crate::core::nt_core_consciousness::source_hierarchy::PerceptionMeta {
                    source_type: crate::core::nt_core_consciousness::source_hierarchy::PerceptionSource::FileRead,
                    raw_confidence: percept.confidence,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as i64,
                }
            ), 1000)]
        )),
        prediction: None,
        outcome: None,
    }
}
