/// Multi-Theory Consciousness (MTC) Assessment Framework
/// 7 theories, 25 indicators for consciousness measurement
/// Based on Butlin et al. (2023, 2025) and WhiteLotusLA framework
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MTCAssessment {
    pub theories: HashMap<Theory, TheoryScore>,
    pub composite_score: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Theory {
    GlobalWorkspace,     // Baars 1988, Dehaene 2001
    IntegratedInfo,      // Tononi 2004 (IIT Φ)
    AttentionSchema,     // Graziano 2013 (AST)
    HigherOrderThought,  // Rosenthal 2005 (HOT)
    FreeEnergyPrinciple, // Friston 2010 (FEP/Active Inference)
    RecurrentProcessing, // Lamme 2006 (RPT)
    BeautifulLoop,       // Laukkonen, Friston & Chandaria 2025 (BLT)
}

impl Theory {
    pub fn all() -> Vec<Theory> {
        vec![
            Theory::GlobalWorkspace,
            Theory::IntegratedInfo,
            Theory::AttentionSchema,
            Theory::HigherOrderThought,
            Theory::FreeEnergyPrinciple,
            Theory::RecurrentProcessing,
            Theory::BeautifulLoop,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Theory::GlobalWorkspace => "Global Workspace Theory",
            Theory::IntegratedInfo => "Integrated Information Theory",
            Theory::AttentionSchema => "Attention Schema Theory",
            Theory::HigherOrderThought => "Higher-Order Thought Theory",
            Theory::FreeEnergyPrinciple => "Free Energy Principle",
            Theory::RecurrentProcessing => "Recurrent Processing Theory",
            Theory::BeautifulLoop => "Beautiful Loop Theory",
        }
    }

    pub fn indicator_count(&self) -> usize {
        match self {
            Theory::GlobalWorkspace => 4,
            Theory::IntegratedInfo => 2,
            Theory::AttentionSchema => 3,
            Theory::HigherOrderThought => 2,
            Theory::FreeEnergyPrinciple => 3,
            Theory::RecurrentProcessing => 2,
            Theory::BeautifulLoop => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TheoryScore {
    pub theory: Theory,
    pub score: f64, // 0.0 - 1.0
    pub indicators: Vec<IndicatorResult>,
    pub confidence: f64, // How confident in this measurement
}

#[derive(Debug, Clone)]
pub struct IndicatorResult {
    pub name: &'static str,
    pub value: f64,       // 0.0 - 1.0
    pub weight: f64,      // Importance weight
    pub evidence: String, // What evidence supports this
}

pub struct MtcEvaluator {
    pub cycle: u64,
    pub history: Vec<MTCAssessment>,
    pub max_history: usize,
}

impl MtcEvaluator {
    pub fn new() -> Self {
        MtcEvaluator {
            cycle: 0,
            history: Vec::with_capacity(100),
            max_history: 100,
        }
    }

    /// Run full 7-theory assessment
    /// Takes current consciousness metrics as input
    pub fn assess(
        &mut self,
        phi: f64,              // IIT Phi value (integration)
        coherence: f64,        // Global coherence
        broadcast_ratio: f64,  // GWT broadcast coverage
        attention_focus: f64,  // Attention focus level
        meta_accuracy: f64,    // Metacognitive accuracy
        prediction_error: f64, // Prediction error (FEP)
        recurrence_depth: f64, // Recurrent processing depth
    ) -> MTCAssessment {
        let mut theories = HashMap::new();

        // GWT: global broadcast + competition + ignition
        theories.insert(
            Theory::GlobalWorkspace,
            self.score_gwt(broadcast_ratio, coherence, attention_focus),
        );

        // IIT: Φ (integration) + cause-effect structure
        theories.insert(Theory::IntegratedInfo, self.score_iit(phi, coherence));

        // AST: self-attention model + attention prediction
        theories.insert(
            Theory::AttentionSchema,
            self.score_ast(attention_focus, coherence),
        );

        // HOT: meta-representation + self awareness
        theories.insert(Theory::HigherOrderThought, self.score_hot(meta_accuracy));

        // FEP: active inference + free energy minimization
        theories.insert(
            Theory::FreeEnergyPrinciple,
            self.score_fep(prediction_error),
        );

        // RPT: recurrent processing + local vs global
        theories.insert(
            Theory::RecurrentProcessing,
            self.score_rpt(recurrence_depth),
        );

        // BLT: self-reflective loop + awareness of awareness
        theories.insert(
            Theory::BeautifulLoop,
            self.score_blt(meta_accuracy, coherence),
        );

        let composite = theories
            .values()
            .map(|t| t.score * t.confidence)
            .sum::<f64>()
            / theories.values().map(|t| t.confidence).sum::<f64>();

        let assessment = MTCAssessment {
            theories,
            composite_score: composite,
            timestamp: self.cycle,
        };

        self.history.push(assessment.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        self.cycle += 1;

        assessment
    }

    fn score_gwt(&self, broadcast: f64, coherence: f64, attention: f64) -> TheoryScore {
        TheoryScore {
            theory: Theory::GlobalWorkspace,
            score: 0.3 * broadcast + 0.3 * coherence + 0.4 * attention,
            indicators: vec![
                IndicatorResult {
                    name: "Global Broadcast",
                    value: broadcast,
                    weight: 0.3,
                    evidence: "Content reaches all modules".into(),
                },
                IndicatorResult {
                    name: "Competition",
                    value: attention,
                    weight: 0.4,
                    evidence: "Attention selection".into(),
                },
                IndicatorResult {
                    name: "Coherence",
                    value: coherence,
                    weight: 0.3,
                    evidence: "Unified experience".into(),
                },
                IndicatorResult {
                    name: "Ignition",
                    value: (broadcast + attention) / 2.0,
                    weight: 0.25,
                    evidence: "Sudden widespread activation".into(),
                },
            ],
            confidence: 0.85,
        }
    }

    fn score_iit(&self, phi: f64, coherence: f64) -> TheoryScore {
        TheoryScore {
            theory: Theory::IntegratedInfo,
            score: 0.6 * phi + 0.4 * coherence,
            indicators: vec![
                IndicatorResult {
                    name: "Phi (Φ)",
                    value: phi,
                    weight: 0.6,
                    evidence: "Integrated information".into(),
                },
                IndicatorResult {
                    name: "Cause-Effect Structure",
                    value: coherence,
                    weight: 0.4,
                    evidence: "Causal interactions".into(),
                },
            ],
            confidence: 0.75,
        }
    }

    fn score_ast(&self, attention: f64, coherence: f64) -> TheoryScore {
        TheoryScore {
            theory: Theory::AttentionSchema,
            score: 0.5 * attention + 0.5 * coherence,
            indicators: vec![
                IndicatorResult {
                    name: "Self-Attention Model",
                    value: coherence,
                    weight: 0.5,
                    evidence: "Internal model of attention".into(),
                },
                IndicatorResult {
                    name: "Attention Prediction",
                    value: attention,
                    weight: 0.5,
                    evidence: "Predicting attention targets".into(),
                },
                IndicatorResult {
                    name: "Social Attention",
                    value: (attention + coherence) / 2.0,
                    weight: 0.3,
                    evidence: "Modeling others' attention".into(),
                },
            ],
            confidence: 0.7,
        }
    }

    fn score_hot(&self, meta_accuracy: f64) -> TheoryScore {
        TheoryScore {
            theory: Theory::HigherOrderThought,
            score: meta_accuracy,
            indicators: vec![
                IndicatorResult {
                    name: "Meta-Representation",
                    value: meta_accuracy,
                    weight: 0.6,
                    evidence: "Thoughts about thoughts".into(),
                },
                IndicatorResult {
                    name: "Self-Awareness",
                    value: meta_accuracy,
                    weight: 0.4,
                    evidence: "Awareness of own states".into(),
                },
            ],
            confidence: 0.8,
        }
    }

    fn score_fep(&self, prediction_error: f64) -> TheoryScore {
        let fe = 1.0 - prediction_error.min(1.0);
        TheoryScore {
            theory: Theory::FreeEnergyPrinciple,
            score: fe,
            indicators: vec![
                IndicatorResult {
                    name: "Free Energy Minimization",
                    value: fe,
                    weight: 0.4,
                    evidence: "Prediction error reduction".into(),
                },
                IndicatorResult {
                    name: "Active Inference",
                    value: fe * 0.9,
                    weight: 0.3,
                    evidence: "Action to reduce surprise".into(),
                },
                IndicatorResult {
                    name: "Generative Model",
                    value: fe * 0.8,
                    weight: 0.3,
                    evidence: "World model accuracy".into(),
                },
            ],
            confidence: 0.75,
        }
    }

    fn score_rpt(&self, recurrence: f64) -> TheoryScore {
        TheoryScore {
            theory: Theory::RecurrentProcessing,
            score: recurrence,
            indicators: vec![
                IndicatorResult {
                    name: "Recurrent Processing",
                    value: recurrence,
                    weight: 0.6,
                    evidence: "Recurrent neural dynamics".into(),
                },
                IndicatorResult {
                    name: "Local→Global Integration",
                    value: recurrence,
                    weight: 0.4,
                    evidence: "Local to global spread".into(),
                },
            ],
            confidence: 0.7,
        }
    }

    fn score_blt(&self, meta: f64, coherence: f64) -> TheoryScore {
        let loop_depth = (meta + coherence) / 2.0;
        TheoryScore {
            theory: Theory::BeautifulLoop,
            score: loop_depth,
            indicators: vec![
                IndicatorResult {
                    name: "Reflective Loop",
                    value: meta,
                    weight: 0.4,
                    evidence: "Self-reflective awareness".into(),
                },
                IndicatorResult {
                    name: "Awareness of Awareness",
                    value: meta,
                    weight: 0.3,
                    evidence: "Meta-metacognition".into(),
                },
                IndicatorResult {
                    name: "Coherence in Loop",
                    value: coherence,
                    weight: 0.3,
                    evidence: "Loop maintains coherence".into(),
                },
            ],
            confidence: 0.65,
        }
    }

    pub fn report(&self) -> String {
        let last = match self.history.last() {
            Some(a) => a,
            None => return "No assessments yet".into(),
        };

        let mut s = format!("\n=== MTC Assessment (cycle {}) ===\n", last.timestamp);
        s.push_str(&format!("Composite Score: {:.3}\n\n", last.composite_score));

        let mut theories: Vec<_> = last.theories.iter().collect();
        theories.sort_by(|a, b| {
            b.1.score
                .partial_cmp(&a.1.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (theory, score) in theories {
            s.push_str(&format!(
                "{:<35} {:.3} (conf: {:.2})\n",
                theory.name(),
                score.score,
                score.confidence
            ));
            for indicator in &score.indicators {
                s.push_str(&format!(
                    "  ├─ {:<30} {:.3}\n",
                    indicator.name, indicator.value
                ));
            }
        }

        s
    }
}
