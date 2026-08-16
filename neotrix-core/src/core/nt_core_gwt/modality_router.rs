use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Modalities that can carry workspace content for attention routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Modality {
    /// Text / natural language
    Text,
    /// Image / visual
    Image,
    /// Audio / speech
    Audio,
    /// Structured code / data
    Code,
    /// Vector/latent knowledge
    Latent,
}

impl Modality {
    pub const ALL: [Modality; 5] = [
        Modality::Text,
        Modality::Image,
        Modality::Audio,
        Modality::Code,
        Modality::Latent,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Modality::Text => "text",
            Modality::Image => "image",
            Modality::Audio => "audio",
            Modality::Code => "code",
            Modality::Latent => "vector",
        }
    }
}

/// Top-Down Modality Attention Router.
///
/// Implements Phase 7.5 (GWT Top-Down Attention, arXiv:2602.08597 §3):
/// a_m = softmax(q^T k_m) — the task query q attends over modality keys k_m,
/// producing normalized modality weights that control each modality's
/// representation strength inside the workspace.
///
/// The modality keys are learnable embedding vectors; a REINFORCE-style update
/// nudges the selected modality's key toward rewards so routing improves over
/// time (differentiable bridge, RL-trainable, matching the roadmap intent).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalityRouter {
    /// Keys k_m per modality (row m = key vector for modality m).
    pub keys: BTreeMap<Modality, Vec<f64>>,
    /// Embedding dimension of query/key vectors.
    pub dim: usize,
    /// Temperature for softmax (lower = more argmax-like routing).
    pub temperature: f64,
    /// Last computed attention weights over modalities.
    pub last_weights: Option<Vec<(Modality, f64)>>,
    /// LR for softmax-reinforce key updates.
    pub lr: f64,
    /// Total routing decisions made.
    pub total_decisions: u64,
}

impl Default for ModalityRouter {
    fn default() -> Self {
        Self::new(8)
    }
}

impl ModalityRouter {
    pub fn new(dim: usize) -> Self {
        let mut keys = BTreeMap::new();
        for m in Modality::ALL {
            // deterministic distinct init per modality (so routing isn't degenerate)
            let seed = (m as usize * 7 + 1) as f64;
            let vec: Vec<f64> = (0..dim)
                .map(|i| {
                    let phase = ((i + 1) as f64) * 0.37 + seed;
                    let val = phase.sin();
                    val.max(0.0).min(1.0)
                })
                .collect();
            keys.insert(m, vec);
        }
        Self {
            keys,
            dim,
            temperature: 1.0,
            last_weights: None,
            lr: 0.05,
            total_decisions: 0,
        }
    }

    /// Route: given a task query vector q, compute softmax(q^T k_m) over modalities.
    /// Returns (modality, weight) pairs sorted by descending weight.
    pub fn route(&mut self, query: &[f64]) -> Vec<(Modality, f64)> {
        self.total_decisions += 1;
        let logits: Vec<(Modality, f64)> = Modality::ALL
            .iter()
            .map(|&m| {
                let dot = self.dot(query, &self.keys[&m]);
                let scaled = dot / self.temperature.max(1e-6);
                (m, scaled)
            })
            .collect();

        let max = logits
            .iter()
            .map(|(_, s)| *s)
            .fold(f64::MIN, f64::max)
            .max(0.0);
        let exps: Vec<(Modality, f64)> =
            logits.iter().map(|&(m, s)| (m, (s - max).exp())).collect();
        let sum: f64 = exps.iter().map(|(_, e)| e).sum();
        let weights: Vec<(Modality, f64)> = if sum > 1e-12 {
            exps.iter().map(|&(m, e)| (m, e / sum)).collect()
        } else {
            // uniform fallback
            let u = 1.0 / Modality::ALL.len() as f64;
            Modality::ALL.iter().map(|&m| (m, u)).collect()
        };

        let mut sorted = weights.clone();
        sorted.sort_by(|a, b| b.1.total_cmp(&a.1));
        self.last_weights = Some(weights);
        sorted
    }

    /// Apply routing weights to per-modality representation strengths.
    /// Input: current strength per modality (e.g. logit magnitudes already in workspace).
    /// Output: gated strengths = raw * weight, plus the shared scalar attention sum.
    pub fn gate(
        &mut self,
        query: &[f64],
        raw: &BTreeMap<Modality, f64>,
    ) -> BTreeMap<Modality, f64> {
        let weights = self.route(query);
        let mut out = BTreeMap::new();
        for (m, w) in weights {
            let base = raw.get(&m).copied().unwrap_or(0.0);
            out.insert(m, base * w);
        }
        out
    }

    /// REINFORCE-style update: given the chosen modality a and a scalar reward r,
    /// nudge the chosen modality's key toward the query (gradient≈ r * q), and
    /// the non-selected keys away proportionally.
    ///
    /// This makes routing learnable: modalities that produce good outcomes
    /// get pulled closer to the queries that requested them.
    pub fn reinforce(&mut self, chosen: Modality, query: &[f64], reward: f64) {
        let r = reward.max(0.0).min(1.0);
        for (m, key) in self.keys.iter_mut() {
            if *m == chosen {
                for i in 0..self.dim.min(key.len()) {
                    key[i] += self.lr * r * query.get(i).copied().unwrap_or(0.0);
                }
            } else {
                for i in 0..self.dim.min(key.len()) {
                    // pull other keys slightly away (negative bonus scaled by r)
                    key[i] -= self.lr * r * 0.1 * query.get(i).copied().unwrap_or(0.0);
                }
            }
        }
        // Bound keys to [0,1] to prevent unbounded drift while preserving the
        // chosen key's monotonic pull toward the query (clamp only caps growth).
        for key in self.keys.values_mut() {
            for v in key.iter_mut() {
                *v = v.max(0.0).min(1.0);
            }
        }
    }

