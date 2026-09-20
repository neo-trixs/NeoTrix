//! Tree-sitter Integration — AST-aware code analysis
//!
//! Provides structural code understanding as an abstraction layer over
//! regex-based extraction. Currently delegates to regex patterns; the
//! `CodeParser` trait allows plugging in tree-sitter or other AST backends
//! without changing callers.
//!
//! Capabilities:
//! - Symbol extraction (functions, structs, enums, traits, impls, etc.)
//! - Call graph construction
//! - Import dependency tracking
//! - Language detection from filename
//!
//! Design:
//! - `CodeParser` trait is the single entry point for all parsing backends
//! - `RegexParser` is the default fallback (zero external dependencies)
//! - `Language` enum covers NeoTrix's supported languages
//! - All types are `Serialize`/`Deserialize` for KB storage and cross-layer transport

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ════════════════════════════════════════════════════════════════
// Public Types
// ════════════════════════════════════════════════════════════════

/// Supported programming languages for parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    Cpp,
    C,
    CSharp,
    Markdown,
    Json,
    Yaml,
    Toml,
    Shell,
    Unknown,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => write!(f, "rust"),
            Self::TypeScript => write!(f, "typescript"),
            Self::JavaScript => write!(f, "javascript"),
            Self::Python => write!(f, "python"),
            Self::Go => write!(f, "go"),
            Self::Java => write!(f, "java"),
            Self::Cpp => write!(f, "cpp"),
            Self::C => write!(f, "c"),
            Self::CSharp => write!(f, "csharp"),
            Self::Markdown => write!(f, "markdown"),
            Self::Json => write!(f, "json"),
            Self::Yaml => write!(f, "yaml"),
            Self::Toml => write!(f, "toml"),
            Self::Shell => write!(f, "shell"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Kind of code symbol extracted from source.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    TypeAlias,
    Constant,
    Module,
    Method,
    Field,
    Import,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function => write!(f, "fn"),
            Self::Struct => write!(f, "struct"),
            Self::Enum => write!(f, "enum"),
            Self::Trait => write!(f, "trait"),
            Self::Impl => write!(f, "impl"),
            Self::TypeAlias => write!(f, "type"),
            Self::Constant => write!(f, "const"),
            Self::Module => write!(f, "mod"),
            Self::Method => write!(f, "method"),
            Self::Field => write!(f, "field"),
            Self::Import => write!(f, "import"),
        }
    }
}

/// A single code symbol extracted from source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    /// Parent symbol name (e.g., method → struct, nested fn → module).
    pub parent: Option<String>,
    /// Names of child symbols (e.g., struct → methods, module → fns).
    pub children: Vec<String>,
}

/// An edge in the call graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEdge {
    pub caller: String,
    pub callee: String,
    pub line: usize,
}

/// An edge in the import/dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportEdge {
    pub source_file: String,
    pub target_module: String,
    pub kind: ImportKind,
}

/// Type of import relationship.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportKind {
    /// `use foo::bar` / `import foo.bar`
    Direct,
    /// `use foo::*` / `import foo.*`
    Glob,
    /// `mod foo` / file-level import
    Module,
}

/// Result of parsing a single source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub symbols: Vec<Symbol>,
    pub call_graph: Vec<CallEdge>,
    pub imports: Vec<ImportEdge>,
    pub language: Language,
}

// ════════════════════════════════════════════════════════════════
// CodeParser Trait
// ════════════════════════════════════════════════════════════════

/// Trait for code parsing backends.
///
/// Implementations produce a [`ParseResult`] from source code. The default
/// implementation uses regex patterns; a tree-sitter backend can be added
/// behind this same trait without changing callers.
pub trait CodeParser: Send + Sync {
    /// Parse source code and extract symbols, call graph, and imports.
    fn parse(&self, code: &str, language: Language) -> ParseResult;

    /// Backend name for diagnostics and logging.
    fn backend_name(&self) -> &str;

    /// Whether this backend supports the given language natively (AST-level).
    fn supports_language(&self, language: Language) -> bool;
}

// ════════════════════════════════════════════════════════════════
// Language Detection
// ════════════════════════════════════════════════════════════════

/// Detect programming language from a filename (extension-based).
pub fn detect_language(filename: &str) -> Language {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "rs" => Language::Rust,
        "ts" | "tsx" => Language::TypeScript,
        "js" | "jsx" | "mjs" | "cjs" => Language::JavaScript,
        "py" | "pyw" => Language::Python,
        "go" => Language::Go,
        "java" => Language::Java,
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => Language::Cpp,
        "c" | "h" => Language::C,
        "cs" => Language::CSharp,
        "md" | "mdx" => Language::Markdown,
        "json" | "jsonc" | "jsonl" => Language::Json,
        "yaml" | "yml" => Language::Yaml,
        "toml" => Language::Toml,
        "sh" | "bash" | "zsh" | "fish" | "ps1" => Language::Shell,
        _ => Language::Unknown,
    }
}

