//! MetaPanelEngine — orchestrates viewpoint generation, fusion, and self-review.

use std::collections::HashMap;

use uuid::Uuid;

use crate::core::nt_core_gate::{
    Claim, GuardrailReport, JudgeFamily, JudgeInput, JudgePanel, ToolSpec, GateDecision,
};
use crate::core::nt_core_self_review::{SelfReviewGate, Severity};
use crate::neotrix::l8_autonomic_impl::nt_mind::reasoning_types::{PerspectiveLens, ReasoningMethod};

use super::fusion::FusionEngine;
use super::types::{AnalysisDepth, FusionResult, MetaPanelResult, Viewpoint};

pub struct MetaPanelEngine {
    pub depth: AnalysisDepth,
    pub strict_review: bool,
    pub viewpoint_registry: HashMap<(PerspectiveLens, ReasoningMethod), String>,
    /// 动作路径工具清单 — 爆炸半径分级的输入 (构建期注册, 非运行期猜测)
    pub tools: Vec<ToolSpec>,
}

impl Default for MetaPanelEngine {
    fn default() -> Self {
        Self::new(AnalysisDepth::Mid, true)
    }
}

impl MetaPanelEngine {
    pub fn new(depth: AnalysisDepth, strict_review: bool) -> Self {
        let mut registry = HashMap::new();
        for p in PerspectiveLens::all() {
            for m in ReasoningMethod::all() {
                registry.insert((p, m), format!("{:?}_{:?}", p, m));
            }
        }
        Self { depth, strict_review, viewpoint_registry: registry, tools: Vec::new() }
    }

    pub fn set_depth(&mut self, depth: AnalysisDepth) {
        self.depth = depth;
    }

    /// 注册动作路径工具 (不可逆/扩权工具会升级门控到人工)。
    pub fn set_tools(&mut self, tools: Vec<ToolSpec>) {
        self.tools = tools;
    }

    pub fn analyze(&self, question: &str) -> MetaPanelResult {
        let mut gate = SelfReviewGate::new(self.strict_review);

        let viewpoints = self.generate_viewpoints(question);
        gate.check(
            !viewpoints.is_empty(),
            Severity::Error,
            "viewpoint_gen",
            format!("Viewpoint generation produced {} viewpoints for: {}", viewpoints.len(), question),
            file!(),
            line!(),
        );

        let fusion = if self.depth.requires_fusion() {
            let fusion_engine = FusionEngine::default();
            let result = fusion_engine.fuse(&viewpoints, self.depth);
            gate.check(
                result.consensus_score > 0.0,
                Severity::Warning,
                "fusion_quality",
                format!("Fusion consensus={:.2}, disagreement={:.2}", result.consensus_score, result.disagreement_score),
                file!(),
                line!(),
            );
            Some(result)
        } else {
            None
        };

        let conclusion = self.build_conclusion(&viewpoints, &fusion);

        let review_report = gate.report();

        // ── 门控接线 (nt_core_gate): 多家族评审组 + eval 护栏 + 爆炸半径分级 ──
        let evidence_ids: Vec<String> = viewpoints.iter().flat_map(|v| v.evidence.iter().cloned()).collect();
        let claims: Vec<Claim> = viewpoints.iter().map(|v| Claim {
            text: v.analysis.clone(),
            evidence_refs: v.evidence.clone(),
        }).collect();
        let input = JudgeInput {
            candidate: conclusion.clone(),
            claims,
            evidence_ids,
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
            rubric: None,
            samples: 1,
            attestation: None,
        };
        let panel = JudgePanel::default_panel();
        let panel_verdict = panel.run(&input);
        let guardrail = GuardrailReport::evaluate(&input, &panel.debias);
        let gate_decision = GateDecision::decide(&self.tools, &input, &panel);

        let review_passed = review_report.is_pass()
            && panel_verdict.is_pass()
            && guardrail.action == crate::core::nt_core_gate::GuardAction::Allow
            && gate_decision.allows_autonomous();

        MetaPanelResult {
            question: question.to_string(),
            depth: self.depth,
            viewpoints,
            fusion,
            conclusion,
            review_passed,
            review_report,
            panel_verdict: Some(panel_verdict),
            guardrail: Some(guardrail),
            gate_decision: Some(gate_decision),
        }
    }

