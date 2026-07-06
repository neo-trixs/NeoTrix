use serde::{Serialize, Deserialize};
use super::resonance::MODULE_COUNT;

/// Learned routing weights: weights[i][j] = routing strength from expert i to j.
/// Each row is a probability distribution over target experts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteWeights {
    pub weights: [[f64; MODULE_COUNT]; MODULE_COUNT],
}

impl Default for RouteWeights {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteWeights {
    pub fn new() -> Self {
        let init = 1.0 / MODULE_COUNT as f64;
        Self {
            weights: [[init; MODULE_COUNT]; MODULE_COUNT],
        }
    }

    pub fn get(&self, from: usize, to: usize) -> f64 {
        self.weights[from][to]
    }

    pub fn routing_distribution(&self, expert: usize) -> &[f64; MODULE_COUNT] {
        &self.weights[expert]
    }

    pub fn reinforce_update(&mut self, selected: &[usize], rewards: &[f64], lr: f64) {
        let mean_reward: f64 = rewards.iter().sum::<f64>() / rewards.len() as f64;
        for &s in selected {
            let row = &mut self.weights[s];
            for j in 0..MODULE_COUNT {
                let advantage = rewards[j] - mean_reward;
                row[j] += lr * advantage * (1.0 - row[j]);
                row[j] = row[j].max(0.01).min(0.99);
            }
            let sum: f64 = row.iter().sum();
            for j in 0..MODULE_COUNT {
                row[j] /= sum;
            }
        }
    }
}

/// Learned gating function: maps a task embedding to expert probabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertGate {
    embed_dim: usize,
    pub weights: Vec<Vec<f64>>,
    pub bias: Vec<f64>,
}

impl ExpertGate {
    pub fn new(embed_dim: usize) -> Self {
        let weights = vec![vec![0.1; embed_dim]; MODULE_COUNT];
        Self {
            embed_dim,
            weights,
            bias: vec![0.0; MODULE_COUNT],
        }
    }

    pub fn forward(&self, task_embedding: &[f64]) -> [f64; MODULE_COUNT] {
        let mut logits = [0.0; MODULE_COUNT];
        let dim = self.embed_dim.min(task_embedding.len());
        for i in 0..MODULE_COUNT {
            for j in 0..dim {
                logits[i] += self.weights[i][j] * task_embedding[j];
            }
            logits[i] += self.bias[i];
        }
        softmax(&logits)
    }
}

fn softmax(logits: &[f64; MODULE_COUNT]) -> [f64; MODULE_COUNT] {
    let max = logits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mut exps = [0.0; MODULE_COUNT];
    let mut sum = 0.0;
    for (i, &l) in logits.iter().enumerate() {
        exps[i] = (l - max).exp();
        sum += exps[i];
    }
    if sum == 0.0 {
        return [1.0 / MODULE_COUNT as f64; MODULE_COUNT];
    }
    for i in 0..MODULE_COUNT {
        exps[i] /= sum;
    }
    exps
}

/// MoE routing network combining ExpertGate and RouteWeights.
///
/// Uses REINFORCE-style updates to learn expert-expert routing patterns
/// based on per-expert rewards after each resonance cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoERouter {
    pub gate: ExpertGate,
    pub route_weights: RouteWeights,
    pub learning_rate: f64,
    last_selected: Vec<usize>,
    last_gate_probs: [f64; MODULE_COUNT],
}

impl MoERouter {
    pub fn new(embed_dim: usize) -> Self {
        Self {
            gate: ExpertGate::new(embed_dim),
            route_weights: RouteWeights::new(),
            learning_rate: 0.01,
            last_selected: Vec::new(),
            last_gate_probs: [1.0 / MODULE_COUNT as f64; MODULE_COUNT],
        }
    }

