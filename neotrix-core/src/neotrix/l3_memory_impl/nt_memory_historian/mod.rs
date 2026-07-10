pub mod nt_evidence_types;
pub mod nt_evidence_store;
pub mod nt_evidence_api;
pub mod nt_evidence_factory_gate;
pub mod nt_evidence_hypothesis;
pub mod nt_evidence_credibility;
pub mod nt_evidence_temporal;
pub mod dmn_consolidation;

pub use nt_evidence_types::{
    era_center, haversine_km, BayesianLink, CalibrationResult, ConfidenceTier, ConflictResolution,
    ContradictionCategory, DatingMethodMeta, EvidenceCluster, EvidenceContradiction, EvidenceRecord,
    EvidenceStats, EvidenceTableSnapshot, FactoryGateConfig, ForgeryRisk, builtin_dating_methods,
    method_weight,
};
pub use nt_evidence_store::EvidenceStore;
pub use nt_evidence_api::{EvidenceApiState, build_ewhr_router};
pub use nt_evidence_factory_gate::EvidenceFactoryGate;
pub use nt_evidence_hypothesis::{
    AuditEntry, AuditTrail, Hypothesis, HypothesisNetwork, HypothesisStatus, SubjectiveOpinion,
    dempster_shafer_combine, weight_of_evidence,
};
pub use nt_evidence_credibility::{
    CredibilityAggregator, CustodyChain, ReviewStatus, SourceCredibility, SourceTier,
};
pub use nt_evidence_temporal::{
    AnachronismDetector, TemporalEvidenceTracker, TemporalRelation, TemporalTrend,
    TimelineReconstructor, TrendDirection, allen_relation,
};
pub use dmn_consolidation::{
    ConsolidationReport, DMNConsolidation, DmnConfig, DmnStats, FamiliarityWeightedRetrieval,
    LearnedGraphExecutor, MemoryItem, MemoryTier, ThreeTierStore,
};
