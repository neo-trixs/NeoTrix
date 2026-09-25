//! E8 transition probability matrix: `EdgeSemantics` and the
//! `E8TransitionMatrix` inherent impl.
//!
//! Struct definition lives in L0 (`nt_core_substrate_types`); the impl was
//! moved out of `nt_core_e8/mod.rs` (facade slimming, behaviour-neutral).
//!
//! Serde compatibility: fixed arrays >32 elements need custom serialization
//! via `FlatCounts` (Vec<u64>) and `SerdeCompat64` wrappers (re-exported
//! from the parent module for backward compatibility).

use crate::l0_substrate::nt_core_substrate_types::E8TransitionMatrix;

/// Semantic edge weight for a ReasoningFlow-style typed transition.
///
/// Maps the typed edge families from `jinulee-v/reasoningflow`
/// (arXiv:2606.05402) onto the E8 transition matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeSemantics {
    /// `reason:infer` — logical premise→conclusion, strongest signal.
    Infer,
    /// `plan:decompose` / `plan:proceed` — structured forward progress.
    Progress,
    /// `plan:verify` / `reflect:*` — verification or reflection; moderate.
    Verify,
    /// `plan:backtrack` / `validate:attack` — reversal or contradiction.
    /// Recording backtrack edges at full weight entrenches the self-loop
    /// dynamics that cause route collapse; damped to keep forward signal.
    Backtrack,
}

impl E8TransitionMatrix {
    /// Initialize transition matrix from the Mythos trace patterns.
    pub fn init_from_trace_patterns(&mut self) {
        let mythos_chain: [u8; 9] = [58, 50, 42, 34, 26, 18, 10, 2, 6];
        for i in 0..8 {
            let from = mythos_chain[i] as usize;
            let to = mythos_chain[i + 1] as usize;
            self.counts.add(from, to, 10);
            self.row_totals.0[from] += 10;
        }
        for &s in &mythos_chain {
            let si = s as usize;
            self.counts.add(si, si, 3);
            self.row_totals.0[si] += 3;
        }
        for i in 1..8 {
            let from = mythos_chain[i] as usize;
            let to = mythos_chain[i - 1] as usize;
            self.counts.add(from, to, 2);
            self.row_totals.0[from] += 2;
        }
        for i in 0..7 {
            let from = mythos_chain[i] as usize;
            let to = mythos_chain[i + 2] as usize;
            self.counts.add(from, to, 1);
            self.row_totals.0[from] += 1;
        }
        let sv = 26usize;
        let dd = 10usize;
        self.counts.add(sv, dd, 5);
        self.row_totals.0[sv] += 5;
        self.counts.add(dd, sv, 3);
        self.row_totals.0[dd] += 3;
        for &s in &mythos_chain {
            self.visit_counts.0[s as usize] += 20;
        }
    }

    /// Record a transition from `from` to `to`.
    pub fn record_transition(&mut self, from: u8, to: u8) {
        let fi = (from.min(63)) as usize;
        let ti = (to.min(63)) as usize;
        self.counts.add(fi, ti, 1);
        self.row_totals.0[fi] = self.row_totals.0[fi].saturating_add(1);
        self.visit_counts.0[fi] = self.visit_counts.0[fi].saturating_add(1);
        self.recent_transitions.push((from, to));
        if self.recent_transitions.len() > self.max_recent {
            self.recent_transitions.remove(0);
        }
    }

    /// Dominance-capped empirical distribution for a source state.
    ///
    /// Mirrors Kimi K3 "Quantile Balancing" routing: no single destination may
    /// claim more than `cap` of the row's probability mass. A row whose count
    /// distribution is monopolized by one cell (e.g. mode 0 accumulating tens of
    /// thousands of transitions while all others stay near zero) is rebalanced by
    /// clamping each cell at `cap`, then redistributing the freed mass to the
    /// under-represented cells in proportion to their raw counts. This ensures a
    /// dominant cell stays at the cap while the rest of the row still gets real
    /// predictive signal. Without this, a single over-visited mode dominates the
    /// prediction, attention stays flat (the source row becomes near-uniform),
    /// and the confidence metric decays — the exact "route collapse" seen in
    /// long-run SEAL tests.
    pub fn dominance_capped_distribution(&self, from: u8, cap: f64) -> Vec<f64> {
        let fi = from.min(63) as usize;
        let total = self.row_totals.0[fi];
        let probs = vec![1.0 / 64.0; 64];
        if total == 0 {
            return probs;
        }
        let cap = cap.clamp(0.05, 1.0);
        let raw: Vec<f64> = (0..64)
            .map(|t| self.counts.get(fi, t) as f64 / total as f64)
            .collect();
        let mut clamped = vec![0.0f64; 64];
        let mut freed = 0.0f64;
        for t in 0..64 {
            clamped[t] = raw[t].min(cap);
            if raw[t] > cap {
                freed += raw[t] - cap;
            }
        }
        let uncapped_raw_sum: f64 = raw
            .iter()
            .zip(clamped.iter())
            .filter(|(r, c)| r <= c)
            .map(|(r, _)| r)
            .sum();
        if freed > 0.0 && uncapped_raw_sum > 0.0 {
            for t in 0..64 {
                if raw[t] > clamped[t] {
                    continue;
                }
                clamped[t] += freed * (raw[t] / uncapped_raw_sum);
            }
        }
        let sum: f64 = clamped.iter().sum();
        if sum > 0.0 {
            for p in clamped.iter_mut() {
                *p /= sum;
            }
        }
        clamped
    }

