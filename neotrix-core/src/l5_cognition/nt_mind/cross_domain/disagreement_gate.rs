#![forbid(unsafe_code)]

//! 分歧门控（Disagreement Gate）
//!
//! 防止知识干扰的关键机制：
//! 1. 检测知识片段之间的分歧
//! 2. 评估分歧的严重程度
//! 3. 根据分歧类型决定是否阻止迁移
//! 4. 提供分歧解决策略
//!
//! 基于 Agent KB 论文中的"知识冲突检测"机制

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 分歧类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DisagreementType {
    /// 事实分歧
    FactualDisagreement,
    /// 概念分歧
    ConceptualDisagreement,
    /// 方法分歧
    MethodologicalDisagreement,
    /// 结构分歧
    StructuralDisagreement,
    /// 语义分歧
    SemanticDisagreement,
}

/// 分歧严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DisagreementSeverity {
    /// 低分歧（可忽略）
    Low,
    /// 中等分歧（需要调整）
    Medium,
    /// 高分歧（需要解决）
    High,
    /// 严重分歧（阻止迁移）
    Critical,
}

/// 分歧检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisagreementDetection {
    /// 分歧类型
    pub disagreement_type: DisagreementType,
    /// 严重程度
    pub severity: DisagreementSeverity,
    /// 涉及的知识片段ID
    pub involved_items: Vec<String>,
    /// 分歧描述
    pub description: String,
    /// 置信度
    pub confidence: f64,
    /// 解决建议
    pub resolution_suggestion: String,
}

/// 门控决策
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GateDecision {
    /// 通过
    Pass,
    /// 警告但通过
    PassWithWarning,
    /// 需要调整
    NeedsAdjustment,
    /// 阻止
    Block,
}

/// 门控评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateEvaluation {
    /// 门控决策
    pub decision: GateDecision,
    /// 检测到的分歧
    pub detected_disagreements: Vec<DisagreementDetection>,
    /// 总体风险评分 (0.0-1.0)
    pub risk_score: f64,
    /// 评估摘要
    pub summary: String,
    /// 建议的调整措施
    pub recommended_adjustments: Vec<String>,
}

/// 分歧门控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisagreementGateConfig {
    /// 最大允许风险评分
    pub max_risk_score: f64,
    /// 最大允许中等分歧数量
    pub max_medium_disagreements: usize,
    /// 最大允许高分歧数量
    pub max_high_disagreements: usize,
    /// 是否允许严重分歧
    pub allow_critical_disagreements: bool,
    /// 分歧检测灵敏度 (0.0-1.0)
    pub detection_sensitivity: f64,
    /// 解决策略
    pub resolution_strategy: ResolutionStrategy,
}

impl Default for DisagreementGateConfig {
    fn default() -> Self {
        Self {
            max_risk_score: 0.7,
            max_medium_disagreements: 3,
            max_high_disagreements: 1,
            allow_critical_disagreements: false,
            detection_sensitivity: 0.8,
            resolution_strategy: ResolutionStrategy::Conservative,
        }
    }
}

/// 解决策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResolutionStrategy {
    /// 保守策略：优先阻止
    Conservative,
    /// 平衡策略：权衡利弊
    Balanced,
    /// 激进策略：优先通过
    Aggressive,
    /// 自适应策略：根据情况调整
    Adaptive,
}

/// 分歧门控
pub struct DisagreementGate {
    /// 配置
    config: DisagreementGateConfig,
    /// 历史分歧记录
    history: Vec<DisagreementDetection>,
    /// 分歧模式库
    disagreement_patterns: Vec<DisagreementPattern>,
}

/// 分歧模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisagreementPattern {
    /// 模式名称
    pub name: String,
    /// 模式类型
    pub pattern_type: DisagreementType,
    /// 匹配条件
    pub match_conditions: Vec<MatchCondition>,
    /// 默认严重程度
    pub default_severity: DisagreementSeverity,
    /// 解决策略
    pub resolution: String,
}

/// 匹配条件
// 注：原此处有 `#[derive(Debug, Clone, Serialize, Deserialize)]`，是 2026-09-29
// `nt_fuse_types.py` 把本文件的 MatchCondition 定义换成 re-export 时的**残留**——
// derive 挂在 `pub use` 上 ⇒ E0774。类型现由 entity_mapping 提供，derive 亦随之。
// 2026-09-29 自动融合（nt_fuse_types.py）：`MatchCondition` 原在本文件与
// `l5_cognition/nt_mind/cross_domain/entity_mapping.rs` 各有一份，字段名+类型+impl 块完全相同。
// 真源是后者（模块 mod.rs 的 re-export 指向它）⇒ 本文件改为 re-export，
// 消除「两份同名类型」的歧义。
pub use super::entity_mapping::MatchCondition;


