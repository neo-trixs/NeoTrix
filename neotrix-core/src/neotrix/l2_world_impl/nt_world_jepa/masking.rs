use serde::{Deserialize, Serialize};

/// Masking strategy for JEPA training.
///
/// Applied to the E8 state patch sequence: selected patches are masked
/// (zeroed out) and the predictor must reconstruct the original.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaskingStrategy {
    /// Mask contiguous blocks of patches.
    /// `block_size`: size of each masked block
    /// `mask_ratio`: fraction of total patches to mask
    BlockMasking {
        block_size: usize,
        mask_ratio: f64,
    },
    /// Mask individual patches uniformly at random.
    /// `mask_ratio`: fraction of patches to mask
    RandomMasking {
        mask_ratio: f64,
    },
    /// No masking — standard prediction.
    None,
}

impl MaskingStrategy {
    /// Apply masking to a list of patches.
    ///
    /// Returns `(masked_patches, mask_indices)` where `masked_patches`
    /// has zeros for masked positions and `mask_indices` lists the indices
    /// that were masked.
    pub fn apply(&self, patches: &[Vec<f64>]) -> (Vec<Vec<f64>>, Vec<usize>) {
        match self {
            MaskingStrategy::BlockMasking { block_size, mask_ratio } => {
                Self::block_mask(patches, *block_size, *mask_ratio)
            }
            MaskingStrategy::RandomMasking { mask_ratio } => {
                Self::random_mask(patches, *mask_ratio)
            }
            MaskingStrategy::None => {
                (patches.to_vec(), Vec::new())
            }
        }
    }

    fn random_mask(patches: &[Vec<f64>], ratio: f64) -> (Vec<Vec<f64>>, Vec<usize>) {
        let n = patches.len();
        let num_mask = ((n as f64) * ratio.max(0.0).min(1.0)).round() as usize;
        let num_mask = num_mask.min(n);

        let mut indices: Vec<usize> = (0..n).collect();
        // Fisher-Yates partial shuffle using simple rand
        for i in (1..n).rev() {
            let j = (rand::random::<f64>() * (i + 1) as f64).floor() as usize;
            indices.swap(i, j.min(i));
        }
        let mask_idx: Vec<usize> = indices[..num_mask].to_vec();

        let mut masked = patches.to_vec();
        for &idx in &mask_idx {
            if idx < masked.len() {
                masked[idx] = vec![0.0; patches[0].len()];
            }
        }

        (masked, mask_idx)
    }

    fn block_mask(patches: &[Vec<f64>], block_size: usize, ratio: f64) -> (Vec<Vec<f64>>, Vec<usize>) {
        let n = patches.len();
        let total_to_mask = ((n as f64) * ratio.max(0.0).min(1.0)).round() as usize;
        let total_to_mask = total_to_mask.min(n);

        if total_to_mask == 0 || block_size == 0 {
            return (patches.to_vec(), Vec::new());
        }

        let block_size = block_size.min(n);
        let num_blocks = total_to_mask.div_ceil(block_size);
        let mut all_idx: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            let j = (rand::random::<f64>() * (i + 1) as f64).floor() as usize;
            all_idx.swap(i, j.min(i));
        }

        let mask_dim = if patches.is_empty() { 0 } else { patches[0].len() };
        let mut masked = patches.to_vec();
        let mut mask_idx_set: Vec<usize> = Vec::new();

        for b in 0..num_blocks {
            let start = all_idx[(b * block_size) % n];
            for offset in 0..block_size {
                let idx = (start + offset) % n;
                if !mask_idx_set.contains(&idx) && mask_idx_set.len() < total_to_mask {
                    masked[idx] = vec![0.0; mask_dim];
                    mask_idx_set.push(idx);
                }
            }
        }

        (masked, mask_idx_set)
    }

    /// Return the ratio of patches to mask for this strategy.
    pub fn mask_ratio(&self) -> f64 {
        match self {
            MaskingStrategy::BlockMasking { mask_ratio, .. } => *mask_ratio,
            MaskingStrategy::RandomMasking { mask_ratio } => *mask_ratio,
            MaskingStrategy::None => 0.0,
        }
    }

    /// Create a default block masking strategy (mask 30% of patches in blocks of 4).
    pub fn default_block() -> Self {
        MaskingStrategy::BlockMasking {
            block_size: 4,
            mask_ratio: 0.3,
        }
    }

    /// Create a default random masking strategy (mask 25% of patches).
    pub fn default_random() -> Self {
        MaskingStrategy::RandomMasking { mask_ratio: 0.25 }
    }
}

/// Tracking info for which patches were masked and their original values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskInfo {
    /// Indices of masked patches in the original sequence.
    pub masked_indices: Vec<usize>,
    /// Original unmasked values of masked patches.
    pub original_values: Vec<Vec<f64>>,
}

impl MaskInfo {
    pub fn new(masked_indices: Vec<usize>, original_values: Vec<Vec<f64>>) -> Self {
        Self { masked_indices, original_values }
    }

    pub fn num_masked(&self) -> usize {
        self.masked_indices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_patches(n: usize, dim: usize) -> Vec<Vec<f64>> {
        (0..n).map(|i| (0..dim).map(|d| (i * dim + d) as f64 / 100.0).collect()).collect()
    }

    #[test]
    fn test_none_masking() {
        let strategy = MaskingStrategy::None;
        let patches = sample_patches(8, 4);
        let (masked, indices) = strategy.apply(&patches);
        assert_eq!(masked.len(), 8);
        assert!(indices.is_empty());
        assert_eq!(masked, patches);
    }

    #[test]
    fn test_random_masking_zero_ratio() {
        let strategy = MaskingStrategy::RandomMasking { mask_ratio: 0.0 };
        let patches = sample_patches(8, 4);
        let (masked, indices) = strategy.apply(&patches);
        assert!(indices.is_empty());
        assert_eq!(masked, patches);
    }

    #[test]
    fn test_random_masking_masks_some() {
        let strategy = MaskingStrategy::RandomMasking { mask_ratio: 0.5 };
        let patches = sample_patches(8, 4);
        let (masked, indices) = strategy.apply(&patches);
        assert_eq!(indices.len(), 4);
        for &idx in &indices {
            assert!(masked[idx].iter().all(|&v| v == 0.0));
        }
    }

    #[test]
    fn test_block_masking_size() {
        let strategy = MaskingStrategy::BlockMasking { block_size: 3, mask_ratio: 0.5 };
        let patches = sample_patches(8, 4);
        let (masked, indices) = strategy.apply(&patches);
        assert!(!indices.is_empty());
        assert!(indices.len() <= 8);
        for &idx in &indices {
            assert!(masked[idx].iter().all(|&v| v == 0.0));
        }
    }

    #[test]
    fn test_mask_info() {
        let info = MaskInfo::new(vec![0, 2], vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        assert_eq!(info.num_masked(), 2);
    }

    #[test]
    fn test_mask_ratio_getter() {
        let s1 = MaskingStrategy::default_block();
        assert!((s1.mask_ratio() - 0.3).abs() < 1e-10);
        let s2 = MaskingStrategy::default_random();
        assert!((s2.mask_ratio() - 0.25).abs() < 1e-10);
        let s3 = MaskingStrategy::None;
        assert!((s3.mask_ratio() - 0.0).abs() < 1e-10);
    }
}
