#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::abductive_solver::AbductiveSolver;
use super::reflection_head::ReflectionHead;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionResult {
    pub explanation: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub contradictions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusReport {
    pub converged: bool,
    pub iterations: u32,
    pub confidence: f64,
    pub explanations: Vec<String>,
    pub conflicts_resolved: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub max_iterations: usize,
    pub convergence_threshold: f64,
    pub min_heads_agreed: usize,
    pub enable_backward_reflection: bool,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            convergence_threshold: 0.7,
            min_heads_agreed: 3,
            enable_backward_reflection: true,
        }
    }
}

pub struct ReflectionPipeline {
    pub heads: Vec<ReflectionHead>,
    pub solver: AbductiveSolver,
    pub config: ConsensusConfig,
}

impl ReflectionPipeline {
    pub fn new(config: ConsensusConfig) -> Self {
        Self {
            heads: Vec::new(),
            solver: AbductiveSolver::default(),
            config,
        }
    }

    pub fn add_head(&mut self, head: ReflectionHead) {
        self.heads.push(head);
    }

    pub fn process(&mut self, observation: &str) -> ReflectionResult {
        let mut combined_evidence = Vec::new();
        let mut combined_contradictions = Vec::new();
        let mut total_confidence = 0.0;
        let mut explanations = Vec::new();

        for head in &self.heads {
            let fwd = head.forward_reflect(observation);
            total_confidence += fwd.confidence;
            for ev in &fwd.supporting_evidence {
                if !combined_evidence.contains(ev) {
                    combined_evidence.push(ev.clone());
                }
            }
            for ct in &fwd.contradictions {
                if !combined_contradictions.contains(ct) {
                    combined_contradictions.push(ct.clone());
                }
            }
            explanations.push(fwd.conclusion.clone());

            if self.config.enable_backward_reflection {
                let bwd = head.backward_reflect(&fwd.conclusion);
                total_confidence += bwd.confidence * 0.5;
                for ev in &bwd.supporting_evidence {
                    if !combined_evidence.contains(ev) {
                        combined_evidence.push(ev.clone());
                    }
                }
                for ct in &bwd.contradictions {
                    if !combined_contradictions.contains(ct) {
                        combined_contradictions.push(ct.clone());
                    }
                }
            }
        }

        let avg_confidence = if !self.heads.is_empty() {
            let divisor = if self.config.enable_backward_reflection {
                self.heads.len() as f64 * 1.5
            } else {
                self.heads.len() as f64
            };
            (total_confidence / divisor).max(0.0).min(1.0)
        } else {
            0.0
        };

        let context = explanations.clone();
        let solved = self.solver.solve(observation, &context);
        let best = solved.first();

        let (explanation, conf) = match best {
            Some(e) => (e.hypothesis.clone(), avg_confidence),
            None => (format!("analysis of: {}", observation), avg_confidence),
        };

        ReflectionResult {
            explanation,
            confidence: conf,
            evidence: combined_evidence,
            contradictions: combined_contradictions,
        }
    }

    pub fn run_consensus_cycle(&mut self, observations: &[String]) -> ConsensusReport {
        let mut all_explanations = Vec::new();
        let mut total_conflicts = 0;
        let mut converged = false;
        let mut iteration = 0u32;

        for _iter in 0..self.config.max_iterations {
            iteration += 1;
            let mut iter_explanations = Vec::new();

            for obs in observations {
                for head in &self.heads {
                    let out = head.forward_reflect(obs);
                    iter_explanations.push(out.conclusion.clone());
                }
            }

            let context = iter_explanations.clone();
            let solved = self.solver.solve("consensus cycle", &context);

            let mut conflicts_resolved = 0;
            if solved.len() >= 2 {
                for i in 0..solved.len().min(5) {
                    for j in i + 1..solved.len().min(5) {
                        if (solved[i].plausibility - solved[j].plausibility).abs() < 0.3 {
                            if let Some(merged) =
                                self.solver.resolve_conflicts(&solved[i], &solved[j])
                            {
                                conflicts_resolved += 1;
                                iter_explanations.push(merged.hypothesis);
                            }
                        }
                    }
                }
            }
            total_conflicts += conflicts_resolved;

            for exp in &iter_explanations {
                if !all_explanations.contains(exp) {
                    all_explanations.push(exp.clone());
                }
            }

            let heads_above_threshold = self
                .heads
                .iter()
                .filter(|h| h.confidence > self.config.convergence_threshold)
                .count();
            if heads_above_threshold >= self.config.min_heads_agreed {
                converged = true;
                break;
            }
        }

        let overall_confidence = if !self.heads.is_empty() {
            let sum: f64 = self.heads.iter().map(|h| h.confidence).sum();
            (sum / self.heads.len() as f64).max(0.0).min(1.0)
        } else {
            0.0
        };

        ConsensusReport {
            converged,
            iterations: iteration,
            confidence: overall_confidence,
            explanations: all_explanations,
            conflicts_resolved: total_conflicts,
        }
    }
}

