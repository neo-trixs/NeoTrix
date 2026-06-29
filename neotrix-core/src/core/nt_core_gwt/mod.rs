// AUTO-GENERATED MODULE DECLARATIONS
pub mod curiosity_exploration;
pub mod epistemic_queue;
pub mod goal_synthesis;
pub mod intrinsic_drive;
pub mod manar_attention;
pub mod module_def;
pub mod monitor;
pub mod multi_modal_curiosity;
pub mod physics_attention;
pub mod resonance;

pub use curiosity_exploration::{
    CuriosityConfig, CuriosityExploration, CuriosityStats, KnowledgeGap,
};
pub mod self_interrupt;

pub use self_interrupt::{
    InterruptLevel, InterruptSignal, InterruptStats, SelfInterruptConfig, SelfInterruptSystem,
};

#[cfg(test)]
mod tests {}
