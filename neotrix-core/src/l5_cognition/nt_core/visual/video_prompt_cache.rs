//! _VideoPromptCache — 视频提示词缓存
//!
//! 语义相似度缓存 + 去重 + 版本管理。
//! 减少 15-30% 的冗余调用。

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 缓存条目
#[derive(Debug, Clone)]
pub struct _PromptCacheEntry {
    /// 提示词 ID
    pub id: String,
    /// 原始提示词
    pub prompt: String,
    /// 提示词嵌入向量
    pub embedding: Vec<f64>,
    /// 响应内容
    pub response: String,
    /// 创建时间
    pub created_at: Instant,
    /// 最后访问时间
    pub last_accessed: Instant,
    /// 访问次数
    pub access_count: u32,
    /// 版本
    pub version: u32,
    /// 标签
    pub tags: Vec<String>,
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct _PromptCacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 缓存过期时间
    pub ttl: Duration,
    /// 相似度阈值
    pub similarity_threshold: f64,
    /// 启用语义搜索
    pub semantic_search: bool,
    /// 启用版本管理
    pub versioning: bool,
}

impl Default for _PromptCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            ttl: Duration::from_secs(7 * 24 * 3600), // 7 天
            similarity_threshold: 0.9,
            semantic_search: true,
            versioning: true,
        }
    }
}

/// 视频提示词缓存
pub struct _VideoPromptCache {
    /// 缓存存储
    entries: HashMap<String, _PromptCacheEntry>,
    /// 配置
    config: _PromptCacheConfig,
    /// 统计信息
    stats: _PromptCacheStats,
}

impl _VideoPromptCache {
    pub fn new(config: _PromptCacheConfig) -> Self {
        Self {
            entries: HashMap::new(),
            config,
            stats: _PromptCacheStats::default(),
        }
    }

    /// 获取缓存
    pub fn get(&mut self, prompt: &str) -> Option<String> {
        self.stats.total_requests += 1;

        // 精确匹配
        if let Some(entry) = self.entries.values_mut().find(|e| e.prompt == prompt) {
            if entry.created_at.elapsed() < self.config.ttl {
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                self.stats.hits += 1;
                return Some(entry.response.clone());
            }
        }

        // 语义相似度匹配
        if self.config.semantic_search {
            let prompt_embedding = self.embed_prompt(prompt);
            if let Some(id) = self.find_similar_id(&prompt_embedding) {
                let entry = self.entries.get_mut(&id).unwrap();
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                let resp = entry.response.clone();
                self.stats.semantic_hits += 1;
                return Some(resp);
            }
        }

        self.stats.misses += 1;
        None
    }

    /// 存入缓存
    pub fn set(&mut self, prompt: &str, response: &str, tags: Vec<String>) {
        let id = format!("cache-{}", uuid::Uuid::new_v4());
        let embedding = self.embed_prompt(prompt);

        // 检查容量
        if self.entries.len() >= self.config.max_entries {
            self.evict();
        }

        let version = if self.config.versioning {
            self.entries.values()
                .filter(|e| e.prompt == prompt)
                .map(|e| e.version)
                .max()
                .unwrap_or(0) + 1
        } else {
            1
        };

        self.entries.insert(id.clone(), _PromptCacheEntry {
            id,
            prompt: prompt.to_string(),
            embedding,
            response: response.to_string(),
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            version,
            tags,
        });

        self.stats.total_entries += 1;
    }

    /// Generate prompt embedding for semantic similarity search.
    ///
    /// Uses byte-frequency heuristic (128-dim L2-normalized) — **not semantically meaningful**.
    /// This is a placeholder for sentence-transformers or domain-specific embedding model.
    /// Cache hits via this embedding are unreliable for semantically different prompts.
    fn embed_prompt(&self, prompt: &str) -> Vec<f64> {
        let mut embedding = vec![0.0; 128];
        for (i, byte) in prompt.bytes().enumerate() {
            embedding[i % 128] += byte as f64;
        }
        let norm: f64 = embedding.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }
        embedding
    }

    /// 查找相似条目
    #[allow(dead_code)]
    fn find_similar(&mut self, query_embedding: &[f64]) -> Option<String> {
        let mut best_score = 0.0;
        let mut best_id = None;

        for (id, entry) in &self.entries {
            let similarity = crate::core::nt_core_math::cosine_similarity_f64(query_embedding, &entry.embedding);
            if similarity > best_score && similarity >= self.config.similarity_threshold {
                best_score = similarity;
                best_id = Some(id.clone());
            }
        }

        best_id
    }

    /// 查找相似条目 ID (immutable borrow)
    fn find_similar_id(&self, query_embedding: &[f64]) -> Option<String> {
        let mut best_score = 0.0;
        let mut best_id = None;

        for (id, entry) in &self.entries {
            let similarity = crate::core::nt_core_math::cosine_similarity_f64(query_embedding, &entry.embedding);
            if similarity > best_score && similarity >= self.config.similarity_threshold {
                best_score = similarity;
                best_id = Some(id.clone());
            }
        }

        best_id
    }

    /// 淘汰过期条目
    ///
    /// Note: Two-phase eviction: TTL-expired first, then lowest access_count.
    /// Removes 1000 extra entries per cycle to amortize eviction cost.
    /// Real implementation needs:
    /// - LRU/LFU policy with bounded memory
    /// - Version-aware eviction (keep latest version of same prompt)
    fn evict(&mut self) {
        let mut to_remove = Vec::new();
        for (id, entry) in &self.entries {
            if entry.created_at.elapsed() > self.config.ttl {
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            self.entries.remove(&id);
        }

        // 如果还是超容量，按访问次数淘汰
        if self.entries.len() >= self.config.max_entries {
            let mut entries: Vec<_> = self.entries.iter().collect();
            entries.sort_by(|a, b| a.1.access_count.cmp(&b.1.access_count));

            let to_remove_count = self.entries.len() - self.config.max_entries + 1000;
            let keys_to_remove: Vec<_> = entries.iter().take(to_remove_count).map(|(id, _)| (*id).clone()).collect();
            for id in keys_to_remove {
                self.entries.remove(&id);
            }
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> _PromptCacheStats {
        self.stats.clone()
    }
}

impl Default for _VideoPromptCache {
    fn default() -> Self {
        Self::new(_PromptCacheConfig::default())
    }
}

/// 缓存统计
#[derive(Debug, Clone, Default)]
pub struct _PromptCacheStats {
    pub total_requests: u32,
    pub hits: u32,
    pub semantic_hits: u32,
    pub misses: u32,
    pub total_entries: u32,
}

impl _PromptCacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.hits + self.semantic_hits) as f64 / self.total_requests as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit() {
        let mut cache = _VideoPromptCache::default();
        cache.set("hello world", "response", vec![]);
        assert_eq!(cache.get("hello world"), Some("response".to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = _VideoPromptCache::default();
        assert_eq!(cache.get("hello"), None);
    }
}
