use serde::{Deserialize, Serialize};

/// ObjectPartition — organizes state dimensions into discrete objects.
///
/// Each object is a contiguous set of dimension indices representing
/// a conceptually distinct entity within the state space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPartition {
    pub num_objects: usize,
    pub objects: Vec<Vec<usize>>,
}

impl ObjectPartition {
    /// Create a new partition with uniform object sizes.
    /// Dimensions are divided as evenly as possible.
    pub fn new(num_objects: usize, state_dim: usize) -> Self {
        let objects = Self::uniform_partition(num_objects, state_dim);
        let n = objects.len();
        Self { num_objects: n, objects }
    }

    /// Construct an adaptive partition from state variance.
    /// Dimensions are grouped into objects where variance differences
    /// between adjacent dimensions are largest.
    pub fn adaptive(state: &[f64], max_objects: usize) -> Self {
        AdaptivePartitioner::adaptive_partition(state, max_objects)
    }

    fn uniform_partition(num_objects: usize, state_dim: usize) -> Vec<Vec<usize>> {
        if num_objects == 0 || state_dim == 0 {
            return Vec::new();
        }
        let n = num_objects.min(state_dim);
        let base = state_dim / n;
        let rem = state_dim % n;
        let mut objects = Vec::with_capacity(n);
        let mut idx = 0;
        for i in 0..n {
            let size = base + if i < rem { 1 } else { 0 };
            objects.push((idx..idx + size).collect());
            idx += size;
        }
        objects
    }

    /// Get the dimension indices for a given object.
    pub fn get(&self, obj_id: usize) -> &[usize] {
        if obj_id < self.objects.len() {
            &self.objects[obj_id]
        } else {
            &[]
        }
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// Total number of dimensions covered across all objects.
    pub fn total_dimensions(&self) -> usize {
        self.objects.iter().map(|o| o.len()).sum()
    }
}

/// AdaptivePartitioner — determines optimal partition boundaries from state variance.
///
/// Strategy: compute variance per dimension, find boundaries where adjacent
/// dimension variances differ most, and split at those boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptivePartitioner;

impl AdaptivePartitioner {
    /// Compute variance per dimension of a state vector.
    pub fn dimension_variances(state: &[f64]) -> Vec<f64> {
        if state.is_empty() {
            return Vec::new();
        }
        let n = state.len() as f64;
        let mean = state.iter().sum::<f64>() / n;
        state.iter().map(|v| (v - mean).powi(2)).collect()
    }

    /// Split state dimensions into objects where variance differences are largest.
    ///
    /// 1. Compute variance for each dimension.
    /// 2. Find adjacent gaps: |var[i] - var[i-1]| for i = 1..state.len().
    /// 3. Cut at the (max_objects - 1) largest gaps.
    /// 4. Fall back to uniform partitioning if all variances are equal.
    pub fn adaptive_partition(state: &[f64], max_objects: usize) -> ObjectPartition {
        let state_dim = state.len();
        if state_dim == 0 || max_objects == 0 {
            return ObjectPartition { num_objects: 0, objects: Vec::new() };
        }
        if max_objects >= state_dim {
            let objects: Vec<Vec<usize>> = (0..state_dim).map(|i| vec![i]).collect();
            return ObjectPartition { num_objects: objects.len(), objects };
        }

        let variances = Self::dimension_variances(state);

        // Check if all variances are equal (within epsilon)
        let all_equal = if variances.is_empty() {
            true
        } else {
            let first = variances[0];
            variances.iter().all(|v| (*v - first).abs() < 1e-12)
        };
        if all_equal {
            return ObjectPartition::new(max_objects, state_dim);
        }

        // Compute boundary gaps: absolute difference in variance between adjacent dimensions
        let mut gaps: Vec<(usize, f64)> = (1..state_dim)
            .map(|i| (i, (variances[i] - variances[i - 1]).abs()))
            .collect();

        // Sort by gap descending
        gaps.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Pick top (max_objects - 1) cut points (unique, in order)
        let n_cuts = (max_objects - 1).min(gaps.len());
        let mut cut_points: Vec<usize> = gaps.iter().take(n_cuts).map(|(i, _)| *i).collect();
        cut_points.sort_unstable();
        cut_points.dedup();

        // Build objects from cut points
        let mut objects = Vec::with_capacity(cut_points.len() + 1);
        let mut start = 0;
        for &cut in &cut_points {
            if cut > start {
                objects.push((start..cut).collect());
            }
            start = cut;
        }
        if start < state_dim {
            objects.push((start..state_dim).collect());
        }

        if objects.is_empty() {
            return ObjectPartition::new(max_objects, state_dim);
        }

        let num_objects = objects.len();
        ObjectPartition { num_objects, objects }
    }

