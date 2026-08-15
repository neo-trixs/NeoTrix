use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSearchResult {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct CodeSearchEngine;

impl CodeSearchEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn search(query: &str, path: &Path) -> Vec<CodeSearchResult> {
        let output = std::process::Command::new("rg")
            .arg("--line-number")
            .arg("--column")
            .arg("--color")
            .arg("never")
            .arg(query)
            .arg(path)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.splitn(4, ':').collect();
                        if parts.len() >= 3 {
                            Some(CodeSearchResult {
                                file: parts[0].to_string(),
                                line: parts[1].parse().unwrap_or(0),
                                column: parts[2].parse().unwrap_or(0),
                                content: parts.get(3).unwrap_or(&"").to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            _ => vec![],
        }
    }

    pub fn search_hybrid(query: &str, path: &Path) -> Vec<RankedHit> {
        let index = SymbolIndex::build(path);
        let results = Self::search(query, path);
        index.rank(results, query)
    }

    pub fn file_symbol_count(path: &Path) -> usize {
        let Some(content) = std::fs::read_to_string(path).ok() else {
            return 0;
        };
        extract_symbols(&path.to_string_lossy(), &content).len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRecord {
    pub file: String,
    pub line: usize,
    pub kind: String,
    pub name: String,
    pub signature: String,
}

#[derive(Debug, Clone)]
pub struct RankedHit {
    pub result: CodeSearchResult,
    pub score: f64,
    pub nearby_symbols: Vec<String>,
}

pub struct SymbolIndex {
    symbols: Vec<SymbolRecord>,
    by_name: HashMap<String, Vec<usize>>,
    by_file: HashMap<String, Vec<usize>>,
}

const CODE_EXTENSIONS: &[&str] = &[
    "rs", "py", "ts", "tsx", "js", "jsx", "go", "java", "rb", "cpp", "c", "h", "hpp", "kt",
    "swift", "toml",
];

impl SymbolIndex {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            by_name: HashMap::new(),
            by_file: HashMap::new(),
        }
    }

    pub fn build(root: &Path) -> Self {
        let mut index = Self::new();
        if root.is_file() {
            if let Some(content) = std::fs::read_to_string(root).ok() {
                index.add_file(root, &content);
            }
            return index;
        }
        index.walk_dir(root);
        index
    }

    fn walk_dir(&mut self, dir: &Path) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let fname = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if fname.starts_with('.') || fname == "target" || fname == "node_modules" {
                continue;
            }
            if path.is_dir() {
                self.walk_dir(&path);
            } else if path.is_file() {
                let ext = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();
                if !CODE_EXTENSIONS.contains(&ext.as_str()) {
                    continue;
                }
                if let Some(content) = std::fs::read_to_string(&path).ok() {
                    self.add_file(&path, &content);
                }
            }
        }
    }

    fn add_file(&mut self, path: &Path, content: &str) {
        let file = path.to_string_lossy().to_string();
        let records = extract_symbols(&file, content);
        let base = self.symbols.len();
        for (i, rec) in records.iter().enumerate() {
            let idx = base + i;
            self.by_name
                .entry(rec.name.to_lowercase())
                .or_default()
                .push(idx);
            self.by_file.entry(file.clone()).or_default().push(idx);
        }
        self.symbols.extend(records);
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    pub fn search_symbols(&self, query: &str) -> Vec<&SymbolRecord> {
        let q = query.to_lowercase();
        let mut hits: Vec<&SymbolRecord> = self
            .symbols
            .iter()
            .filter(|s| s.name.to_lowercase().contains(&q))
            .collect();
        hits.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
        hits
    }

    /// 全部符号 (供模块拓扑统计)。
    pub fn all_symbols(&self) -> &[SymbolRecord] {
        &self.symbols
    }

    pub fn file_symbol_count(&self, file: &str) -> usize {
        self.by_file.get(file).map(|v| v.len()).unwrap_or(0)
    }

    pub fn context_bundle(&self, query: &str, max_entries: usize) -> String {
        let hits = self.search_symbols(query);
        let mut out = String::new();
        for rec in hits.iter().take(max_entries) {
            out.push_str(&format!(
                "{}:{} [{}] {}\n",
                rec.file, rec.line, rec.kind, rec.signature
            ));
        }
        out
    }

    pub fn rank(&self, results: Vec<CodeSearchResult>, query: &str) -> Vec<RankedHit> {
        let tokens: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        if tokens.is_empty() {
            return Vec::new();
        }

        let mut ranked: Vec<RankedHit> = results
            .into_iter()
            .map(|result| {
                let content_lower = result.content.to_lowercase();
                let file_lower = result.file.to_lowercase();
                let mut lexical = 0.0;
                let mut file_symbol = 0.0;
                let mut proximity = false;

                for tok in &tokens {
                    if content_lower.contains(tok) {
                        lexical += 1.0;
                    }
                    if file_lower.contains(tok) {
                        lexical += 0.5;
                    }
                    if let Some(symbols) = self.by_file.get(&result.file) {
                        let mut relevant = 0usize;
                        let mut near = false;
                        for &si in symbols {
                            let sym = &self.symbols[si];
                            if sym.name.to_lowercase().contains(tok) {
                                relevant += 1;
                            }
                            if sym.line.abs_diff(result.line) <= 10 {
                                near = true;
                            }
                        }
                        file_symbol += relevant as f64;
                        if near {
                            proximity = true;
                        }
                    }
                }

                let nearby: Vec<String> = self
                    .by_file
                    .get(&result.file)
                    .map(|symbols| {
                        symbols
                            .iter()
                            .filter(|&&si| self.symbols[si].line.abs_diff(result.line) <= 10)
                            .map(|&si| self.symbols[si].name.clone())
                            .collect()
                    })
                    .unwrap_or_default();

                let score = lexical + file_symbol * 1.5 + if proximity { 2.0 } else { 0.0 };
                RankedHit {
                    result,
                    score,
                    nearby_symbols: nearby,
                }
            })
            .collect();

        ranked.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked
    }
}

fn extract_symbols(file: &str, content: &str) -> Vec<SymbolRecord> {
    let fn_re = regex::Regex::new(
        r"^(?:pub(?:\s*\([^)]*\))?\s+)?(?:async\s+|unsafe\s+|extern\s+|const\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)",
    )
    .unwrap();
    let item_re = regex::Regex::new(
        r"^(?:pub(?:\s*\([^)]*\))?\s+)?(struct|enum|trait|mod|type|const|static|impl|def)\s+([a-zA-Z_][a-zA-Z0-9_:<>]*)",
    )
    .unwrap();
    let mut records = Vec::new();
    for (i, raw) in content.lines().enumerate() {
        let line = raw.trim();
        if line.starts_with("//") || line.starts_with('#') {
            continue;
        }
        let (kind, name): (String, String) =
            if let Some(cap) = fn_re.captures(line) {
                (
                    "fn".to_string(),
                    cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string(),
                )
            } else if let Some(cap) = item_re.captures(line) {
                let kind = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                let name = cap.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
                (kind, name)
            } else {
                continue;
            };
        if name.is_empty() {
            continue;
        }
        let signature: String = raw.chars().take(120).collect();
        records.push(SymbolRecord {
            file: file.to_string(),
            line: i + 1,
            kind,
            name,
            signature,
        });
    }
    records
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_code_search_engine_creation() {
        let _engine = CodeSearchEngine::new();
    }

    #[test]
    #[ignore = "flaky: depends on ripgrep installation and filesystem state"]
    fn test_search_returns_results() {
        let dir = std::env::temp_dir().join("neotrix_code_search_test");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("test.rs");
        let mut file = std::fs::File::create(&file_path).unwrap();
        writeln!(file, "fn hello() {{}}").unwrap();
        writeln!(file, "// test comment").unwrap();
        writeln!(file, "fn world() {{}}").unwrap();

        let results = CodeSearchEngine::search("hello", &dir);
        assert!(results.len() >= 1);
        assert!(results.iter().any(|r| r.content.contains("hello")));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_extract_symbols_finds_definitions() {
        let content = "fn hello() {}\npub struct World {}\nimpl World {}\n// not a symbol\n# not a symbol\nfn hidden";
        let records = extract_symbols("test.rs", content);
        assert!(records.iter().any(|r| r.name == "hello" && r.kind == "fn"));
        assert!(records.iter().any(|r| r.name == "World" && r.kind == "struct"));
        assert!(records.iter().any(|r| r.kind == "impl"));
        assert!(!records.iter().any(|r| r.name.is_empty()));
    }

    #[test]
    fn test_symbol_index_build_and_search() {
        let dir = std::env::temp_dir().join("neotrix_sym_idx_test");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("a.rs"), "fn alpha() {}\npub fn beta() {}").unwrap();
        std::fs::write(dir.join("b.py"), "def alpha():\n    pass").unwrap();

        let index = SymbolIndex::build(&dir);
        assert!(index.len() >= 3);

        let hits = index.search_symbols("alpha");
        assert!(!hits.is_empty());
        assert!(hits.iter().all(|s| s.name.contains("alpha") || s.name.to_lowercase().contains("alpha")));

        let bundle = index.context_bundle("alpha", 10);
        assert!(!bundle.is_empty());
        assert!(bundle.contains("alpha"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_rank_prefers_proximity_and_symbols() {
        let dir = std::env::temp_dir().join("neotrix_rank_test");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(
            dir.join("main.rs"),
            "fn process() {}\n// filler\n// filler\n// filler\nprocess(); // usage line",
        )
        .unwrap();

        let index = SymbolIndex::build(&dir);
        let results = vec![
            CodeSearchResult {
                file: dir.join("main.rs").to_string_lossy().to_string(),
                line: 5,
                column: 1,
                content: "process(); // usage line".to_string(),
            },
            CodeSearchResult {
                file: "other.txt".to_string(),
                line: 1,
                column: 1,
                content: "process".to_string(),
            },
        ];
        let ranked = index.rank(results, "process");
        assert!(!ranked.is_empty());
        assert_eq!(ranked[0].result.line, 5);
        assert!(!ranked[0].nearby_symbols.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_symbol_count() {
        let dir = std::env::temp_dir().join("neotrix_symcount_test");
        let _ = std::fs::create_dir_all(&dir);
        let p = dir.join("x.rs");
        std::fs::write(&p, "fn a() {}\nfn b() {}\n").unwrap();
        assert_eq!(CodeSearchEngine::file_symbol_count(&p), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}