use crate::core::nt_core_experience::constellation::Constellation;
use crate::core::nt_core_experience::multi_timeline::ResearchTimeline;
/// CTI — Cross-Timeline Integrator
/// Merges optimal solutions across timelines, produces evolutionary predictions.

/// An integrated optimal solution from cross-timeline synthesis
#[derive(Debug, Clone)]
pub struct IntegratedSolution {
    pub id: String,
    pub title: String,
    pub description: String,
    pub source_timelines: Vec<String>,
    pub source_constellations: Vec<String>,
    pub confidence: f64,
    pub integration_score: f64,
    pub prediction_accuracy: f64,
    pub created_cycle: u64,
}

/// An evolutionary prediction derived from integrated solutions
#[derive(Debug, Clone)]
pub struct EvolutionPrediction {
    pub id: String,
    pub target: String,
    pub predicted_outcome: String,
    pub confidence: f64,
    pub basis_solutions: Vec<String>,
    pub verified: bool,
    pub actual_outcome: Option<String>,
}

/// Cross-Timeline Integrator
#[derive(Debug, Clone)]
pub struct CrossTimelineIntegrator {
    pub solutions: Vec<IntegratedSolution>,
    pub predictions: Vec<EvolutionPrediction>,
    /// Tracks prediction accuracy over time
    pub prediction_accuracy: f64,
    pub total_predictions: u64,
    pub correct_predictions: u64,
    cycle: u64,
}

impl CrossTimelineIntegrator {
    pub fn new() -> Self {
        Self {
            solutions: Vec::new(),
            predictions: Vec::new(),
            prediction_accuracy: 0.0,
            total_predictions: 0,
            correct_predictions: 0,
            cycle: 0,
        }
    }

    pub fn advance_cycle(&mut self) {
        self.cycle += 1;
    }