// ════════════════════════════════════════════════════════════════
// RegexParser — default fallback backend
// ════════════════════════════════════════════════════════════════

/// Regex-based code parser. Zero external dependencies, works as fallback
/// when tree-sitter is not available. Produces reasonable symbol extraction
/// for well-structured source files.
pub struct RegexParser;

impl Default for RegexParser {
    fn default() -> Self {
        Self
    }
}

impl RegexParser {
    pub fn new() -> Self {
        Self
    }
}

impl CodeParser for RegexParser {
    fn parse(&self, code: &str, language: Language) -> ParseResult {
        let symbols = extract_symbols_regex(code, language);
        let call_graph = extract_call_graph_regex(code, &symbols);
        let imports = extract_imports_regex(code, language);

        ParseResult {
            symbols,
            call_graph,
            imports,
            language,
        }
    }

    fn backend_name(&self) -> &str {
        "regex"
    }

    fn supports_language(&self, _language: Language) -> bool {
        true // regex fallback works for all languages (with varying quality)
    }
}

// ════════════════════════════════════════════════════════════════
// Regex-Based Symbol Extraction
// ════════════════════════════════════════════════════════════════

/// Extract symbols from source code using regex patterns.
///
/// This is the fallback extraction used by `RegexParser` and can also be
/// called directly for ad-hoc analysis.
pub fn extract_symbols_regex(code: &str, lang: Language) -> Vec<Symbol> {
    let lines: Vec<&str> = code.lines().collect();
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }

        match lang {
            Language::Rust => extract_rust_symbols(trimmed, i + 1, &mut symbols),
            Language::Python => extract_python_symbols(trimmed, i + 1, &mut symbols),
            Language::TypeScript | Language::JavaScript => {
                extract_js_ts_symbols(trimmed, i + 1, &mut symbols);
            }
            Language::Go => extract_go_symbols(trimmed, i + 1, &mut symbols),
            Language::Java | Language::Cpp | Language::C | Language::CSharp => {
                extract_c_style_symbols(trimmed, i + 1, &mut symbols);
            }
            Language::Shell => extract_shell_symbols(trimmed, i + 1, &mut symbols),
            _ => {}
        }
    }

    symbols
}

/// Extract call graph edges from source code using simple regex heuristics.
fn extract_call_graph_regex(code: &str, symbols: &[Symbol]) -> Vec<CallEdge> {
    let mut edges = Vec::new();
    let fn_names: Vec<&str> = symbols
        .iter()
        .filter(|s| matches!(s.kind, SymbolKind::Function | SymbolKind::Method))
        .map(|s| s.name.as_str())
        .collect();

    let mut current_fn: Option<String> = None;

    for (line_num, line) in code.lines().enumerate() {
        let trimmed = line.trim();

        // Track which function we're in (simplified: first fn seen)
        if let Some(name) = fn_names
            .iter()
            .find(|n| trimmed.contains(&format!("fn {n}")))
        {
            current_fn = Some(name.to_string());
        }

        if let Some(ref caller) = current_fn {
            for callee in &fn_names {
                if callee == caller {
                    continue;
                }
                // Simple heuristic: function name appears as identifier followed by '('
                if trimmed.contains(&format!("{callee}(")) {
                    edges.push(CallEdge {
                        caller: caller.clone(),
                        callee: callee.to_string(),
                        line: line_num + 1,
                    });
                }
            }
        }
    }

    edges
}

/// Extract import/dependency edges from source code.
fn extract_imports_regex(code: &str, lang: Language) -> Vec<ImportEdge> {
    let mut imports = Vec::new();

    for (line_num, line) in code.lines().enumerate() {
        let trimmed = line.trim();
        let kind = match lang {
            Language::Rust => {
                if trimmed.starts_with("use ") {
                    Some(ImportKind::Direct)
                } else if trimmed.starts_with("mod ") {
                    Some(ImportKind::Module)
                } else {
                    None
                }
            }
            Language::Python => {
                if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                    Some(ImportKind::Direct)
                } else {
                    None
                }
            }
            Language::TypeScript | Language::JavaScript => {
                if trimmed.starts_with("import ") {
                    if trimmed.contains("*") {
                        Some(ImportKind::Glob)
                    } else {
                        Some(ImportKind::Direct)
                    }
                } else if trimmed.starts_with("require(") {
                    Some(ImportKind::Direct)
                } else {
                    None
                }
            }
            Language::Go => {
                if trimmed.starts_with("import ") || trimmed.starts_with("\t\"") {
                    Some(ImportKind::Direct)
                } else {
                    None
                }
            }
            _ => {
                if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                    Some(ImportKind::Direct)
                } else {
                    None
                }
            }
        };

        if let Some(imp_kind) = kind {
            let target = extract_import_path(trimmed);
            if !target.is_empty() {
                imports.push(ImportEdge {
                    source_file: String::new(), // filled by caller if needed
                    target_module: target,
                    kind: imp_kind,
                });
            }
        }
    }

    imports
}

