//! NeoTrix 能力缓存
//!
//! 基于LRU的通用能力结果缓存

use super::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 缓存值
    value: CapabilityOutput,
    /// 创建时间
    #[allow(dead_code)]
    created_at: Instant,
    /// 过期时间
    expires_at: Instant,
    /// 访问次数
    access_count: u64,
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 最大容量
    pub max_capacity: usize,
    /// 默认过期时间
    pub default_ttl: Duration,
    /// 启用统计
    pub enable_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 1000,
            default_ttl: Duration::from_secs(300), // 5分钟
            enable_stats: true,
        }
    }
}

/// 缓存统计
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// 命中次数
    pub hits: u64,
    /// 未命中次数
    pub misses: u64,
    /// 当前条目数
    pub entries: usize,
    /// 内存使用 (近似)
    pub memory_bytes: usize,
}

/// 能力缓存
pub struct CapabilityCache {
    /// 缓存存储
    entries: HashMap<String, CacheEntry>,
    /// 配置
    config: CacheConfig,
    /// 统计
    stats: CacheStats,
}

impl CapabilityCache {
    /// 创建新的缓存
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: HashMap::with_capacity(config.max_capacity),
            config,
            stats: CacheStats {
                hits: 0,
                misses: 0,
                entries: 0,
                memory_bytes: 0,
            },
        }
    }

    /// 生成缓存键
    pub fn make_key(capability_id: &str, input: &CapabilityInput) -> String {
        let input_str = format!("{:?}", input);
        format!("{}:{}", capability_id, input_str.len())
    }

    /// 获取缓存
    pub fn get(&mut self, key: &str) -> Option<CapabilityOutput> {
        if let Some(entry) = self.entries.get(key) {
            if Instant::now() < entry.expires_at {
                self.stats.hits += 1;
                return Some(entry.value.clone());
            }
        }
        self.stats.misses += 1;
        None
    }

    /// 设置缓存
    pub fn set(&mut self, key: String, value: CapabilityOutput, ttl: Option<Duration>) {
        let ttl = ttl.unwrap_or(self.config.default_ttl);

        // 检查容量
        if self.entries.len() >= self.config.max_capacity {
            self.evict_lru();
        }

        let entry = CacheEntry {
            value,
            created_at: Instant::now(),
            expires_at: Instant::now() + ttl,
            access_count: 0,
        };

        self.entries.insert(key, entry);
        self.stats.entries = self.entries.len();
    }

    /// 淘汰LRU
    fn evict_lru(&mut self) {
        if let Some(key) = self
            .entries
            .iter()
            .min_by_key(|(_, e)| e.access_count)
            .map(|(k, _)| k.clone())
        {
            self.entries.remove(&key);
        }
    }

    /// 清理过期条目
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        self.entries.retain(|_, entry| now < entry.expires_at);
        self.stats.entries = self.entries.len();
    }

    /// 获取统计
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.entries.clear();
        self.stats = CacheStats {
            hits: 0,
            misses: 0,
            entries: 0,
            memory_bytes: 0,
        };
    }

    /// 获取容量
    pub fn capacity(&self) -> usize {
        self.config.max_capacity
    }

    /// 获取当前大小
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

/// 缓存包装器
pub struct CachedCapability {
    /// 底层能力
    capability: Arc<dyn UnifiedCapability>,
    /// 缓存
    cache: std::sync::Mutex<CapabilityCache>,
}

impl CachedCapability {
    /// 创建缓存包装器
    pub fn new(capability: Arc<dyn UnifiedCapability>, config: CacheConfig) -> Self {
        Self {
            capability,
            cache: std::sync::Mutex::new(CapabilityCache::new(config)),
        }
    }

    /// 带缓存的执行
    pub fn execute_cached(
        &self,
        input: CapabilityInput,
    ) -> Result<CapabilityOutput, CapabilityError> {
        let key = CapabilityCache::make_key(&self.capability.meta().id, &input);

        // 检查缓存
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(output) = cache.get(&key) {
                return Ok(output);
            }
        }

        // 执行能力
        let output = self.capability.execute(input)?;

        // 存入缓存
        {
            let mut cache = self.cache.lock().unwrap();
            cache.set(key, output.clone(), None);
        }

        Ok(output)
    }

    /// 获取缓存统计
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.lock().unwrap().stats().clone()
    }
}

use std::sync::Arc;

impl UnifiedCapability for CachedCapability {
    fn meta(&self) -> CapabilityMeta {
        self.capability.meta()
    }

    fn health(&self) -> CapabilityHealth {
        self.capability.health()
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        self.execute_cached(input)
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        self.capability.supports(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_creation() {
        let cache = CapabilityCache::new(CacheConfig::default());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn cache_set_get() {
        let mut cache = CapabilityCache::new(CacheConfig::default());
        let key = "test".to_string();
        let value = CapabilityOutput::Text("hello".into());

        cache.set(key.clone(), value, None);

        let result = cache.get(&key);
        assert!(result.is_some());
    }

    #[test]
    fn cache_eviction() {
        let config = CacheConfig {
            max_capacity: 2,
            default_ttl: Duration::from_secs(300),
            enable_stats: true,
        };
        let mut cache = CapabilityCache::new(config);

        cache.set("a".into(), CapabilityOutput::Text("a".into()), None);
        cache.set("b".into(), CapabilityOutput::Text("b".into()), None);
        cache.set("c".into(), CapabilityOutput::Text("c".into()), None);

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn cached_capability() {
        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        let cached = CachedCapability::new(cap, CacheConfig::default());

        let input = CapabilityInput::Nlp(NlpInput {
            task: NlpTask::Tokenize,
            text: "测试".into(),
            language: None,
        });

        let result1 = cached.execute_cached(input.clone()).unwrap();
        let result2 = cached.execute_cached(input).unwrap();

        // 第二次应该从缓存返回
        let stats = cached.cache_stats();
        assert!(stats.hits > 0);
    }
}
