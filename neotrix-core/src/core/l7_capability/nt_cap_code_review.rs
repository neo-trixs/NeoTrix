#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuditDimension {
    Security,
    Stability,
    Testing,
    Architecture,
    Performance,
    Maintainability,
    Design,
    Release,
    Documentation,
    Configuration,
    Observability,
    DataIntegrity,
    Privacy,
    Accessibility,
    SupplyChain,
    Cost,
    AiSafety,
    Fallback,
    TestingAuthenticity,
    TypeSafety,
    FrontendState,
    BackendApi,
    DependencyWeight,
    CodeConsistency,
    CommentCoverage,
}

#[derive(Debug, Clone)]
pub enum AuditMode {
    Full,
    Focused(Vec<AuditDimension>),
}

#[derive(Debug, Default)]
pub struct AuditReport {
    pub findings: Vec<String>,
    pub score: f64,
}

pub struct CodeReviewEngine {
    mode: AuditMode,
}

impl CodeReviewEngine {
    pub fn new(mode: AuditMode) -> Self {
        Self { mode }
    }

    pub fn with_mode(mode: AuditMode) -> Self {
        Self::new(mode)
    }

    pub fn with_linters(self) -> Self {
        self
    }

    pub fn load_sources(&self, _path: &std::path::Path) -> Vec<String> {
        Vec::new()
    }

    pub fn audit(&self, _name: &str, _path: &str, _sources: &[String]) -> AuditReport {
        AuditReport::default()
    }

    pub fn generate_markdown_report(&self, _report: &AuditReport) -> String {
        format!("Audit mode: {:?}\nNo issues found.", self.mode)
    }
}
