//! LMCache HotStore — 热存储层适配器
//!
//! 基于 LMCache 的 KV cache 复用模式。所有 KV 条目驻留于内存 HashMap，
//! 按 LRU 顺序淘汰，超出 `max_size_bytes` 时触发 `evict`。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 热存储统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotStoreStats {
    /// 当前条目数量
    pub entries: usize,
    /// 当前占用字节数
    pub current_size_bytes: usize,
    /// 最大容量字节数
    pub max_size_bytes: usize,
    /// 累计淘汰次数
    pub eviction_count: usize,
}

/// LMCache HotStore — 热存储层适配器
///
/// 所有 KV 条目驻留于内存 HashMap，按插入顺序淘汰。
/// 当总大小超过 `max_size_bytes` 时，`put` 会自动触发淘汰。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LmcacheHotStore {
    store: HashMap<String, Vec<u8>>,
    insert_order: Vec<String>,
    max_size_bytes: usize,
    current_size_bytes: usize,
    eviction_count: usize,
}

impl Default for LmcacheHotStore {
    fn default() -> Self {
        Self::new(256 * 1024 * 1024) // 256 MiB
    }
}

impl LmcacheHotStore {
    /// 创建指定容量的 HotStore
    pub fn new(max_size_bytes: usize) -> Self {
        Self {
            store: HashMap::new(),
            insert_order: Vec::new(),
            max_size_bytes,
            current_size_bytes: 0,
            eviction_count: 0,
        }
    }

    /// 按 key 获取缓存值
    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.store.get(key).map(|v| v.as_slice())
    }

    /// 插入 KV 条目，超出容量时自动淘汰
    pub fn put(&mut self, key: String, value: Vec<u8>) -> Result<(), String> {
        let entry_size = value.len();
        if entry_size > self.max_size_bytes {
            return Err(format!(
                "entry size {entry_size} exceeds max capacity {}",
                self.max_size_bytes
            ));
        }

        // 如果 key 已存在，先减去旧值大小
        if let Some(old) = self.store.get(&key) {
            self.current_size_bytes -= old.len();
            // insert_order 不变，旧 key 仍在正确位置
        } else {
            self.insert_order.push(key.clone());
        }

        self.current_size_bytes += entry_size;
        self.store.insert(key, value);

        // 超出容量时淘汰最旧条目
        while self.current_size_bytes > self.max_size_bytes {
            self.evict_oldest();
        }

        Ok(())
    }

    /// 淘汰最旧的一条，返回被淘汰条目的字节数
    fn evict_oldest(&mut self) -> usize {
        let oldest_key = match self.insert_order.first() {
            Some(k) => k.clone(),
            None => return 0,
        };

        self.insert_order.remove(0);
        let size = self
            .store
            .remove(&oldest_key)
            .map(|v| v.len())
            .unwrap_or(0);
        self.current_size_bytes = self.current_size_bytes.saturating_sub(size);
        self.eviction_count += 1;
        size
    }

    /// 手动触发淘汰，返回被淘汰的总字节数
    pub fn evict(&mut self) -> usize {
        let mut freed = 0;
        while self.current_size_bytes > self.max_size_bytes / 2 {
            freed += self.evict_oldest();
        }
        freed
    }

    /// 返回当前统计
    pub fn stats(&self) -> HotStoreStats {
        HotStoreStats {
            entries: self.store.len(),
            current_size_bytes: self.current_size_bytes,
            max_size_bytes: self.max_size_bytes,
            eviction_count: self.eviction_count,
        }
    }

    /// 当前条目数量
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// 是否包含指定 key
    pub fn contains(&self, key: &str) -> bool {
        self.store.contains_key(key)
    }

    /// 移除指定 key，返回其大小
    pub fn remove(&mut self, key: &str) -> Option<usize> {
        if let Some(idx) = self.insert_order.iter().position(|k| k == key) {
            self.insert_order.remove(idx);
        }
        self.store.remove(key).map(|v| {
            let size = v.len();
            self.current_size_bytes = self.current_size_bytes.saturating_sub(size);
            size
        })
    }

    /// 清空所有条目
    pub fn clear(&mut self) {
        self.store.clear();
        self.insert_order.clear();
        self.current_size_bytes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_get() {
        let mut store = LmcacheHotStore::new(1024);
        store
            .put("key1".into(), b"value1".to_vec())
            .unwrap();
        assert_eq!(store.get("key1"), Some(b"value1".as_slice()));
        assert_eq!(store.get("missing"), None);
    }

    #[test]
    fn test_eviction_on_overflow() {
        let mut store = LmcacheHotStore::new(20);
        store.put("a".into(), vec![0u8; 10]).unwrap();
        store.put("b".into(), vec![0u8; 10]).unwrap();
        assert_eq!(store.len(), 2);

        // 插入第三条会触发淘汰
        store.put("c".into(), vec![0u8; 10]).unwrap();
        assert!(store.len() <= 2);
        let stats = store.stats();
        assert!(stats.eviction_count > 0);
    }

    #[test]
    fn test_entry_too_large() {
        let mut store = LmcacheHotStore::new(10);
        let result = store.put("big".into(), vec![0u8; 11]);
        assert!(result.is_err());
    }

    #[test]
    fn test_manual_evict() {
        let mut store = LmcacheHotStore::new(100);
        for i in 0..5 {
            store
                .put(format!("k{i}"), vec![0u8; 10])
                .unwrap();
        }
        let freed = store.evict();
        assert!(freed > 0);
        assert!(store.stats().current_size_bytes <= 50);
    }

    #[test]
    fn test_stats() {
        let mut store = LmcacheHotStore::new(512);
        store.put("x".into(), vec![0u8; 100]).unwrap();
        let s = store.stats();
        assert_eq!(s.entries, 1);
        assert_eq!(s.current_size_bytes, 100);
        assert_eq!(s.max_size_bytes, 512);
        assert_eq!(s.eviction_count, 0);
    }

    #[test]
    fn test_remove_and_contains() {
        let mut store = LmcacheHotStore::new(1024);
        store.put("a".into(), vec![1, 2, 3]).unwrap();
        assert!(store.contains("a"));
        let size = store.remove("a").unwrap();
        assert_eq!(size, 3);
        assert!(!store.contains("a"));
    }

    #[test]
    fn test_clear() {
        let mut store = LmcacheHotStore::new(1024);
        store.put("a".into(), vec![0u8; 50]).unwrap();
        store.clear();
        assert!(store.is_empty());
        assert_eq!(store.stats().current_size_bytes, 0);
    }

    #[test]
    fn test_overwrite_existing_key() {
        let mut store = LmcacheHotStore::new(1024);
        store.put("k".into(), vec![1, 2, 3]).unwrap();
        store.put("k".into(), vec![4, 5]).unwrap();
        assert_eq!(store.get("k"), Some([4, 5].as_slice()));
        assert_eq!(store.len(), 1);
        assert_eq!(store.stats().current_size_bytes, 2);
    }

    #[test]
    fn test_default_capacity() {
        let store = LmcacheHotStore::default();
        assert_eq!(store.stats().max_size_bytes, 256 * 1024 * 1024);
    }
}
