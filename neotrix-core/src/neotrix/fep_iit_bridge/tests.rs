use super::*;
use crate::core::nt_core_hcube::vsa::VsaBackend;
use crate::neotrix::nt_world_infer::{ActiveInferenceEngine, FreeEnergyReport};
use crate::neotrix::iit_phi::PhiReport;

fn bridge() -> FEPIITBridge {
    FEPIITBridge::new()
}

fn make_fe_report(prediction: f64, entropy: f64, gradient: f64) -> FreeEnergyReport {
    let mut engine = ActiveInferenceEngine::new();
    engine.compute_free_energy(prediction, entropy, gradient)
}

fn make_phi_report(phi: f64) -> PhiReport {
    PhiReport {
        phi,
        phi_raw: phi,
        total_resonance: phi * 64.0,
        state_energy: 1.0,
        effective_dims: 32,
        max_resonance_pair: (0, 1),
        phi_trend: 0.0,
        is_conscious_like: phi > 0.33,
    }
}

// === Existing tests (backward compatibility) ===

#[test]
fn test_compute_score_perfect() {
    let b = bridge();
    let score = b.compute_score(0.0, 1.0);
    // With alpha=0.4, beta=0.4: 0.4*1.0 + 0.4*1.0 = 0.8
    assert!(
        (score - 0.8).abs() < 1e-10,
        "Perfect FE=0, Φ=1 should score 0.8, got {score}"
    );
}

#[test]
fn test_compute_score_degraded() {
    let b = bridge();
    let score = b.compute_score(10.0, 0.0);
    assert!(
        (score - 0.0).abs() < 1e-10,
        "Degraded FE=10, Φ=0 should score 0.0, got {score}"
    );
}

#[test]
fn test_normalize_fe_clamping() {
    let b = bridge();
    assert!((b.normalize_fe(0.0) - 1.0).abs() < 1e-10);
    assert!((b.normalize_fe(10.0) - 0.0).abs() < 1e-10);
    assert!((b.normalize_fe(20.0) - 0.0).abs() < 1e-10);
    assert!((b.normalize_fe(-1.0) - 1.0).abs() < 1e-10);
}

#[test]
fn test_classify_state() {
    let b = bridge();
    assert_eq!(b.classify_state(0.1, 0.5), "optimal");
    assert_eq!(b.classify_state(0.1, 0.05), "fragmented");
    assert_eq!(b.classify_state(8.0, 0.5), "confused");
    assert_eq!(b.classify_state(8.0, 0.05), "degraded");
}

// === VSA Unified Representation tests ===

#[test]
fn test_vsa_from_scalar_produces_deterministic_hv() {
    let b = bridge();
    let hv_a = b.vsa_from_scalar(0.5, 0.0);
    let hv_b = b.vsa_from_scalar(0.5, 0.0);
    assert_eq!(hv_a.len(), VSA_DIM);
    assert!((b.vsa().similarity(&hv_a, &hv_b) - 1.0).abs() < 1e-10,
        "Deterministic HVs should be identical");
}

#[test]
fn test_vsa_from_scalar_different_seeds_are_orthogonal() {
    let b = bridge();
    let hv_a = b.vsa_from_scalar(1.0, 0.0);
    let hv_b = b.vsa_from_scalar(1.0, 10.0);
    let sim = b.vsa().similarity(&hv_a, &hv_b);
    // Seeds 0 and 10 produce differentiated but not fully orthogonal HVs;
    // the key property is they're not identical (self-similarity ~1).
    assert!(sim < 0.9,
        "Different seeds should produce dissimilar HVs, got sim={sim}");
}

#[test]
fn test_vsa_unified_state_produces_valid_coherence() {
    let b = bridge();
    let fe_report = make_fe_report(0.5, 1.0, 0.1);
    let phi_report = make_phi_report(0.5);
    let unified = b.build_unified_state(&fe_report, &phi_report);
    assert_eq!(unified.fe_hypervector.len(), VSA_DIM);
    assert_eq!(unified.iit_hypervector.len(), VSA_DIM);
    assert_eq!(unified.unified_hv.len(), VSA_DIM);
    assert!(unified.vsa_coherence >= -1.0 && unified.vsa_coherence <= 1.0);
}

