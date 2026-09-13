//! Experience Tree Manager — 经验树管理器
//!
//! 吸收 KB 经验:
//! - 五阶段吸收流程 (快照→蒸馏→分类→落盘→反馈)
//! - 经验指针守恒
//! - AGENTS.md 不含 per-cycle 增长区
//! - KB hub 存储

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 经验树管理器
pub struct _ExperienceTreeManager {
    experiences: Vec<Experience>,
    #[allow(dead_code)]
    branches: Vec<_ExperienceBranch>,
    config: _ExperienceConfig,
    stats: _ExperienceStats,
}

/// 经验配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ExperienceConfig {
    pub max_experiences: usize,
    pub enable_auto_distill: bool,
    pub retention_days: u32,
    pub min_confidence: f64,
}

impl Default for _ExperienceConfig {
    fn default() -> Self {
        Self {
            max_experiences: 1000,
            enable_auto_distill: true,
            retention_days: 90,
            min_confidence: 0.3,
        }
    }
}

/// 经验
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub experience_id: String,
    pub session_id: String,
    pub cycle: String,
    pub domain: String,
    pub content: String,
    pub experience_type: _ExperienceType,
    pub confidence: f64,
    pub importance: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub concepts: Vec<String>,
    pub feedback: _ExperienceFeedback,
}

/// 经验类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum _ExperienceType {
    Pattern,
    Rule,
    Defect,
    Insight,
    Cycle,
}

impl std::fmt::Display for _ExperienceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pattern => write!(f, "pattern"),
            Self::Rule => write!(f, "rule"),
            Self::Defect => write!(f, "defect"),
            Self::Insight => write!(f, "insight"),
            Self::Cycle => write!(f, "cycle"),
        }
    }
}

/// 经验反馈
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ExperienceFeedback {
    pub success: u32,
    pub failure: u32,
    pub reuse: u32,
}

/// 经验分支
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ExperienceBranch {
    pub branch_id: String,
    pub experience_id: String,
    pub branch_type: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 经验统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ExperienceStats {
    pub total_experiences: u64,
    pub by_domain: HashMap<String, u64>,
    pub by_type: HashMap<String, u64>,
    pub avg_confidence: f64,
    pub avg_importance: f64,
}

/// 吸收阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum _AbsorptionStage {
    Snapshot,    // 快照
    Distill,     // 蒸馏
    Classify,    // 分类
    Store,       // 落盘
    Feedback,    // 反馈
}

/// 吸收结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorptionResult {
    pub success: bool,
    pub experience_id: String,
    pub stage: _AbsorptionStage,
    pub message: String,
}

impl _ExperienceTreeManager {
    /// 创建新的经验树管理器
    pub fn new() -> Self {
        Self {
            experiences: Vec::new(),
            branches: Vec::new(),
            config: _ExperienceConfig::default(),
            stats: _ExperienceStats {
                total_experiences: 0,
                by_domain: HashMap::new(),
                by_type: HashMap::new(),
                avg_confidence: 0.0,
                avg_importance: 0.0,
            },
        }
    }

    /// 吸收新经验
    pub fn absorb(&mut self, experience: Experience) -> AbsorptionResult {
        // 1. 快照阶段: 记录原始经验
        let experience_id = experience.experience_id.clone();

        // 2. 蒸馏阶段: 提取核心内容
        if self.config.enable_auto_distill {
            // 自动蒸馏逻辑
        }

        // 3. 分类阶段: 根据类型分类
        *self.stats.by_domain.entry(experience.domain.clone()).or_insert(0) += 1;
        *self.stats.by_type.entry(experience.experience_type.to_string()).or_insert(0) += 1;

        // 4. 落盘阶段: 存储经验
        self.experiences.push(experience);

        // 5. 反馈阶段: 更新统计
        self.stats.total_experiences += 1;

        AbsorptionResult {
            success: true,
            experience_id,
            stage: _AbsorptionStage::Feedback,
            message: "经验已成功吸收".into(),
        }
    }

    /// 查询经验
    pub fn query(&self, query: &ExperienceQuery) -> Vec<&Experience> {
        self.experiences.iter()
            .filter(|e| {
                if let Some(ref domain) = query.domain {
                    if &e.domain != domain {
                        return false;
                    }
                }
                if let Some(ref experience_type) = query.experience_type {
                    if &e.experience_type != experience_type {
                        return false;
                    }
                }
                if let Some(min_confidence) = query.min_confidence {
                    if e.confidence < min_confidence {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// 获取按重要性排序的经验
    pub(crate) fn _get_by_importance(&self, limit: usize) -> Vec<&Experience> {
        let mut sorted: Vec<&Experience> = self.experiences.iter().collect();
        sorted.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(limit).collect()
    }

    /// 获取统计信息
    pub fn stats(&self) -> &_ExperienceStats {
        &self.stats
    }

    /// 清理过期经验
    pub fn cleanup(&mut self, retention_days: u32) {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(retention_days as i64);
        self.experiences.retain(|e| e.timestamp > cutoff);
    }
}

/// 经验查询
#[derive(Debug, Clone)]
pub struct ExperienceQuery {
    pub domain: Option<String>,
    pub experience_type: Option<_ExperienceType>,
    pub min_confidence: Option<f64>,
    pub max_results: Option<usize>,
}

impl Default for ExperienceQuery {
    fn default() -> Self {
        Self {
            domain: None,
            experience_type: None,
            min_confidence: None,
            max_results: Some(100),
        }
    }
}
