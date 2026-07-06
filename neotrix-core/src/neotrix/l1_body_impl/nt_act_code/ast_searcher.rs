//! AstCodeSearcher — AST-based structural code search (cocoindex-code inspired)
//!
//! Provides structural code search with metavariable patterns (`\NAME`, `\(ARGS*)`)
//! across 12+ programming languages. Can index directories for fast repeated
//! searches, or scan files on-the-fly.
//!
//! # Pattern syntax
//!
//! - `\NAME` — matches any single identifier token
//! - `\(ARGS*)` — matches any balanced parenthesized expression
//! - All other tokens match literally (after whitespace normalization)

use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ─── Public Types ───

/// A structural code search query.
#[derive(Debug, Clone)]
pub struct AstQuery {
    /// Structural pattern using `\NAME` and `\(ARGS*)` metavariables.
    pub pattern: String,
    /// File path glob filters (e.g. `src/**/*.rs`).
    pub path_globs: Vec<String>,
    /// Optional language whitelist.
    pub languages: Option<Vec<String>>,
    /// Maximum results to return (default 10).
    pub max_results: usize,
}

impl Default for AstQuery {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            path_globs: Vec::new(),
            languages: None,
            max_results: 10,
        }
    }
}

/// A single code match result.
#[derive(Debug, Clone)]
pub struct CodeMatch {
    pub file_path: String,
    pub language: String,
    pub matched_text: String,
    pub line_start: usize,
    pub line_end: usize,
    pub similarity_score: f64,
}

// ─── Internal Types ───

/// A chunk of indexed code (function/class boundary).
#[derive(Debug, Clone)]
struct IndexedChunk {
    file_path: String,
    language: String,
    start_line: usize,
    end_line: usize,
    content: String,
}

// ─── Main Searcher ───

/// Structural code searcher inspired by cocoindex-code.
///
/// Supports on-the-fly file scanning and directory indexing with
/// function/class boundary chunking.
#[derive(Debug)]
pub struct AstCodeSearcher {
    pub supported_languages: HashMap<String, Vec<String>>,
    index: HashMap<String, Vec<IndexedChunk>>,
    is_indexed: bool,
}

impl AstCodeSearcher {
    /// Create a new searcher with 12+ pre-configured language support.
    pub fn new() -> Self {
        let mut supported_languages: HashMap<String, Vec<String>> = HashMap::new();

        supported_languages.insert("rust".into(), vec!["rs".into()]);
        supported_languages.insert("python".into(), vec!["py".into()]);
        supported_languages.insert("javascript".into(), vec!["js".into(), "mjs".into(), "cjs".into()]);
        supported_languages.insert("typescript".into(), vec!["ts".into(), "tsx".into()]);
        supported_languages.insert("go".into(), vec!["go".into()]);
        supported_languages.insert("java".into(), vec!["java".into()]);
        supported_languages.insert("cpp".into(), vec!["cpp".into(), "cc".into(), "cxx".into(), "hpp".into(), "hxx".into()]);
        supported_languages.insert("c".into(), vec!["c".into(), "h".into()]);
        supported_languages.insert("csharp".into(), vec!["cs".into()]);
        supported_languages.insert("ruby".into(), vec!["rb".into()]);
        supported_languages.insert("swift".into(), vec!["swift".into()]);
        supported_languages.insert("kotlin".into(), vec!["kt".into(), "kts".into()]);

        Self {
            supported_languages,
            index: HashMap::new(),
            is_indexed: false,
        }
    }

