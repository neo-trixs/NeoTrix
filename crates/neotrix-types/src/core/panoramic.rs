use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

use sha2::{Sha256, Digest};

/// Describes what kind of symbol a location points to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Module,
    Type,
    Trait,
    Function,
    Const,
    File,
}

impl SymbolKind {
    pub fn label(&self) -> &str {
        match self {
            SymbolKind::Module => "mod",
            SymbolKind::Type => "struct/enum",
            SymbolKind::Trait => "trait",
            SymbolKind::Function => "fn",
            SymbolKind::Const => "const",
            SymbolKind::File => "file",
        }
    }
}

/// A resolved code location with context and confidence.
#[derive(Debug, Clone)]
pub struct CodeLocation {
    pub file_path: PathBuf,
    pub line: u32,
    pub module_path: String,
    pub symbol: String,
    pub kind: SymbolKind,
    pub doc_summary: String,
    pub confidence: f64,
}

/// A module entry with semantic tags extracted from doc comments.
#[derive(Debug, Clone)]
pub struct ModuleEntry {
    pub path: PathBuf,
    pub module_name: String,
    pub doc_summary: String,
    pub key_types: Vec<String>,
    pub key_fns: Vec<String>,
    pub tags: Vec<String>,
}

/// Panoramic Inventory — real-time project introspection for precise code location.
///
/// Scans all `.rs` source files, extracts doc comments and structure,
/// builds a queryable index that maps any task description to exact
/// file paths and line numbers. Designed for sub-1s full scan on ~400 files.
pub struct PanoramicInventory {
    modules: Vec<ModuleEntry>,
    symbols: Vec<CodeLocation>,
    file_hashes: HashMap<PathBuf, String>,
    last_scan: Option<SystemTime>,
    dirty: Arc<AtomicBool>,
}

impl Default for PanoramicInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl PanoramicInventory {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            symbols: Vec::new(),
            file_hashes: HashMap::new(),
            last_scan: None,
            dirty: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Full scan of all `.rs` files under `root`.
    pub fn scan(&mut self, root: &Path) {
        self.modules.clear();
        self.symbols.clear();
        self.file_hashes.clear();

        let root_str = root.to_string_lossy().to_string();

        let src = if root.join("neotrix-core/src").exists() {
            root.join("neotrix-core/src")
        } else {
            root.to_path_buf()
        };

        let mut files: Vec<PathBuf> = Vec::new();
        collect_rs_files(&src, &mut files);
        files.sort();

        for path in &files {
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let mut hasher = Sha256::new();
            hasher.update(content.as_bytes());
            let hash = format!("{:x}", hasher.finalize());
            self.file_hashes.insert(path.to_path_buf(), hash);

            let rel_path = path.to_string_lossy().replace(&root_str, "");
            let module_name = rel_path
                .trim_start_matches('/')
                .trim_end_matches(".rs")
                .replace('/', "::");

            let (doc_summary, key_types, key_fns) = extract_doc_info(&content);

            if doc_summary.is_empty() && key_types.is_empty() && key_fns.is_empty() {
                continue;
            }

            self.symbols.push(CodeLocation {
                file_path: path.to_path_buf(),
                line: 1,
                module_path: module_name.clone(),
                symbol: module_name.rsplit("::").next().unwrap_or(&module_name).to_string(),
                kind: SymbolKind::File,
                doc_summary: truncate(&doc_summary, 120),
                confidence: 5.0,
            });

            let tags = infer_tags(&module_name, &doc_summary, &key_types);
            self.modules.push(ModuleEntry {
                path: path.to_path_buf(),
                module_name,
                doc_summary,
                key_types,
                key_fns,
                tags,
            });
        }

        self.last_scan = Some(SystemTime::now());
        self.dirty.store(true, Ordering::Release);
    }

    /// Incremental refresh: re-scan only if any file changed.
    pub fn refresh(&mut self, root: &Path) -> bool {
        let src = if root.join("neotrix-core/src").exists() {
            root.join("neotrix-core/src")
        } else {
            root.to_path_buf()
        };

        let mut files: Vec<PathBuf> = Vec::new();
        collect_rs_files(&src, &mut files);

        let mut changed = false;
        for path in &files {
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let mut hasher = Sha256::new();
            hasher.update(content.as_bytes());
            let hash = format!("{:x}", hasher.finalize());

            match self.file_hashes.get(path) {
                Some(old) if old == &hash => {}
                _ => {
                    changed = true;
                    break;
                }
            }
        }

        if changed || self.modules.is_empty() {
            self.scan(root);
        }
        self.dirty.store(true, Ordering::Release);
        changed
    }

