use super::context_budget::{CompactionIntent, CompactionPriority};
use std::collections::VecDeque;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub enum CompressionStrategy {
    Whitespace,
    PatternDedup,
    Truncated { max_chars: usize, summary: String },
    Passthrough,
}

#[derive(Debug, Clone)]
pub struct CompressedBlock {
    pub fingerprint: [u8; 16],
    pub original_length: usize,
    pub compressed: String,
    pub compressed_length: usize,
    pub ratio: f64,
    pub strategy: CompressionStrategy,
    pub created_at: i64,
    pub access_count: u64,
    pub hits: u64,
}

#[derive(Debug, Clone)]
pub struct CcrStats {
    pub entries: usize,
    pub capacity: usize,
    pub total_compressed_bytes: usize,
    pub total_original_bytes: usize,
    pub overall_ratio: f64,
    pub total_retrievals: u64,
}

struct CacheEntry {
    original: Vec<u8>,
    fingerprint: [u8; 16],
    created: Instant,
    access_count: u64,
}

pub struct CompressionStore {
    capacity: usize,
    ttl: Duration,
    strategy: CompressionStrategy,
    cache: VecDeque<CacheEntry>,
    total_original_bytes: usize,
    total_compressed_bytes: usize,
    total_retrievals: u64,
}

impl CompressionStore {
    pub fn new(capacity: usize, ttl_secs: u64) -> Self {
        Self {
            capacity,
            ttl: Duration::from_secs(ttl_secs),
            strategy: CompressionStrategy::Whitespace,
            cache: VecDeque::with_capacity(capacity),
            total_original_bytes: 0,
            total_compressed_bytes: 0,
            total_retrievals: 0,
        }
    }

