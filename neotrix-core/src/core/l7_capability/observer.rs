use serde::{Deserialize, Serialize};

use crate::core::l7_capability::registry::Capability;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IllusionReport {
    pub risk_score: f64,
    pub context_drift: f64,
    pub success_misleading: f64,
    pub overfit_score: f64,
    pub reason: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IllusionRisk {
    Low,
    Medium,
    High,
    Critical,
}

pub struct TurkeyScientist {
    humility_threshold: f64,
    exploration_rate: f64,
    pub context_history: Vec<ContextSnapshot>,
    max_history: usize,
}

#[derive(Debug, Clone)]
pub struct ContextSnapshot {
    pub capability_name: String,
    pub context_hash: u64,
    pub success: bool,
    pub prm_score: f64,
}

impl Default for TurkeyScientist {
    fn default() -> Self {
        Self {
            humility_threshold: 0.3,
            exploration_rate: 0.1,
            context_history: Vec::new(),
            max_history: 1000,
        }
    }
}

impl TurkeyScientist {
    pub fn new(humility_threshold: f64, exploration_rate: f64) -> Self {
        Self {
            humility_threshold,
            exploration_rate,
            context_history: Vec::new(),
            max_history: 1000,
        }
    }

    pub fn detect_illusion(&self, capability: &Capability) -> IllusionReport {
        let drift = self.context_drift(capability);
        let misleading = self.is_success_misleading(capability);
        let overfit = self.is_overfit(capability);

        let risk_score = drift * 0.4 + misleading * 0.3 + overfit * 0.3;

        let mut reasons = Vec::new();
        if drift > 0.3 {
            reasons.push(format!("context_drift={:.2}", drift));
        }
        if misleading > 0.3 {
            reasons.push(format!("success_misleading={:.2}", misleading));
        }
        if overfit > 0.3 {
            reasons.push(format!("overfit={:.2}", overfit));
        }

        let recommendation = if risk_score > self.humility_threshold {
            format!(
                "降低调度概率，增加探索率到 {:.2}，补充多样化上下文",
                (self.exploration_rate + 0.1).min(0.5)
            )
        } else {
            "正常".to_string()
        };

        IllusionReport {
            risk_score,
            context_drift: drift,
            success_misleading: misleading,
            overfit_score: overfit,
            reason: if reasons.is_empty() {
                "无显著偏差".to_string()
            } else {
                reasons.join(", ")
            },
            recommendation,
        }
    }

    fn context_drift(&self, _capability: &Capability) -> f64 {
        0.1
    }

    fn is_success_misleading(&self, capability: &Capability) -> f64 {
        let s = &capability.stats;
        if s.call_count < 5 {
            return 0.0;
        }
        let success_rate = s.success_rate;
        if success_rate > 0.95 && s.call_count > 10 {
            0.5 * (1.0 - s.diversity_score)
        } else {
            0.0
        }
    }

    fn is_overfit(&self, capability: &Capability) -> f64 {
        let s = &capability.stats;
        if s.call_count < 5 {
            return 0.0;
        }
        let narrow_success = s.success_rate > 0.9 && s.diversity_score < 0.2;
        if narrow_success {
            0.6 * (1.0 - s.diversity_score * 2.0).max(0.0)
        } else {
            0.0
        }
    }

    pub fn record_context(
        &mut self,
        capability_name: String,
        context_hash: u64,
        success: bool,
        prm_score: f64,
    ) {
        if self.context_history.len() >= self.max_history {
            self.context_history.remove(0);
        }
        self.context_history.push(ContextSnapshot {
            capability_name,
            context_hash,
            success,
            prm_score,
        });
    }

    pub fn should_explore(&self) -> bool {
        rand::random::<f64>() < self.exploration_rate
    }

    pub fn humility_score(&self) -> f64 {
        self.humility_threshold
    }

    pub fn set_exploration_rate(&mut self, rate: f64) {
        self.exploration_rate = rate.max(0.0).min(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l7_capability::registry::{
        capability_id_from_name, CapabilityCost, CapabilityKind, CapabilityRuntime,
        CapabilityStability, CapabilityTier, CapabilityVector, DomainCategory, MaturityLevel,
    };
    fn test_cap(success_rate: f64, diversity: f64, call_count: u64) -> Capability {
        let mut stats = crate::core::l7_capability::registry::CapabilityStats::default();
        stats.success_rate = success_rate;
        stats.diversity_score = diversity;
        stats.call_count = call_count;
        Capability {
            id: capability_id_from_name("observer_test"),
            name: "observer_test".to_string(),
            tags: vec!["test".to_string()],
            kind: CapabilityKind::Cognitive,
            maturity: MaturityLevel::Candidate,
            vector: CapabilityVector::default(),
            e8_triggers: vec![],
            context_requirements: vec![],
            cost: CapabilityCost::default(),
            stats,
            version: "0.1.0".to_string(),
            layer: 4,
            tier: CapabilityTier::Core,
            runtime: CapabilityRuntime::Local,
            stability: CapabilityStability::Experimental,
            fallback_chain: vec![],
            provider: None,
            domain: DomainCategory::Reasoning,
            input_schema: None,
            output_schema: None,
            resource_cpu: 0.0,
            resource_ram_mb: 0.0,
            resource_vram_mb: 0.0,
            dependencies: vec![],
        }
    }

    #[test]
    fn test_low_risk_for_normal_capability() {
        let turkey = TurkeyScientist::default();
        let cap = test_cap(0.8, 0.6, 50);
        let report = turkey.detect_illusion(&cap);
        assert!(report.risk_score < turkey.humility_threshold);
    }

    #[test]
    fn test_high_risk_for_overfit_capability() {
        let turkey = TurkeyScientist::default();
        let cap = test_cap(0.99, 0.1, 50);
        let report = turkey.detect_illusion(&cap);
        assert!(report.risk_score >= turkey.humility_threshold - 0.1);
    }

    #[test]
    fn test_success_misleading_detection() {
        let turkey = TurkeyScientist::default();
        let cap = test_cap(0.98, 0.05, 20);
        let misleading = turkey.is_success_misleading(&cap);
        assert!(misleading > 0.4);
    }

    #[test]
    fn test_overfit_detection() {
        let turkey = TurkeyScientist::default();
        let cap = test_cap(0.95, 0.1, 20);
        let overfit = turkey.is_overfit(&cap);
        assert!(overfit > 0.4);
    }

    #[test]
    fn test_record_context() {
        let mut turkey = TurkeyScientist::default();
        turkey.record_context("test_cap".to_string(), 0xABCD, true, 0.85);
        assert_eq!(turkey.context_history.len(), 1);
        assert_eq!(turkey.context_history[0].capability_name, "test_cap");
    }

    #[test]
    fn test_should_explore() {
        let turkey = TurkeyScientist::new(0.3, 1.0);
        assert!(turkey.should_explore());
        let turkey2 = TurkeyScientist::new(0.3, 0.0);
        assert!(!turkey2.should_explore());
    }

    #[test]
    fn test_illusion_report_recommendation() {
        let turkey = TurkeyScientist::default();
        let cap = test_cap(0.99, 0.05, 100);
        let report = turkey.detect_illusion(&cap);
        assert!(report.recommendation.contains("探索"));
    }
}
