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

/// GWT 谐振路由: 用当前 E8 状态与 14 个专家默认态计算谐振强度,
/// 选出 attention 应投给的专家 (winner-take-most by resonance_strength)。
///
/// 返回 (专家, 谐振强度 0..6, 该专家默认态)。
pub fn route_attention(e8_state: ReasoningHexagram) -> (SpecialistType, u32, ReasoningHexagram) {
    let states = crate::core::nt_core_gwt::resonance::default_specialist_states();
    let mut best: Option<(SpecialistType, u32, ReasoningHexagram)> = None;
    for (idx, st) in states.iter().enumerate() {
        let strength = e8_state.resonance_strength(st);
        let t = specialist_index_inv(idx);
        if best.as_ref().is_none_or(|(_, s, _)| strength > *s) {
            best = Some((t, strength, *st));
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
        route_attention(self.e8_state)
    }

    /// 该文件的静态专家偏好 (按文件大类映射)
    pub fn specialist(&self) -> SpecialistType {
        self.kind.specialist()
    }
}