//! Cross-module integration tests for NeoTrix.
//!
//! Tests full pipelines that span multiple subsystems:
//! memory lifecycle, entity linking, vector store, circuit breakers, and more.

use std::collections::HashMap;

use neotrix::l1_action::nt_infra_breaker::{BreakerConfig, BreakerRegistry, InfraBreaker};
use neotrix::l4_emotion::nt_memory::addressable_store::AddressableStore;
use neotrix::l4_emotion::nt_memory::consolidation::{
    MemoryCache, MemoryConsolidator, PromotionType, WorkingMemory,
};
use neotrix::l4_emotion::nt_memory::consolidation::consolidator::ConsolidationConfig;
use neotrix::l4_emotion::nt_memory::entity_linking::{
    EntityType, extract_entities, link_text,
};
use neotrix::l4_emotion::nt_memory::memory_types::TripleMemoryStore;
use neotrix::l2_perception::nt_core_vector_store::{
    BruteForceVectorStore, IndexConfig, VectorRecord, VectorStore,
};
use neotrix::l6_meta::healing::self_healing::auto_repair::{AutoRepair, FailureSeverity};
use neotrix::l6_meta::healing::self_healing::health_monitor::{ComponentMonitor, HealthStatusKind};

// ---------------------------------------------------------------------------
// Test 1: Memory entry lifecycle — append → recall → compact → cite
// ---------------------------------------------------------------------------

#[test]
fn test_memory_entry_lifecycle_append_recall_compact_cite() {
    let mut store = AddressableStore::new();

    // Append several observations
    let id1 = store.append("web_search", "query: rust async", "found tokio docs");
    let _id2 = store.append("code_review", "file: main.rs", "no issues found");
    let id3 = store.append("web_search", "query: rust error handling", "thiserror recommended");

    assert_eq!(store.len(), 3);
    assert!(!store.is_empty());

    // Recall by §id
    let obs = store.recall(&id1).unwrap();
    assert_eq!(obs.tool_name, "web_search");
    assert_eq!(obs.input, "query: rust async");

    // Recall missing returns None
    assert!(store.recall("§999").is_none());

    // Cite creates compact references
    let citations = store.cite(&[id1.clone(), id3.clone()]);
    assert_eq!(citations.len(), 2);
    assert!(citations[0].summary.contains("web_search"));

    // Compact reduces full observations
    let result = store.compact(1);
    assert_eq!(result.full_observations, 1);
    assert_eq!(result.cited_observations, 2);
    assert!(result.tokens_saved > 0);

    // Latest observation is still accessible
    let latest = store.latest().unwrap();
    assert_eq!(latest.id, id3);
}

// ---------------------------------------------------------------------------
// Test 2: Entity extraction → linking → index query
// ---------------------------------------------------------------------------

#[test]
fn test_entity_extraction_linking_index_query() {
    let text = "Alice Smith met with Bob Johnson at Google Inc. in New York on 2025-03-15. \
                 Later, Alice visited the Massachusetts Institute of Technology.";

    // Extract raw entities
    let raw_entities = extract_entities(text);
    assert!(!raw_entities.is_empty());

    // Link and index
    let index = link_text(text);
    assert!(!index.is_empty());

    // Should find persons
    let persons = index.search_by_type(EntityType::Person);
    assert!(persons.len() >= 2, "expected at least 2 persons, got {}", persons.len());

    // Should find organizations
    let orgs = index.search_by_type(EntityType::Org);
    assert!(!orgs.is_empty(), "expected at least 1 org");

    // MIT and full name should merge
    assert_eq!(orgs.len(), 1, "MIT variants should merge into one entity");

    // Entities should have unique IDs
    let ids: Vec<&str> = index.iter().map(|e| e.id.as_str()).collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len(), "all entity IDs must be unique");

    // Entities should have mentions
    for entity in index.iter() {
        assert!(!entity.mentions.is_empty());
    }
}

// ---------------------------------------------------------------------------
// Test 3: Vector store insert → search → filter pipeline
// ---------------------------------------------------------------------------

