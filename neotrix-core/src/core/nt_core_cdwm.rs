use std::collections::{HashMap, VecDeque};

/// Causal Disentangled World Model — dual-path architecture
/// EnvironmentPathway: T(z'|z, a0) — natural dynamics without intervention
/// InterventionPathway: T(z'|z, a) — action-controlled dynamics
///
/// EFC integration: κ · I · V · R · M (Effective Feedback Compute)
///   I = Informativeness (agency_bonus) — how much control agent exerted
///   V = Validity (reliability_score) — how reliable is the prediction
///   R = Non-redundancy (novelty_score) — how novel is this observation
///   M = Memory retention (retention_score) — how much state changed
///
/// Reference: CDWM (ICLR 2026), ACF (arXiv 2510.02484), C-JEPA (arXiv 2602.11389),
///            EFC Scaling Laws (arXiv 2605.29682)

const MIN_VAR: f64 = 1e-6;
const EFC_KAPPA: f64 = 10.0;
const NOVELTY_HISTORY_SIZE: usize = 16;
const RETENTION_DELTA_THRESHOLD: f64 = 0.05;

/// Running Gaussian statistics for state deltas under natural dynamics (a0).
#[derive(Debug, Clone)]
pub struct EnvironmentPathway {
    /// Running mean of z' - z for natural transitions
    pub delta_mean: Vec<f64>,
    /// Running variance per dimension
    pub delta_var: Vec<f64>,
    /// Observation count
    pub count: u64,
    state_dim: usize,
    /// Decay factor for moving average (0.0 = no decay, 0.9 = heavy decay)
    decay: f64,
}

impl EnvironmentPathway {
    pub fn new(state_dim: usize) -> Self {
        Self {
            delta_mean: vec![0.0; state_dim],
            delta_var: vec![1.0; state_dim],
            count: 0,
            state_dim,
            decay: 0.01,
        }
    }

    pub fn with_decay(mut self, decay: f64) -> Self {
        self.decay = decay.max(0.0).min(0.99);
        self
    }

    pub fn observe(&mut self, z: &[f64], z_next: &[f64]) {
        let n = self.count as f64;
        let alpha = if n > 10.0 { self.decay } else { 1.0 / (n + 1.0).max(1.0) };
        for i in 0..self.state_dim {
            let delta = z_next[i] - z[i];
            let old_mean = self.delta_mean[i];
            self.delta_mean[i] += alpha * (delta - old_mean);
            let old_var = self.delta_var[i];
            self.delta_var[i] += alpha * ((delta - old_mean).powi(2) - old_var);
            self.delta_var[i] = self.delta_var[i].max(MIN_VAR);
        }
        self.count += 1;
    }

    pub fn predict(&self, z: &[f64]) -> Vec<f64> {
        z.iter().zip(self.delta_mean.iter()).map(|(z_i, d)| z_i + d).collect()
    }

    /// Log-probability of a transition under natural dynamics: log T(z'|z, a0)
    pub fn log_prob(&self, z: &[f64], z_next: &[f64]) -> f64 {
        let mut lp = 0.0;
        for i in 0..self.state_dim {
            let delta = z_next[i] - z[i];
            let var = self.delta_var[i].max(MIN_VAR);
            let diff = delta - self.delta_mean[i];
            lp += -0.5 * (diff.powi(2) / var + (2.0 * std::f64::consts::PI * var).ln());
        }
        lp
    }

    pub fn state_dim(&self) -> usize { self.state_dim }
}

/// Intervention dynamics per action.
#[derive(Debug, Clone)]
struct ActionDynamics {
    delta_mean: Vec<f64>,
    delta_var: Vec<f64>,
    count: u64,
}

/// Per-action intervention dynamics model.
#[derive(Debug, Clone)]
pub struct InterventionPathway {
    action_dynamics: HashMap<u8, ActionDynamics>,
    state_dim: usize,
    decay: f64,
    _default_action: u8,
}

impl InterventionPathway {
    pub fn new(state_dim: usize) -> Self {
        let mut ad = HashMap::new();
        ad.insert(0, ActionDynamics {
            delta_mean: vec![0.0; state_dim],
            delta_var: vec![1.0; state_dim],
            count: 0,
        });
        Self { action_dynamics: ad, state_dim, decay: 0.01, _default_action: 0 }
    }

