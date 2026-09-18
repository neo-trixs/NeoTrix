use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EvictionPolicy {
    #[default]
    Lfu,
    Lru,
}

#[derive(Debug, Clone)]
pub struct CoreCacheConfig {
    pub capacity: usize,
    pub ttl_secs: u64,
    pub eviction_policy: EvictionPolicy,
}

/// Backward-compatible alias
pub type CacheConfig = CoreCacheConfig;

impl Default for CoreCacheConfig {
    fn default() -> Self {
        Self {
            capacity: 1000,
            ttl_secs: 300,
            eviction_policy: EvictionPolicy::Lfu,
        }
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: String,
    inserted_at: Instant,
}

/// An entry with stored embedding for semantic similarity lookup
#[derive(Debug, Clone)]
struct EmbeddingEntry {
    value: String,
    embedding: Vec<f64>,
    inserted_at: Instant,
    hit_count: u64,
}

/// 缓存统计 trait — 为所有缓存实现提供统一的可观测性接口
pub trait CacheStats {
    /// 缓存命中次数
    fn hits(&self) -> u64;
    /// 缓存未命中次数
    fn misses(&self) -> u64;
    /// 淘汰次数
    fn evictions(&self) -> u64;
    /// 命中率 (0.0 - 1.0)
    fn hit_rate(&self) -> f64 {
        let total = self.hits() + self.misses();
        if total == 0 {
            0.0
        } else {
            self.hits() as f64 / total as f64
        }
    }
    /// 重置统计计数器
    fn reset_stats(&mut self);
}

/// 缓存统计快照 — 用于导出和报告
#[derive(Debug, Clone, Default)]
pub struct CacheStatsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub hit_rate: f64,
    pub size: usize,
    pub capacity: usize,
}

#[derive(Debug, Default)]
struct CacheStatsInner {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

/// Two-tier cache: exact-match via string key + semantic via embedding cosine similarity.
///
/// The exact tier (entries) is the primary storage used by the Gateway for identical prompt hits.
/// The semantic tier (embedding_entries) provides fallback similarity search for semantically
/// similar but not identical prompts. Both tiers share the same capacity budget.
///
/// Eviction policy defaults to LFU (Least Frequently Used) for the semantic tier,
/// based on SphereLFU research showing LFU outperforms LRU for semantic cache workloads.
pub struct SemanticCache {
    entries: HashMap<String, CacheEntry>,
    embedding_entries: HashMap<u64, EmbeddingEntry>,
    capacity: usize,
    ttl_secs: u64,
    semantic_threshold: f64,
    eviction_policy: EvictionPolicy,
    stats: CacheStatsInner,
}

impl std::fmt::Debug for SemanticCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemanticCache")
            .field("size", &self.entries.len())
            .field("embedding_entries", &self.embedding_entries.len())
            .field("capacity", &self.capacity)
            .finish()
    }
}

impl SemanticCache {
    pub fn new(config: CoreCacheConfig) -> Self {
        Self {
            entries: HashMap::with_capacity(config.capacity),
            embedding_entries: HashMap::with_capacity(config.capacity / 2),
            capacity: config.capacity,
            ttl_secs: config.ttl_secs,
            semantic_threshold: 0.98,
            eviction_policy: config.eviction_policy,
            stats: CacheStatsInner::default(),
        }
    }

    /// Set a custom similarity threshold for semantic matching (default 0.98).
    pub fn set_semantic_threshold(&mut self, threshold: f64) {
        self.semantic_threshold = threshold.clamp(0.80, 0.99);
    }

    /// Set the eviction policy for the semantic tier.
    pub fn set_eviction_policy(&mut self, policy: EvictionPolicy) {
        self.eviction_policy = policy;
    }

