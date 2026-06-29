// AUTO-GENERATED MODULE DECLARATIONS — all sub-modules
pub mod adapt_encoder;
pub mod adaptive_encoder;
pub mod attractor_basin;
pub mod axis;
pub mod coord;
pub mod cortical_rnn;
pub mod cross_modal;
pub mod cube;
pub mod diff_vsa;
pub mod dom_vsa;
pub mod dream_consolidation;
pub mod e8_cortical;
pub mod e8_field;
pub mod e8_lagrangian;
pub mod e8_lattice;
pub mod e8_quantized;
pub mod e8_topological_defects;
pub mod ebbinghaus_decay;
pub mod efe_curiosity_bridge;
pub mod error;
pub mod fpe;
pub mod gap;
pub mod geometric_ssm;
pub mod go_cls_gate;
pub mod hippocampal_trace;
pub mod hlb_bind;
pub mod hopfield_network;
pub mod interaction_trace;
pub mod koopman_operator;
pub mod kroneker_cleanup;
pub mod sindy_engine;
pub mod stream_pipeline;
pub mod linear_code;
pub mod linear_code_vsa;
pub mod magma_memory;
pub mod memory_activation;
pub mod mhn_pattern_separation;
pub mod multi_head_resonator;
pub mod multi_modal_aligner;
pub mod nag_vsa;
pub mod narrative_hypercube_bridge;
pub mod narrative_vsa_binding;
pub mod octonion;
pub mod physics_commonsense;
pub mod primitives;
pub mod qfhrr_vsa;
pub mod resonator_decoder;
pub mod rotation_bind;
pub mod selfref_meta;
pub mod sign_flip_vsa;
pub mod skill_compiler;
pub mod sm2_scheduler;
pub mod sparse_bench;
pub mod sparse_hypercube;
pub mod sparse_vsa;
pub mod sparse_vsa_index;
pub mod spatial_scene;
pub mod spectral_forcing;
pub mod spectral_nsr;
pub mod spectral_vsa;
pub mod subspace;
pub mod thdc_encoder;
pub mod topo_cube;
pub mod topology;
pub mod trigram_index;
pub mod visual_embedding_frontend;
pub mod visual_rag_index;
pub mod vsa;
pub mod vsa_bridge;
pub mod vsa_gpu;
#[cfg(feature = "simd-vsa")]
pub mod vsa_holon;
pub mod vsa_hrr;
pub mod vsa_multi_model;
pub mod vsa_quantized;
pub mod vsa_runtime_ir;
pub mod vsa_spatial_encoder;
pub mod vsa_subspace_carving;
pub mod vsa_vector;
pub mod wave_geometric;

pub use cross_modal::CrossModalAligner;

#[cfg(feature = "simd-vsa")]
pub use vsa_holon::HolonBackend;

pub use e8_lattice::{E8Encoded, E8Lattice, E8Root};
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

// Additional re-exports needed by neotrix/ modules
pub use adaptive_encoder::{AdaptiveVsaEncoder, EncoderMode};
pub use cortical_rnn::{CerebellumResonator, CortexAdaptive, ResonanceMode, CBRNN};
pub use e8_cortical::{
    e8_cortical_vsa_transform, CorticalCoord, E8CorticalMapping, CORTICAL_NEURON_COUNT,
};
pub use e8_field::{E8FieldSolver, FieldSolverConfig};
pub use e8_lagrangian::{E8FieldIntegrator, E8Lagrangian, Lattice3D, PDESolver};
pub use e8_topological_defects::{
    DefectConfig, E8ParticleSpectrum, E8TopologicalDefects, ForceType, HalfIntegerSpin,
    TopologicalCharge, WeylOrbit,
};
pub use efe_curiosity_bridge::{global_efe_bridge, step_efe_bridge};
pub use geometric_ssm::GeometricSSM;
pub use hlb_bind::HLBBind;
pub use kroneker_cleanup::KronekerCodebook;
pub use memory_activation::{global_memory_activation, step_memory_activation};
pub use multi_head_resonator::MultiHeadResonator;
pub use nag_vsa::{batch_nag_bundle, gated_nag_bundle, nag_bundle, nag_similarity, normalize};
pub use physics_commonsense::PhysicsCommonsense;
pub use rotation_bind::{RotationBind, RotationCodebook};
pub use selfref_meta::{global_selfref_meta, step_selfref_meta};
pub use sign_flip_vsa::SignFlipVsa;
pub use sparse_hypercube::SparseHyperCube;
pub use sparse_vsa::SparseBinaryVSA;
pub use sparse_vsa_index::SparseVsaInvertedIndex;
pub use spatial_scene::SpatialSceneEngine;
pub use spectral_nsr::{
    BandPassExpert, FrequencyBand, GraphLaplacian, HighPassExpert, LowPassExpert,
    MoSpectralExperts, SpectralExpert, SpectralFilter, SpectralNSR, SpectralRule,
};
pub use spectral_vsa::SpectralVSA;
pub use stream_pipeline::StreamPipeline;
pub use thdc_encoder::TrainableVsaEncoder;
pub use trigram_index::TrigramInvertedIndex;
pub use visual_embedding_frontend::{VisualEmbeddingFrontend, VisualEmbeddingModel};
pub use visual_rag_index::{
    IndexBackend, IndexedDocument, SearchResult, VisualIndexConfig, VisualRAGIndex,
};
pub use vsa::BinaryVsaBackend;
pub use vsa::{VSAEngine, VsaBackend};
pub use vsa_hrr::HrrBackend;
pub use vsa_quantized::cosine_sim_u8;
pub use vsa_quantized::pack_binary;
pub use vsa_quantized::hamming_distance_packed;
pub use vsa_quantized::similarity_packed;
pub use vsa_quantized::QuantizedVSA;
pub use vsa_quantized::VSA_DIM;
pub use vsa_spatial_encoder::{SpatialAttentionGate, VSASpatialEncoder, Vec3D};
pub use vsa_vector::MapVsaBackend;
pub use vsa_vector::VsaError;
pub use vsa_vector::VsaVector;
pub use wave_geometric::{SpectralDenoiser, WaveGeometricEmbed, WaveGeometricVSA};
