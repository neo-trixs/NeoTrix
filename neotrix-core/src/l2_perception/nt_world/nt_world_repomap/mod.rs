use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CodeSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    Constant,
    Type,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function => write!(f, "fn"),
            Self::Struct => write!(f, "struct"),
            Self::Enum => write!(f, "enum"),
            Self::Trait => write!(f, "trait"),
            Self::Impl => write!(f, "impl"),
            Self::Module => write!(f, "mod"),
            Self::Constant => write!(f, "const"),
            Self::Type => write!(f, "type"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileIndex {
    pub path: PathBuf,
    pub symbols: Vec<CodeSymbol>,
    pub imports: Vec<String>,
    pub line_count: usize,
}

#[derive(Debug, Clone)]
pub struct RepoMap {
    pub files: HashMap<PathBuf, FileIndex>,
    pub symbol_index: HashMap<String, Vec<PathBuf>>,
    root: PathBuf,
}

impl RepoMap {
    pub fn new(root: &Path) -> Self {
        Self {
            files: HashMap::new(),
            symbol_index: HashMap::new(),
            root: root.to_path_buf(),
        }
    }

    pub fn index_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();
        let line_count = lines.len();
        let mut symbols = Vec::new();
        let mut imports = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") || trimmed.starts_with("from ") {
                imports.push(trimmed.to_string());
            }
            if trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") {
                symbols.push(CodeSymbol {
                    name: trimmed
                        .split('(')
                        .next()
                        .unwrap_or("")
                        .trim_start_matches("pub ")
                        .trim_start_matches("async ")
                        .to_string(),
                    kind: SymbolKind::Function,
                    file: path.to_path_buf(),
                    line: i + 1,
                    column: 0,
                    doc_comment: None,
                });
            }
            if trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ") {
                symbols.push(CodeSymbol {
                    name: trimmed
                        .split('{')
                        .next()
                        .unwrap_or("")
                        .trim_start_matches("pub ")
                        .trim_start_matches("struct ")
                        .trim()
                        .to_string(),
                    kind: SymbolKind::Struct,
                    file: path.to_path_buf(),
                    line: i + 1,
                    column: 0,
                    doc_comment: None,
                });
            }
            if trimmed.starts_with("pub enum ") || trimmed.starts_with("enum ") {
                symbols.push(CodeSymbol {
                    name: trimmed
                        .split('{')
                        .next()
                        .unwrap_or("")
                        .trim_start_matches("pub ")
                        .trim_start_matches("enum ")
                        .trim()
                        .to_string(),
                    kind: SymbolKind::Enum,
                    file: path.to_path_buf(),
                    line: i + 1,
                    column: 0,
                    doc_comment: None,
                });
            }
            if trimmed.starts_with("pub trait ") || trimmed.starts_with("trait ") {
                symbols.push(CodeSymbol {
                    name: trimmed
                        .split('{')
                        .next()
                        .unwrap_or("")
                        .trim_start_matches("pub ")
                        .trim_start_matches("trait ")
                        .trim()
                        .to_string(),
                    kind: SymbolKind::Trait,
                    file: path.to_path_buf(),
                    line: i + 1,
                    column: 0,
                    doc_comment: None,
                });
            }
            if trimmed.starts_with("pub mod ") || trimmed.starts_with("mod ") {
                symbols.push(CodeSymbol {
                    name: trimmed
                        .split(';')
                        .next()
                        .unwrap_or("")
                        .trim_start_matches("pub ")
                        .trim_start_matches("mod ")
                        .trim()
                        .to_string(),
                    kind: SymbolKind::Module,
                    file: path.to_path_buf(),
                    line: i + 1,
                    column: 0,
                    doc_comment: None,
                });
            }
        }

        for sym in &symbols {
            self.symbol_index
                .entry(sym.name.clone())
                .or_default()
                .push(path.to_path_buf());
        }
        self.files.insert(
            path.to_path_buf(),
            FileIndex {
                path: path.to_path_buf(),
                symbols,
                imports,
                line_count,
            },
        );
        Ok(())
    }

    pub fn index_directory(&mut self, dir: &Path) -> Result<u32, Box<dyn std::error::Error>> {
        let mut count = 0;
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir()
                    && !path.file_name().map_or(false, |n| {
                        n == "target" || n == ".git" || n == "node_modules"
                    })
                {
                    count += self.index_directory(&path)?;
                } else if path
                    .extension()
                    .map_or(false, |e| e == "rs" || e == "py" || e == "ts" || e == "js")
                {
                    if self.index_file(&path).is_ok() {
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }

    pub fn search_symbol(&self, name: &str) -> Vec<&CodeSymbol> {
        self.files
            .values()
            .flat_map(|f| f.symbols.iter())
            .filter(|s| s.name == name)
            .collect()
    }

    pub fn related_files(&self, name: &str) -> Vec<PathBuf> {
        self.symbol_index.get(name).cloned().unwrap_or_default()
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn symbol_count(&self) -> usize {
        self.files.values().map(|f| f.symbols.len()).sum()
    }

    pub fn total_lines(&self) -> usize {
        self.files.values().map(|f| f.line_count).sum()
    }

    pub fn summary(&self) -> String {
        format!(
            "RepoMap: {} files, {} symbols, {} total lines",
            self.file_count(),
            self.symbol_count(),
            self.total_lines()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_index_file() {
        let dir = std::env::temp_dir().join("repomap_test");
        let _ = std::fs::create_dir_all(&dir);
        let f = dir.join("test.rs");
        let mut fh = std::fs::File::create(&f).unwrap();
        writeln!(
            fh,
            "pub fn hello() -> i32 {{ 42 }}\npub struct Foo {{ x: i32 }}"
        )
        .unwrap();
        let mut map = RepoMap::new(&dir);
        map.index_file(&f).unwrap();
        assert_eq!(map.files.len(), 1);
        assert!(map.search_symbol("hello").len() > 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_empty() {
        let map = RepoMap::new(Path::new("/tmp"));
        assert_eq!(map.file_count(), 0);
    }
}
