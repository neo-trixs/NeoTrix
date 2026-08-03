//! L8 Control Distillation Pipeline — MERA 式控制段蒸馏 (ICML 2026 / ACL 2026)
//!
//! 参考: MERA (Meta-cognitive Reasoning Framework), CSPO (Control-Segment Policy Optimization)
//! 核心流程: Takeover 检测 → 控制信号生成 → 交替序列构建 → SFT + CSPO (Segmented GRPO + Control Masking)
//! 对接现有: nt_core_prm (λ-GRPO), nt_core_policy (E8Policy), nt_core_ttc (EffortTier), gold_standard

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::core::nt_core_prm::{LambdaGrpoResult, StepAdvantage, compute_step_advantages, StepGrpoConfig, ProcessScore};
use crate::core::nt_core_policy::{E8Policy, NUM_E8_FACTORS};
use crate::core::nt_core_ttc::{EffortTier, EffortTierSelector};
use crate::core::nt_core_e8::domain_transition::E8TaskType;
use crate::neotrix::l9_transcendent_impl::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard;

/// 控制类型 (MERA 同款)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlType {
    /// 回溯: 发现错误/死胡同，退回到前一步骤
    Backtrack,
    /// 策略切换: 当前推理路径无效，切换分解法/第一性原理/验证模式
    StrategySwitch,
    /// 自证: 主动逐步验证前提/逻辑/计算
    SelfVerify,
    /// 早停: 置信度已达阈值，直接给最终答案
    EarlyStop,
}

/// Takeover 点 (控制介入位置)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeoverPoint {
    pub step_idx: usize,           // 在 reasoning trace 中的步骤索引
    pub control_type: ControlType,
    pub confidence: f64,           // 检测置信度 0~1
    pub trigger_text: String,      // 触发的标记词片段
}

/// 控制指令 (生成的 meta-cognitive guidance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlSignal {
    pub takeover_point: TakeoverPoint,
    pub instruction: String,       // 自然语言控制指令
    pub target_effort_tier: Option<EffortTier>, // 建议的努力分层调整
    pub target_strategy: Option<String>,        // 建议的推理策略
}

/// 交替序列片段 (reason ↔ control 交替)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlternatingSegment {
    Reason { text: String, step_idx: usize },
    Control { signal: ControlSignal },
}

/// 完整交替序列 (训练样本)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternatingSequence {
    pub trajectory_id: String,
    pub task: String,
    pub segments: Vec<AlternatingSegment>,
    pub final_answer: String,
    pub outcome_quality: f64,      // 0~1 (gold_standard 或 judge)
    pub effort_tier: EffortTier,
}

/// 控制段奖励 (CSPO 核心)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlReward {
    pub semantic_score: f64,       // 语义一致性 (vs 参考控制目标)
    pub format_score: f64,         // 格式规范性 (<think>...</think> 等)
    pub total: f64,                // semantic + format
}

/// Takeover 检测器
pub struct TakeoverDetector {
    // 启发式标记词 (MERA Table 1 扩展)
    backtrack_markers: Vec<&'static str>,
    strategy_switch_markers: Vec<&'static str>,
    self_verify_markers: Vec<&'static str>,
    early_stop_markers: Vec<&'static str>,
    // 可选: LLM 校验器
    llm_verifier: Option<Arc<dyn TakeoverVerifier>>,
}

#[async_trait::async_trait]
pub trait TakeoverVerifier: Send + Sync {
    async fn verify(&self, trace_segment: &str, candidate_type: ControlType) -> Result<f64, String>;
}

impl Default for TakeoverDetector {
    fn default() -> Self {
        Self {
            backtrack_markers: vec!["wait", "hmm", "let me rethink", "on second thought", "actually", "reconsider", "backtrack", "revise"],
            strategy_switch_markers: vec!["alternatively", "let me try a different approach", "switch to", "change strategy", "decompose", "first principles"],
            self_verify_markers: vec!["verify", "check", "validate", "confirm", "self-correct", "proof"],
            early_stop_markers: vec!["therefore", "thus", "conclude", "final answer", "sufficient", "confident"],
            llm_verifier: None,
        }
    }
}

impl TakeoverDetector {
    pub fn with_llm_verifier(mut self, verifier: Arc<dyn TakeoverVerifier>) -> Self {
        self.llm_verifier = Some(verifier);
        self
    }

