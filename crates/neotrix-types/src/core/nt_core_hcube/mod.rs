pub mod axis;
pub mod coord;
pub mod cube;
pub mod gap;
pub mod vsa;

#[cfg(feature = "simd-vsa")]
pub mod vsa_holon;

pub use vsa::{VsaBackend, VSAEngine};

#[cfg(feature = "simd-vsa")]
pub use vsa_holon::HolonBackend;

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
