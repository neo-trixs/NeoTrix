#![forbid(unsafe_code)]

//! Salience calculator (R-P121).
//!
//! Combines three signals into a single salience score [0.0, 1.0]:
//! - **Access frequency**: how often the entry was touched
//! - **Recency**: time since last access (via decay curve)
//! - **Importance**: domain-assigned importance weight [0.0, 1.0]

use super::curves::{DecayCurve, ExponentialDecay};
use crate::l4_emotion::nt_memory::shared_utils::now_ts;

/// A memory entry viewed through the salience lens.
pub trait SalienceEntry {
    /// Last access timestamp (Unix seconds).
    fn last_accessed(&self) -> i64;

    /// Number of times this entry was accessed.
    fn access_count(&self) -> u64;

    /// Domain-assigned importance in [0.0, 1.0].
    fn importance(&self) -> f64;
}

/// Weighting knobs for combining signals.
#[derive(Debug, Clone, Copy)]
pub struct SalienceWeights {
    pub access_freq: f64,
    pub recency: f64,
    pub importance: f64,
}

impl Default for SalienceWeights {
    fn default() -> Self {
        Self {
            access_freq: 0.3,
            recency: 0.4,
            importance: 0.3,
        }
    }
}

/// Computes a composite salience score for memory entries.
// 2026-09-30 加 derive：`MemoryPruner` 需持有本类型并要求 Debug/Clone。
// 字段 `decay`/`weights` 均已 `Debug+Clone+Copy`，`max_access_count` 是 f64 ⇒ 派生成立。
#[derive(Debug, Clone)]
pub struct SalienceCalculator {
    pub decay: ExponentialDecay,
    pub weights: SalienceWeights,
    /// Normalization factor for access count: score = min(count/max, 1.0).
    ///
    /// ⛔ 2026-10-03 由 `pub` 收为**私有**。这不是风格问题，是**堵死一条绕过守卫的路径**。
    ///
    /// 【为什么必须有守卫】`:76` 是 `(access_count / max_access_count).min(1.0)`。
    /// 分母为 0 时：`acc > 0` ⇒ `inf.min(1.0)` = 1.0；`acc == 0` ⇒ `NaN.min(1.0)` = **1.0**
    /// （Rust 的 `f64::min` 返回非 NaN 那个操作数 ⇒ NaN 被**静默**吃掉）。
    /// ⇒ 两种情况**都给出 1.0** ⇒ `access_factor` 对所有条目**恒为 1.0**
    /// ⇒ 该维度**完全失去区分能力**，且调用处**收不到任何信号**。
    ///
    /// 【守卫已存在，但曾可被绕过】`with_max_access_count`（`:63`）有 `.max(1.0)`，
    /// 配置文件路径（`config.rs:80`）也**走这个 setter**，默认值 `100.0`。
    /// ⇒ 唯一绕过方式是**直接给字段赋值**（此前字段是 `pub`）。
    ///
    /// 【实测依据】`rg -ln 'max_access_count'` 全仓 4 个文件命中，但**逐条看**：
    /// · `salience.rs` 本文件内 6 处（声明/默认值/setter/除法/注释/测试）
    /// · `config.rs:34` 是**另一个结构体**的 serde 字段，`:80` 经 setter 传入 ⇒ 已守住
    /// · `compressor.rs:11` 只是注释里的公式文字
    /// · `kv_cache_optimizer.rs:320` 只是注释里的交叉引用
    /// ⇒ **本字段零外部读写** ⇒ 收私有不破坏任何调用方。
    ///
    /// 与 `nt_io_inference/kv_cache_optimizer.rs:312` 的 `sparsity_threshold`
    /// 属**同一类系统性问题**：守卫写在 setter，字段却声明为 `pub`。
    max_access_count: f64,
}

impl SalienceCalculator {
    pub fn new(decay: ExponentialDecay, weights: SalienceWeights) -> Self {
        Self {
            decay,
            weights,
            max_access_count: 100.0,
        }
    }

    pub fn with_max_access_count(mut self, max: f64) -> Self {
        self.max_access_count = max.max(1.0);
        self
    }

    /// Compute salience score for a single entry.
    pub fn score<E: SalienceEntry>(&self, entry: &E) -> f64 {
        let now = now_ts();
        let age_days = ((now - entry.last_accessed()) as f64 / 86_400.0).max(0.0);

        let recency_factor = self.decay.factor(age_days);

        let access_factor =
            (entry.access_count() as f64 / self.max_access_count).min(1.0);

        let importance_factor = entry.importance().clamp(0.0, 1.0);

        let w = &self.weights;
        let total_weight = w.access_freq + w.recency + w.importance;
        if total_weight <= 0.0 {
            return 0.0;
        }

        (access_factor * w.access_freq
            + recency_factor * w.recency
            + importance_factor * w.importance)
            / total_weight
    }

