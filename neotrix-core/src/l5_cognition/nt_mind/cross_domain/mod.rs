#![forbid(unsafe_code)]

//! 跨域迁移框架
//!
//! 基于 Agent KB 论文的 Reason-Retrieve-Refine 模式，
//! 提供完整的跨域知识迁移能力。
//!
//! 模块结构：
//! - reason_retrieve_refine: Reason-Retrieve-Refine 管线
//! - disagreement_gate: 分歧门控（防止知识干扰）
//! - entity_mapping: 实体映射（跨框架兼容）
//! - cross_domain_transfer: 跨域迁移主模块

pub mod cross_domain_transfer;
pub mod disagreement_gate;
pub mod entity_mapping;
pub mod reason_retrieve_refine;

// Re-exports
pub use reason_retrieve_refine::{
    Constraint, ConstraintType, DomainMapping, KnowledgeItem, KnowledgeType, QueryIntent,
    ReasonResult, ReasonRetrieveRefinePipeline, ReasoningStepType, RefineResult, RefineStep,
    RefinedKnowledge, RetrievalQuery, RetrieveResult, RrrPipelineConfig, TaskType,
    TransferContext, TransferError,
};

pub use disagreement_gate::{
    DisagreementDetection, DisagreementGate, DisagreementGateConfig, DisagreementPattern,
    DisagreementSeverity, DisagreementStats, DisagreementType, GateDecision, GateEvaluation,
    MatchCondition, ResolutionStrategy,
};

pub use entity_mapping::{
    AttributeMapping, AttributeType, EntityAttribute, EntityDefinition, EntityMapper,
    EntityMapping, EntityMappingConfig, EntityType, MappingResult, MappingStats, MappingType,
    NamingConvention, TransformationRule, TransformationType,
};

pub use cross_domain_transfer::{
    CrossDomainTransferConfig, CrossDomainTransferFramework, RefinedKnowledgeWithMapping,
    TransferHistoryStats, TransferResult, TransferStats, TransferStatus, TransferTask,
};
