use std::path::Path;
use super::self_model::{ModuleInfo, FileInfo, SelfModel, DepGraph, ComponentMap, ComponentNode, CompilationHealth};

static SKIP_DIRS: &[&str] = &["target", ".git", "node_modules", ".fingerprint", "build"];

#[derive(Debug, Clone)]
pub struct CodeScanner {
    pub project_root: String,
}

impl CodeScanner {
    pub fn new(project_root: &str) -> Self {
        Self { project_root: project_root.to_string() }
    }

    pub fn scan(&self) -> SelfModel {
        let mut model = SelfModel::new();
        model.modules = self.scan_modules();
        model.files = self.scan_files_recursive(Path::new(&self.project_root));
        model.dep_graph = self.build_dep_graph(&model.modules);
        model.component_map = self.build_component_map(&model.modules);
        model.compilation = CompilationHealth::check(&self.project_root);
        model
    }

    /// Discover modules recursively by finding directories with `mod.rs`.
    /// Searches both `src/` and `neotrix-core/src/`.
    pub fn scan_modules(&self) -> Vec<ModuleInfo> {
        let mut modules = Vec::new();
        for root in self.source_roots() {
            if root.exists() {
                self.discover_in_dir(&root, &root, &mut modules);
            }
        }
        modules.sort_by(|a, b| a.name.cmp(&b.name));
        modules
    }

    fn source_roots(&self) -> Vec<std::path::PathBuf> {
        let mut roots = Vec::new();
        let direct = Path::new(&self.project_root).join("src");
        if direct.exists() { roots.push(direct); }
        let alt = Path::new(&self.project_root).join("neotrix-core").join("src");
        if alt.exists() { roots.push(alt); }
        roots
    }

