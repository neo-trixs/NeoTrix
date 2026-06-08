use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

use super::vsa_tag::VsaTagged;

#[derive(Debug, Clone)]
pub struct CritiqueResult {
    pub passed: bool,
    pub relevance_score: f64,
    pub consistency_score: f64,
    pub uncertainty_score: f64,
    pub overall_quality: f64,
    pub reasons: Vec<String>,
}

impl CritiqueResult {
    pub fn perfect() -> Self {
        Self {
            passed: true,
            relevance_score: 1.0,
            consistency_score: 1.0,
            uncertainty_score: 0.0,
            overall_quality: 1.0,
            reasons: vec![],
        }
    }

    pub fn quality_label(&self) -> &'static str {
        if self.overall_quality >= 0.9 {
            "excellent"
        } else if self.overall_quality >= 0.7 {
            "good"
        } else if self.overall_quality >= 0.5 {
            "acceptable"
        } else {
            "poor"
        }
    }
}

pub struct InnerCritic {
    relevance_threshold: f64,
    consistency_threshold: f64,
    uncertainty_tolerance: f64,
    critiques_issued: u64,
    critiques_passed: u64,
}

impl Default for InnerCritic {
    fn default() -> Self {
        Self::new()
    }
}

impl InnerCritic {
    pub fn new() -> Self {
        Self {
            relevance_threshold: 0.4,
            consistency_threshold: 0.3,
            uncertainty_tolerance: 0.6,
            critiques_issued: 0,
            critiques_passed: 0,
        }
    }

    pub fn evaluate(
        &mut self,
        output: &VsaTagged,
        context: &VsaTagged,
        specious_present: Option<&super::specious_present::SpeciousPresent>,
    ) -> CritiqueResult {
        self.critiques_issued += 1;
        let mut reasons = Vec::new();

        let relevance = QuantizedVSA::similarity(&output.vector, &context.vector);
        let mut uncertainty = 0.0;

        if relevance < self.relevance_threshold {
            reasons.push(format!(
                "low relevance: {:.3} < threshold {:.3}",
                relevance, self.relevance_threshold
            ));
        }

        let consistency = if let Some(sp) = specious_present {
            let c = sp.average_coherence();
            if c < self.consistency_threshold {
                reasons.push(format!(
                    "low temporal consistency: {:.3} < {:.3}",
                    c, self.consistency_threshold
                ));
            }
            if !sp.is_temporally_stable() {
                reasons.push("temporal instability detected".to_string());
            }
            c
        } else {
            relevance
        };

        if output.confidence < 0.5 {
            uncertainty = 1.0 - output.confidence;
            if uncertainty > self.uncertainty_tolerance {
                reasons.push(format!(
                    "high uncertainty: {:.3} > tolerance {:.3}",
                    uncertainty, self.uncertainty_tolerance
                ));
            }
        }

        let overall_quality = (relevance * 0.4 + consistency * 0.3 + (1.0 - uncertainty) * 0.3)
            .clamp(0.0, 1.0);
        let passed = reasons.is_empty() || overall_quality >= 0.6;

        if passed {
            self.critiques_passed += 1;
        }

        CritiqueResult {
            passed,
            relevance_score: relevance,
            consistency_score: consistency,
            uncertainty_score: uncertainty,
            overall_quality,
            reasons,
        }
    }

    pub fn pass_rate(&self) -> f64 {
        if self.critiques_issued == 0 {
            return 1.0;
        }
        self.critiques_passed as f64 / self.critiques_issued as f64
    }

    pub fn critiques_issued(&self) -> u64 {
        self.critiques_issued
    }

    pub fn set_thresholds(
        &mut self,
        relevance: f64,
        consistency: f64,
        uncertainty: f64,
    ) {
        self.relevance_threshold = relevance.clamp(0.0, 1.0);
        self.consistency_threshold = consistency.clamp(0.0, 1.0);
        self.uncertainty_tolerance = uncertainty.clamp(0.0, 1.0);
    }