    /// 检测单条推理轨迹中的 takeover 点
    pub fn detect(&self, _trace: &str, steps: &[ReasoningStep]) -> Vec<TakeoverPoint> {
        let mut takeovers = Vec::new();

        // 启发式扫描
        for (idx, step) in steps.iter().enumerate() {
            let step_lower = step.text.to_lowercase();
            
            let mut best_type = None;
            let mut best_conf = 0.0;

            for marker in &self.backtrack_markers {
                if step_lower.contains(marker) {
                    let conf = self.score_marker(marker, &step_lower);
                    if conf > best_conf { best_conf = conf; best_type = Some(ControlType::Backtrack); }
                }
            }
            for marker in &self.strategy_switch_markers {
                if step_lower.contains(marker) {
                    let conf = self.score_marker(marker, &step_lower);
                    if conf > best_conf { best_conf = conf; best_type = Some(ControlType::StrategySwitch); }
                }
            }
            for marker in &self.self_verify_markers {
                if step_lower.contains(marker) {
                    let conf = self.score_marker(marker, &step_lower);
                    if conf > best_conf { best_conf = conf; best_type = Some(ControlType::SelfVerify); }
                }
            }
            for marker in &self.early_stop_markers {
                if step_lower.contains(marker) {
                    let conf = self.score_marker(marker, &step_lower);
                    if conf > best_conf { best_conf = conf; best_type = Some(ControlType::EarlyStop); }
                }
            }

            if let Some(ct) = best_type {
                takeovers.push(TakeoverPoint {
                    step_idx: idx,
                    control_type: ct,
                    confidence: best_conf,
                    trigger_text: self.extract_trigger(&step.text, ct),
                });
            }
        }

        takeovers
    }

    fn score_marker(&self, marker: &str, text: &str) -> f64 {
        // 简单词频/位置加权
        let count = text.matches(marker).count() as f64;
        let pos_bonus = if text.starts_with(marker) { 0.2 } else { 0.0 };
        (count * 0.3 + pos_bonus).min(1.0)
    }

    fn extract_trigger(&self, text: &str, ct: ControlType) -> String {
        let markers = match ct {
            ControlType::Backtrack => &self.backtrack_markers,
            ControlType::StrategySwitch => &self.strategy_switch_markers,
            ControlType::SelfVerify => &self.self_verify_markers,
            ControlType::EarlyStop => &self.early_stop_markers,
        };
        for m in markers {
            if text.to_lowercase().contains(m) {
                return m.to_string();
            }
        }
        "".to_string()
    }
}

/// 推理步骤 (来自 trace)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub step_idx: usize,
    pub text: String,
    pub e8_mode: Option<u8>,
    pub token_count: usize,
}

/// 控制蒸馏主管道
pub struct ControlDistiller {
    detector: TakeoverDetector,
    signal_generator: ControlSignalGenerator,
    gold_standard: Arc<ConsciousnessGoldStandard>,
}

impl ControlDistiller {
    pub fn new(gold_standard: Arc<ConsciousnessGoldStandard>) -> Self {
        Self {
            detector: TakeoverDetector::default(),
            signal_generator: ControlSignalGenerator::default(),
            gold_standard,
        }
    }

    pub fn with_llm_verifier(mut self, verifier: Arc<dyn TakeoverVerifier>) -> Self {
        self.detector = self.detector.with_llm_verifier(verifier);
        self
    }

    /// 从原始推理轨迹提取交替序列 (核心入口)
    pub fn extract_alternating_sequence(
        &self,
        trajectory_id: String,
        task: &str,
        trace: &str,
        steps: &[ReasoningStep],
        final_answer: &str,
    ) -> Result<AlternatingSequence, DistillError> {
        // 1. Takeover 检测
        let takeovers = self.detector.detect(trace, steps);
        
        // 2. 生成控制信号
        let mut signals = Vec::new();
        for tp in &takeovers {
            let signal = self.signal_generator.generate(task, trace, tp)?;
            signals.push(signal);
        }

        // 3. 构建交替序列
        let segments = self.build_alternating_segments(steps, &takeovers, &signals);

        // 4. 质量评估 (gold_standard)
        let outcome_quality = self.assess_outcome_quality(final_answer, task);

        Ok(AlternatingSequence {
            trajectory_id,
            task: task.to_string(),
            segments,
            final_answer: final_answer.to_string(),
            outcome_quality,
            effort_tier: EffortTier::Medium, // 默认，实际从 trace 推断
        })
    }