    /// Score multiple entries, returning (index, score) pairs sorted descending.
    pub fn score_all<'a, E: SalienceEntry>(
        &self,
        entries: &'a [E],
    ) -> Vec<(usize, f64)> {
        let mut scored: Vec<(usize, f64)> = entries
            .iter()
            .enumerate()
            .map(|(i, e)| (i, self.score(e)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }
}

impl Default for SalienceCalculator {
    fn default() -> Self {
        Self::new(ExponentialDecay::default(), SalienceWeights::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct MockEntry {
        last_accessed: i64,
        access_count: u64,
        importance: f64,
    }

    impl SalienceEntry for MockEntry {
        fn last_accessed(&self) -> i64 {
            self.last_accessed
        }
        fn access_count(&self) -> u64 {
            self.access_count
        }
        fn importance(&self) -> f64 {
            self.importance
        }
    }

    fn now() -> i64 {
        now_ts()
    }

    #[test]
    fn fresh_high_importance_scores_high() {
        let calc = SalienceCalculator::default();
        let entry = MockEntry {
            last_accessed: now(),
            access_count: 50,
            importance: 1.0,
        };
        let score = calc.score(&entry);
        assert!(score > 0.7, "score={score}");
    }

    #[test]
    fn old_low_access_scores_low() {
        let calc = SalienceCalculator::default();
        let entry = MockEntry {
            last_accessed: now() - 86_400 * 365, // 1 year ago
            access_count: 0,
            importance: 0.0,
        };
        let score = calc.score(&entry);
        assert!(score < 0.2, "score={score}");
    }

    #[test]
    fn recency_dominates_when_weighted_heavily() {
        let weights = SalienceWeights {
            access_freq: 0.1,
            recency: 0.8,
            importance: 0.1,
        };
        let calc = SalienceCalculator::new(ExponentialDecay::default(), weights);

        let fresh = MockEntry {
            last_accessed: now(),
            access_count: 1,
            importance: 0.0,
        };
        let old = MockEntry {
            last_accessed: now() - 86_400 * 60,
            access_count: 100,
            importance: 1.0,
        };

        assert!(calc.score(&fresh) > calc.score(&old));
    }

    #[test]
    fn score_all_returns_sorted_descending() {
        let calc = SalienceCalculator::default();
        let entries = vec![
            MockEntry { last_accessed: now() - 86_400 * 100, access_count: 1, importance: 0.1 },
            MockEntry { last_accessed: now(), access_count: 50, importance: 0.9 },
            MockEntry { last_accessed: now() - 86_400 * 30, access_count: 10, importance: 0.5 },
        ];
        let scored = calc.score_all(&entries);
        assert_eq!(scored.len(), 3);
        // Most recent should be first
        assert_eq!(scored[0].0, 1);
    }

    #[test]
    fn zero_weights_returns_zero() {
        let calc = SalienceCalculator::new(
            ExponentialDecay::default(),
            SalienceWeights {
                access_freq: 0.0,
                recency: 0.0,
                importance: 0.0,
            },
        );
        let entry = MockEntry {
            last_accessed: now(),
            access_count: 100,
            importance: 1.0,
        };
        assert_eq!(calc.score(&entry), 0.0);
    }

    #[test]
    fn high_access_count_caps_at_one() {
        let calc = SalienceCalculator::default();
        let entry = MockEntry {
            last_accessed: now(),
            access_count: 10_000,
            importance: 0.0,
        };
        let score = calc.score(&entry);
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn with_max_access_count_custom() {
        let calc = SalienceCalculator::default().with_max_access_count(10.0);
        let entry = MockEntry {
            last_accessed: now(),
            access_count: 5,
            importance: 0.0,
        };
        let score = calc.score(&entry);
        assert!(score > 0.0);
    }

    #[test]
    fn importance_clamped_to_unit() {
        let calc = SalienceCalculator::default();
        let entry = MockEntry {
            last_accessed: now(),
            access_count: 0,
            importance: 5.0,
        };
        let score = calc.score(&entry);
        assert!(score <= 1.0);
    }

    #[test]
    fn score_all_empty_slice() {
        let calc = SalienceCalculator::default();
        let entries: Vec<MockEntry> = vec![];
        let scored = calc.score_all(&entries);
        assert!(scored.is_empty());
    }
}
