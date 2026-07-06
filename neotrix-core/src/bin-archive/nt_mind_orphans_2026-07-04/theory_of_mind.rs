/// Theory of Mind — models user's beliefs, intentions, and implicit needs.
///
/// AURA-inspired: IntentFrame inserts an inference step between scene perception
/// and tool use that surfaces the implicit need behind a situated query.
/// Reference: arXiv:2606.05557 (AURA: Intent-Directed Probing)

use std::collections::HashMap;

/// An inferred user intent (literal surface level)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InferredIntent {
    Learn,
    Build,
    Debug,
    Explore,
    Analyze,
    Unknown,
}

/// Implicit need behind the literal query (AURA IntentFrame layer)
///
/// These are the "why" behind the "what" — e.g. "where is Lin Wei?" →
/// implicit needs: check availability, assess mood, decide interruptibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImplicitNeed {
    /// User needs clarification before they know what to ask
    ClarifyGoal,
    /// User wants to know if something is possible
    AssessFeasibility,
    /// User needs to know constraints/limits before deciding
    CheckConstraints,
    /// User wants to understand consequences before committing
    UnderstandImpact,
    /// User wants to compare options
    CompareAlternatives,
    /// User wants to know why something is the way it is
    LearnRationale,
    /// User wants to surface hidden risks
    SurfaceRisks,
    /// User wants confirmation before taking action
    SeekApproval,
    /// User wants secondary verification (not self-trust)
    VerifyInformation,
    /// No implicit need detected (literal is sufficient)
    None,
}

/// AURA-style intent frame: structured estimate of the implicit need
/// with a scalar gap score controlling probe budget and tool selection.
#[derive(Debug, Clone)]
pub struct IntentFrame {
    /// Surface-level intent
    pub literal_intent: InferredIntent,
    /// Implicit needs surfaced by gap calibration
    pub implicit_needs: Vec<ImplicitNeed>,
    /// Gap score [0,1]: how much the implicit need diverges from literal.
    /// 0.0 = literal query is sufficient, 1.0 = completely different need.
    pub gap_score: f64,
    /// Probe budget: how many probing actions to surface the real need.
    /// gap=0.0 → budget=0 (no probes needed), gap=1.0 → budget=5 (deep probing)
    pub probe_budget: usize,
    /// Confidence in the frame [0,1]
    pub confidence: f64,
}

impl IntentFrame {
    pub fn new(literal_intent: InferredIntent) -> Self {
        Self {
            literal_intent,
            implicit_needs: Vec::new(),
            gap_score: 0.0,
            probe_budget: 0,
            confidence: 0.5,
        }
    }

    /// Whether probing is needed before acting
    pub fn needs_probing(&self) -> bool {
        self.probe_budget > 0 && self.confidence < 0.8
    }

    /// Whether the frame has identified any implicit need
    pub fn has_implicit_need(&self) -> bool {
        !self.implicit_needs.is_empty()
    }
}

/// Gap calibrator — determines the divergence between literal query
/// and implicit need, and computes probe budget accordingly.
#[derive(Debug, Clone)]
pub struct GapCalibrator {
    /// Weight for each implicit need signal
    need_weights: HashMap<ImplicitNeed, f64>,
}

impl Default for GapCalibrator {
    fn default() -> Self {
        let mut need_weights = HashMap::new();
        need_weights.insert(ImplicitNeed::ClarifyGoal, 0.4);
        need_weights.insert(ImplicitNeed::AssessFeasibility, 0.6);
        need_weights.insert(ImplicitNeed::CheckConstraints, 0.5);
        need_weights.insert(ImplicitNeed::UnderstandImpact, 0.7);
        need_weights.insert(ImplicitNeed::CompareAlternatives, 0.5);
        need_weights.insert(ImplicitNeed::LearnRationale, 0.3);
        need_weights.insert(ImplicitNeed::SurfaceRisks, 0.8);
        need_weights.insert(ImplicitNeed::SeekApproval, 0.4);
        need_weights.insert(ImplicitNeed::VerifyInformation, 0.6);
        need_weights.insert(ImplicitNeed::None, 0.0);
        Self { need_weights }
    }
}

