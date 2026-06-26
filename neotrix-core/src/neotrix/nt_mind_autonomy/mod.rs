pub mod arch_optimizer;
pub mod awareness_monitor;
pub mod cross_session_memory;
pub mod knowledge_distiller;
pub mod meta_goal_generator;
pub mod oracle_gate;
pub mod trend_analyzer;

pub use arch_optimizer::SelfArchitectureOptimizer;
pub use awareness_monitor::SelfAwarenessMonitor;
pub use cross_session_memory::CrossSessionMemory;
pub use knowledge_distiller::KnowledgeDistiller;
pub use meta_goal_generator::MetaGoalGenerator;
pub use oracle_gate::OracleGate;
pub use trend_analyzer::EvolutionTrendAnalyzer;
