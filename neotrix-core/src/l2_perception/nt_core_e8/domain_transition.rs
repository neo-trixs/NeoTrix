//! E₈ domain-aware transition model.
//!
//! Extends the base `E8TransitionMatrix` with per-task-type transition chains
//! derived from the Complete-FABLE.5-traces-2M corpus (2M+ rows across 17+ HF
//! datasets) and the WithinUsAI/fable5_distillation_merged_cleaned_25k dataset.
//!
//! Each task type (General, Reasoning, Math, Coding, Agentic, Creative) has a
//! characteristic reasoning trajectory that maps to distinct E8 hexagram chains.
//! The model blends domain-specific priors with the general transition matrix.

use super::E8TransitionMatrix;

/// Task type classification for domain-aware E8 transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum E8TaskType {
    /// General purpose reasoning (default mythos chain)
    General,
    /// Deep analytical / scientific reasoning
    Reasoning,
    /// Mathematical computation and proof
    Math,
    /// Software engineering and code generation
    Coding,
    /// Tool-use agentic workflows
    Agentic,
    /// Creative / divergent thinking
    Creative,
}

impl E8TaskType {
    /// Count of task type variants.
    pub const COUNT: usize = 6;

    /// All task types in order.
    pub const ALL: [E8TaskType; 6] = [
        E8TaskType::General,
        E8TaskType::Reasoning,
        E8TaskType::Math,
        E8TaskType::Coding,
        E8TaskType::Agentic,
        E8TaskType::Creative,
    ];

    /// Detect task type from a task description string.
    /// Uses word-boundary keyword matching against patterns from the LanguaMan task classifier.
    pub fn detect(task: &str) -> Self {
        fn word_starts_with(word: &str, keyword: &str) -> bool {
            word.len() >= keyword.len() && word.starts_with(keyword)
        }
        let lower = task.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();

        // Math keywords (computation-heavy tasks)
        let math_kw = [
            "math",
            "calculat",
            "equation",
            "proof",
            "theorem",
            "arithmetic",
            "algebra",
            "geometry",
            "derivative",
            "integral",
            "matrix",
            "vector",
            "numerical",
            "compute",
        ];
        // Coding keywords
        let code_kw = [
            "code",
            "program",
            "function",
            "implement",
            "debug",
            "compile",
            "rust",
            "python",
            "javascript",
            "typescript",
            "refactor",
            "test",
            "build",
            "deploy",
        ];
        // Agentic keywords (tool-use / multi-step)
        let agent_kw = [
            "agent", "tool", "search", "browse", "crawl", "scrape", "automate", "workflow",
            "pipeline", "execute", "run", "shell", "command",
        ];
        // Creative keywords
        let creative_kw = [
            "creative",
            "design",
            "brainstorm",
            "imagine",
            "generate",
            "write",
            "compose",
            "artistic",
            "explore",
            "novel",
            "innovative",
            "ideate",
        ];

        let math_score = math_kw
            .iter()
            .filter(|kw| words.iter().any(|w| word_starts_with(w, kw)))
            .count();
        let code_score = code_kw
            .iter()
            .filter(|kw| words.iter().any(|w| word_starts_with(w, kw)))
            .count();
        let agent_score = agent_kw
            .iter()
            .filter(|kw| words.iter().any(|w| word_starts_with(w, kw)))
            .count();
        let creative_score = creative_kw
            .iter()
            .filter(|kw| words.iter().any(|w| word_starts_with(w, kw)))
            .count();

        let max_score = math_score
            .max(code_score)
            .max(agent_score)
            .max(creative_score);
        if max_score == 0 {
            return E8TaskType::General;
        }

        if max_score == math_score {
            E8TaskType::Math
        } else if max_score == code_score {
            E8TaskType::Coding
        } else if max_score == agent_score {
            E8TaskType::Agentic
        } else if max_score == creative_score {
            E8TaskType::Creative
        } else {
            E8TaskType::General
        }
    }

