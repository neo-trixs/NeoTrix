//! ProcessStage — 过程知识习得阶段 (CoT/推理链监督微调)
//!
//! smol-course 吸收：Unit 1 thinking mode + Unit 3 VLM reasoning → 过程监督
//! 从 ConsciousnessTree EvolutionFruit + KB experience GoldTrajectory 提取推理轨迹，
//! 用过程级 cross-entropy loss 监督微调，学习"如何推理/分解任务/调用工具"。
//!
//! 产出：CapabilityVector 新增 extension 维度 (reasoning_depth, cot_quality, tool_use_fluency)

use std::collections::VecDeque;
use serde::{Serialize, Deserialize};
use super::pipeline::StageResult;

/// 最大轨迹缓冲
pub const PROCESS_BUFFER_SIZE: usize = 500;

/// 过程监督学习率 (smol-course SFT lr 5e-5 为基准，过程级稍大)
pub const PROCESS_LEARNING_RATE: f64 = 1e-4;

/// 单条推理轨迹步骤 (对齐 TrajectoryStep / EvidenceChain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub step_idx: usize,
    pub specialist: String,           // 专家角色: "RiskAssessor" | "Planner" | "Coder" | "Searcher" | ...
    pub e8_mode: u8,                  // E8 推理模式 (0-15)
    pub action: String,               // 动作: "think" | "search" | "code" | "verify" | ...
    pub input: String,                // 步骤输入 (问题/查询/上下文)
    pub output: String,               // 步骤输出 (思考/结果/代码)
    pub duration_ms: Option<u64>,
    pub success: bool,
    pub reward: Option<f64>,          // 步骤级奖励 (PRM)
}

/// 完整推理轨迹 (对齐 AgentTrajectory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTrace {
    pub trace_id: String,
    pub task: String,
    pub steps: Vec<ReasoningStep>,
    pub completed: bool,
    pub final_quality: f64,           // 最终输出质量 (0-1)
    pub source: TraceSource,          // 来源: ConsciousnessTree | KBExperience | Synthesis
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceSource {
    ConsciousnessTree,
    KBExperience,
    Synthesis,
}

/// 过程监督样本：任务 → 期望推理轨迹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessExample {
    pub trace: ReasoningTrace,
    pub weight: f64,                  // 样本权重 (质量 * 来源可信度)
}

/// 过程缓冲
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessBuffer {
    pub traces: VecDeque<ReasoningTrace>,
    pub max_size: usize,
}

impl Default for ProcessBuffer {
    fn default() -> Self { Self::new() }
}

impl ProcessBuffer {
    pub fn new() -> Self {
        Self { traces: VecDeque::with_capacity(PROCESS_BUFFER_SIZE), max_size: PROCESS_BUFFER_SIZE }
    }
    pub fn push(&mut self, trace: ReasoningTrace) {
        if self.traces.len() >= self.max_size { self.traces.pop_front(); }
        self.traces.push_back(trace);
    }
    pub fn len(&self) -> usize { self.traces.len() }
    pub fn is_empty(&self) -> bool { self.traces.is_empty() }
    pub fn clear(&mut self) { self.traces.clear(); }
}

/// 过程阶段统计报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessReport {
    pub total_updates: u64,
    pub buffer_size: usize,
    pub avg_trace_length: f64,
    pub avg_final_quality: f64,
    pub specialist_distribution: std::collections::HashMap<String, usize>,
    pub action_distribution: std::collections::HashMap<String, usize>,
    pub extension_dims_updated: Vec<String>,
}

/// Process Stage for SEAL pipeline.
///
/// 过程知识习得：从高质量推理轨迹学习"如何推理"，扩展 CapabilityVector 过程维度。
#[derive(Debug, Clone)]
pub struct ProcessStage {
    pub buffer: ProcessBuffer,
    pub learning_rate: f64,
    pub total_updates: u64,
    /// 过程能力扩展维度 (动态累积到 CapabilityVector.extension)
    pub reasoning_depth: f64,         // 平均推理步数
    pub cot_quality: f64,             // CoT 质量分
    pub tool_use_fluency: f64,        // 工具调用流畅度
    pub decomposition_skill: f64,     // 任务分解能力
    pub verification_habit: f64,      // 自我验证习惯
}

