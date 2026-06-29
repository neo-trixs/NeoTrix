/// ParetoFrontSelector — Multi-objective Pareto front selection for co-evolution.
///
/// NSGA-II inspired: fast non-dominated sorting + crowding distance.
/// Supports multi-objective selection for dual-population co-evolution.
///
/// Reference: Deb et al., "A Fast and Elitist Multiobjective Genetic Algorithm: NSGA-II", 2002.
/// Applied in GEPA (ICLR 2026 Oral) and DGM-H (Meta, arXiv 2603.19461).
use std::cmp::Ordering;

/// A single objective dimension for Pareto comparison.
#[derive(Debug, Clone)]
pub struct Objective {
    pub name: String,
    pub weight: f64,
    pub minimize: bool,
}

/// A candidate solution with scores across multiple objectives.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub id: u64,
    pub description: String,
    pub scores: Vec<(String, f64)>,
    pub metadata: Vec<(String, String)>,
}

/// One point on the Pareto front — a non-dominated candidate with crowd distance.
#[derive(Debug, Clone)]
pub struct ParetoPoint {
    pub candidate: Candidate,
    pub rank: usize,
    pub crowd_distance: f64,
    pub dominates_count: usize,
}

/// Result of a Pareto selection operation.
#[derive(Debug, Clone)]
pub struct ParetoSelectionResult {
    pub front: Vec<ParetoPoint>,
    pub front_size: usize,
    pub total_candidates: usize,
    pub num_fronts: usize,
    pub selected_ids: Vec<u64>,
}

/// Multi-objective Pareto front selector for evolution.
///
/// NSGA-II inspired: fast non-dominated sorting + crowding distance.
/// Supports multi-objective selection for co-evolution populations.
///
/// Reference: Deb et al., "A Fast and Elitist Multiobjective Genetic Algorithm: NSGA-II", 2002.
/// Applied in GEPA (ICLR 2026 Oral) and DGM-H (Meta, arXiv 2603.19461).
pub struct ParetoFrontSelector {
    pub objectives: Vec<Objective>,
    pub epsilon: f64,
}

impl ParetoFrontSelector {
    /// Creates a new selector with the given objectives.
    pub fn new(objectives: Vec<Objective>) -> Self {
        Self {
            objectives,
            epsilon: 1e-10,
        }
    }

    /// Appends an objective dimension.
    pub fn add_objective(&mut self, name: &str, weight: f64, minimize: bool) {
        self.objectives.push(Objective {
            name: name.to_string(),
            weight,
            minimize,
        });
    }

    /// Returns true if `a` Pareto-dominates `b`:
    /// `a` is at least as good as `b` in all objectives, and strictly better in at least one.
    pub fn dominates(&self, a: &Candidate, b: &Candidate) -> bool {
        let mut strictly_better = false;
        for (i, obj) in self.objectives.iter().enumerate() {
            let score_a = a.scores[i].1;
            let score_b = b.scores[i].1;
            let diff = score_a - score_b;

            let diff_sign = if obj.minimize { -diff } else { diff };
            // diff_sign > 0 means a is better in this objective

            if diff_sign < -self.epsilon {
                // a is worse (outside tolerance)
                return false;
            }
            if diff_sign > self.epsilon {
                strictly_better = true;
            }
        }
        strictly_better
    }