    /// Decide if repartitioning is needed based on prediction error and partition health.
    ///
    /// Returns true if:
    ///   - prediction error exceeds threshold, or
    ///   - the partition is empty or corrupted
    pub fn repartition_strategy(
        state: &[f64],
        current_partition: &ObjectPartition,
        prediction_error: f64,
        error_threshold: f64,
    ) -> bool {
        if prediction_error > error_threshold {
            return true;
        }
        if current_partition.num_objects == 0 || current_partition.objects.is_empty() {
            return true;
        }
        if current_partition.total_dimensions() != state.len() {
            return true;
        }
        false
    }
}

// ============================================================
// Adaptive ObjectPartition — WM-05
//
// Replaces uniform object partitioning with dynamic, adaptive
// partition sizes based on the information content of each region.
//
// Algorithm:
//   1. Start with uniform base tiles of `base_tile_size`
//   2. For each tile, compute activity = variance of elements
//   3. Split high-activity tiles (activity > threshold, size > min)
//   4. Merge low-activity neighbors (activity < threshold/2, merged ≤ max)
//   5. Repeat until convergence (max 3 iterations)
// ============================================================

/// AdaptivePartition — a single tile in an adaptive partition of latent space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptivePartition {
    /// Start index in latent space (inclusive)
    pub start: usize,
    /// End index in latent space (exclusive)
    pub end: usize,
    /// This tile's size (end - start)
    pub size: usize,
    /// Variance/energy within this tile
    pub activity: f64,
    /// Whether this tile is at minimum size (cannot be split further)
    pub is_leaf: bool,
}

/// AdaptiveObjectPartition — dynamically partitions latent space based on information density.
///
/// Core design:
///   - base_tile_size: initial uniform tile size (default 16)
///   - min_tile_size: minimum tile granularity (default 4)
///   - max_tile_size: maximum tile size (default 64)
///   - activity_threshold: variance threshold for splitting (default 0.1)
///
/// Integration: call `partition()` to produce adaptive tiles from a latent vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveObjectPartition {
    pub base_tile_size: usize,
    pub min_tile_size: usize,
    pub max_tile_size: usize,
    pub activity_threshold: f64,
    pub partitions: Vec<AdaptivePartition>,
}

impl AdaptiveObjectPartition {
    /// Create a new AdaptiveObjectPartition.
    pub fn new(
        base_tile_size: usize,
        min_tile_size: usize,
        max_tile_size: usize,
        activity_threshold: f64,
    ) -> Self {
        Self {
            base_tile_size,
            min_tile_size,
            max_tile_size,
            activity_threshold,
            partitions: Vec::new(),
        }
    }

    /// Create with sensible defaults (base=16, min=4, max=64, threshold=0.1)
    pub fn default() -> Self {
        Self::new(16, 4, 64, 0.1)
    }

    /// Builder constructor: enable adaptive partitioning with given threshold.
    pub fn with_adaptive_partitioning(threshold: f64) -> Self {
        Self::new(16, 4, 64, threshold)
    }

    /// Compute activity (population variance) of a tile region.
    ///
    /// Edge cases:
    ///   - Empty tile → 0.0
    ///   - Single element → 0.0
    ///   - Uniform values → 0.0
    pub fn activity_score(tile: &[f64]) -> f64 {
        if tile.len() <= 1 {
            return 0.0;
        }
        let n = tile.len() as f64;
        let mean = tile.iter().sum::<f64>() / n;
        tile.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n
    }

    /// Partition a latent vector into adaptive tiles.
    ///
    /// Algorithm (max 3 iterations of split+merge):
    ///   1. Uniform base tiles of `base_tile_size`
    ///   2. Split: activity > threshold AND size > min_tile_size → halve
    ///   3. Merge: activity < threshold/2 AND neighbor also low AND combined ≤ max_tile_size → combine
    ///   4. Repeat until convergence
    ///
    /// Edge cases:
    ///   - Empty latent → empty partitions
    ///   - Latent < min_tile_size → single partition
    ///   - Uniform values → all merged to max_size
    ///   - All high activity → all split to min_size
    pub fn partition(&mut self, latent: &[f64]) -> &[AdaptivePartition] {
        self.partitions = self.compute_partition(latent);
        &self.partitions
    }

