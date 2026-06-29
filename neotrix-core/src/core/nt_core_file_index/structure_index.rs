//! # L2: StructureIndex — 代码结构索引（轻量）
//!
//! 从 Rust 源文件中提取符号级结构信息：
//! - pub fn / fn 签名
//! - pub struct / enum / trait 声明
//! - impl 块
//!
//! 每个符号作为独立条目索引，支持精确跳转。
//! 使用轻量行匹配而非完整 AST（无需 tree-sitter 外部依赖）。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 代码符号类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    TypeAlias,
    Const,
    Module,
    Macro,
    Unknown,
}

/// L2 结构索引条目
#[derive(Debug, Clone)]
pub struct StructureEntry {
    pub path: PathBuf,
    pub symbol_name: String,
    pub kind: SymbolKind,
    /// 行号 (1-based)
    pub line: usize,
    /// 签名摘要
    pub signature: String,
    /// 可见性
    pub is_pub: bool,
}

/// L2: 基于符号的代码结构索引
#[derive(Debug, Clone)]
pub struct StructureIndex {
    /// 符号名 → 条目
    pub by_name: HashMap<String, Vec<StructureEntry>>,
    /// 路径 → 条目
    pub by_path: HashMap<PathBuf, Vec<StructureEntry>>,
    /// 类型 → 条目
    pub by_kind: HashMap<SymbolKind, Vec<StructureEntry>>,
}

impl StructureIndex {
    pub fn new() -> Self {
        Self {
            by_name: HashMap::new(),
            by_path: HashMap::new(),
            by_kind: HashMap::new(),
        }
    }

    /// 索引单个 .rs 文件的符号
    pub fn upsert_rs(&mut self, path: &Path) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let mut entries = Vec::new();

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            let line_no = i + 1;

