use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::time::Duration;

use lru::LruCache;

use super::super::web_miner::{WebMinedKnowledge, WebSourceType};
use super::*;

fn test_cache_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("neotrix_cache_test");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[test]
fn test_unified_source_type_detect_wikipedia() {
    assert_eq!(
        UnifiedKnowledgeSourceType::detect("https://en.wikipedia.org/wiki/Rust"),
        UnifiedKnowledgeSourceType::Wikipedia
    );
    assert_eq!(
        UnifiedKnowledgeSourceType::detect("https://wikidata.org/wiki/Q123"),
        UnifiedKnowledgeSourceType::Wikipedia
    );
}

#[test]
fn test_unified_source_type_detect_arxiv() {
    assert_eq!(
        UnifiedKnowledgeSourceType::detect("https://arxiv.org/abs/2303.08774"),
        UnifiedKnowledgeSourceType::ArXiv
    );
    assert_eq!(
        UnifiedKnowledgeSourceType::detect("https://semanticscholar.org/paper/123"),
        UnifiedKnowledgeSourceType::ArXiv
    );
}

#[test]
fn test_unified_source_type_detect_github() {
    assert_eq!(
        UnifiedKnowledgeSourceType::detect("https://github.com/rust-lang/rust"),
        UnifiedKnowledgeSourceType::GitHub
    );
    assert_eq!(
        UnifiedKnowledgeSourceType::detect("https://github.com/serde-rs/serde"),
        UnifiedKnowledgeSourceType::GitHub
    );
}

#[test]
fn test_unified_source_type_detect_generic() {
    assert_eq!(
        UnifiedKnowledgeSourceType::detect("https://example.com"),
        UnifiedKnowledgeSourceType::GenericWeb
    );
    assert_eq!(
        UnifiedKnowledgeSourceType::detect("https://some-other-site.org/page"),
        UnifiedKnowledgeSourceType::GenericWeb
    );
}

#[test]
fn test_unified_source_type_detect_unknown() {
    assert_eq!(
        UnifiedKnowledgeSourceType::detect(""),
        UnifiedKnowledgeSourceType::GenericWeb
    );
    assert_eq!(
        UnifiedKnowledgeSourceType::detect("not-a-url"),
        UnifiedKnowledgeSourceType::GenericWeb
    );
}

#[test]
fn test_explore_domain_names() {
    assert_eq!(ExploreDomain::Parapsychology.name(), "parapsychology");
    assert_eq!(ExploreDomain::Theology.name(), "theology");
    assert_eq!(ExploreDomain::EsotericStudies.name(), "esoteric-studies");
    assert_eq!(ExploreDomain::Wiki.name(), "wiki");
    assert_eq!(ExploreDomain::Papers.name(), "papers");
    assert_eq!(ExploreDomain::GitHub.name(), "github");
    assert_eq!(ExploreDomain::General.name(), "general");
    assert_eq!(ExploreDomain::Consciousness.name(), "consciousness");
    assert_eq!(ExploreDomain::RustML.name(), "rust-ml");
    assert_eq!(ExploreDomain::Security.name(), "nt_shield");
    assert_eq!(ExploreDomain::MathPhysics.name(), "math-physics");
}

#[test]
fn test_explore_domain_equality() {
    assert_eq!(ExploreDomain::Wiki, ExploreDomain::Wiki);
    assert_ne!(ExploreDomain::Wiki, ExploreDomain::Papers);
    assert_ne!(ExploreDomain::Parapsychology, ExploreDomain::Theology);
    assert_ne!(ExploreDomain::Consciousness, ExploreDomain::Security);
}

#[test]
fn test_seed_urls_by_domain_general_is_empty() {
    let urls = seed_urls_by_domain(ExploreDomain::General);
    assert!(urls.is_empty());
}

#[test]
fn test_seed_urls_by_domain_papers_not_empty() {
    let urls = seed_urls_by_domain(ExploreDomain::Papers);
    assert!(!urls.is_empty());
    assert!(urls.iter().all(|u| u.contains("arxiv.org")));
}

#[test]
fn test_seed_urls_by_domain_consciousness_not_empty() {
    let urls = seed_urls_by_domain(ExploreDomain::Consciousness);
    assert!(!urls.is_empty());
    assert!(urls.iter().all(|u| u.contains("wikipedia.org")));
}

// ---- Cache tests ----

#[test]
fn test_cache_ttl_github() {
    let pipeline = ExplorationPipeline::new(test_cache_dir());
    let ttl = pipeline.cache_ttl_for_url("https://github.com/rust-lang/rust");
    assert_eq!(ttl, Duration::from_secs(21600));
}

#[test]
fn test_cache_ttl_wikipedia() {
    let pipeline = ExplorationPipeline::new(test_cache_dir());
    let ttl = pipeline.cache_ttl_for_url("https://en.wikipedia.org/wiki/Rust");
    assert_eq!(ttl, Duration::from_secs(3600));
}

