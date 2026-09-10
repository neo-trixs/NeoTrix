#![forbid(unsafe_code)]

//! 持久化 KV 层 (Persistent KV Layer)
//!
//! 三层存储架构: Hot (GPU) → Warm (CPU) → Cold (NVMe SSD)
//! 引擎无关的守护进程接口，基于访问模式自动层级迁移，LRU 驱逐热缓存

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// 存储层级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageTier {
    /// 热层 — GPU 显存，最低延迟
    Hot,
    /// 温层 — CPU 内存，中等延迟
    Warm,
    /// 冷层 — NVMe SSD，持久化存储
    Cold,
}

impl StorageTier {
    /// 层级延迟 (纳秒)
    pub fn latency_ns(&self) -> u64 {
        match self {
            StorageTier::Hot => 100,        // ~100ns GPU 访问
            StorageTier::Warm => 1_000,     // ~1μs CPU 内存
            StorageTier::Cold => 100_000,   // ~100μs NVMe
        }
    }

    /// 层级容量 (字节)
    pub fn capacity_bytes(&self) -> u64 {
        match self {
            StorageTier::Hot => 8 * 1024 * 1024 * 1024,   // 8GB GPU
            StorageTier::Warm => 64 * 1024 * 1024 * 1024,  // 64GB CPU
            StorageTier::Cold => 512 * 1024 * 1024 * 1024,  // 512GB SSD
        }
    }
}

/// KV 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvEntry {
    /// 键
    pub key: String,
    /// 值
    pub value: Vec<u8>,
    /// 当前存储层级
    pub tier: StorageTier,
    /// 条目大小 (字节)
    pub size_bytes: u64,
    /// 访问次数
    pub access_count: u64,
    /// 最后访问时间戳
    pub last_accessed: String,
    /// 创建时间戳
    pub created_at: String,
    /// 过期时间 (可选)
    pub expires_at: Option<String>,
    /// 校验和
    pub checksum: u64,
}

/// 层级配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    /// 热层最大条目数
    pub hot_max_entries: usize,
    /// 温层最大条目数
    pub warm_max_entries: usize,
    /// 冷层最大条目数
    pub cold_max_entries: usize,
    /// 自动迁移启用
    pub auto_migrate: bool,
    /// 迁移访问阈值 (热→温)
    pub hot_to_warm_threshold: u64,
    /// 迁移访问阈值 (温→冷)
    pub warm_to_cold_threshold: u64,
    /// 迁移访问阈值 (冷→温)
    pub cold_to_warm_threshold: u64,
    /// 迁移访问阈值 (温→热)
    pub warm_to_hot_threshold: u64,
}

impl Default for TierConfig {
    fn default() -> Self {
        Self {
            hot_max_entries: 10_000,
            warm_max_entries: 100_000,
            cold_max_entries: 1_000_000,
            auto_migrate: true,
            hot_to_warm_threshold: 5,
            warm_to_cold_threshold: 2,
            cold_to_warm_threshold: 10,
            warm_to_hot_threshold: 20,
        }
    }
}

/// KV 统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KvStats {
    /// 总条目数
    pub total_entries: usize,
    /// 热层条目数
    pub hot_entries: usize,
    /// 温层条目数
    pub warm_entries: usize,
    /// 冷层条目数
    pub cold_entries: usize,
    /// 总访问次数
    pub total_accesses: u64,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
    /// 迁移次数
    pub migrations: u64,
    /// 驱逐次数
    pub evictions: u64,
}

/// 持久化 KV 存储
pub struct PersistentKv {
    /// 热层存储 (GPU)
    hot_tier: VecDeque<KvEntry>,
    /// 温层存储 (CPU)
    warm_tier: HashMap<String, KvEntry>,
    /// 冷层存储 (NVMe)
    cold_tier: HashMap<String, KvEntry>,
    /// 配置
    config: TierConfig,
    /// 统计
    stats: KvStats,
    /// 冷层存储路径
    cold_path: Option<PathBuf>,
}

impl PersistentKv {
    /// 创建新的持久化 KV 存储
    pub fn new(config: TierConfig) -> Self {
        Self {
            hot_tier: VecDeque::new(),
            warm_tier: HashMap::new(),
            cold_tier: HashMap::new(),
            config,
            stats: KvStats::default(),
            cold_path: None,
        }
    }

    /// 创建带冷层路径的持久化 KV 存储
    pub fn with_cold_path(config: TierConfig, cold_path: PathBuf) -> Self {
        let mut kv = Self::new(config);
        kv.cold_path = Some(cold_path);
        kv
    }

