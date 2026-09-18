// L-02: Kronecker-structured O(N log N) cleanup
// This is a thin re-export from the core layer.
// The actual implementation lives in core::kronecker_cleanup to respect
// the core → neotrix layering rule.
pub use crate::l5_cognition::nt_core_kron::*;
