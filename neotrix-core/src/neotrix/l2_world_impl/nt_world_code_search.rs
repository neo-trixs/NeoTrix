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
        // A4 (Axon) 生产接线: RRF 融合符号名命中与正文命中两路排序。
        // 符号名 (identifier) 是精确信号, 正文 (ripgrep) 是召回信号 —
        // 用排名而非原始分融合, 抗不同评分尺度, 双路命中排名靠前者升顶。
        let symbol_files: Vec<String> = index
            .search_symbols(query)
            .iter()
            .map(|s| s.file.clone())
            .collect();
        let content_files: Vec<String> = results.iter().map(|r| r.file.clone()).collect();
        let fused: Vec<RrfHit> = if symbol_files.is_empty() {
            content_files
                .iter()
                .enumerate()
                .map(|(i, f)| RrfHit {
                    file: f.clone(),
                    score: 1.0 / (RRF_K + (i + 1) as f64),
                })
                .collect()
        } else {
            reciprocal_rank_fusion(&[&symbol_files, &content_files], RRF_K)
        };
        let mut ranked = index.rank(results, query);
        ranked.sort_by(|a, b| {
            let fa = fused
                .iter()
                .find(|h| h.file == a.result.file)
                .map(|h| h.score)
                .unwrap_or(0.0);
            let fb = fused
                .iter()
                .find(|h| h.file == b.result.file)
                .map(|h| h.score)
                .unwrap_or(0.0);
            fb.partial_cmp(&fa)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        ranked
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

// ────────────────────────────────────────────────────────────────
// A4 吸收 (Axon, harshkedia177/axon): RRF 混合检索 + 深度分组冲击分析。
// Axon: 预计算结构 → 每次工具调用返回完整上下文; 冲击分析按深度分组
// (will break / may break / review) 带置信度。这里注入 SymbolIndex 作为
// 生产检索层 (R-P42 强化现有节点, 禁平行适配器)。
// ────────────────────────────────────────────────────────────────

/// A4 RRF 常数 (Reciprocal Rank Fusion 标准 60)。
pub const RRF_K: f64 = 60.0;

/// A4 冲击深度 — 被变更影响的程度 (Axon 的 will/may/review 分组)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ImpactDepth {
    /// will break: 直接依赖/同名命中, 高置信。
    WillBreak = 0,
    /// may break: 同文件邻近/强关联, 中置信。
    MayBreak = 1,
    /// review: 弱关联, 需人工复核。
    Review = 2,
}

impl ImpactDepth {
    pub fn label(self) -> &'static str {
        match self {
            ImpactDepth::WillBreak => "will-break",
            ImpactDepth::MayBreak => "may-break",
            ImpactDepth::Review => "review",
        }
    }
}

/// A4 冲击分析单条结果 — 符号 + 所属深度 + 置信度 [0,1]。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactResult {
    pub symbol: SymbolRecord,
    pub depth: ImpactDepth,
    pub confidence: f64,
}

/// A4 混合检索单条融合结果 — doc + RRF 融合分。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RrfHit {
    pub file: String,
    pub score: f64,
}

/// A4: Reciprocal Rank Fusion — 融合多路排序列表 (BM25/语义/模糊) 为单一
/// 排序。RRF 对排名敏感而非原始分, 天然抗不同评分尺度。召回集 = 并集,
/// 排名 r = 出现位置 (1-based), 得分 = Σ 1/(k+r)。
pub fn reciprocal_rank_fusion(lists: &[&[String]], k: f64) -> Vec<RrfHit> {
    let mut scores: HashMap<String, f64> = HashMap::new();
    for list in lists {
        for (rank, item) in list.iter().enumerate() {
            let r = (rank + 1) as f64;
            *scores.entry(item.clone()).or_insert(0.0) += 1.0 / (k + r);
        }
    }
    let mut hits: Vec<RrfHit> = scores
        .into_iter()
        .map(|(file, score)| RrfHit { file, score })
        .collect();
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    hits
}

const CODE_EXTENSIONS: &[&str] = &[
    "rs", "py", "ts", "tsx", "js", "jsx", "go", "java", "rb", "cpp", "c", "h", "hpp", "kt",
    "swift", "toml",
];

