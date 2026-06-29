use super::vsa_tag::{VsaOrigin, VsaSelfCategory, VsaTagged};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathType {
    Constraint,
    Generative,
    Merged,
}

#[derive(Debug, Clone)]
pub struct PathOutput {
    pub path: PathType,
    pub state: VsaTagged,
    pub confidence: f64,
    pub reasoning_steps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DualPathConfig {
    /// Whether to run constraint path
    pub enable_constraint: bool,
    /// Whether to run generative path
    pub enable_generative: bool,
    /// Merge strategy: Consensus (both agree) vs Weighted (confidence-weighted average)
    pub merge_strategy: MergeStrategy,
    /// Constraint path strength factor
    pub constraint_strength: f64,
    /// Generative exploration factor
    pub generative_temperature: f64,
}

impl Default for DualPathConfig {
    fn default() -> Self {
        Self {
            enable_constraint: true,
            enable_generative: true,
            merge_strategy: MergeStrategy::Consensus,
            constraint_strength: 0.7,
            generative_temperature: 0.4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    Consensus,
    Weighted,
    WinnerTakeAll,
}

#[derive(Debug, Clone)]
pub struct DualPathResult {
    pub constraint: Option<PathOutput>,
    pub generative: Option<PathOutput>,
    pub merged: PathOutput,
    pub consensus: f64,
    pub cross_validation: CrossValidation,
}

#[derive(Debug, Clone)]
pub struct CrossValidation {
    pub paths_agree: bool,
    pub agreement_score: f64,
    pub constraint_reliability: f64,
    pub generative_reliability: f64,
}

pub struct DualPathInference {
    config: DualPathConfig,
}

impl DualPathInference {
    pub fn new(config: DualPathConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &DualPathConfig {
        &self.config
    }

    pub fn infer(&self, input: &[u8], task: &str) -> DualPathResult {
        let constraint = if self.config.enable_constraint {
            Some(self.constraint_path(input, task))
        } else {
            None
        };

        let generative = if self.config.enable_generative {
            Some(self.generative_path(input, task))
        } else {
            None
        };

        let merged = self.merge_paths(constraint.as_ref(), generative.as_ref());
        let xv = self.cross_validate(constraint.as_ref(), generative.as_ref());

        DualPathResult {
            constraint,
            generative,
            merged,
            consensus: xv.agreement_score,
            cross_validation: xv,
        }
    }

    fn constraint_path(&self, input: &[u8], task: &str) -> PathOutput {
        let mut steps = Vec::new();
        steps.push(format!("[Constraint] Bind task '{}' to input", task));
        steps.push("[Constraint] Apply deterministic rules".into());

        let mut vec = input.to_vec();
        for byte in vec.iter_mut().take(16) {
            *byte = byte.wrapping_mul(3).wrapping_add(7);
        }
        let state = VsaTagged::new(vec, VsaOrigin::Self_(VsaSelfCategory::Thought))
            .with_confidence(self.config.constraint_strength);

        PathOutput {
            path: PathType::Constraint,
            state,
            confidence: self.config.constraint_strength,
            reasoning_steps: steps,
        }
    }

    fn generative_path(&self, input: &[u8], task: &str) -> PathOutput {
        let mut steps = Vec::new();
        steps.push(format!("[Generative] Explore alternatives for '{}'", task));
        steps.push("[Generative] Apply stochastic variations".into());

        let mut vec = input.to_vec();
        let temp = self.config.generative_temperature;
        for i in 0..vec.len().min(32) {
            let noise = ((i * 17 + 3) as f64 * temp * 256.0) as u8;
            vec[i] = vec[i].wrapping_add(noise);
        }
        let confidence = 0.3 + temp * 0.5;
        let state = VsaTagged::new(vec, VsaOrigin::Self_(VsaSelfCategory::Thought))
            .with_confidence(confidence);

        PathOutput {
            path: PathType::Generative,
            state,
            confidence,
            reasoning_steps: steps,
        }
    }

    fn merge_paths(
        &self,
        constraint: Option<&PathOutput>,
        generative: Option<&PathOutput>,
    ) -> PathOutput {
        match (constraint, generative) {
            (Some(c), Some(g)) => match self.config.merge_strategy {
                MergeStrategy::Consensus => {
                    let vec: Vec<u8> = c
                        .state
                        .vector
                        .iter()
                        .zip(g.state.vector.iter())
                        .map(|(a, b)| (a.wrapping_add(*b)) / 2)
                        .collect();
                    let conf = (c.confidence + g.confidence) / 2.0;
                    let state = VsaTagged::new(vec, VsaOrigin::Self_(VsaSelfCategory::Thought))
                        .with_confidence(conf);
                    PathOutput {
                        path: PathType::Merged,
                        state,
                        confidence: conf,
                        reasoning_steps: vec!["[Merge] Consensus: average of both paths".into()],
                    }
                }
                MergeStrategy::Weighted => {
                    let total = c.confidence + g.confidence;
                    let cw = c.confidence / total;
                    let gw = g.confidence / total;
                    let vec: Vec<u8> = c
                        .state
                        .vector
                        .iter()
                        .zip(g.state.vector.iter())
                        .map(|(a, b)| {
                            (a.wrapping_mul((cw * 256.0) as u8)
                                .wrapping_add(b.wrapping_mul((gw * 256.0) as u8)))
                                / 2
                        })
                        .collect();
                    let conf = c.confidence.max(g.confidence);
                    let state = VsaTagged::new(vec, VsaOrigin::Self_(VsaSelfCategory::Thought))
                        .with_confidence(conf);
                    PathOutput {
                        path: PathType::Merged,
                        state,
                        confidence: conf,
                        reasoning_steps: vec![format!("[Merge] Weighted: c={:.2} g={:.2}", cw, gw)],
                    }
                }
                MergeStrategy::WinnerTakeAll => {
                    if c.confidence >= g.confidence {
                        c.clone()
                    } else {
                        g.clone()
                    }
                }
            },
            (Some(c), None) => PathOutput {
                path: PathType::Merged,
                state: c.state.clone(),
                confidence: c.confidence,
                reasoning_steps: vec!["[Merge] Constraint-only fallback".into()],
            },
            (None, Some(g)) => PathOutput {
                path: PathType::Merged,
                state: g.state.clone(),
                confidence: g.confidence,
                reasoning_steps: vec!["[Merge] Generative-only fallback".into()],
            },
            (None, None) => PathOutput {
                path: PathType::Merged,
                state: VsaTagged::new(vec![0u8; 64], VsaOrigin::Self_(VsaSelfCategory::Thought))
                    .with_confidence(0.0),
                confidence: 0.0,
                reasoning_steps: vec!["[Merge] No paths available".into()],
            },
        }
    }

    fn cross_validate(
        &self,
        constraint: Option<&PathOutput>,
        generative: Option<&PathOutput>,
    ) -> CrossValidation {
        match (constraint, generative) {
            (Some(c), Some(g)) => {
                let n = c.state.vector.len().min(g.state.vector.len());
                let same = c
                    .state
                    .vector
                    .iter()
                    .zip(g.state.vector.iter())
                    .filter(|(a, b)| a == b)
                    .count();
                let agreement = same as f64 / n as f64;
                CrossValidation {
                    paths_agree: agreement > 0.5,
                    agreement_score: agreement,
                    constraint_reliability: c.confidence,
                    generative_reliability: g.confidence,
                }
            }
            (Some(c), None) => CrossValidation {
                paths_agree: true,
                agreement_score: 1.0,
                constraint_reliability: c.confidence,
                generative_reliability: 0.0,
            },
            (None, Some(g)) => CrossValidation {
                paths_agree: true,
                agreement_score: 1.0,
                constraint_reliability: 0.0,
                generative_reliability: g.confidence,
            },
            (None, None) => CrossValidation {
                paths_agree: false,
                agreement_score: 0.0,
                constraint_reliability: 0.0,
                generative_reliability: 0.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_path_default_config() {
        let config = DualPathConfig::default();
        assert!(config.enable_constraint);
        assert!(config.enable_generative);
    }

    #[test]
    fn test_dual_path_runs_both_paths() {
        let engine = DualPathInference::new(DualPathConfig::default());
        let result = engine.infer(&[128u8; 64], "test");
        assert!(result.constraint.is_some());
        assert!(result.generative.is_some());
        assert_eq!(result.merged.path, PathType::Merged);
    }

    #[test]
    fn test_constraint_path_produces_output() {
        let engine = DualPathInference::new(DualPathConfig::default());
        let result = engine.infer(&[128u8; 64], "constraint_test");
        let c = result.constraint.unwrap();
        assert_eq!(c.path, PathType::Constraint);
        assert!(c.confidence > 0.0);
    }

    #[test]
    fn test_generative_path_produces_output() {
        let engine = DualPathInference::new(DualPathConfig::default());
        let result = engine.infer(&[128u8; 64], "gen_test");
        let g = result.generative.unwrap();
        assert_eq!(g.path, PathType::Generative);
        assert!(g.confidence > 0.0);
    }

    #[test]
    fn test_cross_validation_both_paths() {
        let engine = DualPathInference::new(DualPathConfig::default());
        let result = engine.infer(&[128u8; 64], "xv_test");
        assert!(result.cross_validation.agreement_score >= 0.0);
    }

    #[test]
    fn test_winner_take_all() {
        let engine = DualPathInference::new(DualPathConfig {
            merge_strategy: MergeStrategy::WinnerTakeAll,
            ..Default::default()
        });
        let result = engine.infer(&[128u8; 64], "wta");
        assert_eq!(result.merged.path, PathType::Merged);
    }

    #[test]
    fn test_weighted_merge() {
        let engine = DualPathInference::new(DualPathConfig {
            merge_strategy: MergeStrategy::Weighted,
            ..Default::default()
        });
        let result = engine.infer(&[128u8; 64], "weighted");
        assert!(result.merged.confidence > 0.0);
    }

    #[test]
    fn test_constraint_only() {
        let engine = DualPathInference::new(DualPathConfig {
            enable_generative: false,
            ..Default::default()
        });
        let result = engine.infer(&[128u8; 64], "c_only");
        assert!(result.constraint.is_some());
        assert!(result.generative.is_none());
    }

    #[test]
    fn test_generative_only() {
        let engine = DualPathInference::new(DualPathConfig {
            enable_constraint: false,
            ..Default::default()
        });
        let result = engine.infer(&[128u8; 64], "g_only");
        assert!(result.constraint.is_none());
        assert!(result.generative.is_some());
    }
}