impl DisagreementGate {
    /// 创建新的分歧门控
    pub fn new(config: DisagreementGateConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
            disagreement_patterns: Vec::new(),
        }
    }

    /// 注册分歧模式
    pub fn register_pattern(&mut self, pattern: DisagreementPattern) {
        self.disagreement_patterns.push(pattern);
    }

    /// 评估知识迁移的分歧风险
    pub fn evaluate(
        &mut self,
        knowledge_items: &[super::KnowledgeItem],
        context: &super::TransferContext,
    ) -> GateEvaluation {
        let mut detected_disagreements = Vec::new();
        let mut risk_score = 0.0;

        // 检测知识片段之间的分歧
        for i in 0..knowledge_items.len() {
            for j in (i + 1)..knowledge_items.len() {
                if let Some(disagreement) =
                    self.detect_disagreement(&knowledge_items[i], &knowledge_items[j], context)
                {
                    risk_score += self.calculate_risk_contribution(&disagreement);
                    detected_disagreements.push(disagreement);
                }
            }
        }

        // 检测与目标域的分歧
        for item in knowledge_items {
            if let Some(disagreement) = self.detect_domain_disagreement(item, context) {
                risk_score += self.calculate_risk_contribution(&disagreement);
                detected_disagreements.push(disagreement);
            }
        }

        // 归一化风险评分
        risk_score = risk_score.min(1.0);

        // 根据风险评分和分歧类型做出决策
        let decision = self.make_decision(&detected_disagreements, risk_score);

        // 生成调整建议
        let recommended_adjustments = self.generate_adjustments(&detected_disagreements, &decision);

        // 生成摘要
        let summary = self.generate_summary(&detected_disagreements, risk_score, &decision);

        // 记录历史
        self.history.extend(detected_disagreements.clone());

        GateEvaluation {
            decision,
            detected_disagreements,
            risk_score,
            summary,
            recommended_adjustments,
        }
    }

    /// 检测两个知识片段之间的分歧
    fn detect_disagreement(
        &self,
        item1: &super::KnowledgeItem,
        item2: &super::KnowledgeItem,
        _context: &super::TransferContext,
    ) -> Option<DisagreementDetection> {
        // 基于内容相似度检测分歧
        let similarity = self.calculate_content_similarity(&item1.content, &item2.content);

        // 如果内容相似但类型不同，可能存在概念分歧
        if similarity > 0.5 && item1.knowledge_type != item2.knowledge_type {
            return Some(DisagreementDetection {
                disagreement_type: DisagreementType::ConceptualDisagreement,
                severity: DisagreementSeverity::Medium,
                involved_items: vec![item1.id.clone(), item2.id.clone()],
                description: format!("知识片段 {} 和 {} 内容相似但类型不同", item1.id, item2.id),
                confidence: 0.7,
                resolution_suggestion: "考虑合并或重新分类知识片段".to_string(),
            });
        }

        // 如果来自不同域且内容冲突，可能存在事实分歧
        if item1.source_domain != item2.source_domain && similarity > 0.8 {
            return Some(DisagreementDetection {
                disagreement_type: DisagreementType::FactualDisagreement,
                severity: DisagreementSeverity::High,
                involved_items: vec![item1.id.clone(), item2.id.clone()],
                description: format!(
                    "来自不同域的知识片段 {} 和 {} 内容高度相似但可能存在事实冲突",
                    item1.id, item2.id
                ),
                confidence: 0.8,
                resolution_suggestion: "验证事实一致性，考虑来源可靠性".to_string(),
            });
        }

        None
    }

    /// 检测知识片段与目标域的分歧
    fn detect_domain_disagreement(
        &self,
        item: &super::KnowledgeItem,
        context: &super::TransferContext,
    ) -> Option<DisagreementDetection> {
        // 检查域兼容性
        if item.source_domain == context.target_domain {
            return None; // 同域迁移，通常没有分歧
        }

        // 基于元数据检查域特定约束
        if let Some(dominant_style) = item.metadata.get("dominant_style") {
            if let Some(target_style) = context
                .constraints
                .iter()
                .find(|c| {
                    c.constraint_type
                        == super::ConstraintType::DomainCompatibility
                })
                .map(|c| &c.value)
            {
                if dominant_style != target_style {
                    return Some(DisagreementDetection {
                        disagreement_type: DisagreementType::MethodologicalDisagreement,
                        severity: DisagreementSeverity::Medium,
                        involved_items: vec![item.id.clone()],
                        description: format!(
                            "知识片段 {} 的方法风格 '{}' 与目标域要求 '{}' 不符",
                            item.id, dominant_style, target_style
                        ),
                        confidence: 0.6,
                        resolution_suggestion: "调整方法以适应目标域风格".to_string(),
                    });
                }
            }
        }

        None
    }

    /// 计算内容相似度（简化版）
    fn calculate_content_similarity(&self, content1: &str, content2: &str) -> f64 {
        // 简化的相似度计算：基于共同词汇
        let words1: Vec<&str> = content1.split_whitespace().collect();
        let words2: Vec<&str> = content2.split_whitespace().collect();

        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }

        let common_count = words1.iter().filter(|w| words2.contains(w)).count();

        common_count as f64 / words1.len().max(words2.len()) as f64
    }

    /// 计算分歧的风险贡献
    fn calculate_risk_contribution(&self, disagreement: &DisagreementDetection) -> f64 {
        let severity_weight = match disagreement.severity {
            DisagreementSeverity::Low => 0.1,
            DisagreementSeverity::Medium => 0.3,
            DisagreementSeverity::High => 0.6,
            DisagreementSeverity::Critical => 1.0,
        };

        severity_weight * disagreement.confidence
    }

    /// 做出门控决策
    fn make_decision(
        &self,
        disagreements: &[DisagreementDetection],
        risk_score: f64,
    ) -> GateDecision {
        // 检查严重分歧
        let has_critical = disagreements
            .iter()
            .any(|d| d.severity == DisagreementSeverity::Critical);

        if has_critical && !self.config.allow_critical_disagreements {
            return GateDecision::Block;
        }

        // 统计高分歧数量
        let high_count = disagreements
            .iter()
            .filter(|d| d.severity == DisagreementSeverity::High)
            .count();

        if high_count > self.config.max_high_disagreements {
            return GateDecision::Block;
        }

        // 统计中等分歧数量
        let medium_count = disagreements
            .iter()
            .filter(|d| d.severity == DisagreementSeverity::Medium)
            .count();

        if medium_count > self.config.max_medium_disagreements {
            return GateDecision::NeedsAdjustment;
        }

        // 基于风险评分决策
        if risk_score > self.config.max_risk_score {
            GateDecision::NeedsAdjustment
        } else if risk_score > self.config.max_risk_score * 0.7 {
            GateDecision::PassWithWarning
        } else {
            GateDecision::Pass
        }
    }

    /// 生成调整建议
    fn generate_adjustments(
        &self,
        disagreements: &[DisagreementDetection],
        decision: &GateDecision,
    ) -> Vec<String> {
        let mut adjustments = Vec::new();

        match decision {
            GateDecision::Block => {
                adjustments.push("迁移被阻止，需要解决所有严重分歧".to_string());
                for d in disagreements {
                    if d.severity == DisagreementSeverity::Critical
                        || d.severity == DisagreementSeverity::High
                    {
                        adjustments
                            .push(format!("解决 {:?}: {}", d.disagreement_type, d.description));
                    }
                }
            }
            GateDecision::NeedsAdjustment => {
                adjustments.push("需要调整知识片段以减少分歧".to_string());
                for d in disagreements {
                    if d.severity == DisagreementSeverity::Medium {
                        adjustments.push(d.resolution_suggestion.clone());
                    }
                }
            }
            GateDecision::PassWithWarning => {
                adjustments.push("迁移通过但需要注意以下分歧".to_string());
                for d in disagreements {
                    adjustments.push(format!("注意: {}", d.description));
                }
            }
            GateDecision::Pass => {
                adjustments.push("迁移通过，无需调整".to_string());
            }
        }

        adjustments
    }

    /// 生成摘要
    fn generate_summary(
        &self,
        disagreements: &[DisagreementDetection],
        risk_score: f64,
        decision: &GateDecision,
    ) -> String {
        let total = disagreements.len();
        let critical = disagreements
            .iter()
            .filter(|d| d.severity == DisagreementSeverity::Critical)
            .count();
        let high = disagreements
            .iter()
            .filter(|d| d.severity == DisagreementSeverity::High)
            .count();
        let medium = disagreements
            .iter()
            .filter(|d| d.severity == DisagreementSeverity::Medium)
            .count();
        let low = disagreements
            .iter()
            .filter(|d| d.severity == DisagreementSeverity::Low)
            .count();

        format!(
            "分歧检测完成: {} 个分歧 (严重: {}, 高: {}, 中: {}, 低: {}), 风险评分: {:.2}, 决策: {:?}",
            total, critical, high, medium, low, risk_score, decision
        )
    }

    /// 获取历史分歧统计
    pub fn get_history_stats(&self) -> DisagreementStats {
        let total = self.history.len();
        let by_type = self.history.iter().fold(HashMap::new(), |mut acc, d| {
            *acc.entry(d.disagreement_type.clone()).or_insert(0) += 1;
            acc
        });
        let by_severity = self.history.iter().fold(HashMap::new(), |mut acc, d| {
            *acc.entry(d.severity.clone()).or_insert(0) += 1;
            acc
        });

        DisagreementStats {
            total_disagreements: total,
            by_type,
            by_severity,
        }
    }
}

