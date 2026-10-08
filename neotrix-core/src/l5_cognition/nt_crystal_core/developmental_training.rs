//! 发育训练系统 — 模块能力阶段管理 + 任务复杂度分配
//!
//! 发育训练: Seed → Sprout → Growth → Mature → Transcend
//! 每个阶段对应不同任务复杂度范围，通过成功率驱动晋升

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ctm::{Chunk, ChunkType};

/// 模块能力阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DevelopmentStage {
    Seed,      // 种子期: 简单任务 [0.0, 0.3)
    Sprout,    // 萌芽期: 中等任务 [0.3, 0.5)
    Growth,    // 成长期: 复杂任务 [0.5, 0.7)
    Mature,    // 成熟期: 专家任务 [0.7, 0.85)
    Transcend, // 超越期: 自主创新 [0.85, 1.0]
}

impl DevelopmentStage {
    /// 阶段复杂度范围 (min, max)
    pub fn complexity_range(&self) -> (f64, f64) {
        match self {
            Self::Seed => (0.0, 0.3),
            Self::Sprout => (0.3, 0.5),
            Self::Growth => (0.5, 0.7),
            Self::Mature => (0.7, 0.85),
            Self::Transcend => (0.85, 1.0),
        }
    }

    /// 阶段索引 (用于晋升比较)
    pub fn index(&self) -> u8 {
        match self {
            Self::Seed => 0,
            Self::Sprout => 1,
            Self::Growth => 2,
            Self::Mature => 3,
            Self::Transcend => 4,
        }
    }

    /// 下一阶段
    pub fn next(&self) -> Self {
        match self {
            Self::Seed => Self::Sprout,
            Self::Sprout => Self::Growth,
            Self::Growth => Self::Mature,
            Self::Mature => Self::Transcend,
            Self::Transcend => Self::Transcend,
        }
    }
}

impl std::fmt::Display for DevelopmentStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Seed => write!(f, "Seed"),
            Self::Sprout => write!(f, "Sprout"),
            Self::Growth => write!(f, "Growth"),
            Self::Mature => write!(f, "Mature"),
            Self::Transcend => write!(f, "Transcend"),
        }
    }
}

/// 训练记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRecord {
    pub module_name: String,
    pub stage: DevelopmentStage,
    pub task_complexity: f64,
    pub success: bool,
    pub score_delta: f64,
    pub timestamp: u64,
}

/// 晋升配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionConfig {
    /// 晋升所需最近 N 次训练窗口
    pub window_size: usize,
    /// 晋升所需成功率
    pub success_rate_threshold: f64,
    /// 晋升所需平均 score_delta
    pub avg_delta_threshold: f64,
}

impl Default for PromotionConfig {
    fn default() -> Self {
        Self {
            window_size: 10,
            success_rate_threshold: 0.8,
            avg_delta_threshold: 0.05,
        }
    }
}

/// 发育训练配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentalTrainer {
    /// 模块当前阶段
    pub module_stages: HashMap<String, DevelopmentStage>,
    /// 训练历史 (按模块分组)
    pub training_history: Vec<TrainingRecord>,
    /// 晋升配置
    pub promotion_config: PromotionConfig,
    #[serde(skip)]
    counter: u64,
}

impl Default for DevelopmentalTrainer {
    fn default() -> Self {
        Self::new()
    }
}

impl DevelopmentalTrainer {
    pub fn new() -> Self {
        Self {
            module_stages: HashMap::new(),
            training_history: Vec::new(),
            promotion_config: PromotionConfig::default(),
            counter: 0,
        }
    }

    /// 注册新模块 (初始为 Seed 阶段)
    pub fn register_module(&mut self, name: &str) {
        self.module_stages
            .entry(name.to_string())
            .or_insert(DevelopmentStage::Seed);
    }

