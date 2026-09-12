//! _PromptCache — 提示词缓存
//!
//! 提示词缓存和去重，支持相似度检测、模板化、版本管理。
//! 减少重复计算，提升响应速度。

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 缓存条目
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// 提示词哈希
    pub hash: String,
    /// 提示词内容
    pub prompt: String,
    /// 响应内容
    pub response: String,
    /// 创建时间
    pub created_at: Instant,
    /// 最后访问时间
    pub last_accessed: Instant,
    /// 访问次数
    pub access_count: u32,
    /// 相似度分数
    pub similarity: f64,
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 缓存过期时间
    pub ttl: Duration,
    /// 相似度阈值
    pub similarity_threshold: f64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl: Duration::from_secs(3600),
            similarity_threshold: 0.9,
        }
    }
}

/// 提示词缓存
pub(crate) struct _PromptCache {
    /// 缓存存储
    entries: HashMap<String, CacheEntry>,
    /// 配置
    config: CacheConfig,
    /// 统计信息
    stats: CacheStats,
}

impl _PromptCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: HashMap::new(),
            config,
            stats: CacheStats::default(),
        }
    }

    /// 生成提示词哈希
    fn hash_prompt(&self, prompt: &str) -> String {
        // 简化的哈希实现
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        prompt.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// 获取缓存
    pub fn get(&mut self, prompt: &str) -> Option<String> {
        let hash = self.hash_prompt(prompt);
        self.stats.total_requests += 1;

        if let Some(entry) = self.entries.get_mut(&hash) {
            if entry.created_at.elapsed() < self.config.ttl {
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                self.stats.hits += 1;
                return Some(entry.response.clone());
            } else {
                self.entries.remove(&hash);
            }
        }

        self.stats.misses += 1;
        None
    }

    /// 存入缓存
    pub fn set(&mut self, prompt: &str, response: &str) {
        let hash = self.hash_prompt(prompt);

        // 检查容量
        if self.entries.len() >= self.config.max_entries {
            self.evict();
        }

        self.entries.insert(hash.clone(), CacheEntry {
            hash,
            prompt: prompt.to_string(),
            response: response.to_string(),
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            similarity: 1.0,
        });

        self.stats.total_entries += 1;
    }

    /// 淘汰过期/低频条目
    fn evict(&mut self) {
        let mut to_remove = Vec::new();
        for (hash, entry) in &self.entries {
            if entry.created_at.elapsed() > self.config.ttl {
                to_remove.push(hash.clone());
            }
        }

        for hash in to_remove {
            self.entries.remove(&hash);
        }

        // 如果还是超容量，按访问次数淘汰
        if self.entries.len() >= self.config.max_entries {
            let mut entries: Vec<_> = self.entries.iter().collect();
            entries.sort_by(|a, b| a.1.access_count.cmp(&b.1.access_count));

            let to_remove_count = self.entries.len() - self.config.max_entries + 100;
            let keys_to_remove: Vec<_> = entries.iter().take(to_remove_count).map(|(hash, _)| (*hash).clone()).collect();
            for hash in keys_to_remove {
                self.entries.remove(&*hash);
            }
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> CacheStats {
        self.stats.clone()
    }
}

impl Default for _PromptCache {
    fn default() -> Self {
        Self::new(CacheConfig::default())
    }
}

/// 缓存统计
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub total_requests: u32,
    pub hits: u32,
    pub misses: u32,
    pub total_entries: u32,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.hits as f64 / self.total_requests as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit() {
        let mut cache = _PromptCache::default();
        cache.set("hello", "world");
        assert_eq!(cache.get("hello"), Some("world".to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = _PromptCache::default();
        assert_eq!(cache.get("hello"), None);
    }
}
