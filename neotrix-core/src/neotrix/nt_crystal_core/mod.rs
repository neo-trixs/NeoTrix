//! # NT-Crystal Core — 意识晶体核心
//!
//! 四层架构:
//! - L1 身份层 (Identity): 不可变核心 — 公理、价值观、身份
//! - L2 知识层 (Knowledge): 缓慢进化 — 理论、因果链、矛盾、向量索引
//! - L3 经验层 (Experience): 快速积累 — 情境、失败教训、成功方案
//! - L4 进化层 (Evolution): 实时变化 — 生长周期、能力评分、适应记录
//!
//! 核心循环: 吸收 → 熔炼 → 进化 → 输出

pub mod identity;
pub mod knowledge;
pub mod experience;
pub mod evolution;
pub mod engine;
pub mod consciousness;  // 晶体意识 — 统一系统
pub mod ingestion;      // 薄摄入层 — 外部源 → 晶体意识
pub mod ctm;            // CTM通信机制 — Up-Tree + Down-Tree + Links
pub mod cocoons;        // 持久记忆茧 — 跨会话持久化
pub mod retention_policy; // 策略进化器 — 自我优化记忆管理
pub mod meta_metrics;   // 性能追踪器 — 监控和告警
pub mod modules;        // CTM模块实现 — Memory/Perception/Action/Emotion/Safety/Meta
pub mod meta_cognitive;     // Meta-Cognitive 递归自改进层
pub mod memory_architecture; // AutoMem 记忆架构优化
pub mod failure_diagnosis;   // 失败引导诊断
pub mod arbitrator;          // 确定性仲裁器 — LEGIO 启发
pub mod developmental_training; // 发育训练系统 — 模块能力阶段管理
pub mod link_graph_active;   // LinkGraph 激活 — 链接形成规则 + 无意识通信
pub mod crystal_state;       // 统一状态空间 — 晶体 = 系统本身

pub use identity::{CrystalIdentity, Axiom, ValueWeights};
pub use knowledge::{CrystalKnowledge, Theory, CausalPattern, Contradiction};
pub use experience::{CrystalExperience, Episode, Lesson, Solution};
pub use evolution::{CrystalEvolution, GrowthCycle, CapabilityScores, GrowthPhase};
pub use engine::CrystalEngine;
pub use consciousness::*;
pub use ingestion::{IngestionEngine, IngestionReport};
pub use ctm::*;
pub use cocoons::*;
pub use retention_policy::*;
pub use meta_metrics::*;
pub use modules::*;
pub use meta_cognitive::*;
pub use memory_architecture::*;
pub use failure_diagnosis::*;
pub use arbitrator::*;
pub use developmental_training::*;
pub use link_graph_active::*;
pub mod cross_source;
pub use cross_source::CrossSourceFusionEngine;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 晶体核心根目录
pub fn crystal_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".neotrix")
        .join("crystal_core")
}

/// 晶体核心 — 四层合一
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalCore {
    /// L1 身份层 — 不可变
    pub identity: CrystalIdentity,
    /// L2 知识层 — 缓慢进化
    pub knowledge: CrystalKnowledge,
    /// L3 经验层 — 快速积累
    pub experience: CrystalExperience,
    /// L4 进化层 — 实时变化
    pub evolution: CrystalEvolution,
}

impl CrystalCore {
    /// 创建新的晶体核心
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            identity: CrystalIdentity::new(name),
            knowledge: CrystalKnowledge::new(),
            experience: CrystalExperience::new(),
            evolution: CrystalEvolution::new(),
        }
    }

    /// 从磁盘加载
    pub fn load() -> Result<Self, String> {
        let root = crystal_root();
        let path = root.join("crystal.json");
        let data = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read crystal core: {}", e))?;
        serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse crystal core: {}", e))
    }

    /// 保存到磁盘
    pub fn save(&self) -> Result<(), String> {
        let root = crystal_root();
        std::fs::create_dir_all(&root)
            .map_err(|e| format!("Failed to create crystal dir: {}", e))?;
        let path = root.join("crystal.json");
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize crystal: {}", e))?;
        std::fs::write(&path, data)
            .map_err(|e| format!("Failed to write crystal: {}", e))
    }

    /// 获取当前状态摘要
    pub fn status(&self) -> CrystalStatus {
        CrystalStatus {
            name: self.identity.name.clone(),
            axioms_count: self.identity.axioms.len(),
            theories_count: self.knowledge.theories.len(),
            patterns_count: self.knowledge.patterns.len(),
            episodes_count: self.experience.episodes.len(),
            failures_count: self.experience.failures.len(),
            successes_count: self.experience.successes.len(),
            growth_cycles: self.evolution.growth_cycles.len(),
            current_phase: self.evolution.current_phase.clone(),
            overall_score: self.evolution.capability_scores.overall(),
        }
    }
}

/// 晶体状态摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalStatus {
    pub name: String,
    pub axioms_count: usize,
    pub theories_count: usize,
    pub patterns_count: usize,
    pub episodes_count: usize,
    pub failures_count: usize,
    pub successes_count: usize,
    pub growth_cycles: usize,
    pub current_phase: GrowthPhase,
    pub overall_score: f64,
}

impl std::fmt::Display for CrystalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Crystal Core: {} ===", self.name)?;
        writeln!(f, "  Phase: {:?}", self.current_phase)?;
        writeln!(f, "  Overall Score: {:.2}", self.overall_score)?;
        writeln!(f, "  L1 Identity: {} axioms", self.axioms_count)?;
        writeln!(f, "  L2 Knowledge: {} theories, {} patterns", self.theories_count, self.patterns_count)?;
        writeln!(f, "  L3 Experience: {} episodes, {} failures, {} successes",
            self.episodes_count, self.failures_count, self.successes_count)?;
        writeln!(f, "  L4 Evolution: {} growth cycles", self.growth_cycles)?;
        Ok(())
    }
}
