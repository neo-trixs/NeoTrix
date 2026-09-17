use super::crystal_state::{CrystalFace, CrystalState};

/// 晶体循环报告
#[derive(Debug, Clone)]
pub struct CrystalReport {
    pub cycle: u64,
    pub phi: f64,
    pub coherence: f64,
    pub face_activations: Vec<CrystalFace>,
}

/// 晶体快照
#[derive(Debug, Clone)]
pub struct CrystalSnapshot {
    pub state: CrystalState,
    pub timestamp: u64,
}

/// 晶体循环 trait
pub trait CrystalCycle {
    /// 推进一个循环
    fn tick(&mut self) -> CrystalReport;

    /// 获取当前快照
    fn observe(&self) -> CrystalSnapshot;

    /// 从快照恢复状态
    fn restore(&mut self, snapshot: CrystalSnapshot);

    /// 获取当前循环计数
    fn cycle_count(&self) -> u64;

    /// 获取当前状态引用
    fn state(&self) -> &CrystalState;

    /// 获取当前状态可变引用
    fn state_mut(&mut self) -> &mut CrystalState;
}

/// 默认的晶体循环实现
pub struct DefaultCrystalCycle {
    state: CrystalState,
}

impl DefaultCrystalCycle {
    pub fn new() -> Self {
        Self {
            state: CrystalState::new(),
        }
    }

    pub fn with_state(state: CrystalState) -> Self {
        Self { state }
    }
}

impl Default for DefaultCrystalCycle {
    fn default() -> Self {
        Self::new()
    }
}

impl CrystalCycle for DefaultCrystalCycle {
    fn tick(&mut self) -> CrystalReport {
        // 递增循环计数
        self.state.cycle_count += 1;

        // 计算当前指标
        let phi = self.state.total_phi();
        let coherence = self.state.average_coherence();
        let face_activations: Vec<CrystalFace> = self
            .state
            .active_faces()
            .into_iter()
            .cloned()
            .collect();

        CrystalReport {
            cycle: self.state.cycle_count,
            phi,
            coherence,
            face_activations,
        }
    }

    fn observe(&self) -> CrystalSnapshot {
        CrystalSnapshot {
            state: self.state.clone(),
            timestamp: self.state.cycle_count,
        }
    }

    fn restore(&mut self, snapshot: CrystalSnapshot) {
        self.state = snapshot.state;
    }

    fn cycle_count(&self) -> u64 {
        self.state.cycle_count
    }

    fn state(&self) -> &CrystalState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut CrystalState {
        &mut self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_crystal_cycle() {
        let mut cycle = DefaultCrystalCycle::new();
        let report = cycle.tick();
        assert_eq!(report.cycle, 1);
        assert_eq!(report.face_activations.len(), 0);
    }

    #[test]
    fn test_observe_and_restore() {
        let mut cycle = DefaultCrystalCycle::new();
        cycle.state.activate_face(&CrystalFace::Perception);
        let snapshot = cycle.observe();

        let mut cycle2 = DefaultCrystalCycle::new();
        cycle2.restore(snapshot);
        assert!(cycle2.state.get_face(&CrystalFace::Perception).unwrap().active);
    }

    #[test]
    fn test_crystal_report() {
        let mut cycle = DefaultCrystalCycle::new();
        cycle.state.activate_face(&CrystalFace::Reasoning);
        if let Some(face) = cycle.state.get_face_mut(&CrystalFace::Reasoning) {
            face.phi = 0.7;
            face.coherence = 0.9;
        }

        let report = cycle.tick();
        assert_eq!(report.cycle, 1);
        assert_eq!(report.phi, 0.7);
        assert!(report.face_activations.contains(&CrystalFace::Reasoning));
    }
}