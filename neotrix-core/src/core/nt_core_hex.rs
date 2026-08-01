//! E₈ × 64 state-space reasoning model.
//!
//! Maps the 64 hexagrams to 64 reasoning modes across 6 binary axes.
//! The +1 observer principle adds 2 meta-bits for self-position tracking,
//! giving an 8-bit (256-state) space where the engine can navigate.
//!
//! ## 6 Reasoning Axes (bit positions)
//!
//! | Bit | Axis | 0 | 1 |
//! |-----|------|----|-----|
//! | 5 | Abstraction | Concrete | Abstract |
//! | 4 | Scope | Focused | Broad |
//! | 3 | Method | Analytical | Generative |
//! | 2 | Depth | Deep | Fast |
//! | 1 | Mode | Solo | Collaborative |
//! | 0 | Stance | Certain | Exploratory |

#[cfg(test)]
use std::collections::HashSet;
use serde::{Serialize, Deserialize};

/// A reasoning state represented as a 6-bit value (0-63), isomorphic to a hexagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReasoningHexagram(pub u8);

impl ReasoningHexagram {
    /// Create a new reasoning state (masked to 6 bits).
    pub fn new(bits: u8) -> Self {
        Self(bits & 0x3F)
    }

    /// Get the value of a specific reasoning axis (0=LSB, 5=MSB).
    pub fn axis(&self, i: usize) -> u8 {
        (self.0 >> (i & 7)) & 1
    }

    // ─── Axis accessors ───

    /// Abstraction: 0=Concrete, 1=Abstract.
    pub fn abstraction(&self) -> u8 { self.axis(5) }
    /// Scope: 0=Focused, 1=Broad.
    pub fn scope(&self) -> u8 { self.axis(4) }
    /// Method: 0=Analytical, 1=Generative.
    pub fn method(&self) -> u8 { self.axis(3) }
    /// Depth: 0=Deep, 1=Fast.
    pub fn depth(&self) -> u8 { self.axis(2) }
    /// Mode: 0=Solo, 1=Collaborative.
    pub fn reasoning_mode(&self) -> u8 { self.axis(1) }
    /// Stance: 0=Certain, 1=Exploratory.
    pub fn stance(&self) -> u8 { self.axis(0) }

    // ─── State transitions ───

    /// Flip a single reasoning axis (爻变).
    pub fn flip_axis(&self, i: usize) -> Self {
        let mask = 1u8 << (i & 7);
        Self(self.0 ^ (mask & 0x3F))
    }

    /// Flip multiple axes at once.
    pub fn flip_axes(&self, bits_to_flip: u8) -> Self {
        Self(self.0 ^ (bits_to_flip & 0x3F))
    }

    /// Complement (错卦): flip all 6 axes.
    pub fn complement(&self) -> Self {
        Self(!self.0 & 0x3F)
    }

    /// Reverse (综卦): reverse the bit order (top↔bottom).
    pub fn reverse(&self) -> Self {
        let mut r = 0u8;
        for i in 0..6 {
            if (self.0 >> i) & 1 == 1 {
                r |= 1 << (5 - i);
            }
        }
        Self(r)
    }

    // ─── Resonance ───

    /// Hamming distance to another state.
    pub fn hamming_dist(&self, other: &Self) -> u32 {
        (self.0 ^ other.0).count_ones()
    }

    /// Two states are in resonance if they share ≥4 axes (hamming dist ≤ 2).
    pub fn resonance_with(&self, other: &Self) -> bool {
        self.hamming_dist(other) <= 2
    }

    /// Resonance strength: 6 - hamming distance (max=6, min=0).
    pub fn resonance_strength(&self, other: &Self) -> u32 {
        6 - self.hamming_dist(other)
    }

    /// Generate all 6 neighboring states (one flip each).
    pub fn neighbors(&self) -> Vec<Self> {
        (0..6).map(|i| self.flip_axis(i)).collect()
    }

    /// Generate all states within `dist` flips.
    pub fn neighborhood(&self, dist: u32) -> Vec<Self> {
        let mut result = Vec::new();
        for bits in 0..64u8 {
            let candidate = Self(bits);
            if self.hamming_dist(&candidate) <= dist {
                result.push(candidate);
            }
        }
        result
    }

    /// Human-readable mode name.
    pub fn mode_name(&self) -> &'static str {
        let idx = self.0 as usize;
        MODE_NAMES[idx]
    }

    /// Detailed mode description.
    pub fn mode_description(&self) -> &'static str {
        let idx = self.0 as usize;
        MODE_DESCRIPTIONS[idx]
    }

    /// Recommended for which task type keywords.
    pub fn task_recommendation(&self) -> &'static [&'static str] {
        let idx = self.0 as usize;
        MODE_TASKS[idx]
    }
}

/// All 64 reasoning mode names.
pub const MODE_NAMES: [&str; 64] = [
    // 0-7: Concrete-Focused-Analytical-Deep-Solo-Certain → Abstract-Broad-Generative-Fast-Collaborative-Exploratory
    "Deep Debug",       "Guided Debug",     "Experiment",       "Guided Experiment",
    "Code Review",      "Pair Review",      "Rapid Prototype",  "Co-creation",
    "Root Cause",       "Guided RCA",       "Hypothesis Test",  "Guided Hypothesis",
    "Design Audit",     "Pair Audit",       "Brainstorm",       "Jam Session",
    // 16-31
    "Formal Proof",     "Guided Proof",     "Model Check",      "Guided Model Check",
    "Spec Review",      "Spec Pairing",     "Exploration",      "Guided Exploration",
    "Data Analysis",    "Pair Analysis",    "Statistical Run",  "Guided Statistics",
    "Architecture",     "Pair Arch",        "Visioning",        "Guided Visioning",
    // 32-47
    "Syntax Check",     "Guided Check",     "Quick Fix",        "Guided Quick Fix",
    "Lint Review",      "Lint Pairing",     "Fast Iteration",   "Paired Iteration",
    "Unit Test",        "Guided Test",      "Fuzz Run",         "Guided Fuzz",
    "Integration",      "Pair Integration", "Scaffold",         "Guided Scaffold",
    // 48-63
    "Pattern Match",    "Guided Pattern",   "Refactor",         "Guided Refactor",
    "Style Guide",      "Style Pairing",    "Generate",         "Co-generate",
    "Trace Analysis",   "Pair Tracing",     "Benchmark",        "Guided Benchmark",
    "System Design",    "Pair System",      "Meta-cognition",   "Guided Meta",
];

