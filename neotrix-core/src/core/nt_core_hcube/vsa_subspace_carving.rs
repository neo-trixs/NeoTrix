use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::ops::Range;

/// 16 orthogonal subspaces within the 4096-bit VSA space.
/// Each subspace = 256 contiguous bits (4096 / 16).
pub struct VsaSubspaceLayout;

impl VsaSubspaceLayout {
    pub const SUBSPACE_BITS: usize = 256;
    pub const NUM_SUBSPACES: usize = 16;

    pub const SELF: Range<usize> = 0..256;
    pub const WORLD: Range<usize> = 256..512;
    pub const SPATIAL: Range<usize> = 512..768;
    pub const EPISODIC: Range<usize> = 768..1024;
    pub const GOAL: Range<usize> = 1024..1280;
    pub const PHYSICS: Range<usize> = 1280..1536;
    pub const EMOTIONAL: Range<usize> = 1536..1792;
    pub const TRANSLATE: Range<usize> = 1792..2048;
    pub const REASONING: Range<usize> = 2048..2304;
    pub const MEMORY: Range<usize> = 2304..2560;
    pub const PERCEPTION: Range<usize> = 2560..2816;
    pub const ATTENTION: Range<usize> = 2816..3072;
    pub const UNCERTAINTY: Range<usize> = 3072..3328;
    pub const VALUE: Range<usize> = 3328..3584;
    pub const META: Range<usize> = 3584..3840;
    pub const RESERVED: Range<usize> = 3840..4096;

    /// All 16 subspaces in order, for iteration.
    pub const ALL: [Range<usize>; 16] = [
        Self::SELF,
        Self::WORLD,
        Self::SPATIAL,
        Self::EPISODIC,
        Self::GOAL,
        Self::PHYSICS,
        Self::EMOTIONAL,
        Self::TRANSLATE,
        Self::REASONING,
        Self::MEMORY,
        Self::PERCEPTION,
        Self::ATTENTION,
        Self::UNCERTAINTY,
        Self::VALUE,
        Self::META,
        Self::RESERVED,
    ];

    pub fn name(subspace: &Range<usize>) -> &'static str {
        match subspace.start {
            0 => "self",
            256 => "world",
            512 => "spatial",
            768 => "episodic",
            1024 => "goal",
            1280 => "physics",
            1536 => "emotional",
            1792 => "translate",
            2048 => "reasoning",
            2304 => "memory",
            2560 => "perception",
            2816 => "attention",
            3072 => "uncertainty",
            3328 => "value",
            3584 => "meta",
            3840 => "reserved",
            _ => "unknown",
        }
    }

    pub fn from_name(name: &str) -> Option<Range<usize>> {
        match name {
            "self" => Some(Self::SELF),
            "world" => Some(Self::WORLD),
            "spatial" => Some(Self::SPATIAL),
            "episodic" => Some(Self::EPISODIC),
            "goal" => Some(Self::GOAL),
            "physics" => Some(Self::PHYSICS),
            "emotional" => Some(Self::EMOTIONAL),
            "translate" => Some(Self::TRANSLATE),
            "reasoning" => Some(Self::REASONING),
            "memory" => Some(Self::MEMORY),
            "perception" => Some(Self::PERCEPTION),
            "attention" => Some(Self::ATTENTION),
            "uncertainty" => Some(Self::UNCERTAINTY),
            "value" => Some(Self::VALUE),
            "meta" => Some(Self::META),
            "reserved" => Some(Self::RESERVED),
            _ => None,
        }
    }
}

/// Encode text into a specific subspace only. All other bits = 0.
pub fn encode_to_subspace(text: &str, subspace: Range<usize>, seed: u64) -> Vec<u8> {
    let content_seed = seed.wrapping_add(
        text.bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64)),
    );
    let mut rng = StdRng::seed_from_u64(content_seed);
    let mut result = vec![0u8; VSA_DIM];
    for i in subspace {
        if i < VSA_DIM {
            result[i] = rng.gen::<u8>() & 1;
        }
    }
    result
}