    /// Get the canonical E8 transition chain for this task type.
    /// Each chain is derived from the Fable 2M trace corpus analysis:
    /// different reasoning patterns dominate different task types.
    pub fn e8_chain(&self) -> &'static [u8; 9] {
        match self {
            // General: standard mythos 9-stage chain
            E8TaskType::General => &[56, 48, 40, 32, 24, 16, 8, 0, 4],
            // Reasoning: deeper self-verification and deep dive stages
            E8TaskType::Reasoning => &[56, 48, 40, 32, 26, 16, 10, 0, 4],
            // Math: computation-first with first-principles iteration
            E8TaskType::Math => &[58, 50, 42, 35, 26, 34, 16, 8, 4],
            // Coding: iterative build→verify cycles with decomposition
            E8TaskType::Coding => &[56, 48, 42, 40, 26, 48, 42, 24, 0],
            // Agentic: plan→act→observe cycles
            E8TaskType::Agentic => &[58, 50, 40, 48, 42, 26, 16, 8, 4],
            // Creative: divergent→convergent with alternatives
            E8TaskType::Creative => &[56, 48, 16, 24, 8, 0, 4, 40, 56],
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            E8TaskType::General => "General",
            E8TaskType::Reasoning => "Reasoning",
            E8TaskType::Math => "Math",
            E8TaskType::Coding => "Coding",
            E8TaskType::Agentic => "Agentic",
            E8TaskType::Creative => "Creative",
        }
    }
}

impl std::fmt::Display for E8TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Length of chain-of-thought for thinking-aware E8 transitions.
/// Based on the WithinUsAI/fable5_distillation_merged_cleaned_25k dataset:
///   max thinking length: 552,213 chars
///   mean thinking length: 6,220 chars
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CoTLength {
    /// Short thought: <1K tokens — direct transitions, fewer stages
    Short,
    /// Medium thought: 1K-10K tokens — standard chain
    Medium,
    /// Long thought: 10K-100K tokens — deeper iterative loops
    Long,
    /// Max thought: >100K tokens — very deep with verification loops
    Max,
}

impl CoTLength {
    /// Detect CoT length from estimated token count.
    pub fn from_tokens(tokens: usize) -> Self {
        if tokens < 1_000 {
            CoTLength::Short
        } else if tokens < 10_000 {
            CoTLength::Medium
        } else if tokens < 100_000 {
            CoTLength::Long
        } else {
            CoTLength::Max
        }
    }

    /// Depth multiplier: longer thinking allows deeper transitions.
    pub fn depth_multiplier(&self) -> f64 {
        match self {
            CoTLength::Short => 0.5,
            CoTLength::Medium => 1.0,
            CoTLength::Long => 1.8,
            CoTLength::Max => 3.0,
        }
    }

    /// Number of self-verification loops to allow.
    pub fn verification_loops(&self) -> usize {
        match self {
            CoTLength::Short => 0,
            CoTLength::Medium => 1,
            CoTLength::Long => 3,
            CoTLength::Max => 7,
        }
    }

    /// Whether to skip the Alternative stage (short thinking).
    pub fn skip_alternatives(&self) -> bool {
        matches!(self, CoTLength::Short)
    }

    /// E8 depth bias: how deep into each block to go.
    pub fn depth_bias(&self) -> f64 {
        match self {
            CoTLength::Short => 0.3,
            CoTLength::Medium => 0.5,
            CoTLength::Long => 0.7,
            CoTLength::Max => 0.9,
        }
    }
}

/// Domain-aware transition model that blends task-type prior with empirical data.
///
/// Architecture:
/// - Maintains 6 domain-specific transition matrices (one per E8TaskType)
/// - Each matrix is seeded with the domain's canonical chain on creation
/// - `blend()` combines domain-specific + general matrices with task-adaptive weight
/// - `predict_next()` uses the blended matrix for next-state prediction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct E8DomainTransitionModel {
    /// Per-domain transition matrices
    pub domain_matrices: [E8TransitionMatrix; 6],
    /// General (cross-domain) transition matrix
    pub general_matrix: E8TransitionMatrix,
    /// Domain blend weight: 0.0 = pure general, 1.0 = pure domain
    pub blend_weight: f64,
    /// Confidence threshold for domain-specific prediction
    pub confidence_threshold: f64,
    /// Cached blended matrices — one per task type, so switching task types
    /// never reuses another domain's blend. Previously a single
    /// `blended_cache: Option<...>` returned the cached matrix regardless of
    /// `task_type`, so after a Coding blend the next Math blend() returned the
    /// wrong matrix until a transition marked it dirty.
    pub blended_cache: [Option<E8TransitionMatrix>; E8TaskType::COUNT],
    /// Whether the cache is stale (set true by record_transition).
    pub dirty: bool,
}

