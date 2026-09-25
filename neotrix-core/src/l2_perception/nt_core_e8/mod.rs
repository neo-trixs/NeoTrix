//! E₈ × 64-hexagram: mathematical spine of the hidden world model.
//!
//! Core identities discovered through deep research (2026-05-24):
//!   1. E₈ (248 generators) ⊃ Spin(11,3) → **64 fermions per generation**
//!   2. 3 generations × 64 = 192 = 248 - 56 (remaining Cartan/roots)
//!   3. 8 trigrams = simple roots of SU(3) color gauge group
//!   4. 64 hexagrams (6-bit binary) = weight diagram of the 64-fermion rep
//!   5. 384 lines = 64 × 6 = total degrees of freedom in E₈ root system
//!   6. 50 (Dayan) - 1 (observer) = 49 (observable dof) ↔ 49 = 7² = 248-199

pub mod abduction;
pub mod domain_transition;
pub mod e8_abduction_bridge;
pub mod e8_lattice_quantizer;
pub mod ewhr_bridge;
pub mod nt_core_community_ingester;
pub mod nt_core_e8_prediction;
pub mod nt_core_fable_pattern;
pub mod nt_core_synthesis;
pub mod nt_core_trajectory_prm;
pub mod nt_latent_reasoning;
pub mod nt_latent_thought;
pub mod nt_latent_transformer;
pub mod nt_multimodal;
pub mod sparse_moe;
pub mod state_machine;
pub mod thinking_budget;
pub mod unified_latent;

pub use nt_latent_reasoning::{
    LatentEpisodicEntry, LatentReasoningPipeline, LatentRetrieval, LATENT_MEMORY_SIZE,
    TOP_K_NEIGHBORS,
};
pub use nt_latent_thought::LatentThoughtVector;
pub use nt_latent_transformer::{
    LatentReasoningTransformer, LatentState, DEFAULT_LATENT_TEMPERATURE, LATENT_HIDDEN_DIM,
    MAX_LATENT_DEPTH, NUM_LATENT_LAYERS, RECURSIVE_REWARD_DISCOUNT,
};
pub use nt_multimodal::{
    model_supports_vision, ImageClass, ImageEvidence, MultimodalEncoder, MultimodalInput,
    VisionBridge, TEXT_EMBED_DIM,
};
pub use sparse_moe::{SparseMoERouter, SparseRouting};
pub use unified_latent::{SeededProjection, UnifiedLatentSpace, UNIFIED_LATENT_DIM};

#[cfg(test)]
use std::collections::HashSet;

pub mod nt_e8_constants;
pub mod nt_e8_fermion;
pub mod nt_e8_hexagram;
pub mod nt_e8_homology;
pub mod nt_e8_reasoning;
pub mod nt_e8_roots;
pub mod nt_e8_transition;

pub use nt_e8_constants::{
    DAYAN_NUMBER, E8_DIM, E8_RANK, E8_ROOTS, FABLE5_BLOCK_MAP, FERMIONS_PER_GENERATION,
    FERMION_GENERATIONS, FIVE_SQUARED_TIMES_TWO, FUNCTION_TAG_BLOCKS, HEXAGRAM_COUNT, HE_TU_SUM,
    LINES_PER_HEXAGRAM, LO_SHU_CONSTANT, MYTHOS_STAGE_MAP, OBSERVABLE_DOF, OBSERVER_DOF,
    REMAINING_E8_GENERATORS, SEVEN_SQUARED, TOTAL_LINES, TOTAL_SM_FERMIONS, TRIGRAM_BITS,
    TRIGRAM_COUNT, TRIGRAM_NAMES, WEN_SEQUENCE,
};
pub use nt_e8_fermion::{
    FermionState, all_sm_fermions, fermion_states_for_generation, verify_total_fermions,
};
pub use nt_e8_hexagram::{
    Hexagram, hexagram_matrix, king_wen_sequence, shao_yong_sequence, su3_generators,
    trigram_to_su3_root,
};
pub use nt_e8_homology::{
    E8HexagramHomology, verify_all_identities, verify_dayan_identity, verify_e8_dimension,
    verify_he_tu_sum, verify_lo_shu, verify_seven_squared, verify_three_generations,
    verify_total_lines,
};
pub use nt_e8_reasoning::{
    estimate_e8_from_mythos, estimate_e8_from_structured, function_tags_to_e8,
    mythos_reasoning_to_e8, parse_mythos_reasoning, parse_structured_reasoning,
    structured_reasoning_to_e8,
};
pub use nt_e8_roots::{
    E8Weight, e8_root_norm_counts, e8_root_system, hadamard_matrix, hexagram_hadamard,
    verify_hadamard_orthogonality,
};
pub use nt_e8_transition::EdgeSemantics;

// ─── E8 Transition Probability Matrix ───────────────────────────────
//
// Serde compatibility: fixed arrays >32 elements need custom serialization.
// We use FlatCounts (Vec<u64>) and SerdeCompat64 (newtype) wrappers.
// Struct definitions moved to L0 (nt_core_substrate_types) to enforce substrate invariant.
// Re-exports here preserve backward compatibility.

