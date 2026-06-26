use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ConsciousnessError {
    ModuleUnavailable(String),
    Stagnation(String),
    ModuleNotFound(String),
    Internal(String),
}

impl fmt::Display for ConsciousnessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConsciousnessError::ModuleUnavailable(s) => write!(f, "Module unavailable: {}", s),
            ConsciousnessError::Stagnation(s) => write!(f, "Stagnation: {}", s),
            ConsciousnessError::ModuleNotFound(s) => write!(f, "Module not found: {}", s),
            ConsciousnessError::Internal(s) => write!(f, "Internal consciousness error: {}", s),
        }
    }
}

impl std::error::Error for ConsciousnessError {}

impl From<String> for ConsciousnessError {
    fn from(s: String) -> Self {
        ConsciousnessError::Internal(s)
    }
}
impl From<&str> for ConsciousnessError {
    fn from(s: &str) -> Self {
        ConsciousnessError::Internal(s.to_string())
    }
}

impl From<ConsciousnessError> for crate::core::CoreError {
    fn from(e: ConsciousnessError) -> Self {
        match e {
            ConsciousnessError::ModuleUnavailable(s) => {
                crate::core::CoreError::Consciousness(format!("module unavailable: {}", s))
            }
            ConsciousnessError::Stagnation(s) => {
                crate::core::CoreError::Consciousness(format!("stagnation: {}", s))
            }
            ConsciousnessError::ModuleNotFound(s) => {
                crate::core::CoreError::NotFound(format!("consciousness module: {}", s))
            }
            ConsciousnessError::Internal(s) => crate::core::CoreError::Consciousness(s),
        }
    }
}