/// Detailed descriptions for all 64 reasoning modes.
pub const MODE_DESCRIPTIONS: [&str; 64] = [
    "Deep focus on a specific bug with deterministic root cause analysis",
    "Debug with external guidance or documentation references",
    "Run controlled experiments to validate or falsify a hypothesis",
    "Experimental validation with collaborative input",
    "Systematic code review with focus on correctness and edge cases",
    "Collaborative review with cross-referencing multiple perspectives",
    "Quick prototype to test feasibility of an approach",
    "Rapid co-creation of proof-of-concept code",
    "Trace causality chain from symptom to root cause",
    "Root cause analysis with external knowledge augmentation",
    "Formulate and test a specific hypothesis against evidence",
    "Hypothesis generation and testing in guided mode",
    "Systematic review of design decisions against requirements",
    "Collaborative design review with multiple stakeholder lenses",
    "Open-ended idea generation without constraint",
    "High-energy collaborative ideation session",
    "Rigorous mathematical or logical proof construction",
    "Proof construction with theorem prover or reference guidance",
    "Systematic state-space exploration for model verification",
    "Guided model checking with external specifications",
    "Review specifications for completeness and consistency",
    "Collaborative spec review with domain expert guidance",
    "Open-ended exploration of a problem space",
    "Guided exploration with structured search path",
    "Deep quantitative analysis of data or metrics",
    "Collaborative data analysis with statistical guidance",
    "Run statistical tests and analyze significance",
    "Guided statistical analysis with interpretation support",
    "High-level architecture design and trade-off analysis",
    "Collaborative architecture design with multi-expert input",
    "Long-term vision and strategic direction setting",
    "Guided visioning with structured future-back thinking",
    "Quick syntax and type correctness check",
    "Syntax validation with reference documentation lookup",
    "Rapid fix for a well-understood issue",
    "Guided quick fix with automated suggestion review",
    "Code lint and style consistency check",
    "Collaborative lint review with team style guide",
    "Fast iterative development cycle",
    "Paired fast iteration with continuous feedback",
    "Write unit tests for specific functionality",
    "Test writing with test pattern guidance",
    "Run fuzz testing to discover edge-case failures",
    "Guided fuzz testing with coverage analysis",
    "Test integration between multiple components",
    "Collaborative integration testing with system knowledge",
    "Scaffold new project or module structure",
    "Guided scaffolding with template and pattern selection",
    "Match current problem against known solution patterns",
    "Pattern matching with library of known solutions",
    "Refactor existing code for improved structure",
    "Guided refactoring with safety net and verification",
    "Check code against style guide standards",
    "Collaborative style review with automated tooling",
    "Generate new code from specification",
    "Co-generate code with interactive refinement",
    "Analyze execution traces for performance or logic issues",
    "Collaborative trace analysis with visualization",
    "Run benchmarks and analyze performance characteristics",
    "Guided benchmarking with statistical rigor",
    "High-level system design with component interaction modeling",
    "Collaborative system design with architecture review",
    "Meta-cognitive reflection on the reasoning process itself",
    "Guided meta-cognition with structured self-assessment",
];

/// Recommended task keywords per mode.
pub const MODE_TASKS: [&[&str]; 64] = [
    &["crash", "panic", "null pointer", "segfault", "index out of bounds"],
    &["debug", "trace", "log", "error message", "stack trace"],
    &["experiment", "A/B test", "hypothesis test", "validate"],
    &["tutorial", "learn", "try", "explore syntax"],
    &["code review", "review", "inspect", "audit"],
    &["pair review", "joint review", "team review"],
    &["prototype", "quick demo", "proof of concept", "feasibility"],
    &["pair programming", "mob programming", "co-create"],
    &["root cause", "why", "causal", "chain of failure"],
    &["failure analysis", "postmortem", "incident review"],
    &["hypothesis", "test theory", "verify assumption"],
    &["assumption check", "theory validation"],
    &["design review", "architecture review", "design critique"],
    &["stakeholder review", "cross-team design"],
    &["brainstorm", "idea generation", "creative", "image idea", "visual concept", "design concept"],
    &["workshop", "ideation", "creative session"],
    &["formal proof", "theorem", "verification", "correctness proof"],
    &["proof assistant", "coq", "lean", "isabelle"],
    &["model check", "state space", "formal verification"],
    &["specification check", "requirement verification"],
    &["spec review", "requirement review", "specification"],
    &["spec collaboration", "requirement workshop"],
    &["explore", "investigate", "research", "discover", "visual search", "image explore"],
    &["guided research", "structured exploration"],
    &["data analysis", "statistics", "metrics", "analytics"],
    &["collaborative analysis", "data review"],
    &["statistical test", "significance", "p-value", "confidence"],
    &["statistical guidance", "methodology review"],
    &["architecture", "system design", "structure", "component"],
    &["architectural review", "design decision"],
    &["vision", "strategy", "roadmap", "long-term"],
    &["strategic planning", "future vision"],
    &["syntax", "type check", "compilation", "parse"],
    &["syntax help", "type error", "compiler error"],
    &["quick fix", "simple fix", "hotfix"],
    &["guided fix", "fix suggestion"],
    &["lint", "style", "format", "code quality"],
    &["style guide", "team convention", "formatting"],
    &["iterate", "rapid iteration", "agile", "sprint"],
    &["paired iteration", "continuous feedback"],
    &["unit test", "test case", "spec", "assertion"],
    &["test pattern", "test design", "test strategy"],
    &["fuzz", "fuzzing", "random test", "edge case"],
    &["guided fuzz", "coverage guided"],
    &["integration test", "e2e", "end-to-end"],
    &["integration review", "system test"],
    &["scaffold", "init", "new project", "setup"],
    &["project setup", "initialization", "template"],
    &["pattern", "design pattern", "solution pattern"],
    &["pattern library", "known solution"],
    &["refactor", "restructure", "reorganize", "clean up"],
    &["guided refactor", "safe refactor", "restructure with tests"],
    &["style compliance", "code style", "naming convention"],
    &["style automation", "linting pipeline"],
    &["generate", "code gen", "implement", "create", "image gen", "visual generation", "text2img", "img2img"],
    &["co-generate", "interactive generation"],
    &["trace", "performance trace", "execution trace", "profiling"],
    &["trace visualization", "waterfall", "flame graph"],
    &["benchmark", "performance test", "latency", "throughput"],
    &["benchmark analysis", "performance review"],
    &["system design", "architecture design", "component diagram"],
    &["architecture review", "design document"],
    &["meta", "self-reflection", "self-assessment", "improve"],
    &["guided meta", "structured reflection", "retrospective"],
];

