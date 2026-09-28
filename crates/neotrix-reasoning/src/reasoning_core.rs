//! Unified Reasoning Core — 统一推理核心类型与注册表
//!
//! 提供 KB/经验 → Kernel context 自动注入的 ContextBuilder。
//!
//! ⚠️ **更正（原第 3 行称"消除 4 处 ReasoningTrace 重复定义"—— 事实不符）**
//! 本 crate 的 `ReasoningTrace` 统一类型**从未被主代码采纳**，4 处同名定义一直并存。
//! 2026-09-28 逐字段核对后确认**不可统一**，理由：
//! - `l2 crawl/circuits_types.rs` 那份有 `steps: usize`（**计数**），
//!   本类型这里是 `steps: Vec<ReasoningStep>`（**列表**）—— 语义不同，非超集。
//! - `l5 reason/reasoning_types.rs` 那份有 `prompt` / `perspective_lens` /
//!   `error_context` / `success` / `reasoning_type` 共 5 个字段，
//!   **本类型均无对应** —— 若统一需为所有 4 个使用方塞入用不上的字段。
//! - 4 处作用域互斥，**无任何文件同时引用 2 份**，故不存在编译冲突。
//! 因此改为**改名消歧**（保留本 crate 的通用名 `ReasoningTrace` 作为规范名）：
//! - crawl        `ReasoningTrace`→`CircuitTrace`、`ReasoningMethod`→`CircuitMethod`
//! - seal_core    `ReasoningTrace`→`ProcessStageTrace`、`ReasoningStep`→`ProcessStageStep`、
//!                `TraceSource`→`ProcessStageTraceSource`
//! - reason       `ReasoningTrace`→`ReasoningRecord`、`ReasoningMethod`→`ReasoningApproach`
//! 同批完成 `ReasoningStep` 的消歧（本类型保留规范名 `ReasoningStep`）：
//! - `nt_core_ttc.rs:124`            → `TtcStep`（PRM 打分步骤：prm_score/cumulative_score）
//! - `nt_mind/control_distillation.rs` → `ControlDistillStep`（控制蒸馏：e8_mode/token_count）
//! - `nt_mind/cross_domain/reason_retrieve_refine.rs` → `RefineStep`（检索-精炼：input/output/confidence）
//! 这 3 处经 grep 确认**无任何 `use` 导入、无全限定路径引用**，仅本文件内自用，
//! 故改名不波及其他作用域。

use crate::kernel_types::{ReasoningMethod, Vector, EVOLUTION};
use neotrix_types::e8_reasoning::ReasoningHexagram;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Once;

static METHOD_REGISTRY_DEPRECATED: Once = Once::new();

/// 统一推理轨迹 — 覆盖所有 4 处原定义的用例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTrace {
    /// 唯一标识
    pub trace_id: String,
    /// 任务描述
    pub task: String,
    /// 推理方法（27 种之一）
    pub method: ReasoningMethod,
    /// 对应的 E8 Hexagram 状态（0-63）
    pub hexagram: ReasoningHexagram,
    /// 对应的演化阶段（0-18）
    pub stage: usize,
    /// 步骤级详细轨迹（用于过程监督/PRM）
    pub steps: Vec<ReasoningStep>,
    /// Kernel 内部中间状态演化（向量序列，用于收敛分析）
    pub intermediate_states: Vec<Vector>,
    /// 收敛度量（0-1）
    pub convergence: f64,
    /// 最终输出质量（0-1，用于 PRM/过程监督）
    pub final_quality: f64,
    /// LLM 原始响应（若有）
    pub llm_response: Option<String>,
    /// 来源标记
    pub source: TraceSource,
    /// 时间戳
    pub timestamp: u64,
}

/// 单步推理记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub step_index: usize,
    pub description: String,
    pub state_before: Option<Vector>,
    pub state_after: Option<Vector>,
    pub reward: Option<f64>,
    pub hexagram: ReasoningHexagram,
}

/// 轨迹来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceSource {
    ConsciousnessTree,
    KBExperience,
    Synthesis,
    KernelEvolution,
    LLMDriven,
}

/// Method ↔ Stage ↔ Hexagram 统一注册表
///
/// TODO(fusion-plan-215): Merge into `ReasoningStrategyRegistry` — MethodRegistry is a redundant
/// registry that overlaps with reasoning strategy management.
#[derive(Debug, Clone)]
pub struct MethodRegistry {
    /// method -> (stage_range, preferred_hexagrams)
    method_map: HashMap<ReasoningMethod, MethodSpec>,
    /// stage -> available_methods
    stage_methods: Vec<Vec<ReasoningMethod>>,
    /// hexagram -> (method, stage) 反向查找
    hexagram_reverse: HashMap<u8, (ReasoningMethod, usize)>,
}

#[derive(Debug, Clone)]
pub struct MethodSpec {
    pub stage_range: (usize, usize),
    pub preferred_hexagrams: Vec<u8>,
    pub complexity_ceiling: f64,
    pub is_generative: bool,
}

impl Default for MethodRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MethodRegistry {
    pub fn new() -> Self {
        METHOD_REGISTRY_DEPRECATED.call_once(|| {
            tracing::warn!(
                "MethodRegistry is deprecated — merge into ReasoningStrategyRegistry (fusion-plan-215)"
            );
        });
        let mut registry = Self {
            method_map: HashMap::new(),
            stage_methods: vec![Vec::new(); EVOLUTION.len()],
            hexagram_reverse: HashMap::new(),
        };
        registry.build_default_mapping();
        registry
    }

