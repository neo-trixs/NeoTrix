pub mod axis;
pub mod coord;
pub mod cube;
pub mod gap;
pub mod vsa;
pub mod vsa_quantized;
pub mod cross_modal;

pub mod octonion;
pub mod e8_lattice;
pub mod topology;

#[cfg(feature = "simd-vsa")]
pub mod vsa_holon;

pub use vsa::{VsaBackend, VSAEngine};
pub use vsa_quantized::QuantizedVSA;
pub use cross_modal::CrossModalAligner;

#[cfg(feature = "simd-vsa")]
pub use vsa_holon::HolonBackend;

pub use octonion::{Octonion, OctonionEngine};
pub use e8_lattice::{E8Encoded, E8Lattice, E8Root};
pub use topology::{BettiNumbers, PersistentHomology, PointCloud};

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