    /// Record a transition with explicit reasoning edge semantics.
    ///
    /// Maps the typed edge families from `jinulee-v/reasoningflow`
    /// (arXiv:2606.05402) onto the E8 transition matrix. Forward edges
    /// (`Infer`, `Progress`) strengthen the empirical signal; `Backtrack`
    /// edges are damped so a reversal-heavy trace cannot monopolize a row
    /// and flatten attention the way mode 0 did in long-run SEAL tests.
    pub fn record_semantic_transition(&mut self, from: u8, to: u8, edge: EdgeSemantics) {
        let weight: f64 = match edge {
            EdgeSemantics::Infer => 3.0,
            EdgeSemantics::Progress => 2.0,
            EdgeSemantics::Verify => 1.0,
            EdgeSemantics::Backtrack => {
                if from == to {
                    return;
                }
                0.5
            }
        };
        let fi = (from.min(63)) as usize;
        let ti = (to.min(63)) as usize;
        if weight == weight.round() {
            let w = weight as u64;
            self.counts.add(fi, ti, w);
            self.row_totals.0[fi] = self.row_totals.0[fi].saturating_add(w);
        } else {
            self.counts.add(fi, ti, 1);
            self.row_totals.0[fi] = self.row_totals.0[fi].saturating_add(1);
        }
        self.visit_counts.0[fi] = self.visit_counts.0[fi].saturating_add(1);
        self.recent_transitions.push((from, to));
        if self.recent_transitions.len() > self.max_recent {
            self.recent_transitions.remove(0);
        }
    }

    /// Get transition probability from `from` to `to`.
    pub fn transition_prob(&self, from: u8, to: u8) -> f64 {
        let fi = from.min(63) as usize;
        let ti = to.min(63) as usize;
        let total = self.row_totals.0[fi];
        if total == 0 {
            return 1.0 / 64.0;
        }
        self.counts.get(fi, ti) as f64 / total as f64
    }

    /// Predict the most likely next state after `from`.
    pub fn predict_next(&self, from: u8, target_block: Option<u8>) -> (u8, f64) {
        let fi = from.min(63) as usize;
        let total = self.row_totals.0[fi];

        if total == 0 {
            let block_start = from & 0xF8;
            if let Some(block) = target_block {
                let mid_target = block | 0x04;
                return (mid_target, 0.5);
            }
            let next_in_block = block_start + ((from - block_start + 1) % 8);
            return (next_in_block & 0x3F, 0.33);
        }

        let best = (0..64)
            .map(|t| (t as u8, self.counts.get(fi, t) as f64))
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((from, 0.0));

        let confidence = if total > 0 {
            best.1 / total as f64
        } else {
            0.0
        };

        if let Some(block) = target_block {
            let block_end = (block as usize + 8).min(64);
            let block_probs: Vec<(usize, f64)> = (block as usize..block_end)
                .map(|t| (t, self.counts.get(fi, t) as f64 / total as f64))
                .collect();
            if let Some(&(best_block_idx, _)) = block_probs
                .iter()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            {
                let blended = (best.0 as f64 * 0.7 + best_block_idx as f64 * 0.3).round() as u8;
                return (blended.min(63), confidence.max(0.3));
            }
        }

        (best.0, confidence)
    }

    /// Detect oscillation patterns in recent transitions.
    pub fn detect_oscillation(&self, period: usize) -> Option<usize> {
        if self.recent_transitions.len() < period * 2 {
            return None;
        }
        let recent: Vec<u8> = self.recent_transitions.iter().map(|(_, t)| *t).collect();
        let start = recent.len() - period * 2;
        if recent[start..start + period] == recent[start + period..start + period * 2] {
            Some(period)
        } else {
            None
        }
    }

    /// Get the stationary distribution.
    pub fn stationary_distribution(&self) -> [f64; 64] {
        let total_visits: u64 = self.visit_counts.0.iter().sum();
        if total_visits == 0 {
            let mut uniform = [0.0; 64];
            for v in &mut uniform {
                *v = 1.0 / 64.0;
            }
            return uniform;
        }
        let mut dist = [0.0f64; 64];
        for (i, &v) in self.visit_counts.0.iter().enumerate() {
            dist[i] = v as f64 / total_visits as f64;
        }
        dist
    }

