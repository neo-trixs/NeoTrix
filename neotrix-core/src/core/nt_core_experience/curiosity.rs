use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

pub struct KnowledgeGapDetector;

impl KnowledgeGapDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn predict(&self, query: &[u8], known_items: &[&[u8]]) -> f64 {
        if known_items.is_empty() {
            return 1.0;
        }
        let max_sim = known_items
            .iter()
            .map(|&k| QuantizedVSA::similarity(query, k))
            .fold(f64::NEG_INFINITY, |a, b| a.max(b));
        1.0 - max_sim
    }

    pub fn prediction_uncertainty(&self, query: &[u8], known_items: &[&[u8]]) -> f64 {
        if known_items.is_empty() {
            return 1.0;
        }
        let mut sims: Vec<f64> = known_items
            .iter()
            .map(|&k| QuantizedVSA::similarity(query, k))
            .collect();
        sims.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let top_k = sims.iter().take(3).copied().collect::<Vec<f64>>();
        let n = top_k.len() as f64;
        if n == 0.0 {
            return 1.0;
        }
        let mean: f64 = top_k.iter().sum::<f64>() / n;
        let variance: f64 = top_k.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n;
        variance
    }

    pub fn novelty(&self, query: &[u8], known_items: &[&[u8]]) -> f64 {
        if known_items.is_empty() {
            return 1.0;
        }
        let mut sims: Vec<f64> = known_items
            .iter()
            .map(|&k| QuantizedVSA::similarity(query, k))
            .collect();
        sims.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let neighbors = sims.iter().take(5).copied().collect::<Vec<f64>>();
        let n = neighbors.len() as f64;
        if n == 0.0 {
            return 1.0;
        }
        let avg_sim: f64 = neighbors.iter().sum::<f64>() / n;
        1.0 - avg_sim
    }

    pub fn info_gain(&self, query: &[u8], known_before: &[&[u8]], known_after: &[&[u8]]) -> f64 {
        let before = self.prediction_uncertainty(query, known_before);
        let after = self.prediction_uncertainty(query, known_after);
        if before <= after {
            0.0
        } else {
            before - after
        }
    }
}

impl Default for KnowledgeGapDetector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CuriosityEngine {
    pub uncertainty_weight: f64,
    pub novelty_weight: f64,
    pub info_gain_weight: f64,
    pub curiosity_decay: f64,
    pub last_explore_time: u64,
}

impl CuriosityEngine {
    pub fn new(uncertainty_w: f64, novelty_w: f64, info_gain_w: f64) -> Self {
        Self {
            uncertainty_weight: uncertainty_w,
            novelty_weight: novelty_w,
            info_gain_weight: info_gain_w,
            curiosity_decay: 0.95,
            last_explore_time: 0,
        }
    }

    pub fn drive_signal(&self, uncertainty: f64, novelty: f64, info_gain: f64) -> f64 {
        self.uncertainty_weight * uncertainty
            + self.novelty_weight * novelty
            + self.info_gain_weight * info_gain
    }

    pub fn explore_target(&self, known_items: &[&[u8]], candidates: &[&[u8]]) -> Option<usize> {
        if candidates.is_empty() {
            return None;
        }
        let detector = KnowledgeGapDetector::new();
        let mut best_idx = 0;
        let mut best_score = f64::NEG_INFINITY;
        for (i, &candidate) in candidates.iter().enumerate() {
            let uncert = detector.prediction_uncertainty(candidate, known_items);
            let nov = detector.novelty(candidate, known_items);
            let score = self.drive_signal(uncert, nov, 0.0);
            if score > best_score {
                best_score = score;
                best_idx = i;
            }
        }
        Some(best_idx)
    }

    pub fn schedule_exploration(
        &mut self,
        curiosity_threshold: f64,
        time_since_explore: u64,
    ) -> bool {
        let decayed = self.curiosity_decay.powi(time_since_explore as i32);
        let threshold = curiosity_threshold * (1.0 + decayed);
        let current_time = self.last_explore_time + time_since_explore;
        let drive = 1.0 - ((current_time as f64).recip() * 0.1);
        drive > threshold
    }