    fn compute_partition(&self, latent: &[f64]) -> Vec<AdaptivePartition> {
        let len = latent.len();
        if len == 0 {
            return Vec::new();
        }

        if len <= self.min_tile_size {
            return vec![AdaptivePartition {
                start: 0,
                end: len,
                size: len,
                activity: Self::activity_score(latent),
                is_leaf: true,
            }];
        }

        let mut tiles = self.initial_tiles(latent);

        for _ in 0..3 {
            let prev_len = tiles.len();
            tiles = self.split_pass(latent, &tiles);
            tiles = self.merge_pass(latent, &tiles);
            if tiles.len() == prev_len {
                break;
            }
        }

        tiles
    }

    fn initial_tiles(&self, latent: &[f64]) -> Vec<AdaptivePartition> {
        let len = latent.len();
        let mut tiles = Vec::new();
        let mut start = 0;
        while start < len {
            let end = (start + self.base_tile_size).min(len);
            let size = end - start;
            let activity = Self::activity_score(&latent[start..end]);
            let is_leaf = size <= self.min_tile_size || size < 2;
            tiles.push(AdaptivePartition { start, end, size, activity, is_leaf });
            start = end;
        }
        tiles
    }

    fn split_pass(&self, latent: &[f64], tiles: &[AdaptivePartition]) -> Vec<AdaptivePartition> {
        let mut result = Vec::with_capacity(tiles.len());
        for tile in tiles {
            if tile.activity > self.activity_threshold
                && tile.size > self.min_tile_size
                && tile.size >= 2
            {
                let half = tile.size / 2;
                let mid = tile.start + half;

                let left_activity = Self::activity_score(&latent[tile.start..mid]);
                let right_activity = Self::activity_score(&latent[mid..tile.end]);

                result.push(AdaptivePartition {
                    start: tile.start,
                    end: mid,
                    size: half,
                    activity: left_activity,
                    is_leaf: half <= self.min_tile_size,
                });
                result.push(AdaptivePartition {
                    start: mid,
                    end: tile.end,
                    size: tile.size - half,
                    activity: right_activity,
                    is_leaf: (tile.size - half) <= self.min_tile_size,
                });
            } else {
                result.push(tile.clone());
            }
        }
        result
    }

    fn merge_pass(&self, latent: &[f64], tiles: &[AdaptivePartition]) -> Vec<AdaptivePartition> {
        if tiles.is_empty() {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(tiles.len());
        let mut i = 0;
        while i < tiles.len() {
            let mut merged = tiles[i].clone();
            while i + 1 < tiles.len() {
                let next = &tiles[i + 1];
                let combined_size = merged.size + next.size;
                if merged.activity < self.activity_threshold / 2.0
                    && next.activity < self.activity_threshold / 2.0
                    && combined_size <= self.max_tile_size
                {
                    let combined_activity =
                        Self::activity_score(&latent[merged.start..next.end]);
                    merged = AdaptivePartition {
                        start: merged.start,
                        end: next.end,
                        size: combined_size,
                        activity: combined_activity,
                        is_leaf: false,
                    };
                    i += 1;
                } else {
                    break;
                }
            }
            result.push(merged);
            i += 1;
        }
        result
    }

    /// Number of partitions in the current partitioning.
    pub fn num_partitions(&self) -> usize {
        self.partitions.len()
    }

    /// Human-readable summary: "N tiles: F fine + M medium + C coarse"
    pub fn partition_summary(&self) -> String {
        if self.partitions.is_empty() {
            return "0 tiles".to_string();
        }
        let fine_threshold = self.min_tile_size * 2;
        let coarse_threshold = self.max_tile_size / 2;
        let mut fine = 0usize;
        let mut medium = 0usize;
        let mut coarse = 0usize;
        for p in &self.partitions {
            if p.size <= fine_threshold {
                fine += 1;
            } else if p.size >= coarse_threshold {
                coarse += 1;
            } else {
                medium += 1;
            }
        }
        format!(
            "{} tiles: {} fine + {} medium + {} coarse",
            self.partitions.len(),
            fine,
            medium,
            coarse
        )
    }
}

/// Convenience constructor on ObjectPartition for adaptive partitioning.
impl ObjectPartition {
    /// Create an AdaptiveObjectPartition from a latent vector using adaptive partitioning.
    ///
    /// If `threshold` is `None`, uses the default (0.1).
    /// Returns the AdaptiveObjectPartition with computed partitions.
    pub fn adaptive_v2(latent: &[f64], threshold: Option<f64>) -> AdaptiveObjectPartition {
        let mut aop = AdaptiveObjectPartition::with_adaptive_partitioning(threshold.unwrap_or(0.1));
        aop.partition(latent);
        aop
    }
}

// ============================================================
// Tests
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_state() -> Vec<f64> {
        vec![
            0.1, 0.12, 0.09, // low variance region
            0.9, 0.05, 0.88, // high variance region
            0.3, 0.28, 0.32, // low variance region
        ]
    }

