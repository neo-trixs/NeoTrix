//! Tiered Pipeline — 分层内存管线 (M1 from LightMem)
//!
//! Pipeline stages: compress → segment → extract → index → retrieve
//! Implements the "Context as Scarce Resource" axiom (A2): context window /
//! KV capacity is the fundamental bottleneck. TieredPipeline compresses and
//! selectively retains only salient information across tiers.

use std::fmt;

/// Pipeline error type
#[derive(Debug, Clone)]
pub enum PipelineError {
    /// Compression failed — input too short or malformed
    CompressFailed(String),
    /// Segmentation failed — no meaningful segments found
    SegmentEmpty,
    /// Extraction failed — threshold too high, nothing extracted
    ExtractEmpty,
    /// Indexing failed — backend unavailable
    IndexFailed(String),
    /// Retrieval failed — query malformed
    RetrieveFailed(String),
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CompressFailed(msg) => write!(f, "compress failed: {msg}"),
            Self::SegmentEmpty => write!(f, "segmentation produced no segments"),
            Self::ExtractEmpty => write!(f, "extraction produced no entries"),
            Self::IndexFailed(msg) => write!(f, "index failed: {msg}"),
            Self::RetrieveFailed(msg) => write!(f, "retrieve failed: {msg}"),
        }
    }
}

impl std::error::Error for PipelineError {}

pub type Result<T> = std::result::Result<T, PipelineError>;

/// Compressed chunk — output of the compress stage
#[derive(Debug, Clone)]
pub struct CompressedChunk {
    /// Compressed text representation
    pub text: String,
    /// Compression ratio (compressed_len / original_len)
    pub compression_ratio: f64,
    /// Number of tokens preserved after compression
    pub preserved_tokens: usize,
}

/// Topic segment — output of the segment stage
#[derive(Debug, Clone)]
pub struct TopicSegment {
    /// Unique segment identifier
    pub topic_id: u32,
    /// Segment content text
    pub content: String,
    /// Segment importance score [0.0, 1.0]
    pub importance: f64,
    /// Named entities detected in segment
    pub entities: Vec<String>,
}

/// Memory entry — output of the extract stage
#[derive(Debug, Clone)]
pub struct TieredPipelineTieredPipelineMemoryEntry {
    /// Unique entry identifier
    pub id: String,
    /// Entry content
    pub content: String,
    /// Importance score [0.0, 1.0]
    pub importance: f64,
    /// Source segment topic ID
    pub source_topic_id: u32,
    /// Associated entity names
    pub entities: Vec<String>,
}

/// Multi-modal index result — output of the index stage
#[derive(Debug, Clone)]
pub struct IndexResult {
    /// Vector embedding IDs
    pub vector_ids: Vec<u64>,
    /// BM25 indexed terms
    pub bm25_terms: Vec<String>,
    /// Entity link pairs (entity_name, entry_id)
    pub entity_links: Vec<(String, String)>,
}

/// Tiered pipeline trait — five-stage compress → segment → extract → index → retrieve
///
/// Generic over Item/Compressed/Indexed to allow domain-specific implementations.
pub trait TieredPipeline {
    /// Raw input item type
    type Item;
    /// Compressed representation type
    type Compressed;
    /// Indexed representation type
    type Indexed;

    /// Stage 1: Pre-compress raw input (LLMLingua-2 style)
    fn compress(&self, item: &Self::Item) -> Result<Self::Compressed>;

    /// Stage 2: Segment compressed chunk into topic segments
    fn segment(&self, compressed: &Self::Compressed) -> Result<Vec<TopicSegment>>;

    /// Stage 3: Extract memory entries from segments above threshold
    fn extract(&self, segments: &[TopicSegment], threshold: f64) -> Result<Vec<TieredPipelineMemoryEntry>>;

    /// Stage 4: Build multi-modal index (vector + BM25 + entity)
    fn index(&self, entries: &[TieredPipelineMemoryEntry]) -> Result<Self::Indexed>;

    /// Stage 5: Retrieve relevant items by query against the index
    fn retrieve(&self, query: &str, indexed: &Self::Indexed) -> Result<Vec<Self::Item>>;
}

/// Default text-based tiered pipeline — concrete implementation for raw strings
#[derive(Debug, Clone)]
pub struct TextTieredPipeline {
    /// Minimum compression ratio to keep a segment
    pub min_segment_importance: f64,
    /// Default extraction threshold
    pub default_threshold: f64,
}

impl TextTieredPipeline {
    pub fn new(min_segment_importance: f64, default_threshold: f64) -> Self {
        Self {
            min_segment_importance: min_segment_importance.clamp(0.0, 1.0),
            default_threshold: default_threshold.clamp(0.0, 1.0),
        }
    }

    /// Estimate token count (simple whitespace split)
    fn estimate_tokens(text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// Compute compression ratio
    fn compression_ratio(original: &str, compressed: &str) -> f64 {
        if original.is_empty() {
            return 0.0;
        }
        compressed.len() as f64 / original.len() as f64
    }

    /// Extract entities using simple heuristic (capitalized words)
    fn extract_entities(text: &str) -> Vec<String> {
        text.split_whitespace()
            .filter(|w| {
                w.len() > 1
                    && w.chars().next().map_or(false, |c| c.is_uppercase())
                    && w.chars().skip(1).all(|c| c.is_lowercase() || c == '\'')
            })
            .map(|w| w.to_string())
            .collect()
    }

    /// Segment text by sentence boundaries
    fn segment_text(text: &str, topic_id_base: u32) -> Vec<TopicSegment> {
        text.split(|c: char| c == '.' || c == '!' || c == '?')
            .filter(|s| !s.trim().is_empty())
            .enumerate()
            .map(|(i, sentence)| {
                let trimmed = sentence.trim().to_string();
                let importance = (trimmed.len() as f64 / 200.0).min(1.0);
                TopicSegment {
                    topic_id: topic_id_base + i as u32,
                    content: trimmed,
                    importance,
                    entities: Self::extract_entities(sentence),
                }
            })
            .collect()
    }
}

impl TieredPipeline for TextTieredPipeline {
    type Item = String;
    type Compressed = CompressedChunk;
    type Indexed = IndexResult;

