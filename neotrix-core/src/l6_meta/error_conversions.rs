//! L6 → L0 error conversions.
//!
//! `From<L6Error> for NeoTrixError` impls live here (L6) rather than in
//! `l0_substrate::nt_core_error` to respect the L0 ← L6 dependency direction.
//!
//! NOTE: this module is not yet registered in `l6_meta::mod` because
//! that file has concurrent edits from another lane — the owning lane should
//! add `pub mod error_conversions;` to `l6_meta/mod.rs` (one line).

use crate::l0_substrate::nt_core_error::NeoTrixError;

impl From<crate::l6_meta::healing::nt_mind_eval_harness::EvalError> for NeoTrixError {
    fn from(e: crate::l6_meta::healing::nt_mind_eval_harness::EvalError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l6_meta::nt_core_observer_error::ErrorRecoveryError> for NeoTrixError {
    fn from(e: crate::l6_meta::nt_core_observer_error::ErrorRecoveryError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l6_meta::nt_core_capability::CapabilityError> for NeoTrixError {
    fn from(e: crate::l6_meta::nt_core_capability::CapabilityError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}
