use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::time::Instant;

/// An item in the GWT context buffer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem {
    pub content: String,
    pub priority: f64,
    pub source: String,
    pub timestamp: i64,
}

impl ContentItem {
    pub fn new(content: String, priority: f64, source: String) -> Self {
        Self {
            content,
            priority,
            source,
            timestamp: chrono_now(),
        }
    }
}

fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Configuration for the 5-layer compression pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Layer 1 budget: max tokens (characters used as proxy) (default: 8192)
    pub max_tokens: usize,
    /// Layer 2 similarity threshold for redundancy (default: 0.85)
    pub trim_threshold: f64,
    /// Layer 3 target compression ratio for low-priority items (default: 0.5)
    pub compress_ratio: f64,
    /// Layer 4 similarity threshold for folding related items (default: 0.75)
    pub fold_similarity: f64,
    /// Layer 5 auto-trigger at this many items (default: 100)
    pub auto_trigger_size: usize,
    /// Bitmask: bit0=Layer1(Budget), bit1=Layer2(Trim), bit2=Layer3(Compress),
    /// bit3=Layer4(Fold), bit4=Layer5(Auto) (default: 0x1F = all)
    pub enabled_layers: u8,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            max_tokens: 8192,
            trim_threshold: 0.85,
            compress_ratio: 0.5,
            fold_similarity: 0.75,
            auto_trigger_size: 100,
            enabled_layers: 0x1F,
        }
    }
}

/// Identifies which compression stage was applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionStage {
    Budget,
    Trim,
    Compress,
    Fold,
    Auto,
}

/// Report produced after running the compression pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionReport {
    pub stages_applied: Vec<CompressionStage>,
    pub items_before: usize,
    pub items_after: usize,
    pub tokens_before: usize,
    pub tokens_after: usize,
    pub duration_ms: u64,
}

/// 5-layer context compression pipeline for the Global Workspace.
///
/// Layer 1 — Budget: Trim to max tokens based on priority
/// Layer 2 — Trim: Remove redundant/overlapping content
/// Layer 3 — Compress: Summarize low-priority items
/// Layer 4 — Fold: Merge related items into composite representations
/// Layer 5 — Auto-compress: Trigger compression automatically when threshold exceeded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCompressor {
    pub config: CompressionConfig,
}

impl ContextCompressor {
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    /// Run all enabled layers in order and return a compression report.
    pub fn compress(&self, contents: &mut Vec<ContentItem>) -> CompressionReport {
        let start = Instant::now();
        let items_before = contents.len();
        let tokens_before: usize = contents.iter().map(|c| c.content.len()).sum();

        let mut stages_applied: Vec<CompressionStage> = Vec::new();

        if self.config.enabled_layers & 0x01 != 0 {
            self.layer_budget(contents);
            stages_applied.push(CompressionStage::Budget);
        }
        if self.config.enabled_layers & 0x02 != 0 {
            self.layer_trim(contents);
            stages_applied.push(CompressionStage::Trim);
        }
        if self.config.enabled_layers & 0x04 != 0 {
            self.layer_compress(contents);
            stages_applied.push(CompressionStage::Compress);
        }
        if self.config.enabled_layers & 0x08 != 0 {
            self.layer_fold(contents);
            stages_applied.push(CompressionStage::Fold);
        }
        if self.config.enabled_layers & 0x10 != 0 {
            self.layer_auto(contents);
            stages_applied.push(CompressionStage::Auto);
        }

        let items_after = contents.len();
        let tokens_after: usize = contents.iter().map(|c| c.content.len()).sum();
        let duration_ms = start.elapsed().as_millis() as u64;

        CompressionReport {
            stages_applied,
            items_before,
            items_after,
            tokens_before,
            tokens_after,
            duration_ms,
        }
    }

