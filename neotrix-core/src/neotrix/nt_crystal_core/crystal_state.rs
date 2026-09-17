//! CrystalState — 统一状态空间
//!
//! 所有子系统的单一事实源。晶体不是连接模块的中心，晶体是系统本身。
//! 能力/Agent/子系统是晶体的内在属性，不是外部模块。
//!
//! 设计文档: docs/1-DESIGN/unified-crystal-architecture.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::consciousness::CrystalConsciousness;
use super::evolution::GrowthPhase;
use super::experience::CrystalExperience;
use super::identity::CrystalIdentity;
use super::knowledge::CrystalKnowledge;
use super::evolution::CrystalEvolution;

// ═══════════════════════════════════════════════════════════════
// Core Types — 晶体统一状态空间
// ═══════════════════════════════════════════════════════════════

/// 晶体的统一状态空间 — 所有子系统的单一事实源
///
/// CrystalState 是 Unified Crystal Architecture 的核心。
/// 它不是"连接模块的中心"，而是"系统本身"。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalState {
    // === 核心身份 ===
    /// 晶体身份 (不可变)
    pub identity: CrystalIdentity,
    /// 内部时钟
    pub tick: u64,

    // === 四层核心 ===
    /// 知识层 — 缓慢进化 (理论/因果/矛盾)
    pub knowledge: CrystalKnowledge,
    /// 经验层 — 快速积累 (情境/教训/方案)
    pub experience: CrystalExperience,
    /// 进化层 — 实时变化 (生长周期/能力评分)
    pub evolution: CrystalEvolution,

    // === 感知与注意力 ===
    /// 注意力焦点
    pub attention_focus: Option<String>,
    /// 当前目标
    pub current_goal: Option<String>,

    // === 能力坐标系 ===
    /// 能力评分 (名称 → 0.0-1.0)
    pub capabilities: HashMap<String, f64>,
    /// 活跃能力投影 (Agent 在任务空间的临时实例)
    pub projections: Vec<AgentProjection>,

    // === 执行追踪 ===
    /// 执行历史
    pub execution_trace: Vec<ExecutionRecord>,
}

/// Agent 在任务空间的临时投影
///
/// Agent 不是独立引擎，是晶体意识在任务空间的临时投影。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProjection {
    /// 投影 ID
    pub id: String,
    /// 任务描述
    pub task: String,
    /// 投影的能力子集
    pub capabilities_used: Vec<String>,
    /// 置信度
    pub confidence: f64,
    /// 创建时间
    pub created_at: u64,
}

/// 执行记录 — 晶体的神经系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// 时间戳
    pub tick: u64,
    /// 操作类型
    pub action: String,
    /// 输入摘要
    pub input: String,
    /// 输出摘要
    pub output: String,
    /// 成功与否
    pub success: bool,
    /// 耗时 (ms)
    pub duration_ms: u64,
}

// ═══════════════════════════════════════════════════════════════
// Implementation — 统一状态操作
// ═══════════════════════════════════════════════════════════════

impl CrystalState {
    /// 创建新的晶体状态
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let mut capabilities = HashMap::new();
        capabilities.insert("memory_capacity".into(), 0.0);
        capabilities.insert("reasoning_depth".into(), 0.0);
        capabilities.insert("pattern_recognition".into(), 0.0);
        capabilities.insert("self_awareness".into(), 0.0);
        capabilities.insert("creativity".into(), 0.0);