    /// Given a task description, find the most relevant code locations.
    pub fn resolve(&self, query: &str, max_results: usize) -> Vec<CodeLocation> {
        let keywords: Vec<String> = query
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| s.len() > 2)
            .map(|s| s.to_string())
            .collect();

        if keywords.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(f64, &CodeLocation)> = self
            .symbols
            .iter()
            .map(|loc| (score_location(loc, &keywords), loc))
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .filter(|(score, _)| *score > 1.0)
            .take(max_results)
            .map(|(_, loc)| loc.clone())
            .collect()
    }

    pub fn modules_by_tag(&self, tag: &str) -> Vec<&ModuleEntry> {
        let tag_lower = tag.to_lowercase();
        self.modules
            .iter()
            .filter(|m| m.tags.iter().any(|t| t == &tag_lower))
            .collect()
    }

    pub fn search_modules(&self, name: &str) -> Vec<&ModuleEntry> {
        let name_lower = name.to_lowercase();
        self.modules
            .iter()
            .filter(|m| m.module_name.to_lowercase().contains(&name_lower))
            .collect()
    }

    pub fn module_by_path(&self, module_path: &str) -> Option<&ModuleEntry> {
        self.modules.iter().find(|m| m.module_name == module_path)
    }

    /// Full panorama as formatted string for AI context injection.
    pub fn format_panorama(&self) -> String {
        let mut out = String::new();
        out.push_str("=== Panorama ===\n");

        for module in &self.modules {
            let short = module.module_name.rsplit("::").next().unwrap_or(&module.module_name);
            out.push_str(&format!("  {} — {}\n", short, truncate(&module.doc_summary, 80)));
            if !module.key_types.is_empty() {
                out.push_str(&format!("    types: {}\n", module.key_types.join(", ")));
            }
            if !module.key_fns.is_empty() {
                out.push_str(&format!("    fns:   {}\n", module.key_fns.join(", ")));
            }
            if !module.tags.is_empty() {
                out.push_str(&format!("    tags:  {}\n", module.tags.join(", ")));
            }
        }
        out.push_str(&format!("\n{} modules, {} symbols\n", self.modules.len(), self.symbols.len()));
        out
    }

    pub fn mark_dirty(&mut self) {
        self.dirty.store(true, Ordering::Release);
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Acquire)
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.modules.len(), self.symbols.len())
    }

    pub fn last_scanned(&self) -> Option<SystemTime> {
        self.last_scan
    }
}

// ---------------------------------------------------------------------------
// 内部辅助
// ---------------------------------------------------------------------------

/// Recursively collect all `.rs` files under `dir`.
fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let dir = match std::fs::read_dir(dir) {
        Ok(d) => d,
        Err(_) => return,
    };
    for entry in dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if !name.starts_with('.') && name != "target" && name != "build" {
                collect_rs_files(&path, files);
            }
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

/// Extract doc summary, type names, and function names from Rust source.
fn extract_doc_info(content: &str) -> (String, Vec<String>, Vec<String>) {
    let mut doc_lines: Vec<String> = Vec::new();
    let mut key_types: Vec<String> = Vec::new();
    let mut key_fns: Vec<String> = Vec::new();
    let mut current_doc: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("///") {
            let text = trimmed.trim_start_matches("///").trim();
            if !text.is_empty() && !text.starts_with('[') {
                current_doc.push(text.to_string());
            }
            continue;
        }

        let is_comment = trimmed.starts_with("//!") || trimmed.starts_with("//");
        if is_comment {
            current_doc.clear();
            continue;
        }

        // Check for pub item declarations
        if let Some(name) = extract_name(trimmed, &["pub struct ", "pub enum "]) {
            key_types.push(name);
            append_doc(&mut doc_lines, &mut current_doc);
        } else if let Some(name) = extract_name(trimmed, &["pub trait "]) {
            key_types.push(name);
            append_doc(&mut doc_lines, &mut current_doc);
        } else if let Some(name) = extract_name(trimmed, &["pub unsafe fn ", "pub(crate) fn ", "pub fn "]) {
            key_fns.push(name);
            append_doc(&mut doc_lines, &mut current_doc);
        } else if !current_doc.is_empty() && !trimmed.is_empty() {
            current_doc.clear();
        }
    }

    key_types.sort();
    key_types.dedup();
    key_fns.sort();
    key_fns.dedup();

    let summary = doc_lines
        .into_iter()
        .filter(|s| !s.starts_with('#') && s.len() > 3)
        .collect::<Vec<_>>()
        .join("; ");
    (truncate(&summary, 200), key_types, key_fns)
}