    /// Walk `dir` and index all supported files.
    ///
    /// Returns the number of files indexed, or an error description.
    pub fn index_directory(&mut self, dir: &Path, recursive: bool) -> Result<usize, String> {
        let mut files: Vec<std::path::PathBuf> = Vec::new();
        Self::collect_files(dir, recursive, &mut files)?;

        let exts: Vec<String> = self.supported_extensions().iter()
            .map(|e| e.to_string())
            .collect();
        let ext_set: std::collections::HashSet<String> = exts.into_iter().collect();

        let mut indexed = 0usize;
        for path in &files {
            let ext = path.extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase())
                .unwrap_or_default();
            if !ext_set.contains(&ext) {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let lang = Self::language_for_ext(&ext, &self.supported_languages)
                .unwrap_or_else(|| "unknown".into());
            let path_str = path.to_string_lossy().to_string();
            let chunks = Self::chunk_code(&path_str, &lang, &content);

            self.index.insert(path_str, chunks);
            indexed += 1;
        }

        if !self.index.is_empty() {
            self.is_indexed = true;
        }
        Ok(indexed)
    }

    /// Search indexed code for structural matches.
    pub fn search(&self, query: &AstQuery) -> Vec<CodeMatch> {
        if !self.is_indexed || self.index.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        let max = query.max_results.max(1);

        for (file_path, chunks) in &self.index {
            if results.len() >= max {
                break;
            }

            // Language filter
            if let Some(ref langs) = query.languages {
                let chunk_lang = self.language_for_file_path(file_path);
                if let Some(ref cl) = chunk_lang {
                    if !langs.iter().any(|l| l == cl) {
                        continue;
                    }
                }
            }

            // Path glob filter
            if !query.path_globs.is_empty() {
                let matched = query.path_globs.iter().any(|g| path_matches_glob(file_path, g));
                if !matched {
                    continue;
                }
            }

            for chunk in chunks {
                if results.len() >= max {
                    break;
                }

                if Self::structural_match(&chunk.content, &query.pattern) {
                    results.push(CodeMatch {
                        file_path: chunk.file_path.clone(),
                        language: chunk.language.clone(),
                        matched_text: chunk.content.clone(),
                        line_start: chunk.start_line,
                        line_end: chunk.end_line,
                        similarity_score: 1.0,
                    });
                }
            }
        }

        results
    }

    /// Structural pattern match: returns true if `pattern` appears as a
    /// token subsequence of `code`, with metavariable expansion.
    ///
    ///   - `\NAME` matches any single identifier token in code.
    ///   - `\(ARGS*)` matches any balanced parenthesised group in code.
    ///     Whitespace differences are ignored.
    pub fn structural_match(code: &str, pattern: &str) -> bool {
        let code_tokens = Self::tokenize(code);
        let pattern_tokens = Self::tokenize(pattern);
        Self::tokens_match_subsequence(&code_tokens, &pattern_tokens)
    }

    /// Search files on-the-fly without prior indexing.
    ///
    /// Walks `dir` and checks all supported files for structural matches.
    pub fn search_files(&self, query: &AstQuery, dir: &Path) -> Vec<CodeMatch> {
        let exts: Vec<String> = self.supported_extensions().iter()
            .map(|e| e.to_string())
            .collect();
        let ext_set: std::collections::HashSet<String> = exts.into_iter().collect();

        let mut files: Vec<std::path::PathBuf> = Vec::new();
        match Self::collect_files(dir, true, &mut files) {
            Ok(_) => {}
            Err(_) => return Vec::new(),
        }

        let mut results = Vec::new();
        let max = query.max_results.max(1);

        for path in &files {
            if results.len() >= max {
                break;
            }

            let ext = path.extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase())
                .unwrap_or_default();
            if !ext_set.contains(&ext) {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let lang = Self::language_for_ext(&ext, &self.supported_languages)
                .unwrap_or_else(|| "unknown".into());
            let path_str = path.to_string_lossy().to_string();

            // Filter by path globs
            if !query.path_globs.is_empty() {
                let matched = query.path_globs.iter().any(|g| path_matches_glob(&path_str, g));
                if !matched {
                    continue;
                }
            }

            let chunks = Self::chunk_code(&path_str, &lang, &content);
            for chunk in &chunks {
                if results.len() >= max {
                    break;
                }
                if Self::structural_match(&chunk.content, &query.pattern) {
                    results.push(CodeMatch {
                        file_path: chunk.file_path.clone(),
                        language: chunk.language.clone(),
                        matched_text: chunk.content.clone(),
                        line_start: chunk.start_line,
                        line_end: chunk.end_line,
                        similarity_score: 1.0,
                    });
                }
            }
        }

        results
    }

