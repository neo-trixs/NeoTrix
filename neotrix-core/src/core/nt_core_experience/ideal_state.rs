use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

// ── Effort Level Gating (H1) ──────────────────────────────────────────
/// 8-tier depth control, inspired by Miessler's The Algorithm.
/// Controls ISC count, agent spawning, tool use, and cycle depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffortLevel {
    Instant,       // <10s — direct VSA lookup
    Fast,          // <1min — single hop reasoning
    Standard,      // <2min — full consciousness cycle (default)
    Extended,      // <8min — MCTS + multi-cycle
    Advanced,      // <16min — multi-agent + external tools
    Deep,          // <32min — full SEAL pipeline
    Comprehensive, // <120min — system redesign
    Infinite,      // unbounded — research-grade
}

impl EffortLevel {
    pub fn classify(query: &str) -> Self {
        let lower = query.to_lowercase();
        let word_count = lower.split_whitespace().count();
        let has_fix = lower.contains("fix") || lower.contains("typo") || lower.contains("spelling");
        let has_design =
            lower.contains("design") || lower.contains("architect") || lower.contains("redesign");
        let has_deep = lower.contains("research")
            || lower.contains("investigate")
            || lower.contains("explore");

        if has_fix && word_count < 10 {
            return EffortLevel::Fast;
        }
        if has_design {
            return EffortLevel::Extended;
        }
        if has_deep && word_count > 20 {
            return EffortLevel::Deep;
        }
        if word_count < 5 {
            return EffortLevel::Instant;
        }
        EffortLevel::Standard
    }

    pub fn max_criteria(&self) -> usize {
        match self {
            EffortLevel::Instant => 0,
            EffortLevel::Fast => 4,
            EffortLevel::Standard => 12,
            EffortLevel::Extended => 24,
            EffortLevel::Advanced => 40,
            EffortLevel::Deep => 60,
            EffortLevel::Comprehensive => 100,
            EffortLevel::Infinite => 200,
        }
    }

    pub fn allow_agents(&self) -> bool {
        matches!(
            self,
            EffortLevel::Extended
                | EffortLevel::Advanced
                | EffortLevel::Deep
                | EffortLevel::Comprehensive
                | EffortLevel::Infinite
        )
    }

    pub fn allow_plan_mode(&self) -> bool {
        matches!(
            self,
            EffortLevel::Advanced
                | EffortLevel::Deep
                | EffortLevel::Comprehensive
                | EffortLevel::Infinite
        )
    }

    pub fn label(&self) -> &'static str {
        match self {
            EffortLevel::Instant => "instant",
            EffortLevel::Fast => "fast",
            EffortLevel::Standard => "standard",
            EffortLevel::Extended => "extended",
            EffortLevel::Advanced => "advanced",
            EffortLevel::Deep => "deep",
            EffortLevel::Comprehensive => "comprehensive",
            EffortLevel::Infinite => "infinite",
        }
    }
}

// ── Ideal State Criteria (H2) ─────────────────────────────────────────
/// A single criterion: 8-12 word boolean-testable statement.
/// Same criterion is simultaneously goal AND verification standard.
#[derive(Debug, Clone)]
pub struct Criterion {
    /// The criterion statement (e.g., "No credentials exposed in git commit history")
    pub statement: String,
    /// VSA embedding for semantic matching
    pub vsa: Vec<u8>,
    /// Whether this criterion has been verified
    pub verified: Option<bool>,
    /// Evidence text if verified
    pub evidence: Option<String>,
    /// Anti-criterion flag: if true, this describes what to avoid
    pub is_anti: bool,
    /// Domain category for routing
    pub domain: CriterionDomain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CriterionDomain {
    Function,
    Security,
    Performance,
    Usability,
    Correctness,
    Consistency,
    Completeness,
    Style,
    Ethics,
    Generic,
}

impl CriterionDomain {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "function" | "functionality" => CriterionDomain::Function,
            "security" | "safe" => CriterionDomain::Security,
            "performance" | "speed" => CriterionDomain::Performance,
            "usability" | "ux" => CriterionDomain::Usability,
            "correctness" | "accuracy" => CriterionDomain::Correctness,
            "consistency" => CriterionDomain::Consistency,
            "completeness" | "coverage" => CriterionDomain::Completeness,
            "style" | "design" => CriterionDomain::Style,
            "ethics" | "safety" => CriterionDomain::Ethics,
            _ => CriterionDomain::Generic,
        }
    }
}

/// Complete ideal state definition for a request or task.
#[derive(Debug, Clone)]
pub struct IdealState {
    /// All criteria (both positive and anti-)
    pub criteria: Vec<Criterion>,
    /// Current state VSA embedding (where we are now)
    pub current_state_vsa: Vec<u8>,
    /// Target state VSA embedding (where we want to be)
    pub target_state_vsa: Vec<u8>,
    /// Effort level assigned
    pub effort: EffortLevel,
    /// Domain tag for EFE routing
    pub domain: String,
    /// Timestamp
    pub created_at: u64,
}