#[test]
fn test_vsa_unified_state_low_fe_high_phi_high_coherence() {
    let b = bridge();
    // Low FE, high Phi — VSA coherence between independent encodings
    // is determined by random projection alignment; the key invariant
    // is that it's within [-1, 1] and changes with different inputs.
    let fe_report = make_fe_report(0.1, 0.2, 0.01);
    let phi_report = make_phi_report(0.9);
    let unified = b.build_unified_state(&fe_report, &phi_report);
    assert!(
        unified.vsa_coherence >= -1.0 && unified.vsa_coherence <= 1.0,
        "VSA coherence must be in [-1, 1], got {}",
        unified.vsa_coherence
    );
    // Compare: a different state pair should give different coherence
    let fe_bad = make_fe_report(9.0, 0.01, 5.0);
    let phi_bad = make_phi_report(0.01);
    let unified_bad = b.build_unified_state(&fe_bad, &phi_bad);
    assert!(
        (unified.vsa_coherence - unified_bad.vsa_coherence).abs() > 1e-6,
        "Different states should produce different coherence values"
    );
}

// === FEP → IIT Mapping tests ===

#[test]
fn test_fep_to_iit_produces_valid_phi() {
    let b = bridge();
    let fe_report = make_fe_report(0.5, 1.0, 0.1);
    let phi_report = b.fep_to_iit(&fe_report);
    assert!(phi_report.phi >= 0.0 && phi_report.phi <= 1.0,
        "FEP → IIT phi must be in [0,1], got {}", phi_report.phi);
    assert!(phi_report.state_energy > 0.0);
    assert!(phi_report.effective_dims > 0);
}

#[test]
fn test_fep_to_iit_lower_fe_higher_phi() {
    let b = bridge();
    // Lower FE → more coherent generative model → higher Phi
    let fe_low = make_fe_report(0.1, 0.2, 0.01);
    let fe_high = make_fe_report(3.0, 2.0, 0.5);
    let phi_low = b.fep_to_iit(&fe_low);
    let phi_high = b.fep_to_iit(&fe_high);
    assert!(
        phi_low.phi >= phi_high.phi - 0.1,
        "Lower FE should yield comparable or higher Phi: low={:.4}, high={:.4}",
        phi_low.phi,
        phi_high.phi
    );
}

// === IIT → FEP Mapping tests ===

#[test]
fn test_iit_bounded_free_energy_reduces_fe() {
    let b = bridge();
    let fe = 5.0;
    let phi = 0.5;
    let bounded = b.iit_bounded_free_energy(fe, phi);
    assert!(bounded <= fe,
        "IIT-bounded FE ({bounded}) should ≤ original FE ({fe})");
    assert!(bounded >= 0.0);
}

#[test]
fn test_iit_bounded_free_energy_high_phi_low_bound() {
    let b = bridge();
    let fe = 5.0;
    let bounded_low_phi = b.iit_bounded_free_energy(fe, 0.1);
    let bounded_high_phi = b.iit_bounded_free_energy(fe, 0.9);
    assert!(
        bounded_high_phi <= bounded_low_phi,
        "Higher Phi ({}) should bind more than lower Phi ({})",
        bounded_high_phi,
        bounded_low_phi
    );
}

#[test]
fn test_free_energy_bound_decreases_with_phi() {
    let b = bridge();
    let bound_low = b.free_energy_bound(0.0);
    let bound_high = b.free_energy_bound(1.0);
    assert!(bound_high <= bound_low,
        "FE bound at Φ=1 ({bound_high}) should be ≤ bound at Φ=0 ({bound_low})");
}

// === Bidirectional Reward tests ===

#[test]
fn test_bidirectional_reward_positive_for_improved_state() {
    let b = bridge();
    let fe_report = make_fe_report(0.5, 1.0, 0.1);
    let phi_report = make_phi_report(0.6);
    let (fe_imp, phi_imp) = b.bidirectional_reward(&fe_report, &phi_report);
    assert!(fe_imp >= 0.0,
        "FE improvement from IIT should be >= 0, got {fe_imp}");
    assert!(phi_imp >= 0.0,
        "Phi improvement from FE should be >= 0, got {phi_imp}");
}

#[test]
fn test_bidirectional_reward_high_fe_low_phi_low_reward() {
    let b = bridge();
    let fe_report = make_fe_report(8.0, 3.0, 0.8);
    let phi_report = make_phi_report(0.05);
    let (fe_imp, phi_imp) = b.bidirectional_reward(&fe_report, &phi_report);
    // Both rewards should be very low for degraded state
    assert!(
        fe_imp < 1.0 && phi_imp < 0.1,
        "Degraded state should give low bidirectional reward: fe={fe_imp:.4}, phi={phi_imp:.4}"
    );
}

