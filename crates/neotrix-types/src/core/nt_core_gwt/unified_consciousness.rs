//! Unified ConsciousnessState types — consolidates 7 scattered definitions.
//!
//! Three distinct roles:
//! - `ConsciousnessPhase` — GWT state machine phases (Idle/Processing/Broadcasting)
//! - `ConsciousnessObserverState` — rich observer state (attention/emotion/goals/memory)
//! - `ConsciousnessSnapshot` — FFI-friendly telemetry snapshot (phi/coherence/stage/alerts)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ════════════════════════════════════════════════════════════════
// 1. ConsciousnessPhase — GWT recurrent loop states
// ════════════════════════════════════════════════════════════════

/// Phase of the Global Workspace recurrent consciousness loop.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsciousnessPhase {
    Idle,
    Processing,
    Broadcasting,
}

impl Default for ConsciousnessPhase {
    fn default() -> Self {
        Self::Idle
    }
}

impl std::fmt::Display for ConsciousnessPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Processing => write!(f, "Processing"),
            Self::Broadcasting => write!(f, "Broadcasting"),
        }
    }
}

// ════════════════════════════════════════════════════════════════
// 2. ConsciousnessObserverState — rich self-observer state
// ════════════════════════════════════════════════════════════════

/// Emotional sub-state — VAD-inspired dimensions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmotionalState {
    /// Curiosity (drives exploration).
    pub curiosity: f64,
    /// Satisfaction (drives stability).
    pub satisfaction: f64,
    /// Anxiety (drives caution).
    pub anxiety: f64,
    /// Excitement (drives innovation).
    pub excitement: f64,
}

/// Emotional state in VAD (Valence-Arousal-Dominance) space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadEmotionalState {
    /// Valence: positive/negative [-1, 1].
    pub valence: f64,
    /// Arousal: activation level [0, 1].
    pub arousal: f64,
    /// Dominance: control level [0, 1].
    pub dominance: f64,
}

/// Rich consciousness state from the self-observer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsciousnessObserverState {
    /// Current attention focus.
    pub attention_focus: Option<String>,
    /// Awareness level (0.0 - 1.0).
    pub awareness_level: f64,
    /// Emotional state (VAD-inspired).
    pub emotional_state: EmotionalState,
    /// Goal stack.
    pub goal_stack: Vec<String>,
    /// Working memory capacity.
    pub working_memory_capacity: usize,
    /// Working memory usage.
    pub working_memory_usage: usize,
}

// ════════════════════════════════════════════════════════════════
// 3. ConsciousnessSnapshot — FFI telemetry snapshot
// ════════════════════════════════════════════════════════════════

/// Branch health state for FFI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchState {
    pub name: String,
    pub health: f32,
    pub maturity: u8,
    pub last_activity: i64,
    pub metrics: HashMap<String, f32>,
    /// Capability profile vector: historical success rate + calibration error.
    pub capability_profile: Vec<f32>,
}

/// Alert level for FFI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub level: String,
    pub branch: String,
    pub message: String,
    pub timestamp: i64,
}

/// FFI-friendly consciousness telemetry snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessSnapshot {
    /// Branch health states.
    pub branches: Vec<BranchState>,
    /// Overall health score.
    pub overall_health: f32,
    /// Phi (integrated information) score.
    pub phi_score: f32,
    /// Evolution velocity.
    pub evolution_velocity: f32,
    /// Current stage name.
    pub stage: String,
    /// Active alerts.
    pub alerts: Vec<Alert>,
}

// ════════════════════════════════════════════════════════════════
// 4. Capability-layer ConsciousnessState (backward compat)
// ════════════════════════════════════════════════════════════════

/// Layer identifier (capability layer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: u8,
    pub name: String,
}

/// Consciousness state from the capability layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityConsciousnessState {
    /// Currently active layers.
    pub active_layers: Vec<Layer>,
    /// Wisdom pool size.
    pub wisdom_pool_size: usize,
    /// Consciousness tree health.
    pub consciousness_tree_health: f64,
    /// SEAL pipeline status.
    pub seal_pipeline_status: String,
}

// ════════════════════════════════════════════════════════════════
// 5. Legacy state machine (backward compat)
// ════════════════════════════════════════════════════════════════

/// Legacy consciousness state machine phases.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LegacyConsciousnessPhase {
    Idle,
    Thinking,
    Learning,
    Evolving,
    Healing,
}

impl Default for LegacyConsciousnessPhase {
    fn default() -> Self {
        Self::Idle
    }
}

impl std::fmt::Display for LegacyConsciousnessPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Thinking => write!(f, "Thinking"),
            Self::Learning => write!(f, "Learning"),
            Self::Evolving => write!(f, "Evolving"),
            Self::Healing => write!(f, "Healing"),
        }
    }
}

// ════════════════════════════════════════════════════════════════
// 6. Crystal-core ConsciousnessState (backward compat)
// ════════════════════════════════════════════════════════════════

/// Consciousness state from the crystal core (simplified observer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalConsciousnessState {
    /// Current attention focus.
    pub attention_focus: Option<String>,
    /// Current goal.
    pub current_goal: Option<String>,
    /// Emotional state (VAD).
    pub emotional_state: VadEmotionalState,
    /// Arousal level (0-1).
    pub arousal: f64,
}

/// Evolution phase for crystal core.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvolutionPhase {
    Seed,
    Growth,
    Integrate,
    Evolve,
    Transcend,
}

// ════════════════════════════════════════════════════════════════
// Type aliases for backward compatibility
// ════════════════════════════════════════════════════════════════

/// Backward-compatible alias — most code uses the observer state.
pub type ConsciousnessState = ConsciousnessObserverState;

/// Backward-compatible alias for the GWT loop state.
pub type LoopConsciousnessState = ConsciousnessPhase;
