use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use walkdir::WalkDir;

use super::{
    CHUNK_LINES, CHUNK_OVERLAP, COMPAT_PENALTY, DEFINITION_BOOST, EXAMPLE_PENALTY,
    FILE_COHERENCE_BOOST, FILE_SIZE_LIMIT, SATURATION_DECAY, TEST_PENALTY, VSA_DIM,
};

#[derive(Debug, Clone)]
pub struct CodeChunk {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CodeSearchResult {
    pub chunk: CodeChunk,
    pub score: f64,
    pub match_signals: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CodeIndexEntry {
    pub chunk: CodeChunk,
    pub vsa_vector: Vec<u8>,
    pub tokens: Vec<String>,
}

pub struct CodeIndex {
    entries: Vec<CodeIndexEntry>,
    token_index: HashMap<String, Vec<usize>>,
    file_index: HashMap<String, Vec<usize>>,
    root_path: PathBuf,
    file_mtimes: HashMap<String, SystemTime>,
    pub build_time: Instant,
}

impl CodeIndex {
    pub fn new(root_path: &Path) -> Self {
        Self {
            entries: Vec::new(),
            token_index: HashMap::new(),
            file_index: HashMap::new(),
            root_path: root_path.to_path_buf(),
            file_mtimes: HashMap::new(),
            build_time: Instant::now(),
        }
    }

