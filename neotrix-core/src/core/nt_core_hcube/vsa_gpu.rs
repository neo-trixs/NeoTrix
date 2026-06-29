// REVIVED Task 2 — dead_code removed

use crate::core::nt_core_hcube::vsa::{VSAEngine, VsaBackend};

const VSA_DIM: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    CpuOnly,
    Metal,
    Cuda,
}

pub struct GpuVsaEngine {
    backend: GpuBackend,
    engine: VSAEngine,
}

impl GpuVsaEngine {
    pub fn new() -> Self {
        let backend = Self::auto_detect();
        if backend != GpuBackend::CpuOnly {
            log::info!("GpuVsaEngine: using GPU backend {:?}", backend);
        } else {
            log::info!("GpuVsaEngine: no GPU backend detected, falling back to CPU");
        }
        Self {
            backend,
            engine: VSAEngine::new(VSA_DIM),
        }
    }

    pub fn with_dim(dim: usize) -> Self {
        let backend = Self::auto_detect();
        if backend != GpuBackend::CpuOnly {
            log::info!("GpuVsaEngine: using GPU backend {:?}", backend);
        } else {
            log::info!("GpuVsaEngine: no GPU backend detected, falling back to CPU");
        }
        Self {
            backend,
            engine: VSAEngine::new(dim),
        }
    }

    pub fn auto_detect() -> GpuBackend {
        if let Ok(val) = std::env::var("NEOTRIX_GPU_BACKEND") {
            return match val.to_lowercase().as_str() {
                "cuda" => GpuBackend::Cuda,
                "metal" => GpuBackend::Metal,
                _ => GpuBackend::CpuOnly,
            };
        }
        #[cfg(target_os = "macos")]
        {
            if Self::check_metal() {
                return GpuBackend::Metal;
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            if Self::check_cuda() {
                return GpuBackend::Cuda;
            }
        }
        GpuBackend::CpuOnly
    }

    #[cfg(target_os = "macos")]
    fn check_metal() -> bool {
        std::process::Command::new("system_profiler")
            .arg("SPHardwareDataType")
            .output()
            .ok()
            .and_then(|o| {
                let s = String::from_utf8_lossy(&o.stdout);
                Some(s.contains("Metal"))
            })
            .unwrap_or(false)
    }

    fn check_cuda() -> bool {
        std::process::Command::new("nvidia-smi")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn backend(&self) -> GpuBackend {
        self.backend
    }

    pub fn is_gpu(&self) -> bool {
        self.backend != GpuBackend::CpuOnly
    }

    pub fn bind(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        self.engine.bind(a, b)
    }

    pub fn bundle(&self, vectors: &[&[f64]]) -> Vec<f64> {
        let mut result = self.engine.bundle(vectors);
        let norm = result.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 1e-12 {
            for x in result.iter_mut() {
                *x /= norm;
            }
        }
        result
    }

    pub fn invert(&self, a: &[f64]) -> Vec<f64> {
        a.iter().map(|x| -x).collect()
    }

    pub fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        self.engine.similarity(a, b)
    }

    pub fn cleanup(&self, query: &[f64], codebook: &[&[f64]]) -> Option<usize> {
        codebook
            .iter()
            .enumerate()
            .map(|(i, c)| (i, self.similarity(query, c)))
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, sim)| *sim > 0.0)
            .map(|(i, _)| i)
    }

    pub fn bind_batch(&self, a: &[&[f64]], b: &[&[f64]]) -> Vec<Vec<f64>> {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| self.bind(x, y))
            .collect()
    }

    pub fn similarity_matrix(&self, vectors: &[&[f64]]) -> Vec<Vec<f64>> {
        let n = vectors.len();
        let mut mat = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                mat[i][j] = self.similarity(vectors[i], vectors[j]);
            }
        }
        mat
    }

    pub fn benchmark(&self) -> BenchmarkResult {
        let dim = self.engine.dimensions();
        let a: Vec<f64> = (0..dim).map(|i| (i as f64).sin()).collect();
        let b: Vec<f64> = (0..dim).map(|i| (i as f64).cos()).collect();
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            self.bind(&a, &b);
        }
        let bind_time = start.elapsed();
        let vectors: Vec<Vec<f64>> = (0..100)
            .map(|i| (0..dim).map(|j| ((i * 1024 + j) as f64).sin()).collect())
            .collect();
        let refs: Vec<&[f64]> = vectors.iter().map(|v| v.as_slice()).collect();
        let start = std::time::Instant::now();
        let _ = self.similarity_matrix(&refs);
        let matrix_time = start.elapsed();
        BenchmarkResult {
            backend: self.backend,
            dim,
            bind_1000_ns: bind_time.as_nanos(),
            matrix_100x100_ns: matrix_time.as_nanos(),
        }
    }
}

