use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub enum HypothesisType {
    Causal,
    Correlational,
    Counterfactual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterventionScope {
    Hyperparameter,
    Architecture,
    Data,
    TrainingProcedure,
    MemoryPolicy,
}

#[derive(Debug, Clone)]
pub struct InterventionHypothesis {
    pub id: u64,
    pub description: String,
    pub hypothesis_type: HypothesisType,
    pub confidence: f64,
    pub target_observable: String,
    pub expected_effect: String,
    pub expected_direction: &'static str,
    pub scope: InterventionScope,
    pub estimated_effort: &'static str,
}

#[derive(Debug, Clone)]
pub struct CausalLink {
    pub cause: String,
    pub effect: String,
    pub strength: f64,
    pub lag_steps: usize,
    pub evidence_count: u32,
}

pub struct InterventionHypothesisGenerator {
    pub causal_links: Vec<CausalLink>,
    pub hypotheses: Vec<InterventionHypothesis>,
    pub counter: u64,
}

impl InterventionHypothesisGenerator {
    pub fn new() -> Self {
        Self {
            causal_links: Vec::new(),
            hypotheses: Vec::new(),
            counter: 0,
        }
    }

    pub fn record_causal_link(&mut self, cause: &str, effect: &str, strength: f64, lag: usize) {
        let strength = strength.clamp(0.0, 1.0);
        if let Some(existing) = self
            .causal_links
            .iter_mut()
            .find(|l| l.cause == cause && l.effect == effect)
        {
            existing.strength = strength;
            existing.lag_steps = lag;
            existing.evidence_count += 1;
        } else {
            self.causal_links.push(CausalLink {
                cause: cause.to_string(),
                effect: effect.to_string(),
                strength,
                lag_steps: lag,
                evidence_count: 1,
            });
        }
    }

    pub fn generate_hypotheses(
        &mut self,
        observables: &[(&str, f64)],
        n: usize,
    ) -> Vec<InterventionHypothesis> {
        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for link in &self.causal_links {
            if results.len() >= n {
                break;
            }
            let key = format!("causal:{}=>{}", link.cause, link.effect);
            if seen.insert(key) {
                let h = self.generate_causal_hypothesis(&link.cause, &link.effect, link.strength);
                results.push(h);
            }
        }

        if results.len() < n {
            for i in 0..observables.len() {
                if results.len() >= n {
                    break;
                }
                for j in (i + 1)..observables.len() {
                    if results.len() >= n {
                        break;
                    }
                    let key = format!("corr:{}=>{}", observables[i].0, observables[j].0);
                    if seen.insert(key) {
                        let corr = if observables[i].1 > 0.0 && observables[j].1 > 0.0 {
                            0.5
                        } else {
                            0.3
                        };
                        let h = self.generate_correlational_hypothesis(
                            observables[i].0,
                            observables[j].0,
                            corr,
                        );
                        results.push(h);
                    }
                }
            }
        }

        if results.len() < n {
            for obs in observables {
                if results.len() >= n {
                    break;
                }
                let key = format!("cf:{}", obs.0);
                if seen.insert(key) {
                    let target = if obs.1 > 0.5 { 0.1 } else { 0.9 };
                    let h = self.generate_counterfactual(obs.0, obs.1, target);
                    results.push(h);
                }
            }
        }

        for h in &results {
            self.hypotheses.push(h.clone());
        }

        results
    }

    pub fn generate_causal_hypothesis(
        &self,
        cause: &str,
        effect: &str,
        strength: f64,
    ) -> InterventionHypothesis {
        let id = self.counter.wrapping_add(1);
        let confidence = strength * 0.9 + 0.1;
        InterventionHypothesis {
            id,
            description: format!("Changing '{}' will affect '{}'", cause, effect),
            hypothesis_type: HypothesisType::Causal,
            confidence: confidence.clamp(0.0, 1.0),
            target_observable: effect.to_string(),
            expected_effect: format!("Adjust {}", cause),
            expected_direction: if strength > 0.5 {
                "increase"
            } else {
                "decrease"
            },
            scope: InterventionScope::Hyperparameter,
            estimated_effort: "medium",
        }
    }

    pub fn generate_correlational_hypothesis(
        &self,
        obs_a: &str,
        obs_b: &str,
        correlation: f64,
    ) -> InterventionHypothesis {
        let id = self.counter.wrapping_add(2);
        let confidence = correlation.abs() * 0.7 + 0.15;
        InterventionHypothesis {
            id,
            description: format!("'{}' and '{}' move together", obs_a, obs_b),
            hypothesis_type: HypothesisType::Correlational,
            confidence: confidence.clamp(0.0, 1.0),
            target_observable: obs_b.to_string(),
            expected_effect: format!("Monitor {}", obs_a),
            expected_direction: if correlation > 0.0 {
                "increase"
            } else {
                "decrease"
            },
            scope: InterventionScope::Data,
            estimated_effort: "low",
        }
    }

    pub fn generate_counterfactual(
        &self,
        observable: &str,
        current_value: f64,
        target_value: f64,
    ) -> InterventionHypothesis {
        let id = self.counter.wrapping_add(3);
        let delta = (target_value - current_value).abs();
        let confidence = (delta / (current_value.abs() + 1.0)).min(1.0) * 0.6 + 0.2;
        let direction = if target_value > current_value {
            "increase"
        } else {
            "decrease"
        };
        InterventionHypothesis {
            id,
            description: format!(
                "If '{}' were {:.3} instead of {:.3}",
                observable, target_value, current_value
            ),
            hypothesis_type: HypothesisType::Counterfactual,
            confidence: confidence.clamp(0.0, 1.0),
            target_observable: observable.to_string(),
            expected_effect: format!("Shift from {:.3} to {:.3}", current_value, target_value),
            expected_direction: direction,
            scope: InterventionScope::TrainingProcedure,
            estimated_effort: "high",
        }
    }

    pub fn strongest_causal_link(&self, effect: &str) -> Option<&CausalLink> {
        self.causal_links
            .iter()
            .filter(|l| l.effect == effect)
            .max_by(|a, b| {
                a.strength
                    .partial_cmp(&b.strength)
                    .unwrap_or(Ordering::Equal)
            })
    }

    pub fn rank_by_confidence<'a>(
        &self,
        candidates: &'a [InterventionHypothesis],
    ) -> Vec<&'a InterventionHypothesis> {
        let mut sorted: Vec<_> = candidates.iter().collect();
        sorted.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(Ordering::Equal)
        });
        sorted
    }

    pub fn rank_by_effort<'a>(
        &self,
        candidates: &'a [InterventionHypothesis],
    ) -> Vec<&'a InterventionHypothesis> {
        let mut sorted: Vec<_> = candidates.iter().collect();
        sorted.sort_by(|a, b| {
            let a_val = effort_value(a.estimated_effort);
            let b_val = effort_value(b.estimated_effort);
            a_val.cmp(&b_val)
        });
        sorted
    }

    pub fn by_scope(&self, scope: InterventionScope) -> Vec<&InterventionHypothesis> {
        self.hypotheses
            .iter()
            .filter(|h| h.scope == scope)
            .collect()
    }
}

