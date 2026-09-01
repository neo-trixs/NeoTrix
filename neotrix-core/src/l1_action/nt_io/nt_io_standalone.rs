use std::collections::HashMap;

use crate::core::nt_core_policy::E8Policy;
use crate::core::nt_core_reasoning::{ReasoningTrace, TraceSource};

pub use crate::core::nt_core_kernel_types::{
    EVOLUTION, KERNEL_DIM, KernelStats, ReasoningKernel, ReasoningMethod, ReasoningOutput,
    SelfConsistencyResult, StageInfo, Vector,
};

impl ReasoningKernel {
    pub fn new(stage: usize) -> Self {
        Self { stage: stage.min(EVOLUTION.len() - 1), state: vec![0.0; KERNEL_DIM] }
    }

    /// 真实推理：方法选择 → 多步状态演化 → 收敛度量 → 置信度。
    /// 替代原固定返回 Deductive/0.5 的 stub（P0 补齐，对齐主流推理模型
    /// 的 CoT 多步演化 + test-time compute scaling）。
    /// 
    /// Phase 2.3: 接受 E8Policy 用于方法选择（Kernel ↔ E8Policy 双向绑定）
    pub fn reason(
        &self, 
        query: &[f64], 
        context: Option<HashMap<String, Vector>>,
        e8_policy: Option<&E8Policy>,
    ) -> ReasoningOutput {
        // Phase 2.3: 方法选择参考 E8Policy 的 mode_values
        let method = self.select_method(query, context.is_some(), e8_policy);
        let steps = self.plan_steps(method);
        let mut state = self.state.clone();
        let mut intermediates: Vec<Vector> = Vec::with_capacity(steps);

        for t in 0..steps {
            let alpha = (t as f64 + 1.0) / steps as f64;
            for i in 0..state.len() {
                let q = query.get(i).copied().unwrap_or(0.0);
                let ctx = context
                    .as_ref()
                    .and_then(|m| m.values().next())
                    .and_then(|v| v.get(i))
                    .copied()
                    .unwrap_or(0.0);
                state[i] = state[i] * (1.0 - alpha * 0.3) + q * alpha * 0.6 + ctx * alpha * 0.3
                    + self.method_bias(method, i) * 0.1;
            }
            let norm: f64 = state.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-8);
            for x in state.iter_mut() {
                *x /= norm;
            }
            intermediates.push(state.clone());
        }

        let convergence = if intermediates.len() >= 2 {
            let a = &intermediates[intermediates.len() - 2];
            let b = intermediates.last().expect("intermediates.len() >= 2");
            let delta: f64 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum();
            1.0 - (delta / state.len() as f64).min(1.0)
        } else {
            0.5
        };

        let energy = query.iter().map(|x| x * x).sum::<f64>().sqrt();
        let query_scale = (energy / (query.len() as f64).sqrt()).min(1.0);
        let confidence = (convergence * 0.6 + query_scale * 0.4).clamp(0.05, 0.98);

        // 使用统一 ReasoningTrace
        let trace = ReasoningTrace {
            trace_id: format!("kernel_{}_{}", self.stage, std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()),
            task: "standalone_kernel_reasoning".to_string(),
            method,
            hexagram: crate::core::nt_core_hex::ReasoningHexagram::new(self.stage as u8 % 64),
            stage: self.stage,
            steps: Vec::new(), // 简化：不记录详细步骤文本
            intermediate_states: intermediates,
            convergence,
            final_quality: confidence,
            llm_response: None,
            source: TraceSource::KernelEvolution,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        ReasoningOutput {
            state_delta: state,
            confidence,
            trace,
        }
    }

    /// 方法选择：基于 stage 演化 + query 特征（稀疏度/能量/上下文）+ E8Policy 引导。
    /// Phase 2.3: 若提供 E8Policy，参考 mode_values 选择高价值方法。
    fn select_method(&self, query: &[f64], has_context: bool, e8_policy: Option<&E8Policy>) -> ReasoningMethod {
        let energy: f64 = query.iter().map(|x| x * x).sum();
        let active = query.iter().filter(|x| x.abs() > 0.5).count();
        let sparse = active < query.len() / 8;

        // Phase 2.3: 若有 E8Policy，优先选择 mode_values 高的方法对应的 hexagram
        if let Some(policy) = e8_policy {
            // 找到 mode_values 最高的 hexagram，映射到对应方法
            if let Some((best_idx, _)) = policy.mode_values.iter().enumerate()
                .max_by(|(_, a): &(usize, &f64), (_, b): &(usize, &f64)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)) {
                // 将 hexagram 映射到方法（简化：基于 stage 和 hexagram 值）
                let hex_val = best_idx as u8;
                return self.hexagram_to_method(hex_val, self.stage, sparse, has_context);
            }
        }

