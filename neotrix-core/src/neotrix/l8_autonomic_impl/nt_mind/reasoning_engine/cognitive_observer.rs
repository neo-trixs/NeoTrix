/// Stub for missing cognitive_observer module
/// Provides types re-exported by reasoning_engine/mod.rs

#[derive(Debug, Clone)]
pub enum BlindSpotKind {
    RecurringError,
    CommunicationOptimization,
    ProblemDecomposition,
    VerificationImprovement,
    ToolUsagePattern,
    StrategyDiscovery,
    PrincipleUpdate,
}

#[derive(Debug, Clone)]
pub struct CognitiveBlindSpot {
    pub kind: BlindSpotKind,
    pub description: String,
    pub severity: f64,
}

#[derive(Debug, Clone)]
pub struct CognitiveEye {
    pub blind_spots: Vec<CognitiveBlindSpot>,
}

impl CognitiveEye {
    pub fn new() -> Self {
        Self { blind_spots: Vec::new() }
    }
}

impl Default for CognitiveEye {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CognitiveSnapshot {
    pub timestamp: u64,
    pub blind_spot_count: usize,
    pub awareness_level: f64,
}
