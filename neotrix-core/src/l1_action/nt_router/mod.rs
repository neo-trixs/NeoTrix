//! Stub — deleted module, kept for CLI compilation only.
use std::sync::Mutex;
use lazy_static::lazy_static;

#[derive(Clone, Debug, Default)]
pub struct RouterStats {
    pub total_routes: u64,
    pub estimated_savings: f64,
    pub actual_cost: f64,
    pub flagship_cost: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TaskComplexity { Trivial, Simple, Moderate, Complex, Critical }

impl Default for TaskComplexity {
    fn default() -> Self { TaskComplexity::Moderate }
}

impl TaskComplexity {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trivial" => Some(Self::Trivial),
            "simple" => Some(Self::Simple),
            "moderate" => Some(Self::Moderate),
            "complex" => Some(Self::Complex),
            "critical" => Some(Self::Critical),
            _ => None,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Trivial => "trivial",
            Self::Simple => "simple",
            Self::Moderate => "moderate",
            Self::Complex => "complex",
            Self::Critical => "critical",
        }
    }
    pub fn classify(_text: &str, _ctx: &TaskContext) -> Self {
        Self::Moderate
    }
}

#[derive(Clone, Debug, Default)]
pub struct TaskContext {
    pub domain: String,
    pub complexity: TaskComplexity,
    pub priority: u8,
    pub prompt_length: usize,
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub mentions_files: bool,
    pub file_count: usize,
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub has_git_context: bool,
    pub keywords: Vec<String>,
}

impl TaskContext {
    pub fn new(text: &str) -> Self {
        Self { domain: text.to_string(), complexity: TaskComplexity::Moderate, priority: 5,
               prompt_length: text.len(), mentions_files: false, file_count: 0,
               has_git_context: false, keywords: Vec::new() }
    }
}

pub struct SmartRouter {
    pub enabled: bool,
    pub stats: RouterStats,
    rules: Vec<(TaskComplexity, String, String, f64, f64)>,
}

impl SmartRouter {
    pub fn load() -> Self { Self { enabled: true, stats: RouterStats::default(), rules: Vec::new() } }
    pub fn savings_report(&self) -> String {
        format!("Smart Router: {} | Routes: {} | Savings: ${:.4} | Cost: ${:.4} / ${:.4}",
            if self.enabled { "enabled" } else { "disabled" },
            self.stats.total_routes, self.stats.estimated_savings, self.stats.actual_cost, self.stats.flagship_cost)
    }
    pub fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    pub fn save(&self) -> Result<(), String> { Ok(()) }
    pub fn reset_stats(&mut self) { self.stats = RouterStats::default(); }
    pub fn stats(&self) -> &RouterStats { &self.stats }
    pub fn set_rule(&mut self, complexity: TaskComplexity, provider: &str, model: &str, cost_in: f64, cost_out: f64) {
        self.rules.push((complexity, provider.to_string(), model.to_string(), cost_in, cost_out));
    }
}

lazy_static! {
    pub static ref SMART_ROUTER: Mutex<SmartRouter> = Mutex::new(SmartRouter::load());
}
