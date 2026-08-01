use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::neotrix::l3_memory_impl::nt_memory_spatial::types::{
    TileCacheEntry, TileFormat, TileCacheStats, tile_key,
};

pub struct TileCache {
    entries: HashMap<String, TileCacheEntry>,
    max_entries: usize,
    max_bytes: usize,
    total_bytes: usize,
    hits: u64,
    misses: u64,
}

impl TileCache {
    pub fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
            max_bytes,
            total_bytes: 0,
            hits: 0,
            misses: 0,
        }
    }

    pub fn default_tile_cache() -> Self {
        Self::new(10_000, 500_000_000)
    }

    pub fn get(&mut self, z: u8, x: u64, y: u64) -> Option<&TileCacheEntry> {
        let key = tile_key(z, x, y);
        if let Some(entry) = self.entries.get(&key) {
            let now = now_ms();
            if now < entry.cached_at + entry.ttl_ms {
                self.hits += 1;
                return Some(entry);
            }
        }
        self.misses += 1;
        None
    }

    pub fn set(&mut self, z: u8, x: u64, y: u64, data: Vec<u8>, format: TileFormat, ttl_ms: u64) {
        let key = tile_key(z, x, y);
        let size = data.len();
        let entry = TileCacheEntry {
            data,
            format,
            cached_at: now_ms(),
            ttl_ms,
            size_bytes: size,
        };

        if let Some(old) = self.entries.get(&key) {
            self.total_bytes = self.total_bytes.saturating_sub(old.size_bytes);
        }

        self.total_bytes += size;
        self.entries.insert(key, entry);
        self.evict_if_needed();
    }

    fn evict_if_needed(&mut self) {
        while self.entries.len() > self.max_entries || self.total_bytes > self.max_bytes {
            let oldest_key = self.entries.iter()
                .min_by_key(|(_, e)| e.cached_at)
                .map(|(k, _)| k.clone());

            if let Some(key) = oldest_key {
                if let Some(entry) = self.entries.remove(&key) {
                    self.total_bytes = self.total_bytes.saturating_sub(entry.size_bytes);
                }
            } else {
                break;
            }
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.total_bytes = 0;
    }

    pub fn stats(&self) -> TileCacheStats {
        let total = self.hits + self.misses;
        TileCacheStats {
            entries: self.entries.len(),
            total_bytes: self.total_bytes,
            hits: self.hits,
            misses: self.misses,
            hit_rate: if total > 0 { self.hits as f64 / total as f64 } else { 0.0 },
        }
    }

    pub fn invalidate(&mut self, z: u8, x: u64, y: u64) {
        let key = tile_key(z, x, y);
        if let Some(entry) = self.entries.remove(&key) {
            self.total_bytes = self.total_bytes.saturating_sub(entry.size_bytes);
        }
    }

    pub fn invalidate_zoom(&mut self, z: u8) {
        let prefix = format!("tile/{}/", z);
        let to_remove: Vec<String> = self.entries.keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();
        for key in to_remove {
            if let Some(entry) = self.entries.remove(&key) {
                self.total_bytes = self.total_bytes.saturating_sub(entry.size_bytes);
            }
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[derive(Clone)]
pub struct SemanticTileCache {
    tiles: Vec<(Vec<f32>, TileCacheEntry)>,
    max_entries: usize,
    similarity_threshold: f32,
}

impl SemanticTileCache {
    pub fn new(max_entries: usize, similarity_threshold: f32) -> Self {
        Self { tiles: Vec::with_capacity(max_entries), max_entries, similarity_threshold }
    }

    pub fn find_similar(&self, _embedding: &[f32]) -> Option<&TileCacheEntry> {
        for (emb, entry) in &self.tiles {
            let sim = cosine_similarity(_embedding, emb);
            if sim >= self.similarity_threshold {
                return Some(entry);
            }
        }
        None
    }

    pub fn store(&mut self, embedding: Vec<f32>, entry: TileCacheEntry) {
        if self.max_entries == 0 {
            return; // cache disabled
        }
        if self.tiles.len() >= self.max_entries {
            self.tiles.remove(0);
        }
        self.tiles.push((embedding, entry));
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum();
    let nb: f32 = b.iter().map(|x| x * x).sum();
    if na == 0.0 || nb == 0.0 { return 0.0; }
    dot / (na.sqrt() * nb.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_cache_set_get() {
        let mut cache = TileCache::new(100, 1_000_000);
        cache.set(10, 512, 384, vec![0u8; 100], TileFormat::Mvt, 60_000);
        let entry = cache.get(10, 512, 384);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().size_bytes, 100);
    }

    #[test]
    fn test_tile_cache_miss() {
        let mut cache = TileCache::new(100, 1_000_000);
        assert!(cache.get(10, 0, 0).is_none());
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn test_tile_cache_eviction() {
        let mut cache = TileCache::new(2, 1_000_000);
        cache.set(0, 0, 0, vec![0u8; 10], TileFormat::Mvt, 60_000);
        cache.set(0, 0, 1, vec![0u8; 10], TileFormat::Mvt, 60_000);
        cache.set(0, 0, 2, vec![0u8; 10], TileFormat::Mvt, 60_000);
        assert_eq!(cache.stats().entries, 2);
    }

    #[test]
    fn test_tile_cache_invalidate_zoom() {
        let mut cache = TileCache::new(100, 1_000_000);
        cache.set(5, 10, 10, vec![], TileFormat::Mvt, 60_000);
        cache.set(5, 10, 11, vec![], TileFormat::Mvt, 60_000);
        cache.set(6, 20, 20, vec![], TileFormat::Mvt, 60_000);
        cache.invalidate_zoom(5);
        assert_eq!(cache.stats().entries, 1);
    }

    #[test]
    fn test_semantic_tile_cache() {
        let mut sc = SemanticTileCache::new(10, 0.9);
        let emb = vec![1.0, 0.0, 0.0];
        let entry = TileCacheEntry {
            data: vec![0u8; 10], format: TileFormat::Mvt,
            cached_at: 0, ttl_ms: 60_000, size_bytes: 10,
        };
        sc.store(emb.clone(), entry);
        assert!(sc.find_similar(&emb).is_some());
        assert!(sc.find_similar(&[0.0, 1.0, 0.0]).is_none());
    }

    #[test]
    fn test_semantic_tile_cache_zero_capacity_no_panic() {
        // Regression: SemanticTileCache::new(0, _) then store() hit
        // tiles.len() >= 0 -> remove(0) on an empty Vec -> panic. With
        // max_entries == 0 the cache is disabled and store() is a no-op.
        let mut sc = SemanticTileCache::new(0, 0.9);
        let entry = TileCacheEntry {
            data: vec![0u8; 10], format: TileFormat::Mvt,
            cached_at: 0, ttl_ms: 60_000, size_bytes: 10,
        };
        sc.store(vec![1.0, 0.0], entry);
        assert!(sc.find_similar(&[1.0, 0.0]).is_none());
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b)).abs() < 1e-6);
        assert!((cosine_similarity(&a, &a) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_tile_cache_clear() {
        let mut cache = TileCache::new(100, 1_000_000);
        cache.set(0, 0, 0, vec![], TileFormat::Mvt, 60_000);
        cache.clear();
        assert_eq!(cache.stats().entries, 0);
    }
}