impl Default for ProcessStage {
    fn default() -> Self { Self::new() }
}

impl ProcessStage {
    pub fn new() -> Self {
        Self {
            buffer: ProcessBuffer::new(),
            learning_rate: PROCESS_LEARNING_RATE,
            total_updates: 0,
            reasoning_depth: 0.0,
            cot_quality: 0.0,
            tool_use_fluency: 0.0,
            decomposition_skill: 0.0,
            verification_habit: 0.0,
        }
    }

    /// 从 ConsciousnessTree EvolutionFruit 提取推理轨迹
    pub fn extract_from_consciousness_tree(
        fruits: &[crate::core::nt_core_consciousness_tree::EvolutionFruit],
    ) -> Vec<ReasoningTrace> {
        let mut traces = Vec::new();
        for fruit in fruits {
            // EvidenceChain.run_id 格式: "cycle-{cycle}-{kind}"
            if let Some(run_id) = &fruit.evidence.run_id {
                // 从 run_id 反推 cycle，这里简化：用 fruit 生成的 cycle
                let trace = ReasoningTrace {
                    trace_id: format!("ct-{}-{}", fruit.produced_at_cycle, fruit.source_branch.label()),
                    task: fruit.claim.clone(),
                    steps: vec![ReasoningStep {
                        step_idx: 0,
                        specialist: fruit.source_branch.label().to_string(),
                        e8_mode: 0, // 需从 branch 状态推断
                        action: "evolve".to_string(),
                        input: fruit.description.clone(),
                        output: fruit.claim.clone(),
                        duration_ms: None,
                        success: fruit.benchmark.accuracy > 0.5,
                        reward: Some(fruit.benchmark.accuracy),
                    }],
                    completed: true,
                    final_quality: fruit.quality,
                    source: TraceSource::ConsciousnessTree,
                    timestamp: fruit.evidence.timestamp,
                };
                traces.push(trace);
            }
        }
        traces
    }

    /// 从 KB Experience GoldTrajectory 提取推理轨迹
    pub fn extract_from_kb_experience(
        trajectories: &[crate::core::nt_core_prm::AgentTrajectory],
    ) -> Vec<ReasoningTrace> {
        trajectories.iter().enumerate().map(|(i, traj)| {
            let steps: Vec<ReasoningStep> = traj.steps.iter().map(|s| ReasoningStep {
                step_idx: s.step_idx,
                specialist: format!("{:?}", s.specialist),
                e8_mode: s.e8_mode.0 & 0x3F,
                action: s.action.clone(),
                input: s.input.clone(),
                output: s.output.clone(),
                duration_ms: s.duration_ms,
                success: s.success,
                reward: s.external_reward,
            }).collect();
            let avg_reward = steps.iter().filter_map(|s| s.reward).sum::<f64>() / steps.len().max(1) as f64;
            ReasoningTrace {
                trace_id: format!("kb-{}-{}", traj.trajectory_id, i),
                task: traj.task.clone(),
                steps,
                completed: traj.completed,
                final_quality: avg_reward.clamp(0.0, 1.0),
                source: TraceSource::KBExperience,
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
            }
        }).collect()
    }

    /// 计算过程级 SFT loss (过程 cross-entropy 类比)
    /// L_process = -E[weight * log p(trace | task)]
    /// 这里简化为：对每个过程维度的 EMA 与目标分布的偏差
    fn compute_process_loss(&self) -> f64 {
        if self.buffer.is_empty() { return 0.0; }
        let n = self.buffer.len() as f64;
        let mut total = 0.0;
        for trace in &self.buffer.traces {
            // 目标分布：高质量轨迹的统计特征
            let target_depth = (trace.steps.len() as f64 / 10.0).min(1.0);      // 归一化步数
            let target_cot = trace.final_quality;
            let target_tool = trace.steps.iter().filter(|s| s.action == "search" || s.action == "code").count() as f64 / trace.steps.len().max(1) as f64;
            let target_decomp = trace.steps.iter().filter(|s| s.action == "think" || s.action == "plan").count() as f64 / trace.steps.len().max(1) as f64;
            let target_verify = trace.steps.iter().filter(|s| s.action == "verify").count() as f64 / trace.steps.len().max(1) as f64;

            // 当前能力分布 (EMA 累积值)
            let margin_depth = self.reasoning_depth - target_depth;
            let margin_cot = self.cot_quality - target_cot;
            let margin_tool = self.tool_use_fluency - target_tool;
            let margin_decomp = self.decomposition_skill - target_decomp;
            let margin_verify = self.verification_habit - target_verify;

            // softplus(-margin) = -log σ(margin) 数值稳定
            let loss = |m: f64| if m > 0.0 { (-m).exp().ln_1p() } else { m.exp().ln_1p() - m };
            total += trace.final_quality.max(1e-6) * (loss(margin_depth) + loss(margin_cot) + loss(margin_tool) + loss(margin_decomp) + loss(margin_verify)) / 5.0;
        }
        total / n
    }

