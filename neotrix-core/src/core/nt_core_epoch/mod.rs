pub mod definitions;
pub mod types;

pub use definitions::{
    all_frameworks, create_framework, default_router_bias, evaluate_in_epoch, initial_state_for,
    ontology_for,
};
pub use types::{ActivationRecord, CognitiveFramework, DimensionDef, EarthEpoch, FrameworkRoute};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_framework_does_not_panic() {
        let fw = create_framework(EarthEpoch::E4Scientific);
        assert_eq!(fw.epoch, EarthEpoch::E4Scientific);
        assert!(fw.dim() > 0);
    }

    #[test]
    fn test_evaluate_in_epoch_returns_valid_score() {
        let state = vec![0.5; 6];
        let score = evaluate_in_epoch(EarthEpoch::E4Scientific, &state, "analyze data");
        assert!((0.0..=1.0).contains(&score));
    }

    #[test]
    fn test_all_frameworks_have_correct_dimensions() {
        let frameworks = all_frameworks();
        assert_eq!(frameworks.len(), 8, "There should be exactly 8 epochs");

        let expected_dims: Vec<(EarthEpoch, usize)> = vec![
            (EarthEpoch::E1Mythological, 5),
            (EarthEpoch::E2Agricultural, 5),
            (EarthEpoch::E3Axial, 5),
            (EarthEpoch::E4Scientific, 6),
            (EarthEpoch::E5Global, 5),
            (EarthEpoch::E6Planetary, 5),
            (EarthEpoch::E7Network, 6),
            (EarthEpoch::E8Emergent, 5),
        ];

        for (fw, (epoch, expected_dims)) in frameworks.iter().zip(expected_dims.iter()) {
            assert_eq!(
                fw.dim(),
                *expected_dims,
                "Epoch {:?} should have {} dimensions",
                epoch,
                expected_dims
            );
            assert_eq!(fw.epoch, *epoch);
        }
    }

    #[test]
    fn test_epoch_evaluators_produce_valid_scores() {
        for epoch in EarthEpoch::all() {
            let ontology = ontology_for(epoch);
            let state = vec![0.5; ontology.len()];
            let score = evaluate_in_epoch(epoch, &state, "test generic task");
            assert!(
                (0.0..=1.0).contains(&score),
                "Epoch {:?} score {} should be in [0,1]",
                epoch,
                score
            );
        }
    }

    #[test]
    fn test_epoch_evaluators_respond_to_keywords() {
        let state = vec![0.5; ontology_for(EarthEpoch::E4Scientific).len()];
        let generic = evaluate_in_epoch(EarthEpoch::E4Scientific, &state, "write a poem");
        let scientific = evaluate_in_epoch(
            EarthEpoch::E4Scientific,
            &state,
            "analyze experimental data and measure precision",
        );
        assert!(scientific > generic);
    }

    #[test]
    fn test_e8_scores_high_for_self_improvement() {
        let state = vec![0.5; ontology_for(EarthEpoch::E8Emergent).len()];
        let generic = evaluate_in_epoch(EarthEpoch::E8Emergent, &state, "sort a list");
        let meta = evaluate_in_epoch(
            EarthEpoch::E8Emergent,
            &state,
            "meta-cognitive self-improvement loop for autonomous agents",
        );
        assert!(meta > generic);
    }

    #[test]
    fn test_framework_router_bias_defaults() {
        for epoch in EarthEpoch::all() {
            let bias = default_router_bias(epoch);
            assert!(
                (0.0..=1.0).contains(&bias),
                "Router bias for {:?} should be in [0,1], got {}",
                epoch,
                bias
            );
        }
    }

    #[test]
    fn test_update_and_normalize() {
        let mut fw = create_framework(EarthEpoch::E4Scientific);
        let original = fw.state.clone();
        let target: Vec<f64> = original.iter().map(|x| (x + 0.5).min(1.0)).collect();
        fw.update_from(&target, 1.0);
        for (s, t) in fw.state.iter().zip(target.iter()) {
            assert!((s - t).abs() < 1e-10);
        }
        fw.normalize();
        let max_val = fw.state.iter().cloned().fold(0.0f64, |a, x| a.max(x));
        assert!(max_val <= 1.0 + 1e-10);
    }

    #[test]
    fn test_activation_tracking() {
        let mut fw = create_framework(EarthEpoch::E7Network);
        assert_eq!(fw.activation_count, 0);
        fw.record_activation(0.5);
        fw.record_activation(0.8);
        assert_eq!(fw.activation_count, 2);
        assert!((fw.average_reward() - 0.65).abs() < 1e-10);
    }

    #[test]
    fn test_effective_weight_combines_bias_and_reward() {
        let mut fw = create_framework(EarthEpoch::E4Scientific);
        fw.router_bias = 0.5;
        let w0 = fw.effective_weight();
        assert!((w0 - 0.35).abs() < 1e-10);
        fw.record_activation(1.0);
        let w1 = fw.effective_weight();
        assert!((w1 - 0.65).abs() < 1e-10);
    }

    #[test]
    fn test_dimension_access_by_name() {
        let fw = create_framework(EarthEpoch::E7Network);
        assert!(fw.dimension_index("connectivity").is_some());
        assert!(fw.dimension_index("fake_dimension").is_none());
        assert!(fw.get("connectivity").is_some());
        assert!(fw.get("fake_dim").is_none());
    }
}
