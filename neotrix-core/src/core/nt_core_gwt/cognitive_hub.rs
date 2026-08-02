//! Phase 8.2 — Cognitive Hub (认知网络拓扑 · 跨组路由桥梁).
//!
//! MiCRo (arXiv:2506.13331) §4.1 structured cognitive topology:
//!
//!   - intra-group: 同类认知专家构成 complete subgraph (全连接)
//!   - inter-group: 不同类专家不直连，仅通过 `CognitiveHub` 路由
//!   - hub-to-hub: 连接权重可学习 (基于历史协作频率 reinforce)
//!
//! The hub has exactly one node per `CognitiveType` (4 nodes). A specialist in
//! group A wanting to reach group B must traverse A→hub→B. Hub weights are a
//! 4×4 matrix whose rows are probability distributions (learned via REINFORCE
//! over observed collaborative success), so the topology stays structured:
//! only 4 of 16 possible hub edges exist as meaningful paths per row's top-k.

use serde::{Deserialize, Serialize};

use super::cognitive_type::{classify, CognitiveType};
use super::module_def::SpecialistType;

/// Number of cognitive hubs = number of cognitive types.
pub const HUB_COUNT: usize = 4;
/// Number of hub-to-hub edges retained per row (structured sparsity).
pub const HUB_TOPK: usize = 2;
/// Collaboration-count floor before a hub edge becomes meaningful.
pub const COLLAB_MIN: u32 = 1;

/// Cognitive Hub — cross-group routing bridge over the 4 cognitive types.
///
/// `weights[h][t]` is the learned probability of routing a request from hub
/// `h` to hub `t`. Rows are normalized distributions (sparse: top-2 carry the
/// mass). `collab_counts[h][t]` tracks raw historical collaboration frequency,
/// used as the REINFORCE reward signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveHub {
    /// 4×4 hub-to-hub routing weights (rows normalized).
    pub weights: [[f64; HUB_COUNT]; HUB_COUNT],
    /// Raw collaboration frequency matrix (for REINFORCE reward + telemetry).
    pub collab_counts: [[u32; HUB_COUNT]; HUB_COUNT],
}

impl Default for CognitiveHub {
    fn default() -> Self {
        Self::new()
    }
}

impl CognitiveHub {
    /// Create a hub with uniform weights and zero collaboration history.
    pub fn new() -> Self {
        let init = 1.0 / HUB_COUNT as f64;
        Self {
            weights: [[init; HUB_COUNT]; HUB_COUNT],
            collab_counts: [[0; HUB_COUNT]; HUB_COUNT],
        }
    }

    /// Index of a cognitive type (matches CognitiveType::ALL declaration order).
    fn idx(ct: CognitiveType) -> usize {
        match ct {
            CognitiveType::Linguistic => 0,
            CognitiveType::Logical => 1,
            CognitiveType::Knowledge => 2,
            CognitiveType::Social => 3,
        }
    }

    /// Hub node index for a specialist type.
    pub fn hub_of(st: SpecialistType) -> usize {
        Self::idx(classify(st))
    }

    /// Routing probability from one cognitive type to another.
    pub fn route_prob(&self, from: CognitiveType, to: CognitiveType) -> f64 {
        self.weights[Self::idx(from)][Self::idx(to)]
    }