    /// 处理一批过程样本：更新过程能力 EMA
    pub fn process(&mut self, examples: Vec<ProcessExample>) -> (StageResult, f64) {
        let result = StageResult::new("process_stage");
        if examples.is_empty() { return (result, 0.0); }

        for ex in &examples {
            self.buffer.push(ex.trace.clone());
            let trace = &ex.trace;
            let w = ex.weight * self.learning_rate;

            // 目标特征
            let target_depth = (trace.steps.len() as f64 / 10.0).min(1.0);
            let target_cot = trace.final_quality;
            let target_tool = trace.steps.iter().filter(|s| s.action == "search" || s.action == "code").count() as f64 / trace.steps.len().max(1) as f64;
            let target_decomp = trace.steps.iter().filter(|s| s.action == "think" || s.action == "plan").count() as f64 / trace.steps.len().max(1) as f64;
            let target_verify = trace.steps.iter().filter(|s| s.action == "verify").count() as f64 / trace.steps.len().max(1) as f64;

            // EMA 更新
            self.reasoning_depth = self.reasoning_depth * (1.0 - w) + target_depth * w;
            self.cot_quality = self.cot_quality * (1.0 - w) + target_cot * w;
            self.tool_use_fluency = self.tool_use_fluency * (1.0 - w) + target_tool * w;
            self.decomposition_skill = self.decomposition_skill * (1.0 - w) + target_decomp * w;
            self.verification_habit = self.verification_habit * (1.0 - w) + target_verify * w;
        }
        self.total_updates += 1;

        let loss = self.compute_process_loss();
        log::trace!(
            "[process_stage] loss={:.4} depth={:.3} cot={:.3} tool={:.3} decomp={:.3} verify={:.3} traces={}",
            loss, self.reasoning_depth, self.cot_quality, self.tool_use_fluency,
            self.decomposition_skill, self.verification_habit, self.buffer.len()
        );
        (result, loss)
    }

    /// 将过程能力写入 CapabilityVector.extension
    pub fn sync_to_capability_vector(&self, cv: &mut crate::core::CapabilityVector) {
        cv.add_extension_dim("nt_cap:reasoning_depth", self.reasoning_depth);
        cv.add_extension_dim("nt_cap:cot_quality", self.cot_quality);
        cv.add_extension_dim("nt_cap:tool_use_fluency", self.tool_use_fluency);
        cv.add_extension_dim("nt_cap:decomposition_skill", self.decomposition_skill);
        cv.add_extension_dim("nt_cap:verification_habit", self.verification_habit);
        cv.set_provenance("process_stage".to_string());
    }

    /// 从 CapabilityVector 读取过程能力
    pub fn load_from_capability_vector(cv: &crate::core::CapabilityVector) -> Self {
        let mut stage = Self::new();
        for (name, val) in cv.extension() {
            match name.as_str() {
                "nt_cap:reasoning_depth" => stage.reasoning_depth = *val,
                "nt_cap:cot_quality" => stage.cot_quality = *val,
                "nt_cap:tool_use_fluency" => stage.tool_use_fluency = *val,
                "nt_cap:decomposition_skill" => stage.decomposition_skill = *val,
                "nt_cap:verification_habit" => stage.verification_habit = *val,
                _ => {}
            }
        }
        stage
    }

