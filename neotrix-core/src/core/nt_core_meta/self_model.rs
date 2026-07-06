use std::collections::HashMap;

/// Complete representation of the project's current state.
/// This is the "self-image" — what the system knows about itself.
#[derive(Debug, Clone)]
pub struct SelfModel {
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

impl Default for SelfModel {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfModel {
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

#[derive(Debug, Clone, Default)]
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
pub struct DepGraph {
    pub edges: Vec<DepEdge>,
}

impl DepGraph {
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

/// Component-level architecture map for high-level understanding.
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
#[derive(Default)]
pub struct TestCoverage {
    pub total_tests: usize,
    pub passing: usize,
    pub failing: usize,
    pub ignored: usize,
    pub modules_with_tests: Vec<String>,
    pub modules_without_tests: Vec<String>,
}


#[derive(Debug, Clone)]
#[derive(Default)]
pub struct CompilationHealth {
    pub errors: usize,
    pub warnings: usize,
    pub features_tested: Vec<String>,
}


impl CompilationHealth {
    /// Run `cargo check --lib` and parse errors/warnings.
    /// Skips when compiled for tests to avoid lock contention with `cargo test`.
    pub fn check(project_root: &str) -> Self {
        let mut health = CompilationHealth::default();

        if cfg!(test) {
            return health;
        }

        let mut child = match std::process::Command::new("cargo")
            .args(["check", "--lib"])
            .current_dir(project_root)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                health.errors = 1;
                return health;
            }
        };

        let timeout = std::time::Duration::from_secs(120);
        let start = std::time::Instant::now();

        let output = loop {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    let out = child.wait_with_output();
                    break out.ok();
                }
                Ok(None) => {
                    if start.elapsed() > timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        health.errors = 1;
                        return health;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(_) => break None,
            }
        };

        if let Some(out) = output {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let stdout = String::from_utf8_lossy(&out.stdout);
            let combined = format!("{}\n{}", stderr, stdout);

            let error_count = combined.lines()
                .filter(|l| l.starts_with("error") || l.starts_with("error["))
                .count();
            let warning_count = combined.lines()
                .filter(|l| l.starts_with("warning") || l.starts_with("warning["))
                .count();

            health.errors = error_count;
            health.warnings = warning_count;
        } else {
            health.errors = 1;
        }

        health
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_model_new() {
        let model = SelfModel::new();
        assert_eq!(model.module_count(), 0);
        assert_eq!(model.file_count(), 0);
        assert_eq!(model.total_lines(), 0);
    }

    #[test]
    fn test_dep_graph_orphans() {
        let graph = DepGraph {
            edges: vec![
                DepEdge { from: "core".into(), to: "agent".into(), kind: DepKind::ModuleUse },
                DepEdge { from: "core".into(), to: "neotrix".into(), kind: DepKind::ModuleUse },
            ],
        };
        let orphans = graph.orphans();
        assert!(orphans.contains(&"core".to_string()));
        assert!(!orphans.contains(&"agent".to_string()));
    }

    #[test]
    fn test_debt_severity_ordering() {
        assert!(DebtSeverity::Critical > DebtSeverity::Major);
        assert!(DebtSeverity::Major > DebtSeverity::Minor);
        assert!(DebtSeverity::Minor > DebtSeverity::Cosmetic);
    }

    #[test]
    fn test_evolution_events() {
        let mut model = SelfModel::new();
        let evt = EvolutionEvent {
            timestamp: chrono::Utc::now(),
            kind: EventKind::ModuleAdded,
            description: "Added metacognition module".into(),
            affected_modules: vec!["core/metacognition".into()],
        };
        model.register_evolution(evt);
        assert_eq!(model.evolution_history.len(), 1);
        assert_eq!(model.latest_events(1)[0].kind, EventKind::ModuleAdded);
    }

    #[test]
    fn test_component_map_hubs() {
        let map = ComponentMap {
            nodes: vec![
                ComponentNode { name: "core".into(), path: "core/".into(), layer: 1, file_count: 10, lines: 1000, description: "".into() },
                ComponentNode { name: "agent".into(), path: "agent/".into(), layer: 3, file_count: 5, lines: 500, description: "".into() },
                ComponentNode { name: "server".into(), path: "server/".into(), layer: 3, file_count: 3, lines: 300, description: "".into() },
            ],
            edges: vec![
                ("core".into(), "agent".into(), "depends".into()),
                ("core".into(), "server".into(), "depends".into()),
            ],
        };
        let hubs = map.find_hubs(1);
        assert!(hubs.contains(&"core"));
    }

    #[test]
    fn test_modules_without_tests() {
        let mut model = SelfModel::new();
        model.modules.push(ModuleInfo {
            name: "tested_mod".into(), path: "".into(), file_count: 1, total_lines: 100,
            test_count: 5, has_tests: true, unsafe_count: 0, unwrap_count: 0, todo_count: 0,
            public_api_count: 3, description: "".into(),
        });
        model.modules.push(ModuleInfo {
            name: "untested_mod".into(), path: "".into(), file_count: 1, total_lines: 100,
            test_count: 0, has_tests: false, unsafe_count: 0, unwrap_count: 0, todo_count: 0,
            public_api_count: 3, description: "".into(),
        });
        let untested = model.modules_without_tests();
        assert_eq!(untested.len(), 1);
        assert_eq!(untested[0].name, "untested_mod");
    }
}
