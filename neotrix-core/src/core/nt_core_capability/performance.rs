//! 能力性能优化
//!
//! 提供能力调用的性能优化和资源管理

use crate::core::nt_core_capability::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 连接池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// 最小连接数
    pub min_connections: usize,
    /// 最大连接数
    pub max_connections: usize,
    /// 连接超时
    pub connection_timeout: Duration,
    /// 空闲超时
    pub idle_timeout: Duration,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔
    pub retry_interval: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 5,
            max_connections: 20,
            connection_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(300),
            max_retries: 3,
            retry_interval: Duration::from_secs(1),
        }
    }
}

/// 连接池
pub struct ConnectionPool<T> {
    /// 空闲连接
    idle: Vec<T>,
    /// 活跃连接
    active: Vec<T>,
    /// 配置
    config: PoolConfig,
    /// 创建函数
    factory: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T: Clone + PartialEq> ConnectionPool<T> {
    /// 创建新的连接池
    pub fn new(config: PoolConfig, factory: Box<dyn Fn() -> T + Send + Sync>) -> Self {
        Self {
            idle: Vec::new(),
            active: Vec::new(),
            config,
            factory,
        }
    }

    /// 获取连接
    pub fn get(&mut self) -> Option<T> {
        // 尝试从空闲池获取
        if let Some(conn) = self.idle.pop() {
            self.active.push(conn.clone());
            return Some(conn);
        }

        // 如果未达到最大连接数，创建新连接
        if self.active.len() < self.config.max_connections {
            let conn = (self.factory)();
            self.active.push(conn.clone());
            return Some(conn);
        }

        None
    }

    /// 归还连接
    pub fn put(&mut self, conn: T) {
        if let Some(pos) = self.active.iter().position(|c| c == &conn) {
            self.active.remove(pos);
        }
        self.idle.push(conn);
    }

    /// 清理空闲连接
    pub fn cleanup_idle(&mut self, max_idle: usize) {
        if self.idle.len() > max_idle {
            self.idle.drain(max_idle..);
        }
    }

    /// 获取状态
    pub fn status(&self) -> PoolStatus {
        PoolStatus {
            idle_count: self.idle.len(),
            active_count: self.active.len(),
            total_count: self.idle.len() + self.active.len(),
        }
    }
}

/// 连接池状态
#[derive(Debug, Clone)]
pub struct PoolStatus {
    pub idle_count: usize,
    pub active_count: usize,
    pub total_count: usize,
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 最大缓存大小
    pub max_size: usize,
    /// 缓存过期时间
    pub ttl: Duration,
    /// 启用LRU
    pub enable_lru: bool,
    /// 预热缓存
    pub warmup: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            ttl: Duration::from_secs(300),
            enable_lru: true,
            warmup: false,
        }
    }
}

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
}

/// LRU缓存
pub struct LruCache<K, V> {
    entries: std::collections::HashMap<K, CacheEntry<V>>,
    config: CacheConfig,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> LruCache<K, V> {
    /// 创建新的LRU缓存
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: std::collections::HashMap::with_capacity(config.max_size),
            config,
        }
    }

    /// 获取缓存
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.get_mut(key) {
            // 检查是否过期
            if entry.created_at.elapsed() > self.config.ttl {
                self.entries.remove(key);
                return None;
            }

            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            return Some(entry.value.clone());
        }
        None
    }

    /// 设置缓存
    pub fn set(&mut self, key: K, value: V) {
        // 如果缓存已满，淘汰最旧的
        if self.entries.len() >= self.config.max_size && self.config.enable_lru {
            self.evict_lru();
        }

        self.entries.insert(
            key,
            CacheEntry {
                value,
                created_at: Instant::now(),
                last_accessed: Instant::now(),
                access_count: 0,
            },
        );
    }

    /// 淘汰LRU条目
    fn evict_lru(&mut self) {
        if let Some(key) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(key, _)| key.clone())
        {
            self.entries.remove(&key);
        }
    }

    /// 清理过期条目
    pub fn cleanup(&mut self) {
        self.entries
            .retain(|_, entry| entry.created_at.elapsed() <= self.config.ttl);
    }

    /// 获取缓存统计
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.entries.len(),
            max_size: self.config.max_size,
            hit_rate: 0.0, // 需要外部统计
        }
    }
}

/// 缓存统计
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hit_rate: f64,
}

/// 预热器
pub struct Warmer {
    /// 预热任务
    tasks: Vec<Box<dyn Fn() + Send + Sync>>,
    /// 是否已预热
    warmed: bool,
}

impl Warmer {
    /// 创建新的预热器
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            warmed: false,
        }
    }

    /// 添加预热任务
    pub fn add_task(&mut self, task: Box<dyn Fn() + Send + Sync>) {
        self.tasks.push(task);
    }

    /// 执行预热
    pub fn warmup(&mut self) {
        if !self.warmed {
            for task in &self.tasks {
                task();
            }
            self.warmed = true;
        }
    }

    /// 重置预热状态
    pub fn reset(&mut self) {
        self.warmed = false;
    }
}

