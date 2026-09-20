//! 统一状态机层 (UnifiedConsciousnessSM)
//! 按 FUSION-ARCHITECTURE-v4 设计

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 操作状态 — 任务执行视角
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationalState {
    Idle,
    Thinking,
    Learning,
    Evolving,
    Healing,
}

/// 进化阶段 — 成长成熟度视角
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvolutionStage {
    Infant,
    Child,
    Adolescent,
    Adult,
    God,
}

/// 晶体面 — 六面体模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CrystalFace {
    Perception,
    Reasoning,
    Memory,
    Action,
    Emotion,
    Meta,
}

/// 面状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceState {
    pub active: bool,
    pub intensity: f64,
    pub last_activated: Option<i64>,
}

/// 状态转换
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: OperationalState,
    pub to: OperationalState,
    pub guard: String,
    pub action: String,
}

/// 阶段感知转换规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageAwareTransition {
    pub stage: EvolutionStage,
    pub allowed_states: Vec<OperationalState>,
    pub preferred_states: Vec<OperationalState>,
}

/// 面激活规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceActivationRule {
    pub face: CrystalFace,
    pub trigger_state: OperationalState,
    pub intensity_boost: f64,
}

/// 状态事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEvent {
    pub timestamp: i64,
    pub from: OperationalState,
    pub to: OperationalState,
    pub stage: EvolutionStage,
    pub faces_activated: Vec<CrystalFace>,
}

/// 统一状态机
pub struct UnifiedConsciousnessSM {
    operational_state: OperationalState,
    evolution_stage: EvolutionStage,
    crystal_faces: HashMap<CrystalFace, FaceState>,
    transitions: Vec<StateTransition>,
    stage_rules: Vec<StageAwareTransition>,
    face_rules: Vec<FaceActivationRule>,
    history: Vec<StateEvent>,
    knowledge_count: usize,
    pattern_count: usize,
    skill_count: usize,
    decision_count: usize,
}

impl UnifiedConsciousnessSM {
    pub fn new() -> Self {
        let mut crystal_faces = HashMap::new();
        for face in [
            CrystalFace::Perception,
            CrystalFace::Reasoning,
            CrystalFace::Memory,
            CrystalFace::Action,
            CrystalFace::Emotion,
            CrystalFace::Meta,
        ] {
            crystal_faces.insert(
                face,
                FaceState {
                    active: false,
                    intensity: 0.0,
                    last_activated: None,
                },
            );
        }

        Self {
            operational_state: OperationalState::Idle,
            evolution_stage: EvolutionStage::Infant,
            crystal_faces,
            transitions: vec![
                StateTransition {
                    from: OperationalState::Idle,
                    to: OperationalState::Thinking,
                    guard: "收到任务".to_string(),
                    action: "开始思考".to_string(),
                },
                StateTransition {
                    from: OperationalState::Thinking,
                    to: OperationalState::Learning,
                    guard: "发现知识缺口".to_string(),
                    action: "开始学习".to_string(),
                },
                StateTransition {
                    from: OperationalState::Learning,
                    to: OperationalState::Evolving,
                    guard: "学习完成".to_string(),
                    action: "开始进化".to_string(),
                },
                StateTransition {
                    from: OperationalState::Evolving,
                    to: OperationalState::Thinking,
                    guard: "进化完成".to_string(),
                    action: "继续思考".to_string(),
                },
                StateTransition {
                    from: OperationalState::Thinking,
                    to: OperationalState::Healing,
                    guard: "发现异常".to_string(),
                    action: "开始自愈".to_string(),
                },
                StateTransition {
                    from: OperationalState::Healing,
                    to: OperationalState::Idle,
                    guard: "修复完成".to_string(),
                    action: "回到空闲".to_string(),
                },
            ],
            stage_rules: vec![
                StageAwareTransition {
                    stage: EvolutionStage::Infant,
                    allowed_states: vec![
                        OperationalState::Idle,
                        OperationalState::Thinking,
                        OperationalState::Learning,
                    ],
                    preferred_states: vec![OperationalState::Learning],
                },
                StageAwareTransition {
                    stage: EvolutionStage::Child,
                    allowed_states: vec![
                        OperationalState::Idle,
                        OperationalState::Thinking,
                        OperationalState::Learning,
                        OperationalState::Evolving,
                        OperationalState::Healing,
                    ],
                    preferred_states: vec![OperationalState::Thinking, OperationalState::Learning],
                },
                StageAwareTransition {
                    stage: EvolutionStage::Adolescent,
                    allowed_states: vec![
                        OperationalState::Idle,
                        OperationalState::Thinking,
                        OperationalState::Learning,
                        OperationalState::Evolving,
                        OperationalState::Healing,
                    ],
                    preferred_states: vec![OperationalState::Evolving],
                },
                StageAwareTransition {
                    stage: EvolutionStage::Adult,
                    allowed_states: vec![
                        OperationalState::Idle,
                        OperationalState::Thinking,
                        OperationalState::Learning,
                        OperationalState::Evolving,
                        OperationalState::Healing,
                    ],
                    preferred_states: vec![OperationalState::Thinking, OperationalState::Evolving],
                },
                StageAwareTransition {
                    stage: EvolutionStage::God,
                    allowed_states: vec![
                        OperationalState::Idle,
                        OperationalState::Thinking,
                        OperationalState::Learning,
                        OperationalState::Evolving,
                        OperationalState::Healing,
                    ],
                    preferred_states: vec![OperationalState::Healing],
                },
            ],
            face_rules: vec![
                FaceActivationRule {
                    face: CrystalFace::Perception,
                    trigger_state: OperationalState::Thinking,
                    intensity_boost: 0.3,
                },
                FaceActivationRule {
                    face: CrystalFace::Reasoning,
                    trigger_state: OperationalState::Thinking,
                    intensity_boost: 0.4,
                },
                FaceActivationRule {
                    face: CrystalFace::Memory,
                    trigger_state: OperationalState::Learning,
                    intensity_boost: 0.5,
                },
                FaceActivationRule {
                    face: CrystalFace::Action,
                    trigger_state: OperationalState::Evolving,
                    intensity_boost: 0.3,
                },
                FaceActivationRule {
                    face: CrystalFace::Emotion,
                    trigger_state: OperationalState::Healing,
                    intensity_boost: 0.2,
                },
                FaceActivationRule {
                    face: CrystalFace::Meta,
                    trigger_state: OperationalState::Evolving,
                    intensity_boost: 0.4,
                },
            ],
            history: Vec::new(),
            knowledge_count: 0,
            pattern_count: 0,
            skill_count: 0,
            decision_count: 0,
        }
    }