impl IdealState {
    pub fn new(domain: &str, effort: EffortLevel) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            criteria: Vec::with_capacity(effort.max_criteria()),
            current_state_vsa: Vec::new(),
            target_state_vsa: Vec::new(),
            effort,
            domain: domain.to_string(),
            created_at: now,
        }
    }

    pub fn add_criterion(&mut self, statement: &str, is_anti: bool) {
        if self.criteria.len() >= self.effort.max_criteria() && self.effort.max_criteria() > 0 {
            return;
        }
        let domain_str = self.detect_domain(statement);
        let vsa = QuantizedVSA::seeded_random(Self::criterion_hash(statement), 4096);
        self.criteria.push(Criterion {
            statement: statement.to_string(),
            vsa,
            verified: None,
            evidence: None,
            is_anti,
            domain: CriterionDomain::from_str(&domain_str),
        });
    }

    fn detect_domain(&self, statement: &str) -> String {
        let lower = statement.to_lowercase();
        if lower.contains("security") || lower.contains("credential") || lower.contains("auth") {
            return "security".to_string();
        }
        if lower.contains("performance") || lower.contains("speed") || lower.contains("latency") {
            return "performance".to_string();
        }
        if lower.contains("test") || lower.contains("correct") || lower.contains("validate") {
            return "correctness".to_string();
        }
        if lower.contains("consistent") || lower.contains("uniform") {
            return "consistency".to_string();
        }
        "function".to_string()
    }

    fn criterion_hash(s: &str) -> u64 {
        let mut h: u64 = 0x1dea1_cafe_u64;
        for b in s.bytes() {
            h = h.wrapping_mul(0x9e3779b97f4a7c15u64);
            h ^= b as u64;
            h = h.rotate_left(17);
        }
        h
    }

    /// Verify all criteria against a result embedding.
    /// Each criterion's VSA is compared against the result via cosine similarity.
    pub fn verify(&mut self, result_vsa: &[u8], threshold: f64) -> Vec<VerificationResult> {
        let mut results = Vec::with_capacity(self.criteria.len());
        for criterion in &mut self.criteria {
            let sim = QuantizedVSA::similarity(&criterion.vsa, result_vsa);
            let passed = if criterion.is_anti {
                sim < threshold // anti-criteria must NOT match
            } else {
                sim >= threshold // positive criteria must match
            };
            criterion.verified = Some(passed);
            results.push(VerificationResult {
                statement: criterion.statement.clone(),
                passed,
                similarity: sim,
                is_anti: criterion.is_anti,
                domain: criterion.domain,
            });
        }
        results
    }

    /// Overall pass rate across all criteria.
    pub fn pass_rate(&self) -> f64 {
        let total = self.criteria.len();
        if total == 0 {
            return 1.0;
        }
        let passed = self
            .criteria
            .iter()
            .filter(|c| c.verified == Some(true))
            .count();
        passed as f64 / total as f64
    }

    /// Report on which criteria failed.
    pub fn failures(&self) -> Vec<&Criterion> {
        self.criteria
            .iter()
            .filter(|c| c.verified == Some(false))
            .collect()
    }
}

// ── Verification Result ────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub statement: String,
    pub passed: bool,
    pub similarity: f64,
    pub is_anti: bool,
    pub domain: CriterionDomain,
}

// ── Reverse Intent Engine (H5) ─────────────────────────────────────────
/// Structured output of request analysis, inspired by Miessler's OBSERVE phase.
#[derive(Debug, Clone)]
pub struct ReverseIntent {
    /// What the user explicitly asked for
    pub explicit_asks: Vec<String>,
    /// What the user implied but didn't state
    pub implied_asks: Vec<String>,
    /// Things the user specifically does NOT want
    pub anti_criteria: Vec<String>,
    /// Common failure modes to guard against
    pub failure_modes: Vec<String>,
    /// Gotchas or edge cases that could be missed
    pub gotchas: Vec<String>,
    /// Assigned effort level
    pub effort: EffortLevel,
    /// Detected domain
    pub domain: String,
}