    fn constant_state() -> Vec<f64> {
        vec![0.5, 0.5, 0.5, 0.5, 0.5]
    }

    #[test]
    fn test_uniform_partition_basic() {
        let p = ObjectPartition::new(3, 9);
        assert_eq!(p.num_objects, 3);
        assert_eq!(p.objects.len(), 3);
        assert_eq!(p.total_dimensions(), 9);
    }

    #[test]
    fn test_adaptive_partition_dimensions() {
        let state = sample_state();
        let p = ObjectPartition::adaptive(&state, 3);
        assert!(p.num_objects >= 1);
        assert!(p.num_objects <= 3);
        assert_eq!(p.total_dimensions(), state.len());
    }

    #[test]
    fn test_repartition_trigger_high_error() {
        let state = sample_state();
        let p = ObjectPartition::adaptive(&state, 3);
        assert!(AdaptivePartitioner::repartition_strategy(&state, &p, 0.9, 0.5));
    }

    #[test]
    fn test_repartition_no_trigger_low_error() {
        let state = sample_state();
        let p = ObjectPartition::adaptive(&state, 3);
        assert!(!AdaptivePartitioner::repartition_strategy(&state, &p, 0.1, 0.5));
    }

    #[test]
    fn test_variance_based_splitting() {
        // State with clear variance boundary at index 3
        let state = vec![0.1, 0.12, 0.11, 0.9, 0.05, 0.88];
        let p = AdaptivePartitioner::adaptive_partition(&state, 2);
        // Should split near the variance boundary
        assert!(p.num_objects >= 1);
        assert_eq!(p.total_dimensions(), state.len());
    }

    #[test]
    fn test_fallback_to_uniform_constant_state() {
        let state = constant_state();
        let p = AdaptivePartitioner::adaptive_partition(&state, 3);
        // Constant state → uniform fallback
        assert_eq!(p.total_dimensions(), state.len());
        assert!(p.num_objects <= 3);
    }

    #[test]
    fn test_max_objects_respected() {
        let state = (0..20).map(|i| (i as f64) * 0.1).collect::<Vec<f64>>();
        let p = AdaptivePartitioner::adaptive_partition(&state, 5);
        assert!(p.num_objects <= 5);
        assert_eq!(p.total_dimensions(), state.len());
    }

    #[test]
    fn test_empty_state_handling() {
        let state: Vec<f64> = Vec::new();
        let p = AdaptivePartitioner::adaptive_partition(&state, 3);
        assert_eq!(p.num_objects, 0);
        assert!(p.is_empty());
    }

    #[test]
    fn test_consistent_across_calls() {
        let state = sample_state();
        let p1 = AdaptivePartitioner::adaptive_partition(&state, 3);
        let p2 = AdaptivePartitioner::adaptive_partition(&state, 3);
        assert_eq!(p1.num_objects, p2.num_objects);
        assert_eq!(p1.total_dimensions(), p2.total_dimensions());
        for (o1, o2) in p1.objects.iter().zip(p2.objects.iter()) {
            assert_eq!(o1.len(), o2.len());
        }
    }

    // ============================================================
    // AdaptiveObjectPartition tests (WM-05)
    // ============================================================

    /// Helper: uniform latent (all same value) → activity = 0
    fn uniform_latent(size: usize) -> Vec<f64> {
        vec![0.5; size]
    }

    /// Helper: high-activity latent (alternating high/low) → activity > 0.1
    fn high_activity_latent(size: usize) -> Vec<f64> {
        (0..size).map(|i| if i % 2 == 0 { 10.0 } else { -10.0 }).collect()
    }

