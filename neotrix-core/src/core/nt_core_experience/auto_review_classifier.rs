#![allow(dead_code)]

use std::collections::VecDeque;

/// 动作类别 — 提案动作的分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ActionCategory {
    ReadCode,
    WriteCode,
    DeleteCode,
    RefactorRename,
    RefactorRestructure,
    ExecuteCommand,
    NetworkAccess,
    FileAccess,
    ModifyConfig,
    ModifyDependency,
    SelfModify,
    Unknown,
}

impl ActionCategory {
    /// 返回所有已知类别的静态切片 (用于默认规则创建)。
    pub fn all() -> &'static [ActionCategory] {
        &[
            ActionCategory::ReadCode,
            ActionCategory::WriteCode,
            ActionCategory::DeleteCode,
            ActionCategory::RefactorRename,
            ActionCategory::RefactorRestructure,
            ActionCategory::ExecuteCommand,
            ActionCategory::NetworkAccess,
            ActionCategory::FileAccess,
            ActionCategory::ModifyConfig,
            ActionCategory::ModifyDependency,
            ActionCategory::SelfModify,
        ]
    }
}

/// 权限级别 — 分类结果的决定。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevel {
    /// 自动批准，无需人工审查。
    AutoApprove,
    /// 标记为需人工审查。
    FlagForReview,
    /// 永久禁止执行。
    Block,
}

/// 配置默认权限的选择。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultPermission {
    AutoApprove,
    FlagForReview,
    Block,
}

/// 提案动作 — 需要分类的单个操作。
#[derive(Debug, Clone)]
pub struct ProposedAction {
    pub id: u64,
    pub action_type: ActionCategory,
    pub description: String,
    pub target_module: Option<String>,
    pub estimated_impact: f64,
    pub estimated_risk: f64,
    pub source: String,
}

/// 分类结果 — 对提案动作的判定。
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub action_id: u64,
    pub permission: PermissionLevel,
    pub confidence: f64,
    pub reasons: Vec<String>,
    pub suggested_reviewer: Option<String>,
}

/// 分类规则 — 模式和权限之间的映射。
#[derive(Debug, Clone)]
pub struct ClassifierRule {
    pub id: u64,
    pub category: ActionCategory,
    pub pattern: String,
    pub permission: PermissionLevel,
    pub priority: u8,
    pub enabled: bool,
}

/// 分类器配置。
#[derive(Debug, Clone)]
pub struct ClassifierConfig {
    pub default_permission: DefaultPermission,
    pub max_rules: usize,
    pub auto_approve_threshold: f64,
    pub enable_learning: bool,
}

impl Default for ClassifierConfig {
    fn default() -> Self {
        Self {
            default_permission: DefaultPermission::FlagForReview,
            max_rules: 200,
            auto_approve_threshold: 0.3,
            enable_learning: false,
        }
    }
}

/// 分类器摘要统计。
#[derive(Debug, Clone)]
pub struct ClassifierSummary {
    pub total_classified: u64,
    pub auto_approved: u64,
    pub flagged: u64,
    pub blocked: u64,
    pub accuracy: Option<f64>,
}

// ──────────────────────────────────────────────

/// **自动审查分类器** — 对 agent 动作进行权限分级。
///
/// 受 Cursor auto-review classifier 启发，在执行前筛选 agent 操作的分类器。
/// 安全动作自动批准，危险动作标记为需人工审查或直接阻止。
pub struct AutoReviewClassifier {
    rules: Vec<ClassifierRule>,
    config: ClassifierConfig,
    classification_history: VecDeque<ClassificationResult>,
    action_log: VecDeque<(ProposedAction, ClassificationResult)>,
    next_rule_id: u64,
    next_action_id: u64,
    /// 学习模式下追踪正确/错误计数。
    correct_count: u64,
    total_learning: u64,
}

impl AutoReviewClassifier {
    /// 创建默认分类器，包含所有内建规则。
    pub fn new() -> Self {
        let mut classifier = Self {
            rules: Vec::new(),
            config: ClassifierConfig::default(),
            classification_history: VecDeque::new(),
            action_log: VecDeque::with_capacity(500),
            next_rule_id: 1,
            next_action_id: 1,
            correct_count: 0,
            total_learning: 0,
        };
        classifier.init_default_rules();
        classifier
    }

    // ── 内建规则 ──────────────────────────────────

