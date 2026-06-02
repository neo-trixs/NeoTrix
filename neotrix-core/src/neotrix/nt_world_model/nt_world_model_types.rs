use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::neotrix::signal::ops::cosine_similarity;

pub type Vector = Vec<f64>;
pub type Matrix = Vec<Vec<f64>>;

pub const LATENT_DIM: usize = 32;

pub use crate::core::nt_core_knowledge::TaskType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentState {
    pub vector: Vector,
    pub timestamp: i64,
}

impl Default for LatentState {
    fn default() -> Self { Self::new() }
}

impl LatentState {
    /// Creates a zero-initialized latent state with current timestamp.
    pub fn new() -> Self {
        Self {
            vector: vec![0.0; LATENT_DIM],
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Encodes a Context directly into a latent state.
    pub fn from_context(ctx: &Context) -> Self {
        let mut state = Self::new();
        let features = ctx.to_features();
        for (i, &val) in features.iter().take(LATENT_DIM).enumerate() {
            state.vector[i] = val;
        }
        state
    }

    /// 计算与另一个状态的相似度
    pub fn similarity(&self, other: &LatentState) -> f64 {
        cosine_similarity(&self.vector, &other.vector)
    }
}

/// Task context: type, complexity, domain, and input features used for expert prediction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub task_type: TaskType,
    pub complexity: f64,
    pub domain: Domain,
    pub input_features: Vector,
    pub metadata: HashMap<String, String>,
}

impl Context {
    pub fn new(task_type: TaskType) -> Self {
        Self {
            task_type,
            complexity: 0.5,
            domain: Domain::General,
            input_features: vec![0.0; 64],
            metadata: HashMap::new(),
        }
    }

    /// 转换为特征向量
    pub fn to_features(&self) -> Vector {
        let mut features = vec![0.0; 64];

        let task_idx = self.task_type as usize;
        if task_idx < 8 {
            features[task_idx] = 1.0;
        }

        features[8] = self.complexity;

        let domain_idx = self.domain as usize;
        if domain_idx + 9 < 64 {
            features[9 + domain_idx] = 1.0;
        }

        for (i, &val) in self.input_features.iter().take(50).enumerate() {
            if i + 14 < 64 {
                features[14 + i] = val;
            }
        }

        features
    }

    /// 从任务描述推断上下文
    pub fn from_task_description(desc: &str) -> Self {
        let desc_lower = desc.to_lowercase();

        let task_type = if desc_lower.contains("design") || desc_lower.contains("设计") {
            TaskType::Design
        } else if desc_lower.contains("ui") || desc_lower.contains("component")
            || desc_lower.contains("react") || desc_lower.contains("tailwind")
            || desc_lower.contains("heroui") || desc_lower.contains("界面") {
            TaskType::UIDesign
        } else if desc_lower.contains("code") || desc_lower.contains("代码") {
            TaskType::CodeAnalysis
        } else if desc_lower.contains("generate") || desc_lower.contains("生成") {
            TaskType::CodeGeneration
        } else if desc_lower.contains("review") || desc_lower.contains("审查") {
            TaskType::CodeReview
        } else if desc_lower.contains("nt_shield") || desc_lower.contains("安全") {
            TaskType::Security
        } else if desc_lower.contains("plan") || desc_lower.contains("规划") {
            TaskType::Planning
        } else {
            TaskType::General
        };

        let complexity = if desc_lower.contains("complex") || desc_lower.contains("复杂") {
            0.8
        } else if desc_lower.contains("simple") || desc_lower.contains("简单") {
            0.2
        } else {
            0.5
        };

        let domain = if desc_lower.contains("web") || desc_lower.contains("前端") {
            Domain::WebDev
        } else if desc_lower.contains("mobile") || desc_lower.contains("移动") {
            Domain::Mobile
        } else if desc_lower.contains("ai") || desc_lower.contains("机器学习") {
            Domain::AI
        } else {
            Domain::General
        };

        let mut ctx = Context::new(task_type);
        ctx.complexity = complexity;
        ctx.domain = domain;
        ctx
    }
}

/// Application domain for task context classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    General = 0,
    WebDev = 1,
    Mobile = 2,
    AI = 3,
    DataScience = 4,
    DevOps = 5,
}
