//! Context Engine — Real-time Codebase Indexing
//!
//! Implements the Augment Code pattern: real-time semantic indexing of entire
//! codebase including dependencies, architecture, and history.
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │           Context Engine                     │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
//! │  │  Indexer │  │  Search  │  │  Cache   │  │
//! │  │  Engine  │  │  Engine  │  │  Layer   │  │
//! │  └──────────┘  └──────────┘  └──────────┘  │
//! │       ↓              ↓              ↓        │
//! │  ┌──────────────────────────────────────┐   │
//! │  │      File System Watcher             │   │
//! │  └──────────────────────────────────────┘   │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! # Safety
//! - All file operations are read-only
//! - Index updates are atomic
//! - No unsafe code (R-P1)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Core Types
// ============================================================================

/// Index entry for a file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexEntry {
    /// File path relative to workspace root
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// File extension
    pub extension: Option<String>,
    /// Language (detected from extension)
    pub language: Option<String>,
    /// Lines of code
    pub lines: usize,
    /// Imports/dependencies
    pub imports: Vec<String>,
    /// Exports/symbols
    pub exports: Vec<String>,
    /// Functions/methods
    pub functions: Vec<FunctionInfo>,
    /// Classes/structs
    pub types: Vec<TypeInfo>,
    /// Last modified timestamp
    pub last_modified: String,
    /// Content hash (for change detection)
    pub content_hash: String,
    /// Embedding vector (for semantic search)
    pub embedding: Option<Vec<f32>>,
}

/// Function information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,
    /// Function signature
    pub signature: String,
    /// Start line
    pub start_line: usize,
    /// End line
    pub end_line: usize,
    /// Parameters
    pub parameters: Vec<String>,
    /// Return type
    pub return_type: Option<String>,
    /// Whether it's public
    pub is_public: bool,
}

/// Type information (struct, class, enum, etc.)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TypeInfo {
    /// Type name
    pub name: String,
    /// Type kind (struct, class, enum, trait, etc.)
    pub kind: String,
    /// Start line
    pub start_line: usize,
    /// End line
    pub end_line: usize,
    /// Fields/methods
    pub members: Vec<String>,
    /// Whether it's public
    pub is_public: bool,
}

/// Search query
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextQuery {
    /// Search text
    pub text: String,
    /// Filter by language
    pub language: Option<String>,
    /// Filter by file pattern
    pub file_pattern: Option<String>,
    /// Filter by symbol type
    pub symbol_type: Option<String>,
    /// Maximum results
    pub limit: Option<usize>,
    /// Include embeddings in results
    pub include_embeddings: bool,
}

/// Search result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextResult {
    /// Matching entries
    pub entries: Vec<IndexEntry>,
    /// Total matches
    pub total: usize,
    /// Search duration in milliseconds
    pub duration_ms: u64,
}

/// Dependency graph node
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyNode {
    /// File path
    pub path: String,
    /// Dependencies (files this depends on)
    pub depends_on: Vec<String>,
    /// Dependents (files that depend on this)
    pub depended_by: Vec<String>,
}

/// Engine statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextStats {
    pub total_files: usize,
    pub total_lines: usize,
    pub total_functions: usize,
    pub total_types: usize,
    pub languages: HashMap<String, usize>,
    pub last_indexed: Option<String>,
    pub index_size_bytes: u64,
}

// ============================================================================
// Context Engine
// ============================================================================

/// Real-time codebase indexing engine
pub struct ContextEngine {
    /// Indexed files
    index: Arc<RwLock<HashMap<String, IndexEntry>>>,
    /// Dependency graph
    dependencies: Arc<RwLock<HashMap<String, DependencyNode>>>,
    /// Workspace root path
    workspace_root: PathBuf,
    /// File extensions to index
    indexed_extensions: Vec<String>,
    /// Maximum file size to index (bytes)
    max_file_size: u64,
}

