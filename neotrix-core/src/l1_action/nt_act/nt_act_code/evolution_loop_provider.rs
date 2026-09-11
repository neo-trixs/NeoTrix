//! L1-local trait abstraction for evolution loop providers.
//!
//! Breaks the upward L1→L5 dependency by defining the trait and types here in L1.
//! L5's `EvolutionLoop` implements this trait via a bridge adapter.

/// Detected code issue (L1-local, minimal subset of L5 Issue).
#[derive(Debug, Clone)]
pub struct Issue {
    pub issue_type: String,
    pub file: Option<String>,
    pub description: String,
}

/// L1-local action plan — mirrors L5 `self_diagnose::ActionPlan` variants.
#[derive(Debug, Clone)]
pub enum DiagnoseActionPlan {
    AutoFix(String),
    ManualReview(String),
    Skip(String),
    AddTestStub { file: String },
    NoAction { reason: String },
    RunCargoFix,
    SplitLargeFile { file: String },
    ReviewUnsafe { file: String },
    ReplaceUnwrap { file: String },
    RemoveTodo { file: String },
    HumanDecision { reason: String, options: Vec<String> },
}

/// A prioritized diagnostic issue with score and action plan.
#[derive(Debug, Clone)]
pub struct PrioritizedIssue {
    pub issue: Issue,
    pub score: f64,
    pub plan: DiagnoseActionPlan,
}

/// Minimal project health snapshot (L1-local, simplified view for the provider trait).
#[derive(Debug, Clone, Default)]
pub struct ProjectSnapshotLite {
    pub modules: Vec<String>,
    pub health_score: f64,
    pub timestamp: String,
}

/// L1 trait for evolution loop providers — L5 implements this for its `EvolutionLoop`.
pub trait EvolutionLoopProvider {
    fn get_snapshot(&self) -> ProjectSnapshotLite;
    fn self_diagnose(&mut self) -> (Vec<String>, Vec<PrioritizedIssue>);
    fn on_fix_applied(&mut self);
}
