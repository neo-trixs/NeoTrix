//! Unified Reasoning Core — 统一推理核心类型与注册表
//!
//! 消除 4 处 ReasoningTrace 重复定义，建立 Method↔Stage↔Hexagram 显式映射，
//! 提供 KB/经验 → Kernel context 自动注入的 ContextBuilder。

use crate::core::nt_core_hex::ReasoningHexagram;
pub use crate::neotrix::{ReasoningMethod, EVOLUTION, KERNEL_DIM, Vector};
use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub stage_range: (usize, usize),           // 该 method 适用的 stage 范围
    pub preferred_hexagrams: Vec<u8>,          // 偏好 hexagram 位掩码
    pub complexity_ceiling: f64,               // 复杂度上限
    pub is_generative: bool,                   // 是否生成式
}

impl Default for MethodRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MethodRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            method_map: HashMap::new(),
            stage_methods: vec![Vec::new(); EVOLUTION.len()],
            hexagram_reverse: HashMap::new(),
        };
        registry.build_default_mapping();
        registry
    }

    fn build_default_mapping(&mut self) {
        // 基于现有 EVOLUTION stage 与 ReasoningMethod 的语义对应建立映射
        let mappings = [
            // Stage 0-2: 基础推理
            (ReasoningMethod::Deductive, (0, 2), vec![0b000000, 0b000001], 0.3, false),
            (ReasoningMethod::Inductive, (0, 3), vec![0b000010, 0b000011], 0.4, false),
            (ReasoningMethod::KnowledgeRetrieval, (0, 3), vec![0b000100], 0.2, false),
            // Stage 3-4: 类比/递归
            (ReasoningMethod::Analogical, (3, 4), vec![0b001000, 0b001001], 0.5, true),
            (ReasoningMethod::Recursive, (3, 5), vec![0b001010, 0b001011], 0.6, true),
            // Stage 5-6: 组合/对抗
            (ReasoningMethod::Compositional, (5, 6), vec![0b010000, 0b010001], 0.7, true),
            (ReasoningMethod::Adversarial, (5, 7), vec![0b010010, 0b010011], 0.7, true),
            // Stage 7-9: 第一性原理/自动获取
            (ReasoningMethod::FirstPrinciples, (7, 9), vec![0b100000, 0b100001], 0.8, true),
            (ReasoningMethod::AutoFetch, (7, 8), vec![0b100010], 0.5, false),
            // Stage 10-13: 学习/搜索
            (ReasoningMethod::GradientLearning, (10, 13), vec![0b110000], 0.9, true),
            (ReasoningMethod::ArchitectureSearch, (10, 13), vec![0b110001], 0.9, true),
            (ReasoningMethod::GpuCompute, (11, 13), vec![0b110010], 1.0, true),
            // Stage 14-16: 整合/涌现
            (ReasoningMethod::ExperienceDistill, (14, 16), vec![0b111000], 0.8, true),
            (ReasoningMethod::EmergentAnalysis, (14, 16), vec![0b111001], 0.9, true),
            (ReasoningMethod::SystemIntegration, (14, 16), vec![0b111010], 0.9, true),
            // Stage 17-18: 元推理
            (ReasoningMethod::EnsembleVoting, (17, 18), vec![0b111100], 1.0, true),
            (ReasoningMethod::SelfImprovement, (17, 18), vec![0b111101], 1.0, true),
            (ReasoningMethod::SparseRouting, (17, 18), vec![0b111110], 1.0, true),
            // 其余 method 兜底
            (ReasoningMethod::Abductive, (2, 5), vec![0b000110], 0.4, true),
            (ReasoningMethod::DistributedConsensus, (12, 15), vec![0b110100], 0.8, true),
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
                self.hexagram_reverse.insert(h, (method, (stage_min + stage_max) / 2));
            }
        }

        // 去重 stage_methods
        for methods in &mut self.stage_methods {
            methods.sort_by_key(|m| *m as u8);
            methods.dedup();
        }
    }

    /// 根据 method 获取规格
    pub fn get_spec(&self, method: ReasoningMethod) -> Option<&MethodSpec> {
        self.method_map.get(&method)
    }

    /// 根据 stage 获取可用 methods
    pub fn methods_for_stage(&self, stage: usize) -> &[ReasoningMethod] {
        let idx = stage.min(self.stage_methods.len() - 1);
        &self.stage_methods[idx]
    }

    /// 根据 hexagram 反向查找 (method, 推荐 stage)
    pub fn resolve_hexagram(&self, hex: ReasoningHexagram) -> Option<(ReasoningMethod, usize)> {
        self.hexagram_reverse.get(&hex.0).copied()
    }

    /// 根据 task 复杂度推荐 method
    pub fn recommend_method(&self, complexity: f64, current_stage: usize) -> ReasoningMethod {
        let methods = self.methods_for_stage(current_stage);
        methods.iter()
            .filter(|m| self.method_map[m].complexity_ceiling >= complexity)
            .max_by_key(|m| self.method_map[m].complexity_ceiling as u32)
            .copied()
            .unwrap_or(ReasoningMethod::Deductive)
    }
}

