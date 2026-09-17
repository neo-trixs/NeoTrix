use std::collections::HashMap;

/// 晶体六面体状态面
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrystalFace {
    Perception,    // 感知面
    Reasoning,     // 推理面
    Memory,        // 记忆面
    Action,        // 行动面
    Emotion,       // 情感面
    Meta,          // 元认知面
}

impl CrystalFace {
    /// 返回所有面的数组
    pub fn all() -> [CrystalFace; 6] {
        [
            CrystalFace::Perception,
            CrystalFace::Reasoning,
            CrystalFace::Memory,
            CrystalFace::Action,
            CrystalFace::Emotion,
            CrystalFace::Meta,
        ]
    }
}

/// 单个面的状态
#[derive(Debug, Clone)]
pub struct FaceState {
    pub active: bool,
    pub coherence: f64,
    pub phi: f64,  // IIT integrated information
}

impl FaceState {
    pub fn new() -> Self {
        Self {
            active: false,
            coherence: 0.0,
            phi: 0.0,
        }
    }

    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn with_coherence(mut self, coherence: f64) -> Self {
        self.coherence = coherence;
        self
    }

    pub fn with_phi(mut self, phi: f64) -> Self {
        self.phi = phi;
        self
    }
}

/// 晶体核心状态
#[derive(Debug, Clone)]
pub struct CrystalState {
    pub faces: HashMap<CrystalFace, FaceState>,
    pub core_energy: f64,
    pub matrix_density: f64,
    pub shell_integrity: f64,
    pub cycle_count: u64,
}

impl CrystalState {
    /// 创建新的晶体状态
    pub fn new() -> Self {
        let faces = CrystalFace::all()
            .iter()
            .map(|&face| (face, FaceState::new()))
            .collect();

        Self {
            faces,
            core_energy: 1.0,
            matrix_density: 0.5,
            shell_integrity: 1.0,
            cycle_count: 0,
        }
    }

    /// 获取特定面的状态
    pub fn get_face(&self, face: &CrystalFace) -> Option<&FaceState> {
        self.faces.get(face)
    }

    /// 获取特定面的可变状态
    pub fn get_face_mut(&mut self, face: &CrystalFace) -> Option<&mut FaceState> {
        self.faces.get_mut(face)
    }

    /// 激活指定面
    pub fn activate_face(&mut self, face: &CrystalFace) {
        if let Some(state) = self.faces.get_mut(face) {
            state.active = true;
        }
    }

    /// 停用指定面
    pub fn deactivate_face(&mut self, face: &CrystalFace) {
        if let Some(state) = self.faces.get_mut(face) {
            state.active = false;
        }
    }

    /// 计算总 phi 值
    pub fn total_phi(&self) -> f64 {
        self.faces.values().map(|f| f.phi).sum()
    }

    /// 计算平均一致性
    pub fn average_coherence(&self) -> f64 {
        let active_faces: Vec<&FaceState> = self.faces.values().filter(|f| f.active).collect();
        if active_faces.is_empty() {
            return 0.0;
        }
        let sum: f64 = active_faces.iter().map(|f| f.coherence).sum();
        sum / active_faces.len() as f64
    }

    /// 获取激活的面
    pub fn active_faces(&self) -> Vec<&CrystalFace> {
        self.faces
            .iter()
            .filter(|(_, state)| state.active)
            .map(|(face, _)| face)
            .collect()
    }
}

impl Default for CrystalState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crystal_face_all() {
        let faces = CrystalFace::all();
        assert_eq!(faces.len(), 6);
    }

    #[test]
    fn test_crystal_state_new() {
        let state = CrystalState::new();
        assert_eq!(state.faces.len(), 6);
        assert_eq!(state.core_energy, 1.0);
        assert_eq!(state.cycle_count, 0);
    }

    #[test]
    fn test_activate_face() {
        let mut state = CrystalState::new();
        state.activate_face(&CrystalFace::Perception);
        assert!(state.get_face(&CrystalFace::Perception).unwrap().active);
    }

    #[test]
    fn test_total_phi() {
        let mut state = CrystalState::new();
        state.activate_face(&CrystalFace::Perception);
        state.activate_face(&CrystalFace::Reasoning);
        if let Some(face) = state.get_face_mut(&CrystalFace::Perception) {
            face.phi = 0.5;
        }
        if let Some(face) = state.get_face_mut(&CrystalFace::Reasoning) {
            face.phi = 0.3;
        }
        assert_eq!(state.total_phi(), 0.8);
    }
}