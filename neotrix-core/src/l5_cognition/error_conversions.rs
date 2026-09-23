//! L5 → L0 error conversions.
//!
//! `From<L5Error> for NeoTrixError` impls live here (L5) rather than in
//! `l0_substrate::nt_core_error` to respect the L0 ← L5 dependency direction.

use crate::l0_substrate::nt_core_error::NeoTrixError;

impl From<crate::l5_cognition::nt_core::capability::nt_act_orch_patterns::AgentError> for NeoTrixError {
    fn from(e: crate::l5_cognition::nt_core::capability::nt_act_orch_patterns::AgentError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l5_cognition::nt_mind::nt_mind::consciousness::element::ElementError> for NeoTrixError {
    fn from(e: crate::l5_cognition::nt_mind::nt_mind::consciousness::element::ElementError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l5_cognition::nt_mind::nt_game::world::combat::equipment::EquipError> for NeoTrixError {
    fn from(e: crate::l5_cognition::nt_mind::nt_game::world::combat::equipment::EquipError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l5_cognition::nt_mind::nt_game::world::combat::equipment::CraftError> for NeoTrixError {
    fn from(e: crate::l5_cognition::nt_mind::nt_game::world::combat::equipment::CraftError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l5_cognition::nt_core_panic_recovery::BoundaryError> for NeoTrixError {
    fn from(e: crate::l5_cognition::nt_core_panic_recovery::BoundaryError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}