impl Default for SymbolIndex {
    fn default() -> Self {
        Self::new()
    }
}

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
            if let Ok(content) = std::fs::read_to_string(root) {
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
                if let Ok(content) = std::fs::read_to_string(&path) {
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

    /// A4 (Axon) 冲击分析: 变更符号列表 → 按深度分组的受影响符号 +
    /// 置信度。WillBreak = 同名符号被变更符号命中 (精确依赖, 置信 1.0);
    /// MayBreak = 与变更符号同文件且行距 ≤10 (邻近调用, 置信 0.8);
    /// Review = 同文件弱关联 (行距 >10, 置信 0.5)。供变更前风险评估
    /// (R-P85 blast-radius 语义在符号层的落地)。
    pub fn analyze_impact(&self, changed: &[&str]) -> Vec<ImpactResult> {
        let mut results = Vec::new();
        let mut seen: HashMap<(String, u32), ()> = HashMap::new();
        for &name in changed {
            let lower = name.to_lowercase();
            // WillBreak: 同名符号 (精确依赖)。
            if let Some(idxs) = self.by_name.get(&lower) {
                for &i in idxs {
                    let sym = &self.symbols[i];
                    let key = (sym.file.clone(), sym.line as u32);
                    if seen.insert(key, ()).is_none() {
                        results.push(ImpactResult {
                            symbol: sym.clone(),
                            depth: ImpactDepth::WillBreak,
                            confidence: 1.0,
                        });
                    }
                }
            }
            // MayBreak/Review: 同文件邻近符号。
            for (file, idxs) in &self.by_file {
                let mut has_target = false;
                for &i in idxs {
                    if self.symbols[i].name.to_lowercase() == lower {
                        has_target = true;
                        break;
                    }
                }
                if !has_target {
                    continue;
                }
                for &i in idxs {
                    let sym = &self.symbols[i];
                    if sym.name.to_lowercase() == lower {
                        continue;
                    }
                    let key = (sym.file.clone(), sym.line as u32);
                    let target_line = self
                        .by_name
                        .get(&lower)
                        .and_then(|v| v.first())
                        .map(|&ti| self.symbols[ti].line)
                        .unwrap_or(sym.line);
                    let near = sym.line.abs_diff(target_line) <= 10;
                    let (depth, confidence) = if near {
                        (ImpactDepth::MayBreak, 0.8)
                    } else {
                        (ImpactDepth::Review, 0.5)
                    };
                    if seen.insert(key, ()).is_none() {
                        results.push(ImpactResult {
                            symbol: sym.clone(),
                            depth,
                            confidence,
                        });
                    }
                }
                let _ = file;
            }
        }
        results.sort_by(|a, b| {
            a.depth
                .cmp(&b.depth)
                .then_with(|| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal))
        });
        results
    }

    /// A4 (Axon) 深度分组统计: 各深度受影响符号计数。
    pub fn impact_summary(&self, changed: &[&str]) -> (usize, usize, usize) {
        let results = self.analyze_impact(changed);
        let will = results
            .iter()
            .filter(|r| r.depth == ImpactDepth::WillBreak)
            .count();
        let may = results
            .iter()
            .filter(|r| r.depth == ImpactDepth::MayBreak)
            .count();
        let review = results
            .iter()
            .filter(|r| r.depth == ImpactDepth::Review)
            .count();
        (will, may, review)
    }
}

fn extract_symbols(file: &str, content: &str) -> Vec<SymbolRecord> {
    let fn_re = regex::Regex::new(
        r"^(?:pub(?:\s*\([^)]*\))?\s+)?(?:async\s+|unsafe\s+|extern\s+|const\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)",
    )
    .expect("valid fn regex");
    let item_re = regex::Regex::new(
        r"^(?:pub(?:\s*\([^)]*\))?\s+)?(struct|enum|trait|mod|type|const|static|impl|def)\s+([a-zA-Z_][a-zA-Z0-9_:<>]*)",
    )
    .expect("valid item regex");
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

    // ── A4 (Axon): RRF 混合检索 + 深度分组冲击分析 ──
    #[test]
    fn test_rrf_fuses_rankings() {
        let list1: Vec<String> = vec!["b.rs".into(), "a.rs".into(), "c.rs".into()];
        let list2: Vec<String> = vec!["b.rs".into(), "a.rs".into(), "d.rs".into()];
        let fused = reciprocal_rank_fusion(&[&list1, &list2], RRF_K);
        assert_eq!(fused.len(), 4, "并集 4 个文件");
        // b.rs 在两个列表都排第 1 → 融合分最高。
        assert_eq!(fused[0].file, "b.rs", "双列表高排名 → 最高分");
        assert!(fused[0].score > fused[1].score);
    }

    #[test]
    fn test_impact_analysis_groups_by_depth() {
        let dir = std::env::temp_dir().join("neotrix_impact_test");
        let _ = std::fs::create_dir_all(&dir);
        let mut body = String::from("fn target() {}\nfn helper() {}\n");
        // 让 far 与 target 相距 >10 行 (review 分组需要远距离弱关联)。
        for _ in 0..14 {
            body.push_str("// filler\n");
        }
        body.push_str("fn far() {}\n");
        std::fs::write(dir.join("a.rs"), body).unwrap();
        let index = SymbolIndex::build(&dir);
        let (will, may, review) = index.impact_summary(&["target"]);
        assert_eq!(will, 1, "同名符号 → will-break");
        assert!(may >= 1, "邻近 helper → may-break");
        assert!(review >= 1, "远离 far → review");
        let results = index.analyze_impact(&["target"]);
        assert!(
            results.iter().all(|r| r.confidence <= 1.0),
            "置信度 ≤ 1.0"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}