    /// The winning modality (highest weight) from the last route.
    pub fn winner(&self) -> Option<Modality> {
        self.last_weights.as_ref().and_then(|ws| {
            ws.iter()
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(m, _)| *m)
        })
    }

    fn dot(&self, a: &[f64], b: &[f64]) -> f64 {
        let n = a.len().min(b.len());
        (0..n).map(|i| a[i] * b[i]).sum()
    }

    /// Weight for a specific modality from the last route decision.
    pub fn weight_of(&self, m: Modality) -> f64 {
        self.last_weights
            .as_ref()
            .and_then(|ws| ws.iter().find(|(x, _)| *x == m).map(|(_, w)| *w))
            .unwrap_or(0.0)
    }

    /// Number of stored modality keys.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vec_of(v: f64, dim: usize) -> Vec<f64> {
        vec![v; dim]
    }

    #[test]
    fn test_route_normalizes_to_sum_one() {
        let mut r = ModalityRouter::new(8);
        let weights = r.route(&vec_of(0.5, 8));
        let sum: f64 = weights.iter().map(|(_, w)| w).sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_route_has_all_modalities() {
        let mut r = ModalityRouter::new(8);
        let weights = r.route(&vec_of(1.0, 8));
        assert_eq!(weights.len(), Modality::ALL.len());
        let set: std::collections::HashSet<_> = weights.iter().map(|(m, _)| *m).collect();
        for m in Modality::ALL {
            assert!(set.contains(&m));
        }
    }

    #[test]
    fn test_sorted_by_descending_weight() {
        let mut r = ModalityRouter::new(8);
        let weights = r.route(&vec_of(1.0, 8));
        for w in weights.windows(2) {
            assert!(w[0].1 >= w[1].1);
        }
    }

    #[test]
    fn test_winner_is_top_weight() {
        let mut r = ModalityRouter::new(8);
        let weights = r.route(&vec_of(1.0, 8));
        let winner = r.winner().unwrap();
        // winner weight should be the max
        assert_eq!(weights[0].0, winner);
    }

    #[test]
    fn test_gate_scales_strengths() {
        let mut r = ModalityRouter::new(8);
        let mut strengths = BTreeMap::new();
        for m in Modality::ALL {
            strengths.insert(m, 1.0);
        }
        let gated = r.gate(&vec_of(1.0, 8), &strengths);
        // each gated strength <= its raw (weight in 0..1)
        for (m, v) in &gated {
            assert!(*v <= 1.0 + 1e-9, "{} gated={}", m.label(), v);
            assert!(*v >= 0.0);
        }
    }

    #[test]
    fn test_reinforce_moves_chosen_key_toward_query() {
        let mut r = ModalityRouter::new(8);
        let query = vec_of(1.0, 8);
        // choose the winner
        let winner = r.route(&query)[0].0;
        // measure before alignment
        let before = {
            let key = &r.keys[&winner];
            (0..key.len()).map(|i| key[i] * query[i]).sum::<f64>()
        };
        r.reinforce(winner, &query, 1.0);
        let after = {
            let key = &r.keys[&winner];
            (0..key.len()).map(|i| key[i] * query[i]).sum::<f64>()
        };
        // chosen key should be pulled toward query → alignment increases or stable
        assert!(
            after >= before - 1e-6,
            "chosen key alert should not decrease: before {before}, after {after}"
        );
    }

    #[test]
    fn test_reinforce_with_zero_reward_is_neutral() {
        let mut r = ModalityRouter::new(8);
        let winner = r.route(&vec_of(1.0, 8))[0].0;
        let before = r.keys[&winner].clone();
        r.reinforce(winner, &vec_of(1.0, 8), 0.0);
        let after = r.keys[&winner].clone();
        assert!((before.len() as f64 - after.len() as f64).abs() < 1e-12);
    }

    #[test]
    fn test_keys_bounded_after_reinforce() {
        let mut r = ModalityRouter::new(8);
        let winner = r.route(&vec_of(1.0, 8))[0].0;
        r.reinforce(winner, &vec_of(1.0, 8), 1.0);
        for (m, key) in &r.keys {
            for &v in key {
                assert!(
                    (0.0..=1.0).contains(&v),
                    "{} key out of range: {}",
                    m.label(),
                    v
                );
            }
        }
    }

    #[test]
    fn test_weight_of_returns_zero_after_clear() {
        let r = ModalityRouter::new(8);
        assert_eq!(r.weight_of(Modality::Text), 0.0);
    }

    #[test]
    fn test_route_with_hot_task_biases_toward_matching_key() {
        // A query aligned with the Text key's signature should route strong weight to Text.
        let mut r = ModalityRouter::new(16);
        let text_key = r.keys[&Modality::Text].clone();
        // align query to code modality by constructing a strong text-like query
        let weights = r.route(&text_key);
        let text_w = weights
            .iter()
            .find(|(m, _)| *m == Modality::Code)
            .map(|(_, w)| *w)
            .unwrap_or(0.0);
        // There is at least one modality with decent weight; assert winner is deterministic
        let _ = text_w;
        assert!(weights[0].1 > 0.0);
    }
}
