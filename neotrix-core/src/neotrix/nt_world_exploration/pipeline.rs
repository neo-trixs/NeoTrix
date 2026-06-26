use std::collections::HashSet;
use std::hash::Hash;

const MAX_SEEN_CONTENT: usize = 10_000;

use super::content::{NegentropyScore, SourceContent};

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

/// 通用内容指纹 — 来源无关的去重单元
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ContentFingerprint {
    pub id: String,
    pub text_hash: u64,
    pub prefix_hash: u64,
}

impl ContentFingerprint {
    pub fn from_content(c: &SourceContent) -> Self {
        let text_hash = simple_hash(&c.text);
        let prefix = c.text.chars().take(60).collect::<String>();
        let prefix_hash = simple_hash(&prefix);
        Self {
            id: c.id.clone(),
            text_hash,
            prefix_hash,
        }
    }
}

/// 通用负熵管线 — 与来源类型无关
///
/// 负责:
/// - 去重 (id + text_hash)
/// - 信号纯度 (密度 / 噪声 / 互动信号)
/// - 信息增益 (长度 × 纯度 × 已知概念重叠)
/// - 综合负熵评分
#[derive(Debug)]
pub struct NegentropyPipeline {
    pub seen: HashSet<ContentFingerprint>,
    seen_prefixes: HashSet<u64>,
    pub total_seen: usize,
    pub absorbed_count: usize,
}

impl Default for NegentropyPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl NegentropyPipeline {
    pub fn new() -> Self {
        Self {
            seen: HashSet::with_capacity(4096),
            seen_prefixes: HashSet::with_capacity(4096),
            total_seen: 0,
            absorbed_count: 0,
        }
    }

    pub fn is_novel(&self, content: &SourceContent) -> bool {
        let fp = ContentFingerprint::from_content(content);
        if self.seen.contains(&fp) {
            return false;
        }
        let prefix = simple_hash(&content.text.chars().take(60).collect::<String>());
        if self.seen_prefixes.contains(&prefix) {
            return false;
        }
        true
    }

    pub fn mark_seen(&mut self, content: &SourceContent) {
        if self.seen.len() >= MAX_SEEN_CONTENT {
            let drain_count = self.seen.len() / 2;
            let to_remove: Vec<ContentFingerprint> =
                self.seen.iter().take(drain_count).cloned().collect();
            for fp in &to_remove {
                self.seen.remove(fp);
            }
            self.seen_prefixes.clear();
        }
        let fp = ContentFingerprint::from_content(content);
        self.seen.insert(fp);
        let prefix = simple_hash(&content.text.chars().take(60).collect::<String>());
        self.seen_prefixes.insert(prefix);
        self.total_seen += 1;
    }

    pub fn signal_purity(&self, content: &SourceContent) -> f64 {
        let text = content.text.trim();
        if text.len() < 10 {
            return 0.05;
        }
        let word_count = text.split_whitespace().count() as f64;
        let char_count = text.len() as f64;
        let avg_word_len = char_count / word_count.max(1.0);
        let has_url = text.contains("http");
        let mention_ratio = text.matches('@').count() as f64 / word_count.max(1.0);
        let hash_ratio = text.matches('#').count() as f64 / word_count.max(1.0);

        let mut purity = 1.0;
        purity -= mention_ratio.min(0.5) * 0.3;
        purity -= hash_ratio.min(0.5) * 0.2;
        purity += (avg_word_len / 8.0).min(0.2);
        if has_url {
            purity -= 0.1;
        }
        if text.len() < 30 {
            purity -= 0.2;
        }
        if content.engagement.likes < 1 && content.engagement.shares < 1 {
            purity -= 0.1;
        }
        purity.clamp(0.0, 1.0)
    }

    pub fn information_gain(&self, content: &SourceContent, known_concepts: &[String]) -> f64 {
        let text = content.text.trim();
        if text.is_empty() {
            return 0.0;
        }
        let purity = self.signal_purity(content);
        let length_factor = (text.len() as f64 / 500.0).min(1.0);
        let interaction_boost = ((content.engagement.likes as f64 * 0.01)
            + (content.engagement.shares as f64 * 0.03))
            .min(0.3);

        let known_overlap = if known_concepts.is_empty() {
            0.0
        } else {
            let text_lower = text.to_lowercase();
            let matches = known_concepts
                .iter()
                .filter(|c| text_lower.contains(&c.to_lowercase()))
                .count();
            (matches as f64 / known_concepts.len() as f64).min(0.8)
        };

        let gain = length_factor * purity * (1.0 - known_overlap) + interaction_boost;
        gain.clamp(0.0, 1.0)
    }