impl GapCalibrator {
    pub fn new() -> Self { Self::default() }

    /// Calibrate the gap between a query's literal content and
    /// its probable implicit needs.
    ///
    /// Uses keyword heuristics to surface common implicit needs:
    /// - "can I", "is it possible" → AssessFeasibility
    /// - "should I", "worth" → SeekApproval
    /// - "what if", "risks" → SurfaceRisks
    /// - "why", "reason", "rationale" → LearnRationale
    /// - "vs", "or", "alternative" → CompareAlternatives
    /// - "confirm", "verify", "double-check" → VerifyInformation
    /// - "constraint", "limit", "require" → CheckConstraints
    /// - "impact", "affect", "consequence" → UnderstandImpact
    /// - "what do I need", "how to" → ClarifyGoal
    pub fn calibrate(&self, query: &str, intent: InferredIntent) -> IntentFrame {
        let q = query.to_lowercase();
        let mut frame = IntentFrame::new(intent);
        let mut gap_contributions = Vec::new();

        let patterns: Vec<(ImplicitNeed, Vec<&str>)> = vec![
            (ImplicitNeed::AssessFeasibility, vec!["can i", "is it possible", "could i", "is there a way"]),
            (ImplicitNeed::SeekApproval, vec!["should i", "worth", "recommend", "good idea"]),
            (ImplicitNeed::SurfaceRisks, vec!["what if", "risk", "danger", "downside", "worst case"]),
            (ImplicitNeed::LearnRationale, vec!["why", "reason", "rationale", "motivation", "purpose"]),
            (ImplicitNeed::CompareAlternatives, vec![" vs ", " or ", "alternative", "compared to", "better"]),
            (ImplicitNeed::VerifyInformation, vec!["confirm", "verify", "double-check", "check if", "validate"]),
            (ImplicitNeed::CheckConstraints, vec!["constraint", "limit", "require", "prerequisite", "dependency"]),
            (ImplicitNeed::UnderstandImpact, vec!["impact", "affect", "consequence", "effect on", "change"]),
            (ImplicitNeed::ClarifyGoal, vec!["what do i need", "how to", "where to start", "what should i"]),
        ];

        for (need, triggers) in &patterns {
            if triggers.iter().any(|t| q.contains(t)) {
                frame.implicit_needs.push(*need);
                let weight = self.need_weights.get(need).copied().unwrap_or(0.3);
                gap_contributions.push(weight);
            }
        }

        // Compute gap score as weighted mean of detected needs
        frame.gap_score = if gap_contributions.is_empty() {
            0.0
        } else {
            let sum: f64 = gap_contributions.iter().sum();
            (sum / gap_contributions.len() as f64).clamp(0.0, 1.0)
        };

        // Probe budget: 0 for literal, up to 5 for high gap
        frame.probe_budget = match frame.gap_score {
            _ if frame.gap_score >= 0.8 => 5,
            _ if frame.gap_score >= 0.6 => 4,
            _ if frame.gap_score >= 0.4 => 3,
            _ if frame.gap_score >= 0.2 => 2,
            _ if frame.gap_score >= 0.1 => 1,
            _ => 0,
        };

        // Confidence inversely related to gap (high gap = less certain)
        frame.confidence = (1.0 - frame.gap_score * 0.5).max(0.3);

        frame
    }
}

/// User mental model snapshot
#[derive(Debug, Clone)]
pub struct MentalModel {
    pub inferred_intent: InferredIntent,
    pub intent_frame: IntentFrame,
    pub domain_expertise: f64,
    pub verbosity_preference: f64,
    pub technical_depth: f64,
    pub confusion_signals: f64,
    pub confidence: f64,
}

/// Theory of Mind engine — unified user model with AURA intent inference
#[derive(Debug, Clone)]
pub struct TheoryOfMind {
    pub session_intents: Vec<(String, IntentFrame)>,
    pub confusion_history: Vec<(String, f64)>,
    pub max_history: usize,
    calibrator: GapCalibrator,
}

