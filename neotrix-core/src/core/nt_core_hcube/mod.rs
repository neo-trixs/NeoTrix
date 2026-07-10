pub mod axis;
pub mod coord;
pub mod cube;
pub mod fhrr_vsa;
pub mod qfhrr_vsa;
pub mod gap;
pub mod vsa;
pub mod vsa_quantized;
pub mod dream_consolidation;
pub mod cross_modal;
pub mod reflection_consolidation;
pub mod hebbian_memory;

pub mod ghrr_vsa;

pub mod octonion;
pub mod e8_lattice;
pub mod topology;
pub mod aif;

#[cfg(feature = "simd-vsa")]
pub mod vsa_holon;

pub use fhrr_vsa::{
    FhrrHyperCube, FhrrVector, FHRR_DIM, bind, bundle, bundle_two, cleanup, cleanup_always,
    encode_scalar, permute, random_vector, random_vector_dim, similarity,
};

pub use ghrr_vsa::{
    GhrrHyperCube, GhrrVector, GHRR_DIM, GHRR_ETA, ghrr_bind_dir, ghrr_unbind_dir, ghrr_bundle,
    ghrr_bundle_two, ghrr_similarity, ghrr_permute, ghrr_random_vector, ghrr_random_vector_dim,
    ghrr_cleanup, ghrr_cleanup_always, GhrrDiffusionConfig, GhrrDiffusionResult,
};

pub use qfhrr_vsa::{
    QuantizedFhrrVector, QuantizedFhrrHyperCube, QFHRR_DIM, QFHRR_LEVELS,
    qbind, qunbind, qbundle, qsimilarity, QFhrrDiffusionConfig, QFhrrDiffusionResult,
    encode_scalar_qfhrr, random_qfhrr, fhrr_to_qfhrr, qfhrr_to_fhrr,
};

pub use vsa::{VsaBackend, VSAEngine};

#[cfg(feature = "simd-vsa")]
pub use vsa_holon::HolonBackend;

pub use octonion::{Octonion, OctonionEngine};
pub use e8_lattice::{E8Encoded, E8Lattice, E8Root};
pub use topology::{BettiNumbers, PersistentHomology, PointCloud};
pub use hebbian_memory::{HebbianGraph, HebbianRetrievalResult, HebbianStats};

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