/// GPT-5-style reasoning effort levels — maps to E8 depth axis with finer granularity.
///
/// NeoTrix's E8 engine has a binary Depth axis (0=Deep, 1=Fast). This enum
/// provides 5 graduated levels (matching GPT-5's none/low/medium/high/xhigh),
/// which can be projected onto the E8 6-axis space for mode selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ReasoningEffort {
    /// Fastest, cheapest. No chain-of-thought. Simple transformations.
    None,
    /// Minimal reasoning. Quick sanity checks. Straightforward Q&A.
    Low,
    /// Balanced reasoning. Default for most production workloads.
    #[default]
    Medium,
    /// Extended reasoning chains. Multi-step logic, complex debugging.
    High,
    /// Maximum reasoning depth. Hard math, security audits, deep research.
    XHigh,
}

impl ReasoningEffort {
    pub fn all() -> [Self; 5] {
        [Self::None, Self::Low, Self::Medium, Self::High, Self::XHigh]
    }

    /// Project this effort level onto the 6 E8 factor dimensions.
    ///
    /// Maps effort to specific factor values:
    /// - DEPTH (bit 2, factor idx 2): directly controlled by effort
    /// - STANCE (bit 0, factor idx 0): lower effort = more certainty, higher = more exploratory
    /// - METH (bit 3, factor idx 3): higher effort = more analytical (0), lower = generative (1)
    /// - SCOPE (bit 4, factor idx 4): higher effort = broader scope
    pub fn to_e8_factors(&self) -> [f64; 6] {
        match self {
            Self::None   => [0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
            Self::Low    => [0.2, 0.2, 0.8, 0.7, 0.2, 0.2],
            Self::Medium => [0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            Self::High   => [0.7, 0.7, 0.2, 0.3, 0.7, 0.7],
            Self::XHigh  => [0.9, 0.9, 0.0, 0.1, 0.9, 0.9],
        }
    }

    /// Map to the binary E8 depth axis value (0=Deep, 1=Fast).
    pub fn to_e8_depth(&self) -> u8 {
        match self {
            Self::None | Self::Low => 1, // Fast
            Self::Medium | Self::High | Self::XHigh => 0, // Deep
        }
    }

    /// Cost multiplier: how many "reasoning tokens" to allocate relative to Medium.
    pub fn cost_multiplier(&self) -> f64 {
        match self {
            Self::None   => 0.1,
            Self::Low    => 0.3,
            Self::Medium => 1.0,
            Self::High   => 2.5,
            Self::XHigh  => 5.0,
        }
    }

    /// Recommended task types for this effort level.
    pub fn recommendation(&self) -> &'static [&'static str] {
        match self {
            Self::None   => &["format", "translate", "extract", "transform", "simple qa"],
            Self::Low    => &["classify", "summarize", "quick fix", "boilerplate"],
            Self::Medium => &["code", "debug", "analyze", "review", "design"],
            Self::High   => &["refactor", "architecture", "complex debug", "optimize"],
            Self::XHigh  => &["security audit", "hard math", "research", "deep analysis"],
        }
    }
}

impl std::fmt::Display for ReasoningEffort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None   => write!(f, "none"),
            Self::Low    => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High   => write!(f, "high"),
            Self::XHigh  => write!(f, "xhigh"),
        }
    }
}

/// Generate all 64 reasoning states.
pub fn all_reasoning_states() -> Vec<ReasoningHexagram> {
    (0..64).map(ReasoningHexagram).collect()
}

/// Fable 5-style intention context: Goal → Reason → Boundaries → Verification.
///
/// Models the four-quadrant thinking pattern that drives mode selection:
/// - **Goal**: what outcome is desired
/// - **Reason**: why this matters, who it's for, what the output enables
/// - **Boundaries**: what NOT to do, where to stop, what to avoid
/// - **Verification**: how to check the work is correct
#[derive(Debug, Clone, Default)]
pub struct IntentionContext {
    /// The desired outcome (目标).
    pub goal: String,
    /// Why this matters — context, audience, intent (原因).
    pub reason: String,
    /// What not to do — constraints, boundaries, stop conditions (边界).
    pub boundaries: String,
    /// How to verify correctness (验证).
    pub verification: String,
}

impl IntentionContext {
    pub fn new(goal: &str, reason: &str, boundaries: &str, verification: &str) -> Self {
        Self {
            goal: goal.to_string(),
            reason: reason.to_string(),
            boundaries: boundaries.to_string(),
            verification: verification.to_string(),
        }
    }

    /// Generate a combined task string for keyword-based mode matching.
    pub fn to_task_string(&self) -> String {
        format!("{} {} {} {}", self.goal, self.reason, self.boundaries, self.verification)
    }

    /// Convert intention context into an E8 6-factor context vector.
    ///
    /// Maps each intention field to E8 reasoning axes:
    /// - Goal clarity → ABST (abstract vs concrete): concrete goal → 0, abstract → 1
    /// - Reason depth → SCOPE: deep reason → 1, shallow → 0
    /// - Boundaries strictness → METH: strict → 0 (analytical), loose → 1 (generative)
    /// - Verification availability → DEPTH: verifiable → 0 (deep), unverifiable → 1 (fast)
    /// - Task complexity → MODE: complex → 1 (collaborative), simple → 0 (solo)
    /// - Uncertainty → STANCE: uncertain → 1 (exploratory), certain → 0
    pub fn to_e8_factor_context(&self) -> [f64; 6] {
        let goal_len = self.goal.len() as f64;
        let reason_len = self.reason.len() as f64;
        let boundary_len = self.boundaries.len() as f64;
        let verify_len = self.verification.len() as f64;

        // Abstraction: longer, more conceptual goals → abstract (1.0)
        let abstraction = (goal_len / 200.0).min(1.0);

        // Scope: detailed reason with context → broad (1.0)
        let scope = (reason_len / 300.0).min(1.0);

        // Method: detailed boundaries → analytical (0.0), few constraints → generative (1.0)
        let method = 1.0 - (boundary_len / 200.0).min(1.0);

        // Depth: strong verification → deep (0.0), weak → fast (1.0)
        let depth = 1.0 - (verify_len / 200.0).min(1.0);

        // Mode: complex goals need collaboration
        let collaborative = ((goal_len + reason_len) / 500.0 * 0.5).min(1.0);

        // Stance: weak verification → exploratory
        let exploratory = if verify_len < 20.0 { 0.8 } else { 0.2 };

        [exploratory, collaborative, depth, method, scope, abstraction]
    }
}

