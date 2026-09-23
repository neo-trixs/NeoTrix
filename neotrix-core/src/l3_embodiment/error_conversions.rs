//! L3 → L0 error conversions.
//!
//! `From<L3Error> for NeoTrixError` impls live here (L3) rather than in
//! `l0_substrate::nt_core_error` to respect the L0 ← L3 dependency direction.
//!
//! NOTE: this module is not yet registered in `l3_embodiment::mod` because
//! that file has concurrent edits from another lane — the owning lane should
//! add `pub mod error_conversions;` to `l3_embodiment/mod.rs` (one line).

use crate::l0_substrate::nt_core_error::NeoTrixError;

impl From<crate::l3_embodiment::nt_shield::binary_analyzer::BinaryError> for NeoTrixError {
    fn from(e: crate::l3_embodiment::nt_shield::binary_analyzer::BinaryError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l3_embodiment::nt_shield::proxy_detection::ProxyDetectionError> for NeoTrixError {
    fn from(e: crate::l3_embodiment::nt_shield::proxy_detection::ProxyDetectionError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l3_embodiment::nt_shield::nt_shield_ztnet::ZtnetError> for NeoTrixError {
    fn from(e: crate::l3_embodiment::nt_shield::nt_shield_ztnet::ZtnetError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l3_embodiment::nt_shield::nt_shield_ztnet::connectivity::stun::StunError> for NeoTrixError {
    fn from(e: crate::l3_embodiment::nt_shield::nt_shield_ztnet::connectivity::stun::StunError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l3_embodiment::nt_shield::nt_shield_ztnet::packet::ip_parser::ParseError> for NeoTrixError {
    fn from(e: crate::l3_embodiment::nt_shield::nt_shield_ztnet::packet::ip_parser::ParseError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}