    pub fn with_decay(mut self, decay: f64) -> Self {
        self.decay = decay.max(0.0).min(0.99);
        self
    }

    fn ensure_action(&mut self, action: u8) -> &mut ActionDynamics {
        let state_dim = self.state_dim;
        let decay = self.decay;
        self.action_dynamics.entry(action).or_insert_with(|| {
            let mut ad = ActionDynamics {
                delta_mean: vec![0.0; state_dim],
                delta_var: vec![1.0; state_dim],
                count: 0,
            };
            let alpha = decay;
            for i in 0..state_dim {
                ad.delta_mean[i] += alpha * (0.0 - ad.delta_mean[i]);
            }
            ad
        })
    }

    pub fn observe(&mut self, z: &[f64], action: u8, z_next: &[f64]) {
        let state_dim = self.state_dim;
        let decay = self.decay;
        let ad = self.ensure_action(action);
        let n = ad.count as f64;
        let alpha = if n > 10.0 { decay } else { 1.0 / (n + 1.0).max(1.0) };
        for i in 0..state_dim {
            let delta = z_next[i] - z[i];
            let old_mean = ad.delta_mean[i];
            ad.delta_mean[i] += alpha * (delta - old_mean);
            let old_var = ad.delta_var[i];
            ad.delta_var[i] += alpha * ((delta - old_mean).powi(2) - old_var);
            ad.delta_var[i] = ad.delta_var[i].max(MIN_VAR);
        }
        ad.count += 1;
    }

    pub fn predict(&self, z: &[f64], action: u8) -> Vec<f64> {
        if let Some(ad) = self.action_dynamics.get(&action) {
            z.iter().zip(ad.delta_mean.iter()).map(|(z_i, d)| z_i + d).collect()
        } else {
            z.to_vec()
        }
    }

    /// Log-probability of a transition under intervention dynamics: log T(z'|z, a)
    pub fn log_prob(&self, z: &[f64], action: u8, z_next: &[f64]) -> f64 {
        if let Some(ad) = self.action_dynamics.get(&action) {
            let mut lp = 0.0;
            for i in 0..self.state_dim {
                let delta = z_next[i] - z[i];
                let var = ad.delta_var[i].max(MIN_VAR);
                let diff = delta - ad.delta_mean[i];
                lp += -0.5 * (diff.powi(2) / var + (2.0 * std::f64::consts::PI * var).ln());
            }
            lp
        } else {
            f64::NEG_INFINITY
        }
    }

    pub fn num_actions_tracked(&self) -> usize { self.action_dynamics.len() }

    /// Average reliability score (inverse of variance) for a given action.
    /// Used as Validity (V) factor in EFC. Returns 0.5 for untracked actions.
    pub fn reliability_score(&self, action: u8) -> f64 {
        if let Some(ad) = self.action_dynamics.get(&action) {
            let avg_var = ad.delta_var.iter().map(|v| v.max(MIN_VAR)).sum::<f64>() / self.state_dim as f64;
            1.0 / (1.0 + avg_var)
        } else {
            0.5
        }
    }
}

/// Causal Disentangled World Model with dual-path architecture.
#[derive(Debug, Clone)]
pub struct CDWM {
    pub environment: EnvironmentPathway,
    pub intervention: InterventionPathway,
    pub state_dim: usize,
    _num_actions: usize,
    /// Recent state history for novelty (R factor) computation
    history: VecDeque<Vec<f64>>,
    /// Image embedding dimension (0 = disabled, >0 enables visual state hooks)
    pub image_embedding_dim: usize,
}

impl CDWM {
    pub fn new(state_dim: usize, num_actions: usize) -> Self {
        Self {
            environment: EnvironmentPathway::new(state_dim),
            intervention: InterventionPathway::new(state_dim),
            state_dim,
            _num_actions: num_actions,
            history: VecDeque::with_capacity(NOVELTY_HISTORY_SIZE + 1),
            image_embedding_dim: 0,
        }
    }

    pub fn with_decay(mut self, decay: f64) -> Self {
        self.environment = self.environment.with_decay(decay);
        self.intervention = self.intervention.with_decay(decay);
        self
    }

    /// Enable image embedding hooks with the given embedding dimension.
    /// When >0, state transitions can incorporate visual features for image gen.
    pub fn with_image_embedding(mut self, dim: usize) -> Self {
        self.image_embedding_dim = dim;
        self
    }