/// Encode text into all subspaces (legacy behavior, backward compat).
pub fn encode_full(text: &str, seed: u64) -> Vec<u8> {
    let content_seed = seed.wrapping_add(
        text.bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64)),
    );
    let mut rng = StdRng::seed_from_u64(content_seed);
    (0..VSA_DIM).map(|_| rng.gen::<u8>() & 1).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

    #[test]
    fn test_subspaces_are_orthogonal() {
        let all = VsaSubspaceLayout::ALL;
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                let overlap_start = all[i].start.max(all[j].start);
                let overlap_end = all[i].end.min(all[j].end);
                assert!(
                    overlap_start >= overlap_end,
                    "subspaces {} ({:?}) and {} ({:?}) overlap",
                    VsaSubspaceLayout::name(&all[i]),
                    all[i],
                    VsaSubspaceLayout::name(&all[j]),
                    all[j]
                );
            }
        }
    }

    #[test]
    fn test_subspaces_cover_full_range() {
        let all = VsaSubspaceLayout::ALL;
        assert_eq!(all[0].start, 0);
        assert_eq!(all[all.len() - 1].end, VSA_DIM);
        for i in 0..(all.len() - 1) {
            assert_eq!(
                all[i].end,
                all[i + 1].start,
                "gap between subspaces {} and {}",
                VsaSubspaceLayout::name(&all[i]),
                VsaSubspaceLayout::name(&all[i + 1])
            );
        }
    }

    #[test]
    fn test_each_subspace_is_256_bits() {
        for subspace in VsaSubspaceLayout::ALL {
            assert_eq!(subspace.len(), VsaSubspaceLayout::SUBSPACE_BITS);
        }
    }

    #[test]
    fn test_encode_to_subspace_only_target_has_bits() {
        let text = "hello world";
        let subspace = VsaSubspaceLayout::EMOTIONAL;
        let encoded = encode_to_subspace(text, subspace.clone(), 42);

        assert_eq!(encoded.len(), VSA_DIM);

        // Bits outside the subspace must be zero
        for i in 0..VSA_DIM {
            if subspace.contains(&i) {
                continue;
            }
            assert_eq!(encoded[i], 0, "bit {} outside subspace must be zero", i);
        }

        // At least some bits in the subspace should be non-zero
        let active_count: usize = subspace.clone().filter(|&i| encoded[i] != 0).count();
        assert!(active_count > 0, "subspace should have some non-zero bits");
    }

    #[test]
    fn test_encode_to_subspace_deterministic() {
        let text = "deterministic test";
        let subspace = VsaSubspaceLayout::GOAL;
        let a = encode_to_subspace(text, subspace.clone(), 99);
        let b = encode_to_subspace(text, subspace.clone(), 99);
        assert_eq!(
            a, b,
            "encoding must be deterministic for same seed and text"
        );
    }

    #[test]
    fn test_subspace_similarity_identical() {
        let v = QuantizedVSA::random_binary();
        let sim = QuantizedVSA::subspace_similarity(&v, &v, VsaSubspaceLayout::SELF);
        assert!(
            (sim - 1.0).abs() < 1e-10,
            "identical self-subspace similarity should be 1.0"
        );
    }

    #[test]
    fn test_subspace_similarity_orthogonal() {
        let zeros = vec![0u8; VSA_DIM];
        let ones = vec![1u8; VSA_DIM];
        let sim = QuantizedVSA::subspace_similarity(&zeros, &ones, VsaSubspaceLayout::WORLD);
        assert!(
            (sim - 0.0).abs() < 1e-10,
            "all-zeros vs all-ones in subspace should give 0.0 similarity"
        );
    }

    #[test]
    fn test_subspace_similarity_half_subspace() {
        let mut a = vec![0u8; VSA_DIM];
        let mut b = vec![0u8; VSA_DIM];
        let subspace = VsaSubspaceLayout::SPATIAL;
        let half = subspace.start + subspace.len() / 2;
        for i in subspace.start..half {
            a[i] = 1;
            b[i] = 1;
        }
        for i in half..subspace.end {
            a[i] = 1;
            b[i] = 0;
        }
        let sim = QuantizedVSA::subspace_similarity(&a, &b, subspace.clone());
        let expected = 0.5;
        assert!(
            (sim - expected).abs() < 0.01,
            "half-subspace match should give ~0.5, got {}",
            sim
        );
    }

    #[test]
    fn test_subspace_similarity_other_subspace_unaffected() {
        let subspace_a = VsaSubspaceLayout::REASONING;
        let subspace_b = VsaSubspaceLayout::MEMORY;

        let v1 = encode_to_subspace("hello", subspace_a.clone(), 1);
        let v2 = encode_to_subspace("hello", subspace_a.clone(), 1);

        // similarity in subspace_b should be 1.0 (both are zero there)
        let sim_b = QuantizedVSA::subspace_similarity(&v1, &v2, subspace_b.clone());
        assert!(
            (sim_b - 1.0).abs() < 1e-10,
            "unchanged (zero) subspaces should have perfect similarity"
        );

        // similarity in subspace_a should be 1.0 (identical content)
        let sim_a = QuantizedVSA::subspace_similarity(&v1, &v2, subspace_a);
        assert!(
            (sim_a - 1.0).abs() < 1e-10,
            "identical subspace should have perfect similarity"
        );
    }

    #[test]
    fn test_subspace_similarity_different_subspaces_independent() {
        let subspace_a = VsaSubspaceLayout::VALUE;
        let subspace_b = VsaSubspaceLayout::META;

        let v1 = encode_to_subspace("alpha", subspace_a.clone(), 10);
        let mut v2 = v1.clone();
        // Flip all bits in subspace_b
        for i in subspace_b.clone() {
            v2[i] ^= 1;
        }

        // similarity in subspace_a should remain 1.0 (unchanged)
        let sim_a = QuantizedVSA::subspace_similarity(&v1, &v2, subspace_a);
        assert!(
            (sim_a - 1.0).abs() < 1e-10,
            "changes in subspace_b must not affect subspace_a similarity"
        );

        // similarity in subspace_b should be 0.0 (all flipped)
        let sim_b = QuantizedVSA::subspace_similarity(&v1, &v2, subspace_b);
        assert!(
            (sim_b - 0.0).abs() < 1e-10,
            "all bits flipped in subspace_b should give 0.0 similarity"
        );
    }

    #[test]
    fn test_encode_full_backward_compat() {
        let text = "backward compat test";
        let encoded = encode_full(text, 42);
        assert_eq!(encoded.len(), VSA_DIM);
        let has_both = encoded.iter().any(|&x| x == 0) && encoded.iter().any(|&x| x == 1);
        assert!(has_both, "full encoding should contain both zeros and ones");
    }

    #[test]
    fn test_encode_full_deterministic() {
        let text = "deterministic backward compat";
        let a = encode_full(text, 42);
        let b = encode_full(text, 42);
        assert_eq!(a, b, "encode_full must be deterministic");
    }

    #[test]
    fn test_subspace_name_roundtrip() {
        let all = VsaSubspaceLayout::ALL;
        for subspace in all {
            let name = VsaSubspaceLayout::name(&subspace);
            let recovered = VsaSubspaceLayout::from_name(name);
            assert!(
                recovered.is_some(),
                "name '{}' should map to a subspace",
                name
            );
            assert_eq!(
                recovered.unwrap(),
                subspace,
                "name roundtrip failed for '{}'",
                name
            );
        }
    }

    #[test]
    fn test_subspace_name_unknown() {
        assert_eq!(VsaSubspaceLayout::name(&(9999..10000)), "unknown");
        assert!(VsaSubspaceLayout::from_name("nonexistent").is_none());
    }

    #[test]
    fn test_subspace_is_active_active() {
        let mut v = vec![0u8; VSA_DIM];
        // Set a single bit in the ATTENTION subspace
        v[VsaSubspaceLayout::ATTENTION.start] = 1;
        assert!(QuantizedVSA::subspace_is_active(
            &v,
            VsaSubspaceLayout::ATTENTION
        ));
    }

    #[test]
    fn test_subspace_is_active_inactive() {
        let mut v = vec![0u8; VSA_DIM];
        // Only set bits in EMOTIONAL subspace
        v[VsaSubspaceLayout::EMOTIONAL.start] = 1;
        assert!(!QuantizedVSA::subspace_is_active(
            &v,
            VsaSubspaceLayout::ATTENTION
        ));
    }

    #[test]
    fn test_subspace_is_active_all_zero() {
        let v = vec![0u8; VSA_DIM];
        for subspace in VsaSubspaceLayout::ALL {
            assert!(!QuantizedVSA::subspace_is_active(&v, subspace));
        }
    }

    #[test]
    fn test_extract_subspace_correct_bits() {
        let mut v = vec![0u8; VSA_DIM];
        // Fill the TRANSLATE subspace with alternating pattern
        let subspace = VsaSubspaceLayout::TRANSLATE;
        for (i, idx) in subspace.clone().enumerate() {
            v[idx] = (i % 2) as u8;
        }

        let extracted = QuantizedVSA::extract_subspace(&v, subspace.clone());

        assert_eq!(extracted.len(), VSA_DIM);

        // Bits within subspace should be preserved
        for (i, idx) in subspace.clone().enumerate() {
            assert_eq!(
                extracted[idx],
                (i % 2) as u8,
                "bit {} in subspace should be preserved",
                idx
            );
        }

        // Bits outside subspace should be zero
        for i in 0..VSA_DIM {
            if !subspace.contains(&i) {
                assert_eq!(
                    extracted[i], 0,
                    "bit {} outside subspace should be zeroed",
                    i
                );
            }
        }
    }

    #[test]
    fn test_extract_subspace_empty() {
        let v = QuantizedVSA::random_binary();
        let extracted = QuantizedVSA::extract_subspace(&v, 0..0);
        assert_eq!(extracted.len(), VSA_DIM);
        assert!(extracted.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_encode_to_subspace_different_texts_different() {
        let subspace = VsaSubspaceLayout::PERCEPTION;
        let a = encode_to_subspace("cat", subspace.clone(), 42);
        let b = encode_to_subspace("dog", subspace.clone(), 42);
        assert_ne!(a, b, "different texts should produce different encodings");
    }

    #[test]
    fn test_encode_full_uses_all_4096_bits() {
        let v = encode_full("coverage test", 1);
        assert_eq!(v.len(), VSA_DIM);
        // At minimum, ensure we get some 1s (not all zeros)
        let ones: usize = v.iter().map(|&x| x as usize).sum();
        assert!(ones > 0, "full encoding should contain some 1 bits");
        assert!(ones < VSA_DIM, "full encoding should contain some 0 bits");
    }
}
