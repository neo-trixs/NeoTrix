use crate::l2_perception::nt_world::source::types::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 搜索缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    result: SearchResult,
    created_at: Instant,
}

/// 搜索缓存 (LRU, TTL=1h)
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
        // 简单的 LRU: 如果满了，删除最旧的
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