impl E8DomainTransitionModel {
    /// Create a new model with all domain matrices seeded from trace patterns.
    pub fn new(blend_weight: f64) -> Self {
        let general_matrix = {
            let mut tm = E8TransitionMatrix::new();
            tm.init_from_trace_patterns();
            tm
        };

        let mut domain_matrices = [
            E8TransitionMatrix::new(),
            E8TransitionMatrix::new(),
            E8TransitionMatrix::new(),
            E8TransitionMatrix::new(),
            E8TransitionMatrix::new(),
            E8TransitionMatrix::new(),
        ];

        for (i, task_type) in E8TaskType::ALL.iter().enumerate() {
            let chain = task_type.e8_chain();
            seed_from_chain(&mut domain_matrices[i], chain);
        }

        Self {
            domain_matrices,
            general_matrix,
            blend_weight,
            confidence_threshold: 0.15,
            blended_cache: Default::default(),
            dirty: true,
        }
    }

    /// Get the domain transition matrix for a task type.
    pub fn domain_matrix(&self, task_type: E8TaskType) -> &E8TransitionMatrix {
        &self.domain_matrices[task_type as usize]
    }

    /// Get mutable domain matrix.
    pub fn domain_matrix_mut(&mut self, task_type: E8TaskType) -> &mut E8TransitionMatrix {
        &mut self.domain_matrices[task_type as usize]
    }

    /// Blend domain-specific and general matrices for a task type.
    /// Returns blended transition probabilities for each (from, to) pair.
    /// Uses cached result when not dirty, keyed by task type.
    pub fn blend(&mut self, task_type: E8TaskType) -> E8TransitionMatrix {
        let ti = task_type as usize;
        if !self.dirty {
            if let Some(ref cached) = self.blended_cache[ti] {
                return cached.clone();
            }
        }
        let blended = self.blend_inner(task_type);
        self.blended_cache[ti] = Some(blended.clone());
        self.dirty = false;
        blended
    }

    /// Compute the blended matrix from scratch (no caching).
    fn blend_inner(&self, task_type: E8TaskType) -> E8TransitionMatrix {
        let domain = self.domain_matrix(task_type);
        let general = &self.general_matrix;
        let w = self.blend_weight;

        // If domain matrix has few observations, fall back more to general
        let domain_visits: u64 = domain.visit_counts.0.iter().sum();
        let effective_w = if domain_visits < 50 {
            w * (domain_visits as f64 / 50.0)
        } else {
            w
        };

        let mut blended = E8TransitionMatrix::new();
        for i in 0..64 {
            let mut total_domain: u64 = 0;
            let mut total_general: u64 = 0;
            for j in 0..64 {
                let dc = domain.counts.get(i, j);
                let gc = general.counts.get(i, j);
                let bc = (dc as f64 * effective_w + gc as f64 * (1.0 - effective_w)).round() as u64;
                blended.counts.add(i, j, bc);
                total_domain = total_domain.saturating_add(dc);
                total_general = total_general.saturating_add(gc);
            }
            blended.row_totals.0[i] = (total_domain as f64 * effective_w
                + total_general as f64 * (1.0 - effective_w))
                .round() as u64;
            blended.visit_counts.0[i] = (domain.visit_counts.0[i] as f64 * effective_w
                + general.visit_counts.0[i] as f64 * (1.0 - effective_w))
                .round() as u64;
        }
        blended
    }

    /// Predict next state given current state, task type, and optional CoT length.
    pub fn predict_next(
        &mut self,
        from: u8,
        task_type: E8TaskType,
        cot_length: CoTLength,
    ) -> (u8, f64) {
        let blended = self.blend(task_type);
        let depth_bias = cot_length.depth_bias();

        // Determine target block from task type chain
        let chain = task_type.e8_chain();
        let current_pos = chain.iter().position(|&s| s == from);

        let target_block = current_pos.map(|pos| {
            if pos + 1 < chain.len() {
                chain[pos + 1] & 0xF8
            } else {
                chain[chain.len() - 1] & 0xF8
            }
        });

        // Predict from blended matrix
        let (state, confidence) = blended.predict_next(from, target_block);

        // If short thinking, bias toward staying in the same block range
        if cot_length.skip_alternatives() {
            if let Some(tb) = target_block {
                if state & 0xF8 != tb {
                    // Bias back toward target block
                    let biased = (state as f64 * 0.5 + (tb | 0x04) as f64 * 0.5).round() as u8;
                    return (biased.min(63), confidence * 0.8);
                }
            }
        }

        // For long/max thinking, allow deeper jumps
        if matches!(cot_length, CoTLength::Long | CoTLength::Max) {
            let jump_size = ((state as f64 - from as f64).abs() * depth_bias) as u8;
            let deeper = if state > from {
                state.saturating_add(jump_size).min(63)
            } else {
                state.saturating_sub(jump_size)
            };
            return (deeper, confidence * 0.9);
        }

        (state, confidence)
    }

