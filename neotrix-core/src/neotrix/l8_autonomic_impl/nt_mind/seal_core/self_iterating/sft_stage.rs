//! SftStage — Supervised Fine-Tuning stage for SEAL pipeline
//!
//! smol-course (Hugging Face) 吸收：SFT 是 DPO 的前置阶段。
//! smol-course 强调两阶段顺序：base model → SFT（监督微调）→ DPO（偏好对齐），
//! DPO 需要一个已经指令微调的模型作为 reference policy (π_ref)。
//!
//! 本阶段在 SEAL pipeline 中位于 DpoWrapperStage 之前：
//! 1. 从 KB 经验/监督信号收集 (prompt, target_mode) 对
//! 2. 计算 SFT loss: -log p_θ(y | x)（token-level cross-entropy 的能力向量类比）
//! 3. 将能力向量推向监督目标模式（行为重塑，而非知识注入）
//!
//! Reference: Supervised Fine-Tuning (Wei et al., 2021; Ouyang et al., 2022)

use std::collections::VecDeque;
use serde::{Serialize, Deserialize};
use super::pipeline::StageResult;

/// Maximum number of supervised examples to store.
pub const SFT_BUFFER_SIZE: usize = 200;

/// SFT learning rate (smol-course 建议 5e-5 保守起步).
pub const SFT_LEARNING_RATE: f64 = 5e-5;

/// 单一监督样本：一个任务期望达到的目标模式。
/// 类比 SFT 数据集的 (prompt → completion) 对；completion 在符号域表示为 target_mode。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisedExample {
    /// 任务标识
    pub task: String,
    /// 目标 E8 模式（监督标签，类比 token 序列的期望输出）
    pub target_mode: u8,
    /// 目标能力增量方向（可选，用于能力向量级 SFT）
    pub target_capability: Option<Vec<f64>>,
    /// 数据质量权重 (0.0~1.0) — smol-course: 1000 高质量 > 10000 平庸
    pub quality: f64,
    /// 时间戳
    pub timestamp: u64,
}

impl SupervisedExample {
    /// 创建一个简单监督样本
    pub fn new(task: &str, target_mode: u8, quality: f64) -> Self {
        Self {
            task: task.to_string(),
            target_mode,
            target_capability: None,
            quality: quality.clamp(0.0, 1.0),
            timestamp: current_timestamp(),
        }
    }

    /// 设置时间戳（供 pipeline wrapper 使用）
    pub fn with_timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }
}

/// SFT 缓冲：存储监督样本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftBuffer {
    pub examples: VecDeque<SupervisedExample>,
    pub max_size: usize,
}

impl Default for SftBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl SftBuffer {
    pub fn new() -> Self {
        Self {
            examples: VecDeque::with_capacity(SFT_BUFFER_SIZE),
            max_size: SFT_BUFFER_SIZE,
        }
    }

    pub fn push(&mut self, example: SupervisedExample) {
        if self.examples.len() >= self.max_size {
            self.examples.pop_front();
        }
        self.examples.push_back(example);
    }

    pub fn len(&self) -> usize {
        self.examples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.examples.is_empty()
    }

    pub fn clear(&mut self) {
        self.examples.clear();
    }
}

/// SFT 阶段统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftReport {
    pub total_updates: u64,
    pub buffer_size: usize,
    pub last_loss: f64,
    pub avg_quality: f64,
    pub aligned_modes: Vec<u8>,
}

/// SFT Stage for the SEAL pipeline.
///
/// 监督微调：将能力向量推向目标模式（行为重塑），为 DPO 提供 π_ref 基础。
#[derive(Debug, Clone)]
pub struct SftStage {
    pub buffer: SftBuffer,
    pub learning_rate: f64,
    pub total_updates: u64,
    /// 模式→目标质量 的监督对齐表（类比 SFT 学到的行为分布）
    pub mode_quality: Vec<f64>,
}

impl Default for SftStage {
    fn default() -> Self {
        Self::new()
    }
}

impl SftStage {
    pub fn new() -> Self {
        Self {
            buffer: SftBuffer::new(),
            learning_rate: SFT_LEARNING_RATE,
            total_updates: 0,
            // 16 个 E8 模式（与 E8Policy::new 的 16 模式对齐）
            mode_quality: vec![0.0; 16],
        }
    }

    /// 计算 SFT loss（质量加权的模式分布偏差）。
    /// 类比 token-level cross-entropy：模型输出分布 vs 监督目标分布。
    pub fn compute_sft_loss(&self) -> f64 {
        if self.buffer.is_empty() {
            return 0.0;
        }
        let n = self.buffer.len() as f64;
        let total: f64 = self.buffer.examples.iter().map(|ex| {
            let current = self.mode_quality.get(ex.target_mode as usize).copied().unwrap_or(0.0);
            let target = ex.quality;
            // cross-entropy 类比：-log p_θ(y|x)，p 是当前对目标模式的对齐度
            // 数值稳定：-log(σ(margin)) = softplus(-margin)
            let margin = current - target;
            if margin > 0.0 {
                (-margin).exp().ln_1p()
            } else {
                margin.exp().ln_1p() - margin
            }
        }).sum();
        total / n
    }

    /// 平均数据质量（smol-course: 质量 > 数量）
    pub fn avg_quality(&self) -> f64 {
        if self.buffer.is_empty() {
            return 0.0;
        }
        self.buffer.examples.iter().map(|e| e.quality).sum::<f64>() / self.buffer.len() as f64
    }

