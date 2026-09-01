use crate::domain::{ActionSpec, DomainError, DomainPlugin, ParamSpec};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ========== Types ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub language: String,
    pub file_count: usize,
}

// ========== Plugin ==========

pub struct FilePlugin;

impl FilePlugin {
    /// 路径安全校验：最终路径必须在用户主目录内，且不包含 `..` 逃逸
    fn resolve_safe_path(path: &str) -> Result<PathBuf, DomainError> {
        let p = std::path::Path::new(path);
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        let home_path = std::path::Path::new(&home);

        let abs = if p.is_absolute() {
            p.to_path_buf()
        } else {
            home_path.join(p)
        };
        if !abs.starts_with(home_path) {
            return Err(DomainError { code: "PATH_ESCAPE".into(), message: format!("Path escapes home directory: {}", path), recoverable: true });
        }
        for comp in abs.components() {
            if matches!(comp, std::path::Component::ParentDir) {
                return Err(DomainError { code: "PATH_ESCAPE".into(), message: format!("Path traversal not allowed: {}", path), recoverable: true });
            }
        }
        Ok(abs)
    }

    fn read_dir_recursive(path: &str, max_depth: u32) -> Result<Vec<FileNode>, DomainError> {
        let safe = Self::resolve_safe_path(path)?;
        let mut out = Vec::new();
        Self::read_dir(&safe, 0, max_depth, &mut out);
        Ok(out)
    }