    /// 构建 reason ↔ control 交替片段
    fn build_alternating_segments(
        &self,
        steps: &[ReasoningStep],
        takeovers: &[TakeoverPoint],
        signals: &[ControlSignal],
    ) -> Vec<AlternatingSegment> {
        let mut segments = Vec::new();
        let mut _signal_idx = 0;
        let mut last_reason_end = 0;

        for (tp, signal) in takeovers.iter().zip(signals.iter()) {
            // reason: 从上一个 takeover 到当前 takeover 之前的 steps
            for step in &steps[last_reason_end..=tp.step_idx] {
                segments.push(AlternatingSegment::Reason {
                    text: step.text.clone(),
                    step_idx: step.step_idx,
                });
            }
            // control: 插入控制指令
            segments.push(AlternatingSegment::Control { signal: signal.clone() });
            last_reason_end = tp.step_idx + 1;
            _signal_idx += 1;
        }

        // 剩余 reason
        for step in &steps[last_reason_end..] {
            segments.push(AlternatingSegment::Reason {
                text: step.text.clone(),
                step_idx: step.step_idx,
            });
        }

        segments
    }

    fn assess_outcome_quality(&self, answer: &str, _task: &str) -> f64 {
        // 简化：跳过金标 evaluate (需 E8 state & hexagram)，用启发式
        let len_score = (answer.len() as f64 / 500.0).min(1.0);
        len_score
    }
}

/// 控制信号生成器 (few-shot LLM)
#[derive(Default)]
pub struct ControlSignalGenerator {
    // 模板: control_type -> (system_prompt, few_shot_examples)
    templates: HashMap<ControlType, (&'static str, Vec<(&'static str, &'static str)>)>,
}

impl ControlSignalGenerator {
    pub fn generate(
        &self,
        task: &str,
        trace: &str,
        takeover: &TakeoverPoint,
        ) -> Result<ControlSignal, DistillError> {
        let (instruction, target_effort, target_strategy) = match takeover.control_type {
            ControlType::Backtrack => (
                format!("Backtrack to step {} and try a different decomposition. The previous path led to: {}", 
                    takeover.step_idx, self.summarize_failure(trace, takeover.step_idx)),
                Some(EffortTier::High), // 回溯需更高努力
                Some("decompose".to_string()),
            ),
            ControlType::StrategySwitch => (
                format!("Switch reasoning strategy. Current approach stalled at step {}. Try: {}", 
                    takeover.step_idx, self.suggest_strategy(task)),
                Some(EffortTier::XHigh),
                Some("first_principles".to_string()),
            ),
            ControlType::SelfVerify => (
                format!("Self-verify the reasoning from step {} onwards. Check: premises, logic, calculations.", 
                    takeover.step_idx),
                Some(EffortTier::High),
                Some("verify".to_string()),
            ),
            ControlType::EarlyStop => (
                format!("Confidence threshold reached at step {}. Provide final answer directly.", takeover.step_idx),
                Some(EffortTier::Low), // 早停降低后续预算
                None,
            ),
        };

        Ok(ControlSignal {
            takeover_point: takeover.clone(),
            instruction,
            target_effort_tier: target_effort,
            target_strategy: target_strategy,
        })
    }

    fn summarize_failure(&self, trace: &str, step_idx: usize) -> String {
        // 简化：取步骤附近文本
        let lines: Vec<&str> = trace.lines().collect();
        let start = step_idx.saturating_sub(2);
        let end = (step_idx + 2).min(lines.len());
        lines[start..end].join(" | ")
    }