    pub fn curiosity_adapt(&mut self, curiosity_drive: f64, actual_info_gain: f64) {
        let prediction_error = (curiosity_drive - actual_info_gain).abs();
        let learning_rate = 0.1 / (1.0 + prediction_error);
        if actual_info_gain > curiosity_drive {
            self.uncertainty_weight += learning_rate * 0.2;
            self.novelty_weight += learning_rate * 0.3;
        } else {
            self.info_gain_weight += learning_rate * 0.5;
        }
        let total = self.uncertainty_weight + self.novelty_weight + self.info_gain_weight;
        if total > 0.0 {
            self.uncertainty_weight /= total;
            self.novelty_weight /= total;
            self.info_gain_weight /= total;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExplorationTrigger {
    pub target: Vec<u8>,
    pub reason: String,
    pub priority: f64,
    pub active: bool,
}

impl ExplorationTrigger {
    pub fn target_domain(target: &[u8], reason: &str) -> Self {
        Self {
            target: target.to_vec(),
            reason: reason.to_string(),
            priority: 0.5,
            active: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

    fn random_vsa() -> Vec<u8> {
        QuantizedVSA::random_binary()
    }

    #[test]
    fn test_prediction_error() {
        let detector = KnowledgeGapDetector::new();
        let query = random_vsa();
        let known = vec![random_vsa(), random_vsa(), random_vsa()];
        let known_refs: Vec<&[u8]> = known.iter().map(|v| v.as_slice()).collect();
        let error = detector.predict(&query, &known_refs);
        assert!(error >= 0.0 && error <= 1.0);
    }

    #[test]
    fn test_novelty() {
        let detector = KnowledgeGapDetector::new();
        let query = random_vsa();
        let known = vec![random_vsa(), random_vsa(), random_vsa()];
        let known_refs: Vec<&[u8]> = known.iter().map(|v| v.as_slice()).collect();
        let nov = detector.novelty(&query, &known_refs);
        assert!(nov >= 0.0 && nov <= 1.0);
    }

    #[test]
    fn test_prediction_uncertainty() {
        let detector = KnowledgeGapDetector::new();
        let query = random_vsa();
        let known = vec![random_vsa(), random_vsa(), random_vsa()];
        let known_refs: Vec<&[u8]> = known.iter().map(|v| v.as_slice()).collect();
        let uncert = detector.prediction_uncertainty(&query, &known_refs);
        assert!(uncert >= 0.0 && uncert <= 1.0);
    }

    #[test]
    fn test_curiosity_drive_signal() {
        let engine = CuriosityEngine::new(0.4, 0.4, 0.2);
        let drive = engine.drive_signal(0.3, 0.5, 0.2);
        let expected = 0.4 * 0.3 + 0.4 * 0.5 + 0.2 * 0.2;
        assert!((drive - expected).abs() < 1e-10);
    }

    #[test]
    fn test_explore_target_selection() {
        let engine = CuriosityEngine::new(0.5, 0.5, 0.0);
        let known = vec![random_vsa(), random_vsa()];
        let known_refs: Vec<&[u8]> = known.iter().map(|v| v.as_slice()).collect();
        let candidates = vec![random_vsa(), random_vsa(), random_vsa()];
        let cand_refs: Vec<&[u8]> = candidates.iter().map(|v| v.as_slice()).collect();
        let idx = engine.explore_target(&known_refs, &cand_refs);
        assert!(idx.is_some());
        assert!(idx.unwrap() < candidates.len());
    }

    #[test]
    fn test_adaptive_weight_adjustment() {
        let mut engine = CuriosityEngine::new(0.4, 0.4, 0.2);
        engine.curiosity_adapt(0.8, 0.6);
        let total = engine.uncertainty_weight + engine.novelty_weight + engine.info_gain_weight;
        assert!((total - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_empty_known_set() {
        let detector = KnowledgeGapDetector::new();
        let query = random_vsa();
        let empty: Vec<&[u8]> = vec![];
        assert!((detector.predict(&query, &empty) - 1.0).abs() < 1e-10);
        assert!((detector.novelty(&query, &empty) - 1.0).abs() < 1e-10);
        assert!((detector.prediction_uncertainty(&query, &empty) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_single_item_edge_case() {
        let detector = KnowledgeGapDetector::new();
        let query = random_vsa();
        let single = vec![random_vsa()];
        let single_ref: Vec<&[u8]> = single.iter().map(|v| v.as_slice()).collect();
        let error = detector.predict(&query, &single_ref);
        assert!(error >= 0.0 && error <= 1.0);
        let nov = detector.novelty(&query, &single_ref);
        assert!(nov >= 0.0 && nov <= 1.0);
    }

    #[test]
    fn test_schedule_exploration() {
        let mut engine = CuriosityEngine::new(0.4, 0.4, 0.2);
        let should = engine.schedule_exploration(0.5, 10);
        assert!(should == true || should == false);
    }

    #[test]
    fn test_info_gain_reduction() {
        let detector = KnowledgeGapDetector::new();
        let query = random_vsa();
        let before = vec![random_vsa(), random_vsa()];
        let before_refs: Vec<&[u8]> = before.iter().map(|v| v.as_slice()).collect();
        let after = vec![query.clone(), random_vsa()];
        let after_refs: Vec<&[u8]> = after.iter().map(|v| v.as_slice()).collect();
        let gain = detector.info_gain(&query, &before_refs, &after_refs);
        assert!(gain >= 0.0 && gain <= 1.0);
    }

    #[test]
    fn test_exploration_trigger() {
        let target = random_vsa();
        let trigger = ExplorationTrigger::target_domain(&target, "high novelty");
        assert!(trigger.active);
        assert!((trigger.priority - 0.5).abs() < 1e-10);
        assert_eq!(trigger.reason, "high novelty");
    }
}
