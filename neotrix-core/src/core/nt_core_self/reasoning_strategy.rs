use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum StrategyKind {
    Direct,
    ChainOfThought,
    Reflection,
    ToolAssisted,
    RecursiveDecomposition,
    Deliberate,
    Intuitive,
    Hypothetical,
    CompareAndContrast,
    IterativeRefinement,
    LifecycleAnalysis,
    StructuralIntegrity,
}

impl StrategyKind {
    pub fn all() -> Vec<StrategyKind> {
        vec![
            StrategyKind::Direct,
            StrategyKind::ChainOfThought,
            StrategyKind::Reflection,
            StrategyKind::ToolAssisted,
            StrategyKind::RecursiveDecomposition,
            StrategyKind::Deliberate,
            StrategyKind::Intuitive,
            StrategyKind::Hypothetical,
            StrategyKind::CompareAndContrast,
            StrategyKind::IterativeRefinement,
            StrategyKind::LifecycleAnalysis,
            StrategyKind::StructuralIntegrity,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            StrategyKind::Direct => "direct",
            StrategyKind::ChainOfThought => "chain_of_thought",
            StrategyKind::Reflection => "reflection",
            StrategyKind::ToolAssisted => "tool_assisted",
            StrategyKind::RecursiveDecomposition => "recursive_decomposition",
            StrategyKind::Deliberate => "deliberate",
            StrategyKind::Intuitive => "intuitive",
            StrategyKind::Hypothetical => "hypothetical",
            StrategyKind::CompareAndContrast => "compare_and_contrast",
            StrategyKind::IterativeRefinement => "iterative_refinement",
            StrategyKind::LifecycleAnalysis => "lifecycle_analysis",
            StrategyKind::StructuralIntegrity => "structural_integrity",
        }
    }

