//! NeoTrix 意识核心 — 迭代验证引擎
//! 
//! 实现 1000+ 次循环迭代验证，每次寻找漏洞缺陷并循环补齐

pub mod agent;
pub mod probes;
pub mod patches;
pub mod convergence;
pub mod state;
pub mod demo;
pub mod self_observer;
pub mod self_evolver;
pub mod information_absorber;
pub mod knowledge_distiller;
pub mod resource_router;
pub mod pattern_engine;
pub mod causal_engine;
pub mod abstract_engine;
pub mod reasoning_generator;
pub mod extrapolator;
pub mod generator;
pub mod emergence_engine;
pub mod goal_setter;
pub mod value_judge;
pub mod meta_learner;
pub mod memory_kernel;
pub mod tamper_engine;
pub mod task_categorizer;
pub mod response_parser;

pub use agent::IterationAgent;
pub use probes::*;
pub use patches::*;
pub use convergence::ConvergenceChecker;
pub use state::StateSnapshot;
pub use self_observer::SelfObserver;
pub use self_evolver::SelfEvolver;
pub use information_absorber::InformationAbsorber;
pub use knowledge_distiller::KnowledgeDistiller;
pub use resource_router::ResourceRouter;
pub use pattern_engine::PatternEngine;
pub use causal_engine::CausalEngine;
pub use abstract_engine::AbstractEngine;
pub use reasoning_generator::ReasoningGenerator;
pub use extrapolator::Extrapolator;
pub use generator::Generator;
pub use emergence_engine::EmergenceEngine;
pub use goal_setter::GoalSetter;
pub use value_judge::ValueJudge;
pub use meta_learner::MetaLearner;
pub use memory_kernel::MemoryKernel;
pub use tamper_engine::TamperEngine;
pub use task_categorizer::TaskCategorizer;
pub use response_parser::ResponseParser;
