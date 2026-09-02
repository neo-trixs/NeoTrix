//! NT-GAME — Self-Play Training Loop
//!
//! Training infrastructure for game-based self-evolution:
//!
//! - **adaptive**: Adaptive difficulty adjuster — tracks win rate and scales difficulty
//! - **advantage**: RAE (Role-conditioned Advantage Estimation) for per-turn credit assignment
//! - **buffer**: Trajectory replay buffer with ring storage and batch sampling
//! - **grpo_adapter**: Bridge from game trajectories to GRPO policy updates
//! - **scaling**: ScalingInter-RL progressive horizon scheduler
//!
//! ## Pipeline
//!
//! ```text
//! Episode → Trajectory → Buffer → Advantage Estimation → GRPO Update
//!                                      ↑
//!                               ScalingScheduler (horizon control)
//! ```

pub mod adaptive;
pub mod advantage;
pub mod buffer;
pub mod grpo_adapter;
pub mod scaling;
pub mod self_play_loop;

pub use adaptive::{AdaptiveDifficultyConfig, DifficultyAdjuster, DifficultyAdjustment};
pub use advantage::{AdvantageConfig, GameAdvantageEstimator};
pub use buffer::{BufferStats, GameTrajectoryBuffer};
pub use grpo_adapter::{GameGrpoAdapter, GameGrpoReport, GrpoBatch};
pub use scaling::{ScalingConfig, ScalingScheduler};
pub use self_play_loop::{SelfPlayConfig, SelfPlayLoop, TrainingMetrics, TrainingReport};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_re_exports() {
        // Verify all public types are accessible from the play module
        let _ = std::any::type_name::<DifficultyAdjuster>();
        let _ = std::any::type_name::<GameAdvantageEstimator>();
        let _ = std::any::type_name::<GameTrajectoryBuffer>();
        let _ = std::any::type_name::<GameGrpoAdapter>();
        let _ = std::any::type_name::<ScalingScheduler>();
    }
}