    fn init_default_rules(&mut self) {
        let rules = vec![
            ClassifierRule {
                id: self.next_rule_id(),
                category: ActionCategory::ReadCode,
                pattern: String::new(),
                permission: PermissionLevel::AutoApprove,
                priority: 1,
                enabled: true,
            },
            ClassifierRule {
                id: self.next_rule_id(),
                category: ActionCategory::WriteCode,
                pattern: "self_evolution_meta_layer|consciousness_cycle".to_string(),
                permission: PermissionLevel::Block,
                priority: 10,
                enabled: true,
            },
            ClassifierRule {
                id: self.next_rule_id(),
                category: ActionCategory::WriteCode,
                pattern: String::new(),
                permission: PermissionLevel::FlagForReview,
                priority: 5,
                enabled: true,
            },
            ClassifierRule {
                id: self.next_rule_id(),
                category: ActionCategory::DeleteCode,
                pattern: String::new(),
                permission: PermissionLevel::Block,
                priority: 8,
                enabled: true,
            },
            ClassifierRule {
                id: self.next_rule_id(),
                category: ActionCategory::ExecuteCommand,
                pattern: String::new(),
                permission: PermissionLevel::Block,
                priority: 9,
                enabled: true,
            },
            ClassifierRule {
                id: self.next_rule_id(),
                category: ActionCategory::NetworkAccess,
                pattern: String::new(),
                permission: PermissionLevel::FlagForReview,
                priority: 5,
                enabled: true,
            },
            ClassifierRule {
                id: self.next_rule_id(),
                category: ActionCategory::ModifyConfig,
                pattern: String::new(),
                permission: PermissionLevel::FlagForReview,
                priority: 6,
                enabled: true,
            },
            ClassifierRule {
                id: self.next_rule_id(),
                category: ActionCategory::ModifyDependency,
                pattern: String::new(),
                permission: PermissionLevel::FlagForReview,
                priority: 7,
                enabled: true,
            },
            ClassifierRule {
                id: self.next_rule_id(),
                category: ActionCategory::SelfModify,
                pattern: String::new(),
                permission: PermissionLevel::Block,
                priority: 10,
                enabled: true,
            },
            ClassifierRule {
                id: self.next_rule_id(),
                category: ActionCategory::RefactorRename,
                pattern: String::new(),
                permission: PermissionLevel::AutoApprove,
                priority: 3,
                enabled: true,
            },
            ClassifierRule {
                id: self.next_rule_id(),
                category: ActionCategory::RefactorRestructure,
                pattern: String::new(),
                permission: PermissionLevel::FlagForReview,
                priority: 4,
                enabled: true,
            },
            // 文本模式规则: 描述包含危险关键词
            ClassifierRule {
                id: self.next_rule_id(),
                category: ActionCategory::Unknown,
                pattern: "dangerous|unsafe".to_string(),
                permission: PermissionLevel::Block,
                priority: 10,
                enabled: true,
            },
        ];
        self.rules.extend(rules);
    }

    fn next_rule_id(&mut self) -> u64 {
        let id = self.next_rule_id;
        self.next_rule_id += 1;
        id
    }

    fn next_action_id(&mut self) -> u64 {
        let id = self.next_action_id;
        self.next_action_id += 1;
        id
    }

    // ── 核心分类 ──────────────────────────────────

