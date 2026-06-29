// Module registry: dependency graph, cycle detection, topological sort
// Phase 2d-P3/4: Orphan detection + safe codegen ordering

use std::collections::{HashMap, HashSet, VecDeque};

use crate::ir::Module;

/// Registry of all known .nt modules with their import dependencies.
#[derive(Debug, Default)]
pub struct ModuleRegistry {
    modules: HashMap<String, Module>,
    deps: HashMap<String, Vec<String>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a parsed module. Returns Err if name conflicts with existing module.
    pub fn register(&mut self, module: Module) -> Result<(), String> {
        let name = module.name.clone();
        if self.modules.contains_key(&name) {
            return Err(format!("Duplicate module '{}'", name));
        }
        let deps: Vec<String> = module.imports.iter().map(|i| i.path.clone()).collect();
        self.modules.insert(name.clone(), module);
        self.deps.insert(name, deps);
        Ok(())
    }

    /// Get a module by name.
    pub fn get(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    /// Check if a module is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    /// Number of registered modules.
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// List all registered module names.
    pub fn module_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.modules.keys().cloned().collect();
        names.sort();
        names
    }

    // ---- Orphan detection (P3) ----

    /// Find imports that reference non-existent modules.
    pub fn orphans(&self) -> Vec<OrphanImport> {
        let mut result = Vec::new();
        for (module_name, deps) in &self.deps {
            for dep in deps {
                if !self.modules.contains_key(dep.as_str()) {
                    result.push(OrphanImport {
                        source: module_name.clone(),
                        missing: dep.clone(),
                    });
                }
            }
        }
        result.sort_by(|a, b| a.source.cmp(&b.source));
        result
    }

    /// Returns true if there are no orphan imports.
    pub fn is_consistent(&self) -> bool {
        self.orphans().is_empty()
    }

    // ---- DFS cycle detection (P3) ----

    /// Detect cycles in the import dependency graph.
    /// Returns the first cycle found (as a list of module names forming the cycle).
    pub fn detect_cycle(&self) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();
        let mut path = Vec::new();

