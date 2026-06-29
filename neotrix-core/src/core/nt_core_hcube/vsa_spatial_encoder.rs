use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use std::collections::VecDeque;

/// 3D coordinate
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    pub fn origin() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
    pub fn distance(&self, other: &Vec3D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Multi-view depth fusion: integrates depth from multiple perspectives
#[derive(Debug, Clone)]
pub struct MultiViewDepthFusion {
    pub views: Vec<ViewDescriptor>,
    pub fused_depth: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct ViewDescriptor {
    pub origin: Vec3D,
    pub direction: Vec3D,
    pub depth_map: Vec<f64>,
    pub confidence: f64,
}

impl MultiViewDepthFusion {
    pub fn new() -> Self {
        Self {
            views: Vec::new(),
            fused_depth: Vec::new(),
        }
    }

    pub fn add_view(
        &mut self,
        origin: Vec3D,
        direction: Vec3D,
        depth_map: Vec<f64>,
        confidence: f64,
    ) {
        self.views.push(ViewDescriptor {
            origin,
            direction,
            depth_map,
            confidence,
        });
        self.fuse();
    }

    fn fuse(&mut self) {
        if self.views.is_empty() {
            return;
        }
        let n = self.views[0].depth_map.len();
        let mut depth_sum = vec![0.0; n];
        let mut weight_sum = vec![0.0; n];
        for view in &self.views {
            let w = view.confidence;
            for (d, s) in view.depth_map.iter().zip(depth_sum.iter_mut()) {
                *s += d * w;
            }
            for s in weight_sum.iter_mut() {
                *s += w;
            }
        }
        self.fused_depth = depth_sum
            .iter()
            .zip(weight_sum.iter())
            .map(|(d, w)| if *w > 0.0 { d / w } else { 0.0 })
            .collect();
    }

    pub fn fused_depth_map(&self) -> &[f64] {
        &self.fused_depth
    }
}

/// Ego state: position + orientation at a point in time
#[derive(Debug, Clone)]
pub struct EgoState {
    pub position: Vec3D,
    pub orientation: Vec3D,
    pub velocity: Vec3D,
    pub angular_velocity: Vec3D,
    pub timestamp: u64,
}

/// Historical ego state tracking
#[derive(Debug, Clone)]
pub struct EgoStateHistory {
    pub states: VecDeque<EgoState>,
    pub max_len: usize,
}

impl EgoStateHistory {
    pub fn new(max_len: usize) -> Self {
        Self {
            states: VecDeque::with_capacity(max_len),
            max_len,
        }
    }

    pub fn record(&mut self, state: EgoState) {
        if self.states.len() >= self.max_len {
            self.states.pop_front();
        }
        self.states.push_back(state);
    }

    pub fn recent(&self, n: usize) -> Vec<&EgoState> {
        self.states.iter().rev().take(n).collect()
    }

    pub fn current(&self) -> Option<&EgoState> {
        self.states.back()
    }
}

/// VSA-based 3D position encoder (SpaceDrive-inspired)
#[derive(Debug, Clone)]
pub struct VSASpatialEncoder {
    pub spatial_resolution: f64,
    pub n_frequencies: usize,
    seed_base: u64,
}

impl Default for VSASpatialEncoder {
    fn default() -> Self {
        Self {
            spatial_resolution: 0.01,
            n_frequencies: 32,
            seed_base: 0xE8,
        }
    }
}

impl VSASpatialEncoder {
    pub fn new(spatial_resolution: f64, n_frequencies: usize) -> Self {
        Self {
            spatial_resolution: spatial_resolution.max(0.001),
            n_frequencies: n_frequencies.max(4).min(256),
            seed_base: 0xE8,
        }
    }

    /// Encode a 3D coordinate to a VSA vector using Fourier feature encoding
    pub fn encode(&self, coord: &Vec3D) -> Vec<u8> {
        let mut vecs = Vec::with_capacity(self.n_frequencies * 3);
        for i in 0..self.n_frequencies {
            let freq = (i as f64 + 1.0) * self.spatial_resolution;
            let phase = freq * std::f64::consts::PI * 2.0;
            for (j, &val) in [coord.x, coord.y, coord.z].iter().enumerate() {
                let angle = val * phase;
                let sin_val = angle.sin();
                let cos_val = angle.cos();
                let seed = self
                    .seed_base
                    .wrapping_mul(i as u64 + 1)
                    .wrapping_mul(j as u64 + 1);
                let mut v = QuantizedVSA::seeded_random(seed, VSA_DIM / self.n_frequencies / 3);
                let scale = (sin_val.abs() + cos_val.abs()) / 2.0;
                for b in v.iter_mut() {
                    if fastrand::f64() > scale {
                        *b = 0;
                    }
                }
                vecs.push(v);
            }
        }
        let refs: Vec<&[u8]> = vecs.iter().map(|v| v.as_slice()).collect();
        QuantizedVSA::bundle(&refs)
    }

    /// Compute spatial similarity between two encoded positions
    pub fn spatial_similarity(&self, a: &Vec<u8>, b: &Vec<u8>) -> f64 {
        QuantizedVSA::cosine(a, b)
    }

    /// Encode a trajectory as a sequence of VSA vectors
    pub fn encode_trajectory(&self, coords: &[Vec3D]) -> Vec<Vec<u8>> {
        coords.iter().map(|c| self.encode(c)).collect()
    }
}

/// Spatial attention gate: scores spatial relevance
#[derive(Debug, Clone)]
pub struct SpatialAttentionGate {
    pub position_bias: f64,
    pub scale: f64,
}

impl Default for SpatialAttentionGate {
    fn default() -> Self {
        Self {
            position_bias: 0.0,
            scale: 1.0,
        }
    }
}

impl SpatialAttentionGate {
    pub fn new(position_bias: f64, scale: f64) -> Self {
        Self {
            position_bias,
            scale: scale.max(0.01),
        }
    }

    /// Compute spatial attention score between query and key positions
    pub fn attention_score(&self, query: &Vec<u8>, key: &Vec<u8>) -> f64 {
        let sim = QuantizedVSA::cosine(query, key);
        ((sim + self.position_bias) / self.scale).clamp(0.0, 1.0)
    }

    /// Apply spatial attention as a mask over values
    pub fn apply_attention(&self, query: &Vec<u8>, keys: &[Vec<u8>], values: &[f64]) -> Vec<f64> {
        let scores: Vec<f64> = keys
            .iter()
            .map(|k| self.attention_score(query, k))
            .collect();
        let total: f64 = scores.iter().sum();
        if total <= 0.0 {
            return vec![0.0; values.len()];
        }
        values
            .iter()
            .zip(scores.iter())
            .map(|(v, s)| v * s / total)
            .collect()
    }

    /// Gaussian spatial bias based on Euclidean distance
    pub fn spatial_bias(&self, distance: f64, sigma: f64) -> f64 {
        (-distance * distance / (2.0 * sigma * sigma)).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3d_distance() {
        let a = Vec3D::origin();
        let b = Vec3D::new(1.0, 0.0, 0.0);
        assert!((a.distance(&b) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_vsa_spatial_encoder_encode() {
        let encoder = VSASpatialEncoder::default();
        let coord = Vec3D::new(0.5, 0.3, 0.8);
        let encoded = encoder.encode(&coord);
        assert_eq!(encoded.len(), VSA_DIM);
    }

    #[test]
    fn test_vsa_spatial_encoder_deterministic() {
        let encoder = VSASpatialEncoder::default();
        let coord = Vec3D::new(0.5, 0.3, 0.8);
        let a = encoder.encode(&coord);
        let b = encoder.encode(&coord);
        assert_eq!(a, b, "encoding must be deterministic");
    }

    #[test]
    fn test_vsa_spatial_encoder_different_positions_differ() {
        let encoder = VSASpatialEncoder::default();
        let a = encoder.encode(&Vec3D::new(0.0, 0.0, 0.0));
        let b = encoder.encode(&Vec3D::new(1.0, 1.0, 1.0));
        let sim = encoder.spatial_similarity(&a, &b);
        assert!(
            sim < 0.9,
            "different positions should have lower similarity: {}",
            sim
        );
    }

    #[test]
    fn test_ego_state_history() {
        let mut history = EgoStateHistory::new(5);
        assert!(history.current().is_none());
        for i in 0..10 {
            history.record(EgoState {
                position: Vec3D::new(i as f64, 0.0, 0.0),
                orientation: Vec3D::origin(),
                velocity: Vec3D::origin(),
                angular_velocity: Vec3D::origin(),
                timestamp: i as u64,
            });
        }
        assert_eq!(history.states.len(), 5);
        assert!(history.current().is_some());
        let recent = history.recent(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_multi_view_depth_fusion() {
        let mut fusion = MultiViewDepthFusion::new();
        fusion.add_view(
            Vec3D::origin(),
            Vec3D::new(1.0, 0.0, 0.0),
            vec![1.0, 2.0, 3.0],
            0.9,
        );
        fusion.add_view(
            Vec3D::new(1.0, 0.0, 0.0),
            Vec3D::new(-1.0, 0.0, 0.0),
            vec![1.1, 2.1, 3.1],
            0.8,
        );
        let fused = fusion.fused_depth_map();
        assert_eq!(fused.len(), 3);
        assert!(fused[0] > 0.0);
    }

    #[test]
    fn test_spatial_attention_gate() {
        let gate = SpatialAttentionGate::default();
        let encoder = VSASpatialEncoder::default();
        let q = encoder.encode(&Vec3D::new(0.0, 0.0, 0.0));
        let k = encoder.encode(&Vec3D::new(0.1, 0.0, 0.0));
        let score = gate.attention_score(&q, &k);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_spatial_attention_apply() {
        let gate = SpatialAttentionGate::default();
        let encoder = VSASpatialEncoder::default();
        let q = encoder.encode(&Vec3D::new(0.0, 0.0, 0.0));
        let keys: Vec<Vec<u8>> = (0..5)
            .map(|i| encoder.encode(&Vec3D::new(i as f64 * 0.1, 0.0, 0.0)))
            .collect();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let attended = gate.apply_attention(&q, &keys, &values);
        assert_eq!(attended.len(), values.len());
    }

    #[test]
    fn test_spatial_bias() {
        let gate = SpatialAttentionGate::default();
        let bias = gate.spatial_bias(0.0, 1.0);
        assert!((bias - 1.0).abs() < 1e-9);
        let bias_far = gate.spatial_bias(10.0, 1.0);
        assert!(bias_far < 0.01);
    }
}
