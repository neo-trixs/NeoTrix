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
    /// Create a new reasoning state (panics if bits >= 64).
    pub fn new(bits: u8) -> Self {
        assert!(bits < 64, "ReasoningHexagram must be 0..63, got {bits}");
        Self(bits)
    }

    /// Get the value of a specific reasoning axis (0=LSB, 5=MSB).
    pub fn axis(&self, i: usize) -> u8 {
        (self.0 >> i) & 1
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
        Self(self.0 ^ (1 << i))
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
    &["brainstorm", "idea generation", "creative"],
    &["workshop", "ideation", "creative session"],
    &["formal proof", "theorem", "verification", "correctness proof"],
    &["proof assistant", "coq", "lean", "isabelle"],
    &["model check", "state space", "formal verification"],
    &["specification check", "requirement verification"],
    &["spec review", "requirement review", "specification"],
    &["spec collaboration", "requirement workshop"],
    &["explore", "investigate", "research", "discover"],
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
    &["generate", "code gen", "implement", "create"],
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

/// Generate all 64 reasoning states.
pub fn all_reasoning_states() -> Vec<ReasoningHexagram> {
    (0..64).map(ReasoningHexagram).collect()
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    let approach = (current.0 >> 3) as usize;
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
        assert_eq!(*path.states.last().expect("path should have states"), goal);
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
}
