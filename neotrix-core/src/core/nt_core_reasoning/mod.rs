pub mod binary_vsa_attention;
pub mod dead_end_detector;
pub mod parallel_hypothesis;
pub mod spike_processor;
mod vsa_blackboard;
mod vsa_reasoner;

pub mod mcts_cognitive_module;
pub mod mcts_reasoner;
pub use binary_vsa_attention::{BinaryAttentionHead, BinaryHDVector, LARSVSAttention};
pub use dead_end_detector::{
    DeadEndConfig, DeadEndDetector, DeadEndReport, DeadEndStats, DeadEndType, RecoveryStrategy,
};
pub use mcts_reasoner::{MctsConfig, MctsNode, MctsReasoner, MctsStats};
pub use parallel_hypothesis::{
    CompetingHypothesis, ParallelHypEvalStats, ParallelHypothesisConfig,
    ParallelHypothesisEvaluator,
};
pub use vsa_blackboard::{ExpertType, Hypothesis, VsaBlackboard};
pub use vsa_reasoner::{BenchmarkReport, ReasonerConfig, VsaReasoner};

pub mod bidirectional_pruner;
pub mod counterfactual_cognitive_module;
pub mod counterfactual_simulator;
pub mod dead_end_cognitive_module;
pub mod parallel_hypothesis_cognitive_module;
pub mod pipeline_orchestrator;
pub mod prm_cognitive_module;
pub mod process_reward_model;
pub mod pruner_cognitive_module;
pub mod selector_cognitive_module;
pub mod strategy_selector;

pub use bidirectional_pruner::{
    BidirectionalPruner, PruneReason, PruneReport, PrunerConfig, PrunerStats, ReasoningPath,
};
pub use counterfactual_cognitive_module::CounterfactualCognitiveModule;
pub use counterfactual_simulator::{
    CounterfactualConfig, CounterfactualScenario, CounterfactualSimulator, CounterfactualStats,
    CounterfactualType, SimulatedOutcome,
};
pub use dead_end_cognitive_module::DeadEndCognitiveModule;
pub use parallel_hypothesis_cognitive_module::ParallelHypothesisCognitiveModule;
pub use pipeline_orchestrator::{PipelineConfig, PipelineReport, ReasoningPipeline};
pub use prm_cognitive_module::PrmCognitiveModule;
pub use process_reward_model::{PrmConfig, PrmStats, ProcessRewardModel, ReasoningStep, StepType};
pub use pruner_cognitive_module::PrunerCognitiveModule;
pub use selector_cognitive_module::SelectorCognitiveModule;
pub use strategy_selector::{
    ReasoningStrategy, SelectorStats, SelfHealingSelector, StrategyConfig, StrategyPerformance,
};
