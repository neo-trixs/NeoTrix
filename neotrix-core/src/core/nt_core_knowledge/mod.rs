pub mod error;
pub use error::KnowledgeError;

mod activation;
pub mod fringe_mix;
pub mod hubness_detector;
mod sources;
mod tracker;
mod types;
mod vectors_group_a;
mod vectors_group_b;
pub mod versioning;

pub use activation::{
    ActivationPolicy, CascadeSelector, KSActivationEngine, KsLifecycle, RegisteredSource,
};
pub use fringe_mix::FringeMixStrategy;
pub use hubness_detector::HubnessDetector;
pub use tracker::*;
pub use types::*;
pub use versioning::{KnowledgeVersion, StalenessLevel, VersionManager};
pub mod entity_resolver;
pub mod keyword_lexicon;
// okf_exporter moved to neotrix/nt_io_provider/okf_exporter.rs
pub mod entity_extractor;
pub mod progress_aware_rag;
pub mod self_inspect;
pub mod semantic_compressor;

pub use entity_extractor::EntityExtractor;
pub use entity_resolver::EntityResolver;
pub use keyword_lexicon::KeywordLexicon;
pub use self_inspect::SelfInspectable;
pub mod behavioral_personality;
pub mod bookmark;
pub mod storage_coordinator;
pub mod atomic_fact;
pub mod evidence;
pub mod evidence_inspector;
pub mod execution_trace;
pub mod graph_r1;
pub mod hypergraph;
pub mod knowledge_routing;
pub mod multimodal_storyteller;
pub mod spread_activation;
pub mod system_card;
pub mod vsa_vocabulary;
pub mod osint;
pub use execution_trace::*;
pub mod forgetting_strategy;
pub use evidence_inspector::{
    Claim, ClaimId, EvidenceInspector, EvidenceVerificationResult, VerifiabilityGate,
    VerificationStatus,
};
pub use forgetting_strategy::{
    ebbinghaus_decay, importance_score, ForgettingReport, ForgettingStrategy,
};
pub use multimodal_storyteller::{
    Angle, AngleSelector, Audience, DataSet, Modality, ModalityPlanner, MultimodalStoryteller,
    Story, StoryPlan, StoryRenderer, StorySection,
};
pub use system_card::SystemCardGenerator;
pub use vsa_vocabulary::{SemanticPattern, VsaVocabulary};
// belief_revision — was unstable module, removed
// formal_inspect — was unstable module, removed
// graph_rl — was unstable module, removed
// metta_rewrite — was unstable module, removed
// ntsseg_compaction — was unstable module, removed
// prediction_signals — was unstable module, removed
// progress_rag — was unstable module, removed
// provenance — was unstable module, removed
// sparql — was unstable module, removed
// synthesizer — was unstable module, removed
pub mod research_kg;
pub use research_kg::{DocumentJob, ForceGraph, KgEdge, KgNode, ResearchKnowledgeGraph};

// mock_source — was unstable module, removed
