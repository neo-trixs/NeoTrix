//! JEV Gate — unified decision gate that replaces scattered gate patterns
//!
//! The gate takes a set of JEV questions, evaluates them (via backend or simulation),
//! validates results, and returns a unified verdict with exit code.

use super::presets::ABSTAIN_OPTION;
use super::primitives::*;
use super::validation::validate_result_set;
use std::collections::HashMap;

/// A question to be evaluated by the gate
#[derive(Debug, Clone)]
pub struct GateQuestion {
    /// Unique question ID
    pub id: String,
    /// Question type description
    pub question_type: GateQuestionType,
    /// Priority (higher = more important)
    pub priority: u8,
}

/// Question types for the gate
#[derive(Debug, Clone)]
pub enum GateQuestionType {
    /// Boolean: "Is this safe?"
    Noul { instructions: String },
    /// Selection: "Which tool?"
    Choice { instructions: String, options: Vec<String> },
    /// Rating: "Severity level?"
    Score { instructions: String, levels: Vec<String> },
}

/// Gate configuration
#[derive(Debug, Clone)]
pub struct GateConfig {
    /// Confidence threshold below which needs_review is set
    pub confidence_threshold: f64,
    /// Probability mass validation tolerance
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub validation_tolerance: f64,
    /// Block on any error (hard gate)
    pub hard_block: bool,
    /// Maximum questions per evaluation
    pub max_questions: usize,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.7,
            validation_tolerance: 0.05,
            hard_block: true,
            max_questions: 32,
        }
    }
}

/// Gate evaluation result
#[derive(Debug, Clone)]
pub struct GateResult {
    /// All decisions
    pub decisions: JevResultSet,
    /// Overall verdict: Pass/Review/Block
    pub verdict: GateVerdict,
    /// Exit code for process integration
    pub exit_code: ExitCode,
    /// Validation errors (empty if valid)
    pub validation_errors: HashMap<String, Vec<String>>,
    /// Model used (if any)
    pub model: Option<String>,
}

/// Gate-level verdict
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateVerdict {
    /// All decisions passed, no review needed
    Pass,
    /// One or more decisions need human review
    Review,
    /// One or more decisions failed validation or were blocked
    Block,
}

/// The JEV Gate — unified decision gate
pub struct JevGate {
    config: GateConfig,
}

impl JevGate {
    pub fn new(config: GateConfig) -> Self {
        Self { config }
    }

    pub fn default_config() -> Self {
        Self::new(GateConfig::default())
    }

    /// Evaluate questions and produce a gate result
    ///
    /// This is the primary entry point for JEV gate evaluation.
    /// The `evaluator` function takes a question and returns a JevDecision.
    pub fn evaluate<F>(
        &self,
        questions: &[GateQuestion],
        evaluator: F,
    ) -> GateResult
    where
        F: Fn(&GateQuestion) -> JevDecision,
    {
        // Enforce max questions
        if questions.len() > self.config.max_questions {
            return GateResult {
                decisions: HashMap::new(),
                verdict: GateVerdict::Block,
                exit_code: ExitCode::Error,
                validation_errors: {
                    let mut m = HashMap::new();
                    m.insert("_gate".to_string(), vec![
                        format!("too many questions: {} > max {}", questions.len(), self.config.max_questions)
                    ]);
                    m
                },
                model: None,
            };
        }

        // Evaluate each question
        let mut decisions = JevResultSet::new();
        for q in questions {
            let decision = evaluator(q);
            decisions.insert(q.id.clone(), decision);
        }

        // Apply confidence threshold
        for (_id, decision) in &mut decisions {
            self.apply_confidence_threshold(decision);
        }

        // Validate
        let validation_errors = validate_result_set(&decisions);

        // Determine verdict
        let verdict = if self.config.hard_block && !validation_errors.is_empty() {
            GateVerdict::Block
        } else if decisions.values().any(|d| d.needs_review()) {
            GateVerdict::Review
        } else if validation_errors.values().any(|e| !e.is_empty()) {
            GateVerdict::Block
        } else {
            GateVerdict::Pass
        };

        // Determine exit code
        let exit_code = match verdict {
            GateVerdict::Block => ExitCode::Error,
            GateVerdict::Review => ExitCode::NeedsReview,
            GateVerdict::Pass => ExitCode::Success,
        };

        GateResult {
            decisions,
            verdict,
            exit_code,
            validation_errors,
            model: None,
        }
    }