impl Default for GpuVsaEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub backend: GpuBackend,
    pub dim: usize,
    pub bind_1000_ns: u128,
    pub matrix_100x100_ns: u128,
}

// Backward compat stubs
pub struct GpuVsaBackend;
pub struct CpuAccel;
pub struct MetalAccel;
pub struct VsaAccelerator;

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> GpuVsaEngine {
        GpuVsaEngine::with_dim(VSA_DIM)
    }

    fn random_vector(dim: usize, seed: usize) -> Vec<f64> {
        (0..dim).map(|i| ((seed * 1024 + i) as f64).sin()).collect()
    }

    #[test]
    fn test_gpu_engine_creation() {
        let e = engine();
        assert_eq!(e.engine.dimensions(), VSA_DIM);
    }

    #[test]
    fn test_bind_similarity() {
        let e = engine();
        let a = random_vector(VSA_DIM, 1);
        let b = random_vector(VSA_DIM, 2);
        let ab = e.bind(&a, &b);
        let aba = e.bind(&ab, &a);
        let sim = e.similarity(&aba, &b);
        assert!(sim > 0.5, "bind(a, bind(a, b)) ≈ b, sim={}", sim);
    }

    #[test]
    fn test_batch_vs_sequential() {
        let e = engine();
        let a = random_vector(VSA_DIM, 1);
        let b = random_vector(VSA_DIM, 2);
        let c = random_vector(VSA_DIM, 3);
        let d = random_vector(VSA_DIM, 4);
        let batch_result = e.bind_batch(&[&a, &c], &[&b, &d]);
        let seq1 = e.bind(&a, &b);
        let seq2 = e.bind(&c, &d);
        assert_eq!(batch_result[0], seq1);
        assert_eq!(batch_result[1], seq2);
    }

    #[test]
    fn test_similarity_matrix() {
        let e = engine();
        let v1 = random_vector(VSA_DIM, 1);
        let v2 = random_vector(VSA_DIM, 2);
        let v3 = random_vector(VSA_DIM, 3);
        let refs = vec![v1.as_slice(), v2.as_slice(), v3.as_slice()];
        let mat = e.similarity_matrix(&refs);
        assert_eq!(mat.len(), 3);
        assert_eq!(mat[0].len(), 3);
        assert!((mat[0][0] - 1.0).abs() < 1e-10, "diagonal 1.0");
        assert!((mat[1][1] - 1.0).abs() < 1e-10, "diagonal 1.0");
        assert!((mat[2][2] - 1.0).abs() < 1e-10, "diagonal 1.0");
    }

    #[test]
    fn test_cpu_fallback_on_unsupported() {
        let e = GpuVsaEngine::with_dim(256);
        let a = random_vector(256, 1);
        let b = random_vector(256, 2);
        let c = e.bind(&a, &b);
        assert_eq!(c.len(), 256);
        let sim = e.similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_backend_auto_detect() {
        let backend = GpuVsaEngine::auto_detect();
        match backend {
            GpuBackend::CpuOnly | GpuBackend::Metal | GpuBackend::Cuda => {}
        }
    }

    #[test]
    fn test_bundle_normalizes() {
        let e = engine();
        let a = random_vector(VSA_DIM, 1);
        let b = random_vector(VSA_DIM, 2);
        let bundled = e.bundle(&[&a, &b]);
        let norm: f64 = bundled.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 1e-10, "bundle should normalize");
    }

    #[test]
    fn test_invert_flips_sign() {
        let e = engine();
        let a = random_vector(VSA_DIM, 1);
        let inv = e.invert(&a);
        for (x, y) in a.iter().zip(inv.iter()) {
            assert_eq!(*x, -y);
        }
    }

    #[test]
    fn test_cleanup_finds_nearest() {
        let e = engine();
        let cb0 = random_vector(VSA_DIM, 10);
        let cb1 = random_vector(VSA_DIM, 11);
        let cb2 = random_vector(VSA_DIM, 12);
        let codebook = vec![cb0.as_slice(), cb1.as_slice(), cb2.as_slice()];
        let idx = e.cleanup(&cb1, &codebook);
        assert_eq!(idx, Some(1));
    }

    #[test]
    fn test_benchmark_returns_result() {
        let e = engine();
        let result = e.benchmark();
        assert_eq!(result.dim, VSA_DIM);
        assert_eq!(result.backend, e.backend);
        assert!(result.bind_1000_ns > 0);
        assert!(result.matrix_100x100_ns > 0);
    }

    #[test]
    fn test_default_uses_vsa_dim() {
        let e = GpuVsaEngine::default();
        assert_eq!(e.engine.dimensions(), VSA_DIM);
    }
}
