use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// A single peer's knowledge state — a VSA vector (binary) with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerState {
    pub peer_id: String,
    pub vector: Vec<u8>,
    pub confidence: f64,
    pub timestamp: u64,
}

/// Result of a consensus round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub converged: bool,
    pub agreed_vector: Option<Vec<u8>>,
    pub iterations: usize,
    pub agreement_ratio: f64,
    pub peer_contributions: Vec<(String, f64)>,
}

/// Banach fixed-point consensus engine.
///
/// Peers iteratively update their beliefs toward the group mean.
/// Converges to a fixed point under the Banach fixed-point theorem
/// when the update function is a contraction mapping.
#[derive(Debug, Clone)]
pub struct BanachConsensus {
    /// Peer states
    pub peers: HashMap<String, PeerState>,
    /// Convergence threshold (max vector difference)
    pub convergence_threshold: f64,
    /// Maximum iterations
    pub max_iterations: usize,
    /// Contraction factor (λ in [0,1))
    pub lambda: f64,
    /// Consensus round counter
    pub round: u64,
}

impl BanachConsensus {
    pub fn new(convergence_threshold: f64, max_iterations: usize, lambda: f64) -> Self {
        Self {
            peers: HashMap::new(),
            convergence_threshold,
            max_iterations,
            lambda,
            round: 0,
        }
    }

    /// Register or update a peer's state vector
    pub fn register_peer(&mut self, peer_id: &str, vector: Vec<u8>, confidence: f64) {
        self.peers.insert(
            peer_id.to_string(),
            PeerState {
                peer_id: peer_id.to_string(),
                vector,
                confidence: confidence.clamp(0.0, 1.0),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            },
        );
    }