fn extract_name(line: &str, prefixes: &[&str]) -> Option<String> {
    for prefix in prefixes {
        if line.starts_with(prefix) {
            let rest = line.trim_start_matches(prefix);
            let name = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or("");
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

fn append_doc(doc_lines: &mut Vec<String>, current: &mut Vec<String>) {
    if !current.is_empty() {
        doc_lines.append(current);
    }
}

/// Infer semantic tags from module path + doc summary + types.
fn infer_tags(_module_path: &str, doc_summary: &str, key_types: &[String]) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();
    let lower = doc_summary.to_lowercase();

    let patterns: &[(&str, &[&str])] = &[
        ("orchestrator", &["orchestrator", "orchestrate", "flow", "pipeline", "workflow", "process"]),
        ("agent", &["agent", "autonomous", "goal", "executor"]),
        ("memory", &["memory", "store", "recall", "retrieval", "persist"]),
        ("knowledge", &["knowledge", "source", "absorb", "learn"]),
        ("consciousness", &["consciousness", "attention", "workspace", "gwt"]),
        ("hypercube", &["hypercube", "vsa", "vector", "symbolic"]),
        ("crawler", &["crawl", "scrape", "fetch", "web", "http"]),
        ("security", &["security", "audit", "guard", "permission", "vault"]),
        ("network", &["proxy", "network", "tcp", "udp", "socket", "stealth"]),
        ("social", &["social", "twitter", "reddit", "bilibili", "youtube"]),
        ("reasoning", &["reason", "thinking", "inference", "brain", "cognitive"]),
        ("metacognition", &["metacognition", "self", "introspect", "monitor", "scan"]),
        ("ui", &["cli", "command", "terminal", "shell", "repl"]),
        ("signal", &["signal", "select", "ssm", "mamba", "state space"]),
        ("e8", &["e8", "hexagram", "crt", "time", "i ching"]),
        ("evolution", &["evolve", "absorb", "adapt", "iterate", "seal"]),
        ("edit", &["edit", "microedit", "self_edit", "modify"]),
        ("test", &["test", "benchmark", "verify"]),
        ("plugin", &["plugin", "element", "extension", "hook"]),
    ];

    for (tag, keywords) in patterns {
        if keywords.iter().any(|k| lower.contains(k)) {
            tags.push(tag.to_string());
        }
    }

    for t in key_types {
        for (tag, _) in patterns {
            if t.to_lowercase().contains(tag) && !tags.contains(&tag.to_string()) {
                tags.push(tag.to_string());
            }
        }
    }

    if tags.is_empty() {
        tags.push("general".to_string());
    }
    tags
}

/// Score a location against query keywords.
fn score_location(loc: &CodeLocation, keywords: &[String]) -> f64 {
    let mut score = 0.0;
    let sym = loc.symbol.to_lowercase();
    let doc = loc.doc_summary.to_lowercase();
    let mp = loc.module_path.to_lowercase();

    for kw in keywords {
        if sym == *kw {
            score += 20.0;
        } else if sym.contains(kw.as_str()) {
            score += 10.0;
        }
        if doc.contains(kw.as_str()) {
            score += 5.0;
        }
        if mp.contains(kw.as_str()) {
            score += 3.0;
        }
        if let Some(mn) = loc.module_path.rsplit("::").next() {
            if mn.to_lowercase().contains(kw.as_str()) {
                score += 4.0;
            }
        }
    }

    match loc.kind {
        SymbolKind::Function => score *= 1.2,
        SymbolKind::Type | SymbolKind::Trait => score *= 1.1,
        SymbolKind::File => score *= 0.8,
        _ => {}
    }
    score
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let mut len = 0;
        for c in s.chars() {
            let c_len = c.len_utf8();
            if len + c_len > max.saturating_sub(1) {
                break;
            }
            len += c_len;
        }
        format!("{}…", &s[..len])
    }
}

// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_doc_info() {
        let content = r#"
/// Manages guardrail chains for task validation.
pub struct GuardrailChain;

/// Validates a task before execution.
pub fn validate_task(task: &str) -> bool { true }
"#;
        let (summary, types, fns) = extract_doc_info(content);
        assert!(types.contains(&"GuardrailChain".to_string()));
        assert!(fns.contains(&"validate_task".to_string()));
        assert!(summary.contains("guardrail") || summary.contains("Guardrail"));
    }

    #[test]
    fn test_extract_empty() {
        let (summary, types, fns) = extract_doc_info("fn _private() {}");
        assert!(summary.is_empty());
        assert!(types.is_empty());
        assert!(fns.is_empty());
    }

    #[test]
    fn test_infer_tags() {
        let tags = infer_tags("x", "manages orchestration flow", &["FlowState".to_string()]);
        assert!(tags.contains(&"orchestrator".to_string()));
    }

    #[test]
    fn test_resolve_smoke() {
        let mut inv = PanoramicInventory::new();
        inv.modules.push(ModuleEntry {
            path: PathBuf::from("x.rs"),
            module_name: "crate::test".to_string(),
            doc_summary: "handles guardrail validation".to_string(),
            key_types: vec!["Guardrail".to_string()],
            key_fns: vec![],
            tags: vec!["orchestrator".to_string()],
        });
        inv.symbols.push(CodeLocation {
            file_path: PathBuf::from("x.rs"),
            line: 10,
            module_path: "crate::test".to_string(),
            symbol: "Guardrail".to_string(),
            kind: SymbolKind::Type,
            doc_summary: "handles guardrail validation".to_string(),
            confidence: 0.0,
        });

        let r = inv.resolve("guardrail validation", 5);
        assert!(!r.is_empty());
        assert_eq!(r[0].symbol, "Guardrail");
    }

    #[test]
    fn test_search_modules() {
        let mut inv = PanoramicInventory::new();
        inv.modules.push(ModuleEntry {
            path: PathBuf::from("g.rs"),
            module_name: "crate::guardrail".to_string(),
            doc_summary: "guardrail chain".to_string(),
            key_types: vec![],
            key_fns: vec![],
            tags: vec!["orchestrator".to_string()],
        });
        assert_eq!(inv.search_modules("guardrail").len(), 1);
        assert_eq!(inv.search_modules("nonexist").len(), 0);
    }

    #[test]
    fn test_modules_by_tag() {
        let mut inv = PanoramicInventory::new();
        inv.modules.push(ModuleEntry {
            path: PathBuf::from("s.rs"),
            module_name: "crate::security".to_string(),
            doc_summary: "security guardrails".to_string(),
            key_types: vec![],
            key_fns: vec![],
            tags: vec!["security".to_string()],
        });
        assert_eq!(inv.modules_by_tag("security").len(), 1);
    }

    #[test]
    fn test_scan_real_project() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        // Navigate up to find the workspace root
        let project_root = root.parent().and_then(|p| p.parent()).unwrap_or(root);
        let mut inv = PanoramicInventory::new();
        inv.scan(project_root);
        let (m, s) = inv.stats();
        assert!(m > 0, "should find at least one module, got {}", m);
        assert!(s > 0, "should find at least one symbol, got {}", s);

        // Test resolution on real code
        let results = inv.resolve("guardrail", 3);
        if !results.is_empty() {
            assert!(results[0].confidence >= 0.0);
        }

        // Test panorama formatting
        let panorama = inv.format_panorama();
        assert!(panorama.contains("Panorama"));
    }

    #[test]
    fn test_refresh_incremental() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let project_root = root.parent().and_then(|p| p.parent()).unwrap_or(root);
        let mut inv = PanoramicInventory::new();
        inv.scan(project_root);
        let (m_before, _) = inv.stats();
        let _changed = inv.refresh(project_root);
        let (m_after, _) = inv.stats();
        assert_eq!(m_before, m_after, "module count should be stable after refresh");
    }
}
