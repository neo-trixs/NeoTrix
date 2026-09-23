//! L2 → L0 error conversions.
//!
//! `From<L2Error> for NeoTrixError` impls live here (L2) rather than in
//! `l0_substrate::nt_core_error` to respect the L0 ← L2 dependency direction.

use crate::l0_substrate::nt_core_error::NeoTrixError;

impl From<crate::l2_perception::nt_world::asset_map::query::ParseError> for NeoTrixError {
    fn from(e: crate::l2_perception::nt_world::asset_map::query::ParseError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l2_perception::nt_world::social_access::traits::SocialAccessError> for NeoTrixError {
    fn from(e: crate::l2_perception::nt_world::social_access::traits::SocialAccessError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l2_perception::nt_world::source::offline_download::OfflineError> for NeoTrixError {
    fn from(e: crate::l2_perception::nt_world::source::offline_download::OfflineError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}