    fn generate_viewpoints(&self, question: &str) -> Vec<Viewpoint> {
        let count = self.depth.viewpoint_count();
        let all_perspectives = PerspectiveLens::all();
        let all_methods = ReasoningMethod::all();
        let mut viewpoints = Vec::with_capacity(count);

        let mut idx = 0usize;
        while viewpoints.len() < count {
            let p = all_perspectives[idx % all_perspectives.len()];
            let m = all_methods[(idx / all_perspectives.len()) % all_methods.len()];
            idx += 1;

            let vp = self.build_viewpoint(question, p, m, idx);
            viewpoints.push(vp);
        }

        viewpoints
    }

    fn build_viewpoint(&self, question: &str, perspective: PerspectiveLens, method: ReasoningMethod, seed: usize) -> Viewpoint {
        let id = Uuid::new_v4().to_string();
        let label = format!("{:?}+{:?}", perspective, method);
        let confidence = self.compute_confidence(method, perspective, seed);
        let evidence = self.generate_evidence(question, perspective, method, seed);
        let analysis = self.synthesize_analysis(question, perspective, method, confidence, &evidence);

        Viewpoint::new(id, label, perspective, method, analysis, confidence, evidence)
    }

    fn compute_confidence(&self, method: ReasoningMethod, perspective: PerspectiveLens, seed: usize) -> f64 {
        let base = match method {
            ReasoningMethod::Direct => 0.8,
            ReasoningMethod::FirstPrinciples => 0.7,
            ReasoningMethod::Adversarial => 0.6,
            ReasoningMethod::EdgeCaseFocus => 0.5,
            ReasoningMethod::ConstraintPropagation => 0.6,
            ReasoningMethod::ReverseEngineering => 0.7,
            ReasoningMethod::HistoricalEmpirical => 0.6,
            ReasoningMethod::Analogical => 0.5,
        };
        let perspective_mod = match perspective {
            PerspectiveLens::Builder => 0.1,
            PerspectiveLens::Architect => 0.1,
            PerspectiveLens::Skeptic => -0.1,
            PerspectiveLens::User => 0.0,
            PerspectiveLens::Economist => 0.0,
            PerspectiveLens::Historian => -0.1,
            PerspectiveLens::Contrarian => -0.2,
            PerspectiveLens::Ethicist => 0.0,
        };
        let raw = base + perspective_mod + (seed as f64 * 0.01).sin() * 0.1;
        raw.max(0.1).min(1.0)
    }

    fn generate_evidence(&self, question: &str, perspective: PerspectiveLens, method: ReasoningMethod, _seed: usize) -> Vec<String> {
        let mut ev = Vec::new();
        ev.push(format!("Question: {}", question));
        ev.push(format!("Perspective: {} — {}", perspective_label_str(perspective), perspective_desc_str(perspective)));
        ev.push(format!("Method: {} — {}", method_label_str(method), method_desc_str(method)));
        ev
    }

    fn synthesize_analysis(&self, _question: &str, perspective: PerspectiveLens, method: ReasoningMethod, confidence: f64, evidence: &[String]) -> String {
        format!(
            "[{}][{}] c={:.2} ev={} — Simulated: {} via {}",
            perspective_label_str(perspective),
            method_label_str(method),
            confidence,
            evidence.len(),
            perspective_desc_str(perspective),
            method_desc_str(method),
        )
    }

    fn build_conclusion(&self, viewpoints: &[Viewpoint], fusion: &Option<FusionResult>) -> String {
        match fusion {
            Some(f) => {
                if f.consensus_score >= 0.65 {
                    format!("Consensus ({:.2}): {}", f.consensus_score, f.weighted_conclusion)
                } else if f.disagreement_score > 0.35 {
                    format!("Divergent ({:.2}): Bull case — {} Bear case — {}", f.disagreement_score, f.bull_case, f.bear_case)
                } else {
                    format!("Mixed (c={:.2} d={:.2}): {}", f.consensus_score, f.disagreement_score, f.weighted_conclusion)
                }
            }
            None => {
                if viewpoints.is_empty() {
                    "No viewpoints generated.".into()
                } else {
                    let top = &viewpoints[0];
                    format!("Quick scan: Top viewpoint [{:?}+{:?}] c={:.2}", top.perspective, top.method, top.confidence)
                }
            }
        }
    }
}

fn perspective_label_str(p: PerspectiveLens) -> &'static str {
    match p {
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

fn perspective_desc_str(p: PerspectiveLens) -> &'static str {
    p.description()
}

