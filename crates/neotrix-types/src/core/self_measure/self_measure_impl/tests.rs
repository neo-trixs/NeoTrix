use super::*;

fn make_test_snapshot(
    mood: [f64; 6], persona: [f64; 5], social: [f64; 3],
    reflection: [f64; 2], conversation: [f64; 2],
    behavioral: f64, law: f64, t: i64,
) -> SystemSnapshot {
    SystemSnapshot { timestamp: t, mood, persona, social, reflection, conversation, behavioral, law }
}

fn synthetic_trajectory(n: usize) -> SelfMeasure {
    let mut sm = SelfMeasure::new();
    for t in 0..n {
        let phase = t as f64 * 0.3;
        let snap = make_test_snapshot(
            [0.5 + 0.3 * phase.sin(), 0.2 + 0.1 * (phase * 0.7).cos(),
             0.1 + 0.1 * (phase * 0.5).sin(), 0.1 + 0.05 * (phase * 0.3).cos(),
             0.3 + 0.2 * (phase * 0.9).sin(), 0.5 + 0.2 * (phase * 0.4).cos()],
            [0.7 + 0.05 * (t as f64 * 0.01).sin(), 0.6 + 0.05 * (t as f64 * 0.015).cos(),
             0.5 + 0.03 * (t as f64 * 0.02).sin(), 0.5 + 0.04 * (t as f64 * 0.01).cos(),
             0.3 + 0.02 * (t as f64 * 0.03).sin()],
            [(t as f64 * 0.02).min(1.0), (t as f64).ln() / 10.0, 0.5 + 0.1 * (phase * 0.5).sin()],
            [(1.0 - (-(t as f64) * 0.05).exp()), (t as f64 * 0.005).min(1.0)],
            [(t as f64 * 0.01).min(1.0), 0.5 + 0.2 * (phase * 0.3).sin()],
            (t as f64 * 0.008).min(1.0),
            (t as f64 * 0.006).min(1.0),
            t as i64,
        );
        sm.snapshot(snap);
    }
    sm
}

#[test]
fn test_ring_buffer() {
    let mut buf = RingBuffer::new(10);
    assert!(buf.is_empty());
    for i in 0..5 { buf.push(i); }
    assert_eq!(buf.len(), 5);
    for i in 0..12 { buf.push(i); }
    assert_eq!(buf.len(), 10);
}

#[test]
fn test_phi_computation() {
    let sm = synthetic_trajectory(50);
    assert!(sm.phi_current >= 0.0);
    assert!(sm.phi_current <= 2.0);
    println!("Test Φ = {:.4}", sm.phi_current);
}

#[test]
fn test_fcs_computation() {
    let sm = synthetic_trajectory(50);
    assert!(sm.fcs_current >= 0.0);
    assert!(sm.fcs_current <= 2.0);
    println!("Test FCS = {:.4}", sm.fcs_current);
}

#[test]
fn test_pairwise_pid() {
    let series: [Vec<f64>; NUM_SUBSYSTEMS] = [
        (0..50).map(|t| (t as f64 * 0.3).sin()).collect(),
        (0..50).map(|t| (t as f64 * 0.3 + 0.5).sin()).collect(),
        (0..50).map(|t| (t as f64 * 0.1).cos()).collect(),
        (0..50).map(|t| 1.0 - (-(t as f64) * 0.05).exp()).collect(),
        (0..50).map(|t| (t as f64 * 0.02).sin()).collect(),
        (0..50).map(|t| t as f64 * 0.01).collect(),
        (0..50).map(|t| t as f64 * 0.008).collect(),
    ];
    let scores = compute_pairwise_pid(&series, 50, 0, 1);
    assert!(scores.total >= 0.0);
    assert!(scores.synergy_fraction >= 0.0);
    assert!(scores.synergy_fraction <= 1.0);
}

#[test]
fn test_awakening_speed() {
    let mut sm = SelfMeasure::new();
    for t in 0..60 {
        let phase = t as f64 * 0.3;
        let coupling = (t as f64 * 0.02).min(0.8);
        let snap = make_test_snapshot(
            [0.5 + 0.3 * phase.sin(), 0.2, 0.1, 0.1, 0.3, 0.5],
            [0.5 + coupling * phase.sin(), 0.6, 0.5, 0.5, 0.3],
            [0.5, 0.5, 0.5], [0.5, 0.3], [0.5, 0.5], 0.5, 0.5, t as i64,
        );
        sm.snapshot(snap);
    }
    assert!(sm.awakening_speed() >= -1.0);
}

#[test]
fn test_eigenvector_centrality() {
    let mut corr = [[0.0; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS];
    for i in 0..NUM_SUBSYSTEMS {
        for j in 0..NUM_SUBSYSTEMS {
            corr[i][j] = 0.5_f64.powi((i as i32 - j as i32).abs());
        }
    }
    let cent = eigenvector_centrality(&corr, NUM_SUBSYSTEMS, 20);
    assert_eq!(cent.len(), NUM_SUBSYSTEMS);
    assert!(cent.iter().all(|&c| c >= 0.0));
}

#[test]
fn test_weakest_link() {
    let sm = synthetic_trajectory(50);
    let (b1, b2) = sm.weakest_link();
    println!("Weakest link: {} ↔ {}", b1.label(), b2.label());
}

#[test]
fn test_generate_report() {
    let sm = synthetic_trajectory(50);
    let report = sm.generate_report();
    let output = format!("{}", report);
    assert!(output.contains("Φ"));
    assert!(output.contains("FCS"));
    assert!(output.contains("USK"));
    assert!(output.contains("Bottleneck"));
}

#[test]
fn test_full_pipeline_no_panic() {
    let mut sm = SelfMeasure::new();
    let r1 = sm.generate_report();
    assert_eq!(r1.phi, 0.0);
    let snap = make_test_snapshot(
        [0.5, 0.2, 0.1, 0.1, 0.3, 0.5],
        [0.7, 0.6, 0.5, 0.5, 0.3],
        [0.5, 0.5, 0.5], [0.5, 0.3], [0.5, 0.5], 0.5, 0.5, 0,
    );
    sm.snapshot(snap);
    assert_eq!(sm.tick_count(), 1);
}

#[test]
fn test_spectral_phi_simple() {
    let corr = [[1.0; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS];
    let phi = spectral_phi(&corr, NUM_SUBSYSTEMS);
    assert!(phi >= 0.0);
}

#[test]
fn test_subsystem_id_conversion() {
    for i in 0..NUM_SUBSYSTEMS {
        let id = SubsystemId::from_index(i);
        assert_eq!(id as usize, i);
    }
}