    /// Serialize to JSON bytes for binary storage.
    pub fn to_json_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    /// Deserialize from JSON bytes.
    pub fn from_json_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }

    /// Serialize to JSON string for KB TEXT column storage.
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Deserialize from JSON string.
    pub fn from_json_str(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }

    /// Merge another transition matrix into this one.
    pub fn merge(&mut self, other: &E8TransitionMatrix) {
        for i in 0..64 {
            for j in 0..64 {
                let from_other = other.counts.get(i, j);
                self.counts.add(i, j, from_other);
            }
            self.row_totals.0[i] = self.row_totals.0[i].saturating_add(other.row_totals.0[i]);
            self.visit_counts.0[i] = self.visit_counts.0[i].saturating_add(other.visit_counts.0[i]);
        }
    }

    /// mHC doubly stochastic projection (Sinkhorn-Knopp).
    ///
    /// DeepSeek-V4's mHC architecture enforces that the transition matrix
    /// is doubly stochastic: every row AND every column sums to 1.0.
    /// This prevents any single destination state from monopolizing
    /// transitions across the entire matrix — not just within one row.
    ///
    /// The Sinkhorn-Knopp algorithm alternates row-normalization and
    /// column-normalization until convergence. The result is the unique
    /// doubly stochastic matrix closest to the original in KL divergence.
    ///
    /// This directly counters route collapse: when mode 0 accumulates
    /// tens of thousands of transitions while all others stay near zero,
    /// a row-only normalization (dominance_capped_distribution) caps the
    /// dominant cell but leaves column imbalance intact. The Birkhoff
    /// projection ensures that mode 0 cannot dominate its column either,
    /// so other source states still have a path *to* diverse destinations.
    ///
    /// `max_iter` — maximum Sinkhorn iterations (default 100).
    /// `tol` — convergence tolerance on row/column sum deviation (default 1e-6).
    /// Project the transition matrix onto the Birkhoff polytope (doubly
    /// stochastic matrices) via Sinkhorn-Knopp iteration (DeepSeek-V4 mHC).
    /// Returns the projected matrix as a float probability matrix: every row
    /// and every column sums to 1, so no single destination can monopolize a
    /// source's routing mass (anti-monopolization).
    pub fn birkhoff_projected_matrix(&self, max_iter: usize, tol: f64) -> [[f64; 64]; 64] {
        let total: u64 = self.row_totals.0.iter().sum();
        if total == 0 {
            let mut uniform = [[0.0f64; 64]; 64];
            for i in 0..64 {
                for j in 0..64 {
                    uniform[i][j] = 1.0 / 64.0;
                }
            }
            return uniform;
        }

        // Build raw probability matrix from counts
        let mut mat = [[0.0f64; 64]; 64];
        for i in 0..64 {
            let ri = self.row_totals.0[i];
            if ri == 0 {
                for j in 0..64 {
                    mat[i][j] = 1.0 / 64.0;
                }
            } else {
                for j in 0..64 {
                    mat[i][j] = self.counts.get(i, j) as f64 / ri as f64;
                }
            }
        }

        // Sinkhorn-Knopp: alternate row and column normalization.
        // NOTE: `mat` is the only working matrix — row/column scales are applied
        // directly to it (never kept in separate scale vectors that then get
        // re-applied, which double-counts and breaks convergence).
        for _iter in 0..max_iter {
            // Row normalize: scale each row to sum to 1
            for i in 0..64 {
                let row_sum: f64 = (0..64).map(|j| mat[i][j]).sum();
                if row_sum > 0.0 {
                    let inv = 1.0 / row_sum;
                    for j in 0..64 {
                        mat[i][j] *= inv;
                    }
                }
            }

            // Column normalize: scale each column to sum to 1
            let mut max_dev = 0.0f64;
            for j in 0..64 {
                let col_sum: f64 = (0..64).map(|i| mat[i][j]).sum();
                if col_sum > 0.0 {
                    let inv = 1.0 / col_sum;
                    max_dev = max_dev.max((inv - 1.0).abs());
                    for i in 0..64 {
                        mat[i][j] *= inv;
                    }
                }
            }

            if max_dev < tol {
                break;
            }
        }

        mat
    }

    /// Project the transition matrix onto the Birkhoff polytope and reconstruct
    /// integer counts from the doubly-stochastic probability matrix. Each row is
    /// scaled by its own original mass so the total transition mass is preserved
    /// (rows with no data stay zero); the per-row normalized distribution is the
    /// doubly-stochastic row, so anti-monopolization survives in probability space.
    pub fn birkhoff_projection(&self, max_iter: usize, tol: f64) -> E8TransitionMatrix {
        let mat = self.birkhoff_projected_matrix(max_iter, tol);

        // Reconstruct counts, scaling each row by its own original mass so the
        // total transition mass is preserved. Row i's normalized distribution
        // remains the doubly-stochastic row mat[i][:], so the anti-monopolization
        // property survives in probability space (rows with no data stay zero).
        let mut result = E8TransitionMatrix::new();
        for i in 0..64 {
            let ri = self.row_totals.0[i];
            if ri == 0 {
                result.visit_counts.0[i] = self.visit_counts.0[i];
                continue;
            }
            for j in 0..64 {
                let count = (mat[i][j] * ri as f64).round() as u64;
                if count > 0 {
                    result.counts.add(i, j, count);
                    result.row_totals.0[i] += count;
                }
            }
            result.visit_counts.0[i] = self.visit_counts.0[i];
        }
        result
    }
}
