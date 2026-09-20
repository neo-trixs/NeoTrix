pub mod module_def;
pub mod workspace;
pub mod resonance;
pub mod recurrent;
pub mod unified_consciousness;

/// Provider Benchmark — LLM Challenge results shared across layers (L1/L5).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProviderBenchmark {
    pub provider: String,
    pub model: String,
    pub accuracy: f64,
    pub latency_ms: u64,
    pub cost_usd: f64,
    pub task_type: String,
    pub timestamp: u64,
}

impl ProviderBenchmark {
    pub fn new(provider: String, model: String, task_type: String) -> Self {
        Self {
            provider,
            model,
            task_type,
            ..Default::default()
        }
    }
}

// Unified consciousness types
pub use unified_consciousness::{
    ConsciousnessPhase,
    ConsciousnessObserverState, ConsciousnessState as UnifiedConsciousnessState,
    EmotionalState, VadEmotionalState,
    BranchState, Alert, ConsciousnessSnapshot,
    CapabilityConsciousnessState, Layer,
    LegacyConsciousnessPhase,
    CrystalConsciousnessState, EvolutionPhase,
    LoopConsciousnessState,
};