impl TheoryOfMind {
    pub fn new() -> Self {
        Self {
            session_intents: Vec::new(),
            confusion_history: Vec::new(),
            max_history: 50,
            calibrator: GapCalibrator::new(),
        }
    }

    /// Infer user intent and build IntentFrame from a query
    pub fn infer(&mut self, query: &str) -> IntentFrame {
        let literal = self.literal_intent(query);
        let frame = self.calibrator.calibrate(query, literal);
        self.session_intents.push((query.to_string(), frame.clone()));
        if self.session_intents.len() > self.max_history {
            self.session_intents.remove(0);
        }
        frame
    }

    /// Surface-level intent classification (original behavior)
    pub fn literal_intent(&self, task: &str) -> InferredIntent {
        let task_lower = task.to_lowercase();
        if task_lower.contains("learn") || task_lower.contains("understand")
            || task_lower.contains("explain") || task_lower.contains("what is")
            || task_lower.contains("how does")
        {
            InferredIntent::Learn
        } else if task_lower.contains("build") || task_lower.contains("create")
            || task_lower.contains("implement") || task_lower.contains("write")
            || task_lower.contains("make")
        {
            InferredIntent::Build
        } else if task_lower.contains("fix") || task_lower.contains("bug")
            || task_lower.contains("debug") || task_lower.contains("error")
            || task_lower.contains("issue") || task_lower.contains("not working")
        {
            InferredIntent::Debug
        } else if task_lower.contains("search") || task_lower.contains("find")
            || task_lower.contains("research") || task_lower.contains("look up")
        {
            InferredIntent::Explore
        } else if task_lower.contains("analyze") || task_lower.contains("compare")
            || task_lower.contains("evaluate") || task_lower.contains("review")
        {
            InferredIntent::Analyze
        } else {
            InferredIntent::Unknown
        }
    }

    /// Build a complete mental model from the latest IntentFrame
    pub fn build_model(&self, frame: &IntentFrame) -> MentalModel {
        MentalModel {
            inferred_intent: frame.literal_intent,
            intent_frame: frame.clone(),
            domain_expertise: 0.5,
            verbosity_preference: 0.5,
            technical_depth: 0.5,
            confusion_signals: 0.0,
            confidence: frame.confidence,
        }
    }

    /// Record a possible confusion signal (repeated requests, frustration words)
    pub fn record_confusion(&mut self, task: &str) {
        let signal = if task.contains("again") || task.contains("still")
            || task.contains("not right") || task.contains("wrong")
        {
            0.8
        } else {
            0.2
        };
        self.confusion_history.push((task.to_string(), signal));
        if self.confusion_history.len() > self.max_history {
            self.confusion_history.remove(0);
        }
    }

    /// Get the most frequent literal intent in recent history
    pub fn dominant_intent(&self, last_n: usize) -> InferredIntent {
        let recent: Vec<_> = self.session_intents.iter().rev().take(last_n).collect();
        if recent.is_empty() {
            return InferredIntent::Unknown;
        }
        let mut counts: HashMap<InferredIntent, usize> = HashMap::new();
        for (_, frame) in &recent {
            *counts.entry(frame.literal_intent).or_insert(0) += 1;
        }
        counts.into_iter().max_by_key(|&(_, c)| c).map(|(i, _)| i).unwrap_or(InferredIntent::Unknown)
    }

    /// Current confusion level (0.0-1.0)
    pub fn confusion_level(&self, last_n: usize) -> f64 {
        let recent: Vec<_> = self.confusion_history.iter().rev().take(last_n).collect();
        if recent.is_empty() { return 0.0; }
        recent.iter().map(|(_, s)| s).sum::<f64>() / recent.len() as f64
    }

    /// Expose calibrator for external gap calibration
    pub fn calibrator(&self) -> &GapCalibrator {
        &self.calibrator
    }
}