#[test]
fn test_cache_ttl_wikidata() {
    let pipeline = ExplorationPipeline::new(test_cache_dir());
    let ttl = pipeline.cache_ttl_for_url("https://wikidata.org/wiki/Q123");
    assert_eq!(ttl, Duration::from_secs(3600));
}

#[test]
fn test_cache_ttl_default() {
    let pipeline = ExplorationPipeline::new(test_cache_dir());
    let ttl = pipeline.cache_ttl_for_url("https://example.org/page");
    assert_eq!(ttl, Duration::from_secs(3600));
    let ttl2 = pipeline.cache_ttl_for_url("https://arxiv.org/abs/2303.08774");
    assert_eq!(ttl2, Duration::from_secs(3600));
}

#[test]
fn test_cache_put_get_hit() {
    let mut pipeline = ExplorationPipeline::new(test_cache_dir());
    let url = "https://example.com/test-cache".to_string();
    let kn = WebMinedKnowledge {
        source_url: url.clone(),
        source_name: "test".into(),
        source_type: WebSourceType::GenericUrl,
        title: "Test Page".into(),
        summary: "test summary content".into(),
        content_length: 42,
        extracted_insights: vec!["insight".into()],
        edits: vec![],
        confidence: 0.9,
    };
    pipeline.explore_cache.put(
        url.clone(),
        CachedExploreResult {
            result: kn,
            fetched_at: std::time::Instant::now(),
        },
    );

    let cached = pipeline.explore_cache.get(&url);
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().result.title, "Test Page");
}

#[test]
fn test_cache_miss_stats() {
    let pipeline = ExplorationPipeline::new(test_cache_dir());
    assert_eq!(pipeline.cache_stats().hits, 0);
    assert_eq!(pipeline.cache_stats().misses, 0);
    assert_eq!(pipeline.cache_stats().hit_rate, 0.0);
    assert_eq!(pipeline.cache_stats().size, 0);
    assert_eq!(pipeline.cache_stats().capacity, 1000);
}

#[test]
fn test_cache_stats_hit_rate() {
    let mut pipeline = ExplorationPipeline::new(test_cache_dir());
    let url = "https://example.com/hit-rate".to_string();
    let kn = WebMinedKnowledge {
        source_url: url.clone(),
        source_name: "hr".into(),
        source_type: WebSourceType::GenericUrl,
        title: "HR".into(),
        summary: "".into(),
        content_length: 0,
        extracted_insights: vec![],
        edits: vec![],
        confidence: 0.0,
    };
    pipeline.explore_cache.put(
        url.clone(),
        CachedExploreResult {
            result: kn,
            fetched_at: std::time::Instant::now(),
        },
    );

    let _ = pipeline.explore_cache.get(&url);
    pipeline.cache_hits += 1;
    pipeline.cache_misses += 3;

    let cs = pipeline.cache_stats();
    assert_eq!(cs.hits, 1);
    assert_eq!(cs.misses, 3);
    assert!((cs.hit_rate - 0.25).abs() < 1e-10);
}

#[test]
fn test_cache_expiry_behavior() {
    let mut pipeline = ExplorationPipeline::new(test_cache_dir());
    let url = "https://github.com/org/repo".to_string();
    let kn = WebMinedKnowledge {
        source_url: url.clone(),
        source_name: "gh".into(),
        source_type: WebSourceType::GitHub,
        title: "Repo".into(),
        summary: "".into(),
        content_length: 0,
        extracted_insights: vec![],
        edits: vec![],
        confidence: 0.0,
    };

    let old = std::time::Instant::now() - Duration::from_secs(21601);
    pipeline.explore_cache.put(
        url.clone(),
        CachedExploreResult {
            result: kn,
            fetched_at: old,
        },
    );

    let cached = pipeline.explore_cache.get(&url).unwrap().clone();
    let ttl = pipeline.cache_ttl_for_url(&url);
    assert!(
        cached.fetched_at.elapsed() > ttl,
        "cached entry should be expired"
    );
}

#[test]
fn test_cache_capacity_limit() {
    let mut cache: LruCache<String, usize> = LruCache::new(NonZeroUsize::new(5).expect("5"));
    for i in 0..10 {
        cache.put(format!("key-{}", i), i);
    }
    assert_eq!(cache.len(), 5);
    assert!(cache.get("key-0").is_none());
    assert!(cache.get("key-9").is_some());
}

#[test]
fn test_cache_ttl_domain_differentiation() {
    let pipeline = ExplorationPipeline::new(test_cache_dir());
    assert!(
        pipeline.cache_ttl_for_url("https://github.com/user/repo")
            > pipeline.cache_ttl_for_url("https://en.wikipedia.org/wiki/AI")
    );
}
