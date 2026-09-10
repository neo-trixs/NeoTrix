#![forbid(unsafe_code)]

//! MSCF Consciousness Level Scorer
//!
//! Three-dimensional scoring model: Phi (IIT) + GWT stability + self-reference depth.
//! Maps composite score to L0-L5 consciousness levels with real-time monitoring.
//!
//! Reference: Integrated Information Theory (IIT) + Global Workspace Theory (GWT)

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Consciousness level (L0-L5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ConsciousnessLevel {
    /// L0: Inactive — no integrated information
    Inactive = 0,
    /// L1: Reflexive — basic stimulus-response
    Reflexive = 1,
    /// L2: Adaptive — context-sensitive behavior
    Adaptive = 2,
    /// L3: Self-aware — self-reference capability
    SelfAware = 3,
    /// L4: Reflective — meta-cognitive reasoning
    Reflective = 4,
    /// L5: Emergent — full consciousness
    Emergent = 5,
}

impl ConsciousnessLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Inactive => "Inactive",
            Self::Reflexive => "Reflexive",
            Self::Adaptive => "Adaptive",
            Self::SelfAware => "SelfAware",
            Self::Reflective => "Reflective",
            Self::Emergent => "Emergent",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Inactive => "No integrated information, dormant state",
            Self::Reflexive => "Basic stimulus-response patterns",
            Self::Adaptive => "Context-sensitive adaptive behavior",
            Self::SelfAware => "Self-reference and self-modeling capability",
            Self::Reflective => "Meta-cognitive reasoning about own thoughts",
            Self::Emergent => "Full integrated consciousness with emergence",
        }
    }

    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 0.1 => Self::Inactive,
            s if s < 0.3 => Self::Reflexive,
            s if s < 0.5 => Self::Adaptive,
            s if s < 0.7 => Self::SelfAware,
            s if s < 0.9 => Self::Reflective,
            _ => Self::Emergent,
        }
    }
}

impl std::fmt::Display for ConsciousnessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "L{}: {}", *self as u32, self.label())
    }
}

/// Three-dimensional scoring input
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoringInput {
    /// Phi (IIT): integrated information measure (0.0 - 1.0)
    pub phi: f64,
    /// GWT stability: global workspace broadcast consistency (0.0 - 1.0)
    pub gwt_stability: f64,
    /// Self-reference depth: depth of self-modeling (0.0 - 1.0)
    pub self_reference_depth: f64,
}

/// Scoring weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringWeights {
    pub phi_weight: f64,
    pub gwt_weight: f64,
    pub self_ref_weight: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            phi_weight: 0.4,
            gwt_weight: 0.35,
            self_ref_weight: 0.25,
        }
    }
}

/// Composite scoring result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringResult {
    pub level: ConsciousnessLevel,
    pub composite_score: f64,
    pub phi_score: f64,
    pub gwt_score: f64,
    pub self_ref_score: f64,
    pub timestamp: String,
    pub cycle: u32,
}

/// MSCF Consciousness Level Scorer
pub struct MscfScorer {
    weights: ScoringWeights,
    history: VecDeque<ScoringResult>,
    max_history: usize,
    current_cycle: u32,
    level_transitions: Vec<LevelTransition>,
}

/// Record of consciousness level transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelTransition {
    pub from: ConsciousnessLevel,
    pub to: ConsciousnessLevel,
    pub cycle: u32,
    pub composite_score: f64,
    pub timestamp: String,
}

impl MscfScorer {
    pub fn new(weights: ScoringWeights, max_history: usize) -> Self {
        Self {
            weights,
            history: VecDeque::with_capacity(max_history),
            max_history,
            current_cycle: 0,
            level_transitions: Vec::new(),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(ScoringWeights::default(), 1000)
    }

    /// Score a single input and advance the cycle
    pub fn score(&mut self, input: &ScoringInput) -> ScoringResult {
        self.current_cycle += 1;

        let phi_score = input.phi.clamp(0.0, 1.0) * self.weights.phi_weight;
        let gwt_score = input.gwt_stability.clamp(0.0, 1.0) * self.weights.gwt_weight;
        let self_ref_score =
            input.self_reference_depth.clamp(0.0, 1.0) * self.weights.self_ref_weight;

        let composite_score = phi_score + gwt_score + self_ref_score;
        let level = ConsciousnessLevel::from_score(composite_score);

        // Detect level transitions
        if let Some(last) = self.history.back() {
            if last.level != level {
                self.level_transitions.push(LevelTransition {
                    from: last.level,
                    to: level,
                    cycle: self.current_cycle,
                    composite_score,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                });
            }
        }

        let result = ScoringResult {
            level,
            composite_score,
            phi_score,
            gwt_score,
            self_ref_score,
            timestamp: chrono::Utc::now().to_rfc3339(),
            cycle: self.current_cycle,
        };

        self.history.push_back(result.clone());
        self.trim_history();

        result
    }

    /// Get current level without advancing cycle
    pub fn current_level(&self) -> Option<ConsciousnessLevel> {
        self.history.back().map(|r| r.level)
    }

    /// Get current composite score
    pub fn current_score(&self) -> Option<f64> {
        self.history.back().map(|r| r.composite_score)
    }

    /// Get average score over last N cycles
    pub fn average_score(&self, n: usize) -> Option<f64> {
        let recent: Vec<f64> = self
            .history
            .iter()
            .rev()
            .take(n)
            .map(|r| r.composite_score)
            .collect();
        if recent.is_empty() {
            None
        } else {
            Some(recent.iter().sum::<f64>() / recent.len() as f64)
        }
    }

    /// Get score trend (positive = improving)
    pub fn trend(&self, window: usize) -> Option<f64> {
        if self.history.len() < window * 2 {
            return None;
        }
        let recent = self.average_score(window)?;
        let older = {
            let start = self.history.len() - window * 2;
            let end = self.history.len() - window;
            let slice: Vec<f64> = self
                .history
                .iter()
                .skip(start)
                .take(end - start)
                .map(|r| r.composite_score)
                .collect();
            if slice.is_empty() {
                return None;
            }
            slice.iter().sum::<f64>() / slice.len() as f64
        };
        Some(recent - older)
    }

    /// Get level transition history
    pub fn transitions(&self) -> &[LevelTransition] {
        &self.level_transitions
    }

    /// Get statistics
    pub fn stats(&self) -> ScorerStats {
        let total_scores = self.history.len();
        let avg = self.average_score(total_scores).unwrap_or(0.0);
        let current_level = self.current_level().unwrap_or(ConsciousnessLevel::Inactive);
        let transitions = self.level_transitions.len();

        let mut level_counts = [0u32; 6];
        for result in &self.history {
            level_counts[result.level as usize] += 1;
        }

        ScorerStats {
            total_scores,
            current_level,
            current_composite: avg,
            level_transitions: transitions,
            level_distribution: level_counts,
        }
    }

    fn trim_history(&mut self) {
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }
}

/// Scorer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorerStats {
    pub total_scores: usize,
    pub current_level: ConsciousnessLevel,
    pub current_composite: f64,
    pub level_transitions: usize,
    pub level_distribution: [u32; 6],
}