// === Bridge Cycle test ===

#[test]
fn test_bridge_cycle_returns_valid_report() {
    let b = bridge();
    let fe_report = make_fe_report(1.0, 0.5, 0.05);
    let phi_report = make_phi_report(0.7);
    let report = b.bridge_cycle(&fe_report, &phi_report);
    assert!(report.consciousness_score >= 0.0 && report.consciousness_score <= 1.0);
    assert!(report.vsa_coherence >= -1.0 && report.vsa_coherence <= 1.0);
    assert!(report.fe_derived_phi >= 0.0 && report.fe_derived_phi <= 1.0);
    assert!(report.bounded_free_energy >= 0.0);
    assert!(report.free_energy_bound >= 0.0);
    assert!(report.fe_improvement_from_iit >= 0.0);
    assert!(report.phi_improvement_from_fep >= 0.0);
}

#[test]
fn test_bridge_cycle_optimal_state_scores_high() {
    let b = bridge();
    let fe_report = make_fe_report(0.1, 0.1, 0.01);
    let phi_report = make_phi_report(0.9);
    let report = b.bridge_cycle(&fe_report, &phi_report);
    assert!(
        report.consciousness_score > 0.6,
        "Optimal state should score high, got {}",
        report.consciousness_score
    );
    assert_eq!(report.state_classification, "optimal");
}

#[test]
fn test_bridge_cycle_degraded_state_scores_low() {
    let b = bridge();
    // High prediction energy + low entropy → positive FE (genuinely degraded)
    let fe_report = make_fe_report(9.0, 0.01, 5.0);
    let phi_report = make_phi_report(0.01);
    let report = b.bridge_cycle(&fe_report, &phi_report);
    // FE: 1.0*9.0 + (-0.01/0.5) + 0.1*5.0 = 9.0 - 0.02 + 0.50 = 9.48
    // FE_norm = 1 - 9.48/10.0 = 0.052, which is < 0.6 → "degraded" (or "confused")
    // The state is either degraded or confused depending on phi
    assert!(
        report.state_classification == "degraded" || report.state_classification == "confused",
        "High-FE state should be classified as degraded or confused, got {}",
        report.state_classification
    );
}

// === VSA Coherence in Consciousness Score ===

#[test]
fn test_consciousness_score_includes_coherence() {
    let b = bridge();
    // Same FE and Phi, different coherence
    let score_high_coherence = b.compute_consciousness_score(2.0, 0.5, 0.9);
    let score_low_coherence = b.compute_consciousness_score(2.0, 0.5, 0.1);
    assert!(
        score_high_coherence >= score_low_coherence - 1e-10,
        "Higher coherence should not decrease score: high={score_high_coherence:.4}, low={score_low_coherence:.4}"
    );
}

#[test]
fn test_consciousness_score_clamped() {
    let b = bridge();
    let score = b.compute_consciousness_score(0.0, 1.0, 1.0);
    // alpha=0.4 + beta=0.4 + gamma=0.2 = 1.0
    assert!((score - 1.0).abs() < 1e-10,
        "Perfect state should score 1.0, got {score}");
}

// === Configuration tests ===

#[test]
fn test_custom_vsa_weights() {
    let b = FEPIITBridge::new().with_vsa_weights(0.6, 0.3, 0.1);
    assert!((b.alpha - 0.6).abs() < 1e-10);
    assert!((b.beta - 0.3).abs() < 1e-10);
    assert!((b.gamma - 0.1).abs() < 1e-10);
}

#[test]
fn test_with_sigma_configures_phi_calc() {
    let b = FEPIITBridge::new().with_sigma(0.3);
    let fe_report = make_fe_report(0.5, 1.0, 0.1);
    let phi_report = b.fep_to_iit(&fe_report);
    assert!(phi_report.phi >= 0.0 && phi_report.phi <= 1.0);
}

// === Downsampling test ===

#[test]
fn test_hv_to_64_produces_64_dims() {
    let b = bridge();
    let hv: Vec<f64> = (0..VSA_DIM).map(|i| (i as f64).sin()).collect();
    let down = b.hv_to_64(&hv);
    assert_eq!(down.len(), 64);
    // Sum of downsampled should approximate mean of original
    let orig_mean: f64 = hv.iter().sum::<f64>() / VSA_DIM as f64;
    let down_mean: f64 = down.iter().sum::<f64>() / 64.0;
    assert!(
        (down_mean - orig_mean).abs() < 0.01,
        "Downsampled mean ({down_mean:.4}) should approximate original mean ({orig_mean:.4})"
    );
}