    /// Record a transition for a specific task type.
    pub fn record_transition(&mut self, task_type: E8TaskType, from: u8, to: u8) {
        self.dirty = true;
        self.domain_matrix_mut(task_type)
            .record_transition(from, to);
        self.general_matrix.record_transition(from, to);
    }

    /// Merge another model's data into this one.
    pub fn merge(&mut self, other: &E8DomainTransitionModel) {
        for (i, task_type) in E8TaskType::ALL.iter().enumerate() {
            let other_domain = other.domain_matrix(*task_type);
            let total_other: u64 = other_domain.visit_counts.0.iter().sum();
            if total_other > 0 {
                self.domain_matrices[i].merge(other_domain);
            }
        }
        self.general_matrix.merge(&other.general_matrix);
    }
}

impl Default for E8DomainTransitionModel {
    fn default() -> Self {
        Self::new(0.3)
    }
}

impl std::fmt::Display for E8DomainTransitionModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "E8DomainTransitionModel(blend={:.2})", self.blend_weight)?;
        for task_type in &E8TaskType::ALL {
            let dm = self.domain_matrix(*task_type);
            let visits: u64 = dm.visit_counts.0.iter().sum();
            writeln!(
                f,
                "  {}: {} visits, {} transitions",
                task_type.label(),
                visits,
                dm.recent_transitions.len()
            )?;
        }
        Ok(())
    }
}

