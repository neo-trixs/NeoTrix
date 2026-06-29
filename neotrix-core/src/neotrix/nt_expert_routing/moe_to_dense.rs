use std::collections::HashMap;

use crate::neotrix::nt_core_signal::ops::cosine_similarity;

/// DO-ACP diversity score for a single expert
#[derive(Debug, Clone)]
pub struct DiversityScore {
    pub expert_id: usize,
    pub specialization: f64,
    pub performance: f64,
    pub redundancy: f64,
    pub composite: f64,
}

/// Pruning configuration
#[derive(Debug, Clone)]
pub struct PruningConfig {
    pub min_experts: usize,
    pub diversity_weight: f64,
    pub performance_weight: f64,
    pub redundancy_weight: f64,
    pub prune_threshold: f64,
}

impl Default for PruningConfig {
    fn default() -> Self {
        Self {
            min_experts: 2,
            diversity_weight: 0.4,
            performance_weight: 0.4,
            redundancy_weight: 0.2,
            prune_threshold: 0.3,
        }
    }
}

/// Pruning result
#[derive(Debug, Clone)]
pub struct PruningResult {
    pub kept_experts: Vec<usize>,
    pub pruned_experts: Vec<usize>,
    pub scores: Vec<DiversityScore>,
    pub estimated_quality_loss: f64,
}

/// Compute population variance of a slice.
pub fn variance(values: &[f64]) -> f64 {
    let n = values.len();
    if n <= 1 {
        return 0.0;
    }
    let mean: f64 = values.iter().sum::<f64>() / n as f64;
    let sum_sq: f64 = values.iter().map(|v| (v - mean).powi(2)).sum();
    sum_sq / n as f64
}

/// Compute diversity scores for all experts from their weight vectors and performance history.
pub fn compute_diversity_scores(
    expert_weights: &HashMap<usize, Vec<f64>>,
    performance_history: &HashMap<usize, Vec<f64>>,
) -> Vec<DiversityScore> {
    if expert_weights.is_empty() {
        return Vec::new();
    }

    let max_count = expert_weights.len();

    // Compute specialization: normalized variance of each expert's weight vector
    let mut variances: Vec<(usize, f64)> = Vec::with_capacity(max_count);
    let mut max_variance = 0.0_f64;

    for (&id, weights) in expert_weights.iter() {
        let var = variance(weights);
        if var > max_variance {
            max_variance = var;
        }
        variances.push((id, var));
    }

    // Compute similarity matrix for redundancy
    let similarity_matrix = compute_weight_similarity_matrix(expert_weights);

    let mut scores: Vec<DiversityScore> = Vec::with_capacity(max_count);

    for (idx, (&id, _weights)) in expert_weights.iter().enumerate() {
        let specialization = if max_variance > 0.0 {
            variances[idx].1 / max_variance
        } else {
            0.0
        };

        let perf = performance_history.get(&id).map_or(0.5, |history| {
            if history.is_empty() {
                0.5
            } else {
                let sum: f64 = history.iter().sum();
                (sum / history.len() as f64).clamp(0.0, 1.0)
            }
        });

        // Redundancy: average of top-3 highest similarities with other experts
        let mut sims: Vec<f64> = similarity_matrix[idx]
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != idx)
            .map(|(_, &s)| s)
            .collect();

        sims.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let redundancy = if sims.is_empty() {
            0.0
        } else {
            let k = sims.len().min(3);
            sims[..k].iter().sum::<f64>() / k as f64
        };

        scores.push(DiversityScore {
            expert_id: id,
            specialization,
            performance: perf,
            redundancy,
            composite: 0.0,
        });
    }

    // Compute composite scores
    let config = PruningConfig::default();
    for score in scores.iter_mut() {
        score.composite = config.diversity_weight * score.specialization
            + config.performance_weight * score.performance
            - config.redundancy_weight * score.redundancy;
    }

    scores
}

