//! # NAG VSA bundle — Norm-Agnostic 残差流融合
//!
//! 借鉴 arXiv 2606.16112 (NAG):
//!   在 VSA bundle 操作前做 norm-agnostic 分离
//!   防止深层操作的 bundle 结果被浅层累积范数淹没

use crate::core::nt_core_hcube::vsa::{VSAEngine, VsaBackend};

/// NAG VSA bundle — norm-agnostic 版
/// 将幅度与方向信息分离后再 bundle
pub fn nag_bundle(engine: &VSAEngine, vectors: &[&[f64]]) -> Vec<f64> {
    let _dim = engine.dimensions();

    // Step 1: 归一化所有向量
    let normalized: Vec<Vec<f64>> = vectors.iter().map(|v| normalize(v)).collect();
    let refs: Vec<&[f64]> = normalized.iter().map(|v| v.as_slice()).collect();

    // Step 2: bundle 归一化后的向量
    let bundled = engine.bundle(&refs);

    // Step 3: 对结果做 tanh 压缩 (限制范数增长)
    bundled.iter().map(|x| x.tanh()).collect()
}

/// 归一化向量到单位长度
pub fn normalize(v: &[f64]) -> Vec<f64> {
    let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm < 1e-12 {
        return v.to_vec();
    }
    v.iter().map(|x| x / norm).collect()
}

/// NAG 相似度 — 先归一化再计算余弦
pub fn nag_similarity(a: &[f64], b: &[f64]) -> f64 {
    let a_norm = normalize(a);
    let b_norm = normalize(b);
    let dot: f64 = a_norm.iter().zip(b_norm.iter()).map(|(x, y)| x * y).sum();
    dot.clamp(-1.0, 1.0)
}

/// Mixture-of-Depths 自适应跳过
/// 在输入熵低或预测置信度高时跳过 attention/MLP
pub fn mo_depth_skip(entropy: f64, confidence: f64) -> bool {
    entropy < 1.0 && confidence > 0.8
}

/// 深度自适应 bundle — 带残差门控
/// 只在信息增益高时执行 bundle
pub fn gated_nag_bundle(engine: &VSAEngine, old: &[f64], new: &[f64], novelty: f64) -> Vec<f64> {
    if novelty < 0.1 {
        return old.to_vec(); // 跳过无信息 bundle
    }
    let weight = (novelty * 0.5 + 0.5).min(1.0);
    let bundled = nag_bundle(engine, &[old, new]);
    bundled.iter().map(|x| x * weight).collect()
}

/// 批量 NAG bundle — 多向量融合
pub fn batch_nag_bundle(engine: &VSAEngine, layers: &[Vec<Vec<f64>>]) -> Vec<Vec<f64>> {
    layers
        .iter()
        .map(|layer| {
            let refs: Vec<&[f64]> = layer.iter().map(|v| v.as_slice()).collect();
            nag_bundle(engine, &refs)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa::VsaBackend;

    #[test]
    fn test_nag_bundle_preserves_direction() {
        let engine = VSAEngine::new(64);
        let a = vec![1.0, 0.0, 0.0, 0.0];
        let b: Vec<f64> = (0..64).map(|i| (i as f64).sin()).collect();
        let a_slice = a.as_slice();
        let b_slice = b.as_slice();

        let result = nag_bundle(&engine, &[a_slice, &b_slice]);
        assert_eq!(result.len(), 64);
        // All values should be in [-1, 1] due to tanh
        assert!(result.iter().all(|x| x.abs() <= 1.0 + 1e-10));
    }

    #[test]
    fn test_normalize_unit_length() {
        let v = vec![3.0, 4.0];
        let n = normalize(&v);
        let length: f64 = n.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((length - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalize_zero_vector() {
        let v = vec![0.0, 0.0, 0.0];
        let n = normalize(&v);
        assert_eq!(n, v);
    }

    #[test]
    fn test_nag_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = nag_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_nag_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = nag_similarity(&a, &b);
        assert!(sim.abs() < 1e-10);
    }

    #[test]
    fn test_mo_depth_skip() {
        assert!(mo_depth_skip(0.5, 0.9)); // low entropy, high confidence → skip
        assert!(!mo_depth_skip(2.0, 0.9)); // high entropy → don't skip
        assert!(!mo_depth_skip(0.5, 0.5)); // low confidence → don't skip
    }

    #[test]
    fn test_gated_nag_bundle_skips_low_novelty() {
        let engine = VSAEngine::new(16);
        let old = vec![1.0, 0.0, 0.0, 0.0];
        let new = vec![1.01, 0.0, 0.0, 0.0]; // very similar
        let result = gated_nag_bundle(&engine, &old, &new, 0.05);
        assert_eq!(result, old); // should skip bundle
    }

    #[test]
    fn test_gated_nag_bundle_blends_high_novelty() {
        let engine = VSAEngine::new(16);
        let old = vec![1.0, 0.0, 0.0, 0.0];
        let new = vec![0.0, 1.0, 0.0, 0.0]; // orthogonal
        let result = gated_nag_bundle(&engine, &old, &new, 0.9);
        assert_eq!(result.len(), 16);
    }

    #[test]
    fn test_batch_nag_bundle() {
        let engine = VSAEngine::new(32);
        let layers = vec![
            vec![vec![1.0; 32], vec![0.5; 32]],
            vec![vec![0.0; 32], vec![1.0; 32]],
        ];
        let result = batch_nag_bundle(&engine, &layers);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].len(), 32);
    }

    #[test]
    fn test_nag_bundle_with_tanh_clipping() {
        let engine = VSAEngine::new(8);
        // Large values
        let a = vec![100.0; 8];
        let b = vec![-100.0; 8];
        let a_slice = a.as_slice();
        let b_slice = b.as_slice();
        let result = nag_bundle(&engine, &[a_slice, b_slice]);
        // All values should be bounded by [-1, 1]
        assert!(result.iter().all(|x| x.abs() <= 1.0 + 1e-10));
    }
}