    /// 处理一批监督样本：吸收监督信号，更新模式对齐表，返回调整后奖励。
    /// SFT 阶段本身不惩罚 —— 它建立行为基础；DPO 在其上做偏好区分。
    pub fn process(
        &mut self,
        examples: Vec<SupervisedExample>,
        current_reward: f64,
    ) -> (StageResult, f64) {
        let result = StageResult::new("sft_stage");

        if examples.is_empty() {
            return (result, current_reward);
        }

        // 吸收监督信号（按质量加权更新模式对齐表）
        for ex in &examples {
            self.buffer.push(ex.clone());
            let idx = (ex.target_mode as usize).min(self.mode_quality.len() - 1);
            // 指数移动平均：新监督 = lr * quality + (1-lr) * old
            self.mode_quality[idx] = self.mode_quality[idx] * (1.0 - self.learning_rate)
                + ex.quality * self.learning_rate;
        }
        self.total_updates += 1;

        // SFT loss 是训练诊断指标，不直接惩罚奖励（SFT 是前置阶段）
        let loss = self.compute_sft_loss();
        log::trace!(
            "[sft_stage] loss={:.4} avg_quality={:.4} examples={} updates={}",
            loss, self.avg_quality(), self.buffer.len(), self.total_updates
        );

        (result, current_reward)
    }

    /// 当前对某模式的对齐度（作为 DPO 的 π_ref 质量估计）
    pub fn reference_quality(&self, mode: u8) -> f64 {
        self.mode_quality.get(mode as usize).copied().unwrap_or(0.0)
    }

    /// 报告当前 SFT 状态
    pub fn report(&self) -> SftReport {
        // 收到有效监督的最小对齐度（默认 lr=5e-5 下单条样本约产生 1e-5 级移动）
        const ALIGNED_EPS: f64 = 1e-6;
        let aligned_modes: Vec<u8> = self.mode_quality.iter().enumerate()
            .filter(|(_, q)| **q > ALIGNED_EPS)
            .map(|(i, _)| i as u8)
            .collect();
        SftReport {
            total_updates: self.total_updates,
            buffer_size: self.buffer.len(),
            last_loss: self.compute_sft_loss(),
            avg_quality: self.avg_quality(),
            aligned_modes,
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example(task: &str, mode: u8, quality: f64) -> SupervisedExample {
        SupervisedExample::new(task, mode, quality)
    }

    #[test]
    fn test_buffer_push_and_len() {
        let mut buffer = SftBuffer::new();
        assert!(buffer.is_empty());
        buffer.push(example("task1", 0, 0.9));
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn test_buffer_max_size() {
        let mut buffer = SftBuffer::new();
        buffer.max_size = 3;
        for i in 0..5 {
            buffer.push(example(&format!("t{}", i), (i % 16) as u8, 0.8));
        }
        assert_eq!(buffer.len(), 3);
    }

    #[test]
    fn test_sft_loss_empty_buffer() {
        let stage = SftStage::new();
        assert_eq!(stage.compute_sft_loss(), 0.0);
    }

    #[test]
    fn test_sft_loss_with_aligned_mode() {
        let mut stage = SftStage::new();
        stage.mode_quality[2] = 0.9;
        stage.buffer.push(example("task", 2, 0.95));
        let loss = stage.compute_sft_loss();
        assert!(loss >= 0.0);
        assert!(loss < 1.0, "aligned mode should have small loss");
    }

    #[test]
    fn test_sft_loss_with_unaligned_mode() {
        let mut stage = SftStage::new();
        stage.mode_quality[2] = 0.1;
        stage.buffer.push(example("task", 2, 0.95));
        let loss = stage.compute_sft_loss();
        assert!(loss >= 0.0);
        assert!(loss > 0.1, "unaligned mode should have larger loss");
    }

    #[test]
    fn test_process_absorbs_supervision() {
        let mut stage = SftStage::new();
        let examples = vec![example("code_review", 5, 0.9)];
        let (result, reward) = stage.process(examples, 1.0);
        assert!((reward - 1.0).abs() < 1e-9, "SFT should not penalize reward");
        assert!(!result.stage_name.is_empty());
        assert_eq!(stage.total_updates, 1);
        // 模式 5 的对齐度应被推动
        assert!(stage.reference_quality(5) > 0.0);
    }

    #[test]
    fn test_process_empty_examples() {
        let mut stage = SftStage::new();
        let (_, reward) = stage.process(vec![], 0.7);
        assert!((reward - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_learning_rate_moves_mode_quality() {
        let mut stage = SftStage::new();
        stage.learning_rate = 0.5;
        stage.process(vec![example("t", 3, 1.0)], 0.0);
        // EMA: 0.0*(1-0.5) + 1.0*0.5 = 0.5
        assert!((stage.reference_quality(3) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_avg_quality() {
        let mut stage = SftStage::new();
        stage.buffer.push(example("a", 0, 0.8));
        stage.buffer.push(example("b", 1, 0.6));
        assert!((stage.avg_quality() - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_report_tracks_state() {
        let mut stage = SftStage::new();
        stage.process(vec![example("t", 7, 0.85)], 0.5);
        let report = stage.report();
        assert_eq!(report.total_updates, 1);
        assert_eq!(report.buffer_size, 1);
        assert!(report.aligned_modes.contains(&7));
    }

    #[test]
    fn test_mode_quality_bounds() {
        let mut stage = SftStage::new();
        // 模式 200 越界应被 clamp 到最后一个索引
        stage.process(vec![example("t", 200, 0.9)], 0.0);
        assert!((stage.reference_quality(15) - stage.mode_quality[15]).abs() < 1e-9);
    }

    #[test]
    fn test_example_quality_clamped() {
        let ex = SupervisedExample::new("t", 0, 2.5);
        assert!(ex.quality <= 1.0);
        let ex2 = SupervisedExample::new("t", 0, -1.0);
        assert!(ex2.quality >= 0.0);
    }
}
