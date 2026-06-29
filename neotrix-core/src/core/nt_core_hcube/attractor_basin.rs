use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BasinType {
    FixedPoint,
    LimitCycle,
    Chaotic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttractorBasin {
    pub id: u64,
    pub center: Vec<u8>,
    pub radius: f64,
    pub label: String,
    pub basin_type: BasinType,
    pub stability: f64,
    pub energy: f64,
    pub creation_step: u64,
    pub hit_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasinStats {
    pub basin_count: usize,
    pub average_stability: f64,
    pub total_energy: f64,
    pub entropy: f64,
    pub fixed_point_count: usize,
    pub limit_cycle_count: usize,
    pub chaotic_count: usize,
}

#[derive(Debug, Clone)]
pub struct AttractorBasinDynamics {
    basins: Vec<AttractorBasin>,
    noise_temperature: f64,
    relaxation_steps: usize,
    next_id: u64,
    step_count: u64,
}

impl AttractorBasinDynamics {
    pub fn new(noise_temperature: f64, relaxation_steps: usize) -> Self {
        Self {
            basins: Vec::new(),
            noise_temperature,
            relaxation_steps,
            next_id: 1,
            step_count: 0,
        }
    }

    pub fn add_basin(
        &mut self,
        center: Vec<u8>,
        radius: f64,
        label: &str,
        basin_type: BasinType,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let stability = match basin_type {
            BasinType::FixedPoint => 0.9,
            BasinType::LimitCycle => 0.6,
            BasinType::Chaotic => 0.3,
        };
        let energy = 1.0 - stability;
        self.basins.push(AttractorBasin {
            id,
            center,
            radius,
            label: label.to_string(),
            basin_type,
            stability,
            energy,
            creation_step: self.step_count,
            hit_count: 0,
        });
        id
    }

    pub fn find_basin(&self, query: &[u8]) -> Option<&AttractorBasin> {
        let dim = query.len().max(1) as f64;
        self.basins.iter().find(|b| {
            let dist = QuantizedVSA::hamming_distance(query, &b.center) as f64 / dim;
            dist <= b.radius
        })
    }

    pub fn nearest_basins(&self, query: &[u8], k: usize) -> Vec<(&AttractorBasin, f64)> {
        if self.basins.is_empty() || k == 0 {
            return Vec::new();
        }
        let dim = query.len().max(1) as f64;
        let mut scored: Vec<_> = self
            .basins
            .iter()
            .map(|b| {
                let dist = QuantizedVSA::hamming_distance(query, &b.center) as f64 / dim;
                (b, dist)
            })
            .collect();
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }

    pub fn relax_to_attractor(&self, state: &[u8]) -> Vec<u8> {
        let mut current = state.to_vec();
        let mut rng = rand::thread_rng();

        for _ in 0..self.relaxation_steps {
            let nearest = self.nearest_basins(&current, 1);
            if nearest.is_empty() {
                break;
            }

            let (basin, distance) = nearest[0];

            if distance > basin.radius {
                if self.noise_temperature > 0.0 {
                    let noise_prob = self.noise_temperature * 0.01;
                    for bit in &mut current {
                        if rng.gen::<f64>() < noise_prob {
                            *bit ^= 1;
                        }
                    }
                }
                continue;
            }

            let blending = if basin.radius > 0.0 {
                (1.0 - distance / basin.radius).clamp(0.0, 1.0)
            } else {
                1.0
            };

            let mut changed = false;
            for (c, center_bit) in current.iter_mut().zip(basin.center.iter()) {
                if rng.gen::<f64>() < blending {
                    if *c != *center_bit {
                        *c = *center_bit;
                        changed = true;
                    }
                }
            }

            if self.noise_temperature > 0.0 {
                let noise_prob = self.noise_temperature * (1.0 - blending) * 0.05;
                for bit in &mut current {
                    if rng.gen::<f64>() < noise_prob {
                        *bit ^= 1;
                        changed = true;
                    }
                }
            }

            if !changed {
                break;
            }
        }

        current
    }

    pub fn compute_basin_energy(&self, basin_id: u64) -> f64 {
        self.basins
            .iter()
            .find(|b| b.id == basin_id)
            .map(|basin| {
                let avg_dist = basin.radius * 0.5;
                let density = (basin.hit_count as f64) / (1.0 + basin.hit_count as f64);
                let raw = (avg_dist / (VSA_DIM as f64)) * density;
                (raw * 4.0).clamp(0.0, 1.0)
            })
            .unwrap_or(-1.0)
    }

    pub fn basin_overlap(&self, id_a: u64, id_b: u64) -> f64 {
        let a = self.basins.iter().find(|b| b.id == id_a);
        let b = self.basins.iter().find(|b| b.id == id_b);
        match (a, b) {
            (Some(a), Some(b)) => {
                let sim = QuantizedVSA::similarity(&a.center, &b.center);
                let r_min = a.radius.min(b.radius);
                let r_max = a.radius.max(b.radius);
                let radius_ratio = if r_max > 0.0 { r_min / r_max } else { 1.0 };
                sim * radius_ratio
            }
            _ => -1.0,
        }
    }

    pub fn basin_entropy(&self) -> f64 {
        let n = self.basins.len();
        if n == 0 {
            return 0.0;
        }
        let total_stability: f64 = self.basins.iter().map(|b| b.stability).sum();
        if total_stability <= 0.0 {
            return 0.0;
        }
        let entropy: f64 = self
            .basins
            .iter()
            .map(|b| {
                let p = b.stability / total_stability;
                if p > 0.0 {
                    -p * p.ln()
                } else {
                    0.0
                }
            })
            .sum();
        let max_entropy = (n as f64).ln();
        if max_entropy > 0.0 {
            entropy / max_entropy
        } else {
            0.0
        }
    }

    pub fn remove_unstable_basin(&mut self, threshold: f64) {
        self.basins.retain(|b| b.stability >= threshold);
    }

    pub fn stats(&self) -> BasinStats {
        let count = self.basins.len();
        let avg_stability = if count > 0 {
            self.basins.iter().map(|b| b.stability).sum::<f64>() / count as f64
        } else {
            0.0
        };
        let total_energy = self.basins.iter().map(|b| b.energy).sum();
        let entropy = self.basin_entropy();
        let fixed = self
            .basins
            .iter()
            .filter(|b| matches!(b.basin_type, BasinType::FixedPoint))
            .count();
        let limit = self
            .basins
            .iter()
            .filter(|b| matches!(b.basin_type, BasinType::LimitCycle))
            .count();
        let chaotic = self
            .basins
            .iter()
            .filter(|b| matches!(b.basin_type, BasinType::Chaotic))
            .count();
        BasinStats {
            basin_count: count,
            average_stability: avg_stability,
            total_energy,
            entropy,
            fixed_point_count: fixed,
            limit_cycle_count: limit,
            chaotic_count: chaotic,
        }
    }

    pub fn step(&mut self) {
        self.step_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_basin_vec(value: u8) -> Vec<u8> {
        vec![value; VSA_DIM]
    }

    #[test]
    fn test_add_and_find_basin() {
        let mut abd = AttractorBasinDynamics::new(0.0, 10);
        let center = test_basin_vec(1);
        let id = abd.add_basin(center.clone(), 0.1, "test_fixed", BasinType::FixedPoint);
        assert_eq!(id, 1);

        let near = test_basin_vec(1);
        let found = abd.find_basin(&near);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 1);

        let far = test_basin_vec(0);
        let not_found = abd.find_basin(&far);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_relax_to_attractor() {
        let mut abd = AttractorBasinDynamics::new(0.0, 20);
        let center = test_basin_vec(1);
        abd.add_basin(center.clone(), 0.3, "fixed", BasinType::FixedPoint);

        let noisy: Vec<u8> = center
            .iter()
            .map(|&x| {
                if rand::random::<f64>() < 0.2 {
                    x ^ 1
                } else {
                    x
                }
            })
            .collect();

        let original_dist = QuantizedVSA::hamming_distance(&noisy, &center) as f64 / VSA_DIM as f64;

        let relaxed = abd.relax_to_attractor(&noisy);
        let relaxed_dist =
            QuantizedVSA::hamming_distance(&relaxed, &center) as f64 / VSA_DIM as f64;

        assert!(
            relaxed_dist <= original_dist + 0.01,
            "relaxed ({:.4}) should be as close or closer than original ({:.4})",
            relaxed_dist,
            original_dist
        );
    }

    #[test]
    fn test_nearest_basins() {
        let mut abd = AttractorBasinDynamics::new(0.0, 10);
        let center_a = test_basin_vec(0);
        let center_b = test_basin_vec(1);
        let _center_c = test_basin_vec(0);

        abd.add_basin(center_a.clone(), 0.1, "basin_a", BasinType::FixedPoint);
        abd.add_basin(center_b.clone(), 0.1, "basin_b", BasinType::FixedPoint);

        let query = test_basin_vec(0);
        let nearest = abd.nearest_basins(&query, 2);
        assert_eq!(nearest.len(), 2);
        assert_eq!(nearest[0].1, 0.0);
    }

    #[test]
    fn test_basin_overlap() {
        let mut abd = AttractorBasinDynamics::new(0.0, 10);
        let center_a = test_basin_vec(0);
        let center_b = test_basin_vec(1);
        let center_c = test_basin_vec(0);

        let id_a = abd.add_basin(center_a, 0.2, "overlap_a", BasinType::FixedPoint);
        let id_b = abd.add_basin(center_b, 0.2, "overlap_b", BasinType::LimitCycle);
        let id_c = abd.add_basin(center_c, 0.2, "overlap_c", BasinType::FixedPoint);

        let overlap_ab = abd.basin_overlap(id_a, id_b);
        assert!(overlap_ab >= 0.0);
        let overlap_ac = abd.basin_overlap(id_a, id_c);
        assert!((overlap_ac - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_remove_unstable() {
        let mut abd = AttractorBasinDynamics::new(0.0, 10);
        abd.add_basin(test_basin_vec(0), 0.1, "stable", BasinType::FixedPoint);
        abd.add_basin(test_basin_vec(1), 0.1, "unstable", BasinType::Chaotic);

        assert_eq!(abd.basins.len(), 2);
        abd.remove_unstable_basin(0.5);
        assert_eq!(abd.basins.len(), 1);
        assert_eq!(abd.basins[0].label, "stable");
    }

    #[test]
    fn test_basin_entropy() {
        let mut abd = AttractorBasinDynamics::new(0.0, 10);
        let empty_entropy = abd.basin_entropy();
        assert!((empty_entropy - 0.0).abs() < 1e-6);

        abd.add_basin(test_basin_vec(0), 0.1, "a", BasinType::FixedPoint);
        let single_entropy = abd.basin_entropy();
        assert!((single_entropy - 0.0).abs() < 1e-6);

        abd.add_basin(test_basin_vec(1), 0.1, "b", BasinType::FixedPoint);
        let multi_entropy = abd.basin_entropy();
        assert!(
            multi_entropy > 0.0 && multi_entropy <= 1.0,
            "entropy should be in (0,1] for multiple basins, got {}",
            multi_entropy
        );
    }

    #[test]
    fn test_basin_stats() {
        let mut abd = AttractorBasinDynamics::new(0.0, 10);
        abd.add_basin(test_basin_vec(0), 0.1, "fp", BasinType::FixedPoint);
        abd.add_basin(test_basin_vec(1), 0.1, "lc", BasinType::LimitCycle);
        abd.add_basin(test_basin_vec(0), 0.1, "ch", BasinType::Chaotic);

        let stats = abd.stats();
        assert_eq!(stats.basin_count, 3);
        assert_eq!(stats.fixed_point_count, 1);
        assert_eq!(stats.limit_cycle_count, 1);
        assert_eq!(stats.chaotic_count, 1);
        assert!(stats.average_stability > 0.0);
        assert!(stats.total_energy > 0.0);
        assert!(stats.entropy > 0.0);
    }

    #[test]
    fn test_relax_with_noise() {
        let mut abd = AttractorBasinDynamics::new(0.1, 30);
        let center = test_basin_vec(1);
        abd.add_basin(center.clone(), 0.5, "noisy_basin", BasinType::FixedPoint);

        let far: Vec<u8> = center
            .iter()
            .map(|&x| {
                if rand::random::<f64>() < 0.4 {
                    x ^ 1
                } else {
                    x
                }
            })
            .collect();

        let before_dist = QuantizedVSA::hamming_distance(&far, &center) as f64 / VSA_DIM as f64;

        let relaxed = abd.relax_to_attractor(&far);
        let after_dist = QuantizedVSA::hamming_distance(&relaxed, &center) as f64 / VSA_DIM as f64;

        assert!(
            after_dist <= before_dist + 0.02,
            "relaxation should not increase distance: before={:.4} after={:.4}",
            before_dist,
            after_dist
        );
    }

    #[test]
    fn test_nonexistent_basin_energy_returns_neg_one() {
        let abd = AttractorBasinDynamics::new(0.0, 10);
        let energy = abd.compute_basin_energy(999);
        assert!((energy + 1.0).abs() < 1e-6);
    }
}
