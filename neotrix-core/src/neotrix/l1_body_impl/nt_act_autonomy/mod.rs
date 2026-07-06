pub mod awareness_monitor;
pub mod oracle_gate;
pub mod knowledge_distiller;
pub mod cross_session_memory;
pub mod arch_optimizer;
pub mod trend_analyzer;
pub mod meta_goal_generator;
pub mod per_agent;
pub mod nt_mind_automation;

pub use awareness_monitor::SelfAwarenessMonitor;
pub use oracle_gate::OracleGate;
pub use knowledge_distiller::KnowledgeDistiller;
pub use cross_session_memory::CrossSessionMemory;
pub use arch_optimizer::SelfArchitectureOptimizer;
pub use trend_analyzer::EvolutionTrendAnalyzer;
pub use meta_goal_generator::MetaGoalGenerator;
pub use nt_mind_automation::{AutomationEngine, AutomationRule, AutomationTrigger, AutomationAction, PrAction};
pub use per_agent::{
    ActionResult, Artifact, Executor, ExecutorAgent, LoopIteration, LoopOutcome,
    PerConfig, PlanExecuteReflectLoop, PlanRevision, PlanStep, Planner, PlannerAgent,
    Reflector, ReflectorAgent, StepStatus, TaskPlan, ModifiedStep,
};