    /// Remove a peer
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.remove(peer_id);
    }

    /// Run Banach fixed-point iteration.
    ///
    /// Each iteration:
    /// 1. Compute the mean vector across all peers (weighted by confidence)
    /// 2. Update each peer's vector toward the mean: v_i' = (1-λ)*v_i + λ*mean
    /// 3. Check convergence: max|v_i' - v_i| < threshold
    /// 4. Return consensus result
    pub fn converge(&mut self) -> ConsensusResult {
        self.round += 1;

        if self.peers.len() < 2 {
            return ConsensusResult {
                converged: false,
                agreed_vector: None,
                iterations: 0,
                agreement_ratio: 0.0,
                peer_contributions: vec![],
            };
        }

        let mut contributions: Vec<(String, f64)> = Vec::new();

        for iteration in 1..=self.max_iterations {
            let mean = self.weighted_mean();

            let mut max_dist = 0.0_f64;
            let mut all_distances: Vec<f64> = Vec::new();

            for peer in self.peers.values_mut() {
                if peer.vector.len() != mean.len() {
                    continue;
                }

                let mut dist = 0.0_f64;

                for (v, m) in peer.vector.iter().zip(mean.iter()) {
                    let v_f64 = *v as f64;
                    let diff = (v_f64 - m).abs();
                    if diff > dist {
                        dist = diff;
                    }
                }

                if dist > max_dist {
                    max_dist = dist;
                }
                all_distances.push(dist);

                let lambda = self.lambda;
                let one_minus_lambda = 1.0 - lambda;

                for (v, m) in peer.vector.iter_mut().zip(mean.iter()) {
                    let v_f64 = *v as f64;
                    let updated = one_minus_lambda * v_f64 + lambda * m;
                    *v = if updated >= 128.0 { 1 } else { 0 };
                }
            }

            if iteration == self.max_iterations || max_dist < self.convergence_threshold {
                let agreed = self.weighted_mean();
                let agreed_binary: Vec<u8> = agreed.iter().map(|&x| if x >= 128.0 { 1 } else { 0 }).collect();

                let total_peers = self.peers.len() as f64;
                let close_count = all_distances.iter().filter(|&&d| d < self.convergence_threshold).count();
                let agreement_ratio = close_count as f64 / total_peers;

                contributions = self
                    .peers
                    .values()
                    .map(|p| (p.peer_id.clone(), p.confidence))
                    .collect();

                return ConsensusResult {
                    converged: iteration < self.max_iterations || max_dist < self.convergence_threshold,
                    agreed_vector: Some(agreed_binary),
                    iterations: iteration,
                    agreement_ratio,
                    peer_contributions: contributions,
                };
            }
        }

        ConsensusResult {
            converged: false,
            agreed_vector: None,
            iterations: self.max_iterations,
            agreement_ratio: 0.0,
            peer_contributions: contributions,
        }
    }

    /// Compute weighted mean vector across all peers.
    ///
    /// Weight = confidence / sum(confidences).
    /// For each position, compute weighted average of f64 values,
    /// then threshold back to binary u8 (≥128 → 1, else 0).
    fn weighted_mean(&self) -> Vec<f64> {
        if self.peers.is_empty() {
            return vec![];
        }

        let total_confidence: f64 = self.peers.values().map(|p| p.confidence).sum();
        if total_confidence <= 0.0 {
            return vec![];
        }

        let max_len = self
            .peers
            .values()
            .map(|p| p.vector.len())
            .max()
            .unwrap_or(0);

        let mut weighted_sum: Vec<f64> = vec![0.0; max_len];

        for peer in self.peers.values() {
            let weight = peer.confidence / total_confidence;
            for (i, &v) in peer.vector.iter().enumerate() {
                if i < max_len {
                    weighted_sum[i] += v as f64 * weight;
                }
            }
        }

        weighted_sum
            .iter()
            .map(|&sum| if sum >= 0.5 { 255.0 } else { 0.0 })
            .collect()
    }

    /// Compute maximum difference between any two peer vectors
    pub fn max_disagreement(&self) -> f64 {
        let peers: Vec<&PeerState> = self.peers.values().collect();
        let mut max_diff = 0.0_f64;

        for i in 0..peers.len() {
            for j in (i + 1)..peers.len() {
                let a = &peers[i].vector;
                let b = &peers[j].vector;
                let len = a.len().min(b.len());

                for k in 0..len {
                    let diff = (a[k] as f64 - b[k] as f64).abs();
                    if diff > max_diff {
                        max_diff = diff;
                    }
                }
            }
        }

        max_diff
    }

    /// Peer count
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Metrics for dashboard
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "round": self.round,
            "peer_count": self.peers.len(),
            "convergence_threshold": self.convergence_threshold,
            "max_iterations": self.max_iterations,
            "lambda": self.lambda,
            "max_disagreement": self.max_disagreement(),
            "peers": self.peers.values().map(|p| {
                serde_json::json!({
                    "peer_id": p.peer_id,
                    "vector_len": p.vector.len(),
                    "confidence": p.confidence,
                    "timestamp": p.timestamp,
                })
            }).collect::<Vec<_>>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_peer_vec(bits: &[u8]) -> Vec<u8> {
        bits.to_vec()
    }

    #[test]
    fn test_new_consensus_empty() {
        let c = BanachConsensus::new(0.1, 100, 0.5);
        assert_eq!(c.peer_count(), 0);
        assert_eq!(c.round, 0);
    }

    #[test]
    fn test_register_peer() {
        let mut c = BanachConsensus::new(0.1, 100, 0.5);
        c.register_peer("alice", make_peer_vec(&[1, 0, 1, 0]), 0.8);
        assert_eq!(c.peer_count(), 1);
        assert!(c.peers.contains_key("alice"));
    }

    #[test]
    fn test_remove_peer() {
        let mut c = BanachConsensus::new(0.1, 100, 0.5);
        c.register_peer("alice", make_peer_vec(&[1, 0, 1, 0]), 0.8);
        c.register_peer("bob", make_peer_vec(&[0, 1, 0, 1]), 0.6);
        assert_eq!(c.peer_count(), 2);
        c.remove_peer("alice");
        assert_eq!(c.peer_count(), 1);
        assert!(!c.peers.contains_key("alice"));
    }

    #[test]
    fn test_converge_needs_two_peers() {
        let mut c = BanachConsensus::new(0.1, 100, 0.5);
        c.register_peer("alice", make_peer_vec(&[1, 0, 1, 0]), 0.8);
        let result = c.converge();
        assert!(!result.converged);
        assert!(result.agreed_vector.is_none());
        assert_eq!(result.iterations, 0);
    }

    #[test]
    fn test_converge_identical_vectors() {
        let mut c = BanachConsensus::new(10.0, 100, 0.5);
        c.register_peer("alice", make_peer_vec(&[1, 0, 1, 0]), 1.0);
        c.register_peer("bob", make_peer_vec(&[1, 0, 1, 0]), 1.0);
        let result = c.converge();
        assert!(result.converged);
        assert_eq!(result.iterations, 1);
        assert!((result.agreement_ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_converge_converges_toward_mean() {
        let mut c = BanachConsensus::new(0.5, 100, 0.3);
        c.register_peer("alice", make_peer_vec(&[1, 1, 1, 1]), 1.0);
        c.register_peer("bob", make_peer_vec(&[0, 0, 0, 0]), 1.0);
        let result = c.converge();
        assert!(result.converged);
        assert!(result.agreed_vector.is_some());
    }

    #[test]
    fn test_max_disagreement() {
        let mut c = BanachConsensus::new(0.1, 100, 0.5);
        c.register_peer("alice", make_peer_vec(&[0, 0, 0, 0]), 1.0);
        c.register_peer("bob", make_peer_vec(&[1, 1, 1, 1]), 1.0);
        assert!((c.max_disagreement() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_max_disagreement_identical() {
        let mut c = BanachConsensus::new(0.1, 100, 0.5);
        c.register_peer("alice", make_peer_vec(&[1, 0, 1, 0]), 1.0);
        c.register_peer("bob", make_peer_vec(&[1, 0, 1, 0]), 1.0);
        assert!((c.max_disagreement() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_metrics_contains_keys() {
        let mut c = BanachConsensus::new(0.1, 100, 0.5);
        c.register_peer("alice", make_peer_vec(&[1, 0]), 0.9);
        let metrics = c.metrics();
        assert_eq!(metrics["peer_count"], 1);
        assert_eq!(metrics["round"], 0);
        assert_eq!(metrics["lambda"], 0.5);
    }

    #[test]
    fn test_weighted_mean_biased() {
        let mut c = BanachConsensus::new(0.1, 100, 0.5);
        c.register_peer("alice", make_peer_vec(&[1, 1, 1, 1]), 1.0);
        c.register_peer("bob", make_peer_vec(&[0, 0, 0, 0]), 0.1);
        let mean = c.weighted_mean();
        // Alice has 10x the confidence, so mean should be close to all 1s
        assert_eq!(mean.len(), 4);
        // weighted mean values: alice weight = 1.0/1.1 ≈ 0.909, bob weight = 0.1/1.1 ≈ 0.091
        // for each position: 1*0.909 + 0*0.091 = 0.909 → >= 0.5 → 255.0
        for &v in &mean {
            assert!((v - 255.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_converge_peer_contributions() {
        let mut c = BanachConsensus::new(10.0, 100, 0.5);
        c.register_peer("alice", make_peer_vec(&[1, 0, 1, 0]), 1.0);
        c.register_peer("bob", make_peer_vec(&[0, 1, 0, 1]), 1.0);
        let result = c.converge();
        assert_eq!(result.peer_contributions.len(), 2);
        let alice_contrib = result.peer_contributions.iter().find(|(id, _)| id == "alice");
        assert!(alice_contrib.is_some());
    }

    #[test]
    fn test_converge_increments_round() {
        let mut c = BanachConsensus::new(10.0, 100, 0.5);
        c.register_peer("alice", make_peer_vec(&[1, 0]), 1.0);
        c.register_peer("bob", make_peer_vec(&[0, 1]), 1.0);
        assert_eq!(c.round, 0);
        c.converge();
        assert_eq!(c.round, 1);
        c.converge();
        assert_eq!(c.round, 2);
    }
}
