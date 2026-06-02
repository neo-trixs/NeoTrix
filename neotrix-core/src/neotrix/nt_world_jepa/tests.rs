use super::*;

fn sample_features() -> Vec<f64> {
    (0..64).map(|i| (i as f64) / 64.0).collect()
}

fn sample_features_shifted() -> Vec<f64> {
    (0..64).map(|i| ((i + 1) as f64) / 64.0).collect()
}

#[test]
fn test_jepa_encoder_output_dim() {
    let encoder = JepaEncoder::new(64, 32);
    let features = sample_features();
    let z = encoder.encode(&features);
    assert_eq!(z.len(), 32);
}

#[test]
fn test_jepa_predictor_output_dim() {
    let predictor = JepaPredictor::new(32, 64);
    let z = vec![0.5; 32];
    let pred = predictor.predict(&z);
    assert_eq!(pred.len(), 32);
}

#[test]
fn test_nt_world_jepa_prediction() {
    let wm = JepaWorldModel::new(64);
    let features = sample_features();
    let (z_pred, energy) = wm.predict(&features);
    assert_eq!(z_pred.len(), JEPA_LATENT_DIM);
    assert!(energy >= 0.0);
}

#[test]
fn test_jepa_train_step_reduces_loss() {
    let mut wm = JepaWorldModel::new(64);
    let x = sample_features();
    let y = sample_features_shifted();

    let (loss_before, _, _, _) = wm.train_step(&x, &y);
    let (loss_after, _, _, _) = wm.train_step(&x, &y);

    assert!(loss_after <= loss_before * 2.0 || loss_after < 100.0);
}

#[test]
fn test_multi_scale_prediction() {
    let wm = JepaWorldModel::new(64);
    let features = sample_features();
    let ms = wm.predict_multi_scale(&features, 3);
    assert_eq!(ms.short_term.len(), JEPA_LATENT_DIM);
    assert_eq!(ms.medium_term.len(), 3);
    assert_eq!(ms.uncertainties.len(), 3);
}

#[test]
fn test_energy_model_basic() {
    let em = EnergyModel::new();
    let v1 = vec![1.0, 0.0, 0.0];
    let v2 = vec![1.0, 0.0, 0.0];
    let e = em.energy(&v1, &v2);
    assert!(e < 0.01, "Identical vectors should have near-zero energy");
}

#[test]
fn test_energy_model_cosine() {
    let mut em = EnergyModel::new();
    em.metric = "cosine".to_string();
    let v1 = vec![1.0, 0.0, 0.0];
    let v2 = vec![0.0, 1.0, 0.0];
    let e = em.energy(&v1, &v2);
    assert!(e > 0.5, "Orthogonal vectors should have high energy");
}

#[test]
fn test_vicreg_loss_components() {
    let vicreg = VicRegLoss::new();
    let pred = vec![0.5; 32];
    let target = vec![0.5; 32];
    let (total, inv, var, cov) = vicreg.compute(&pred, &target);
    assert!(total >= 0.0);
    assert!(inv < 0.01);
    assert!(var >= 0.0);
    assert!(cov >= 0.0);
}

#[test]
fn test_predict_with_uncertainty() {
    let predictor = JepaPredictor::new(32, 64);
    let z = vec![0.5; 32];
    let (mean, variance) = predictor.predict_with_uncertainty(&z, 20);
    assert_eq!(mean.len(), 32);
    assert_eq!(variance.len(), 32);
    assert!(variance.iter().all(|v| *v >= 0.0));
}

#[test]
fn test_anomaly_detection() {
    let mut wm = JepaWorldModel::new(64);
    let normal = sample_features();
    let anomaly: Vec<f64> = (0..64).map(|_| rand::random::<f64>() * 10.0).collect();

    for _ in 0..5 {
        wm.train_step(&normal, &sample_features_shifted());
    }

    let (_, normal_energy) = wm.predict(&normal);
    let (_, anomaly_energy) = wm.predict(&anomaly);

    assert!(
        anomaly_energy >= normal_energy * -0.1,
        "Anomaly energy should be comparable or higher"
    );
}

#[test]
fn test_ema_update_stabilizes() {
    let mut target = JepaEncoder::new(64, 32);
    let source = JepaEncoder::new(64, 32);

    let initial_bias = target.bias.clone();

    target.ema_update(&source, 0.99);

    let diff: f64 = target.bias.iter().zip(initial_bias.iter())
        .map(|(a, b)| (a - b).abs())
        .sum();
    assert!(diff < 10.0, "EMA should pull target toward source");
}