            if let Some((kind, name, is_pub)) = Self::parse_symbol(trimmed) {
                let entry = StructureEntry {
                    path: path.to_path_buf(),
                    symbol_name: name.to_string(),
                    kind,
                    line: line_no,
                    signature: trimmed.to_string(),
                    is_pub,
                };
                entries.push(entry);
            }
        }

        self.by_path.insert(path.to_path_buf(), entries.clone());
        for entry in entries {
            self.by_name
                .entry(entry.symbol_name.clone())
                .or_default()
                .push(entry.clone());
            self.by_kind
                .entry(entry.kind.clone())
                .or_default()
                .push(entry);
        }
    }

    /// 轻量符号解析（无需 AST）
    fn parse_symbol(line: &str) -> Option<(SymbolKind, &str, bool)> {
        let line = line.trim();

        if line.starts_with("pub fn ") {
            let name = line.split("pub fn ").nth(1)?.split('(').next()?.trim();
            return Some((SymbolKind::Function, name, true));
        }
        if line.starts_with("fn ") {
            let name = line.split("fn ").nth(1)?.split('(').next()?.trim();
            return Some((SymbolKind::Function, name, false));
        }
        if line.starts_with("pub struct ") {
            let name = line
                .split("pub struct ")
                .nth(1)?
                .split('<')
                .next()?
                .split('{')
                .next()?
                .trim();
            return Some((SymbolKind::Struct, name, true));
        }
        if line.starts_with("struct ") {
            let name = line
                .split("struct ")
                .nth(1)?
                .split('<')
                .next()?
                .split('{')
                .next()?
                .trim();
            return Some((SymbolKind::Struct, name, false));
        }
        if line.starts_with("pub enum ") {
            let name = line
                .split("pub enum ")
                .nth(1)?
                .split('<')
                .next()?
                .split('{')
                .next()?
                .trim();
            return Some((SymbolKind::Enum, name, true));
        }
        if line.starts_with("pub trait ") {
            let name = line
                .split("pub trait ")
                .nth(1)?
                .split('<')
                .next()?
                .split('{')
                .next()?
                .trim();
            return Some((SymbolKind::Trait, name, true));
        }
        if line.starts_with("pub type ") {
            let name = line.split("pub type ").nth(1)?.split('=').next()?.trim();
            return Some((SymbolKind::TypeAlias, name, true));
        }
        if line.starts_with("impl") {
            let name = line
                .split("impl")
                .nth(1)?
                .split("for")
                .next()
                .or_else(|| line.split("impl").nth(1))?
                .trim();
            return Some((SymbolKind::Impl, name, false));
        }
        if line.starts_with("pub const ") {
            let name = line.split("pub const ").nth(1)?.split(':').next()?.trim();
            return Some((SymbolKind::Const, name, true));
        }

        None
    }

    /// 按符号名查询
    pub fn query_by_name(&self, name: &str, top_k: usize) -> Vec<StructureEntry> {
        let mut results = Vec::new();
        for (sym_name, entries) in &self.by_name {
            if sym_name.to_lowercase().contains(&name.to_lowercase()) {
                results.extend(entries.clone());
            }
        }
        results.sort_by_key(|e| e.line);
        results.truncate(top_k);
        results
    }

    /// 按类型查询
    pub fn query(&self, q: &super::FileQuery) -> Vec<super::ScoredFile> {
        let mut results = Vec::new();

        for kw in &q.keywords {
            // 按符号名匹配
            if let Some(entries) = self.by_name.get(kw) {
                for e in entries {
                    let module_match = q
                        .module_hint
                        .as_ref()
                        .map_or(true, |m| e.path.to_string_lossy().contains(m));
                    if module_match {
                        results.push(super::ScoredFile {
                            path: e.path.clone(),
                            score: 1.0,
                            source_layer: "L2:Struct",
                            snippet: Some(e.signature.clone()),
                            line: Some(e.line),
                        });
                    }
                }
            }

            // 模糊匹配
            for (sym_name, entries) in &self.by_name {
                if sym_name.to_lowercase().contains(&kw.to_lowercase()) && *sym_name != *kw {
                    for e in entries {
                        results.push(super::ScoredFile {
                            path: e.path.clone(),
                            score: 0.7,
                            source_layer: "L2:Struct",
                            snippet: Some(e.signature.clone()),
                            line: Some(e.line),
                        });
                    }
                }
            }
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut seen = std::collections::HashSet::new();
        results.retain(|r| seen.insert((r.path.clone(), r.line)));
        results.truncate(q.top_k);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pub_fn() {
        let (kind, name, is_pub) = StructureIndex::parse_symbol("pub fn foo() -> i32 {}").unwrap();
        assert_eq!(kind, SymbolKind::Function);
        assert_eq!(name, "foo");
        assert!(is_pub);
    }

    #[test]
    fn test_parse_private_fn() {
        let (kind, name, is_pub) = StructureIndex::parse_symbol("fn bar(x: usize) {}").unwrap();
        assert_eq!(kind, SymbolKind::Function);
        assert_eq!(name, "bar");
        assert!(!is_pub);
    }

    #[test]
    fn test_parse_struct() {
        let (kind, name, _) = StructureIndex::parse_symbol("pub struct MyStruct<T> {").unwrap();
        assert_eq!(kind, SymbolKind::Struct);
        assert_eq!(name, "MyStruct");
    }

    #[test]
    fn test_upsert_and_query() {
        let dir = std::env::temp_dir();
        let f = dir.join("_si_test.rs");
        std::fs::write(&f, "pub fn hello() {}\npub struct World;\nfn hidden() {}\n").ok();

        let mut si = StructureIndex::new();
        si.upsert_rs(&f);

        let fns = si.by_kind.get(&SymbolKind::Function).unwrap();
        assert_eq!(fns.len(), 2);

        let structs = si.by_kind.get(&SymbolKind::Struct).unwrap();
        assert_eq!(structs.len(), 1);

        let _ = std::fs::remove_file(&f);
    }
}
