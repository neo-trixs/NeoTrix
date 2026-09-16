//! NT-GAME — Self-Evolving Consciousness Training Game Framework
//!
//! A game framework that trains NeoTrix's reasoning capabilities through
//! self-play, adaptive difficulty, and consciousness feedback loops.
//!
//! ## Architecture
//!
//! ```text
//! nt_game/
//! ├── framework.rs    — Core primitives: Actor, Episode, Rubric, Arena
//! ├── env.rs          — NtGameEnv trait + GameState types
//! ├── hex_crucible.rs — Primary training game (E8 hexagram strategy)
//! └── builtin/        — Built-in games (HexTicTacToe, 2048, ...)
//! ```
//!
//! ## Design Principles
//!
//! 1. **Text-native observations** (LLM-TeamGym): All game states are text strings
//! 2. **E8 hexagram mapping**: Game states map to reasoning modes
//! 3. **Consciousness feedback**: Game outcomes train SelfModel, Emotion, GWT, VSA
//! 4. **ScalingInter-RL**: Progressive difficulty with constellation levels
//! 5. **Self-play**: Both players share the same policy (SPIRAL pattern)

pub mod ai;
pub mod builtin;
pub mod consciousness;
pub mod ecs;
pub mod env;
pub mod events;
pub mod evolution;
pub mod framework;
pub mod hex_crucible;
pub mod mcp;
pub mod persistence;
pub mod play;
pub mod rpg;
pub mod world;

#[cfg(test)]
pub mod tests;

// Re-exports
pub use ai::{Action as AiAction, EpsilonGreedyPolicy, GreedyPolicy, Policy, RandomPolicy};
pub use ai::{BehaviorNode, BehaviorTree, Inverter, NodeStatus, Selector, Sequence};
pub use consciousness::{
    generate_attention_report, generate_feedback, generate_health_report, generate_phi_report,
    generate_vsa_report, AppraisalSignal, AttentionMapping, AttentionReport, EmotionLabel,
    FeedbackReport, GameVsaEncoder, HealthReport, PhiReport, PressureSignal, Recommendation,
    StrategySignature, VsaReport,
};
pub use env::{
    CognitiveSkill, Difficulty, GameMeta, GameRegistry, GameState, NtGameEnv, RenderMode,
};
pub use evolution::{
    GameEvolutionConfig, GameEvolutionLoop, GameEvolutionState,
    GameTickReport as EvolutionTickReport,
};
pub use framework::{
    Action, Actor, Arena, ArenaConfig, ArenaStats, Episode, EpisodeResult, Observation, Role,
    Rubric, StepResult, Trajectory,
};
pub use hex_crucible::{HexCrucible, HexCrucibleConfig};
pub use mcp::{GameSession, GameSessionManager, GameTool, GameToolRegistry, McpTrainingMetrics};
pub use play::{
    AdvantageConfig, BufferStats, GameAdvantageEstimator, GameGrpoAdapter, GameGrpoReport,
    GameTrajectoryBuffer, ScalingConfig, ScalingScheduler,
};
pub use rpg::cultivation::{
    BreakthroughResult, CultivationRealm, CultivationState, CultivationTechnique, SpiritualRoot,
    TechniqueSlots, TechniqueType,
};
pub use rpg::{
    Character, CharacterStats, EquipmentItem, EquipmentLoadout, EquipmentSlot, LevelSystem,
    SkillNode, SkillNodeType, SkillTree, Stat,
};

// ═══════════════════════════════════════════════════════════════════
// Pre-built game registry
// ═══════════════════════════════════════════════════════════════════

/// Create a default game registry with all built-in games.
pub fn default_registry() -> GameRegistry {
    let mut reg = GameRegistry::new();
    hex_crucible::register_hex_crucible(&mut reg);
    builtin::hex_tictactoe::register_hex_tictactoe(&mut reg);
    builtin::game_2048::register_game_2048(&mut reg);
    reg
}