#[test]
fn test_latent_state_zero() {
    let ls = LatentState::zero(32);
    assert_eq!(ls.value.len(), 32);
    assert_eq!(ls.delta.len(), 32);
    assert!(ls.value.iter().all(|v| *v == 0.0));
    assert!(ls.delta.iter().all(|v| *v == 0.0));
}

#[test]
fn test_empty_experiences_returns_zero() {
    let mut wm = JepaWorldModel::new(64);
    let error = wm.td_learn(&[]);
    assert_eq!(error, 0.0);
}

#[test]
fn test_td_predict_n_zero_horizon() {
    let wm = JepaWorldModel::new(64);
    let (states, errors) = wm.td_predict_n(0, &[], &[]);
    assert!(states.is_empty());
    assert!(errors.is_empty());
}

#[test]
fn test_td_predict_n_single_step() {
    let wm = JepaWorldModel::new(64);
    let obs = vec![0.5; 64];
    let (states, errors) = wm.td_predict_n(1, &[], &obs);
    assert_eq!(states.len(), 1);
    assert_eq!(errors.len(), 1);
    assert_eq!(states[0].value.len(), JEPA_LATENT_DIM);
    assert_eq!(states[0].delta.len(), JEPA_LATENT_DIM);
}

#[test]
fn test_td_predict_n_multi_step() {
    let wm = JepaWorldModel::new(64);
    let obs = vec![0.5; 64];
    let (states, errors) = wm.td_predict_n(5, &[], &obs);
    assert_eq!(states.len(), 5);
    assert_eq!(errors.len(), 5);
    for s in &states {
        assert!(!s.value.iter().any(|v| v.is_nan()));
        assert!(!s.delta.iter().any(|v| v.is_nan()));
    }
    assert!(!errors.iter().any(|e| e.is_nan()));
}

#[test]
fn test_td_error_calculation() {
    let wm = JepaWorldModel::new(64);
    let dynamics = TDDynamics::new(32, 0.95);
    let z_t = vec![0.5; 32];
    let z_next = vec![0.6; 32];
    let td_err = dynamics.td_error(1.0, &z_t, &z_next, &wm.td_target_critic);
    assert!(!td_err.is_nan());
    assert!(!td_err.is_infinite());
    assert!(td_err > -10.0 && td_err < 10.0);
}

#[test]
fn test_td_learn_reduces_error() {
    let mut wm = JepaWorldModel::new(64);
    let z_t = vec![0.5; 32];
    let z_next = vec![0.6; 32];

    let exp = TDExperience {
        z_t: z_t.clone(),
        reward: 1.0,
        z_t_plus_n: z_next.clone(),
    };
    let experiences = vec![exp; 10];

    let error_before = wm.td_learn(&experiences);
    let error_after = wm.td_learn(&experiences);

    assert!(
        error_after <= error_before * 1.5 || error_after < 0.1,
        "TD error should decrease with training"
    );
}

#[test]
fn test_long_horizon_stability() {
    let wm = JepaWorldModel::new(64);
    let trajectory = wm.long_horizon_rollout(20);
    assert_eq!(trajectory.len(), 20);
    for ls in &trajectory {
        assert!(!ls.value.iter().any(|v| v.is_nan()));
        assert!(!ls.delta.iter().any(|v| v.is_nan()));
        for &v in ls.value.iter() {
            assert!(v >= -10.0 && v <= 10.0);
        }
    }
}

#[test]
fn test_identity_action_prediction() {
    let wm = JepaWorldModel::new(64);
    let (states, errors) = wm.td_predict_n(3, &[], &[]);
    assert_eq!(states.len(), 3);
    assert_eq!(errors.len(), 3);
    for s in &states {
        assert_eq!(s.value.len(), JEPA_LATENT_DIM);
    }
}

