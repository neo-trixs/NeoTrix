// REVIVED Task 2 — dead_code removed
use crate::core::nt_core_hcube::vsa_vector::VsaVector;

/// Straight-Through Estimator (STE) for VSA operations.
///
/// Forward pass: discrete VSA operation (XOR bind, majority-sum bundle).
/// Backward pass: identity function (STE) — gradient passes through unchanged.
/// This enables VSA operations to participate in gradient-based optimization
/// (e.g., Ne language tensor_graph.rs, SEAL evolution).
///
/// The actual gradient computation happens in the computation graph
/// (nt-lang::tensor_graph::GraphNode::VsaBind, GraphNode::VsaBundle).
/// This module provides the STE contract + forward-only helpers.

/// Forward bind via XOR with STE contract.
/// Matches tensor_graph.rs VsaBind forward semantics.
pub fn ste_bind<const DIM: usize>(a: &VsaVector<DIM>, b: &VsaVector<DIM>) -> VsaVector<DIM> {
    let mut bytes: Vec<u8> = a.as_bytes().to_vec();
    for (x, y) in bytes.iter_mut().zip(b.as_bytes().iter()) {
        *x ^= y;
    }
    VsaVector::from_bytes(bytes).expect("STE bind: DIM mismatch")
}

/// Forward majority-sum bundle with STE contract.
/// Tie-breaks to 1 on even counts.
/// Matches tensor_graph.rs VsaBundle forward semantics.
pub fn ste_bundle<const DIM: usize>(items: &[&VsaVector<DIM>]) -> VsaVector<DIM> {
    if items.is_empty() {
        return VsaVector::new();
    }
    let n = items.len();
    let threshold = n as f64 / 2.0;
    let mut bytes = Vec::with_capacity(DIM);
    for i in 0..DIM {
        let sum: u64 = items.iter().map(|v| v.as_bytes()[i] as u64).sum();
        bytes.push(if (sum as f64) >= threshold { 1u8 } else { 0u8 });
    }
    VsaVector::from_bytes(bytes).expect("STE bundle: DIM mismatch")
}

/// Marker trait: types supporting STE for gradient-based optimization.
pub trait SteVsaOp<const DIM: usize>: Sized {
    /// Forward bind (XOR) — discrete operation with STE backward contract
    fn forward_bind(&self, other: &Self) -> Self;
    /// Forward bundle (majority sum + threshold) with STE backward contract
    fn forward_bundle(items: &[&Self]) -> Self;
}

impl<const DIM: usize> SteVsaOp<DIM> for VsaVector<DIM> {
    fn forward_bind(&self, other: &Self) -> Self {
        ste_bind(self, other)
    }
    fn forward_bundle(items: &[&Self]) -> Self {
        ste_bundle(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_vector::VsaVector;

    const TEST_DIM: usize = 128;

    #[test]
    fn test_ste_bind_deterministic() {
        let a = VsaVector::<TEST_DIM>::random(42);
        let b = VsaVector::<TEST_DIM>::random(7);
        let r1 = ste_bind::<TEST_DIM>(&a, &b);
        let r2 = ste_bind::<TEST_DIM>(&a, &b);
        assert_eq!(r1, r2, "STE bind must be deterministic");
    }

    #[test]
    fn test_ste_bind_xor_contract() {
        let a = VsaVector::<TEST_DIM>::random(42);
        let b = VsaVector::<TEST_DIM>::random(7);
        let bound = ste_bind::<TEST_DIM>(&a, &b);
        // Bind is its own inverse: bind(bind(a,b), b) == a
        let unbound = ste_bind::<TEST_DIM>(&bound, &b);
        assert_eq!(a, unbound, "STE bind must satisfy bind(bind(a,b),b) == a");
    }

    #[test]
    fn test_ste_bundle_majority() {
        let a = VsaVector::<TEST_DIM>::random(42);
        let b = VsaVector::<TEST_DIM>::random(7);
        let c = VsaVector::<TEST_DIM>::random(99);
        let bundle = ste_bundle::<TEST_DIM>(&[&a, &b, &c]);
        // Bundle of 3 should produce a valid result (not all zeros)
        assert!(
            bundle.as_bytes().iter().any(|&x| x != 0),
            "Bundle of 3 random vectors should have some 1s"
        );
    }

    #[test]
    fn test_ste_bundle_empty_returns_default() {
        let result = ste_bundle::<TEST_DIM>(&[]);
        assert!(
            result.as_bytes().iter().all(|&x| x == 0),
            "Empty bundle should return all zeros"
        );
    }

    #[test]
    fn test_ste_bundle_single_is_identity() {
        let a = VsaVector::<TEST_DIM>::random(42);
        let bundle = ste_bundle::<TEST_DIM>(&[&a]);
        assert_eq!(a, bundle, "Bundle of single item should be identity");
    }

    #[test]
    fn test_ste_trait_bind() {
        let a = VsaVector::<TEST_DIM>::random(42);
        let b = VsaVector::<TEST_DIM>::random(7);
        let r1 = a.forward_bind(&b);
        let r2 = ste_bind::<TEST_DIM>(&a, &b);
        assert_eq!(r1, r2, "Trait method must match free function");
    }
}
