//! Crystal Metrics — 晶体度量指标

use serde::{Deserialize, Serialize};

/// 晶体度量三元组
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrystalMetrics {
    /// CII — Crystal Integration Index (晶体整合指数)
    pub cii: f64,
    /// CI — Crystal Coherence (晶体一致性)
    pub ci: f64,
    /// CB — Crystal Balance (晶体平衡度)
    pub cb: f64,
}

impl CrystalMetrics {
    pub fn compute(cii: f64, ci: f64, cb: f64) -> Self {
        Self { cii, ci, cb }
    }

    pub fn health_score(&self) -> f64 {
        (self.cii * 0.4 + self.ci * 0.35 + self.cb * 0.25).clamp(0.0, 1.0)
    }
}
