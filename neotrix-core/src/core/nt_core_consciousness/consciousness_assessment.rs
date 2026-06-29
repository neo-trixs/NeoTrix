#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConsciousnessTheory {
    GlobalWorkspace,
    AttentionSchema,
    HigherOrderThought,
    FreeEnergyPrinciple,
    IntegratedInformation,
    BlackboardTheory,
    RecurrentProcessing,
}

#[derive(Debug, Clone)]
pub struct TheoryIndicator {
    pub theory: ConsciousnessTheory,
    pub score: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct ConsciousnessAssessment {
    pub indicators: Vec<TheoryIndicator>,
}

impl ConsciousnessAssessment {
    pub fn new() -> Self {
        Self {
            indicators: vec![
                TheoryIndicator {
                    theory: ConsciousnessTheory::GlobalWorkspace,
                    score: 0.0,
                    confidence: 0.0,
                },
                TheoryIndicator {
                    theory: ConsciousnessTheory::AttentionSchema,
                    score: 0.0,
                    confidence: 0.0,
                },
                TheoryIndicator {
                    theory: ConsciousnessTheory::HigherOrderThought,
                    score: 0.0,
                    confidence: 0.0,
                },
                TheoryIndicator {
                    theory: ConsciousnessTheory::FreeEnergyPrinciple,
                    score: 0.0,
                    confidence: 0.0,
                },
                TheoryIndicator {
                    theory: ConsciousnessTheory::IntegratedInformation,
                    score: 0.0,
                    confidence: 0.0,
                },
                TheoryIndicator {
                    theory: ConsciousnessTheory::BlackboardTheory,
                    score: 0.0,
                    confidence: 0.0,
                },
                TheoryIndicator {
                    theory: ConsciousnessTheory::RecurrentProcessing,
                    score: 0.0,
                    confidence: 0.0,
                },
            ],
        }
    }

    pub fn set_indicator(&mut self, theory: ConsciousnessTheory, score: f64, confidence: f64) {
        for i in &mut self.indicators {
            if i.theory == theory {
                i.score = score.clamp(0.0, 1.0);
                i.confidence = confidence.clamp(0.0, 1.0);
            }
        }
    }

    pub fn best_theory(&self) -> Option<ConsciousnessTheory> {
        self.indicators
            .iter()
            .max_by(|a, b| {
                (a.score * a.confidence)
                    .partial_cmp(&(b.score * b.confidence))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|i| i.theory)
    }

    pub fn report(&self) -> String {
        let mut lines: Vec<String> = self
            .indicators
            .iter()
            .map(|i| format!("{:?}: {:.2} (conf: {:.2})", i.theory, i.score, i.confidence))
            .collect();
        lines.sort();
        lines.join("\n")
    }
}

impl Default for ConsciousnessAssessment {
    fn default() -> Self {
        Self::new()
    }
}