    /// Apply confidence threshold — set needs_review if confidence too low
    fn apply_confidence_threshold(&self, decision: &mut JevDecision) {
        match decision {
            JevDecision::Choice(c) => {
                if c.confidence < self.config.confidence_threshold {
                    c.needs_review = true;
                    c.reason = Some(format!(
                        "confidence {:.4} < threshold {:.4}",
                        c.confidence, self.config.confidence_threshold
                    ));
                    c.status = DecisionStatus::Review;
                }
            }
            JevDecision::Score(s) => {
                if s.confidence < self.config.confidence_threshold {
                    s.needs_review = true;
                    s.reason = Some(format!(
                        "confidence {:.4} < threshold {:.4}",
                        s.confidence, self.config.confidence_threshold
                    ));
                    s.status = DecisionStatus::Review;
                }
            }
            JevDecision::Noul(n) => {
                // For Noul, use probability near 0.5 as low confidence
                let uncertainty = (n.noul - 0.5).abs() * 2.0; // 0 at 0.5, 1 at extremes
                if uncertainty < self.config.confidence_threshold {
                    n.needs_review = true;
                    n.reason = Some(format!(
                        "noul={:.4} is uncertain (uncertainty={:.4} < threshold {:.4})",
                        n.noul, uncertainty, self.config.confidence_threshold
                    ));
                    n.status = DecisionStatus::Review;
                }
            }
        }
    }

    /// Run a dry-run validation without evaluation
    pub fn dry_run(&self, questions: &[GateQuestion]) -> Vec<String> {
        let mut errors = Vec::new();

        if questions.is_empty() {
            errors.push("no questions provided".into());
        }

        if questions.len() > self.config.max_questions {
            errors.push(format!(
                "too many questions: {} > max {}",
                questions.len(),
                self.config.max_questions
            ));
        }

        // Check for duplicate IDs
        let mut seen = std::collections::HashSet::new();
        for q in questions {
            if !seen.insert(&q.id) {
                errors.push(format!("duplicate question ID: '{}'", q.id));
            }
        }

        errors
    }
}

/// Flip a contested Choice decision to abstain ("unknown").
///
/// If the decision is a Choice whose margin is below [`CONTESTED_MARGIN`] (see
/// [`choice_is_contested`]), the winning label is unstable, so the gate says
/// "I don't know" instead of forcing a closed-set classification: the choice
/// becomes [`ABSTAIN_OPTION`] with `needs_review = true` and
/// [`DecisionStatus::Review`]. The original probabilities and confidence are
/// kept as-is for audit. All other decisions are returned unchanged.
pub fn abstain_if_contested(decision: JevDecision) -> JevDecision {
    match decision {
        JevDecision::Choice(mut c) => {
            if c.choice != ABSTAIN_OPTION && choice_is_contested(&c) {
                let margin = c.margin;
                c.choice = ABSTAIN_OPTION.to_string();
                c.needs_review = true;
                c.status = DecisionStatus::Review;
                c.reason = Some(format!(
                    "contested margin {:.4} < {:.4}; abstained to \"{}\"",
                    margin, CONTESTED_MARGIN, ABSTAIN_OPTION
                ));
                JevDecision::Choice(c)
            } else {
                JevDecision::Choice(c)
            }
        }
        other => other,
    }
}

