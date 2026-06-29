use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReducedSPD {
    pub diagonal: Vec<f64>,
    pub correlations: Vec<(u32, u32, f64)>,
    pub dim: usize,
}

#[derive(Debug, Clone)]
pub struct ManifoldState {
    pub spd: ReducedSPD,
    pub label: String,
    pub energy: f64,
    pub cycle: u64,
}

pub struct GeometricSSM {
    pub current_state: Option<ManifoldState>,
    pub previous_state: Option<ManifoldState>,
    pub state_trajectory: Vec<ManifoldState>,
    pub max_trajectory: usize,
    pub geodesic_threshold: f64,
    pub active_dimensions: usize,
    cycle: u64,
}

impl ReducedSPD {
    pub fn from_vsa(v: &[u8], sample_stride: usize) -> Self {
        let len = v.len();
        let diagonal: Vec<f64> = v.iter().map(|&b| b as f64).collect();
        let mut correlations = Vec::new();
        let mut i = 0;
        while i < len {
            let mut j = i + sample_stride;
            while j < len && j < i + sample_stride * 4 {
                let corr = (v[i] as f64 - 0.5) * (v[j] as f64 - 0.5);
                if corr.abs() > 0.1 {
                    correlations.push((i as u32, j as u32, corr));
                }
                j += sample_stride;
            }
            i += sample_stride;
        }
        ReducedSPD {
            diagonal,
            correlations,
            dim: len,
        }
    }

    pub fn geodesic_distance(&self, other: &ReducedSPD) -> f64 {
        if self.dim != other.dim {
            return 1.0;
        }
        let diag_dist: f64 = self
            .diagonal
            .iter()
            .zip(other.diagonal.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
            / self.dim as f64;

        let corr_dist = if self.correlations.is_empty() && other.correlations.is_empty() {
            0.0
        } else {
            let self_set: std::collections::HashSet<(u32, u32)> =
                self.correlations.iter().map(|(i, j, _)| (*i, *j)).collect();
            let other_set: std::collections::HashSet<(u32, u32)> = other
                .correlations
                .iter()
                .map(|(i, j, _)| (*i, *j))
                .collect();
            let intersect = self_set.intersection(&other_set).count();
            let union_len = self_set.len().max(other_set.len());
            if union_len == 0 {
                0.0
            } else {
                1.0 - intersect as f64 / union_len as f64
            }
        };

        0.7 * diag_dist + 0.3 * corr_dist
    }

    pub fn energy(&self) -> f64 {
        self.diagonal.iter().map(|&x| x * x).sum::<f64>() / self.dim as f64
    }

    pub fn interpolate(&self, other: &ReducedSPD, t: f64) -> ReducedSPD {
        let diagonal: Vec<f64> = self
            .diagonal
            .iter()
            .zip(other.diagonal.iter())
            .map(|(a, b)| a * (1.0 - t) + b * t)
            .collect();
        let correlations = if t < 0.5 {
            self.correlations.clone()
        } else {
            other.correlations.clone()
        };
        ReducedSPD {
            diagonal,
            correlations,
            dim: self.dim,
        }
    }
}

impl GeometricSSM {
    pub fn new() -> Self {
        Self {
            current_state: None,
            previous_state: None,
            state_trajectory: Vec::new(),
            max_trajectory: 100,
            geodesic_threshold: 0.15,
            active_dimensions: VSA_DIM,
            cycle: 0,
        }
    }

    pub fn update_from_vsa(&mut self, vsa_vector: &[u8], label: &str) -> f64 {
        let spd = ReducedSPD::from_vsa(vsa_vector, 64);
        let mut state = ManifoldState {
            spd,
            label: label.to_string(),
            energy: 0.0,
            cycle: self.cycle,
        };

        self.previous_state = self.current_state.take();

        let distance = match &self.previous_state {
            Some(prev) => state.spd.geodesic_distance(&prev.spd),
            None => 0.0,
        };

        state.energy = state.spd.energy();
        self.current_state = Some(state.clone());
        self.state_trajectory.push(state);

        while self.state_trajectory.len() > self.max_trajectory {
            self.state_trajectory.remove(0);
        }

        self.cycle += 1;
        distance
    }

    pub fn state_change(&self) -> f64 {
        match (&self.current_state, &self.previous_state) {
            (Some(curr), Some(prev)) => curr.spd.geodesic_distance(&prev.spd),
            _ => 0.0,
        }
    }

    pub fn predict_next(&self) -> Option<ReducedSPD> {
        if self.state_trajectory.len() < 2 {
            return None;
        }
        let n = self.state_trajectory.len();
        let second_last = &self.state_trajectory[n - 2].spd;
        let last = &self.state_trajectory[n - 1].spd;
        Some(second_last.interpolate(last, 2.0))
    }

    pub fn check_vsa_convergence(current: &[u8], previous: &[u8], threshold: f64) -> bool {
        let spd_cur = ReducedSPD::from_vsa(current, 64);
        let spd_prev = ReducedSPD::from_vsa(previous, 64);
        spd_cur.geodesic_distance(&spd_prev) < threshold
    }

    pub fn estimate_betti(&self) -> (usize, usize) {
        if self.state_trajectory.len() < 3 {
            return (1, 0);
        }
        let mut clusters = Vec::new();
        for state in &self.state_trajectory {
            let mut found = false;
            for cluster in &clusters {
                if state.spd.geodesic_distance(cluster) < self.geodesic_threshold {
                    found = true;
                    break;
                }
            }
            if !found {
                clusters.push(state.spd.clone());
            }
        }
        let beta_0 = clusters.len();
        let beta_1 = if beta_0 > 1 {
            self.state_trajectory.len() - beta_0
        } else {
            0
        };
        (beta_0, beta_1.min(beta_0 * 2))
    }
}