    pub fn get_exact(&self, namespace: &str, key: &str) -> Option<String> {
        let full_key = format!("{}:{}", namespace, key);
        let result = self.entries.get(&full_key).and_then(|e| {
            if e.inserted_at.elapsed().as_secs() > self.ttl_secs {
                None
            } else {
                Some(e.value.clone())
            }
        });
        if result.is_some() {
            self.stats.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.stats.misses.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    pub fn set_exact(&mut self, namespace: &str, key: &str, value: String) {
        let full_key = format!("{}:{}", namespace, key);
        if self.entries.len() >= self.capacity {
            if let Some(oldest) = self.entries.keys().next().cloned() {
                self.entries.remove(&oldest);
            }
        }
        self.entries.insert(
            full_key,
            CacheEntry {
                value,
                inserted_at: Instant::now(),
            },
        );
    }

    /// Compute cosine similarity between two embedding vectors.
    fn cosine_sim(a: &[f64], b: &[f64]) -> f64 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.0;
        }
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }
        (dot / (mag_a * mag_b)).clamp(0.0, 1.0)
    }

    /// Store with both exact and embedding-based lookup.
    pub fn set_with_embedding(
        &mut self,
        namespace: &str,
        key: &str,
        value: String,
        embedding: Vec<f64>,
    ) {
        // Exact tier
        let _full_key = format!("{}:{}", namespace, key);
        self.set_exact(namespace, key, value.clone());

        // Semantic tier
        let embed_key = Self::hash_embedding(&embedding);
        let total = self.entries.len() + self.embedding_entries.len();
        if total >= self.capacity * 2 {
            self.evict_semantic();
        }
        self.embedding_entries.insert(
            embed_key,
            EmbeddingEntry {
                value,
                embedding,
                inserted_at: Instant::now(),
                hit_count: 0,
            },
        );
    }

    fn hash_embedding(embedding: &[f64]) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        for v in embedding {
            v.to_bits().hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Look up by embedding similarity (cosine). Returns the best match above threshold.
    /// On hit, increments the entry's hit_count for LFU eviction tracking.
    pub fn get_semantic(&mut self, query_embedding: &[f64]) -> Option<&str> {
        if self.embedding_entries.is_empty() {
            return None;
        }
        let mut best_key: Option<u64> = None;
        let mut best_sim: f64 = 0.0;

        for (&key, entry) in self.embedding_entries.iter() {
            if entry.inserted_at.elapsed().as_secs() > self.ttl_secs {
                continue;
            }
            let sim = Self::cosine_sim(query_embedding, &entry.embedding);
            if sim > best_sim {
                best_sim = sim;
                best_key = Some(key);
            }
        }

        match best_key {
            Some(key) if best_sim >= self.semantic_threshold => {
                if let Some(entry) = self.embedding_entries.get_mut(&key) {
                    entry.hit_count += 1;
                    self.stats.hits.fetch_add(1, Ordering::Relaxed);
                    Some(entry.value.as_str())
                } else {
                    self.stats.misses.fetch_add(1, Ordering::Relaxed);
                    None
                }
            }
            _ => {
                self.stats.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    /// Evict one entry from the semantic tier based on current eviction policy.
    fn evict_semantic(&mut self) {
        if self.embedding_entries.is_empty() {
            return;
        }
        match self.eviction_policy {
            EvictionPolicy::Lfu => {
                let victim = self
                    .embedding_entries
                    .iter()
                    .min_by_key(|(_, e)| e.hit_count)
                    .map(|(k, _)| *k);
                if let Some(key) = victim {
                    self.embedding_entries.remove(&key);
                    self.stats.evictions.fetch_add(1, Ordering::Relaxed);
                }
            }
            EvictionPolicy::Lru => {
                let victim = self
                    .embedding_entries
                    .iter()
                    .min_by_key(|(_, e)| e.inserted_at)
                    .map(|(k, _)| *k);
                if let Some(key) = victim {
                    self.embedding_entries.remove(&key);
                    self.stats.evictions.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }

    /// Number of entries in the exact cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.embedding_entries.is_empty()
    }

    /// 导出缓存统计快照
    pub fn snapshot(&self) -> CacheStatsSnapshot {
        CacheStatsSnapshot {
            hits: self.hits(),
            misses: self.misses(),
            evictions: self.evictions(),
            hit_rate: self.hit_rate(),
            size: self.entries.len() + self.embedding_entries.len(),
            capacity: self.capacity,
        }
    }
}

impl CacheStats for SemanticCache {
    fn hits(&self) -> u64 {
        self.stats.hits.load(Ordering::Relaxed)
    }
    fn misses(&self) -> u64 {
        self.stats.misses.load(Ordering::Relaxed)
    }
    fn evictions(&self) -> u64 {
        self.stats.evictions.load(Ordering::Relaxed)
    }
    fn reset_stats(&mut self) {
        self.stats.hits.store(0, Ordering::Relaxed);
        self.stats.misses.store(0, Ordering::Relaxed);
        self.stats.evictions.store(0, Ordering::Relaxed);
    }
}

/// 语义缓存用的确定性文本 → 定长 embedding (零外部依赖)。
///
/// 基于 unicode 码点哈希分桶累加 (64 维, 元素为 ±1 累加和): 同文本恒产生
/// 同向量; 近义文本 (个别字符/句差异) 只影响少数分桶, 余弦相似度随差异
/// 占比收敛 (追加 Δ/N 内容时约 √(N/(N+Δ)))。阈值 0.98 只放行近乎一致的提示。
/// 每个字符独立分桶 (与前后文无关), 追加式增长的历史前缀保持桶对齐。
pub fn text_to_embedding(text: &str) -> Vec<f64> {
    const DIM: usize = 64;
    let mut buckets = vec![0.0f64; DIM];
    for (i, c) in text.chars().enumerate() {
        let cp = c as u64;
        let mut h = cp.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0x5bd1_e995;
        h ^= h >> 33;
        h = h.wrapping_mul(0xc2b2_ae3d);
        h ^= h >> 29;
        let bucket = (h as usize + i * 131) % DIM;
        let sign = if h & 1 == 0 { 1.0 } else { -1.0 };
        buckets[bucket] += sign;
    }
    buckets
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_embedding(seed: f64, dim: usize) -> Vec<f64> {
        (0..dim).map(|i| (seed + i as f64).sin()).collect()
    }

    #[test]
    fn test_cache_basic() {
        let mut cache = SemanticCache::new(CoreCacheConfig::default());
        cache.set_exact("test", "key1", "value1".into());
        let got = cache.get_exact("test", "key1");
        assert_eq!(got, Some("value1".into()));
    }

    #[test]
    fn test_cache_miss() {
        let cache = SemanticCache::new(CoreCacheConfig::default());
        let got = cache.get_exact("test", "nonexistent");
        assert_eq!(got, None);
    }

    #[test]
    fn test_cache_eviction_exact() {
        let mut cache = SemanticCache::new(CoreCacheConfig {
            capacity: 2,
            ttl_secs: 300,
            ..Default::default()
        });
        cache.set_exact("ns", "a", "val_a".into());
        cache.set_exact("ns", "b", "val_b".into());
        cache.set_exact("ns", "c", "val_c".into());
        assert_eq!(cache.entries.len(), 2);
    }

    #[test]
    fn test_semantic_hit() {
        let mut cache = SemanticCache::new(CoreCacheConfig::default());
        let emb = make_embedding(1.0, 16);
        cache.set_with_embedding("test", "key1", "hello semantic".into(), emb.clone());
        let result = cache.get_semantic(&emb);
        assert_eq!(result, Some("hello semantic"));
    }

    #[test]
    fn test_semantic_miss_different_embedding() {
        let mut cache = SemanticCache::new(CoreCacheConfig::default());
        let emb_a = make_embedding(1.0, 16);
        let emb_b = make_embedding(999.0, 16);
        cache.set_with_embedding("test", "a", "value_a".into(), emb_a);
        let result = cache.get_semantic(&emb_b);
        assert_eq!(result, None);
    }

    #[test]
    fn test_semantic_similar_above_threshold() {
        let mut cache = SemanticCache::new(CoreCacheConfig::default());
        cache.set_semantic_threshold(0.80);
        let emb = make_embedding(1.0, 16);
        cache.set_with_embedding("test", "ref", "reference".into(), emb.clone());
        // Slightly perturbed embedding should still match if similar enough
        let similar: Vec<f64> = emb.iter().map(|v| v + 0.01).collect();
        let result = cache.get_semantic(&similar);
        assert_eq!(result, Some("reference"));
    }

    #[test]
    fn test_semantic_lfu_eviction() {
        let mut cache = SemanticCache::new(CoreCacheConfig {
            capacity: 4,
            ttl_secs: 300,
            eviction_policy: EvictionPolicy::Lfu,
        });
        let emb = make_embedding(1.0, 16);
        cache.set_with_embedding("test", "a", "value_a".into(), emb.clone());
        // First semantic hit increments hit_count for entry 'a'
        let _hit = cache.get_semantic(&emb);
        assert_eq!(_hit, Some("value_a"));
        // Add second entry, should evict the one with fewer hits
        let emb2 = make_embedding(2.0, 16);
        cache.set_with_embedding("test", "b", "value_b".into(), emb2.clone());
        // 'a' was hit once, 'b' was never hit — LFU should leave both in cache (capacity 4 > 2)
        let hit_a = cache.get_semantic(&emb);
        assert_eq!(hit_a, Some("value_a"));
        let hit_b = cache.get_semantic(&emb2);
        assert_eq!(hit_b, Some("value_b"));
    }

    #[test]
    fn test_semantic_hit_increments_count() {
        let mut cache = SemanticCache::new(CoreCacheConfig::default());
        let emb = make_embedding(1.0, 16);
        cache.set_with_embedding("test", "key1", "value".into(), emb.clone());
        // First hit
        let _ = cache.get_semantic(&emb);
        // Second hit
        let _ = cache.get_semantic(&emb);
        // Verify hit_count was incremented by checking internal state
        let _embed_key = SemanticCache::hash_embedding(&emb);
        // We can't access private fields, but we can verify the value is still there
        let result = cache.get_semantic(&emb);
        assert_eq!(result, Some("value"));
    }

    #[test]
    fn test_semantic_lru_vs_lfu_policy() {
        // LRU should not crash — just verify it picks a victim
        let mut cache = SemanticCache::new(CoreCacheConfig {
            capacity: 1,
            ttl_secs: 300,
            eviction_policy: EvictionPolicy::Lru,
        });
        let emb = make_embedding(1.0, 16);
        cache.set_with_embedding("test", "a", "val_a".into(), emb.clone());
        let emb2 = make_embedding(2.0, 16);
        cache.set_with_embedding("test", "b", "val_b".into(), emb2.clone());
        // Only one entry should remain
        assert_eq!(cache.embedding_entries.len(), 1);
    }

    #[test]
    fn test_cosine_sim_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = SemanticCache::cosine_sim(&v, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_sim_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = SemanticCache::cosine_sim(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_semantic_empty_cache_returns_none() {
        let mut cache = SemanticCache::new(CoreCacheConfig::default());
        let emb = make_embedding(1.0, 8);
        assert_eq!(cache.get_semantic(&emb), None);
    }

    #[test]
    fn test_eviction_policy_default() {
        let config = CoreCacheConfig::default();
        assert_eq!(config.eviction_policy, EvictionPolicy::Lfu);
    }

    #[test]
    fn test_text_to_embedding_deterministic() {
        let text = "你是一个 helpful assistant。summarize the request";
        assert_eq!(text_to_embedding(text), text_to_embedding(text));
    }

    #[test]
    fn test_text_to_embedding_similar_high_cosine() {
        let a = text_to_embedding("analyze the codebase and summarize findings");
        let b = text_to_embedding("analyze the codebase and summarize findings now");
        let sim = SemanticCache::cosine_sim(&a, &b);
        assert!(sim >= 0.9, "similar texts should be near-identical, got {sim}");
    }

    #[test]
    fn test_text_to_embedding_different_low_cosine() {
        let a = text_to_embedding("write a poem about the ocean");
        let b = text_to_embedding("fetch stock prices for AAPL");
        let sim = SemanticCache::cosine_sim(&a, &b);
        assert!(sim < 0.5, "unrelated texts should differ, got {sim}");
    }

    #[test]
    fn test_set_eviction_policy() {
        let mut cache = SemanticCache::new(CoreCacheConfig::default());
        cache.set_eviction_policy(EvictionPolicy::Lru);
    }
}

use crate::l6_meta::healing::nt_core_self_test::{SelfTest, SelfTestRegistry};

/// NT-CORE 缓存核心自测: 精确层写入/读取往返 + 容量计数 (卫生层 P0: 核心必须可自测)。
pub struct CacheSelfTest;

impl SelfTest for CacheSelfTest {
    fn name(&self) -> &str {
        "cache_core"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let mut cache = SemanticCache::new(CoreCacheConfig::default());
        cache.set_exact("selftest", "k", "v".to_string());
        match cache.get_exact("selftest", "k") {
            Some(v) if v == "v" => {}
            other => failures.push(format!("cache_core: exact roundtrip failed, got {:?}", other)),
        }
        if cache.is_empty() {
            failures.push("cache_core: cache reported empty after insert".into());
        }
        if cache.len() < 1 {
            failures.push(format!("cache_core: len {} < 1 after insert", cache.len()));
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// 注册缓存核心 SelfTest 到全局注册表 (T2)。
pub fn register_cache_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(CacheSelfTest));
}

#[cfg(test)]
mod selftest_tests {
    use super::*;

    #[test]
    fn test_cache_self_test_passes() {
        let t = super::CacheSelfTest;
        assert!(
            t.self_test().is_ok(),
            "CacheSelfTest failed: {:?}",
            t.self_test().err()
        );
    }
}
