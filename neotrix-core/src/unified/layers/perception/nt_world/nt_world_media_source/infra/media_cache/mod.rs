pub mod multi_level;
pub mod warmup;
pub mod consistency;

pub struct MultiLevelCache;
impl MultiLevelCache {
    pub fn new() -> Self {
        Self
    }
}

pub struct CacheWarmer;
impl CacheWarmer {
    pub fn new() -> Self {
        Self
    }
}

pub struct CacheConsistency;
impl CacheConsistency {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionStrategy {
    Lru,
    Lfu,
    Ttl,
    Random,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WarmupConfig {
    pub enabled: bool,
    pub max_items: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InvalidationRule {
    pub pattern: String,
    pub strategy: InvalidationStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InvalidationStrategy {
    Immediate,
    Lazy,
    Scheduled,
}
