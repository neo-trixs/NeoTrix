//! Experience Tree Manager — 经验树管理器
//!
//! 吸收 KB 经验:
//! - 五阶段吸收流程 (快照→蒸馏→分类→落盘→反馈)
//! - 经验指针守恒
//! - AGENTS.md 不含 per-cycle 增长区
//! - KB hub 存储

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 经验树管理器
pub struct _ExperienceTreeManager {
    experiences: Vec<Experience>,
    config: _ExperienceConfig,
    stats: _ExperienceStats,
    /// P0-2: 失败胶囊流 (负记忆). 只追加; 与成功路径经验向量解耦存放.
    failure_capsules: Vec<NtFailureCapsule>,
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
    Snapshot, // 快照
    Distill,  // 蒸馏
    Classify, // 分类
    Store,    // 落盘
    Feedback, // 反馈
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
            config: _ExperienceConfig::default(),
            stats: _ExperienceStats {
                total_experiences: 0,
                by_domain: HashMap::new(),
                by_type: HashMap::new(),
                avg_confidence: 0.0,
                avg_importance: 0.0,
            },
            failure_capsules: Vec::new(),
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
        *self
            .stats
            .by_domain
            .entry(experience.domain.clone())
            .or_insert(0) += 1;
        *self
            .stats
            .by_type
            .entry(experience.experience_type.to_string())
            .or_insert(0) += 1;

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
        self.experiences
            .iter()
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
        sorted.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
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

// ============================================================================
// P0-2: 失败胶囊 → 负记忆流 (Evo-Harness: 只在失败处反思, A)
// ============================================================================
//
// 约定:
// - 成功经验走既有 absorb(); 失败/负反馈走 nt_absorb_failure_capsule();
// - Critic verdict 四值: helped / hurt / neutral / inapplicable;
// - 同一 pattern_label 下 hurt 胶囊聚类 ≥ 阈值 → promotion_triggered,
//   调用方据此合成新技能候选或追加 skill exclusions;
// - hurt_ratio 持续走高是漂移先兆 (early-warning, 与技能账本 engagement 互补).

/// Critic 归因 verdict (与技能贡献账本同词表, 跨层可对账).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum NtFailureVerdict {
    Helped,
    Hurt,
    Neutral,
    #[default]
    Inapplicable,
}