fn effort_value(effort: &str) -> u8 {
    match effort {
        "low" => 0,
        "medium" => 1,
        "high" => 2,
        _ => 1,
    }
}

pub struct InterventionPlan {
    pub hypotheses: Vec<InterventionHypothesis>,
    pub expected_improvement: f64,
    pub total_effort: &'static str,
}

impl InterventionPlan {
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "InterventionPlan: {} hypotheses, expected improvement {:.3}, total effort: {}\n",
            self.hypotheses.len(),
            self.expected_improvement,
            self.total_effort
        ));
        for h in &self.hypotheses {
            s.push_str(&format!(
                "  [{}] {} (conf={:.3}, dir={}, effort={})\n",
                h.id, h.description, h.confidence, h.expected_direction, h.estimated_effort
            ));
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_generator_empty() {
        let gen = InterventionHypothesisGenerator::new();
        assert!(gen.causal_links.is_empty());
        assert!(gen.hypotheses.is_empty());
        assert_eq!(gen.counter, 0);
    }

    #[test]
    fn test_record_causal_link() {
        let mut gen = InterventionHypothesisGenerator::new();
        gen.record_causal_link("lr", "loss", 0.8, 3);
        assert_eq!(gen.causal_links.len(), 1);
        assert_eq!(gen.causal_links[0].cause, "lr");
        assert_eq!(gen.causal_links[0].effect, "loss");
        assert!((gen.causal_links[0].strength - 0.8).abs() < 1e-6);
        assert_eq!(gen.causal_links[0].lag_steps, 3);
        assert_eq!(gen.causal_links[0].evidence_count, 1);
    }

    #[test]
    fn test_record_causal_link_updates_existing() {
        let mut gen = InterventionHypothesisGenerator::new();
        gen.record_causal_link("lr", "loss", 0.8, 3);
        gen.record_causal_link("lr", "loss", 0.9, 2);
        assert_eq!(gen.causal_links.len(), 1);
        assert!((gen.causal_links[0].strength - 0.9).abs() < 1e-6);
        assert_eq!(gen.causal_links[0].evidence_count, 2);
    }

    #[test]
    fn test_record_causal_link_clamps_strength() {
        let mut gen = InterventionHypothesisGenerator::new();
        gen.record_causal_link("x", "y", 1.5, 0);
        assert!((gen.causal_links[0].strength - 1.0).abs() < 1e-6);
        gen.record_causal_link("a", "b", -0.5, 0);
        assert!((gen.causal_links[1].strength - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_generate_causal_hypothesis() {
        let gen = InterventionHypothesisGenerator::new();
        let h = gen.generate_causal_hypothesis("lr", "loss", 0.75);
        assert_eq!(h.hypothesis_type, HypothesisType::Causal);
        assert!(h.confidence >= 0.1 && h.confidence <= 1.0);
        assert_eq!(h.target_observable, "loss");
    }

    #[test]
    fn test_generate_correlational_hypothesis() {
        let gen = InterventionHypothesisGenerator::new();
        let h = gen.generate_correlational_hypothesis("grad_norm", "loss", 0.6);
        assert_eq!(h.hypothesis_type, HypothesisType::Correlational);
        assert_eq!(h.estimated_effort, "low");
    }

    #[test]
    fn test_generate_counterfactual() {
        let gen = InterventionHypothesisGenerator::new();
        let h = gen.generate_counterfactual("accuracy", 0.45, 0.85);
        assert_eq!(h.hypothesis_type, HypothesisType::Counterfactual);
        assert_eq!(h.estimated_effort, "high");
        assert_eq!(h.expected_direction, "increase");
    }

    #[test]
    fn test_strongest_causal_link() {
        let mut gen = InterventionHypothesisGenerator::new();
        gen.record_causal_link("lr", "loss", 0.5, 1);
        gen.record_causal_link("batch_size", "loss", 0.9, 2);
        let strongest = gen.strongest_causal_link("loss");
        assert!(strongest.is_some());
        assert_eq!(strongest.unwrap().cause, "batch_size");
    }

    #[test]
    fn test_generate_hypotheses_from_observables() {
        let mut gen = InterventionHypothesisGenerator::new();
        gen.record_causal_link("lr", "loss", 0.8, 3);
        let obs = vec![("loss", 0.5), ("accuracy", 0.7)];
        let results = gen.generate_hypotheses(&obs, 3);
        assert!(!results.is_empty());
        assert!(results.len() <= 3);
    }

    #[test]
    fn test_rank_by_confidence() {
        let gen = InterventionHypothesisGenerator::new();
        let mut h1 = gen.generate_causal_hypothesis("a", "b", 0.9);
        h1.confidence = 0.3;
        let mut h2 = gen.generate_causal_hypothesis("c", "d", 0.9);
        h2.confidence = 0.8;
        let mut h3 = gen.generate_causal_hypothesis("e", "f", 0.9);
        h3.confidence = 0.5;
        let candidates = vec![h1, h2, h3];
        let ranked = gen.rank_by_confidence(&candidates);
        assert!(ranked[0].confidence >= ranked[1].confidence);
        assert!(ranked[1].confidence >= ranked[2].confidence);
    }

    #[test]
    fn test_rank_by_effort() {
        let gen = InterventionHypothesisGenerator::new();
        let mut h1 = gen.generate_causal_hypothesis("a", "b", 0.5);
        h1.estimated_effort = "high";
        let mut h2 = gen.generate_causal_hypothesis("c", "d", 0.5);
        h2.estimated_effort = "low";
        let candidates = vec![h1, h2];
        let ranked = gen.rank_by_effort(&candidates);
        assert_eq!(ranked[0].estimated_effort, "low");
        assert_eq!(ranked[1].estimated_effort, "high");
    }

    #[test]
    fn test_by_scope() {
        let mut gen = InterventionHypothesisGenerator::new();
        let mut h1 = gen.generate_causal_hypothesis("a", "b", 0.5);
        h1.scope = InterventionScope::Architecture;
        gen.hypotheses.push(h1);
        let mut h2 = gen.generate_causal_hypothesis("c", "d", 0.5);
        h2.scope = InterventionScope::Data;
        gen.hypotheses.push(h2);
        let arch_h = gen.by_scope(InterventionScope::Architecture);
        assert_eq!(arch_h.len(), 1);
        let data_h = gen.by_scope(InterventionScope::Data);
        assert_eq!(data_h.len(), 1);
    }

    #[test]
    fn test_intervention_plan_summary() {
        let gen = InterventionHypothesisGenerator::new();
        let h = gen.generate_causal_hypothesis("lr", "loss", 0.8);
        let plan = InterventionPlan {
            hypotheses: vec![h],
            expected_improvement: 0.25,
            total_effort: "medium",
        };
        let s = plan.summary();
        assert!(s.contains("InterventionPlan:"));
        assert!(s.contains("expected improvement 0.250"));
        assert!(s.contains("total effort: medium"));
    }

    #[test]
    fn test_hypothesis_type_equality() {
        assert_eq!(HypothesisType::Causal, HypothesisType::Causal);
        assert_ne!(HypothesisType::Causal, HypothesisType::Counterfactual);
    }

    #[test]
    fn test_strongest_causal_link_nonexistent() {
        let gen = InterventionHypothesisGenerator::new();
        assert!(gen.strongest_causal_link("nonexistent").is_none());
    }
}
