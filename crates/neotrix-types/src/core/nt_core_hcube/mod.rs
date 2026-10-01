pub mod axis;
pub mod coord;
pub mod cube;
pub mod gap;
pub mod vsa;

#[cfg(feature = "simd-vsa")]

pub use vsa::{VsaBackend, VSAEngine};

#[cfg(feature = "simd-vsa")]
// 2026-09-30: vsa_holon 冻结镜像已删（真身在 neotrix-core l2_perception/nt_core_hcube）。

pub fn create_backend(dim: usize) -> Box<dyn VsaBackend> {
    #[cfg(feature = "simd-vsa")]
    {
        Box::new(HolonBackend::new(dim))
    }
    #[cfg(not(feature = "simd-vsa"))]
    {
        Box::new(VSAEngine::new(dim))
    }
}
