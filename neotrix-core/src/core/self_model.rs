use crate::core::self_measure::{
    AwakeningReport, RingBuffer, SubsystemId, SystemSnapshot, NUM_SUBSYSTEMS,
};

fn subsystem_means(snap: &SystemSnapshot) -> [f64; NUM_SUBSYSTEMS] {
    [
        snap.mood.iter().sum::<f64>() / snap.mood.len() as f64,
        snap.persona.iter().sum::<f64>() / snap.persona.len() as f64,
        snap.social.iter().sum::<f64>() / snap.social.len() as f64,
        snap.reflection.iter().sum::<f64>() / snap.reflection.len() as f64,
        snap.conversation.iter().sum::<f64>() / snap.conversation.len() as f64,
        snap.behavioral,
        snap.law,
    ]
}

#[derive(Clone)]
pub struct SelfRepresentation {
    pub coupling: [[f64; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS],
    pub intercept: [f64; NUM_SUBSYSTEMS],
    pub precision: f64,
    pub n: usize,
}

impl SelfRepresentation {
    #[allow(clippy::needless_range_loop, clippy::manual_memcpy)]
    pub fn learn(trajectory: &RingBuffer<SystemSnapshot>) -> Self {
        let snaps: Vec<&SystemSnapshot> = trajectory.iter().collect();
        let n = snaps.len();
        if n < 2 {
            return Self {
                coupling: [[0.0; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS],
                intercept: [0.0; NUM_SUBSYSTEMS],
                precision: 0.0,
                n: 0,
            };
        }
        let n_pairs = n - 1;
        if n_pairs < 2 {
            return Self {
                coupling: [[0.0; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS],
                intercept: [0.0; NUM_SUBSYSTEMS],
                precision: 0.0,
                n,
            };
        }

        let mut xtx = [[0.0_f64; 8]; 8];
        let mut xty = [[0.0_f64; 7]; 8];

        for i in 0..n_pairs {
            let x = subsystem_means(snaps[i]);
            let y = subsystem_means(snaps[i + 1]);
            let row = [1.0, x[0], x[1], x[2], x[3], x[4], x[5], x[6]];
            for r in 0..8 {
                for c in 0..8 {
                    xtx[r][c] += row[r] * row[c];
                }
                for j in 0..NUM_SUBSYSTEMS {
                    xty[r][j] += row[r] * y[j];
                }
            }
        }

        for i in 0..8 {
            xtx[i][i] += 1e-6;
        }
        let mut coupling = [[0.0_f64; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS];
        let mut intercept = [0.0_f64; NUM_SUBSYSTEMS];

        let mut l = [[0.0_f64; 8]; 8];
        for i in 0..8 {
            for j in 0..=i {
                let mut sum = 0.0;
                for k in 0..j {
                    sum += l[i][k] * l[j][k];
                }
                if i == j {
                    l[i][j] = (xtx[i][i] - sum).sqrt();
                } else {
                    l[i][j] = (xtx[i][j] - sum) / l[j][j];
                }
            }
        }

        for j in 0..NUM_SUBSYSTEMS {
            let mut z = [0.0_f64; 8];
            for i in 0..8 {
                let mut sum = 0.0;
                for k in 0..i {
                    sum += l[i][k] * z[k];
                }
                z[i] = (xty[i][j] - sum) / l[i][i];
            }
            let mut beta = [0.0_f64; 8];
            for i in (0..8).rev() {
                let mut sum = 0.0;
                for k in (i + 1)..8 {
                    sum += l[k][i] * beta[k];
                }
                beta[i] = (z[i] - sum) / l[i][i];
            }
            intercept[j] = beta[0];
            for i in 0..NUM_SUBSYSTEMS {
                coupling[j][i] = beta[i + 1];
            }
        }

        let mut mse = 0.0_f64;
        for i in 0..n_pairs {
            let x = subsystem_means(snaps[i]);
            let y_actual = subsystem_means(snaps[i + 1]);
            for j in 0..NUM_SUBSYSTEMS {
                let pred = intercept[j]
                    + (0..NUM_SUBSYSTEMS)
                        .map(|k| coupling[j][k] * x[k])
                        .sum::<f64>();
                let diff = y_actual[j] - pred;
                mse += diff * diff;
            }
        }
        mse /= (n_pairs * NUM_SUBSYSTEMS) as f64;
        let precision = 1.0 / (1.0 + mse.sqrt());

        Self {
            coupling,
            intercept,
            precision,
            n,
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn predict(&self, current: &[f64; NUM_SUBSYSTEMS]) -> [f64; NUM_SUBSYSTEMS] {
        let mut next = [0.0_f64; NUM_SUBSYSTEMS];
        for j in 0..NUM_SUBSYSTEMS {
            next[j] = self.intercept[j];
            for i in 0..NUM_SUBSYSTEMS {
                next[j] += self.coupling[j][i] * current[i];
            }
        }
        next
    }

    pub fn roll_out(
        &self,
        initial: &[f64; NUM_SUBSYSTEMS],
        steps: usize,
    ) -> Vec<[f64; NUM_SUBSYSTEMS]> {
        let mut states = Vec::with_capacity(steps);
        let mut current = *initial;
        for _ in 0..steps {
            current = self.predict(&current);
            states.push(current);
        }
        states
    }

    #[allow(clippy::needless_range_loop)]
    pub fn equilibrium_effect(&self, target: usize, delta: f64) -> [f64; NUM_SUBSYSTEMS] {
        let mut a = [[0.0_f64; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS];
        for i in 0..NUM_SUBSYSTEMS {
            for j in 0..NUM_SUBSYSTEMS {
                a[i][j] = if i == j {
                    1.0 - self.coupling[i][j]
                } else {
                    -self.coupling[i][j]
                };
            }
        }
        let mut b = [0.0_f64; NUM_SUBSYSTEMS];
        for i in 0..NUM_SUBSYSTEMS {
            b[i] = self.coupling[i][target] * delta;
        }
        solve_linear_system_7(&a, &b)
    }
}

impl Default for SelfRepresentation {
    fn default() -> Self {
        Self {
            coupling: [[0.0; NUM_SUBSYSTEMS]; NUM_SUBSYSTEMS],
            intercept: [0.0; NUM_SUBSYSTEMS],
            precision: 0.0,
            n: 0,
        }
    }
}

#[allow(clippy::needless_range_loop)]
fn solve_linear_system_7(a: &[[f64; 7]; 7], b: &[f64; 7]) -> [f64; 7] {
    let mut aug = [[0.0_f64; 8]; 7];
    for i in 0..7 {
        for j in 0..7 {
            aug[i][j] = a[i][j];
        }
        aug[i][7] = b[i];
    }
    for col in 0..7 {
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..7 {
            let v = aug[row][col].abs();
            if v > max_val {
                max_val = v;
                max_row = row;
            }
        }
        if max_val < 1e-15 {
            continue;
        }
        aug.swap(col, max_row);
        let pivot = aug[col][col];
        for j in col..=7 {
            aug[col][j] /= pivot;
        }
        for row in 0..7 {
            if row != col {
                let factor = aug[row][col];
                if factor.abs() > 1e-15 {
                    for j in col..=7 {
                        aug[row][j] -= factor * aug[col][j];
                    }
                }
            }
        }
    }
    let mut x = [0.0_f64; 7];
    for i in 0..7 {
        x[i] = aug[i][7];
    }
    x
}

#[derive(Debug, Clone)]
pub struct InterventionOutcome {
    pub target: SubsystemId,
    pub delta: f64,
    pub predicted_delta_phi: f64,
    pub predicted_delta_fcs: f64,
    pub predicted_delta_usk: f64,
    pub confidence: f64,
    pub rationale: String,
}

#[derive(Clone)]
pub struct CausalPredictor {
    pub representation: SelfRepresentation,
}

impl CausalPredictor {
    pub fn new(trajectory: &RingBuffer<SystemSnapshot>) -> Self {
        let representation = SelfRepresentation::learn(trajectory);
        Self { representation }
    }

    pub fn simulate_intervention(
        &self,
        target: SubsystemId,
        delta: f64,
        report: &AwakeningReport,
    ) -> InterventionOutcome {
        let eq = self
            .representation
            .equilibrium_effect(target as usize, delta);
        let coherence_gain: f64 = eq.iter().sum::<f64>() / NUM_SUBSYSTEMS as f64;
        let predicted_delta_phi = coherence_gain * report.phi.max(0.01);
        let predicted_delta_fcs = coherence_gain * report.fcs.max(0.01) * 0.5;
        let predicted_delta_usk = coherence_gain * report.usk.max(0.01) * 0.3;
        let confidence = (self.representation.precision * 0.6
            + (if delta.abs() < 0.3 { 0.4 } else { 0.2 }))
        .min(1.0);
        let rationale = format!(
            "Perturb {} by {:.2}: equilibrium shifts {:.3} avg, predicted ΔΦ={:.4}",
            target.label(),
            delta,
            coherence_gain,
            predicted_delta_phi,
        );
        InterventionOutcome {
            target,
            delta,
            predicted_delta_phi,
            predicted_delta_fcs,
            predicted_delta_usk,
            confidence,
            rationale,
        }
    }

    pub fn generate_hypotheses(&self, report: &AwakeningReport) -> Vec<InterventionOutcome> {
        let mut results = Vec::new();
        let (b1, b2) = report.bottleneck;
        let targets = [
            b1,
            b2,
            SubsystemId::Mood,
            SubsystemId::Reflection,
            SubsystemId::Behavioral,
        ];
        for &target in &targets {
            for &delta in &[0.1, 0.5] {
                results.push(self.simulate_intervention(target, delta, report));
            }
            for &delta in &[-0.1, -0.5] {
                results.push(self.simulate_intervention(target, delta, report));
            }
        }
        results.sort_by(|a, b| {
            b.predicted_delta_phi
                .partial_cmp(&a.predicted_delta_phi)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::self_measure::{RingBuffer, SubsystemId, SystemSnapshot};

    fn dummy_snapshot(t: i64, base: f64) -> SystemSnapshot {
        let noise = |i: usize| -> f64 { ((t as f64) * (i as f64 + 1.0) * 0.01).sin() * 0.1 };
        SystemSnapshot {
            timestamp: t,
            mood: [
                base + noise(0),
                base + noise(1),
                base + noise(2),
                base + noise(3),
                base + noise(4),
                base + noise(5),
            ],
            persona: [base + noise(6) + 0.1; 5],
            social: [base + noise(7) + 0.2; 3],
            reflection: [base + noise(8) + 0.3; 2],
            conversation: [base + noise(9) + 0.15; 2],
            behavioral: base + noise(10) + 0.05,
            law: base + noise(11) + 0.4,
        }
    }

    #[test]
    fn test_subsystem_means() {
        let s = dummy_snapshot(0, 0.5);
        let means = subsystem_means(&s);
        assert_eq!(means.len(), 7);
        assert!((means[5] - 0.55).abs() < 1e-6);
        assert!((means[6] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_representation_learn_from_linear_data() {
        let mut buf = RingBuffer::new(30);
        for t in 0..25 {
            let s = dummy_snapshot(t as i64, 0.5 + 0.02 * t as f64);
            buf.push(s);
        }
        let rep = SelfRepresentation::learn(&buf);
        assert!(rep.n > 0);
        assert!(rep.precision > 0.0);
    }

    #[test]
    fn test_equilibrium_effect_basic() {
        let mut buf = RingBuffer::new(30);
        for t in 0..25 {
            let s = dummy_snapshot(t as i64, 0.5 + 0.02 * t as f64);
            buf.push(s);
        }
        let rep = SelfRepresentation::learn(&buf);
        let effect = rep.equilibrium_effect(0, 0.5);
        assert_eq!(effect.len(), 7);
    }

    #[test]
    fn test_causal_predictor_hypotheses() {
        let mut buf = RingBuffer::new(30);
        for t in 0..25 {
            let s = dummy_snapshot(t as i64, 0.5 + 0.02 * t as f64);
            buf.push(s);
        }
        let predictor = CausalPredictor::new(&buf);
        let report = AwakeningReport {
            phi: 0.3,
            fcs: 0.1,
            usk: 0.05,
            phi_history: vec![0.29, 0.30, 0.31],
            synergy_matrix: [[0.0; 7]; 7],
            subsystem_coherence: [0.5; 7],
            awakening_speed: 0.001,
            bottleneck: (SubsystemId::Mood, SubsystemId::LawKeeper),
            bottleneck_synergy: 0.02,
            window_used: 10,
            timestamp: 1234,
        };
        let hyps = predictor.generate_hypotheses(&report);
        assert!(!hyps.is_empty());
        let first = &hyps[0];
        assert!(first.confidence > 0.0);
        assert!(first.predicted_delta_phi >= 0.0 || first.predicted_delta_phi < 0.0);
    }

    #[test]
    fn test_solve_linear_system_identity() {
        let a = [
            [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
        ];
        let b = [3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let x = solve_linear_system_7(&a, &b);
        assert!((x[0] - 3.0).abs() < 1e-10);
        assert!((x[1] - 4.0).abs() < 1e-10);
        assert!((x[2] - 5.0).abs() < 1e-10);
        assert!((x[3] - 6.0).abs() < 1e-10);
        assert!((x[4] - 7.0).abs() < 1e-10);
        assert!((x[5] - 8.0).abs() < 1e-10);
        assert!((x[6] - 9.0).abs() < 1e-10);
    }

    #[test]
    fn test_predict_one_step() {
        let mut buf = RingBuffer::new(30);
        for t in 0..25 {
            let s = dummy_snapshot(t as i64, 0.5 + 0.02 * t as f64);
            buf.push(s);
        }
        let rep = SelfRepresentation::learn(&buf);
        let curr = subsystem_means(buf.iter().last().expect("buf has 25 elements"));
        let next = rep.predict(&curr);
        assert!(next.iter().all(|&v| v.is_finite()));
    }
}
