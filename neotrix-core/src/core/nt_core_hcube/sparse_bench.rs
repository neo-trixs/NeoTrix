use crate::core::nt_core_hcube::sparse_vsa::SparseBinaryVSA;
use crate::core::nt_core_hcube::vsa_bridge::VsaBridge;
use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use std::time::Instant;

/// 稀疏 vs 密集 VSA 操作性能对比报告
#[derive(Debug, Clone)]
pub struct SparseDenseBenchReport {
    pub dense_bind_ns: f64,
    pub sparse_bind_ns: f64,
    pub dense_bundle_ns: f64,
    pub sparse_bundle_ns: f64,
    pub dense_similarity_ns: f64,
    pub sparse_similarity_ns: f64,
    pub dense_bytes: usize,
    pub sparse_bytes: usize,
    pub speedup_bind: f64,
    pub speedup_similarity: f64,
    pub compression_ratio: f64,
}

/// 稀疏 vs 密集 VSA 操作性能对比
pub fn benchmark_sparse_vs_dense(iterations: usize) -> SparseDenseBenchReport {
    type SparseVSA = SparseBinaryVSA<VSA_DIM, 32>;

    let dense_vectors: Vec<Vec<u8>> = (0..100).map(|_| QuantizedVSA::random_binary()).collect();
    let sparse_vectors: Vec<SparseVSA> = dense_vectors
        .iter()
        .map(|v| VsaBridge::dense_to_sparse::<32>(v))
        .collect();

    let pairs: Vec<(usize, usize)> = (0..50).map(|i| (i, (i + 1) % 100)).collect();
    let groups: Vec<Vec<usize>> = (0..100)
        .collect::<Vec<_>>()
        .chunks(10)
        .map(|c| c.to_vec())
        .collect();

    let n_ops = (iterations * pairs.len()) as f64;

    let start = Instant::now();
    for _ in 0..iterations {
        for &(i, j) in &pairs {
            let _ = QuantizedVSA::bind(&dense_vectors[i], &dense_vectors[j]);
        }
    }
    let dense_bind_ns = start.elapsed().as_nanos() as f64 / n_ops;

    let start = Instant::now();
    for _ in 0..iterations {
        for &(i, j) in &pairs {
            let _ = SparseVSA::bind(&sparse_vectors[i], &sparse_vectors[j]);
        }
    }
    let sparse_bind_ns = start.elapsed().as_nanos() as f64 / n_ops;

    let n_bundle_ops = (iterations * groups.len()) as f64;

    let start = Instant::now();
    for _ in 0..iterations {
        for group in &groups {
            let vecs: Vec<&[u8]> = group.iter().map(|&i| dense_vectors[i].as_slice()).collect();
            let _ = QuantizedVSA::bundle(&vecs);
        }
    }
    let dense_bundle_ns = start.elapsed().as_nanos() as f64 / n_bundle_ops;

    let start = Instant::now();
    for _ in 0..iterations {
        for group in &groups {
            let vecs: Vec<&SparseVSA> = group.iter().map(|&i| &sparse_vectors[i]).collect();
            let _ = SparseVSA::bundle(&vecs);
        }
    }
    let sparse_bundle_ns = start.elapsed().as_nanos() as f64 / n_bundle_ops;

    let start = Instant::now();
    for _ in 0..iterations {
        for &(i, j) in &pairs {
            let _ = QuantizedVSA::similarity(&dense_vectors[i], &dense_vectors[j]);
        }
    }
    let dense_similarity_ns = start.elapsed().as_nanos() as f64 / n_ops;

    let start = Instant::now();
    for _ in 0..iterations {
        for &(i, j) in &pairs {
            let _ = SparseVSA::similarity(&sparse_vectors[i], &sparse_vectors[j]);
        }
    }
    let sparse_similarity_ns = start.elapsed().as_nanos() as f64 / n_ops;

    let dense_bytes = VSA_DIM;
    let sparse_bytes = sparse_vectors[0].indices().len() * 2;

    SparseDenseBenchReport {
        dense_bind_ns,
        sparse_bind_ns,
        dense_bundle_ns,
        sparse_bundle_ns,
        dense_similarity_ns,
        sparse_similarity_ns,
        dense_bytes,
        sparse_bytes,
        speedup_bind: dense_bind_ns / sparse_bind_ns.max(1.0),
        speedup_similarity: dense_similarity_ns / sparse_similarity_ns.max(1.0),
        compression_ratio: dense_bytes as f64 / sparse_bytes as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_runs() {
        let report = benchmark_sparse_vs_dense(1);
        assert!(report.dense_bind_ns > 0.0);
        assert!(report.sparse_bind_ns > 0.0);
    }

    #[test]
    fn test_benchmark_report_fields() {
        let report = benchmark_sparse_vs_dense(2);
        assert!(report.dense_bytes == VSA_DIM);
        assert!(report.sparse_bytes > 0);
        assert!(report.speedup_bind > 0.0);
        assert!(report.speedup_similarity > 0.0);
        assert!(report.compression_ratio > 1.0);
        assert!(report.dense_bind_ns > 0.0);
        assert!(report.sparse_bind_ns > 0.0);
        assert!(report.dense_bundle_ns > 0.0);
        assert!(report.sparse_bundle_ns > 0.0);
        assert!(report.dense_similarity_ns > 0.0);
        assert!(report.sparse_similarity_ns > 0.0);
    }
}
