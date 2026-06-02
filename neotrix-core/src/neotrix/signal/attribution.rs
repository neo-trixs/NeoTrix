//! SIGReg 和归因系统
use serde::{Deserialize, Serialize};
use rand::Rng;
use super::core::Vector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SIGReg {
    pub num_projections: usize,
    pub lambda: f64,
    random_directions: Vec<Vector>,
}

impl SIGReg {
    pub fn new(num_projections: usize, lambda: f64, embed_dim: usize) -> Self {
        let random_directions = (0..num_projections)
            .map(|_| Self::random_unit_vector(embed_dim))
            .collect();
        Self {
            num_projections,
            lambda,
            random_directions,
        }
    }

    fn random_unit_vector(dim: usize) -> Vector {
        let mut rng = rand::thread_rng();
        let mut v: Vector = (0..dim).map(|_| rng.gen::<f64>()).collect();
        let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 0.0 {
            for x in &mut v {
                *x /= norm;
            }
        }
        v
    }

    /// Epps-Pulley 正态性检验统计量
    fn epps_pulley_statistic(sample: &[f64]) -> f64 {
        let n = sample.len() as f64;
        if n < 5.0 {
            return 0.0;
        }
        let sample_mean: f64 = sample.iter().sum::<f64>() / n;
        let sample_std = (sample.iter().map(|x| (x - sample_mean).powi(2)).sum::<f64>() / n).sqrt();
        if sample_std <= 0.0 {
            return 0.0;
        }
        let standardized: Vector = sample.iter().map(|x| (x - sample_mean) / sample_std).collect();
        let m3: f64 = standardized.iter().map(|x| x.powi(3)).sum::<f64>() / n;
        let m6: f64 = standardized.iter().map(|x| x.powi(6)).sum::<f64>() / n;
        let skewness = m3.powi(2);
        let kurtosis = m6 - 3.0;
        skewness + 0.25 * kurtosis.powi(2)
    }

    /// 计算 SIGReg 损失
    pub fn compute_loss(&self, embeddings: &[Vector]) -> f64 {
        if embeddings.is_empty() {
            return 0.0;
        }
        let total_loss: f64 = self.random_directions.iter()
            .map(|direction| {
                let projected: Vector = embeddings.iter()
                    .map(|emb| Self::dot_product(emb, direction))
                    .collect();
                Self::epps_pulley_statistic(&projected)
            })
            .sum();
        total_loss / self.num_projections as f64
    }

    fn dot_product(a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// LeWM 2-term 损失: pred_loss + λ * sigreg_loss
    pub fn lewm_loss(
        &self,
        pred_embeddings: &[Vector],
        target_embeddings: &[Vector],
    ) -> (f64, f64, f64) {
        assert_eq!(pred_embeddings.len(), target_embeddings.len());
        let n = pred_embeddings.len();
        if n == 0 {
            return (0.0, 0.0, 0.0);
        }
        let mut pred_loss = 0.0;
        for (pred, tgt) in pred_embeddings.iter().zip(target_embeddings.iter()) {
            let diff: f64 = pred.iter().zip(tgt.iter()).map(|(p, t)| (p - t).powi(2)).sum();
            pred_loss += diff;
        }
        pred_loss /= n as f64;
        let sigreg_loss = self.compute_loss(pred_embeddings);
        let total_loss = pred_loss + self.lambda * sigreg_loss;
        (total_loss, pred_loss, sigreg_loss)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributionSource {
    User,
    LLMGenerated,
    MemoryRetrieved,
    External,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionSummary {
    pub ai_contribution: f64,
    pub user_input: f64,
    pub memory_retrieved: f64,
    pub source: AttributionSource,
    pub trace_id: Option<String>,
    pub timestamp: i64,
}

impl AttributionSummary {
    pub fn new(
        ai_contribution: f64,
        user_input: f64,
        memory_retrieved: f64,
        source: AttributionSource,
    ) -> Self {
        Self {
            ai_contribution,
            user_input,
            memory_retrieved,
            source,
            trace_id: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigreg_new() {
        let reg = SIGReg::new(10, 0.1, 8);
        assert_eq!(reg.num_projections, 10);
        assert_eq!(reg.lambda, 0.1);
        assert_eq!(reg.random_directions.len(), 10);
        for dir in &reg.random_directions {
            assert_eq!(dir.len(), 8);
            let norm: f64 = dir.iter().map(|x| x * x).sum::<f64>().sqrt();
            assert!((norm - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn test_sigreg_compute_loss_empty() {
        let reg = SIGReg::new(5, 0.1, 4);
        let loss = reg.compute_loss(&[]);
        assert!((loss - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_sigreg_compute_loss_single_embedding() {
        let reg = SIGReg::new(3, 0.1, 4);
        let embeddings = vec![vec![1.0, 0.0, 0.0, 0.0]];
        let loss = reg.compute_loss(&embeddings);
        assert!(!loss.is_nan());
    }

    #[test]
    fn test_sigreg_dot_product() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let dp = SIGReg::dot_product(&a, &b);
        assert!((dp - 32.0).abs() < 1e-9);
    }

    #[test]
    fn test_epps_pulley_small_sample() {
        let sample = vec![1.0, 2.0];
        let stat = SIGReg::epps_pulley_statistic(&sample);
        assert!((stat - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_epps_pulley_normal_like() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let sample: Vec<f64> = (0..100).map(|_| rng.gen::<f64>()).collect();
        let stat = SIGReg::epps_pulley_statistic(&sample);
        assert!(stat >= 0.0);
    }

    #[test]
    fn test_lewm_loss_empty() {
        let reg = SIGReg::new(5, 0.1, 4);
        let (total, pred, sig) = reg.lewm_loss(&[], &[]);
        assert!((total - 0.0).abs() < 1e-9);
        assert!((pred - 0.0).abs() < 1e-9);
        assert!((sig - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_lewm_loss_identical() {
        let reg = SIGReg::new(3, 0.1, 4);
        let pred = vec![vec![1.0, 0.0, 0.0, 0.0]];
        let target = vec![vec![1.0, 0.0, 0.0, 0.0]];
        let (total, pred_loss, _) = reg.lewm_loss(&pred, &target);
        assert!((pred_loss - 0.0).abs() < 1e-9);
        assert!(total >= 0.0);
    }

    #[test]
    fn test_attribution_summary_new() {
        let s = AttributionSummary::new(0.7, 0.2, 0.1, AttributionSource::User);
        assert!((s.ai_contribution - 0.7).abs() < 1e-9);
        assert!((s.user_input - 0.2).abs() < 1e-9);
    }

    #[test]
    fn test_attribution_source_variants() {
        let sources = vec![
            AttributionSource::User,
            AttributionSource::LLMGenerated,
            AttributionSource::MemoryRetrieved,
            AttributionSource::External,
            AttributionSource::Mixed,
        ];
        assert_eq!(sources.len(), 5);
    }
}
