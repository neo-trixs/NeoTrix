//! AttentionManager — allocates attention budget across components based on salience.
//!
//! Given a set of salience scores and a total budget, distributes attention proportionally.
//! Higher-salience components receive more budget. Implements R-P123 (decompose by domain).

/// Attention allocation for a single component.
#[derive(Debug, Clone, PartialEq)]
pub struct Allocation {
    /// Domain/component identifier.
    pub domain: String,
    /// Allocated budget share.
    pub allocated: f64,
    /// Original salience score.
    pub salience: f64,
    /// Fraction of total budget [0,1].
    pub fraction: f64,
}

/// AttentionManager — budget allocator for GWT routing.
pub struct AttentionManager {
    /// Minimum allocation per component (prevents starvation).
    pub min_allocation: f64,
    /// Maximum fraction any single component can receive.
    pub max_fraction: f64,
}

impl AttentionManager {
    pub fn new() -> Self {
        Self {
            min_allocation: 0.01,
            max_fraction: 0.8,
        }
    }

    /// Allocate attention budget across components based on salience scores.
    ///
    /// Algorithm:
    /// 1. Normalize salience scores to get raw fractions.
    /// 2. Enforce max_fraction cap.
    /// 3. Enforce min_allocation floor.
    /// 4. Re-normalize to fit within total_budget.
    pub fn allocate(
        &self,
        salience_scores: &[(String, f64)],
        total_budget: f64,
    ) -> Vec<Allocation> {
        if salience_scores.is_empty() {
            return Vec::new();
        }

        let total_salience: f64 = salience_scores.iter().map(|(_, s)| s).sum();
        if total_salience <= 0.0 {
            let equal_share = total_budget / salience_scores.len() as f64;
            return salience_scores
                .iter()
                .map(|(domain, salience)| Allocation {
                    domain: domain.clone(),
                    allocated: equal_share,
                    salience: *salience,
                    fraction: 1.0 / salience_scores.len() as f64,
                })
                .collect();
        }

        // Step 1: raw fractions
        let mut fractions: Vec<(String, f64, f64)> = salience_scores
            .iter()
            .map(|(d, s)| (d.clone(), s / total_salience, *s))
            .collect();

        // Step 2: enforce max_fraction cap
        let max_cap = self.max_fraction;
        for (_, frac, _) in &mut fractions {
            if *frac > max_cap {
                *frac = max_cap;
            }
        }

        // Step 3: enforce min_allocation floor
        let min_frac = self.min_allocation / total_budget;
        for (_, frac, _) in &mut fractions {
            if *frac < min_frac {
                *frac = min_frac;
            }
        }

        // Step 4: re-normalize
        let frac_sum: f64 = fractions.iter().map(|(_, f, _)| f).sum();
        if frac_sum > 0.0 {
            for (_, frac, _) in &mut fractions {
                *frac /= frac_sum;
            }
        }

        fractions
            .into_iter()
            .map(|(domain, frac, salience)| Allocation {
                allocated: frac * total_budget,
                fraction: frac,
                domain,
                salience,
            })
            .collect()
    }

    /// Reallocate attention — subtract used budget and redistribute remainder.
    pub fn reallocate(
        &self,
        salience_scores: &[(String, f64)],
        total_budget: f64,
        used_budget: f64,
    ) -> Vec<Allocation> {
        let remaining = (total_budget - used_budget).max(0.0);
        self.allocate(salience_scores, remaining)
    }

    /// Get the domain with highest allocated attention.
    pub fn top_domain(allocations: &[Allocation]) -> Option<&Allocation> {
        allocations.iter().max_by(|a, b| {
            a.allocated
                .partial_cmp(&b.allocated)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

impl Default for AttentionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_proportional() {
        let mgr = AttentionManager::new();
        let scores = vec![
            ("a".into(), 0.8),
            ("b".into(), 0.2),
        ];
        let allocs = mgr.allocate(&scores, 1000.0);
        assert_eq!(allocs.len(), 2);
        let a = allocs.iter().find(|a| a.domain == "a").unwrap();
        let b = allocs.iter().find(|a| a.domain == "b").unwrap();
        assert!(a.allocated > b.allocated);
        assert!((a.allocated + b.allocated - 1000.0).abs() < 0.01);
    }

    #[test]
    fn allocate_enforces_min() {
        let mgr = AttentionManager {
            min_allocation: 100.0,
            max_fraction: 0.8,
        };
        let scores = vec![
            ("a".into(), 0.99),
            ("b".into(), 0.01),
        ];
        let allocs = mgr.allocate(&scores, 1000.0);
        let b = allocs.iter().find(|a| a.domain == "b").unwrap();
        assert!(b.allocated >= 100.0);
    }

    #[test]
    fn allocate_enforces_max() {
        let mgr = AttentionManager {
            min_allocation: 0.01,
            max_fraction: 0.6,
        };
        let scores = vec![
            ("a".into(), 0.9),
            ("b".into(), 0.1),
        ];
        let allocs = mgr.allocate(&scores, 1000.0);
        let a = allocs.iter().find(|a| a.domain == "a").unwrap();
        assert!(a.allocated <= 600.0);
    }

    #[test]
    fn allocate_equal_scores() {
        let mgr = AttentionManager::new();
        let scores = vec![
            ("a".into(), 0.5),
            ("b".into(), 0.5),
            ("c".into(), 0.5),
        ];
        let allocs = mgr.allocate(&scores, 900.0);
        for a in &allocs {
            assert!((a.allocated - 300.0).abs() < 0.01);
        }
    }

    #[test]
    fn reallocate_subtracts_used() {
        let mgr = AttentionManager::new();
        let scores = vec![("a".into(), 1.0)];
        let allocs = mgr.reallocate(&scores, 1000.0, 500.0);
        assert!((allocs[0].allocated - 500.0).abs() < 0.01);
    }

    #[test]
    fn top_domain_returns_highest() {
        let allocs = vec![
            Allocation {
                domain: "a".into(),
                allocated: 300.0,
                salience: 0.3,
                fraction: 0.3,
            },
            Allocation {
                domain: "b".into(),
                allocated: 700.0,
                salience: 0.7,
                fraction: 0.7,
            },
        ];
        assert_eq!(AttentionManager::top_domain(&allocs).unwrap().domain, "b");
    }

    #[test]
    fn empty_scores_returns_empty() {
        let mgr = AttentionManager::new();
        assert!(mgr.allocate(&[], 1000.0).is_empty());
    }
}
