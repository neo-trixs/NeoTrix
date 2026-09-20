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
