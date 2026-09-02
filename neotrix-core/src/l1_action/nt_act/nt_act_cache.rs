//! Unified Cache Layer — LRU/TTL 缓存
//!
//! 吸收 KB 经验:
//! - LRU (Least Recently Used) 淘汰策略
//! - TTL (Time-To-Live) 过期机制
//! - 分层缓存 (L1 memory / L2 disk)
//! - 缓存预热
//! - 缓存穿透保护

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// 统一缓存层
pub struct CacheLayer {
    l1_cache: LRUCache<String, CacheEntry>,
    l2_cache: Option<DiskCache>,
    stats: CacheStats,
    config: CacheConfig,
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub l1_max_entries: usize,
    pub l1_ttl: Duration,
    pub l2_enabled: bool,
    pub l2_path: Option<String>,
    pub l2_max_size_mb: usize,
    pub penetration_protection: bool,
    pub warmup_enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_max_entries: 1000,
            l1_ttl: Duration::from_secs(300), // 5 minutes
            l2_enabled: false,
            l2_path: None,
            l2_max_size_mb: 100,
            penetration_protection: true,
            warmup_enabled: false,
        }
    }
}

/// 缓存条目
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub ttl: Option<Duration>,
    pub tags: Vec<String>,
}

/// LRU 缓存
pub struct LRUCache<K, V> {
    entries: HashMap<K, V>,
    access_order: VecDeque<K>,
    max_entries: usize,
}

/// 磁盘缓存 (L2)
pub struct DiskCache {
    path: String,
    max_size_mb: usize,
    current_size_mb: usize,
}

/// 缓存统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: usize,
    pub hit_rate: f64,
}

/// 缓存查询结果
#[derive(Debug, Clone)]
pub enum CacheResult {
    Hit(CacheEntry),
    Miss,
    Expired,
    PenetrationProtected,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> LRUCache<K, V> {
    /// 创建新的 LRU 缓存
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            access_order: VecDeque::new(),
            max_entries,
        }
    }

    /// 获取缓存
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(value) = self.entries.get(key) {
            // 更新访问顺序
            self.access_order.retain(|k| k != key);
            self.access_order.push_back(key.clone());
            Some(value.clone())
        } else {
            None
        }
    }

    /// 插入缓存
    pub fn insert(&mut self, key: K, value: V) {
        if self.entries.len() >= self.max_entries {
            // 淘汰最久未使用的
            if let Some(oldest) = self.access_order.pop_front() {
                self.entries.remove(&oldest);
            }
        }

        self.entries.insert(key.clone(), value);
        self.access_order.push_back(key);
    }

    /// 移除缓存
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.access_order.retain(|k| k != key);
        self.entries.remove(key)
    }

    /// 获取缓存大小
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
    }
}

impl CacheLayer {
    /// 创建新的缓存层
    pub fn new(config: CacheConfig) -> Self {
        let l2_cache = if config.l2_enabled {
            config.l2_path.as_ref().map(|path| DiskCache {
                path: path.clone(),
                max_size_mb: config.l2_max_size_mb,
                current_size_mb: 0,
            })
        } else {
            None
        };

        Self {
            l1_cache: LRUCache::new(config.l1_max_entries),
            l2_cache,
            stats: CacheStats {
                hits: 0,
                misses: 0,
                evictions: 0,
                size: 0,
                hit_rate: 0.0,
            },
            config,
        }
    }

    /// 获取缓存
    pub fn get(&mut self, key: &str) -> CacheResult {
        // L1 查找
        if let Some(entry) = self.l1_cache.get(&key.to_string()) {
            // 检查 TTL
            if let Some(ttl) = entry.ttl {
                if entry.created_at.elapsed() > ttl {
                    self.l1_cache.remove(&key.to_string());
                    self.stats.misses += 1;
                    return CacheResult::Expired;
                }
            }

            self.stats.hits += 1;
            self.update_hit_rate();
            return CacheResult::Hit(entry);
        }

        // L2 查找
        if let Some(ref mut disk_cache) = self.l2_cache {
            // TODO: 实际从磁盘读取
        }

        // 穿透保护
        if self.config.penetration_protection {
            // TODO: 实现布隆过滤器
        }

        self.stats.misses += 1;
        self.update_hit_rate();
        CacheResult::Miss
    }

    /// 插入缓存
    pub fn insert(&mut self, key: String, value: serde_json::Value, ttl: Option<Duration>, tags: Vec<String>) {
        let entry = CacheEntry {
            key: key.clone(),
            value,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            ttl,
            tags,
        };

        self.l1_cache.insert(key, entry);
        self.stats.size = self.l1_cache.len();
    }

    /// 按标签失效
    pub fn invalidate_by_tag(&mut self, tag: &str) {
        let keys_to_remove: Vec<String> = self.l1_cache.entries.iter()
            .filter(|(_, entry)| entry.tags.contains(&tag.to_string()))
            .map(|(key, _)| key.clone())
            .collect();

        for key in keys_to_remove {
            self.l1_cache.remove(&key);
            self.stats.evictions += 1;
        }
    }

    /// 清理过期条目
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        let keys_to_remove: Vec<String> = self.l1_cache.entries.iter()
            .filter(|(_, entry)| {
                if let Some(ttl) = entry.ttl {
                    now.duration_since(entry.created_at) > ttl
                } else {
                    false
                }
            })
            .map(|(key, _)| key.clone())
            .collect();

        for key in keys_to_remove {
            self.l1_cache.remove(&key);
            self.stats.evictions += 1;
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> &CacheStats {
        &self.stats
    }

    /// 更新命中率
    fn update_hit_rate(&mut self) {
        let total = self.stats.hits + self.stats.misses;
        if total > 0 {
            self.stats.hit_rate = self.stats.hits as f64 / total as f64;
        }
    }

    /// 预热缓存
    pub fn warm_up(&mut self, entries: Vec<(String, serde_json::Value)>) {
        for (key, value) in entries {
            self.insert(key, value, None, vec![]);
        }
    }
}
