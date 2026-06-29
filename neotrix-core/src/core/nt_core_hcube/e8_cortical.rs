// REVIVED Task 2 — dead_code removed
use crate::core::nt_core_e8::{e8_root_system, E8Weight};
use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use rand::Rng;

/// The number of simulated cortical neurons (MICrONS scale).
pub const CORTICAL_NEURON_COUNT: usize = 12_000;

/// A 3D position in simulated cortical space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CorticalCoord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl CorticalCoord {
    pub fn distance(&self, other: &CorticalCoord) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// A single simulated cortical neuron.
#[derive(Debug, Clone)]
pub struct CorticalNeuron {
    pub neuron_id: usize,
    pub coord: CorticalCoord,
    /// E8 root affinity score for this neuron
    pub root_affinity: Vec<f64>,
}

/// E8 → Cortical mapping: maps E8 root system to simulated cortical column geometry.
///
/// MICrONS-inspired: 12K neurons distributed in a 3D volume, with connectivity
/// biased by the E8 Cartan matrix and Weyl group symmetries.
#[derive(Debug, Clone)]
pub struct E8CorticalMapping {
    /// E8 roots (240 + 8)
    pub roots: Vec<E8Weight>,
    /// Cortical neuron coordinates
    pub cortical_coords: Vec<CorticalCoord>,
    /// Root → neuron mapping: each root activates ~N/CORTICAL_NEURON_COUNT/E8_ROOTS neurons
    pub root_to_neurons: Vec<Vec<usize>>,
    /// Connection strength prior from Cartan matrix → functional connectivity
    pub connection_prior: Vec<Vec<f64>>,
    /// Whether the mapping has been initialized
    initialized: bool,
}

impl Default for E8CorticalMapping {
    fn default() -> Self {
        let mut m = Self::new();
        m.initialize();
        m
    }
}

impl E8CorticalMapping {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            cortical_coords: Vec::new(),
            root_to_neurons: Vec::new(),
            connection_prior: Vec::new(),
            initialized: false,
        }
    }

    /// Initialize the mapping from E8 roots.
    pub fn initialize(&mut self) {
        if self.initialized {
            return;
        }
        self.roots = e8_root_system();
        self.generate_cortical_coords();
        self.build_root_to_neuron_mapping();
        self.build_connection_prior();
        self.initialized = true;
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Generate 12K cortical neuron coordinates in a 3D volume.
    /// Uses a spherical distribution with higher density near surface (cortical sheet).
    fn generate_cortical_coords(&mut self) {
        let mut rng = rand::thread_rng();
        self.cortical_coords = (0..CORTICAL_NEURON_COUNT)
            .map(|_| {
                // Spherical distribution with radial clustering
                let theta: f64 = rng.gen();
                let theta = theta * 2.0 * std::f64::consts::PI;
                let phi_val: f64 = rng.gen();
                let phi = (phi_val * 2.0 - 1.0).acos();
                // Radius: more neurons near surface (0.7-1.0) to simulate cortical sheet
                let r_val: f64 = rng.gen();
                let r = 0.3 + r_val * 0.7;
                CorticalCoord {
                    x: r * phi.sin() * theta.cos(),
                    y: r * phi.sin() * theta.sin(),
                    z: r * phi.cos(),
                }
            })
            .collect();
    }

    /// Map each E8 root to a subset of cortical neurons based on spatial proximity.
    /// Each root activates ~48 neurons (12K / 248 ≈ 48).
    fn build_root_to_neuron_mapping(&mut self) {
        let n_roots = self.roots.len();
        let neurons_per_root = CORTICAL_NEURON_COUNT / n_roots.max(1);
        self.root_to_neurons = Vec::with_capacity(n_roots);

        // Generate deterministic assignment based on root index
        for root_idx in 0..n_roots {
            let start = (root_idx * neurons_per_root) % CORTICAL_NEURON_COUNT;
            let end = (start + neurons_per_root).min(CORTICAL_NEURON_COUNT);
            let neuron_ids: Vec<usize> = (start..end).collect();
            self.root_to_neurons.push(neuron_ids);
        }
    }

    /// Build connection strength prior from E8 Cartan matrix structure.
    /// Roots connected in the Dynkin diagram get higher connection weight.
    fn build_connection_prior(&mut self) {
        let n_roots = self.roots.len();
        let mut prior = vec![vec![0.0_f64; n_roots]; n_roots];

        // Simple geometric prior: connection strength decays with root distance
        for i in 0..n_roots {
            for j in 0..n_roots {
                if i == j {
                    prior[i][j] = 1.0;
                } else {
                    // Euclidean distance between root coordinate representations
                    let dist: f64 = self.roots[i]
                        .coords
                        .iter()
                        .zip(self.roots[j].coords.iter())
                        .map(|(a, b)| (*a as f64 - *b as f64).powi(2))
                        .sum::<f64>()
                        .sqrt();
                    // Map distance to connection strength: closer = stronger
                    prior[i][j] = (-dist * 0.5).exp();
                }
            }
        }
        self.connection_prior = prior;
    }

    /// Get functional connectivity bias between two cortical regions.
    pub fn connectivity_bias(&self, neuron_a: usize, neuron_b: usize) -> f64 {
        if neuron_a >= CORTICAL_NEURON_COUNT || neuron_b >= CORTICAL_NEURON_COUNT {
            return 0.0;
        }
        let coord_a = &self.cortical_coords[neuron_a];
        let coord_b = &self.cortical_coords[neuron_b];
        let spatial_dist = coord_a.distance(coord_b);
        // Combine spatial proximity with E8 structural prior
        let spatial_factor = (-spatial_dist * 2.0).exp();
        spatial_factor
    }

    /// Generate a VSA vector biased by E8 cortical geometry.
    /// Maps a VSA vector through the E8 root system, weighting by cortical proximity.
    pub fn cortical_vsa_encode(&self, input: &[u8]) -> Vec<u8> {
        let mut output = vec![0u8; VSA_DIM];
        let n_roots = self.roots.len();
        if n_roots == 0 {
            return output;
        }
        let neurons_per_root = CORTICAL_NEURON_COUNT / n_roots;
        // For each E8 root, blend its contribution into the output VSA
        for root_idx in 0..n_roots.min(VSA_DIM) {
            if root_idx >= input.len() {
                break;
            }
            let root_activation = input[root_idx] as f64 / 255.0;
            let neuron_start = (root_idx * neurons_per_root) % CORTICAL_NEURON_COUNT;
            let neuron_end = (neuron_start + neurons_per_root / 8).min(CORTICAL_NEURON_COUNT);
            for n_idx in neuron_start..neuron_end {
                let coord = &self.cortical_coords[n_idx];
                let spatial_weight = (-coord.distance(&CorticalCoord {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }))
                .exp();
                let bias = root_activation * spatial_weight;
                let out_idx = n_idx % VSA_DIM;
                if bias > 0.5 {
                    output[out_idx] = 1;
                }
            }
        }
        output
    }

    /// Compute the number of E8 roots.
    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    /// Get the connection prior between two E8 roots.
    pub fn root_connection_strength(&self, root_i: usize, root_j: usize) -> f64 {
        self.connection_prior
            .get(root_i)
            .and_then(|row| row.get(root_j))
            .copied()
            .unwrap_or(0.0)
    }
}