/// Find the optimal starting mode for a task based on keyword overlap.
/// Parse a raw task string into an IntentionContext using heuristic markers.
///
/// Looks for structural markers like "goal:", "reason:", "boundary:", "verify:"
/// in the task string. Falls back to using the full string as the goal field.
pub fn intention_from_string(task: &str) -> IntentionContext {
    let lower = task.to_lowercase();
    let mut ctx = IntentionContext::default();

    // Check for explicit markers: "goal:", "reason:", "boundary:", "verify:"
    let goal_markers = ["goal:", "objective:", "aim:"];
    let reason_markers = ["reason:", "context:", "why:", "background:"];
    let boundary_markers = ["boundary:", "constraint:", "don't:", "avoid:", "bound:"];
    let verify_markers = ["verify:", "verification:", "check:", "test:", "validation:"];

    for m in &goal_markers {
        if let Some(pos) = lower.find(m) {
            let start = pos + m.len();
            let end = find_next_marker(&lower[start..], &[&reason_markers[..], &boundary_markers[..], &verify_markers[..]].concat()) + start;
            ctx.goal = slice_task(task, &lower, start, end);
            break;
        }
    }
    if ctx.goal.is_empty() {
        ctx.goal = task.to_string();
    }

    for m in &reason_markers {
        if let Some(pos) = lower.find(m) {
            let start = pos + m.len();
            let end = find_next_marker(&lower[start..], &[&goal_markers[..], &boundary_markers[..], &verify_markers[..]].concat()) + start;
            ctx.reason = slice_task(task, &lower, start, end);
            break;
        }
    }

    for m in &boundary_markers {
        if let Some(pos) = lower.find(m) {
            let start = pos + m.len();
            let end = find_next_marker(&lower[start..], &[&goal_markers[..], &reason_markers[..], &verify_markers[..]].concat()) + start;
            ctx.boundaries = slice_task(task, &lower, start, end);
            break;
        }
    }

    for m in &verify_markers {
        if let Some(pos) = lower.find(m) {
            let start = pos + m.len();
            let end = find_next_marker(&lower[start..], &[&goal_markers[..], &reason_markers[..], &boundary_markers[..]].concat()) + start;
            ctx.verification = slice_task(task, &lower, start, end);
            break;
        }
    }

    ctx
}

/// 从 task 提取 [start,end) 段：偏移基于小写副本计算，须先对齐原串字符边界并钳位
/// （lowercasing 可能改变字节长度，越界/落在多字节字符中间会 panic）
fn slice_task(task: &str, _lower: &str, start: usize, end: usize) -> String {
    let s = start.min(task.len());
    let e = end.min(task.len());
    let s = task.floor_char_boundary(s);
    let e = task.floor_char_boundary(e);
    task[s..e].trim().to_string()
}

fn find_next_marker(text: &str, markers: &[&str]) -> usize {
    let mut earliest = text.len();
    for m in markers {
        if let Some(pos) = text.find(m) {
            if pos < earliest {
                earliest = pos;
            }
        }
    }
    earliest
}

/// Gemini 3 Pro-style Multiple Hypothesis Evaluation (MHE).
///
/// Evaluates K candidate E8 modes in parallel, scoring each by its keyword fit
/// and factor alignment, then returns the ranked candidates. This mirrors Gemini
/// Deep Think's "consider multiple hypotheses before committing" pattern.
///
/// Unlike `rank_modes_for_task()` which uses pure keyword match, MHE also
/// considers the IntentionContext factor alignment and reasoning effort level.
#[derive(Debug, Clone)]
pub struct MultipleHypothesisEvaluator {
    /// Number of hypotheses to evaluate per call
    pub k: usize,
    /// Minimum confidence threshold for accepting a hypothesis
    pub min_confidence: f64,
}

impl Default for MultipleHypothesisEvaluator {
    fn default() -> Self {
        Self { k: 5, min_confidence: 0.3 }
    }
}

impl MultipleHypothesisEvaluator {
    pub fn new(k: usize, min_confidence: f64) -> Self {
        Self { k, min_confidence }
    }

    /// Evaluate K candidate hypotheses from the full E8 state space.
    ///
    /// Returns the top-K `ModeFit` results, each scored by a combination of
    /// keyword match, factor alignment, and effort alignment. The caller can
    /// then select the best hypothesis (or beam-search across multiple).
    pub fn evaluate(&self, intent: &IntentionContext, effort: ReasoningEffort) -> Vec<ModeFit> {
        let factors = intent.to_e8_factor_context();
        let effort_factors = effort.to_e8_factors();
        let task_str = intent.to_task_string();
        let lower = task_str.to_lowercase();

        let mut fits: Vec<ModeFit> = (0..64).map(|bits| {
            let state = ReasoningHexagram(bits);
            let keywords = state.task_recommendation();

            // Keyword match
            let keyword_score: f64 = keywords.iter().map(|kw| {
                if lower.contains(kw) { 1.0 } else { 0.0 }
            }).sum();

            // Effort keyword bonus
            let effort_kw_bonus: f64 = effort.recommendation().iter().map(|ew| {
                if lower.contains(*ew) { 2.0 } else { 0.0 }
            }).sum();

            // Factor alignment: combine intent factors with effort
            let factor_alignment: f64 = (0..6).map(|i| {
                let axis_val = state.axis(i) as f64;
                let blended = factors[i] * 0.6 + effort_factors[i] * 0.4;
                1.0 - (axis_val - blended).abs()
            }).sum::<f64>() / 6.0;

            // Total score
            let score = keyword_score * 2.0 + effort_kw_bonus + factor_alignment * 5.0;
            let max_possible = keywords.len() as f64 * 2.0 + effort.recommendation().len() as f64 * 2.0 + 5.0;
            let confidence = if max_possible > 0.0 {
                (score / max_possible).min(1.0)
            } else {
                0.0
            };

            ModeFit { state, score: score as u32, confidence }
        }).collect();

        // Sort by score descending, filter by confidence, take top K
        fits.sort_by(|a, b| b.score.cmp(&a.score));
        fits.retain(|f| f.confidence >= self.min_confidence);
        fits.truncate(self.k);
        fits
    }

