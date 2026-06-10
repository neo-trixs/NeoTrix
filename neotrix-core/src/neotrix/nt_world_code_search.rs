use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use walkdir::WalkDir;

const VSA_DIM: usize = 4096;
const CHUNK_LINES: usize = 50;
const CHUNK_OVERLAP: usize = 10;
const CACHE_TTL: Duration = Duration::from_secs(300);
const FILE_SIZE_LIMIT: u64 = 1_048_576;
const FILE_COHERENCE_BOOST: f64 = 0.2;
const DEFINITION_BOOST: f64 = 3.0;
const TEST_PENALTY: f64 = 0.3;
const COMPAT_PENALTY: f64 = 0.3;
const EXAMPLE_PENALTY: f64 = 0.3;
const SATURATION_DECAY: f64 = 0.5;

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
    build_time: Instant,
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
            "rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "rb", "c",
            "cpp", "h", "hpp", "swift", "kt", "scala", "rs", "toml", "json",
            "yaml", "yml", "md", "sql", "sh", "bash", "zsh", "css", "html",
            "vue", "svelte", "zig", "ex", "exs", "clj", "cljs", "lua",
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
            "rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "rb", "c",
            "cpp", "h", "hpp", "swift", "kt", "scala", "toml", "json",
            "yaml", "yml", "md", "sql", "sh", "css", "html",
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

            let mut signals: Vec<&str> = Vec::new();
            if vsa_sim > 0.3 {
                signals.push("vsa");
            }
            if lexical_score > 0.0 {
                signals.push("lexical");
            }
            if vsa_sim > 0.1 || lexical_score > 0.0 {
                let score = hybrid;
                scored.push((score, idx));
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

                if is_symbol || query_tokens.iter().any(|t| {
                    entry
                        .chunk
                        .content
                        .contains(&format!("def {}", t))
                        || entry
                            .chunk
                            .content
                            .contains(&format!("fn {}", t))
                        || entry
                            .chunk
                            .content
                            .contains(&format!("class {}", t))
                        || entry
                            .chunk
                            .content
                            .contains(&format!("struct {}", t))
                        || entry
                            .chunk
                            .content
                            .contains(&format!("pub fn {}", t))
                        || entry
                            .chunk
                            .content
                            .contains(&format!("pub struct {}", t))
                        || entry
                            .chunk
                            .content
                            .contains(&format!("impl {}", t))
                        || entry
                            .chunk
                            .content
                            .contains(&format!("function {}", t))
                        || entry
                            .chunk
                            .content
                            .contains(&format!("func {}", t))
                }) {
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

fn chunk_code(content: &str, ext: &str) -> Vec<(usize, usize, String)> {
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

fn tokenize_code(content: &str) -> Vec<String> {
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

fn encode_vsa(tokens: &[String]) -> Vec<u8> {
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

fn compute_lexical_score(query_tokens: &[String], doc_tokens: &[String]) -> f64 {
    if query_tokens.is_empty() || doc_tokens.is_empty() {
        return 0.0;
    }

    let doc_set: std::collections::HashSet<&str> =
        doc_tokens.iter().map(|s| s.as_str()).collect();

    let mut matches = 0usize;
    for qt in query_tokens {
        if doc_set.contains(qt.as_str()) {
            matches += 1;
        }
    }

    matches as f64 / query_tokens.len() as f64
}

fn detect_symbol_query(tokens: &[String]) -> bool {
    let symbol_patterns = [
        "::", "->", "=>", ".",
    ];
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

fn detect_test_file(path: &str) -> bool {
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

pub struct CodeSearchEngine {
    index: Option<CodeIndex>,
    root_path: PathBuf,
    last_build: Option<Instant>,
}

impl Default for CodeSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeSearchEngine {
    pub fn new() -> Self {
        Self {
            index: None,
            root_path: PathBuf::from("."),
            last_build: None,
        }
    }

    pub fn with_root(root_path: &Path) -> Self {
        Self {
            index: None,
            root_path: root_path.to_path_buf(),
            last_build: None,
        }
    }

    pub fn ensure_index(&mut self) -> Result<(), String> {
        let needs_rebuild = match &self.index {
            Some(idx) => {
                idx.build_time.elapsed() > CACHE_TTL || idx.needs_rebuild()
            }
            None => true,
        };

        if needs_rebuild {
            let mut new_index = CodeIndex::new(&self.root_path);
            new_index.build()?;
            let count = new_index.entry_count();
            log::info!(
                "[CodeSearch] Index rebuilt: {} chunks from {:?}",
                count,
                self.root_path
            );
            self.index = Some(new_index);
            self.last_build = Some(Instant::now());
        }

        Ok(())
    }

    pub fn search(&mut self, query: &str, top_k: usize) -> Result<Vec<CodeSearchResult>, String> {
        self.ensure_index()?;
        match &self.index {
            Some(idx) => Ok(idx.search(query, top_k)),
            None => Err("Index not available".into()),
        }
    }

    pub fn format_results(&mut self, query: &str, top_k: usize) -> Result<String, String> {
        let results = self.search(query, top_k)?;
        if results.is_empty() {
            return Ok(format!("No code search results for: {}", query));
        }

        let mut output = format!("Code search results for \"{}\":\n\n", query);
        for (i, r) in results.iter().enumerate() {
            let path_str = if r.chunk.file_path.len() > 80 {
                format!("...{}", &r.chunk.file_path[r.chunk.file_path.len() - 77..])
            } else {
                r.chunk.file_path.clone()
            };
            output.push_str(&format!(
                "{}. {}:{}–{} (score: {:.3})\n",
                i + 1,
                path_str,
                r.chunk.start_line,
                r.chunk.end_line,
                r.score
            ));

            let preview: String = r
                .chunk
                .content
                .lines()
                .take(8)
                .collect::<Vec<&str>>()
                .join("\n");
            output.push_str(&preview);
            if r.chunk.content.lines().count() > 8 {
                output.push_str("\n...");
            }
            output.push('\n');
            if i < results.len() - 1 {
                output.push('\n');
            }
        }

        Ok(output)
    }
}

pub fn code_search(workspace_path: &str, query: &str, top_k: usize) -> Result<String, String> {
    let mut engine = CodeSearchEngine::with_root(Path::new(workspace_path));
    engine.format_results(query, top_k)
}

// ============================================================================
// Block-level chunk extraction (brace-matching + Python indentation)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    Function,
    Method,
    Class,
    Struct,
    Trait,
    Enum,
    Impl,
    Module,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct BlockChunk {
    pub name: String,
    pub block_type: BlockType,
    pub start_line: usize,
    pub end_line: usize,
    pub source: String,
}

fn strip_trailing_comment(line: &str) -> &str {
    if let Some(pos) = line.find("//") {
        &line[..pos]
    } else {
        line
    }
}

fn count_braces(line: &str) -> (usize, usize) {
    (line.matches('{').count(), line.matches('}').count())
}

fn try_extract_name(line: &str) -> Option<(String, BlockType)> {
    let trimmed = line.trim();

    if trimmed.starts_with("//")
        || trimmed.starts_with('#')
        || trimmed.starts_with("/*")
    {
        return None;
    }

    if trimmed.starts_with('}') {
        return None;
    }

    let mut s = trimmed;

    loop {
        if s.starts_with("pub") {
            if s.starts_with("pub(") {
                if let Some(end) = s.find(')') {
                    s = s[end + 1..].trim_start();
                } else {
                    return None;
                }
            } else {
                s = s[3..].trim_start();
            }
        } else if s.starts_with("async ") {
            s = s[6..].trim_start();
        } else if s.starts_with("unsafe ") {
            s = s[7..].trim_start();
        } else if s.starts_with("extern ") {
            s = s[7..].trim_start();
        } else {
            break;
        }
    }

    if s.starts_with("fn ") || s.starts_with("def ") {
        let rest = &s[s.find(' ').unwrap_or(3) + 1..].trim_start();
        let name = rest
            .split(|c: char| c == '(' || c == '<' || c == ' ' || c == '\t' || c == '{' || c == ';')
            .next()
            .unwrap_or("")
            .to_string();
        if !name.is_empty() {
            return Some((name, BlockType::Function));
        }
    }

    if s.starts_with("struct ") {
        let rest = &s[7..].trim_start();
        let name = rest
            .split(|c: char| c == '{' || c == '<' || c == ' ' || c == '\t' || c == ';')
            .next()
            .unwrap_or("")
            .to_string();
        if !name.is_empty() {
            return Some((name, BlockType::Struct));
        }
    }

    if s.starts_with("enum ") {
        let rest = &s[5..].trim_start();
        let name = rest
            .split(|c: char| c == '{' || c == ' ' || c == '\t' || c == ';')
            .next()
            .unwrap_or("")
            .to_string();
        if !name.is_empty() {
            return Some((name, BlockType::Enum));
        }
    }

    if s.starts_with("trait ") {
        let rest = &s[6..].trim_start();
        let name = rest
            .split(|c: char| c == '{' || c == '<' || c == ':' || c == ' ' || c == '\t' || c == ';')
            .next()
            .unwrap_or("")
            .to_string();
        if !name.is_empty() {
            return Some((name, BlockType::Trait));
        }
    }

    if s.starts_with("impl")
        && (s.len() == 4
            || s[4..].starts_with(' ')
            || s[4..].starts_with('\t')
            || s[4..].starts_with('<'))
    {
        let rest = s[4..].trim_start();
        let name = if rest.is_empty() || rest.starts_with('{') {
            "impl".to_string()
        } else {
            let after_generics = if rest.starts_with('<') {
                let mut depth = 0i32;
                let mut end = 0;
                for (i, c) in rest.char_indices() {
                    if c == '<' {
                        depth += 1;
                    }
                    if c == '>' {
                        depth -= 1;
                        if depth == 0 {
                            end = i + 1;
                            break;
                        }
                    }
                }
                if end > 0 {
                    rest[end..].trim_start()
                } else {
                    rest
                }
            } else {
                rest
            };
            let name_end = after_generics
                .find(|c: char| c == '{' || c == ' ' || c == '\t')
                .unwrap_or(after_generics.len());
            let name_part = &after_generics[..name_end];
            let clean = name_part.split('<').next().unwrap_or(name_part).trim();
            if clean.is_empty() {
                "impl".to_string()
            } else {
                clean.to_string()
            }
        };
        return Some((name, BlockType::Impl));
    }

    if s.starts_with("class ") {
        let rest = &s[6..].trim_start();
        let name = rest
            .split(|c: char| c == '(' || c == ':' || c == ' ' || c == '\t' || c == '{')
            .next()
            .unwrap_or("")
            .to_string();
        if !name.is_empty() {
            return Some((name, BlockType::Class));
        }
    }

    if s.starts_with("module ") {
        let rest = &s[7..].trim_start();
        let name = rest
            .split(|c: char| c == '{' || c == ';' || c == ' ' || c == '\t')
            .next()
            .unwrap_or("")
            .to_string();
        if !name.is_empty() {
            return Some((name, BlockType::Module));
        }
    }

    None
}

pub fn extract_blocks(source: &str) -> Vec<BlockChunk> {
    let lines: Vec<&str> = source.lines().collect();
    let mut blocks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if let Some((name, block_type)) = try_extract_name(lines[i]) {
            let no_comment = strip_trailing_comment(lines[i]).trim().to_string();
            if no_comment.ends_with(';') {
                i += 1;
                continue;
            }

            let is_python_style =
                (block_type == BlockType::Function && lines[i].trim().starts_with("def "))
                    || (block_type == BlockType::Class && lines[i].trim().starts_with("class "));

            if is_python_style {
                if !lines[i].trim().ends_with(':') {
                    i += 1;
                    continue;
                }
                let decl_indent = lines[i].len() - lines[i].trim_start().len();

                let mut j = i + 1;
                while j < lines.len() && lines[j].trim().is_empty() {
                    j += 1;
                }
                if j >= lines.len() {
                    i += 1;
                    continue;
                }
                let body_indent = lines[j].len() - lines[j].trim_start().len();
                if body_indent <= decl_indent {
                    blocks.push(BlockChunk {
                        name,
                        block_type,
                        start_line: i + 1,
                        end_line: j + 1,
                        source: lines[i..=j].join("\n"),
                    });
                    i = j + 1;
                    continue;
                }

                let mut end = j;
                while end + 1 < lines.len() {
                    let next = lines[end + 1];
                    let next_indent = next.len() - next.trim_start().len();
                    if next.trim().is_empty() {
                        end += 1;
                        continue;
                    }
                    if next_indent >= body_indent {
                        end += 1;
                    } else {
                        break;
                    }
                }

                blocks.push(BlockChunk {
                    name,
                    block_type,
                    start_line: i + 1,
                    end_line: end + 1,
                    source: lines[i..=end].join("\n"),
                });
                i = end + 1;
                continue;
            }

            let mut brace_depth = 0i32;
            let mut j = i;
            let mut found_block = false;

            while j < lines.len() {
                let (open, close) = count_braces(lines[j]);
                brace_depth += open as i32;
                brace_depth -= close as i32;

                if !found_block {
                    if open > 0 {
                        found_block = true;
                        if brace_depth <= 0 {
                            blocks.push(BlockChunk {
                                name,
                                block_type,
                                start_line: i + 1,
                                end_line: j + 1,
                                source: lines[i..=j].join("\n"),
                            });
                            i = j + 1;
                            break;
                        }
                    }
                } else if brace_depth <= 0 {
                    blocks.push(BlockChunk {
                        name,
                        block_type,
                        start_line: i + 1,
                        end_line: j + 1,
                        source: lines[i..=j].join("\n"),
                    });
                    i = j + 1;
                    break;
                }

                j += 1;
            }

            if !found_block {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    blocks
}

// ============================================================================
// Multi-resolution index (file-level + function-level)
// ============================================================================

#[derive(Debug, Clone)]
pub struct FileChunkEntry {
    pub file_path: PathBuf,
    pub vsa: Vec<u8>,
    pub tokens: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CodeChunkIndex {
    pub chunks: Vec<BlockChunk>,
    pub file_path: PathBuf,
    pub vsa: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ScoredFile {
    pub file_path: PathBuf,
    pub score: f64,
    pub matching_chunks: Vec<BlockChunk>,
}

fn encode_block_vsa(blocks: &[BlockChunk]) -> Vec<u8> {
    let tokens: Vec<String> = blocks.iter().map(|b| b.name.clone()).collect();
    if tokens.is_empty() {
        return vec![0u8; VSA_DIM];
    }
    encode_vsa(&tokens)
}

pub struct MultiResolutionIndex {
    pub file_chunks: Vec<FileChunkEntry>,
    pub function_chunks: Vec<CodeChunkIndex>,
    root_path: PathBuf,
}

impl MultiResolutionIndex {
    pub fn new(root_path: &Path) -> Self {
        Self {
            file_chunks: Vec::new(),
            function_chunks: Vec::new(),
            root_path: root_path.to_path_buf(),
        }
    }

    pub fn index_file(&mut self, path: &Path, source: &str) -> Result<(), String> {
        let rel = path.strip_prefix(&self.root_path).unwrap_or(path);

        let tokens = tokenize_code(source);
        let vsa = encode_vsa(&tokens);
        self.file_chunks.push(FileChunkEntry {
            file_path: rel.to_path_buf(),
            vsa,
            tokens,
        });

        let blocks = extract_blocks(source);
        if !blocks.is_empty() {
            let block_vsa = encode_block_vsa(&blocks);
            self.function_chunks.push(CodeChunkIndex {
                chunks: blocks,
                file_path: rel.to_path_buf(),
                vsa: block_vsa,
            });
        }

        Ok(())
    }

    pub fn search(&self, query: &str, top_k: usize) -> Vec<ScoredFile> {
        if self.file_chunks.is_empty() || query.trim().is_empty() {
            return Vec::new();
        }

        let query_tokens = tokenize_code(query);
        if query_tokens.is_empty() {
            return Vec::new();
        }

        let query_vector = encode_vsa(&query_tokens);
        let is_symbol_query = detect_symbol_query(&query_tokens);
        let top_k = top_k.max(1);

        let file_scores: Vec<(f64, usize)> = self
            .file_chunks
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let vsa_sim = QuantizedVSA::similarity(&query_vector, &entry.vsa);
                let lexical = compute_lexical_score(&query_tokens, &entry.tokens);
                let alpha = if is_symbol_query { 0.3 } else { 0.5 };
                let score = alpha * vsa_sim + (1.0 - alpha) * lexical;
                (score, idx)
            })
            .filter(|(s, _)| *s > 0.005)
            .collect();

        let mut func_hits: HashMap<PathBuf, Vec<BlockChunk>> = HashMap::new();
        for func_idx in &self.function_chunks {
            let mut matching: Vec<BlockChunk> = Vec::new();
            for block in &func_idx.chunks {
                let block_tokens = tokenize_code(&block.source);
                let block_vsa = encode_vsa(&block_tokens);
                let sim = QuantizedVSA::similarity(&query_vector, &block_vsa);
                let lex = compute_lexical_score(&query_tokens, &block_tokens);
                if sim > 0.15 || lex > 0.0 {
                    matching.push(block.clone());
                }
            }
            if !matching.is_empty() {
                func_hits
                    .entry(func_idx.file_path.clone())
                    .or_default()
                    .extend(matching);
            }
        }

        let mut merged: Vec<ScoredFile> = file_scores
            .into_iter()
            .filter_map(|(score, idx)| {
                let entry = &self.file_chunks[idx];
                let bonus = if func_hits.contains_key(&entry.file_path) {
                    0.2
                } else {
                    0.0
                };
                let chunks = func_hits.remove(&entry.file_path).unwrap_or_default();
                Some(ScoredFile {
                    file_path: entry.file_path.clone(),
                    score: score + bonus,
                    matching_chunks: chunks,
                })
            })
            .collect();

        for (path, chunks) in func_hits.drain() {
            merged.push(ScoredFile {
                file_path: path,
                score: 0.5,
                matching_chunks: chunks,
            });
        }

        merged.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        merged.truncate(top_k);
        merged
    }
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
        assert!((sim - 1.0).abs() < 1e-10, "deterministic encoding should be identical");
    }

    #[test]
    fn test_lexical_score_matches_identical() {
        let query = vec!["auth".to_string(), "config".to_string()];
        let doc = vec!["auth".to_string(), "config".to_string(), "parse".to_string()];
        let score = compute_lexical_score(&query, &doc);
        assert!((score - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_lexical_score_partial_match() {
        let query = vec!["auth".to_string(), "config".to_string(), "parse".to_string()];
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
    fn test_code_search_engine_format() {
        let (_dir, root) = create_temp_workspace();
        let mut engine = CodeSearchEngine::with_root(&root);
        let result = engine.format_results("parse config", 5);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Code search results"));
        assert!(output.contains("utils.py"));
    }

    #[test]
    fn test_code_search_engine_empty_query() {
        let (_dir, root) = create_temp_workspace();
        let mut engine = CodeSearchEngine::with_root(&root);
        let result = engine.format_results("", 5);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("No code search results"));
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

    #[test]
    fn test_extract_blocks_rust_fn() {
        let source = "fn hello() {\n    println!(\"world\");\n}\n\nfn add(a: i32, b: i32) -> i32 {\n    a + b\n}";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].name, "hello");
        assert_eq!(blocks[0].block_type, BlockType::Function);
        assert_eq!(blocks[0].start_line, 1);
        assert_eq!(blocks[0].end_line, 3);
        assert_eq!(blocks[1].name, "add");
        assert_eq!(blocks[1].start_line, 5);
        assert_eq!(blocks[1].end_line, 7);
    }

    #[test]
    fn test_extract_blocks_rust_struct() {
        let source = "pub struct Config {\n    pub name: String,\n    pub version: u64,\n}";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "Config");
        assert_eq!(blocks[0].block_type, BlockType::Struct);
    }

    #[test]
    fn test_extract_blocks_rust_impl() {
        let source = "impl Config {\n    pub fn new() -> Self {\n        Self { name: \"\".into(), version: 0 }\n    }\n}";
        let blocks = extract_blocks(source);
        assert!(blocks.len() >= 1);
        let impl_block = blocks.iter().find(|b| b.block_type == BlockType::Impl).unwrap();
        assert_eq!(impl_block.name, "Config");
    }

    #[test]
    fn test_extract_blocks_rust_enum() {
        let source = "enum Status {\n    Active,\n    Inactive,\n    Pending,\n}";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "Status");
        assert_eq!(blocks[0].block_type, BlockType::Enum);
    }

    #[test]
    fn test_extract_blocks_python_fn() {
        let source = "def parse_config(path: str) -> dict:\n    import json\n    with open(path) as f:\n        return json.load(f)\n\ndef validate(data: dict) -> bool:\n    return 'name' in data";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].name, "parse_config");
        assert_eq!(blocks[0].block_type, BlockType::Function);
        assert!(blocks[0].source.contains("import json"));
        assert_eq!(blocks[1].name, "validate");
        assert!(blocks[1].source.contains("'name' in data"));
    }

    #[test]
    fn test_extract_blocks_python_class() {
        let source = "class User:\n    def __init__(self, name: str):\n        self.name = name\n\n    def greet(self) -> str:\n        return f\"Hello, {self.name}\"";
        let blocks = extract_blocks(source);
        let class_block = blocks.iter().find(|b| b.block_type == BlockType::Class).unwrap();
        assert_eq!(class_block.name, "User");
        assert!(class_block.source.contains("def __init__"));
    }

    #[test]
    fn test_extract_blocks_forward_decl() {
        let source = "fn foo();\n\nfn bar() {\n    foo()\n}";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "bar");
    }

    #[test]
    fn test_extract_blocks_empty_body() {
        let source = "fn foo() {}\nfn bar() {\n    // just a comment\n}";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].name, "foo");
        assert_eq!(blocks[0].start_line, 1);
        assert_eq!(blocks[1].name, "bar");
    }

    #[test]
    fn test_extract_blocks_nested_braces() {
        let source = "fn outer() {\n    if true {\n        loop {\n            break;\n        }\n    }\n    let x = 1;\n}";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "outer");
        assert_eq!(blocks[0].start_line, 1);
        assert_eq!(blocks[0].end_line, 8);
    }

    #[test]
    fn test_extract_blocks_pub_unsafe_fn() {
        let source = "pub unsafe fn dangerous() {\n    // risky business\n}";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "dangerous");
    }

    #[test]
    fn test_multi_resolution_index_search() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let mut mri = MultiResolutionIndex::new(&root);

        let file1 = root.join("math.rs");
        std::fs::write(&file1, "fn add(a: i32, b: i32) -> i32 { a + b }\nfn multiply(a: i32, b: i32) -> i32 { a * b }").unwrap();

        let file2 = root.join("strings.rs");
        std::fs::write(&file2, "fn to_upper(s: &str) -> String { s.to_uppercase() }\nfn to_lower(s: &str) -> String { s.to_lowercase() }").unwrap();

        mri.index_file(&file1, &std::fs::read_to_string(&file1).unwrap()).unwrap();
        mri.index_file(&file2, &std::fs::read_to_string(&file2).unwrap()).unwrap();

        assert_eq!(mri.file_chunks.len(), 2);
        assert_eq!(mri.function_chunks.len(), 2);

        let results = mri.search("multiply numbers", 5);
        assert!(!results.is_empty());
        let top = &results[0];
        assert!(top.score > 0.0);
    }

    #[test]
    fn test_multi_resolution_function_bonus() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let mut mri = MultiResolutionIndex::new(&root);

        let file1 = root.join("auth.rs");
        std::fs::write(&file1, "fn parse_token(s: &str) -> bool { true }\nfn validate_token(t: &str) -> bool { true }").unwrap();

        let file2 = root.join("utils.rs");
        std::fs::write(&file2, "mod helpers {\n    pub fn do_thing() {}\n}\nfn utility() {}").unwrap();

        mri.index_file(&file1, &std::fs::read_to_string(&file1).unwrap()).unwrap();
        mri.index_file(&file2, &std::fs::read_to_string(&file2).unwrap()).unwrap();

        let results = mri.search("parse_token validate", 5);
        assert!(!results.is_empty());
        let top_file = results.iter().find(|r| r.file_path.ends_with("auth.rs")).unwrap();
        assert!(top_file.score > 0.3, "auth.rs should have good score for token query");
    }

    #[test]
    fn test_multi_resolution_empty_search() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let mri = MultiResolutionIndex::new(&root);
        let results = mri.search("anything", 5);
        assert!(results.is_empty());
    }
}
