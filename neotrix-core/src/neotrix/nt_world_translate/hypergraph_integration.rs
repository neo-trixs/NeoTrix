use crate::core::nt_core_knowledge::hypergraph::{Hyperedge, HypergraphStore, NaryRelationType};
use crate::core::nt_core_translate::bilingual::BilingualEntry;
use crate::core::nt_core_translate::translate_engine::VsaTranslationEngine;

/// Stores a bilingual translation as a Composition hyperedge in the hypergraph.
pub fn store_translation_as_hyperedge(store: &mut HypergraphStore, entry: &BilingualEntry) -> bool {
    let edge = Hyperedge {
        id: format!("translation:{}", entry.id),
        entities: vec![
            format!("word:{}:{}", entry.source_lang.code(), entry.source_text),
            format!("word:{}:{}", entry.target_lang.code(), entry.target_text),
        ],
        relation_type: NaryRelationType::Composition,
        weight: entry.confidence,
        confidence: entry.confidence,
        context: format!(
            "VSA translation: {} {} → {} {}",
            entry.source_lang.code(),
            entry.source_text,
            entry.target_lang.code(),
            entry.target_text
        ),
        source_url: String::new(),
        created_at: entry.created_at,
        temporal_order: None,
        vsa_fingerprint: None,
    };
    store.insert(edge)
}

/// Queries the hypergraph for translations of a source text.
/// Returns (target_text, confidence) pairs matching the source and language pair.
pub fn query_translations_from_hypergraph(
    store: &HypergraphStore,
    source_text: &str,
    source_lang: &str,
    target_lang: &str,
) -> Vec<(String, f64)> {
    let entity = format!("word:{}:{}", source_lang, source_text);
    let prefix = format!("word:{}:", target_lang);

    store
        .hyperedges_for_entity(&entity)
        .into_iter()
        .filter(|e| matches!(e.relation_type, NaryRelationType::Composition))
        .filter(|e| e.entities.len() >= 2 && e.entities[1].starts_with(&prefix))
        .map(|e| {
            let target_text = e.entities[1][prefix.len()..].to_string();
            (target_text, e.confidence)
        })
        .collect()
}

/// Stores all entries from the translation engine's lexicon as hyperedges.
pub fn store_all_translations(store: &mut HypergraphStore, engine: &VsaTranslationEngine) {
    for entry in &engine.lexicon.entries {
        store_translation_as_hyperedge(store, entry);
    }
}

/// Stores only entries not yet present in the hypergraph.
/// Returns the count of new entries stored.
pub fn sync_lexicon_to_hypergraph(
    store: &mut HypergraphStore,
    engine: &mut VsaTranslationEngine,
) -> usize {
    let existing: std::collections::HashSet<String> = store
        .all_hyperedges()
        .iter()
        .filter(|e| e.id.starts_with("translation:"))
        .map(|e| e.id.clone())
        .collect();

    let mut count = 0;
    for entry in &engine.lexicon.entries {
        let edge_id = format!("translation:{}", entry.id);
        if !existing.contains(&edge_id) && store_translation_as_hyperedge(store, entry) {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
    use crate::core::nt_core_translate::bilingual::BilingualLexicon;
    use crate::core::nt_core_translate::language::Language;

    fn make_store() -> HypergraphStore {
        HypergraphStore::new(100)
    }

    fn make_entry(
        id: u64,
        src: &str,
        tgt: &str,
        src_lang: Language,
        tgt_lang: Language,
        confidence: f64,
    ) -> BilingualEntry {
        let source_vsa = BilingualLexicon::encode_text(src);
        let target_vsa = BilingualLexicon::encode_text(tgt);
        let bound_vsa = QuantizedVSA::bind(&source_vsa, &target_vsa);
        BilingualEntry {
            id,
            source_text: src.to_string(),
            target_text: tgt.to_string(),
            source_lang: src_lang,
            target_lang: tgt_lang,
            bound_vsa,
            source_vsa,
            target_vsa,
            confidence,
            access_count: 0,
            created_at: 1_000_000,
            evidence_ids: Vec::new(),
        }
    }

    #[test]
    fn test_store_and_query_translation() {
        let mut store = make_store();
        let entry = make_entry(
            1,
            "hello",
            "hola",
            Language::English,
            Language::Spanish,
            0.95,
        );
        assert!(store_translation_as_hyperedge(&mut store, &entry));

        let results = query_translations_from_hypergraph(&store, "hello", "en", "es");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "hola");
        assert!((results[0].1 - 0.95).abs() < 1e-10);
    }

    #[test]
    fn test_multiple_translations_same_source_word() {
        let mut store = make_store();
        let es = make_entry(
            1,
            "hello",
            "hola",
            Language::English,
            Language::Spanish,
            0.95,
        );
        let fr = make_entry(
            2,
            "hello",
            "bonjour",
            Language::English,
            Language::French,
            0.90,
        );
        store_translation_as_hyperedge(&mut store, &es);
        store_translation_as_hyperedge(&mut store, &fr);

        let es_results = query_translations_from_hypergraph(&store, "hello", "en", "es");
        assert_eq!(es_results.len(), 1);
        assert_eq!(es_results[0].0, "hola");

        let fr_results = query_translations_from_hypergraph(&store, "hello", "en", "fr");
        assert_eq!(fr_results.len(), 1);
        assert_eq!(fr_results[0].0, "bonjour");
    }

    #[test]
    fn test_empty_hypergraph_returns_empty_results() {
        let store = make_store();
        let results = query_translations_from_hypergraph(&store, "hello", "en", "es");
        assert!(results.is_empty());
    }

    #[test]
    fn test_store_all_translations_stores_entire_lexicon() {
        let mut store = make_store();
        let mut engine = VsaTranslationEngine::new();
        engine
            .lexicon
            .store("hello", "hola", Language::English, Language::Spanish, 0.95);
        engine.lexicon.store(
            "goodbye",
            "adiós",
            Language::English,
            Language::Spanish,
            0.90,
        );

        store_all_translations(&mut store, &engine);
        assert_eq!(store.count(), 2);

        let results = query_translations_from_hypergraph(&store, "hello", "en", "es");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "hola");
    }

    #[test]
    fn test_sync_lexicon_only_stores_new_entries() {
        let mut store = make_store();
        let mut engine = VsaTranslationEngine::new();
        engine
            .lexicon
            .store("hello", "hola", Language::English, Language::Spanish, 0.95);
        engine.lexicon.store(
            "goodbye",
            "adiós",
            Language::English,
            Language::Spanish,
            0.90,
        );

        // First sync should store both
        let count = sync_lexicon_to_hypergraph(&mut store, &mut engine);
        assert_eq!(count, 2);
        assert_eq!(store.count(), 2);

        // Add a third entry
        engine.lexicon.store(
            "thanks",
            "gracias",
            Language::English,
            Language::Spanish,
            0.98,
        );

        // Second sync should store only the new one
        let count = sync_lexicon_to_hypergraph(&mut store, &mut engine);
        assert_eq!(count, 1);
        assert_eq!(store.count(), 3);
    }
}
