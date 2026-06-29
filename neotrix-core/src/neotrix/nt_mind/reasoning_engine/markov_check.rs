pub struct MarkovCheck {
    pub inverse_accuracy: f64,
    pub contrastive_score: f64,
    pub check_count: u64,
    recent_inverse: Vec<f64>,
    recent_contrastive: Vec<f64>,
    window_size: usize,
}

impl MarkovCheck {
    pub fn new() -> Self {
        Self {
            inverse_accuracy: 0.0,
            contrastive_score: 0.0,
            check_count: 0,
            recent_inverse: Vec::with_capacity(100),
            recent_contrastive: Vec::with_capacity(100),
            window_size: 100,
        }
    }

    fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        (dot / (norm_a * norm_b)).max(0.0).min(1.0)
    }

    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    pub fn check_inverse(&mut self, z_prev: &[f64], action: u8, z_next: &[f64]) -> f64 {
        let cos_sim = Self::cosine_similarity(z_prev, z_next);
        let action_encoding = (action as f64 + 1.0) / 64.0;
        let accuracy = Self::sigmoid(cos_sim * action_encoding);
        let clamped = accuracy.max(0.0).min(1.0);
        self.recent_inverse.push(clamped);
        if self.recent_inverse.len() > self.window_size {
            self.recent_inverse.remove(0);
        }
        self.inverse_accuracy = if self.recent_inverse.is_empty() {
            clamped
        } else {
            self.recent_inverse.iter().sum::<f64>() / self.recent_inverse.len() as f64
        };
        clamped
    }

    pub fn check_contrastive(&mut self, z_prev: &[f64], z_next: &[f64], random_z: &[f64]) -> f64 {
        let pos_sim = Self::cosine_similarity(z_prev, z_next);
        let neg_sim = Self::cosine_similarity(z_prev, random_z);
        let score = ((pos_sim - neg_sim) + 1.0) / 2.0;
        let clamped = score.max(0.0).min(1.0);
        self.recent_contrastive.push(clamped);
        if self.recent_contrastive.len() > self.window_size {
            self.recent_contrastive.remove(0);
        }
        self.contrastive_score = if self.recent_contrastive.is_empty() {
            clamped
        } else {
            self.recent_contrastive.iter().sum::<f64>() / self.recent_contrastive.len() as f64
        };
        clamped
    }

    pub fn evaluate(
        &mut self,
        z_prev: &[f64],
        action: u8,
        z_next: &[f64],
        random_z: &[f64],
    ) -> f64 {
        let inv = self.check_inverse(z_prev, action, z_next);
        let cont = self.check_contrastive(z_prev, z_next, random_z);
        self.check_count += 1;
        (inv * 0.6 + cont * 0.4).max(0.0).min(1.0)
    }

    pub fn markov_score(&self) -> f64 {
        (self.inverse_accuracy * 0.6 + self.contrastive_score * 0.4)
            .max(0.0)
            .min(1.0)
    }

    pub fn is_markovian(&self) -> bool {
        self.markov_score() > 0.5
    }
}

impl Default for MarkovCheck {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similar_states_same_action_high_inverse_accuracy() {
        let mut mc = MarkovCheck::new();
        let z1 = vec![0.8, 0.2, 0.5, 0.1];
        let z2 = vec![0.79, 0.21, 0.51, 0.09];
        let acc = mc.check_inverse(&z1, 42, &z2);
        assert!(
            acc > 0.5,
            "similar states should have high inverse accuracy, got {}",
            acc
        );
    }

    #[test]
    fn test_different_states_lower_inverse_accuracy() {
        let mut mc = MarkovCheck::new();
        let z1 = vec![0.8, 0.2, 0.5, 0.1];
        let z2 = vec![0.1, 0.5, 0.2, 0.8];
        let z_sim = vec![0.79, 0.21, 0.51, 0.09];
        let acc_diff = mc.check_inverse(&z1, 42, &z2);
        let acc_sim = mc.check_inverse(&z1, 42, &z_sim);
        assert!(acc_sim > acc_diff, "similar states should have higher inverse accuracy than different states (sim={}, diff={})", acc_sim, acc_diff);
    }

    #[test]
    fn test_contrastive_score_higher_for_temporally_close_states() {
        let mut mc = MarkovCheck::new();
        let z_prev = vec![0.5, 0.3, 0.7];
        let z_next = vec![0.52, 0.29, 0.71];
        let z_random = vec![-0.5, -0.3, -0.7];
        let score = mc.check_contrastive(&z_prev, &z_next, &z_random);
        assert!(
            score > 0.5,
            "temporally close states should have high contrastive score, got {}",
            score
        );
    }

    #[test]
    fn test_contrastive_score_lower_for_random_pairs() {
        let mut mc = MarkovCheck::new();
        let z_prev = vec![0.5, 0.3, 0.7];
        let z_next = vec![-0.5, -0.3, -0.7];
        let z_random = vec![0.5, 0.3, 0.7];
        let score = mc.check_contrastive(&z_prev, &z_next, &z_random);
        assert!(
            score < 0.5,
            "dissimilar pairs should have low contrastive score, got {}",
            score
        );
    }

    #[test]
    fn test_markov_score_combines_metrics() {
        let mut mc = MarkovCheck::new();
        let z1 = vec![0.8, 0.2, 0.5, 0.1];
        let z2 = vec![0.79, 0.21, 0.51, 0.09];
        let zr = vec![-0.5, -0.3, -0.7];
        let score = mc.evaluate(&z1, 42, &z2, &zr);
        assert!(
            score > 0.0 && score <= 1.0,
            "markov score should be in [0,1], got {}",
            score
        );
    }

    #[test]
    fn test_is_markovian_threshold() {
        let mc = MarkovCheck::new();
        assert!(!mc.is_markovian(), "fresh check should not be markovian");
    }

    #[test]
    fn test_default_impl() {
        let mc = MarkovCheck::default();
        assert_eq!(mc.inverse_accuracy, 0.0);
        assert_eq!(mc.contrastive_score, 0.0);
        assert_eq!(mc.check_count, 0);
    }
}