    /// Fast non-dominated sorting (NSGA-II).
    /// Returns fronts, where each front is a `Vec<usize>` of indices into `candidates`.
    pub fn fast_non_dominated_sort(&self, candidates: &[Candidate]) -> Vec<Vec<usize>> {
        let n = candidates.len();
        if n == 0 {
            return vec![];
        }

        let mut domination_count = vec![0usize; n];
        let mut dominated_sets: Vec<Vec<usize>> = vec![vec![]; n];

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                if self.dominates(&candidates[i], &candidates[j]) {
                    dominated_sets[i].push(j);
                } else if self.dominates(&candidates[j], &candidates[i]) {
                    domination_count[i] += 1;
                }
            }
        }

        let mut fronts: Vec<Vec<usize>> = vec![];
        let mut current: Vec<usize> = (0..n).filter(|&i| domination_count[i] == 0).collect();
        if !current.is_empty() {
            fronts.push(current.clone());
        }

        while !current.is_empty() {
            let mut next = vec![];
            for &i in &current {
                for &j in &dominated_sets[i] {
                    domination_count[j] = domination_count[j].saturating_sub(1);
                    if domination_count[j] == 0 {
                        next.push(j);
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            fronts.push(next.clone());
            current = next;
        }

        fronts
    }

    /// Computes crowding distance for each point in a front.
    /// Boundary points get infinite distance; interior points are measured
    /// by normalized Manhattan distance to their neighbors across all objectives.
    pub fn crowding_distance(&self, front: &[usize], candidates: &[Candidate]) -> Vec<f64> {
        let m = front.len();
        if m == 0 {
            return vec![];
        }
        let obj_count = self.objectives.len();
        let mut distances = vec![0.0; m];

        for obj_idx in 0..obj_count {
            let mut sorted: Vec<(usize, f64)> = front
                .iter()
                .enumerate()
                .map(|(local_idx, &cidx)| (local_idx, candidates[cidx].scores[obj_idx].1))
                .collect();
            sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

            // Boundary points get infinite distance (always selected first)
            distances[sorted[0].0] = f64::INFINITY;
            distances[sorted[m - 1].0] = f64::INFINITY;

            let range = sorted[m - 1].1 - sorted[0].1;
            if range.abs() < self.epsilon {
                continue;
            }

            for k in 1..(m - 1) {
                let contribution = (sorted[k + 1].1 - sorted[k - 1].1) / range;
                distances[sorted[k].0] += contribution;
            }
        }

        distances
    }

    /// Main selection entry: sort into Pareto fronts, then use crowding distance
    /// within the last partially-taken front to select exactly `n` candidates.
    pub fn select(&self, candidates: &[Candidate], n: usize) -> ParetoSelectionResult {
        let total_candidates = candidates.len();

        if total_candidates == 0 || n == 0 {
            return ParetoSelectionResult {
                front: vec![],
                front_size: 0,
                total_candidates,
                num_fronts: 0,
                selected_ids: vec![],
            };
        }

        let fronts = self.fast_non_dominated_sort(candidates);
        let num_fronts = fronts.len();

        // Pre-compute dominates_count for each candidate
        let dominates_counts: Vec<usize> = (0..total_candidates)
            .map(|i| {
                (0..total_candidates)
                    .filter(|&j| i != j && self.dominates(&candidates[i], &candidates[j]))
                    .count()
            })
            .collect();

        let mut selected: Vec<ParetoPoint> = vec![];
        let mut remaining = n;

        for (rank, front_indices) in fronts.iter().enumerate() {
            if front_indices.is_empty() {
                continue;
            }

            let dists = self.crowding_distance(front_indices, candidates);

            if front_indices.len() <= remaining {
                // Take the entire front
                for (local_idx, &cidx) in front_indices.iter().enumerate() {
                    selected.push(ParetoPoint {
                        candidate: candidates[cidx].clone(),
                        rank: rank + 1,
                        crowd_distance: dists[local_idx],
                        dominates_count: dominates_counts[cidx],
                    });
                }
                remaining -= front_indices.len();
            } else {
                // Partially take this front by crowding distance (descending)
                let mut with_dist: Vec<(usize, usize, f64)> = front_indices
                    .iter()
                    .enumerate()
                    .map(|(local_idx, &cidx)| (local_idx, cidx, dists[local_idx]))
                    .collect();
                with_dist.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(Ordering::Equal));

                for &(local_idx, cidx, _) in with_dist.iter().take(remaining) {
                    selected.push(ParetoPoint {
                        candidate: candidates[cidx].clone(),
                        rank: rank + 1,
                        crowd_distance: dists[local_idx],
                        dominates_count: dominates_counts[cidx],
                    });
                }
                break;
            }
        }

        let front_size = fronts.first().map_or(0, |f| f.len());

        ParetoSelectionResult {
            front_size,
            total_candidates,
            num_fronts,
            selected_ids: selected.iter().map(|p| p.candidate.id).collect(),
            front: selected,
        }
    }

    /// Tournament between two candidates by rank then crowding distance.
    /// Lower rank wins; if same rank, higher crowding distance wins.
    pub fn crowd_tournament_select(
        &self,
        idx_a: usize,
        idx_b: usize,
        rank: &[usize],
        crowd_dist: &[f64],
    ) -> usize {
        match rank[idx_a].cmp(&rank[idx_b]) {
            Ordering::Less => idx_a,
            Ordering::Greater => idx_b,
            Ordering::Equal => {
                if crowd_dist[idx_a] > crowd_dist[idx_b] {
                    idx_a
                } else {
                    idx_b
                }
            }
        }
    }

    /// Returns the ideal point: best value in each objective across all candidates.
    pub fn ideal_point(&self, candidates: &[Candidate]) -> Vec<f64> {
        if candidates.is_empty() {
            return vec![];
        }
        self.objectives
            .iter()
            .enumerate()
            .map(|(i, obj)| {
                candidates.iter().map(|c| c.scores[i].1).fold(
                    candidates[0].scores[i].1,
                    |best, val| {
                        if obj.minimize {
                            best.min(val)
                        } else {
                            best.max(val)
                        }
                    },
                )
            })
            .collect()
    }

    /// Returns the nadir point: worst value in each objective across all candidates.
    pub fn nadir_point(&self, candidates: &[Candidate]) -> Vec<f64> {
        if candidates.is_empty() {
            return vec![];
        }
        self.objectives
            .iter()
            .enumerate()
            .map(|(i, obj)| {
                candidates.iter().map(|c| c.scores[i].1).fold(
                    candidates[0].scores[i].1,
                    |worst, val| {
                        if obj.minimize {
                            worst.max(val)
                        } else {
                            worst.min(val)
                        }
                    },
                )
            })
            .collect()
    }
}