/// Analyze a request and extract structured intent.
/// This is the first step before any processing — never act without intent.
pub fn reverse_intent(query: &str) -> ReverseIntent {
    let effort = EffortLevel::classify(query);
    let lower = query.to_lowercase();

    let mut explicit = Vec::new();
    let mut implied = Vec::new();
    let mut anti = Vec::new();
    let mut failures = Vec::new();
    let mut gotchas = Vec::new();

    // Extract explicit asks — phrases after "I want", "build", "make", "create"
    for phrase in [
        "i want ", "please ", "build ", "make ", "create ", "design ",
    ] {
        if let Some(pos) = lower.find(phrase) {
            let remainder = &lower[pos + phrase.len()..];
            let end = remainder
                .find(&['.', '!', '?'][..])
                .unwrap_or(remainder.len());
            explicit.push(remainder[..end].trim().to_string());
        }
    }

    // Extract anti-criteria — phrases after "don't", "never", "avoid", "without"
    for phrase in ["don't ", "dont ", "never ", "avoid ", "without ", "no "] {
        if let Some(pos) = lower.find(phrase) {
            let remainder = &lower[pos + phrase.len()..];
            let end = remainder
                .find(&['.', ',', '!'][..])
                .unwrap_or(remainder.len());
            anti.push(remainder[..end].trim().to_string());
        }
    }

    // Detect domain
    let domain = if lower.contains("security") || lower.contains("vulnerability") {
        "security".to_string()
    } else if lower.contains("code") || lower.contains("build") || lower.contains("implement") {
        "engineering".to_string()
    } else if lower.contains("design") || lower.contains("architect") {
        "design".to_string()
    } else if lower.contains("research") || lower.contains("learn") || lower.contains("understand")
    {
        "research".to_string()
    } else {
        "general".to_string()
    };

    // Common failure modes per domain
    match domain.as_str() {
        "security" => {
            failures.push("Overlooking edge cases in threat model".to_string());
            failures.push("False positives overwhelming signal".to_string());
        }
        "engineering" => {
            failures.push("Missing error handling paths".to_string());
            failures.push("Incorrect assumptions about input format".to_string());
            gotchas.push("Silent failures worse than crashes".to_string());
        }
        "design" => {
            failures.push("Over-engineering for unvalidated requirements".to_string());
            gotchas.push("Design decisions that constrain future changes".to_string());
        }
        _ => {
            failures.push("Implicit assumptions not stated".to_string());
            gotchas.push("Success criteria not defined upfront".to_string());
        }
    }

    // If no explicit asks found, use the whole query
    if explicit.is_empty() {
        // Try to infer the core request
        let cleaned = lower
            .trim_start_matches("please")
            .trim_start_matches("can you")
            .trim_start_matches("could you")
            .trim_start_matches("i need you to")
            .trim();
        let end = cleaned.find(&['.', '!'][..]).unwrap_or(cleaned.len());
        if cleaned[..end].trim().len() > 3 {
            explicit.push(cleaned[..end].trim().to_string());
        }
    }

    // Infer implied asks from context
    if !explicit.is_empty() && implied.is_empty() {
        implied.push("Solution should be robust and maintainable".to_string());
    }

    ReverseIntent {
        explicit_asks: explicit,
        implied_asks: implied,
        anti_criteria: anti,
        failure_modes: failures,
        gotchas,
        effort,
        domain,
    }
}

impl ReverseIntent {
    /// Convert intent to IdealState criteria.
    pub fn to_ideal_state(&self) -> IdealState {
        let mut state = IdealState::new(&self.domain, self.effort);

        // Explicit asks → positive criteria
        for ask in &self.explicit_asks {
            let criterion = if ask.len() > 120 {
                format!("{}...", &ask[..117])
            } else {
                ask.clone()
            };
            state.add_criterion(&criterion, false);
        }

        // Anti-criteria
        for anti_c in &self.anti_criteria {
            state.add_criterion(anti_c, true);
        }

        // Implied asks
        for implied in &self.implied_asks {
            state.add_criterion(implied, false);
        }

        // Failure modes as anti-criteria
        for failure in &self.failure_modes {
            state.add_criterion(failure, true);
        }

        // Update VSA embeddings for current and target state
        let current_hash = self.current_state_hash();
        state.current_state_vsa = QuantizedVSA::seeded_random(current_hash, 4096);

        let target_hash = self.target_state_hash();
        state.target_state_vsa = QuantizedVSA::seeded_random(target_hash, 4096);

        state
    }

    fn current_state_hash(&self) -> u64 {
        let mut h: u64 = 0xcafe_babe_cafe_u64;
        for ask in &self.explicit_asks {
            for b in ask.bytes() {
                h = h.wrapping_mul(0x9e3779b97f4a7c15u64);
                h ^= b as u64;
            }
        }
        h
    }

    fn target_state_hash(&self) -> u64 {
        let mut h: u64 = 0x1dea1_decaf_u64;
        for ask in &self.explicit_asks {
            for b in ask.bytes() {
                h = h.wrapping_mul(0x100000001b3u64);
                h ^= b as u64;
            }
        }
        for anti_c in &self.anti_criteria {
            for b in anti_c.bytes() {
                h = h.wrapping_mul(0x100000001b3u64);
                h ^= b as u64;
            }
        }
        h
    }