impl ContextEngine {
    /// Create a new context engine
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            index: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            workspace_root,
            indexed_extensions: vec![
                "rs".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
                "js".to_string(),
                "jsx".to_string(),
                "py".to_string(),
                "go".to_string(),
                "java".to_string(),
                "cpp".to_string(),
                "c".to_string(),
                "h".to_string(),
                "md".to_string(),
                "toml".to_string(),
                "yaml".to_string(),
                "json".to_string(),
            ],
            max_file_size: 1024 * 1024, // 1MB
        }
    }

    /// Index the entire workspace
    pub async fn index_workspace(&self) -> Result<usize, ContextError> {
        let mut count = 0;
        self.index_directory(&self.workspace_root.clone(), &mut count).await?;
        Ok(count)
    }

    /// Recursively index a directory
    fn index_directory<'a>(&'a self, dir: &'a Path, count: &'a mut usize) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ContextError>> + Send + 'a>> {
        Box::pin(async move {
            let mut entries = tokio::fs::read_dir(dir)
                .await
                .map_err(|e| ContextError::Io(e.to_string()))?;

            while let Some(entry) = entries.next_entry().await.map_err(|e| ContextError::Io(e.to_string()))? {
                let path = entry.path();

                // Skip hidden directories and target directories
                if let Some(name) = path.file_name() {
                    let name = name.to_string_lossy();
                    if name.starts_with('.') || name == "target" || name == "node_modules" {
                        continue;
                    }
                }

                if path.is_dir() {
                    self.index_directory(&path, count).await?;
                } else if path.is_file() {
                    if let Some(entry) = self.index_file(&path).await? {
                        let mut index = self.index.write().await;
                        index.insert(entry.path.clone(), entry);
                        *count += 1;
                    }
                }
            }

            Ok(())
        })
    }

    /// Index a single file
    async fn index_file(&self, path: &Path) -> Result<Option<IndexEntry>, ContextError> {
        // Check extension
        let extension = path.extension()
            .map(|e| e.to_string_lossy().to_string());

        let ext = match extension {
            Some(ext) => ext,
            None => return Ok(None),
        };

        if !self.indexed_extensions.contains(&ext) {
            return Ok(None);
        }

        // Check file size
        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|e| ContextError::Io(e.to_string()))?;

        if metadata.len() > self.max_file_size {
            return Ok(None);
        }

        // Read content
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ContextError::Io(e.to_string()))?;

        // Calculate content hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let content_hash = format!("{:x}", hasher.finish());

        // Parse content
        let lines = content.lines().count();
        let functions = self.extract_functions(&content, &ext);
        let types = self.extract_types(&content, &ext);
        let imports = self.extract_imports(&content, &ext);
        let exports = self.extract_exports(&content, &ext);

        // Get relative path
        let rel_path = path.strip_prefix(&self.workspace_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        // Detect language
        let language = match ext.as_str() {
            "rs" => Some("rust".to_string()),
            "ts" | "tsx" => Some("typescript".to_string()),
            "js" | "jsx" => Some("javascript".to_string()),
            "py" => Some("python".to_string()),
            "go" => Some("go".to_string()),
            "java" => Some("java".to_string()),
            "cpp" | "c" | "h" => Some("c".to_string()),
            "md" => Some("markdown".to_string()),
            "toml" | "yaml" | "json" => Some("config".to_string()),
            _ => None,
        };

        let last_modified = metadata.modified()
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                datetime.to_rfc3339()
            })
            .unwrap_or_default();

        Ok(Some(IndexEntry {
            path: rel_path,
            size: metadata.len(),
            extension: Some(ext),
            language,
            lines,
            imports,
            exports,
            functions,
            types,
            last_modified,
            content_hash,
            embedding: None, // TODO: Generate embeddings
        }))
    }

    /// Extract functions from content (simplified)
    fn extract_functions(&self, content: &str, ext: &str) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            match ext {
                "rs" => {
                    if trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") {
                        let name = trimmed
                            .split_whitespace()
                            .nth(2)
                            .unwrap_or("unknown")
                            .split('(')
                            .next()
                            .unwrap_or("unknown")
                            .to_string();

                        functions.push(FunctionInfo {
                            name,
                            signature: trimmed.to_string(),
                            start_line: line_num + 1,
                            end_line: line_num + 1, // Simplified
                            parameters: Vec::new(),
                            return_type: None,
                            is_public: trimmed.starts_with("pub fn"),
                        });
                    }
                }
                "ts" | "js" => {
                    if trimmed.starts_with("function ") || trimmed.starts_with("export function ") {
                        let name = trimmed
                            .split_whitespace()
                            .nth(1)
                            .unwrap_or("unknown")
                            .split('(')
                            .next()
                            .unwrap_or("unknown")
                            .to_string();

                        functions.push(FunctionInfo {
                            name,
                            signature: trimmed.to_string(),
                            start_line: line_num + 1,
                            end_line: line_num + 1,
                            parameters: Vec::new(),
                            return_type: None,
                            is_public: trimmed.starts_with("export"),
                        });
                    }
                }
                _ => {}
            }
        }

        functions
    }

    /// Extract types from content (simplified)
    fn extract_types(&self, content: &str, ext: &str) -> Vec<TypeInfo> {
        let mut types = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            match ext {
                "rs" => {
                    if trimmed.starts_with("pub struct ") || trimmed.starts_with("struct ") {
                        let name = trimmed
                            .split_whitespace()
                            .nth(2)
                            .unwrap_or("unknown")
                            .split('{')
                            .next()
                            .unwrap_or("unknown")
                            .trim()
                            .to_string();

                        types.push(TypeInfo {
                            name,
                            kind: "struct".to_string(),
                            start_line: line_num + 1,
                            end_line: line_num + 1,
                            members: Vec::new(),
                            is_public: trimmed.starts_with("pub struct"),
                        });
                    } else if trimmed.starts_with("pub enum ") || trimmed.starts_with("enum ") {
                        let name = trimmed
                            .split_whitespace()
                            .nth(2)
                            .unwrap_or("unknown")
                            .split('{')
                            .next()
                            .unwrap_or("unknown")
                            .trim()
                            .to_string();

                        types.push(TypeInfo {
                            name,
                            kind: "enum".to_string(),
                            start_line: line_num + 1,
                            end_line: line_num + 1,
                            members: Vec::new(),
                            is_public: trimmed.starts_with("pub enum"),
                        });
                    } else if trimmed.starts_with("pub trait ") || trimmed.starts_with("trait ") {
                        let name = trimmed
                            .split_whitespace()
                            .nth(2)
                            .unwrap_or("unknown")
                            .split('{')
                            .next()
                            .unwrap_or("unknown")
                            .trim()
                            .to_string();

                        types.push(TypeInfo {
                            name,
                            kind: "trait".to_string(),
                            start_line: line_num + 1,
                            end_line: line_num + 1,
                            members: Vec::new(),
                            is_public: trimmed.starts_with("pub trait"),
                        });
                    }
                }
                "ts" => {
                    if trimmed.starts_with("export interface ") || trimmed.starts_with("interface ") {
                        let name = trimmed
                            .split_whitespace()
                            .nth(2)
                            .unwrap_or("unknown")
                            .split('{')
                            .next()
                            .unwrap_or("unknown")
                            .trim()
                            .to_string();

                        types.push(TypeInfo {
                            name,
                            kind: "interface".to_string(),
                            start_line: line_num + 1,
                            end_line: line_num + 1,
                            members: Vec::new(),
                            is_public: trimmed.starts_with("export"),
                        });
                    } else if trimmed.starts_with("export type ") || trimmed.starts_with("type ") {
                        let name = trimmed
                            .split_whitespace()
                            .nth(2)
                            .unwrap_or("unknown")
                            .split('=')
                            .next()
                            .unwrap_or("unknown")
                            .trim()
                            .to_string();

                        types.push(TypeInfo {
                            name,
                            kind: "type".to_string(),
                            start_line: line_num + 1,
                            end_line: line_num + 1,
                            members: Vec::new(),
                            is_public: trimmed.starts_with("export"),
                        });
                    }
                }
                _ => {}
            }
        }

        types
    }

    /// Extract imports from content (simplified)
    fn extract_imports(&self, content: &str, ext: &str) -> Vec<String> {
        let mut imports = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            match ext {
                "rs" => {
                    if trimmed.starts_with("use ") {
                        let module = trimmed
                            .trim_start_matches("use ")
                            .trim_end_matches(';')
                            .to_string();
                        imports.push(module);
                    }
                }
                "ts" | "js" => {
                    if trimmed.starts_with("import ") {
                        let module = trimmed
                            .trim_start_matches("import ")
                            .split("from")
                            .last()
                            .unwrap_or("")
                            .trim()
                            .trim_matches('\'')
                            .trim_matches('"')
                            .to_string();
                        if !module.is_empty() {
                            imports.push(module);
                        }
                    }
                }
                "py" => {
                    if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                        let module = trimmed
                            .trim_start_matches("import ")
                            .trim_start_matches("from ")
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .to_string();
                        if !module.is_empty() {
                            imports.push(module);
                        }
                    }
                }
                _ => {}
            }
        }

        imports
    }

    /// Extract exports from content (simplified)
    fn extract_exports(&self, content: &str, ext: &str) -> Vec<String> {
        let mut exports = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            match ext {
                "rs" => {
                    if trimmed.starts_with("pub ") {
                        if let Some(name) = trimmed.split_whitespace().nth(2) {
                            let name = name.split('(').next().unwrap_or(name).to_string();
                            exports.push(name);
                        }
                    }
                }
                "ts" | "js" => {
                    if trimmed.starts_with("export ") {
                        if let Some(name) = trimmed.split_whitespace().nth(1) {
                            let name = name.split('(').next().unwrap_or(name).to_string();
                            exports.push(name);
                        }
                    }
                }
                _ => {}
            }
        }

        exports
    }

    /// Search the index
    pub async fn search(&self, query: &ContextQuery) -> ContextResult {
        let start = std::time::Instant::now();
        let index = self.index.read().await;

        let mut results: Vec<IndexEntry> = index.values()
            .filter(|entry| {
                // Filter by language
                if let Some(ref lang) = query.language {
                    if entry.language.as_ref() != Some(lang) {
                        return false;
                    }
                }

                // Filter by file pattern
                if let Some(ref pattern) = query.file_pattern {
                    if !entry.path.contains(pattern) {
                        return false;
                    }
                }

                // Text search
                let query_lower = query.text.to_lowercase();
                let path_match = entry.path.to_lowercase().contains(&query_lower);
                let function_match = entry.functions.iter().any(|f| 
                    f.name.to_lowercase().contains(&query_lower)
                );
                let type_match = entry.types.iter().any(|t| 
                    t.name.to_lowercase().contains(&query_lower)
                );

                path_match || function_match || type_match
            })
            .cloned()
            .collect();

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        let duration = start.elapsed().as_millis() as u64;

        ContextResult {
            total: results.len(),
            entries: results,
            duration_ms: duration,
        }
    }

    /// Get dependency graph for a file
    pub async fn get_dependencies(&self, path: &str) -> Option<DependencyNode> {
        let dependencies = self.dependencies.read().await;
        dependencies.get(path).cloned()
    }

    /// Get engine statistics
    pub async fn stats(&self) -> ContextStats {
        let index = self.index.read().await;
        let total_files = index.len();
        let total_lines: usize = index.values().map(|e| e.lines).sum();
        let total_functions: usize = index.values().map(|e| e.functions.len()).sum();
        let total_types: usize = index.values().map(|e| e.types.len()).sum();

        let mut languages = HashMap::new();
        for entry in index.values() {
            if let Some(ref lang) = entry.language {
                *languages.entry(lang.clone()).or_insert(0) += 1;
            }
        }

        let index_size: usize = index.values().map(|e| std::mem::size_of_val(e)).sum();

        ContextStats {
            total_files,
            total_lines,
            total_functions,
            total_types,
            languages,
            last_indexed: Some(chrono::Utc::now().to_rfc3339()),
            index_size_bytes: index_size as u64,
        }
    }

    /// Refresh index for a single file (for file watcher)
    pub async fn refresh_file(&self, path: &Path) -> Result<(), ContextError> {
        if let Some(entry) = self.index_file(path).await? {
            let mut index = self.index.write().await;
            index.insert(entry.path.clone(), entry);
        }
        Ok(())
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Context engine errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum ContextError {
    #[error("io error: {0}")]
    Io(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("index error: {0}")]
    IndexError(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_engine_creation() {
        let engine = ContextEngine::new(PathBuf::from("/tmp/test"));
        assert_eq!(engine.indexed_extensions.len(), 15);
    }

    #[tokio::test]
    async fn test_empty_stats() {
        let engine = ContextEngine::new(PathBuf::from("/tmp/test"));
        let stats = engine.stats().await;
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_lines, 0);
    }
}