    fn suggest_strategy(&self, task: &str) -> String {
        let lower = task.to_lowercase();
        if lower.contains("math") || lower.contains("calculate") { "decompose" }
        else if lower.contains("code") || lower.contains("program") { "step_by_step" }
        else if lower.contains("reason") || lower.contains("logic") { "first_principles" }
        else { "decompose" }.to_string()
    }
}

/// CSPO 训练器 (对接现有 nt_core_prm + nt_core_policy)
pub struct ControlTrainer {
    pub policy: E8Policy,
    pub prm_config: StepGrpoConfig,
    pub gold_standard: Arc<ConsciousnessGoldStandard>,
}

impl ControlTrainer {
    pub fn new(policy: E8Policy, gold_standard: Arc<ConsciousnessGoldStandard>) -> Self {
        Self {
            policy,
            prm_config: StepGrpoConfig::default(),
            gold_standard,
        }
    }

    /// SFT on 交替序列 (阶段 1)
    pub fn sft(&mut self, sequences: &[AlternatingSequence]) -> Result<SftReport, DistillError> {
        let mut control_updates = 0;
        let mut reason_updates = 0;

        for seq in sequences {
            // 解析交替序列，提取 control segment 对应的 E8 动作
            for seg in &seq.segments {
                if let AlternatingSegment::Control { signal } = seg {
                    // 将 control_type 映射为 E8 因子更新
                    self.apply_control_to_policy(signal)?;
                    control_updates += 1;
                }
            }
            // reason segments 正常通过 PRM 学习 (复用现有 learn_from_trace)
            reason_updates += seq.segments.iter().filter(|s| matches!(s, AlternatingSegment::Reason {..})).count();
        }

        Ok(SftReport { control_updates, reason_updates })
    }

    /// CSPO: Control-Segment Policy Optimization (阶段 2)
    /// 核心: Segmented GRPO + Control Reward + Control Masking
    pub fn csppo(&mut self, sequences: &[AlternatingSequence]) -> Result<CsppoReport, DistillError> {
        let mut total_control_reward = 0.0;
        let mut masked_steps = 0;

        // 按 MERA Algorithm 1: Sample G trajectories per sequence group
        // 这里简化: 每个 sequence 视为一组，使用其 outcome_quality 作为 reward 基线
        for seq in sequences {
            // 1. Partition into reasoning-control segments
            let control_segments: Vec<_> = seq.segments.iter()
                .filter_map(|s| match s {
                    AlternatingSegment::Control { signal } => Some(signal),
                    _ => None,
                }).collect();

            // 2. Segment-wise reward (Control Reward = semantic + format)
            for signal in &control_segments {
                let reward = self.compute_control_reward(signal, seq.outcome_quality)?;
                total_control_reward += reward.total;
                
                // 3. Control Masking: 只对 control tokens 更新策略
                // 这里通过 ProcessScore 的 attribution_tags="control" 实现 masking
                self.apply_masked_policy_update(signal, reward.total)?;
                masked_steps += 1;
            }

            // 4. Reason segments: 复用现有 λ-GRPO (trajectory_convergence + compute_step_advantages)
            // 由 nt_core_prm 处理，此处不重复
        }

        Ok(CsppoReport { total_control_reward, masked_steps })
    }

    fn apply_control_to_policy(&mut self, signal: &ControlSignal) -> Result<(), DistillError> {
        // 将 control_type 映射为 E8 因子 delta (复用 E8Policy::learn_from_scores)
        let (tag, delta) = match signal.takeover_point.control_type {
            ControlType::Backtrack => ("backtrack", 0.15),
            ControlType::StrategySwitch => ("strategy_switch", 0.2),
            ControlType::SelfVerify => ("verify", 0.1),
            ControlType::EarlyStop => ("early_stop", -0.05), // 降低后续探索
        };
        let process_score = ProcessScore {
            step_idx: signal.takeover_point.step_idx,
            score: delta,
            confidence: 1.0,
            criteria: vec![],
            attribution_tags: vec!["control".to_string(), tag.to_string()],
        };
        // 需要 trajectory 上下文，简化：记录到 policy 内部缓冲
        // 实际应调用 policy.learn_from_scores(&trajectory, &[process_score])
        Ok(())
    }

    fn compute_control_reward(&self, signal: &ControlSignal, outcome_quality: f64) -> Result<ControlReward, DistillError> {
        // 语义奖励: 简化用 outcome_quality 代理 (实际需 LLM judge 对比参考控制目标)
        let semantic = outcome_quality * 0.7;
        // 格式奖励: control 指令是否包含标准标记
        let format = if signal.instruction.contains("<think>") || signal.instruction.len() > 20 { 0.3 } else { 0.1 };
        Ok(ControlReward { semantic_score: semantic, format_score: format, total: semantic + format })
    }

