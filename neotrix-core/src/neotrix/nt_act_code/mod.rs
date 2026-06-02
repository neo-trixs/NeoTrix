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
pub mod code_writer;
pub mod edit_history;
pub mod pattern_extractor;
pub mod pipeline_autofixer;
pub mod safe_applier;
pub mod semantic_entropy;
pub mod template_registry;

pub use agentic_reasoning::{
    AgenticCodeReasoner, ReasoningStep, SemiFormalTemplate,
    SemiFormalPremise, PremiseCategory, ExecutionTrace,
    FormalConclusion, Verdict,
};
pub use code_writer::{CodeGenRequest, CodeGenResult, CodeContentEntropy, SelfCodeWriter};
pub use edit_history::EditHistoryTracker;
pub use pattern_extractor::PatternExtractor;
pub use pipeline_autofixer::PipelineAutoFixer;
pub use safe_applier::SafeCodeApplier;
pub use semantic_entropy::{SemanticEntropy, SemanticEntropyGate, EntropyAction, EntropyRecord, EditContext, TrendDirection};
pub use template_registry::{CodeTemplate, CodeTemplateRegistry, TemplateCategory};
