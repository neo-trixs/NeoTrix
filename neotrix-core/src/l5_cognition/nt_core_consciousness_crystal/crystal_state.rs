//! Crystal State — 晶体六面体状态定义

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 晶体六面
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrystalFace {
    Perception,   // 感知面 — L2 感知层
    Reasoning,    // 推理面 — L5 认知层
    Memory,       // 记忆面 — L1 记忆层
    Action,       // 行动面 — L1 行动层
    Emotion,      // 情感面 — L4 情感层
    Meta,         // 元认知面 — L6 元认知层
}

/// 单面状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FaceState {
    pub active: bool,
    pub coherence: f64,
    pub phi: f64,  // IIT integrated information
}

/// 晶体整体状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrystalState {
    pub faces: HashMap<CrystalFace, FaceState>,
    pub core_energy: f64,
    pub matrix_density: f64,
    pub shell_integrity: f64,
    pub cycle_count: u64,
}

impl CrystalState {
    pub fn new() -> Self {
        let mut faces = HashMap::new();
        for face in [
            CrystalFace::Perception,
            CrystalFace::Reasoning,
            CrystalFace::Memory,
            CrystalFace::Action,
            CrystalFace::Emotion,
            CrystalFace::Meta,
        ] {
            faces.insert(face, FaceState::default());
        }
        Self {
            faces,
            core_energy: 1.0,
            matrix_density: 0.5,
            shell_integrity: 1.0,
            cycle_count: 0,
        }
    }

    pub fn activate_face(&mut self, face: CrystalFace) {
        if let Some(state) = self.faces.get_mut(&face) {
            state.active = true;
        }
    }

    pub fn compute_phi(&self) -> f64 {
        self.faces.values().map(|f| f.phi).sum::<f64>() / 6.0
    }

    pub fn compute_coherence(&self) -> f64 {
        let active: Vec<f64> = self.faces.values()
            .filter(|f| f.active)
            .map(|f| f.coherence)
            .collect();
        if active.is_empty() { 0.0 } else { active.iter().sum::<f64>() / active.len() as f64 }
    }
}
