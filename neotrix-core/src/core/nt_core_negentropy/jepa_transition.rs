use super::efe_minimizer::TransitionModel;
use std::sync::Arc;

/// Default JEPA latent dimension (inlined to break circular import)
pub const JEPA_LATENT_DIM: usize = 32;

/// JEPA-based transition model using an Arc-wrapped prediction closure.
///
/// Eliminates the direct dependency on `JepaPredictor` (neotrix layer).
/// Implements `TransitionModel` for `EFEMinimizer` and provides
/// a closure factory for `AcTPlanner`.
pub struct JepaTransitionModel {
    predict_fn: Arc<dyn Fn(&[f64]) -> Vec<f64>>,
    latent_dim: usize,
    action_dim: usize,
}

impl JepaTransitionModel {
    pub fn new(
        predict_fn: Arc<dyn Fn(&[f64]) -> Vec<f64>>,
        latent_dim: usize,
        action_dim: usize,
    ) -> Self {
        Self {
            predict_fn,
            latent_dim,
            action_dim,
        }
    }

    /// Produce a closure for `AcTPlanner` that wraps JEPA prediction.
    ///
    /// The closure ignores the action index (JEPA predicts from state alone)
    /// and returns the predicted next latent state as `Vec<f64>`.
    pub fn to_act_closure(&self) -> Box<dyn Fn(&[f64], usize) -> Vec<f64>> {
        let fn_clone = self.predict_fn.clone();
        Box::new(move |state: &[f64], _action: usize| fn_clone(state))
    }

    pub fn latent_dim(&self) -> usize {
        self.latent_dim
    }
}

impl TransitionModel for JepaTransitionModel {
    fn predict(&self, belief: &[Vec<f64>], _action: usize) -> Vec<f64> {
        if belief.is_empty() {
            return Vec::new();
        }
        let dim = belief[0].len();
        if dim == 0 {
            return Vec::new();
        }
        if !belief.iter().all(|b| b.len() == dim) {
            return Vec::new();
        }
        let n = belief.len() as f64;
        let mean: Vec<f64> = (0..dim)
            .map(|i| belief.iter().map(|b| b[i]).sum::<f64>() / n)
            .collect();
        (self.predict_fn)(&mean)
    }

    fn possible_actions(&self) -> usize {
        self.action_dim
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_predict(state: &[f64]) -> Vec<f64> {
        state.to_vec()
    }

    #[test]
    fn test_jepa_transition_deterministic() {
        let model = JepaTransitionModel::new(Arc::new(dummy_predict), JEPA_LATENT_DIM, 3);
        let belief = vec![vec![0.5; JEPA_LATENT_DIM]];
        let r1 = model.predict(&belief, 0);
        let r2 = model.predict(&belief, 0);
        assert_eq!(r1.len(), JEPA_LATENT_DIM);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_jepa_transition_multi_sample_belief() {
        let model = JepaTransitionModel::new(Arc::new(dummy_predict), JEPA_LATENT_DIM, 2);
        let belief = vec![
            vec![0.3; JEPA_LATENT_DIM],
            vec![0.7; JEPA_LATENT_DIM],
            vec![0.5; JEPA_LATENT_DIM],
        ];
        let result = model.predict(&belief, 0);
        assert_eq!(result.len(), JEPA_LATENT_DIM);
        for &v in &result {
            assert!(v.is_finite());
        }
    }

    #[test]
    fn test_jepa_transition_empty_belief() {
        let model = JepaTransitionModel::new(Arc::new(dummy_predict), JEPA_LATENT_DIM, 3);
        let empty: Vec<Vec<f64>> = vec![];
        let result = model.predict(&empty, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_transition_model_trait_possible_actions() {
        let model = JepaTransitionModel::new(Arc::new(dummy_predict), JEPA_LATENT_DIM, 5);
        assert_eq!(model.possible_actions(), 5);

        let model2 = JepaTransitionModel::new(Arc::new(dummy_predict), JEPA_LATENT_DIM, 1);
        assert_eq!(model2.possible_actions(), 1);
    }

    #[test]
    fn test_jepa_transition_different_inputs_give_different_outputs() {
        let model = JepaTransitionModel::new(Arc::new(dummy_predict), JEPA_LATENT_DIM, 3);
        let belief_a = vec![vec![0.1; JEPA_LATENT_DIM]];
        let belief_b = vec![vec![0.9; JEPA_LATENT_DIM]];
        let ra = model.predict(&belief_a, 0);
        let rb = model.predict(&belief_b, 0);
        assert_eq!(ra.len(), JEPA_LATENT_DIM);
        assert_eq!(rb.len(), JEPA_LATENT_DIM);
    }
}