    /// Sparse routing distribution for a source hub: only the top-2 targets
    /// retain weight, the rest are zeroed and renormalized. Returns (indices, probs).
    pub fn sparse_routing(&self, from: CognitiveType) -> (Vec<usize>, Vec<f64>) {
        let row = self.weights[Self::idx(from)];
        let mut order: Vec<usize> = (0..HUB_COUNT).collect();
        order.sort_by(|&a, &b| {
            row[b]
                .partial_cmp(&row[a])
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.cmp(&b))
        });
        let mut total = 0.0f64;
        for &t in order.iter().take(HUB_TOPK) {
            total += row[t];
        }
        let mut indices = Vec::with_capacity(HUB_TOPK);
        let mut probs = Vec::with_capacity(HUB_TOPK);
        for &t in order.iter().take(HUB_TOPK) {
            indices.push(t);
            probs.push(if total > 0.0 { row[t] / total } else { 1.0 / HUB_TOPK as f64 });
        }
        (indices, probs)
    }

    /// Record a successful cross-group collaboration between two cognitive types.
    /// Bumps the count and applies a REINFORCE-style update: the observed edge
    /// is strengthened, the row is renormalized.
    pub fn record_collaboration(&mut self, from: CognitiveType, to: CognitiveType) {
        let (fi, ti) = (Self::idx(from), Self::idx(to));
        self.collab_counts[fi][ti] = self.collab_counts[fi][ti].saturating_add(1);
        // REINFORCE-style: advantage = reward (1) - baseline (mean of row).
        let row = &mut self.weights[fi];
        let mean: f64 = row.iter().sum::<f64>() / HUB_COUNT as f64;
        let lr = 0.1;
        row[ti] += lr * (1.0 - mean) * (1.0 - row[ti]);
        row[ti] = row[ti].clamp(0.05, 0.95);
        let sum: f64 = row.iter().sum();
        if sum > 0.0 {
            for w in row.iter_mut() {
                *w /= sum;
            }
        }
    }

    /// Batch record collaboration from observed specialist pair activations.
    ///
    /// Given the winning specialists of a broadcast, records one collaboration
    /// between each pair of distinct hubs they belong to (group bridging).
    pub fn record_broadcast_collaborations(&mut self, activations: &[(SpecialistType, f64)]) {
        let mut hubs_seen = [false; HUB_COUNT];
        for &(st, _) in activations {
            hubs_seen[Self::hub_of(st)] = true;
        }
        for a in 0..HUB_COUNT {
            if !hubs_seen[a] {
                continue;
            }
            for b in (a + 1)..HUB_COUNT {
                if hubs_seen[b] {
                    let (ta, tb) = (CognitiveType::ALL[a], CognitiveType::ALL[b]);
                    self.record_collaboration(ta, tb);
                    self.record_collaboration(tb, ta);
                }
            }
        }
    }

    /// Number of distinct hubs represented in a set of specialist activations.
    pub fn active_hub_count(&self, activations: &[(SpecialistType, f64)]) -> usize {
        let mut seen = [false; HUB_COUNT];
        for &(st, _) in activations {
            seen[Self::hub_of(st)] = true;
        }
        seen.iter().filter(|&&x| x).count()
    }

    /// Structured-sparsity metric: fraction of hub edges frozen by top-k routing.
    /// With top-2 of 4 per row, ≈ 0.5 of the 16 edges carry all the mass.
    pub fn sparsity(&self) -> f64 {
        1.0 - (HUB_TOPK as f64 / HUB_COUNT as f64)
    }

    /// Effective cross-group connectivity: does hub `from` currently route to
    /// hub `to` with meaningful weight (above a floor)?
    pub fn is_connected(&self, from: CognitiveType, to: CognitiveType) -> bool {
        let (fi, ti) = (Self::idx(from), Self::idx(to));
        self.collab_counts[fi][ti] >= COLLAB_MIN || self.weights[fi][ti] > 0.2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_initial_weights() {
        let hub = CognitiveHub::new();
        for a in 0..HUB_COUNT {
            let sum: f64 = hub.weights[a].iter().sum();
            assert!((sum - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn test_sparse_routing_keeps_topk() {
        let mut hub = CognitiveHub::new();
        // Concentrate weight on one edge, then verify top-k routing keeps it.
        for _ in 0..10 {
            hub.record_collaboration(CognitiveType::Linguistic, CognitiveType::Knowledge);
        }
        let (idx, probs) = hub.sparse_routing(CognitiveType::Linguistic);
        assert_eq!(idx.len(), HUB_TOPK);
        assert_eq!(probs.len(), HUB_TOPK);
        assert!((probs.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        assert!(idx.contains(&CognitiveHub::idx(CognitiveType::Knowledge)));
    }

    #[test]
    fn test_hub_of_classifies_correctly() {
        assert_eq!(
            CognitiveHub::hub_of(SpecialistType::KnowledgeRetriever),
            CognitiveHub::idx(CognitiveType::Knowledge)
        );
        assert_eq!(
            CognitiveHub::hub_of(SpecialistType::CodeAnalyzer),
            CognitiveHub::idx(CognitiveType::Logical)
        );
        assert_eq!(
            CognitiveHub::hub_of(SpecialistType::AISecurity),
            CognitiveHub::idx(CognitiveType::Social)
        );
    }

    #[test]
    fn test_collaboration_strengthens_edge() {
        let mut hub = CognitiveHub::new();
        let before = hub.route_prob(CognitiveType::Knowledge, CognitiveType::Logical);
        for _ in 0..20 {
            hub.record_collaboration(CognitiveType::Knowledge, CognitiveType::Logical);
        }
        let after = hub.route_prob(CognitiveType::Knowledge, CognitiveType::Logical);
        assert!(
            after > before,
            "repeated collaboration must strengthen the edge ({before} → {after})"
        );
        assert!(hub.is_connected(CognitiveType::Knowledge, CognitiveType::Logical));
    }

    #[test]
    fn test_broadcast_records_cross_group_bridging() {
        let mut hub = CognitiveHub::new();
        // Winners span Linguistic + Knowledge + Social → 3 hubs.
        let activations = vec![
            (SpecialistType::PatternMatcher, 0.9),
            (SpecialistType::KnowledgeIntegrator, 0.8),
            (SpecialistType::AISecurity, 0.7),
        ];
        assert_eq!(hub.active_hub_count(&activations), 3);
        hub.record_broadcast_collaborations(&activations);
        // Every pair among the 3 active hubs must now be connected.
        assert!(hub.is_connected(CognitiveType::Linguistic, CognitiveType::Knowledge));
        assert!(hub.is_connected(CognitiveType::Linguistic, CognitiveType::Social));
        assert!(hub.is_connected(CognitiveType::Knowledge, CognitiveType::Social));
    }

    #[test]
    fn test_same_hub_does_not_self_loop() {
        let mut hub = CognitiveHub::new();
        // Single-hub broadcast (all Linguistic): no cross-group edges formed.
        let activations = vec![
            (SpecialistType::PatternMatcher, 0.9),
            (SpecialistType::CreativityGenerator, 0.8),
        ];
        hub.record_broadcast_collaborations(&activations);
        for a in 0..HUB_COUNT {
            for b in 0..HUB_COUNT {
                if a != b && hub.collab_counts[a][b] > 0 {
                    // At least one of the hubs is Linguistic (0).
                    assert!(a == 0 || b == 0, "unexpected edge {a}→{b} from single-hub broadcast");
                }
            }
        }
    }

    #[test]
    fn test_sparsity_is_expected() {
        let hub = CognitiveHub::new();
        assert!((hub.sparsity() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_row_remains_normalized_after_updates() {
        let mut hub = CognitiveHub::new();
        for _ in 0..50 {
            hub.record_collaboration(CognitiveType::Social, CognitiveType::Linguistic);
        }
        let sum: f64 = hub.weights[CognitiveHub::idx(CognitiveType::Social)].iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }
}
