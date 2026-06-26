use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HyperCubeError {
    DimensionMismatch { expected: usize, got: usize },
    EntryNotFound(String),
    InvalidOperation(String),
    Internal(String),
}

impl fmt::Display for HyperCubeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HyperCubeError::DimensionMismatch { expected, got } => {
                write!(f, "Dimension mismatch: expected {}, got {}", expected, got)
            }
            HyperCubeError::EntryNotFound(s) => write!(f, "HyperCube entry not found: {}", s),
            HyperCubeError::InvalidOperation(s) => write!(f, "Invalid HyperCube operation: {}", s),
            HyperCubeError::Internal(s) => write!(f, "Internal HyperCube error: {}", s),
        }
    }
}

impl std::error::Error for HyperCubeError {}

impl From<String> for HyperCubeError {
    fn from(s: String) -> Self {
        HyperCubeError::Internal(s)
    }
}
impl From<&str> for HyperCubeError {
    fn from(s: &str) -> Self {
        HyperCubeError::Internal(s.to_string())
    }
}

impl From<HyperCubeError> for crate::core::CoreError {
    fn from(e: HyperCubeError) -> Self {
        match e {
            HyperCubeError::DimensionMismatch { expected, got } => {
                crate::core::CoreError::HcubeDimMismatch { expected, got }
            }
            HyperCubeError::EntryNotFound(s) => {
                crate::core::CoreError::HyperCube(format!("entry not found: {}", s))
            }
            HyperCubeError::InvalidOperation(s) => {
                crate::core::CoreError::HyperCube(format!("invalid operation: {}", s))
            }
            HyperCubeError::Internal(s) => crate::core::CoreError::HyperCube(s),
        }
    }
}
