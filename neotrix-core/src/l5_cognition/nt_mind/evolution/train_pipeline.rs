//! train_pipeline — 从 `evolution_loop.rs` 拆分 (卫星件, 零生产引用, 行为零变更).

use serde::{Deserialize, Serialize};

/// 训练阶段 — Pretrain→Sft→Rm→{Ppo,Dpo,Grpo 任一}→Done
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum _TrainStage {
    Pretrain,
    Sft,
    Rm,
    Ppo,
    Dpo,
    Grpo,
    Done,
}

impl _TrainStage {
    pub fn label(self) -> &'static str {
        match self {
            _TrainStage::Pretrain => "pretrain",
            _TrainStage::Sft => "sft",
            _TrainStage::Rm => "rm",
            _TrainStage::Ppo => "ppo",
            _TrainStage::Dpo => "dpo",
            _TrainStage::Grpo => "grpo",
            _TrainStage::Done => "done",
        }
    }

    /// 阶段推进: Pretrain→Sft→Rm→{Ppo,Dpo,Grpo 任一}→Done; Done 终止
    pub fn next(self) -> Option<_TrainStage> {
        match self {
            _TrainStage::Pretrain => Some(_TrainStage::Sft),
            _TrainStage::Sft => Some(_TrainStage::Rm),
            _TrainStage::Rm => Some(_TrainStage::Ppo),
            _TrainStage::Ppo => Some(_TrainStage::Done),
            _TrainStage::Dpo => Some(_TrainStage::Done),
            _TrainStage::Grpo => Some(_TrainStage::Done),
            _TrainStage::Done => None,
        }
    }
}

/// 训练超参 — 端到端管线的全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _TrainConfig {
    /// 模型参数量 (scale), lr 缩放基准 1e9
    pub model_scale: f64,
    /// 学习率
    pub lr: f64,
    /// warmup 步数
    pub warmup_steps: usize,
    /// 每步 batch 大小
    pub batch_size: usize,
    /// 最大 epoch 数
    pub max_epochs: usize,
}

impl Default for _TrainConfig {
    fn default() -> Self {
        Self {
            model_scale: 1e9,
            lr: 3e-4,
            warmup_steps: 500,
            batch_size: 32,
            max_epochs: 3,
        }
    }
}

/// 训练管线状态机 — 阶段推进 + 超参 + 历史追踪
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _TrainPipeline {
    pub current: _TrainStage,
    pub config: _TrainConfig,
    pub epochs_run: usize,
    /// 每阶段完成时记录 (阶段, 当时 epoch 数)
    pub history: Vec<(_TrainStage, usize)>,
}

impl Default for _TrainPipeline {
    fn default() -> Self {
        Self::new(_TrainConfig::default())
    }
}

impl _TrainPipeline {
    pub fn new(config: _TrainConfig) -> Self {
        Self {
            current: _TrainStage::Pretrain,
            config,
            epochs_run: 0,
            history: Vec::new(),
        }
    }

    /// 推进到下一阶段: epochs_run += 1, 记录 (current, epochs_run), current = next()?,
    /// 返回新阶段。Done 之后返回 None (R-P3: ? 传播终止)。
    pub fn advance(&mut self) -> Option<_TrainStage> {
        self.epochs_run += 1;
        self.history.push((self.current, self.epochs_run));
        let next = self.current.next()?;
        self.current = next;
        Some(next)
    }

    /// 训练策略选择 — 各阶段对应方法论 (train-llm-from-scratch 知识)
    pub(crate) fn _recommend_strategy(&self, stage: _TrainStage) -> &'static str {
        match stage {
            _TrainStage::Sft => "supervised fine-tuning: next-token",
            _TrainStage::Rm => "reward model: pairwise ranking",
            _TrainStage::Ppo => "PPO: on-policy RLHF",
            _TrainStage::Dpo => "DPO: off-policy preference",
            _TrainStage::Grpo => "GRPO: group relative policy optimization",
            _ => "pretraining: next-token on corpus",
        }
    }

    /// 依据模型规模缩放 lr (经验法则: lr ∝ scale^-0.15), 更新 config.lr 并返回。
    /// 更大模型 → 更小 lr。
    pub(crate) fn _scale_lr(&mut self, scale: f64) -> f64 {
        let scaled = self.config.lr * (scale / 1e9).powf(-0.15);
        self.config.lr = scaled;
        scaled
    }

    pub fn is_complete(&self) -> bool {
        self.current == _TrainStage::Done
    }

    /// 完成阶段占比 (6 阶段含 Done)。R-P6: max(0.0).min(1.0) 钳制。
    pub(crate) fn _stage_progress(&self) -> f64 {
        let idx = match self.current {
            _TrainStage::Pretrain => 0,
            _TrainStage::Sft => 1,
            _TrainStage::Rm => 2,
            _TrainStage::Ppo => 3,
            _TrainStage::Dpo => 4,
            _TrainStage::Grpo => 5,
            _TrainStage::Done => 6,
        };
        (idx as f64 / 6.0).max(0.0).min(1.0)
    }
}

impl crate::l0_substrate::nt_core_self_test::SelfTest for _TrainPipeline {
    fn name(&self) -> &str {
        "nt_mind_train_pipeline"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut pipe = _TrainPipeline::new(_TrainConfig::default());
        if pipe.is_complete() {
            return Err(vec!["pipeline must start incomplete".into()]);
        }
        pipe.advance()
            .ok_or_else(|| vec!["advance from Pretrain must yield a stage".into()])?;
        let base = pipe.config.lr;
        let scaled = pipe._scale_lr(1e10);
        if !scaled.is_finite() || scaled >= base {
            return Err(vec!["_scale_lr must lower lr for larger models".into()]);
        }
        let p = pipe._stage_progress();
        if !(0.0..=1.0).contains(&p) {
            return Err(vec![format!("_stage_progress out of bounds: {p}")]);
        }
        Ok(())
    }
}