    /// All supported file extensions (without leading dot).
    pub fn supported_extensions(&self) -> Vec<&str> {
        let mut exts: Vec<&str> = Vec::new();
        for lang_exts in self.supported_languages.values() {
            for ext in lang_exts {
                if !exts.contains(&ext.as_str()) {
                    exts.push(ext);
                }
            }
        }
        exts.sort();
        exts
    }

    /// Returns `(num_files, num_chunks)` for the current index.
    pub fn index_stats(&self) -> (usize, usize) {
        let num_files = self.index.len();
        let num_chunks: usize = self.index.values().map(|v| v.len()).sum();
        (num_files, num_chunks)
    }

    // ─── Internal Helpers ───

    /// Tokenize a string into tokens (words + symbols), skipping whitespace.
    /// `\NAME` and `\(ARGS*)` are emitted as single tokens.
    fn tokenize(s: &str) -> Vec<String> {
        let chars: Vec<char> = s.chars().collect();
        let mut tokens = Vec::new();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            // Skip whitespace
            if ch.is_whitespace() {
                i += 1;
                continue;
            }

            // Metavar: \NAME or \(ARGS*)
            if ch == '\\' {
                let mut metavar = String::new();
                metavar.push('\\');
                i += 1;

                if i < chars.len() && chars[i] == '(' {
                    // \(ARGS*) — read until matching )
                    metavar.push('(');
                    i += 1;
                    let mut depth = 1usize;
                    while i < chars.len() && depth > 0 {
                        if chars[i] == '(' {
                            depth += 1;
                        } else if chars[i] == ')' {
                            depth -= 1;
                        }
                    if depth > 0 || (depth == 0 && chars[i] == ')') {
                        metavar.push(chars[i]);
                    }
                    i += 1;
                }
                } else {
                    // \NAME — read alphanumeric + underscore
                    while i < chars.len()
                        && (chars[i].is_alphanumeric() || chars[i] == '_')
                    {
                        metavar.push(chars[i]);
                        i += 1;
                    }
                }

                tokens.push(metavar);
                continue;
            }

            // Identifier or keyword
            if ch.is_alphanumeric() || ch == '_' {
                let mut word = String::new();
                word.push(ch);
                i += 1;
                while i < chars.len()
                    && (chars[i].is_alphanumeric() || chars[i] == '_')
                {
                    word.push(chars[i]);
                    i += 1;
                }
                tokens.push(word);
                continue;
            }

            // Punctuation / symbol — single character
            tokens.push(ch.to_string());
            i += 1;
        }

