//! ⛔ **孤儿接线**：本文件仅被同目录 `cache_warmer.rs` 消费，而 `cache_warmer`
//! 仅被 `source/mod.rs:248` 的 `pub use` 再导出 ⇒ **全仓零外部消费者**
//! （已实测：`MultiLevelCache` 在本目录外零命中）。
//!
//! 本文件用**真 LRU**（`lru::LruCache` + TTL）；同目录 `search_cache.rs` 其实是
//! **FIFO**（其原头自称 LRU，2026-10-07 已修正）。
//! ⇒ 若接线，应以**本文件为真源**，`search_cache.rs` 另行裁决。
use crate::l2_perception::nt_world::source::types::*;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

const DEFAULT_L1_CAPACITY: usize = 1000;
const DEFAULT_TTL_SECS: u64 = 300;

#[derive(Debug, Clone)]
struct CacheEntry {
    result: SearchResult,
    created_at: Instant,
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub l1_size: usize,
    pub l1_capacity: usize,
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub hit_rate: f64,
}

pub struct MultiLevelCache {
    l1: LruCache<String, CacheEntry>,
    ttl: Duration,
    hit_count: u64,
    miss_count: u64,
}

impl MultiLevelCache {
    pub fn new() -> Self {
        Self {
            l1: LruCache::new(NonZeroUsize::new(DEFAULT_L1_CAPACITY).expect("non-zero")),
            ttl: Duration::from_secs(DEFAULT_TTL_SECS),
            hit_count: 0,
            miss_count: 0,
        }
    }

    pub fn with_capacity(capacity: usize, ttl_secs: u64) -> Self {
        Self {
            l1: LruCache::new(NonZeroUsize::new(capacity.max(1)).unwrap()),
            ttl: Duration::from_secs(ttl_secs),
            hit_count: 0,
            miss_count: 0,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<SearchResult> {
        if let Some(entry) = self.l1.get(key) {
            if entry.created_at.elapsed() < self.ttl {
                self.hit_count += 1;
                return Some(entry.result.clone());
            }
        }
        self.miss_count += 1;
        None
    }

    pub fn set(&mut self, key: String, result: SearchResult) {
        self.l1.put(
            key,
            CacheEntry {
                result,
                created_at: Instant::now(),
            },
        );
    }

    pub fn remove(&mut self, key: &str) -> Option<SearchResult> {
        self.l1.pop(key).map(|e| e.result)
    }

    pub fn clear(&mut self) {
        self.l1.clear();
    }

    pub fn contains(&self, key: &str) -> bool {
        self.l1.contains(key)
    }

    pub fn stats(&self) -> CacheStats {
        let total = self.hit_count + self.miss_count;
        let hit_rate = if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        };
        CacheStats {
            l1_size: self.l1.len(),
            l1_capacity: self.l1.cap().get(),
            l1_hits: self.hit_count,
            l1_misses: self.miss_count,
            hit_rate,
        }
    }
}

impl Default for MultiLevelCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_result(query: &str) -> SearchResult {
        SearchResult {
            data: vec![],
            total: 0,
            source: query.to_string(),
            page: 1,
        }
    }

    #[test]
    fn test_set_and_get() {
        let mut cache = MultiLevelCache::with_capacity(10, 300);
        cache.set("test".into(), sample_result("test"));
        assert!(cache.get("test").is_some());
    }

    #[test]
    fn test_miss() {
        let mut cache = MultiLevelCache::new();
        assert!(cache.get("nonexistent").is_none());
        assert_eq!(cache.stats().l1_misses, 1);
    }

    #[test]
    fn test_lru_eviction() {
        // ⛔ 2026-10-08 补强：原序列是 `set a, set b, set c` 后断言 `a` 被淘汰 ——
        // 该序列在 **LRU 与 FIFO 下输出完全相同**（`a` 都是最老的那个），
        // ⇒ 旧断言**结构上不可能失败**，它证明不了本类型是 LRU 而非 FIFO。
        // 仓外探针实测（逐字复刻两份策略）：
        //   原序列      LRU: a=false b=true c=true | FIFO: 同 ⇒ 区分不了
        //   加 get(a) 后 LRU: a=true  b=false c=true | FIFO: a=false b=true c=true ⇒ 分得开
        // 真正的判别式是「**读**是否提升 recency」，故必须先 get 再 set。
        let mut cache = MultiLevelCache::with_capacity(2, 300);
        cache.set("a".into(), sample_result("a"));
        cache.set("b".into(), sample_result("b"));
        // 关键一步：读 a 把它从 LRU 端提升为 MRU（`lru::LruCache::get` 的语义）
        assert!(cache.get("a").is_some(), "前置：a 必须先能读到");
        cache.set("c".into(), sample_result("c"));
        // LRU 下被淘汰的是**最久未用**的 b；FIFO 下则仍是 a —— 这一条断言才 discriminating
        assert!(cache.get("a").is_some(), "a 刚被读过 ⇒ LRU 下不该被淘汰");
        assert!(cache.get("b").is_none(), "b 是最久未用 ⇒ 应被淘汰（若此条红，说明实现是 FIFO 而非 LRU）");
        assert!(cache.get("c").is_some());
    }

    #[test]
    fn test_ttl_expiry() {
        let mut cache = MultiLevelCache::with_capacity(10, 0);
        cache.set("expired".into(), sample_result("expired"));
        assert!(cache.get("expired").is_none());
    }

    #[test]
    fn test_stats() {
        let mut cache = MultiLevelCache::with_capacity(10, 300);
        cache.set("q".into(), sample_result("q"));
        cache.get("q");
        cache.get("miss");
        let stats = cache.stats();
        assert_eq!(stats.l1_hits, 1);
        assert_eq!(stats.l1_misses, 1);
        assert!((stats.hit_rate - 0.5).abs() < f64::EPSILON);
    }
}
