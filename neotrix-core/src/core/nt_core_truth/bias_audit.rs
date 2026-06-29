#[derive(Debug, Clone, PartialEq)]
pub enum BiasType {
    ConfirmationBias,
    AvailabilityHeuristic,
    AffectHeuristic,
    Anchoring,
    DunningKruger,
    MotivatedReasoning,
    Groupthink,
}

impl BiasType {
    pub fn label(&self) -> &'static str {
        match self {
            BiasType::ConfirmationBias => "confirmation_bias",
            BiasType::AvailabilityHeuristic => "availability_heuristic",
            BiasType::AffectHeuristic => "affect_heuristic",
            BiasType::Anchoring => "anchoring",
            BiasType::DunningKruger => "dunning_kruger",
            BiasType::MotivatedReasoning => "motivated_reasoning",
            BiasType::Groupthink => "groupthink",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BiasType::ConfirmationBias => "Seeking only evidence that supports existing beliefs",
            BiasType::AvailabilityHeuristic => {
                "Judging probability by recency/salience not frequency"
            }
            BiasType::AffectHeuristic => "Emotional state substituting for risk/benefit analysis",
            BiasType::Anchoring => "First impression dominating subsequent judgment",
            BiasType::DunningKruger => "Low-ability overestimating their competence",
            BiasType::MotivatedReasoning => "Desired conclusion driving spurious reasoning",
            BiasType::Groupthink => "Group consensus suppressing dissenting views",
        }
    }

    pub fn defense_strategy(&self) -> &'static str {
        match self {
            BiasType::ConfirmationBias => {
                "ACH falsification method + active search for opposing evidence"
            }
            BiasType::AvailabilityHeuristic => "Base rate calibration + frequency statistics",
            BiasType::AffectHeuristic => "EmotionTag pre-annotation + deferred judgment",
            BiasType::Anchoring => "Reference frame shifting + reverse interval estimation",
            BiasType::DunningKruger => "Metacognitive calibration via confidence scoring",
            BiasType::MotivatedReasoning => "Pre-commit to evaluation criteria + ACH matrix",
            BiasType::Groupthink => "Devil's Advocate automatic role + ACH anonymous scoring",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BiasSignal {
    pub bias: BiasType,
    pub strength: f64, // 0..1 how strongly this bias is indicated
    pub evidence: String,
}

#[derive(Debug, Clone)]
pub struct BiasAuditReport {
    pub signals: Vec<BiasSignal>,
    pub overall_risk: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BiasAuditor {
    pub thresholds: BiasThresholds,
    pub reasoning_history: Vec<ReasoningSnapshot>,
    pub max_history: usize,
}

#[derive(Debug, Clone)]
pub struct BiasThresholds {
    pub high_emotional_valence: f64, // > this triggers affect heuristic check
    pub single_hypothesis_focus: usize, // how many hypotheses considered
    pub quick_conclusion_threshold_ms: u64,
}

impl Default for BiasThresholds {
    fn default() -> Self {
        Self {
            high_emotional_valence: 0.6,
            single_hypothesis_focus: 1,
            quick_conclusion_threshold_ms: 500,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReasoningSnapshot {
    pub timestamp: u64,
    pub emotional_valence: f64,
    pub hypothesis_count: usize,
    pub evidence_sources: Vec<String>,
    pub conclusion: String,
}

impl Default for BiasAuditor {
    fn default() -> Self {
        Self::new()
    }
}

impl BiasAuditor {
    pub fn new() -> Self {
        Self {
            thresholds: BiasThresholds::default(),
            reasoning_history: Vec::new(),
            max_history: 100,
        }
    }

    pub fn record_snapshot(&mut self, snapshot: ReasoningSnapshot) {
        if self.reasoning_history.len() >= self.max_history {
            self.reasoning_history.remove(0);
        }
        self.reasoning_history.push(snapshot);
    }

    /// Detect confirmation bias: look for trend of ignoring contradictory evidence
    pub fn detect_confirmation_bias(&self) -> Option<BiasSignal> {
        if self.reasoning_history.len() < 3 {
            return None;
        }

        let recent: Vec<_> = self.reasoning_history.iter().rev().take(5).collect();
        if recent.len() < 3 {
            return None;
        }

        // Check if hypothesis count has been steadily at 1 across latest snapshots
        let single_hypo_count = recent.iter().filter(|s| s.hypothesis_count <= 1).count();
        if single_hypo_count as f64 / recent.len() as f64 > 0.6 {
            return Some(BiasSignal {
                bias: BiasType::ConfirmationBias,
                strength: 0.6,
                evidence: format!(
                    "Only 1 hypothesis considered in {}/{} recent snapshots",
                    single_hypo_count,
                    recent.len()
                ),
            });
        }
        None
    }

    /// Detect affect heuristic: high emotional valence without multi-hypothesis reasoning
    pub fn detect_affect_heuristic(
        &self,
        current_valence: f64,
        hypothesis_count: usize,
    ) -> Option<BiasSignal> {
        if current_valence.abs() > self.thresholds.high_emotional_valence && hypothesis_count <= 1 {
            Some(BiasSignal {
                bias: BiasType::AffectHeuristic,
                strength: current_valence.abs(),
                evidence: format!(
                    "High emotional valence ({:.2}) with single hypothesis focus",
                    current_valence
                ),
            })
        } else {
            None
        }
    }

    /// Detect anchoring: first evidence source dominating later reasoning
    pub fn detect_anchoring(&self) -> Option<BiasSignal> {
        if self.reasoning_history.len() < 2 {
            return None;
        }

        let first = self.reasoning_history.first()?;
        let last = self.reasoning_history.last()?;

        // If conclusion hasn't changed despite accumulating more evidence
        if first.conclusion == last.conclusion
            && first.evidence_sources.len() < last.evidence_sources.len()
        {
            Some(BiasSignal {
                bias: BiasType::Anchoring,
                strength: 0.5,
                evidence: format!(
                    "Same conclusion ({}) persists despite {}→{} evidence sources",
                    first.conclusion,
                    first.evidence_sources.len(),
                    last.evidence_sources.len()
                ),
            })
        } else {
            None
        }
    }

    /// Detect motivated reasoning: conclusion aligns with desire despite evidence
    pub fn detect_motivated_reasoning(
        &self,
        desired_outcome: &str,
        actual_conclusion: &str,
        confidence: f64,
    ) -> Option<BiasSignal> {
        if actual_conclusion
            .to_lowercase()
            .contains(&desired_outcome.to_lowercase())
            && confidence > 0.8
        {
            Some(BiasSignal {
                bias: BiasType::MotivatedReasoning,
                strength: 0.5,
                evidence: format!(
                    "Conclusion matches desired outcome '{}' with high confidence ({:.2})",
                    desired_outcome, confidence
                ),
            })
        } else {
            None
        }
    }

    /// Run all bias checks
    pub fn audit(
        &mut self,
        current_valence: f64,
        hypothesis_count: usize,
        desired_outcome: &str,
        conclusion: &str,
        confidence: f64,
    ) -> BiasAuditReport {
        let mut signals = Vec::new();
        let mut recommendations = Vec::new();

        // Check affect heuristic
        if let Some(s) = self.detect_affect_heuristic(current_valence, hypothesis_count) {
            recommendations.push(format!(
                "Apply {} — {}",
                s.bias.label(),
                s.bias.defense_strategy()
            ));
            signals.push(s);
        }

        // Check confirmation bias
        if let Some(s) = self.detect_confirmation_bias() {
            recommendations.push(format!(
                "Apply {} — {}",
                s.bias.label(),
                s.bias.defense_strategy()
            ));
            signals.push(s);
        }

        // Check anchoring
        if let Some(s) = self.detect_anchoring() {
            recommendations.push(format!(
                "Apply {} — {}",
                s.bias.label(),
                s.bias.defense_strategy()
            ));
            signals.push(s);
        }

        // Check motivated reasoning
        if let Some(s) = self.detect_motivated_reasoning(desired_outcome, conclusion, confidence) {
            recommendations.push(format!(
                "Apply {} — {}",
                s.bias.label(),
                s.bias.defense_strategy()
            ));
            signals.push(s);
        }

        let overall_risk = signals.iter().map(|s| s.strength).sum::<f64>() / 4.0;

        BiasAuditReport {
            signals,
            overall_risk,
            recommendations,
        }
    }

    pub fn summary(&self, report: &BiasAuditReport) -> String {
        if report.signals.is_empty() {
            return "✓ No cognitive biases detected in reasoning process".into();
        }
        let details: Vec<String> = report
            .signals
            .iter()
            .map(|s| format!("{}:{}", s.bias.label(), (s.strength * 100.0) as i32))
            .collect();
        format!(
            "⚠ Bias risk {:.1}% — {}",
            report.overall_risk * 100.0,
            details.join(" ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_affect_heuristic_detection() {
        let auditor = BiasAuditor::new();
        let signal = auditor.detect_affect_heuristic(0.85, 1);
        assert!(signal.is_some());
        assert_eq!(signal.unwrap().bias, BiasType::AffectHeuristic);
    }

    #[test]
    fn test_no_bias_with_multi_hypothesis() {
        let auditor = BiasAuditor::new();
        let signal = auditor.detect_affect_heuristic(0.85, 3);
        assert!(signal.is_none());
    }

    #[test]
    fn test_anchoring_detection() {
        let mut auditor = BiasAuditor::new();
        auditor.record_snapshot(ReasoningSnapshot {
            timestamp: 1,
            emotional_valence: 0.0,
            hypothesis_count: 2,
            evidence_sources: vec!["src1".into()],
            conclusion: "X is true".into(),
        });
        auditor.record_snapshot(ReasoningSnapshot {
            timestamp: 2,
            emotional_valence: 0.0,
            hypothesis_count: 3,
            evidence_sources: vec!["src1".into(), "src2".into(), "src3".into()],
            conclusion: "X is true".into(),
        });
        let signal = auditor.detect_anchoring();
        assert!(signal.is_some());
    }

    #[test]
    fn test_full_audit() {
        let mut auditor = BiasAuditor::new();
        let report = auditor.audit(0.9, 1, "prove X", "X is true", 0.95);
        assert!(!report.signals.is_empty());
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_clean_audit() {
        let mut auditor = BiasAuditor::new();
        let report = auditor.audit(0.2, 4, "impartial", "maybe Y", 0.5);
        assert!(report.signals.is_empty());
    }
}