        tokens
    }

    /// Check if `pattern_tokens` appear as a subsequence of `code_tokens`,
    /// with metavar expansion.
    fn tokens_match_subsequence(code_tokens: &[String], pattern_tokens: &[String]) -> bool {
        let mut ci = 0usize;
        let code_len = code_tokens.len();

        for pt in pattern_tokens {
            if pt == "\\NAME" {
                // Match any single identifier
                while ci < code_len && !is_identifier_token(&code_tokens[ci]) {
                    ci += 1;
                }
                if ci >= code_len {
                    return false;
                }
                ci += 1; // consume identifier
            } else if pt == "\\(ARGS*)" {
                // Match any balanced parenthesised expression
                while ci < code_len && code_tokens[ci] != "(" {
                    ci += 1;
                }
                if ci >= code_len {
                    return false;
                }
                ci += 1; // consume '('
                let mut depth = 1isize;
                while ci < code_len && depth > 0 {
                    if code_tokens[ci] == "(" {
                        depth += 1;
                    } else if code_tokens[ci] == ")" {
                        depth -= 1;
                    }
                    ci += 1;
                }
                if depth != 0 {
                    return false; // unbalanced parens in code
                }
            } else {
                // Literal token match
                while ci < code_len && code_tokens[ci] != *pt {
                    ci += 1;
                }
                if ci >= code_len {
                    return false;
                }
                ci += 1; // consume matched token
            }
        }

        true
    }

    /// Collect all files under `dir` (recursively if `recursive`).
    fn collect_files(dir: &Path, recursive: bool, files: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
        let entries = fs::read_dir(dir).map_err(|e| format!("Cannot read dir {:?}: {}", dir, e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Entry error: {}", e))?;
            let path = entry.path();
            if path.is_dir() {
                if recursive {
                    Self::collect_files(&path, recursive, files)?;
                }
            } else if path.is_file() {
                files.push(path);
            }
        }
        Ok(())
    }

    /// Determine the language for a file path based on its extension.
    fn language_for_file_path(&self, file_path: &str) -> Option<String> {
        let ext = std::path::Path::new(file_path)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        Self::language_for_ext(&ext, &self.supported_languages)
    }

    /// Determine the language name for a file extension.
    fn language_for_ext(ext: &str, lang_map: &HashMap<String, Vec<String>>) -> Option<String> {
        for (lang, exts) in lang_map {
            if exts.iter().any(|e| e == ext) {
                return Some(lang.clone());
            }
        }
        None
    }

    /// Chunk code by function/class definition boundaries using brace-depth
    /// tracking for brace-based languages and simple line grouping for others.
    fn chunk_code(file_path: &str, language: &str, content: &str) -> Vec<IndexedChunk> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();

        if lines.is_empty() {
            return chunks;
        }

        // Indentation-based language detection
        let indent_based = matches!(language, "python" | "ruby");

        let mut chunk_start: Option<usize> = None;
        let mut brace_depth: isize = 0;
        let mut def_indent: Option<usize> = None;

        // Closure: finalize a chunk if one is open
        let finalize = |start: usize, end: usize, chunks: &mut Vec<IndexedChunk>| {
            if end <= start {
                return;
            }
            let slice: Vec<&str> = lines[start..end].to_vec();
            chunks.push(IndexedChunk {
                file_path: file_path.to_string(),
                language: language.to_string(),
                start_line: start + 1,
                end_line: end,
                content: slice.join("\n"),
            });
        };

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let is_def = !trimmed.is_empty() && is_def_keyword(trimmed);
            let curr_indent = line.len() - trimmed.len();

            // Capture brace depth before counting current line's braces
            // so should_split sees the depth from the previous line.
            let pre_brace_depth = brace_depth;
            if !indent_based {
                for ch in line.chars() {
                    match ch {
                        '{' => brace_depth += 1,
                        '}' => brace_depth -= 1,
                        _ => {}
                    }
                }
            }

            if is_def {
                // If we were in a chunk and the definition is at a "top level"
                // (depth 0, or indentation back to base), finalize previous chunk.
                let should_split = if indent_based {
                    match def_indent {
                        Some(base) => curr_indent <= base,
                        None => true,
                    }
                } else {
                    pre_brace_depth <= 0
                };

                if let Some(start) = chunk_start.take() {
                    if should_split {
                        finalize(start, i, &mut chunks);
                    } else {
                        // keep extending — nested def
                        chunk_start = Some(start);
                    }
                }

                if chunk_start.is_none() {
                    chunk_start = Some(i);
                    def_indent = Some(curr_indent);
                    if !indent_based {
                        // Reset brace depth for the new chunk
                        brace_depth = 0;
                        for ch in line.chars() {
                            match ch {
                                '{' => brace_depth += 1,
                                '}' => brace_depth -= 1,
                                _ => {}
                            }
                        }
                    }
                }
            } else if chunk_start.is_some() && indent_based {
                // For indentation-based languages, check if we've returned
                // to or above the definition's indentation level (empty lines exempt).
                if !trimmed.is_empty() {
                    if let Some(base) = def_indent {
                        if curr_indent <= base {
                            // End chunk
                            if let Some(start) = chunk_start.take() {
                                finalize(start, i, &mut chunks);
                            }
                            def_indent = None;
                        }
                    }
                }
            }
        }

        // Finalize last chunk
        if let Some(start) = chunk_start {
            finalize(start, lines.len(), &mut chunks);
        }

        // If no chunks were found (no function/class defs), create one for the whole file.
        if chunks.is_empty() {
            chunks.push(IndexedChunk {
                file_path: file_path.to_string(),
                language: language.to_string(),
                start_line: 1,
                end_line: lines.len(),
                content: content.to_string(),
            });
        }

        chunks
    }
}