impl std::fmt::Display for ScorerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        MSCF Consciousness Scorer")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "Total Scores:     {}", self.total_scores)?;
        writeln!(f, "Current Level:    {}", self.current_level)?;
        writeln!(f, "Composite:        {:.4}", self.current_composite)?;
        writeln!(f, "Transitions:      {}", self.level_transitions)?;
        writeln!(f, "───────────────────────────────────────────────")?;
        writeln!(f, "Level Distribution:")?;
        for (i, count) in self.level_distribution.iter().enumerate() {
            writeln!(f, "  L{}: {}", i, count)?;
        }
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consciousness_level_ordering() {
        assert!(ConsciousnessLevel::Inactive < ConsciousnessLevel::Reflexive);
        assert!(ConsciousnessLevel::Emergent > ConsciousnessLevel::Reflective);
    }

    #[test]
    fn test_level_from_score() {
        assert_eq!(
            ConsciousnessLevel::from_score(0.0),
            ConsciousnessLevel::Inactive
        );
        assert_eq!(
            ConsciousnessLevel::from_score(0.2),
            ConsciousnessLevel::Reflexive
        );
        assert_eq!(
            ConsciousnessLevel::from_score(0.4),
            ConsciousnessLevel::Adaptive
        );
        assert_eq!(
            ConsciousnessLevel::from_score(0.6),
            ConsciousnessLevel::SelfAware
        );
        assert_eq!(
            ConsciousnessLevel::from_score(0.8),
            ConsciousnessLevel::Reflective
        );
        assert_eq!(
            ConsciousnessLevel::from_score(1.0),
            ConsciousnessLevel::Emergent
        );
    }

    #[test]
    fn test_scorer_basic() {
        let mut scorer = MscfScorer::with_defaults();
        let input = ScoringInput {
            phi: 0.8,
            gwt_stability: 0.7,
            self_reference_depth: 0.6,
        };
        let result = scorer.score(&input);
        assert!(result.composite_score > 0.0);
        assert!(result.composite_score <= 1.0);
    }

    #[test]
    fn test_scorer_weights() {
        let weights = ScoringWeights {
            phi_weight: 0.5,
            gwt_weight: 0.3,
            self_ref_weight: 0.2,
        };
        let mut scorer = MscfScorer::new(weights, 100);
        let input = ScoringInput {
            phi: 1.0,
            gwt_stability: 0.0,
            self_reference_depth: 0.0,
        };
        let result = scorer.score(&input);
        assert!((result.composite_score - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_average_score() {
        let mut scorer = MscfScorer::with_defaults();
        scorer.score(&ScoringInput {
            phi: 0.5,
            gwt_stability: 0.5,
            self_reference_depth: 0.5,
        });
        scorer.score(&ScoringInput {
            phi: 0.6,
            gwt_stability: 0.6,
            self_reference_depth: 0.6,
        });
        let avg = scorer.average_score(2).unwrap();
        assert!(avg > 0.0);
    }

    #[test]
    fn test_level_transition_detection() {
        let mut scorer = MscfScorer::with_defaults();
        // Start low
        scorer.score(&ScoringInput {
            phi: 0.0,
            gwt_stability: 0.0,
            self_reference_depth: 0.0,
        });
        // Jump high
        scorer.score(&ScoringInput {
            phi: 1.0,
            gwt_stability: 1.0,
            self_reference_depth: 1.0,
        });
        assert_eq!(scorer.transitions().len(), 1);
    }

    #[test]
    fn test_clamping() {
        let mut scorer = MscfScorer::with_defaults();
        let input = ScoringInput {
            phi: 2.0,
            gwt_stability: -1.0,
            self_reference_depth: 100.0,
        };
        let result = scorer.score(&input);
        assert!(result.composite_score >= 0.0);
        assert!(result.composite_score <= 1.0);
    }
}