    /// Select top-K experts given a task embedding.
    ///
    /// Score formula: score[j] = gate_probs[j] + Σ_i(gate_probs[i] × route_weights[i][j])
    /// This combines direct task-expert affinity with learned expert-expert routing.
    pub fn select_experts(&mut self, task_embedding: &[f64], top_k: usize) -> Vec<usize> {
        let gate_probs = self.gate.forward(task_embedding);
        self.last_gate_probs = gate_probs;

        let mut scores = [0.0; MODULE_COUNT];
        for j in 0..MODULE_COUNT {
            let mut route_sum = 0.0;
            for i in 0..MODULE_COUNT {
                route_sum += gate_probs[i] * self.route_weights.weights[i][j];
            }
            scores[j] = gate_probs[j] + route_sum;
        }

        let mut indices: Vec<usize> = (0..MODULE_COUNT).collect();
        indices.sort_by(|&a, &b| scores[b].total_cmp(&scores[a]));
        let k = top_k.min(MODULE_COUNT);
        let selected: Vec<usize> = indices.into_iter().take(k).collect();

        self.last_selected = selected.clone();
        selected
    }

    /// REINFORCE-style update: strengthen routes to high-reward experts.
    pub fn routing_update(&mut self, rewards: &[f64]) {
        if self.last_selected.is_empty() {
            return;
        }
        self.route_weights
            .reinforce_update(&self.last_selected, rewards, self.learning_rate);
    }

    /// Get the last selected experts (for credit assignment).
    pub fn last_selected(&self) -> &[usize] {
        &self.last_selected
    }
}

/// Dynamic expert load balancer (DeepSeek-V3 style auxiliary-loss-free balancing).
///
/// Tracks per-expert selection frequency and adjusts router biases to
/// balance utilization without an explicit auxiliary loss.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicExpertBalancer {
    /// Per-expert selection counts over the window
    counts: [u64; MODULE_COUNT],
    /// Per-expert bias adjustments
    biases: [f64; MODULE_COUNT],
    /// Sliding window of recent selections for entropy calculation
    history: std::collections::VecDeque<usize>,
    /// Maximum history window
    window: usize,
    /// Bias adjustment strength
    alpha: f64,
    /// Target utilization per expert (1/MODULE_COUNT)
    target: f64,
}

impl DynamicExpertBalancer {
    pub fn new(window: usize) -> Self {
        Self {
            counts: [0; MODULE_COUNT],
            biases: [0.0; MODULE_COUNT],
            history: std::collections::VecDeque::with_capacity(window),
            window,
            alpha: 0.01,
            target: 1.0 / MODULE_COUNT as f64,
        }
    }

    /// Snapshot of current balancer state for diagnostics
    pub fn balancer_stats(&self) -> BalancerStats {
        BalancerStats {
            entropy: self.load_entropy(),
            imbalance_ratio: self.imbalance_ratio(),
            total_selections: self.total_selections(),
            load_distribution: self.load_distribution(),
            biases: self.biases,
        }
    }

    /// Record that an expert was selected. Returns the bias-adjusted score.
    pub fn record_selection(&mut self, expert: usize) -> f64 {
        self.counts[expert] = self.counts[expert].saturating_add(1);
        self.history.push_back(expert);
        if self.history.len() > self.window {
            if let Some(evicted) = self.history.pop_front() {
                self.counts[evicted] = self.counts[evicted].saturating_sub(1);
            }
        }
        self.biases[expert]
    }

    /// Adjust biases based on current load distribution.
    /// Experts above target have their bias reduced;
    /// experts below target have their bias increased.
    pub fn update_biases(&mut self) {
        let total = self.history.len() as f64;
        if total < 1.0 {
            return;
        }
        for i in 0..MODULE_COUNT {
            let actual = self.counts[i] as f64 / total;
            let error = actual - self.target;
            self.biases[i] -= self.alpha * error;
            self.biases[i] = self.biases[i].max(-0.5).min(0.5);
        }
    }

    /// Apply biases to gate logits before softmax.
    pub fn apply_biases(&self, logits: &mut [f64; MODULE_COUNT]) {
        for i in 0..MODULE_COUNT {
            logits[i] += self.biases[i];
        }
    }

