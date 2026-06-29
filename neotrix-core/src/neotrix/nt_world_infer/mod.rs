//! Active Inference Engine — re-exported from core/nt_core_infer for backward compatibility
//! Plus the memory_palace submodule (local to neotrix layer)

// memory_palace lives in core; re-exported here for backward compat
pub use crate::core::nt_core_consciousness::memory_palace::{
    MemoryPalace, MemoryRoom, PalaceEntry, PalaceSnapshot,
};

pub use crate::core::nt_core_infer::{
    ActiveInferenceEngine, FreeEnergyReport, DEFAULT_GRADIENT_WEIGHT, DEFAULT_SENSORY_PRECISION,
    DEFAULT_TEMPERATURE, FE_CONVERGENCE_THRESHOLD, FE_WINDOW_SIZE,
};