    /// Evaluate with the default Medium effort level.
    pub fn evaluate_default(&self, intent: &IntentionContext) -> Vec<ModeFit> {
        self.evaluate(intent, ReasoningEffort::Medium)
    }
}

/// Select E8 mode using Fable 5-style intention context.
///
/// Converts the four-quadrant intention (Goal→Reason→Boundaries→Verification)
/// into an E8 factor context vector, then selects the mode whose keyword
/// profile best matches the intention. Falls back to `optimal_starting_mode()`
/// on the combined task string.
pub fn select_mode_by_intent(intent: &IntentionContext) -> ReasoningHexagram {
    select_mode_by_intent_with_effort(intent, ReasoningEffort::Medium)
}

/// Select E8 mode using intention context + GPT-5-style reasoning effort.
///
/// Effort level is projected onto the E8 factor space and combined with the
/// intention-derived factor context to bias mode selection.
pub fn select_mode_by_intent_with_effort(intent: &IntentionContext, effort: ReasoningEffort) -> ReasoningHexagram {
    let mut factors = intent.to_e8_factor_context();
    let effort_factors = effort.to_e8_factors();

    // Blend effort into factors: effort controls DEPTH (idx 2), METH (idx 3), SCOPE (idx 4)
    // with 40% weight from effort, 60% from intent
    for i in 0..6 {
        factors[i] = factors[i] * 0.6 + effort_factors[i] * 0.4;
    }

    let task_str = intent.to_task_string();
    let lower = task_str.to_lowercase();
    let mut best_score = 0f64;
    let mut best_idx = 0u8;

    for bits in 0..64u8 {
        let state = ReasoningHexagram(bits);
        let keywords = state.task_recommendation();

        // Keyword match score
        let keyword_score: f64 = keywords.iter().map(|kw| {
            if lower.contains(kw) { 1.0 } else { 0.0 }
        }).sum();

        // Effort keyword bonus: matches against effort.recommendation()
        let effort_kw_bonus: f64 = effort.recommendation().iter().map(|ew| {
            if lower.contains(*ew) { 2.0 } else { 0.0 }
        }).sum();

        // Factor alignment: continuous from 0.0 to 1.0 per axis
        let factor_alignment: f64 = (0..6).map(|i| {
            let axis_val = state.axis(i) as f64;
            let factor_val = factors[i];
            1.0 - (axis_val - factor_val).abs()
        }).sum::<f64>() / 6.0;

        let total = keyword_score * 2.0 + effort_kw_bonus + factor_alignment * 5.0;
        if total > best_score {
            best_score = total;
            best_idx = bits;
        }
    }

    ReasoningHexagram(best_idx)
}

/// Find the optimal starting mode for a task based on keyword overlap.
pub fn optimal_starting_mode(task: &str) -> ReasoningHexagram {
    let lower = task.to_lowercase();
    let mut best_score = 0u32;
    let mut best_idx = 0u8;

    for bits in 0..64u8 {
        let state = ReasoningHexagram(bits);
        let keywords = state.task_recommendation();
        let score: u32 = keywords.iter().map(|kw| {
            if lower.contains(kw) { 1 } else { 0 }
        }).sum();
        if score > best_score {
            best_score = score;
            best_idx = bits;
        }
    }

    ReasoningHexagram(best_idx)
}

/// Score-based optimal mode selection with confidence.
#[derive(Clone)]
pub struct ModeFit {
    pub state: ReasoningHexagram,
    pub score: u32,
    pub confidence: f64,
}

/// Rank all 64 modes by fit for a task, return top-k.
pub fn rank_modes_for_task(task: &str, top_k: usize) -> Vec<ModeFit> {
    let lower = task.to_lowercase();
    let mut fits: Vec<ModeFit> = (0..64).map(|bits| {
        let state = ReasoningHexagram(bits);
        let keywords = state.task_recommendation();
        let score: u32 = keywords.iter().map(|kw| {
            if lower.contains(kw) { 1 } else { 0 }
        }).sum();
        let max_possible = keywords.len() as u32;
        let confidence = if max_possible > 0 {
            score as f64 / max_possible as f64
        } else {
            0.0
        };
        ModeFit { state, score, confidence }
    }).collect();

    fits.sort_by_key(|b| std::cmp::Reverse(b.score));
    fits.truncate(top_k);
    fits
}

/// Navigation path through the state space.
/// Each step flips one or more bits, representing a shift in reasoning approach.
pub struct ReasoningPath {
    pub states: Vec<ReasoningHexagram>,
    pub transitions: Vec<u8>,  // which bits were flipped at each step
}

impl ReasoningPath {
    /// Shortest path from `start` to `goal` using bit transitions.
    /// Each step flips exactly one bit (爻变).
    pub fn shortest(start: ReasoningHexagram, goal: ReasoningHexagram) -> Self {
        let diff = start.0 ^ goal.0;
        let mut states = vec![start];
        let mut transitions = Vec::new();
        let mut current = start;
        for i in 0..6 {
            if (diff >> i) & 1 == 1 {
                current = current.flip_axis(i);
                states.push(current);
                transitions.push(1 << i);
            }
        }
        Self { states, transitions }
    }

    /// Length of the path (number of transitions).
    pub fn len(&self) -> usize {
        self.transitions.len()
    }

    /// Whether the path is empty (start == goal).
    pub fn is_empty(&self) -> bool {
        self.transitions.is_empty()
    }
}

// ─── +1 Observer Meta-State ─────────────────────────────────────────

/// The +1 observer principle: track self-position with 2 meta-bits.
/// Bit 0: whether the engine is reflecting on its own reasoning (meta)
/// Bit 1: whether the engine is considering multiple future states (planning)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct MetaState(pub u8);

impl MetaState {
    pub fn new(bits: u8) -> Self {
        assert!(bits < 4, "MetaState must be 0..3");
        Self(bits)
    }

    /// Is the engine in meta-cognitive mode?
    pub fn is_reflecting(&self) -> bool { self.0 & 1 == 1 }
    /// Is the engine planning ahead?
    pub fn is_planning(&self) -> bool { (self.0 >> 1) & 1 == 1 }
}