    pub fn score(
        &self,
        content: &SourceContent,
        known_concepts: &[String],
        curiosity_bonus: f64,
    ) -> NegentropyScore {
        let info_gain = self.information_gain(content, known_concepts);
        let purity = self.signal_purity(content);
        let novelty = if self.total_seen == 0 {
            1.0
        } else {
            1.0 - (self.total_seen as f64 / 1000.0).min(0.5)
        };
        let relevance = (info_gain * 0.5 + purity * 0.3 + curiosity_bonus * 0.2).min(1.0);
        let negentropy = info_gain * (0.4 + 0.3 * novelty + 0.2 * relevance + 0.1 * purity);

        NegentropyScore {
            content: content.clone(),
            information_gain: info_gain,
            novelty,
            relevance,
            signal_purity: purity,
            negentropy,
        }
    }

    /// 批量处理: 去重 → 评分 → 排序 → 返回值得吸收的
    pub fn process_batch(
        &mut self,
        items: &[SourceContent],
        known_concepts: &[String],
        curiosity_bonus: f64,
        max_absorb: usize,
    ) -> Vec<NegentropyScore> {
        let mut scored: Vec<NegentropyScore> = Vec::new();

        for item in items {
            if !self.is_novel(item) {
                continue;
            }
            self.mark_seen(item);
            let score = self.score(item, known_concepts, curiosity_bonus);
            if score.is_worth_absorbing() {
                scored.push(score);
            }
        }

        scored.sort_by(|a, b| {
            b.negentropy
                .partial_cmp(&a.negentropy)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(max_absorb);

        self.absorbed_count += scored.len();
        scored
    }

    pub fn extract_keywords(text: &str) -> Vec<String> {
        let mut words: Vec<String> = text
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .map(|w| {
                w.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase()
            })
            .filter(|w| w.len() > 2 && !STOP_WORDS.contains(&w.as_str()))
            .collect();
        words.sort();
        words.dedup();
        words.truncate(20);
        words
    }
}

const STOP_WORDS: &[&str] = &[
    "the", "this", "that", "with", "from", "have", "been", "were", "they", "what", "when", "where",
    "which", "their", "there", "about", "would", "could", "should", "into", "over", "such", "only",
    "than", "then", "also", "just", "more", "some", "them", "very", "well", "here", "like", "much",
    "will", "made", "your", "its", "our", "has",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_world_exploration::content::ExplorationSourceType;

    fn sample(id: &str, text: &str) -> SourceContent {
        SourceContent::new(id, text, ExplorationSourceType::BrowserSocial)
    }

    #[test]
    fn test_novelty_and_dedup() {
        let mut p = NegentropyPipeline::new();
        let a = sample("1", "hello world");
        assert!(p.is_novel(&a));
        p.mark_seen(&a);
        assert!(!p.is_novel(&a));
    }

    #[test]
    fn test_signal_purity_short() {
        let p = NegentropyPipeline::new();
        let a = sample("1", "hi");
        assert!(p.signal_purity(&a) < 0.1);
    }

    #[test]
    fn test_score_zero_known() {
        let p = NegentropyPipeline::new();
        let a = sample(
            "1",
            "this is a reasonably long test sentence with enough words to score well",
        );
        let s = p.score(&a, &[], 0.0);
        assert!(s.negentropy > 0.0);
    }

    #[test]
    fn test_process_batch_filters_low() {
        let mut p = NegentropyPipeline::new();
        let items = vec![
            sample("1", "a"),
            sample(
                "2",
                "this is a real tweet with enough content for absorption testing purposes here",
            ),
        ];
        let results = p.process_batch(&items, &[], 0.3, 5);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_extract_keywords() {
        let kw = NegentropyPipeline::extract_keywords(
            "this is a test about artificial intelligence and machine learning",
        );
        assert!(kw.contains(&"artificial".to_string()));
    }
}
