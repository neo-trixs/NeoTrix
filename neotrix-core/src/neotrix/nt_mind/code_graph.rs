use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::neotrix::nt_mind::graph_build::{
    parse_rust_file, register_fn_defs, resolve_import, ParsedItem,
};
use crate::neotrix::nt_mind::graph_types::{EdgeKind, GraphEdge, GraphNode, NodeKind};

fn generate_node_id(
    project_id: &str,
    file_path: &str,
    name: &str,
    kind: &NodeKind,
    line: usize,
) -> String {
    let input = format!("{}:{}:{}:{:?}:{}", project_id, file_path, name, kind, line);
    format!("{:x}", Sha256::digest(input.as_bytes()))[..32].to_string()
}

/// 代码图引擎
pub struct CodeGraph {
    pub(crate) nodes: HashMap<String, GraphNode>,
    pub(crate) edges: Vec<GraphEdge>,
    pub(crate) file_nodes: HashMap<PathBuf, String>,
    pub(crate) communities: HashMap<String, usize>,
    pub(crate) project_id: Option<String>,
}

impl Default for CodeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            file_nodes: HashMap::new(),
            communities: HashMap::new(),
            project_id: None,
        }
    }

    pub fn with_project_id(mut self, pid: &str) -> Self {
        self.project_id = Some(pid.to_string());
        self
    }

    /// 从目录构建代码图
    pub fn build<P: AsRef<Path>>(&mut self, root: P) -> Result<usize, String> {
        let root = root.as_ref();
        if !root.is_dir() {
            return Err(format!("not a directory: {}", root.display()));
        }

        // Phase 1: walk files, parse items, create nodes
        let mut file_items: Vec<(PathBuf, Vec<ParsedItem>)> = Vec::new();
        for entry in walkdir::WalkDir::new(root).into_iter().filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && name != "target" && name != "node_modules"
        }) {
            let entry = entry.map_err(|e| e.to_string())?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path().to_path_buf();
            if path.extension().map(|e| e != "rs").unwrap_or(true) {
                continue;
            }

            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let items = parse_rust_file(&content);
            let _node_id = self.add_file_node(&path, &items);
            file_items.push((path.clone(), items));
        }

        // Phase 2: build edges from use statements + call analysis
        let mut fn_defs: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();
        for (path, items) in &file_items {
            register_fn_defs(path, items, &mut fn_defs);
        }

        for (p, items) in &file_items {
            let file_id = self.file_nodes.get(p).cloned().unwrap_or_default();
            for item in items.iter() {
                match item {
                    ParsedItem::UseStatement { target, .. } => {
                        let target_path = resolve_import(p, target);
                        let target_id =
                            target_path.and_then(|tp| self.file_nodes.get(&tp).cloned());
                        if let Some(target_id) = target_id {
                            self.add_edge(&file_id, &target_id, EdgeKind::Imports, 0.9);
                        }
                    }
                    ParsedItem::Function {
                        name, line, calls, ..
                    } => {
                        let fn_det_id = self.add_node(
                            name.as_str(),
                            NodeKind::Function,
                            Some(p.clone()),
                            *line,
                            *line,
                        );
                        self.add_edge(&file_id, &fn_det_id, EdgeKind::Contains, 1.0);
                        for c in calls {
                            if let Some(defs) = fn_defs.get(c.as_str()) {
                                for (def_path, def_line) in defs {
                                    let callee_det_id = self.add_node(
                                        c.as_str(),
                                        NodeKind::Function,
                                        Some(def_path.clone()),
                                        *def_line,
                                        *def_line,
                                    );
                                    self.add_edge(&fn_det_id, &callee_det_id, EdgeKind::Calls, 0.7);
                                }
                            }
                        }
                    }
                    ParsedItem::StructDef { name, line, .. } => {
                        let struct_det_id = self.add_node(
                            name.as_str(),
                            NodeKind::Struct,
                            Some(p.clone()),
                            *line,
                            *line,
                        );
                        self.add_edge(&file_id, &struct_det_id, EdgeKind::Contains, 1.0);
                    }
                    ParsedItem::TraitDef { name, line, .. } => {
                        let trait_det_id = self.add_node(
                            name.as_str(),
                            NodeKind::Trait,
                            Some(p.clone()),
                            *line,
                            *line,
                        );
                        self.add_edge(&file_id, &trait_det_id, EdgeKind::Contains, 1.0);
                    }
                    _ => {}
                }
            }
        }

        // Phase 3: simple community detection
        self.detect_communities();

        Ok(self.nodes.len())
    }

    fn add_file_node(&mut self, path: &Path, _items: &[ParsedItem]) -> String {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let id = generate_node_id(
            self.project_id.as_deref().unwrap_or("default"),
            &path.to_string_lossy(),
            &name,
            &NodeKind::File,
            0,
        );
        if !self.nodes.contains_key(&id) {
            self.nodes.insert(
                id.clone(),
                GraphNode {
                    id: id.clone(),
                    name,
                    kind: NodeKind::File,
                    file_path: Some(path.to_path_buf()),
                    start_line: 0,
                    end_line: 0,
                },
            );
            self.file_nodes.insert(path.to_path_buf(), id.clone());
        }
        id
    }

    fn add_node(
        &mut self,
        name: &str,
        kind: NodeKind,
        path: Option<PathBuf>,
        start: usize,
        end: usize,
    ) -> String {
        let path_str = path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let det_id = generate_node_id(
            self.project_id.as_deref().unwrap_or("default"),
            &path_str,
            name,
            &kind,
            start,
        );
        if !self.nodes.contains_key(&det_id) {
            self.nodes.insert(
                det_id.clone(),
                GraphNode {
                    id: det_id.clone(),
                    name: name.to_string(),
                    kind,
                    file_path: path,
                    start_line: start,
                    end_line: end,
                },
            );
        }
        det_id
    }

    fn add_edge(&mut self, from: &str, to: &str, kind: EdgeKind, confidence: f64) {
        if from == to {
            return;
        }
        self.edges.push(GraphEdge {
            from: from.to_string(),
            to: to.to_string(),
            kind,
            confidence,
        });
    }

    pub fn get_node(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.get(id)
    }
    pub fn nodes(&self) -> &HashMap<String, GraphNode> {
        &self.nodes
    }
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }
    pub fn communities(&self) -> &HashMap<String, usize> {
        &self.communities
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_use_statement() {
        let content = "use std::collections::HashMap;\nuse crate::neotrix::my_mod;\n";
        let items = parse_rust_file(content);
        let use_items: Vec<&ParsedItem> = items
            .iter()
            .filter(|i| matches!(i, ParsedItem::UseStatement { .. }))
            .collect();
        assert_eq!(use_items.len(), 2);
    }

    #[test]
    fn test_parse_function() {
        let content = "pub fn test_function() {\n    helper_call();\n}\nfn helper_call() {}\n";
        let items = parse_rust_file(content);
        let fns: Vec<&ParsedItem> = items
            .iter()
            .filter(|i| matches!(i, ParsedItem::Function { .. }))
            .collect();
        assert_eq!(fns.len(), 2);
    }

    #[test]
    fn test_parse_struct() {
        let content = "pub struct TestStruct {\n    field: i32,\n}\n";
        let items = parse_rust_file(content);
        assert!(items
            .iter()
            .any(|i| matches!(i, ParsedItem::StructDef { name, .. } if name == "TestStruct")));
    }

    #[test]
    fn test_parse_trait() {
        let content = "pub trait TestTrait {\n    fn method();\n}\n";
        let items = parse_rust_file(content);
        assert!(items
            .iter()
            .any(|i| matches!(i, ParsedItem::TraitDef { name, .. } if name == "TestTrait")));
    }

    fn fixture_project() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("create temp dir");
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).expect("create src dir");
        let files = [
            ("main.rs", "mod math;\nmod utils;\nfn main() { math::add(1, 2); utils::greet(); }\n"),
            ("math.rs", "pub fn add(a: i32, b: i32) -> i32 { a + b }\npub fn sub(a: i32, b: i32) -> i32 { a - b }\n"),
            ("utils.rs", "pub struct Config { pub name: String }\npub fn greet() -> String { \"Hello\".into() }\npub struct Helper;\nimpl Helper { pub fn run(&self) {} }\n"),
            ("lib.rs", "pub mod math;\npub mod utils;\npub trait TestTrait { fn method(&self); }\n"),
            ("extra.rs", "use crate::math;\npub fn compute() -> i32 { math::add(10, 20) }\n"),
        ];
        for (name, content) in &files {
            let mut f = std::fs::File::create(src.join(name)).expect("create file");
            f.write_all(content.as_bytes()).expect("write file");
        }
        (dir, src)
    }

    #[test]
    fn test_code_graph_build_from_fixture() {
        let (_dir, src) = fixture_project();
        let mut graph = CodeGraph::new();
        let count = graph.build(&src).expect("build should succeed");
        assert!(count > 3, "should find at least 3 nodes, got {}", count);
        assert!(graph.stats().total_edges > 0);
        assert!(graph.stats().community_count > 0);
    }

    #[test]
    fn test_impact_analysis_on_fixture() {
        let (_dir, src) = fixture_project();
        let mut graph = CodeGraph::new();
        graph.build(&src).expect("build should succeed");
        if let Some(id) = graph.nodes().keys().next() {
            let impact = graph.impact_analysis(id, 3);
            assert!(!impact.downstream.is_empty() || !impact.upstream.is_empty());
        }
    }

    #[test]
    fn test_resolve_import() {
        let test_path = PathBuf::from("src/neotrix/mod.rs");
        let _ = resolve_import(&test_path, "std::collections::HashMap");
        let _ = resolve_import(&test_path, "crate::neotrix::mod");
    }

    #[test]
    fn test_function_call_detection() {
        let content = "fn caller() {\n    callee_one();\n    let x = callee_two(42);\n    inner::nested_call();\n}\n";
        let items = parse_rust_file(content);
        let fns: Vec<&ParsedItem> = items
            .iter()
            .filter(|i| matches!(i, ParsedItem::Function { .. }))
            .collect();
        assert_eq!(fns.len(), 1);
        if let ParsedItem::Function { calls, .. } = &fns[0] {
            assert!(calls.contains(&"callee_one".to_string()));
            assert!(calls.contains(&"callee_two".to_string()));
        }
    }

    #[test]
    fn test_empty_graph_stats() {
        let graph = CodeGraph::new();
        let stats = graph.stats();
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.total_edges, 0);
        assert_eq!(stats.community_count, 0);
    }

    #[test]
    fn test_code_graph_stats_format() {
        let root =
            PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()));
        let src = root.join("src/neotrix/nt_mind/element");
        if !src.exists() {
            return;
        }
        let mut graph = CodeGraph::new();
        graph.build(&src).expect("value should be ok in test");
        let stats = graph.stats();
        assert!(stats.total_nodes > 0);
        assert!(stats.community_count >= 1);
        assert!(stats.type_counts.contains_key("file"));
        assert!(
            stats.type_counts.contains_key("function") || stats.type_counts.contains_key("struct")
        );
    }
}
