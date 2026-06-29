use std::collections::HashMap;

use crate::core::nt_core_consciousness::memory_lattice::{
    LatticeEntry, LatticeLayer, MemoryLattice, MemoryOrigin,
};
use crate::core::nt_core_knowledge::entity_extractor::EntityExtractor;

/// Result of compressing a piece of content through the SimpleMem-style
/// semantic compression pipeline.
#[derive(Debug, Clone)]
pub struct CompressedEntry {
    pub entities: Vec<String>,
    pub entity_types: Vec<String>,
    pub relations: Vec<String>,
    pub summary: String,
    pub key_terms: Vec<String>,
    pub original_length: usize,
    pub compressed_length: usize,
    pub compression_ratio: f64,
    pub confidence: f64,
}

/// SimpleMem-style semantic compressor (arXiv 2601.02553).
///
/// Provides a 3-stage pipeline baseline:
/// 1. Semantic compression (entity extraction + summarization)
/// 2. Multi-view indexing (via LatticeLayer dispatch)
/// 3. Adaptive retrieval (delegated to MemoryLattice)
#[derive(Clone)]
pub struct SemanticCompressor {
    max_summary_chars: usize,
    min_entity_confidence: f64,
    extractor: Option<EntityExtractor>,

    total_original_chars: u64,
    total_compressed_chars: u64,
    total_entries: u64,
}

impl SemanticCompressor {
    pub fn new(max_summary_chars: usize) -> Self {
        Self {
            max_summary_chars,
            min_entity_confidence: 0.5,
            extractor: None,
            total_original_chars: 0,
            total_compressed_chars: 0,
            total_entries: 0,
        }
    }

    pub fn with_entity_extractor(mut self, extractor: EntityExtractor) -> Self {
        self.extractor = Some(extractor);
        self
    }

    pub fn compress(&mut self, content: &str) -> CompressedEntry {
        let original_length = content.len();

        let (entities, entity_types, relations) = self.extract_entities_and_relations(content);

        let summary = if content.len() <= self.max_summary_chars {
            content.to_string()
        } else {
            let boundary = self.find_sentence_boundary(content, self.max_summary_chars);
            content[..boundary].to_string()
        };

        let key_terms = self.extract_key_terms(content);

        let compressed_length = summary.len();
        let compression_ratio = if original_length > 0 {
            compressed_length as f64 / original_length as f64
        } else {
            1.0
        };

        self.total_original_chars += original_length as u64;
        self.total_compressed_chars += compressed_length as u64;
        self.total_entries += 1;

        CompressedEntry {
            entities,
            entity_types,
            relations,
            summary,
            key_terms,
            original_length,
            compressed_length,
            compression_ratio,
            confidence: self.confidence_from_extraction(),
        }
    }

    pub fn compress_to_lattice_entry(
        &mut self,
        content: &str,
        vsa_hash: Vec<u8>,
        layer: LatticeLayer,
    ) -> LatticeEntry {
        let compressed = self.compress(content);
        let metadata = format!(
            "entities={}|key_terms={}|ratio={:.3}|conf={:.2}",
            compressed.entities.join(","),
            compressed.key_terms.join(","),
            compressed.compression_ratio,
            compressed.confidence,
        );
        let store_content = format!("{} [compressed: {}]", content, metadata);
        LatticeEntry {
            content: store_content,
            vsa_hash,
            layer,
            confidence: compressed.confidence,
            invocation_count: 0,
            last_accessed: 0,
            source_layer: None,
            consolidated: false,
            q_value: 0.5,
            valid_from: None,
            valid_to: None,
            origin: MemoryOrigin::default(),
            provenance_parent: None,
            belief_state: crate::core::nt_core_consciousness::memory_lattice::BeliefState::Inferred,
            domain: "general".to_string(),
        }
    }

    pub fn stats(&self) -> (u64, u64, u64, f64) {
        let avg_ratio = if self.total_entries > 0 {
            self.total_compressed_chars as f64 / self.total_original_chars as f64
        } else {
            1.0
        };
        (
            self.total_entries,
            self.total_original_chars,
            self.total_compressed_chars,
            avg_ratio,
        )
    }

    // ── Private helpers ──