/// 分歧统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisagreementStats {
    /// 总分歧数量
    pub total_disagreements: usize,
    /// 按类型统计
    pub by_type: HashMap<DisagreementType, usize>,
    /// 按严重程度统计
    pub by_severity: HashMap<DisagreementSeverity, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l5_cognition::nt_mind::cross_domain::{
        Constraint, ConstraintType, KnowledgeItem, KnowledgeType, TaskType, TransferContext,
    };
    use std::collections::HashMap;

    fn create_test_item(
        id: &str,
        domain: &str,
        content: &str,
        knowledge_type: KnowledgeType,
    ) -> KnowledgeItem {
        KnowledgeItem {
            id: id.to_string(),
            content: content.to_string(),
            knowledge_type,
            source_domain: domain.to_string(),
            metadata: HashMap::new(),
            embedding: None,
        }
    }

    fn create_test_context(source: &str, target: &str) -> TransferContext {
        TransferContext {
            source_domain: source.to_string(),
            target_domain: target.to_string(),
            task_description: "测试任务".to_string(),
            task_type: TaskType::ConceptTransfer,
            constraints: vec![Constraint {
                constraint_type: ConstraintType::DomainCompatibility,
                value: "functional".to_string(),
                strength: 0.8,
            }],
        }
    }

    #[test]
    fn test_gate_config_default() {
        let config = DisagreementGateConfig::default();
        assert_eq!(config.max_risk_score, 0.7);
        assert_eq!(config.max_medium_disagreements, 3);
        assert_eq!(config.max_high_disagreements, 1);
        assert!(!config.allow_critical_disagreements);
    }

    #[test]
    fn test_no_disagreement() {
        let config = DisagreementGateConfig::default();
        let mut gate = DisagreementGate::new(config);

        let items = vec![
            create_test_item("1", "rust", "Rust 所有权系统", KnowledgeType::Concept),
            create_test_item("2", "rust", "Rust 借用检查器", KnowledgeType::Concept),
        ];

        let context = create_test_context("rust", "python");
        let evaluation = gate.evaluate(&items, &context);

        assert_eq!(evaluation.decision, GateDecision::Pass);
        assert!(evaluation.risk_score < 0.5);
    }

    #[test]
    fn test_conceptual_disagreement() {
        let config = DisagreementGateConfig::default();
        let mut gate = DisagreementGate::new(config);

        let items = vec![
            create_test_item("1", "rust", "相同内容", KnowledgeType::Concept),
            create_test_item("2", "rust", "相同内容", KnowledgeType::Procedural),
        ];

        let context = create_test_context("rust", "python");
        let evaluation = gate.evaluate(&items, &context);

        assert!(!evaluation.detected_disagreements.is_empty());
        assert!(evaluation
            .detected_disagreements
            .iter()
            .any(|d| d.disagreement_type == DisagreementType::ConceptualDisagreement));
    }

    #[test]
    fn test_factual_disagreement() {
        let config = DisagreementGateConfig::default();
        let mut gate = DisagreementGate::new(config);

        let items = vec![
            create_test_item(
                "1",
                "rust",
                "非常相似的内容用于测试事实分歧检测",
                KnowledgeType::Concept,
            ),
            create_test_item(
                "2",
                "python",
                "非常相似的内容用于测试事实分歧检测",
                KnowledgeType::Concept,
            ),
        ];

        let context = create_test_context("rust", "python");
        let evaluation = gate.evaluate(&items, &context);

        assert!(!evaluation.detected_disagreements.is_empty());
        assert!(evaluation
            .detected_disagreements
            .iter()
            .any(|d| d.disagreement_type == DisagreementType::FactualDisagreement));
    }
}