#[test]
fn test_vector_store_insert_search_filter_pipeline() {
    let config = IndexConfig {
        num_partitions: 2,
        ..IndexConfig::default()
    };
    let mut store = BruteForceVectorStore::new(config);

    // Insert records with metadata
    let mut science_meta = HashMap::new();
    science_meta.insert("domain".to_string(), "science".to_string());
    let mut art_meta = HashMap::new();
    art_meta.insert("domain".to_string(), "art".to_string());

    store
        .insert(
            VectorRecord::new("sci1".to_string(), vec![0x00; 4]).with_metadata(science_meta.clone()),
        )
        .unwrap();
    store
        .insert(
            VectorRecord::new("sci2".to_string(), vec![0x01; 4]).with_metadata(science_meta),
        )
        .unwrap();
    store
        .insert(
            VectorRecord::new("art1".to_string(), vec![0xFF; 4]).with_metadata(art_meta),
        )
        .unwrap();

    assert_eq!(store.len(), 3);

    // Unfiltered search returns results sorted by distance
    let results = store.search(&[0x00; 4], 3);
    assert_eq!(results.len(), 3);
    assert!(results[0].distance <= results[1].distance);

    // Filtered search returns only matching domain
    let mut filter = HashMap::new();
    filter.insert("domain".to_string(), "science".to_string());
    let filtered = store.search_with_filter(&[0x00; 4], 10, &filter);
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|r| r.id.starts_with("sci")));

    // Remove a record
    store.remove("sci1").unwrap();
    assert_eq!(store.len(), 2);
    assert!(store.remove("nonexistent").is_err());
}

// ---------------------------------------------------------------------------
// Test 4: Circuit breaker → trip → reset → resume (infra layer)
// ---------------------------------------------------------------------------

#[test]
fn test_infra_circuit_breaker_trip_reset_resume() {
    let config = BreakerConfig {
        error_threshold: 0.5,
        window_size: 4,
        open_duration_ms: 0, // instant for testing
        half_open_max_calls: 3,
    };
    let mut breaker = InfraBreaker::new(config);

    // Initially closed — calls allowed
    assert!(breaker.allow());
    assert_eq!(breaker.state(), neotrix_types::shared::BreakerState::Closed);

    // Record failures to exceed threshold
    breaker.record_result(false);
    breaker.record_result(false);
    breaker.record_result(true);
    breaker.record_result(false); // 75% error rate → open

    assert_eq!(breaker.state(), neotrix_types::shared::BreakerState::Open);
    assert!(!breaker.allow());

    // After duration, transitions to half-open
    breaker.record_result(true); // triggers state check
    assert!(breaker.allow() || breaker.state() == neotrix_types::shared::BreakerState::HalfOpen);
}

// ---------------------------------------------------------------------------
// Test 5: BreakerRegistry — per-capability circuit breaking
// ---------------------------------------------------------------------------

#[test]
fn test_breaker_registry_per_capability() {
    let mut registry = BreakerRegistry::new();

    // Default: all capabilities allowed
    assert!(registry.allow("capability_a"));
    assert!(registry.allow("capability_b"));

    // Record failures for capability_a only
    let config = BreakerConfig {
        error_threshold: 0.5,
        window_size: 2,
        open_duration_ms: 0,
        half_open_max_calls: 2,
    };
    registry = BreakerRegistry::with_config(config);
    registry.record_result("capability_a", false);
    registry.record_result("capability_a", false);

    // capability_a is now open, capability_b is still allowed
    assert!(!registry.allow("capability_a"));
    assert!(registry.allow("capability_b"));

    // States reflect per-capability state
    let states = registry.states();
    assert_eq!(
        states.get("capability_a"),
        Some(&neotrix_types::shared::BreakerState::Open)
    );
}

// ---------------------------------------------------------------------------
// Test 6: TripleMemoryStore — workflow + subtask + function memory
// ---------------------------------------------------------------------------

#[test]
fn test_triple_memory_store_stats() {
    let store = TripleMemoryStore::new();

    let (w, s, f) = store.stats();
    assert_eq!(w, 0);
    assert_eq!(s, 0);
    assert_eq!(f, 0);

    // Internal methods are crate-private, so we verify stats returns zeros
    // for a fresh store. Full workflow recording is tested in unit tests.
}

// ---------------------------------------------------------------------------
// Test 7: Working memory → add → recall → eviction
// ---------------------------------------------------------------------------

#[test]
fn test_working_memory_add_recall_eviction() {
    let mut wm = WorkingMemory::new(3);

    wm.add(neotrix::l4_emotion::nt_memory::consolidation::working::MemoryRef {
        id: "1".to_string(),
        content: "Rust ownership rules".to_string(),
        last_recalled_at: 0,
        recall_count: 0,
    });
    wm.add(neotrix::l4_emotion::nt_memory::consolidation::working::MemoryRef {
        id: "2".to_string(),
        content: "Tokio async runtime".to_string(),
        last_recalled_at: 0,
        recall_count: 0,
    });
    wm.add(neotrix::l4_emotion::nt_memory::consolidation::working::MemoryRef {
        id: "3".to_string(),
        content: "Error handling with thiserror".to_string(),
        last_recalled_at: 0,
        recall_count: 0,
    });

    assert_eq!(wm.len(), 3);

    // Add a 4th item → evicts LRU
    wm.add(neotrix::l4_emotion::nt_memory::consolidation::working::MemoryRef {
        id: "4".to_string(),
        content: "Serde serialization".to_string(),
        last_recalled_at: 0,
        recall_count: 0,
    });

    assert_eq!(wm.len(), 3);

    // Recall "rust" → should find the ownership entry
    let results = wm.recall("rust");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "1");
    assert_eq!(results[0].recall_count, 1);

    // Recall "async" → should find tokio
    let results = wm.recall("async");
    assert_eq!(results.len(), 1);
}

