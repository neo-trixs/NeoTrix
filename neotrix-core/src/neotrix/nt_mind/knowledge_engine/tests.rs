use super::*;
use crate::neotrix::nt_mind::knowledge_engine::types::compute_provenance_hash;

#[test]
fn test_entry_creation() {
    let e = KnowledgeEntry::new(
        "Test Title",
        "Test body content",
        KnowledgeSourceType::Wikipedia,
        "https://example.com",
    );
    assert_eq!(e.title, "Test Title");
    assert!(e.id.len() > 0);
}

#[test]
fn test_add_and_search() {
    let mut eng = KnowledgeEngine::new(100);
    eng.add_entry(KnowledgeEntry::new(
        "Earth History",
        "The Earth formed 4.5 billion years ago",
        KnowledgeSourceType::Wikipedia,
        "url1",
    ));
    eng.add_entry(KnowledgeEntry::new(
        "Machine Learning",
        "ML is a subset of AI",
        KnowledgeSourceType::ArXiv,
        "url2",
    ));

    let results = eng.search("Earth planet formation", 5);
    assert!(!results.is_empty());
    assert!(results[0].0.title.contains("Earth"));
}

#[test]
fn test_add_relation() {
    let mut eng = KnowledgeEngine::new(100);
    let id1 = eng.add_entry(KnowledgeEntry::new(
        "A",
        "content a",
        KnowledgeSourceType::Wikipedia,
        "url1",
    ));
    let id2 = eng.add_entry(KnowledgeEntry::new(
        "B",
        "content b",
        KnowledgeSourceType::Wikipedia,
        "url2",
    ));
    eng.add_relation(&id1, &id2, RelationType::References, 1.0, "A references B");

    let related = eng.get_related(&id1, 10);
    assert_eq!(related.len(), 1);
    assert_eq!(related[0].title, "B");
}

