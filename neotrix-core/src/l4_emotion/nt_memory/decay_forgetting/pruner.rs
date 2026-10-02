#![forbid(unsafe_code)]

//! Memory pruner (R-P121).
//!
//! Computes retention for entries using access-frequency-boosted decay and
//! prunes entries that fall below a configurable minimum retention threshold.

use super::curves::ebbinghaus_retention;

/// Configuration for decay-based pruning.
#[derive(Debug, Clone, Copy)]
pub struct DecayPruneConfig {
    /// Half-life in hours for the decay curve.
    pub half_life_hours: f64,
    /// Minimum retention score; entries below this are pruned.
    pub min_retention: f64,
}

impl Default for DecayPruneConfig {
    fn default() -> Self {
        Self {
            half_life_hours: 168.0, // 7 days
            min_retention: 0.1,
        }
    }
}

/// Compute retention for an entry given its age and access count.
///
/// Access count boosts retention: each access resets part of the decay.
/// The effective half-life is extended by `log2(1 + access_count)` hours.
pub fn compute_retention(age_hours: f64, access_count: u32, config: &DecayPruneConfig) -> f64 {
    let boost = if access_count > 0 {
        (1.0 + access_count as f64).log2()
    } else {
        0.0
    };
    let effective_half_life = config.half_life_hours + boost * config.half_life_hours;
    ebbinghaus_retention(age_hours, effective_half_life)
}

/// Prune a vector of entries by retention score.
///
/// - `age_fn`: returns the age in hours for an entry.
/// - `access_fn`: returns the access count for an entry.
/// - `config`: decay configuration.
///
/// Returns the number of removed entries. Retained entries keep their original order.
pub fn prune_by_retention<T>(
    entries: &mut Vec<T>,
    age_fn: impl Fn(&T) -> f64,
    access_fn: impl Fn(&T) -> u32,
    config: &DecayPruneConfig,
) -> usize {
    let before = entries.len();
    entries.retain(|e| {
        let age = age_fn(e);
        let count = access_fn(e);
        compute_retention(age, count, config) >= config.min_retention
    });
    before - entries.len()
}

// ─── MemoryPruner（2026-09-30 补齐）──────────────────────────────────
// 同 `curves.rs` 的 trait 层：类型**全仓都不存在**，而 `config.rs` / `mod.rs`
// 都在 `use` 它 ⇒ 模块因此不可编译、734 行不参与编译。
// ⇒ 补齐并**复用本文件既有的 `compute_retention` / `prune_by_retention`**
// （不重写公式，避免两处实现漂移 —— 本会话已多次因副本漂移出问题）。

/// 剪枝模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PruneMode {
    /// 只剪掉显著性低于阈值的条目（默认）。
    #[default]
    BelowThreshold,
    /// 额外限制总条目数，按显著性从低到高裁剪。
    Capacity,
    /// 先按阈值剪，再按容量裁。
    Both,
}

/// 剪枝结果。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PruneResult {
    /// 被移除的条目数。
    pub removed: usize,
    /// 剪枝后剩余条目数。
    pub remaining: usize,
}

/// 基于显著性计算器的剪枝器。
#[derive(Debug, Clone)]
pub struct MemoryPruner {
    calculator: super::salience::SalienceCalculator,
    threshold: f64,
    /// 最大条目数；`None` = 不限容量。
    pub max_entries: Option<usize>,
    mode: PruneMode,
}

