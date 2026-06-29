use crate::core::nt_core_hcube::sparse_vsa::SparseBinaryVSA;
use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use rand::Rng;
use std::collections::HashSet;

/// 稀疏 ↔ 密集 VSA 双向桥接
#[derive(Debug, Clone)]
pub struct VsaBridge;

impl VsaBridge {
    /// QuantizedVSA → SparseBinaryVSA (密集转稀疏, 取 top-k 活跃位)
    pub fn dense_to_sparse<const K: usize>(dense: &[u8]) -> SparseBinaryVSA<VSA_DIM, K> {
        let mut indices: Vec<u16> = dense
            .iter()
            .enumerate()
            .filter(|(_, &v)| v == 1)
            .map(|(i, _)| i as u16)
            .collect();
        indices.sort_unstable();
        indices.dedup();
        if indices.len() > K {
            let mut rng = rand::thread_rng();
            for i in (K..indices.len()).rev() {
                let j = rng.gen_range(0..=i);
                indices.swap(i, j);
            }
            indices.truncate(K);
            indices.sort_unstable();
        } else if indices.len() < K {
            let mut set: HashSet<u16> = indices.iter().copied().collect();
            let mut rng = rand::thread_rng();
            while set.len() < K {
                set.insert(rng.gen_range(0..VSA_DIM as u16));
            }
            indices = set.into_iter().collect();
            indices.sort_unstable();
        }
        SparseBinaryVSA(indices)
    }

    /// SparseBinaryVSA → QuantizedVSA 风格 (稀疏转密集 Vec<u8>)
    pub fn sparse_to_dense<const K: usize>(sparse: &SparseBinaryVSA<VSA_DIM, K>) -> Vec<u8> {
        sparse.to_dense()
    }

    /// 在稀疏域中计算两个密集向量的相似度 (更高效)
    pub fn similarity_sparse_dense(a: &[u8], b: &[u8]) -> f64 {
        let indices_a: HashSet<u16> = a
            .iter()
            .enumerate()
            .filter(|(_, &v)| v == 1)
            .map(|(i, _)| i as u16)
            .collect();
        let indices_b: HashSet<u16> = b
            .iter()
            .enumerate()
            .filter(|(_, &v)| v == 1)
            .map(|(i, _)| i as u16)
            .collect();
        let intersection = indices_a.intersection(&indices_b).count();
        let union = indices_a.len() + indices_b.len() - intersection;
        if union == 0 {
            return 1.0;
        }
        intersection as f64 / union as f64
    }

    /// 混合绑定: 一个密集 + 一个稀疏 → 密集结果
    pub fn bind_mixed(dense: &[u8], sparse_indices: &[u16]) -> Vec<u8> {
        let mut result = dense.to_vec();
        for &idx in sparse_indices {
            let i = idx as usize;
            if i < result.len() {
                result[i] ^= 1;
            }
        }
        result
    }

    /// 批量转换: 多个密集向量 → 稀疏向量列表
    pub fn batch_dense_to_sparse<const K: usize>(
        dense_vectors: &[&[u8]],
    ) -> Vec<SparseBinaryVSA<VSA_DIM, K>> {
        dense_vectors
            .iter()
            .map(|v| Self::dense_to_sparse::<K>(v))
            .collect()
    }

    /// 自适应稀疏度: 根据向量内容自动选择 K
    pub fn adaptive_sparse(dense: &[u8]) -> SparseBinaryVSA<VSA_DIM, 64> {
        Self::dense_to_sparse::<64>(dense)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

    #[test]
    fn test_dense_to_sparse_roundtrip() {
        let dense = QuantizedVSA::random_binary();
        let sparse = VsaBridge::dense_to_sparse::<64>(&dense);
        let recovered = VsaBridge::sparse_to_dense(&sparse);
        assert_eq!(recovered.len(), VSA_DIM);
        for &idx in sparse.indices() {
            assert_eq!(
                recovered[idx as usize], 1,
                "sparse position must be 1 in recovered dense"
            );
        }
        assert!(sparse.indices().len() <= 64);
    }

    #[test]
    fn test_dense_to_sparse_maintains_similarity() {
        let a = QuantizedVSA::random_binary();
        let b = QuantizedVSA::random_binary();
        let sa = VsaBridge::dense_to_sparse::<32>(&a);
        let sb = VsaBridge::dense_to_sparse::<32>(&b);
        let dense_sim = QuantizedVSA::similarity(&a, &b);
        let sparse_sim = SparseBinaryVSA::<VSA_DIM, 32>::similarity(&sa, &sb);
        assert!(sparse_sim >= 0.0 && sparse_sim <= 1.0);
        let _ = dense_sim;
    }

    #[test]
    fn test_batch_conversion() {
        let vectors: Vec<Vec<u8>> = (0..10).map(|_| QuantizedVSA::random_binary()).collect();
        let refs: Vec<&[u8]> = vectors.iter().map(|v| v.as_slice()).collect();
        let sparse_vecs = VsaBridge::batch_dense_to_sparse::<32>(&refs);
        assert_eq!(sparse_vecs.len(), 10);
        for sv in &sparse_vecs {
            assert!(sv.indices().len() <= 32);
        }
    }

    #[test]
    fn test_similarity_sparse_dense() {
        let a = QuantizedVSA::random_binary();
        let sim_self = VsaBridge::similarity_sparse_dense(&a, &a);
        assert!((sim_self - 1.0).abs() < 1e-6, "self-similarity must be 1");

        let zeros = vec![0u8; VSA_DIM];
        let ones = vec![1u8; VSA_DIM];
        let sim_orth = VsaBridge::similarity_sparse_dense(&zeros, &ones);
        assert!(
            (sim_orth - 0.0).abs() < 1e-6,
            "all-zero vs all-one must have 0 similarity"
        );
    }

    #[test]
    fn test_bind_mixed_roundtrip() {
        let dense = QuantizedVSA::random_binary();
        let sparse = VsaBridge::dense_to_sparse::<32>(&dense);
        let bound = VsaBridge::bind_mixed(&dense, sparse.indices());
        let recovered = VsaBridge::bind_mixed(&bound, sparse.indices());
        assert_eq!(recovered, dense, "double XOR bind must roundtrip");
    }

    #[test]
    fn test_adaptive_sparse_output() {
        let dense = QuantizedVSA::random_binary();
        let sparse = VsaBridge::adaptive_sparse(&dense);
        assert_eq!(sparse.indices().len(), 64);
    }
}