impl Default for AstCodeSearcher {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Free Helper Functions ───

/// Check if a line appears to start a function or class definition.
fn is_def_keyword(line: &str) -> bool {
    let line = line.trim();
    // Skip comments
    if line.starts_with("//") || line.starts_with('#') || line.starts_with("/*") {
        return false;
    }

    // All definition-starting keywords across supported languages
    line.starts_with("fn ") || line.starts_with("pub fn ")
        || line.starts_with("pub(crate) fn ") || line.starts_with("unsafe fn ")
        || line.starts_with("pub unsafe fn ")
        || line.starts_with("def ") || line.starts_with("async def ")
        || line.starts_with("class ")
        || line.starts_with("struct ")
        || line.starts_with("enum ")
        || line.starts_with("trait ") || line.starts_with("pub trait ")
        || line.starts_with("impl") || line.starts_with("pub impl")
        || line.starts_with("interface ")
        || line.starts_with("func ") || line.starts_with("function ")
        || line.starts_with("async function ")
        || line.starts_with("fun ") || line.starts_with("data class ")
        || line.starts_with("protocol ") || line.starts_with("extension ")
        || line.starts_with("module ") || line.starts_with("type ")
        || line.starts_with("macro_rules!")
        || line.starts_with("object ")
        || line.starts_with("export function") || line.starts_with("export class")
        || line.starts_with("export interface") || line.starts_with("export type")
}

/// Check if a token is an identifier (alphanumeric + underscore, starts with letter or `_`).
fn is_identifier_token(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    let first = token.chars().next().unwrap_or(' ');
    if !first.is_alphabetic() && first != '_' {
        return false;
    }
    token.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Simple path glob matching (suffix/prefix/contains patterns without external deps).
fn path_matches_glob(path: &str, glob: &str) -> bool {
    if glob.starts_with("**/") {
        let rest = &glob[3..];
        if rest.contains("**") {
            // `**/dir/**` — check it contains `/dir/`
            let inner = rest.trim_start_matches('*').trim_end_matches('*')
                        .trim_matches('/');
            if !inner.is_empty() {
                let needle = format!("/{}/", inner);
                return path.contains(&needle) || path.starts_with(needle.trim_start_matches('/'));
            }
        }
        if let Some(star_pos) = rest.find('*') {
            // `**/prefix*suffix`
            let prefix = &rest[..star_pos];
            let suffix = &rest[star_pos + 1..];
            if prefix.is_empty() {
                return path.ends_with(suffix);
            }
            if suffix.is_empty() {
                return path.ends_with(prefix);
            }
            return path.ends_with(suffix) && path.contains(prefix);
        }
        // `**/specific/path.rs` — suffix check
        return path.ends_with(rest);
    }
    if glob.starts_with("*.") {
        // `*.rs` — match any file with this extension
        let suffix = &glob[1..]; // `.rs`
        return path.ends_with(suffix);
    }
    if let Some(star_pos) = glob.find('*') {
        let prefix = &glob[..star_pos];
        let suffix = &glob[star_pos + 1..];
        if prefix.is_empty() {
            return path.ends_with(suffix);
        }
        if suffix.is_empty() {
            return path.starts_with(prefix);
        }
        // `src/*.rs` — starts with prefix AND ends with suffix
        return path.starts_with(prefix) && path.ends_with(suffix);
    }
    path == glob
}

// ─── Tests ───

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("neotrix_ast_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_new_searcher_has_supported_languages() {
        let s = AstCodeSearcher::new();
        assert!(!s.supported_languages.is_empty(), "should have languages");
        assert!(s.supported_languages.contains_key("rust"));
        assert!(s.supported_languages.contains_key("python"));
        assert!(s.supported_languages.contains_key("javascript"));
        assert!(s.supported_languages.contains_key("go"));
        assert!(s.supported_languages.contains_key("java"));
        assert!(s.supported_languages.contains_key("csharp"));
        assert!(s.supported_languages.contains_key("swift"));
        assert!(s.supported_languages.contains_key("kotlin"));
        assert!(s.supported_languages.contains_key("ruby"));
        assert!(s.supported_languages.contains_key("cpp"));
        assert!(s.supported_languages.contains_key("c"));
        assert!(s.supported_languages.contains_key("typescript"));
    }

    #[test]
    fn test_structural_match_simple() {
        assert!(AstCodeSearcher::structural_match("fn foo() {}", "fn foo() {}"));
        assert!(AstCodeSearcher::structural_match("fn  foo ( )  { }", "fn foo() {}"));
        assert!(AstCodeSearcher::structural_match("let x = 42;", "let x = 42;"));
    }

    #[test]
    fn test_structural_match_with_name_metavar() {
        assert!(AstCodeSearcher::structural_match("fn foo() {}", "fn \\NAME() {}"));
        assert!(AstCodeSearcher::structural_match("let bar = 1;", "let \\NAME = 1;"));
        assert!(AstCodeSearcher::structural_match("let bar_baz = 1;", "let \\NAME = 1;"));
    }

    #[test]
    fn test_structural_match_with_args_metavar() {
        assert!(AstCodeSearcher::structural_match(
            "fn foo(x: i32, y: i32) -> bool",
            "fn \\NAME\\(ARGS*)"
        ));
        assert!(AstCodeSearcher::structural_match(
            "call(a, b + c, d)",
            "call\\(ARGS*)"
        ));
        assert!(AstCodeSearcher::structural_match(
            "nested(outer(inner))",
            "\\NAME\\(ARGS*)"
        ));
    }

    #[test]
    fn test_structural_match_no_match() {
        assert!(!AstCodeSearcher::structural_match("fn foo() {}", "fn bar() {}"));
        assert!(!AstCodeSearcher::structural_match("let x = 1;", "fn \\NAME() {}"));
        assert!(!AstCodeSearcher::structural_match("class Foo {}", "struct Foo {}"));
    }

    #[test]
    fn test_structural_match_case_sensitive() {
        // Must be case-sensitive
        assert!(!AstCodeSearcher::structural_match("fn FOO() {}", "fn foo() {}"));
        assert!(AstCodeSearcher::structural_match("fn FOO() {}", "fn FOO() {}"));
    }

    #[test]
    fn test_supported_extensions_contains_rust() {
        let s = AstCodeSearcher::new();
        let exts = s.supported_extensions();
        assert!(exts.contains(&"rs"), "should contain .rs extension");
        assert!(exts.contains(&"py"));
        assert!(exts.contains(&"js"));
        assert!(exts.contains(&"go"));
        assert!(exts.contains(&"java"));
        assert!(exts.contains(&"cpp"));
        assert!(exts.contains(&"cs"));
        assert!(exts.contains(&"swift"));
        assert!(exts.contains(&"kt"));
    }

    #[test]
    fn test_search_empty_index_returns_empty() {
        let s = AstCodeSearcher::new();
        let query = AstQuery {
            pattern: "fn foo".into(),
            ..Default::default()
        };
        let results = s.search(&query);
        assert!(results.is_empty(), "empty index should return empty results");
    }

    #[test]
    fn test_index_empty_directory() {
        let dir = tmp_dir("empty");
        let mut s = AstCodeSearcher::new();
        let count = s.index_directory(&dir, true).unwrap();
        assert_eq!(count, 0, "empty dir should index 0 files");
        let (f, c) = s.index_stats();
        assert_eq!(f, 0);
        assert_eq!(c, 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_chunking_detects_function_boundaries() {
        let dir = tmp_dir("chunking");
        let code = [
            "fn foo() {",
            "    let x = 1;",
            "}",
            "",
            "fn bar() {",
            "    let y = 2;",
            "}",
        ].join("\n");
        fs::write(dir.join("test.rs"), &code).unwrap();

        let mut s = AstCodeSearcher::new();
        let count = s.index_directory(&dir, true).unwrap();
        assert_eq!(count, 1, "should index 1 file");

        let (num_files, num_chunks) = s.index_stats();
        assert_eq!(num_files, 1);
        assert!(num_chunks >= 2, "should have at least 2 chunks for 2 functions, got {}", num_chunks);

        let query = AstQuery {
            pattern: "fn bar() {}".into(),
            ..Default::default()
        };
        let results = s.search(&query);
        assert_eq!(results.len(), 1, "should find fn bar in one chunk");
        if !results.is_empty() {
            assert!(results[0].matched_text.contains("fn bar()"));
        }

        let query2 = AstQuery {
            pattern: "fn \\NAME() {}".into(),
            ..Default::default()
        };
        let results2 = s.search(&query2);
        assert_eq!(results2.len(), 2, "metavar should match both functions");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_search_files_finds_matches() {
        let dir = tmp_dir("search_files");
        fs::write(dir.join("main.rs"), "fn compute(x: i32) -> i32 { x + 1 }").unwrap();
        fs::write(dir.join("lib.py"), "def compute(x):\n    return x + 1").unwrap();
        fs::write(dir.join("ignore.txt"), "this is not code").unwrap();

        let s = AstCodeSearcher::new();

        let query = AstQuery {
            pattern: "fn \\NAME\\(ARGS*)".into(),
            ..Default::default()
        };
        let results = s.search_files(&query, &dir);
        assert!(!results.is_empty(), "should find structural matches");
        assert!(results.iter().any(|r| r.file_path.ends_with("main.rs")),
            "should match Rust file");

        let query_py = AstQuery {
            pattern: "def \\NAME\\(ARGS*)".into(),
            ..Default::default()
        };
        let results_py = s.search_files(&query_py, &dir);
        assert!(results_py.iter().any(|r| r.file_path.ends_with("lib.py")),
            "should match Python file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_index_and_search_rust_code() {
        let dir = tmp_dir("rust_search");
        fs::write(dir.join("math.rs"), [
            "fn add(a: i32, b: i32) -> i32 {",
            "    a + b",
            "}",
            "",
            "fn sub(a: i32, b: i32) -> i32 {",
            "    a - b",
            "}",
        ].join("\n")).unwrap();

        let mut s = AstCodeSearcher::new();
        s.index_directory(&dir, true).unwrap();

        let query = AstQuery {
            pattern: "fn add\\(ARGS*) -> i32".into(),
            max_results: 5,
            ..Default::default()
        };
        let results = s.search(&query);
        assert!(!results.is_empty(), "should find add function");
        assert!(results[0].matched_text.contains("fn add"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_path_globs_filter() {
        let dir = tmp_dir("globs");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("tests")).unwrap();
        fs::write(dir.join("src/lib.rs"), "fn helper() {}").unwrap();
        fs::write(dir.join("tests/test.rs"), "fn test_me() {}").unwrap();

        let mut s = AstCodeSearcher::new();
        s.index_directory(&dir, true).unwrap();

        let query = AstQuery {
            pattern: "fn \\NAME() {}".into(),
            path_globs: vec!["**/src/**".into()],
            max_results: 10,
            ..Default::default()
        };
        let results = s.search(&query);
        assert_eq!(results.len(), 1, "should only match src/lib.rs");
        assert!(results[0].file_path.contains("src/lib.rs"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_index_stats_tracking() {
        let dir = tmp_dir("stats");
        fs::write(dir.join("a.rs"), "fn a() {}").unwrap();
        fs::write(dir.join("b.rs"), "fn b() {}").unwrap();
        fs::write(dir.join("c.py"), "def c(): pass").unwrap();

        let mut s = AstCodeSearcher::new();
        s.index_directory(&dir, true).unwrap();
        let (files, _chunks) = s.index_stats();
        assert_eq!(files, 3, "should index 3 files");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_tokenize_metavars() {
        let tokens = AstCodeSearcher::tokenize("fn \\NAME\\(ARGS*) {");
        assert!(tokens.contains(&"fn".to_string()));
        assert!(tokens.contains(&"\\NAME".to_string()));
        assert!(tokens.contains(&"\\(ARGS*)".to_string()));
        assert!(tokens.contains(&"{".to_string()));
    }

    #[test]
    fn test_chunking_single_chunk_no_defs() {
        let chunks = AstCodeSearcher::chunk_code("no_defs.rs", "rust", "let x = 1;\nlet y = 2;\n");
        assert_eq!(chunks.len(), 1, "no defs → one chunk");
        assert_eq!(chunks[0].start_line, 1);
        assert_eq!(chunks[0].end_line, 2);
    }

    #[test]
    fn test_structural_match_nested_parens() {
        assert!(AstCodeSearcher::structural_match(
            "deeply(nested(fn(a, b)))",
            "deeply\\(ARGS*)"
        ));
        assert!(AstCodeSearcher::structural_match(
            "deeply(nested(fn(a, b)))",
            "\\NAME\\(ARGS*)"
        ));
    }

    #[test]
    fn test_structural_match_subsequence_ignores_extra_code() {
        // Pattern tokens appear as subsequence
        assert!(AstCodeSearcher::structural_match(
            "  fn  hello_world ( x : i32 ) { return x + 1; }  ",
            "fn \\NAME\\(ARGS*) { }"
        ));
    }

    #[test]
    fn test_language_for_ext() {
        let mut lang_map: HashMap<String, Vec<String>> = HashMap::new();
        lang_map.insert("rust".into(), vec!["rs".into()]);
        lang_map.insert("python".into(), vec!["py".into()]);

        assert_eq!(
            AstCodeSearcher::language_for_ext("rs", &lang_map),
            Some("rust".into())
        );
        assert_eq!(
            AstCodeSearcher::language_for_ext("py", &lang_map),
            Some("python".into())
        );
        assert_eq!(
            AstCodeSearcher::language_for_ext("js", &lang_map),
            None
        );
    }

    #[test]
    fn test_path_glob_matching() {
        assert!(path_matches_glob("src/main.rs", "**/*.rs"));
        assert!(path_matches_glob("src/main.rs", "**/main.rs"));
        assert!(!path_matches_glob("src/main.rs", "**/test.rs"));
        assert!(path_matches_glob("main.rs", "*.rs"));
        assert!(!path_matches_glob("main.rs", "*.py"));
        assert!(path_matches_glob("src/lib.rs", "src/*.rs"));
        assert!(!path_matches_glob("other/lib.rs", "src/*.rs"));
        assert!(path_matches_glob("/tmp/xxx/src/lib.rs", "**/src/**"));
        assert!(path_matches_glob("/tmp/xxx/src/nested/lib.rs", "**/src/**"));
        assert!(!path_matches_glob("/tmp/xxx/lib.rs", "**/src/**"));
    }
}