    fn compress(&self, item: &String) -> Result<CompressedChunk> {
        if item.trim().is_empty() {
            return Err(PipelineError::CompressFailed("empty input".into()));
        }

        let _original_tokens = Self::estimate_tokens(item);
        let compressed_text = item
            .split_whitespace()
            .filter(|w| w.len() > 2 || w.chars().next().map_or(false, |c| c.is_uppercase()))
            .collect::<Vec<_>>()
            .join(" ");

        if compressed_text.is_empty() {
            return Err(PipelineError::CompressFailed(
                "all tokens filtered out".into(),
            ));
        }

        Ok(CompressedChunk {
            compression_ratio: Self::compression_ratio(item, &compressed_text),
            preserved_tokens: Self::estimate_tokens(&compressed_text),
            text: compressed_text,
        })
    }

    fn segment(&self, compressed: &CompressedChunk) -> Result<Vec<TopicSegment>> {
        let segments = Self::segment_text(&compressed.text, 0);
        if segments.is_empty() {
            return Err(PipelineError::SegmentEmpty);
        }
        Ok(segments
            .into_iter()
            .filter(|s| s.importance >= self.min_segment_importance)
            .collect())
    }

    fn extract(
        &self,
        segments: &[TopicSegment],
        threshold: f64,
    ) -> Result<Vec<TieredPipelineMemoryEntry>> {
        let entries: Vec<TieredPipelineMemoryEntry> = segments
            .iter()
            .filter(|s| s.importance >= threshold)
            .enumerate()
            .map(|(i, seg)| TieredPipelineMemoryEntry {
                id: format!("entry_{}", i),
                content: seg.content.clone(),
                importance: seg.importance,
                source_topic_id: seg.topic_id,
                entities: seg.entities.clone(),
            })
            .collect();

        if entries.is_empty() {
            return Err(PipelineError::ExtractEmpty);
        }
        Ok(entries)
    }

    fn index(&self, entries: &[TieredPipelineMemoryEntry]) -> Result<IndexResult> {
        let mut vector_ids = Vec::new();
        let mut bm25_terms = Vec::new();
        let mut entity_links = Vec::new();

        for (i, entry) in entries.iter().enumerate() {
            vector_ids.push(i as u64);

            for word in entry.content.split_whitespace() {
                if !bm25_terms.contains(&word.to_lowercase()) {
                    bm25_terms.push(word.to_lowercase());
                }
            }

            for entity in &entry.entities {
                entity_links.push((entity.clone(), entry.id.clone()));
            }
        }

        Ok(IndexResult {
            vector_ids,
            bm25_terms,
            entity_links,
        })
    }

    fn retrieve(&self, query: &str, indexed: &IndexResult) -> Result<Vec<String>> {
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let matched_terms: Vec<&String> = indexed
            .bm25_terms
            .iter()
            .filter(|term| query_terms.iter().any(|qt| term.contains(qt)))
            .collect();

        if matched_terms.is_empty() {
            return Err(PipelineError::RetrieveFailed(
                "no matching terms found".into(),
            ));
        }

        Ok(matched_terms.into_iter().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_preserves重要内容() {
        let pipeline = TextTieredPipeline::new(0.1, 0.1);
        let result = pipeline
            .compress(&"The NeoTrix consciousness module implements Global Workspace Theory routing"
                .to_string())
            .unwrap();
        assert!(result.preserved_tokens > 0);
        assert!(result.compression_ratio > 0.0);
        assert!(result.compression_ratio <= 1.0);
    }

    #[test]
    fn test_segment_produces_topics() {
        let pipeline = TextTieredPipeline::new(0.0, 0.1);
        let chunk = CompressedChunk {
            text: "First topic about E8. Second topic about memory. Third topic about routing."
                .into(),
            compression_ratio: 0.8,
            preserved_tokens: 12,
        };
        let segments = pipeline.segment(&chunk).unwrap();
        assert!(!segments.is_empty());
    }

    #[test]
    fn test_extract_filters_by_threshold() {
        let pipeline = TextTieredPipeline::new(0.0, 0.5);
        let segments = vec![
            TopicSegment {
                topic_id: 0,
                content: "Important segment".into(),
                importance: 0.9,
                entities: vec!["NeoTrix".into()],
            },
            TopicSegment {
                topic_id: 1,
                content: "Unimportant segment".into(),
                importance: 0.2,
                entities: vec![],
            },
        ];
        let entries = pipeline.extract(&segments, 0.5).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].importance, 0.9);
    }

    #[test]
    fn test_full_pipeline_roundtrip() {
        let pipeline = TextTieredPipeline::new(0.0, 0.1);
        let input = "NeoTrix implements memory tiering with compression. The E8 manifold guides attention routing. Memory segments are indexed by importance."
            .to_string();

        let compressed = pipeline.compress(&input).unwrap();
        let segments = pipeline.segment(&compressed).unwrap();
        let entries = pipeline.extract(&segments, 0.1).unwrap();
        let indexed = pipeline.index(&entries).unwrap();

        assert!(!indexed.vector_ids.is_empty());
        assert!(!indexed.bm25_terms.is_empty());
    }

    #[test]
    fn test_empty_input_errors() {
        let pipeline = TextTieredPipeline::new(0.0, 0.1);
        assert!(pipeline.compress(&String::new()).is_err());
    }
}
