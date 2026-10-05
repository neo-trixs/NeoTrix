//! # KV-Context 虚拟化 (KVMem pattern)
//!
//! 三级 KV 缓存管理: GPU → Host → NVMe

// ⛔ 原首行 `use serde::{Deserialize, Serialize};` 在挂载本模块时暴露为
//    `unused import`（KVBackend 等均无 serde derive）⇒ 挂载即编译失败。
use std::collections::HashMap;

/// KV 缓存后端 trait
pub trait KVBackend: Send + Sync {
    fn store(&mut self, key: &str, value: Vec<u8>) -> bool;
    fn load(&self, key: &str) -> Option<Vec<u8>>;
    fn evict(&mut self, keys: &[String]) -> usize;
    fn size(&self) -> usize;
}

/// 内存后端
pub struct MemoryBackend {
    data: HashMap<String, Vec<u8>>,
    max_size: usize,
}

impl MemoryBackend {
    pub fn new(max_size: usize) -> Self {
        Self { data: HashMap::new(), max_size }
    }
}

impl KVBackend for MemoryBackend {
    fn store(&mut self, key: &str, value: Vec<u8>) -> bool {
        if self.data.len() >= self.max_size {
            return false;
        }
        self.data.insert(key.to_string(), value);
        true
    }
    
    fn load(&self, key: &str) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }
    
    fn evict(&mut self, keys: &[String]) -> usize {
        let mut count = 0;
        for key in keys {
            if self.data.remove(key).is_some() {
                count += 1;
            }
        }
        count
    }
    
    fn size(&self) -> usize {
        self.data.len()
    }
}

/// 三级 KV 缓存管理器
pub struct PagedKVManager {
    /// GPU 层 (热数据)
    gpu_tier: Box<dyn KVBackend>,
    /// Host 内存层 (温数据)
    host_tier: Box<dyn KVBackend>,
    /// NVMe 层 (冷数据)
    nvme_tier: Box<dyn KVBackend>,
    /// 访问计数器
    access_counts: HashMap<String, usize>,
}

impl PagedKVManager {
    pub fn new(gpu_size: usize, host_size: usize, nvme_size: usize) -> Self {
        Self {
            gpu_tier: Box::new(MemoryBackend::new(gpu_size)),
            host_tier: Box::new(MemoryBackend::new(host_size)),
            nvme_tier: Box::new(MemoryBackend::new(nvme_size)),
            access_counts: HashMap::new(),
        }
    }
    
    /// 查询: 优先从 GPU 层读取
    pub fn query(&mut self, key: &str) -> Option<Vec<u8>> {
        *self.access_counts.entry(key.to_string()).or_insert(0) += 1;
        
        // 优先 GPU
        if let Some(value) = self.gpu_tier.load(key) {
            return Some(value);
        }
        // 其次 Host
        if let Some(value) = self.host_tier.load(key) {
            // 提升到 GPU
            self.gpu_tier.store(key, value.clone());
            return Some(value);
        }
        // 最后 NVMe
        if let Some(value) = self.nvme_tier.load(key) {
            // 提升到 Host
            self.host_tier.store(key, value.clone());
            return Some(value);
        }
        None
    }
    
    /// 写入: 自动选择最佳层
    pub fn store(&mut self, key: &str, value: Vec<u8>) -> bool {
        if self.gpu_tier.size() < 1000 {
            self.gpu_tier.store(key, value)
        } else if self.host_tier.size() < 10000 {
            self.host_tier.store(key, value)
        } else {
            self.nvme_tier.store(key, value)
        }
    }
    
    /// 统计
    pub fn stats(&self) -> (usize, usize, usize) {
        (self.gpu_tier.size(), self.host_tier.size(), self.nvme_tier.size())
    }
}