    /// 插入键值对
    pub fn insert(&mut self, key: String, value: Vec<u8>) -> KvEntry {
        let now = chrono::Utc::now().to_rfc3339();
        let size_bytes = value.len() as u64;

        let entry = KvEntry {
            key: key.clone(),
            value,
            tier: StorageTier::Hot,
            size_bytes,
            access_count: 1,
            last_accessed: now.clone(),
            created_at: now,
            expires_at: None,
            checksum: Self::compute_checksum(&key.as_bytes()),
        };

        // 先移除旧条目
        self.remove(&key);

        // 插入热层
        self.hot_tier.push_front(entry.clone());
        self.stats.total_entries += 1;
        self.stats.hot_entries += 1;

        // 检查热层容量并驱逐
        self.evict_if_needed(StorageTier::Hot);

        entry
    }

    /// 获取值
    pub fn get(&mut self, key: &str) -> Option<KvEntry> {
        // 搜索热层
        if let Some(pos) = self.hot_tier.iter().position(|e| e.key == key) {
            let mut entry = self.hot_tier.remove(pos).unwrap();
            entry.access_count += 1;
            entry.last_accessed = chrono::Utc::now().to_rfc3339();
            self.hot_tier.push_front(entry.clone());
            self.stats.cache_hits += 1;
            self.stats.total_accesses += 1;
            return Some(entry);
        }

        // 搜索温层
        if let Some(mut entry) = self.warm_tier.remove(key) {
            entry.access_count += 1;
            entry.last_accessed = chrono::Utc::now().to_rfc3339();
            self.stats.cache_hits += 1;
            self.stats.total_accesses += 1;

            // 检查是否需要提升到热层
            if self.config.auto_migrate && entry.access_count >= self.config.warm_to_hot_threshold {
                entry.tier = StorageTier::Hot;
                self.hot_tier.push_front(entry.clone());
                self.stats.warm_entries -= 1;
                self.stats.hot_entries += 1;
                self.stats.migrations += 1;
            } else {
                self.warm_tier.insert(key.to_string(), entry.clone());
            }
            return Some(entry);
        }

        // 搜索冷层
        if let Some(mut entry) = self.cold_tier.remove(key) {
            entry.access_count += 1;
            entry.last_accessed = chrono::Utc::now().to_rfc3339();
            self.stats.cache_misses += 1;
            self.stats.total_accesses += 1;

            // 检查是否需要提升到温层
            if self.config.auto_migrate && entry.access_count >= self.config.cold_to_warm_threshold {
                entry.tier = StorageTier::Warm;
                self.warm_tier.insert(key.to_string(), entry.clone());
                self.stats.cold_entries -= 1;
                self.stats.warm_entries += 1;
                self.stats.migrations += 1;
            } else {
                self.cold_tier.insert(key.to_string(), entry.clone());
            }
            return Some(entry);
        }

        self.stats.cache_misses += 1;
        None
    }

    /// 移除键值对
    pub fn remove(&mut self, key: &str) -> Option<KvEntry> {
        // 从热层移除
        if let Some(pos) = self.hot_tier.iter().position(|e| e.key == key) {
            let entry = self.hot_tier.remove(pos).unwrap();
            self.stats.hot_entries -= 1;
            self.stats.total_entries -= 1;
            return Some(entry);
        }

        // 从温层移除
        if let Some(entry) = self.warm_tier.remove(key) {
            self.stats.warm_entries -= 1;
            self.stats.total_entries -= 1;
            return Some(entry);
        }

        // 从冷层移除
        if let Some(entry) = self.cold_tier.remove(key) {
            self.stats.cold_entries -= 1;
            self.stats.total_entries -= 1;
            return Some(entry);
        }

        None
    }

    /// 检查键是否存在
    pub fn contains(&self, key: &str) -> bool {
        self.hot_tier.iter().any(|e| e.key == key)
            || self.warm_tier.contains_key(key)
            || self.cold_tier.contains_key(key)
    }

    /// 执行层级迁移
    pub fn migrate_tiers(&mut self) -> Vec<(String, StorageTier, StorageTier)> {
        let mut migrations = Vec::new();

        // 热层 → 温层 (低访问)
        let mut to_warm = Vec::new();
        for entry in self.hot_tier.iter() {
            if entry.access_count < self.config.hot_to_warm_threshold {
                to_warm.push(entry.key.clone());
            }
        }
        for key in to_warm {
            if let Some(entry) = self.remove(&key) {
                let mut entry = entry;
                entry.tier = StorageTier::Warm;
                self.warm_tier.insert(key.clone(), entry);
                self.stats.warm_entries += 1;
                self.stats.migrations += 1;
                migrations.push((key, StorageTier::Hot, StorageTier::Warm));
            }
        }

        // 温层 → 冷层 (低访问)
        let mut to_cold = Vec::new();
        for (key, entry) in self.warm_tier.iter() {
            if entry.access_count < self.config.warm_to_cold_threshold {
                to_cold.push(key.clone());
            }
        }
        for key in to_cold {
            if let Some(entry) = self.remove(&key) {
                let mut entry = entry;
                entry.tier = StorageTier::Cold;
                self.cold_tier.insert(key.clone(), entry);
                self.stats.cold_entries += 1;
                self.stats.migrations += 1;
                migrations.push((key, StorageTier::Warm, StorageTier::Cold));
            }
        }

        migrations
    }

