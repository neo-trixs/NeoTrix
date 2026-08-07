//! NT-CORE GWT 歧义 Park (D5) — 低置信度副作用提案进 park 区, 不直接执行。
//!
//! 参照: maka (不确定工具副作用 park) + hermes-dreaming (proposals 带 confidence)。
//! 机制: 副作用提案若置信度低于阈值, 不进工作区广播, 而是进 park 区暂存,
//! 等待更多证据或人工裁决。防未验证副作用直接执行。

use serde::{Deserialize, Serialize};

/// 副作用提案 — 描述一个可能执行的副作用动作。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffectProposal {
    pub id: String,
    /// 副作用描述 (如 "删除缓存", "写入文件 X")。
    pub description: String,
    /// 触发者模块。
    pub origin: String,
    /// 置信度 (0.0-1.0): 提案被验证/可确信的程度。
    pub confidence: f64,
    /// 是否可逆 (可逆副作用允许稍低阈值)。
    pub reversible: bool,
    /// 附加证据。
    pub evidence: Vec<String>,
}

impl SideEffectProposal {
    pub fn new(id: &str, description: &str, origin: &str, confidence: f64, reversible: bool) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            origin: origin.to_string(),
            confidence,
            reversible,
            evidence: Vec::new(),
        }
    }

    /// 该提案是否安全直接执行 (阈值 = 置信度 ≥ 门限)。
    pub fn safe_to_execute(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }
}

/// Park 决策。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ParkDecision {
    /// 直接执行 (高置信)
    Execute,
    /// 进入 park 区暂存
    Park,
    /// 执行但需后续复核 (中等置信 + 可逆)
    ExecuteWithReview,
}

/// 歧义 Park 区 — 低置信度副作用提案暂存地。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AmbiguityPark {
    pub parked: Vec<SideEffectProposal>,
    pub threshold: f64,
    pub review_threshold: f64,
}

impl AmbiguityPark {
    pub fn new(threshold: f64, review_threshold: f64) -> Self {
        Self {
            parked: Vec::new(),
            threshold,
            review_threshold,
        }
    }

    /// 提交提案 → 依置信度决策。
    pub fn submit(&mut self, p: SideEffectProposal) -> ParkDecision {
        if p.confidence >= self.threshold {
            ParkDecision::Execute
        } else if p.confidence >= self.review_threshold && p.reversible {
            ParkDecision::ExecuteWithReview
        } else {
            self.parked.push(p);
            ParkDecision::Park
        }
    }

    /// 检索 park 区中某一来源的待决提案。
    pub fn pending_by_origin(&self, origin: &str) -> Vec<&SideEffectProposal> {
        self.parked.iter().filter(|p| p.origin == origin).collect()
    }

    /// 补充证据后重新评估 park 区全部提案, 满足阈值的放行 (移出 park)。
    /// 返回被放行的提案数。
    pub fn reevaluate(&mut self) -> usize {
        let mut released = Vec::new();
        let mut kept = Vec::new();
        for p in self.parked.drain(..) {
            if p.confidence >= self.threshold {
                released.push(p);
            } else {
                kept.push(p);
            }
        }
        self.parked = kept;
        released.len()
    }

    /// 外部人工/证据给某提案加置信度。
    pub fn add_confidence(&mut self, id: &str, delta: f64) {
        for p in self.parked.iter_mut() {
            if p.id == id {
                p.confidence = (p.confidence + delta).clamp(0.0, 1.0);
                p.evidence.push(format!("+{:.2} confidence adjustment", delta));
            }
        }
    }
}

/// D5 决策门: 对单个提案返回是否直接执行, 附带决策。
pub fn decide(p: &SideEffectProposal, park: &mut AmbiguityPark) -> ParkDecision {
    park.submit(p.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_confidence_executes() {
        let mut park = AmbiguityPark::new(0.8, 0.5);
        let p = SideEffectProposal::new("p1", "run tests", "agent", 0.95, true);
        assert_eq!(park.submit(p), ParkDecision::Execute);
        assert!(park.parked.is_empty());
    }

    #[test]
    fn low_confidence_irreversible_parks() {
        let mut park = AmbiguityPark::new(0.8, 0.5);
        let p = SideEffectProposal::new("p2", "delete cache", "agent", 0.3, false);
        assert_eq!(park.submit(p), ParkDecision::Park);
        assert_eq!(park.parked.len(), 1);
        assert_eq!(park.pending_by_origin("agent").len(), 1);
    }

    #[test]
    fn mid_confidence_reversible_review() {
        let mut park = AmbiguityPark::new(0.8, 0.5);
        let p = SideEffectProposal::new("p3", "touch file", "agent", 0.6, true);
        assert_eq!(park.submit(p), ParkDecision::ExecuteWithReview);
        assert!(park.parked.is_empty());
    }

    #[test]
    fn confidence_threshold_reevaluate_releases() {
        let mut park = AmbiguityPark::new(0.8, 0.5);
        park.submit(SideEffectProposal::new("p4", "write doc", "agent", 0.2, false));
        assert_eq!(park.parked.len(), 1);
        // 补充证据 +0.7 → 0.9 ≥ 0.8 → 放行
        park.add_confidence("p4", 0.7);
        let released = park.reevaluate();
        assert_eq!(released, 1);
        assert!(park.parked.is_empty());
    }

    #[test]
    fn decide_gate_uses_same_threshold() {
        let mut park = AmbiguityPark::new(0.8, 0.5);
        let p = SideEffectProposal::new("p5", "clean tmp", "scheduler", 0.9, false);
        assert_eq!(decide(&p, &mut park), ParkDecision::Execute);
        let p2 = SideEffectProposal::new("p6", "clean tmp", "scheduler", 0.4, false);
        assert_eq!(decide(&p2, &mut park), ParkDecision::Park);
    }
}
