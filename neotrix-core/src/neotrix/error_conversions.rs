//! neotrix → L0 error conversions.
//!
//! `From<NeotrixError> for NeoTrixError` impls live here (neotrix layer) rather
//! than in `l0_substrate::nt_core_error` to respect the L0 ← neotrix direction.
//! This also hosts the `nt_core_capability_tree::registry::RegistryError`
//! conversion, as that crate's source lives under `src/neotrix/`.

use crate::l0_substrate::nt_core_error::NeoTrixError;

impl From<crate::neotrix::nt_file_ability::capability::CapabilityError> for NeoTrixError {
    fn from(e: crate::neotrix::nt_file_ability::capability::CapabilityError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::neotrix::nt_file_ability::types::FileAbilityError> for NeoTrixError {
    fn from(e: crate::neotrix::nt_file_ability::types::FileAbilityError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::neotrix::nt_file_ability::image_super_resolution::SuperResolutionError> for NeoTrixError {
    fn from(e: crate::neotrix::nt_file_ability::image_super_resolution::SuperResolutionError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<nt_core_capability_tree::registry::RegistryError> for NeoTrixError {
    fn from(e: nt_core_capability_tree::registry::RegistryError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}