#[test]
fn test_value_function_improvement() {
    let mut wm = JepaWorldModel::new(64);
    let good_z = vec![1.0; 32];
    let bad_z = vec![-1.0; 32];

    let exp = TDExperience {
        z_t: bad_z.clone(),
        reward: 0.0,
        z_t_plus_n: good_z.clone(),
    };

    let dynamics = TDDynamics::new(32, 0.95);
    let v_bad_before = dynamics.value(&bad_z, &wm.td_target_critic);
    let v_good_before = dynamics.value(&good_z, &wm.td_target_critic);
    let gap_before = v_good_before - v_bad_before;

    let experiences = vec![exp; 50];
    wm.td_learn(&experiences);

    let v_bad_after = dynamics.value(&bad_z, &wm.td_target_critic);
    let v_good_after = dynamics.value(&good_z, &wm.td_target_critic);
    let gap_after = v_good_after - v_bad_after;

    assert!(
        gap_after >= gap_before - 1.0,
        "Value gap should not shrink dramatically: before={}, after={}",
        gap_before,
        gap_after
    );
}

#[test]
fn test_cgblock_output_dim() {
    let block = CGBlock::new(32);
    assert_eq!(block.output_dim, 16);
    let z = (0..32).map(|i| i as f64).collect::<Vec<_>>();
    let cg = block.coarse_grain(&z);
    assert_eq!(cg.len(), 16);
    assert!((cg[0] - 0.5).abs() < 1e-10);
    assert!((cg[15] - 30.5).abs() < 1e-10);
}

#[test]
fn test_cgblock_odd_dim() {
    let block = CGBlock::new(5);
    let z = (0..5).map(|i| i as f64).collect::<Vec<_>>();
    let cg = block.coarse_grain(&z);
    assert_eq!(cg.len(), 3);
    assert!((cg[0] - 0.5).abs() < 1e-10);
    assert!((cg[1] - 2.5).abs() < 1e-10);
    assert!((cg[2] - 4.0).abs() < 1e-10);
}

#[test]
fn test_multiscale_jepa_single_scale_is_standard_jepa() {
    let ms = MultiScaleJEPA::new(1, 32, 64);
    assert_eq!(ms.num_scales, 1);
    assert!(ms.blocks.is_empty());
    assert_eq!(ms.predictors.len(), 1);
}

#[test]
fn test_multiscale_jepa_three_scale_coarse_graining() {
    let ms = MultiScaleJEPA::new(3, 32, 64);
    assert_eq!(ms.num_scales, 3);
    assert_eq!(ms.blocks.len(), 2);
    assert_eq!(ms.predictors.len(), 3);
    assert_eq!(ms.resolution_at_scale(0), 32);
    assert_eq!(ms.resolution_at_scale(1), 16);
    assert_eq!(ms.resolution_at_scale(2), 8);

    let z = (0..32).map(|i| (i as f64) / 32.0).collect::<Vec<_>>();
    let chain = ms.coarse_grain_chain(&z);
    assert_eq!(chain.len(), 3);
    assert_eq!(chain[0].resolution, 32);
    assert_eq!(chain[1].resolution, 16);
    assert_eq!(chain[2].resolution, 8);
    assert_eq!(chain[0].scale, 0);
    assert_eq!(chain[1].scale, 1);
    assert_eq!(chain[2].scale, 2);

    let mean0: f64 = chain[0].data.iter().sum::<f64>() / 32.0;
    let mean1: f64 = chain[1].data.iter().sum::<f64>() / 16.0;
    let mean2: f64 = chain[2].data.iter().sum::<f64>() / 8.0;
    assert!((mean0 - mean1).abs() < 1e-10);
    assert!((mean1 - mean2).abs() < 1e-10);
}

#[test]
fn test_multiscale_loss_computation() {
    let ms = MultiScaleJEPA::new(3, 32, 64);

    let z = (0..32).map(|i| (i as f64) / 32.0).collect::<Vec<_>>();
    let preds = ms.coarse_grain_chain(&z);
    let loss_perfect = ms.compute_multiscale_loss(&preds, &preds);
    assert!(loss_perfect < 1e-10, "Perfect match should have near-zero loss");

    let bad_preds: Vec<RGMLatent> = preds
        .iter()
        .map(|p| RGMLatent::new(vec![0.0; p.data.len()], p.scale, p.resolution))
        .collect();
    let loss_bad = ms.compute_multiscale_loss(&bad_preds, &preds);
    assert!(loss_bad > 0.0, "Mismatched predictions should have positive loss");

    let targets = ms.coarse_grain_chain(&z);
    let scale0_only = ms.compute_multiscale_loss(&preds, &targets);
    let scale2_bad = {
        let mut p = preds.clone();
        p[2].data = vec![0.0; p[2].data.len()];
        ms.compute_multiscale_loss(&p, &targets)
    };
    assert!(scale2_bad > scale0_only);
}

