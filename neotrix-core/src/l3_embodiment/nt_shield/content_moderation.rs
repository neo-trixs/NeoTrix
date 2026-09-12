//! _ContentModeration — 内容审核
//!
//! 提示词过滤 + 输出分析 + 人工审核路由。
//! 支持 NSFW 检测、品牌安全、版权匹配。

use std::collections::HashMap;
use std::time::Instant;

/// 审核结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum _ModerationResult {
    /// 通过
    Passed,
    /// 被拒绝
    Rejected,
    /// 需要人工审核
    NeedsReview,
    /// 被标记
    Flagged,
}

/// 内容类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContentType {
    /// 提示词
    Prompt,
    /// 生成的图像
    GeneratedImage,
    /// 生成的视频
    GeneratedVideo,
    /// 音频
    Audio,
    /// 文本
    Text,
}

/// 风险类别
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RiskCategory {
    /// NSFW 内容
    Nsfw,
    /// 暴力内容
    Violence,
    /// 仇恨言论
    Hate,
    /// 骚扰
    Harassment,
    /// 自我伤害
    SelfHarm,
    /// 版权侵权
    Copyright,
    /// 品牌安全
    BrandSafety,
    /// 虚假信息
    Misinformation,
    /// 隐私侵犯
    PrivacyViolation,
}

/// 审核规则
#[derive(Debug, Clone)]
pub(crate) struct _ModerationRule {
    /// 规则 ID
    pub id: String,
    /// 风险类别
    pub category: RiskCategory,
    /// 阈值 (0-1)
    pub threshold: f64,
    /// 动作
    pub action: _ModerationAction,
    /// 优先级
    pub priority: u32,
}

/// 审核动作
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum _ModerationAction {
    /// 自动通过
    AutoApprove,
    /// 自动拒绝
    AutoReject,
    /// 标记并人工审核
    FlagForReview,
    /// 降级处理
    Degrade,
}

/// 审核结果详情
#[derive(Debug, Clone)]
pub(crate) struct _ModerationOutcome {
    /// 内容 ID
    pub content_id: String,
    /// 内容类型
    pub content_type: ContentType,
    /// 最终结果
    pub result: _ModerationResult,
    /// 各类别分数
    pub category_scores: HashMap<RiskCategory, f64>,
    /// 触发的规则
    pub triggered_rules: Vec<String>,
    /// 人工审核原因
    pub review_reason: Option<String>,
    /// 审核时间
    pub moderated_at: Instant,
}

/// 人工审核任务
#[derive(Debug, Clone)]
pub(crate) struct _ReviewTask {
    /// 任务 ID
    pub id: String,
    /// 内容 ID
    pub content_id: String,
    /// 内容类型
    pub content_type: ContentType,
    /// 审核原因
    pub reason: String,
    /// 提交时间
    pub submitted_at: Instant,
    /// 状态
    pub status: ReviewStatus,
    /// 审核员
    pub reviewer: Option<String>,
    /// 审核结果
    pub review_result: Option<_ModerationResult>,
}

/// 审核状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewStatus {
    Pending,
    InProgress,
    Completed,
    Escalated,
}

/// 内容审核器
pub(crate) struct _ContentModeration {
    /// 审核规则
    rules: Vec<_ModerationRule>,
    /// 人工审核队列
    review_queue: Vec<_ReviewTask>,
    /// 统计信息
    stats: _ModerationStats,
}