    fn extract_entities_and_relations(
        &self,
        content: &str,
    ) -> (Vec<String>, Vec<String>, Vec<String>) {
        if let Some(ref extractor) = self.extractor {
            let mentions = extractor.extract_entities(content);
            let entities: Vec<String> = mentions
                .iter()
                .filter(|m| m.confidence >= self.min_entity_confidence)
                .map(|m| m.name.clone())
                .collect();
            let entity_types: Vec<String> = mentions
                .iter()
                .filter(|m| m.confidence >= self.min_entity_confidence)
                .map(|m| m.entity_type.name().to_string())
                .collect();

            if content.len() > 10 {
                let relations = extractor.extract_relations(content, &mentions);
                let rel_strings: Vec<String> = relations
                    .iter()
                    .map(|r| format!("{} {} {}", r.subject, r.relation.name(), r.object))
                    .collect();
                (entities, entity_types, rel_strings)
            } else {
                (entities, entity_types, Vec::new())
            }
        } else {
            let (entities, entity_types) = self.heuristic_entity_extraction(content);
            let relations = self.heuristic_relation_extraction(content);
            (entities, entity_types, relations)
        }
    }

    fn heuristic_entity_extraction(&self, content: &str) -> (Vec<String>, Vec<String>) {
        let mut seen = std::collections::HashSet::new();
        let mut entities = Vec::new();
        let mut entity_types = Vec::new();

        for word in content.split_whitespace() {
            let cleaned = word
                .trim_matches(|c: char| c.is_ascii_punctuation())
                .to_string();
            if cleaned.len() < 2 {
                continue;
            }

            let first = cleaned.chars().next().unwrap();
            if first.is_uppercase() && !self.is_stop_word(&cleaned) {
                if seen.insert(cleaned.clone()) {
                    // Classify entity type heuristically
                    let etype = if cleaned
                        .chars()
                        .all(|c| c.is_uppercase() && c.is_ascii_uppercase())
                    {
                        "acronym"
                    } else if cleaned.len() > 2
                        && cleaned[cleaned.len() - 2..].to_lowercase() == "s"
                    {
                        // Plural → likely organization/group
                        "organization"
                    } else {
                        "concept"
                    };
                    entities.push(cleaned);
                    entity_types.push(etype.to_string());
                }
            }
        }

        (entities, entity_types)
    }

    fn heuristic_relation_extraction(&self, _content: &str) -> Vec<String> {
        Vec::new()
    }

    fn extract_key_terms(&self, content: &str) -> Vec<String> {
        let mut freq: HashMap<String, usize> = HashMap::new();
        for word in content.split_whitespace() {
            let cleaned: String = word
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if cleaned.len() > 3 && !self.is_stop_word(&cleaned) {
                *freq.entry(cleaned.to_lowercase()).or_default() += 1;
            }
        }
        let mut terms: Vec<(usize, String)> = freq
            .into_iter()
            .map(|(word, count)| (count, word))
            .collect();
        terms.sort_by(|a, b| b.0.cmp(&a.0));
        terms.truncate(10);
        terms.into_iter().map(|(_, word)| word).collect()
    }

    fn is_stop_word(&self, word: &str) -> bool {
        matches!(
            word.to_lowercase().as_str(),
            "the"
                | "this"
                | "that"
                | "with"
                | "from"
                | "have"
                | "been"
                | "were"
                | "will"
                | "what"
                | "when"
                | "where"
                | "which"
                | "their"
                | "there"
                | "about"
                | "would"
                | "could"
                | "should"
                | "into"
                | "over"
                | "such"
                | "only"
                | "also"
                | "than"
                | "then"
                | "very"
                | "just"
                | "like"
                | "does"
                | "more"
                | "some"
                | "them"
                | "well"
                | "here"
                | "each"
                | "other"
                | "being"
                | "made"
                | "much"
                | "still"
                | "done"
                | "most"
        )
    }

    fn find_sentence_boundary(&self, text: &str, start: usize) -> usize {
        let search_start = start.saturating_sub(30);
        let search_end = (start + 30).min(text.len());
        // Try to find a sentence boundary (., !, ?) within range
        if let Some(pos) =
            text[search_start..search_end].rfind(|c: char| c == '.' || c == '!' || c == '?')
        {
            search_start + pos + 1
        } else if let Some(pos) =
            text[search_start..search_end].rfind(|c: char| c == ',' || c == ';' || c == ':')
        {
            search_start + pos + 1
        } else {
            start
        }
    }

    fn confidence_from_extraction(&self) -> f64 {
        if self.extractor.is_some() {
            0.85
        } else {
            0.55
        }
    }
}

