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

pub mod framework;
pub mod env;
pub mod hex_crucible;
pub mod builtin;
pub mod play;
pub mod consciousness;
pub mod mcp;
pub mod evolution;

// Re-exports
pub use framework::{Actor, Episode, Rubric, Arena, Trajectory, Action, Observation, StepResult, EpisodeResult, ArenaConfig, ArenaStats, Role};
pub use env::{NtGameEnv, GameState, GameRegistry, GameMeta, RenderMode, Difficulty, CognitiveSkill};
pub use hex_crucible::{HexCrucible, HexCrucibleConfig};
pub use play::{
    GameAdvantageEstimator, AdvantageConfig,
    GameTrajectoryBuffer, BufferStats,
    GameGrpoAdapter, GameGrpoReport,
    ScalingScheduler, ScalingConfig,
};
pub use consciousness::{
    FeedbackReport, AppraisalSignal, PressureSignal, EmotionLabel, generate_feedback,
    AttentionReport, AttentionMapping, generate_attention_report,
    PhiReport, generate_phi_report,
    VsaReport, GameVsaEncoder, StrategySignature, generate_vsa_report,
    HealthReport, Recommendation, generate_health_report,
};
pub use mcp::{GameToolRegistry, GameSession, GameSessionManager, GameTool, McpTrainingMetrics};
pub use evolution::{GameEvolutionLoop, GameEvolutionConfig, GameEvolutionState, GameTickReport as EvolutionTickReport};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry() {
        let reg = default_registry();
        assert_eq!(reg.list().len(), 3);
        assert!(reg.find("HexCrucible").is_some());
        assert!(reg.find("HexTicTacToe").is_some());
        assert!(reg.find("2048").is_some());
    }

    #[test]
    fn test_constellation_filter() {
        let reg = default_registry();
        let c0 = reg.for_constellation(0);
        assert!(c0.iter().any(|g| g.name == "HexTicTacToe"));
        assert!(c0.iter().any(|g| g.name == "HexCrucible"));
    }
}