    /// Helper: mixed activity — first half uniform, second half alternating
    fn mixed_latent(size: usize) -> Vec<f64> {
        let mut v = vec![0.5; size];
        for i in size / 2..size {
            v[i] = if (i - size / 2) % 2 == 0 { 10.0 } else { -10.0 };
        }
        v
    }

    /// Helper: increasing ramp — moderate activity
    fn ramp_latent(size: usize) -> Vec<f64> {
        (0..size).map(|i| i as f64 * 0.1).collect()
    }

    #[test]
    fn test_adaptive_activity_score_uniform() {
        let tile = vec![0.5; 10];
        let score = AdaptiveObjectPartition::activity_score(&tile);
        assert!((score - 0.0).abs() < 1e-12, "Uniform tile should have 0 activity");
    }

    #[test]
    fn test_adaptive_activity_score_high() {
        let tile: Vec<f64> = (0..10).map(|i| if i % 2 == 0 { 10.0 } else { -10.0 }).collect();
        let score = AdaptiveObjectPartition::activity_score(&tile);
        assert!(score > 0.1, "Alternating values should have high activity: {}", score);
    }

    #[test]
    fn test_adaptive_activity_score_empty() {
        let tile: Vec<f64> = Vec::new();
        let score = AdaptiveObjectPartition::activity_score(&tile);
        assert!((score - 0.0).abs() < 1e-12, "Empty tile should have 0 activity");
    }

    #[test]
    fn test_adaptive_activity_score_single() {
        let tile = vec![3.14];
        let score = AdaptiveObjectPartition::activity_score(&tile);
        assert!((score - 0.0).abs() < 1e-12, "Single element should have 0 activity");
    }

    #[test]
    fn test_adaptive_partition_uniform_latent_merges_to_coarse() {
        // 128-dim uniform → all activity ≈ 0 → merges to max_tile_size (64)
        let latent = uniform_latent(128);
        let mut aop = AdaptiveObjectPartition::with_adaptive_partitioning(0.1);
        let partitions = aop.partition(&latent);

        assert!(partitions.len() >= 2, "Expected ≥2 tiles from uniform 128-dim: got {}", partitions.len());
        // All tiles should be coarse (size ≥ max_tile_size/2 = 32)
        for p in partitions {
            assert!(p.size >= 32, "Uniform latent should produce coarse tiles, got size {}", p.size);
            assert!((p.activity - 0.0).abs() < 1e-12, "Uniform tile should have 0 activity");
        }
        // Summary should reflect coarse tiles
        let summary = aop.partition_summary();
        assert!(summary.contains("coarse"), "Uniform partition summary should mention coarse tiles: {}", summary);
    }

    #[test]
    fn test_adaptive_partition_high_activity_splits_to_fine() {
        // High-activity latent → splits to min_tile_size (4)
        let latent = high_activity_latent(64);
        let mut aop = AdaptiveObjectPartition::with_adaptive_partitioning(0.1);
        let partitions = aop.partition(&latent);

        // With activity > 0.1, all tiles should be fine (size ≤ min*2 = 8)
        for p in partitions {
            assert!(p.size <= 8, "High-activity tile should be fine, got size {}", p.size);
            assert!(p.activity > 0.05, "High-activity tile should have high activity: {}", p.activity);
        }
        let summary = aop.partition_summary();
        assert!(summary.contains("fine"), "High-activity partition should mention fine tiles: {}", summary);
    }

    #[test]
    fn test_adaptive_partition_mixed_activity() {
        // Mixed: first half uniform (low activity), second half alternating (high)
        let latent = mixed_latent(64);
        let mut aop = AdaptiveObjectPartition::with_adaptive_partitioning(0.1);
        let partitions = aop.partition(&latent);

        // Should have both low-activity (coarse) and high-activity (fine) tiles
        let low_activity = partitions.iter().filter(|p| p.activity < 0.05).count();
        let high_activity = partitions.iter().filter(|p| p.activity > 0.05).count();
        assert!(low_activity > 0, "Mixed latent should have low-activity tiles, found {}", low_activity);
        assert!(high_activity > 0, "Mixed latent should have high-activity tiles, found {}", high_activity);
    }

    #[test]
    fn test_adaptive_partition_too_small_for_min_tile() {
        // Latent smaller than min_tile_size → single partition
        let latent = vec![0.1, 0.2, 0.3];
        let mut aop = AdaptiveObjectPartition::new(16, 4, 64, 0.1);
        let partitions = aop.partition(&latent);

        assert_eq!(partitions.len(), 1, "Latent < min_tile_size should produce 1 partition");
        assert_eq!(partitions[0].start, 0);
        assert_eq!(partitions[0].end, 3);
        assert!(partitions[0].is_leaf);
    }