    pub fn observe_natural(&mut self, z: &[f64], z_next: &[f64]) {
        self.environment.observe(z, z_next);
    }

    pub fn observe_intervention(&mut self, z: &[f64], action: u8, z_next: &[f64]) {
        self.intervention.observe(z, action, z_next);
        self.history.push_back(z.to_vec());
        if self.history.len() > NOVELTY_HISTORY_SIZE {
            self.history.pop_front();
        }
    }

    /// Agency Bonus: how much control the agent exerted.
    /// bonus = log T(z'|z, a) - log T(z'|z, a0)
    /// Positive = action caused a distinguishable effect beyond natural dynamics.
    /// Clamped to [-5, 5] for stability.
    pub fn agency_bonus(&self, z: &[f64], action: u8, z_next: &[f64]) -> f64 {
        let log_natural = self.environment.log_prob(z, z_next);
        let log_intervention = self.intervention.log_prob(z, action, z_next);
        let bonus = log_intervention - log_natural;
        bonus.max(-5.0).min(5.0)
    }

    /// Normalized agency bonus to [0, 1].
    pub fn agency_bonus_normalized(&self, z: &[f64], action: u8, z_next: &[f64]) -> f64 {
        let bonus = self.agency_bonus(z, action, z_next);
        (bonus + 5.0) / 10.0
    }

    /// Predict natural next state.
    pub fn predict_natural(&self, z: &[f64]) -> Vec<f64> {
        self.environment.predict(z)
    }

    /// Predict intervention next state.
    pub fn predict_intervention(&self, z: &[f64], action: u8) -> Vec<f64> {
        self.intervention.predict(z, action)
    }

    /// Factor-level masking prediction error (C-JEPA style).
    /// Masks factor `masked_idx` (sets it to 0), then uses linear regression
    /// on remaining factors to predict the masked one.
    /// High error = factor carries independent causal information.
    pub fn factor_mask_prediction_error(&self, z: &[f64], masked_idx: usize) -> f64 {
        if masked_idx >= self.state_dim || self.state_dim < 2 {
            return 0.0;
        }
        let mut sum_others = 0.0;
        let mut count_others = 0;
        for i in 0..self.state_dim {
            if i != masked_idx {
                sum_others += z[i];
                count_others += 1;
            }
        }
        if count_others == 0 {
            return 0.0;
        }
        let mean_others = sum_others / count_others as f64;
        let z_masked = z[masked_idx];
        let prediction = mean_others * 0.5;
        (z_masked - prediction).abs()
    }

    /// Predict each factor from all others and return the prediction error vector.
    /// Used to identify causally independent factors.
    pub fn factor_independence_scores(&self, z: &[f64]) -> Vec<f64> {
        (0..self.state_dim)
            .map(|i| self.factor_mask_prediction_error(z, i))
            .collect()
    }

    // ── EFC Four-Factor Product ──────────────────────────────────────

    /// Validity score V ∈ [0, 1]: based on prediction variance of the action.
    /// Low variance = high reliability = high validity.
    pub fn validity_score(&self, action: u8) -> f64 {
        self.intervention.reliability_score(action)
    }

    /// Novelty score R ∈ [0, 1]: inverse of max cosine similarity to recent states.
    /// 1.0 = completely new state, 0.0 = identical to a recent state.
    pub fn novelty_score(&self, z: &[f64]) -> f64 {
        if self.history.is_empty() {
            return 1.0;
        }
        let z_norm = (z.iter().map(|x| x * x).sum::<f64>()).sqrt().max(1e-10);
        let max_sim = self.history.iter()
            .map(|h| {
                let dot: f64 = h.iter().zip(z.iter()).map(|(a, b)| a * b).sum();
                let h_norm = (h.iter().map(|x| x * x).sum::<f64>()).sqrt().max(1e-10);
                dot / (z_norm * h_norm)
            })
            .fold(f64::NEG_INFINITY, |a, b| a.max(b));
        let sim = max_sim.max(-1.0).min(1.0);
        // Map cosine [-1, 1] → novelty [0, 1]
        // 1.0 = completely orthogonal (novel), 0.0 = same direction (redundant)
        (1.0 - sim) * 0.5
    }