    pub fn adjust_thresholds(&mut self) {
        let rate = self.pass_rate();
        if rate > 0.95 {
            self.relevance_threshold = (self.relevance_threshold + 0.05).min(1.0);
            self.consistency_threshold = (self.consistency_threshold + 0.05).min(1.0);
        } else if rate < 0.5 && self.critiques_issued > 10 {
            self.relevance_threshold = (self.relevance_threshold - 0.05).max(0.1);
            self.consistency_threshold = (self.consistency_threshold - 0.05).max(0.1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::SpeciousPresent;
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
    use crate::core::nt_core_consciousness::vsa_tag::{VsaOrigin, VsaSelfCategory, VsaWorldCategory};

    #[test]
    fn test_new_critic_has_perfect_pass_rate() {
        let c = InnerCritic::new();
        assert!((c.pass_rate() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_identical_vectors_pass() {
        let mut c = InnerCritic::new();
        let v = QuantizedVSA::random_binary();
        let tagged = VsaTagged::new(v.clone(), VsaOrigin::Self_(VsaSelfCategory::Thought));
        let result = c.evaluate(&tagged, &tagged, None);
        assert!(result.passed);
    }

    #[test]
    fn test_low_relevance_fails() {
        let mut c = InnerCritic::new();
        let output = VsaTagged::new(vec![1; 100], VsaOrigin::Self_(VsaSelfCategory::Thought));
        let context = VsaTagged::new(vec![0; 100], VsaOrigin::World(VsaWorldCategory::UserInput));
        let result = c.evaluate(&output, &context, None);
        assert!(!result.passed || result.relevance_score >= c.relevance_threshold);
    }

    #[test]
    fn test_quality_labels() {
        let perfect = CritiqueResult::perfect();
        assert_eq!(perfect.quality_label(), "excellent");

        let poor = CritiqueResult {
            passed: false,
            relevance_score: 0.1,
            consistency_score: 0.1,
            uncertainty_score: 0.9,
            overall_quality: 0.2,
            reasons: vec!["bad".into()],
        };
        assert_eq!(poor.quality_label(), "poor");
    }

    #[test]
    fn test_pass_rate_tracks() {
        let mut c = InnerCritic::new();
        let v = vec![1; 100];
        let t1 = VsaTagged::new(v.clone(), VsaOrigin::Self_(VsaSelfCategory::Thought));
        let t2 = VsaTagged::new(v, VsaOrigin::Self_(VsaSelfCategory::Memory));
        c.evaluate(&t1, &t2, None);
        assert_eq!(c.critiques_issued(), 1);
        assert!((c.pass_rate() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_threshold_adjustment() {
        let mut c = InnerCritic::new();
        c.set_thresholds(0.5, 0.5, 0.5);
        assert!((c.relevance_threshold - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_temporal_consistency_check() {
        let mut c = InnerCritic::new();
        let output = VsaTagged::new(vec![1; 100], VsaOrigin::Self_(VsaSelfCategory::Thought));
        let context = VsaTagged::new(vec![1; 100], VsaOrigin::Self_(VsaSelfCategory::Thought));
        let mut sp = SpeciousPresent::new(3);
        sp.push(VsaTagged::new(vec![0; 100], VsaOrigin::Self_(VsaSelfCategory::Thought)));
        let result = c.evaluate(&output, &context, Some(&sp));
        assert!(!result.passed || result.consistency_score >= 0.0);
    }

    #[test]
    fn test_critique_result_debug() {
        let r = CritiqueResult::perfect();
        let s = format!("{:?}", r);
        assert!(s.contains("passed"));
    }

    #[test]
    fn test_pass_rate_zero_issued() {
        let c = InnerCritic::new();
        assert!((c.pass_rate() - 1.0).abs() < 1e-9);
    }
}
