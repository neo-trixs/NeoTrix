#![forbid(unsafe_code)]

pub mod rsi_core;

pub use rsi_core::{
    FailureRecord, ImprovementProposal, ImprovementResult, ImprovementType, ModuleMetrics,
    RsiController, RsiCore, RsiSafetyStatus, RsiStatusReport, SystemPerformanceData,
};