fn method_label_str(m: ReasoningMethod) -> &'static str {
    match m {
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

fn method_desc_str(m: ReasoningMethod) -> &'static str {
    m.description()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_lite() {
        let engine = MetaPanelEngine::new(AnalysisDepth::Lite, false);
        let result = engine.analyze("Should we adopt a microservices architecture?");
        assert_eq!(result.depth, AnalysisDepth::Lite);
        assert_eq!(result.viewpoints.len(), 3);
        assert!(result.fusion.is_none());
        assert!(!result.conclusion.is_empty());
        assert!(result.review_passed);
    }

    #[test]
    fn test_analyze_mid() {
        let engine = MetaPanelEngine::new(AnalysisDepth::Mid, true);
        let result = engine.analyze("Evaluate the performance of our caching layer.");
        assert_eq!(result.depth, AnalysisDepth::Mid);
        assert_eq!(result.viewpoints.len(), 12);
        assert!(result.fusion.is_some());
        let f = result.fusion.as_ref().unwrap();
        assert!(f.consensus_score > 0.0);
        assert!(f.viewpoint_count == 12);
        assert!(result.review_passed);
    }

    #[test]
    fn test_analyze_deep() {
        let engine = MetaPanelEngine::new(AnalysisDepth::Deep, true);
        let result = engine.analyze("Should we rewrite the legacy payment system?");
        assert_eq!(result.viewpoints.len(), 24);
        let f = result.fusion.as_ref().unwrap();
        assert!(!f.bull_case.is_empty() || !f.bear_case.is_empty());
    }

    #[test]
    fn test_fusion_default() {
        let engine = MetaPanelEngine::new(AnalysisDepth::Mid, false);
        let result = engine.analyze("What is the best error handling strategy?");
        let f = result.fusion.unwrap();
        assert!(f.consensus_score >= 0.0);
        assert!(f.consensus_score <= 1.0);
        assert!(f.disagreement_score >= 0.0);
        assert!(f.disagreement_score <= 1.0);
        assert!(!f.weighted_conclusion.is_empty());
    }

    #[test]
    fn test_self_review_integration() {
        let engine = MetaPanelEngine::new(AnalysisDepth::Mid, true);
        let result = engine.analyze("Is our CI/CD pipeline robust enough?");
        assert!(result.review_passed);
        assert!(result.review_report.is_pass());
        let has_viewpoint_check = result.review_report.findings.iter().any(|f| f.category == "viewpoint_gen");
        assert!(!has_viewpoint_check, "viewpoint_gen should pass (no finding) for normal input");
        let has_fusion_check = result.review_report.findings.iter().any(|f| f.category == "fusion_quality");
        assert!(!has_fusion_check, "fusion_quality should pass for normal input");
    }

    #[test]
    fn test_self_review_no_findings_for_normal_input() {
        let engine = MetaPanelEngine::new(AnalysisDepth::Lite, false);
        let result = engine.analyze("test");
        assert!(result.review_report.findings.is_empty(), "should have zero findings: {:?}", result.review_report.findings);
        assert_eq!(result.review_report.passed, 0);
        assert_eq!(result.review_report.failed, 0);
    }

    #[test]
    fn test_conclusion_variation() {
        let engine = MetaPanelEngine::new(AnalysisDepth::Lite, false);
        let result = engine.analyze("Quick question");
        assert!(result.conclusion.starts_with("Quick scan") || result.conclusion == "No viewpoints generated.");
        assert!(!result.viewpoints.is_empty());
    }

    #[test]
    fn test_summary_format() {
        let engine = MetaPanelEngine::new(AnalysisDepth::Mid, false);
        let result = engine.analyze("Test summary");
        let summary = result.summary();
        assert!(summary.contains("MetaPanel"));
        assert!(summary.contains("viewpoints"));
        assert!(summary.contains("PASS") || summary.contains("FAIL"));
    }

    #[test]
    fn test_fusion_engine_direct() {
        let fusion = FusionEngine::default();
        let vp = Viewpoint::new("id1".into(), "test".into(),
            PerspectiveLens::Builder, ReasoningMethod::Direct,
            "analysis".into(), 0.8, vec!["ev".into()]);
        let result = fusion.fuse(&[vp], AnalysisDepth::Mid);
        assert!(result.consensus_score > 0.5);
        assert_eq!(result.viewpoint_count, 1);
    }

    #[test]
    fn test_fusion_engine_empty() {
        let fusion = FusionEngine::default();
        let result = fusion.fuse(&[], AnalysisDepth::Deep);
        assert_eq!(result.consensus_score, 0.0);
        assert_eq!(result.viewpoint_count, 0);
    }
}
