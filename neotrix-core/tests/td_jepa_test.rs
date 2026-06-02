use neotrix::world_model::world_model_td_jepa::TemporalDifferenceJEPA;

fn make_state() -> Vec<f64> {
    vec![0.1, -0.2, 0.3, -0.1, 0.05, -0.05, 0.15, -0.25]
}

fn make_action() -> Vec<f64> {
    vec![0.5, -0.3, 0.1, 0.0, -0.2, 0.4, -0.1, 0.3]
}

fn make_next_state(state: &[f64]) -> Vec<f64> {
    state.iter().map(|s| s + 0.1 * (rand::random::<f64>() - 0.5)).collect()
}

#[test]
fn test_td_jepa_new() {
    let jepa = TemporalDifferenceJEPA::new(8);
    assert_eq!(jepa.latent_dim, 8);
    assert!((jepa.lambda - 0.9).abs() < 1e-10);
    assert!((jepa.gamma - 0.99).abs() < 1e-10);
    assert_eq!(jepa.delta_predictor.len(), 8 * 64);
    assert_eq!(jepa.value_head.len(), 8);
}

#[test]
fn test_predict_delta_basic() {
    let jepa = TemporalDifferenceJEPA::new(8);
    let state = make_state();
    let action = make_action();
    let delta = jepa.predict_delta(&state, &action);
    assert_eq!(delta.len(), 8);
    for &d in &delta {
        assert!(d.is_finite(), "delta value should be finite");
    }
}

#[test]
fn test_rollout_multi_step() {
    let jepa = TemporalDifferenceJEPA::new(8);
    let initial = make_state();
    let actions = vec![make_action(), make_action(), make_action()];
    let states = jepa.rollout(&initial, &actions, 0.9);
    assert_eq!(states.len(), 4);
    for s in &states {
        assert_eq!(s.len(), 8);
    }
    assert_eq!(states[0], initial);
}

#[test]
fn test_evaluate_policy_basic() {
    let jepa = TemporalDifferenceJEPA::new(8);
    let initial = make_state();
    let actions = vec![make_action(), make_action()];
    let value = jepa.evaluate_policy(&initial, &actions);
    assert!(value.is_finite(), "policy value should be finite");
}

#[test]
fn test_update_reduces_error() {
    let mut jepa = TemporalDifferenceJEPA::new(8);
    let state = make_state();
    let action = make_action();
    let next_state = make_next_state(&state);
    let reward = 0.5;

    let td_before = jepa.td_error(&state, &action, &next_state, reward).abs();
    for _ in 0..50 {
        jepa.update(&state, &action, &next_state, reward);
    }
    let td_after = jepa.td_error(&state, &action, &next_state, reward).abs();
    assert!(
        td_after <= td_before + 1e-6,
        "TD error should not increase after training: before={}, after={}",
        td_before,
        td_after
    );
}

#[test]
fn test_td_error_computation() {
    let mut jepa = TemporalDifferenceJEPA::new(4);
    jepa.value_head = vec![1.0, 0.5, -0.3, 0.2];
    let state = vec![1.0, 2.0, 3.0, 4.0];
    let next_state = vec![0.5, 1.0, 2.0, 3.0];
    let reward = 1.0;
    let action = vec![0.1, -0.1, 0.2, -0.2];
    let td = jepa.td_error(&state, &action, &next_state, reward);
    let expected = 1.0 + 0.99 * 1.0 - 1.9;
    assert!((td - expected).abs() < 1e-10, "td={}, expected={}", td, expected);
}

#[test]
fn test_rollout_convergence() {
    let jepa = TemporalDifferenceJEPA::new(8);
    let initial = make_state();
    let mut actions = Vec::new();
    for _ in 0..100 {
        actions.push(make_action());
    }
    let states = jepa.rollout(&initial, &actions, 0.9);
    assert_eq!(states.len(), 101);
    for s in &states {
        for &val in s {
            assert!(val.is_finite(), "state value should stay finite");
        }
    }
}

#[test]
fn test_invariant_under_scale() {
    let jepa = TemporalDifferenceJEPA::new(8);
    let state = make_state();
    let action = make_action();
    let delta1 = jepa.predict_delta(&state, &action);
    let scaled_state: Vec<f64> = state.iter().map(|s| s * 2.0).collect();
    let delta2 = jepa.predict_delta(&scaled_state, &action);
    assert_eq!(delta1.len(), delta2.len());
    for (_, &d2) in delta1.iter().zip(delta2.iter()) {
        assert!(d2.is_finite(), "scaled delta should be finite");
    }
    let delta3 = jepa.predict_delta(&state, &action);
    for (d_a, d_b) in delta1.iter().zip(delta3.iter()) {
        assert!((d_a - d_b).abs() < 1e-12, "deterministic prediction");
    }
}