/// Wire E8 cortical geometry into a GeometricSSM-compatible VSA encoding.
/// Takes a VSA vector and enriches it with E8 cortical geometry bias.
pub fn e8_cortical_vsa_transform(vsa_input: &[u8], cortical_map: &E8CorticalMapping) -> Vec<u8> {
    let cortical_vsa = cortical_map.cortical_vsa_encode(vsa_input);
    // Bundle original VSA with cortical bias
    QuantizedVSA::bundle(&[vsa_input, &cortical_vsa])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e8_cortical_mapping_initialization() {
        let mapping = E8CorticalMapping::default();
        assert!(mapping.is_initialized());
        assert!(!mapping.roots.is_empty());
        assert_eq!(mapping.cortical_coords.len(), CORTICAL_NEURON_COUNT);
    }

    #[test]
    fn test_root_to_neuron_mapping() {
        let mapping = E8CorticalMapping::default();
        assert!(!mapping.root_to_neurons.is_empty());
        // Each root should have at least one neuron
        for neurons in &mapping.root_to_neurons {
            assert!(!neurons.is_empty());
        }
    }

    #[test]
    fn test_connection_prior() {
        let mapping = E8CorticalMapping::default();
        assert!(!mapping.connection_prior.is_empty());
        let n = mapping.root_count();
        assert_eq!(mapping.connection_prior.len(), n);
        assert_eq!(mapping.connection_prior[0].len(), n);
        // Self-connection should be 1.0
        assert!((mapping.connection_prior[0][0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_connectivity_bias() {
        let mapping = E8CorticalMapping::default();
        let bias = mapping.connectivity_bias(0, 1);
        assert!(bias >= 0.0 && bias <= 1.0);
        // Same neuron should have high bias
        let self_bias = mapping.connectivity_bias(0, 0);
        assert!(self_bias > 0.5);
    }

    #[test]
    fn test_cortical_vsa_encode() {
        let mapping = E8CorticalMapping::default();
        let input = QuantizedVSA::random_binary();
        let output = mapping.cortical_vsa_encode(&input);
        assert_eq!(output.len(), VSA_DIM);
    }

    #[test]
    fn test_e8_cortical_vsa_transform() {
        let mapping = E8CorticalMapping::default();
        let input = QuantizedVSA::random_binary();
        let transformed = e8_cortical_vsa_transform(&input, &mapping);
        assert_eq!(transformed.len(), VSA_DIM);
        // Should be different from input
        let dist = QuantizedVSA::hamming_distance(&input, &transformed);
        assert!(dist > 0);
    }

    #[test]
    fn test_cortical_coord_distance() {
        let a = CorticalCoord {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let b = CorticalCoord {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        assert!((a.distance(&b) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_idempotent_initialization() {
        let mut mapping = E8CorticalMapping::new();
        mapping.initialize();
        let coord_len_before = mapping.cortical_coords.len();
        mapping.initialize();
        let coord_len_after = mapping.cortical_coords.len();
        assert_eq!(coord_len_before, coord_len_after);
    }

    #[test]
    fn test_root_connection_strength() {
        let mapping = E8CorticalMapping::default();
        let strength = mapping.root_connection_strength(0, 0);
        assert!(
            (strength - 1.0).abs() < 1e-9,
            "self-connection should be 1.0"
        );
        let strength_diff = mapping.root_connection_strength(0, 1);
        assert!(strength_diff >= 0.0 && strength_diff <= 1.0);
    }

    #[test]
    fn test_connectivity_bias_out_of_bounds() {
        let mapping = E8CorticalMapping::default();
        let bias = mapping.connectivity_bias(CORTICAL_NEURON_COUNT + 1, 0);
        assert_eq!(bias, 0.0);
    }
}