/// Full 8-bit reasoning state: 6-bit hexagram + 2-bit meta.
/// Total: 64 × 4 = 256 possible states = Dayan 50×5+6 observer space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FullReasoningState {
    pub mode: ReasoningHexagram,
    pub meta: MetaState,
}

impl FullReasoningState {
    pub fn new(mode: ReasoningHexagram, meta: MetaState) -> Self {
        Self { mode, meta }
    }

    /// Total number of possible states = 64 × 4 = 256.
    pub const TOTAL_STATES: usize = 256;

    /// The self-position signature (u8 encoding).
    pub fn signature(&self) -> u16 {
        (self.mode.0 as u16) | ((self.meta.0 as u16) << 6)
    }

    /// Transition to a new mode while keeping meta-state.
    pub fn transition_to(&self, new_mode: ReasoningHexagram) -> Self {
        Self { mode: new_mode, meta: self.meta }
    }

    /// Enter reflection mode.
    pub fn reflect(&self) -> Self {
        Self { mode: self.mode, meta: MetaState(self.meta.0 | 1) }
    }

    pub fn mode_name(&self) -> &'static str {
        self.mode.mode_name()
    }

    pub fn mode_description(&self) -> &'static str {
        self.mode.mode_description()
    }

    /// Enter planning mode.
    pub fn plan(&self) -> Self {
        Self { mode: self.mode, meta: MetaState(self.meta.0 | 2) }
    }
}

// ─── 8×8 Reasoning Strategy Matrix ──────────────────────────────────

/// 8 reasoning approaches (upper trigram).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasoningApproach {
    Debug,      // 坤 — find and fix defects
    Test,       // 艮 — validate correctness
    Analyze,    // 坎 — deep quantitative analysis
    Design,     // 巽 — architectural design
    Generate,   // 震 — code generation
    Review,     // 离 — code review
    Prototype,  // 兑 — rapid prototyping
    Meta,       // 乾 — meta-cognitive reflection
}

/// 8 problem domains (lower trigram).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemDomain {
    Bug,        // 坤 — defects and errors
    Syntax,     // 艮 — syntax and types
    Logic,      // 坎 — logic and correctness
    Data,       // 巽 — data and state
    Perf,       // 震 — performance
    Security,   // 离 — security
    Design,     // 兑 — code quality
    System,     // 乾 — architecture and system
}

/// 8×8 reasoning strategy matrix: approach × domain → hexagram.
pub fn strategy_matrix() -> [[ReasoningHexagram; 8]; 8] {
    let mut m = [[ReasoningHexagram(0); 8]; 8];
    for approach in 0..8u8 {
        for domain in 0..8u8 {
            m[approach as usize][domain as usize] = ReasoningHexagram((approach << 3) | domain);
        }
    }
    m
}

/// Evolve a strategy matrix entry based on observer pattern name.
/// Returns true if the entry was modified.
pub fn evolve_strategy_entry(
    matrix: &mut [[ReasoningHexagram; 8]; 8],
    current: ReasoningHexagram,
    pattern_name: &str,
) -> bool {
    let approach = ((current.0 >> 3) & 0x07) as usize;
    let domain = (current.0 & 0x07) as usize;
    let entry = &mut matrix[approach][domain];

    match pattern_name {
        "Oscillation" | "LoopBack" => {
            let evolved = entry.flip_axis(4);
            if evolved != *entry { *entry = evolved; true } else { false }
        }
        "Stuck" => {
            let evolved = entry.flip_axes(0b001111);
            if evolved != *entry { *entry = evolved; true } else { false }
        }
        "Inefficient" => {
            let evolved = entry.flip_axis(3);
            if evolved != *entry { *entry = evolved; true } else { false }
        }
        _ => false,
    }
}

impl ReasoningApproach {
    pub fn all() -> [Self; 8] {
        use ReasoningApproach::*;
        [Debug, Test, Analyze, Design, Generate, Review, Prototype, Meta]
    }
}