        Self {
            identity: CrystalIdentity::new(name),
            tick: 0,
            knowledge: CrystalKnowledge::new(),
            experience: CrystalExperience::new(),
            evolution: CrystalEvolution::new(),
            attention_focus: None,
            current_goal: None,
            capabilities,
            projections: Vec::new(),
            execution_trace: Vec::new(),
        }
    }

    /// 获取当前 tick
    pub fn current_tick(&self) -> u64 {
        self.tick
    }

    /// 推进 tick
    pub fn advance_tick(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }

    /// 记忆 — 统一入口
    pub fn remember(&mut self, content: impl Into<String>, domain: impl Into<String>) {
        self.experience.record_episode(
            content,
            "remember",      // action
            "recorded",      // result
            "self-observe",  // reflection
            domain,
            1.0,             // quality
        );
    }

    /// 记录执行
    pub fn record_execution(&mut self, action: String, input: String, output: String, success: bool, duration_ms: u64) {
        self.tick += 1;
        self.execution_trace.push(ExecutionRecord {
            tick: self.tick,
            action,
            input,
            output,
            success,
            duration_ms,
        });
    }

    /// 创建 Agent 投影
    pub fn project_agent(&mut self, task: String, capabilities: Vec<String>) -> AgentProjection {
        let id = format!("agent-{}", self.tick);
        let projection = AgentProjection {
            id: id.clone(),
            task,
            capabilities_used: capabilities,
            confidence: 0.5,
            created_at: self.tick,
        };
        self.projections.push(projection.clone());
        projection
    }

    /// 获取整体能力评分
    pub fn overall_capability(&self) -> f64 {
        if self.capabilities.is_empty() {
            return 0.0;
        }
        self.capabilities.values().sum::<f64>() / self.capabilities.len() as f64
    }

    /// 获取状态摘要
    pub fn summary(&self) -> CrystalStateSummary {
        CrystalStateSummary {
            name: self.identity.name.clone(),
            tick: self.tick,
            knowledge_count: self.knowledge.theories.len() + self.knowledge.patterns.len(),
            experience_count: self.experience.episodes.len(),
            evolution_phase: self.evolution.current_phase.clone(),
            capability_score: self.overall_capability(),
            active_projections: self.projections.len(),
            execution_count: self.execution_trace.len(),
        }
    }

    /// 从 CrystalCore 构建
    pub fn from_core(core: super::CrystalCore) -> Self {
        let mut state = Self::new(&core.identity.name);
        state.knowledge = core.knowledge;
        state.experience = core.experience;
        state.evolution = core.evolution;
        state
    }

    /// 从 CrystalConsciousness 构建
    pub fn from_consciousness(cc: CrystalConsciousness) -> Self {
        let mut state = Self::new(&cc.identity.name);
        state.capabilities = cc.capabilities;
        state.attention_focus = cc.state.attention_focus;
        state.current_goal = cc.state.current_goal;
        // 迁移记忆
        for (_, memory) in cc.memories {
            state.remember(&memory.content, &memory.domain);
        }
        state
    }
}

/// 状态摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalStateSummary {
    pub name: String,
    pub tick: u64,
    pub knowledge_count: usize,
    pub experience_count: usize,
    pub evolution_phase: GrowthPhase,
    pub capability_score: f64,
    pub active_projections: usize,
    pub execution_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crystal_state_new() {
        let state = CrystalState::new("TestCrystal");
        assert_eq!(state.identity.name, "TestCrystal");
        assert_eq!(state.current_tick(), 0);
        assert!(state.overall_capability() >= 0.0);
    }

    #[test]
    fn test_crystal_state_remember() {
        let mut state = CrystalState::new("TestCrystal");
        state.remember("Test fact", "engineering");
        assert_eq!(state.experience.episodes.len(), 1);
    }

    #[test]
    fn test_crystal_state_project_agent() {
        let mut state = CrystalState::new("TestCrystal");
        let agent = state.project_agent("test task".into(), vec!["reasoning".into()]);
        assert!(!agent.id.is_empty());
        assert_eq!(state.projections.len(), 1);
    }

    #[test]
    fn test_crystal_state_summary() {
        let state = CrystalState::new("TestCrystal");
        let summary = state.summary();
        assert_eq!(summary.name, "TestCrystal");
        assert_eq!(summary.tick, 0);
    }
}
