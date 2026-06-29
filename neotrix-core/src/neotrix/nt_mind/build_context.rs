use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum DependencySource {
    CratesIo,
    Git(String),
    Path(PathBuf),
    Workspace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DependencyInfo {
    pub name: String,
    pub version_req: String,
    pub is_optional: bool,
    pub source: DependencySource,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub dependencies: Vec<DependencyInfo>,
    pub features: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct BuildContextStats {
    pub total_packages: usize,
    pub total_dependencies: usize,
    pub external_count: usize,
    pub workspace_count: usize,
    pub cycle_count: usize,
}

pub struct BuildContext {
    pub packages: Vec<PackageInfo>,
    pub workspace_members: Vec<String>,
    pub package_index: HashMap<String, usize>,
}

impl BuildContext {
    pub fn analyze(root: &Path) -> Result<Self, String> {
        let mut packages: Vec<PackageInfo> = Vec::new();
        let mut workspace_members: Vec<String> = Vec::new();
        let mut seen_paths: HashSet<PathBuf> = HashSet::new();
        let canonical_root = root
            .canonicalize()
            .map_err(|e| format!("cannot canonicalize root path: {}", e))?;

        let mut queue: Vec<PathBuf> = vec![canonical_root.clone()];

        while let Some(dir) = queue.pop() {
            let tom = dir.join("Cargo.toml");
            if !tom.exists()
                || !seen_paths.insert(tom.canonicalize().unwrap_or_else(|_| tom.clone()))
            {
                continue;
            }
            let content = fs::read_to_string(&tom)
                .map_err(|e| format!("cannot read {}: {}", tom.display(), e))?;
            let value: toml::Value = toml::from_str(&content)
                .map_err(|e| format!("cannot parse {}: {}", tom.display(), e))?;

            // parse [workspace] members
            if let Some(ws) = value.get("workspace").and_then(|v| v.as_table()) {
                if let Some(members) = ws.get("members").and_then(|v| v.as_array()) {
                    for m in members {
                        if let Some(member_str) = m.as_str() {
                            let member_path = dir.join(member_str);
                            workspace_members.push(member_str.to_string());
                            if member_path.exists() {
                                queue.push(member_path);
                            }
                        }
                    }
                }
            }

            // parse [package]
            let pkg = match value.get("package").and_then(|v| v.as_table()) {
                Some(p) => p,
                None => continue,
            };

            let name = match pkg.get("name").and_then(|v| v.as_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let version = pkg
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string();

            let mut dependencies: Vec<DependencyInfo> = Vec::new();
            let table = match value.as_table() {
                Some(t) => t,
                None => continue,
            };

            for dep_section in &["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(deps) = table.get(*dep_section).and_then(|v| v.as_table()) {
                    for (dep_name, dep_value) in deps {
                        let (version_req, is_optional, source) = parse_dep(dep_value, dep_name);
                        dependencies.push(DependencyInfo {
                            name: dep_name.clone(),
                            version_req,
                            is_optional,
                            source,
                        });
                    }
                }
            }

            let features = table
                .get("features")
                .and_then(|v| v.as_table())
                .map(|ft| {
                    ft.iter()
                        .map(|(k, v)| {
                            let list = v
                                .as_array()
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|item| item.as_str().map(String::from))
                                        .collect()
                                })
                                .unwrap_or_default();
                            (k.clone(), list)
                        })
                        .collect::<HashMap<String, Vec<String>>>()
                })
                .unwrap_or_default();

            packages.push(PackageInfo {
                name: name.clone(),
                version,
                path: dir,
                dependencies,
                features,
            });
        }

        let mut package_index: HashMap<String, usize> = HashMap::new();
        for (i, pkg) in packages.iter().enumerate() {
            package_index.entry(pkg.name.clone()).or_insert(i);
        }

        Ok(BuildContext {
            packages,
            workspace_members,
            package_index,
        })
    }

    pub fn find_package(&self, name: &str) -> Option<&PackageInfo> {
        self.package_index
            .get(name)
            .and_then(|&i| self.packages.get(i))
    }

    pub fn get_dependency_graph(&self, package: &str) -> Vec<Vec<String>> {
        let mut levels: Vec<Vec<String>> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();

        if self.package_index.contains_key(package) {
            queue.push_back((package.to_string(), 0));
            visited.insert(package.to_string());
        }

        while let Some((current, level)) = queue.pop_front() {
            if level >= levels.len() {
                levels.push(Vec::new());
            }
            levels[level].push(current.clone());

            if let Some(pkg) = self.find_package(&current) {
                for dep in &pkg.dependencies {
                    if !visited.contains(&dep.name) && self.package_index.contains_key(&dep.name) {
                        visited.insert(dep.name.clone());
                        queue.push_back((dep.name.clone(), level + 1));
                    }
                }
            }
        }

        levels
    }

    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let adj: HashMap<&str, Vec<&str>> = self
            .packages
            .iter()
            .map(|p| {
                let deps: Vec<&str> = p
                    .dependencies
                    .iter()
                    .filter(|d| self.package_index.contains_key(&d.name))
                    .map(|d| d.name.as_str())
                    .collect();
                (p.name.as_str(), deps)
            })
            .collect();

        let mut cycles: Vec<Vec<String>> = Vec::new();
        let mut visited: HashSet<&str> = HashSet::new();
        let mut rec_stack: Vec<&str> = Vec::new();
        let mut rec_set: HashSet<&str> = HashSet::new();

        fn dfs<'a>(
            node: &'a str,
            adj: &HashMap<&'a str, Vec<&'a str>>,
            visited: &mut HashSet<&'a str>,
            rec_stack: &mut Vec<&'a str>,
            rec_set: &mut HashSet<&'a str>,
            cycles: &mut Vec<Vec<String>>,
        ) {
            visited.insert(node);
            rec_stack.push(node);
            rec_set.insert(node);

            if let Some(neighbors) = adj.get(node) {
                for &next in neighbors {
                    if !visited.contains(next) {
                        dfs(next, adj, visited, rec_stack, rec_set, cycles);
                    } else if rec_set.contains(next) {
                        let pos = rec_stack
                            .iter()
                            .position(|&x| x == next)
                            .expect("node in rec_stack confirmed by rec_set");
                        let cycle: Vec<String> =
                            rec_stack[pos..].iter().map(|&s| s.to_string()).collect();
                        cycles.push(cycle);
                    }
                }
            }

            rec_stack.pop();
            rec_set.remove(node);
        }

        for node in adj.keys() {
            if !visited.contains(node) {
                dfs(
                    node,
                    &adj,
                    &mut visited,
                    &mut rec_stack,
                    &mut rec_set,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    pub fn stats(&self) -> BuildContextStats {
        let total_packages = self.packages.len();
        let mut total_dependencies = 0;
        let mut external_count = 0;
        let mut workspace_count = 0;

        for pkg in &self.packages {
            total_dependencies += pkg.dependencies.len();
            for dep in &pkg.dependencies {
                if self.package_index.contains_key(&dep.name) {
                    workspace_count += 1;
                } else {
                    external_count += 1;
                }
            }
        }

        let cycle_count = self.detect_cycles().len();

        BuildContextStats {
            total_packages,
            total_dependencies,
            external_count,
            workspace_count,
            cycle_count,
        }
    }
}

fn parse_dep(dep_value: &toml::Value, _dep_name: &str) -> (String, bool, DependencySource) {
    match dep_value {
        toml::Value::String(ver) => (ver.clone(), false, DependencySource::CratesIo),
        toml::Value::Table(t) => {
            let version_req = t
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("*")
                .to_string();
            let is_optional = t.get("optional").and_then(|v| v.as_bool()).unwrap_or(false);
            let source = if let Some(git_url) = t.get("git").and_then(|v| v.as_str()) {
                DependencySource::Git(git_url.to_string())
            } else if let Some(path_str) = t.get("path").and_then(|v| v.as_str()) {
                DependencySource::Path(PathBuf::from(path_str))
            } else if t
                .get("workspace")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                DependencySource::Workspace
            } else {
                DependencySource::CratesIo
            };
            (version_req, is_optional, source)
        }
        _ => ("*".to_string(), false, DependencySource::CratesIo),
    }
}

impl std::fmt::Display for BuildContextStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BuildContextStats {{ packages: {}, total_deps: {}, external: {}, workspace: {}, cycles: {} }}",
            self.total_packages,
            self.total_dependencies,
            self.external_count,
            self.workspace_count,
            self.cycle_count,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_temp_toml(dir: &Path, _name: &str, content: &str) -> PathBuf {
        fs::create_dir_all(dir).expect("create temp dir for test");
        let path = dir.join("Cargo.toml");
        let mut f = fs::File::create(&path).expect("create Cargo.toml for test");
        write!(f, "{}", content).expect("write Cargo.toml content");
        path
    }

    #[test]
    fn test_analyze_single_package() {
        let dir = std::env::temp_dir().join("neotrix_test_analyze_single");
        let _ = fs::remove_dir_all(&dir);
        create_temp_toml(
            &dir,
            "test_pkg",
            r#"
[package]
name = "test_pkg"
version = "1.0.0"

[dependencies]
serde = "1"
tokio = { version = "1", features = ["full"] }
"#,
        );
        let ctx = BuildContext::analyze(&dir).expect("analyze single-package temp project");
        assert_eq!(ctx.packages.len(), 1);
        assert_eq!(ctx.packages[0].name, "test_pkg");
        assert_eq!(ctx.packages[0].version, "1.0.0");
        assert_eq!(ctx.packages[0].dependencies.len(), 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_workspace_members() {
        let dir = std::env::temp_dir().join("neotrix_test_workspace");
        let _ = fs::remove_dir_all(&dir);
        let root_toml = r#"
[package]
name = "root"
version = "0.1.0"

[workspace]
members = ["crates/a"]

[dependencies]
a = { path = "crates/a" }
"#;
        create_temp_toml(&dir, "root", root_toml);
        let crates_a = dir.join("crates").join("a");
        let a_toml = r#"
[package]
name = "a"
version = "0.1.0"

[dependencies]
serde = "1"
"#;
        create_temp_toml(&crates_a, "a", a_toml);
        let ctx = BuildContext::analyze(&dir).expect("analyze workspace temp project");
        assert_eq!(ctx.packages.len(), 2);
        assert!(ctx.package_index.contains_key("a"));
        assert!(ctx.package_index.contains_key("root"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_package() {
        let dir = std::env::temp_dir().join("neotrix_test_find");
        let _ = fs::remove_dir_all(&dir);
        create_temp_toml(
            &dir,
            "mycrate",
            r#"
[package]
name = "mycrate"
version = "2.0.0"
"#,
        );
        let ctx = BuildContext::analyze(&dir).expect("analyze project for find_package test");
        let pkg = ctx
            .find_package("mycrate")
            .expect("mycrate must exist in analyzed context");
        assert_eq!(pkg.version, "2.0.0");
        assert_eq!(ctx.find_package("nonexistent"), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_dependency_graph() {
        let dir = std::env::temp_dir().join("neotrix_test_depgraph");
        let _ = fs::remove_dir_all(&dir);
        let root_toml = r#"
[package]
name = "app"
version = "0.1.0"

[workspace]
members = ["crates/lib_a", "crates/lib_b"]

[dependencies]
lib_a = { path = "crates/lib_a" }
lib_b = { path = "crates/lib_b" }
"#;
        create_temp_toml(&dir, "app", root_toml);
        create_temp_toml(
            &dir.join("crates").join("lib_a"),
            "lib_a",
            r#"
[package]
name = "lib_a"
version = "0.1.0"

[dependencies]
lib_b = { path = "../lib_b" }
"#,
        );
        create_temp_toml(
            &dir.join("crates").join("lib_b"),
            "lib_b",
            r#"
[package]
name = "lib_b"
version = "0.1.0"
"#,
        );
        let ctx = BuildContext::analyze(&dir).expect("analyze project for dependency graph test");
        let graph = ctx.get_dependency_graph("app");
        assert!(!graph.is_empty());
        assert_eq!(graph[0][0], "app");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_cycles() {
        let dir = std::env::temp_dir().join("neotrix_test_cycles");
        let _ = fs::remove_dir_all(&dir);
        let root_toml = r#"
[package]
name = "a"
version = "0.1.0"

[workspace]
members = ["b", "c"]

[dependencies]
b = { path = "b" }
"#;
        create_temp_toml(&dir, "a", root_toml);
        create_temp_toml(
            &dir.join("b"),
            "b",
            r#"
[package]
name = "b"
version = "0.1.0"

[dependencies]
c = { path = "../c" }
"#,
        );
        create_temp_toml(
            &dir.join("c"),
            "c",
            r#"
[package]
name = "c"
version = "0.1.0"

[dependencies]
a = { path = "../a" }
"#,
        );
        let ctx = BuildContext::analyze(&dir).expect("analyze project for cycle detection test");
        let cycles = ctx.detect_cycles();
        assert!(!cycles.is_empty(), "expected at least one cycle");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_stats() {
        let dir = std::env::temp_dir().join("neotrix_test_stats");
        let _ = fs::remove_dir_all(&dir);
        create_temp_toml(
            &dir,
            "pkg",
            r#"
[package]
name = "pkg"
version = "1.0.0"

[dependencies]
serde = "1"
tokio = { version = "1", optional = true }
"#,
        );
        let ctx = BuildContext::analyze(&dir).expect("analyze project for stats test");
        let s = ctx.stats();
        assert_eq!(s.total_packages, 1);
        assert_eq!(s.total_dependencies, 2);
        assert_eq!(s.external_count, 2);
        assert_eq!(s.workspace_count, 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_features_parsing() {
        let dir = std::env::temp_dir().join("neotrix_test_features");
        let _ = fs::remove_dir_all(&dir);
        create_temp_toml(
            &dir,
            "feat_pkg",
            r#"
[package]
name = "feat_pkg"
version = "0.5.0"

[features]
default = ["std"]
std = []
extra = ["dep::extra_dep"]
"#,
        );
        let ctx = BuildContext::analyze(&dir).expect("analyze project for features test");
        let pkg = ctx
            .find_package("feat_pkg")
            .expect("feat_pkg must exist in analyzed context");
        assert!(pkg.features.contains_key("default"));
        assert!(pkg.features.contains_key("extra"));
        assert_eq!(pkg.features["default"], vec!["std"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_optional_dependency() {
        let dir = std::env::temp_dir().join("neotrix_test_optional");
        let _ = fs::remove_dir_all(&dir);
        create_temp_toml(
            &dir,
            "opt_pkg",
            r#"
[package]
name = "opt_pkg"
version = "1.0.0"

[dependencies]
serde = { version = "1", optional = true }
"#,
        );
        let ctx = BuildContext::analyze(&dir).expect("analyze project for optional dep test");
        let pkg = ctx
            .find_package("opt_pkg")
            .expect("opt_pkg must exist in analyzed context");
        let dep = &pkg.dependencies[0];
        assert!(dep.is_optional);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_git_dependency() {
        let dir = std::env::temp_dir().join("neotrix_test_gitdep");
        let _ = fs::remove_dir_all(&dir);
        create_temp_toml(
            &dir,
            "git_pkg",
            r#"
[package]
name = "git_pkg"
version = "1.0.0"

[dependencies]
my_lib = { git = "https://github.com/user/my_lib.git", branch = "main" }
"#,
        );
        let ctx = BuildContext::analyze(&dir).expect("analyze project for git dep test");
        let pkg = ctx
            .find_package("git_pkg")
            .expect("git_pkg must exist in analyzed context");
        let dep = &pkg.dependencies[0];
        assert_eq!(dep.name, "my_lib");
        assert!(matches!(dep.source, DependencySource::Git(_)));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_dependency_graph_non_existent() {
        let dir = std::env::temp_dir().join("neotrix_test_dep_nonexist");
        let _ = fs::remove_dir_all(&dir);
        create_temp_toml(
            &dir,
            "only",
            r#"
[package]
name = "only"
version = "0.1.0"
"#,
        );
        let ctx =
            BuildContext::analyze(&dir).expect("analyze project for non-existent dep graph test");
        let graph = ctx.get_dependency_graph("nope");
        assert!(graph.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}
