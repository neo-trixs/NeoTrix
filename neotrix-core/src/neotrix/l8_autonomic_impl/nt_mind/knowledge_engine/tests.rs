use super::*;

#[test]
fn test_entry_creation() {
    let e = KnowledgeEntry::new("Test Title", "Test body content", SourceType::Wikipedia, "https://example.com");
    assert_eq!(e.title, "Test Title");
    assert!(e.id.len() > 0);
}

#[test]
fn test_add_and_search() {
    let mut eng = KnowledgeEngine::new(100);
    eng.add_entry(KnowledgeEntry::new("Earth History", "The Earth formed 4.5 billion years ago", SourceType::Wikipedia, "url1"));
    eng.add_entry(KnowledgeEntry::new("Machine Learning", "ML is a subset of AI", SourceType::ArXiv, "url2"));

    let results = eng.search("Earth planet formation", 5);
    assert!(!results.is_empty());
    assert!(results[0].0.title.contains("Earth"));
}

#[test]
fn test_add_relation() {
    let mut eng = KnowledgeEngine::new(100);
    let id1 = eng.add_entry(KnowledgeEntry::new("A", "content a", SourceType::Wikipedia, "url1"));
    let id2 = eng.add_entry(KnowledgeEntry::new("B", "content b", SourceType::Wikipedia, "url2"));
    eng.add_relation(&id1, &id2, RelationType::References, 1.0, "A references B");

    let related = eng.get_related(&id1, 10);
    assert_eq!(related.len(), 1);
    assert_eq!(related[0].title, "B");
}

#[test]
fn test_search_by_tag() {
    let mut eng = KnowledgeEngine::new(100);
    eng.add_entry(KnowledgeEntry::new("Evolution", "text", SourceType::Wikipedia, "url").with_tags(vec!["biology".to_string()]));
    let results = eng.search_by_tag("biology", 10);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_stats() {
    let mut eng = KnowledgeEngine::new(100);
    eng.add_entry(KnowledgeEntry::new("A", "text", SourceType::Wikipedia, "url"));
    let stats = eng.stats();
    assert_eq!(stats.total_entries, 1);
    assert_eq!(stats.per_source.get("wikipedia").expect("result"), &1);
}

#[test]
fn test_remove_entry() {
    let mut eng = KnowledgeEngine::new(100);
    let id = eng.add_entry(KnowledgeEntry::new("A", "text", SourceType::Wikipedia, "url"));
    assert!(eng.remove_entry(&id));
    assert_eq!(eng.entries.len(), 0);
}

#[test]
fn test_max_entries() {
    let mut eng = KnowledgeEngine::new(3);
    eng.add_entry(KnowledgeEntry::new("A", "text", SourceType::Wikipedia, "u1").with_importance(0.9));
    eng.add_entry(KnowledgeEntry::new("B", "text", SourceType::Wikipedia, "u2").with_importance(0.8));
    eng.add_entry(KnowledgeEntry::new("C", "text", SourceType::Wikipedia, "u3").with_importance(0.7));
    eng.add_entry(KnowledgeEntry::new("D", "text", SourceType::Wikipedia, "u4").with_importance(1.0));
    assert!(eng.entries.len() <= 3);
}

#[test]
fn test_report() {
    let mut eng = KnowledgeEngine::new(100);
    eng.add_entry(KnowledgeEntry::new("Test", "body", SourceType::Wikipedia, "url"));
    let report = eng.report();
    assert!(report.contains("Test"));
}

#[test]
fn test_source_type_name() {
    assert_eq!(SourceType::Wikipedia.name(), "wikipedia");
    assert_eq!(SourceType::ArXiv.name(), "arxiv");
}

#[test]
fn test_strip_html() {
    assert_eq!(strip_html("<p>Hello <b>World</b></p>"), "Hello World");
}
