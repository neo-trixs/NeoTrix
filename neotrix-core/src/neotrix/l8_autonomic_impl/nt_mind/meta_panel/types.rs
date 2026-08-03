use serde::{Deserialize, Serialize};

use crate::core::nt_core_gate::{GateDecision, GuardrailReport, PanelVerdict};
use crate::core::nt_core_self_review::SelfReviewReport;
use crate::neotrix::l8_autonomic_impl::nt_mind::reasoning_types::{PerspectiveLens, ReasoningMethod};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnalysisDepth {
    Lite,
    Mid,
    Deep,
}

impl AnalysisDepth {
    pub fn viewpoint_count(&self) -> usize {
        match self {
            AnalysisDepth::Lite => 3,
            AnalysisDepth::Mid => 12,
            AnalysisDepth::Deep => 24,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AnalysisDepth::Lite => "快速扫描: 3个E8视角, 无辩论, 自检门快速",
            AnalysisDepth::Mid => "标准分析: 12个视角, EWHR加权融合, 自检门完整",
            AnalysisDepth::Deep => "深度研究: 24个视角, Bull/Bear辩论, 完整自检+修正",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Lite, Self::Mid, Self::Deep]
    }

    pub fn requires_debate(&self) -> bool {
        matches!(self, AnalysisDepth::Deep)
    }

    pub fn requires_fusion(&self) -> bool {
        !matches!(self, AnalysisDepth::Lite)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewpoint {
    pub id: String,
    pub label: String,
    pub perspective: PerspectiveLens,
    pub method: ReasoningMethod,
    pub analysis: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

impl Viewpoint {
    pub fn new(
        id: String,
        label: String,
        perspective: PerspectiveLens,
        method: ReasoningMethod,
        analysis: String,
        confidence: f64,
        evidence: Vec<String>,
    ) -> Self {
        Self { id, label, perspective, method, analysis, confidence, evidence }
    }

    pub fn perspective_label(&self) -> &'static str {
        match self.perspective {
            PerspectiveLens::Builder => "Builder",
            PerspectiveLens::Architect => "Architect",
            PerspectiveLens::Skeptic => "Skeptic",
            PerspectiveLens::User => "User",
            PerspectiveLens::Economist => "Economist",
            PerspectiveLens::Historian => "Historian",
            PerspectiveLens::Contrarian => "Contrarian",
            PerspectiveLens::Ethicist => "Ethicist",
        }
    }

    pub fn method_label(&self) -> &'static str {
        match self.method {
            ReasoningMethod::Direct => "Direct",
            ReasoningMethod::FirstPrinciples => "FirstPrinciples",
            ReasoningMethod::Adversarial => "Adversarial",
            ReasoningMethod::EdgeCaseFocus => "EdgeCaseFocus",
            ReasoningMethod::ConstraintPropagation => "ConstraintPropagation",
            ReasoningMethod::ReverseEngineering => "ReverseEngineering",
            ReasoningMethod::HistoricalEmpirical => "HistoricalEmpirical",
            ReasoningMethod::Analogical => "Analogical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionResult {
    pub consensus_score: f64,
    pub disagreement_score: f64,
    pub weighted_conclusion: String,
    pub bull_case: String,
    pub bear_case: String,
    pub viewpoint_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaPanelResult {
    pub question: String,
    pub depth: AnalysisDepth,
    pub viewpoints: Vec<Viewpoint>,
    pub fusion: Option<FusionResult>,
    pub conclusion: String,
    pub review_passed: bool,
    pub review_report: SelfReviewReport,
    /// 多家族评审组裁决 (nt_core_gate) — 高分歧/低分不自动放行
    pub panel_verdict: Option<PanelVerdict>,
    /// eval 护栏 (grounding/schema/幻觉隔离)
    pub guardrail: Option<GuardrailReport>,
    /// 爆炸半径分级组合裁决 (确定性检查优先于 LLM 分数)
    pub gate_decision: Option<GateDecision>,
}

impl MetaPanelResult {
    pub fn summary(&self) -> String {
        format!(
            "MetaPanel[depth={:?}]: {} viewpoints, consensus={:.2}, disagreement={:.2}, review={}",
            self.depth,
            self.viewpoints.len(),
            self.fusion.as_ref().map(|f| f.consensus_score).unwrap_or(0.0),
            self.fusion.as_ref().map(|f| f.disagreement_score).unwrap_or(0.0),
            if self.review_passed { "PASS" } else { "FAIL" }
        )
    }
}