/// Extract the module path from an import line.
fn extract_import_path(line: &str) -> String {
    // Strip common prefixes
    let path = line
        .trim_start_matches("use ")
        .trim_start_matches("import ")
        .trim_start_matches("from ")
        .trim_start_matches("pub ")
        .trim_start_matches("mod ")
        .trim();

    // Remove trailing semicolons and braces
    let path = path.trim_end_matches(';').trim();

    // For "use foo::bar" style, keep as-is
    // For "from 'foo' import bar" style, extract the string literal
    if let Some(start) = path.find('\'') {
        if let Some(end) = path[start + 1..].find('\'') {
            return path[start + 1..start + 1 + end].to_string();
        }
    }
    if let Some(start) = path.find('"') {
        if let Some(end) = path[start + 1..].find('"') {
            return path[start + 1..start + 1 + end].to_string();
        }
    }

    path.to_string()
}

// ════════════════════════════════════════════════════════════════
// Language-Specific Symbol Extractors
// ════════════════════════════════════════════════════════════════

fn extract_rust_symbols(line: &str, line_num: usize, symbols: &mut Vec<Symbol>) {
    let kind = if line.starts_with("pub fn ") || line.starts_with("fn ") {
        Some(SymbolKind::Function)
    } else if line.starts_with("pub struct ") || line.starts_with("struct ") {
        Some(SymbolKind::Struct)
    } else if line.starts_with("pub enum ") || line.starts_with("enum ") {
        Some(SymbolKind::Enum)
    } else if line.starts_with("pub trait ") || line.starts_with("trait ") {
        Some(SymbolKind::Trait)
    } else if line.starts_with("impl ") || line.starts_with("impl<") {
        Some(SymbolKind::Impl)
    } else if line.starts_with("pub type ") || line.starts_with("type ") {
        Some(SymbolKind::TypeAlias)
    } else if line.starts_with("pub const ") || line.starts_with("const ") {
        Some(SymbolKind::Constant)
    } else if line.starts_with("pub mod ") || line.starts_with("mod ") {
        Some(SymbolKind::Module)
    } else if line.starts_with("pub async fn ") || line.starts_with("async fn ") {
        Some(SymbolKind::Function)
    } else {
        None
    };

    if let Some(kind) = kind {
        let name = extract_rust_name(line, &kind);
        if !name.is_empty() {
            symbols.push(Symbol {
                name,
                kind,
                line_start: line_num,
                line_end: line_num, // regex backend: single-line precision
                column_start: 0,
                column_end: 0,
                parent: None,
                children: Vec::new(),
            });
        }
    }
}

fn extract_rust_name(line: &str, kind: &SymbolKind) -> String {
    let prefix = match kind {
        SymbolKind::Function => {
            if line.contains("pub async fn ") {
                "pub async fn "
            } else if line.contains("async fn ") {
                "async fn "
            } else if line.contains("pub fn ") {
                "pub fn "
            } else {
                "fn "
            }
        }
        SymbolKind::Struct => {
            if line.contains("pub struct ") {
                "pub struct "
            } else {
                "struct "
            }
        }
        SymbolKind::Enum => {
            if line.contains("pub enum ") {
                "pub enum "
            } else {
                "enum "
            }
        }
        SymbolKind::Trait => {
            if line.contains("pub trait ") {
                "pub trait "
            } else {
                "trait "
            }
        }
        SymbolKind::Impl => "impl",
        SymbolKind::TypeAlias => {
            if line.contains("pub type ") {
                "pub type "
            } else {
                "type "
            }
        }
        SymbolKind::Constant => {
            if line.contains("pub const ") {
                "pub const "
            } else {
                "const "
            }
        }
        SymbolKind::Module => {
            if line.contains("pub mod ") {
                "pub mod "
            } else {
                "mod "
            }
        }
        _ => "",
    };

    let rest = line.strip_prefix(prefix).unwrap_or(line);
    // For impl blocks, extract the type name
    if *kind == SymbolKind::Impl {
        let without_impl = rest.trim_start_matches("impl").trim();
        // Handle "impl<T> Foo" or "impl Foo"
        if let Some(gt_pos) = without_impl.find('>') {
            let name = without_impl[gt_pos + 1..].trim();
            // Extract up to '{' or 'for'
            let name = name.split('{').next().unwrap_or(name).trim();
            let name = name.split(" for ").next().unwrap_or(name).trim();
            return name.to_string();
        }
        let name = without_impl
            .split('{')
            .next()
            .unwrap_or(without_impl)
            .trim();
        let name = name.split(" for ").next().unwrap_or(name).trim();
        return name.to_string();
    }

    // Extract name: take up to '(' for fns, '{' or ':' for types
    match kind {
        SymbolKind::Function => {
            let name = rest.split('(').next().unwrap_or(rest).trim();
            let name = name.split('<').next().unwrap_or(name).trim();
            name.to_string()
        }
        _ => {
            let name = rest.split('{').next().unwrap_or(rest).trim();
            let name = name.split('<').next().unwrap_or(name).trim();
            let name = name.split(':').next().unwrap_or(name).trim();
            name.to_string()
        }
    }
}