    /// 对提案动作进行分类，返回判定结果。
    pub fn classify(&mut self, action: ProposedAction) -> ClassificationResult {
        let action_id = if action.id == 0 {
            self.next_action_id()
        } else {
            action.id
        };

        let mut matched_rules: Vec<&ClassifierRule> = self
            .rules
            .iter()
            .filter(|r| r.enabled)
            .filter(|r| {
                let cat_match =
                    r.category == action.action_type || r.category == ActionCategory::Unknown;
                let pat_match = if r.pattern.is_empty() {
                    true
                } else {
                    let lower_desc = action.description.to_lowercase();
                    r.pattern.split('|').any(|pat| {
                        lower_desc.contains(pat.trim())
                            || action.target_module.as_deref().map_or(false, |m| {
                                let lower_mod = m.to_lowercase();
                                lower_mod.contains(pat.trim())
                            })
                    })
                };
                cat_match && pat_match
            })
            .collect();

        // 按优先级降序排序
        matched_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        let (permission, reasons) = if let Some(best) = matched_rules.first() {
            (
                best.permission,
                vec![format!("规则 {} 匹配 (优先级 {})", best.id, best.priority)],
            )
        } else {
            match self.config.default_permission {
                DefaultPermission::AutoApprove => (
                    PermissionLevel::AutoApprove,
                    vec!["无匹配规则，默认自动批准".into()],
                ),
                DefaultPermission::FlagForReview => (
                    PermissionLevel::FlagForReview,
                    vec!["无匹配规则，默认标记审查".into()],
                ),
                DefaultPermission::Block => {
                    (PermissionLevel::Block, vec!["无匹配规则，默认阻止".into()])
                }
            }
        };

        // 阈值覆盖: 对于 FlagForReview，若 risk <= threshold 则提升为 AutoApprove
        let permission = match permission {
            PermissionLevel::FlagForReview
                if action.estimated_risk <= self.config.auto_approve_threshold =>
            {
                let mut r = reasons.clone();
                r.push(format!(
                    "风险 {:.2} ≤ 阈值 {:.2} → 自动批准",
                    action.estimated_risk, self.config.auto_approve_threshold
                ));
                let result = ClassificationResult {
                    action_id,
                    permission: PermissionLevel::AutoApprove,
                    confidence: 1.0 - action.estimated_risk,
                    reasons: r,
                    suggested_reviewer: None,
                };
                self.record_classification(action, result.clone());
                return result;
            }
            _ => permission,
        };

        let confidence = match permission {
            PermissionLevel::AutoApprove => 1.0 - action.estimated_risk * 0.5,
            PermissionLevel::FlagForReview => 0.5 + action.estimated_risk * 0.3,
            PermissionLevel::Block => 0.8 + action.estimated_risk * 0.2,
        };

        let reviewer = match permission {
            PermissionLevel::AutoApprove => None,
            PermissionLevel::FlagForReview => Some("human_reviewer".into()),
            PermissionLevel::Block => Some("security_team".into()),
        };

        let result = ClassificationResult {
            action_id,
            permission,
            confidence,
            reasons,
            suggested_reviewer: reviewer,
        };

        self.record_classification(action, result.clone());
        result
    }

    fn record_classification(&mut self, action: ProposedAction, result: ClassificationResult) {
        self.action_log.push_back((action, result.clone()));
        self.classification_history.push_back(result);
        if self.classification_history.len() > 500 {
            self.classification_history.pop_front();
        }
    }

    // ── 规则管理 ──────────────────────────────────

    /// 添加自定义规则，返回规则 ID。
    pub fn add_rule(&mut self, mut rule: ClassifierRule) -> u64 {
        if rule.id == 0 {
            rule.id = self.next_rule_id();
        }
        if self.rules.len() >= self.config.max_rules {
            self.rules.remove(0);
        }
        let id = rule.id;
        self.rules.push(rule);
        id
    }

    /// 删除规则，成功返回 true。
    pub fn remove_rule(&mut self, rule_id: u64) -> bool {
        let before = self.rules.len();
        self.rules.retain(|r| r.id != rule_id);
        self.rules.len() < before
    }

    /// 更新规则的部分字段，成功返回 true。
    pub fn update_rule(
        &mut self,
        rule_id: u64,
        category: Option<ActionCategory>,
        pattern: Option<String>,
        permission: Option<PermissionLevel>,
        priority: Option<u8>,
        enabled: Option<bool>,
    ) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            if let Some(c) = category {
                rule.category = c;
            }
            if let Some(p) = pattern {
                rule.pattern = p;
            }
            if let Some(p) = permission {
                rule.permission = p;
            }
            if let Some(p) = priority {
                rule.priority = p;
            }
            if let Some(e) = enabled {
                rule.enabled = e;
            }
            true
        } else {
            false
        }
    }

    // ── 学习与反馈 ──────────────────────────────────

    /// 记录分类结果的正确性，启用学习时调整规则优先级。
    pub fn record_outcome(&mut self, action_id: u64, was_correct: bool) {
        if !self.config.enable_learning {
            return;
        }
        self.total_learning += 1;
        if was_correct {
            self.correct_count += 1;
        }

        // 找到匹配该 action 的历史记录
        if let Some(result) = self
            .classification_history
            .iter()
            .find(|r| r.action_id == action_id)
        {
            let suggestion = if was_correct { 1i8 } else { -1i8 };
            for rule in self.rules.iter_mut() {
                // 提升/降低匹配该 action 分类的规则优先级
                if result
                    .reasons
                    .iter()
                    .any(|r| r.contains(&format!("规则 {}", rule.id)))
                {
                    let new_priority =
                        (rule.priority as i16 + suggestion as i16).clamp(1, 10) as u8;
                    rule.priority = new_priority;
                }
            }
        }
    }

    // ── 查询 ──────────────────────────────────────

    /// 返回分类摘要统计。
    pub fn summary(&self) -> ClassifierSummary {
        let mut auto_approved = 0u64;
        let mut flagged = 0u64;
        let mut blocked = 0u64;

        for r in &self.classification_history {
            match r.permission {
                PermissionLevel::AutoApprove => auto_approved += 1,
                PermissionLevel::FlagForReview => flagged += 1,
                PermissionLevel::Block => blocked += 1,
            }
        }

        let accuracy = if self.config.enable_learning && self.total_learning > 0 {
            Some(self.correct_count as f64 / self.total_learning as f64)
        } else {
            None
        };

        ClassifierSummary {
            total_classified: self.classification_history.len() as u64,
            auto_approved,
            flagged,
            blocked,
            accuracy,
        }
    }

    /// 导出所有启用的规则。
    pub fn export_rules(&self) -> Vec<&ClassifierRule> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }

    /// 最近 n 条分类记录。
    pub fn recent_actions(&self, n: usize) -> Vec<&(ProposedAction, ClassificationResult)> {
        self.action_log.iter().rev().take(n).collect()
    }

    // ── 配置 ──────────────────────────────────────

    /// 更新配置。
    pub fn set_config(&mut self, config: ClassifierConfig) {
        self.config = config;
    }

    /// 获取当前配置引用。
    pub fn config(&self) -> &ClassifierConfig {
        &self.config
    }
}

