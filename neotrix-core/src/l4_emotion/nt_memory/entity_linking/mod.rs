#![forbid(unsafe_code)]

//! Entity Linking — extraction, linking, and indexing of named entities.
//!
//! Pipeline:
//! 1. **Extract** (`extractor::extract_entities`): rule-based regex extraction
//!    produces raw `Entity` records with individual mentions.
//! 2. **Link** (`linker::EntityLinker::link`): deduplicates and merges
//!    same-entity mentions, assigns stable IDs, updates temporal windows.
//! 3. **Index** (`index::EntityIndex`): in-memory lookup by ID, name, or type.

pub mod entity;
pub mod extractor;
pub mod index;
pub mod linker;

pub use entity::{Entity, EntityType, Mention};
pub use extractor::extract_entities;
pub use index::EntityIndex;
pub use linker::{EntityLinker, LinkerConfig, levenshtein, merge_entities};

/// Run the full entity linking pipeline: extract → link → index.
///
/// Returns a ready-to-query `EntityIndex`.
pub fn link_text(text: &str) -> EntityIndex {
    let entities = extract_entities(text);
    let mut linker = EntityLinker::with_default_config();
    let linked = linker.link(entities);
    let mut index = EntityIndex::new();
    for e in linked {
        index.add(e);
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_pipeline() {
        let text = "Alice Smith met with Bob Johnson at Google Inc. in New York on 2025-03-15.";
        let index = link_text(text);

        assert!(!index.is_empty());

        // Should find persons
        let persons = index.search_by_type(EntityType::Person);
        assert!(persons.len() >= 2, "expected at least 2 persons, got {}", persons.len());

        // Should find organizations
        let orgs = index.search_by_type(EntityType::Org);
        assert!(!orgs.is_empty(), "expected at least 1 org");

        // Should find events (dates)
        let events = index.search_by_type(EntityType::Event);
        assert!(!events.is_empty(), "expected at least 1 event");
    }

    #[test]
    fn link_text_empty() {
        let index = link_text("");
        assert!(index.is_empty());
    }

    #[test]
    fn pipeline_deduplicates_entities() {
        let text = "MIT is great. Massachusetts Institute of Technology is in Cambridge.";
        let index = link_text(text);
        let orgs = index.search_by_type(EntityType::Org);
        assert_eq!(orgs.len(), 1, "MIT and full name should merge into one");
    }

    #[test]
    fn pipeline_assigns_unique_ids() {
        let text = "Alice Smith and Bob Johnson met.";
        let index = link_text(text);
        let ids: Vec<&str> = index.iter().map(|e| e.id.as_str()).collect();
        let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "all IDs should be unique");
    }

    #[test]
    fn pipeline_links_entities_have_mentions() {
        let text = "Google Inc is a tech company. Google was founded in 1998.";
        let index = link_text(text);
        for entity in index.iter() {
            assert!(!entity.mentions.is_empty(), "entity should have at least one mention");
        }
    }

    #[test]
    fn pipeline_text_with_only_stopwords() {
        let text = "the is a an of in on at to for";
        let index = link_text(text);
        assert!(index.is_empty());
    }

    #[test]
    fn pipeline_places_extracted() {
        let text = "We visited Central Park and Hyde Park.";
        let index = link_text(text);
        let places = index.search_by_type(EntityType::Place);
        assert!(places.len() >= 1, "expected at least one place");
    }

    #[test]
    fn pipeline_concepts_extracted() {
        let text = "We use \"deep learning\" and AI every day.";
        let index = link_text(text);
        let concepts = index.search_by_type(EntityType::Concept);
        assert!(concepts.len() >= 1, "expected at least one concept");
    }
}
