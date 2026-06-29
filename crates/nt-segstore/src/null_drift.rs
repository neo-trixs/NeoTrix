use crate::{VsaTag, VSA_BYTES, cosine_similarity};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy)]
pub struct NullDriftStats {
    pub count: usize,
    pub capacity: usize,
    pub utilization_pct: f64,
}

pub struct NullDriftMemory {
    buffer: Vec<[u8; VSA_BYTES]>,
    tags: Vec<VsaTag>,
    timestamps: Vec<u64>,
    capacity: usize,
    head: usize,
    count: usize,
}

impl NullDriftMemory {
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(1);
        let tags = vec![VsaTag::SelfMemory; cap];
        let timestamps = vec![0u64; cap];
        let buffer = vec![[0u8; VSA_BYTES]; cap];
        Self { buffer, tags, timestamps, capacity: cap, head: 0, count: 0 }
    }

    pub fn insert(&mut self, vector: &[u8], tag: VsaTag) -> usize {
        let slot = self.head;
        let mut buf = [0u8; VSA_BYTES];
        let copy_len = vector.len().min(VSA_BYTES);
        buf[..copy_len].copy_from_slice(&vector[..copy_len]);
        self.buffer[slot] = buf;
        self.tags[slot] = tag;
        self.timestamps[slot] = elapsed_nanos();
        self.head = (self.head + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
        slot
    }

    pub fn get(&self, index: usize) -> Option<(&[u8], VsaTag, u64)> {
        if index >= self.capacity || self.timestamps[index] == 0 { return None; }
        if self.count < self.capacity && index >= self.count { return None; }
        Some((&self.buffer[index], self.tags[index], self.timestamps[index]))
    }

    pub fn search(&self, query: &[u8], k: usize) -> Vec<(usize, f64)> {
        if self.count == 0 || query.len() != VSA_BYTES { return Vec::new(); }
        let valid = if self.count < self.capacity { self.count } else { self.capacity };
        let mut results = Vec::with_capacity(valid);
        for i in 0..valid {
            if self.timestamps[i] == 0 { continue; }
            let sim = cosine_similarity(query, &self.buffer[i]);
            results.push((i, sim));
        }
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        results
    }

    pub fn search_by_tag(&self, query: &[u8], tag: VsaTag, k: usize) -> Vec<(usize, f64)> {
        if self.count == 0 || query.len() != VSA_BYTES { return Vec::new(); }
        let valid = if self.count < self.capacity { self.count } else { self.capacity };
        let mut results = Vec::new();
        for i in 0..valid {
            if self.timestamps[i] == 0 || self.tags[i] != tag { continue; }
            let sim = cosine_similarity(query, &self.buffer[i]);
            results.push((i, sim));
        }
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);
        results
    }

    pub fn count(&self) -> usize { self.count }
    pub fn capacity(&self) -> usize { self.capacity }

    pub fn clear(&mut self) {
        self.head = 0;
        self.count = 0;
        self.timestamps.iter_mut().for_each(|t| *t = 0);
    }

    pub fn stats(&self) -> NullDriftStats {
        NullDriftStats {
            count: self.count,
            capacity: self.capacity,
            utilization_pct: if self.capacity > 0 {
                (self.count as f64 / self.capacity as f64) * 100.0
            } else { 0.0 },
        }
    }
}

fn elapsed_nanos() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vector(byte: u8) -> [u8; VSA_BYTES] { [byte; VSA_BYTES] }

    #[test]
    fn test_insert_and_get() {
        let mut mem = NullDriftMemory::new(10);
        let v = make_vector(0xAB);
        let idx = mem.insert(&v, VsaTag::SelfThought);
        assert_eq!(idx, 0);
        let (vec, tag, ts) = mem.get(idx).unwrap();
        assert_eq!(vec, &v[..]);
        assert_eq!(tag, VsaTag::SelfThought);
        assert!(ts > 0);
    }

    #[test]
    fn test_search_finds_nearest() {
        let mut mem = NullDriftMemory::new(10);
        mem.insert(&make_vector(0x00), VsaTag::SelfMemory);
        mem.insert(&make_vector(0xFF), VsaTag::SelfMemory);
        mem.insert(&make_vector(0x0F), VsaTag::SelfMemory);
        let results = mem.search(&make_vector(0x00), 3);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, 0);
        assert!((results[0].1 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_auto_eviction() {
        let mut mem = NullDriftMemory::new(3);
        mem.insert(&make_vector(0x00), VsaTag::SelfThought);
        mem.insert(&make_vector(0x11), VsaTag::SelfThought);
        mem.insert(&make_vector(0x22), VsaTag::SelfThought);
        assert_eq!(mem.count(), 3);
        let idx = mem.insert(&make_vector(0x33), VsaTag::SelfThought);
        assert_eq!(idx, 0);
        assert_eq!(mem.count(), 3);
    }

    #[test]
    fn test_clear() {
        let mut mem = NullDriftMemory::new(5);
        mem.insert(&make_vector(0x01), VsaTag::SelfPlan);
        mem.insert(&make_vector(0x02), VsaTag::SelfGoal);
        assert_eq!(mem.count(), 2);
        mem.clear();
        assert_eq!(mem.count(), 0);
        assert!(mem.get(0).is_none());
        assert!(mem.get(1).is_none());
    }
}
