use std::collections::HashMap;
use std::path::Path;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::Instant;

use super::code_graph::CodeGraph;
use super::graph_types::{CodeGraphStats, ImpactResult};

/// A single recorded tool invocation trace.
pub struct ToolTrace {
    pub tool_name: String,
    pub params: HashMap<String, String>,
    pub result_size: usize,
    pub duration_ms: u64,
    pub success: bool,
}

/// Search result enriched with node metadata.
pub struct EnrichedSearchResult {
    pub node_id: String,
    pub node_name: String,
    pub node_kind: String,
    pub relevance: f64,
    pub file_path: Option<String>,
}

/// CountingExecutor wraps a CodeGraph with call-counting and tracing,
/// inspired by codegraph-rust's CountingExecutor pattern.
///
/// Traces can be drained for SEAL reward signals.
pub struct CountingExecutor {
    graph: Arc<Mutex<CodeGraph>>,
    call_count: Arc<AtomicUsize>,
    traces: Arc<Mutex<Vec<ToolTrace>>>,
}

impl CountingExecutor {
    pub fn new(graph: Arc<Mutex<CodeGraph>>) -> Self {
        Self {
            graph,
            call_count: Arc::new(AtomicUsize::new(0)),
            traces: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn record(
        &self,
        tool_name: &str,
        params: HashMap<String, String>,
        result_size: usize,
        duration_ms: u64,
        success: bool,
    ) {
        self.call_count.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut traces) = self.traces.lock() {
            traces.push(ToolTrace {
                tool_name: tool_name.to_string(),
                params,
                result_size,
                duration_ms,
                success,
            });
            if traces.len() > 10_000 {
                let excess = traces.len() - 10_000;
                traces.drain(0..excess);
            }
        }
    }

    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::Relaxed)
    }

    /// Drain all recorded traces (for SEAL reward signal).
    pub fn take_traces(&self) -> Vec<ToolTrace> {
        let mut traces = self.traces.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut traces)
    }

    pub fn reset_count(&self) {
        self.call_count.store(0, Ordering::Relaxed);
    }

    /// Impact analysis for a target node.
    pub fn execute_impact(&self, target: &str, max_depth: usize) -> Result<ImpactResult, String> {
        let start = Instant::now();
        let graph = self
            .graph
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let resolved = graph.resolve_id(target);
        if !graph.nodes().contains_key(&resolved) {
            let duration = start.elapsed().as_millis() as u64;
            let mut params = HashMap::new();
            params.insert("target".into(), target.into());
            params.insert("max_depth".into(), max_depth.to_string());
            self.record("execute_impact", params, 0, duration, false);
            return Err(format!(
                "Node not found: '{}' (resolved: '{}')",
                target, resolved
            ));
        }
        let impact = graph.impact_analysis(&resolved, max_depth);
        let result_size = impact.upstream.len() + impact.downstream.len();
        let duration = start.elapsed().as_millis() as u64;
        let mut params = HashMap::new();
        params.insert("target".into(), target.into());
        params.insert("max_depth".into(), max_depth.to_string());
        self.record("execute_impact", params, result_size, duration, true);
        Ok(impact)
    }

    /// Build graph from path and return stats.
    pub fn execute_stats(&self, path: &str) -> Result<CodeGraphStats, String> {
        let start = Instant::now();
        let mut graph = self
            .graph
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let node_count = graph.build(path)?;
        let stats = graph.stats();
        let duration = start.elapsed().as_millis() as u64;
        let mut params = HashMap::new();
        params.insert("path".into(), path.into());
        self.record("execute_stats", params, node_count, duration, true);
        Ok(stats)
    }

    /// Dependencies of a file: what it imports.
    pub fn execute_dependencies(&self, file_path: &str) -> Result<Vec<String>, String> {
        let start = Instant::now();
        let graph = self
            .graph
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let path = Path::new(file_path);
        let deps = graph.file_dependencies(path);
        let result_size = deps.len();
        let duration = start.elapsed().as_millis() as u64;
        let mut params = HashMap::new();
        params.insert("file_path".into(), file_path.into());
        self.record("execute_dependencies", params, result_size, duration, true);
        Ok(deps)
    }

    /// Dependents of a file: what imports it.
    pub fn execute_dependents(&self, file_path: &str) -> Result<Vec<String>, String> {
        let start = Instant::now();
        let graph = self
            .graph
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let path = Path::new(file_path);
        let deps = graph.file_dependents(path);
        let result_size = deps.len();
        let duration = start.elapsed().as_millis() as u64;
        let mut params = HashMap::new();
        params.insert("file_path".into(), file_path.into());
        self.record("execute_dependents", params, result_size, duration, true);
        Ok(deps)
    }

    /// Search graph nodes by name/ID matching the query string.
    /// Returns up to `max` results sorted by relevance.
    pub fn execute_search_enriched(&self, query: &str, max: usize) -> Vec<EnrichedSearchResult> {
        let start = Instant::now();
        let query_lower = query.to_lowercase();

        let results = match self.graph.lock() {
            Ok(graph) => {
                let mut matches: Vec<EnrichedSearchResult> = graph
                    .nodes()
                    .values()
                    .filter(|n| {
                        n.name.to_lowercase().contains(&query_lower)
                            || n.id.to_lowercase().contains(&query_lower)
                    })
                    .map(|n| {
                        let relevance = if n.name.to_lowercase() == query_lower {
                            1.0
                        } else if n.id.to_lowercase() == query_lower {
                            0.95
                        } else if n.name.to_lowercase().starts_with(&query_lower) {
                            0.8
                        } else {
                            0.5
                        };
                        EnrichedSearchResult {
                            node_id: n.id.clone(),
                            node_name: n.name.clone(),
                            node_kind: n.kind.as_str().to_string(),
                            relevance,
                            file_path: n
                                .file_path
                                .as_ref()
                                .map(|p| p.to_string_lossy().to_string()),
                        }
                    })
                    .collect();
                matches.sort_by(|a, b| {
                    b.relevance
                        .partial_cmp(&a.relevance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                matches.truncate(max);
                matches
            }
            Err(_e) => {
                let duration = start.elapsed().as_millis() as u64;
                let mut params = HashMap::new();
                params.insert("query".into(), query.into());
                params.insert("max".into(), max.to_string());
                self.record("execute_search_enriched", params, 0, duration, false);
                return Vec::new();
            }
        };

        let result_size = results.len();
        let duration = start.elapsed().as_millis() as u64;
        let mut params = HashMap::new();
        params.insert("query".into(), query.into());
        params.insert("max".into(), max.to_string());
        self.record(
            "execute_search_enriched",
            params,
            result_size,
            duration,
            true,
        );
        results
    }
}

/// Convenience constructor — wraps a fresh CodeGraph.
impl Default for CountingExecutor {
    fn default() -> Self {
        Self::new(Arc::new(Mutex::new(CodeGraph::new())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    fn executor() -> CountingExecutor {
        CountingExecutor::default()
    }

    /// 创建小规模 fixture 项目 (3-5 个 .rs 文件) 用于快速测试
    fn fixture_project() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("create temp dir");
        let src = dir.path().join("src");
        std::fs::create_dir_all(&src).expect("create src dir");

        let files = [
            ("main.rs", "mod math;\nfn main() { math::add(1, 2); }\n"),
            ("math.rs", "pub fn add(a: i32, b: i32) -> i32 { a + b }\npub fn sub(a: i32, b: i32) -> i32 { a - b }\n"),
            ("utils.rs", "pub struct Config { pub name: String }\npub fn greet(c: &Config) -> String { format!(\"Hello {}\", c.name) }\n"),
            ("lib.rs", "pub mod math;\npub mod utils;\n"),
        ];
        for (name, content) in &files {
            let mut f = std::fs::File::create(src.join(name)).expect("create file");
            f.write_all(content.as_bytes()).expect("write file");
        }
        (dir, src)
    }

    #[test]
    fn test_search_enriched_on_built_graph() {
        let (_dir, src) = fixture_project();
        let e = executor();
        e.execute_stats(src.to_str().expect("src path is valid utf-8"))
            .expect("execute_stats should succeed");
        let results = e.execute_search_enriched("main", 10);
        assert!(!results.is_empty(), "should find nodes matching 'main'");
        for r in &results {
            assert!(!r.node_id.is_empty());
            assert!(!r.node_name.is_empty());
        }
    }

    #[test]
    fn test_search_enriched_respects_max() {
        let (_dir, src) = fixture_project();
        let e = executor();
        e.execute_stats(src.to_str().expect("src path is valid utf-8"))
            .expect("execute_stats should succeed");
        let results = e.execute_search_enriched("e", 3);
        assert!(results.len() <= 3);
    }

    #[test]
    fn test_call_count_starts_zero() {
        let e = executor();
        assert_eq!(e.call_count(), 0);
    }

    #[test]
    fn test_reset_count() {
        let e = executor();
        e.call_count.fetch_add(5, Ordering::Relaxed);
        assert_eq!(e.call_count(), 5);
        e.reset_count();
        assert_eq!(e.call_count(), 0);
    }

    #[test]
    fn test_take_traces_drains() {
        let e = executor();
        let mut p = HashMap::new();
        p.insert("a".into(), "b".into());
        e.record("test", p, 10, 5, true);
        assert_eq!(e.take_traces().len(), 1);
        assert_eq!(e.take_traces().len(), 0);
    }

    #[test]
    fn test_execute_stats_builds_graph() {
        let root =
            PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()));
        let src = root.join("src/neotrix/nt_mind/element");
        if !src.exists() {
            return;
        }
        let e = executor();
        let stats = e
            .execute_stats(src.to_str().expect("src path is valid utf-8"))
            .expect("execute_stats should succeed");
        assert!(stats.total_nodes > 0);
        assert!(stats.community_count >= 1);
        assert!(e.call_count() > 0);
    }

    #[test]
    fn test_execute_impact_missing_node() {
        let e = executor();
        let r = e.execute_impact("nonexistent", 3);
        assert!(r.is_err());
        assert_eq!(e.call_count(), 1);
        let traces = e.take_traces();
        assert!(!traces[0].success);
    }

    #[test]
    fn test_execute_dependencies_empty() {
        let e = executor();
        let deps = e
            .execute_dependencies("src/main.rs")
            .expect("execute_dependencies should succeed");
        assert!(deps.is_empty());
    }

    #[test]
    fn test_execute_dependents_empty() {
        let e = executor();
        let deps = e
            .execute_dependents("src/main.rs")
            .expect("execute_dependents should succeed");
        assert!(deps.is_empty());
    }

    #[test]
    fn test_trace_contains_tool_name() {
        let e = executor();
        let _ = e.execute_impact("x", 1);
        let traces = e.take_traces();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].tool_name, "execute_impact");
    }

    #[test]
    fn test_sequential_calls_increment_count() {
        let e = executor();
        assert_eq!(e.call_count(), 0);
        let _ = e.execute_impact("x", 1);
        assert_eq!(e.call_count(), 1);
        let _ = e.execute_impact("y", 2);
        assert_eq!(e.call_count(), 2);
    }
}