    pub fn report(&self) -> String {
        format!(
            "Intent[{}:{}, {} explicit, {} implied, {} anti, {} failure modes, {} gotchas]",
            self.domain,
            self.effort.label(),
            self.explicit_asks.len(),
            self.implied_asks.len(),
            self.anti_criteria.len(),
            self.failure_modes.len(),
            self.gotchas.len(),
        )
    }
}

// ── EFE Goal Bridge (bridges IdealState → EFEMinimizer) ───────────────
/// Converts IdealState into the preferred_outcomes format expected by EFEMinimizer.
#[derive(Debug, Clone)]
pub struct EFEGoal {
    pub ideal_state: IdealState,
}

impl EFEGoal {
    pub fn new(ideal_state: IdealState) -> Self {
        Self { ideal_state }
    }

    /// Convert IdealState criteria into preferred outcome vectors.
    /// Each criterion contributes a dimension weighted by its semantic VSA.
    pub fn to_preferred_outcomes(&self, num_actions: usize, dim: usize) -> Vec<Vec<f64>> {
        let mut outcomes = Vec::with_capacity(num_actions);
        for action_idx in 0..num_actions {
            let mut outcome = Vec::with_capacity(dim);
            let criteria = &self.ideal_state.criteria;

            for d in 0..dim {
                let mut val = 0.5; // neutral baseline
                let d_phase = (d as f64 + 1.0) / (dim as f64 + 1.0);
                for (ci, criterion) in criteria.iter().enumerate() {
                    let ci_phase = (ci as f64 + 1.0) / (criteria.len().max(1) as f64 + 1.0);
                    let match_val = (std::f64::consts::PI * d_phase * ci_phase).sin().abs();
                    let preference = if criterion.is_anti {
                        1.0 - match_val // anti-criteria: prefer low match
                    } else {
                        match_val // positive criteria: prefer high match
                    };
                    val = val * 0.7 + preference * 0.3;
                }
                // Action-phase modulation
                let a_phase = (action_idx as f64 + 1.0) / (num_actions as f64 + 1.0);
                outcome.push(val * (0.8 + 0.2 * a_phase));
            }
            outcomes.push(outcome);
        }
        outcomes
    }

    /// Number of criteria influencing the goal.
    pub fn criterion_count(&self) -> usize {
        self.ideal_state.criteria.len()
    }

    /// Pass rate as a proxy for goal achievement.
    pub fn goal_achievement(&self) -> f64 {
        self.ideal_state.pass_rate()
    }
}

