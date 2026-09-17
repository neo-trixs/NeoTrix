// Backward compatibility re-exports for gradual type migration.
// This file preserves old import paths while canonical types live in their
// respective modules (nt_trade_crm, nt_trade_email, nt_trade_pipeline).
//
// Usage: `use super::unified_types_compat::*;` (old code)
//        `use super::unified_types::*;` (new code, preferred)

// CRM backward compat
pub use super::nt_trade_crm::{
    Contact as ContactCompat,
    Company as CompanyCompat,
    CustomerProfile as CustomerProfileCompat,
    CustomerGrade as CustomerGradeCompat,
    CustomerSource as CustomerSourceCompat,
    CustomerStatus as CustomerStatusCompat,
    CrmSummary as CrmSummaryCompat,
    InteractionRecord as InteractionRecordCompat,
    InteractionType as InteractionTypeCompat,
};

// Email backward compat
pub use super::nt_trade_email::{
    EmailRecord as EmailRecordCompat,
    EmailTemplate as EmailTemplateCompat,
    EmailTemplateType as EmailTemplateTypeCompat,
    EmailTracking as EmailTrackingCompat,
    EmailAttachment as EmailAttachmentCompat,
    EmailStatus as EmailStatusCompat,
    EmailSummary as EmailSummaryCompat,
};

// Pipeline backward compat
pub use super::nt_trade_pipeline::{
    Deal as DealCompat,
    DealStage as DealStageCompat,
    DealSummary as DealSummaryCompat,
    LossReason as LossReasonCompat,
    PipelineStageSummary as PipelineStageSummaryCompat,
    PipelineView as PipelineViewCompat,
    StageChange as StageChangeCompat,
};
