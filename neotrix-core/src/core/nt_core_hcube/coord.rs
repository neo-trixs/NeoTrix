use super::axis::DimensionAxis;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperCoord {
    values: HashMap<DimensionAxis, f64>,
}

impl Default for HyperCoord {
    fn default() -> Self {
        Self::new()
    }
}

impl HyperCoord {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn with(axis: DimensionAxis, value: f64) -> Self {
        let mut values = HashMap::new();
        values.insert(axis, value);
        Self { values }
    }

    pub fn set(&mut self, axis: DimensionAxis, value: f64) {
        self.values.insert(axis, value);
    }

    pub fn get(&self, axis: &DimensionAxis) -> f64 {
        self.values.get(axis).copied().unwrap_or(0.0)
    }

    pub fn dims(&self) -> impl Iterator<Item = (&DimensionAxis, &f64)> {
        self.values.iter()
    }

    pub fn euclidean_distance(&self, other: &HyperCoord) -> f64 {
        let all_axes = DimensionAxis::all();
        let mut sum_sq = 0.0_f64;
        for axis in all_axes {
            let d = self.get(axis) - other.get(axis);
            sum_sq += d * d;
        }
        sum_sq.sqrt()
    }

    pub fn cosine_similarity(&self, other: &HyperCoord) -> f64 {
        let all_axes = DimensionAxis::all();
        let mut dot = 0.0_f64;
        let mut mag_a = 0.0_f64;
        let mut mag_b = 0.0_f64;
        for axis in all_axes {
            let a = self.get(axis);
            let b = other.get(axis);
            dot += a * b;
            mag_a += a * a;
            mag_b += b * b;
        }
        let denom = mag_a.sqrt() * mag_b.sqrt();
        if denom < 1e-12 {
            0.0
        } else {
            dot / denom
        }
    }

    pub fn to_dense(&self) -> [f64; 16] {
        let mut arr = [0.0_f64; 16];
        for (axis, &val) in &self.values {
            arr[*axis as usize] = val;
        }
        arr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_coord_empty() {
        let c = HyperCoord::new();
        assert_eq!(c.dims().count(), 0);
    }

    #[test]
    fn test_coord_with_sets_initial_value() {
        let c = HyperCoord::with(DimensionAxis::Creativity, 0.8);
        assert!((c.get(&DimensionAxis::Creativity) - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_coord_get_returns_zero_for_unset_axis() {
        let c = HyperCoord::new();
        assert!((c.get(&DimensionAxis::Debugging) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_coord_set_overwrites() {
        let mut c = HyperCoord::with(DimensionAxis::Performance, 0.5);
        c.set(DimensionAxis::Performance, 0.9);
        assert!((c.get(&DimensionAxis::Performance) - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_coord_multiple_axes() {
        let mut c = HyperCoord::new();
        c.set(DimensionAxis::Safety, 0.3);
        c.set(DimensionAxis::Scale, 0.7);
        assert_eq!(c.dims().count(), 2);
    }

    #[test]
    fn test_coord_default_is_empty() {
        let c: HyperCoord = Default::default();
        assert_eq!(c.dims().count(), 0);
    }

    #[test]
    fn test_euclidean_distance_self_is_zero() {
        let c = HyperCoord::with(DimensionAxis::Abstraction, 0.8);
        assert!((c.euclidean_distance(&c)).abs() < 1e-12);
    }

    #[test]
    fn test_euclidean_distance_orthogonal() {
        let a = HyperCoord::with(DimensionAxis::Abstraction, 1.0);
        let b = HyperCoord::with(DimensionAxis::Creativity, 1.0);
        let d = a.euclidean_distance(&b);
        assert!((d - 2.0_f64.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn test_cosine_similarity_identical_is_one() {
        let c = HyperCoord::with(DimensionAxis::Safety, 0.7);
        assert!((c.cosine_similarity(&c) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_cosine_similarity_orthogonal_is_zero() {
        let a = HyperCoord::with(DimensionAxis::Abstraction, 1.0);
        let b = HyperCoord::with(DimensionAxis::Creativity, 1.0);
        assert!((a.cosine_similarity(&b)).abs() < 1e-12);
    }

    #[test]
    fn test_cosine_similarity_all_zeros_is_zero() {
        let a = HyperCoord::new();
        let b = HyperCoord::new();
        assert!((a.cosine_similarity(&b)).abs() < 1e-12);
    }

    #[test]
    fn test_to_dense_all_axes_present() {
        let mut c = HyperCoord::new();
        c.set(DimensionAxis::CodeUnderstanding, 0.5);
        c.set(DimensionAxis::Modality, 1.0);
        let dense = c.to_dense();
        assert_eq!(dense.len(), 16);
        assert!((dense[0] - 0.5).abs() < 1e-9);
        assert!((dense[15] - 1.0).abs() < 1e-9);
        assert!((dense[1] - 0.0).abs() < 1e-9);
    }
}