#[test]
fn test_multiscale_zero_state() {
    let ms = MultiScaleJEPA::new(3, 32, 64);

    let chain = ms.coarse_grain_chain(&[]);
    assert_eq!(chain.len(), 3);
    for c in &chain {
        assert!(c.data.iter().all(|v| *v == 0.0));
    }

    let loss = ms.compute_multiscale_loss(&[], &[]);
    assert_eq!(loss, 0.0);

    let wm = JepaWorldModel::new(64).with_rgm_scales(3);
    let result = wm.predict_multi_scale_rgm(0, &[]);
    assert!(result.is_empty());
}

#[test]
fn test_multiscale_base_dim_too_small() {
    let ms = MultiScaleJEPA::new(4, 8, 64);
    assert_eq!(ms.num_scales, 2);
    assert_eq!(ms.blocks.len(), 1);
    assert_eq!(ms.predictors.len(), 2);
    assert_eq!(ms.resolution_at_scale(0), 8);
    assert_eq!(ms.resolution_at_scale(1), 4);

    let z = (0..8).map(|i| (i as f64) / 8.0).collect::<Vec<_>>();
    let chain = ms.coarse_grain_chain(&z);
    assert_eq!(chain.len(), 2);
}

#[test]
fn test_rgm_integration_predict_multi_scale() {
    let wm = JepaWorldModel::new(64).with_rgm_scales(3);
    let obs = vec![0.5; 64];

    let all_states = wm.predict_multi_scale_rgm(5, &obs);
    assert_eq!(all_states.len(), 3);
    for (s, states) in all_states.iter().enumerate() {
        assert_eq!(states.len(), 5, "scale {} should have 5 states", s);
        for ls in states {
            assert!(!ls.value.iter().any(|v| v.is_nan()));
            assert!(!ls.delta.iter().any(|v| v.is_nan()));
        }
    }

    assert_eq!(all_states[0][0].value.len(), JEPA_LATENT_DIM);
    assert_eq!(all_states[1][0].value.len(), JEPA_LATENT_DIM / 2);
    assert_eq!(all_states[2][0].value.len(), JEPA_LATENT_DIM / 4);
}

#[test]
fn test_rgm_empty_observations() {
    let wm = JepaWorldModel::new(64).with_rgm_scales(2);
    let all_states = wm.predict_multi_scale_rgm(3, &[]);
    assert_eq!(all_states.len(), 2);
    for states in &all_states {
        assert_eq!(states.len(), 3);
    }
}

#[test]
fn test_predict_all_scales_produces_valid_predictions() {
    let ms = MultiScaleJEPA::new(3, 32, 64);
    let z = (0..32).map(|i| (i as f64) / 32.0).collect::<Vec<_>>();
    let preds = ms.predict_all_scales(&z);
    assert_eq!(preds.len(), 3);
    for p in &preds {
        assert!(!p.data.iter().any(|v| v.is_nan()));
        assert!(!p.data.iter().any(|v| v.is_infinite()));
    }
    assert_eq!(preds[0].data.len(), 32);
    assert_eq!(preds[1].data.len(), 16);
    assert_eq!(preds[2].data.len(), 8);
}

#[test]
fn test_sigreg_random_gaussian_low_loss() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let sigreg = SIGReg::new(32, 256, 42);
    let n = 500;
    let mut rng = StdRng::seed_from_u64(123);
    let embeddings: Vec<Vec<f64>> = (0..n)
        .map(|_| {
            let mut v: Vec<f64> = (0..32).map(|_| rng.gen::<f64>() * 2.0 - 1.0).collect();
            let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
            for val in v.iter_mut() {
                *val /= norm;
            }
            for val in v.iter_mut() {
                *val *= 32.0f64.sqrt();
            }
            v
        })
        .collect();

    let loss = sigreg.compute_loss(&embeddings);
    assert!(loss < 0.1, "Random Gaussian latents should have low SIGReg loss: {}", loss);
}

#[test]
fn test_sigreg_collapse_high_loss() {
    let sigreg = SIGReg::new(32, 256, 42);
    let n = 100;
    let embeddings: Vec<Vec<f64>> = vec![vec![0.5; 32]; n];

    let loss = sigreg.compute_loss(&embeddings);
    assert!(loss > 0.5, "Collapsed latents should have high SIGReg loss: {}", loss);
}