fn extract_python_symbols(line: &str, line_num: usize, symbols: &mut Vec<Symbol>) {
    let kind = if line.starts_with("def ") {
        Some(SymbolKind::Function)
    } else if line.starts_with("class ") {
        Some(SymbolKind::Struct) // Python classes map to Struct
    } else if line.starts_with("async def ") {
        Some(SymbolKind::Function)
    } else if line.starts_with("import ") || line.starts_with("from ") {
        Some(SymbolKind::Import)
    } else {
        None
    };

    if let Some(kind) = kind {
        let name = match kind {
            SymbolKind::Function => {
                let prefix = if line.starts_with("async def ") {
                    "async def "
                } else {
                    "def "
                };
                line.strip_prefix(prefix)
                    .unwrap_or(line)
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string()
            }
            SymbolKind::Struct => line
                .strip_prefix("class ")
                .unwrap_or(line)
                .split('(')
                .next()
                .unwrap_or("")
                .split(':')
                .next()
                .unwrap_or("")
                .trim()
                .to_string(),
            SymbolKind::Import => {
                // For imports, extract the module path
                let rest = line
                    .strip_prefix("from ")
                    .or_else(|| line.strip_prefix("import "))
                    .unwrap_or(line);
                rest.split(" import")
                    .next()
                    .unwrap_or(rest)
                    .trim()
                    .trim_end_matches(';')
                    .to_string()
            }
            _ => String::new(),
        };

        if !name.is_empty() {
            symbols.push(Symbol {
                name,
                kind,
                line_start: line_num,
                line_end: line_num,
                column_start: 0,
                column_end: 0,
                parent: None,
                children: Vec::new(),
            });
        }
    }
}

fn extract_js_ts_symbols(line: &str, line_num: usize, symbols: &mut Vec<Symbol>) {
    let kind = if line.starts_with("function ") || line.starts_with("async function ") {
        Some(SymbolKind::Function)
    } else if line.starts_with("const ") && line.contains("=>") {
        // Arrow function: const foo = () =>
        Some(SymbolKind::Function)
    } else if line.starts_with("export function ") || line.starts_with("export async function ") {
        Some(SymbolKind::Function)
    } else if line.starts_with("export default function ") {
        Some(SymbolKind::Function)
    } else if line.starts_with("class ") || line.starts_with("export class ") {
        Some(SymbolKind::Struct)
    } else if line.starts_with("interface ") || line.starts_with("export interface ") {
        Some(SymbolKind::Trait)
    } else if line.starts_with("type ") || line.starts_with("export type ") {
        Some(SymbolKind::TypeAlias)
    } else if line.starts_with("const ") || line.starts_with("let ") || line.starts_with("var ") {
        // Could be a constant or variable
        if line.contains("function") || line.contains("=>") {
            None // already handled above
        } else {
            Some(SymbolKind::Constant)
        }
    } else {
        None
    };

    if let Some(kind) = kind {
        let name = extract_js_ts_name(line, &kind);
        if !name.is_empty() {
            symbols.push(Symbol {
                name,
                kind,
                line_start: line_num,
                line_end: line_num,
                column_start: 0,
                column_end: 0,
                parent: None,
                children: Vec::new(),
            });
        }
    }
}

