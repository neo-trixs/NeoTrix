use std::collections::VecDeque;

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum CompressionStrategy {
    Summarize,
    TruncateOld { keep_ratio: f64 },
    PrioritizeByVsa,
    ClusterMerge,
    DropLowSignal,
}

#[derive(Debug, Clone)]
pub struct CompressConfig {
    pub max_tokens: usize,
    pub warn_ratio: f64,
    pub force_ratio: f64,
    pub strategy_order: Vec<CompressionStrategy>,
    pub vsa_similarity_threshold: f64,
}

impl Default for CompressConfig {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            warn_ratio: 0.70,
            force_ratio: 0.90,
            strategy_order: vec![
                CompressionStrategy::DropLowSignal,
                CompressionStrategy::TruncateOld { keep_ratio: 0.5 },
                CompressionStrategy::Summarize,
            ],
            vsa_similarity_threshold: 0.65,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContextItem {
    pub id: String,
    pub content: String,
    pub vsa_hash: u64,
    pub signal_strength: f64,
    pub created_cycle: u64,
    pub access_count: u64,
}

#[derive(Debug, Clone)]
pub struct CompressionReport {
    pub tokens_before: usize,
    pub tokens_after: usize,
    pub strategies_applied: Vec<CompressionStrategy>,
    pub items_removed: usize,
    pub items_summarized: usize,
    pub vsa_clusters_merged: usize,
}

#[derive(Debug, Clone)]
pub struct CompressCommand {
    pub strategy: CompressionStrategy,
    pub items: Vec<(String, String)>,
    pub threshold: f64,
}

impl CompressCommand {
    pub fn apply(&self) -> Vec<(String, String)> {
        match self.strategy {
            CompressionStrategy::Summarize => {
                let mut out: Vec<(String, String)> = Vec::new();
                let mid = self.items.len() / 2;
                for (i, (id, content)) in self.items.iter().enumerate() {
                    if i < mid {
                        let words: Vec<&str> = content.split_whitespace().collect();
                        let summary_len = words.len().max(20) / 3;
                        let compressed: String = words
                            .iter()
                            .take(summary_len)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(" ");
                        out.push((id.clone(), format!("[summarized] {}", compressed)));
                    } else {
                        out.push((id.clone(), content.clone()));
                    }
                }
                out
            }
            CompressionStrategy::TruncateOld { keep_ratio } => {
                let keep = (self.items.len() as f64 * keep_ratio).ceil() as usize;
                let start = self.items.len().saturating_sub(keep);
                self.items.iter().skip(start).cloned().collect()
            }
            CompressionStrategy::PrioritizeByVsa => {
                let mut sorted = self.items.clone();
                sorted.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
                let keep = (sorted.len() + 1) / 2;
                sorted.truncate(keep);
                sorted
            }
            CompressionStrategy::ClusterMerge => {
                let mut merged: Vec<(String, String)> = Vec::new();
                let mut seen: Vec<String> = Vec::new();
                for (id, content) in &self.items {
                    let words: Vec<&str> = content.split_whitespace().collect();
                    let key = words.iter().take(3).cloned().collect::<Vec<_>>().join(" ");
                    if seen.contains(&key) {
                        if let Some(last) = merged.last_mut() {
                            last.1.push(' ');
                            last.1.push_str(content);
                        }
                    } else {
                        seen.push(key);
                        merged.push((id.clone(), content.clone()));
                    }
                }
                merged
            }
            CompressionStrategy::DropLowSignal => {
                let threshold = self.threshold;
                self.items
                    .iter()
                    .filter(|(_, content)| {
                        let words: Vec<&str> = content.split_whitespace().collect();
                        let signal = words.iter().filter(|w| w.len() > 3).count().max(1) as f64
                            / words.len().max(1) as f64;
                        signal >= threshold
                    })
                    .cloned()
                    .collect()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContextCompressor {
    pub config: CompressConfig,
    pub items: VecDeque<ContextItem>,
    pub total_compressed: usize,
    pub total_tokens_saved: usize,
}

impl ContextCompressor {
    pub fn new(config: CompressConfig) -> Self {
        Self {
            config,
            items: VecDeque::new(),
            total_compressed: 0,
            total_tokens_saved: 0,
        }
    }

    pub fn add_item(&mut self, item: ContextItem) {
        self.items.push_back(item);
    }

    pub fn estimate_tokens(content: &str) -> usize {
        content.len() / 4 + 1
    }

    pub fn current_tokens(&self) -> usize {
        self.items
            .iter()
            .map(|i| Self::estimate_tokens(&i.content))
            .sum()
    }

    pub fn should_compress(&self) -> bool {
        let ratio = self.current_tokens() as f64 / self.config.max_tokens as f64;
        ratio >= self.config.warn_ratio
    }

    pub fn compress(&mut self) -> CompressionReport {
        let tokens_before = self.current_tokens();
        let ratio = tokens_before as f64 / self.config.max_tokens as f64;

        let mut strategies_applied: Vec<CompressionStrategy> = Vec::new();
        let mut items_removed = 0;
        let mut items_summarized = 0;
        let mut vsa_clusters_merged = 0;

        if ratio >= self.config.force_ratio {
            for strategy in &self.config.strategy_order {
                let before_len = self.items.len();
                let cmd = CompressCommand {
                    strategy: strategy.clone(),
                    items: self
                        .items
                        .iter()
                        .map(|i| (i.id.clone(), i.content.clone()))
                        .collect(),
                    threshold: self.config.vsa_similarity_threshold,
                };
                let result = cmd.apply();
                let old_items: std::collections::HashMap<String, ContextItem> =
                    self.items.drain(..).map(|i| (i.id.clone(), i)).collect();
                for (id, content) in result {
                    if let Some(mut o) = old_items.get(&id).cloned() {
                        o.content = content;
                        self.items.push_back(o);
                    } else {
                        self.items.push_back(ContextItem {
                            id,
                            content,
                            vsa_hash: 0,
                            signal_strength: 0.5,
                            created_cycle: 0,
                            access_count: 0,
                        });
                    }
                }
                let removed = before_len.saturating_sub(self.items.len());
                items_removed += removed;
                strategies_applied.push(strategy.clone());
                match strategy {
                    CompressionStrategy::Summarize => items_summarized += removed,
                    CompressionStrategy::ClusterMerge => vsa_clusters_merged += removed,
                    _ => {}
                }
                if self.current_tokens() <= self.config.max_tokens {
                    break;
                }
            }
            self.total_compressed += 1;
        }

        let tokens_after = self.current_tokens();
        self.total_tokens_saved += tokens_before.saturating_sub(tokens_after);

        CompressionReport {
            tokens_before,
            tokens_after,
            strategies_applied,
            items_removed,
            items_summarized,
            vsa_clusters_merged,
        }
    }

    pub fn compress_if_needed(&mut self) -> Option<CompressionReport> {
        if self.should_compress() {
            Some(self.compress())
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn stats(&self) -> CompressorStats {
        CompressorStats {
            item_count: self.items.len(),
            current_tokens: self.current_tokens(),
            total_compressed: self.total_compressed,
            total_tokens_saved: self.total_tokens_saved,
            max_tokens: self.config.max_tokens,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompressorStats {
    pub item_count: usize,
    pub current_tokens: usize,
    pub total_compressed: usize,
    pub total_tokens_saved: usize,
    pub max_tokens: usize,
}

/// Stats for the VSA-based thought history compressor.
#[derive(Debug, Clone)]
pub struct VsaCompressorStats {
    pub active_bundles: usize,
    pub total_raw_compressed: u64,
    pub total_compression_events: u64,
}

/// A compressed bundle of thought history entries.
/// Replaces N raw entries with a single VSA bundle vector + metadata.
#[derive(Debug, Clone)]
pub struct ThoughtBundle {
    pub label: String,
    pub vsa_bundle: Vec<u8>,
    pub count: usize,
    pub cycle_range: (u64, u64),
    pub avg_confidence: f64,
}

/// VSA-aware thought history compressor.
/// Replaces old entries with bundled VSA vectors instead of dropping them.
#[derive(Debug, Clone)]
pub struct VsaThoughtCompressor {
    pub raw_keep_count: usize,
    pub max_bundles: usize,
    pub bundle_size: usize,
    pub bundles: VecDeque<ThoughtBundle>,
    pub total_raw_compressed: u64,
    pub total_compression_events: u64,
}

impl VsaThoughtCompressor {
    pub fn new() -> Self {
        Self {
            raw_keep_count: 30,
            max_bundles: 10,
            bundle_size: 10,
            bundles: VecDeque::with_capacity(10),
            total_raw_compressed: 0,
            total_compression_events: 0,
        }
    }

    /// Compress old entries from thought_history into VSA bundles.
    /// Replaces oldest entries with bundled VSA vectors in-place.
    /// Keeps `max_raw` most recent entries uncompressed.
    pub fn compress(
        &mut self,
        history: &mut VecDeque<(String, Vec<u8>, f64)>,
        max_raw: usize,
    ) -> Option<Vec<ThoughtBundle>> {
        if history.len() <= max_raw + self.bundle_size {
            return None;
        }
        let compress_count = history.len().saturating_sub(max_raw);
        let compress_count = compress_count.min(self.bundle_size * 2);
        if compress_count < self.bundle_size {
            return None;
        }

        let mut new_bundles: Vec<ThoughtBundle> = Vec::new();
        let mut to_remove = 0usize;

        for chunk_start in (0..compress_count).step_by(self.bundle_size) {
            let chunk_end = (chunk_start + self.bundle_size).min(compress_count);
            let chunk: Vec<&(String, Vec<u8>, f64)> = history
                .iter()
                .skip(chunk_start)
                .take(chunk_end - chunk_start)
                .collect();
            if chunk.is_empty() {
                break;
            }

            let vsa_refs: Vec<&[u8]> = chunk.iter().map(|(_, v, _)| v.as_slice()).collect();
            let bundled = QuantizedVSA::bundle(&vsa_refs);
            let avg_timestamp: f64 =
                chunk.iter().map(|(_, _, t)| t).sum::<f64>() / chunk.len() as f64;

            let label = format!("[bundle:{}/{}]", chunk_start, chunk_end);

            new_bundles.push(ThoughtBundle {
                label: label.clone(),
                vsa_bundle: bundled,
                count: chunk.len(),
                cycle_range: (chunk_start as u64, chunk_end as u64),
                avg_confidence: avg_timestamp,
            });
            to_remove = chunk_end;
            self.total_raw_compressed += chunk.len() as u64;
        }

        if to_remove > 0 {
            let drain_count = to_remove.min(history.len());
            for _ in 0..drain_count {
                history.pop_front();
            }
        }

        for bundle in new_bundles.iter().rev() {
            history.push_front((
                bundle.label.clone(),
                bundle.vsa_bundle.clone(),
                bundle.avg_confidence,
            ));
        }

        let bundle_count = self.bundles.len() + new_bundles.len();
        if bundle_count > self.max_bundles {
            let excess = bundle_count.saturating_sub(self.max_bundles);
            for _ in 0..excess {
                self.bundles.pop_front();
            }
        }

        for b in &new_bundles {
            self.bundles.push_back(b.clone());
        }

        self.total_compression_events += 1;

        if new_bundles.is_empty() {
            None
        } else {
            Some(new_bundles)
        }
    }

    pub fn stats(&self) -> VsaCompressorStats {
        VsaCompressorStats {
            active_bundles: self.bundles.len(),
            total_raw_compressed: self.total_raw_compressed,
            total_compression_events: self.total_compression_events,
        }
    }
}

impl Default for VsaThoughtCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_item(id: &str, content: &str) -> ContextItem {
        ContextItem {
            id: id.to_string(),
            content: content.to_string(),
            vsa_hash: 0,
            signal_strength: 0.5,
            created_cycle: 0,
            access_count: 0,
        }
    }

    #[test]
    fn test_default_config() {
        let cfg = CompressConfig::default();
        assert_eq!(cfg.max_tokens, 4096);
        assert!(cfg.strategy_order.len() >= 3);
    }

    #[test]
    fn test_should_compress_below_warn() {
        let mut c = ContextCompressor::new(CompressConfig::default());
        for i in 0..3 {
            c.add_item(dummy_item(&format!("i{}", i), "hello world"));
        }
        assert!(!c.should_compress());
    }

    #[test]
    fn test_should_compress_above_force() {
        let mut cfg = CompressConfig::default();
        cfg.max_tokens = 10;
        let mut c = ContextCompressor::new(cfg);
        for i in 0..10 {
            let content = "this is a long content item that should trigger compression ".repeat(5);
            c.add_item(dummy_item(&format!("i{}", i), &content));
        }
        assert!(c.should_compress());
        let report = c.compress();
        assert!(report.tokens_before > report.tokens_after);
        assert!(!report.strategies_applied.is_empty());
    }

    #[test]
    fn test_truncate_old_strategy() {
        let items = vec![
            ("a".to_string(), "alpha content here".to_string()),
            ("b".to_string(), "beta content here".to_string()),
            ("c".to_string(), "gamma content here".to_string()),
            ("d".to_string(), "delta content here".to_string()),
        ];
        let cmd = CompressCommand {
            strategy: CompressionStrategy::TruncateOld { keep_ratio: 0.5 },
            items: items.clone(),
            threshold: 0.0,
        };
        let result = cmd.apply();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "c");
    }

    #[test]
    fn test_summarize_strategy() {
        let items = vec![
            (
                "a".to_string(),
                "one two three four five six seven eight nine ten".to_string(),
            ),
            ("b".to_string(), "hello world foo bar baz".to_string()),
        ];
        let cmd = CompressCommand {
            strategy: CompressionStrategy::Summarize,
            items,
            threshold: 0.0,
        };
        let result = cmd.apply();
        assert_eq!(result.len(), 2);
        assert!(result[0].1.starts_with("[summarized]"));
        assert!(!result[1].1.starts_with("[summarized]"));
    }

    #[test]
    fn test_drop_low_signal_strategy() {
        let items = vec![
            ("a".to_string(), "a b c d e f g h".to_string()),
            (
                "b".to_string(),
                "meaningful substantial important content words here".to_string(),
            ),
        ];
        let cmd = CompressCommand {
            strategy: CompressionStrategy::DropLowSignal,
            items,
            threshold: 0.4,
        };
        let result = cmd.apply();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "b");
    }

    #[test]
    fn test_cluster_merge_strategy() {
        let items = vec![
            ("a".to_string(), "the quick brown fox".to_string()),
            ("b".to_string(), "the quick brown fox jumps".to_string()),
            ("c".to_string(), "completely different topic".to_string()),
        ];
        let cmd = CompressCommand {
            strategy: CompressionStrategy::ClusterMerge,
            items,
            threshold: 0.0,
        };
        let result = cmd.apply();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_compress_if_needed_noop() {
        let mut c = ContextCompressor::new(CompressConfig::default());
        c.add_item(dummy_item("a", "short"));
        assert!(c.compress_if_needed().is_none());
    }

    #[test]
    fn test_stats() {
        let mut c = ContextCompressor::new(CompressConfig::default());
        c.add_item(dummy_item("a", "hello world test content"));
        let s = c.stats();
        assert_eq!(s.item_count, 1);
        assert_eq!(s.total_compressed, 0);
    }

    #[test]
    fn test_clear() {
        let mut c = ContextCompressor::new(CompressConfig::default());
        c.add_item(dummy_item("a", "test"));
        c.clear();
        assert_eq!(c.items.len(), 0);
    }

    #[test]
    fn test_prioritize_by_vsa() {
        let items = vec![
            ("a".to_string(), "short".to_string()),
            (
                "b".to_string(),
                "this is a much longer content to test prioritization behavior".to_string(),
            ),
        ];
        let cmd = CompressCommand {
            strategy: CompressionStrategy::PrioritizeByVsa,
            items,
            threshold: 0.0,
        };
        let result = cmd.apply();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "b");
    }

    #[test]
    fn test_compress_multiple_strategies() {
        let mut cfg = CompressConfig::default();
        cfg.max_tokens = 20;
        cfg.strategy_order = vec![
            CompressionStrategy::DropLowSignal,
            CompressionStrategy::TruncateOld { keep_ratio: 0.5 },
        ];
        let mut c = ContextCompressor::new(cfg);
        for i in 0..8 {
            let content = "word ".repeat(if i < 4 { 20 } else { 5 });
            c.add_item(dummy_item(&format!("i{}", i), &content));
        }
        let report = c.compress();
        assert!(!report.strategies_applied.is_empty());
        assert!(report.tokens_before > report.tokens_after || report.items_removed > 0);
    }

    #[test]
    fn test_compressor_tracks_savings() {
        let mut cfg = CompressConfig::default();
        cfg.max_tokens = 5;
        let mut c = ContextCompressor::new(cfg);
        for i in 0..15 {
            c.add_item(dummy_item(
                &format!("i{}", i),
                &"content with enough words to exceed tiny limit ".repeat(3),
            ));
        }
        c.compress();
        assert!(c.total_compressed > 0);
    }

    // ── VsaThoughtCompressor tests ──

    fn dummy_thought(label: &str) -> (String, Vec<u8>, f64) {
        (
            label.to_string(),
            QuantizedVSA::seeded_random(label.len() as u64, 512),
            0.5,
        )
    }

    fn dummy_bundle_vec() -> Vec<u8> {
        QuantizedVSA::seeded_random(42, 512)
    }

    #[test]
    fn test_vsa_compressor_new() {
        let c = VsaThoughtCompressor::new();
        assert_eq!(c.raw_keep_count, 30);
        assert_eq!(c.max_bundles, 10);
        assert_eq!(c.total_raw_compressed, 0);
    }

    #[test]
    fn test_vsa_compressor_noop_below_limit() {
        let mut compressor = VsaThoughtCompressor::new();
        let mut history: VecDeque<(String, Vec<u8>, f64)> = VecDeque::new();
        for i in 0..35 {
            history.push_back(dummy_thought(&format!("t{}", i)));
        }
        // max_raw=30, size=35, diff=5 < bundle_size=10 → no compression
        let result = compressor.compress(&mut history, 30);
        assert!(result.is_none());
        assert_eq!(history.len(), 35);
    }

    #[test]
    fn test_vsa_compressor_compresses_old_entries() {
        let mut compressor = VsaThoughtCompressor::new();
        let mut history: VecDeque<(String, Vec<u8>, f64)> = VecDeque::new();
        for i in 0..50 {
            history.push_back(dummy_thought(&format!("t{}", i)));
        }
        // max_raw=30, size=50, diff=20, bundle_size=10 → 2 bundles
        let result = compressor.compress(&mut history, 30);
        assert!(result.is_some(), "should have compressed");
        let bundles = result.unwrap();
        assert_eq!(bundles.len(), 2, "should create 2 bundles from 20 entries");
        // 2 bundles + 30 raw = 32 entries
        assert_eq!(history.len(), 32);
    }

    #[test]
    fn test_vsa_compressor_tracks_stats() {
        let mut compressor = VsaThoughtCompressor::new();
        let mut history: VecDeque<(String, Vec<u8>, f64)> = VecDeque::new();
        for i in 0..50 {
            history.push_back(dummy_thought(&format!("t{}", i)));
        }
        compressor.compress(&mut history, 30);
        let stats = compressor.stats();
        assert_eq!(stats.total_raw_compressed, 20);
        assert_eq!(stats.active_bundles, 2);
    }

    #[test]
    fn test_vsa_compressor_preserves_recent_entries() {
        let mut compressor = VsaThoughtCompressor::new();
        let mut history: VecDeque<(String, Vec<u8>, f64)> = VecDeque::new();
        for i in 0..50 {
            history.push_back(dummy_thought(&format!("t{}", i)));
        }
        let recent_text = history
            .back()
            .expect("50 items just pushed, back exists")
            .0
            .clone();
        compressor.compress(&mut history, 30);
        // Most recent entry should still be present
        assert_eq!(history.back().unwrap().0, recent_text);
    }

    #[test]
    fn test_vsa_compressor_max_bundles_enforced() {
        let mut compressor = VsaThoughtCompressor::new();
        compressor.max_bundles = 3;
        compressor.bundle_size = 5;
        let mut history: VecDeque<(String, Vec<u8>, f64)> = VecDeque::new();

        // First compress: 50 entries → should create bundles
        for i in 0..50 {
            history.push_back(dummy_thought(&format!("t{}", i)));
        }
        compressor.compress(&mut history, 30);
        let s1 = compressor.stats();
        assert!(s1.active_bundles <= 3);

        // Second compress: add more entries
        for i in 50..70 {
            history.push_back(dummy_thought(&format!("t{}", i)));
        }
        compressor.compress(&mut history, 30);
        let s2 = compressor.stats();
        assert!(
            s2.active_bundles <= 3,
            "active_bundles={} should be <= 3",
            s2.active_bundles
        );
    }
}