    pub fn with_strategy(mut self, strategy: CompressionStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn compress(&mut self, content: &str) -> CompressedBlock {
        let original_len = content.len();
        let fp = compute_fingerprint(content.as_bytes());

        let compressed = match &self.strategy {
            CompressionStrategy::Whitespace => compress_whitespace(content),
            CompressionStrategy::PatternDedup => compress_pattern_dedup(content),
            CompressionStrategy::Truncated { max_chars, summary } => {
                let s = compress_whitespace(content);
                if s.len() > *max_chars {
                    let truncated: String = s.chars().take(*max_chars).collect();
                    format!("{}\n\n[--- truncated, summary: {} ---]", truncated, summary)
                } else {
                    s
                }
            }
            CompressionStrategy::Passthrough => content.to_string(),
        };

        let compressed_len = compressed.len();
        let ratio = if original_len > 0 {
            compressed_len as f64 / original_len as f64
        } else {
            1.0
        };

        self.evict_expired();
        if self.cache.len() >= self.capacity {
            self.cache.pop_front();
        }

        if self.cache.iter().all(|e| e.fingerprint != fp) {
            self.cache.push_back(CacheEntry {
                original: content.as_bytes().to_vec(),
                fingerprint: fp,
                created: Instant::now(),
                access_count: 0,
            });
            self.total_original_bytes += original_len;
            self.total_compressed_bytes += compressed_len;
        }

        let now_ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        CompressedBlock {
            fingerprint: fp,
            original_length: original_len,
            compressed,
            compressed_length: compressed_len,
            ratio,
            strategy: self.strategy.clone(),
            created_at: now_ts,
            access_count: 0,
            hits: 0,
        }
    }

    pub fn retrieve(&mut self, fingerprint: &[u8; 16]) -> Option<Vec<u8>> {
        self.evict_expired();
        let pos = self
            .cache
            .iter()
            .position(|e| e.fingerprint == *fingerprint)?;
        let entry = &mut self.cache[pos];
        entry.access_count += 1;
        self.total_retrievals += 1;
        Some(entry.original.clone())
    }

    pub fn is_cached(&self, fingerprint: &[u8; 16]) -> bool {
        self.cache
            .iter()
            .any(|e| e.fingerprint == *fingerprint && e.created.elapsed() < self.ttl)
    }

    pub fn stats(&self) -> CcrStats {
        let overall = if self.total_original_bytes > 0 {
            self.total_compressed_bytes as f64 / self.total_original_bytes as f64
        } else {
            1.0
        };

        CcrStats {
            entries: self.cache.len(),
            capacity: self.capacity,
            total_compressed_bytes: self.total_compressed_bytes,
            total_original_bytes: self.total_original_bytes,
            overall_ratio: overall,
            total_retrievals: self.total_retrievals,
        }
    }

    fn evict_expired(&mut self) {
        let ttl = self.ttl;
        self.cache.retain(|e| e.created.elapsed() < ttl);
    }
}

pub fn compute_fingerprint(content: &[u8]) -> [u8; 16] {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let hash = hasher.finish();
    let mut fp = [0u8; 16];
    fp[..8].copy_from_slice(&hash.to_le_bytes());
    fp[8..16].copy_from_slice(&hash.to_le_bytes());
    fp
}

pub fn should_compress(intent: &CompactionIntent, _store: &CompressionStore) -> bool {
    matches!(
        intent.priority,
        CompactionPriority::Normal | CompactionPriority::Low
    )
}

fn compress_whitespace(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut in_space = false;
    for ch in content.chars() {
        if ch.is_whitespace() && ch != '\n' {
            if !in_space {
                result.push(' ');
                in_space = true;
            }
        } else {
            result.push(ch);
            in_space = false;
        }
    }
    result
}

fn compress_pattern_dedup(content: &str) -> String {
    let mut result;
    let mut lines: Vec<&str> = content.lines().collect();
    if lines.len() > 100 {
        let repeated = find_repeated_lines(&lines);
        if repeated > 10 {
            lines.truncate(50);
            result = lines.join("\n");
            result.push_str(&format!("\n[... {} repeated lines removed ...]", repeated));
            return result;
        }
    }
    result = compress_whitespace(content);
    result
}

fn find_repeated_lines(lines: &[&str]) -> usize {
    use std::collections::HashMap;
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            *counts.entry(trimmed).or_insert(0) += 1;
        }
    }
    counts.values().filter(|&&c| c > 3).map(|c| c - 1).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_passthrough() {
        let mut store = CompressionStore::new(100, 3600);
        let content = "hello world";
        let block = store.compress(content);
        assert_eq!(block.original_length, content.len());
        assert!(block.ratio <= 1.0);
        assert_eq!(block.strategy, CompressionStrategy::Whitespace);
    }

    #[test]
    fn test_compress_and_retrieve_roundtrip() {
        let mut store = CompressionStore::new(100, 3600);
        let content = "The quick brown fox jumps over the lazy dog.";
        let block = store.compress(content);
        let retrieved = store.retrieve(&block.fingerprint);
        assert!(retrieved.is_some());
        assert_eq!(String::from_utf8(retrieved.unwrap()).unwrap(), content);
    }

    #[test]
    fn test_compress_whitespace_normalizes() {
        let compressed = compress_whitespace("hello     world\n\n\nend");
        assert_eq!(compressed, "hello world\n\n\nend");
    }

    #[test]
    fn test_retrieve_non_existent_returns_none() {
        let mut store = CompressionStore::new(100, 3600);
        let fp = [0u8; 16];
        assert!(store.retrieve(&fp).is_none());
    }

    #[test]
    fn test_cache_eviction_when_over_capacity() {
        let mut store = CompressionStore::new(2, 3600);
        store.compress("content1");
        store.compress("content2");
        let _block3 = store.compress("content3");
        assert_eq!(store.cache.len(), 2);
        assert!(!store.is_cached(&[0u8; 16]));
    }

    #[test]
    fn test_ttl_expiration() {
        let mut store = CompressionStore::new(100, 0);
        let block = store.compress("ephemeral content");
        assert!(!store.is_cached(&block.fingerprint));
    }

    #[test]
    fn test_stats_accuracy() {
        let mut store = CompressionStore::new(100, 3600);
        store.compress("hello");
        store.compress("world");
        let stats = store.stats();
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.capacity, 100);
    }

    #[test]
    fn test_should_compress_normal() {
        let store = CompressionStore::new(100, 3600);
        let intent = CompactionIntent {
            session_id: "sess".into(),
            previous_salience: 0.5,
            target_cursor: 0,
            summary_blocks: vec![],
            current_source: super::super::context_budget::BudgetSourceType::Stream,
            reserve_tokens: 0,
            priority: CompactionPriority::Normal,
        };
        assert!(should_compress(&intent, &store));
    }

    #[test]
    fn test_should_not_compress_critical() {
        let store = CompressionStore::new(100, 3600);
        let intent = CompactionIntent {
            session_id: "sess".into(),
            previous_salience: 0.5,
            target_cursor: 0,
            summary_blocks: vec![],
            current_source: super::super::context_budget::BudgetSourceType::Stream,
            reserve_tokens: 0,
            priority: CompactionPriority::Critical,
        };
        assert!(!should_compress(&intent, &store));
    }

    #[test]
    fn test_compute_fingerprint_deterministic() {
        let content = b"deterministic test";
        let fp1 = compute_fingerprint(content);
        let fp2 = compute_fingerprint(content);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_different_content_different_fingerprint() {
        let fp1 = compute_fingerprint(b"content a");
        let fp2 = compute_fingerprint(b"content b");
        assert_ne!(fp1, fp2);
    }
}

pub struct AdaptiveRateController {
    rate: f64,
    hits: usize,
    misses: usize,
}

impl AdaptiveRateController {
    pub fn new(initial_rate: f64, _window_size: usize) -> Self {
        Self {
            rate: initial_rate,
            hits: 0,
            misses: 0,
        }
    }

    pub fn current_rate(&self) -> f64 {
        self.rate
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            self.rate
        } else {
            self.hits as f64 / total as f64
        }
    }

    pub fn record_access(&mut self, is_hit: bool) {
        if is_hit {
            self.hits += 1;
        } else {
            self.misses += 1;
        }
    }

    pub fn adjust_rate(&mut self) {
        let hit_rate = self.hit_rate();
        if hit_rate < 0.5 {
            self.rate *= 1.2;
        } else {
            self.rate *= 0.9;
        }
        self.rate = self.rate.clamp(0.1, 10.0);
    }
}