        let names: Vec<String> = self.deps.keys().cloned().collect();
        for name in &names {
            if !visited.contains(name) {
                if let Some(cycle) = self.dfs_cycle(name, &mut visited, &mut in_stack, &mut path) {
                    return Some(cycle);
                }
            }
        }
        None
    }

    fn dfs_cycle(
        &self,
        current: &str,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(current.to_string());
        in_stack.insert(current.to_string());
        path.push(current.to_string());

        if let Some(deps) = self.deps.get(current) {
            for dep in deps {
                if !in_stack.contains(dep.as_str()) {
                    if !visited.contains(dep.as_str()) {
                        if let Some(cycle) = self.dfs_cycle(dep, visited, in_stack, path) {
                            return Some(cycle);
                        }
                    }
                } else {
                    // Found a cycle — extract it from the path
                    let pos = path.iter().position(|n| n == dep).unwrap();
                    let cycle: Vec<String> = path[pos..].to_vec();
                    path.pop();
                    in_stack.remove(current);
                    return Some(cycle);
                }
            }
        }

        path.pop();
        in_stack.remove(current);
        None
    }

    // ---- Topological sort (P4) ----

    /// Returns modules in dependency order (dependencies before dependents).
    /// Uses Kahn's algorithm. Returns Err if a cycle is detected.
    pub fn topological_sort(&self) -> Result<Vec<String>, Vec<String>> {
        // Compute in-degree
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        for name in self.deps.keys() {
            in_degree.entry(name.as_str()).or_insert(0);
        }
        for deps in self.deps.values() {
            for dep in deps {
                if self.modules.contains_key(dep.as_str()) {
                    *in_degree.entry(dep.as_str()).or_insert(0) += 0; // ensure exists
                    for target in self.deps.keys() {
                        if target.as_str() == dep.as_str() {
                            // target depends on... wait this is wrong logic
                        }
                    }
                }
            }
        }

        // Correct approach: in_degree[name] = number of deps that are in the registry
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();

        for name in self.deps.keys() {
            in_degree.entry(name.as_str()).or_insert(0);
            graph.entry(name.as_str()).or_default();
        }

        for (from, deps) in &self.deps {
            for dep in deps {
                if self.modules.contains_key(dep.as_str()) {
                    // from depends on dep: edge dep → from
                    graph.entry(dep.as_str()).or_default().push(from.as_str());
                    *in_degree.entry(from.as_str()).or_insert(0) += 1;
                }
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<&str> = VecDeque::new();
        for (name, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(name);
            }
        }

        let mut sorted = Vec::new();
        while let Some(name) = queue.pop_front() {
            sorted.push(name.to_string());
            if let Some(neighbors) = graph.get(name) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        if sorted.len() != self.deps.len() {
            // There's a cycle — find it
            let cycle = self.detect_cycle().unwrap_or_default();
            return Err(cycle);
        }

        Ok(sorted)
    }

    /// Clear all registered modules.
    pub fn clear(&mut self) {
        self.modules.clear();
        self.deps.clear();
    }
}

/// An import that references a module not in the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrphanImport {
    pub source: String,
    pub missing: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;
    use std::path::PathBuf;

    fn make_module(name: &str, imports: Vec<&str>) -> Module {
        Module {
            name: name.to_string(),
            description: String::new(),
            source_file: PathBuf::from(format!("{}.nt", name)),
            vsa_dim: None,
            imports: imports
                .into_iter()
                .map(|p| Import {
                    path: p.to_string(),
                    alias: None,
                })
                .collect(),
            functions: vec![],
            pipeline: None,
            tests: vec![],
        }
    }

    #[test]
    fn test_register_and_get() {
        let mut reg = ModuleRegistry::new();
        let m = make_module("math", vec![]);
        reg.register(m).unwrap();
        assert!(reg.contains("math"));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_duplicate_rejected() {
        let mut reg = ModuleRegistry::new();
        reg.register(make_module("dup", vec![])).unwrap();
        assert!(reg.register(make_module("dup", vec![])).is_err());
    }

    #[test]
    fn test_orphan_detection() {
        let mut reg = ModuleRegistry::new();
        reg.register(make_module("main", vec!["utils", "nonexistent"]))
            .unwrap();
        reg.register(make_module("utils", vec![])).unwrap();
        let orphans = reg.orphans();
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].source, "main");
        assert_eq!(orphans[0].missing, "nonexistent");
    }

    #[test]
    fn test_no_orphans() {
        let mut reg = ModuleRegistry::new();
        reg.register(make_module("a", vec!["b"])).unwrap();
        reg.register(make_module("b", vec![])).unwrap();
        assert!(reg.is_consistent());
    }

    #[test]
    fn test_cycle_detection_simple() {
        let mut reg = ModuleRegistry::new();
        reg.register(make_module("a", vec!["b"])).unwrap();
        reg.register(make_module("b", vec!["a"])).unwrap();
        let cycle = reg.detect_cycle();
        assert!(cycle.is_some());
        let c = cycle.unwrap();
        assert!(c.len() >= 2);
    }

    #[test]
    fn test_no_cycle() {
        let mut reg = ModuleRegistry::new();
        reg.register(make_module("a", vec!["b"])).unwrap();
        reg.register(make_module("b", vec!["c"])).unwrap();
        reg.register(make_module("c", vec![])).unwrap();
        assert!(reg.detect_cycle().is_none());
    }

    #[test]
    fn test_cycle_detection_self_loop() {
        let mut reg = ModuleRegistry::new();
        reg.register(make_module("selfie", vec!["selfie"])).unwrap();
        let cycle = reg.detect_cycle();
        assert!(cycle.is_some());
        assert_eq!(cycle.unwrap(), vec!["selfie"]);
    }

    #[test]
    fn test_topological_sort_linear() {
        let mut reg = ModuleRegistry::new();
        reg.register(make_module("c", vec![])).unwrap();
        reg.register(make_module("b", vec!["c"])).unwrap();
        reg.register(make_module("a", vec!["b"])).unwrap();
        let sorted = reg.topological_sort().unwrap();
        // c must come before b, b before a
        let pos_c = sorted.iter().position(|n| n == "c").unwrap();
        let pos_b = sorted.iter().position(|n| n == "b").unwrap();
        let pos_a = sorted.iter().position(|n| n == "a").unwrap();
        assert!(pos_c < pos_b);
        assert!(pos_b < pos_a);
    }

    #[test]
    fn test_topological_sort_cycle_returns_err() {
        let mut reg = ModuleRegistry::new();
        reg.register(make_module("a", vec!["b"])).unwrap();
        reg.register(make_module("b", vec!["a"])).unwrap();
        assert!(reg.topological_sort().is_err());
    }

    #[test]
    fn test_clear() {
        let mut reg = ModuleRegistry::new();
        reg.register(make_module("x", vec![])).unwrap();
        reg.clear();
        assert!(reg.is_empty());
    }
}
