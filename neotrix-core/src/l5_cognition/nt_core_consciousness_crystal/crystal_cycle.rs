//! Crystal Cycle — 晶体意识循环

use super::crystal_state::CrystalState;
use serde::{Deserialize, Serialize};

/// 晶体循环报告
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrystalReport {
    pub cycle: u64,
    pub phi: f64,
    pub coherence: f64,
    pub active_faces: Vec<String>,
    pub energy_delta: f64,
}

/// 晶体循环 trait
pub trait CrystalCycle {
    fn tick(&mut self) -> CrystalReport;
    fn observe(&self) -> CrystalState;
    fn restore(&mut self, state: CrystalState);
}

impl CrystalCycle for CrystalState {
    fn tick(&mut self) -> CrystalReport {
        self.cycle_count += 1;

        // 模拟能量衰减
        self.core_energy = (self.core_energy * 0.99).max(0.1);

        // 模拟矩阵密度增长
        self.matrix_density = (self.matrix_density + 0.001).min(1.0);

        let active_faces: Vec<String> = self.faces.iter()
            .filter(|(_, s)| s.active)
            .map(|(f, _)| format!("{:?}", f))
            .collect();

        CrystalReport {
            cycle: self.cycle_count,
            phi: self.compute_phi(),
            coherence: self.compute_coherence(),
            active_faces,
            energy_delta: -0.01,
        }
    }

    fn observe(&self) -> CrystalState {
        self.clone()
    }

    fn restore(&mut self, state: CrystalState) {
        *self = state;
    }
}
