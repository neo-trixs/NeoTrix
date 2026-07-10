#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbductiveExplanation {
    pub hypothesis: String,
    pub plausibility: f64,
    pub supporting_evidence: Vec<String>,
    pub contradictory_evidence: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbductiveSolver {
    pub explanations: Vec<AbductiveExplanation>,
    pub max_explanations: usize,
    pub min_plausibility: f64,
}

impl AbductiveSolver {
    pub fn new(max_explanations: usize, min_plausibility: f64) -> Self {
        Self {
            explanations: Vec::new(),
            max_explanations,
            min_plausibility: min_plausibility.max(0.0).min(1.0),
        }
    }

    pub fn solve(&mut self, observation: &str, context: &[String]) -> Vec<AbductiveExplanation> {
        let tokens: Vec<&str> = observation
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|w| !w.is_empty())
            .collect();

        let mut generated = Vec::new();

        for token in &tokens {
            if generated.len() >= self.max_explanations {
                break;
            }
            let hypothesis = format!("{} could cause: {}", token, observation);
            let plausibility = 1.0 / tokens.len() as f64;
            if plausibility >= self.min_plausibility {
                generated.push(AbductiveExplanation {
                    hypothesis,
                    plausibility,
                    supporting_evidence: vec![format!("token '{}' present in observation", token)],
                    contradictory_evidence: Vec::new(),
                    confidence: plausibility,
                });
            }
        }

        for ctx in context {
            if generated.len() >= self.max_explanations {
                break;
            }
            let overlap = tokens
                .iter()
                .filter(|t| ctx.to_lowercase().contains(&t.to_lowercase()))
                .count();
            let plausibility = if !tokens.is_empty() {
                (overlap as f64 / tokens.len() as f64).max(0.0).min(1.0)
            } else {
                0.1
            };
            if plausibility >= self.min_plausibility {
                generated.push(AbductiveExplanation {
                    hypothesis: format!("context '{}' explains: {}", ctx, observation),
                    plausibility,
                    supporting_evidence: vec![format!(
                        "context overlaps with {} of {} tokens",
                        overlap,
                        tokens.len()
                    )],
                    contradictory_evidence: if overlap == 0 {
                        vec!["no token overlap with context".to_string()]
                    } else {
                        Vec::new()
                    },
                    confidence: plausibility,
                });
            }
        }

        if generated.is_empty() {
            generated.push(AbductiveExplanation {
                hypothesis: format!("default explanation for: {}", observation),
                plausibility: 0.1,
                supporting_evidence: vec!["fallback explanation".to_string()],
                contradictory_evidence: vec!["no strong evidence".to_string()],
                confidence: 0.1,
            });
        }