    /// 尝试操作状态转换
    pub fn try_operational_transition(&mut self, target: OperationalState) -> Result<(), String> {
        let stage_rule = self
            .stage_rules
            .iter()
            .find(|r| r.stage == self.evolution_stage);
        if let Some(rule) = stage_rule {
            if !rule.allowed_states.contains(&target) {
                return Err(format!(
                    "阶段 {:?} 不允许状态 {:?}",
                    self.evolution_stage, target
                ));
            }
        }

        let transition = self
            .transitions
            .iter()
            .find(|t| t.from == self.operational_state && t.to == target);

        match transition {
            Some(_t) => {
                let event = StateEvent {
                    timestamp: chrono::Utc::now().timestamp(),
                    from: self.operational_state.clone(),
                    to: target.clone(),
                    stage: self.evolution_stage.clone(),
                    faces_activated: Vec::new(),
                };
                self.history.push(event);
                self.operational_state = target;
                self.activate_faces();
                Ok(())
            }
            None => Err(format!(
                "无效转换: {:?} → {:?}",
                self.operational_state, target
            )),
        }
    }

    /// 激活面
    fn activate_faces(&mut self) {
        for rule in &self.face_rules {
            if rule.trigger_state == self.operational_state {
                if let Some(face) = self.crystal_faces.get_mut(&rule.face) {
                    face.active = true;
                    face.intensity = (face.intensity + rule.intensity_boost).min(1.0);
                    face.last_activated = Some(chrono::Utc::now().timestamp());
                }
            }
        }
    }

    /// 添加知识
    pub fn add_knowledge(&mut self) {
        self.knowledge_count += 1;
        self.update_evolution_stage();
    }

    /// 添加模式
    pub fn add_pattern(&mut self) {
        self.pattern_count += 1;
        self.update_evolution_stage();
    }

    /// 添加技能
    pub fn add_skill(&mut self) {
        self.skill_count += 1;
        self.update_evolution_stage();
    }

    /// 添加决策
    pub fn add_decision(&mut self) {
        self.decision_count += 1;
        self.update_evolution_stage();
    }