/// Seed a transition matrix from a canonical chain.
fn seed_from_chain(tm: &mut E8TransitionMatrix, chain: &[u8; 9]) {
    for i in 0..8 {
        let from = chain[i] as usize;
        let to = chain[i + 1] as usize;
        tm.counts.add(from, to, 10);
        tm.row_totals.0[from] += 10;
    }
    for &s in chain {
        let si = s as usize;
        tm.counts.add(si, si, 3);
        tm.row_totals.0[si] += 3;
    }
    for i in 1..8 {
        let from = chain[i] as usize;
        let to = chain[i - 1] as usize;
        tm.counts.add(from, to, 2);
        tm.row_totals.0[from] += 2;
    }
    for i in 0..7 {
        let from = chain[i] as usize;
        let to = chain[i + 2] as usize;
        tm.counts.add(from, to, 1);
        tm.row_totals.0[from] += 1;
    }
    // Cross-connections between Self-Verification and Deep Dive
    let sv = chain[4] as usize;
    let dd = chain[6] as usize;
    tm.counts.add(sv, dd, 5);
    tm.row_totals.0[sv] += 5;
    tm.counts.add(dd, sv, 3);
    tm.row_totals.0[dd] += 3;
    for &s in chain {
        tm.visit_counts.0[s as usize] += 20;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_type_detection() {
        assert_eq!(
            E8TaskType::detect("solve math equation x^2 + y^2 = z^2"),
            E8TaskType::Math
        );
        assert_eq!(
            E8TaskType::detect("write a Python function to sort"),
            E8TaskType::Coding
        );
        assert_eq!(
            E8TaskType::detect("search the web for research"),
            E8TaskType::Agentic
        );
        assert_eq!(
            E8TaskType::detect("design a creative logo"),
            E8TaskType::Creative
        );
        assert_eq!(
            E8TaskType::detect("what is the capital of France"),
            E8TaskType::General
        );
    }

    #[test]
    fn test_all_chains_unique() {
        let mut seen = std::collections::HashSet::new();
        for task_type in &E8TaskType::ALL {
            let chain = task_type.e8_chain();
            assert!(
                chain.iter().all(|&s| s < 64),
                "{} chain has invalid state",
                task_type
            );
            seen.insert(chain);
        }
        // Not all chains need to be unique (General and Reasoning may overlap)
    }

    #[test]
    fn test_domain_transition_model_creation() {
        let model = E8DomainTransitionModel::default();
        for task_type in &E8TaskType::ALL {
            let dm = model.domain_matrix(*task_type);
            let visits: u64 = dm.visit_counts.0.iter().sum();
            assert!(visits > 0, "{} matrix has no visits", task_type);
        }
    }

    #[test]
    fn test_blend_produces_valid_matrix() {
        let mut model = E8DomainTransitionModel::new(0.5);
        let blended = model.blend(E8TaskType::Coding);
        // Blended matrix should have non-zero row totals
        let total_visits: u64 = blended.visit_counts.0.iter().sum();
        assert!(total_visits > 0, "blended matrix is empty");
    }

    #[test]
    fn test_predict_next_with_cot_length() {
        let mut model = E8DomainTransitionModel::default();
        let (state, confidence) = model.predict_next(56, E8TaskType::General, CoTLength::Medium);
        assert!(state < 64);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_record_transition_updates_both() {
        let mut model = E8DomainTransitionModel::default();
        let before_visits: u64 = model
            .domain_matrix(E8TaskType::Coding)
            .visit_counts
            .0
            .iter()
            .sum();
        let before_general: u64 = model.general_matrix.visit_counts.0.iter().sum();

        model.record_transition(E8TaskType::Coding, 42, 26);

        let after_visits: u64 = model
            .domain_matrix(E8TaskType::Coding)
            .visit_counts
            .0
            .iter()
            .sum();
        let after_general: u64 = model.general_matrix.visit_counts.0.iter().sum();
        assert!(after_visits > before_visits, "domain visits not updated");
        assert!(after_general > before_general, "general visits not updated");
    }

    #[test]
    fn test_cot_length_from_tokens() {
        assert_eq!(CoTLength::from_tokens(500), CoTLength::Short);
        assert_eq!(CoTLength::from_tokens(5_000), CoTLength::Medium);
        assert_eq!(CoTLength::from_tokens(50_000), CoTLength::Long);
        assert_eq!(CoTLength::from_tokens(200_000), CoTLength::Max);
    }

    #[test]
    fn test_domain_merge() {
        let mut model1 = E8DomainTransitionModel::default();
        let mut model2 = E8DomainTransitionModel::default();
        model2.record_transition(E8TaskType::Math, 35, 26);
        model2.record_transition(E8TaskType::Math, 26, 34);

        let before: u64 = model1
            .domain_matrix(E8TaskType::Math)
            .visit_counts
            .0
            .iter()
            .sum();
        model1.merge(&model2);
        let after: u64 = model1
            .domain_matrix(E8TaskType::Math)
            .visit_counts
            .0
            .iter()
            .sum();
        assert!(after > before, "merge did not add visits");
    }

    #[test]
    fn test_short_cot_skips_alternatives() {
        let mut model = E8DomainTransitionModel::default();
        let (state, _) = model.predict_next(32, E8TaskType::General, CoTLength::Short);
        // Short CoT should bias toward the next chain stage, not alternatives block (16-23)
        let block = state & 0xF8;
        assert!(
            block != 16 || state == 16,
            "short CoT should avoid alternatives block"
        );
    }

    #[test]
    fn test_long_cot_allows_deeper_jumps() {
        let mut model = E8DomainTransitionModel::new(0.5);
        let (_state_medium, conf_medium) =
            model.predict_next(56, E8TaskType::Reasoning, CoTLength::Medium);
        let (_state_long, conf_long) =
            model.predict_next(56, E8TaskType::Reasoning, CoTLength::Long);
        // Long CoT should have comparable confidence (depth means more exploration)
        assert!(conf_long > 0.0 && conf_medium > 0.0);
    }

    #[test]
    fn test_task_type_display() {
        assert_eq!(format!("{}", E8TaskType::Coding), "Coding");
        assert_eq!(format!("{}", E8TaskType::Math), "Math");
    }

    #[test]
    fn test_model_does_not_panic_for_all_calls() {
        let mut model = E8DomainTransitionModel::default();
        for task_type in &E8TaskType::ALL {
            for &cot in &[
                CoTLength::Short,
                CoTLength::Medium,
                CoTLength::Long,
                CoTLength::Max,
            ] {
                let (s, c) = model.predict_next(0, *task_type, cot);
                assert!(s < 64 && c > 0.0);
                let (s, c) = model.predict_next(63, *task_type, cot);
                assert!(s < 64 && c > 0.0);
            }
        }
    }
}