        self.explanations = generated.clone();
        generated
    }

    pub fn rank_explanations(&self) -> Vec<&AbductiveExplanation> {
        let mut ranked: Vec<&AbductiveExplanation> = self.explanations.iter().collect();
        ranked.sort_by(|a, b| {
            b.plausibility
                .partial_cmp(&a.plausibility)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked
    }

    pub fn best_explanation(&self) -> Option<&AbductiveExplanation> {
        self.explanations
            .iter()
            .max_by(|a, b| {
                a.plausibility
                    .partial_cmp(&b.plausibility)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn resolve_conflicts(
        &mut self,
        a: &AbductiveExplanation,
        b: &AbductiveExplanation,
    ) -> Option<AbductiveExplanation> {
        let merged_plausibility = ((a.plausibility + b.plausibility) / 2.0).max(0.0).min(1.0);

        let mut merged_supporting = a.supporting_evidence.clone();
        for ev in &b.supporting_evidence {
            if !merged_supporting.contains(ev) {
                merged_supporting.push(ev.clone());
            }
        }

        let merged_contradictions: Vec<String> = a
            .contradictory_evidence
            .iter()
            .filter(|c| b.contradictory_evidence.contains(c))
            .cloned()
            .collect();

        let merged_confidence = ((a.confidence + b.confidence) / 2.0).max(0.0).min(1.0);

        Some(AbductiveExplanation {
            hypothesis: format!("compromise: {} | {}", a.hypothesis, b.hypothesis),
            plausibility: merged_plausibility,
            supporting_evidence: merged_supporting,
            contradictory_evidence: merged_contradictions,
            confidence: merged_confidence,
        })
    }
}

impl Default for AbductiveSolver {
    fn default() -> Self {
        Self::new(5, 0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_creation() {
        let solver = AbductiveSolver::new(10, 0.2);
        assert_eq!(solver.max_explanations, 10);
        assert!((solver.min_plausibility - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_solve_from_tokens() {
        let mut solver = AbductiveSolver::new(5, 0.1);
        let result = solver.solve("error occurred", &[]);
        assert!(!result.is_empty());
        assert!(result.iter().any(|e| e.hypothesis.contains("error")));
    }

    #[test]
    fn test_solve_with_context() {
        let mut solver = AbductiveSolver::new(5, 0.0);
        let ctx = vec!["system error".to_string(), "normal operation".to_string()];
        let result = solver.solve("error", &ctx);
        let has_context_hypothesis = result.iter().any(|e| e.hypothesis.contains("context"));
        assert!(has_context_hypothesis);
    }

    #[test]
    fn test_solve_fallback() {
        let mut solver = AbductiveSolver::new(3, 0.5);
        let ctx = vec!["unrelated".to_string()];
        let result = solver.solve("completely different text", &ctx);
        assert_eq!(result.len(), 1);
        assert!((result[0].plausibility - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_rank_explanations() {
        let mut solver = AbductiveSolver::new(10, 0.0);
        solver.explanations = vec![
            AbductiveExplanation {
                hypothesis: "low".to_string(),
                plausibility: 0.2,
                supporting_evidence: vec![],
                contradictory_evidence: vec![],
                confidence: 0.2,
            },
            AbductiveExplanation {
                hypothesis: "high".to_string(),
                plausibility: 0.9,
                supporting_evidence: vec![],
                contradictory_evidence: vec![],
                confidence: 0.9,
            },
        ];
        let ranked = solver.rank_explanations();
        assert_eq!(ranked[0].hypothesis, "high");
        assert_eq!(ranked[1].hypothesis, "low");
    }

    #[test]
    fn test_best_explanation() {
        let mut solver = AbductiveSolver::new(10, 0.0);
        solver.explanations = vec![
            AbductiveExplanation {
                hypothesis: "weak".to_string(),
                plausibility: 0.3,
                supporting_evidence: vec![],
                contradictory_evidence: vec![],
                confidence: 0.3,
            },
            AbductiveExplanation {
                hypothesis: "strong".to_string(),
                plausibility: 0.95,
                supporting_evidence: vec![],
                contradictory_evidence: vec![],
                confidence: 0.95,
            },
        ];
        let best = solver.best_explanation();
        assert!(best.is_some());
        assert_eq!(best.unwrap().hypothesis, "strong");
    }

    #[test]
    fn test_best_explanation_empty() {
        let solver = AbductiveSolver::new(5, 0.1);
        assert!(solver.best_explanation().is_none());
    }

    #[test]
    fn test_resolve_conflicts() {
        let mut solver = AbductiveSolver::new(5, 0.1);
        let a = AbductiveExplanation {
            hypothesis: "cause A".to_string(),
            plausibility: 0.7,
            supporting_evidence: vec!["evidence A1".to_string()],
            contradictory_evidence: vec!["against A".to_string()],
            confidence: 0.7,
        };
        let b = AbductiveExplanation {
            hypothesis: "cause B".to_string(),
            plausibility: 0.6,
            supporting_evidence: vec!["evidence B1".to_string()],
            contradictory_evidence: vec!["against A".to_string()],
            confidence: 0.6,
        };
        let merged = solver.resolve_conflicts(&a, &b);
        assert!(merged.is_some());
        let m = merged.unwrap();
        assert!((m.plausibility - 0.65).abs() < 1e-6);
        assert_eq!(m.supporting_evidence.len(), 2);
        assert_eq!(m.contradictory_evidence.len(), 1);
    }

    #[test]
    fn test_resolve_conflicts_no_common_contradiction() {
        let mut solver = AbductiveSolver::new(5, 0.1);
        let a = AbductiveExplanation {
            hypothesis: "A".to_string(),
            plausibility: 0.5,
            supporting_evidence: vec![],
            contradictory_evidence: vec!["a1".to_string()],
            confidence: 0.5,
        };
        let b = AbductiveExplanation {
            hypothesis: "B".to_string(),
            plausibility: 0.5,
            supporting_evidence: vec![],
            contradictory_evidence: vec!["b1".to_string()],
            confidence: 0.5,
        };
        let merged = solver.resolve_conflicts(&a, &b);
        assert!(merged.is_some());
        assert!(merged.unwrap().contradictory_evidence.is_empty());
    }

    #[test]
    fn test_default_solver() {
        let solver = AbductiveSolver::default();
        assert_eq!(solver.max_explanations, 5);
        assert!((solver.min_plausibility - 0.1).abs() < 1e-6);
    }
}