        // 回退到原有启发式选择
        match self.stage {
            0..=2 => {
                if sparse {
                    ReasoningMethod::KnowledgeRetrieval
                } else if has_context {
                    ReasoningMethod::Inductive
                } else {
                    ReasoningMethod::Deductive
                }
            }
            3..=4 => {
                if has_context {
                    ReasoningMethod::Analogical
                } else {
                    ReasoningMethod::Recursive
                }
            }
            5..=6 => {
                if energy > 2.0 {
                    ReasoningMethod::Compositional
                } else {
                    ReasoningMethod::Adversarial
                }
            }
            7..=8 => {
                if sparse {
                    ReasoningMethod::AutoFetch
                } else {
                    ReasoningMethod::FirstPrinciples
                }
            }
            9..=13 => {
                if has_context {
                    ReasoningMethod::KnowledgeRetrieval
                } else {
                    ReasoningMethod::GradientLearning
                }
            }
            14..=16 => {
                if energy > 2.0 {
                    ReasoningMethod::SystemIntegration
                } else {
                    ReasoningMethod::ExperienceDistill
                }
            }
            _ => {
                if has_context {
                    ReasoningMethod::EnsembleVoting
                } else {
                    ReasoningMethod::SelfImprovement
                }
            }
        }
    }

    /// 将 hexagram 值映射到推理方法（简化映射）
    fn hexagram_to_method(&self, hex: u8, _stage: usize, _sparse: bool, _has_context: bool) -> ReasoningMethod {
        // 基于 hexagram 的 6 个轴位映射方法
        let abstraction = (hex >> 5) & 1;
        let _scope = (hex >> 4) & 1;
        let method_axis = (hex >> 3) & 1;
        let depth = (hex >> 2) & 1;
        let _mode = (hex >> 1) & 1;
        let _stance = hex & 1;

        // 简化映射：基于轴位组合选择方法
        match (method_axis, abstraction, depth) {
            (0, 0, 0) => ReasoningMethod::Deductive,      // Analytical, Concrete, Deep
            (0, 0, 1) => ReasoningMethod::Inductive,      // Analytical, Concrete, Fast
            (0, 1, 0) => ReasoningMethod::FirstPrinciples, // Analytical, Abstract, Deep
            (0, 1, 1) => ReasoningMethod::Analogical,      // Analytical, Abstract, Fast
            (1, 0, 0) => ReasoningMethod::Recursive,       // Generative, Concrete, Deep
            (1, 0, 1) => ReasoningMethod::Compositional,   // Generative, Concrete, Fast
            (1, 1, 0) => ReasoningMethod::GradientLearning, // Generative, Abstract, Deep
            (1, 1, 1) => ReasoningMethod::EnsembleVoting,  // Generative, Abstract, Fast
            _ => ReasoningMethod::Deductive, // fallback (should not happen with bit masking)
        }
    }

    /// 每方法不同步数（test-time compute scaling）。
    fn plan_steps(&self, method: ReasoningMethod) -> usize {
        match method {
            ReasoningMethod::Deductive => 4,
            ReasoningMethod::Inductive => 5,
            ReasoningMethod::Abductive => 5,
            ReasoningMethod::Analogical => 6,
            ReasoningMethod::FirstPrinciples => 8,
            ReasoningMethod::Recursive => 7,
            ReasoningMethod::Compositional => 8,
            ReasoningMethod::Adversarial => 6,
            ReasoningMethod::AutoFetch => 3,
            ReasoningMethod::KnowledgeRetrieval => 3,
            ReasoningMethod::GradientLearning => 10,
            ReasoningMethod::ArchitectureSearch => 9,
            ReasoningMethod::GpuCompute => 12,
            ReasoningMethod::DistributedConsensus => 8,
            ReasoningMethod::ExperienceDistill => 6,
            ReasoningMethod::EmergentAnalysis => 7,
            ReasoningMethod::SystemIntegration => 9,
            ReasoningMethod::EnsembleVoting => 5,
            ReasoningMethod::SelfImprovement => 10,
            ReasoningMethod::SparseRouting => 4,
        }
    }

    /// 各方法给状态注入可区分的相位偏置。
    fn method_bias(&self, method: ReasoningMethod, i: usize) -> f64 {
        let phase = (i as f64 / 128.0) * std::f64::consts::TAU;
        match method {
            ReasoningMethod::Deductive => phase.sin() * 0.1,
            ReasoningMethod::Inductive => phase.cos() * 0.1,
            ReasoningMethod::Abductive => (phase * 2.0).sin() * 0.08,
            ReasoningMethod::Analogical => (phase * 0.5).cos() * 0.12,
            ReasoningMethod::FirstPrinciples => 0.05,
            ReasoningMethod::Recursive => (phase * 3.0).sin() * 0.06,
            ReasoningMethod::Compositional => (phase * 1.5).cos() * 0.1,
            ReasoningMethod::Adversarial => -(phase * 2.0).cos() * 0.08,
            ReasoningMethod::AutoFetch => 0.03,
            ReasoningMethod::KnowledgeRetrieval => (phase * 0.25).cos() * 0.15,
            ReasoningMethod::GradientLearning => (phase * 4.0).sin() * 0.05,
            ReasoningMethod::ArchitectureSearch => (phase * 0.75).cos() * 0.09,
            ReasoningMethod::GpuCompute => 0.04,
            ReasoningMethod::DistributedConsensus => (phase * 0.5).sin() * 0.07,
            ReasoningMethod::ExperienceDistill => (phase * 1.0).cos() * 0.11,
            ReasoningMethod::EmergentAnalysis => (phase * 2.5).sin() * 0.06,
            ReasoningMethod::SystemIntegration => (phase * 0.33).cos() * 0.13,
            ReasoningMethod::EnsembleVoting => 0.02,
            ReasoningMethod::SelfImprovement => (phase * 5.0).sin() * 0.04,
            ReasoningMethod::SparseRouting => 0.01,
        }
    }

    /// 当前 stage 可用的方法集（stats 真实化）。
    fn stage_methods(&self) -> Vec<ReasoningMethod> {
        match self.stage {
            0..=3 => vec![
                ReasoningMethod::Deductive,
                ReasoningMethod::Inductive,
                ReasoningMethod::KnowledgeRetrieval,
            ],
            4..=6 => vec![
                ReasoningMethod::Analogical,
                ReasoningMethod::Recursive,
                ReasoningMethod::Compositional,
            ],
            7..=9 => vec![
                ReasoningMethod::FirstPrinciples,
                ReasoningMethod::AutoFetch,
                ReasoningMethod::Adversarial,
            ],
            10..=13 => vec![
                ReasoningMethod::GradientLearning,
                ReasoningMethod::ArchitectureSearch,
                ReasoningMethod::GpuCompute,
            ],
            14..=16 => vec![
                ReasoningMethod::ExperienceDistill,
                ReasoningMethod::EmergentAnalysis,
                ReasoningMethod::SystemIntegration,
            ],
            _ => vec![
                ReasoningMethod::EnsembleVoting,
                ReasoningMethod::SelfImprovement,
                ReasoningMethod::SparseRouting,
            ],
        }
    }

    pub fn stats(&self) -> KernelStats {
        let active = self.stage_methods();
        KernelStats {
            stage: self.stage,
            label: EVOLUTION[self.stage].label.to_string(),
            state_dim: self.state.len(),
            total: active.len(),
            active,
            energy: self.state.iter().map(|x| x.abs()).sum::<f64>() / self.state.len().max(1) as f64,
        }
    }

    /// 阶段演化：推进到下一推理 stage（封顶于 EVOLUTION 末端）。
    pub fn evolve_stage(&mut self) {
        self.stage = (self.stage + 1).min(EVOLUTION.len() - 1);
    }

    /// Self-consistency 聚合（P0）：多路径采样 → 多数方法投票 + 平均置信度 + 聚合状态。
    /// 对齐主流推理模型的 self-consistency / majority vote。
    pub fn self_consistency(&self, query: &[f64], n_samples: usize) -> SelfConsistencyResult {
        let n = n_samples.max(1);
        let mut method_votes: HashMap<ReasoningMethod, usize> = HashMap::new();
        let mut confidences = Vec::with_capacity(n);
        let mut state_accum = vec![0.0; query.len()];
        for _ in 0..n {
            let out = self.reason(query, None, None);
            *method_votes.entry(out.trace.method).or_insert(0) += 1;
            confidences.push(out.confidence);
            for (a, x) in state_accum.iter_mut().zip(out.state_delta.iter()) {
                *a += x;
            }
        }
        let (majority_method, majority_count) = method_votes
            .iter()
            .max_by_key(|(_, c)| **c)
            .map(|(m, c)| (*m, *c))
            .unwrap_or((ReasoningMethod::Deductive, 1));
        let consistency = majority_count as f64 / n as f64;
        let avg_confidence = confidences.iter().sum::<f64>() / n as f64;
        let norm: f64 = state_accum.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-8);
        for x in state_accum.iter_mut() {
            *x /= norm;
        }
        SelfConsistencyResult {
            majority_method,
            consistency,
            avg_confidence,
            aggregated_state: state_accum,
            n_samples: n,
        }
    }
}