/// 失败胶囊: 一次失败执行的归因快照 (合成 substrate + 诊断信号双用).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtFailureCapsule {
    #[serde(default)]
    pub capsule_id: String,
    #[serde(default)]
    pub task_id: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub skill_id: Option<String>,
    #[serde(default)]
    pub verdict: NtFailureVerdict,
    /// 规范模式标签 (如 "naked-date-logs-query"); 聚类键.
    #[serde(default)]
    pub pattern_label: String,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl NtFailureCapsule {
    /// 构造失败胶囊; confidence 按范围 clamp.
    pub fn nt_new(
        capsule_id: &str,
        task_id: &str,
        session_id: &str,
        skill_id: Option<&str>,
        verdict: NtFailureVerdict,
        pattern_label: &str,
        confidence: f64,
        detail: &str,
    ) -> Self {
        Self {
            capsule_id: capsule_id.to_string(),
            task_id: task_id.to_string(),
            session_id: session_id.to_string(),
            skill_id: skill_id.map(|s| s.to_string()),
            verdict,
            pattern_label: pattern_label.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            detail: detail.to_string(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// 同一 hurt 模式聚类 ≥ 此数即触发技能合成/排除 (合成 substrate).
pub const NT_CAPSULE_CLUSTER_THRESHOLD: usize = 3;

/// 胶囊吸收结果.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtCapsuleOutcome {
    pub accepted: bool,
    pub cluster_size: usize,
    pub promotion_triggered: bool,
    pub hurt_ratio: f64,
}

impl _ExperienceTreeManager {
    /// 吸收一枚失败胶囊 (append-only).
    ///
    /// 空 pattern_label 视为无效输入 → accepted=false, 其余原样返回诊断值.
    pub fn nt_absorb_failure_capsule(&mut self, capsule: NtFailureCapsule) -> NtCapsuleOutcome {
        if capsule.pattern_label.is_empty() {
            return NtCapsuleOutcome {
                accepted: false,
                cluster_size: 0,
                promotion_triggered: false,
                hurt_ratio: self.nt_hurt_ratio(),
            };
        }
        let label = capsule.pattern_label.clone();
        let is_hurt = capsule.verdict == NtFailureVerdict::Hurt;
        self.failure_capsules.push(capsule);
        let cluster_size = self
            .failure_capsules
            .iter()
            .filter(|c| c.verdict == NtFailureVerdict::Hurt && c.pattern_label == label)
            .count();
        NtCapsuleOutcome {
            accepted: true,
            cluster_size,
            promotion_triggered: is_hurt && cluster_size >= NT_CAPSULE_CLUSTER_THRESHOLD,
            hurt_ratio: self.nt_hurt_ratio(),
        }
    }

    /// hurt 胶囊占比 (drift early-warning; 空池返回 0.0).
    pub fn nt_hurt_ratio(&self) -> f64 {
        if self.failure_capsules.is_empty() {
            return 0.0;
        }
        let hurt = self
            .failure_capsules
            .iter()
            .filter(|c| c.verdict == NtFailureVerdict::Hurt)
            .count();
        hurt as f64 / self.failure_capsules.len() as f64
    }

    /// 按 pattern_label 聚合的 hurt 簇大小 (仅 hurt verdict 参与).
    pub fn nt_hurt_cluster_sizes(&self) -> HashMap<String, usize> {
        let mut sizes: HashMap<String, usize> = HashMap::new();
        for c in &self.failure_capsules {
            if c.verdict == NtFailureVerdict::Hurt {
                *sizes.entry(c.pattern_label.clone()).or_insert(0) += 1;
            }
        }
        sizes
    }

    /// 胶囊池深度 (诊断用).
    pub fn nt_capsule_count(&self) -> usize {
        self.failure_capsules.len()
    }
}

#[cfg(test)]
mod nt_failure_capsule_tests {
    use super::*;

    fn nt_hurt(id: &str, label: &str) -> NtFailureCapsule {
        NtFailureCapsule::nt_new(
            id,
            "task-1",
            "sess-1",
            Some("skill-x"),
            NtFailureVerdict::Hurt,
            label,
            0.8,
            "detail",
        )
    }

    #[test]
    fn test_nt_capsule_cluster_triggers_at_three() {
        let mut mgr = _ExperienceTreeManager::new();
        let o1 = mgr.nt_absorb_failure_capsule(nt_hurt("c1", "naked-date-query"));
        assert!(o1.accepted);
        assert_eq!(o1.cluster_size, 1);
        assert!(!o1.promotion_triggered);
        let o2 = mgr.nt_absorb_failure_capsule(nt_hurt("c2", "naked-date-query"));
        assert_eq!(o2.cluster_size, 2);
        assert!(!o2.promotion_triggered);
        let o3 = mgr.nt_absorb_failure_capsule(nt_hurt("c3", "naked-date-query"));
        assert_eq!(o3.cluster_size, 3);
        assert!(o3.promotion_triggered);
        // 不同 label 不混簇
        let o4 = mgr.nt_absorb_failure_capsule(nt_hurt("c4", "other-pattern"));
        assert_eq!(o4.cluster_size, 1);
        assert!(!o4.promotion_triggered);
        assert_eq!(mgr.nt_capsule_count(), 4);
    }

    #[test]
    fn test_nt_hurt_ratio_early_warning() {
        let mut mgr = _ExperienceTreeManager::new();
        assert!((mgr.nt_hurt_ratio() - 0.0).abs() < 1e-10);
        mgr.nt_absorb_failure_capsule(nt_hurt("c1", "p1"));
        mgr.nt_absorb_failure_capsule(NtFailureCapsule::nt_new(
            "c2",
            "task-2",
            "sess-1",
            None,
            NtFailureVerdict::Neutral,
            "p2",
            0.5,
            "ok-ish",
        ));
        assert!((mgr.nt_hurt_ratio() - 0.5).abs() < 1e-10);
        let sizes = mgr.nt_hurt_cluster_sizes();
        assert_eq!(sizes.get("p1"), Some(&1));
        assert!(sizes.get("p2").is_none());
    }

    #[test]
    fn test_nt_empty_pattern_rejected() {
        let mut mgr = _ExperienceTreeManager::new();
        let bad = NtFailureCapsule::nt_new(
            "c0",
            "task-0",
            "sess-0",
            None,
            NtFailureVerdict::Hurt,
            "",
            0.9,
            "no label",
        );
        let out = mgr.nt_absorb_failure_capsule(bad);
        assert!(!out.accepted);
        assert!(!out.promotion_triggered);
        assert_eq!(mgr.nt_capsule_count(), 0);
    }
}