/// Returns true iff the decision is a Choice abstention (choice == "unknown").
pub fn is_abstained(decision: &JevDecision) -> bool {
    match decision {
        JevDecision::Choice(c) => c.choice == ABSTAIN_OPTION,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_evaluator(q: &GateQuestion) -> JevDecision {
        match &q.question_type {
            GateQuestionType::Noul { .. } => JevDecision::Noul(NoulAnswer {
                noul: 0.9,
                needs_review: false,
                reason: None,
                status: DecisionStatus::Selected,
            }),
            GateQuestionType::Choice { options, .. } => {
                let choice = options.first().cloned().unwrap_or_default();
                let mut probs = HashMap::new();
                probs.insert(choice.clone(), 0.9);
                for o in options.iter().skip(1) {
                    probs.insert(o.clone(), 0.1 / (options.len() - 1).max(1) as f64);
                }
                JevDecision::Choice(ChoiceAnswer {
                    choice,
                    probabilities: probs,
                    confidence: 0.8,
                    margin: 0.8,
                    needs_review: false,
                    reason: None,
                    status: DecisionStatus::Selected,
                })
            }
            GateQuestionType::Score { levels, .. } => {
                let legend = levels.clone();
                let mut probs = HashMap::new();
                probs.insert("0".to_string(), 0.8);
                probs.insert("1".to_string(), 0.2);
                JevDecision::Score(ScoreAnswer {
                    score: 0.0,
                    probabilities: probs,
                    confidence: 0.7,
                    legend,
                    needs_review: false,
                    reason: None,
                    status: DecisionStatus::Scored,
                })
            }
        }
    }

    #[test]
    fn test_gate_pass() {
        let gate = JevGate::default_config();
        let questions = vec![
            GateQuestion {
                id: "q1".into(),
                question_type: GateQuestionType::Noul { instructions: "test?".into() },
                priority: 1,
            },
        ];
        let result = gate.evaluate(&questions, mock_evaluator);
        assert_eq!(result.verdict, GateVerdict::Pass);
        assert_eq!(result.exit_code, ExitCode::Success);
    }

    #[test]
    fn test_gate_too_many_questions() {
        let gate = JevGate::new(GateConfig { max_questions: 2, ..GateConfig::default() });
        let questions = vec![
            GateQuestion { id: "q1".into(), question_type: GateQuestionType::Noul { instructions: "test?".into() }, priority: 1 },
            GateQuestion { id: "q2".into(), question_type: GateQuestionType::Noul { instructions: "test?".into() }, priority: 1 },
            GateQuestion { id: "q3".into(), question_type: GateQuestionType::Noul { instructions: "test?".into() }, priority: 1 },
        ];
        let result = gate.evaluate(&questions, mock_evaluator);
        assert_eq!(result.verdict, GateVerdict::Block);
        assert_eq!(result.exit_code, ExitCode::Error);
    }

    #[test]
    fn test_gate_dry_run() {
        let gate = JevGate::default_config();
        let questions = vec![
            GateQuestion { id: "q1".into(), question_type: GateQuestionType::Noul { instructions: "test?".into() }, priority: 1 },
        ];
        let errors = gate.dry_run(&questions);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_gate_dry_run_duplicate_id() {
        let gate = JevGate::default_config();
        let questions = vec![
            GateQuestion { id: "q1".into(), question_type: GateQuestionType::Noul { instructions: "test?".into() }, priority: 1 },
            GateQuestion { id: "q1".into(), question_type: GateQuestionType::Noul { instructions: "test?".into() }, priority: 1 },
        ];
        let errors = gate.dry_run(&questions);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_gate_low_confidence_triggers_review() {
        let gate = JevGate::new(GateConfig { confidence_threshold: 0.9, ..GateConfig::default() });

        let low_conf_evaluator = |_q: &GateQuestion| {
            JevDecision::Choice(ChoiceAnswer {
                choice: "a".into(),
                probabilities: [("a".into(), 0.6), ("b".into(), 0.4)].into_iter().collect(),
                confidence: 0.5, // Below threshold
                margin: 0.2,
                needs_review: false,
                reason: None,
                status: DecisionStatus::Selected,
            })
        };

        let questions = vec![
            GateQuestion { id: "q1".into(), question_type: GateQuestionType::Choice { instructions: "test?".into(), options: vec!["a".into(), "b".into()] }, priority: 1 },
        ];
        let result = gate.evaluate(&questions, low_conf_evaluator);
        assert_eq!(result.verdict, GateVerdict::Review);
    }

    #[test]
    fn test_abstain_if_contested_flips_close_call() {
        let mut probs = HashMap::new();
        probs.insert("a".to_string(), 0.51);
        probs.insert("b".to_string(), 0.49);
        let c = ChoiceAnswer::new("a".to_string(), probs.clone());
        assert!(choice_is_contested(&c));
        let confidence = c.confidence;

        let out = abstain_if_contested(JevDecision::Choice(c));
        match out {
            JevDecision::Choice(c) => {
                assert_eq!(c.choice, "unknown");
                assert!(c.needs_review);
                assert_eq!(c.status, DecisionStatus::Review);
                assert!(c
                    .reason
                    .as_deref()
                    .unwrap_or("")
                    .contains("contested margin"));
                assert_eq!(c.probabilities, probs);
                assert!((c.confidence - confidence).abs() < 1e-12);
            }
            other => panic!("expected Choice, got {:?}", other),
        }
    }

    #[test]
    fn test_abstain_if_contested_leaves_clear_winner_untouched() {
        let decision = JevDecision::Choice(ChoiceAnswer {
            choice: "a".into(),
            probabilities: [("a".into(), 0.9), ("b".into(), 0.1)].into_iter().collect(),
            confidence: 0.8,
            margin: 0.8,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        });
        let out = abstain_if_contested(decision);
        match out {
            JevDecision::Choice(c) => {
                assert_eq!(c.choice, "a");
                assert!(!c.needs_review);
                assert_eq!(c.status, DecisionStatus::Selected);
                assert!(c.reason.is_none());
            }
            other => panic!("expected Choice, got {:?}", other),
        }
    }

    #[test]
    fn test_abstain_if_contested_leaves_noul_untouched() {
        let decision = JevDecision::Noul(NoulAnswer::new(0.9));
        let out = abstain_if_contested(decision);
        match &out {
            JevDecision::Noul(n) => {
                assert!((n.noul - 0.9).abs() < 1e-12);
                assert!(!n.needs_review);
            }
            other => panic!("expected Noul, got {:?}", other),
        }
    }

    #[test]
    fn test_is_abstained() {
        let abstained = JevDecision::Choice(ChoiceAnswer {
            choice: "unknown".into(),
            probabilities: [("unknown".into(), 1.0)].into_iter().collect(),
            confidence: 1.0,
            margin: 1.0,
            needs_review: true,
            reason: None,
            status: DecisionStatus::Review,
        });
        assert!(is_abstained(&abstained));

        let chosen = JevDecision::Choice(ChoiceAnswer {
            choice: "a".into(),
            probabilities: [("a".into(), 0.9), ("b".into(), 0.1)].into_iter().collect(),
            confidence: 0.8,
            margin: 0.8,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        });
        assert!(!is_abstained(&chosen));

        let noul = JevDecision::Noul(NoulAnswer::new(0.9));
        assert!(!is_abstained(&noul));
    }
}