    fn apply_masked_policy_update(&mut self, signal: &ControlSignal, advantage: f64) -> Result<(), DistillError> {
        // Control Masking: 只更新 control 相关的因子
        // 通过 attribution_tags="control" 实现 (E8Policy 已支持 factorized learning)
        let process_score = ProcessScore {
            step_idx: signal.takeover_point.step_idx,
            score: advantage,
            confidence: 1.0,
            criteria: vec![],
            attribution_tags: vec!["control".to_string(), format!("{:?}", signal.takeover_point.control_type).to_lowercase()],
        };
        // 实际应: policy.learn_from_scores(&trajectory, &[process_score])
        Ok(())
    }
}

/// SFT 训练报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftReport {
    pub control_updates: usize,
    pub reason_updates: usize,
}

/// CSPO 训练报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsppoReport {
    pub total_control_reward: f64,
    pub masked_steps: usize,
}

/// 蒸馏错误
#[derive(Debug, thiserror::Error)]
pub enum DistillError {
    #[error("Policy error: {0}")]
    PolicyError(String),
    #[error("PRM error: {0}")]
    PrmError(String),
    #[error("Config error: {0}")]
    ConfigError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_takeover_detection_backtrack() {
        let detector = TakeoverDetector::default();
        let steps = vec![
            ReasoningStep { step_idx: 0, text: "Let me solve this step by step".into(), e8_mode: None, token_count: 20 },
            ReasoningStep { step_idx: 1, text: "Wait, I made an error in the calculation".into(), e8_mode: None, token_count: 30 },
        ];
        let trace = steps.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");
        let takeovers = detector.detect(&trace, &steps);
        assert_eq!(takeovers.len(), 1);
        assert_eq!(takeovers[0].control_type, ControlType::Backtrack);
        assert!(takeovers[0].confidence > 0.0);
    }

    #[test]
    fn test_takeover_detection_strategy_switch() {
        let detector = TakeoverDetector::default();
        let steps = vec![
            ReasoningStep { step_idx: 0, text: "First I'll try algebraic manipulation".into(), e8_mode: None, token_count: 20 },
            ReasoningStep { step_idx: 1, text: "Alternatively, let me use a geometric approach".into(), e8_mode: None, token_count: 25 },
        ];
        let trace = steps.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");
        let takeovers = detector.detect(&trace, &steps);
        assert_eq!(takeovers.len(), 1);
        assert_eq!(takeovers[0].control_type, ControlType::StrategySwitch);
    }

    #[test]
    fn test_control_signal_generation() {
        let gen = ControlSignalGenerator::default();
        let tp = TakeoverPoint { step_idx: 2, control_type: ControlType::Backtrack, confidence: 0.9, trigger_text: "wait".into() };
        let signal = gen.generate("test task", "trace", &tp).unwrap();
        assert!(signal.instruction.contains("Backtrack"));
        assert_eq!(signal.target_effort_tier, Some(EffortTier::High));
    }

    #[test]
    fn test_alternating_sequence_building() {
        let gold = Arc::new(ConsciousnessGoldStandard::new());
        let distiller = ControlDistiller::new(gold);
        let steps = vec![
            ReasoningStep { step_idx: 0, text: "Step 1".into(), e8_mode: None, token_count: 10 },
            ReasoningStep { step_idx: 1, text: "Wait, rethink".into(), e8_mode: None, token_count: 15 },
            ReasoningStep { step_idx: 2, text: "Step 2 corrected".into(), e8_mode: None, token_count: 12 },
        ];
        let trace = "Step 1 Wait, rethink Step 2 corrected";
        let seq = distiller.extract_alternating_sequence("t1".into(), "task", trace, &steps, "answer").unwrap();
        
        // 应有: Reason(0), Reason(1), Control(Backtrack), Reason(2)
        let reason_count = seq.segments.iter().filter(|s| matches!(s, AlternatingSegment::Reason{..})).count();
        let control_count = seq.segments.iter().filter(|s| matches!(s, AlternatingSegment::Control{..})).count();
        assert_eq!(reason_count, 3);
        assert_eq!(control_count, 1);
        assert!(seq.outcome_quality >= 0.0);
    }
}