    /// Integrate a constellation into a solution
    pub fn integrate_constellation(
        &mut self,
        constellation: &Constellation,
        _all_timelines: &[ResearchTimeline],
    ) -> IntegratedSolution {
        let id = format!("sol_{}", self.solutions.len() + 1);
        let source_tls: Vec<String> = constellation
            .stars
            .iter()
            .flat_map(|s| s.source_timelines.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let source_consts = vec![constellation.id.clone()];

        let avg_confidence = constellation
            .stars
            .iter()
            .map(|s| s.confidence)
            .sum::<f64>()
            / constellation.stars.len().max(1) as f64;
        let intensity_factor = constellation.stars.iter().map(|s| s.intensity).sum::<f64>()
            / constellation.stars.len().max(1) as f64
            / 10.0;

        let description = constellation
            .stars
            .iter()
            .map(|s| s.description.clone())
            .collect::<Vec<_>>()
            .join("; ");

        let solution = IntegratedSolution {
            id,
            title: format!("Integrated: {}", constellation.name),
            description,
            source_timelines: source_tls,
            source_constellations: source_consts,
            confidence: avg_confidence,
            integration_score: (avg_confidence * 0.6 + intensity_factor * 0.4).clamp(0.0, 1.0),
            prediction_accuracy: 0.0,
            created_cycle: self.cycle,
        };
        self.solutions.push(solution.clone());
        solution
    }

    /// Generate an evolutionary prediction from an integrated solution
    pub fn generate_prediction(
        &mut self,
        solution: &IntegratedSolution,
        target: &str,
    ) -> EvolutionPrediction {
        self.total_predictions += 1;
        let id = format!("pred_{}", self.total_predictions);
        let predicted = format!(
            "Based on integrated solution '{}': {} will improve by {:.1}%",
            solution.title,
            target,
            solution.integration_score * 50.0 + 10.0,
        );
        let prediction = EvolutionPrediction {
            id,
            target: target.to_string(),
            predicted_outcome: predicted,
            confidence: solution.confidence * solution.integration_score,
            basis_solutions: vec![solution.id.clone()],
            verified: false,
            actual_outcome: None,
        };
        self.predictions.push(prediction.clone());
        prediction
    }

    /// Verify a prediction against actual outcome
    pub fn verify_prediction(
        &mut self,
        pred_id: &str,
        actual_outcome: &str,
        success: bool,
    ) -> bool {
        if let Some(pred) = self.predictions.iter_mut().find(|p| p.id == pred_id) {
            pred.verified = true;
            pred.actual_outcome = Some(actual_outcome.to_string());
            if success {
                self.correct_predictions += 1;
                // Boost confidence of basis solutions
                for sol_id in &pred.basis_solutions {
                    if let Some(sol) = self.solutions.iter_mut().find(|s| s.id == *sol_id) {
                        sol.confidence = (sol.confidence + 0.1).min(1.0);
                        sol.prediction_accuracy = (sol.prediction_accuracy + 1.0) / 2.0;
                    }
                }
            } else {
                // Penalize
                for sol_id in &pred.basis_solutions {
                    if let Some(sol) = self.solutions.iter_mut().find(|s| s.id == *sol_id) {
                        sol.confidence = (sol.confidence - 0.05).max(0.1);
                        sol.prediction_accuracy = sol.prediction_accuracy * 0.9;
                    }
                }
            }
            self.update_accuracy();
            true
        } else {
            false
        }
    }

    fn update_accuracy(&mut self) {
        self.prediction_accuracy = if self.total_predictions > 0 {
            self.correct_predictions as f64 / self.total_predictions as f64
        } else {
            0.0
        };
    }

    /// Get the highest-confidence integration for a domain
    pub fn best_solution_for_domain(&self, domain_keyword: &str) -> Option<&IntegratedSolution> {
        self.solutions
            .iter()
            .filter(|s| s.description.contains(domain_keyword) || s.title.contains(domain_keyword))
            .max_by(|a, b| {
                a.integration_score
                    .partial_cmp(&b.integration_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Merge multiple solutions into a higher-order synthesis
    pub fn synthesize(&mut self, solution_ids: &[String]) -> Option<IntegratedSolution> {
        if solution_ids.is_empty() {
            return None;
        }
        let base: Vec<&IntegratedSolution> = solution_ids
            .iter()
            .filter_map(|id| self.solutions.iter().find(|s| s.id == *id))
            .collect();
        if base.is_empty() || base.len() < 2 {
            return None;
        }
        let id = format!("sol_synth_{}", self.solutions.len() + 1);
        let all_tls: Vec<String> = base
            .iter()
            .flat_map(|s| s.source_timelines.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let all_consts: Vec<String> = base
            .iter()
            .flat_map(|s| s.source_constellations.clone())
            .collect();
        let avg_conf = base.iter().map(|s| s.confidence).sum::<f64>() / base.len() as f64;
        let avg_score = base.iter().map(|s| s.integration_score).sum::<f64>() / base.len() as f64;

        let desc_parts: Vec<&str> = base.iter().map(|s| s.description.as_str()).collect();
        let description = desc_parts.join(" | ");

        let solution = IntegratedSolution {
            id,
            title: format!("Synthesis of {} solutions", base.len()),
            description,
            source_timelines: all_tls,
            source_constellations: all_consts,
            confidence: avg_conf * 1.1,
            integration_score: avg_score * 1.05,
            prediction_accuracy: base.iter().map(|s| s.prediction_accuracy).sum::<f64>()
                / base.len() as f64,
            created_cycle: self.cycle,
        };
        self.solutions.push(solution.clone());
        Some(solution)
    }

    pub fn summary(&self) -> String {
        format!(
            "Integrator: {} solutions, {} predictions, {}/{} correct ({:.1}%)",
            self.solutions.len(),
            self.predictions.len(),
            self.correct_predictions,
            self.total_predictions,
            self.prediction_accuracy * 100.0,
        )
    }
}

impl Default for CrossTimelineIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_experience::constellation::Star;

    fn make_test_constellation(name: &str, stars: usize) -> Constellation {
        Constellation {
            id: format!("c_{}", name),
            name: name.to_string(),
            stars: (0..stars)
                .map(|i| Star {
                    id: format!("s_{}_{}", name, i),
                    title: format!("Star {}", i),
                    description: format!("insight about {} pattern number {}", name, i),
                    source_timelines: vec![format!("tl_{}", i)],
                    confidence: 0.7 + (i as f64 * 0.05),
                    first_seen_cycle: 0,
                    last_seen_cycle: 0,
                    intensity: 0.8,
                })
                .collect(),
            formation_cycle: 0,
            integrated: false,
            integration_cycle: None,
            emergence_score: 0.8,
        }
    }

    #[test]
    fn test_integrate_constellation() {
        let mut integrator = CrossTimelineIntegrator::new();
        let c = make_test_constellation("Emerge1", 3);
        let solution = integrator.integrate_constellation(&c, &[]);
        assert_eq!(integrator.solutions.len(), 1);
        assert!(solution.confidence > 0.0);
        assert!(solution.integration_score > 0.0);
    }

    #[test]
    fn test_generate_prediction() {
        let mut integrator = CrossTimelineIntegrator::new();
        let c = make_test_constellation("Pred", 2);
        let sol = integrator.integrate_constellation(&c, &[]);
        let pred = integrator.generate_prediction(&sol, "consciousness_cycle_accuracy");
        assert_eq!(integrator.predictions.len(), 1);
        assert!(pred.confidence > 0.0);
        assert!(pred
            .predicted_outcome
            .contains("consciousness_cycle_accuracy"));
    }

    #[test]
    fn test_verify_correct_prediction() {
        let mut integrator = CrossTimelineIntegrator::new();
        let c = make_test_constellation("Verify", 2);
        let sol = integrator.integrate_constellation(&c, &[]);
        let pred = integrator.generate_prediction(&sol, "test_metric");
        let conf_before = sol.confidence;
        assert!(integrator.verify_prediction(&pred.id, "improved by 20%", true));
        assert_eq!(integrator.correct_predictions, 1);
        let sol_after = integrator
            .solutions
            .iter()
            .find(|s| s.id == sol.id)
            .unwrap();
        assert!(
            sol_after.confidence >= conf_before,
            "confidence should increase after correct prediction"
        );
    }

    #[test]
    fn test_verify_wrong_prediction() {
        let mut integrator = CrossTimelineIntegrator::new();
        let c = make_test_constellation("Wrong", 2);
        let sol = integrator.integrate_constellation(&c, &[]);
        let conf_before = sol.confidence;
        let pred = integrator.generate_prediction(&sol, "test_metric");
        assert!(integrator.verify_prediction(&pred.id, "no improvement", false));
        assert_eq!(integrator.correct_predictions, 0);
        assert_eq!(integrator.total_predictions, 1);
        let sol_after = integrator
            .solutions
            .iter()
            .find(|s| s.id == sol.id)
            .unwrap();
        assert!(
            sol_after.confidence <= conf_before
                || (sol_after.confidence - conf_before).abs() < 0.06,
            "confidence should not increase after wrong prediction"
        );
    }

    #[test]
    fn test_best_solution_for_domain() {
        let mut integrator = CrossTimelineIntegrator::new();
        let c1 = make_test_constellation("AI", 3);
        let mut sol1 = integrator.integrate_constellation(&c1, &[]);
        sol1.integration_score = 0.9;
        integrator.solutions.push(sol1);
        let c2 = make_test_constellation("Cooking", 2);
        let mut sol2 = integrator.integrate_constellation(&c2, &[]);
        sol2.integration_score = 0.3;
        integrator.solutions.push(sol2);
        let best = integrator.best_solution_for_domain("AI");
        assert!(best.is_some());
        assert!(best.unwrap().title.contains("AI"));
    }

    #[test]
    fn test_synthesize_multiple_solutions() {
        let mut integrator = CrossTimelineIntegrator::new();
        let c1 = make_test_constellation("Alpha", 2);
        let s1 = integrator.integrate_constellation(&c1, &[]);
        let c2 = make_test_constellation("Beta", 2);
        let s2 = integrator.integrate_constellation(&c2, &[]);
        let synth = integrator.synthesize(&[s1.id.clone(), s2.id.clone()]);
        assert!(synth.is_some());
        let s = synth.unwrap();
        assert!(
            s.confidence > s1.confidence || s.confidence > s2.confidence,
            "synthesis should boost confidence"
        );
        assert_eq!(s.source_timelines.len(), 4);
    }

    #[test]
    fn test_synthesize_single_solution_returns_none() {
        let mut integrator = CrossTimelineIntegrator::new();
        let c = make_test_constellation("Single", 2);
        let s = integrator.integrate_constellation(&c, &[]);
        let synth = integrator.synthesize(&[s.id.clone()]);
        assert!(synth.is_none(), "need at least 2 solutions to synthesize");
    }

    #[test]
    fn test_synthesize_empty_returns_none() {
        let mut integrator = CrossTimelineIntegrator::new();
        assert!(integrator.synthesize(&[]).is_none());
    }

    #[test]
    fn test_prediction_accuracy_tracking() {
        let mut integrator = CrossTimelineIntegrator::new();
        let c = make_test_constellation("Acc", 2);
        let sol = integrator.integrate_constellation(&c, &[]);
        let p1 = integrator.generate_prediction(&sol, "m1");
        integrator.verify_prediction(&p1.id, "ok", true);
        let p2 = integrator.generate_prediction(&sol, "m2");
        integrator.verify_prediction(&p2.id, "fail", false);
        assert!(
            (integrator.prediction_accuracy - 0.5).abs() < 0.01,
            "1 correct out of 2 = 0.5"
        );
    }

    #[test]
    fn test_summary() {
        let integrator = CrossTimelineIntegrator::new();
        let s = integrator.summary();
        assert!(s.contains("solutions"));
        assert!(s.contains("predictions"));
    }

    #[test]
    fn test_multiple_integrations() {
        let mut integrator = CrossTimelineIntegrator::new();
        for i in 0..5 {
            let c = make_test_constellation(&format!("C{}", i), 2);
            integrator.integrate_constellation(&c, &[]);
        }
        assert_eq!(integrator.solutions.len(), 5);
    }
}
