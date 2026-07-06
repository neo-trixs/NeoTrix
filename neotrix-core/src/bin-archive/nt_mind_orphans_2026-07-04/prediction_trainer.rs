use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::neotrix::nt_world_jepa::JepaWorldModel;

const FEATURE_DIM: usize = 32;
const MAX_TRAINING_PAIRS: usize = 200;
const HISTORY_SIZE: usize = 20;

fn text_to_seed(text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

pub struct FeatureExtractor;

impl FeatureExtractor {
    pub fn from_text(text: &str, dim: usize) -> Vec<f64> {
        let seed = text_to_seed(text);
        let bytes = QuantizedVSA::seeded_random(seed, 4096);
        let mut features = vec![0.0_f64; dim];
        for i in 0..dim {
            let idx = (i * 4096 / dim) % 4096;
            let byte_idx = idx / 8;
            let bit_idx = idx % 8;
            let bit = if byte_idx < bytes.len() {
                (bytes[byte_idx] >> bit_idx) & 1
            } else {
                0
            };
            features[i] = if bit == 1 { 1.0 } else { -1.0 };
        }
        features
    }

    pub fn from_node(
        node: &crate::neotrix::nt_memory_kb::nt_memory_types::KnowledgeNode,
    ) -> Vec<f64> {
        let text = format!(
            "{} {} {:?} {}",
            node.title,
            node.summary.as_deref().unwrap_or(""),
            node.node_type,
            node.domain.as_deref().unwrap_or(""),
        );
        Self::from_text(&text, FEATURE_DIM)
    }
}

#[derive(Debug, Clone)]
pub struct TrainingMetrics {
    pub total_pairs: usize,
    pub avg_loss: f64,
    pub avg_energy: f64,
    pub accuracy: f64,
    pub recent_accuracy: Vec<f64>,
    pub prediction_error: f64,
}

#[derive(Debug, Clone)]
pub struct PredictionTrainer {
    pub training_history: VecDeque<TrainingMetrics>,
    pub total_trained: usize,
    pub accuracy_trend: Vec<f64>,
}

impl PredictionTrainer {
    pub fn new() -> Self {
        Self {
            training_history: VecDeque::with_capacity(HISTORY_SIZE),
            total_trained: 0,
            accuracy_trend: Vec::new(),
        }
    }

    pub fn extract_training_pairs(
        &self,
        kb: &crate::neotrix::nt_memory_kb::KnowledgeBase,
    ) -> Vec<(Vec<f64>, Vec<f64>)> {
        let mut pairs = Vec::new();
        let conn = match kb.conn.lock() {
            Ok(c) => c,
            Err(_) => return pairs,
        };

        let mut stmt = match conn.prepare(
            "SELECT n1.id, n1.title, n1.summary, n1.node_type, n1.domain,
                    n2.id, n2.title, n2.summary, n2.node_type, n2.domain
             FROM edges e
             JOIN nodes n1 ON n1.id = e.source_id
             JOIN nodes n2 ON n2.id = e.target_id
             WHERE e.weight > 0.5
             ORDER BY RANDOM()
             LIMIT ?1",
        ) {
            Ok(s) => s,
            Err(_) => return pairs,
        };

        let limit = MAX_TRAINING_PAIRS as i64;
        let rows = match stmt.query_map([limit], |row| {
            let s_id: String = row.get(0)?;
            let s_title: String = row.get(1)?;
            let s_summary: Option<String> = row.get(2)?;
            let s_type: String = row.get(3)?;
            let s_domain: Option<String> = row.get(4)?;
            let t_id: String = row.get(5)?;
            let t_title: String = row.get(6)?;
            let t_summary: Option<String> = row.get(7)?;
            let t_type: String = row.get(8)?;
            let t_domain: Option<String> = row.get(9)?;
            Ok((
                s_id, s_title, s_summary, s_type, s_domain,
                t_id, t_title, t_summary, t_type, t_domain,
            ))
        }) {
            Ok(r) => r,
            Err(_) => return pairs,
        };

        for row in rows.flatten() {
            let (_s_id, s_title, s_summary, s_type, s_domain,
                 _t_id, t_title, t_summary, t_type, t_domain) = row;

            let source_text = format!(
                "{} {} {} {}",
                s_title,
                s_summary.unwrap_or_default(),
                s_type,
                s_domain.unwrap_or_default(),
            );
            let target_text = format!(
                "{} {} {} {}",
                t_title,
                t_summary.unwrap_or_default(),
                t_type,
                t_domain.unwrap_or_default(),
            );

            let ctx = FeatureExtractor::from_text(&source_text, FEATURE_DIM);
            let tgt = FeatureExtractor::from_text(&target_text, FEATURE_DIM);
            pairs.push((ctx, tgt));
        }

        pairs
    }

    pub fn train_jepa(
        &self,
        jepa: &mut JepaWorldModel,
        pairs: &[(Vec<f64>, Vec<f64>)],
    ) -> TrainingMetrics {
        if pairs.is_empty() {
            return TrainingMetrics {
                total_pairs: 0,
                avg_loss: 0.0,
                avg_energy: 0.0,
                accuracy: 0.0,
                recent_accuracy: Vec::new(),
                prediction_error: 0.0,
            };
        }

        let batch_x: Vec<Vec<f64>> = pairs.iter().map(|(x, _)| x.clone()).collect();
        let batch_y: Vec<Vec<f64>> = pairs.iter().map(|(_, y)| y.clone()).collect();

        let avg_loss = jepa.train_batch(&batch_x, &batch_y);

        let mut total_energy = 0.0;
        let mut correct = 0;
        for (ctx, tgt) in pairs {
            let (pred, energy) = jepa.predict(ctx);
            total_energy += energy;
            let sim = cosine_similarity(&pred, tgt);
            if sim > 0.7 {
                correct += 1;
            }
        }
        let n = pairs.len() as f64;
        let accuracy = if n > 0.0 { correct as f64 / n } else { 0.0 };
        let avg_energy = if n > 0.0 { total_energy / n } else { 0.0 };

        TrainingMetrics {
            total_pairs: pairs.len(),
            avg_loss,
            avg_energy,
            accuracy,
            recent_accuracy: Vec::new(),
            prediction_error: 1.0 - accuracy,
        }
    }

    pub fn train_on_kb(
        &mut self,
        jepa: &mut JepaWorldModel,
        kb: &crate::neotrix::nt_memory_kb::KnowledgeBase,
    ) -> TrainingMetrics {
        let pairs = self.extract_training_pairs(kb);
        let metrics = self.train_jepa(jepa, &pairs);
        self.total_trained += 1;
        self.accuracy_trend.push(metrics.accuracy);
        if self.accuracy_trend.len() > 20 {
            self.accuracy_trend.remove(0);
        }
        self.training_history.push_back(metrics.clone());
        if self.training_history.len() > HISTORY_SIZE {
            self.training_history.pop_front();
        }
        metrics
    }

    pub fn accuracy_trend_slope(&self) -> f64 {
        let n = self.accuracy_trend.len();
        if n < 3 {
            return 0.0;
        }
        let indices: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let vals = &self.accuracy_trend;
        let mean_x = indices.iter().sum::<f64>() / n as f64;
        let mean_y = vals.iter().sum::<f64>() / n as f64;
        let num: f64 = indices.iter().zip(vals.iter()).map(|(x, y)| (x - mean_x) * (y - mean_y)).sum();
        let den: f64 = indices.iter().map(|x| (x - mean_x).powi(2)).sum();
        if den.abs() < 1e-12 { 0.0 } else { num / den }
    }

    pub fn prediction_quality(&self) -> f64 {
        if self.training_history.is_empty() {
            return 0.5;
        }
        let recent: Vec<&TrainingMetrics> = self.training_history.iter().rev().take(5).collect();
        if recent.is_empty() {
            return 0.5;
        }
        let avg_acc: f64 = recent.iter().map(|m| m.accuracy).sum::<f64>() / recent.len() as f64;
        let slope = self.accuracy_trend_slope();
        (avg_acc * 0.7 + (slope * 0.5 + 0.5).clamp(0.0, 1.0) * 0.3).clamp(0.0, 1.0)
    }
}

impl Default for PredictionTrainer {
    fn default() -> Self {
        Self::new()
    }
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f64 = a.iter().map(|x| x * x).sum();
    let nb: f64 = b.iter().map(|x| x * x).sum();
    let mag = na.sqrt() * nb.sqrt();
    if mag < 1e-12 { 0.0 } else { (dot / mag).clamp(-1.0, 1.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_extraction() {
        let features = FeatureExtractor::from_text("Test node title", FEATURE_DIM);
        assert_eq!(features.len(), FEATURE_DIM);
        for &v in &features {
            assert!(v == 1.0 || v == -1.0);
        }
    }

    #[test]
    fn test_feature_deterministic() {
        let a = FeatureExtractor::from_text("hello world", FEATURE_DIM);
        let b = FeatureExtractor::from_text("hello world", FEATURE_DIM);
        assert_eq!(a, b);
    }

    #[test]
    fn test_feature_different_texts() {
        let a = FeatureExtractor::from_text("AAA", FEATURE_DIM);
        let b = FeatureExtractor::from_text("BBB", FEATURE_DIM);
        assert_ne!(a, b);
    }

    #[test]
    fn test_train_jepa_empty() {
        let mut jepa = JepaWorldModel::new(FEATURE_DIM);
        let trainer = PredictionTrainer::new();
        let metrics = trainer.train_jepa(&mut jepa, &[]);
        assert_eq!(metrics.total_pairs, 0);
    }

    #[test]
    fn test_train_jepa_basic() {
        let mut jepa = JepaWorldModel::new(FEATURE_DIM);
        let trainer = PredictionTrainer::new();
        let pairs = vec![
            (vec![1.0; FEATURE_DIM], vec![1.0; FEATURE_DIM]),
            (vec![-1.0; FEATURE_DIM], vec![-1.0; FEATURE_DIM]),
        ];
        let metrics = trainer.train_jepa(&mut jepa, &pairs);
        assert!(metrics.total_pairs > 0);
        assert!(metrics.avg_loss.is_finite());
    }

    #[test]
    fn test_prediction_quality_init() {
        let trainer = PredictionTrainer::new();
        assert!((trainer.prediction_quality() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_accuracy_trend_slope_empty() {
        let trainer = PredictionTrainer::new();
        assert!((trainer.accuracy_trend_slope() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_accuracy_trend_slope_positive() {
        let mut trainer = PredictionTrainer::new();
        trainer.accuracy_trend = vec![0.1, 0.3, 0.5, 0.7, 0.9];
        assert!(trainer.accuracy_trend_slope() > 0.0);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 2.0];
        let b = vec![-1.0, -2.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - (-1.0)).abs() < 1e-10);
    }
}
