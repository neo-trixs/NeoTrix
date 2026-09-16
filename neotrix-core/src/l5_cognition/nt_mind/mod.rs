pub mod nt_mind;
pub mod nt_mind_background_loop;
pub mod nt_mind_benchmark;
pub mod nt_mind_hook;
pub mod nt_mind_skill_engine;
pub mod nt_game;

// ── Re-exports from nt_mind submodule for external examples/benches ──
pub use nt_mind::knowledge_engine;
pub use nt_mind::self_iterating;
pub use nt_mind::self_evolver;
pub use nt_mind::memory;
pub use nt_mind::web_miner;
pub use nt_mind::knowledge_miner;
pub use nt_mind::knowledge_chain;
pub use nt_mind::exploration_pipeline;
pub use nt_mind::reasoning_engine;
pub use nt_mind::reasoning_types;
pub use nt_mind::core;
pub use nt_mind::self_edit;

// Direct type re-exports for examples/benches
pub use nt_mind::{ReasoningBrain, SelfIteratingBrain, SelfEvolver, KnowledgeEngine,
    CapabilityVector, KnowledgeSource, ReasoningBank, ReasoningMemory, SelfEdit,
    KnowledgeEntry, RelationType, SourceType};

pub mod evolution;
pub mod foundation;
pub mod mind_modules;
pub mod seal;
pub mod harness;

pub mod mid_turn_steering;
pub use mid_turn_steering::MidTurnSteering;

pub mod rise_reflector;
pub use rise_reflector::RISEReflector;

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