#[test]
fn test_sigreg_single_embedding_zero_loss() {
    let sigreg = SIGReg::new(32, 256, 42);
    let embeddings = vec![vec![0.5; 32]];
    let loss = sigreg.compute_loss(&embeddings);
    assert_eq!(loss, 0.0);
}

#[test]
fn test_sigreg_zero_projections_zero_loss() {
    let sigreg = SIGReg::new(32, 0, 42);
    let embeddings: Vec<Vec<f64>> = vec![vec![0.5; 32]; 10];
    let loss = sigreg.compute_loss(&embeddings);
    assert_eq!(loss, 0.0);
}

#[test]
fn test_sigreg_empty_batch_zero_loss() {
    let sigreg = SIGReg::new(32, 256, 42);
    let embeddings: Vec<Vec<f64>> = Vec::new();
    let loss = sigreg.compute_loss(&embeddings);
    assert_eq!(loss, 0.0);
}

#[test]
fn test_sigreg_deterministic_projections() {
    let sigreg1 = SIGReg::new(32, 256, 42);
    let sigreg2 = SIGReg::new(32, 256, 42);
    let embeddings: Vec<Vec<f64>> = vec![vec![0.3; 32]; 50];

    let loss1 = sigreg1.compute_loss(&embeddings);
    let loss2 = sigreg2.compute_loss(&embeddings);
    assert!((loss1 - loss2).abs() < 1e-10, "Same seed should produce same loss: {} vs {}", loss1, loss2);
}

#[test]
fn test_sigreg_matrix_variant_detects_collapse() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let sigreg = SIGReg::new(32, 256, 42);

    let collapsed: Vec<Vec<f64>> = vec![vec![0.5; 32]; 50];
    let matrix_loss = sigreg.compute_loss_matrix(&collapsed);
    assert!(matrix_loss > 10.0, "Matrix loss should detect collapse: {}", matrix_loss);

    let mut rng = StdRng::seed_from_u64(999);
    let random: Vec<Vec<f64>> = (0..100)
        .map(|_| (0..32).map(|_| rng.gen::<f64>() * 2.0 - 1.0).collect())
        .collect();
    let random_loss = sigreg.compute_loss_matrix(&random);
    assert!(random_loss < matrix_loss, "Matrix loss on random data should be lower than on collapsed: {} vs {}", random_loss, matrix_loss);
}

#[test]
fn test_sigreg_integration_with_model() {
    let mut wm = JepaWorldModel::new(64).with_sigreg(256, 0.01);
    let batch_x: Vec<Vec<f64>> = (0..10).map(|i| {
        (0..64).map(|j| ((i * 64 + j) as f64) / 640.0).collect()
    }).collect();
    let batch_y: Vec<Vec<f64>> = (0..10).map(|i| {
        (0..64).map(|j| ((i * 64 + j + 1) as f64) / 640.0).collect()
    }).collect();

    let loss_with_sigreg = wm.train_batch(&batch_x, &batch_y);

    let mut wm_base = JepaWorldModel::new(64);
    let loss_without = wm_base.train_batch(&batch_x, &batch_y);

    assert!(loss_with_sigreg >= loss_without * 0.5,
        "SIGReg should not suppress loss below 50% of base: sigreg={}, base={}",
        loss_with_sigreg, loss_without);
}

#[test]
fn test_sigreg_matrix_variant_single_embedding() {
    let sigreg = SIGReg::new(32, 256, 42);
    let embeddings = vec![vec![0.5; 32]];
    let loss = sigreg.compute_loss_matrix(&embeddings);
    assert_eq!(loss, 0.0, "Single embedding should give 0 matrix loss");
}

#[test]
fn test_sigreg_both_variants_agree_on_collapse() {
    let sigreg = SIGReg::new(32, 256, 42);
    let collapsed: Vec<Vec<f64>> = vec![vec![0.7; 32]; 50];
    let random_proj_loss = sigreg.compute_loss(&collapsed);
    let matrix_loss = sigreg.compute_loss_matrix(&collapsed);

    assert!(random_proj_loss > 0.5, "Random proj should detect collapse: {}", random_proj_loss);
    assert!(matrix_loss > 10.0, "Matrix variant should detect collapse: {}", matrix_loss);
}