fn extract_js_ts_name(line: &str, kind: &SymbolKind) -> String {
    let line = line
        .trim_start_matches("export ")
        .trim_start_matches("default ")
        .trim();

    match kind {
        SymbolKind::Function => {
            let prefix = if line.starts_with("async function ") {
                "async function "
            } else if line.starts_with("function ") {
                "function "
            } else {
                // Arrow function: const name = ...
                let rest = line
                    .strip_prefix("const ")
                    .or_else(|| line.strip_prefix("let "))
                    .or_else(|| line.strip_prefix("var "))
                    .unwrap_or(line);
                return rest.split('=').next().unwrap_or(rest).trim().to_string();
            };
            line.strip_prefix(prefix)
                .unwrap_or(line)
                .split('(')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        }
        SymbolKind::Struct => {
            // class Name { ... }
            let prefix = if line.starts_with("class ") {
                "class "
            } else {
                "class "
            };
            line.strip_prefix(prefix)
                .unwrap_or(line)
                .split('{')
                .next()
                .unwrap_or("")
                .split('(')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        }
        SymbolKind::Trait => {
            // interface Name { ... }
            line.strip_prefix("interface ")
                .unwrap_or(line)
                .split('{')
                .next()
                .unwrap_or("")
                .split('<')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        }
        SymbolKind::TypeAlias => {
            // type Name = ...
            line.strip_prefix("type ")
                .unwrap_or(line)
                .split('=')
                .next()
                .unwrap_or("")
                .split('<')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        }
        SymbolKind::Constant => {
            // const NAME = ...
            let prefix = if line.starts_with("const ") {
                "const "
            } else if line.starts_with("let ") {
                "let "
            } else {
                "var "
            };
            line.strip_prefix(prefix)
                .unwrap_or(line)
                .split('=')
                .next()
                .unwrap_or("")
                .split(':')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        }
        _ => String::new(),
    }
}

fn extract_go_symbols(line: &str, line_num: usize, symbols: &mut Vec<Symbol>) {
    let kind = if line.starts_with("func ") || line.starts_with("func\t") {
        Some(SymbolKind::Function)
    } else if line.starts_with("type ") && line.contains("struct") {
        Some(SymbolKind::Struct)
    } else if line.starts_with("type ") && line.contains("interface") {
        Some(SymbolKind::Trait)
    } else if line.starts_with("type ") {
        Some(SymbolKind::TypeAlias)
    } else if line.starts_with("const ") || line.starts_with("var ") {
        Some(SymbolKind::Constant)
    } else {
        None
    };

    if let Some(kind) = kind {
        let name = match kind {
            SymbolKind::Function => {
                // func (receiver) Name(params) or func Name(params)
                let rest = line.strip_prefix("func ").unwrap_or(line);
                // Skip receiver if present
                if rest.starts_with('(') {
                    if let Some(end_paren) = rest.find(')') {
                        let after_receiver = rest[end_paren + 1..].trim();
                        after_receiver
                            .split('(')
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string()
                    } else {
                        rest.split('(').next().unwrap_or("").trim().to_string()
                    }
                } else {
                    rest.split('(').next().unwrap_or("").trim().to_string()
                }
            }
            _ => {
                let rest = line.strip_prefix("type ").unwrap_or(line);
                let rest = rest.strip_prefix("const ").unwrap_or(rest);
                let rest = rest.strip_prefix("var ").unwrap_or(rest);
                rest.split('=')
                    .next()
                    .unwrap_or("")
                    .split('{')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string()
            }
        };

        if !name.is_empty() {
            symbols.push(Symbol {
                name,
                kind,
                line_start: line_num,
                line_end: line_num,
                column_start: 0,
                column_end: 0,
                parent: None,
                children: Vec::new(),
            });
        }
    }
}

fn extract_c_style_symbols(line: &str, line_num: usize, symbols: &mut Vec<Symbol>) {
    let kind = if line.contains("def ") || line.contains("function ") {
        // Python-style inside C-like (shouldn't happen, but defensive)
        return;
    } else if line.starts_with("class ") || line.starts_with("public class ") {
        Some(SymbolKind::Struct)
    } else if line.starts_with("interface ") || line.starts_with("public interface ") {
        Some(SymbolKind::Trait)
    } else if line.starts_with("enum ") || line.starts_with("public enum ") {
        Some(SymbolKind::Enum)
    } else if line.starts_with("struct ") || line.starts_with("public struct ") {
        Some(SymbolKind::Struct)
    } else if line.starts_with("typedef ") {
        Some(SymbolKind::TypeAlias)
    } else if line.starts_with("#define ") {
        Some(SymbolKind::Constant)
    } else {
        None
    };

    if let Some(kind) = kind {
        let name = match kind {
            SymbolKind::Struct | SymbolKind::Trait | SymbolKind::Enum => {
                let stripped = line
                    .trim_start_matches("public ")
                    .trim_start_matches("private ")
                    .trim_start_matches("protected ")
                    .trim_start_matches("abstract ")
                    .trim_start_matches("static ");
                let keyword = match kind {
                    SymbolKind::Struct => "class",
                    SymbolKind::Trait => "interface",
                    SymbolKind::Enum => "enum",
                    _ => "",
                };
                stripped
                    .strip_prefix(keyword)
                    .unwrap_or(stripped)
                    .split('{')
                    .next()
                    .unwrap_or("")
                    .split('<')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string()
            }
            SymbolKind::TypeAlias => line
                .strip_prefix("typedef ")
                .unwrap_or(line)
                .split(';')
                .next()
                .unwrap_or("")
                .split('{')
                .next()
                .unwrap_or("")
                .trim()
                .to_string(),
            SymbolKind::Constant => line
                .strip_prefix("#define ")
                .unwrap_or(line)
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .to_string(),
            _ => String::new(),
        };

        if !name.is_empty() {
            symbols.push(Symbol {
                name,
                kind,
                line_start: line_num,
                line_end: line_num,
                column_start: 0,
                column_end: 0,
                parent: None,
                children: Vec::new(),
            });
        }
    }
}

