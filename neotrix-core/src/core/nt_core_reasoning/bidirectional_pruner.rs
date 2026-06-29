use std::collections::VecDeque;

use crate::core::nt_core_hcube::QuantizedVSA;

use super::dead_end_detector::DeadEndType;
use super::vsa_blackboard::{ExpertType, Hypothesis};

#[derive(Debug, Clone)]
pub struct PrunerConfig {
    pub pre_eval_threshold: f64,
    pub post_eval_threshold: f64,
    pub similarity_threshold: f64,
    pub max_duplicate_memory: usize,
    pub min_path_length: usize,
    pub diversity_bonus: f64,
}

impl Default for PrunerConfig {
    fn default() -> Self {
        Self {
            pre_eval_threshold: 0.15,
            post_eval_threshold: 0.3,
            similarity_threshold: 0.92,
            max_duplicate_memory: 5,
            min_path_length: 2,
            diversity_bonus: 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReasoningPath {
    pub path_id: u64,
    pub hypothesis_ids: Vec<u64>,
    pub vsa_signatures: Vec<Vec<u8>>,
    pub cumulative_reward: f64,
    pub avg_step_quality: f64,
    pub diversity_score: f64,
    pub is_active: bool,
    pub pruned_at: Option<PruneReason>,
}

#[derive(Debug, Clone)]
pub enum PruneReason {
    PreEvalTooLow(f64),
    PostEvalTooLow(f64),
    DuplicateVsaState(Vec<u8>),
    DeadEndDetected(DeadEndType),
    DominatedByOther(u64),
    MaxDepthReached,
}

#[derive(Debug, Clone)]
pub struct PruneReport {
    pub path_id: u64,
    pub reason: PruneReason,
    pub step_count: usize,
    pub reward_before_prune: f64,
    pub pruner_efficiency: f64,
}

#[derive(Debug)]
pub struct BidirectionalPruner {
    pub config: PrunerConfig,
    pub active_paths: Vec<ReasoningPath>,
    pub pruned_paths: Vec<PruneReport>,
    pub next_path_id: u64,
    pub memory_set: VecDeque<Vec<u8>>,
    pub total_pruned: usize,
    pub total_evaluated: usize,
    pre_eval_saved: usize,
    post_eval_saved: usize,
    dedup_hits: usize,
    dominance_hits: usize,
    own_path_quality: Vec<(u64, f64)>,
}

impl BidirectionalPruner {
    pub fn new(config: PrunerConfig) -> Self {
        let dup_cap = config.max_duplicate_memory * 4;
        Self {
            config,
            active_paths: Vec::new(),
            pruned_paths: Vec::new(),
            next_path_id: 1,
            memory_set: VecDeque::with_capacity(dup_cap),
            total_pruned: 0,
            total_evaluated: 0,
            pre_eval_saved: 0,
            post_eval_saved: 0,
            dedup_hits: 0,
            dominance_hits: 0,
            own_path_quality: Vec::new(),
        }
    }

    pub fn pre_eval(
        &self,
        candidate_hypothesis: &Hypothesis,
        current_path: &ReasoningPath,
    ) -> Option<f64> {
        let base_score = candidate_hypothesis.confidence;

        let diversity_boost = self.diversity_estimate(candidate_hypothesis, current_path);

        let expert_prior = match candidate_hypothesis.expert {
            ExpertType::Causal => 0.25,
            ExpertType::Analogical => 0.20,
            ExpertType::MultiHop => 0.30,
            ExpertType::Contradiction => 0.10,
            ExpertType::Synthesis => 0.35,
        };

        let composite = base_score * 0.5 + diversity_boost * 0.3 + expert_prior * 0.2;

        if composite < self.config.pre_eval_threshold {
            Some(composite)
        } else {
            None
        }
    }

    pub fn post_eval(&mut self, path_id: u64, reward: f64) -> Option<PruneReason> {
        self.total_evaluated += 1;

        let (step_count, diversity_score) = match self.active_paths.iter().find(|p| p.path_id == path_id) {
            Some(p) => (p.hypothesis_ids.len(), p.diversity_score),
            None => return None,
        };

        if step_count < self.config.min_path_length {
            return None;
        }

        let avg_reward = if step_count > 0 {
            reward / step_count as f64
        } else {
            reward
        };

        let diversity_boost = if diversity_score > 0.5 {
            self.config.diversity_bonus
        } else {
            0.0
        };

        let adjusted = avg_reward + diversity_boost;

        if adjusted < self.config.post_eval_threshold {
            self.post_eval_saved += 1;
            Some(PruneReason::PostEvalTooLow(adjusted))
        } else {
            None
        }
    }

    pub fn dedup_check(&mut self, vsa_sig: &[u8]) -> Option<PruneReason> {
        if self.memory_set.len() < 2 {
            self.memory_set.push_back(vsa_sig.to_vec());
            return None;
        }

        let count = self.config.max_duplicate_memory.min(self.memory_set.len());
        let recent: Vec<Vec<u8>> = self.memory_set.iter().rev().take(count).cloned().collect();

        for stored in &recent {
            let sim = QuantizedVSA::similarity(vsa_sig, stored);
            if sim > self.config.similarity_threshold {
                self.dedup_hits += 1;
                self.memory_set.push_back(vsa_sig.to_vec());
                return Some(PruneReason::DuplicateVsaState(stored.to_vec()));
            }
        }

        self.memory_set.push_back(vsa_sig.to_vec());
        if self.memory_set.len() > self.config.max_duplicate_memory * 4 {
            let overshoot = self.memory_set.len() - self.config.max_duplicate_memory * 4;
            for _ in 0..overshoot {
                self.memory_set.pop_front();
            }
        }

        None
    }

    pub fn start_path(&mut self, initial_hypothesis: &Hypothesis) -> u64 {
        let path_id = self.next_path_id;
        self.next_path_id += 1;

        let path = ReasoningPath {
            path_id,
            hypothesis_ids: vec![initial_hypothesis.id],
            vsa_signatures: vec![initial_hypothesis.content.clone()],
            cumulative_reward: initial_hypothesis.confidence,
            avg_step_quality: initial_hypothesis.confidence,
            diversity_score: 0.5,
            is_active: true,
            pruned_at: None,
        };

        self.memory_set
            .push_back(initial_hypothesis.content.clone());
        self.active_paths.push(path);
        path_id
    }

    pub fn extend_path(
        &mut self,
        path_id: u64,
        hypothesis: &Hypothesis,
        reward: f64,
    ) -> Option<PruneReason> {
        let path_idx = match self.active_paths.iter().position(|p| p.path_id == path_id) {
            Some(idx) => idx,
            None => return Some(PruneReason::DeadEndDetected(DeadEndType::DepthExceeded)),
        };

        if !self.active_paths[path_idx].is_active {
            return None;
        }

        if let Some(reason) = self.dedup_check(&hypothesis.content) {
            let efficiency = self.pruning_ratio();
            let path = &mut self.active_paths[path_idx];
            path.is_active = false;
            path.pruned_at = Some(reason.clone());
            self.total_pruned += 1;
            let report = PruneReport {
                path_id,
                reason: reason.clone(),
                step_count: path.hypothesis_ids.len(),
                reward_before_prune: path.cumulative_reward,
                pruner_efficiency: efficiency,
            };
            self.pruned_paths.push(report);
            return Some(reason);
        }

        let cumulative_reward: f64;
        let hypothesis_count: usize;
        {
            let path = &mut self.active_paths[path_idx];
            path.hypothesis_ids.push(hypothesis.id);
            path.vsa_signatures.push(hypothesis.content.clone());
            path.cumulative_reward += reward;
            let count = path.hypothesis_ids.len();
            path.avg_step_quality = path.cumulative_reward / count as f64;
            cumulative_reward = path.cumulative_reward;
            hypothesis_count = count;
        }

        let diversity = {
            let path_ref = &self.active_paths[path_idx];
            self.diversity_estimate(hypothesis, path_ref)
        };
        self.active_paths[path_idx].diversity_score = diversity;

        if let Some(reason) = self.post_eval(path_id, cumulative_reward) {
            let efficiency = self.pruning_ratio();
            let path = &mut self.active_paths[path_idx];
            path.is_active = false;
            path.pruned_at = Some(reason.clone());
            self.total_pruned += 1;
            let report = PruneReport {
                path_id,
                reason: reason.clone(),
                step_count: hypothesis_count,
                reward_before_prune: cumulative_reward,
                pruner_efficiency: efficiency,
            };
            self.pruned_paths.push(report);
            return Some(reason);
        }

        self.own_path_quality.push((path_id, cumulative_reward));

        let dominated = self.pareto_dominance_check();
        for &(dominated_id, dominator_id) in &dominated {
            if dominated_id == path_id {
                self.prune_path(path_id, PruneReason::DominatedByOther(dominator_id));
                return Some(PruneReason::DominatedByOther(dominator_id));
            }
        }

        None
    }

    pub fn prune_path(&mut self, path_id: u64, reason: PruneReason) {
        let path_idx = match self.active_paths.iter().position(|p| p.path_id == path_id) {
            Some(idx) => idx,
            None => return,
        };

        if !self.active_paths[path_idx].is_active {
            return;
        }

        let step_count = self.active_paths[path_idx].hypothesis_ids.len();
        let reward_before_prune = self.active_paths[path_idx].cumulative_reward;

        self.active_paths[path_idx].is_active = false;
        self.active_paths[path_idx].pruned_at = Some(reason.clone());
        self.total_pruned += 1;

        let efficiency = self.pruning_ratio();

        let report = PruneReport {
            path_id,
            reason: reason.clone(),
            step_count,
            reward_before_prune,
            pruner_efficiency: efficiency,
        };
        self.pruned_paths.push(report);
    }

    pub fn pareto_dominance_check(&self) -> Vec<(u64, u64)> {
        let mut dominated: Vec<(u64, u64)> = Vec::new();
        let active: Vec<&ReasoningPath> =
            self.active_paths.iter().filter(|p| p.is_active).collect();

        if active.len() < 2 {
            return dominated;
        }

        for i in 0..active.len() {
            for j in 0..active.len() {
                if i == j {
                    continue;
                }
                let a = active[i];
                let b = active[j];
                let a_reward_per_step = if a.hypothesis_ids.len() > 0 {
                    a.cumulative_reward / a.hypothesis_ids.len() as f64
                } else {
                    0.0
                };
                let b_reward_per_step = if b.hypothesis_ids.len() > 0 {
                    b.cumulative_reward / b.hypothesis_ids.len() as f64
                } else {
                    0.0
                };

                if a_reward_per_step >= b_reward_per_step
                    && a.hypothesis_ids.len() >= b.hypothesis_ids.len()
                    && (a_reward_per_step > b_reward_per_step
                        || a.hypothesis_ids.len() > b.hypothesis_ids.len())
                {
                    dominated.push((b.path_id, a.path_id));
                }
            }
        }

        dominated.sort();
        dominated.dedup();
        dominated
    }

    pub fn diversity_boost(&self, path: &ReasoningPath) -> f64 {
        if self.active_paths.len() < 2 {
            return 0.0;
        }

        let mut min_sim = 1.0_f64;
        for other in &self.active_paths {
            if other.path_id == path.path_id || !other.is_active {
                continue;
            }
            for a_sig in &path.vsa_signatures {
                for b_sig in &other.vsa_signatures {
                    let sim = QuantizedVSA::similarity(a_sig, b_sig);
                    if sim < min_sim {
                        min_sim = sim;
                    }
                }
            }
        }

        let raw_diversity = 1.0 - min_sim;
        raw_diversity * self.config.diversity_bonus
    }

    fn diversity_estimate(&self, candidate: &Hypothesis, current: &ReasoningPath) -> f64 {
        if current.vsa_signatures.is_empty() {
            return 0.5;
        }

        let mut max_sim = 0.0_f64;
        for sig in &current.vsa_signatures {
            let sim = QuantizedVSA::similarity(&candidate.content, sig);
            if sim > max_sim {
                max_sim = sim;
            }
        }

        1.0 - max_sim
    }

    pub fn pruning_ratio(&self) -> f64 {
        let total = self.total_pruned + self.active_paths.iter().filter(|p| p.is_active).count();
        if total == 0 {
            return 0.0;
        }
        self.total_pruned as f64 / total as f64
    }

    pub fn pre_eval_savings(&self) -> f64 {
        if self.total_evaluated == 0 {
            return 0.0;
        }
        self.pre_eval_saved as f64 / self.total_evaluated as f64
    }

    pub fn post_eval_savings(&self) -> f64 {
        if self.total_evaluated == 0 {
            return 0.0;
        }
        self.post_eval_saved as f64 / self.total_evaluated as f64
    }

    pub fn stats(&self) -> PrunerStats {
        let active_count = self.active_paths.iter().filter(|p| p.is_active).count();
        let total_paths = self.total_pruned + active_count;
        let efficiency_gain = if total_paths > 0 {
            (self.pre_eval_saved + self.post_eval_saved + self.dedup_hits + self.dominance_hits)
                as f64
                / total_paths as f64
        } else {
            0.0
        };

        PrunerStats {
            total_paths_started: total_paths,
            total_pruned: self.total_pruned,
            pre_eval_pruned: self.pre_eval_saved,
            post_eval_pruned: self.post_eval_saved,
            dedup_pruned: self.dedup_hits,
            dominance_pruned: self.dominance_hits,
            active_paths: active_count,
            pruning_ratio: self.pruning_ratio(),
            efficiency_gain,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrunerStats {
    pub total_paths_started: usize,
    pub total_pruned: usize,
    pub pre_eval_pruned: usize,
    pub post_eval_pruned: usize,
    pub dedup_pruned: usize,
    pub dominance_pruned: usize,
    pub active_paths: usize,
    pub pruning_ratio: f64,
    pub efficiency_gain: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::unix_now_ms;

    fn make_hypothesis(
        id: u64,
        content: Vec<u8>,
        confidence: f64,
        expert: ExpertType,
    ) -> Hypothesis {
        Hypothesis {
            id,
            content,
            confidence,
            expert,
            supporting_evidence: vec![],
            created_at: unix_now_ms(),
            is_contradicted: false,
        }
    }

    #[test]
    fn test_new_pruner_default_config() {
        let config = PrunerConfig::default();
        let pruner = BidirectionalPruner::new(config);
        assert_eq!(pruner.active_paths.len(), 0);
        assert_eq!(pruner.pruned_paths.len(), 0);
        assert_eq!(pruner.next_path_id, 1);
        assert!((pruner.config.pre_eval_threshold - 0.15).abs() < 1e-6);
    }

    #[test]
    fn test_start_path_creates_active_path() {
        let mut pruner = BidirectionalPruner::new(PrunerConfig::default());
        let h = make_hypothesis(1, vec![1, 2, 3], 0.8, ExpertType::Causal);
        let id = pruner.start_path(&h);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_pre_eval_below_threshold_returns_score() {
        let config = PrunerConfig {
            pre_eval_threshold: 0.5,
            ..Default::default()
        };
        let pruner = BidirectionalPruner::new(config);
        let h = make_hypothesis(1, vec![1, 2, 3], 0.1, ExpertType::Contradiction);
        let path = pruner
            .active_paths
            .first()
            .cloned()
            .unwrap_or(ReasoningPath {
                path_id: 0,
                hypothesis_ids: vec![],
                vsa_signatures: vec![],
                cumulative_reward: 0.0,
                avg_step_quality: 0.0,
                diversity_score: 0.0,
                is_active: true,
                pruned_at: None,
            });
        let result = pruner.pre_eval(&h, &path);
        assert!(result.is_some());
        let (score,) = match result {
            Some(s) => (s,),
            None => unreachable!("pre_eval returned None unexpectedly"),
        };
        assert!(score < 0.5);
    }

    #[test]
    fn test_pre_eval_above_threshold_returns_none() {
        let config = PrunerConfig {
            pre_eval_threshold: 0.1,
            ..Default::default()
        };
        let pruner = BidirectionalPruner::new(config);
        let h = make_hypothesis(1, vec![1, 2, 3], 0.9, ExpertType::Synthesis);
        let path = pruner
            .active_paths
            .first()
            .cloned()
            .unwrap_or(ReasoningPath {
                path_id: 0,
                hypothesis_ids: vec![],
                vsa_signatures: vec![],
                cumulative_reward: 0.0,
                avg_step_quality: 0.0,
                diversity_score: 0.0,
                is_active: true,
                pruned_at: None,
            });
        assert!(pruner.pre_eval(&h, &path).is_none());
    }

    #[test]
    fn test_dedup_check_detects_duplicate() {
        let config = PrunerConfig {
            similarity_threshold: 0.9,
            max_duplicate_memory: 10,
            ..Default::default()
        };
        let mut pruner = BidirectionalPruner::new(config);
        let sig = vec![10u8, 20, 30, 40];
        assert!(pruner.dedup_check(&sig).is_none());
        let sig2 = vec![10u8, 20, 30, 41];
        let result = pruner.dedup_check(&sig2);
        assert!(result.is_some());
        match result.unwrap() {
            PruneReason::DuplicateVsaState(_) => {}
            _ => panic!("Expected DuplicateVsaState"),
        }
    }

    #[test]
    fn test_post_eval_prunes_low_reward() {
        let config = PrunerConfig {
            post_eval_threshold: 0.5,
            min_path_length: 1,
            ..Default::default()
        };
        let mut pruner = BidirectionalPruner::new(config);
        let h1 = make_hypothesis(1, vec![1, 2, 3], 0.4, ExpertType::Causal);
        let id = pruner.start_path(&h1);
        let h2 = make_hypothesis(2, vec![4, 5, 6], 0.1, ExpertType::Causal);
        let result = pruner.extend_path(id, &h2, 0.1);
        assert!(result.is_some());
        match result.unwrap() {
            PruneReason::PostEvalTooLow(v) => assert!(v < 0.5),
            other => panic!("Expected PostEvalTooLow, got {:?}", other),
        }
    }

    #[test]
    fn test_pareto_dominance_detects_inferior_path() {
        let config = PrunerConfig::default();
        let mut pruner = BidirectionalPruner::new(config);
        let h1 = make_hypothesis(1, vec![1], 0.9, ExpertType::Causal);
        let h2 = make_hypothesis(2, vec![2], 0.3, ExpertType::Analogical);
        let id_a = pruner.start_path(&h1);
        let id_b = pruner.start_path(&h2);
        let dominated = pruner.pareto_dominance_check();
        let has_b_dominated = dominated.iter().any(|&(d, _)| d == id_b);
        assert!(has_b_dominated);
    }

    #[test]
    fn test_prune_path_deactivates() {
        let mut pruner = BidirectionalPruner::new(PrunerConfig::default());
        let h = make_hypothesis(1, vec![1, 2, 3], 0.5, ExpertType::Causal);
        let id = pruner.start_path(&h);
        pruner.prune_path(id, PruneReason::MaxDepthReached);
        let path = pruner
            .active_paths
            .iter()
            .find(|p| p.path_id == id)
            .unwrap();
        assert!(!path.is_active);
        assert_eq!(pruner.total_pruned, 1);
    }

    #[test]
    fn test_pruning_ratio_zero_when_no_paths() {
        let pruner = BidirectionalPruner::new(PrunerConfig::default());
        assert!((pruner.pruning_ratio() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_diversity_boost_returns_value() {
        let mut pruner = BidirectionalPruner::new(PrunerConfig::default());
        let h1 = make_hypothesis(1, vec![1, 2, 3], 0.5, ExpertType::Causal);
        let h2 = make_hypothesis(2, vec![100, 101, 102], 0.5, ExpertType::Causal);
        let id_a = pruner.start_path(&h1);
        let _id_b = pruner.start_path(&h2);
        let path = pruner
            .active_paths
            .iter()
            .find(|p| p.path_id == id_a)
            .unwrap();
        let boost = pruner.diversity_boost(path);
        assert!(boost >= 0.0);
    }

    #[test]
    fn test_stats_after_pruning() {
        let config = PrunerConfig {
            post_eval_threshold: 0.8,
            min_path_length: 1,
            ..Default::default()
        };
        let mut pruner = BidirectionalPruner::new(config);
        let h = make_hypothesis(1, vec![1, 2, 3], 0.9, ExpertType::Causal);
        let id = pruner.start_path(&h);
        let h2 = make_hypothesis(2, vec![4, 5, 6], 0.1, ExpertType::Causal);
        let _ = pruner.extend_path(id, &h2, 0.1);
        let stats = pruner.stats();
        assert_eq!(stats.total_paths_started, 2);
        assert!(stats.pruning_ratio > 0.0);
    }

    #[test]
    fn test_extend_path_nonexistent_returns_dead_end() {
        let mut pruner = BidirectionalPruner::new(PrunerConfig::default());
        let h = make_hypothesis(99, vec![1], 0.5, ExpertType::Causal);
        let result = pruner.extend_path(999, &h, 0.5);
        assert!(result.is_some());
        match result.unwrap() {
            PruneReason::DeadEndDetected(DeadEndType::DepthExceeded) => {}
            _ => panic!("Expected DepthExceeded"),
        }
    }

    #[test]
    fn test_extend_path_success_no_prune() {
        let config = PrunerConfig {
            post_eval_threshold: 0.0,
            pre_eval_threshold: 0.0,
            similarity_threshold: 1.0,
            ..Default::default()
        };
        let mut pruner = BidirectionalPruner::new(config);
        let h1 = make_hypothesis(1, vec![1, 2, 3], 0.9, ExpertType::Causal);
        let id = pruner.start_path(&h1);
        let h2 = make_hypothesis(2, vec![4, 5, 6], 0.8, ExpertType::Causal);
        let result = pruner.extend_path(id, &h2, 0.8);
        assert!(result.is_none());
        let path = pruner
            .active_paths
            .iter()
            .find(|p| p.path_id == id)
            .unwrap();
        assert!(path.is_active);
        assert_eq!(path.hypothesis_ids.len(), 2);
    }

    #[test]
    fn test_pre_eval_savings_tracks_correctly() {
        let mut pruner = BidirectionalPruner::new(PrunerConfig::default());
        assert!((pruner.pre_eval_savings() - 0.0).abs() < 1e-6);
        pruner.pre_eval_saved = 3;
        pruner.total_evaluated = 10;
        assert!((pruner.pre_eval_savings() - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_post_eval_savings_tracks_correctly() {
        let mut pruner = BidirectionalPruner::new(PrunerConfig::default());
        pruner.post_eval_saved = 2;
        pruner.total_evaluated = 8;
        assert!((pruner.post_eval_savings() - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_diversity_estimate_produces_valid_range() {
        let pruner = BidirectionalPruner::new(PrunerConfig::default());
        let h = make_hypothesis(1, vec![1, 2, 3, 4], 0.5, ExpertType::Causal);
        let path = ReasoningPath {
            path_id: 0,
            hypothesis_ids: vec![],
            vsa_signatures: vec![vec![100, 101, 102, 103]],
            cumulative_reward: 0.0,
            avg_step_quality: 0.0,
            diversity_score: 0.0,
            is_active: true,
            pruned_at: None,
        };
        let score = pruner.diversity_estimate(&h, &path);
        assert!(score >= 0.0);
        assert!(score <= 1.0);
    }
}
