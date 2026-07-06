//! EWHR-inspired viewpoint fusion engine.
//! Weighted consensus scoring with bull/bear case extraction.

use super::types::{AnalysisDepth, FusionResult, Viewpoint};

pub struct FusionEngine {
    pub consensus_threshold: f64,
    pub disagreement_threshold: f64,
}

impl Default for FusionEngine {
    fn default() -> Self {
        Self {
            consensus_threshold: 0.65,
            disagreement_threshold: 0.35,
        }
    }
}

impl FusionEngine {
    pub fn new(consensus_threshold: f64, disagreement_threshold: f64) -> Self {
        Self { consensus_threshold, disagreement_threshold }
    }

    pub fn fuse(&self, viewpoints: &[Viewpoint], depth: AnalysisDepth) -> FusionResult {
        if viewpoints.is_empty() {
            return FusionResult {
                consensus_score: 0.0,
                disagreement_score: 0.0,
                weighted_conclusion: String::new(),
                bull_case: String::new(),
                bear_case: String::new(),
                viewpoint_count: 0,
            };
        }

        let count = viewpoints.len() as f64;
        let total_confidence: f64 = viewpoints.iter().map(|v| v.confidence).sum();
        let avg_confidence = if total_confidence > 0.0 { total_confidence / count } else { 0.0 };

        let confidence_variance: f64 = viewpoints.iter()
            .map(|v| (v.confidence - avg_confidence).powi(2))
            .sum::<f64>() / count;

        let consensus_score = self.compute_consensus(viewpoints, avg_confidence, confidence_variance);
        let disagreement_score = self.compute_disagreement(viewpoints, confidence_variance);

        let (bull_case, bear_case) = if depth.requires_debate() {
            self.extract_bull_bear(viewpoints)
        } else {
            (String::new(), String::new())
        };

        let weighted_conclusion = self.compute_weighted_conclusion(viewpoints, consensus_score, disagreement_score, depth);

        FusionResult {
            consensus_score,
            disagreement_score,
            weighted_conclusion,
            bull_case,
            bear_case,
            viewpoint_count: viewpoints.len(),
        }
    }

    fn compute_consensus(&self, viewpoints: &[Viewpoint], avg_confidence: f64, variance: f64) -> f64 {
        let mut score = avg_confidence;
        if variance < 0.1 {
            score += 0.1;
        }
        if viewpoints.iter().all(|v| v.confidence >= 0.5) {
            score += 0.1;
        }
        score.max(0.0).min(1.0)
    }

    fn compute_disagreement(&self, viewpoints: &[Viewpoint], variance: f64) -> f64 {
        let high_low_ratio = if viewpoints.len() >= 2 {
            let max_conf = viewpoints.iter().map(|v| v.confidence).fold(0.0_f64, f64::max);
            let min_conf = viewpoints.iter().map(|v| v.confidence).fold(1.0_f64, f64::min);
            if min_conf > 0.0 { max_conf / min_conf } else { 1.0 }
        } else {
            1.0
        };
        let disagreement = variance * high_low_ratio * 0.5;
        disagreement.max(0.0).min(1.0)
    }

    fn extract_bull_bear(&self, viewpoints: &[Viewpoint]) -> (String, String) {
        let bullish: Vec<&Viewpoint> = viewpoints.iter().filter(|v| v.confidence >= 0.6).collect();
        let bearish: Vec<&Viewpoint> = viewpoints.iter().filter(|v| v.confidence <= 0.4).collect();

        let bull_case = if bullish.is_empty() {
            "No strongly bullish viewpoints.".into()
        } else {
            let mut text = String::new();
            for v in bullish.iter().take(3) {
                text.push_str(&format!("[{}:{}] {} ", v.perspective_label(), v.method_label(), v.analysis));
            }
            text
        };

        let bear_case = if bearish.is_empty() {
            "No strongly bearish viewpoints.".into()
        } else {
            let mut text = String::new();
            for v in bearish.iter().take(3) {
                text.push_str(&format!("[{}:{}] {} ", v.perspective_label(), v.method_label(), v.analysis));
            }
            text
        };

        (bull_case, bear_case)
    }

    fn compute_weighted_conclusion(&self, viewpoints: &[Viewpoint], consensus: f64, disagreement: f64, depth: AnalysisDepth) -> String {
        if consensus >= self.consensus_threshold {
            let mut reasoning = Vec::new();
            for v in viewpoints.iter().filter(|v| v.confidence >= consensus) {
                reasoning.push(format!("{} ({}): c={:.2}", v.label, v.perspective_label(), v.confidence));
            }
            format!("Consensus ({:.2}): viewpoints agree on direction. {}", consensus, reasoning.join("; "))
        } else if disagreement >= self.disagreement_threshold && depth.requires_debate() {
            let bullish_count = viewpoints.iter().filter(|v| v.confidence >= 0.6).count();
            let bearish_count = viewpoints.iter().filter(|v| v.confidence <= 0.4).count();
            format!(
                "Deep divergence: {} bullish vs {} bearish. Disagreement={:.2}. Requires further investigation.",
                bullish_count, bearish_count, disagreement
            )
        } else {
            let avg_conf = viewpoints.iter().map(|v| v.confidence).sum::<f64>() / viewpoints.len() as f64;
            format!("Mixed signals (avg c={:.2}): no strong consensus. Disagreement={:.2}. Recommend shallow investigation first.", avg_conf, disagreement)
        }
    }
}
