use crate::core::nt_core_truth::{
    ach::AchEngine,
    bias_audit::{BiasAuditReport, BiasAuditor},
    disinfo_scan::{DisinfoScanReport, DisinfoScanner},
    emotion_tag::{EmotionAnalyzer, EmotionTag},
    fact_tier::{ClaimTierAssignment, FactTierAnalyzer},
    fallacy_filter::{FallacyFilter, FallacyReport},
    source_tri::{SourceRecord, SourceTriangulator, TriangulationResult},
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct TruthEstimate {
    pub claim: String,
    pub emotion_tag: EmotionTag,
    pub tier_assignment: ClaimTierAssignment,
    pub fallacy_report: FallacyReport,
    pub disinfo_report: DisinfoScanReport,
    pub triangulation: Option<TriangulationResult>,
    pub ach_summary: String,
    pub bias_report: BiasAuditReport,
    pub overall_confidence: f64,
    pub blocked: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct TruthPipelineConfig {
    pub min_sources: usize,
    pub max_hypotheses: usize,
    pub enable_blocking: bool,
    pub use_weakest_link: bool,
}

impl Default for TruthPipelineConfig {
    fn default() -> Self {
        Self {
            min_sources: 2,
            max_hypotheses: 6,
            enable_blocking: true,
            use_weakest_link: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TruthPipeline {
    pub config: TruthPipelineConfig,
    pub emotion: EmotionAnalyzer,
    pub tiers: FactTierAnalyzer,
    pub fallacies: FallacyFilter,
    pub disinfo: DisinfoScanner,
    pub sources: SourceTriangulator,
    pub ach: AchEngine,
    pub bias: BiasAuditor,
}

impl Default for TruthPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl TruthPipeline {
    pub fn new() -> Self {
        Self {
            config: TruthPipelineConfig::default(),
            emotion: EmotionAnalyzer::new(),
            tiers: FactTierAnalyzer::new(),
            fallacies: FallacyFilter::new(),
            disinfo: DisinfoScanner::new(),
            sources: SourceTriangulator::new(),
            ach: AchEngine::new(),
            bias: BiasAuditor::new(),
        }
    }

    pub fn evaluate(
        &mut self,
        claim: &str,
        source_desc: &str,
        related_texts: &[&str],
        hypotheses: &[(&str, &str)],
    ) -> TruthEstimate {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let emotion_tag = self.emotion.tag_text(claim, source_desc);
        let tier_assignment = self.tiers.assign_tier(claim, source_desc);
        let fallacy_report = self.fallacies.scan(claim);
        let disinfo_report = self.disinfo.scan_full(claim, related_texts);

        let blocked = self.config.enable_blocking && fallacy_report.blocked;

        let triangulation = if !blocked && !hypotheses.is_empty() {
            let source_records: Vec<SourceRecord> = related_texts
                .iter()
                .enumerate()
                .map(|(i, text)| {
                    self.sources.create_source_record(
                        &format!("src_{}", i),
                        &format!("Source {}", i),
                        text,
                        true,
                        i == 0,
                    )
                })
                .collect();

            let result = self.sources.triangulate(
                claim,
                &source_records,
                &disinfo_report.source_homology,
                &disinfo_report.narrative_conflicts,
                hypotheses.len().max(1),
                hypotheses.len(),
            );
            Some(result)
        } else {
            None
        };

        let ach_summary = if !blocked && !hypotheses.is_empty() {
            let evidence_items: Vec<&str> = related_texts.iter().map(|s| *s).collect();
            self.ach.run_ach(claim, hypotheses, &evidence_items)
        } else {
            "ACH skipped (blocked or no hypotheses)".into()
        };

        let conclusion = if blocked { "BLOCKED" } else { claim };
        let confidence = if blocked {
            0.0
        } else {
            tier_assignment.confidence_adjustment
        };
        let bias_report = self.bias.audit(
            emotion_tag.valence,
            hypotheses.len(),
            "unbiased truth",
            conclusion,
            confidence,
        );

        self.bias
            .record_snapshot(crate::core::nt_core_truth::bias_audit::ReasoningSnapshot {
                timestamp,
                emotional_valence: emotion_tag.valence,
                hypothesis_count: hypotheses.len(),
                evidence_sources: related_texts.iter().map(|s| s.to_string()).collect(),
                conclusion: conclusion.to_string(),
            });

        let overall_confidence = if blocked {
            0.0
        } else if let Some(ref t) = triangulation {
            t.confidence.overall
        } else {
            let disinfo_penalty = disinfo_report.overall_suspicion * 0.3;
            let bias_penalty = bias_report.overall_risk * 0.1;
            (tier_assignment.confidence_adjustment - disinfo_penalty - bias_penalty)
                .max(0.0)
                .min(1.0)
        };

        TruthEstimate {
            claim: claim.to_string(),
            emotion_tag,
            tier_assignment,
            fallacy_report,
            disinfo_report,
            triangulation,
            ach_summary,
            bias_report,
            overall_confidence,
            blocked,
            timestamp,
        }
    }

    pub fn quick_check(&mut self, claim: &str, source_desc: &str) -> TruthEstimate {
        self.evaluate(claim, source_desc, &[], &[])
    }

    pub fn summary(&self, estimate: &TruthEstimate) -> String {
        let mut s = String::new();

        if estimate.blocked {
            s.push_str(&format!("[BLOCKED] {} — ", estimate.claim));
        } else {
            s.push_str(&format!(
                "[conf:{:.0}%] {} — ",
                estimate.overall_confidence * 100.0,
                estimate.claim
            ));
        }

        s.push_str(&self.emotion.summarize(&estimate.emotion_tag));
        s.push_str(" | ");
        s.push_str(&format!(
            "tier:{}",
            estimate.tier_assignment.assigned_tier.label()
        ));
        s.push_str(" | ");
        s.push_str(&self.fallacies.summary(&estimate.fallacy_report));
        s.push_str(" | ");
        s.push_str(&self.disinfo.summary(&estimate.disinfo_report));

        if let Some(ref t) = estimate.triangulation {
            s.push_str(" | ");
            s.push_str(&self.sources.summary(t));
        }

        s.push_str(" | ");
        s.push_str(&self.bias.summary(&estimate.bias_report));

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_pipeline_clean_claim() {
        let mut pipeline = TruthPipeline::new();
        let estimate = pipeline.evaluate(
            "Water boils at 100 degrees Celsius at standard atmospheric pressure",
            "Published in peer-reviewed chemistry textbook",
            &["Multiple independent thermometers confirm 100°C boiling point at sea level"],
            &[(
                "physical_constants",
                "Physical constants are invariant under standard conditions",
            )],
        );
        assert!(!estimate.blocked);
        assert!(estimate.overall_confidence > 0.3);
        assert!(estimate.fallacy_report.hits.is_empty());
    }

    #[test]
    fn test_pipeline_blocks_manipulation() {
        let mut pipeline = TruthPipeline::new();
        let estimate = pipeline.evaluate(
            "This heartbreaking scandal will shock you! Think of the children — everyone knows this is true!",
            "Anonymous blog",
            &[],
            &[],
        );
        assert!(estimate.blocked);
        assert!((estimate.overall_confidence - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_quick_check() {
        let mut pipeline = TruthPipeline::new();
        let estimate = pipeline.quick_check("The chair is blue", "Direct observation");
        assert!(!estimate.blocked);
    }

    #[test]
    fn test_summary_format() {
        let mut pipeline = TruthPipeline::new();
        let estimate = pipeline.evaluate(
            "Test claim for summary",
            "test source",
            &["Related info"],
            &[("test_hypothesis", "Testing hypothesis")],
        );
        let summary = pipeline.summary(&estimate);
        assert!(!summary.is_empty());
        assert!(summary.contains("emotion"));
    }

    #[test]
    fn test_emotional_blocked_claim() {
        let mut pipeline = TruthPipeline::new();
        let estimate = pipeline.evaluate(
            "This is a catastrophe and an outrage!",
            "unknown",
            &[],
            &[("test", "test hypothesis")],
        );
        assert!(!estimate.blocked);
    }
}
