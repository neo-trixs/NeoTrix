//! Memory subsystem integration tests for NeoTrix.
//!
//! Tests tiered memory consolidation, distillation, vector store operations,
//! and memory lifecycle management.


use neotrix::l4_emotion::nt_memory::consolidation::cache::MemoryCache;
use neotrix::l4_emotion::nt_memory::consolidation::consolidator::{
    ConsolidationConfig, ConsolidationMemoryEntry, MemoryConsolidator, PromotionType,
};
use neotrix::l4_emotion::nt_memory::consolidation::working::{MemoryRef, WorkingMemory};
use neotrix::l4_emotion::nt_memory::distillation::MemoryCompressor;
use neotrix::l2_perception::nt_core_vector_store::{
    BruteForceVectorStore, IndexConfig, VectorRecord, VectorStore,
};

#[test]
fn test_tiered_consolidation_promote_to_long_term() {
    let config = ConsolidationConfig::default();
    let consolidator = MemoryConsolidator::new(config);

    let entries = vec![
        ConsolidationMemoryEntry {
            id: "1".to_string(),
            content: "frequently accessed".to_string(),
            access_count: 10,
            created_at_turn: 0,
            novelty: 0.8,
        },
        ConsolidationMemoryEntry {
            id: "2".to_string(),
            content: "old memory".to_string(),
            access_count: 1,
            created_at_turn: 0,
            novelty: 0.5,
        },
    ];

    let results = consolidator.consolidate(entries);
    assert_eq!(results.len(), 2);

    // First entry should be promoted to Core (high access count)
    assert_eq!(results[0].promotion_type, PromotionType::Core);

    // Second entry should be Archived (old)
    assert_eq!(results[1].promotion_type, PromotionType::Archive);
}

#[test]
fn test_working_memory_lru_eviction() {
    let mut wm = WorkingMemory::new(3);

    wm.add(MemoryRef {
        id: "a".to_string(),
        content: "item a".to_string(),
        last_recalled_at: 0,
        recall_count: 0,
    });
    wm.add(MemoryRef {
        id: "b".to_string(),
        content: "item b".to_string(),
        last_recalled_at: 0,
        recall_count: 0,
    });
    wm.add(MemoryRef {
        id: "c".to_string(),
        content: "item c".to_string(),
        last_recalled_at: 0,
        recall_count: 0,
    });

    assert_eq!(wm.get_items().len(), 3);

    wm.add(MemoryRef {
        id: "d".to_string(),
        content: "item d".to_string(),
        last_recalled_at: 0,
        recall_count: 0,
    });

    let items = wm.get_items();
    assert_eq!(items.len(), 3);
    // "a" should be evicted (LRU)
    assert!(!items.iter().any(|i| i.id == "a"));
    assert!(items.iter().any(|i| i.id == "d"));
}

#[test]
fn test_working_memory_recall() {
    let mut wm = WorkingMemory::new(10);

    wm.add(MemoryRef {
        id: "1".to_string(),
        content: "Rust programming".to_string(),
        last_recalled_at: 0,
        recall_count: 0,
    });
    wm.add(MemoryRef {
        id: "2".to_string(),
        content: "Python scripting".to_string(),
        last_recalled_at: 0,
        recall_count: 0,
    });

    let results = wm.recall("Rust");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "1");
    assert_eq!(results[0].recall_count, 1);
}

#[test]
fn test_cache_ttl_eviction() {
    let mut cache = MemoryCache::new(10, 3);

    cache.put("key1".to_string(), "value1".to_string());
    cache.put("key2".to_string(), "value2".to_string());

    assert_eq!(cache.get("key1"), Some("value1".to_string()));
    assert_eq!(cache.get("key2"), Some("value2".to_string()));

    // Advance turns to expire key1
    for _ in 0..5 {
        let _ = cache.get("key3");
    }

    assert_eq!(cache.get("key1"), None);
    assert_eq!(cache.get("key2"), Some("value2".to_string()));
}

#[test]
fn test_consolidation_config_defaults() {
    let config = ConsolidationConfig::default();
    assert_eq!(config.access_count_core, 5);
    assert_eq!(config.age_turns_archive, 100);
    assert!((config.novelty_prune - 0.2).abs() < f64::EPSILON);
}

#[test]
fn test_memory_compressor_effectiveness() {
    use neotrix::l4_emotion::nt_memory::distillation::distiller::MemoryEntry as DistillEntry;

    let compressor = MemoryCompressor::default();

    let entries: Vec<DistillEntry> = (0..10)
        .map(|i| DistillEntry {
            id: format!("mem_{}", i),
            content: format!("similar memory content item {}", i),
            embedding: vec![0.1; 128],
            access_count: i,
            created_at: 1000 + i as i64,
            tags: vec!["test".to_string()],
        })
        .collect();
    let compressed = compressor.compress(entries, 3);
    assert!(!compressed.is_empty());
}

#[test]
fn test_consolidation_prune_low_novelty() {
    let config = ConsolidationConfig::default();
    let consolidator = MemoryConsolidator::new(config);

    let entries = vec![
        ConsolidationMemoryEntry {
            id: "1".to_string(),
            content: "boring".to_string(),
            access_count: 1,
            created_at_turn: 0,
            novelty: 0.05, // below threshold
        },
    ];

    let results = consolidator.consolidate(entries);
    assert_eq!(results[0].promotion_type, PromotionType::Pruned);
}

#[test]
fn test_consolidation_retain_recent() {
    let config = ConsolidationConfig::default();
    let consolidator = MemoryConsolidator::new(config);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let entries = vec![
        ConsolidationMemoryEntry {
            id: "1".to_string(),
            content: "recent".to_string(),
            access_count: 2,
            created_at_turn: now,
            novelty: 0.5,
        },
    ];

    let results = consolidator.consolidate(entries);
    assert_eq!(results[0].promotion_type, PromotionType::Retained);
}

#[test]
fn test_cache_stats_tracking() {
    let mut cache = MemoryCache::new(10, 10);

    cache.put("a".to_string(), "va".to_string());
    let _ = cache.get("a"); // hit
    let _ = cache.get("b"); // miss

    let stats = cache.cache_stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert!(stats.hit_rate() > 0.0);
}

#[test]
fn test_vector_store_basic_operations() {
    let config = IndexConfig::default();
    let mut store = BruteForceVectorStore::new(config);

    let r1 = store.insert(VectorRecord {
        id: "vec1".to_string(),
        vector: vec![1u8, 0, 0],
        metadata: std::collections::HashMap::new(),
        timestamp: 1000,
    });
    assert!(r1.is_ok());

    let results = store.search(&[1u8, 0, 0], 1);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "vec1");
}
