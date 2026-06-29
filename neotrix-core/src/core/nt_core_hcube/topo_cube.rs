use crate::core::nt_core_hcube::vsa_quantized::{pack_binary, similarity_packed, VSA_DIM};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 3D spatial position for a topographic VSA cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopoPosition {
    pub x: f64,
    pub y: f64,
    pub layer: u8,
}

/// A single cell in the topographic VSA index.
/// Stores its 3D position, binary VSA vector, label, and access tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopoVsaCell {
    pub position: TopoPosition,
    pub vector: Vec<u8>,
    pub label: String,
    pub access_count: u64,
    pub layer: u8,
}

/// Topographic VSA spatial index — VSA vectors arranged on a 3D lattice.
/// Vectors near each other in (x, y, layer) space represent similar concepts.
/// Uses SOM-style attractor dynamics for spatial smoothness and a learned
/// diagonal causal inner product matrix for similarity queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopoCubeIndex {
    cells: HashMap<(u32, u32, u8), TopoVsaCell>,
    /// Learned diagonal causal inner product matrix M ∈ ℝ^(VSA_DIM)
    causal_diag: Vec<f64>,
    /// Learning rate for SOM-style updates
    learning_rate: f64,
    /// Neighborhood radius for SOM-style smoothing
    neighborhood_radius: f64,
}

impl Default for TopoCubeIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl TopoCubeIndex {
    /// Create an empty topographic index with identity causal matrix (all 1.0),
    /// learning rate 0.1, and neighborhood radius 5.0.
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            causal_diag: vec![1.0; VSA_DIM],
            learning_rate: 0.1,
            neighborhood_radius: 5.0,
        }
    }

    /// Insert or update a VSA vector at the given topographic position.
    ///
    /// 1. Rounds position to grid coordinates (x*10, y*10, layer).
    /// 2. If a cell already exists at that grid position and hamming
    ///    similarity > 0.8, nudges the cell's vector toward the new vector
    ///    with probability 0.3 / (1 + access_count) per differing bit.
    /// 3. Creates a new cell if none exists at that position.
    /// 4. Applies SOM-style neighborhood smoothing to all cells within
    ///    `neighborhood_radius` Euclidean distance in grid space.
    pub fn insert(&mut self, position: TopoPosition, vector: Vec<u8>, label: &str) {
        let layer = position.layer;
        let grid_x = (position.x * 10.0) as u32;
        let grid_y = (position.y * 10.0) as u32;
        let grid_key = (grid_x, grid_y, layer);

        // Create or update the cell at the target grid position
        if !self.cells.contains_key(&grid_key) {
            self.cells.insert(
                grid_key,
                TopoVsaCell {
                    position,
                    vector: vector.clone(),
                    label: label.to_string(),
                    access_count: 1,
                    layer,
                },
            );
        } else if let Some(cell) = self.cells.get_mut(&grid_key) {
            let sim = similarity_packed(&pack_binary(&cell.vector), &pack_binary(&vector));
            if sim > 0.8 {
                let flip_prob = 0.3 / (1.0 + cell.access_count as f64);
                som_nudge(cell, &vector, flip_prob);
            }
            cell.access_count += 1;
        }

        // SOM-style neighborhood update: nudge all cells within radius
        // toward the new vector with distance-decayed learning rate.
        for ((cx, cy, cl), cell) in self.cells.iter_mut() {
            if *cl != layer {
                continue;
            }
            let dx = *cx as f64 - grid_x as f64;
            let dy = *cy as f64 - grid_y as f64;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= self.neighborhood_radius {
                let decay = (-dist / self.neighborhood_radius).exp();
                let neighbor_lr = self.learning_rate * decay;
                som_nudge(cell, &vector, neighbor_lr);
            }
        }
    }

    /// Find the top_k most similar cells using the causal inner product.
    /// Returns cells sorted by descending causal similarity.
    pub fn query(&self, vector: &[u8], top_k: usize) -> Vec<&TopoVsaCell> {
        let mut scored: Vec<(&TopoVsaCell, f64)> = self
            .cells
            .values()
            .map(|cell| {
                let sim = causal_similarity(vector, &cell.vector, &self.causal_diag);
                (cell, sim)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored.into_iter().map(|(cell, _)| cell).collect()
    }

    /// Spatial range query: return all cells whose (x, y, layer) falls
    /// within the given bounds. Optional layer filter.
    pub fn query_by_region(
        &self,
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
        layer: Option<u8>,
    ) -> Vec<&TopoVsaCell> {
        let min_gx = (min_x * 10.0) as u32;
        let max_gx = (max_x * 10.0) as u32;
        let min_gy = (min_y * 10.0) as u32;
        let max_gy = (max_y * 10.0) as u32;
        self.cells
            .values()
            .filter(|cell| {
                let gx = (cell.position.x * 10.0) as u32;
                let gy = (cell.position.y * 10.0) as u32;
                gx >= min_gx
                    && gx <= max_gx
                    && gy >= min_gy
                    && gy <= max_gy
                    && layer.map_or(true, |l| l == cell.layer)
            })
            .collect()
    }

    /// Online training of the diagonal causal matrix M.
    /// For each (query_vector, target_vector, target_score) pair,
    /// updates each diagonal element via gradient descent:
    ///   M_ii += 0.01 * (score - M_ii * q_i * t_i) * q_i * t_i
    pub fn train_causal_matrix(&mut self, pairs: &[(Vec<u8>, Vec<u8>, f64)]) {
        let dim = VSA_DIM.min(self.causal_diag.len());
        for (query_vec, target_vec, target_score) in pairs {
            let limit = dim.min(query_vec.len()).min(target_vec.len());
            for i in 0..limit {
                let qv = query_vec[i] as f64;
                let tv = target_vec[i] as f64;
                let prediction = self.causal_diag[i] * qv * tv;
                self.causal_diag[i] += 0.01 * (target_score - prediction) * qv * tv;
            }
        }
    }

    /// Return the top n cells sorted by descending access count.
    pub fn cells_sorted_by_access(&self, n: usize) -> Vec<&TopoVsaCell> {
        let mut sorted: Vec<&TopoVsaCell> = self.cells.values().collect();
        sorted.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        sorted.truncate(n);
        sorted
    }
}

/// Causal inner product similarity: aᵀ M b / (|a| · |b|)
/// where M is a learned diagonal matrix. Operates on raw binary VSA vectors
/// (each element is 0 or 1) with f64 floating math.
pub fn causal_similarity(a: &[u8], b: &[u8], diag: &[f64]) -> f64 {
    let len = a.len().min(b.len()).min(diag.len()).min(VSA_DIM);
    if len == 0 {
        return 0.0;
    }
    let mut dot = 0.0f64;
    let mut mag_a = 0.0f64;
    let mut mag_b = 0.0f64;
    for i in 0..len {
        let va = a[i] as f64;
        let vb = b[i] as f64;
        dot += diag[i] * va * vb;
        mag_a += va * va;
        mag_b += vb * vb;
    }
    let denom = (mag_a.sqrt() * mag_b.sqrt()).max(1e-12);
    dot / denom
}

/// SOM-style nudge: for each bit position where the cell's vector differs
/// from the target, flip the bit with probability `lr`.
fn som_nudge(cell: &mut TopoVsaCell, target: &[u8], lr: f64) {
    let mut rng = rand::thread_rng();
    let len = cell.vector.len().min(target.len()).min(VSA_DIM);
    for i in 0..len {
        if cell.vector[i] != target[i] {
            if rng.gen_bool(lr) {
                cell.vector[i] = target[i];
            }
        }
    }
}