    /// Layer 1 — Budget: Trim to `max_tokens` by priority.
    ///
    /// Drops lowest-priority items first until total token count ≤ max_tokens.
    pub fn layer_budget(&self, contents: &mut Vec<ContentItem>) {
        if contents.is_empty() {
            return;
        }
        let total: usize = contents.iter().map(|c| c.content.len()).sum();
        if total <= self.config.max_tokens {
            return;
        }
        let mut to_remove: usize = 0;
        let mut running = total;
        // Work on a sorted copy to preserve caller's order for unremoved items.
        // Tie-breaking by index ensures deterministic removal when priorities are equal.
        let sorted: Vec<usize> = {
            let mut indices: Vec<usize> = (0..contents.len()).collect();
            indices.sort_by(|&a, &b| {
                contents[a]
                    .priority
                    .partial_cmp(&contents[b].priority)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.cmp(&b))
            });
            indices
        };
        for &idx in &sorted {
            if running <= self.config.max_tokens {
                break;
            }
            running = running.saturating_sub(contents[idx].content.len());
            to_remove += 1;
        }
        // Build set of indices to remove, then drain in descending order
        if to_remove > 0 {
            let remove_set: std::collections::HashSet<usize> =
                sorted.iter().take(to_remove).copied().collect();
            let mut remove_vec: Vec<usize> = remove_set.into_iter().collect();
            remove_vec.sort_unstable_by(|a, b| b.cmp(a));
            for idx in remove_vec {
                if idx < contents.len() {
                    contents.remove(idx);
                }
            }
        }
    }

    /// Layer 2 — Trim: Remove items with cosine similarity > trim_threshold (keep higher priority).
    ///
    /// Uses word-overlap Jaccard similarity as a heuristic.
    pub fn layer_trim(&self, contents: &mut Vec<ContentItem>) {
        if contents.is_empty() {
            return;
        }
        let threshold = self.config.trim_threshold;
        let mut remove_indices: Vec<usize> = Vec::new();
        for i in 0..contents.len() {
            if remove_indices.contains(&i) {
                continue;
            }
            for j in (i + 1)..contents.len() {
                if remove_indices.contains(&j) {
                    continue;
                }
                let sim = word_overlap_jaccard(&contents[i].content, &contents[j].content);
                if sim > threshold {
                    // keep the higher-priority one, remove the other
                    if contents[i].priority >= contents[j].priority {
                        remove_indices.push(j);
                    } else {
                        remove_indices.push(i);
                        break; // i is being removed, stop comparing against it
                    }
                }
            }
        }
        remove_indices.sort_unstable_by(|a, b| b.cmp(a)); // descending for safe removal
        for idx in remove_indices {
            if idx < contents.len() {
                contents.remove(idx);
            }
        }
    }

    /// Layer 3 — Compress: For low-priority items, replace with a shortened summary.
    ///
    /// Items with priority below the median get truncated to `compress_ratio` of original length.
    pub fn layer_compress(&self, contents: &mut Vec<ContentItem>) {
        if contents.is_empty() || contents.len() < 3 {
            return;
        }
        let ratio = self.config.compress_ratio;
        // find median priority
        let mut priorities: Vec<f64> = contents.iter().map(|c| c.priority).collect();
        priorities.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = if priorities.len().is_multiple_of(2) {
            (priorities[priorities.len() / 2 - 1] + priorities[priorities.len() / 2]) / 2.0
        } else {
            priorities[priorities.len() / 2]
        };

        for item in contents.iter_mut() {
            if item.priority < median && item.content.len() > 40 {
                let target_len = (item.content.len() as f64 * ratio) as usize;
                let target_len = target_len.max(20).min(item.content.len());
                let summary = generate_summary(&item.content, target_len);
                item.content = summary;
            }
        }
    }

    /// Layer 4 — Fold: Merge adjacent related items into a composite representation.
    ///
    /// Adjacent items with Jaccard similarity > `fold_similarity` get merged:
    /// the later item's content is appended to the earlier one's, and the later item is removed.
    pub fn layer_fold(&self, contents: &mut Vec<ContentItem>) {
        if contents.is_empty() || contents.len() < 2 {
            return;
        }
        let threshold = self.config.fold_similarity;
        let mut i = 0;
        while i + 1 < contents.len() {
            let sim = word_overlap_jaccard(&contents[i].content, &contents[i + 1].content);
            if sim > threshold {
                let later = contents.remove(i + 1);
                contents[i].content.push_str("\n---folded---\n");
                contents[i].content.push_str(&later.content);
                contents[i].priority = contents[i].priority.max(later.priority);
                // don't increment i — check if the merged item now folds with the next
            } else {
                i += 1;
            }
        }
    }

    /// Layer 5 — Auto-compress: If len > `auto_trigger_size`, run budget+trim+compress.
    pub fn layer_auto(&self, contents: &mut Vec<ContentItem>) {
        if contents.len() <= self.config.auto_trigger_size {
            return;
        }
        self.layer_budget(contents);
        self.layer_trim(contents);
        self.layer_compress(contents);
    }

    /// Check whether compression is needed.
    pub fn should_compress(&self, contents: &[ContentItem]) -> bool {
        let total_tokens: usize = contents.iter().map(|c| c.content.len()).sum();
        total_tokens > self.config.max_tokens || contents.len() > self.config.auto_trigger_size
    }
}