    #[test]
    fn test_adaptive_partition_convergence_max_three_iterations() {
        // Use a 256-dim ramp → moderate activity, should converge within 3 iterations
        let latent = ramp_latent(256);
        let mut aop = AdaptiveObjectPartition::with_adaptive_partitioning(0.1);
        let partitions = aop.partition(&latent);

        assert!(!partitions.is_empty(), "Partitions should be non-empty");
        // All partitions should cover the full latent
        let covered: usize = partitions.iter().map(|p| p.size).sum();
        assert_eq!(covered, 256, "Partitions should cover entire latent");

        let summary = aop.partition_summary();
        assert!(summary.contains("tiles:"), "Summary should have tile count");
    }

    #[test]
    fn test_adaptive_partition_empty_latent() {
        let latent: Vec<f64> = Vec::new();
        let mut aop = AdaptiveObjectPartition::with_adaptive_partitioning(0.1);
        let partitions = aop.partition(&latent);

        assert!(partitions.is_empty(), "Empty latent should produce empty partitions");
        assert_eq!(aop.num_partitions(), 0);
        assert_eq!(aop.partition_summary(), "0 tiles");
    }

    #[test]
    fn test_adaptive_partition_not_divisible_by_base() {
        // 100-dim latent with base_tile_size=16 → last tile absorbs remainder (100 % 16 = 4)
        let latent = high_activity_latent(100);
        let mut aop = AdaptiveObjectPartition::with_adaptive_partitioning(0.1);
        let partitions = aop.partition(&latent);

        let covered: usize = partitions.iter().map(|p| p.size).sum();
        assert_eq!(covered, 100, "Partitions should cover all 100 dimensions");
    }

    #[test]
    fn test_adaptive_partition_summary_format() {
        let latent = uniform_latent(64);
        let mut aop = AdaptiveObjectPartition::with_adaptive_partitioning(0.1);
        aop.partition(&latent);

        let summary = aop.partition_summary();
        // Format: "N tiles: X fine + Y medium + Z coarse"
        assert!(summary.contains("tiles:"), "Summary should contain 'tiles:'");
        assert!(summary.contains("fine"), "Summary should contain 'fine'");
        assert!(summary.contains("medium"), "Summary should contain 'medium'");
        assert!(summary.contains("coarse"), "Summary should contain 'coarse'");

        // Parse and verify counts sum to total
        let parts: Vec<&str> = summary.split_whitespace().collect();
        let total: usize = parts[0].parse().expect("First token should be total count");
        assert_eq!(total, aop.num_partitions(), "Summary total should match num_partitions");
    }

    #[test]
    fn test_object_partition_adaptive_v2_convenience() {
        let latent = high_activity_latent(64);
        let aop = ObjectPartition::adaptive_v2(&latent, None);

        assert!(aop.num_partitions() > 0, "adaptive_v2 should produce partitions");
        let summary = aop.partition_summary();
        assert!(summary.contains("fine"), "High-activity using adaptive_v2 should produce fine tiles");
    }

    #[test]
    fn test_adaptive_partition_odd_base_behavior() {
        // 17-dim latent with base=16, min=4 → first tile of 16, last of 1
        // min_tile_size=4 > 1, so last tile is a leaf
        let latent = vec![0.5; 17];
        let mut aop = AdaptiveObjectPartition::new(16, 4, 64, 0.1);
        let partitions = aop.partition(&latent);

        let covered: usize = partitions.iter().map(|p| p.size).sum();
        assert_eq!(covered, 17, "All dimensions must be covered");
    }

    #[test]
    fn test_adaptive_partition_covers_all_dimensions() {
        // Verify invariants for various sizes
        for size in [16, 32, 64, 128, 200, 300].iter() {
            let mut aop = AdaptiveObjectPartition::with_adaptive_partitioning(0.1);
            let latent = ramp_latent(*size);
            let partitions = aop.partition(&latent);

            // Contiguity: each partition starts where previous ended
            let mut expected_start = 0;
            for p in partitions {
                assert_eq!(p.start, expected_start, "Non-contiguous partition");
                assert_eq!(p.end, p.start + p.size, "Size mismatch");
                expected_start = p.end;
            }
            assert_eq!(expected_start, *size, "Doesn't cover all dimensions");
        }
    }
}