    /// 更新进化阶段
    fn update_evolution_stage(&mut self) {
        match self.evolution_stage {
            EvolutionStage::Infant => {
                if self.knowledge_count > 100 {
                    self.evolution_stage = EvolutionStage::Child;
                    println!("[UnifiedSM] 婴儿期 → 儿童期");
                }
            }
            EvolutionStage::Child => {
                if self.pattern_count > 50 {
                    self.evolution_stage = EvolutionStage::Adolescent;
                    println!("[UnifiedSM] 儿童期 → 青少年期");
                }
            }
            EvolutionStage::Adolescent => {
                if self.skill_count > 30 {
                    self.evolution_stage = EvolutionStage::Adult;
                    println!("[UnifiedSM] 青少年期 → 成年期");
                }
            }
            EvolutionStage::Adult => {
                if self.decision_count > 100 {
                    self.evolution_stage = EvolutionStage::God;
                    println!("[UnifiedSM] 成年期 → 成神期");
                }
            }
            EvolutionStage::God => {}
        }
    }

    /// 获取当前操作状态
    pub fn operational_state(&self) -> &OperationalState {
        &self.operational_state
    }

    /// 获取当前进化阶段
    pub fn evolution_stage(&self) -> &EvolutionStage {
        &self.evolution_stage
    }

    /// 获取晶体面状态
    pub fn crystal_faces(&self) -> &HashMap<CrystalFace, FaceState> {
        &self.crystal_faces
    }

    /// 获取历史
    pub fn history(&self) -> &[StateEvent] {
        &self.history
    }

    /// 获取统计
    pub fn get_stats(&self) -> UnifiedSMStats {
        UnifiedSMStats {
            operational_state: self.operational_state.clone(),
            evolution_stage: self.evolution_stage.clone(),
            knowledge_count: self.knowledge_count,
            pattern_count: self.pattern_count,
            skill_count: self.skill_count,
            decision_count: self.decision_count,
            history_length: self.history.len(),
        }
    }
}

impl Default for UnifiedConsciousnessSM {
    fn default() -> Self {
        Self::new()
    }
}

/// 统一状态机统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSMStats {
    pub operational_state: OperationalState,
    pub evolution_stage: EvolutionStage,
    pub knowledge_count: usize,
    pub pattern_count: usize,
    pub skill_count: usize,
    pub decision_count: usize,
    pub history_length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let sm = UnifiedConsciousnessSM::new();
        assert_eq!(*sm.operational_state(), OperationalState::Idle);
        assert_eq!(*sm.evolution_stage(), EvolutionStage::Infant);
    }

    #[test]
    fn test_valid_transition() {
        let mut sm = UnifiedConsciousnessSM::new();
        assert!(sm
            .try_operational_transition(OperationalState::Thinking)
            .is_ok());
        assert_eq!(*sm.operational_state(), OperationalState::Thinking);
    }

    #[test]
    fn test_invalid_transition() {
        let mut sm = UnifiedConsciousnessSM::new();
        assert!(sm
            .try_operational_transition(OperationalState::Learning)
            .is_err());
        assert_eq!(*sm.operational_state(), OperationalState::Idle);
    }

    #[test]
    fn test_full_lifecycle() {
        let mut sm = UnifiedConsciousnessSM::new();
        sm.try_operational_transition(OperationalState::Thinking)
            .unwrap();
        sm.try_operational_transition(OperationalState::Learning)
            .unwrap();
        sm.try_operational_transition(OperationalState::Evolving)
            .unwrap();
        sm.try_operational_transition(OperationalState::Thinking)
            .unwrap();
        sm.try_operational_transition(OperationalState::Healing)
            .unwrap();
        sm.try_operational_transition(OperationalState::Idle)
            .unwrap();
        assert_eq!(*sm.operational_state(), OperationalState::Idle);
        assert_eq!(sm.history().len(), 6);
    }

    #[test]
    fn test_crystal_face_activation() {
        let mut sm = UnifiedConsciousnessSM::new();
        sm.try_operational_transition(OperationalState::Thinking)
            .unwrap();
        let faces = sm.crystal_faces();
        assert!(faces[&CrystalFace::Perception].active);
        assert!(faces[&CrystalFace::Reasoning].active);
        assert!(!faces[&CrystalFace::Memory].active);
    }

    #[test]
    fn test_knowledge_evolution() {
        let mut sm = UnifiedConsciousnessSM::new();
        for _ in 0..101 {
            sm.add_knowledge();
        }
        assert_eq!(*sm.evolution_stage(), EvolutionStage::Child);
    }

    #[test]
    fn test_stats() {
        let mut sm = UnifiedConsciousnessSM::new();
        sm.add_knowledge();
        sm.add_pattern();
        let stats = sm.get_stats();
        assert_eq!(stats.knowledge_count, 1);
        assert_eq!(stats.pattern_count, 1);
    }
}
