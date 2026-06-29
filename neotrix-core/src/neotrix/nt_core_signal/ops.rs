//! 向量/矩阵运算函数
use super::core::{Matrix, MatrixError, Vector};

pub fn l2_norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

pub fn softmax(v: &[f64]) -> Vector {
    let max = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exp: Vector = v.iter().map(|x| (x - max).exp()).collect();
    let sum: f64 = exp.iter().sum();
    if sum <= 0.0 {
        return vec![0.0; v.len()];
    }
    exp.iter().map(|x| x / sum).collect()
}

pub fn relu(v: &[f64]) -> Vector {
    v.iter().map(|x| x.max(0.0)).collect()
}

pub fn sigmoid(v: &[f64]) -> Vector {
    v.iter().map(|x| 1.0 / (1.0 + (-x).exp())).collect()
}

pub fn gelu(v: &[f64]) -> Vector {
    v.iter()
        .map(|x| {
            let cdf = 0.5
                * (1.0 + (2.0 / std::f64::consts::PI).sqrt() * x)
                    .tanh()
                    .abs()
                    .min(1.0);
            x * cdf
        })
        .collect()
}

pub fn matrix_vector_mul(matrix: &Matrix, vector: &[f64]) -> Vector {
    matrix
        .iter()
        .map(|row| row.iter().zip(vector.iter()).map(|(m, v)| m * v).sum())
        .collect()
}

pub fn matrix_vector_mul_safe(matrix: &Matrix, vector: &[f64]) -> Result<Vector, MatrixError> {
    if matrix.is_empty() {
        return Err(MatrixError::EmptyMatrix);
    }
    if vector.is_empty() {
        return Err(MatrixError::EmptyVector);
    }
    for row in matrix {
        if row.len() != vector.len() {
            return Err(MatrixError::DimensionMismatch {
                expected: vector.len(),
                got: row.len(),
            });
        }
    }
    Ok(matrix_vector_mul(matrix, vector))
}

pub fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot = dot_product(a, b);
    let norm_a = l2_norm(a);
    let norm_b = l2_norm(b);
    if norm_a == 0.0 && norm_b == 0.0 {
        return 1.0;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

pub fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

pub fn normalize(v: &[f64]) -> Vector {
    let norm = l2_norm(v);
    if norm == 0.0 {
        return vec![0.0; v.len()];
    }
    v.iter().map(|x| x / norm).collect()
}

pub fn clamp(v: &[f64], min: f64, max: f64) -> Vector {
    v.iter().map(|x| x.min(max).max(min)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l2_norm_positive() {
        let v = vec![3.0, 4.0];
        assert!((l2_norm(&v) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_l2_norm_zero() {
        assert!((l2_norm(&[0.0, 0.0]) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_l2_norm_empty() {
        assert!((l2_norm(&[]) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_softmax_sums_to_one() {
        let v = vec![1.0, 2.0, 3.0];
        let s = softmax(&v);
        let sum: f64 = s.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_softmax_all_zero() {
        let v = vec![0.0, 0.0, 0.0];
        let s = softmax(&v);
        for val in &s {
            assert!((val - 1.0 / 3.0).abs() < 1e-9);
        }
    }

    #[test]
    fn test_softmax_empty() {
        let s = softmax(&[]);
        assert!(s.is_empty());
    }

    #[test]
    fn test_relu_negative_to_zero() {
        let v = vec![-2.0, -0.5, 0.0, 1.0, 3.0];
        let r = relu(&v);
        assert!((r[0] - 0.0).abs() < 1e-9);
        assert!((r[1] - 0.0).abs() < 1e-9);
        assert!((r[2] - 0.0).abs() < 1e-9);
        assert!((r[3] - 1.0).abs() < 1e-9);
        assert!((r[4] - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_sigmoid_bounds() {
        let v = vec![-100.0, 0.0, 100.0];
        let s = sigmoid(&v);
        assert!((s[0] - 0.0).abs() < 1e-9);
        assert!((s[1] - 0.5).abs() < 1e-9);
        assert!((s[2] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_dot_product_positive() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        assert!((dot_product(&a, &b) - 32.0).abs() < 1e-9);
    }

    #[test]
    fn test_dot_product_zero() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 5.0];
        assert!((dot_product(&a, &b) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_cosine_similarity_both_zero() {
        assert!((cosine_similarity(&[0.0, 0.0], &[0.0, 0.0]) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_cosine_similarity_one_zero() {
        assert!((cosine_similarity(&[1.0, 2.0], &[0.0, 0.0]) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_euclidean_distance_zero() {
        assert!((euclidean_distance(&[1.0, 2.0], &[1.0, 2.0]) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_euclidean_distance_positive() {
        assert!((euclidean_distance(&[0.0, 0.0], &[3.0, 4.0]) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_normalize_unit_length() {
        let v = vec![3.0, 4.0];
        let n = normalize(&v);
        assert!((l2_norm(&n) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_normalize_zero_vector() {
        let v = vec![0.0, 0.0];
        let n = normalize(&v);
        assert_eq!(n, vec![0.0, 0.0]);
    }

    #[test]
    fn test_clamp_bounds() {
        let v = vec![-5.0, 0.0, 5.0, 10.0];
        let c = clamp(&v, -1.0, 3.0);
        assert!((c[0] - (-1.0)).abs() < 1e-9);
        assert!((c[1] - 0.0).abs() < 1e-9);
        assert!((c[2] - 3.0).abs() < 1e-9);
        assert!((c[3] - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_matrix_vector_mul_identity() {
        let matrix = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let v = vec![3.0, 4.0];
        let r = matrix_vector_mul(&matrix, &v);
        assert!((r[0] - 3.0).abs() < 1e-9);
        assert!((r[1] - 4.0).abs() < 1e-9);
    }

    #[test]
    fn test_matrix_vector_mul_safe_ok() {
        let matrix = vec![vec![2.0, 0.0], vec![0.0, 3.0]];
        let v = vec![1.0, 2.0];
        let r = matrix_vector_mul_safe(&matrix, &v).unwrap();
        assert!((r[0] - 2.0).abs() < 1e-9);
        assert!((r[1] - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_matrix_vector_mul_safe_empty_matrix() {
        let r = matrix_vector_mul_safe(&Vec::<Vec<f64>>::new(), &[1.0, 2.0]);
        assert!(r.is_err());
    }

    #[test]
    fn test_matrix_vector_mul_safe_dimension_mismatch() {
        let matrix = vec![vec![1.0, 2.0, 3.0]];
        let v = vec![1.0, 2.0];
        let r = matrix_vector_mul_safe(&matrix, &v);
        assert!(r.is_err());
    }

    #[test]
    fn test_gelu_positive_increasing() {
        let v = vec![0.0, 1.0, 2.0, 3.0];
        let g = gelu(&v);
        assert!(g[0] < g[1] && g[1] < g[2] && g[2] < g[3]);
    }

    #[test]
    fn test_gelu_zero() {
        let g = gelu(&[0.0]);
        assert!((g[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_gelu_positive_larger_than_input() {
        let v = vec![3.0];
        let g = gelu(&v);
        assert!(g[0] > 0.0);
    }

    #[test]
    fn test_gelu_negative_smaller_than_zero() {
        let v = vec![-1.0];
        let g = gelu(&v);
        assert!(g[0] < 0.0);
    }
}