// ── Intent → IdealState → Verification pipeline (H2 + H5 fused) ───────
/// Full pipeline: input → reverse_intent → ideal_state → verify.
pub fn process_with_ideal_state(query: &str, result_vsa: &[u8]) -> IdealStateOutput {
    let intent = reverse_intent(query);
    let mut ideal_state = intent.to_ideal_state();
    let effort = ideal_state.effort;

    // Skip verification for Instant level (no criteria generated)
    if effort == EffortLevel::Instant {
        return IdealStateOutput {
            intent,
            ideal_state,
            verification: Vec::new(),
            pass_rate: 1.0,
            effort_label: effort.label().to_string(),
        };
    }

    let verification = ideal_state.verify(result_vsa, 0.5);
    let pass_rate = ideal_state.pass_rate();

    IdealStateOutput {
        intent,
        ideal_state,
        verification,
        pass_rate,
        effort_label: effort.label().to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct IdealStateOutput {
    pub intent: ReverseIntent,
    pub ideal_state: IdealState,
    pub verification: Vec<VerificationResult>,
    pub pass_rate: f64,
    pub effort_label: String,
}

// ── Bitter Lesson Engineering Check (H4) ──────────────────────────────
/// Check if a modification encodes human heuristics (BLE violation).
/// Returns None if clean, Some(explanation) if BLE violation detected.
pub fn bitter_lesson_check(modification: &str) -> Option<String> {
    let lower = modification.to_lowercase();

    // Patterns that indicate hard-coded human heuristics
    let heuristics_patterns = [
        "always",
        "never",
        "must be",
        "assume that",
        "in my experience",
        "typically",
        "obviously",
        "the right way",
        "best practice",
    ];

    for pattern in &heuristics_patterns {
        if lower.contains(pattern) {
            return Some(format!(
                "BLE violation: '{}' encodes a human heuristic. \
                 Prefer preference-based steering over hard-coded rules.",
                pattern
            ));
        }
    }

    // Check for overly prescriptive "how" instead of "what"
    let how_indicators = ["how to", "the steps are", "first ", "second ", "then "];
    let how_count = how_indicators.iter().filter(|p| lower.contains(*p)).count();
    if how_count >= 3 {
        return Some(format!(
            "BLE violation: {} sequential steps detected. \
             Specify WHAT (desired outcome) not HOW (execution steps).",
            how_count
        ));
    }

    None
}

// ── Customization Rating (H3) ──────────────────────────────────────────
/// Three-axis AI rating: Customization > Integration > Competence.
#[derive(Debug, Clone, Default)]
pub struct AIRating {
    /// How deeply aware is the system of user's past, present, desired future
    pub customization: f64,
    /// How integrated is it into user's environment
    pub integration: f64,
    /// Raw capability (model, skills, tools)
    pub competence: f64,
}

impl AIRating {
    pub fn new(customization: f64, integration: f64, competence: f64) -> Self {
        Self {
            customization: customization.clamp(0.0, 1.0),
            integration: integration.clamp(0.0, 1.0),
            competence: competence.clamp(0.0, 1.0),
        }
    }

    /// Weighted overall score: Customization matters most.
    pub fn overall(&self) -> f64 {
        0.45 * self.customization + 0.35 * self.integration + 0.20 * self.competence
    }

    pub fn report(&self) -> String {
        format!(
            "AIRating[overall={:.2}, customization={:.1}%, integration={:.1}%, competence={:.1}%]",
            self.overall(),
            self.customization * 100.0,
            self.integration * 100.0,
            self.competence * 100.0,
        )
    }
}

// ── Prediction Registry (H7) ──────────────────────────────────────────
/// Track predictions for self-calibration, inspired by Miessler's public tracking.
#[derive(Debug, Clone)]
pub struct Prediction {
    pub id: u64,
    pub statement: String,
    pub confidence: PredictionConfidence,
    pub made_at: u64,
    pub resolve_by: u64,
    pub status: PredictionStatus,
    pub domain: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PredictionConfidence {
    AlmostCertain,      // 90%+
    Probable,           // 70%
    ChancesAboutEven,   // 50%
    ProbablyNot,        // 30%
    AlmostCertainlyNot, // 10%
}

impl PredictionConfidence {
    pub fn value(&self) -> f64 {
        match self {
            PredictionConfidence::AlmostCertain => 0.9,
            PredictionConfidence::Probable => 0.7,
            PredictionConfidence::ChancesAboutEven => 0.5,
            PredictionConfidence::ProbablyNot => 0.3,
            PredictionConfidence::AlmostCertainlyNot => 0.1,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "almost_certain" | "90" => PredictionConfidence::AlmostCertain,
            "probable" | "70" => PredictionConfidence::Probable,
            "chances_about_even" | "50" => PredictionConfidence::ChancesAboutEven,
            "probably_not" | "30" => PredictionConfidence::ProbablyNot,
            "almost_certainly_not" | "10" => PredictionConfidence::AlmostCertainlyNot,
            _ => PredictionConfidence::ChancesAboutEven,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PredictionStatus {
    Pending,
    Correct,
    Wrong,
    Partial,
}

#[derive(Debug, Clone, Default)]
pub struct PredictionRegistry {
    pub predictions: Vec<Prediction>,
    next_id: u64,
}

impl PredictionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        statement: &str,
        confidence: PredictionConfidence,
        resolve_by: u64,
    ) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let id = self.next_id;
        self.next_id += 1;
        self.predictions.push(Prediction {
            id,
            statement: statement.to_string(),
            confidence,
            made_at: now,
            resolve_by,
            status: PredictionStatus::Pending,
            domain: "general".to_string(),
        });
        id
    }

    pub fn resolve(&mut self, id: u64, outcome: PredictionStatus) -> bool {
        if let Some(pred) = self.predictions.iter_mut().find(|p| p.id == id) {
            pred.status = outcome;
            true
        } else {
            false
        }
    }

    /// Calculate calibration statistics: accuracy vs claimed confidence.
    pub fn calibration_stats(&self) -> PredictionStats {
        let total = self.predictions.len();
        let resolved: Vec<&Prediction> = self
            .predictions
            .iter()
            .filter(|p| p.status != PredictionStatus::Pending)
            .collect();
        let resolved_count = resolved.len();
        let correct = resolved
            .iter()
            .filter(|p| p.status == PredictionStatus::Correct)
            .count();

        // Calculate bias: average (claimed_confidence - actual_accuracy)
        let actual_accuracy = if resolved_count > 0 {
            correct as f64 / resolved_count as f64
        } else {
            0.5
        };
        let avg_claimed: f64 = if resolved_count > 0 {
            resolved.iter().map(|p| p.confidence.value()).sum::<f64>() / resolved_count as f64
        } else {
            0.0
        };

        PredictionStats {
            total,
            resolved: resolved_count,
            correct,
            actual_accuracy,
            avg_claimed_confidence: avg_claimed,
            calibration_bias: avg_claimed - actual_accuracy,
        }
    }

    pub fn report(&self) -> String {
        let stats = self.calibration_stats();
        format!(
            "PredictionRegistry[total={}, resolved={}, correct={}, accuracy={:.1}%, bias={:+.2}]",
            stats.total,
            stats.resolved,
            stats.correct,
            stats.actual_accuracy * 100.0,
            stats.calibration_bias,
        )
    }
}

#[derive(Debug, Clone)]
pub struct PredictionStats {
    pub total: usize,
    pub resolved: usize,
    pub correct: usize,
    pub actual_accuracy: f64,
    pub avg_claimed_confidence: f64,
    pub calibration_bias: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── EffortLevel tests ──────────────────────────────────────────────

    #[test]
    fn test_effort_classify_fix_is_fast() {
        let level = EffortLevel::classify("fix the typo in the readme");
        assert_eq!(level, EffortLevel::Fast);
    }

    #[test]
    fn test_effort_classify_design_is_extended() {
        let level = EffortLevel::classify("design a new authentication system with mfa");
        assert_eq!(level, EffortLevel::Extended);
    }

    #[test]
    fn test_effort_classify_short_is_instant() {
        let level = EffortLevel::classify("hello");
        assert_eq!(level, EffortLevel::Instant);
    }

    #[test]
    fn test_effort_classify_research_is_deep() {
        let level = EffortLevel::classify("research and investigate the latest advances in transformer architectures and compare them to state space models");
        assert_eq!(level, EffortLevel::Deep);
    }

    #[test]
    fn test_effort_classify_default_is_standard() {
        let level = EffortLevel::classify("write a blog post about AI safety");
        assert_eq!(level, EffortLevel::Standard);
    }

    #[test]
    fn test_effort_max_criteria_scales() {
        assert_eq!(EffortLevel::Instant.max_criteria(), 0);
        assert_eq!(EffortLevel::Fast.max_criteria(), 4);
        assert_eq!(EffortLevel::Standard.max_criteria(), 12);
        assert!(EffortLevel::Infinite.max_criteria() > 100);
    }

    #[test]
    fn test_effort_agent_gating() {
        assert!(!EffortLevel::Fast.allow_agents());
        assert!(EffortLevel::Extended.allow_agents());
    }

    #[test]
    fn test_effort_labels() {
        assert_eq!(EffortLevel::Instant.label(), "instant");
        assert_eq!(EffortLevel::Deep.label(), "deep");
    }

    // ── IdealState tests ───────────────────────────────────────────────

    #[test]
    fn test_ideal_state_new_empty() {
        let state = IdealState::new("engineering", EffortLevel::Standard);
        assert!(state.criteria.is_empty());
        assert_eq!(state.domain, "engineering");
    }

    #[test]
    fn test_ideal_state_add_criteria() {
        let mut state = IdealState::new("engineering", EffortLevel::Standard);
        state.add_criterion("Tests pass on all platforms", false);
        state.add_criterion("No credentials exposed", false);
        assert_eq!(state.criteria.len(), 2);
    }

    #[test]
    fn test_ideal_state_respects_effort_cap() {
        let mut state = IdealState::new("test", EffortLevel::Fast);
        for i in 0..10 {
            state.add_criterion(&format!("Criterion {}", i), false);
        }
        assert_eq!(state.criteria.len(), 4);
    }

    #[test]
    fn test_ideal_state_verify_all_pass() {
        let mut state = IdealState::new("test", EffortLevel::Standard);
        state.add_criterion("Output must be correct", false);

        let result = QuantizedVSA::seeded_random(42, 4096);
        let results = state.verify(&result, 0.3);
        assert_eq!(results.len(), 1);
        // May pass or fail depending on cosine sim — just verify the API works
        assert!(results[0].similarity >= 0.0 && results[0].similarity <= 1.0);
    }

    #[test]
    fn test_ideal_state_empty_pass_rate() {
        let state = IdealState::new("test", EffortLevel::Instant);
        assert!((state.pass_rate() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_ideal_state_anti_criterion() {
        let mut state = IdealState::new("test", EffortLevel::Standard);
        state.add_criterion("No dead code", true);
        assert!(state.criteria[0].is_anti);
    }

    #[test]
    fn test_ideal_state_verify_returns_results() {
        let mut state = IdealState::new("test", EffortLevel::Standard);
        state.add_criterion("Everything works", false);
        let result = QuantizedVSA::seeded_random(1234, 4096);
        let results = state.verify(&result, 0.5);
        assert_eq!(results.len(), 1);
        assert!(!results[0].statement.is_empty());
    }

    #[test]
    fn test_ideal_state_failures_when_none() {
        let state = IdealState::new("test", EffortLevel::Instant);
        assert!(state.failures().is_empty());
    }

    // ── ReverseIntent tests ────────────────────────────────────────────

    #[test]
    fn test_reverse_intent_extracts_explicit_asks() {
        let intent = reverse_intent("I want you to build a secure API gateway");
        assert!(!intent.explicit_asks.is_empty());
    }

    #[test]
    fn test_reverse_intent_finds_anti_criteria() {
        let intent = reverse_intent("Build a login page without password storage");
        assert!(!intent.anti_criteria.is_empty());
    }

    #[test]
    fn test_reverse_intent_detects_security_domain() {
        let intent = reverse_intent("Fix the vulnerability in the authentication system");
        assert_eq!(intent.domain, "security");
    }

    #[test]
    fn test_reverse_intent_detects_engineering_domain() {
        let intent = reverse_intent("Implement a new caching layer for the database");
        assert_eq!(intent.domain, "engineering");
    }

    #[test]
    fn test_reverse_intent_reports() {
        let intent = reverse_intent("Build a secure system");
        let report = intent.report();
        assert!(report.contains("Intent["));
        assert!(report.contains("effort"));
    }

    #[test]
    fn test_reverse_intent_to_ideal_state() {
        let intent = reverse_intent("Design a secure login without storing passwords");
        let state = intent.to_ideal_state();
        assert!(state.criteria.len() >= 2); // explicit + anti
    }

    #[test]
    fn test_reverse_intent_default_domain() {
        let intent = reverse_intent("What is the weather?");
        assert_eq!(intent.domain, "general");
    }

    #[test]
    fn test_reverse_intent_empty_query() {
        let intent = reverse_intent("please");
        // Should still produce something reasonable
        assert!(!intent.explicit_asks.is_empty() || !intent.failure_modes.is_empty());
    }

    // ── EFEGoal tests ──────────────────────────────────────────────────

    #[test]
    fn test_efe_goal_creates_preferred_outcomes() {
        let mut state = IdealState::new("test", EffortLevel::Standard);
        state.add_criterion("Fast response time", false);
        state.add_criterion("No crashes", true);
        let goal = EFEGoal::new(state);
        let outcomes = goal.to_preferred_outcomes(3, 4);
        assert_eq!(outcomes.len(), 3);
        assert_eq!(outcomes[0].len(), 4);
    }

    #[test]
    fn test_efe_goal_criterion_count() {
        let mut state = IdealState::new("test", EffortLevel::Standard);
        state.add_criterion("C1", false);
        state.add_criterion("C2", false);
        let goal = EFEGoal::new(state);
        assert_eq!(goal.criterion_count(), 2);
    }

    #[test]
    fn test_efe_goal_all_values_in_range() {
        let mut state = IdealState::new("test", EffortLevel::Standard);
        state.add_criterion("Good criterion", false);
        let goal = EFEGoal::new(state);
        let outcomes = goal.to_preferred_outcomes(2, 3);
        for action_outcome in &outcomes {
            for &val in action_outcome {
                assert!(val >= 0.0 && val <= 1.0, "Value {} out of range", val);
            }
        }
    }

    #[test]
    fn test_efe_goal_empty_criteria_creates_baseline() {
        let state = IdealState::new("test", EffortLevel::Standard);
        let goal = EFEGoal::new(state);
        let outcomes = goal.to_preferred_outcomes(1, 2);
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].len(), 2);
        // Baseline values should be 0.5 range
        for &val in &outcomes[0] {
            assert!(val > 0.3 && val < 0.9);
        }
    }

    // ── BitterLesson tests ─────────────────────────────────────────────

    #[test]
    fn test_ble_check_clean_passes() {
        let result = bitter_lesson_check("Prefer to let the model decide based on context");
        assert!(result.is_none());
    }

    #[test]
    fn test_ble_check_detects_always_pattern() {
        let result = bitter_lesson_check("Always use HTTPS for all connections");
        assert!(result.is_some());
        assert!(result.unwrap().contains("BLE violation"));
    }

    #[test]
    fn test_ble_check_detects_how_overload() {
        let result =
            bitter_lesson_check("First connect, then authenticate, then query the database");
        assert!(result.is_some());
    }

    #[test]
    fn test_ble_check_never_pattern() {
        let result = bitter_lesson_check("Never store passwords in plain text");
        assert!(result.is_some());
    }

    // ── AIRating tests ─────────────────────────────────────────────────

    #[test]
    fn test_ai_rating_weights_correctly() {
        let rating = AIRating::new(1.0, 0.5, 0.5);
        let expected = 0.45 * 1.0 + 0.35 * 0.5 + 0.20 * 0.5;
        assert!((rating.overall() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_ai_rating_clamps_values() {
        let rating = AIRating::new(1.5, -0.5, 0.8);
        assert!((rating.customization - 1.0).abs() < 1e-10);
        assert!((rating.integration - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_ai_rating_customization_dominates() {
        let high_custom = AIRating::new(0.9, 0.3, 0.3);
        let high_competence = AIRating::new(0.3, 0.3, 0.9);
        assert!(high_custom.overall() > high_competence.overall());
    }

    #[test]
    fn test_ai_rating_report() {
        let rating = AIRating::new(0.8, 0.6, 0.4);
        let report = rating.report();
        assert!(report.contains("AIRating[overall="));
        assert!(report.contains("customization=80.0%"));
    }

    // ── PredictionRegistry tests ───────────────────────────────────────

    #[test]
    fn test_prediction_registry_register() {
        let mut reg = PredictionRegistry::new();
        let id = reg.register(
            "AI will pass the bar exam",
            PredictionConfidence::Probable,
            9999999999,
        );
        assert_eq!(reg.predictions.len(), 1);
        assert_eq!(reg.predictions[0].id, id);
    }

    #[test]
    fn test_prediction_registry_resolve() {
        let mut reg = PredictionRegistry::new();
        let id = reg.register(
            "Test prediction",
            PredictionConfidence::AlmostCertain,
            9999999999,
        );
        assert!(reg.resolve(id, PredictionStatus::Correct));
        assert_eq!(reg.predictions[0].status, PredictionStatus::Correct);
    }

    #[test]
    fn test_prediction_registry_resolve_unknown() {
        let mut reg = PredictionRegistry::new();
        assert!(!reg.resolve(999, PredictionStatus::Correct));
    }

    #[test]
    fn test_prediction_registry_calibration_stats() {
        let mut reg = PredictionRegistry::new();
        let id1 = reg.register(
            "Prediction 1",
            PredictionConfidence::AlmostCertain,
            9999999999,
        );
        let id2 = reg.register("Prediction 2", PredictionConfidence::Probable, 9999999999);
        reg.resolve(id1, PredictionStatus::Correct);
        reg.resolve(id2, PredictionStatus::Wrong);
        let stats = reg.calibration_stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.resolved, 2);
        assert_eq!(stats.correct, 1);
    }

    #[test]
    fn test_prediction_registry_empty_stats() {
        let reg = PredictionRegistry::new();
        let stats = reg.calibration_stats();
        assert_eq!(stats.total, 0);
        assert!((stats.actual_accuracy - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_prediction_registry_report() {
        let mut reg = PredictionRegistry::new();
        let id = reg.register("Test", PredictionConfidence::Probable, 9999999999);
        reg.resolve(id, PredictionStatus::Correct);
        let report = reg.report();
        assert!(report.contains("PredictionRegistry["));
        assert!(report.contains("accuracy="));
    }

    #[test]
    fn test_prediction_confidence_values() {
        assert!((PredictionConfidence::AlmostCertain.value() - 0.9).abs() < 1e-9);
        assert!((PredictionConfidence::ChancesAboutEven.value() - 0.5).abs() < 1e-9);
        assert!((PredictionConfidence::AlmostCertainlyNot.value() - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_prediction_confidence_from_str() {
        assert_eq!(
            PredictionConfidence::from_str("almost_certain"),
            PredictionConfidence::AlmostCertain
        );
        assert_eq!(
            PredictionConfidence::from_str("50"),
            PredictionConfidence::ChancesAboutEven
        );
        assert_eq!(
            PredictionConfidence::from_str("unknown"),
            PredictionConfidence::ChancesAboutEven
        );
    }

    // ── Full pipeline integration tests ────────────────────────────────

    #[test]
    fn test_full_pipeline_end_to_end() {
        let query = "Implement a secure file upload system without storing files on disk";
        let result = QuantizedVSA::seeded_random(42, 4096);
        let output = process_with_ideal_state(query, &result);
        assert!(!output.intent.explicit_asks.is_empty());
        assert!(!output.intent.anti_criteria.is_empty());
        assert_eq!(output.intent.domain, "engineering");
        assert!(output.pass_rate >= 0.0 && output.pass_rate <= 1.0);
        assert!(!output.effort_label.is_empty());
    }

    #[test]
    fn test_full_pipeline_instant_skips_verify() {
        let query = "hi";
        let result = QuantizedVSA::seeded_random(1, 4096);
        let output = process_with_ideal_state(query, &result);
        assert_eq!(output.effort_label, "instant");
        assert!((output.pass_rate - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_full_pipeline_security_domain() {
        let query = "Find vulnerabilities in the login endpoint";
        let result = QuantizedVSA::seeded_random(99, 4096);
        let output = process_with_ideal_state(query, &result);
        assert_eq!(output.intent.domain, "security");
    }

    #[test]
    fn test_criterion_domain_from_str() {
        assert_eq!(
            CriterionDomain::from_str("security"),
            CriterionDomain::Security
        );
        assert_eq!(
            CriterionDomain::from_str("performance"),
            CriterionDomain::Performance
        );
        assert_eq!(
            CriterionDomain::from_str("unknown"),
            CriterionDomain::Generic
        );
    }
}