impl Default for ReflectionPipeline {
    fn default() -> Self {
        Self::new(ConsensusConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let config = ConsensusConfig {
            max_iterations: 5,
            convergence_threshold: 0.7,
            min_heads_agreed: 2,
            enable_backward_reflection: true,
        };
        let pipeline = ReflectionPipeline::new(config);
        assert_eq!(pipeline.config.max_iterations, 5);
        assert!(pipeline.heads.is_empty());
    }

    #[test]
    fn test_add_head() {
        let pipeline = ReflectionPipeline::new(ConsensusConfig::default());
        assert!(pipeline.heads.is_empty());
    }

    #[test]
    fn test_process_with_single_head() {
        let mut pipeline = ReflectionPipeline::new(ConsensusConfig::default());
        pipeline.add_head(ReflectionHead::new(0, "test".to_string()));
        let result = pipeline.process("something happened");
        assert!(!result.explanation.is_empty());
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
    }

    #[test]
    fn test_process_with_multiple_heads() {
        let mut pipeline = ReflectionPipeline::new(ConsensusConfig::default());
        pipeline.add_head(ReflectionHead::new(0, "security".to_string()));
        pipeline.add_head(ReflectionHead::new(1, "performance".to_string()));
        let result = pipeline.process("system alert with high memory");
        assert!(result.confidence > 0.0);
        assert!(!result.evidence.is_empty());
    }

    #[test]
    fn test_process_empty_heads() {
        let mut pipeline = ReflectionPipeline::new(ConsensusConfig::default());
        let result = pipeline.process("test");
        assert!((result.confidence - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_consensus_cycle_basic() {
        let mut pipeline = ReflectionPipeline::new(ConsensusConfig::default());
        pipeline.add_head(ReflectionHead::new(0, "observer".to_string()));
        pipeline.add_head(ReflectionHead::new(1, "analyzer".to_string()));
        let obs = vec!["error detected".to_string(), "system running".to_string()];
        let report = pipeline.run_consensus_cycle(&obs);
        assert!(report.iterations >= 1);
        assert!(!report.explanations.is_empty());
    }

    #[test]
    fn test_consensus_cycle_convergence() {
        let config = ConsensusConfig {
            max_iterations: 20,
            convergence_threshold: 0.5,
            min_heads_agreed: 1,
            enable_backward_reflection: true,
        };
        let mut pipeline = ReflectionPipeline::new(config);
        let mut head = ReflectionHead::new(0, "confident".to_string());
        head.confidence = 0.9;
        pipeline.add_head(head);
        let obs = vec!["consistent signal".to_string()];
        let report = pipeline.run_consensus_cycle(&obs);
        assert!(report.converged);
    }

    #[test]
    fn test_consensus_cycle_max_iterations() {
        let config = ConsensusConfig {
            max_iterations: 3,
            convergence_threshold: 0.99,
            min_heads_agreed: 1,
            enable_backward_reflection: true,
        };
        let mut pipeline = ReflectionPipeline::new(config);
        pipeline.add_head(ReflectionHead::new(0, "low".to_string()));
        let obs = vec!["test observation".to_string()];
        let report = pipeline.run_consensus_cycle(&obs);
        assert!(report.iterations <= 3);
    }

    #[test]
    fn test_consensus_config_default() {
        let config = ConsensusConfig::default();
        assert_eq!(config.max_iterations, 10);
        assert!((config.convergence_threshold - 0.7).abs() < 1e-6);
        assert_eq!(config.min_heads_agreed, 3);
        assert!(config.enable_backward_reflection);
    }

    #[test]
    fn test_process_with_biases() {
        let mut pipeline = ReflectionPipeline::new(ConsensusConfig::default());
        let mut head = ReflectionHead::new(0, "biased".to_string());
        head.biases.push("critical".to_string());
        pipeline.add_head(head);
        let result = pipeline.process("critical failure");
        assert!(result.evidence.iter().any(|e| e.contains("critical")));
    }

    #[test]
    fn test_pipeline_default() {
        let pipeline = ReflectionPipeline::default();
        assert_eq!(pipeline.config.max_iterations, 10);
    }
}
