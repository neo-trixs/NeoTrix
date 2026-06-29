//! SelfCode — 自代码生成子系统 (Phase 3)
//!
//! 核心职责:
//!   1. 记录所有代码变更 (EditHistoryTracker)
//!   2. 管理代码变换模板 (CodeTemplateRegistry)
//!   3. 从编辑历史提取模式 (PatternExtractor)
//!   4. 基于模式 + 模板生成代码 (SelfCodeWriter)
//!   5. 安全应用 + 回滚 (SafeCodeApplier)
//!
//! 设计原则:
//!   - 零 LLM 依赖: 所有代码生成基于已有历史 + 确定性模板
//!   - 增量学习: 每次成功编辑都存入 history, 下次可用
//!   - 回滚安全: git-based 保护, 失败自动回退

pub mod agentic_reasoning;
pub mod code_review_pipeline;
pub mod code_writer;
pub mod compatibility_agent;
pub mod edit_history;
pub mod pattern_extractor;
pub mod pipeline_autofixer;
pub mod review_aggregator;
pub mod review_orchestrator;
pub mod safe_applier;
pub mod semantic_entropy;
pub mod template_registry;
pub mod test_agent;
pub mod ultra_review;
pub mod verifier_stage;

pub use agentic_reasoning::{
    AgenticCodeReasoner, ExecutionTrace, FormalConclusion, PremiseCategory, ReasoningStep,
    SemiFormalPremise, SemiFormalTemplate, Verdict,
};
pub use code_writer::{CodeContentEntropy, CodeGenRequest, CodeGenResult, SelfCodeWriter};
pub use edit_history::EditHistoryTracker;
pub use pattern_extractor::PatternExtractor;
pub use pipeline_autofixer::PipelineAutoFixer;
pub use safe_applier::SafeCodeApplier;
pub use semantic_entropy::{
    EditContext, EntropyAction, EntropyRecord, SemanticEntropy, SemanticEntropyGate, TrendDirection,
};
pub use template_registry::{CodeTemplate, CodeTemplateRegistry, TemplateCategory};
pub use verifier_stage::{SafeWriteGate, VerifierStage};

pub use code_review_pipeline::{
    load_rules_for_language, CodeReviewPipeline, CommentResolver, DiffHunk, DiffLine, DiffLineType,
    DiffParser, DiffStatus, IssueCategory, IssueSeverity, LayeredRuleResolver, PathRule,
    RelocationRequest, ResolvedRule, ReviewCmdConfig, ReviewComment, ReviewFileDiff, ReviewResult,
    ReviewSession, RuleEntry,
};
pub use compatibility_agent::CompatibilityAgent;
pub use review_aggregator::{AgentReviewResult, AggregatedReviewReport, ReviewAggregator};
pub use review_orchestrator::ReviewOrchestrator;
pub use test_agent::TestEdgeCaseAgent;
pub use ultra_review::*;