    fn build_default_mapping(&mut self) {
        let mappings = [
            (
                ReasoningMethod::Deductive,
                (0, 2),
                vec![0b000000, 0b000001],
                0.3,
                false,
            ),
            (
                ReasoningMethod::Inductive,
                (0, 3),
                vec![0b000010, 0b000011],
                0.4,
                false,
            ),
            (
                ReasoningMethod::KnowledgeRetrieval,
                (0, 3),
                vec![0b000100],
                0.2,
                false,
            ),
            (
                ReasoningMethod::Analogical,
                (3, 4),
                vec![0b001000, 0b001001],
                0.5,
                true,
            ),
            (
                ReasoningMethod::Recursive,
                (3, 5),
                vec![0b001010, 0b001011],
                0.6,
                true,
            ),
            (
                ReasoningMethod::Compositional,
                (5, 6),
                vec![0b010000, 0b010001],
                0.7,
                true,
            ),
            (
                ReasoningMethod::Adversarial,
                (5, 7),
                vec![0b010010, 0b010011],
                0.7,
                true,
            ),
            (
                ReasoningMethod::FirstPrinciples,
                (7, 9),
                vec![0b100000, 0b100001],
                0.8,
                true,
            ),
            (
                ReasoningMethod::AutoFetch,
                (7, 8),
                vec![0b100010],
                0.5,
                false,
            ),
            (
                ReasoningMethod::GradientLearning,
                (10, 13),
                vec![0b110000],
                0.9,
                true,
            ),
            (
                ReasoningMethod::ArchitectureSearch,
                (10, 13),
                vec![0b110001],
                0.9,
                true,
            ),
            (
                ReasoningMethod::GpuCompute,
                (11, 13),
                vec![0b110010],
                1.0,
                true,
            ),
            (
                ReasoningMethod::ExperienceDistill,
                (14, 16),
                vec![0b111000],
                0.8,
                true,
            ),
            (
                ReasoningMethod::EmergentAnalysis,
                (14, 16),
                vec![0b111001],
                0.9,
                true,
            ),
            (
                ReasoningMethod::SystemIntegration,
                (14, 16),
                vec![0b111010],
                0.9,
                true,
            ),
            (
                ReasoningMethod::EnsembleVoting,
                (17, 18),
                vec![0b111100],
                1.0,
                true,
            ),
            (
                ReasoningMethod::SelfImprovement,
                (17, 18),
                vec![0b111101],
                1.0,
                true,
            ),
            (
                ReasoningMethod::SparseRouting,
                (17, 18),
                vec![0b111110],
                1.0,
                true,
            ),
            (
                ReasoningMethod::Abductive,
                (2, 5),
                vec![0b000110],
                0.4,
                true,
            ),
            (
                ReasoningMethod::DistributedConsensus,
                (12, 15),
                vec![0b110100],
                0.8,
                true,
            ),
        ];

        for (method, (stage_min, stage_max), hexagrams, complexity, generative) in mappings {
            let spec = MethodSpec {
                stage_range: (stage_min, stage_max),
                preferred_hexagrams: hexagrams.clone(),
                complexity_ceiling: complexity,
                is_generative: generative,
            };
            self.method_map.insert(method, spec);
            for stage in stage_min..=stage_max.min(EVOLUTION.len() - 1) {
                self.stage_methods[stage].push(method);
            }
            for h in hexagrams {
                self.hexagram_reverse
                    .insert(h, (method, (stage_min + stage_max) / 2));
            }
        }

        for methods in &mut self.stage_methods {
            methods.sort_by_key(|m| *m as u8);
            methods.dedup();
        }
    }

    pub fn get_spec(&self, method: ReasoningMethod) -> Option<&MethodSpec> {
        self.method_map.get(&method)
    }

    pub fn methods_for_stage(&self, stage: usize) -> &[ReasoningMethod] {
        let idx = stage.min(self.stage_methods.len() - 1);
        &self.stage_methods[idx]
    }

    pub fn resolve_hexagram(&self, hex: ReasoningHexagram) -> Option<(ReasoningMethod, usize)> {
        self.hexagram_reverse.get(&hex.0).copied()
    }

    pub fn recommend_method(&self, complexity: f64, current_stage: usize) -> ReasoningMethod {
        let methods = self.methods_for_stage(current_stage);
        methods
            .iter()
            .filter(|m| self.method_map[m].complexity_ceiling >= complexity)
            .max_by_key(|m| self.method_map[m].complexity_ceiling as u32)
            .copied()
            .unwrap_or(ReasoningMethod::Deductive)
    }
}

/// 便利函数：创建默认 MethodRegistry
pub fn default_method_registry() -> MethodRegistry {
    MethodRegistry::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_registry_basic() {
        let reg = MethodRegistry::new();
        let spec = reg.get_spec(ReasoningMethod::Deductive).unwrap();
        assert_eq!(spec.stage_range, (0, 2));
        assert!(!spec.is_generative);
    }

    #[test]
    fn test_method_registry_stage_methods() {
        let reg = MethodRegistry::new();
        let methods = reg.methods_for_stage(0);
        assert!(methods.contains(&ReasoningMethod::Deductive));
        assert!(methods.contains(&ReasoningMethod::KnowledgeRetrieval));
    }

    #[test]
    fn test_method_registry_hexagram_resolve() {
        let reg = MethodRegistry::new();
        let resolved = reg.resolve_hexagram(ReasoningHexagram(0));
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().0, ReasoningMethod::Deductive);
    }
}