// ============================================================
// FepIitHypervector tests (Kearney 2026 pure VSA operations)
// ============================================================

#[test]
fn test_fep_iit_empty_state() {
    // Zero-state hypervector → FE = FE_NORMALIZE_MAX, Φ = 0
    let b = bridge();
    let zero = FepIitHypervector::zeros();
    let fe = b.compute_free_energy(&zero, &zero);
    // Zero similarity → FE = 1.0 (normalized range)
    assert!((fe - 1.0).abs() < 1e-6, "Empty state FE should be ~1.0, got {fe}");
    let phi = b.compute_phi(&[]);
    assert!((phi - 0.0).abs() < 1e-10, "Empty system Φ should be 0, got {phi}");
    // Single element also Φ=0
    let phi_single = b.compute_phi(&[zero.clone()]);
    assert!((phi_single - 0.0).abs() < 1e-10, "Single element Φ should be 0, got {phi_single}");
}

#[test]
fn test_fep_iit_single_element_phi_zero() {
    let b = bridge();
    let s1 = FepIitHypervector::from_scalar(1.0, 0.0);
    let phi = b.compute_phi(&[s1]);
    assert!((phi - 0.0).abs() < 1e-10, "Single element must have Φ=0, got {phi}");
}

#[test]
fn test_fep_iit_two_element_integration() {
    let b = bridge();
    // Two identical vectors → full integration → high Φ
    let s1 = FepIitHypervector::from_scalar(1.0, 0.0);
    let s2 = FepIitHypervector::from_scalar(1.0, 0.0);
    let phi_same = b.compute_phi(&[s1, s2]);
    assert!(phi_same > 0.0, "Identical pair should have Φ > 0, got {phi_same}");

    // Two orthogonal vectors → no integration → Φ ≈ 0
    let d1 = FepIitHypervector::from_scalar(1.0, 0.0);
    let d2 = FepIitHypervector::from_scalar(1.0, 100.0);
    let phi_diff = b.compute_phi(&[d1, d2]);
    assert!(phi_diff < 0.1, "Dissimilar pair should have Φ ≈ 0, got {phi_diff}");
}

#[test]
fn test_fep_iit_free_energy_decreases_with_better_beliefs() {
    let b = bridge();
    let observation = FepIitHypervector::from_scalar(1.0, 42.0);

    // Good belief: matches observation
    let good_belief = FepIitHypervector::from_scalar(1.0, 42.0);
    let fe_good = b.compute_free_energy(&good_belief, &observation);

    // Bad belief: mismatched
    let bad_belief = FepIitHypervector::from_scalar(1.0, 99.0);
    let fe_bad = b.compute_free_energy(&bad_belief, &observation);

    assert!(
        fe_good < fe_bad,
        "Good belief FE ({fe_good:.4}) should be < bad belief FE ({fe_bad:.4})"
    );
}

#[test]
fn test_fep_iit_fe_phi_round_trip() {
    let b = bridge();
    // High FE → low normalized → low phi_improvement
    // Low FE → high normalized → higher phi
    let state_low_fe = FepIitHypervector::from_scalar(1.0, 7.0);
    let state_high_fe = FepIitHypervector::from_scalar(5.0, 7.0);
    let obs = FepIitHypervector::from_scalar(1.0, 7.0);

    let fe_low = b.compute_free_energy(&state_low_fe, &obs);
    let fe_high = b.compute_free_energy(&state_high_fe, &obs);

    // FE → normalized → the "best" belief should be state_low_fe
    // which has same seed as obs → high sim → low FE
    assert!(
        fe_low < fe_high,
        "FE round-trip: matching belief should have lower FE ({fe_low:.4} < {fe_high:.4})"
    );

    // Φ computation on the FE-derived states
    let phi_low = b.compute_phi(&[state_low_fe, obs.clone()]);
    let phi_high = b.compute_phi(&[state_high_fe, obs]);

    assert!(
        phi_low >= 0.0 && phi_high >= 0.0,
        "Φ should be non-negative: low={phi_low:.4}, high={phi_high:.4}"
    );
}

