//! 动态 GWT 专家路由 (Ext-5) — E8 状态 → 谐振路由选 attention 目标。

use crate::core::nt_core_hex::ReasoningHexagram;
use crate::core::nt_core_traits::SpecialistType;

use super::core::FileAbility;

/// 将 SpecialistType 映射到 default_specialist_states() 的索引。
/// `nt_core_gwt::resonance::default_specialist_states()` 按 SpecialistType 枚举
/// 顺序返回 14 个推理态 (PatternMatcher=0 ... EvidenceWeightedHypothesis=13)。
pub fn specialist_index(t: SpecialistType) -> usize {
    match t {
        SpecialistType::PatternMatcher => 0,
        SpecialistType::AnomalyDetector => 1,
        SpecialistType::KnowledgeRetriever => 2,
        SpecialistType::CodeAnalyzer => 3,
        SpecialistType::Planner => 4,
        SpecialistType::KnowledgeIntegrator => 5,
        SpecialistType::GoalPrioritizer => 6,
        SpecialistType::RiskAssessor => 7,
        SpecialistType::CreativityGenerator => 8,
        SpecialistType::ReflectionEngine => 9,
        SpecialistType::MetaCognitionAnalyst => 10,
        SpecialistType::AISecurity => 11,
        SpecialistType::ImageGenerator => 12,
        SpecialistType::EvidenceWeightedHypothesis => 13,
        SpecialistType::Orchestrator => 4, // 无专属谐振态, 借 Planner
    }
}

/// PDF 增强 salience 权重
pub fn pdf_enhance_salience(task_summary: &str) -> f64 {
    let lower = task_summary.to_lowercase();

    // 高 salience 关键词
    if lower.contains("pdf") && (lower.contains("增强") || lower.contains("enhance") || lower.contains("清晰")) {
        0.9
    } else if lower.contains("pdf") && lower.contains("图标") {
        0.85
    } else if lower.contains("pdf") && (lower.contains("超分") || lower.contains("super")) {
        0.8
    } else if lower.contains("pdf") {
        0.5
    } else {
        0.0
    }
}

/// GWT 谐振路由: 用当前 E8 状态与 14 个专家默认态计算谐振强度,
/// 选出 attention 应投给的专家 (winner-take-most by resonance_strength)。
///
/// 返回 (专家, 谐振强度 0..6, 该专家默认态)。
pub fn route_attention(e8_state: ReasoningHexagram, task_summary: Option<&str>) -> (SpecialistType, u32, ReasoningHexagram) {
    let states = crate::core::nt_core_gwt::resonance::default_specialist_states();
    let salience = task_summary.map(pdf_enhance_salience).unwrap_or(0.0);
    let mut best: Option<(SpecialistType, u32, ReasoningHexagram)> = None;
    for (idx, st) in states.iter().enumerate() {
        let base = e8_state.resonance_strength(st);
        let boosted = ((base as f64) * (1.0 + salience)) as u32;
        let t = specialist_index_inv(idx);
        if best.as_ref().is_none_or(|(_, s, _)| boosted > *s) {
            best = Some((t, boosted, *st));
        }
    }
    best.unwrap_or((SpecialistType::PatternMatcher, 0, ReasoningHexagram::new(0)))
}

/// 索引 → SpecialistType (specialist_index 逆映射)
pub fn specialist_index_inv(idx: usize) -> SpecialistType {
    match idx {
        0 => SpecialistType::PatternMatcher,
        1 => SpecialistType::AnomalyDetector,
        2 => SpecialistType::KnowledgeRetriever,
        3 => SpecialistType::CodeAnalyzer,
        4 => SpecialistType::Planner,
        5 => SpecialistType::KnowledgeIntegrator,
        6 => SpecialistType::GoalPrioritizer,
        7 => SpecialistType::RiskAssessor,
        8 => SpecialistType::CreativityGenerator,
        9 => SpecialistType::ReflectionEngine,
        10 => SpecialistType::MetaCognitionAnalyst,
        11 => SpecialistType::AISecurity,
        12 => SpecialistType::ImageGenerator,
        13 => SpecialistType::EvidenceWeightedHypothesis,
        _ => SpecialistType::Orchestrator,
    }
}

impl FileAbility {
    /// 当前 E8 状态对应的 GWT 注意力投递目标
    pub fn gwt_route(&self) -> (SpecialistType, u32, ReasoningHexagram) {
        route_attention(self.e8_state, None)
    }

    /// 该文件的静态专家偏好 (按文件大类映射)
    pub fn specialist(&self) -> SpecialistType {
        self.kind.specialist()
    }
}