/// Pre-built objective configurations for common evolution scenarios.
pub mod objectives {
    use super::Objective;

    pub fn standard_evolution() -> Vec<Objective> {
        vec![
            Objective {
                name: "fitness".into(),
                weight: 0.5,
                minimize: false,
            },
            Objective {
                name: "diversity".into(),
                weight: 0.3,
                minimize: false,
            },
            Objective {
                name: "novelty".into(),
                weight: 0.2,
                minimize: false,
            },
        ]
    }

    pub fn efficiency_focused() -> Vec<Objective> {
        vec![
            Objective {
                name: "fitness".into(),
                weight: 0.4,
                minimize: false,
            },
            Objective {
                name: "latency".into(),
                weight: 0.3,
                minimize: true,
            },
            Objective {
                name: "memory".into(),
                weight: 0.3,
                minimize: true,
            },
        ]
    }

    pub fn exploration_focused() -> Vec<Objective> {
        vec![
            Objective {
                name: "novelty".into(),
                weight: 0.5,
                minimize: false,
            },
            Objective {
                name: "diversity".into(),
                weight: 0.3,
                minimize: false,
            },
            Objective {
                name: "fitness".into(),
                weight: 0.2,
                minimize: false,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: u64, scores: Vec<f64>) -> Candidate {
        let names = vec![
            "fitness".to_string(),
            "diversity".to_string(),
            "novelty".to_string(),
        ];
        Candidate {
            id,
            description: format!("candidate_{}", id),
            scores: scores
                .into_iter()
                .enumerate()
                .map(|(i, v)| (names[i].clone(), v))
                .collect(),
            metadata: vec![],
        }
    }

    fn candidate_with_obj(id: u64, score_pairs: Vec<(&str, f64)>) -> Candidate {
        Candidate {
            id,
            description: format!("c_{}", id),
            scores: score_pairs
                .into_iter()
                .map(|(n, v)| (n.to_string(), v))
                .collect(),
            metadata: vec![],
        }
    }

    fn default_selector() -> ParetoFrontSelector {
        ParetoFrontSelector::new(objectives::standard_evolution())
    }

    // -----------------------------------------------------------------------
    // Basic dominance
    // -----------------------------------------------------------------------

    #[test]
    fn test_dominates_simple() {
        let sel = ParetoFrontSelector::new(vec![Objective {
            name: "fitness".into(),
            weight: 1.0,
            minimize: false,
        }]);
        let high = candidate_with_obj(1, vec![("fitness", 0.9)]);
        let low = candidate_with_obj(2, vec![("fitness", 0.5)]);
        assert!(sel.dominates(&high, &low));
        assert!(!sel.dominates(&low, &high));
    }

    #[test]
    fn test_dominates_equal_is_not_domination() {
        let sel = ParetoFrontSelector::new(vec![
            Objective {
                name: "fitness".into(),
                weight: 0.5,
                minimize: false,
            },
            Objective {
                name: "latency".into(),
                weight: 0.5,
                minimize: true,
            },
        ]);
        let a = candidate_with_obj(1, vec![("fitness", 0.8), ("latency", 0.3)]);
        let b = candidate_with_obj(2, vec![("fitness", 0.8), ("latency", 0.3)]);
        assert!(!sel.dominates(&a, &b));
        assert!(!sel.dominates(&b, &a));
    }

    #[test]
    fn test_dominates_trade_off() {
        let sel = ParetoFrontSelector::new(vec![
            Objective {
                name: "fitness".into(),
                weight: 0.5,
                minimize: false,
            },
            Objective {
                name: "latency".into(),
                weight: 0.5,
                minimize: true,
            },
        ]);
        // a: better fitness, worse latency
        let a = candidate_with_obj(1, vec![("fitness", 0.9), ("latency", 0.5)]);
        // b: worse fitness, better latency
        let b = candidate_with_obj(2, vec![("fitness", 0.7), ("latency", 0.2)]);
        // Neither dominates — trade-off
        assert!(!sel.dominates(&a, &b));
        assert!(!sel.dominates(&b, &a));
    }

    // -----------------------------------------------------------------------
    // Non-dominated sort
    // -----------------------------------------------------------------------

    #[test]
    fn test_fast_non_dominated_sort() {
        let sel = default_selector();
        // Three candidates: a dominates b, c is non-dominated w.r.t a
        let a = candidate(1, vec![0.9, 0.8, 0.7]);
        let b = candidate(2, vec![0.4, 0.3, 0.2]);
        let c = candidate(3, vec![0.85, 0.1, 0.9]);

        let candidates = vec![a, b, c];
        let fronts = sel.fast_non_dominated_sort(&candidates);
        assert_eq!(fronts.len(), 2, "should have 2 fronts");
        assert!(fronts[0].contains(&0), "a should be in first front");
        assert!(fronts[0].contains(&2), "c should be in first front");
        assert_eq!(fronts[1], vec![1], "b alone in second front");
    }

    #[test]
    fn test_fast_non_dominated_sort_empty() {
        let sel = default_selector();
        let fronts = sel.fast_non_dominated_sort(&[]);
        assert!(fronts.is_empty());
    }

    // -----------------------------------------------------------------------
    // Crowding distance
    // -----------------------------------------------------------------------

    #[test]
    fn test_crowding_distance() {
        let sel = default_selector();
        let candidates = vec![
            candidate(1, vec![0.0, 0.0, 0.0]),
            candidate(2, vec![0.5, 0.5, 0.5]),
            candidate(3, vec![1.0, 1.0, 1.0]),
        ];
        let front = vec![0usize, 1, 2];
        let dists = sel.crowding_distance(&front, &candidates);

        assert_eq!(dists.len(), 3);
        // Boundaries get infinity
        assert!(dists[0].is_infinite(), "min boundary should be infinite");
        assert!(dists[2].is_infinite(), "max boundary should be infinite");
        // Interior point gets positive distance
        assert!(
            dists[1] > 0.0,
            "interior point should have positive distance"
        );
    }

    // -----------------------------------------------------------------------
    // Selection
    // -----------------------------------------------------------------------

    #[test]
    fn test_select_returns_front() {
        let sel = default_selector();
        let candidates = vec![
            candidate(1, vec![0.9, 0.9, 0.9]),
            candidate(2, vec![0.8, 0.8, 0.8]),
            candidate(3, vec![0.7, 0.7, 0.7]),
            candidate(4, vec![0.1, 0.1, 0.1]),
        ];
        let result = sel.select(&candidates, 2);
        assert_eq!(result.selected_ids.len(), 2);
        assert_eq!(result.total_candidates, 4);
        assert_eq!(result.front_size, 3, "first 3 are non-dominated");
        assert!(result.num_fronts >= 1);
        // The dominated one (id=4) should never be selected
        assert!(!result.selected_ids.contains(&4));
    }

    #[test]
    fn test_select_empty_candidates() {
        let sel = default_selector();
        let result = sel.select(&[], 5);
        assert_eq!(result.selected_ids.len(), 0);
        assert_eq!(result.total_candidates, 0);
    }

    // -----------------------------------------------------------------------
    // Ideal and nadir
    // -----------------------------------------------------------------------

    #[test]
    fn test_ideal_and_nadir_points() {
        let sel = ParetoFrontSelector::new(vec![
            Objective {
                name: "fitness".into(),
                weight: 0.5,
                minimize: false,
            },
            Objective {
                name: "latency".into(),
                weight: 0.5,
                minimize: true,
            },
        ]);
        let candidates = vec![
            candidate_with_obj(1, vec![("fitness", 0.9), ("latency", 0.5)]),
            candidate_with_obj(2, vec![("fitness", 0.7), ("latency", 0.2)]),
            candidate_with_obj(3, vec![("fitness", 0.5), ("latency", 0.1)]),
        ];
        let ideal = sel.ideal_point(&candidates);
        let nadir = sel.nadir_point(&candidates);

        assert!((ideal[0] - 0.9).abs() < 1e-9, "max fitness");
        assert!((ideal[1] - 0.1).abs() < 1e-9, "min latency");
        assert!((nadir[0] - 0.5).abs() < 1e-9, "min fitness");
        assert!((nadir[1] - 0.5).abs() < 1e-9, "max latency");
    }

    // -----------------------------------------------------------------------
    // Objective configurations
    // -----------------------------------------------------------------------

    #[test]
    fn test_standard_objectives_config() {
        let objs = objectives::standard_evolution();
        assert_eq!(objs.len(), 3);
        assert_eq!(objs[0].name, "fitness");
        assert!((objs[0].weight - 0.5).abs() < 1e-9);
        assert!(!objs[0].minimize);
        assert!(!objs[1].minimize);
        assert!(!objs[2].minimize);
    }

    #[test]
    fn test_efficiency_focused_config() {
        let objs = objectives::efficiency_focused();
        assert_eq!(objs.len(), 3);
        assert!(objs[1].minimize, "latency should minimize");
        assert!(objs[2].minimize, "memory should minimize");
        assert!(!objs[0].minimize, "fitness should maximize");
    }

    #[test]
    fn test_exploration_focused_config() {
        let objs = objectives::exploration_focused();
        assert_eq!(objs.len(), 3);
        assert!(
            (objs[0].weight - 0.5).abs() < 1e-9,
            "novelty highest weight"
        );
    }

    // -----------------------------------------------------------------------
    // Multi-objective selection order
    // -----------------------------------------------------------------------

    #[test]
    fn test_multi_objective_selection_order() {
        let sel = ParetoFrontSelector::new(vec![
            Objective {
                name: "fitness".into(),
                weight: 0.4,
                minimize: false,
            },
            Objective {
                name: "latency".into(),
                weight: 0.3,
                minimize: true,
            },
            Objective {
                name: "diversity".into(),
                weight: 0.3,
                minimize: false,
            },
        ]);
        // a: high fitness, high latency, high diversity
        let a = candidate_with_obj(
            1,
            vec![("fitness", 0.9), ("latency", 0.9), ("diversity", 0.8)],
        );
        // b: moderate fitness, low latency, moderate diversity
        let b = candidate_with_obj(
            2,
            vec![("fitness", 0.6), ("latency", 0.1), ("diversity", 0.5)],
        );
        // c: low fitness, low latency, low diversity (dominated by b)
        let c = candidate_with_obj(
            3,
            vec![("fitness", 0.3), ("latency", 0.3), ("diversity", 0.2)],
        );

        let candidates = vec![a, b, c];
        let result = sel.select(&candidates, 3);
        // All should be selected since n == 3
        assert_eq!(result.selected_ids.len(), 3);
        // The dominated one (c) should be in a later rank
        let c_rank = result
            .front
            .iter()
            .find(|p| p.candidate.id == 3)
            .map(|p| p.rank);
        let a_rank = result
            .front
            .iter()
            .find(|p| p.candidate.id == 1)
            .map(|p| p.rank);
        assert!(
            a_rank.unwrap() < c_rank.unwrap(),
            "dominated should have worse rank"
        );
    }

    #[test]
    fn test_crowd_tournament_select() {
        let sel = default_selector();

        // Different ranks: lower rank wins
        let rank = vec![1, 2];
        let crowd_dist = vec![0.1, 0.5];
        let winner = sel.crowd_tournament_select(0, 1, &rank, &crowd_dist);
        assert_eq!(winner, 0, "lower rank should win");

        // Same rank: higher crowding distance wins
        let rank2 = vec![1, 1];
        let winner2 = sel.crowd_tournament_select(0, 1, &rank2, &crowd_dist);
        assert_eq!(winner2, 1, "higher crowd distance should win at same rank");
    }

    #[test]
    fn test_add_objective() {
        let mut sel = ParetoFrontSelector::new(vec![]);
        assert_eq!(sel.objectives.len(), 0);

        sel.add_objective("fitness", 1.0, false);
        assert_eq!(sel.objectives.len(), 1);
        assert_eq!(sel.objectives[0].name, "fitness");
        assert!(!sel.objectives[0].minimize);
    }

    #[test]
    fn test_select_tightly_bounded() {
        let sel = default_selector();
        // All candidates on the same Pareto front
        let candidates = vec![
            candidate(1, vec![0.9, 0.1, 0.5]),
            candidate(2, vec![0.8, 0.9, 0.3]),
            candidate(3, vec![0.7, 0.5, 0.9]),
            candidate(4, vec![0.6, 0.7, 0.7]),
        ];
        let result = sel.select(&candidates, 2);
        assert_eq!(result.selected_ids.len(), 2);
        // 4 from same front, all rank=1
        for point in &result.front {
            assert_eq!(point.rank, 1);
        }
    }
}
