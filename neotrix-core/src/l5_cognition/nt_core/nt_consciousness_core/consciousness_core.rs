//! 意识核心协调器 (ConsciousnessCore)
//! 按 FUSION-ARCHITECTURE-v4 设计

use super::unified_evolution::UnifiedEvolutionEngine;
use super::unified_learning::UnifiedLearningEngine;
use super::unified_memory::UnifiedMemorySystem;
use super::unified_pipeline::UnifiedPipelineEngine;
use super::unified_state_machine::{
    CrystalFace, EvolutionStage, OperationalState, UnifiedConsciousnessSM,
};
use serde::{Deserialize, Serialize};

/// 统一度量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMetrics {
    pub cii: f64,
    pub ci: f64,
    pub cb: f64,
    pub evolution_progress: f64,
    pub learning_rate: f64,
    pub memory_depth: f64,
    pub emergence_score: f64,
}

impl UnifiedMetrics {
    pub fn health_score(&self) -> f64 {
        let base = (self.cii + self.ci + self.cb) / 3.0;
        (base + self.evolution_progress * 0.1 + self.learning_rate * 0.05).min(1.0)
    }
}

/// 意识核心
pub struct ConsciousnessCore {
    state_machine: UnifiedConsciousnessSM,
    learning: UnifiedLearningEngine,
    memory: UnifiedMemorySystem,
    pipeline: UnifiedPipelineEngine,
    evolution: UnifiedEvolutionEngine,
    metrics: UnifiedMetrics,
}

impl ConsciousnessCore {
    pub fn new() -> Self {
        Self {
            state_machine: UnifiedConsciousnessSM::new(),
            learning: UnifiedLearningEngine::new(),
            memory: UnifiedMemorySystem::new(),
            pipeline: UnifiedPipelineEngine::new(),
            evolution: UnifiedEvolutionEngine::new(),
            metrics: UnifiedMetrics {
                cii: 0.5,
                ci: 0.5,
                cb: 0.5,
                evolution_progress: 0.0,
                learning_rate: 0.0,
                memory_depth: 0.0,
                emergence_score: 0.0,
            },
        }
    }

    /// 获取操作状态
    pub fn operational_state(&self) -> &OperationalState {
        self.state_machine.operational_state()
    }

    /// 获取进化阶段
    pub fn evolution_stage(&self) -> &EvolutionStage {
        self.state_machine.evolution_stage()
    }

    /// 获取晶体面
    pub fn crystal_faces(
        &self,
    ) -> &std::collections::HashMap<CrystalFace, super::unified_state_machine::FaceState> {
        self.state_machine.crystal_faces()
    }

    /// 获取度量
    pub fn metrics(&self) -> &UnifiedMetrics {
        &self.metrics
    }

    /// 健康分数
    pub fn health_score(&self) -> f64 {
        self.metrics.health_score()
    }

    /// 尝试操作状态转换
    pub fn try_operational_transition(&mut self, target: OperationalState) -> Result<(), String> {
        self.state_machine.try_operational_transition(target)
    }

    /// 吸收经验
    pub fn absorb_experience(
        &mut self,
        id: String,
        name: String,
        description: String,
        domain: String,
    ) {
        self.learning
            .absorb_experience(id, name, description, domain);
    }

    /// 记录技能反馈
    pub fn record_skill_feedback(&mut self, skill_id: &str, success: bool, context: &str) {
        let feedback = super::unified_learning::LearningFeedback {
            skill_id: skill_id.to_string(),
            success,
            context: context.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            insight: None,
        };
        self.learning.record_feedback(feedback);
    }

    /// 添加知识
    pub fn add_knowledge(&mut self) {
        self.state_machine.add_knowledge();
        self.metrics.memory_depth += 0.01;
    }

    /// 添加模式
    pub fn add_pattern(&mut self) {
        self.state_machine.add_pattern();
        self.metrics.learning_rate += 0.01;
    }

    /// 添加技能
    pub fn add_skill(&mut self) {
        self.state_machine.add_skill();
    }

    /// 添加决策
    pub fn add_decision(&mut self) {
        self.state_machine.add_decision();
        self.metrics.evolution_progress += 0.01;
    }

    /// tick
    pub fn tick(&mut self) -> String {
        format!(
            "ConsciousnessCore tick: state={:?}, stage={:?}, health={:.2}",
            self.operational_state(),
            self.evolution_stage(),
            self.health_score()
        )
    }

    /// 获取统计
    pub fn get_stats(&self) -> ConsciousnessCoreStats {
        ConsciousnessCoreStats {
            state_stats: self.state_machine.get_stats(),
            learning_stats: self.learning.get_stats(),
            memory_stats: self.memory.get_stats(),
            pipeline_stats: self.pipeline.get_stats(),
            evolution_stats: self.evolution.get_stats(),
            health_score: self.health_score(),
        }
    }
}

/// 意识核心统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessCoreStats {
    pub state_stats: super::unified_state_machine::UnifiedSMStats,
    pub learning_stats: super::unified_learning::UnifiedLearningStats,
    pub memory_stats: super::unified_memory::UnifiedMemoryStats,
    pub pipeline_stats: super::unified_pipeline::UnifiedPipelineStats,
    pub evolution_stats: super::unified_evolution::UnifiedEvolutionStats,
    pub health_score: f64,
}