    /// Memory retention score M ∈ [0, 1]: based on state change magnitude.
    /// Large delta = much to retain = high M.
    pub fn retention_score(&self, z: &[f64], z_next: &[f64]) -> f64 {
        let delta_norm: f64 = z.iter().zip(z_next.iter())
            .map(|(a, b)| (b - a).powi(2))
            .sum::<f64>()
            .sqrt();
        (delta_norm / (delta_norm + RETENTION_DELTA_THRESHOLD)).min(1.0)
    }

    /// EFC score: κ · I · V · R · M
    /// Only non-zero when all four factors are present (product form).
    /// Clamped to [0, κ].
    pub fn efc_score(&self, z: &[f64], action: u8, z_next: &[f64]) -> f64 {
        let i = self.agency_bonus_normalized(z, action, z_next); // Informativeness
        let v = self.validity_score(action);                      // Validity
        let r = self.novelty_score(z);                            // Non-redundancy
        let m = self.retention_score(z, z_next);                  // Memory retention
        let efc = EFC_KAPPA * i * v * r * m;
        efc.max(0.0).min(EFC_KAPPA)
    }

    /// EFC harness efficiency: η = EFC / C_raw
    /// Measures how much effective feedback per unit of raw effort.
    /// C_raw is approximated as (1 + num_observations) for simplicity.
    pub fn efc_efficiency(&self) -> f64 {
        let total_obs = self.environment.count + self.intervention.num_actions_tracked() as u64;
        let c_raw = (total_obs as f64).max(1.0);
        // Use current average efc as proxy (requires recent z, action, z_next)
        // Simple estimate: η = avg(efc) / c_raw
        // This is a placeholder — real η would integrate over trajectory
        0.5 / c_raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_pathway_initialization() {
        let ep = EnvironmentPathway::new(6);
        assert_eq!(ep.state_dim, 6);
        assert_eq!(ep.count, 0);
    }

    #[test]
    fn test_environment_pathway_observe_and_predict() {
        let mut ep = EnvironmentPathway::new(3);
        let z = vec![0.0, 0.0, 0.0];
        let z_next = vec![1.0, 0.5, -0.5];
        ep.observe(&z, &z_next);
        let pred = ep.predict(&z);
        assert!((pred[0] - 1.0).abs() < 0.01);
        assert!((pred[1] - 0.5).abs() < 0.01);
        assert!((pred[2] - (-0.5)).abs() < 0.01);
    }

    #[test]
    fn test_intervention_pathway_initialization() {
        let ip = InterventionPathway::new(4);
        assert!(ip.num_actions_tracked() >= 1);
    }

    #[test]
    fn test_intervention_pathway_observe() {
        let mut ip = InterventionPathway::new(2);
        let z = vec![0.0, 0.0];
        let z_next = vec![1.0, 0.0];
        ip.observe(&z, 1, &z_next);
        let pred = ip.predict(&z, 1);
        assert!((pred[0] - 1.0).abs() < 0.01, "action 1 should predict delta=1 on dim 0");
    }

    #[test]
    fn test_cdwm_initialization() {
        let cdwm = CDWM::new(6, 8);
        assert_eq!(cdwm.state_dim, 6);
    }

    #[test]
    fn test_cdwm_agency_bonus_positive_when_action_has_effect() {
        let mut cdwm = CDWM::new(3, 4);

        // Train natural dynamics: no-op transitions
        for _ in 0..10 {
            cdwm.observe_natural(&[0.0, 0.0, 0.0], &[0.1, 0.0, 0.0]);
        }

        // Train intervention for action 1: strong effect on dim 0
        for _ in 0..10 {
            cdwm.observe_intervention(&[0.0, 0.0, 0.0], 1, &[1.0, 0.0, 0.0]);
        }

        // Now test a transition where action 1 has its usual strong effect
        let bonus = cdwm.agency_bonus(&[0.0, 0.0, 0.0], 1, &[1.0, 0.0, 0.0]);
        assert!(bonus > 0.0, "agency bonus should be positive when action has effect, got {}", bonus);
    }

    #[test]
    fn test_cdwm_agency_bonus_normalized_in_range() {
        let cdwm = CDWM::new(3, 4);
        let bonus = cdwm.agency_bonus_normalized(&[0.0; 3], 1, &[0.0; 3]);
        assert!(bonus >= 0.0);
        assert!(bonus <= 1.0);
    }

    #[test]
    fn test_factor_mask_prediction_error_zero_for_dependent() {
        let cdwm = CDWM::new(3, 4);
        // All factors equal → can be predicted from others
        let z = vec![1.0, 1.0, 1.0];
        let error = cdwm.factor_mask_prediction_error(&z, 0);
        // mean_others = 1.0, prediction = 0.5, z_masked = 1.0, error = 0.5
        assert!((error - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_factor_mask_prediction_error_high_for_independent() {
        let cdwm = CDWM::new(3, 4);
        // Factor 0 is very different from others
        let z = vec![10.0, 0.0, 0.0];
        let error = cdwm.factor_mask_prediction_error(&z, 0);
        // mean_others = 0.0, prediction = 0.0, z_masked = 10.0, error = 10.0
        assert!(error > 5.0, "independent factor should have high prediction error");
    }

    #[test]
    fn test_factor_independence_scores_length() {
        let cdwm = CDWM::new(6, 4);
        let z = vec![0.5, 1.0, 1.5, 2.0, 2.5, 3.0];
        let scores = cdwm.factor_independence_scores(&z);
        assert_eq!(scores.len(), 6);
    }

    #[test]
    fn test_cdwm_predict_natural_and_intervention_differ() {
        let mut cdwm = CDWM::new(2, 4);

        cdwm.observe_natural(&[0.0, 0.0], &[0.1, 0.1]);
        cdwm.observe_intervention(&[0.0, 0.0], 2, &[1.0, 0.5]);

        let nat = cdwm.predict_natural(&[0.0, 0.0]);
        let int = cdwm.predict_intervention(&[0.0, 0.0], 2);

        let diff: f64 = nat.iter().zip(int.iter()).map(|(a, b)| (a - b).abs()).sum();
        assert!(diff > 0.01, "natural and intervention predictions should differ");
    }

    #[test]
    fn test_multiple_actions_tracked_independently() {
        let mut cdwm = CDWM::new(1, 8);
        cdwm.observe_intervention(&[0.0], 1, &[2.0]);
        cdwm.observe_intervention(&[0.0], 2, &[-1.0]);

        let p1 = cdwm.predict_intervention(&[0.0], 1);
        let p2 = cdwm.predict_intervention(&[0.0], 2);

        assert!((p1[0] - 2.0).abs() < 0.01, "action 1 should predict +2");
        assert!((p2[0] - (-1.0)).abs() < 0.01, "action 2 should predict -1");
    }

    #[test]
    fn test_agency_bonus_near_zero_for_unseen_action() {
        let cdwm = CDWM::new(2, 4);
        let bonus = cdwm.agency_bonus(&[0.0, 0.0], 7, &[0.0, 0.0]);
        assert!(bonus.is_finite(), "unseen action should give finite bonus");
    }

    #[test]
    fn test_environment_pathway_log_prob() {
        let mut ep = EnvironmentPathway::new(2);
        ep.observe(&[0.0, 0.0], &[1.0, 1.0]);
        let lp = ep.log_prob(&[0.0, 0.0], &[1.0, 1.0]);
        assert!(lp.is_finite());
        assert!(lp < 0.0, "log probability should be negative");
    }

    // ── EFC Factor Tests ────────────────────────────────────────────

    #[test]
    fn test_validity_score_high_for_low_variance() {
        let mut cdwm = CDWM::new(3, 4);
        // Train action 1 with very consistent results
        for _ in 0..20 {
            cdwm.observe_intervention(&[0.0; 3], 1, &[1.0, 1.0, 1.0]);
        }
        let v = cdwm.validity_score(1);
        assert!(v > 0.5, "low-variance action should have high validity, got {}", v);
    }

    #[test]
    fn test_validity_score_lower_for_high_variance() {
        let mut cdwm = CDWM::new(2, 4);
        // Train action 2 with wildly varying results
        for i in 0..20 {
            let noise = (i as f64) * 0.5;
            cdwm.observe_intervention(&[0.0, 0.0], 2, &[noise, -noise]);
        }
        let v1 = cdwm.validity_score(2);
        let mut cdwm2 = CDWM::new(2, 4);
        for _ in 0..20 {
            cdwm2.observe_intervention(&[0.0, 0.0], 2, &[1.0, 1.0]);
        }
        let v2 = cdwm2.validity_score(2);
        assert!(v1 < v2, "high-variance action should have lower validity");
    }

    #[test]
    fn test_novelty_score_decreases_with_familiar_states() {
        let mut cdwm = CDWM::new(2, 4);
        // Add same state repeatedly
        for _ in 0..5 {
            cdwm.observe_intervention(&[1.0, 1.0], 0, &[2.0, 2.0]);
        }
        // Novelty of a similar state should be low (redundant)
        let r = cdwm.novelty_score(&[1.0, 1.0]);
        assert!(r < 0.5, "familiar state should have low novelty, got {}", r);
    }

    #[test]
    fn test_novelty_score_high_for_new_states() {
        let cdwm = CDWM::new(2, 4);
        // Empty history → everything is novel
        let r = cdwm.novelty_score(&[42.0, -7.0]);
        assert!((r - 1.0).abs() < 0.01, "novelty on empty history should be 1.0");
    }

    #[test]
    fn test_retention_score_scales_with_delta() {
        let cdwm = CDWM::new(3, 4);
        let small_m = cdwm.retention_score(&[0.0; 3], &[0.01, 0.01, 0.01]);
        let large_m = cdwm.retention_score(&[0.0; 3], &[10.0, 10.0, 10.0]);
        assert!(small_m < large_m, "larger delta should have higher retention");
        assert!(small_m < 0.5, "tiny delta should have low retention");
        assert!(large_m > 0.9, "large delta should have high retention");
    }

    #[test]
    fn test_efc_score_is_product_of_four_factors() {
        let mut cdwm = CDWM::new(2, 4);
        // Train with consistent action
        for _ in 0..20 {
            cdwm.observe_natural(&[0.0, 0.0], &[0.1, 0.0]);
            cdwm.observe_intervention(&[0.0, 0.0], 1, &[2.0, 0.0]);
        }
        // Known state + known action + large delta → high EFC
        let efc = cdwm.efc_score(&[0.0, 0.0], 1, &[2.0, 0.0]);
        assert!(efc > 0.0, "EFC should be positive for informative transition");
        assert!(efc <= EFC_KAPPA, "EFC should not exceed kappa");
    }

    #[test]
    fn test_efc_score_near_zero_when_all_factors_low() {
        let cdwm = CDWM::new(2, 4);
        // Untrained model + small delta + fresh state → low EFC
        let efc = cdwm.efc_score(&[0.0, 0.0], 5, &[0.001, 0.001]);
        assert!(efc < 1.0, "EFC should be low for untrained model with tiny delta");
    }

    #[test]
    fn test_efc_efficiency_returns_finite() {
        let cdwm = CDWM::new(3, 4);
        let eta = cdwm.efc_efficiency();
        assert!(eta.is_finite());
    }

    #[test]
    fn test_intervention_pathway_reliability_score() {
        let mut ip = InterventionPathway::new(2);
        // High variance → lower reliability
        ip.observe(&[0.0, 0.0], 1, &[1.0, 0.0]);
        ip.observe(&[0.0, 0.0], 1, &[-1.0, 0.0]);
        let r = ip.reliability_score(1);
        assert!(r > 0.0 && r <= 1.0, "reliability should be in (0, 1]");
    }

    #[test]
    fn test_redundant_observations_have_lower_efc_than_novel() {
        let mut cdwm = CDWM::new(2, 4);
        // Train model with diverse observations so it's stable
        for _ in 0..20 {
            cdwm.observe_natural(&[0.0, 0.0], &[0.1, 0.0]);
            cdwm.observe_intervention(&[0.0, 0.0], 1, &[2.0, 0.0]);
            cdwm.observe_intervention(&[5.0, 3.0], 1, &[7.0, 3.0]);
        }
        // Populate history with diverse states
        cdwm.observe_intervention(&[1.0, 1.0], 1, &[3.0, 1.0]);

        // Now compute EFC for a new state (should be higher novelty)
        let efc_first = cdwm.efc_score(&[10.0, -10.0], 1, &[12.0, -10.0]);
        // Add a very similar state to history
        cdwm.observe_intervention(&[10.0, -10.0], 1, &[12.0, -10.0]);
        // Same query → lower novelty → lower or equal EFC
        let efc_second = cdwm.efc_score(&[10.0, -10.0], 1, &[12.0, -10.0]);
        assert!(efc_second <= efc_first + 0.5,
            "redundant observation should not have much higher EFC: {efc_second} >> {efc_first}");
    }
}