fn extract_shell_symbols(line: &str, line_num: usize, symbols: &mut Vec<Symbol>) {
    let kind = if line.starts_with("function ") || line.starts_with("function\t") {
        Some(SymbolKind::Function)
    } else if line.contains("() {") || line.contains("() {") {
        Some(SymbolKind::Function)
    } else {
        None
    };

    if let Some(kind) = kind {
        let name = if line.starts_with("function ") {
            line.strip_prefix("function ")
                .unwrap_or(line)
                .split('{')
                .next()
                .unwrap_or("")
                .split('(')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        } else {
            // NAME() {
            line.split("()").next().unwrap_or("").trim().to_string()
        };

        if !name.is_empty() {
            symbols.push(Symbol {
                name,
                kind,
                line_start: line_num,
                line_end: line_num,
                column_start: 0,
                column_end: 0,
                parent: None,
                children: Vec::new(),
            });
        }
    }
}

// ════════════════════════════════════════════════════════════════
// TreeSitterBackend — stub for future tree-sitter integration
// ════════════════════════════════════════════════════════════════

/// Placeholder for a tree-sitter based parser backend.
///
/// Currently delegates to `RegexParser`. When the `tree-sitter` crate is
/// added as a dependency, this struct can be filled in with actual AST
/// traversal logic. The trait boundary ensures callers don't need to change.
pub struct TreeSitterBackend {
    fallback: RegexParser,
}

impl Default for TreeSitterBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeSitterBackend {
    pub fn new() -> Self {
        Self {
            fallback: RegexParser::new(),
        }
    }
}

impl CodeParser for TreeSitterBackend {
    fn parse(&self, code: &str, language: Language) -> ParseResult {
        // When tree-sitter crate is added, replace this delegation
        // with actual AST parsing:
        //   let parser = &mut self.parsers.get(&language);
        //   let tree = parser.parse(code, None);
        //   // Walk tree nodes → Symbol, CallEdge, ImportEdge
        self.fallback.parse(code, language)
    }

    fn backend_name(&self) -> &str {
        "tree-sitter (regex fallback)"
    }

    fn supports_language(&self, language: Language) -> bool {
        matches!(
            language,
            Language::Rust
                | Language::TypeScript
                | Language::JavaScript
                | Language::Python
                | Language::Go
                | Language::Java
                | Language::Cpp
                | Language::C
                | Language::CSharp
        )
    }
}

// ════════════════════════════════════════════════════════════════
// Convenience: parse_file — high-level entry point
// ════════════════════════════════════════════════════════════════

/// Parse a source file, auto-detecting language from the filename.
///
/// Uses `RegexParser` as the default backend. To use a different backend,
/// call `backend.parse(code, lang)` directly.
pub fn parse_file(code: &str, filename: &str) -> ParseResult {
    let lang = detect_language(filename);
    let parser = RegexParser::new();
    parser.parse(code, lang)
}

/// Parse a source file with an explicit backend.
pub fn parse_file_with(code: &str, filename: &str, backend: &dyn CodeParser) -> ParseResult {
    let lang = detect_language(filename);
    backend.parse(code, lang)
}