// ---------------------------------------------------------------------------
// Test 8: MemoryCache — TTL-based cache with stats
// ---------------------------------------------------------------------------

#[test]
fn test_memory_cache_ttl_hit_miss_eviction() {
    let mut cache = MemoryCache::new(3, 5); // max 3 entries, TTL 5 turns

    cache.put("k1".to_string(), "v1".to_string());
    cache.put("k2".to_string(), "v2".to_string());

    // Hit
    assert_eq!(cache.get("k1"), Some("v1".to_string()));

    // Miss
    assert_eq!(cache.get("missing"), None);

    let stats = cache.cache_stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert!(stats.hit_rate() > 0.0);

    // Eviction when at capacity
    cache.put("k3".to_string(), "v3".to_string());
    cache.put("k4".to_string(), "v4".to_string()); // should evict oldest
    let stats2 = cache.cache_stats();
    assert!(stats2.evictions > 0);

    // TTL expiration
    for _ in 0..10 {
        cache.get("k2"); // advance turns
    }
    // k2 should now be expired
    assert_eq!(cache.get("k2"), None);
}

// ---------------------------------------------------------------------------
// Test 9: Memory consolidation pipeline — short-term → core/archive/prune
// ---------------------------------------------------------------------------

#[test]
fn test_memory_consolidation_pipeline() {
    let config = ConsolidationConfig {
        access_count_core: 5,
        age_turns_archive: 100,
        novelty_prune: 0.2,
    };
    let consolidator = MemoryConsolidator::new(config);

    let entries = vec![
        // High access → Core
        neotrix::l4_emotion::nt_memory::consolidation::consolidator::ConsolidationMemoryEntry {
            id: "m1".to_string(),
            content: "frequently accessed".to_string(),
            access_count: 10,
            created_at_turn: 0,
            novelty: 0.8,
        },
        // Low novelty → Pruned
        neotrix::l4_emotion::nt_memory::consolidation::consolidator::ConsolidationMemoryEntry {
            id: "m2".to_string(),
            content: "boring entry".to_string(),
            access_count: 1,
            created_at_turn: 0,
            novelty: 0.1,
        },
        // Normal → Retained
        neotrix::l4_emotion::nt_memory::consolidation::consolidator::ConsolidationMemoryEntry {
            id: "m3".to_string(),
            content: "normal entry".to_string(),
            access_count: 2,
            created_at_turn: 0,
            novelty: 0.5,
        },
    ];

    let results = consolidator.consolidate(entries);
    assert_eq!(results.len(), 3);

    let m1 = results.iter().find(|r| r.original_id == "m1").unwrap();
    assert_eq!(m1.promotion_type, PromotionType::Core);

    let m2 = results.iter().find(|r| r.original_id == "m2").unwrap();
    assert_eq!(m2.promotion_type, PromotionType::Pruned);

    let m3 = results.iter().find(|r| r.original_id == "m3").unwrap();
    assert_eq!(m3.promotion_type, PromotionType::Retained);

    // Retained IDs excludes pruned
    let retained = consolidator.retained_ids(&results);
    assert!(retained.contains(&"m1"));
    assert!(retained.contains(&"m3"));
    assert!(!retained.contains(&"m2"));
}

// ---------------------------------------------------------------------------
// Test 10: Health monitor → auto repair → verify healing
// ---------------------------------------------------------------------------

#[test]
fn test_health_monitor_auto_repair_pipeline() {
    let mut monitor = ComponentMonitor::new();

    // Add a component that will be Critical (unknown → Critical)
    monitor.register_component("bogus_component");
    let status = monitor.check_component("bogus_component");
    assert_eq!(status.status, HealthStatusKind::Critical);

    // Auto-repair detects and heals
    let repair = AutoRepair::new(monitor);
    let failures = repair.detect_failures();
    assert!(!failures.is_empty());

    let bogus_failure = failures.iter().find(|f| f.component == "bogus_component").unwrap();
    assert_eq!(bogus_failure.severity, FailureSeverity::High);

    // Select and execute repair action
    let action = repair.select_action(bogus_failure);
    assert!(matches!(
        action,
        neotrix::l6_meta::healing::self_healing::auto_repair::RepairAction::RestartComponent { .. }
    ));

    let result = repair.repair(bogus_failure);
    assert!(result.success);

    // Full auto_heal cycle
    let results = repair.auto_heal();
    assert!(!results.is_empty());
    assert!(results.iter().all(|r| r.success));
}