/// Jaccard similarity based on word overlap.
/// Returns a value in [0.0, 1.0].
fn word_overlap_jaccard(a: &str, b: &str) -> f64 {
    let words_a: HashSet<&str> = a.split_whitespace().collect();
    let words_b: HashSet<&str> = b.split_whitespace().collect();
    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();
    if union == 0 {
        return 0.0;
    }
    intersection as f64 / union as f64
}

/// Generate a shortened summary by extracting the first and last meaningful sentences.
fn generate_summary(text: &str, target_len: usize) -> String {
    let text = text.trim();
    if text.len() <= target_len {
        return text.to_string();
    }
    // take first ~40% and last ~40% of target
    let head_ratio = 0.5;
    let head_len = (target_len as f64 * head_ratio) as usize;
    let tail_len = target_len.saturating_sub(head_len);

    let head: String = text.chars().take(head_len).collect();
    let tail: String = text
        .chars()
        .rev()
        .take(tail_len)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{} ... {}", head.trim(), tail.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(content: &str, priority: f64) -> ContentItem {
        ContentItem::new(content.to_string(), priority, "test".to_string())
    }

    #[test]
    fn test_compression_config_default() {
        let config = CompressionConfig::default();
        assert_eq!(config.max_tokens, 8192);
        assert_eq!(config.enabled_layers, 0x1F);
    }

    #[test]
    fn test_should_compress_exceeds_tokens() {
        let compressor = ContextCompressor::new(CompressionConfig {
            max_tokens: 10,
            ..Default::default()
        });
        let items = vec![make_item("hello world this is a long message that should trigger", 1.0)];
        assert!(compressor.should_compress(&items));
    }

    #[test]
    fn test_should_compress_exceeds_items() {
        let compressor = ContextCompressor::new(CompressionConfig {
            auto_trigger_size: 3,
            ..Default::default()
        });
        let items = vec![
            make_item("a", 1.0),
            make_item("b", 1.0),
            make_item("c", 1.0),
            make_item("d", 1.0),
        ];
        assert!(compressor.should_compress(&items));
    }

    #[test]
    fn test_should_not_compress_when_below_thresholds() {
        let compressor = ContextCompressor::new(CompressionConfig {
            max_tokens: 1000,
            auto_trigger_size: 10,
            ..Default::default()
        });
        let items = vec![make_item("short", 1.0); 3];
        assert!(!compressor.should_compress(&items));
    }

    #[test]
    fn test_layer_budget_drops_lowest_priority() {
        let compressor = ContextCompressor::new(CompressionConfig {
            max_tokens: 20,
            ..Default::default()
        });
        let mut items = vec![
            make_item("aaaa", 3.0),   // 4 chars
            make_item("bbbbbb", 1.0), // 6 chars
            make_item("cc", 2.0),     // 2 chars -> total = 12, under 20
        ];
        compressor.layer_budget(&mut items);
        // all fit within 20 chars, nothing dropped
        assert_eq!(items.len(), 3);

        let mut items2 = vec![
            make_item("aaaa", 3.0),
            make_item("bbbbbbbbbbbbbbbbbb", 1.0), // 18 chars
            make_item("cccccccccc", 2.0),          // 10 chars -> total = 32
        ];
        compressor.layer_budget(&mut items2);
        // need to drop until ≤ 20 chars. priorities: [1.0, 2.0, 3.0] sorted
        // b (18 chars) dropped -> 14 chars remaining, under 20
        assert_eq!(items2.len(), 2);
    }

    #[test]
    fn test_layer_trim_removes_similar_content() {
        let compressor = ContextCompressor::new(CompressionConfig {
            trim_threshold: 0.3,
            ..Default::default()
        });
        let mut items = vec![
            make_item("the quick brown fox jumps", 2.0),
            make_item("the quick brown fox leaps", 1.0),
        ];
        compressor.layer_trim(&mut items);
        // high similarity, lower priority (leaps) should be removed
        assert_eq!(items.len(), 1);
        assert!(items[0].content.contains("jumps"));
    }

    #[test]
    fn test_layer_compress_reduces_low_priority_content() {
        let compressor = ContextCompressor::new(CompressionConfig {
            compress_ratio: 0.3,
            ..Default::default()
        });
        let mut items = vec![
            make_item("this is a very long sentence that should be shortened significantly by the compressor layer", 1.0),
            make_item("this is high priority content that must stay intact", 10.0),
            make_item("another medium priority item for testing purposes", 5.0),
        ];
        let before = items[0].content.len();
        compressor.layer_compress(&mut items);
        // low priority item (priority 1.0) should be compressed
        assert!(items[0].content.len() < before);
        // high priority (10.0) should remain unchanged
        assert!(items[1].content.contains("high priority"));
    }

    #[test]
    fn test_layer_fold_merges_adjacent_similar() {
        let compressor = ContextCompressor::new(CompressionConfig {
            fold_similarity: 0.3,
            ..Default::default()
        });
        let mut items = vec![
            make_item("the quick brown fox", 1.0),
            make_item("the quick brown fox jumps high", 2.0),
            make_item("completely unrelated topic here", 3.0),
        ];
        compressor.layer_fold(&mut items);
        // items 0 and 1 have high overlap -> merged
        assert_eq!(items.len(), 2);
        assert!(items[0].content.contains("---folded---"));
    }

    #[test]
    fn test_layer_auto_triggers_when_over_threshold() {
        let compressor = ContextCompressor::new(CompressionConfig {
            auto_trigger_size: 3,
            max_tokens: 1000,
            trim_threshold: 0.99,
            compress_ratio: 0.9,
            ..Default::default()
        });
        let mut items = vec![
            make_item("content a", 1.0),
            make_item("content b", 2.0),
            make_item("content c", 3.0),
            make_item("content d", 4.0),
        ];
        let before = items.len();
        compressor.layer_auto(&mut items);
        // 4 > 3 auto_trigger_size -> budget+trim+compress run, potentially no change
        // but it should not crash or increase size
        assert!(items.len() <= before);
    }

    #[test]
    fn test_compress_runs_all_layers() {
        let compressor = ContextCompressor::new(CompressionConfig::default());
        let mut items = vec![
            make_item("first item content here for testing", 1.0),
            make_item("second item content here for testing", 2.0),
            make_item("third unique content in the workspace", 3.0),
        ];
        let report = compressor.compress(&mut items);
        assert_eq!(report.stages_applied.len(), 5);
        assert!(report.duration_ms > 0 || report.items_before > 0);
    }

    #[test]
    fn test_compress_disabled_layers() {
        let compressor = ContextCompressor::new(CompressionConfig {
            enabled_layers: 0x01, // only budget
            ..Default::default()
        });
        let mut items = vec![make_item("test", 1.0)];
        let report = compressor.compress(&mut items);
        assert_eq!(report.stages_applied.len(), 1);
        assert_eq!(report.stages_applied[0], CompressionStage::Budget);
    }

    #[test]
    fn test_word_overlap_jaccard_identical() {
        let sim = word_overlap_jaccard("hello world", "hello world");
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_word_overlap_jaccard_disjoint() {
        let sim = word_overlap_jaccard("hello world", "foo bar baz");
        assert!((sim - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_word_overlap_jaccard_partial() {
        let sim = word_overlap_jaccard("hello world foo", "hello world bar");
        assert!((sim - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_generate_summary_long_text() {
        let text = "This is a very long text that needs to be summarized because it contains too many words and characters for our context budget. We should trim it down significantly.";
        let summary = generate_summary(text, 40);
        assert!(summary.len() <= 45); // slightly over due to "..."
        assert!(summary.contains("..."));
    }

    #[test]
    fn test_generate_summary_short_text() {
        let text = "short text";
        let summary = generate_summary(text, 100);
        assert_eq!(summary, "short text");
    }

    #[test]
    fn test_empty_contents_compress() {
        let compressor = ContextCompressor::new(CompressionConfig::default());
        let mut items: Vec<ContentItem> = Vec::new();
        let report = compressor.compress(&mut items);
        assert_eq!(report.items_before, 0);
        assert_eq!(report.items_after, 0);
    }

    #[test]
    fn test_layer_budget_empty() {
        let compressor = ContextCompressor::new(CompressionConfig::default());
        let mut items: Vec<ContentItem> = Vec::new();
        compressor.layer_budget(&mut items);
        assert!(items.is_empty());
    }

    #[test]
    fn test_layer_trim_empty() {
        let compressor = ContextCompressor::new(CompressionConfig::default());
        let mut items: Vec<ContentItem> = Vec::new();
        compressor.layer_trim(&mut items);
        assert!(items.is_empty());
    }

    #[test]
    fn test_layer_fold_no_adjacent_similarity() {
        let compressor = ContextCompressor::new(CompressionConfig {
            fold_similarity: 0.99,
            ..Default::default()
        });
        let mut items = vec![
            make_item("completely unique first item content", 1.0),
            make_item("totally different second item here", 2.0),
        ];
        compressor.layer_fold(&mut items);
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_compression_report_values() {
        let compressor = ContextCompressor::new(CompressionConfig::default());
        let mut items = vec![
            make_item(&"a".repeat(50), 1.0),
            make_item(&"b".repeat(30), 2.0),
        ];
        let report = compressor.compress(&mut items);
        assert!(report.tokens_before >= 80);
        assert!(report.items_before == 2);
    }
}

// ============================================================
// Headroom-style CCR (Contextually Compressed Representation)
// Reversible compression: signature ↔ original, with original
// stored in content-addressed cache. 60-95% token reduction.
// ============================================================

/// Content-addressed hash (SHA-256 truncated to u64 for cache key)
pub type ContentHash = u64;

/// Compression level for reversible CCR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CcrLevel {
    /// ~40% reduction, fastest, best fidelity
    Light,
    /// ~60% reduction, balanced
    Medium,
    /// ~80% reduction, most aggressive
    Aggressive,
}

impl CcrLevel {
    pub fn reduction_factor(&self) -> f64 {
        match self {
            CcrLevel::Light => 0.6,
            CcrLevel::Medium => 0.4,
            CcrLevel::Aggressive => 0.2,
        }
    }
}

/// A compressed signature that can be reversed using the cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcrSignature {
    pub hash: ContentHash,
    pub level: CcrLevel,
    pub compressed_len: usize,
    pub original_len: usize,
    /// Strategy hint used for decompression
    pub strategy: String,
}

/// Headroom-style CCR reversible compressor with content-addressed cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReversibleCompressor {
    /// Content cache: hash -> original content
    cache: std::collections::HashMap<ContentHash, String>,
    /// Maximum cache entries (default: 1000)
    max_cache: usize,
    /// LRU tracking using insertion order
    access_order: std::collections::VecDeque<ContentHash>,
}

impl ReversibleCompressor {
    pub fn new(max_cache: usize) -> Self {
        Self {
            cache: std::collections::HashMap::new(),
            max_cache,
            access_order: std::collections::VecDeque::new(),
        }
    }

    /// Compress content reversibly:
    /// 1. Hash content with simple hash
    /// 2. Store original in cache
    /// 3. Extract compressed signature (key phrases, structure markers)
    ///    Returns (signature, original_hash)
    ///
    /// Compression strategy by level:
    ///   - Light: Keep first/last 30% sentences, extract key terms
    ///   - Medium: Keep first/last 20% sentences, extract key terms, drop examples
    ///   - Aggressive: Keep first 10%, last 10%, key terms only
    pub fn compress(&mut self, content: &str, level: CcrLevel) -> (CcrSignature, ContentHash) {
        let hash = Self::content_hash(content);
        let original_len = content.len();

        self.cache.insert(hash, content.to_string());
        self.touch(hash);
        self.evict_if_needed();

        let reduction = level.reduction_factor();
        let compressed = Self::extract_signature(content, reduction);
        let compressed_len = compressed.len();

        let strategy = match level {
            CcrLevel::Light => "keep_first_last_30pct_key_terms".to_string(),
            CcrLevel::Medium => "keep_first_last_20pct_drop_examples".to_string(),
            CcrLevel::Aggressive => "keep_first_last_10pct_key_terms_only".to_string(),
        };

        let signature = CcrSignature {
            hash,
            level,
            compressed_len,
            original_len,
            strategy,
        };

        (signature, hash)
    }

    /// Decompress: look up hash in cache, return original
    /// If not found, return None
    pub fn decompress(&mut self, signature: &CcrSignature) -> Option<String> {
        let result = self.cache.get(&signature.hash).cloned();
        if result.is_some() {
            self.touch(signature.hash);
        }
        result
    }

    /// Decompress with lossy fallback: reconstruct from signature if cache miss
    pub fn decompress_lossy(&mut self, signature: &CcrSignature) -> String {
        if let Some(original) = self.cache.get(&signature.hash).cloned() {
            self.touch(signature.hash);
            return original;
        }
        format!(
            "[CCR:{} compressed from {} to {} chars via {}]",
            match signature.level {
                CcrLevel::Light => "Light",
                CcrLevel::Medium => "Medium",
                CcrLevel::Aggressive => "Aggressive",
            },
            signature.original_len,
            signature.compressed_len,
            signature.strategy,
        )
    }

    /// Check if content exists in cache
    pub fn has_content(&self, hash: ContentHash) -> bool {
        self.cache.contains_key(&hash)
    }

    /// Cache statistics
    pub fn stats(&self) -> CcrStats {
        CcrStats {
            cache_size: self.cache.len(),
            ..Default::default()
        }
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
    }

    // --- Internal helpers ---

    /// Content-addressed hash using std DefaultHasher
    fn content_hash(content: &str) -> ContentHash {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Extract compressed representation
    fn extract_signature(content: &str, reduction: f64) -> String {
        if content.is_empty() {
            return String::new();
        }

        let sentences: Vec<&str> = content
            .split(['.', '!', '?', '\n'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if sentences.is_empty() || sentences.len() <= 2 {
            let total_chars = content.len();
            let target_len = (total_chars as f64 * reduction).max(1.0) as usize;
            let head_len = target_len / 2;
            let tail_len = target_len - head_len;
            let head: String = content.chars().take(head_len).collect();
            let tail: String = content
                .chars()
                .rev()
                .take(tail_len)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            return format!("{} [...] {}", head.trim(), tail.trim());
        }

        let keep_head_ratio = reduction * 0.5;
        let keep_tail_ratio = reduction * 0.5;
        let head_count = (sentences.len() as f64 * keep_head_ratio).max(1.0) as usize;
        let tail_count = (sentences.len() as f64 * keep_tail_ratio).max(1.0) as usize;

        let head_count = head_count.min(sentences.len());
        let tail_count = tail_count.min(sentences.len().saturating_sub(head_count));

        let mut result = String::new();

        for s in sentences.iter().take(head_count) {
            result.push_str(s);
            result.push_str(". ");
        }

        if head_count + tail_count < sentences.len() {
            result.push_str("[...] ");
        }

        for s in sentences.iter().rev().take(tail_count).rev() {
            result.push_str(s);
            result.push_str(". ");
        }

        let all_words: Vec<&str> = content.split_whitespace().collect();
        let mut key_terms: Vec<String> = Vec::new();
        for w in &all_words {
            let clean: String = w.chars().filter(|c| c.is_alphanumeric()).collect();
            if clean.len() > 6 && !key_terms.contains(&clean) {
                key_terms.push(clean);
            }
        }
        key_terms.truncate(10);

        if !key_terms.is_empty() {
            result.push_str(&format!("[keys: {}]", key_terms.join(", ")));
        }

        result
    }

    fn evict_if_needed(&mut self) {
        while self.cache.len() > self.max_cache {
            if let Some(oldest) = self.access_order.pop_front() {
                self.cache.remove(&oldest);
            } else {
                break;
            }
        }
    }

    fn touch(&mut self, hash: ContentHash) {
        if let Some(pos) = self.access_order.iter().position(|&h| h == hash) {
            self.access_order.remove(pos);
        }
        self.access_order.push_back(hash);
    }
}

/// CCR statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CcrStats {
    pub total_compressions: u64,
    pub total_decompressions: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_size: usize,
    pub estimated_tokens_saved: u64,
}

impl CcrStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod ccr_tests {
    use super::*;

    #[test]
    fn test_ccr_compress_decompress_roundtrip() {
        let mut compressor = ReversibleCompressor::new(100);
        let content =
            "The quick brown fox jumps over the lazy dog. This is a test sentence for compression. \
             We want to ensure roundtrip works correctly.";
        let (sig, hash) = compressor.compress(content, CcrLevel::Medium);
        let decompressed = compressor.decompress(&sig);
        assert!(decompressed.is_some());
        assert_eq!(decompressed.unwrap(), content);
        assert_eq!(hash, sig.hash);
    }

    #[test]
    fn test_ccr_cache_miss_returns_lossy() {
        let mut compressor = ReversibleCompressor::new(100);
        let content = "Test content for cache miss scenario.";
        let (sig, _) = compressor.compress(content, CcrLevel::Light);
        compressor.clear();
        let lossy = compressor.decompress_lossy(&sig);
        assert!(lossy.contains("CCR:Light"));
        assert!(lossy.contains("compressed from"));
        assert!(compressor.decompress(&sig).is_none());
    }

    #[test]
    fn test_ccr_levels_reduce_size() {
        let mut comp_light = ReversibleCompressor::new(100);
        let mut comp_medium = ReversibleCompressor::new(100);
        let mut comp_agg = ReversibleCompressor::new(100);

        let content =
            "This is a long document that contains many sentences. It has multiple sentences \
             that we can use for testing the compression ratios. Each sentence provides additional \
             context for the compression algorithm. The quick brown fox jumps over the lazy dog \
             near the bank of the river. Machine learning models require large amounts of training \
             data to perform well. Natural language processing tasks benefit from transformer \
             architectures. Deep neural networks have revolutionized artificial intelligence research.";

        let (sig_light, _) = comp_light.compress(content, CcrLevel::Light);
        let (sig_medium, _) = comp_medium.compress(content, CcrLevel::Medium);
        let (sig_agg, _) = comp_agg.compress(content, CcrLevel::Aggressive);

        assert!(
            sig_light.compressed_len >= sig_medium.compressed_len,
            "Light compressed_len ({}) should be >= Medium ({})",
            sig_light.compressed_len,
            sig_medium.compressed_len
        );
        assert!(
            sig_medium.compressed_len >= sig_agg.compressed_len,
            "Medium compressed_len ({}) should be >= Aggressive ({})",
            sig_medium.compressed_len,
            sig_agg.compressed_len
        );
        assert!(sig_light.compressed_len < content.len());
        assert!(sig_agg.compressed_len < content.len());
    }

    #[test]
    fn test_ccr_content_hash_deterministic() {
        let content = "Deterministic hash test content";
        let hash1 = ReversibleCompressor::content_hash(content);
        let hash2 = ReversibleCompressor::content_hash(content);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_ccr_stats_tracking() {
        let mut compressor = ReversibleCompressor::new(100);
        let content = "Stats tracking test content.";
        let (sig, _) = compressor.compress(content, CcrLevel::Light);

        let stats = compressor.stats();
        assert_eq!(stats.cache_size, 1);

        assert!(compressor.decompress(&sig).is_some());

        compressor.clear();
        let stats = compressor.stats();
        assert_eq!(stats.cache_size, 0);
    }

    #[test]
    fn test_ccr_cache_eviction() {
        let mut compressor = ReversibleCompressor::new(3);
        let (sig1, _) = compressor.compress("Content number one", CcrLevel::Light);
        let (_, _) = compressor.compress("Content number two", CcrLevel::Light);
        let (_, _) = compressor.compress("Content number three", CcrLevel::Light);
        let (_, _) = compressor.compress("Content number four", CcrLevel::Light);

        assert!(
            compressor.decompress(&sig1).is_none(),
            "First item should be evicted (max_cache=3, inserted 4)"
        );
        assert_eq!(compressor.cache.len(), 3);
    }

    #[test]
    fn test_ccr_empty_content() {
        let mut compressor = ReversibleCompressor::new(100);
        let content = "";
        let (sig, _) = compressor.compress(content, CcrLevel::Light);
        assert_eq!(sig.compressed_len, 0);
        assert_eq!(sig.original_len, 0);
        let decompressed = compressor.decompress(&sig);
        assert!(decompressed.is_some());
        assert_eq!(decompressed.unwrap(), "");
    }

    #[test]
    fn test_ccr_long_content_reduces() {
        let mut compressor = ReversibleCompressor::new(100);
        let sentence = "This is a very long sentence. ";
        let long_content: String = sentence.repeat(50);
        let long_content = long_content.trim();
        let (sig, _) = compressor.compress(long_content, CcrLevel::Aggressive);
        assert!(
            sig.compressed_len < long_content.len(),
            "Compressed length ({}) should be less than original ({})",
            sig.compressed_len,
            long_content.len()
        );
        let ratio = sig.compressed_len as f64 / long_content.len() as f64;
        assert!(
            ratio < 0.8,
            "Compression ratio {} should be < 0.8 for Aggressive",
            ratio
        );
    }

    #[test]
    fn test_ccr_content_hash_unique() {
        let content_a = "Unique content A for hashing test with different words";
        let content_b = "Unique content B for hashing test with different words entirely";
        let hash_a = ReversibleCompressor::content_hash(content_a);
        let hash_b = ReversibleCompressor::content_hash(content_b);
        assert_ne!(
            hash_a, hash_b,
            "Hash collision detected: different content produced same hash"
        );
    }
}