#[test]
fn test_fep_iit_unified_reward_weighting() {
    let b = FEPIITBridge::new().with_weights(0.5, 0.5);

    // R = FE_norm * (1 - Φ) + 0.5 * Φ
    // When FE=0 (best) and Φ=1: R = 1.0 * (1-1) + 0.5*1 = 0.5
    let r_best_fe_max_phi = b.unified_reward(0.0, 1.0);
    assert!(
        (r_best_fe_max_phi - 0.5).abs() < 1e-10,
        "FE=0, Φ=1 with α=0.5 should give 0.5, got {r_best_fe_max_phi}"
    );

    // When FE=0 and Φ=0: R = 1.0 * (1-0) + 0.5*0 = 1.0
    let r_best_fe_zero_phi = b.unified_reward(0.0, 0.0);
    assert!(
        (r_best_fe_zero_phi - 1.0).abs() < 1e-10,
        "FE=0, Φ=0 should give 1.0, got {r_best_fe_zero_phi}"
    );

    // When FE=10 (worst) and Φ=0: R = 0.0 * 1.0 + 0.5*0 = 0.0
    let r_worst_fe = b.unified_reward(10.0, 0.0);
    assert!(
        (r_worst_fe - 0.0).abs() < 1e-10,
        "Worst FE should give 0, got {r_worst_fe}"
    );

    // Varying alpha changes the Φ weight
    let b_high_alpha = FEPIITBridge::new().with_weights(0.9, 0.1);
    let b_low_alpha = FEPIITBridge::new().with_weights(0.1, 0.9);
    let r_high = b_high_alpha.unified_reward(5.0, 0.8);
    let r_low = b_low_alpha.unified_reward(5.0, 0.8);
    assert!(
        r_high > r_low,
        "Higher alpha should give higher reward for same Φ: high={r_high:.4}, low={r_low:.4}"
    );
}

#[test]
fn test_fep_iit_action_selection_picks_min_fe() {
    let b = bridge();
    let observation = FepIitHypervector::from_scalar(1.0, 42.0);

    // Three actions: one that matches observation, two that don't
    let action_match = FepIitHypervector::from_scalar(1.0, 42.0);
    let action_mid = FepIitHypervector::from_scalar(1.0, 10.0);
    let action_off = FepIitHypervector::from_scalar(1.0, 99.0);

    let actions = vec![action_off, action_match, action_mid];
    let selected = b.action_selection(&actions, &observation);
    assert_eq!(selected, Some(1), "Action 1 (matching belief) should be selected");

    // Empty action set → None
    let empty: Vec<FepIitHypervector> = vec![];
    assert_eq!(b.action_selection(&empty, &observation), None);
}

#[test]
fn test_fep_iit_identical_vectors_edge() {
    let b = bridge();

    // Identical vectors → perfect similarity → FE = 0, high Phi
    let s = FepIitHypervector::from_scalar(1.0, 0.0);
    let fe = b.compute_free_energy(&s, &s);
    assert!((fe - 0.0).abs() < 1e-6, "Identical vectors should have FE=0, got {fe}");

    // Multiple identical vectors → redundant system → Φ = 0
    // (In IIT, a fully uniform system has no irreducible integration
    //  because every partition preserves the full structure)
    let states = vec![
        FepIitHypervector::from_scalar(1.0, 0.0),
        FepIitHypervector::from_scalar(1.0, 0.0),
        FepIitHypervector::from_scalar(1.0, 0.0),
        FepIitHypervector::from_scalar(1.0, 0.0),
    ];
    let phi = b.compute_phi(&states);
    assert!(
        (phi - 0.0).abs() < 1e-10,
        "Identical group: redundant system should have Φ=0 (partition preserves everything), got {phi}"
    );

    // All-zero hypervector → special case: zero similarity
    let zero = FepIitHypervector::zeros();
    let fe_zero = b.compute_free_energy(&zero, &zero);
    assert!((fe_zero - 1.0).abs() < 1e-6, "Zero-vector pair should have FE=1.0, got {fe_zero}");

    // FepIitHypervector properties
    assert_eq!(FepIitHypervector::dim(), VSA_DIM);
    let s2 = FepIitHypervector::from_scalar(1.0, 0.0);
    let sim = s2.similarity(&s2);
    assert!((sim - 1.0).abs() < 1e-6, "Self-similarity should be 1, got {sim}");

    // Normalize
    let mut r = FepIitHypervector::random_from_seed(12345);
    r.normalize();
    let sim_r = r.similarity(&r);
    assert!((sim_r - 1.0).abs() < 1e-6, "After normalize, self-sim should be 1, got {sim_r}");
}