/// Compute pairwise cosine similarity matrix for all expert weight vectors.
pub fn compute_weight_similarity_matrix(
    expert_weights: &HashMap<usize, Vec<f64>>,
) -> Vec<Vec<f64>> {
    let ids: Vec<&usize> = expert_weights.keys().collect();
    let n = ids.len();
    let mut matrix = vec![vec![0.0; n]; n];

    for i in 0..n {
        let w_i = &expert_weights[ids[i]];
        for j in 0..n {
            let w_j = &expert_weights[ids[j]];
            matrix[i][j] = cosine_similarity(w_i, w_j);
        }
    }

    matrix
}

/// Prune experts based on diversity scores and configuration.
pub fn prune_experts(scores: &[DiversityScore], config: &PruningConfig) -> PruningResult {
    let mut candidates: Vec<(usize, f64, &DiversityScore)> = scores
        .iter()
        .map(|s| (s.expert_id, s.composite, s))
        .collect();

    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let min_keep = config.min_experts.min(candidates.len());

    let keep_count = candidates
        .iter()
        .position(|(_, score, _)| *score < config.prune_threshold)
        .map(|pos| pos.max(min_keep))
        .unwrap_or(candidates.len());

    let kept_experts: Vec<usize> = candidates
        .iter()
        .take(keep_count)
        .map(|(id, _, _)| *id)
        .collect();

    let pruned_experts: Vec<usize> = candidates
        .iter()
        .skip(keep_count)
        .map(|(id, _, _)| *id)
        .collect();

    let estimated_quality_loss = estimate_quality_loss(scores, &pruned_experts);
    let scored: Vec<DiversityScore> = scores
        .iter()
        .map(|s| DiversityScore {
            expert_id: s.expert_id,
            specialization: s.specialization,
            performance: s.performance,
            redundancy: s.redundancy,
            composite: s.composite,
        })
        .collect();

    PruningResult {
        kept_experts,
        pruned_experts,
        scores: scored,
        estimated_quality_loss,
    }
}

/// Estimate quality loss from pruning — sum of weighted composites of pruned experts.
pub fn estimate_quality_loss(scores: &[DiversityScore], to_prune: &[usize]) -> f64 {
    if scores.is_empty() || to_prune.is_empty() {
        return 0.0;
    }

    let total: f64 = scores.iter().map(|s| s.composite.abs()).sum();
    if total == 0.0 {
        return 0.0;
    }

    let pruned_total: f64 = scores
        .iter()
        .filter(|s| to_prune.contains(&s.expert_id))
        .map(|s| s.composite.abs())
        .sum();

    (pruned_total / total).clamp(0.0, 1.0)
}

