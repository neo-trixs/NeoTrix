/// Precomputed E8 root system — 240 non-zero roots in 8D.
/// Each root is `[i8; 8]` with values in {-2,-1,0,1,2}.
pub struct E8RootSystem {
    pub roots: Vec<[i8; 8]>,
    pub norms: Vec<f32>,
}

impl E8RootSystem {
    pub fn new() -> Self {
        let mut roots = Vec::with_capacity(240);
        // Type I: (±2, ±2, 0^6) permutations — 112 roots
        for i in 0..8 {
            for j in (i + 1)..8 {
                for &s1 in &[-1i8, 1] {
                    for &s2 in &[-1i8, 1] {
                        let mut v = [0i8; 8];
                        v[i] = 2 * s1;
                        v[j] = 2 * s2;
                        roots.push(v);
                    }
                }
            }
        }
        // Type II: (±1, ..., ±1) with even number of minus signs — 128 roots
        for mask in 0u8..=255 {
            let ones: u32 = mask.count_ones();
            if ones.is_multiple_of(2) {
                let mut v = [0i8; 8];
                for k in 0..8 {
                    v[k] = if (mask >> k) & 1 == 1 { -1 } else { 1 };
                }
                roots.push(v);
            }
        }
        debug_assert_eq!(roots.len(), 240, "E8 must have exactly 240 roots");
        let norms: Vec<f32> = roots
            .iter()
            .map(|r| (r.iter().map(|&x| (x * x) as f32).sum::<f32>()).sqrt())
            .collect();
        E8RootSystem { roots, norms }
    }

    pub fn root(&self, i: usize) -> &[i8; 8] {
        &self.roots[i % 240]
    }

    pub fn dot_prod_i8(a: &[i8; 8], b: &[i8; 8]) -> i16 {
        a.iter()
            .zip(b.iter())
            .map(|(&x, &y)| (x as i16) * (y as i16))
            .sum()
    }

    pub fn nearest_root(&self, state: &[f32; 8]) -> (usize, f32) {
        let mut best = 0usize;
        let mut best_dist = f32::INFINITY;
        for (i, root) in self.roots.iter().enumerate() {
            let dist: f32 = root
                .iter()
                .zip(state.iter())
                .map(|(&r, &s)| {
                    let d = r as f32 - s;
                    d * d
                })
                .sum();
            if dist < best_dist {
                best_dist = dist;
                best = i;
            }
        }
        (best, best_dist)
    }
}

impl Default for E8RootSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// E8 Lattice Quantizer — soft quantization of hidden states into 240 E8 roots.
/// Implements the Sovereign-Lila-E8 approach with straight-through estimator.
pub struct E8LatticeQuantizer {
    pub root_system: E8RootSystem,
    pub scale: f32,
}

impl E8LatticeQuantizer {
    pub fn new(scale: f32) -> Self {
        E8LatticeQuantizer {
            root_system: E8RootSystem::new(),
            scale,
        }
    }

    pub fn quantize(&self, hidden_state: &[f32; 8]) -> ([f32; 8], usize, f32) {
        let (nearest, dist_sq) = self.root_system.nearest_root(hidden_state);
        let root = self.root_system.root(nearest);
        let quantized = [
            root[0] as f32 * self.scale,
            root[1] as f32 * self.scale,
            root[2] as f32 * self.scale,
            root[3] as f32 * self.scale,
            root[4] as f32 * self.scale,
            root[5] as f32 * self.scale,
            root[6] as f32 * self.scale,
            root[7] as f32 * self.scale,
        ];
        (quantized, nearest, dist_sq)
    }

    pub fn quantize_with_ste(&self, hidden_state: &[f32; 8]) -> ([f32; 8], usize, f32) {
        let (quantized, root_idx, dist_sq) = self.quantize(hidden_state);
        let ste = [
            hidden_state[0] + (quantized[0] - hidden_state[0]),
            hidden_state[1] + (quantized[1] - hidden_state[1]),
            hidden_state[2] + (quantized[2] - hidden_state[2]),
            hidden_state[3] + (quantized[3] - hidden_state[3]),
            hidden_state[4] + (quantized[4] - hidden_state[4]),
            hidden_state[5] + (quantized[5] - hidden_state[5]),
            hidden_state[6] + (quantized[6] - hidden_state[6]),
            hidden_state[7] + (quantized[7] - hidden_state[7]),
        ];
        (ste, root_idx, dist_sq)
    }

    pub fn geometric_attention_bias(&self, query: &[f32; 8], keys: &[[f32; 8]]) -> Vec<f32> {
        let (q_nearest, _, _) = self.quantize(query);
        keys.iter()
            .map(|key| {
                let (k_nearest, _, _) = self.quantize(key);
                let bias: f32 = q_nearest
                    .iter()
                    .zip(k_nearest.iter())
                    .map(|(&q, &k)| q * k)
                    .sum();
                bias * self.scale
            })
            .collect()
    }

    pub fn root_distance_matrix(&self) -> [[f32; 240]; 240] {
        let mut mat = [[0.0f32; 240]; 240];
        for i in 0..240 {
            let ri = self.root_system.root(i);
            for j in 0..240 {
                let rj = self.root_system.root(j);
                let dist: f32 = ri
                    .iter()
                    .zip(rj.iter())
                    .map(|(&a, &b)| {
                        let d = a as f32 - b as f32;
                        d * d
                    })
                    .sum();
                mat[i][j] = dist.sqrt();
            }
        }
        mat
    }
}

impl Default for E8LatticeQuantizer {
    fn default() -> Self {
        Self::new(1.0)
    }
}