impl MemoryPruner {
    /// `threshold`：显著性低于此值的条目被剪掉。
    pub fn new(calculator: super::salience::SalienceCalculator, threshold: f64) -> Self {
        Self { calculator, threshold, max_entries: None, mode: PruneMode::default() }
    }

    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = Some(max);
        self
    }

    pub fn with_mode(mut self, mode: PruneMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    /// 判定哪些条目**应被剪掉**，返回它们的**下标**（升序）。
    ///
    /// ⚠️ 返回下标而非 `Vec<E>`：首版要求 `E: Clone` 并克隆存活条目，
    /// 但 `prune_in_place` 的调用方拿不到这个约束 ⇒ 编译失败。
    /// ⇒ 改为**只收集下标**，完全不要求 `Clone`，
    /// 也让调用方能自选 `retain` / `swap_remove` 等任意移除策略。
    pub fn select_prunable_indices<E: super::salience::SalienceEntry>(
        &self,
        entries: &[E],
    ) -> Vec<usize> {
        entries
            .iter()
            .enumerate()
            .filter(|(_, e)| self.calculator.score(*e) < self.threshold)
            .map(|(i, _)| i)
            .collect()
    }

    /// 就地剪枝 `Vec`，返回 [`PruneResult`]。
    ///
    /// 流程：先按显著性阈值剔除（`BelowThreshold`），
    /// 再按容量裁剪（`Capacity` / `Both`）。
    /// 全程用 `retain` + **稳定**排序，**不要求 `E: Clone`**。
    pub fn prune_in_place<E: super::salience::SalienceEntry>(
        &self,
        entries: &mut Vec<E>,
    ) -> PruneResult {
        let before = entries.len();

        if matches!(self.mode, PruneMode::BelowThreshold | PruneMode::Both) {
            let threshold = self.threshold;
            let calc = &self.calculator;
            entries.retain(|e| calc.score(e) >= threshold);
        }

        if matches!(self.mode, PruneMode::Capacity | PruneMode::Both) {
            if let Some(cap) = self.max_entries {
                if entries.len() > cap {
                    let calc = &self.calculator;
                    // 稳定排序：分数**低**者在前；同分保持原顺序
                    //（`sort_by` 是稳定排序 ⇒ 同分相对次序不变）。
                    entries.sort_by(|a, b| {
                        calc.score(a)
                            .partial_cmp(&calc.score(b))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    entries.truncate(cap);
                }
            }
        }

        PruneResult { removed: before - entries.len(), remaining: entries.len() }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_retention_fresh_entry_high() {
        let config = DecayPruneConfig::default();
        let r = compute_retention(0.0, 0, &config);
        assert!((r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn compute_retention_old_entry_low() {
        let config = DecayPruneConfig {
            half_life_hours: 24.0,
            min_retention: 0.0,
        };
        let r = compute_retention(720.0, 0, &config); // 30 days, no access
        assert!(r < 0.1, "retention = {r}");
    }

    #[test]
    fn access_count_boosts_retention() {
        let config = DecayPruneConfig {
            half_life_hours: 24.0,
            min_retention: 0.0,
        };
        let r_no_access = compute_retention(48.0, 0, &config);
        let r_many_access = compute_retention(48.0, 10, &config);
        assert!(r_many_access > r_no_access);
    }

    #[test]
    fn prune_by_retention_removes_old() {
        let config = DecayPruneConfig {
            half_life_hours: 24.0,
            min_retention: 0.3,
        };
        let mut entries = vec![
            ("fresh", 0.0, 0u32),
            ("old", 200.0, 0),
            ("accessed_old", 200.0, 5),
        ];
        let removed = prune_by_retention(&mut entries, |e| e.1, |e| e.2, &config);
        assert!(removed >= 1);
        assert!(entries.iter().any(|e| e.0 == "fresh"));
    }

    #[test]
    fn prune_by_retention_preserves_order() {
        let config = DecayPruneConfig {
            half_life_hours: 720.0,
            min_retention: 0.0,
        };
        let mut entries: Vec<(u32, f64)> = vec![(1, 0.0), (2, 10.0), (3, 20.0)];
        let removed = prune_by_retention(&mut entries, |e| e.1, |_| 0, &config);
        assert_eq!(removed, 0);
        assert_eq!(entries, vec![(1, 0.0), (2, 10.0), (3, 20.0)]);
    }

    #[test]
    fn prune_empty_vec() {
        let config = DecayPruneConfig::default();
        let mut entries: Vec<(f64, u32)> = vec![];
        let removed = prune_by_retention(&mut entries, |e| e.0, |e| e.1, &config);
        assert_eq!(removed, 0);
        assert!(entries.is_empty());
    }

    #[test]
    fn default_config_reasonable() {
        let c = DecayPruneConfig::default();
        assert!(c.half_life_hours > 0.0);
        assert!((0.0..=1.0).contains(&c.min_retention));
    }

    #[test]
    fn compute_retention_zero_age_zero_access() {
        let config = DecayPruneConfig::default();
        let r = compute_retention(0.0, 0, &config);
        assert!((r - 1.0).abs() < 1e-6);
    }

    #[test]
    fn prune_all_old_entries() {
        let config = DecayPruneConfig {
            half_life_hours: 1.0,
            min_retention: 0.99,
        };
        let mut entries = vec![("old1", 100.0, 0u32), ("old2", 200.0, 0)];
        let removed = prune_by_retention(&mut entries, |e| e.1, |e| e.2, &config);
        assert_eq!(removed, 2);
        assert!(entries.is_empty());
    }
}