/// ContextBuilder: KB/经验 → Kernel context 自动注入
#[derive(Debug, Clone)]
pub struct ContextBuilder {
    pub max_context_items: usize,
    pub relevance_threshold: f64,
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self {
            max_context_items: 5,
            relevance_threshold: 0.3,
        }
    }
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 KB 检索相关经验并构建 Kernel context HashMap
    pub fn build_context(
        &self,
        kb: &KnowledgeBase,
        task: &str,
        _current_hexagram: ReasoningHexagram,
    ) -> HashMap<String, Vector> {
        let mut context = HashMap::new();

        // 1. 基于任务文本检索（统一 search 入口：PQ → semantic → hybrid BM25+FTS）
        if let Ok(results) = kb.search(task, self.max_context_items) {
            for (i, r) in results.iter().enumerate() {
                if r.score >= self.relevance_threshold {
                    // 优先用 content，回退 summary，再回退 title
                    let text = r.node.content.as_deref()
                        .or(r.node.summary.as_deref())
                        .unwrap_or(&r.node.title);
                    let vec = self.text_to_vector(text, KERNEL_DIM);
                    context.insert(format!("kb_search_{}", i), vec);
                }
            }
        }

        // 2. 任务本身向量
        context.insert("task".to_string(), self.text_to_vector(task, KERNEL_DIM));

        context
    }

    fn text_to_vector(&self, text: &str, dim: usize) -> Vector {
        if text.is_empty() || dim == 0 {
            return vec![0.0; dim];
        }
        let bytes: Vec<u8> = text.bytes().collect();
        let mut v = vec![0.0; dim];
        for (i, &b) in bytes.iter().enumerate() {
            let pos_phase = (i as f64 / bytes.len() as f64) * std::f64::consts::PI;
            let idx = i % dim;
            v[idx] = (b as f64 / 255.0) * 2.0 - 1.0 + pos_phase.sin() * 0.2;
        }
        for i in 0..dim.saturating_sub(bytes.len()) {
            let byte_idx = i % bytes.len().max(1);
            let b = bytes[byte_idx] as f64;
            v[bytes.len() + i] = ((b / 255.0) * 2.0 - 1.0) * 0.5;
        }
        let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-8);
        v.iter_mut().for_each(|x| *x /= norm);
        v
    }
}

/// 便利函数：创建默认 MethodRegistry
pub fn default_method_registry() -> MethodRegistry {
    MethodRegistry::new()
}

/// 便利函数：创建默认 ContextBuilder
pub fn default_context_builder() -> ContextBuilder {
    ContextBuilder::new()
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

    #[test]
    fn test_context_builder_empty() {
        let builder = ContextBuilder::new();
        // 无 KB 时返回仅含 task 的 context
        // 这里只测试 text_to_vector 不 panic
        let v = builder.text_to_vector("test", 128);
        assert_eq!(v.len(), 128);
    }
}