    pub fn build(&mut self) -> Result<(), String> {
        let mut all_entries = Vec::new();
        let mut token_idx: HashMap<String, Vec<usize>> = HashMap::new();
        let mut file_idx: HashMap<String, Vec<usize>> = HashMap::new();
        let mut mtimes: HashMap<String, SystemTime> = HashMap::new();

        let source_extensions = [
            "rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "rb", "c", "cpp", "h", "hpp",
            "swift", "kt", "scala", "rs", "toml", "json", "yaml", "yml", "md", "sql", "sh", "bash",
            "zsh", "css", "html", "vue", "svelte", "zig", "ex", "exs", "clj", "cljs", "lua",
        ];

        for entry in WalkDir::new(&self.root_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let rel_path = path
                .strip_prefix(&self.root_path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if rel_path.contains("node_modules")
                || rel_path.contains("target/")
                || rel_path.contains(".git/")
                || rel_path.contains(".venv/")
                || rel_path.contains("__pycache__/")
                || rel_path.contains("dist/")
                || rel_path.contains("build/")
                || rel_path.starts_with('.')
            {
                continue;
            }

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !source_extensions.contains(&ext) {
                continue;
            }

            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if metadata.len() > FILE_SIZE_LIMIT || metadata.len() < 4 {
                continue;
            }
            if let Ok(mtime) = metadata.modified() {
                mtimes.insert(rel_path.clone(), mtime);
            }

            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let language = detect_language(ext);
            let chunks = chunk_code(&content, ext);

            for (_chunk_idx, (start_line, end_line, chunk_text)) in chunks.iter().enumerate() {
                if chunk_text.trim().is_empty() {
                    continue;
                }

                let tokens = tokenize_code(chunk_text);
                let vsa_vector = encode_vsa(&tokens);

                let chunk = CodeChunk {
                    file_path: rel_path.clone(),
                    start_line: *start_line,
                    end_line: *end_line,
                    content: chunk_text.clone(),
                    language: language.clone(),
                };

                let entry_idx = all_entries.len();
                all_entries.push(CodeIndexEntry {
                    chunk,
                    vsa_vector,
                    tokens: tokens.clone(),
                });

                file_idx
                    .entry(rel_path.clone())
                    .or_default()
                    .push(entry_idx);

                for token in &tokens {
                    if token.len() > 1 {
                        token_idx.entry(token.clone()).or_default().push(entry_idx);
                    }
                }
            }
        }

        self.entries = all_entries;
        self.token_index = token_idx;
        self.file_index = file_idx;
        self.file_mtimes = mtimes;
        self.build_time = Instant::now();

        Ok(())
    }

    pub fn needs_rebuild(&self) -> bool {
        let source_extensions = [
            "rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "rb", "c", "cpp", "h", "hpp",
            "swift", "kt", "scala", "toml", "json", "yaml", "yml", "md", "sql", "sh", "css",
            "html",
        ];

        for entry in WalkDir::new(&self.root_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let rel = path.strip_prefix(&self.root_path).unwrap_or(path);
            let rel_str = rel.to_string_lossy().to_string();

            if rel_str.contains("node_modules")
                || rel_str.contains("target/")
                || rel_str.contains(".git/")
            {
                continue;
            }

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !source_extensions.contains(&ext) {
                continue;
            }

            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if let Ok(mtime) = metadata.modified() {
                if let Some(cached) = self.file_mtimes.get(&rel_str) {
                    if mtime > *cached {
                        return true;
                    }
                } else {
                    return true;
                }
            }
        }
        false
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn search(&self, query: &str, top_k: usize) -> Vec<CodeSearchResult> {
        if self.entries.is_empty() || query.trim().is_empty() {
            return Vec::new();
        }

        let query_tokens = tokenize_code(query);
        if query_tokens.is_empty() {
            return Vec::new();
        }

        let query_vector = encode_vsa(&query_tokens);
        let is_symbol_query = detect_symbol_query(&query_tokens);

        let top_k = top_k.max(1);

        let mut scored: Vec<(f64, usize)> = Vec::with_capacity(self.entries.len());

        for (idx, entry) in self.entries.iter().enumerate() {
            let vsa_sim = QuantizedVSA::similarity(&query_vector, &entry.vsa_vector);

            let lexical_score = compute_lexical_score(&query_tokens, &entry.tokens);
            let alpha = if is_symbol_query { 0.3 } else { 0.5 };
            let hybrid = alpha * vsa_sim + (1.0 - alpha) * lexical_score;

            if vsa_sim > 0.1 || lexical_score > 0.0 {
                scored.push((hybrid, idx));
            }
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let candidate_count = (top_k * 5).min(scored.len());
        let candidates: Vec<(f64, usize)> = scored.into_iter().take(candidate_count).collect();

        let reranked = self.rerank(candidates, &query_tokens, is_symbol_query);

        let top: Vec<CodeSearchResult> = reranked
            .into_iter()
            .take(top_k)
            .map(|(score, idx)| {
                let entry = &self.entries[idx];
                let mut signals = Vec::new();
                if score > 0.0 {
                    signals.push("reranked".to_string());
                }
                CodeSearchResult {
                    chunk: entry.chunk.clone(),
                    score,
                    match_signals: signals,
                }
            })
            .collect();

        top
    }

    fn rerank(
        &self,
        candidates: Vec<(f64, usize)>,
        query_tokens: &[String],
        is_symbol: bool,
    ) -> Vec<(f64, usize)> {
        if candidates.is_empty() {
            return candidates;
        }

        let max_score = candidates
            .iter()
            .map(|(s, _)| *s)
            .fold(0.0f64, |a, b| a.max(b))
            .max(1e-10);

        let mut boosted: Vec<(f64, usize)> = candidates
            .into_iter()
            .map(|(score, idx)| {
                let entry = &self.entries[idx];
                let mut s = score;

                let path = &entry.chunk.file_path;

                if detect_test_file(path) {
                    s *= TEST_PENALTY;
                }
                if path.contains("compat/") || path.contains("legacy/") || path.contains("_compat/")
                {
                    s *= COMPAT_PENALTY;
                }
                if path.contains("examples/")
                    || path.contains("docs_src/")
                    || path.contains("example/")
                {
                    s *= EXAMPLE_PENALTY;
                }

                if is_symbol
                    || query_tokens.iter().any(|t| {
                        entry.chunk.content.contains(&format!("def {}", t))
                            || entry.chunk.content.contains(&format!("fn {}", t))
                            || entry.chunk.content.contains(&format!("class {}", t))
                            || entry.chunk.content.contains(&format!("struct {}", t))
                            || entry.chunk.content.contains(&format!("pub fn {}", t))
                            || entry.chunk.content.contains(&format!("pub struct {}", t))
                            || entry.chunk.content.contains(&format!("impl {}", t))
                            || entry.chunk.content.contains(&format!("function {}", t))
                            || entry.chunk.content.contains(&format!("func {}", t))
                    })
                {
                    s += max_score * DEFINITION_BOOST;
                }

                (s, idx)
            })
            .collect();

        let mut file_score: HashMap<&str, f64> = HashMap::new();
        for &(score, idx) in &boosted {
            let path = &self.entries[idx].chunk.file_path;
            *file_score.entry(path.as_str()).or_insert(0.0) += score;
        }
        let file_sums: HashMap<&str, f64> = file_score;
        let max_file_sum = file_sums
            .values()
            .copied()
            .fold(0.0f64, f64::max)
            .max(1e-10);

        let mut file_best: HashMap<&str, (f64, usize)> = HashMap::new();
        for &(score, idx) in &boosted {
            let path = self.entries[idx].chunk.file_path.as_str();
            let entry = file_best.entry(path).or_insert((score, idx));
            if score > entry.0 {
                *entry = (score, idx);
            }
        }

        let boost_unit = max_score * FILE_COHERENCE_BOOST;
        for (score, idx) in &mut boosted {
            let path = self.entries[*idx].chunk.file_path.as_str();
            let file_sum = file_sums.get(path).copied().unwrap_or(0.0);
            if file_sum > 0.0 {
                *score += boost_unit * file_sum / max_file_sum;
            }
        }

        boosted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut selected: Vec<(f64, usize)> = Vec::new();
        let mut file_count: HashMap<&str, usize> = HashMap::new();

        for (score, idx) in boosted {
            let path = self.entries[idx].chunk.file_path.as_str();
            let count = file_count.get(path).copied().unwrap_or(0);
            let effective_score = if count > 0 {
                score * SATURATION_DECAY.powi(count as i32)
            } else {
                score
            };
            selected.push((effective_score, idx));
            *file_count.entry(path).or_insert(0) += 1;
        }

        selected.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        selected
    }
}

fn detect_language(ext: &str) -> Option<String> {
    match ext {
        "rs" => Some("rust".into()),
        "py" => Some("python".into()),
        "js" => Some("javascript".into()),
        "ts" | "tsx" => Some("typescript".into()),
        "jsx" => Some("react".into()),
        "go" => Some("go".into()),
        "java" => Some("java".into()),
        "rb" => Some("ruby".into()),
        "c" | "h" => Some("c".into()),
        "cpp" | "hpp" | "cc" => Some("cpp".into()),
        "swift" => Some("swift".into()),
        "kt" => Some("kotlin".into()),
        "scala" => Some("scala".into()),
        "vue" | "svelte" => Some("frontend".into()),
        _ => None,
    }
}

pub fn chunk_code(content: &str, ext: &str) -> Vec<(usize, usize, String)> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let chunk_size = if ext == "json" || ext == "toml" || ext == "yaml" || ext == "yml" {
        CHUNK_LINES * 2
    } else {
        CHUNK_LINES
    };

    let step = chunk_size.saturating_sub(CHUNK_OVERLAP);
    let mut start = 0;

    while start < lines.len() {
        let end = (start + chunk_size).min(lines.len());
        let chunk_text = lines[start..end].join("\n");
        chunks.push((start + 1, end, chunk_text));
        if end >= lines.len() {
            break;
        }
        start += step;
    }

    chunks
}

pub fn tokenize_code(content: &str) -> Vec<String> {
    let mut tokens = Vec::new();

    for word in content.split(|c: char| !c.is_alphanumeric() && c != '_') {
        let word = word.trim();
        if word.is_empty() || word.len() == 1 || is_stopword(word) {
            continue;
        }

        tokens.push(word.to_lowercase());

        if word.contains('_') {
            for part in word.split('_') {
                if part.len() > 1 {
                    tokens.push(part.to_lowercase());
                }
            }
        }

        let camel_parts = split_camel_case(word);
        if camel_parts.len() > 1 {
            for part in camel_parts {
                if part.len() > 1 && !tokens.contains(&part.to_lowercase()) {
                    tokens.push(part.to_lowercase());
                }
            }
        }
    }

    tokens.sort();
    tokens.dedup();
    tokens
}

fn split_camel_case(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_uppercase() {
            if !current.is_empty() && current.len() > 1 {
                parts.push(current.clone());
                current.clear();
            }
            current.push(chars[i]);
        } else {
            current.push(chars[i]);
        }
        i += 1;
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

pub fn encode_vsa(tokens: &[String]) -> Vec<u8> {
    if tokens.is_empty() {
        return vec![0u8; VSA_DIM];
    }

    let vectors: Vec<Vec<u8>> = tokens
        .iter()
        .map(|t| {
            let mut hasher = DefaultHasher::new();
            t.hash(&mut hasher);
            let seed = hasher.finish();
            let mut v = QuantizedVSA::seeded_random(seed, VSA_DIM);
            for byte in &mut v {
                *byte = if *byte >= 128 { 1 } else { 0 };
            }
            v
        })
        .collect();

    let refs: Vec<&[u8]> = vectors.iter().map(|v| v.as_slice()).collect();
    QuantizedVSA::bundle(&refs)
}

pub fn compute_lexical_score(query_tokens: &[String], doc_tokens: &[String]) -> f64 {
    if query_tokens.is_empty() || doc_tokens.is_empty() {
        return 0.0;
    }

    let doc_set: HashSet<&str> = doc_tokens.iter().map(|s| s.as_str()).collect();

    let mut matches = 0usize;
    for qt in query_tokens {
        if doc_set.contains(qt.as_str()) {
            matches += 1;
        }
    }

    matches as f64 / query_tokens.len() as f64
}

pub fn detect_symbol_query(tokens: &[String]) -> bool {
    let symbol_patterns = ["::", "->", "=>", "."];
    for t in tokens {
        for pat in &symbol_patterns {
            if t.contains(pat) {
                return true;
            }
        }
        if t.starts_with('_') || t.starts_with(|c: char| c.is_uppercase()) {
            return true;
        }
    }
    false
}

pub fn detect_test_file(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    if path_lower.contains("test") {
        return true;
    }
    if let Some(stem) = Path::new(path).file_stem().and_then(|s| s.to_str()) {
        if stem.starts_with("test_")
            || stem.ends_with("_test")
            || stem.ends_with("_spec")
            || stem == "tests"
        {
            return true;
        }
    }
    false
}

fn is_stopword(s: &str) -> bool {
    matches!(
        s,
        "a" | "an"
            | "and"
            | "are"
            | "as"
            | "at"
            | "be"
            | "by"
            | "do"
            | "does"
            | "for"
            | "from"
            | "has"
            | "have"
            | "how"
            | "if"
            | "in"
            | "is"
            | "it"
            | "not"
            | "of"
            | "on"
            | "or"
            | "the"
            | "to"
            | "was"
            | "what"
            | "when"
            | "where"
            | "which"
            | "who"
            | "why"
            | "with"
            | "this"
            | "that"
            | "use"
            | "let"
            | "var"
            | "pub"
            | "fn"
            | "mut"
            | "new"
            | "impl"
            | "self"
            | "true"
            | "false"
            | "none"
            | "null"
            | "void"
            | "int"
            | "str"
            | "bool"
            | "i32"
            | "i64"
            | "u32"
            | "u64"
            | "f32"
            | "f64"
            | "string"
            | "option"
            | "result"
            | "vec"
            | "map"
            | "set"
            | "list"
            | "dict"
            | "len"
            | "get"
            | "add"
            | "remove"
            | "find"
            | "search"
            | "sort"
            | "filter"
            | "fold"
            | "iter"
            | "into"
            | "clone"
            | "copy"
            | "debug"
            | "display"
            | "default"
            | "eq"
            | "ord"
            | "hash"
            | "serde"
            | "derive"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_temp_workspace() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();

        let mut file1 = std::fs::File::create(root.join("main.rs")).unwrap();
        writeln!(
            file1,
            "fn main() {{\n    println!(\"Hello\");\n}}\n\nfn add(a: i32, b: i32) -> i32 {{\n    a + b\n}}"
        )
        .unwrap();

        let mut file2 = std::fs::File::create(root.join("utils.py")).unwrap();
        writeln!(
            file2,
            "def parse_config(path: str) -> dict:\n    import json\n    with open(path) as f:\n        return json.load(f)\n\ndef validate(data: dict) -> bool:\n    return 'name' in data"
        )
        .unwrap();

        (dir, root)
    }

    #[test]
    fn test_chunk_code_splits_lines() {
        let content = "a\nb\nc\nd\ne\nf\ng\nh\ni\nj";
        let chunks = chunk_code(content, "rs");
        assert!(!chunks.is_empty());
        for (start, end, text) in &chunks {
            assert!(end > start);
            assert!(!text.is_empty());
        }
    }

    #[test]
    fn test_tokenize_code_splits_identifiers() {
        let tokens = tokenize_code("parseConfig getHTTPResponse");
        assert!(tokens.contains(&"parseconfig".to_string()));
        assert!(tokens.contains(&"parse".to_string()));
        assert!(tokens.contains(&"config".to_string()));
        assert!(tokens.contains(&"gethttpresponse".to_string()));
        assert!(tokens.contains(&"http".to_string()));
        assert!(tokens.contains(&"response".to_string()));
    }

    #[test]
    fn test_tokenize_snake_case() {
        let tokens = tokenize_code("user_auth_token");
        assert!(tokens.contains(&"user_auth_token".to_string()));
        assert!(tokens.contains(&"user".to_string()));
        assert!(tokens.contains(&"auth".to_string()));
        assert!(tokens.contains(&"token".to_string()));
    }

    #[test]
    fn test_encode_vsa_produces_correct_dim() {
        let tokens = vec!["hello".to_string(), "world".to_string()];
        let vec = encode_vsa(&tokens);
        assert_eq!(vec.len(), VSA_DIM);
        assert!(vec.iter().all(|&x| x == 0 || x == 1));
    }

    #[test]
    fn test_vsa_deterministic() {
        let tokens = vec!["test".to_string(), "code".to_string()];
        let v1 = encode_vsa(&tokens);
        let v2 = encode_vsa(&tokens);
        let sim = QuantizedVSA::similarity(&v1, &v2);
        assert!(
            (sim - 1.0).abs() < 1e-10,
            "deterministic encoding should be identical"
        );
    }

    #[test]
    fn test_lexical_score_matches_identical() {
        let query = vec!["auth".to_string(), "config".to_string()];
        let doc = vec![
            "auth".to_string(),
            "config".to_string(),
            "parse".to_string(),
        ];
        let score = compute_lexical_score(&query, &doc);
        assert!((score - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_lexical_score_partial_match() {
        let query = vec![
            "auth".to_string(),
            "config".to_string(),
            "parse".to_string(),
        ];
        let doc = vec!["auth".to_string(), "token".to_string()];
        let score = compute_lexical_score(&query, &doc);
        assert!((score - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_lexical_score_no_match() {
        let query = vec!["aaa".to_string()];
        let doc = vec!["bbb".to_string()];
        let score = compute_lexical_score(&query, &doc);
        assert!((score - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_detect_symbol_query() {
        let sym_tokens = vec!["Foo::bar".to_string(), "test".to_string()];
        let nl_tokens = vec!["how".to_string(), "does".to_string(), "auth".to_string()];
        assert!(detect_symbol_query(&sym_tokens));
        assert!(!detect_symbol_query(&nl_tokens));
    }

    #[test]
    fn test_detect_test_file() {
        assert!(detect_test_file("src/test_auth.rs"));
        assert!(detect_test_file("src/auth_test.py"));
        assert!(!detect_test_file("src/auth.rs"));
        assert!(detect_test_file("tests/test_auth.rs"));
    }

    #[test]
    fn test_code_index_build_and_search() {
        let (_dir, root) = create_temp_workspace();
        let mut index = CodeIndex::new(&root);
        assert!(index.build().is_ok());
        assert!(index.entry_count() > 0);

        let results = index.search("parse config", 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_code_index_search_finds_by_vsa_semantics() {
        let (_dir, root) = create_temp_workspace();
        let mut index = CodeIndex::new(&root);
        index.build().unwrap();

        let results = index.search("configuration parsing", 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_code_index_search_finds_symbols() {
        let (_dir, root) = create_temp_workspace();
        let mut index = CodeIndex::new(&root);
        index.build().unwrap();

        let results = index.search("parse_config", 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_code_index_search_no_results() {
        let (_dir, root) = create_temp_workspace();
        let mut index = CodeIndex::new(&root);
        index.build().unwrap();

        let results = index.search("xyznonexistentkeyword", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_rerank_boosts_definitions() {
        let (_dir, root) = create_temp_workspace();
        let mut index = CodeIndex::new(&root);
        index.build().unwrap();

        let results = index.search("fn add", 5);
        assert!(!results.is_empty());
        let top = &results[0];
        assert!(top.score > 0.0);
    }

    #[test]
    fn test_index_rebuild_detection() {
        let (_dir, root) = create_temp_workspace();
        let mut index = CodeIndex::new(&root);
        index.build().unwrap();
        assert!(!index.needs_rebuild());
    }

    #[test]
    fn test_empty_index_search() {
        let dir = tempfile::tempdir().unwrap();
        let mut index = CodeIndex::new(dir.path());
        index.build().unwrap();
        let results = index.search("anything", 5);
        assert!(results.is_empty());
    }
}
