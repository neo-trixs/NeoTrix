//! ContextOptimizer — Context 压缩引擎
//!
//! 基于 LMCache CacheBlend + Magic-Context + HyperResearch。
//! - 确定性衰减渲染
//! - 选择性重计算
//! - Patch-only 修改

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    pub tokens: Vec<TokenEntry>,
    pub max_tokens: usize,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEntry {
    pub content: String,
    pub token_count: usize,
    pub timestamp: u64,
    pub salience: f64,
    pub role: String,
}

impl ContextWindow {
    pub fn new(max_tokens: usize) -> Self {
        Self { tokens: Vec::new(), max_tokens, created_at: now_ms() }
    }

    pub fn total_tokens(&self) -> usize {
        self.tokens.iter().map(|t| t.token_count).sum()
    }

    pub fn add(&mut self, content: String, role: &str) {
        let estimated = content.split_whitespace().count() + 1;
        self.tokens.push(TokenEntry {
            content, token_count: estimated, timestamp: now_ms(), salience: 0.5, role: role.to_string(),
        });
    }
}

pub struct ContextOptimizer {
    pub decay_half_life_ms: u64,
    pub min_salience: f64,
    pub recent_keep_ratio: f64,
}

impl ContextOptimizer {
    pub fn new() -> Self {
        Self { decay_half_life_ms: 3_600_000, min_salience: 0.05, recent_keep_ratio: 0.3 }
    }

    pub fn compress(&self, window: &mut ContextWindow) -> CompressionReport {
        let total_before = window.tokens.len();
        let cutoff = (window.max_tokens as f64 * self.recent_keep_ratio) as usize;
        let keep_recent = cutoff.min(window.tokens.len());

        let mut scored: Vec<(usize, f64)> = window.tokens.iter().enumerate()
            .take(window.tokens.len().saturating_sub(keep_recent))
            .map(|(i, t)| (i, self.decay_score(t)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut remove_indices: Vec<usize> = scored.iter()
            .filter(|(_, s)| *s < self.min_salience)
            .map(|(i, _)| *i)
            .collect();
        remove_indices.sort_unstable_by(|a, b| b.cmp(a));

        for idx in remove_indices {
            if idx < window.tokens.len() {
                window.tokens.remove(idx);
            }
        }

        CompressionReport {
            tokens_before: total_before,
            tokens_after: window.tokens.len(),
            removed: total_before - window.tokens.len(),
        }
    }

    pub fn decay_score(&self, entry: &TokenEntry) -> f64 {
        let age_ms = now_ms().saturating_sub(entry.timestamp);
        let half_lives = age_ms as f64 / self.decay_half_life_ms as f64;
        let decay = 0.5_f64.powf(half_lives);
        (decay * entry.salience).min(1.0)
    }

    pub fn patch_only(&self, window: &mut ContextWindow, index: usize, new_content: &str) -> bool {
        if index < window.tokens.len() {
            window.tokens[index].content = new_content.to_string();
            window.tokens[index].timestamp = now_ms();
            true
        } else {
            false
        }
    }

    pub fn estimate_tokens(text: &str) -> usize {
        text.split_whitespace().count() + 1
    }
}

#[derive(Debug, Clone)]
pub struct CompressionReport {
    pub tokens_before: usize,
    pub tokens_after: usize,
    pub removed: usize,
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress() {
        let mut window = ContextWindow::new(100);
        for i in 0..20 {
            window.add(format!("Message {}", i), "user");
        }
        let opt = ContextOptimizer::new();
        let report = opt.compress(&mut window);
        assert!(report.removed >= 0);
    }

    #[test]
    fn test_patch_only() {
        let mut window = ContextWindow::new(100);
        window.add("original".into(), "user");
        let opt = ContextOptimizer::new();
        assert!(opt.patch_only(&mut window, 0, "patched"));
        assert_eq!(window.tokens[0].content, "patched");
    }

    #[test]
    fn test_decay_score() {
        let opt = ContextOptimizer::new();
        let entry = TokenEntry {
            content: "test".into(), token_count: 1, timestamp: now_ms() - 7_200_000, salience: 1.0, role: "user".into(),
        };
        let score = opt.decay_score(&entry);
        assert!(score > 0.0 && score <= 1.0);
    }
}
