// FUTURE - not yet wired
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// 去重守卫 — 防止同一任务被多次处理
///
/// 受 OmniGet `has_url()` + BloomFilter 启发。
/// 三个层级:
/// - `SeenSet`: 精确去重 (HashSet)
/// - `BloomFilter`: 概率去重 (低内存，可误判)
/// - `IdempotencyGuard`: 带 TTL 的去重 + 并发保护
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IdempotencyKey {
    pub namespace: String,
    pub key: String,
}

impl IdempotencyKey {
    pub fn new(namespace: &str, key: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            key: key.to_string(),
        }
    }

    pub fn for_url(url: &str) -> Self {
        Self::new("url", url)
    }

    pub fn for_task(kind: &str, id: &str) -> Self {
        Self::new(kind, id)
    }

    pub fn composite(&self) -> String {
        format!("{}::{}", self.namespace, self.key)
    }
}

impl Hash for IdempotencyKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.namespace.hash(state);
        self.key.hash(state);
    }
}

impl PartialEq for IdempotencyKey {
    fn eq(&self, other: &Self) -> bool {
        self.namespace == other.namespace && self.key == other.key
    }
}

impl Eq for IdempotencyKey {}

/// 精确去重集合 — 用 HashSet 追踪已看到的 key
pub struct SeenSet {
    seen: Mutex<HashSet<IdempotencyKey>>,
    max_size: usize,
}

impl SeenSet {
    pub fn new(max_size: usize) -> Self {
        Self {
            seen: Mutex::new(HashSet::new()),
            max_size,
        }
    }

    /// 检查并插入。true = 首次见到，false = 已存在
    pub fn check_and_set(&self, key: IdempotencyKey) -> bool {
        let mut seen = self.seen.lock().unwrap_or_else(|e| e.into_inner());
        if seen.len() >= self.max_size {
            seen.clear();
        }
        seen.insert(key)
    }

    pub fn contains(&self, key: &IdempotencyKey) -> bool {
        self.seen
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains(key)
    }

    pub fn remove(&self, key: &IdempotencyKey) {
        self.seen
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(key);
    }

    pub fn clear(&self) {
        self.seen.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }

    pub fn len(&self) -> usize {
        self.seen.lock().unwrap_or_else(|e| e.into_inner()).len()
    }
}

/// 带 TTL 和并发保护的去重守卫
pub struct IdempotencyGuard {
    inner: Mutex<Vec<(IdempotencyKey, Instant)>>,
    ttl: Duration,
    max_entries: usize,
}

impl IdempotencyGuard {
    pub fn new(ttl_secs: u64, max_entries: usize) -> Self {
        Self {
            inner: Mutex::new(Vec::new()),
            ttl: Duration::from_secs(ttl_secs),
            max_entries,
        }
    }

    /// 尝试获取 key 的独占使用权
    /// 返回 Ok(true) = 首次获取，Ok(false) = 重复
    pub fn try_acquire(&self, key: IdempotencyKey) -> Result<bool, &'static str> {
        let mut inner = self.inner.lock().map_err(|_| "lock poisoned")?;
        let now = Instant::now();

        // 清理过期条目
        inner.retain(|(_, t)| now.duration_since(*t) < self.ttl);

        // 检查是否已存在
        if inner.iter().any(|(k, _)| *k == key) {
            return Ok(false);
        }

        // 容量保护
        if inner.len() >= self.max_entries {
            inner.clear();
        }

        inner.push((key, now));
        Ok(true)
    }

    /// 释放 key
    pub fn release(&self, key: &IdempotencyKey) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.retain(|(k, _)| k != key);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.clear();
        }
    }

    pub fn active_count(&self) -> usize {
        self.inner.lock().map(|i| i.len()).unwrap_or(0)
    }
}

/// 简单的 BloomFilter (非概率性，用于小型集合时的快速去重)
///
/// 真正 BloomFilter 需要 bit-vec 依赖，这里是轻量版
pub struct SimpleBloom {
    bits: Vec<bool>,
    num_hashes: usize,
}

impl SimpleBloom {
    pub fn new(size: usize, num_hashes: usize) -> Self {
        Self {
            bits: vec![false; size],
            num_hashes,
        }
    }

    fn hash_at(&self, item: &str, i: usize) -> usize {
        let mut h = 0u64;
        for (_, b) in item.bytes().enumerate() {
            h = h.wrapping_mul(31).wrapping_add(b as u64);
            h ^= (i as u64).wrapping_mul(2654435761);
        }
        (h as usize) % self.bits.len()
    }

    pub fn insert(&mut self, item: &str) {
        for i in 0..self.num_hashes {
            let idx = self.hash_at(item, i);
            if idx < self.bits.len() {
                self.bits[idx] = true;
            }
        }
    }

    pub fn contains(&self, item: &str) -> bool {
        (0..self.num_hashes).all(|i| {
            let idx = self.hash_at(item, i);
            idx < self.bits.len() && self.bits[idx]
        })
    }

    pub fn insert_and_check(&mut self, item: &str) -> bool {
        let seen = self.contains(item);
        if !seen {
            self.insert(item);
        }
        !seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idempotency_key_composite() {
        let k = IdempotencyKey::new("test", "value");
        assert_eq!(k.composite(), "test::value");
    }

    #[test]
    fn test_idempotency_key_eq() {
        let a = IdempotencyKey::new("ns", "key");
        let b = IdempotencyKey::new("ns", "key");
        assert_eq!(a, b);
    }

    #[test]
    fn test_seen_set_dedup() {
        let set = SeenSet::new(100);
        let k = IdempotencyKey::new("url", "https://example.com");
        assert!(set.check_and_set(k.clone()));
        assert!(!set.check_and_set(k));
    }

    #[test]
    fn test_seen_set_max_size_resets() {
        let set = SeenSet::new(2);
        assert!(set.check_and_set(IdempotencyKey::new("a", "1")));
        assert!(set.check_and_set(IdempotencyKey::new("a", "2")));
        // 第三个应该触发 clear
        assert!(set.check_and_set(IdempotencyKey::new("a", "3")));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_idempotency_guard_basic() {
        let guard = IdempotencyGuard::new(60, 100);
        let k = IdempotencyKey::new("task", "123");
        assert_eq!(guard.try_acquire(k.clone()).unwrap(), true);
        assert_eq!(guard.try_acquire(k.clone()).unwrap(), false);
        guard.release(&k);
        assert_eq!(guard.try_acquire(k).unwrap(), true);
    }

    #[tokio::test]
    async fn test_idempotency_guard_ttl() {
        let guard = IdempotencyGuard::new(0, 100);
        let k = IdempotencyKey::new("task", "instant-expire");
        assert_eq!(guard.try_acquire(k.clone()).unwrap(), true);
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(guard.try_acquire(k).unwrap(), true);
    }

    #[test]
    fn test_simple_bloom() {
        let mut bloom = SimpleBloom::new(64, 3);
        assert!(bloom.insert_and_check("hello"));
        assert!(!bloom.insert_and_check("hello"));
        assert!(bloom.insert_and_check("world"));
    }
}