    pub fn report(&self) -> ProcessReport {
        use std::collections::HashMap;
        let mut specialist_dist = HashMap::new();
        let mut action_dist = HashMap::new();
        let mut total_len = 0.0;
        let mut total_quality = 0.0;
        for trace in &self.buffer.traces {
            total_len += trace.steps.len() as f64;
            total_quality += trace.final_quality;
            for step in &trace.steps {
                *specialist_dist.entry(step.specialist.clone()).or_insert(0) += 1;
                *action_dist.entry(step.action.clone()).or_insert(0) += 1;
            }
        }
        let n = self.buffer.len().max(1) as f64;
        ProcessReport {
            total_updates: self.total_updates,
            buffer_size: self.buffer.len(),
            avg_trace_length: total_len / n,
            avg_final_quality: total_quality / n,
            specialist_distribution: specialist_dist,
            action_distribution: action_dist,
            extension_dims_updated: vec![
                "nt_cap:reasoning_depth".into(),
                "nt_cap:cot_quality".into(),
                "nt_cap:tool_use_fluency".into(),
                "nt_cap:decomposition_skill".into(),
                "nt_cap:verification_habit".into(),
            ],
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trace(steps: usize, quality: f64, actions: Vec<&str>) -> ReasoningTrace {
        ReasoningTrace {
            trace_id: "test".into(),
            task: "test task".into(),
            steps: actions.into_iter().enumerate().map(|(i, a)| ReasoningStep {
                step_idx: i, specialist: "Tester".into(), e8_mode: 0, action: a.into(),
                input: "in".into(), output: "out".into(), duration_ms: None, success: true, reward: Some(quality),
            }).collect(),
            completed: true, final_quality: quality, source: TraceSource::Synthesis, timestamp: current_timestamp(),
        }
    }

    #[test]
    fn test_buffer_push_and_len() {
        let mut buf = ProcessBuffer::new();
        buf.push(make_trace(3, 0.9, vec!["think", "search", "verify"]));
        assert_eq!(buf.len(), 1);
    }

    #[test]
    fn test_process_updates_ema() {
        let mut stage = ProcessStage::new();
        stage.learning_rate = 0.5;
        let ex = ProcessExample { trace: make_trace(4, 1.0, vec!["think", "plan", "code", "verify"]), weight: 1.0 };
        stage.process(vec![ex]);
        // EMA: 0*(1-0.5) + target*0.5
        assert!((stage.reasoning_depth - 0.2).abs() < 1e-6); // 4/10 * 0.5
        assert!((stage.cot_quality - 0.5).abs() < 1e-6);
        assert!((stage.verification_habit - 0.125).abs() < 1e-6); // 1/4 * 0.5
    }

    #[test]
    fn test_sync_to_capability_vector() {
        let mut stage = ProcessStage::new();
        stage.reasoning_depth = 0.7; stage.cot_quality = 0.8;
        let mut cv = crate::core::CapabilityVector::default();
        stage.sync_to_capability_vector(&mut cv);
        assert!((cv.extension().iter().find(|(n,_)| n=="nt_cap:reasoning_depth").unwrap().1 - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_load_from_capability_vector() {
        let mut cv = crate::core::CapabilityVector::default();
        cv.add_extension_dim("nt_cap:tool_use_fluency", 0.6);
        cv.add_extension_dim("nt_cap:decomposition_skill", 0.4);
        let stage = ProcessStage::load_from_capability_vector(&cv);
        assert!((stage.tool_use_fluency - 0.6).abs() < 1e-9);
        assert!((stage.decomposition_skill - 0.4).abs() < 1e-9);
    }

    #[test]
    fn test_compute_process_loss() {
        let mut stage = ProcessStage::new();
        stage.reasoning_depth = 0.5; stage.cot_quality = 0.5;
        stage.buffer.push(make_trace(5, 0.5, vec!["think", "search", "code", "verify", "think"]));
        let loss = stage.compute_process_loss();
        assert!(loss >= 0.0);
    }

    #[test]
    fn test_report_tracks_distributions() {
        let mut stage = ProcessStage::new();
        stage.process(vec![ProcessExample { trace: make_trace(3, 0.9, vec!["think", "search", "verify"]), weight: 1.0 }]);
        let r = stage.report();
        assert_eq!(r.buffer_size, 1);
        assert!(r.specialist_distribution.contains_key("Tester"));
        assert!(r.action_distribution.contains_key("search"));
    }
}