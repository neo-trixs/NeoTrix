use super::crystal_state::CrystalState;

/// 晶体度量指标
#[derive(Debug, Clone, Copy)]
pub struct CrystalMetrics {
    pub cii: f64,  // Crystal Integration Index
    pub ci: f64,   // Crystal Integration
    pub cb: f64,   // Crystal Balance
}

impl CrystalMetrics {
    /// 创建新的度量指标
    pub fn new(cii: f64, ci: f64, cb: f64) -> Self {
        Self { cii, ci, cb }
    }

    /// 从晶体状态计算度量指标
    pub fn from_state(state: &CrystalState) -> Self {
        // CII: 基于总 phi 和激活面数量
        let total_phi = state.total_phi();
        let active_faces = state.active_faces().len() as f64;
        let cii = if active_faces > 0.0 {
            total_phi * active_faces / 6.0
        } else {
            0.0
        };

        // CI: 基于平均一致性
        let ci = state.average_coherence();

        // CB: 基于核心能量和壳层完整性
        let cb = (state.core_energy + state.shell_integrity) / 2.0;

        Self { cii, ci, cb }
    }

    /// 计算综合健康分数 (0.0 - 1.0)
    pub fn health_score(&self) -> f64 {
        (self.cii + self.ci + self.cb) / 3.0
    }

    /// 检查是否健康
    pub fn is_healthy(&self, threshold: f64) -> bool {
        self.health_score() >= threshold
    }
}

impl Default for CrystalMetrics {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

/// 晶体性能度量
#[derive(Debug, Clone)]
pub struct CrystalPerformance {
    pub metrics: CrystalMetrics,
    pub history: Vec<CrystalMetrics>,
    pub max_history: usize,
}

impl CrystalPerformance {
    pub fn new() -> Self {
        Self {
            metrics: CrystalMetrics::default(),
            history: Vec::new(),
            max_history: 100,
        }
    }

    pub fn update(&mut self, state: &CrystalState) {
        self.metrics = CrystalMetrics::from_state(state);
        self.history.push(self.metrics);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    pub fn average_metrics(&self) -> CrystalMetrics {
        if self.history.is_empty() {
            return CrystalMetrics::default();
        }

        let sum_cii: f64 = self.history.iter().map(|m| m.cii).sum();
        let sum_ci: f64 = self.history.iter().map(|m| m.ci).sum();
        let sum_cb: f64 = self.history.iter().map(|m| m.cb).sum();
        let len = self.history.len() as f64;

        CrystalMetrics::new(sum_cii / len, sum_ci / len, sum_cb / len)
    }
}

impl Default for CrystalPerformance {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l5_cognition::nt_core::nt_core_consciousness_crystal::crystal_state::CrystalFace;

    #[test]
    fn test_crystal_metrics_from_state() {
        let mut state = CrystalState::new();
        state.activate_face(&CrystalFace::Perception);
        state.activate_face(&CrystalFace::Reasoning);

        let metrics = CrystalMetrics::from_state(&state);
        assert!(metrics.cii > 0.0);
        assert_eq!(metrics.ci, 0.0); // 一致性为0，因为没有设置
    }

    #[test]
    fn test_health_score() {
        let metrics = CrystalMetrics::new(0.8, 0.9, 0.7);
        let health = metrics.health_score();
        assert!((health - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_crystal_performance() {
        let mut perf = CrystalPerformance::new();
        let state = CrystalState::new();
        perf.update(&state);
        assert_eq!(perf.history.len(), 1);
    }
}