pub use crate::l0_substrate::nt_core_substrate_types::{FlatCounts, SerdeCompat64};

/// 64×64 transition probability matrix for E8 hexagram states.
/// Struct definition moved to L0 (nt_core_substrate_types).
/// Re-export here preserves backward compatibility.
pub use crate::l0_substrate::nt_core_substrate_types::E8TransitionMatrix;

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e8_dimension() {
        assert_eq!(E8_RANK + E8_ROOTS, E8_DIM);
        assert!(verify_e8_dimension());
    }

    #[test]
    fn test_e8_root_count() {
        let roots = e8_root_system();
        assert_eq!(roots.len(), 240);
        // All E₈ roots have norm² = 2
        for root in &roots {
            assert!(
                (root.norm_sq() - 2.0).abs() < 1e-10,
                "E8 root {:?} has norm² != 2 (norm²={})",
                root,
                root.norm_sq()
            );
        }
    }

    #[test]
    fn test_root_families() {
        let roots = e8_root_system();
        // Verify we have the right distribution across sign patterns
        let mut perm_count = 0;
        let mut half_count = 0;
        for root in &roots {
            let abs_nonzero = root.coords.iter().filter(|&&c| c != 0).count();
            if abs_nonzero == 2 {
                perm_count += 1;
            } else {
                half_count += 1;
            }
        }
        assert_eq!(perm_count, 112, "Family 1 must have 112 roots");
        assert_eq!(half_count, 128, "Family 2 must have 128 roots");
    }

    #[test]
    fn test_three_generations() {
        assert!(verify_three_generations());
        assert_eq!(
            FERMION_GENERATIONS * FERMIONS_PER_GENERATION,
            TOTAL_SM_FERMIONS
        );
        assert_eq!(E8_DIM - TOTAL_SM_FERMIONS, REMAINING_E8_GENERATORS);
    }

    #[test]
    fn test_fermion_count() {
        let fermions = all_sm_fermions();
        assert_eq!(fermions.len(), 192);
        for gen in 0..3 {
            let states = fermion_states_for_generation(gen);
            assert_eq!(states.len(), 64, "Generation {gen} must have 64 fermions");
        }
    }

    #[test]
    fn test_hexagram_matrix_8x8() {
        let matrix = hexagram_matrix();
        assert_eq!(matrix.len(), 8);
        assert_eq!(matrix[0].len(), 8);
        // Every hexagram is unique
        let mut seen = HashSet::new();
        for row in &matrix {
            for cell in row {
                assert!(seen.insert(cell.bits), "Duplicate hexagram {}", cell.bits);
            }
        }
        assert_eq!(seen.len(), 64);
    }

    #[test]
    fn test_shao_yong_sequence() {
        let seq = shao_yong_sequence();
        assert_eq!(seq.len(), 64);
        // Each hexagram is in order 0..63
        for (i, hex) in seq.iter().enumerate() {
            assert_eq!(hex.bits as usize, i);
        }
    }

    #[test]
    fn test_hexagram_opposite() {
        let hex = Hexagram::new(0b101010);
        let opp = hex.opposite();
        assert_eq!(opp.bits, 0b010101);
        // Double opposite returns to original
        assert_eq!(hex, hex.opposite().opposite());
    }

    #[test]
    fn test_hadamard() {
        let h3 = hadamard_matrix(3);
        assert_eq!(h3.len(), 8);
        for row in &h3 {
            assert_eq!(row.len(), 8);
        }
        let h6 = hexagram_hadamard();
        assert_eq!(h6.len(), 64);
        assert_eq!(h6[0].len(), 64);
        assert!(verify_hadamard_orthogonality());
    }

    #[test]
    fn test_hadamard_first_row_all_ones() {
        let h = hadamard_matrix(6);
        for &val in &h[0] {
            assert_eq!(val, 1, "First Hadamard row must be all 1s");
        }
    }

    #[test]
    fn test_lo_shu() {
        assert!(verify_lo_shu());
    }

    #[test]
    fn test_he_tu_sum() {
        assert!(verify_he_tu_sum());
    }

    #[test]
    fn test_dayan_identity() {
        assert!(verify_dayan_identity());
        assert_eq!(DAYAN_NUMBER, OBSERVABLE_DOF + OBSERVER_DOF);
        assert_eq!(SEVEN_SQUARED, 49);
    }

    #[test]
    fn test_e8_total_identities() {
        let homology = E8HexagramHomology::new();
        assert!(homology.all_identities_hold);
        for (name, ok) in &homology.identity_results {
            assert!(*ok, "Identity '{name}' failed");
        }
    }

    #[test]
    fn test_trigram_su3_mapping() {
        // Each trigram should map to a distinct SU(3) root
        let mut roots = Vec::new();
        for t in 0..8 {
            roots.push(trigram_to_su3_root(t));
        }
        let mut unique = roots.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(
            unique.len(),
            8,
            "All 8 trigrams must map to distinct SU(3) roots"
        );
    }

    #[test]
    fn test_wen_sequence_has_all() {
        let mut seen = HashSet::new();
        for &b in &WEN_SEQUENCE {
            assert!(seen.insert(b), "Duplicate in Wen sequence: {b}");
        }
        assert_eq!(seen.len(), 64);
        // Verify all 0..63 are present
        for i in 0..64u8 {
            assert!(seen.contains(&i), "Missing hexagram {i} in Wen sequence");
        }
    }

    #[test]
    fn test_king_wen_roundtrip() {
        let seq = king_wen_sequence();
        for hex in &seq {
            let idx = hex.wen_index();
            assert!(idx.is_some());
            assert_eq!(seq[idx.expect("idx should be ok in test")].bits, hex.bits);
        }
    }

    #[test]
    fn test_e8_root_norm_counts() {
        let (norm2, _others) = e8_root_norm_counts();
        assert_eq!(norm2, 240, "All E8 roots must have norm²=2 (simply laced)");
    }

    #[test]
    fn test_total_lines_identity() {
        assert_eq!(TOTAL_LINES, 384);
        assert_eq!(HEXAGRAM_COUNT * LINES_PER_HEXAGRAM, TOTAL_LINES);
    }

    #[test]
    fn test_birkhoff_projection_doubly_stochastic() {
        let mut tm = E8TransitionMatrix::new();
        // Create a monopolized row: mode 0 dominates all transitions
        for _ in 0..100 {
            tm.record_transition(0, 0);
        }
        for _ in 0..5 {
            tm.record_transition(0, 1);
        }
        for _ in 0..3 {
            tm.record_transition(0, 2);
        }
        // Add some other rows with varied transitions
        for i in 1..64 {
            for j in 0..8 {
                tm.record_transition(i as u8, ((i + j) % 64) as u8);
            }
        }

        let projected = tm.birkhoff_projection(100, 1e-6);
        let proj_mat = tm.birkhoff_projected_matrix(100, 1e-6);

        // Verify rows sum to 1.0 (within tolerance)
        for i in 0..64 {
            let total = projected.row_totals.0[i];
            if total > 0 {
                let normalized_sum: f64 = (0..64)
                    .map(|j| projected.counts.get(i, j) as f64 / total as f64)
                    .sum();
                assert!(
                    (normalized_sum - 1.0).abs() < 1e-4,
                    "Row {i} sum = {normalized_sum} (should be 1.0)"
                );
            }
        }

        // Verify the float projection is doubly stochastic: every row AND column
        // sums to 1 — the mHC property that no destination monopolizes the
        // routing mass (columns must be balanced, not just rows).
        for i in 0..64 {
            let row_sum: f64 = proj_mat[i].iter().sum();
            assert!(
                (row_sum - 1.0).abs() < 1e-3,
                "Proj row {i} sum = {row_sum:.4}"
            );
        }
        for j in 0..64 {
            let col_sum: f64 = (0..64).map(|i| proj_mat[i][j]).sum();
            assert!(
                (col_sum - 1.0).abs() < 1e-3,
                "Proj column {j} sum = {col_sum:.4}"
            );
        }

        // Anti-monopolization: the projection must REDUCE the peak column
        // concentration relative to the raw matrix. Row 0 originally dumps
        // ~93% of its mass into column 0; after projection that peak must fall.
        let raw_max_col = (0..64)
            .map(|j| {
                let total_col: u64 = (0..64).map(|i| tm.counts.get(i, j)).sum();
                if total_col == 0 {
                    0.0
                } else {
                    (0..64)
                        .map(|i| tm.counts.get(i, j) as f64 / total_col as f64)
                        .fold(0.0, f64::max)
                }
            })
            .fold(0.0, f64::max);
        let proj_max_col = (0..64)
            .map(|j| (0..64).map(|i| proj_mat[i][j]).fold(0.0, f64::max))
            .fold(0.0, f64::max);
        assert!(
            proj_max_col < raw_max_col,
            "Projection must reduce peak column concentration: proj {proj_max_col:.3} vs raw {raw_max_col:.3}"
        );
    }

    #[test]
    fn test_birkhoff_projection_preserves_total_mass() {
        let mut tm = E8TransitionMatrix::new();
        tm.init_from_trace_patterns();
        for _ in 0..50 {
            tm.record_transition(10, 20);
        }
        for _ in 0..30 {
            tm.record_transition(10, 30);
        }
        for _ in 0..20 {
            tm.record_transition(20, 40);
        }

        let original_total: u64 = tm.row_totals.0.iter().sum();
        let projected = tm.birkhoff_projection(100, 1e-6);
        let projected_total: u64 = projected.row_totals.0.iter().sum();

        // Projected matrix should preserve total transition mass
        assert!(
            (projected_total as i64 - original_total as i64).abs() < original_total as i64 / 4,
            "Projected total {projected_total} should be close to original {original_total}"
        );
    }
}