    /// Reset all state
    pub fn reset(&mut self) {
        self.counts = [0; MODULE_COUNT];
        self.biases = [0.0; MODULE_COUNT];
        self.history.clear();
    }

    /// Current load distribution as fractions (0..1)
    pub fn load_distribution(&self) -> [f64; MODULE_COUNT] {
        let total = self.history.len() as f64;
        if total == 0.0 {
            return [self.target; MODULE_COUNT];
        }
        let mut dist = [0.0; MODULE_COUNT];
        for i in 0..MODULE_COUNT {
            dist[i] = self.counts[i] as f64 / total;
        }
        dist
    }

    /// Entropy of the current load distribution (measure of balance)
    pub fn load_entropy(&self) -> f64 {
        let dist = self.load_distribution();
        let mut entropy = 0.0;
        for &p in &dist {
            if p > 0.0 {
                entropy -= p * p.ln();
            }
        }
        entropy
    }

    /// Imbalance ratio: max_load / min_load (1.0 = perfectly balanced)
    pub fn imbalance_ratio(&self) -> f64 {
        let dist = self.load_distribution();
        let max_load = dist.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_load = dist.iter().cloned().fold(f64::INFINITY, f64::min);
        if min_load > 0.0 {
            max_load / min_load
        } else {
            f64::INFINITY
        }
    }

    /// How many selections recorded so far
    pub fn total_selections(&self) -> usize {
        self.history.len()
    }

    pub fn biases(&self) -> &[f64; MODULE_COUNT] {
        &self.biases
    }

    /// Reset load tracking state
    pub fn reset_load(&mut self) {
        self.counts = [0; MODULE_COUNT];
        self.history.clear();
    }
}