/// 性能优化器
pub struct PerformanceOptimizer {
    /// 连接池配置
    #[allow(dead_code)]
    pool_config: PoolConfig,
    /// 缓存配置
    #[allow(dead_code)]
    cache_config: CacheConfig,
    /// 预热器
    warmer: Warmer,
    /// 性能统计
    stats: PerformanceStats,
}

/// 性能统计
#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    pub total_requests: u64,
    pub cached_requests: u64,
    pub pool_hits: u64,
    pub pool_misses: u64,
    pub avg_response_time_ms: f64,
    pub memory_usage_mb: f64,
}

impl PerformanceOptimizer {
    /// 创建新的性能优化器
    pub fn new(pool_config: PoolConfig, cache_config: CacheConfig) -> Self {
        Self {
            pool_config,
            cache_config,
            warmer: Warmer::new(),
            stats: PerformanceStats::default(),
        }
    }

    /// 添加预热任务
    pub fn add_warmup_task(&mut self, task: Box<dyn Fn() + Send + Sync>) {
        self.warmer.add_task(task);
    }

    /// 执行预热
    pub fn warmup(&mut self) {
        self.warmer.warmup();
    }

    /// 记录请求
    pub fn record_request(&mut self, cached: bool, response_time_ms: u64) {
        self.stats.total_requests += 1;
        if cached {
            self.stats.cached_requests += 1;
        }

        // 更新平均响应时间
        let total_time = self.stats.avg_response_time_ms * (self.stats.total_requests - 1) as f64
            + response_time_ms as f64;
        self.stats.avg_response_time_ms = total_time / self.stats.total_requests as f64;
    }

    /// 记录连接池使用
    pub fn record_pool_usage(&mut self, hit: bool) {
        if hit {
            self.stats.pool_hits += 1;
        } else {
            self.stats.pool_misses += 1;
        }
    }

    /// 获取性能统计
    pub fn stats(&self) -> &PerformanceStats {
        &self.stats
    }

    /// 获取缓存命中率
    pub fn cache_hit_rate(&self) -> f64 {
        if self.stats.total_requests == 0 {
            0.0
        } else {
            self.stats.cached_requests as f64 / self.stats.total_requests as f64
        }
    }

    /// 获取连接池命中率
    pub fn pool_hit_rate(&self) -> f64 {
        let total = self.stats.pool_hits + self.stats.pool_misses;
        if total == 0 {
            0.0
        } else {
            self.stats.pool_hits as f64 / total as f64
        }
    }

    /// 重置统计
    pub fn reset_stats(&mut self) {
        self.stats = PerformanceStats::default();
    }
}

/// 优化包装器
pub struct OptimizedCapability {
    /// 底层能力
    capability: Arc<dyn UnifiedCapability>,
    /// 性能优化器
    optimizer: Arc<std::sync::Mutex<PerformanceOptimizer>>,
}

impl OptimizedCapability {
    /// 创建优化包装器
    pub fn new(
        capability: Arc<dyn UnifiedCapability>,
        optimizer: Arc<std::sync::Mutex<PerformanceOptimizer>>,
    ) -> Self {
        Self {
            capability,
            optimizer,
        }
    }

    /// 带优化的执行
    pub fn execute_optimized(
        &self,
        input: CapabilityInput,
    ) -> Result<CapabilityOutput, CapabilityError> {
        let start = Instant::now();
        let result = self.capability.execute(input);
        let duration = start.elapsed().as_millis() as u64;

        // 记录性能
        let mut optimizer = self.optimizer.lock().unwrap();
        optimizer.record_request(false, duration);

        result
    }
}

impl UnifiedCapability for OptimizedCapability {
    fn meta(&self) -> CapabilityMeta {
        self.capability.meta()
    }

    fn health(&self) -> CapabilityHealth {
        self.capability.health()
    }

    fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError> {
        self.execute_optimized(input)
    }

    fn supports(&self, input: &CapabilityInput) -> bool {
        self.capability.supports(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lru_cache() {
        let config = CacheConfig {
            max_size: 3,
            ttl: Duration::from_secs(60),
            enable_lru: true,
            warmup: false,
        };
        let mut cache = LruCache::new(config);

        cache.set("a", 1);
        cache.set("b", 2);
        cache.set("c", 3);

        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"b"), Some(2));
        assert_eq!(cache.get(&"c"), Some(3));

        // 添加第四个，应该淘汰最旧的
        cache.set("d", 4);
        assert_eq!(cache.get(&"a"), None);
    }

    #[test]
    fn performance_optimizer() {
        let pool_config = PoolConfig::default();
        let cache_config = CacheConfig::default();
        let mut optimizer = PerformanceOptimizer::new(pool_config, cache_config);

        optimizer.record_request(true, 100);
        optimizer.record_request(false, 200);

        assert_eq!(optimizer.stats().total_requests, 2);
        assert_eq!(optimizer.cache_hit_rate(), 0.5);
    }
}
