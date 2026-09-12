pub mod nt_mind;
pub mod nt_mind_background_loop;
pub mod nt_mind_benchmark;
pub mod nt_mind_hook;
pub mod nt_mind_skill_engine;

pub mod evolution;
pub mod foundation;
pub mod mind_modules;
pub mod seal;
pub mod harness;

// 从 L1 nt_act_autonomy 迁移过来的模块
pub mod meta_goal_generator;
pub mod knowledge_distiller;
pub mod trend_analyzer;
pub mod nt_mind_automation;

pub use meta_goal_generator::MetaGoalGenerator;
pub use knowledge_distiller::KnowledgeDistiller;
pub use trend_analyzer::EvolutionTrendAnalyzer;
pub use nt_mind_automation::{AutomationEngine, AutomationRule, AutomationTrigger, AutomationAction, PrAction};

pub mod reason {
    pub use super::nt_mind::reason::*;
}
pub mod infrastructure {
    pub use super::nt_mind::infrastructure::*;
}
pub mod benchmark {
    pub use super::nt_mind_benchmark::*;
}

// Re-exports for harness module
pub use harness::refinement::ContinualRefiner;
