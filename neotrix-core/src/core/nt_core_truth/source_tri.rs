#![allow(dead_code)]
use crate::core::nt_core_truth::fact_tier::{FactTier, FactTierAnalyzer};

#[derive(Debug, Clone)]
pub struct SourceRecord {
    pub source_id: String,
    pub source_name: String,
    pub tier: FactTier,
    pub independence: f64,        // 0..1 how independent from other sources
    pub primary: bool,            // primary vs secondary source
    pub reliability_history: f64, // historical reliability 0..1
}

#[derive(Debug, Clone)]
pub struct ConfidenceScore {
    pub source_strength: f64, // A: based on tier + independence + history
    pub contradiction_resistance: f64, // B: based on conflict count
    pub completeness: f64,    // C: hypothesis space coverage
    pub overall: f64,
}

#[derive(Debug, Clone)]
pub struct TriangulationResult {
    pub claim: String,
    pub sources_consulted: Vec<String>,
    pub confidence: ConfidenceScore,
    pub contradictions: Vec<String>,
    pub unresolved_conflicts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SourceTriangulator {
    tier_analyzer: FactTierAnalyzer,
    min_sources_for_triangulation: usize,
}

impl Default for SourceTriangulator {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceTriangulator {
    pub fn new() -> Self {
        Self {
            tier_analyzer: FactTierAnalyzer::new(),
            min_sources_for_triangulation: 2,
        }
    }

    /// Evaluate source strength (A axis)
    pub fn evaluate_source_strength(&self, sources: &[SourceRecord]) -> f64 {
        if sources.is_empty() {
            return 0.0;
        }

        let mut total = 0.0;
        for src in sources {
            let tier_base = src.tier.confidence_base();
            let independent_bonus = src.independence * 0.1;
            let primary_bonus = if src.primary { 0.05 } else { 0.0 };
            let reliability = src.reliability_history * 0.1;
            total += (tier_base + independent_bonus + primary_bonus + reliability).min(1.0);
        }
        (total / sources.len() as f64).min(1.0)
    }

    /// Evaluate contradiction resistance (B axis)
    pub fn evaluate_contradiction_resistance(
        &self,
        contradictions: &[String],
        unresolved: &[String],
    ) -> f64 {
        let total_issues = (contradictions.len() + unresolved.len()) as f64;
        if total_issues == 0.0 {
            return 1.0;
        }

        // Each documented contradiction reduces score
        // Unresolved conflicts are worse than documented contradictions
        let resolved_penalty = contradictions.len() as f64 * 0.15;
        let unresolved_penalty = unresolved.len() as f64 * 0.30;

        (1.0 - resolved_penalty - unresolved_penalty).max(0.0)
    }

    /// Evaluate completeness (C axis)
    pub fn evaluate_completeness(&self, hypothesis_count: usize, covered_hypotheses: usize) -> f64 {
        if hypothesis_count == 0 {
            return 0.5;
        }
        (covered_hypotheses as f64 / hypothesis_count as f64).min(1.0)
    }

    /// Compute overall confidence: weakest-link principle by default
    pub fn compute_overall(&self, a: f64, b: f64, c: f64, use_min: bool) -> f64 {
        if use_min {
            a.min(b).min(c)
        } else {
            // Weighted: source strength most important
            a * 0.5 + b * 0.3 + c * 0.2
        }
    }

    /// Triangulate a claim across multiple sources
    pub fn triangulate(
        &self,
        claim: &str,
        sources: &[SourceRecord],
        contradictions: &[String],
        unresolved: &[String],
        hypothesis_count: usize,
        covered_hypotheses: usize,
    ) -> TriangulationResult {
        let a = self.evaluate_source_strength(sources);
        let b = self.evaluate_contradiction_resistance(contradictions, unresolved);
        let c = self.evaluate_completeness(hypothesis_count, covered_hypotheses);
        let overall = self.compute_overall(a, b, c, true);

        let source_names: Vec<String> = sources.iter().map(|s| s.source_id.clone()).collect();

        TriangulationResult {
            claim: claim.to_string(),
            sources_consulted: source_names,
            confidence: ConfidenceScore {
                source_strength: a,
                contradiction_resistance: b,
                completeness: c,
                overall,
            },
            contradictions: contradictions.to_vec(),
            unresolved_conflicts: unresolved.to_vec(),
        }
    }

    pub fn create_source_record(
        &self,
        id: &str,
        name: &str,
        description: &str,
        independent: bool,
        is_primary: bool,
    ) -> SourceRecord {
        let assignment = self.tier_analyzer.assign_tier(name, description);
        SourceRecord {
            source_id: id.to_string(),
            source_name: name.to_string(),
            tier: assignment.assigned_tier,
            independence: if independent { 0.9 } else { 0.3 },
            primary: is_primary,
            reliability_history: assignment.confidence_adjustment,
        }
    }

    pub fn summary(&self, result: &TriangulationResult) -> String {
        format!(
            "Triangulation: [A:{:.0} B:{:.0} C:{:.0} → {:.0}] | sources={} | conflicts={}",
            result.confidence.source_strength * 100.0,
            result.confidence.contradiction_resistance * 100.0,
            result.confidence.completeness * 100.0,
            result.confidence.overall * 100.0,
            result.sources_consulted.len(),
            result.contradictions.len() + result.unresolved_conflicts.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_source_low_confidence() {
        let t = SourceTriangulator::new();
        let sources =
            vec![t.create_source_record("src1", "blog post", "Personal blog", false, false)];
        let result = t.triangulate("test claim", &sources, &[], &[], 3, 1);
        assert!(result.confidence.overall < 0.5);
    }

    #[test]
    fn test_multiple_independent_sources_high_confidence() {
        let t = SourceTriangulator::new();
        let sources = vec![
            t.create_source_record(
                "pubmed1",
                "Clinical trial",
                "PubMed indexed, Nature",
                true,
                true,
            ),
            t.create_source_record(
                "pubmed2",
                "Replication study",
                "PubMed indexed, The Lancet",
                true,
                true,
            ),
            t.create_source_record(
                "gov_report",
                "Official report",
                "cdc.gov official publication",
                true,
                true,
            ),
        ];
        let result = t.triangulate("proven medical fact", &sources, &[], &[], 3, 3);
        assert!(result.confidence.source_strength > 0.5);
    }

    #[test]
    fn test_contradiction_penalty() {
        let t = SourceTriangulator::new();
        let b = t.evaluate_contradiction_resistance(
            &["Contradiction A vs B".to_string()],
            &["Unresolved conflict about X".to_string()],
        );
        assert!(b < 0.8);
        assert!(b >= 0.0);
    }

    #[test]
    fn test_weakest_link_principle() {
        let t = SourceTriangulator::new();
        let overall = t.compute_overall(0.9, 0.2, 0.9, true);
        assert!((overall - 0.2).abs() < 0.001);
    }
}