/// Map an E8 root index (0-239) to a hexagram state (0-63).
pub fn root_to_hexagram(root_idx: usize) -> u8 {
    let root_system = E8RootSystem::new();
    let root = root_system.root(root_idx);
    let mut hex: u8 = 0;
    for i in 0..6 {
        if root[i] != 0 || root[i + 2] != 0 {
            hex |= 1 << (5 - i);
        }
    }
    hex % 64
}

/// Geometric coherence: measure alignment between a trajectory and E8 lattice.
pub fn geometric_coherence(trajectory: &[u8], quantizer: &E8LatticeQuantizer) -> f32 {
    if trajectory.len() < 2 {
        return 1.0;
    }
    let mut transitions = 0u32;
    let mut aligned = 0u32;
    for pair in trajectory.windows(2) {
        let from = pair[0];
        let to = pair[1];
        let fx = (from as f32 / 64.0) * 2.0 - 1.0;
        let tx = (to as f32 / 64.0) * 2.0 - 1.0;
        let fh = [fx, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let th = [tx, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let (_, _, d1) = quantizer.quantize(&fh);
        let (_, _, d2) = quantizer.quantize(&th);
        if (d1.sqrt() - d2.sqrt()).abs() < 0.5 {
            aligned += 1;
        }
        transitions += 1;
    }
    aligned as f32 / transitions as f32
}

/// E8-LoRA style: frozen geometric core with learnable scaling per layer.
pub struct E8LoraScale {
    pub layer_scales: Vec<f32>,
}

impl E8LoraScale {
    pub fn new(num_layers: usize, initial: f32) -> Self {
        E8LoraScale {
            layer_scales: vec![initial; num_layers],
        }
    }

    pub fn apply(&self, layer: usize, base_logits: &[f32]) -> Vec<f32> {
        if self.layer_scales.is_empty() {
            return base_logits.to_vec();
        }
        let scale = self.layer_scales[layer % self.layer_scales.len()];
        let root_bias = self.compute_root_bias(base_logits);
        base_logits
            .iter()
            .zip(root_bias.iter())
            .map(|(&logit, &bias)| logit + scale * bias)
            .collect()
    }

    fn compute_root_bias(&self, _logits: &[f32]) -> Vec<f32> {
        let rs = E8RootSystem::new();
        let n = _logits.len();
        (0..n)
            .map(|i| {
                let ri = rs.root(i % 240);
                let mut sum = 0.0f32;
                for j in 0..n.min(8) {
                    let rj = rs.root(j % 240);
                    sum += ri
                        .iter()
                        .zip(rj.iter())
                        .map(|(&a, &b)| (a as f32) * (b as f32))
                        .sum::<f32>();
                }
                sum / n.max(1) as f32
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e8_root_system_240() {
        let rs = E8RootSystem::new();
        assert_eq!(rs.roots.len(), 240);
        assert_eq!(rs.norms.len(), 240);
    }

    #[test]
    fn test_e8_root_norms() {
        let rs = E8RootSystem::new();
        for (&r, &_n) in rs.roots.iter().zip(rs.norms.iter()) {
            let sq_norm: i16 = r.iter().map(|&x| (x * x) as i16).sum();
            assert!(
                sq_norm == 4 || sq_norm == 8,
                "E8 root norm must be 4 (type II) or 8 (type I), got {} for {:?}",
                sq_norm,
                r
            );
        }
    }

    #[test]
    fn test_nearest_root_origin() {
        let quantizer = E8LatticeQuantizer::new(1.0);
        let origin = [0.0f32; 8];
        let (quantized, idx, dist) = quantizer.quantize(&origin);
        assert!(dist >= 0.0);
        assert!(idx < 240);
        let norm_sq: f32 = quantized.iter().map(|x| x * x).sum();
        assert!(norm_sq > 0.0, "nearest root to origin must be non-zero");
    }

    #[test]
    fn test_ste_preserves_gradient() {
        let quantizer = E8LatticeQuantizer::new(0.5);
        let state = [0.3, -0.2, 0.1, -0.4, 0.05, 0.0, -0.1, 0.2];
        let (ste_vec, _, _) = quantizer.quantize_with_ste(&state);
        assert_eq!(ste_vec.len(), 8);
        assert!((ste_vec[0] - state[0]).abs() < 2.0);
    }

    #[test]
    fn test_geometric_attention_bias() {
        let quantizer = E8LatticeQuantizer::new(1.0);
        let query = [0.5f32; 8];
        let keys = vec![[0.3f32; 8], [-0.2f32; 8]];
        let biases = quantizer.geometric_attention_bias(&query, &keys);
        assert_eq!(biases.len(), 2);
        assert!(biases[0] != 0.0 || biases[1] != 0.0);
    }

    #[test]
    fn test_root_distance_matrix_symmetric() {
        let quantizer = E8LatticeQuantizer::new(1.0);
        let mat = quantizer.root_distance_matrix();
        for i in 0..240 {
            for j in 0..240 {
                assert!(
                    (mat[i][j] - mat[j][i]).abs() < 1e-6,
                    "distance matrix must be symmetric at {},{}",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn test_geometric_coherence() {
        let quantizer = E8LatticeQuantizer::new(1.0);
        let coherent = vec![0, 8, 16, 24, 32, 40, 48, 56];
        let c = geometric_coherence(&coherent, &quantizer);
        assert!(c >= 0.0 && c <= 1.0);
    }

    #[test]
    fn test_e8_lora_scale() {
        let lora = E8LoraScale::new(4, 0.05);
        let logits = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let scaled = lora.apply(0, &logits);
        assert_eq!(scaled.len(), logits.len());
    }

    #[test]
    fn test_root_to_hexagram_mapping() {
        for i in 0..240 {
            let h = root_to_hexagram(i);
            assert!(h < 64, "hexagram must be in 0..63, got {}", h);
        }
    }
}