    pub fn complexity(&self) -> u8 {
        match self {
            StrategyKind::Direct => 1,
            StrategyKind::Intuitive => 2,
            StrategyKind::ChainOfThought => 4,
            StrategyKind::Reflection => 5,
            StrategyKind::ToolAssisted => 3,
            StrategyKind::RecursiveDecomposition => 6,
            StrategyKind::Deliberate => 4,
            StrategyKind::Hypothetical => 3,
            StrategyKind::CompareAndContrast => 3,
            StrategyKind::IterativeRefinement => 5,
            StrategyKind::LifecycleAnalysis => 4,
            StrategyKind::StructuralIntegrity => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReasoningStrategy {
    pub kind: StrategyKind,
    pub description: String,
    pub preconditions: Vec<String>,
    pub steps: Vec<String>,
    pub success_criteria: Vec<String>,
    pub use_count: usize,
    pub effectiveness: f64,
}

impl ReasoningStrategy {
    pub fn new(kind: StrategyKind) -> Self {
        let (description, steps) = match kind {
            StrategyKind::Direct => (
                "Answer directly with available knowledge, no intermediate reasoning".into(),
                vec!["Understand question".into(), "Retrieve knowledge".into(), "Answer directly".into()],
            ),
            StrategyKind::ChainOfThought => (
                "Break down reasoning into intermediate steps, showing work".into(),
                vec!["Parse problem".into(), "Identify knowns/unknowns".into(), "Reason step by step".into(), "Verify intermediate results".into(), "Conclude".into()],
            ),
            StrategyKind::Reflection => (
                "Generate hypothesis, then critically examine it".into(),
                vec!["Generate initial answer".into(), "Critique own answer".into(), "Identify weaknesses".into(), "Revise".into(), "Finalize".into()],
            ),
            StrategyKind::ToolAssisted => (
                "Use external tools to augment reasoning".into(),
                vec!["Identify information need".into(), "Select appropriate tool".into(), "Execute tool call".into(), "Analyze result".into(), "Synthesize answer".into()],
            ),
            StrategyKind::RecursiveDecomposition => (
                "Recursively decompose complex problems into sub-problems".into(),
                vec!["Decompose problem".into(), "Solve sub-problems".into(), "Merge solutions".into(), "Verify completeness".into()],
            ),
            StrategyKind::Deliberate => (
                "Slow, careful reasoning with explicit consideration of alternatives".into(),
                vec!["Analyze problem space".into(), "List alternatives".into(), "Evaluate each".into(), "Select best".into(), "Justify choice".into()],
            ),
            StrategyKind::Intuitive => (
                "Pattern-match based rapid response".into(),
                vec!["Pattern match".into(), "Retrieve similar case".into(), "Adapt known solution".into()],
            ),
            StrategyKind::Hypothetical => (
                "Consider hypothetical scenarios and their implications".into(),
                vec!["State hypothesis".into(), "Assume conditions".into(), "Trace implications".into(), "Evaluate likelihood".into(), "Conclude".into()],
            ),
            StrategyKind::CompareAndContrast => (
                "Compare multiple options systematically".into(),
                vec!["List options".into(), "Define criteria".into(), "Compare each".into(), "Identify trade-offs".into(), "Recommend".into()],
            ),
            StrategyKind::IterativeRefinement => (
                "Start with rough solution, iteratively improve".into(),
                vec!["Draft solution".into(), "Test/review".into(), "Identify improvement".into(), "Refine".into(), "Repeat until satisfied".into()],
            ),
            StrategyKind::LifecycleAnalysis => (
                "Analyze resource lifecycles when wrapping OS event sources. Ensure the resource producer outlives the consumer.".into(),
                vec!["Identify OS resource".into(), "Determine producer scope".into(), "Ensure producer outlives consumer".into(), "Verify drop order".into(), "Test event delivery".into()],
            ),
            StrategyKind::StructuralIntegrity => (
                "After structural edits, verify brace matching, indent boundaries, and scope integrity.".into(),
                vec!["Identify structural change".into(), "Check {/} balance".into(), "Verify indent levels".into(), "Confirm no broken scopes".into(), "Compile check".into()],
            ),
        };
        Self {
            kind,
            description,
            preconditions: Vec::new(),
            steps,
            success_criteria: Vec::new(),
            use_count: 0,
            effectiveness: 0.5,
        }
    }

    pub fn record_use(&mut self, successful: bool) {
        self.use_count += 1;
        self.effectiveness = self.effectiveness * 0.9 + if successful { 0.1 } else { 0.0 };
    }
}

pub struct ReasoningStrategyRegistry {
    pub strategies: HashMap<StrategyKind, ReasoningStrategy>,
}

impl Default for ReasoningStrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ReasoningStrategyRegistry {
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        for kind in StrategyKind::all() {
            strategies.insert(kind, ReasoningStrategy::new(kind));
        }
        Self { strategies }
    }

    pub fn select(
        &self,
        task_complexity: u8,
        needs_tools: bool,
        needs_reflection: bool,
    ) -> StrategyKind {
        if needs_tools && task_complexity >= 3 {
            return StrategyKind::LifecycleAnalysis;
        }
        if needs_tools {
            return StrategyKind::ToolAssisted;
        }
        if needs_reflection && task_complexity < 4 {
            return StrategyKind::StructuralIntegrity;
        }
        if needs_reflection || task_complexity >= 6 {
            return StrategyKind::RecursiveDecomposition;
        }
        if task_complexity >= 4 {
            return StrategyKind::ChainOfThought;
        }
        StrategyKind::Direct
    }

    pub fn best_by_effectiveness(&self) -> Option<StrategyKind> {
        self.strategies
            .values()
            .max_by(|a, b| {
                a.effectiveness
                    .partial_cmp(&b.effectiveness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.kind)
    }

    pub fn record_outcome(&mut self, kind: StrategyKind, successful: bool) {
        if let Some(strategy) = self.strategies.get_mut(&kind) {
            strategy.record_use(successful);
        }
    }

    pub fn most_used(&self) -> Option<StrategyKind> {
        self.strategies
            .values()
            .max_by_key(|s| s.use_count)
            .map(|s| s.kind)
    }

    pub fn reset_stats(&mut self) {
        for strategy in self.strategies.values_mut() {
            strategy.use_count = 0;
            strategy.effectiveness = 0.5;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_strategies_created() {
        let registry = ReasoningStrategyRegistry::new();
        assert_eq!(registry.strategies.len(), 12);
    }

    #[test]
    fn test_select_direct_for_simple() {
        let registry = ReasoningStrategyRegistry::new();
        assert_eq!(registry.select(1, false, false), StrategyKind::Direct);
    }

    #[test]
    fn test_select_cot_for_complex() {
        let registry = ReasoningStrategyRegistry::new();
        assert_eq!(
            registry.select(5, false, false),
            StrategyKind::ChainOfThought
        );
    }

    #[test]
    fn test_select_tool_assisted() {
        let registry = ReasoningStrategyRegistry::new();
        assert_eq!(registry.select(1, true, false), StrategyKind::ToolAssisted);
    }

    #[test]
    fn test_select_recursive_for_reflection() {
        let registry = ReasoningStrategyRegistry::new();
        assert_eq!(
            registry.select(6, false, true),
            StrategyKind::RecursiveDecomposition
        );
    }

    #[test]
    fn test_record_outcome_affects_effectiveness() {
        let mut registry = ReasoningStrategyRegistry::new();
        let eff_before = registry.strategies[&StrategyKind::Direct].effectiveness;
        registry.record_outcome(StrategyKind::Direct, true);
        let eff_after = registry.strategies[&StrategyKind::Direct].effectiveness;
        assert!(eff_after > eff_before);
    }

    #[test]
    fn test_best_by_effectiveness() {
        let mut registry = ReasoningStrategyRegistry::new();
        for _ in 0..10 {
            registry.record_outcome(StrategyKind::Direct, true);
        }
        assert_eq!(registry.best_by_effectiveness(), Some(StrategyKind::Direct));
    }

    #[test]
    fn test_most_used_tracks_count() {
        let mut registry = ReasoningStrategyRegistry::new();
        registry.record_outcome(StrategyKind::Reflection, true);
        registry.record_outcome(StrategyKind::Reflection, true);
        registry.record_outcome(StrategyKind::Direct, true);
        assert_eq!(registry.most_used(), Some(StrategyKind::Reflection));
    }

    #[test]
    fn test_strategy_complexity_ordering() {
        assert!(
            StrategyKind::RecursiveDecomposition.complexity() > StrategyKind::Direct.complexity()
        );
        assert!(StrategyKind::ChainOfThought.complexity() > StrategyKind::Intuitive.complexity());
    }

    #[test]
    fn test_select_lifecycle_analysis() {
        let registry = ReasoningStrategyRegistry::new();
        assert_eq!(
            registry.select(3, true, false),
            StrategyKind::LifecycleAnalysis
        );
    }

    #[test]
    fn test_select_structural_integrity() {
        let registry = ReasoningStrategyRegistry::new();
        assert_eq!(
            registry.select(3, false, true),
            StrategyKind::StructuralIntegrity
        );
    }

    #[test]
    fn test_reset_stats() {
        let mut registry = ReasoningStrategyRegistry::new();
        registry.record_outcome(StrategyKind::Direct, true);
        registry.reset_stats();
        assert_eq!(registry.strategies[&StrategyKind::Direct].use_count, 0);
        assert!((registry.strategies[&StrategyKind::Direct].effectiveness - 0.5).abs() < 1e-6);
    }
}
