#![forbid(unsafe_code)]

pub fn cosine_similarity_f32(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(&x, &y)| (x as f64) * (y as f64)).sum();
    let norm_a: f64 = a.iter().map(|&x| (x as f64) * (x as f64)).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|&x| (x as f64) * (x as f64)).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

pub fn cosine_similarity_f64(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

pub fn cosine_similarity_bytes(a: &[u8], b: &[u8]) -> f64 {
    let hd: u64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones() as u64)
        .sum();
    let dim = (a.len().min(b.len()) * 8) as f64;
    if dim == 0.0 {
        return 0.0;
    }
    1.0 - 2.0 * hd as f64 / dim
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_identical() {
        let a = [1.0, 0.0, 0.0];
        assert!((cosine_similarity_f32(&a, &a) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn f32_orthogonal() {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        assert!(cosine_similarity_f32(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn f32_mismatch_length() {
        assert_eq!(cosine_similarity_f32(&[1.0, 0.0], &[1.0, 0.0, 0.0]), 0.0);
    }

    #[test]
    fn f64_identical() {
        let a = [1.0, 0.0, 0.0];
        assert!((cosine_similarity_f64(&a, &a) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn f64_zero_norm() {
        assert_eq!(cosine_similarity_f64(&[0.0], &[1.0]), 0.0);
    }

    #[test]
    fn bytes_identical() {
        let v = vec![0b10101010u8];
        assert!((cosine_similarity_bytes(&v, &v) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn bytes_opposite() {
        let a = vec![0b00000000u8];
        let b = vec![0b11111111u8];
        assert!((cosine_similarity_bytes(&a, &b) - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn bytes_big_vectors() {
        let a = vec![0xFFu8; 64];
        let b = vec![0xFFu8; 64];
        assert!((cosine_similarity_bytes(&a, &b) - 1.0).abs() < 1e-10);
    }
}
