//! 共享工具函数 — 消除跨模块重复

/// 获取当前时间戳 (Unix seconds, i64)
pub fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 获取当前时间戳 (Unix seconds, u64)
pub fn now_ts_u64() -> u64 {
    now_ts() as u64
}

/// 余弦相似度 — f32 slices → f64 score
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)) as f64
}

/// 环形缓冲区 — 固定容量，FIFO 淘汰
pub struct RingBuffer<T> {
    items: Vec<T>,
    max_size: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            items: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.items.len() >= self.max_size {
            self.items.remove(0);
        }
        self.items.push(item);
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl<T> Default for RingBuffer<T> {
    fn default() -> Self {
        Self::new(64)
    }
}

/// 通用 HashMap 索引 — 快速 key→value 查找
pub struct IndexMap<K, V> {
    map: std::collections::HashMap<K, V>,
}

impl<K: Eq + std::hash::Hash, V> IndexMap<K, V> {
    pub fn new() -> Self {
        Self {
            map: std::collections::HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.map.get_mut(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter()
    }
}

impl<K: Eq + std::hash::Hash, V> Default for IndexMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_ts() {
        let ts = now_ts();
        assert!(ts > 0);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = [1.0, 2.0, 3.0];
        let b = [1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let a: [f32; 0] = [];
        let b: [f32; 0] = [];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_ring_buffer_basic() {
        let mut buf = RingBuffer::new(3);
        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.items(), &[1, 2, 3]);
        assert_eq!(buf.len(), 3);

        buf.push(4);
        assert_eq!(buf.items(), &[2, 3, 4]);
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn test_ring_buffer_default() {
        let buf: RingBuffer<i32> = RingBuffer::default();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }
}
