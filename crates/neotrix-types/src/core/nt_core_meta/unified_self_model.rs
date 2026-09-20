//! Unified SelfModel types — consolidates 4 scattered SelfModel variants.
//!
//! Three distinct roles:
//! - `StaticIdentityModel` — "我是什么？" 结构身份快照 (模块/文件/依赖图)
//! - `DynamicPerformanceModel` — "我表现如何？" 性能估算 (能力/不确定性/疲劳)
//! - `ValueFunctionModel` — "我重视什么？" 价值评估 (身份/目标/权重)

use std::collections::HashMap;

// ════════════════════════════════════════════════════════════════
// Shared sub-types
// ════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub path: String,
    pub file_count: usize,
    pub total_lines: usize,
    pub test_count: usize,
    pub has_tests: bool,
    pub unsafe_count: usize,
    pub unwrap_count: usize,
    pub todo_count: usize,
    pub public_api_count: usize,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub module: String,
    pub lines: usize,
    pub is_test_file: bool,
    pub has_unsafe: bool,
    pub has_todos: bool,
    pub pub_fns: usize,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct DepEdge {
    pub from: String,
    pub to: String,
    pub kind: DepKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DepKind {
    ModuleUse,
    TraitImpl,
    FunctionCall,
}

#[derive(Debug, Clone)]
pub struct DepGraph {
    pub edges: Vec<DepEdge>,
}

impl DepGraph {
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    pub fn find_cycles(&self) -> Vec<Vec<String>> {
        let edges: Vec<(String, String)> = self.edges.iter()
            .map(|e| (e.from.clone(), e.to.clone()))
            .collect();
        let mut cycles = Vec::new();
        let node_set: std::collections::HashSet<String> = edges.iter()
            .flat_map(|(f, t)| [f.clone(), t.clone()])
            .collect();
        let nodes: Vec<&str> = node_set.iter().map(|s| s.as_str()).collect();

        for start in &nodes {
            let mut visited = std::collections::HashSet::new();
            let mut path = Vec::new();
            if Self::cycle_dfs(&edges, start, start, &mut visited, &mut path) {
                cycles.push(path.clone());
            }
        }
        cycles.dedup();
        cycles
    }

    fn cycle_dfs(
        edges: &[(String, String)], current: &str, target: &str,
        visited: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        if !visited.insert(current.to_string()) {
            return false;
        }
        path.push(current.to_string());

        for (from, to) in edges {
            if from == current {
                if to == target && path.len() > 1 {
                    return true;
                }
                if Self::cycle_dfs(edges, to, target, visited, path) {
                    return true;
                }
            }
        }

        path.pop();
        false
    }

    pub fn orphans(&self) -> Vec<String> {
        let mut deps_from = std::collections::HashSet::new();
        let mut deps_to = std::collections::HashSet::new();
        for e in &self.edges {
            deps_from.insert(e.from.clone());
            deps_to.insert(e.to.clone());
        }
        deps_from.into_iter().filter(|m| !deps_to.contains(m)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct ComponentNode {
    pub name: String,
    pub path: String,
    pub layer: u8,
    pub file_count: usize,
    pub lines: usize,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ComponentMap {
    pub nodes: Vec<ComponentNode>,
    pub edges: Vec<(String, String, String)>,
}

impl ComponentMap {
    pub fn find_orphan_components(&self) -> Vec<&ComponentNode> {
        let referenced: std::collections::HashSet<&str> = self.edges.iter()
            .flat_map(|(a, b, _)| [a.as_str(), b.as_str()])
            .collect();
        self.nodes.iter().filter(|n| !referenced.contains(n.name.as_str())).collect()
    }

    pub fn find_hubs(&self, threshold: usize) -> Vec<&str> {
        let mut degree: HashMap<&str, usize> = HashMap::new();
        for (a, b, _) in &self.edges {
            *degree.entry(a.as_str()).or_insert(0) += 1;
            *degree.entry(b.as_str()).or_insert(0) += 1;
        }
        degree.into_iter()
            .filter(|(_, d)| *d > threshold)
            .map(|(n, _)| n)
            .collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct TestCoverage {
    pub total_tests: usize,
    pub passing: usize,
    pub failing: usize,
    pub ignored: usize,
    pub modules_with_tests: Vec<String>,
    pub modules_without_tests: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CompilationHealth {
    pub errors: usize,
    pub warnings: usize,
    pub features_tested: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TechDebtInventory {
    pub items: Vec<TechDebtItem>,
    pub total_count: usize,
}

#[derive(Debug, Clone)]
pub struct TechDebtItem {
    pub file: String,
    pub line: Option<usize>,
    pub kind: TechDebtKind,
    pub description: String,
    pub severity: DebtSeverity,
    pub suggested_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TechDebtKind {
    UnwrapCall,
    LargeFile,
    MissingTests,
    UnsafeBlock,
    DeadCode,
    TodoComment,
    CircularDependency,
    OrphanModule,
    LargePublicApi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DebtSeverity {
    Critical = 3,
    Major = 2,
    Minor = 1,
    Cosmetic = 0,
}

#[derive(Debug, Clone)]
pub struct EvolutionEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub kind: EventKind,
    pub description: String,
    pub affected_modules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
    ModuleAdded,
    ModuleRefactored,
    BugFixed,
    FeatureAdded,
    TechDebtResolved,
    WeaknessDetected,
    EvolutionPlanned,
    MetaCognitionUpdated,
}

// ════════════════════════════════════════════════════════════════
// 1. StaticIdentityModel — "我是什么？"
// ════════════════════════════════════════════════════════════════

/// Complete representation of the project's structural identity.
/// This is the "self-image" — what the system knows about itself.
#[derive(Debug, Clone)]
pub struct StaticIdentityModel {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub modules: Vec<ModuleInfo>,
    pub files: Vec<FileInfo>,
    pub dep_graph: DepGraph,
    pub component_map: ComponentMap,
    pub test_coverage: TestCoverage,
    pub compilation: CompilationHealth,
    pub tech_debt: TechDebtInventory,
    pub evolution_history: Vec<EvolutionEvent>,
}

impl Default for StaticIdentityModel {
    fn default() -> Self {
        Self::new()
    }
}

impl StaticIdentityModel {
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            modules: Vec::new(),
            files: Vec::new(),
            dep_graph: DepGraph { edges: Vec::new() },
            component_map: ComponentMap { nodes: Vec::new(), edges: Vec::new() },
            test_coverage: TestCoverage::default(),
            compilation: CompilationHealth::default(),
            tech_debt: TechDebtInventory { items: Vec::new(), total_count: 0 },
            evolution_history: Vec::new(),
        }
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn total_lines(&self) -> usize {
        self.modules.iter().map(|m| m.total_lines).sum()
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn test_count(&self) -> usize {
        self.test_coverage.total_tests
    }

    pub fn modules_without_tests(&self) -> Vec<&ModuleInfo> {
        self.modules.iter().filter(|m| !m.has_tests).collect()
    }

    pub fn modules_with_high_unsafe(&self, threshold: usize) -> Vec<&ModuleInfo> {
        self.modules.iter().filter(|m| m.unsafe_count > threshold).collect()
    }

    pub fn tech_debt_by_severity(&self, severity: DebtSeverity) -> Vec<&TechDebtItem> {
        self.tech_debt.items.iter().filter(|i| i.severity == severity).collect()
    }

    pub fn register_evolution(&mut self, event: EvolutionEvent) {
        self.evolution_history.push(event);
    }

    pub fn latest_events(&self, n: usize) -> &[EvolutionEvent] {
        let len = self.evolution_history.len();
        let start = len.saturating_sub(n);
        &self.evolution_history[start..]
    }
}

// ════════════════════════════════════════════════════════════════
// 2. DynamicPerformanceModel — "我表现如何？"
// ════════════════════════════════════════════════════════════════

/// Number of observed-behavior samples retained for self-error estimation.
pub const SELF_HISTORY: usize = 32;

/// Current self-estimate produced by the dynamic self-model.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SelfState {
    /// Aggregate capability estimate (0..1) — how well the system is performing.
    pub capability: f64,
    /// Uncertainty estimate (0..1) — 0 = confident, 1 = extremely uncertain.
    pub uncertainty: f64,
    /// Fatigue level (0..1) — accumulated load vs. available budget.
    pub fatigue: f64,
    /// Self-model error vs. observed behavior (0..1). Higher = worse self-model.
    pub self_error: f64,
}

impl Default for SelfState {
    fn default() -> Self {
        Self {
            capability: 0.5,
            uncertainty: 0.5,
            fatigue: 0.0,
            self_error: 0.0,
        }
    }
}

/// Dynamic performance self-model.
///
/// Tracks a rolling record of (predicted capability, observed outcome) pairs and
/// updates a running estimate of capability, uncertainty, and fatigue each tick.
/// The self-error term is the discrepancy between the model's own estimate and
/// the observed outcome — used as an intrinsic reward signal.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DynamicPerformanceModel {
    /// Current self-estimate.
    pub state: SelfState,
    /// Rolling predicted-capability history.
    pub predicted_history: std::collections::VecDeque<f64>,
    /// Rolling observed-outcome history.
    pub observed_history: std::collections::VecDeque<f64>,
    /// Running fatigue budget (0..1).
    pub fatigue: f64,
    /// Number of updates performed.
    pub updates: u64,
    /// Recent self-model error (for telemetry).
    pub last_self_error: f64,
    /// Learning rate for the fatigue/capability estimator.
    pub lr: f64,
}

impl Default for DynamicPerformanceModel {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicPerformanceModel {
    pub fn new() -> Self {
        Self {
            state: SelfState::default(),
            predicted_history: std::collections::VecDeque::with_capacity(SELF_HISTORY),
            observed_history: std::collections::VecDeque::with_capacity(SELF_HISTORY),
            fatigue: 0.0,
            updates: 0,
            last_self_error: 0.0,
            lr: 0.1,
        }
    }

    /// One self-observation tick.
    pub fn tick(&mut self, workspace_signal: f64, load_delta: f64, meta_alarm: usize) -> SelfState {
        self.updates += 1;

        let prior = self.state.capability;
        let observed = workspace_signal.clamp(0.0, 1.0);

        self.predicted_history.push_back(prior);
        self.observed_history.push_back(observed);
        while self.predicted_history.len() > SELF_HISTORY {
            self.predicted_history.pop_front();
            self.observed_history.pop_front();
        }

        let n = self.predicted_history.len() as f64;
        let err_sum: f64 = self.predicted_history.iter()
            .zip(self.observed_history.iter())
            .map(|(p, o)| (p - o).abs())
            .sum();
        let self_error = if n > 0.0 { err_sum / n } else { 0.0 };
        self.last_self_error = self_error;

        self.state.capability += self.lr * (observed - self.state.capability);
        self.fatigue = (self.fatigue + self.lr * load_delta - 0.005).clamp(0.0, 1.0);
        self.state.fatigue = self.fatigue;

        let u = (0.1 + self_error * 0.5 + (meta_alarm as f64).min(3.0) * 0.1).clamp(0.0, 1.0);
        self.state.uncertainty = u;
        self.state.self_error = self_error;
        self.state
    }

    pub fn self_reward(&self) -> f64 {
        -self.last_self_error
    }

    pub fn combined_intrinsic_reward(&self) -> f64 {
        -self.last_self_error - self.fatigue * 0.3
    }

    pub fn current(&self) -> SelfState {
        self.state
    }

    pub fn reset(&mut self) {
        self.state = SelfState::default();
        self.predicted_history.clear();
        self.observed_history.clear();
        self.fatigue = 0.0;
        self.updates = 0;
        self.last_self_error = 0.0;
    }
}

// ════════════════════════════════════════════════════════════════
// 3. ValueFunctionModel — "我重视什么？"
// ════════════════════════════════════════════════════════════════

/// Single value dimension weight (used by the value function to evaluate actions).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ValueWeight {
    /// Dimension name (e.g. "coherence" / "safety" / "growth").
    pub dimension: String,
    /// Weight [0,1], sum of all weights conventionally equals 1.0.
    pub weight: f64,
}

/// Value function self-model — evaluates actions against self-defined value weights.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValueFunctionModel {
    /// System identity label.
    pub identity: String,
    /// Current self-goals (candidate behavior changes are evaluated against these).
    pub goals: Vec<String>,
    /// Value weights driving the linear heuristic.
    pub value_weights: Vec<ValueWeight>,
    /// Iteration count (SEAL `update` call count).
    pub revision: u64,
}

impl Default for ValueFunctionModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ValueFunctionModel {
    pub fn new() -> Self {
        Self {
            identity: "neotrix-core".to_string(),
            goals: Vec::new(),
            value_weights: vec![
                ValueWeight { dimension: "coherence".to_string(), weight: 1.0 / 3.0 },
                ValueWeight { dimension: "safety".to_string(), weight: 1.0 / 3.0 },
                ValueWeight { dimension: "growth".to_string(), weight: 1.0 / 3.0 },
            ],
            revision: 0,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Value function: evaluate an action's alignment with self-value weights.
    pub fn value_function(&self, action: &str) -> f64 {
        if action.is_empty() {
            return 0.0;
        }
        let action_lower = action.to_lowercase();

        let dimension_scores: Vec<f64> = self.value_weights.iter().map(|vw| {
            let keywords = match vw.dimension.as_str() {
                "coherence" => vec!["consistent", "aligned", "unified", "coherent", "integrate"],
                "safety" => vec!["safe", "secure", "protect", "guard", "defend", "harden"],
                "growth" => vec!["learn", "evolve", "improve", "grow", "adapt", "optimize"],
                "efficiency" => vec!["fast", "efficient", "optimize", "cache", "batch", "parallel"],
                "autonomy" => vec!["autonomous", "self", "independent", "decide", "choose"],
                _ => vec![],
            };

            let matches = keywords.iter()
                .filter(|kw| action_lower.contains(*kw))
                .count();
            if keywords.is_empty() {
                0.0
            } else {
                (matches as f64) / (keywords.len() as f64)
            }
        }).collect();

        let total_weight: f64 = self.value_weights.iter().map(|w| w.weight).sum();
        if total_weight <= 0.0 {
            return 0.5;
        }

        let weighted_sum: f64 = dimension_scores.iter()
            .zip(self.value_weights.iter())
            .map(|(score, vw)| score * vw.weight)
            .sum();

        (weighted_sum / total_weight).clamp(0.0, 1.0)
    }
}

// ════════════════════════════════════════════════════════════════
// Type aliases for backward compatibility
// ════════════════════════════════════════════════════════════════

/// Backward-compatible alias — the original "SelfModel" was the static identity one.
pub type SelfModel = StaticIdentityModel;
