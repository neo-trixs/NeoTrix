/// Shared FloatVec + cosine distance for HNSW indexing.

/// f32 vector wrapper implementing `instant_distance::Point` (cosine distance).
#[derive(Clone, Debug)]
pub struct FloatVec(pub Vec<f32>);

impl instant_distance::Point for FloatVec {
    fn distance(&self, other: &Self) -> f32 {
        cosine_distance(&self.0, &other.0)
    }
}

/// Cosine distance: `1.0 - (a · b) / (‖a‖ * ‖b‖)`.
/// Returns `1.0` for zero-norm vectors.
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na * nb > 0.0 {
        1.0 - dot / (na * nb)
    } else {
        1.0
    }
}

/// Convert little-endian byte slice to `Vec<f32>`.
pub fn bytes_to_f32s(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_distance_identical() {
        let v = vec![1.0, 0.0, 0.0];
        assert!((cosine_distance(&v, &v)).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_distance_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_distance(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bytes_to_f32s_roundtrip() {
        let vals = vec![1.0f32, 2.0, 3.0];
        let bytes: Vec<u8> = vals.iter().flat_map(|v| v.to_le_bytes()).collect();
        let back = bytes_to_f32s(&bytes);
        assert_eq!(back, vals);
    }
}