#[test]
fn test_search_by_tag() {
    let mut eng = KnowledgeEngine::new(100);
    eng.add_entry(
        KnowledgeEntry::new("Evolution", "text", KnowledgeSourceType::Wikipedia, "url")
            .with_tags(vec!["biology".to_string()]),
    );
    let results = eng.search_by_tag("biology", 10);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_stats() {
    let mut eng = KnowledgeEngine::new(100);
    eng.add_entry(KnowledgeEntry::new(
        "A",
        "text",
        KnowledgeSourceType::Wikipedia,
        "url",
    ));
    let stats = eng.stats();
    assert_eq!(stats.total_entries, 1);
    assert_eq!(stats.per_source.get("wikipedia").expect("result"), &1);
}

#[test]
fn test_remove_entry() {
    let mut eng = KnowledgeEngine::new(100);
    let id = eng.add_entry(KnowledgeEntry::new(
        "A",
        "text",
        KnowledgeSourceType::Wikipedia,
        "url",
    ));
    assert!(eng.remove_entry(&id));
    assert_eq!(eng.entries.len(), 0);
}

#[test]
fn test_max_entries() {
    let mut eng = KnowledgeEngine::new(3);
    eng.add_entry(
        KnowledgeEntry::new("A", "text", KnowledgeSourceType::Wikipedia, "u1").with_importance(0.9),
    );
    eng.add_entry(
        KnowledgeEntry::new("B", "text", KnowledgeSourceType::Wikipedia, "u2").with_importance(0.8),
    );
    eng.add_entry(
        KnowledgeEntry::new("C", "text", KnowledgeSourceType::Wikipedia, "u3").with_importance(0.7),
    );
    eng.add_entry(
        KnowledgeEntry::new("D", "text", KnowledgeSourceType::Wikipedia, "u4").with_importance(1.0),
    );
    assert!(eng.entries.len() <= 3);
}

#[test]
fn test_report() {
    let mut eng = KnowledgeEngine::new(100);
    eng.add_entry(KnowledgeEntry::new(
        "Test",
        "body",
        KnowledgeSourceType::Wikipedia,
        "url",
    ));
    let report = eng.report();
    assert!(report.contains("Test"));
}

#[test]
fn test_source_type_name() {
    assert_eq!(KnowledgeSourceType::Wikipedia.name(), "wikipedia");
    assert_eq!(KnowledgeSourceType::ArXiv.name(), "arxiv");
}

#[test]
fn test_strip_html() {
    assert_eq!(strip_html("<p>Hello <b>World</b></p>"), "Hello World");
}

#[test]
fn test_search_by_vsa_sorted_order() {
    use crate::core::nt_core_hcube::vsa_vector::VsaBackend;
    use crate::core::nt_core_hcube::{MapVsaBackend, VsaVector};

    let mut eng = KnowledgeEngine::new(100);
    let id_a = eng.add_entry(KnowledgeEntry::new(
        "Alpha",
        "content alpha",
        KnowledgeSourceType::Wikipedia,
        "u1",
    ));
    let id_b = eng.add_entry(KnowledgeEntry::new(
        "Beta",
        "content beta",
        KnowledgeSourceType::ArXiv,
        "u2",
    ));
    let id_c = eng.add_entry(KnowledgeEntry::new(
        "Gamma",
        "content gamma",
        KnowledgeSourceType::GitHub,
        "u3",
    ));

    eng.encode_entry_vsa(&id_a, 10);
    eng.encode_entry_vsa(&id_b, 20);
    eng.encode_entry_vsa(&id_c, 30);

    let query = VsaVector::random(20);
    let results = eng.search_by_vsa(&query, 3);

    assert_eq!(results.len(), 3);
    let mut prev_sim = f64::MAX;
    for r in &results {
        let sim = MapVsaBackend.similarity(r.vsa.as_ref().expect("entry must have vsa"), &query);
        assert!(
            sim <= prev_sim + 1e-12,
            "results must be sorted descending by similarity"
        );
        prev_sim = sim;
    }
    assert_eq!(results[0].title, "Beta", "exact seed match must rank first");
}

// ── N11: Knowledge Value Provable tests ──────────────────────────────────

#[test]
fn test_provenance_hash_deterministic() {
    let h1 = compute_provenance_hash("https://example.com", "the earth is round", 1_700_000_000);
    let h2 = compute_provenance_hash("https://example.com", "the earth is round", 1_700_000_000);
    assert_eq!(h1, h2, "same inputs must produce same hash");

    let h3 = compute_provenance_hash("https://example.com", "the earth is flat", 1_700_000_000);
    assert_ne!(h1, h3, "different quotation must produce different hash");
}

#[test]
fn test_cross_reference_verification() {
    let mut eng = KnowledgeEngine::new(100);

    let src_id = eng.add_entry(KnowledgeEntry::new(
        "Source",
        "source content",
        KnowledgeSourceType::WebPage,
        "https://source.com",
    ));
    let ref_id = eng.add_entry(KnowledgeEntry::new(
        "Reference",
        "reference content",
        KnowledgeSourceType::ArXiv,
        "https://arxiv.org/abs/1234",
    ));

    eng.store_with_provenance(
        &src_id,
        "https://source.com",
        "source content",
        1_700_000_000,
    )
    .unwrap();
    let ref_hash = eng
        .store_with_provenance(
            &ref_id,
            "https://arxiv.org/abs/1234",
            "reference content",
            1_700_000_000,
        )
        .unwrap();

    // Add cross-reference from src to ref
    if let Some(entry) = eng.entries.get_mut(&src_id) {
        entry.cross_references.push((ref_id.clone(), ref_hash));
    }

    // Valid cross-reference should verify
    assert!(
        eng.verify_cross_references(&src_id),
        "valid cross-reference must verify"
    );

    // Tamper with the referenced entry's body → provenance hash changes
    if let Some(entry) = eng.entries.get_mut(&ref_id) {
        entry.body = "tampered content".to_string();
    }

    // Cross-reference should now fail
    assert!(
        !eng.verify_cross_references(&src_id),
        "cross-reference must detect tampering"
    );
}

#[test]
fn test_knowledge_engine_provenance_wire_up() {
    let mut eng = KnowledgeEngine::new(100);
    let id = eng.add_entry(KnowledgeEntry::new(
        "Provable Entry",
        "verifiable content",
        KnowledgeSourceType::Wikipedia,
        "https://wiki.com/provable",
    ));

    // Store provenance
    let hash = eng.store_with_provenance(
        &id,
        "https://wiki.com/provable",
        "verifiable content",
        1_700_000_000,
    );
    assert!(hash.is_ok(), "store_with_provenance must succeed");

    // Verify provenance integrity
    assert!(
        eng.verify_entry_provenance(&id),
        "freshly stored provenance must verify"
    );

    // Tamper
    if let Some(entry) = eng.entries.get_mut(&id) {
        entry.body = "tampered".to_string();
    }
    assert!(
        !eng.verify_entry_provenance(&id),
        "tampered entry must fail verification"
    );

    // Report
    let report = eng.report_provenance(&id);
    assert!(report.contains("Provable Entry"));
    assert!(
        report.contains("TAMPERED") || report.contains("✗"),
        "report must indicate tampering"
    );
}