/// Store content to MemoryLattice through the semantic compression pipeline.
/// Returns the LatticeLayer index and compression summary.
pub fn compress_and_store(
    compressor: &mut SemanticCompressor,
    lattice: &mut MemoryLattice,
    content: &str,
    vsa_hash: Vec<u8>,
    layer: LatticeLayer,
) -> (usize, String) {
    let compressed = compressor.compress(content);
    let store_content = format!(
        "{} [entities={}|terms={}]",
        compressed.summary,
        compressed.entities.join(","),
        compressed.key_terms.join(","),
    );

    lattice.store(store_content, vsa_hash, layer);

    let idx = match layer {
        LatticeLayer::Episodic => lattice.episodic.len().saturating_sub(1),
        LatticeLayer::Facts => lattice.facts.len().saturating_sub(1),
        LatticeLayer::Skills => lattice.skills.len().saturating_sub(1),
        LatticeLayer::MetaRules => lattice.meta_rules.len().saturating_sub(1),
        LatticeLayer::Identity => lattice.identity.len().saturating_sub(1),
    };

    let summary = format!(
        "compressed {}→{} chars ({:.1}×, ratio={:.3}, conf={:.2})",
        compressed.original_length,
        compressed.compressed_length,
        if compressed.compressed_length > 0 {
            compressed.original_length as f64 / compressed.compressed_length as f64
        } else {
            1.0
        },
        compressed.compression_ratio,
        compressed.confidence,
    );

    (idx, summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_content() {
        let mut compressor = SemanticCompressor::new(200);
        let entry = compressor.compress("");
        assert!(entry.entities.is_empty());
        assert!(entry.summary.is_empty());
        assert_eq!(entry.original_length, 0);
        assert_eq!(entry.compression_ratio, 1.0);
    }

    #[test]
    fn test_short_content() {
        let mut compressor = SemanticCompressor::new(200);
        let short = "Hello world, this is a short test.";
        let entry = compressor.compress(short);
        assert_eq!(entry.summary, short);
        assert_eq!(entry.compression_ratio, 1.0);
    }

    #[test]
    fn test_long_content_truncated() {
        let mut compressor = SemanticCompressor::new(50);
        let long = "This is a very long piece of content that should definitely exceed the maximum summary character limit and therefore be truncated appropriately by the compressor.";
        let entry = compressor.compress(long);
        assert!(entry.summary.len() <= 80); // 50 + sentence boundary wiggle
        assert!(entry.compression_ratio < 1.0);
    }

    #[test]
    fn test_heuristic_entity_extraction() {
        let mut compressor = SemanticCompressor::new(200);
        let text = "Alice works at Acme Corporation in Berlin. Bob likes Python.";
        let entry = compressor.compress(text);
        // Capitalized words detected heuristically: Alice, Acme, Corporation, Berlin, Bob, Python
        assert!(
            !entry.entities.is_empty(),
            "should find capitalized entities"
        );
        assert!(
            entry.entities.contains(&"Alice".to_string())
                || entry.entities.contains(&"Acme".to_string())
                || entry.entities.contains(&"Berlin".to_string())
                || entry.entities.contains(&"Python".to_string())
        );
    }

    #[test]
    fn test_key_terms_frequency() {
        let mut compressor = SemanticCompressor::new(200);
        let text = "neural network architecture neural network training neural network inference neural network optimization";
        let entry = compressor.compress(text);
        assert!(entry.key_terms.contains(&"neural".to_string()));
        assert!(entry.key_terms.contains(&"network".to_string()));
    }

    #[test]
    fn test_compression_stats_tracking() {
        let mut compressor = SemanticCompressor::new(100);
        compressor.compress("short");
        compressor.compress("a bit longer content to compress");
        compressor.compress(
            "this is a much longer piece that will definitely trigger truncation because it exceeds one hundred characters quite easily with all this extra text",
        );
        let (total, orig, comp, avg) = compressor.stats();
        assert_eq!(total, 3);
        assert!(orig > 0);
        assert!(comp > 0);
        assert!(avg > 0.0);
    }

    #[test]
    fn test_compress_to_lattice_entry() {
        let mut compressor = SemanticCompressor::new(200);
        let entry = compressor.compress_to_lattice_entry(
            "Test content for lattice entry conversion",
            vec![1, 2, 3, 4],
            LatticeLayer::Episodic,
        );
        assert!(entry.content.contains("Test content"));
        assert_eq!(entry.vsa_hash, vec![1, 2, 3, 4]);
        assert_eq!(entry.layer, LatticeLayer::Episodic);
        assert_eq!(entry.confidence, 0.55);
    }

    #[test]
    fn test_compress_and_store_function() {
        let mut compressor = SemanticCompressor::new(200);
        let mut lattice = MemoryLattice::new();

        let (idx, summary) = compress_and_store(
            &mut compressor,
            &mut lattice,
            "Semantic compression test data for storage pipeline",
            vec![10, 20, 30],
            LatticeLayer::Facts,
        );

        assert_eq!(idx, 0);
        assert!(summary.contains("compressed"));
        assert!(summary.contains("ratio="));
    }
}