    fn read_dir(path: &std::path::Path, depth: u32, max_depth: u32, out: &mut Vec<FileNode>) {
        if depth > max_depth { return; }
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let file_path = entry.path().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                out.push(FileNode { name, path: file_path, is_dir, depth });
                if is_dir {
                    Self::read_dir(&entry.path(), depth + 1, max_depth, out);
                }
            }
        }
    }

    fn read_file(path: &str) -> Result<String, DomainError> {
        let safe = Self::resolve_safe_path(path)?;
        std::fs::read_to_string(&safe)
            .map_err(|e| DomainError { code: "IO_ERROR".into(), message: format!("读取文件失败: {}", e), recoverable: true })
    }

    fn write_file(path: &str, content: &str) -> Result<(), DomainError> {
        let safe = Self::resolve_safe_path(path)?;
        std::fs::write(&safe, content)
            .map_err(|e| DomainError { code: "IO_ERROR".into(), message: format!("写入文件失败: {}", e), recoverable: true })
    }

    fn count_files(path: &std::path::Path, depth: u32, max_depth: u32) -> usize {
        if depth > max_depth { return 0; }
        let mut count = 0;
        if path.is_dir() {
            for entry in std::fs::read_dir(path).into_iter().flatten() {
                if let Ok(entry) = entry {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        count += Self::count_files(&entry.path(), depth + 1, max_depth);
                    } else {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn detect_project_type(path: &std::path::Path) -> String {
        if path.join(".git").exists() { return "git".into(); }
        if path.join("Cargo.toml").exists() { return "rust".into(); }
        if path.join("package.json").exists() { return "node".into(); }
        if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() { return "python".into(); }
        if path.join("go.mod").exists() { return "go".into(); }
        if path.join("pom.xml").exists() || path.join("build.gradle").exists() { return "java".into(); }
        "folder".into()
    }

    fn detect_project(path: Option<&str>) -> Result<ProjectInfo, DomainError> {
        let p = path.unwrap_or(".");
        let pb = std::path::PathBuf::from(p);
        let abs = if pb.is_absolute() { pb.clone() } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(&pb)
        };
        let language = Self::detect_project_type(&abs);
        let file_count = Self::count_files(&abs, 0, 5);
        let name = abs.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| p.to_string());
        let path_str = abs.to_string_lossy().to_string();
        Ok(ProjectInfo { name, path: path_str, language, file_count })
    }

    fn search_files(query: &str, root: Option<&str>) -> Result<Vec<String>, DomainError> {
        let search_root = Self::resolve_safe_path(root.unwrap_or("."))?;
        let q = query.to_lowercase();
        let mut results = Vec::new();
        Self::search_recursive(&search_root, &q, 0, 5, &mut results);
        Ok(results)
    }

    fn search_recursive(path: &std::path::Path, query: &str, depth: u32, max_depth: u32, out: &mut Vec<String>) {
        if depth > max_depth { return; }
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.to_lowercase().contains(query) {
                    out.push(entry.path().to_string_lossy().to_string());
                }
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    // 跳过 node_modules, .git 等
                    if name == "node_modules" || name == ".git" || name == "target" { continue; }
                    Self::search_recursive(&entry.path(), query, depth + 1, max_depth, out);
                }
            }
        }
    }

    fn file_diff(path: &str) -> Result<String, DomainError> {
        let safe = Self::resolve_safe_path(path)?;
        let output = std::process::Command::new("git")
            .args(["diff", "HEAD", "--", &safe.to_string_lossy()])
            .output()
            .map_err(|e| DomainError { code: "GIT_ERROR".into(), message: format!("git diff 失败: {}", e), recoverable: true })?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl DomainPlugin for FilePlugin {
    fn name(&self) -> &str { "file" }
    fn description(&self) -> &str { "文件系统操作：读写、目录树、搜索、diff" }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec { name: "read".into(), description: "读取文件内容".into(),
                params: vec![ParamSpec { name: "path".into(), r#type: "string".into(), description: "文件路径".into(), optional: false }],
                returns: "string".into() },
            ActionSpec { name: "write".into(), description: "写入文件内容".into(),
                params: vec![
                    ParamSpec { name: "path".into(), r#type: "string".into(), description: "文件路径".into(), optional: false },
                    ParamSpec { name: "content".into(), r#type: "string".into(), description: "文件内容".into(), optional: false },
                ],
                returns: "void".into() },
            ActionSpec { name: "tree".into(), description: "列出目录树".into(),
                params: vec![
                    ParamSpec { name: "path".into(), r#type: "string".into(), description: "目录路径".into(), optional: true },
                    ParamSpec { name: "max_depth".into(), r#type: "number".into(), description: "最大深度".into(), optional: true },
                ],
                returns: "FileNode[]".into() },
            ActionSpec { name: "diff".into(), description: "显示文件 git diff".into(),
                params: vec![ParamSpec { name: "path".into(), r#type: "string".into(), description: "文件路径".into(), optional: false }],
                returns: "string".into() },
            ActionSpec { name: "search".into(), description: "搜索文件名".into(),
                params: vec![
                    ParamSpec { name: "query".into(), r#type: "string".into(), description: "搜索关键词".into(), optional: false },
                    ParamSpec { name: "root".into(), r#type: "string".into(), description: "搜索根目录".into(), optional: true },
                ],
                returns: "string[]".into() },
            ActionSpec { name: "detect_project".into(), description: "检测项目类型".into(),
                params: vec![ParamSpec { name: "path".into(), r#type: "string".into(), description: "项目路径".into(), optional: true }],
                returns: "ProjectInfo".into() },
        ]
    }

    fn call(&self, action: &str, args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        match action {
            "read" => {
                let path = args.get("path").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'path'".into(), recoverable: true })?;
                let content = Self::read_file(path)?;
                Ok(serde_json::json!(content))
            }
            "write" => {
                let path = args.get("path").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'path'".into(), recoverable: true })?;
                let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
                Self::write_file(path, content)?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "tree" => {
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                let max_depth = args.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(3) as u32;
                let nodes = Self::read_dir_recursive(path, max_depth)?;
                Ok(serde_json::to_value(nodes).unwrap_or_default())
            }
            "diff" => {
                let path = args.get("path").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'path'".into(), recoverable: true })?;
                let diff = Self::file_diff(path)?;
                Ok(serde_json::json!(diff))
            }
            "search" => {
                let query = args.get("query").and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "missing 'query'".into(), recoverable: true })?;
                let root = args.get("root").and_then(|v| v.as_str());
                let results = Self::search_files(query, root)?;
                Ok(serde_json::to_value(results).unwrap_or_default())
            }
            "detect_project" => {
                let path = args.get("path").and_then(|v| v.as_str());
                let info = Self::detect_project(path)?;
                Ok(serde_json::to_value(info).unwrap_or_default())
            }
            _ => Err(DomainError { code: "UNKNOWN_ACTION".into(), message: format!("Unknown action: {}", action), recoverable: true }),
        }
    }
}