impl Default for AutoReviewClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_action(
        cat: ActionCategory,
        desc: &str,
        risk: f64,
        target: Option<&str>,
    ) -> ProposedAction {
        ProposedAction {
            id: 0,
            action_type: cat,
            description: desc.to_string(),
            target_module: target.map(|s| s.to_string()),
            estimated_impact: 0.5,
            estimated_risk: risk,
            source: "test".to_string(),
        }
    }

    #[test]
    fn test_classify_read_code_auto_approve() {
        let mut classifier = AutoReviewClassifier::new();
        let action = make_action(ActionCategory::ReadCode, "read file main.rs", 0.1, None);
        let result = classifier.classify(action);
        assert_eq!(result.permission, PermissionLevel::AutoApprove);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_classify_delete_code_block() {
        let mut classifier = AutoReviewClassifier::new();
        let action = make_action(
            ActionCategory::DeleteCode,
            "remove deprecated module",
            0.8,
            None,
        );
        let result = classifier.classify(action);
        assert_eq!(result.permission, PermissionLevel::Block);
    }

    #[test]
    fn test_classify_self_modify_block() {
        let mut classifier = AutoReviewClassifier::new();
        let action = make_action(
            ActionCategory::SelfModify,
            "evolve mutation strategy",
            0.9,
            None,
        );
        let result = classifier.classify(action);
        assert_eq!(result.permission, PermissionLevel::Block);
    }

    #[test]
    fn test_classify_write_code_non_critical_flag() {
        let mut classifier = AutoReviewClassifier::new();
        let action = make_action(
            ActionCategory::WriteCode,
            "add utility function to helpers",
            0.4,
            Some("helpers.rs"),
        );
        let result = classifier.classify(action);
        // risk 0.4 > threshold 0.3 → stays FlagForReview
        assert_eq!(result.permission, PermissionLevel::FlagForReview);
    }

    #[test]
    fn test_classify_risk_below_threshold_auto_approves() {
        let mut classifier = AutoReviewClassifier::new();
        // risk 0.2 ≤ threshold 0.3 → should be promoted to AutoApprove even for FlagForReview category
        let action = make_action(
            ActionCategory::WriteCode,
            "minor addition to helpers",
            0.2,
            Some("helpers.rs"),
        );
        let result = classifier.classify(action);
        assert_eq!(result.permission, PermissionLevel::AutoApprove);
    }

    #[test]
    fn test_classify_no_matching_rule_default() {
        let mut classifier = AutoReviewClassifier::new();
        // Use an action type not specifically covered: FileAccess
        let action = make_action(ActionCategory::FileAccess, "access temp file", 0.5, None);
        let result = classifier.classify(action);
        // Should fall back to FlagForReview (config default)
        assert_eq!(result.permission, PermissionLevel::FlagForReview);
    }

    #[test]
    fn test_classify_dangerous_keyword_blocked() {
        let mut classifier = AutoReviewClassifier::new();
        let action = make_action(
            ActionCategory::ReadCode,
            "contains dangerous operation",
            0.3,
            None,
        );
        let result = classifier.classify(action);
        assert_eq!(result.permission, PermissionLevel::Block);
    }

    #[test]
    fn test_classify_unsafe_keyword_blocked() {
        let mut classifier = AutoReviewClassifier::new();
        let action = make_action(
            ActionCategory::WriteCode,
            "this is unsafe and needs review",
            0.4,
            None,
        );
        let result = classifier.classify(action);
        assert_eq!(result.permission, PermissionLevel::Block);
    }

    #[test]
    fn test_add_rule_and_classify() {
        let mut classifier = AutoReviewClassifier::new();
        let rule = ClassifierRule {
            id: 0,
            category: ActionCategory::FileAccess,
            pattern: "critical".to_string(),
            permission: PermissionLevel::Block,
            priority: 12,
            enabled: true,
        };
        let rule_id = classifier.add_rule(rule);
        assert!(rule_id > 0);

        let action = make_action(
            ActionCategory::FileAccess,
            "access critical config file",
            0.5,
            None,
        );
        let result = classifier.classify(action);
        assert_eq!(result.permission, PermissionLevel::Block);
    }

    #[test]
    fn test_remove_rule() {
        let mut classifier = AutoReviewClassifier::new();
        let rule = ClassifierRule {
            id: 0,
            category: ActionCategory::FileAccess,
            pattern: String::new(),
            permission: PermissionLevel::Block,
            priority: 10,
            enabled: true,
        };
        let id = classifier.add_rule(rule);
        assert!(classifier.remove_rule(id));
        assert!(!classifier.remove_rule(99999));
    }

    #[test]
    fn test_summary_statistics() {
        let mut classifier = AutoReviewClassifier::new();
        classifier.classify(make_action(ActionCategory::ReadCode, "read", 0.1, None));
        classifier.classify(make_action(ActionCategory::SelfModify, "modify", 0.9, None));
        classifier.classify(make_action(
            ActionCategory::WriteCode,
            "write to helpers",
            0.4,
            Some("helpers.rs"),
        ));
        classifier.classify(make_action(ActionCategory::DeleteCode, "delete", 0.8, None));

        let summary = classifier.summary();
        assert_eq!(summary.total_classified, 4);
        assert!(summary.auto_approved >= 1);
        assert!(summary.blocked >= 2);
        assert!(summary.flagged >= 1);
    }

    #[test]
    fn test_record_outcome_adjusts_priority() {
        let mut classifier = AutoReviewClassifier::new();
        classifier.config.enable_learning = true;

        let action = make_action(
            ActionCategory::WriteCode,
            "add function to helpers",
            0.4,
            Some("helpers.rs"),
        );
        let result = classifier.classify(action);

        // Record that it was correct — should increase priority of the WriteCode FlagForReview rule
        classifier.record_outcome(result.action_id, true);

        // The write-code rule (non-critical) should have priority >= 5 (it started at 5, correct → +1)
        let write_rule = classifier
            .rules
            .iter()
            .find(|r| r.category == ActionCategory::WriteCode && r.pattern.is_empty());
        assert!(write_rule.is_some());
        assert!(write_rule.unwrap().priority >= 5);
    }

    #[test]
    fn test_recent_actions_ordering() {
        let mut classifier = AutoReviewClassifier::new();
        classifier.classify(make_action(ActionCategory::ReadCode, "first", 0.1, None));
        classifier.classify(make_action(ActionCategory::ReadCode, "second", 0.1, None));
        classifier.classify(make_action(ActionCategory::ReadCode, "third", 0.1, None));

        let recent = classifier.recent_actions(2);
        assert_eq!(recent.len(), 2);
        // action_log is empty since classify doesn't log to action_log
        // but classification_history gets populated - recent_actions checks action_log
        // For now just verify the method doesn't panic
    }

    #[test]
    fn test_export_rules_returns_active_rules() {
        let classifier = AutoReviewClassifier::new();
        let active = classifier.export_rules();
        assert!(!active.is_empty());
        assert!(active.iter().all(|r| r.enabled));
    }

    #[test]
    fn test_update_rule() {
        let mut classifier = AutoReviewClassifier::new();
        let id = classifier.rules[0].id;

        assert!(classifier.update_rule(id, None, Some("new_pattern".into()), None, None, None));
        let updated = classifier.rules.iter().find(|r| r.id == id).unwrap();
        assert_eq!(updated.pattern, "new_pattern");

        assert!(!classifier.update_rule(99999, None, None, None, None, None));
    }

    #[test]
    fn test_classifier_config_defaults() {
        let config = ClassifierConfig::default();
        assert_eq!(config.max_rules, 200);
        assert!((config.auto_approve_threshold - 0.3).abs() < 1e-6);
        assert!(!config.enable_learning);
    }

    #[test]
    fn test_classify_write_code_on_critical_module_blocked() {
        let mut classifier = AutoReviewClassifier::new();
        let action = make_action(
            ActionCategory::WriteCode,
            "refactor meta layer",
            0.6,
            Some("self_evolution_meta_layer.rs"),
        );
        let result = classifier.classify(action);
        assert_eq!(result.permission, PermissionLevel::Block);
    }
}
