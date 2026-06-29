pub mod bridge;
pub mod types;

#[cfg(test)]
pub mod tests;

pub use bridge::FEPIITBridge;
pub use types::{BridgeReport, FepIitHypervector, VSAUnifiedState};

pub(crate) const VSA_DIM: usize = 4096;
pub(crate) const DEFAULT_ALPHA: f64 = 0.4;
pub(crate) const DEFAULT_BETA: f64 = 0.4;
pub(crate) const DEFAULT_GAMMA: f64 = 0.2;
pub(crate) const FE_NORMALIZE_MAX: f64 = 10.0;
