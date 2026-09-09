//! Evolution loop shared types — used by both L1 (pipeline_autofixer) and L5 (self_diagnose).


/// Action plan for auto-fixing detected issues.
#[derive(Debug, Clone)]
pub enum ActionPlan {
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

/// A prioritized issue with scoring and action plan.
#[derive(Debug, Clone)]
pub struct PrioritizedIssue {
    pub issue: Issue,
    pub score: f64,
    pub plan: ActionPlan,
    pub composite_score: f64,
    pub underlying_issue: CodeUnderlyingIssue,
}

/// Underlying code issue details.
#[derive(Debug, Clone)]
pub struct CodeUnderlyingIssue {
    pub file: String,
    pub line: usize,
    pub message: String,
}

/// Issue detected during self-diagnosis.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Issue {
    pub issue_type: IssueType,
    pub severity: u8,
    pub file: Option<String>,
    pub description: String,
    pub suggestion: String,
    pub auto_fixable: bool,
    pub cycle_discovered: u64,
}

/// Types of issues that can be detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum IssueType {
    LargeFile,
    MissingTests,
    ExcessUnsafe,
    ExcessUnwrap,
    TodoLeftovers,
    CompileWarning,
    HighFreeEnergy,
    LowPhi,
    UnusedImport,
    Other,
}

/// Project snapshot for diagnosis.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectSnapshot {
    pub total_files: usize,
    pub total_lines: usize,
    pub large_files: Vec<String>,
    pub modules_without_tests: Vec<String>,
    pub file_unsafe_hotspots: Vec<String>,
    pub unsafe_count: usize,
    pub unwrap_count: usize,
    pub todo_count: usize,
    pub compile_errors: usize,
    pub compile_warnings: usize,
    pub test_count: usize,
    pub test_failures: usize,
}

/// Provider trait for the evolution loop — L1 defines, L5 implements.
pub trait EvolutionLoopProvider {
    fn get_snapshot(&self) -> ProjectSnapshot;
    fn self_diagnose(&mut self) -> (Vec<String>, Vec<PrioritizedIssue>);
    fn on_fix_applied(&mut self);
}