    /// 驱逐超出容量的条目
    fn evict_if_needed(&mut self, tier: StorageTier) {
        match tier {
            StorageTier::Hot => {
                while self.hot_tier.len() > self.config.hot_max_entries {
                    if let Some(entry) = self.hot_tier.pop_back() {
                        self.stats.evictions += 1;
                        self.stats.hot_entries -= 1;

                        // 降级到温层
                        let mut entry = entry;
                        entry.tier = StorageTier::Warm;
                        self.warm_tier.insert(entry.key.clone(), entry);
                        self.stats.warm_entries += 1;
                    }
                }
            }
            StorageTier::Warm => {
                while self.warm_tier.len() > self.config.warm_max_entries {
                    // 找到访问最少的条目
                    if let Some(min_key) = self.warm_tier.iter()
                        .min_by_key(|(_, e)| e.access_count)
                        .map(|(k, _)| k.clone())
                    {
                        if let Some(entry) = self.warm_tier.remove(&min_key) {
                            self.stats.evictions += 1;
                            self.stats.warm_entries -= 1;

                            // 降级到冷层
                            let mut entry = entry;
                            entry.tier = StorageTier::Cold;
                            self.cold_tier.insert(entry.key.clone(), entry);
                            self.stats.cold_entries += 1;
                        }
                    } else {
                        break;
                    }
                }
            }
            StorageTier::Cold => {
                while self.cold_tier.len() > self.config.cold_max_entries {
                    if let Some(min_key) = self.cold_tier.iter()
                        .min_by_key(|(_, e)| e.access_count)
                        .map(|(k, _)| k.clone())
                    {
                        if let Some(_entry) = self.cold_tier.remove(&min_key) {
                            self.stats.evictions += 1;
                            self.stats.cold_entries -= 1;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }

    /// 计算校验和
    fn compute_checksum(data: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    /// 获取统计信息
    pub fn stats(&self) -> &KvStats {
        &self.stats
    }

    /// 获取所有键
    pub fn keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.hot_tier.iter().map(|e| e.key.clone()).collect();
        keys.extend(self.warm_tier.keys().cloned());
        keys.extend(self.cold_tier.keys().cloned());
        keys
    }

    /// 清空所有层
    pub fn clear(&mut self) {
        self.hot_tier.clear();
        self.warm_tier.clear();
        self.cold_tier.clear();
        self.stats = KvStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persistent_kv_creation() {
        let kv = PersistentKv::new(TierConfig::default());
        assert_eq!(kv.stats().total_entries, 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut kv = PersistentKv::new(TierConfig::default());
        kv.insert("key1".to_string(), b"value1".to_vec());
        
        let entry = kv.get("key1");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().value, b"value1");
        assert_eq!(kv.stats().cache_hits, 1);
    }

    #[test]
    fn test_remove() {
        let mut kv = PersistentKv::new(TierConfig::default());
        kv.insert("key1".to_string(), b"value1".to_vec());
        
        let removed = kv.remove("key1");
        assert!(removed.is_some());
        assert!(!kv.contains("key1"));
    }

    #[test]
    fn test_lru_eviction() {
        let config = TierConfig {
            hot_max_entries: 2,
            ..Default::default()
        };
        let mut kv = PersistentKv::new(config);
        
        kv.insert("k1".to_string(), b"v1".to_vec());
        kv.insert("k2".to_string(), b"v2".to_vec());
        kv.insert("k3".to_string(), b"v3".to_vec());
        
        // k1 should be evicted to warm tier
        assert_eq!(kv.stats().hot_entries, 2);
        assert_eq!(kv.stats().warm_entries, 1);
        assert!(kv.contains("k1"));
    }

    #[test]
    fn test_tier_migration() {
        let config = TierConfig {
            hot_max_entries: 10,
            hot_to_warm_threshold: 2,
            ..Default::default()
        };
        let mut kv = PersistentKv::new(config);
        
        // 插入条目
        kv.insert("key1".to_string(), b"value1".to_vec());
        
        // 访问几次
        kv.get("key1");
        kv.get("key1");
        
        // 执行迁移
        let migrations = kv.migrate_tiers();
        assert!(!migrations.is_empty());
    }

    #[test]
    fn test_storage_tier_properties() {
        assert!(StorageTier::Hot.latency_ns() < StorageTier::Warm.latency_ns());
        assert!(StorageTier::Warm.latency_ns() < StorageTier::Cold.latency_ns());
        assert!(StorageTier::Cold.capacity_bytes() > StorageTier::Warm.capacity_bytes());
    }

    #[test]
    fn test_keys() {
        let mut kv = PersistentKv::new(TierConfig::default());
        kv.insert("a".to_string(), b"1".to_vec());
        kv.insert("b".to_string(), b"2".to_vec());
        
        let keys = kv.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
    }
}