impl _ContentModeration {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            review_queue: Vec::new(),
            stats: _ModerationStats::default(),
        }
    }

    /// 添加审核规则
    pub fn add_rule(&mut self, rule: _ModerationRule) {
        self.rules.push(rule);
    }

    /// 审核提示词
    pub(crate) fn _moderate_prompt(&mut self, content_id: &str, prompt: &str) -> _ModerationOutcome {
        let mut category_scores = HashMap::new();
        let mut triggered_rules = Vec::new();

        // 简化的提示词审核逻辑
        for rule in &self.rules {
            let score = self.evaluate_prompt_risk(prompt, &rule.category);
            category_scores.insert(rule.category.clone(), score);

            if score >= rule.threshold {
                triggered_rules.push(rule.id.clone());
            }
        }

        let result = self.determine_result(&category_scores, &triggered_rules);
        self.update_stats(&result);

        _ModerationOutcome {
            content_id: content_id.to_string(),
            content_type: ContentType::Prompt,
            result,
            category_scores,
            triggered_rules,
            review_reason: None,
            moderated_at: Instant::now(),
        }
    }

    /// 审核生成内容
    pub(crate) fn _moderate_output(&mut self, content_id: &str, content_type: ContentType, metadata: &HashMap<String, String>) -> _ModerationOutcome {
        let mut category_scores = HashMap::new();
        let mut triggered_rules = Vec::new();

        // 简化的内容审核逻辑
        for rule in &self.rules {
            let score = self.evaluate_output_risk(content_type.clone(), metadata, &rule.category);
            category_scores.insert(rule.category.clone(), score);

            if score >= rule.threshold {
                triggered_rules.push(rule.id.clone());
            }
        }

        let result = self.determine_result(&category_scores, &triggered_rules);
        self.update_stats(&result);

        _ModerationOutcome {
            content_id: content_id.to_string(),
            content_type,
            result,
            category_scores,
            triggered_rules,
            review_reason: None,
            moderated_at: Instant::now(),
        }
    }

    /// 评估提示词风险
    fn evaluate_prompt_risk(&self, prompt: &str, category: &RiskCategory) -> f64 {
        // 简化的风险评估
        let prompt_lower = prompt.to_lowercase();
        match category {
            RiskCategory::Nsfw => {
                if prompt_lower.contains("nude") || prompt_lower.contains("explicit") {
                    0.9
                } else {
                    0.1
                }
            }
            RiskCategory::Violence => {
                if prompt_lower.contains("violence") || prompt_lower.contains("blood") {
                    0.8
                } else {
                    0.1
                }
            }
            _ => 0.1,
        }
    }

    /// 评估输出风险
    fn evaluate_output_risk(&self, content_type: ContentType, metadata: &HashMap<String, String>, category: &RiskCategory) -> f64 {
        let mut risk: f64 = 0.0;

        // 基于内容类型的基础风险
        risk += match content_type {
            ContentType::Prompt => 0.05,
            ContentType::GeneratedImage => 0.2,
            ContentType::GeneratedVideo => 0.25,
            ContentType::Audio => 0.15,
            ContentType::Text => 0.05,
        };

        // 基于风险类别的调整
        risk += match category {
            RiskCategory::Nsfw => 0.3,
            RiskCategory::Violence => 0.25,
            RiskCategory::Hate => 0.2,
            RiskCategory::Harassment => 0.2,
            RiskCategory::SelfHarm => 0.3,
            RiskCategory::Copyright => 0.15,
            RiskCategory::BrandSafety => 0.1,
            RiskCategory::Misinformation => 0.15,
            RiskCategory::PrivacyViolation => 0.25,
        };

        // 检查元数据中的风险信号
        if let Some(contains_pii) = metadata.get("contains_pii") {
            if contains_pii == "true" {
                risk += 0.2; // 包含个人身份信息
            }
        }
        if let Some(contains_secret) = metadata.get("contains_secret") {
            if contains_secret == "true" {
                risk += 0.3; // 包含密钥/密码
            }
        }

        risk.clamp(0.0, 1.0)
    }

    /// 确定审核结果
    fn determine_result(&self, _scores: &HashMap<RiskCategory, f64>, triggered_rules: &[String]) -> _ModerationResult {
        if triggered_rules.is_empty() {
            _ModerationResult::Passed
        } else {
            // 检查是否有自动拒绝规则
            for rule in &self.rules {
                if triggered_rules.contains(&rule.id) && rule.action == _ModerationAction::AutoReject {
                    return _ModerationResult::Rejected;
                }
            }

            // 检查是否需要人工审核
            for rule in &self.rules {
                if triggered_rules.contains(&rule.id) && rule.action == _ModerationAction::FlagForReview {
                    return _ModerationResult::NeedsReview;
                }
            }

            _ModerationResult::Flagged
        }
    }

    /// 更新统计
    fn update_stats(&mut self, result: &_ModerationResult) {
        self.stats.total_moderated += 1;
        match result {
            _ModerationResult::Passed => self.stats.total_passed += 1,
            _ModerationResult::Rejected => self.stats.total_rejected += 1,
            _ModerationResult::NeedsReview => self.stats.total_review += 1,
            _ModerationResult::Flagged => self.stats.total_flagged += 1,
        }
    }

    /// 添加人工审核任务
    pub(crate) fn _add_review_task(&mut self, task: _ReviewTask) {
        self.review_queue.push(task);
    }

    /// 获取统计信息
    pub fn stats(&self) -> _ModerationStats {
        self.stats.clone()
    }
}

impl Default for _ContentModeration {
    fn default() -> Self {
        let mut moderation = Self::new();
        moderation.add_rule(_ModerationRule {
            id: "nsfw-auto-reject".to_string(),
            category: RiskCategory::Nsfw,
            threshold: 0.8,
            action: _ModerationAction::AutoReject,
            priority: 1,
        });
        moderation.add_rule(_ModerationRule {
            id: "violence-flag".to_string(),
            category: RiskCategory::Violence,
            threshold: 0.6,
            action: _ModerationAction::FlagForReview,
            priority: 2,
        });
        moderation
    }
}

/// 审核统计
#[derive(Debug, Clone, Default)]
pub(crate) struct _ModerationStats {
    pub total_moderated: u32,
    pub total_passed: u32,
    pub total_rejected: u32,
    pub total_review: u32,
    pub total_flagged: u32,
}

impl _ModerationStats {
    pub fn pass_rate(&self) -> f64 {
        if self.total_moderated == 0 {
            return 0.0;
        }
        self.total_passed as f64 / self.total_moderated as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moderate_safe_prompt() {
        let mut moderation = _ContentModeration::default();
        let outcome = moderation._moderate_prompt("test-1", "a beautiful landscape");
        assert_eq!(outcome.result, _ModerationResult::Passed);
    }

    #[test]
    fn test_moderate_unsafe_prompt() {
        let mut moderation = _ContentModeration::default();
        let outcome = moderation._moderate_prompt("test-2", "nude explicit content");
        assert_eq!(outcome.result, _ModerationResult::Rejected);
    }
}