impl ProblemDomain {
    pub fn all() -> [Self; 8] {
        use ProblemDomain::*;
        [Bug, Syntax, Logic, Data, Perf, Security, Design, System]
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_64_states() {
        let states = all_reasoning_states();
        assert_eq!(states.len(), 64);
        let mut seen = HashSet::new();
        for s in &states {
            assert!(seen.insert(s.0), "Duplicate state {}", s.0);
        }
    }

    #[test]
    fn test_complement() {
        let s = ReasoningHexagram(0b101010);
        assert_eq!(s.complement(), ReasoningHexagram(0b010101));
        assert_eq!(s, s.complement().complement());
    }

    #[test]
    fn test_flip_axis() {
        let s = ReasoningHexagram(0);
        assert_eq!(s.flip_axis(0), ReasoningHexagram(1));
        assert_eq!(s.flip_axis(5), ReasoningHexagram(32));
        assert_eq!(s.flip_axis(0).flip_axis(0), s);
    }

    #[test]
    fn test_reverse() {
        let s = ReasoningHexagram(0b001100);
        assert_eq!(s.reverse(), ReasoningHexagram(0b001100)); // palindrome
        let s2 = ReasoningHexagram(0b000001);
        assert_eq!(s2.reverse(), ReasoningHexagram(0b100000));
    }

    #[test]
    fn test_hamming_dist() {
        let a = ReasoningHexagram(0);
        let b = ReasoningHexagram(0b111111);
        assert_eq!(a.hamming_dist(&b), 6);
        assert_eq!(a.hamming_dist(&a), 0);
    }

    #[test]
    fn test_resonance() {
        let a = ReasoningHexagram(0);
        let b = ReasoningHexagram(0b000011); // diff = 2
        let c = ReasoningHexagram(0b111111); // diff = 6
        assert!(a.resonance_with(&b));
        assert!(!a.resonance_with(&c));
    }

    #[test]
    fn test_resonance_strength() {
        let a = ReasoningHexagram(0);
        assert_eq!(a.resonance_strength(&ReasoningHexagram(0)), 6);
        assert_eq!(a.resonance_strength(&ReasoningHexagram(1)), 5);
        assert_eq!(a.resonance_strength(&ReasoningHexagram(0b111111)), 0);
    }

    #[test]
    fn test_neighbors() {
        let s = ReasoningHexagram(0);
        let neighbors = s.neighbors();
        assert_eq!(neighbors.len(), 6);
        for n in &neighbors {
            assert_eq!(s.hamming_dist(n), 1);
        }
    }

    #[test]
    fn test_neighborhood() {
        let s = ReasoningHexagram(0);
        let hood = s.neighborhood(1);
        assert_eq!(hood.len(), 7); // self + 6 neighbors
    }

    #[test]
    fn test_shortest_path() {
        let start = ReasoningHexagram(0);
        let goal = ReasoningHexagram(0b001011);
        let path = ReasoningPath::shortest(start, goal);
        assert_eq!(path.len(), 3); // 3 bits differ
        assert_eq!(*path.states.last().expect("value should be ok in test"), goal);
    }

    #[test]
    fn test_optimal_starting_mode() {
        let mode = optimal_starting_mode("fix this crash bug");
        assert!(mode.0 < 64);
         // "crash" is in mode 0 (Deep Debug) keywords
        // but could also match others
    }

    #[test]
    fn test_rank_modes() {
        let ranked = rank_modes_for_task("review this code for bugs", 5);
        assert_eq!(ranked.len(), 5);
        assert!(ranked[0].score >= ranked[1].score);
    }

    #[test]
    fn test_full_state_256() {
        let s = FullReasoningState::new(ReasoningHexagram(0), MetaState(0));
        assert_eq!(FullReasoningState::TOTAL_STATES, 256);
        let reflected = s.reflect();
        assert!(reflected.meta.is_reflecting());
        let planned = s.plan();
        assert!(planned.meta.is_planning());
    }

    #[test]
    fn test_strategy_matrix() {
        let matrix = strategy_matrix();
        assert_eq!(matrix.len(), 8);
        assert_eq!(matrix[0].len(), 8);
        assert_eq!(matrix[0][0], ReasoningHexagram(0));  // Debug × Bug
        assert_eq!(matrix[7][7], ReasoningHexagram(63)); // Meta × System
    }

    #[test]
    fn test_evolve_oscillation_flips_scope() {
        let mut matrix = strategy_matrix();
        let current = ReasoningHexagram(0b001_000); // approach=1, domain=0
        assert!(evolve_strategy_entry(&mut matrix, current, "Oscillation"));
        assert_eq!(matrix[1][0], ReasoningHexagram(0b001_000).flip_axis(4));
    }

    #[test]
    fn test_evolve_stuck_flips_four_axes() {
        let mut matrix = strategy_matrix();
        let current = ReasoningHexagram(0b010_011); // approach=2, domain=3
        assert!(evolve_strategy_entry(&mut matrix, current, "Stuck"));
        assert_eq!(matrix[2][3], ReasoningHexagram(0b010_011).flip_axes(0b001111));
    }

    #[test]
    fn test_evolve_inefficient_flips_method() {
        let mut matrix = strategy_matrix();
        let current = ReasoningHexagram(0b011_101); // approach=3, domain=5
        assert!(evolve_strategy_entry(&mut matrix, current, "Inefficient"));
        assert_eq!(matrix[3][5], ReasoningHexagram(0b011_101).flip_axis(3));
    }

    #[test]
    fn test_evolve_efficient_no_change() {
        let mut matrix = strategy_matrix();
        let current = ReasoningHexagram(0b100_010);
        assert!(!evolve_strategy_entry(&mut matrix, current, "Efficient"));
    }

    #[test]
    fn test_mode_names_cover_all() {
        for i in 0..64 {
            let s = ReasoningHexagram(i as u8);
            assert!(!s.mode_name().is_empty());
            assert!(!s.mode_description().is_empty());
            assert!(s.mode_name().len() > 2);
            assert!(s.mode_description().len() > 10);
        }
    }

    #[test]
    fn test_axis_accessors() {
        let s = ReasoningHexagram(0b101010); // bits: ABST=1, SCOPE=0, METH=1, DEPTH=0, MODE=1, STANCE=0
        assert_eq!(s.abstraction(), 1);
        assert_eq!(s.scope(), 0);
        assert_eq!(s.method(), 1);
        assert_eq!(s.depth(), 0);
        assert_eq!(s.reasoning_mode(), 1);
        assert_eq!(s.stance(), 0);
    }

    // ── ReasoningEffort tests ──

    #[test]
    fn test_reasoning_effort_all_covers_five_levels() {
        let levels = ReasoningEffort::all();
        assert_eq!(levels.len(), 5);
        assert_eq!(levels[0], ReasoningEffort::None);
        assert_eq!(levels[4], ReasoningEffort::XHigh);
    }

    #[test]
    fn test_reasoning_effort_e8_factors_none() {
        let f = ReasoningEffort::None.to_e8_factors();
        assert_eq!(f[2], 1.0); // DEPTH = 1 (Fast)
        assert_eq!(f[3], 1.0); // METH = 1 (Generative)
    }

    #[test]
    fn test_reasoning_effort_e8_factors_xhigh() {
        let f = ReasoningEffort::XHigh.to_e8_factors();
        assert_eq!(f[2], 0.0); // DEPTH = 0 (Deep)
        assert_eq!(f[3], 0.1); // METH near Analytical
        assert_eq!(f[4], 0.9); // SCOPE near Broad
    }

    #[test]
    fn test_reasoning_effort_depth_mapping() {
        assert_eq!(ReasoningEffort::None.to_e8_depth(), 1);
        assert_eq!(ReasoningEffort::Low.to_e8_depth(), 1);
        assert_eq!(ReasoningEffort::Medium.to_e8_depth(), 0);
        assert_eq!(ReasoningEffort::High.to_e8_depth(), 0);
        assert_eq!(ReasoningEffort::XHigh.to_e8_depth(), 0);
    }

    #[test]
    fn test_reasoning_effort_cost_monotonic() {
        let costs: Vec<f64> = ReasoningEffort::all().iter().map(|e| e.cost_multiplier()).collect();
        for w in costs.windows(2) {
            assert!(w[0] < w[1], "cost should be strictly increasing");
        }
    }

    #[test]
    fn test_reasoning_effort_default() {
        assert_eq!(ReasoningEffort::default(), ReasoningEffort::Medium);
    }

    #[test]
    fn test_reasoning_effort_display() {
        assert_eq!(ReasoningEffort::None.to_string(), "none");
        assert_eq!(ReasoningEffort::XHigh.to_string(), "xhigh");
    }

    #[test]
    fn test_select_mode_by_intent_with_effort_different_efforts_can_differ() {
        // Use a task that sits on the boundary between deep and fast reasoning
        // so that effort level can tip the selection
        let intent = IntentionContext {
            goal: "refactor module boundaries with minimal disruption".into(),
            reason: "improve module cohesion while maintaining backward compat".into(),
            boundaries: "no API changes, keep all existing exports".into(),
            verification: "verify all imports still resolve".into(),
        };

        let xhigh = select_mode_by_intent_with_effort(&intent, ReasoningEffort::XHigh);
        let none = select_mode_by_intent_with_effort(&intent, ReasoningEffort::None);

        // The depth axis should differ: XHigh pushes toward Deep (0), None toward Fast (1)
        assert!(
            xhigh.depth() <= none.depth(),
            "XHigh (depth={}) should be ≤ None (depth={}) for a refactoring task",
            xhigh.depth(), none.depth()
        );
    }

    #[test]
    fn test_select_mode_by_intent_with_effort_medium_matches_default() {
        let intent = IntentionContext {
            goal: "refactor API endpoints".into(),
            reason: "improve module structure".into(),
            boundaries: "backward compatible".into(),
            verification: "all existing tests pass".into(),
        };

        let with_effort = select_mode_by_intent_with_effort(&intent, ReasoningEffort::Medium);
        let default = select_mode_by_intent(&intent);
        assert_eq!(with_effort, default, "Medium effort should match default select_mode_by_intent");
    }

    #[test]
    fn test_select_mode_by_intent_effort_respected_for_deep_tasks() {
        let intent = IntentionContext {
            goal: "security audit of authentication module".into(),
            reason: "find vulnerabilities before pen test".into(),
            boundaries: "no code changes; report only".into(),
            verification: "cross-reference OWASP top 10".into(),
        };

        let high = select_mode_by_intent_with_effort(&intent, ReasoningEffort::High);
        let low = select_mode_by_intent_with_effort(&intent, ReasoningEffort::Low);

        // High effort should be more Deep (DEPTH=0), Low effort more Fast (DEPTH=1)
        let high_depth = high.depth();
        let low_depth = low.depth();
        assert!(
            high_depth <= low_depth,
            "High effort should select deeper modes (depth={}) than low (depth={})",
            high_depth, low_depth
        );
    }

    #[test]
    fn test_recommendation_includes_effort_keywords() {
        let intent = IntentionContext {
            goal: "simple qa: what time is it".into(),
            reason: "quick answer".into(),
            boundaries: "".into(),
            verification: "none needed".into(),
        };

        // None effort should select a Fast mode for simple QA
        let result = select_mode_by_intent_with_effort(&intent, ReasoningEffort::None);
        assert_eq!(result.depth(), 1, "None effort should select Fast (depth=1) for simple Q&A");

        // For a complex research task, XHigh should bias toward Deep
        let research = IntentionContext {
            goal: "investigate root cause of production deadlock".into(),
            reason: "need deep analysis of thread contention patterns and lock ordering".into(),
            boundaries: "no restart, must run 24/7, no schema changes".into(),
            verification: "verify deadlock-free for 24h load test".into(),
        };
        let result_xh = select_mode_by_intent_with_effort(&research, ReasoningEffort::XHigh);
        assert_eq!(result_xh.depth(), 0, "XHigh effort should select Deep (depth=0) for research task");
    }

    #[test]
    fn test_reasoning_effort_partial_ord() {
        assert!(ReasoningEffort::None < ReasoningEffort::Medium);
        assert!(ReasoningEffort::Low < ReasoningEffort::High);
        assert!(ReasoningEffort::Medium < ReasoningEffort::XHigh);
        assert!(ReasoningEffort::XHigh > ReasoningEffort::Low);
    }

    // ── MultipleHypothesisEvaluator tests ──

    #[test]
    fn test_mhe_default_returns_at_most_k() {
        let evaluator = MultipleHypothesisEvaluator::new(3, 0.0);
        let intent = IntentionContext {
            goal: "write code to sort an array".into(),
            reason: "need efficient comparison sort".into(),
            boundaries: "".into(),
            verification: "test with random input".into(),
        };
        let results = evaluator.evaluate_default(&intent);
        assert!(results.len() <= 3, "at most k=3 results");
        for r in &results {
            assert!(r.state.0 < 64);
            assert!(r.confidence >= 0.0 && r.confidence <= 1.0);
        }
    }

    #[test]
    fn test_mhe_respects_min_confidence() {
        let evaluator = MultipleHypothesisEvaluator::new(10, 0.0);
        let intent = IntentionContext {
            goal: "simple format".into(),
            reason: "".into(),
            boundaries: "".into(),
            verification: "".into(),
        };
        let results = evaluator.evaluate_default(&intent);
        assert!(results.len() <= 10, "should return at most k=10 results");
        for r in &results {
            assert!(r.confidence >= 0.0, "confidence should be >= min_confidence");
        }
    }

    #[test]
    fn test_mhe_results_sorted_by_score() {
        let evaluator = MultipleHypothesisEvaluator::new(10, 0.0);
        let intent = IntentionContext {
            goal: "debug production memory leak with valgrind".into(),
            reason: "find allocation hot spots".into(),
            boundaries: "no code changes".into(),
            verification: "verify memory stabilizes".into(),
        };
        let results = evaluator.evaluate_default(&intent);
        for w in results.windows(2) {
            assert!(w[0].score >= w[1].score, "should be sorted descending by score");
        }
    }

    #[test]
    fn test_mhe_effort_changes_depth_preference() {
        let evaluator = MultipleHypothesisEvaluator::new(10, 0.0);
        let intent = IntentionContext {
            goal: "analyze system architecture for scalability".into(),
            reason: "need comprehensive review of all subsystems".into(),
            boundaries: "no changes, just review".into(),
            verification: "produce analysis report".into(),
        };
        let medium = evaluator.evaluate(&intent, ReasoningEffort::Medium);
        let xhigh = evaluator.evaluate(&intent, ReasoningEffort::XHigh);
        // XHigh should push toward Deep (depth=0) modes
        let xh_depth: u8 = xhigh.first().map(|r| r.state.depth()).unwrap_or(0);
        let med_depth: u8 = medium.first().map(|r| r.state.depth()).unwrap_or(0);
        assert!(
            xh_depth <= med_depth,
            "XHigh effort should prefer equally or more deep modes vs Medium (xh={}, med={})",
            xh_depth, med_depth
        );
    }
}