    /// 根据当前能力阶段分配任务复杂度
    pub fn assign_complexity(&self, module_name: &str) -> f64 {
        let stage = self
            .module_stages
            .get(module_name)
            .unwrap_or(&DevelopmentStage::Seed);
        let (min, max) = stage.complexity_range();
        // 在范围内取中间偏高值，给模块一定挑战
        min + (max - min) * 0.7
    }

    /// 记录训练结果
    pub fn record_training(
        &mut self,
        module: &str,
        complexity: f64,
        success: bool,
        delta: f64,
    ) {
        self.counter += 1;
        let stage = self
            .module_stages
            .get(module)
            .cloned()
            .unwrap_or(DevelopmentStage::Seed);

        let record = TrainingRecord {
            module_name: module.to_string(),
            stage,
            task_complexity: complexity,
            success,
            score_delta: delta,
            timestamp: self.counter,
        };
        self.training_history.push(record);
    }

    /// 获取模块最近 N 次训练记录
    fn recent_records(&self, module: &str, n: usize) -> Vec<&TrainingRecord> {
        self.training_history
            .iter()
            .filter(|r| r.module_name == module)
            .rev()
            .take(n)
            .collect()
    }

    /// 检查是否应该晋升
    pub fn should_promote(&self, module: &str) -> bool {
        let recent = self.recent_records(module, self.promotion_config.window_size);
        if recent.len() < self.promotion_config.window_size {
            return false;
        }

        let success_count = recent.iter().filter(|r| r.success).count();
        let success_rate = success_count as f64 / recent.len() as f64;
        let avg_delta: f64 = recent.iter().map(|r| r.score_delta).sum::<f64>() / recent.len() as f64;

        success_rate >= self.promotion_config.success_rate_threshold
            && avg_delta >= self.promotion_config.avg_delta_threshold
    }

    /// 晋升模块到下一阶段
    pub fn promote(&mut self, module: &str) -> Option<DevelopmentStage> {
        let current = self
            .module_stages
            .get(module)?
            .clone();
        let next = current.next();
        if next.index() > current.index() {
            self.module_stages.insert(module.to_string(), next.clone());
            Some(next)
        } else {
            None // 已在最高阶段
        }
    }

    /// 获取所有模块的发育状态
    pub fn status(&self) -> HashMap<String, DevelopmentStage> {
        self.module_stages.clone()
    }

    /// 获取模块训练统计
    pub fn module_stats(&self, module: &str) -> TrainingStats {
        let records = self.recent_records(module, usize::MAX);
        let total = records.len();
        let successes = records.iter().filter(|r| r.success).count();
        let avg_complexity = if total > 0 {
            records.iter().map(|r| r.task_complexity).sum::<f64>() / total as f64
        } else {
            0.0
        };
        let avg_delta = if total > 0 {
            records.iter().map(|r| r.score_delta).sum::<f64>() / total as f64
        } else {
            0.0
        };

        TrainingStats {
            total,
            successes,
            success_rate: if total > 0 {
                successes as f64 / total as f64
            } else {
                0.0
            },
            avg_complexity,
            avg_delta,
        }
    }

    /// 为模块生成适合其阶段的训练 Chunk
    pub fn generate_training_task(&self, module_name: &str) -> Chunk {
        let complexity = self.assign_complexity(module_name);
        let stage = self
            .module_stages
            .get(module_name)
            .cloned()
            .unwrap_or(DevelopmentStage::Seed);

        Chunk {
            content: format!(
                "[training:{:?}] complexity={:.2}",
                stage, complexity
            ),
            score: complexity,
            source_module: module_name.to_string(),
            chunk_type: ChunkType::Memory,
            metadata: {
                let mut m = HashMap::new();
                m.insert("task_type".into(), "developmental_training".into());
                m.insert("complexity".into(), format!("{:.2}", complexity));
                m.insert("stage".into(), stage.to_string());
                m
            },
        }
    }
}

/// 模块训练统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingStats {
    pub total: usize,
    pub successes: usize,
    pub success_rate: f64,
    pub avg_complexity: f64,
    pub avg_delta: f64,
}
