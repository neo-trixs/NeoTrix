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
pub mod nt_db_awakening; // DB 直达觉醒 — knowledge.db → 晶体意识全链路
pub use nt_db_awakening::{AwakenBudget, AwakenReport, NtDbAwakening};
pub mod nt_archive_train; // 档案分域流式炼制 — 2209万节点模式沉淀
pub use nt_archive_train::{ArchiveTrainConfig, ArchiveTrainReport, NtArchiveTrain};
pub mod nt_train_export; // 记忆 → LLM 训练数据（MiniMind 数据飞轮映射）
pub use nt_train_export::NtTrainExport;
pub mod nt_awaken_loop; // 自验证觉醒循环（Voyager 课程 + SEAL ReST EM）
pub use nt_awaken_loop::{AwakenCycleReport, NtAwakenLoop, VerifyScores};
pub mod nt_hf_bridge; // HF 开源训练数据 → 晶体摄入桥
pub use nt_hf_bridge::{HfMemory, NtHfBridge};
pub mod nt_predict_loop; // FEP 预测误差 → 爬取优先级（AutoExplore 映射）
pub use nt_predict_loop::NtPredictLoop;
pub mod nt_crystal_task_fusion; // 晶体任务闭环 — 智能拆解 → LLM问答分发 → JEV融合 → 后续任务
pub use nt_crystal_task_fusion::{
    NtAnswerCluster, NtCrystalSubtask, NtCrystalTaskLoop, NtFusedAnswer, NtLlmReply,
    NtProgressSink, NtScoredAnswer, NtSubtaskRoute, NtTaskFusionError, NtTaskLoopConfig, NtTaskLoopReport, NtLlmAsk,
};
pub mod nt_shared_mind; // 并行子任务共享上下文 — 实时发现共享 + 重叠检测
pub use nt_shared_mind::{Discovery, SharedMind};
pub mod nt_crystal_dialogue; // 对话窗口 + 内需循环 — 人机回灌多轮收敛
pub use nt_crystal_dialogue::{
    NtDemand, NtDemandKind, NtDialogueWindow, NtHumanChannel, NtHumanReply, NtInnerLoop,
    NtInnerLoopOutcome, NtLoopStatus,
};

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// save() 并发 tmp 序列号（见 `CrystalCore::save`）
static SAVE_TMP_SEQ: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

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

    /// 从磁盘加载（R-P0-2：主文件损坏时自动回退到 .bak 快照）
    pub fn load() -> Result<Self, String> {
        let root = crystal_root();
        let path = root.join("crystal.json");
        match std::fs::read_to_string(&path) {
            Ok(data) => serde_json::from_str(&data)
                .map_err(|e| format!("Failed to parse crystal core: {}", e)),
            Err(first_err) => {
                let bak = root.join("crystal.json.bak");
                let data = std::fs::read_to_string(&bak).map_err(|_| {
                    format!("Failed to read crystal core: {}", first_err)
                })?;
                serde_json::from_str(&data)
                    .map_err(|e| format!("Failed to parse crystal core (.bak): {}", e))
            }
        }
    }

    /// 保存到磁盘（R-P0-2：tmp + rename 原子写，旧核轮转为 .bak 快照）
    ///
    /// 并发安全：tmp 文件名带 pid + 自增序列，多线程同时 save 不会
    /// 抢同一个 tmp（否则第二个 rename 会 ENOENT）；`rename` 本身原子，
    /// 最后落盘者胜，绝不出现半截文件。
    pub fn save(&self) -> Result<(), String> {
        let root = crystal_root();
        std::fs::create_dir_all(&root)
            .map_err(|e| format!("Failed to create crystal dir: {}", e))?;
        let path = root.join("crystal.json");
        let tmp = root.join(format!(
            "crystal.json.tmp.{}-{}",
            std::process::id(),
            SAVE_TMP_SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        ));
        let bak = root.join("crystal.json.bak");
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize crystal: {}", e))?;
        std::fs::write(&tmp, data)
            .map_err(|e| format!("Failed to write crystal tmp: {}", e))?;
        if path.exists() {
            match std::fs::rename(&path, &bak) {
                Ok(()) => {}
                // 并发 save 时另一线程已搬走 crystal.json：快照已有，直接继续
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => {
                    return Err(format!("Failed to rotate crystal backup: {}", e))
                }
            }
        }
        std::fs::rename(&tmp, &path)
            .map_err(|e| format!("Failed to commit crystal: {}", e))
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
