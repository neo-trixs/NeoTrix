/// Bloom filter for probabilistic membership testing
pub struct BloomFilter {
    bits: Vec<bool>,
    size: usize,
    hash_count: usize,
    count: usize,
}

impl BloomFilter {
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        let size = Self::optimal_size(expected_items, false_positive_rate);
        let hash_count = Self::optimal_hash_count(size, expected_items);
        Self {
            bits: vec![false; size],
            size,
            hash_count,
            count: 0,
        }
    }

    fn optimal_size(n: usize, p: f64) -> usize {
        let ln2 = std::f64::consts::LN_2;
        ((-(n as f64) * p.ln()) / (ln2 * ln2)).ceil() as usize
    }

    fn optimal_hash_count(m: usize, n: usize) -> usize {
        ((m as f64 / n as f64) * std::f64::consts::LN_2).ceil() as usize
    }

    fn hash(&self, item: &str, seed: usize) -> usize {
        let mut h: usize = seed;
        for b in item.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as usize);
        }
        h % self.size
    }

    pub fn insert(&mut self, item: &str) {
        for i in 0..self.hash_count {
            let pos = self.hash(item, i + 1);
            self.bits[pos] = true;
        }
        self.count += 1;
    }

    pub fn contains(&self, item: &str) -> bool {
        (0..self.hash_count).all(|i| self.bits[self.hash(item, i + 1)])
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn estimated_false_positive_rate(&self) -> f64 {
        let n = self.count as f64;
        let m = self.size as f64;
        let k = self.hash_count as f64;
        (1.0 - (-k * n / m).exp()).powf(k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_check() {
        let mut f = BloomFilter::new(100, 0.01);
        f.insert("hello");
        assert!(f.contains("hello"));
        assert!(!f.contains("world"));
    }

    #[test]
    fn test_false_positive_rate() {
        let mut f = BloomFilter::new(1000, 0.01);
        for i in 0..500 {
            f.insert(&format!("item-{}", i));
        }
        let fps = (0..100)
            .filter(|i| f.contains(&format!("missing-{}", i)))
            .count();
        assert!(fps < 20, "Too many false positives: {}", fps);
    }

    #[test]
    fn test_count() {
        let mut f = BloomFilter::new(100, 0.01);
        for i in 0..10 {
            f.insert(&format!("x{}", i));
        }
        assert_eq!(f.count(), 10);
    }
}