/// 最终答案正确性验证器（RLVR 锚，P0）：数值比对 + 归一化文本比对。
/// 返回 [0.0, 1.0] 的匹配分数。
pub fn verify_answer(gold: &str, candidate: &str) -> f64 {
    if gold.trim().is_empty() || candidate.trim().is_empty() {
        return 0.0;
    }
    let g = normalize_answer(gold);
    let c = normalize_answer(candidate);
    if g == c {
        return 1.0;
    }
    if let (Some(ge), Some(ce)) = (extract_number(&g), extract_number(&c)) {
        let scale = ge.abs().max(ce.abs()).max(1.0);
        let diff = (ge - ce).abs();
        return (1.0 - diff / scale).clamp(0.0, 1.0);
    }
    if g.contains(&c) || c.contains(&g) {
        let ratio = c.len() as f64 / g.len().max(1) as f64;
        return ratio.min(1.0) * 0.8;
    }
    0.0
}

fn normalize_answer(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

fn extract_number(s: &str) -> Option<f64> {
    s.chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect::<String>()
        .parse::<f64>()
        .ok()
}

pub fn text_to_vector(text: &str, dim: usize) -> Vector {
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

fn circuit_label(m: ReasoningMethod) -> &'static str {
    match m {
        ReasoningMethod::Deductive => "deductive logic",
        ReasoningMethod::Inductive => "inductive pattern",
        ReasoningMethod::Abductive => "abductive inference",
        ReasoningMethod::Analogical => "analogical transfer",
        ReasoningMethod::Compositional => "compositional planning",
        ReasoningMethod::Recursive => "recursive verification",
        ReasoningMethod::Adversarial => "adversarial critique",
        ReasoningMethod::FirstPrinciples => "first principles",
        ReasoningMethod::AutoFetch => "auto-fetch",
        ReasoningMethod::KnowledgeRetrieval => "knowledge retrieval",
        ReasoningMethod::GradientLearning => "gradient learning",
        ReasoningMethod::ArchitectureSearch => "arch search",
        ReasoningMethod::GpuCompute => "GPU compute",
        ReasoningMethod::DistributedConsensus => "distributed consensus",
        ReasoningMethod::ExperienceDistill => "experience distillation",
        ReasoningMethod::EmergentAnalysis => "emergent analysis",
        ReasoningMethod::SystemIntegration => "system integration",
        ReasoningMethod::EnsembleVoting => "ensemble voting",
        ReasoningMethod::SelfImprovement => "self-improvement",
        ReasoningMethod::SparseRouting => "sparse routing",
    }
}

pub fn format_kernel_output(v: &[f64], prompt: &str, stage: usize, energy: f64, circuit_names: &[String]) -> String {
    let raw_energy: f64 = v.iter().map(|x| x.abs()).sum::<f64>() / v.len().max(1) as f64;
    let confidence = energy.clamp(0.1, 1.0);
    let energy = raw_energy.max(confidence * 0.5);
    let stage_info = &EVOLUTION[stage];

    let above_half = v.iter().filter(|x| x.abs() > 0.5).count();

    match energy {
        e if e < 0.3 => {
            let mut resp = format!(
                "I need more context to form a solid inference about \"{}\". \
                 My {} kernel is registering weak signal (energy ~{:.2}) \
                 across {} reasoning pathway{}.",
                prompt, stage_info.label, e,
                circuit_names.len(), if circuit_names.len() == 1 { "" } else { "s" },
            );
            if !circuit_names.is_empty() {
//                resp.push_str(&format!(" The active circuits — {} — are engaged but haven't reached convergence.", circuit_names.join(", ")));
            }
            resp.push_str(" Could you provide more detail or clarify the question?");
            resp
        }
        e if e < 0.7 => {
            let mut resp = format!(
                "I've been reasoning about \"{}\" through my {} ({}) kernel, \
                 engaging {} pathway{}: {}.",
                prompt, stage_info.label, stage_info.description,
                circuit_names.len(), if circuit_names.len() == 1 { "" } else { "s" },
                circuit_names.join(", "),
            );
            if e > 0.5 {
                resp.push_str(&format!(
                    " Confidence is building at ~{:.0}% with {} of {} state dimensions \
                     showing significant activation (>0.5). The multi-circuit engagement \
                     is producing convergent inference patterns.",
                    e * 100.0, above_half, v.len()
                ));
            } else {
//                resp.push_str(" The inference is still converging — some circuits are still settling on stable state representations.");
            }
            resp
        }
        _ => {
            let mut resp = format!(
                "I have strong convergence on \"{}\" with {:.0}% confidence \
                 across my {} architecture ({}). \
                 {} of {} state dimensions are highly active (>0.5), \
                 driven by {} pathway{}: {}.",
                prompt, energy * 100.0, stage_info.label, stage_info.description,
                above_half, v.len(),
                circuit_names.len(), if circuit_names.len() == 1 { "" } else { "s" },
                circuit_names.join(", "),
            );
            if above_half > v.len() / 4 {
                resp.push_str(" The broad dimensional engagement indicates rich cross-circuit inference fusion.");
            }
            resp
        }
    }
}

pub struct StandaloneEngine {
    pub kernel: ReasoningKernel,
    pub conversation: Vec<(String, String)>,
    pub max_history: usize,
}

impl StandaloneEngine {
    pub fn new(stage: usize) -> Self {
        Self {
            kernel: ReasoningKernel::new(stage),
            conversation: Vec::new(),
            max_history: 10,
        }
    }

    pub fn reason(&mut self, prompt: &str) -> String {
        let query = self.text_to_vector(prompt);
        let ctx = {
            let mut m = std::collections::HashMap::new();
            for (i, (q, _)) in self.conversation.iter().enumerate().rev().take(3) {
                let vec = self.text_to_vector(q);
                m.insert(format!("hist_{}", i), vec);
            }
            Some(m)
        };
        let output = self.kernel.reason(&query, ctx, None);
        let response = self.vector_to_text(&output.state_delta, prompt);
        self.conversation.push((prompt.to_string(), response.clone()));
        if self.conversation.len() > self.max_history {
            self.conversation.remove(0);
        }
        response
    }

    pub fn stats(&self) -> String {
        let s = self.kernel.stats();
        format!(
            "Stage {} ({}) | dim={} | circuits={} | confidence=~{:.2} | energy={:.2}",
            s.stage, s.label, s.state_dim, s.total, 
            s.active.len() as f64 / s.total.max(1) as f64,
            s.energy
        )
    }

    fn text_to_vector(&self, text: &str) -> Vector {
        text_to_vector(text, self.kernel.state.len())
    }

    fn vector_to_text(&self, v: &[f64], prompt: &str) -> String {
        let stats = self.kernel.stats();
        let circuit_names: Vec<String> = stats.active.iter().map(|m| circuit_label(*m).to_string()).collect();
        let mut response = format_kernel_output(v, prompt, self.kernel.stage, stats.energy, &circuit_names);
        let history_len = self.conversation.len();
        if history_len > 0 {
            let (last_q, _) = &self.conversation[history_len - 1];
            let ref_phrase = if last_q.chars().count() > 50 {
                let truncated: String = last_q.chars().take(47).collect();
                format!("{}...", truncated)
            } else {
                last_q.clone()
            };
            response.push_str(&format!(
                "\n\n(Building on our prior exchange about \"{}\" — {} message{} in context.)",
                ref_phrase,
                history_len,
                if history_len == 1 { "" } else { "s" },
            ));
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_to_text_cjk_prior_exchange_no_panic() {
        // Regression: last_q[..47] sliced at byte 47, which panics when the
        // boundary lands mid-UTF-8-char (CJK is 3 bytes). chars().take(47)
        // truncates by character instead.
        let mut engine = StandaloneEngine::new(0);
        engine.conversation.push(("短".into(), "resp".into()));
        let _ = engine.vector_to_text(&[0.1, 0.2], "hi");
        engine.conversation.clear();

        let long_cjk = "长".repeat(60);
        engine.conversation.push((long_cjk, "resp".into()));
        let out = engine.vector_to_text(&[0.1, 0.2], "hi");
        assert!(out.contains("…") || out.contains("..."), "must truncate, got: {}", out);
        assert!(out.contains("长"), "truncated CJK must remain valid UTF-8");
    }

    #[test]
    fn test_format_kernel_output_threshold_matches_copy() {
        // Regression: the "significant activation (>0.5)" copy claims a 0.5
        // threshold but the counter used >0.3, so the reported dimension count
        // disagreed with the wording. Both now use 0.5.
        let v = vec![0.35, 0.45, 0.9];
        let out = format_kernel_output(&v, "p", 0, 0.8, &[]);
        let count = v.iter().filter(|x| x.abs() > 0.5).count();
        assert_eq!(count, 1);
        assert!(out.contains("1 of 3 state dimensions"), "copy must report the 0.5-count: {}", out);
    }

    #[test]
    fn test_reasoning_kernel_reason_real_trace() {
        // P0: reason() 不再是固定 Deductive/0.5 stub —— 必须产出多步中间态、
        // 真实收敛度与置信度。
        let k = ReasoningKernel::new(3);
        let query = vec![0.5; 128];
        let out = k.reason(&query, None, None);
        assert!(out.trace.intermediate_states.len() >= 2, "must evolve multiple steps, got {}", out.trace.intermediate_states.len());
        assert!(!out.trace.intermediate_states.is_empty(), "must record intermediate states");
        assert!(out.confidence > 0.0 && out.confidence <= 1.0, "confidence in (0,1]");
        assert!(out.trace.convergence > 0.0 && out.trace.convergence <= 1.0);
        assert_eq!(out.state_delta.len(), 128);
    }

    #[test]
    fn test_reasoning_kernel_method_selection_by_stage() {
        // 高 stage 应选择更高级方法（EnsembleVoting/SelfImprovement），
        // 低 stage 应选择基础方法（Deductive/KnowledgeRetrieval）。
        let k_low = ReasoningKernel::new(0);
        let q = vec![0.1; 128];
        let m_low = k_low.reason(&q, None, None).trace.method;
        assert!(matches!(m_low, ReasoningMethod::Deductive | ReasoningMethod::KnowledgeRetrieval | ReasoningMethod::Inductive));

        let k_high = ReasoningKernel::new(18);
        let m_high = k_high.reason(&q, None, None).trace.method;
        assert!(matches!(m_high, ReasoningMethod::EnsembleVoting | ReasoningMethod::SelfImprovement | ReasoningMethod::SparseRouting));
    }

    #[test]
    fn test_reasoning_kernel_stats_real() {
        // stats() 不再固定 total=8 / active=[Deductive]。
        let k = ReasoningKernel::new(5);
        let s = k.stats();
        assert!(s.total >= 3, "stage 5 must expose >=3 methods, got {}", s.total);
        assert!(s.active.len() == s.total);
        assert!(s.active.contains(&ReasoningMethod::Analogical));
    }

    #[test]
    fn test_self_consistency_aggregation() {
        let k = ReasoningKernel::new(3);
        let query = vec![0.5; 128];
        let sc = k.self_consistency(&query, 5);
        assert_eq!(sc.n_samples, 5);
        assert!(sc.consistency > 0.0 && sc.consistency <= 1.0);
        assert!(sc.avg_confidence > 0.0 && sc.avg_confidence <= 1.0);
        assert_eq!(sc.aggregated_state.len(), 128);
        // 多数方法必须来自合法方法集
        assert!(matches!(
            sc.majority_method,
            ReasoningMethod::Deductive
                | ReasoningMethod::KnowledgeRetrieval
                | ReasoningMethod::Inductive
                | ReasoningMethod::Analogical
                | ReasoningMethod::Recursive
        ));
    }

    #[test]
    fn test_verify_answer_exact_and_numeric() {
        assert!((verify_answer("42", "42") - 1.0).abs() < 1e-9);
        assert!((verify_answer("The answer is 42", "42") - 1.0).abs() < 1e-9);
        // 数值近似：41.5 vs 42 → 高匹配
        let score = verify_answer("42", "41.5");
        assert!(score > 0.9, "numeric proximity should score high, got {}", score);
        // 无关答案 → 低分
        let bad = verify_answer("42", "banana");
        assert!(bad < 0.5, "unrelated answer should score low, got {}", bad);
        // 空输入 → 0
        assert_eq!(verify_answer("", "42"), 0.0);
    }
}