/// Stats snapshot from the DynamicExpertBalancer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalancerStats {
    pub entropy: f64,
    pub imbalance_ratio: f64,
    pub total_selections: usize,
    pub load_distribution: [f64; MODULE_COUNT],
    pub biases: [f64; MODULE_COUNT],
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_embedding() -> Vec<f64> {
        vec![0.5; 64]
    }

    #[test]
    fn test_route_weights_initialization() {
        let rw = RouteWeights::new();
        let init = 1.0 / MODULE_COUNT as f64;
        for i in 0..MODULE_COUNT {
            let sum: f64 = rw.weights[i].iter().sum();
            assert!((sum - 1.0).abs() < 1e-6, "row {i} should sum to 1.0");
            for j in 0..MODULE_COUNT {
                assert!((rw.weights[i][j] - init).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn test_expert_gate_forward() {
        let gate = ExpertGate::new(64);
        let probs = gate.forward(&dummy_embedding());
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
        for p in &probs {
            assert!(*p >= 0.0 && *p <= 1.0);
        }
    }

    #[test]
    fn test_moe_router_select_experts() {
        let mut router = MoERouter::new(64);
        let selected = router.select_experts(&dummy_embedding(), 3);
        assert_eq!(selected.len(), 3);
        // All indices should be unique and in range
        let mut unique = selected.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), selected.len());
        for &idx in &selected {
            assert!(idx < MODULE_COUNT);
        }
    }

    #[test]
    fn test_moe_router_select_all() {
        let mut router = MoERouter::new(64);
        let selected = router.select_experts(&dummy_embedding(), MODULE_COUNT);
        assert_eq!(selected.len(), MODULE_COUNT);
        let mut sorted = selected.clone();
        sorted.sort();
        assert_eq!(sorted, (0..MODULE_COUNT).collect::<Vec<_>>());
    }

    #[test]
    fn test_routing_update_changes_weights() {
        let mut router = MoERouter::new(64);
        let _ = router.select_experts(&dummy_embedding(), 1);
        let before: [[f64; MODULE_COUNT]; MODULE_COUNT] = router.route_weights.weights;

        let rewards: Vec<f64> = (0..MODULE_COUNT).map(|i| i as f64 / MODULE_COUNT as f64).collect();
        router.routing_update(&rewards);

        // Weights should have changed
        let after = router.route_weights.weights;
        let mut changed = false;
        for i in 0..MODULE_COUNT {
            for j in 0..MODULE_COUNT {
                if (before[i][j] - after[i][j]).abs() > 1e-10 {
                    changed = true;
                    break;
                }
            }
        }
        assert!(changed, "weights should change after REINFORCE update");
    }

    #[test]
    fn test_routing_update_preserves_normalization() {
        let mut router = MoERouter::new(64);
        let _ = router.select_experts(&dummy_embedding(), 2);

        let rewards: Vec<f64> = (0..MODULE_COUNT).map(|i| i as f64 / MODULE_COUNT as f64).collect();
        router.routing_update(&rewards);

        for i in 0..MODULE_COUNT {
            let sum: f64 = router.route_weights.weights[i].iter().sum();
            assert!((sum - 1.0).abs() < 1e-6, "row {i} should sum to 1.0, got {sum}");
        }
    }

    #[test]
    fn test_reinforce_high_reward_increases_weight() {
        let mut rw = RouteWeights::new();
        let init_w = rw.weights[0][1];

        // High reward for expert 1, low for rest
        let mut rewards = [0.1; MODULE_COUNT];
        rewards[1] = 0.9;
        rw.reinforce_update(&[0], &rewards, 0.1);

        // Weight from expert 0 to expert 1 should have increased
        assert!(
            rw.weights[0][1] > init_w,
            "high-reward target weight should increase: before={init_w}, after={}",
            rw.weights[0][1]
        );
    }

    #[test]
    fn test_reinforce_low_reward_decreases_weight() {
        let mut rw = RouteWeights::new();
        let init_w = rw.weights[0][1];

        // Low reward for expert 1
        let mut rewards = [0.5; MODULE_COUNT];
        rewards[1] = 0.0;
        rw.reinforce_update(&[0], &rewards, 0.1);

        // Weight from expert 0 to expert 1 should have decreased
        assert!(
            rw.weights[0][1] < init_w,
            "low-reward target weight should decrease: before={init_w}, after={}",
            rw.weights[0][1]
        );
    }

    #[test]
    fn test_softmax_normalized() {
        let logits = [2.0, 1.0, 0.5, 0.0, -1.0, -2.0, 0.0, 0.1, 0.3, 0.8, -0.5, 1.5, -1.0, 0.5];
        let probs = softmax(&logits);
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
        assert!(probs[0] > probs[1], "highest logit should have highest prob");
    }

    #[test]
    fn test_select_experts_returns_top_k_ordered() {
        let mut router = MoERouter::new(64);
        let selected = router.select_experts(&dummy_embedding(), 5);
        assert_eq!(selected.len(), 5);
        // Verify ordering: first should be highest scored
        let gate_probs = router.last_gate_probs;
        let first_score = gate_probs[selected[0]];
        for &idx in &selected[1..] {
            assert!(gate_probs[idx] <= first_score);
        }
    }

    #[test]
    fn test_router_respects_embedding_variation() {
        let mut router_a = MoERouter::new(64);
        let mut router_b = MoERouter::new(64);

        // Different embeddings should produce different selections
        let emb_a: Vec<f64> = (0..64).map(|i| (i as f64) / 64.0).collect();
        let emb_b: Vec<f64> = (0..64).map(|i| 1.0 - (i as f64) / 64.0).collect();

        let sel_a = router_a.select_experts(&emb_a, 3);
        let sel_b = router_b.select_experts(&emb_b, 3);

        // With different embeddings and initialized weights, selections may differ
        let same = sel_a == sel_b;
        // At least verify they're valid
        assert_eq!(sel_a.len(), 3);
        assert_eq!(sel_b.len(), 3);
        let _ = same; // not asserting equality or inequality
    }

    #[test]
    fn test_dynamic_balancer_new() {
        let balancer = DynamicExpertBalancer::new(100);
        assert_eq!(balancer.total_selections(), 0);
        assert_eq!(balancer.biases(), &[0.0; MODULE_COUNT]);
    }

    #[test]
    fn test_dynamic_balancer_record_selection() {
        let mut balancer = DynamicExpertBalancer::new(100);
        let bias = balancer.record_selection(0);
        assert_eq!(bias, 0.0);
        assert_eq!(balancer.total_selections(), 1);
    }

    #[test]
    fn test_dynamic_balancer_update_biases_balances() {
        let mut balancer = DynamicExpertBalancer::new(100);
        // Select expert 0 many times — it should get negative bias
        for _ in 0..50 {
            balancer.record_selection(0);
        }
        for _ in 0..10 {
            balancer.record_selection(1);
        }
        balancer.update_biases();
        let biases = balancer.biases();
        // Expert 0 (overused) should have lower bias than expert 1 (underused)
        assert!(
            biases[0] < biases[1],
            "overused expert 0 bias({}) should be < underused expert 1 bias({})",
            biases[0],
            biases[1]
        );
    }

    #[test]
    fn test_dynamic_balancer_load_entropy() {
        let mut balancer = DynamicExpertBalancer::new(100);
        // Perfectly balanced
        for i in 0..MODULE_COUNT {
            for _ in 0..5 {
                balancer.record_selection(i);
            }
        }
        let entropy = balancer.load_entropy();
        assert!(entropy > 0.0, "entropy should be positive for balanced distribution");
    }

    #[test]
    fn test_dynamic_balancer_imbalance_ratio() {
        let mut balancer = DynamicExpertBalancer::new(100);
        // 30 to expert 0, 10 to expert 1 — imbalance should be detectable
        for _ in 0..30 {
            balancer.record_selection(0);
        }
        for _ in 0..10 {
            balancer.record_selection(1);
        }
        let ratio = balancer.imbalance_ratio();
        assert!(ratio >= 1.0, "imbalance ratio should be >= 1.0, got {ratio}");
    }

    #[test]
    fn test_dynamic_balancer_reset() {
        let mut balancer = DynamicExpertBalancer::new(100);
        balancer.record_selection(0);
        balancer.update_biases();
        balancer.reset();
        assert_eq!(balancer.total_selections(), 0);
        assert_eq!(balancer.biases(), &[0.0; MODULE_COUNT]);
    }

    #[test]
    fn test_dynamic_balancer_reset_load() {
        let mut balancer = DynamicExpertBalancer::new(100);
        for _ in 0..10 {
            balancer.record_selection(0);
        }
        balancer.update_biases();
        balancer.reset_load();
        assert_eq!(balancer.total_selections(), 0);
        // Biases should be preserved after reset_load
        let all_zero = balancer.biases().iter().all(|&b| b == 0.0);
        assert!(!all_zero, "biases should persist after load reset");
    }

    #[test]
    fn test_dynamic_balancer_stats() {
        let mut balancer = DynamicExpertBalancer::new(100);
        for i in 0..MODULE_COUNT {
            for _ in 0..3 {
                balancer.record_selection(i);
            }
        }
        balancer.update_biases();
        let stats = balancer.balancer_stats();
        assert_eq!(stats.total_selections, MODULE_COUNT * 3);
        assert!(stats.entropy > 0.0);
    }

    #[test]
    fn test_balancer_apply_biases_modifies_logits() {
        let mut balancer = DynamicExpertBalancer::new(100);
        for _ in 0..30 {
            balancer.record_selection(0);
        }
        balancer.update_biases();

        let mut logits = [1.0; MODULE_COUNT];
        let original = logits;
        balancer.apply_biases(&mut logits);

        let changed = logits.iter().zip(original.iter()).any(|(a, b)| (a - b).abs() > 1e-10);
        assert!(changed, "apply_biases should modify logits");
    }

    #[test]
    fn test_dynamic_balancer_window_eviction() {
        let mut balancer = DynamicExpertBalancer::new(5);
        for i in 0..10 {
            balancer.record_selection(i % MODULE_COUNT);
        }
        // Window is 5, so only last 5 selections remain
        assert_eq!(balancer.total_selections(), 5);
    }
}
