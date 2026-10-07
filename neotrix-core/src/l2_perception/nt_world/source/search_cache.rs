use crate::l2_perception::nt_world::source::types::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 搜索缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    result: SearchResult,
    created_at: Instant,
}

/// 搜索缓存 (**FIFO + TTL=1h**，⛔ 不是 LRU)
///
/// 2026-10-07 修正**头注释说谎**的缺陷：原文写「(LRU, TTL=1h)」，但：
///   - `cache` 是 `HashMap`，**没有 recency 字段**（`:14`）
///   - 淘汰按 `min_by_key(|(_, e)| e.created_at)`（`:48-50`），
///     而 `created_at` **只在 `set()` 写入**（`:57`），`get()` **命中时不刷新**（`:34-38`）
/// ⇒ 淘汰依据是**插入时刻**，故实际语义是 **FIFO**，不是 LRU。
///
/// ⛔ 本类型**全仓零外部消费者**（`mod.rs:235` 仅 `pub use` 再导出），
/// 而同目录 `multi_cache.rs` 用的是**真 LRU**（`LruCache` + TTL）。
/// ⇒ 见同目录 `//孤儿接线` 约定：接线前不动结构，只让注释与实现一致。
pub struct SearchCache {
    cache: HashMap<String, CacheEntry>,
    max_size: usize,
    ttl: Duration,
    hits: u64,
    misses: u64,
}

impl SearchCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            ttl: Duration::from_secs(3600), // 1 hour
            hits: 0,
            misses: 0,
        }
    }

    /// 获取缓存
    pub fn get(&mut self, key: &str) -> Option<SearchResult> {
        if let Some(entry) = self.cache.get(key) {
            if entry.created_at.elapsed() < self.ttl {
                self.hits += 1;
                return Some(entry.result.clone());
            }
        }
        self.misses += 1;
        None
    }

    /// 设置缓存
    pub fn set(&mut self, key: String, result: SearchResult) {
        // FIFO 淘汰（⛔ 非 LRU）：按 `created_at` 最小者出局。
        // `created_at` 只在此处写入、`get()` 命中时不刷新 ⇒ 只能反映插入序。
        // 真 LRU 见同目录 `multi_cache.rs`（`LruCache` + TTL）。
        if self.cache.len() >= self.max_size {
            if let Some(oldest_key) = self.cache.iter()
                .min_by_key(|(_, entry)| entry.created_at)
                .map(|(k, _)| k.clone())
            {
                self.cache.remove(&oldest_key);
            }
        }
        self.cache.insert(key, CacheEntry {
            result,
            created_at: Instant::now(),
        });
    }

    /// 获取缓存命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// 获取缓存统计
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.len(),
            max_size: self.max_size,
            hits: self.hits,
            misses: self.misses,
            hit_rate: self.hit_rate(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}