pub fn suggest_pruning(model: &super::WorldModel, config: &PruningConfig) -> PruningResult {
    let weights: HashMap<usize, Vec<f64>> = model
        .expert_predictor
        .expert_weights
        .iter()
        .map(|(&id, w)| (id, w.clone()))
        .collect();

    let mut perf_history: HashMap<usize, Vec<f64>> = HashMap::new();
    for record in &model.expert_predictor.performance_history {
        perf_history
            .entry(record.expert_id)
            .or_default()
            .push(record.performance);
    }

    let scores = compute_diversity_scores(&weights, &perf_history);
    prune_experts(&scores, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_weights_empty() -> HashMap<usize, Vec<f64>> {
        HashMap::new()
    }

    fn make_weights_single() -> HashMap<usize, Vec<f64>> {
        let mut m = HashMap::new();
        m.insert(0, vec![0.5, 0.3, 0.1, -0.2]);
        m
    }

    fn make_weights_two_diverse() -> HashMap<usize, Vec<f64>> {
        let mut m = HashMap::new();
        m.insert(0, vec![1.0, 0.0, 0.0, 0.0]);
        m.insert(1, vec![0.0, 1.0, 0.0, 0.0]);
        m
    }

    fn make_weights_identical() -> HashMap<usize, Vec<f64>> {
        let mut m = HashMap::new();
        m.insert(0, vec![0.5, 0.3, 0.1]);
        m.insert(1, vec![0.5, 0.3, 0.1]);
        m.insert(2, vec![0.5, 0.3, 0.1]);
        m
    }

    fn make_weights_three() -> HashMap<usize, Vec<f64>> {
        let mut m = HashMap::new();
        m.insert(0, vec![1.0, 0.0, 0.0, 0.0]);
        m.insert(1, vec![0.0, 1.0, 0.0, 0.0]);
        m.insert(2, vec![0.5, 0.5, 0.0, 0.0]);
        m
    }

    #[test]
    fn test_variance_empty() {
        assert_eq!(variance(&[]), 0.0);
    }

    #[test]
    fn test_variance_single() {
        assert_eq!(variance(&[3.0]), 0.0);
    }

    #[test]
    fn test_variance_constant() {
        assert_eq!(variance(&[2.0, 2.0, 2.0]), 0.0);
    }

    #[test]
    fn test_variance_computed() {
        let v = variance(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!((v - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_weight_similarity_matrix_empty() {
        let m = make_weights_empty();
        let mat = compute_weight_similarity_matrix(&m);
        assert!(mat.is_empty());
    }

    #[test]
    fn test_weight_similarity_matrix_single() {
        let m = make_weights_single();
        let mat = compute_weight_similarity_matrix(&m);
        assert_eq!(mat.len(), 1);
        assert!((mat[0][0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_weight_similarity_matrix_orthogonal() {
        let m = make_weights_two_diverse();
        let mat = compute_weight_similarity_matrix(&m);
        assert!((mat[0][1]).abs() < 1e-10);
        assert!((mat[1][0]).abs() < 1e-10);
        assert!((mat[0][0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_diversity_scores_empty_weights() {
        let scores = compute_diversity_scores(&make_weights_empty(), &HashMap::new());
        assert!(scores.is_empty());
    }

    #[test]
    fn test_diversity_scores_single_expert() {
        let perf = HashMap::new();
        let scores = compute_diversity_scores(&make_weights_single(), &perf);
        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].expert_id, 0);
        assert!((scores[0].specialization - 1.0).abs() < 1e-10);
        assert!((scores[0].performance - 0.5).abs() < 1e-10);
        assert!((scores[0].redundancy - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_diversity_scores_with_performance() {
        let weights = make_weights_two_diverse();
        let mut perf = HashMap::new();
        perf.insert(0, vec![0.9, 0.8, 0.85]);
        perf.insert(1, vec![0.3, 0.4, 0.35]);
        let scores = compute_diversity_scores(&weights, &perf);
        assert_eq!(scores.len(), 2);
        assert!((scores[0].performance - 0.85).abs() < 1e-10);
        assert!((scores[1].performance - 0.35).abs() < 1e-10);
    }

    #[test]
    fn test_diversity_scores_identical_experts_high_redundancy() {
        let weights = make_weights_identical();
        let perf = HashMap::new();
        let scores = compute_diversity_scores(&weights, &perf);
        assert_eq!(scores.len(), 3);
        for s in &scores {
            assert!((s.redundancy - 1.0).abs() < 1e-10);
            assert!((s.specialization - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_prune_removes_low_composite() {
        let weights = make_weights_three();
        let mut perf = HashMap::new();
        perf.insert(0, vec![0.1, 0.1]);
        perf.insert(1, vec![0.9, 0.9]);
        perf.insert(2, vec![0.5, 0.5]);
        let scores = compute_diversity_scores(&weights, &perf);
        let config = PruningConfig {
            prune_threshold: 0.4,
            min_experts: 1,
            ..Default::default()
        };
        let result = prune_experts(&scores, &config);
        assert!(!result.pruned_experts.is_empty() || result.kept_experts.len() >= 1);
        for &pruned_id in &result.pruned_experts {
            let s = scores.iter().find(|s| s.expert_id == pruned_id).unwrap();
            assert!(s.composite < config.prune_threshold);
        }
    }

    #[test]
    fn test_prune_keeps_min_experts() {
        let mut weights = HashMap::new();
        let mut perf = HashMap::new();
        for i in 0..5 {
            weights.insert(i, vec![0.1; 4]);
            perf.insert(i, vec![0.2]);
        }
        let scores = compute_diversity_scores(&weights, &perf);
        let config = PruningConfig {
            prune_threshold: 0.5,
            min_experts: 3,
            ..Default::default()
        };
        let result = prune_experts(&scores, &config);
        assert!(result.kept_experts.len() >= 3);
        assert_eq!(result.kept_experts.len() + result.pruned_experts.len(), 5);
    }

    #[test]
    fn test_quality_loss_empty_prune() {
        let weights = make_weights_two_diverse();
        let scores = compute_diversity_scores(&weights, &HashMap::new());
        let loss = estimate_quality_loss(&scores, &[]);
        assert!((loss - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_quality_loss_non_empty() {
        let weights = make_weights_three();
        let mut perf = HashMap::new();
        perf.insert(0, vec![0.9]);
        perf.insert(1, vec![0.8]);
        perf.insert(2, vec![0.7]);
        let scores = compute_diversity_scores(&weights, &perf);
        let loss = estimate_quality_loss(&scores, &[2]);
        assert!(loss >= 0.0 && loss <= 1.0);
    }

    #[test]
    fn test_prune_integration_with_world_model() {
        use crate::neotrix::nt_expert_routing::WorldModel;
        let mut wm = WorldModel::new(4);
        let latent = vec![0.5; 32];

        for expert_id in 0..4 {
            for _ in 0..5 {
                wm.expert_predictor
                    .update(expert_id, &latent, 0.5 + 0.1 * expert_id as f64, 0.01);
            }
        }

        let config = PruningConfig::default();
        let result = suggest_pruning(&wm, &config);
        assert_eq!(result.kept_experts.len() + result.pruned_experts.len(), 4);
        assert!(result.estimated_quality_loss >= 0.0 && result.estimated_quality_loss <= 1.0);
    }

    #[test]
    fn test_prune_all_above_threshold() {
        let weights = make_weights_two_diverse();
        let mut perf = HashMap::new();
        perf.insert(0, vec![0.9, 0.95]);
        perf.insert(1, vec![0.85, 0.9]);
        let scores = compute_diversity_scores(&weights, &perf);
        let config = PruningConfig {
            prune_threshold: 0.0,
            min_experts: 1,
            ..Default::default()
        };
        let result = prune_experts(&scores, &config);
        assert!(result.pruned_experts.is_empty());
        assert_eq!(result.kept_experts.len(), 2);
    }

    #[test]
    fn test_prune_all_below_threshold() {
        let weights = make_weights_two_diverse();
        let mut perf = HashMap::new();
        perf.insert(0, vec![0.1]);
        perf.insert(1, vec![0.1]);
        let mut scores = compute_diversity_scores(&weights, &perf);
        for s in &mut scores {
            s.composite = 0.0;
        }
        let config = PruningConfig {
            prune_threshold: 0.5,
            min_experts: 2,
            ..Default::default()
        };
        let result = prune_experts(&scores, &config);
        assert_eq!(result.kept_experts.len(), 2);
        assert!(result.pruned_experts.is_empty());
    }

    #[test]
    fn test_cosine_similarity_edge_cases() {
        let a = vec![0.0, 0.0];
        let b = vec![0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-10);
        let c = vec![1.0, 0.0];
        let d = vec![0.0, 0.0];
        assert!((cosine_similarity(&c, &d) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_suggest_pruning_with_empty_model() {
        use crate::neotrix::nt_expert_routing::WorldModel;
        let wm = WorldModel::new(0);
        let config = PruningConfig::default();
        let result = suggest_pruning(&wm, &config);
        assert!(result.kept_experts.is_empty());
        assert!(result.pruned_experts.is_empty());
    }
}