    fn discover_in_dir(&self, root: &Path, dir: &Path, modules: &mut Vec<ModuleInfo>) {
        if dir == root {
            // root might not have a mod.rs — check subdirectories
        } else {
            let mod_file = dir.join("mod.rs");
            if mod_file.exists() {
                if let Some(info) = self.build_module_info_from_dir(dir, root) {
                    modules.push(info);
                }
            }
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let fpath = entry.path();
                if fpath.is_dir() {
                    let name = fpath.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if SKIP_DIRS.contains(&name.as_str()) || name.starts_with('.') {
                        continue;
                    }
                    self.discover_in_dir(root, &fpath, modules);
                }
            }
        }
    }

    fn module_name(rel_path: &Path) -> String {
        let components: Vec<String> = rel_path.components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();
        components.join("::")
    }

    fn build_module_info_from_dir(&self, dir: &Path, root: &Path) -> Option<ModuleInfo> {
        let rel = dir.strip_prefix(root).ok()?;
        let name = Self::module_name(rel);

        let mut file_count = 0;
        let mut total_lines = 0;
        let mut test_count = 0;
        let mut has_tests = false;
        let mut unsafe_count = 0;
        let mut unwrap_count = 0;
        let mut todo_count = 0;
        let mut public_api_count = 0;

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let fpath = entry.path();
                if fpath.extension().map(|e| e == "rs").unwrap_or(false) {
                    file_count += 1;
                    if let Ok(content) = std::fs::read_to_string(&fpath) {
                        let line_count = content.lines().count();
                        total_lines += line_count;
                        unsafe_count += content.matches("unsafe").count();
                        unwrap_count += content.matches(".unwrap()").count();
                        todo_count += content.matches("TODO").count() + content.matches("todo!").count();
                        public_api_count += content.matches("pub fn").count();
                        if content.contains("#[cfg(test)]") || content.contains("#[test]") {
                            has_tests = true;
                            test_count += content.matches("#[test]").count();
                        }
                    }
                }
            }
        }

        Some(ModuleInfo {
            name,
            path: dir.to_string_lossy().to_string(),
            file_count,
            total_lines,
            test_count,
            has_tests,
            unsafe_count,
            unwrap_count,
            todo_count,
            public_api_count,
            description: String::new(),
        })
    }

    pub fn scan_single_module(&self, name: &str) -> Option<ModuleInfo> {
        let modules = self.scan_modules();
        modules.into_iter().find(|m| m.name == name)
            .or_else(|| {
                let path = self.module_path(name);
                if path.exists() {
                    let root = if Path::new(&self.project_root).join("src").join(name).exists() {
                        Path::new(&self.project_root).join("src")
                    } else {
                        Path::new(&self.project_root).join("neotrix-core").join("src")
                    };
                    self.build_module_info_from_dir(&path, &root)
                } else {
                    None
                }
            })
    }

    fn module_path(&self, name: &str) -> std::path::PathBuf {
        let direct = Path::new(&self.project_root).join("src").join(name);
        if direct.exists() {
            return direct;
        }
        let alt = Path::new(&self.project_root)
            .join("neotrix-core").join("src").join(name);
        if alt.exists() {
            return alt;
        }
        direct
    }

    fn scan_files_recursive(&self, dir: &Path) -> Vec<FileInfo> {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let fpath = entry.path();
                if fpath.is_dir() {
                    let name = fpath.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if SKIP_DIRS.contains(&name.as_str()) || name.starts_with('.') {
                        continue;
                    }
                    files.extend(self.scan_files_recursive(&fpath));
                } else if fpath.extension().map(|e| e == "rs").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&fpath) {
                        let path_str = fpath.to_string_lossy().to_string();
                        files.push(FileInfo {
                            path: path_str.clone(),
                            module: self.guess_module(&path_str),
                            lines: content.lines().count(),
                            is_test_file: path_str.ends_with("_test.rs") || path_str.ends_with("tests.rs"),
                            has_unsafe: content.contains("unsafe"),
                            has_todos: content.contains("TODO") || content.contains("todo!"),
                            pub_fns: content.matches("pub fn").count(),
                            last_modified: chrono::Utc::now(),
                        });
                    }
                }
            }
        }
        files.sort_by_key(|b| std::cmp::Reverse(b.lines));
        files
    }

    fn guess_module(&self, path: &str) -> String {
        let normalized = path.replace('\\', "/");
        if let Some(pos) = normalized.find("/src/") {
            let remainder = &normalized[pos + 5..];
            if let Some(slash) = remainder.find('/') {
                return remainder[..slash].to_string();
            }
        }
        "unknown".to_string()
    }

    fn build_dep_graph(&self, modules: &[ModuleInfo]) -> DepGraph {
        use super::self_model::DepEdge;
        let mut edges = Vec::new();
        for module in modules {
            let path = Path::new(&module.path);
            if !path.exists() { continue; }
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let fpath = entry.path();
                    if fpath.extension().map(|e| e == "rs").unwrap_or(false) {
                        if let Ok(content) = std::fs::read_to_string(&fpath) {
                            for target in modules {
                                if target.name == module.name { continue; }
                                let crate_pattern = format!("use crate::{}", target.name);
                                if content.contains(&crate_pattern) {
                                    edges.push(DepEdge {
                                        from: module.name.clone(),
                                        to: target.name.clone(),
                                        kind: super::self_model::DepKind::ModuleUse,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        DepGraph { edges }
    }

    fn build_component_map(&self, modules: &[ModuleInfo]) -> ComponentMap {
        let nodes: Vec<ComponentNode> = modules.iter().map(|module| {
            let top = module.name.split("::").next().unwrap_or("");
            let layer = match top {
                "core" => 1,
                "nt_mind" | "neotrix" => 2,
                "agent" => 3,
                "security" | "sandbox" | "orchestrator" | "background_loop" => 4,
                _ => 5,
            };
            ComponentNode {
                name: module.name.clone(),
                path: module.path.clone(),
                layer,
                file_count: module.file_count,
                lines: module.total_lines,
                description: module.description.clone(),
            }
        }).collect();
        ComponentMap { nodes, edges: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_guess_module() {
        let scanner = CodeScanner::new("/test");
        assert_eq!(scanner.guess_module("/project/src/core/capability.rs"), "core");
        assert_eq!(scanner.guess_module("/project/src/agent/sub_agent.rs"), "agent");
    }

    #[test]
    fn test_build_component_map() {
        let scanner = CodeScanner::new("/test");
        let modules = vec![
            ModuleInfo {
                name: "core".into(), path: "".into(), file_count: 10, total_lines: 1000,
                test_count: 20, has_tests: true, unsafe_count: 0, unwrap_count: 0,
                todo_count: 0, public_api_count: 30, description: "".into(),
            },
            ModuleInfo {
                name: "agent".into(), path: "".into(), file_count: 5, total_lines: 500,
                test_count: 10, has_tests: true, unsafe_count: 0, unwrap_count: 0,
                todo_count: 0, public_api_count: 15, description: "".into(),
            },
        ];
        let map = scanner.build_component_map(&modules);
        assert_eq!(map.nodes.len(), 2);
        assert_eq!(map.nodes[0].layer, 1);
    }

    #[test]
    fn test_module_path_not_exists() {
        let scanner = CodeScanner::new("/nonexistent");
        assert!(scanner.scan_single_module("core").is_none());
    }

    #[test]
    fn test_module_name_format() {
        let name = CodeScanner::module_name(Path::new("core/metacognition"));
        assert_eq!(name, "core::metacognition");
    }

    #[test]
    fn test_module_name_single() {
        let name = CodeScanner::module_name(Path::new("core"));
        assert_eq!(name, "core");
    }

    #[test]
    fn test_dynamic_discovery_no_crash() {
        let scanner = CodeScanner::new(&std::env::current_dir().expect("value should be ok in test")
            .to_string_lossy());
        let modules = scanner.scan_modules();
        assert!(!modules.is_empty());
    }
}
