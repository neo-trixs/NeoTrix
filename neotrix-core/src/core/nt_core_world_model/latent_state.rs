/// A VSA-encoded latent state for the world model.
///
/// 4096-bit representation compatible with HyperCube VSA.
/// Stores the latent vector, an energy/uncertainty estimate,
/// and the number of refinement iterations used to reach this state.
#[derive(Debug, Clone)]
pub struct VsaLatentState {
    /// The 4096-bit VSA vector representing current world state (512 bytes)
    pub vector: Vec<u8>,
    /// Energy/uncertainty estimate
    pub energy: f64,
    /// Number of refinement iterations used to reach this state
    pub iterations_used: usize,
}

impl VsaLatentState {
    /// Create a zero-initialised latent state.
    pub fn empty(latent_dim: usize) -> Self {
        Self {
            vector: vec![0u8; latent_dim],
            energy: 1.0,
            iterations_used: 0,
        }
    }

    /// Cosine similarity between this state and another.
    ///
    /// Delegates to the HyperCube `cosine_sim_u8` primitive when dimension is 4096.
    pub fn cosine_similarity(&self, other: &VsaLatentState) -> f64 {
        if self.vector.len() != other.vector.len() {
            return 0.0;
        }
        if self.vector.len() == 4096 {
            crate::core::nt_core_hcube::cosine_sim_u8(&self.vector, &other.vector)
        } else {
            let dot: f64 = self
                .vector
                .iter()
                .zip(other.vector.iter())
                .map(|(a, b)| (*a as f64) * (*b as f64))
                .sum();
            let na: f64 = self.vector.iter().map(|x| (*x as f64) * (*x as f64)).sum();
            let nb: f64 = other.vector.iter().map(|x| (*x as f64) * (*x as f64)).sum();
            let norm = na.sqrt() * nb.sqrt();
            if norm < 1e-12 {
                0.0
            } else {
                dot / norm
            }
        }
    }

    /// Magnitude of change from a previous state (1 - cosine_similarity).
    pub fn delta(&self, other: &VsaLatentState) -> f64 {
        1.0 - self.cosine_similarity(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_state() {
        let s = VsaLatentState::empty(4096);
        assert_eq!(s.vector.len(), 4096);
        assert!(s.vector.iter().all(|&b| b == 0));
        assert_eq!(s.energy, 1.0);
        assert_eq!(s.iterations_used, 0);
    }

    #[test]
    fn test_self_similarity() {
        let s = VsaLatentState::empty(4096);
        let sim = s.cosine_similarity(&s);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_zero_delta() {
        let s = VsaLatentState::empty(4096);
        let d = s.delta(&s);
        assert!((d - 0.0).abs() < 1e-6);
    }
}
