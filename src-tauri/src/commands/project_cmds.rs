use crate::commands::types::Project;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use tauri::command;
use neotrix::neotrix::nt_core_error::NeoTrixError;

static PROJECTS: LazyLock<Mutex<HashMap<String, Project>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn generate_id(prefix: &str) -> String {
    format!("{}-{}", prefix, uuid::Uuid::new_v4().simple())
}

fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}

#[command]
pub fn project_list() -> Result<Vec<Project>, String> {
    let projects = PROJECTS.lock().map_err(|e| e.to_string())?;
    let mut list: Vec<Project> = projects.values().cloned().collect();
    list.sort_by(|a, b| {
        (b.pinned as i32).cmp(&(a.pinned as i32))
            .then_with(|| b.updated_at.cmp(&a.updated_at))
    });
    Ok(list)
}

#[command]
pub fn project_create(
    name: String,
    path: String,
    project_type: Option<String>,
    description: Option<String>,
    color: Option<String>,
    icon: Option<String>,
) -> Result<Project, String> {
    let now = now_ts();
    let project = Project {
        id: generate_id("proj"),
        name,
        path,
        project_type: project_type.unwrap_or_else(|| "local".to_string()),
        description,
        created_at: now,
        updated_at: now,
        pinned: false,
        archived: false,
        color,
        icon,
    };
    PROJECTS.lock().map_err(|e| e.to_string())?.insert(project.id.clone(), project.clone());
    Ok(project)
}

use crate::commands::types::{FlatFileNode, ProjectInfo};

/// 路径安全校验：最终路径必须在用户主目录内，且不包含 `..` 逃逸。
/// 与 capabilities 的 fs scope（$HOME/**）保持一致，防止前端任意文件读写。
///
/// 注意：绝对路径（如前端文件对话框返回的 `/Users/neo/...`）是合法的，
/// 只要其落在主目录内——不能因含 RootDir 组件就整体拒绝（否则功能回归）。
pub(crate) fn resolve_safe_path(path: &str) -> Result<std::path::PathBuf, NeoTrixError> {
    let p = std::path::Path::new(path);
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
    let home_path = std::path::Path::new(&home);

    // 相对路径锚定到主目录
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        home_path.join(p)
    };
    // 最终路径必须落在主目录内（词法前缀检查）
    if !abs.starts_with(home_path) {
        return Err(NeoTrixError::Brain(format!(
            "Path escapes home directory: {}",
            path
        )));
    }
    // 拒绝路径穿越组件（`..` 逃逸）。RootDir/Prefix 本身合法（绝对路径），
    // 但 ParentDir 在任何位置都是逃逸信号。
    for comp in abs.components() {
        if matches!(comp, std::path::Component::ParentDir) {
            return Err(NeoTrixError::Brain(format!(
                "Invalid path (traversal not allowed): {}",
                path
            )));
        }
    }
    Ok(abs)
}

#[command]
pub fn read_dir_recursive(path: String, max_depth: Option<u32>) -> Result<Vec<FlatFileNode>, NeoTrixError> {
    fn read_dir(path: &std::path::Path, depth: u32, max_depth: u32, out: &mut Vec<FlatFileNode>) {
        if depth > max_depth { return; }
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let file_path = entry.path().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                out.push(FlatFileNode { name, path: file_path, is_dir, depth });
                if is_dir {
                    read_dir(&entry.path(), depth + 1, max_depth, out);
                }
            }
        }
    }
    let safe = resolve_safe_path(&path)?;
    let mut out = Vec::new();
    read_dir(&safe, 0, max_depth.unwrap_or(3), &mut out);
    Ok(out)
}

#[command]
pub fn read_file(path: String) -> Result<String, NeoTrixError> {
    let safe = resolve_safe_path(&path)?;
    std::fs::read_to_string(&safe).map_err(|e| NeoTrixError::Brain(e.to_string()))
}

#[command]
pub fn write_file(path: String, content: String) -> Result<(), NeoTrixError> {
    let safe = resolve_safe_path(&path)?;
    std::fs::write(&safe, &content).map_err(|e| NeoTrixError::Brain(e.to_string()))
}

#[command]
pub fn detect_project(path: Option<String>) -> Result<ProjectInfo, NeoTrixError> {
    let path = path.unwrap_or_else(|| {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string())
    });
    let pb = std::path::PathBuf::from(&path);
    let language = detect_project_type(&pb);
    let file_count = count_files(&pb, 0, 5).unwrap_or(0);
    let name = pb.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or(path.clone());
    Ok(ProjectInfo { name, path, language, file_count })
}

fn count_files(path: &std::path::Path, depth: u32, max_depth: u32) -> std::io::Result<usize> {
    if depth > max_depth { return Ok(0); }
    let mut count = 0;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                count += count_files(&entry.path(), depth + 1, max_depth)?;
            } else {
                count += 1;
            }
        }
    }
    Ok(count)
}

fn detect_project_type(path: &std::path::Path) -> String {
    if path.join(".git").exists() { return "git".to_string(); }
    if path.join("Cargo.toml").exists() { return "rust".to_string(); }
    if path.join("package.json").exists() { return "node".to_string(); }
    if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() { return "python".to_string(); }
    if path.join("go.mod").exists() { return "go".to_string(); }
    if path.join("pom.xml").exists() || path.join("build.gradle").exists() { return "java".to_string(); }
    "folder".to_string()
}
/* ══════════════════════════════════════════
   W1 文件上传: doc-parse 管线 Tauri 暴露
   (复用 nt_file_ability::doc_parse, R-P42 零平行解析器)
   ══════════════════════════════════════════ */

#[derive(serde::Serialize)]
pub struct ParsedDocFile {
    pub path: String,
    pub title: String,
    pub format: String,
    pub text: String,
    pub tables: Option<serde_json::Value>,
}

/// 解析本地文档为统一 FileModel 文本 (md/txt/pdf/docx/xlsx…)。
/// 供对话附件与「存入知识库」联动 (kb_doc_ingest 直接吃 text)。
#[command]
pub fn parse_doc_file(path: String) -> Result<ParsedDocFile, NeoTrixError> {
    use neotrix::neotrix::nt_file_ability::doc_parse::parse_document;
    let model = parse_document(std::path::Path::new(&path))
        .map_err(|e| NeoTrixError::Memory(format!("parse {}: {:?}", path, e)))?;
    let title = model
        .title
        .clone()
        .unwrap_or_else(|| std::path::Path::new(&path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default());
    Ok(ParsedDocFile {
        path,
        title,
        format: model.format.clone(),
        text: model.content.clone(),
        tables: model.tables.clone(),
    })
}
