//! 向量/矩阵运算函数
//!
//! 生产路径仅消费 `cosine_similarity`（knowledge_engine/search、cortex_memory/engine、
//! seal_core/embedding）。其余导出曾为平行副本，全部无生产消费方，已按 R-P42/R-P76
//! 收敛（cycle 204 审计 MID-1）。

pub fn l2_norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