impl Default for TheoryOfMind {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_intent_learn() {
        let tom = TheoryOfMind::new();
        assert_eq!(tom.literal_intent("learn about quantum computing"), InferredIntent::Learn);
    }

    #[test]
    fn test_infer_returns_intent_frame() {
        let mut tom = TheoryOfMind::new();
        let frame = tom.infer("build a REST API");
        assert_eq!(frame.literal_intent, InferredIntent::Build);
        assert!(frame.confidence > 0.0);
    }

    #[test]
    fn test_gap_calibration_assess_feasibility() {
        let cal = GapCalibrator::new();
        let frame = cal.calibrate("can I deploy this to production?", InferredIntent::Unknown);
        assert!(frame.implicit_needs.contains(&ImplicitNeed::AssessFeasibility));
        assert!(frame.gap_score > 0.0);
    }

    #[test]
    fn test_gap_calibration_risks() {
        let cal = GapCalibrator::new();
        let frame = cal.calibrate("what are the risks of upgrading?", InferredIntent::Analyze);
        assert!(frame.implicit_needs.contains(&ImplicitNeed::SurfaceRisks));
        assert!(frame.probe_budget >= 3);
    }

    #[test]
    fn test_gap_calibration_seek_approval() {
        let cal = GapCalibrator::new();
        let frame = cal.calibrate("should I refactor this module?", InferredIntent::Build);
        assert!(frame.implicit_needs.contains(&ImplicitNeed::SeekApproval));
    }

    #[test]
    fn test_gap_calibration_verify() {
        let cal = GapCalibrator::new();
        let frame = cal.calibrate("double-check the encryption logic", InferredIntent::Debug);
        assert!(frame.implicit_needs.contains(&ImplicitNeed::VerifyInformation));
    }

    #[test]
    fn test_no_implicit_need_for_literal_query() {
        let cal = GapCalibrator::new();
        let frame = cal.calibrate("implement login feature", InferredIntent::Build);
        assert!(!frame.has_implicit_need());
        assert_eq!(frame.probe_budget, 0);
    }

    #[test]
    fn test_probe_budget_scales_with_gap() {
        let cal = GapCalibrator::new();
        let low = cal.calibrate("implement login feature", InferredIntent::Build);
        let high = cal.calibrate("what if the migration fails?", InferredIntent::Analyze);
        assert!(low.probe_budget < high.probe_budget);
    }

    #[test]
    fn test_needs_probing() {
        let cal = GapCalibrator::new();
        let frame = cal.calibrate("what if the migration fails?", InferredIntent::Analyze);
        assert!(frame.needs_probing());
    }

    #[test]
    fn test_dominant_intent() {
        let mut tom = TheoryOfMind::new();
        tom.infer("build");
        tom.infer("fix bug");
        tom.infer("build more");
        assert_eq!(tom.dominant_intent(3), InferredIntent::Build);
    }

    #[test]
    fn test_confusion() {
        let mut tom = TheoryOfMind::new();
        tom.record_confusion("still not working");
        assert!(tom.confusion_level(5) > 0.0);
    }

    #[test]
    fn test_build_model() {
        let mut tom = TheoryOfMind::new();
        let frame = tom.infer("debug the crash in module X");
        let model = tom.build_model(&frame);
        assert_eq!(model.inferred_intent, InferredIntent::Debug);
        assert_eq!(model.intent_frame.literal_intent, InferredIntent::Debug);
    }

    #[test]
    fn test_multi_need_accumulation() {
        let cal = GapCalibrator::new();
        let frame = cal.calibrate(
            "why is this failing? what are the risks? can I fix it?",
            InferredIntent::Debug,
        );
        assert!(frame.implicit_needs.len() >= 2);
        assert!(frame.gap_score > 0.3);
    }

    #[test]
    fn test_confidence_inversely_related_to_gap() {
        let cal = GapCalibrator::new();
        let low_gap = cal.calibrate("implement login", InferredIntent::Build);
        let high_gap = cal.calibrate("what if the migration fails?", InferredIntent::Analyze);
        assert!(low_gap.confidence > high_gap.confidence);
    }
}
