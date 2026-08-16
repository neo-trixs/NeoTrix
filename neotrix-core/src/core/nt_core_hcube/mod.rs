pub mod axis;
pub mod coord;
pub mod cross_modal;
pub mod cube;
pub mod dream_consolidation;
pub mod fhrr_vsa;
pub mod gap;
pub mod hebbian_memory;
pub mod qfhrr_vsa;
pub mod reflection_consolidation;
pub mod vsa;
pub mod vsa_quantized;

pub mod ghrr_vsa;

pub mod aif;
pub mod e8_lattice;
pub mod octonion;
pub mod topology;

pub mod latent_recurrent;

pub mod bayesian_experiment;
#[cfg(feature = "simd-vsa")]
pub mod vsa_holon;

pub use fhrr_vsa::{
    bind, bundle, bundle_two, cleanup, cleanup_always, encode_scalar, permute, random_vector,
    random_vector_dim, similarity, FhrrHyperCube, FhrrVector, FHRR_DIM,
};

pub use ghrr_vsa::{
    ghrr_bind_dir, ghrr_bundle, ghrr_bundle_two, ghrr_cleanup, ghrr_cleanup_always, ghrr_permute,
    ghrr_random_vector, ghrr_random_vector_dim, ghrr_similarity, ghrr_unbind_dir,
    GhrrDiffusionConfig, GhrrDiffusionResult, GhrrHyperCube, GhrrVector, GHRR_DIM, GHRR_ETA,
};

pub use qfhrr_vsa::{
    encode_scalar_qfhrr, fhrr_to_qfhrr, qbind, qbundle, qfhrr_to_fhrr, qsimilarity, qunbind,
    random_qfhrr, QFhrrDiffusionConfig, QFhrrDiffusionResult, QuantizedFhrrHyperCube,
    QuantizedFhrrVector, QFHRR_DIM, QFHRR_LEVELS,
};

pub use vsa::{VSAEngine, VsaBackend};

#[cfg(feature = "simd-vsa")]
pub use vsa_holon::HolonBackend;

pub use e8_lattice::{E8Encoded, E8Lattice, E8Root};
pub use hebbian_memory::{HebbianGraph, HebbianRetrievalResult, HebbianStats};
pub use octonion::{Octonion, OctonionEngine};
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