// ════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Language detection ──

    #[test]
    fn test_detect_language_rust() {
        assert_eq!(detect_language("main.rs"), Language::Rust);
        assert_eq!(detect_language("lib.rs"), Language::Rust);
    }

    #[test]
    fn test_detect_language_typescript() {
        assert_eq!(detect_language("index.ts"), Language::TypeScript);
        assert_eq!(detect_language("app.tsx"), Language::TypeScript);
    }

    #[test]
    fn test_detect_language_python() {
        assert_eq!(detect_language("main.py"), Language::Python);
        assert_eq!(detect_language("script.pyw"), Language::Python);
    }

    #[test]
    fn test_detect_language_go() {
        assert_eq!(detect_language("main.go"), Language::Go);
    }

    #[test]
    fn test_detect_language_unknown() {
        assert_eq!(detect_language("README"), Language::Unknown);
        assert_eq!(detect_language("Makefile"), Language::Unknown);
    }

    // ── Rust symbol extraction ──

    #[test]
    fn test_extract_rust_fn() {
        let code = "pub fn hello() -> i32 { 42 }";
        let symbols = extract_symbols_regex(code, Language::Rust);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "hello");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
    }

    #[test]
    fn test_extract_rust_async_fn() {
        let code = "pub async fn fetch() -> Result<()> { Ok(()) }";
        let symbols = extract_symbols_regex(code, Language::Rust);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "fetch");
    }

    #[test]
    fn test_extract_rust_struct() {
        let code = "pub struct Config {\n    pub name: String,\n}";
        let symbols = extract_symbols_regex(code, Language::Rust);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Config");
        assert_eq!(symbols[0].kind, SymbolKind::Struct);
    }

    #[test]
    fn test_extract_rust_enum() {
        let code = "pub enum Status {\n    Active,\n    Inactive,\n}";
        let symbols = extract_symbols_regex(code, Language::Rust);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Status");
        assert_eq!(symbols[0].kind, SymbolKind::Enum);
    }

    #[test]
    fn test_extract_rust_trait() {
        let code = "pub trait Drawable { fn draw(&self); }";
        let symbols = extract_symbols_regex(code, Language::Rust);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Drawable");
        assert_eq!(symbols[0].kind, SymbolKind::Trait);
    }

    #[test]
    fn test_extract_rust_impl() {
        let code = "impl Drawable for Circle { fn draw(&self) {} }";
        let symbols = extract_symbols_regex(code, Language::Rust);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Circle");
        assert_eq!(symbols[0].kind, SymbolKind::Impl);
    }

    #[test]
    fn test_extract_rust_impl_generic() {
        let code = "impl<T> Vec<T> { fn push(&mut self, val: T) {} }";
        let symbols = extract_symbols_regex(code, Language::Rust);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Vec");
        assert_eq!(symbols[0].kind, SymbolKind::Impl);
    }

    #[test]
    fn test_extract_rust_type_alias() {
        let code = "pub type Result<T> = std::result::Result<T, Error>;";
        let symbols = extract_symbols_regex(code, Language::Rust);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Result");
        assert_eq!(symbols[0].kind, SymbolKind::TypeAlias);
    }

    #[test]
    fn test_extract_rust_const() {
        let code = "pub const MAX_SIZE: usize = 1024;";
        let symbols = extract_symbols_regex(code, Language::Rust);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MAX_SIZE");
        assert_eq!(symbols[0].kind, SymbolKind::Constant);
    }

    #[test]
    fn test_extract_rust_mod() {
        let code = "pub mod utils;";
        let symbols = extract_symbols_regex(code, Language::Rust);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "utils");
        assert_eq!(symbols[0].kind, SymbolKind::Module);
    }

    // ── Python symbol extraction ──

    #[test]
    fn test_extract_python_fn() {
        let code = "def hello(): pass";
        let symbols = extract_symbols_regex(code, Language::Python);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "hello");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
    }

    #[test]
    fn test_extract_python_class() {
        let code = "class Foo: pass";
        let symbols = extract_symbols_regex(code, Language::Python);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Foo");
        assert_eq!(symbols[0].kind, SymbolKind::Struct);
    }

    #[test]
    fn test_extract_python_import() {
        let code = "import os";
        let symbols = extract_symbols_regex(code, Language::Python);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "os");
        assert_eq!(symbols[0].kind, SymbolKind::Import);
    }

    // ── JS/TS symbol extraction ──

    #[test]
    fn test_extract_js_function() {
        let code = "function greet(name) {}";
        let symbols = extract_symbols_regex(code, Language::JavaScript);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "greet");
    }

    #[test]
    fn test_extract_ts_interface() {
        let code = "interface User { name: string; }";
        let symbols = extract_symbols_regex(code, Language::TypeScript);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "User");
        assert_eq!(symbols[0].kind, SymbolKind::Trait);
    }

    #[test]
    fn test_extract_ts_type_alias() {
        let code = "type ID = string | number;";
        let symbols = extract_symbols_regex(code, Language::TypeScript);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "ID");
        assert_eq!(symbols[0].kind, SymbolKind::TypeAlias);
    }

    // ── Go symbol extraction ──

    #[test]
    fn test_extract_go_fn() {
        let code = "func main() {}";
        let symbols = extract_symbols_regex(code, Language::Go);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "main");
    }

    #[test]
    fn test_extract_go_struct() {
        let code = "type Config struct { Name string }";
        let symbols = extract_symbols_regex(code, Language::Go);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Config");
        assert_eq!(symbols[0].kind, SymbolKind::Struct);
    }

    // ── RegexParser trait ──

    #[test]
    fn test_regex_parser_backend_name() {
        let parser = RegexParser::new();
        assert_eq!(parser.backend_name(), "regex");
    }

    #[test]
    fn test_regex_parser_supports_all_languages() {
        let parser = RegexParser::new();
        assert!(parser.supports_language(Language::Rust));
        assert!(parser.supports_language(Language::Python));
        assert!(parser.supports_language(Language::Unknown));
    }

    // ── TreeSitterBackend stub ──

    #[test]
    fn test_tree_sitter_backend_delegates_to_regex() {
        let parser = TreeSitterBackend::new();
        let code = "pub fn hello() -> i32 { 42 }";
        let result = parser.parse(code, Language::Rust);
        assert_eq!(result.symbols.len(), 1);
        assert_eq!(result.symbols[0].name, "hello");
        assert_eq!(parser.backend_name(), "tree-sitter (regex fallback)");
    }

    // ── ParseResult serialization round-trip ──

    #[test]
    fn test_parse_result_serde_roundtrip() {
        let result = ParseResult {
            symbols: vec![Symbol {
                name: "foo".to_string(),
                kind: SymbolKind::Function,
                line_start: 1,
                line_end: 5,
                column_start: 0,
                column_end: 10,
                parent: Some("Bar".to_string()),
                children: vec!["inner".to_string()],
            }],
            call_graph: vec![CallEdge {
                caller: "foo".to_string(),
                callee: "bar".to_string(),
                line: 3,
            }],
            imports: vec![ImportEdge {
                source_file: "a.rs".to_string(),
                target_module: "std::collections::HashMap".to_string(),
                kind: ImportKind::Direct,
            }],
            language: Language::Rust,
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: ParseResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.symbols.len(), 1);
        assert_eq!(parsed.symbols[0].name, "foo");
        assert_eq!(parsed.call_graph.len(), 1);
        assert_eq!(parsed.imports.len(), 1);
        assert_eq!(parsed.language, Language::Rust);
    }

    // ── Import extraction ──

    #[test]
    fn test_extract_rust_imports() {
        let code = "use std::collections::HashMap;\nuse crate::foo::Bar;";
        let imports = extract_imports_regex(code, Language::Rust);
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].target_module, "std::collections::HashMap");
        assert_eq!(imports[1].target_module, "crate::foo::Bar");
    }

    #[test]
    fn test_extract_js_imports() {
        let code = "import React from 'react';\nimport { useState } from 'react';";
        let imports = extract_imports_regex(code, Language::TypeScript);
        assert_eq!(imports.len(), 2);
    }

    // ── Multi-symbol extraction ──

    #[test]
    fn test_extract_multiple_rust_symbols() {
        let code = r#"
pub struct Config {
    pub name: String,
}

impl Config {
    pub fn new() -> Self { Self { name: String::new() } }
    pub fn get_name(&self) -> &str { &self.name }
}

pub trait Drawable {
    fn draw(&self);
}

pub fn main() {
    let c = Config::new();
    println!("{}", c.get_name());
}
"#;
        let symbols = extract_symbols_regex(code, Language::Rust);
        let kinds: Vec<&SymbolKind> = symbols.iter().map(|s| &s.kind).collect();
        assert!(kinds.contains(&&SymbolKind::Struct));
        assert!(kinds.contains(&&SymbolKind::Impl));
        assert!(kinds.contains(&&SymbolKind::Trait));
        assert!(kinds.contains(&&SymbolKind::Function));
    }

    // ── parse_file convenience ──

    #[test]
    fn test_parse_file_convenience() {
        let code = "pub fn hello() -> i32 { 42 }";
        let result = parse_file(code, "src/main.rs");
        assert_eq!(result.language, Language::Rust);
        assert!(!result.symbols.is_empty());
    }

    // ── Empty/whitespace input ──

    #[test]
    fn test_parse_empty_code() {
        let result = parse_file("", "test.rs");
        assert!(result.symbols.is_empty());
        assert!(result.call_graph.is_empty());
        assert!(result.imports.is_empty());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let result = parse_file("   \n  \n   ", "test.rs");
        assert!(result.symbols.is_empty());
    }
}
