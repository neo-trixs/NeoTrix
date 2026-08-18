use crate::commands::types::{Project, ProjectChat, ProjectSource, ProjectInstruction};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use tauri::command;
use neotrix::neotrix::nt_core_error::NeoTrixError;

static PROJECTS: LazyLock<Mutex<HashMap<String, Project>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static PROJECT_CHATS: LazyLock<Mutex<HashMap<String, Vec<ProjectChat>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static PROJECT_SOURCES: LazyLock<Mutex<HashMap<String, Vec<ProjectSource>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static PROJECT_INSTRUCTIONS: LazyLock<Mutex<HashMap<String, Vec<ProjectInstruction>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

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

#[command]
pub fn project_get(id: String) -> Result<Option<Project>, String> {
    let projects = PROJECTS.lock().map_err(|e| e.to_string())?;
    Ok(projects.get(&id).cloned())
}

#[command]
pub fn project_update(
    id: String,
    name: Option<String>,
    description: Option<String>,
    color: Option<String>,
    icon: Option<String>,
    pinned: Option<bool>,
    archived: Option<bool>,
) -> Result<Project, String> {
    let mut projects = PROJECTS.lock().map_err(|e| e.to_string())?;
    let project = projects.get_mut(&id).ok_or("Project not found")?;
    if let Some(v) = name { project.name = v; }
    if let Some(v) = description { project.description = Some(v); }
    if let Some(v) = color { project.color = Some(v); }
    if let Some(v) = icon { project.icon = Some(v); }
    if let Some(v) = pinned { project.pinned = v; }
    if let Some(v) = archived { project.archived = v; }
    project.updated_at = now_ts();
    Ok(project.clone())
}

#[command]
pub fn project_delete(id: String) -> Result<(), NeoTrixError> {
    PROJECTS.lock().map_err(|e| e.to_string())?.remove(&id);
    PROJECT_CHATS.lock().map_err(|e| e.to_string())?.remove(&id);
    PROJECT_SOURCES.lock().map_err(|e| e.to_string())?.remove(&id);
    PROJECT_INSTRUCTIONS.lock().map_err(|e| e.to_string())?.remove(&id);
    Ok(())
}

#[command]
pub fn project_chat_list(project_id: String) -> Result<Vec<ProjectChat>, String> {
    let chats = PROJECT_CHATS.lock().map_err(|e| e.to_string())?;
    let mut list = chats.get(&project_id).cloned().unwrap_or_default();
    list.sort_by(|a, b| {
        (b.pinned as i32).cmp(&(a.pinned as i32))
            .then_with(|| b.updated_at.cmp(&a.updated_at))
    });
    Ok(list)
}

#[command]
pub fn project_chat_create(project_id: String, name: String, session_id: Option<String>) -> Result<ProjectChat, NeoTrixError> {
    let now = now_ts();
    let chat = ProjectChat {
        id: generate_id("chat"),
        project_id: project_id.clone(),
        name,
        session_id,
        message_count: 0,
        created_at: now,
        updated_at: now,
        pinned: false,
        archived: false,
    };
    PROJECT_CHATS.lock().map_err(|e| e.to_string())?
        .entry(project_id).or_default().push(chat.clone());
    Ok(chat)
}

#[command]
pub fn project_chat_update(
    chat_id: String,
    name: Option<String>,
    pinned: Option<bool>,
    archived: Option<bool>,
    message_count: Option<usize>,
) -> Result<ProjectChat, NeoTrixError> {
    let mut chats = PROJECT_CHATS.lock().map_err(|e| e.to_string())?;
    for list in chats.values_mut() {
        if let Some(chat) = list.iter_mut().find(|c| c.id == chat_id) {
            if let Some(v) = name { chat.name = v; }
            if let Some(v) = pinned { chat.pinned = v; }
            if let Some(v) = archived { chat.archived = v; }
            if let Some(v) = message_count { chat.message_count = v; }
            chat.updated_at = now_ts();
            return Ok(chat.clone());
        }
    }
    Err(NeoTrixError::Brain("Chat not found".to_string()))
}

#[command]
pub fn project_chat_delete(chat_id: String) -> Result<(), NeoTrixError> {
    let mut chats = PROJECT_CHATS.lock().map_err(|e| e.to_string())?;
    for list in chats.values_mut() {
        if let Some(pos) = list.iter().position(|c| c.id == chat_id) {
            list.remove(pos);
            return Ok(());
        }
    }
    Err(NeoTrixError::Brain("Chat not found".to_string()))
}

#[command]
pub fn project_source_list(project_id: String) -> Result<Vec<ProjectSource>, String> {
    let sources = PROJECT_SOURCES.lock().map_err(|e| e.to_string())?;
    Ok(sources.get(&project_id).cloned().unwrap_or_default())
}

#[command]
pub fn project_source_add(
    project_id: String,
    source_type: String,
    path: Option<String>,
    url: Option<String>,
    name: String,
) -> Result<ProjectSource, NeoTrixError> {
    let source = ProjectSource {
        id: generate_id("src"),
        project_id: project_id.clone(),
        source_type,
        path,
        url,
        name,
        enabled: true,
        created_at: now_ts(),
    };
    PROJECT_SOURCES.lock().map_err(|e| e.to_string())?
        .entry(project_id).or_default().push(source.clone());
    Ok(source)
}

#[command]
pub fn project_source_update(
    source_id: String,
    enabled: Option<bool>,
    name: Option<String>,
) -> Result<ProjectSource, NeoTrixError> {
    let mut sources = PROJECT_SOURCES.lock().map_err(|e| e.to_string())?;
    for list in sources.values_mut() {
        if let Some(src) = list.iter_mut().find(|s| s.id == source_id) {
            if let Some(v) = enabled { src.enabled = v; }
            if let Some(v) = name { src.name = v; }
            return Ok(src.clone());
        }
    }
    Err(NeoTrixError::Brain("Source not found".to_string()))
}

#[command]
pub fn project_source_delete(source_id: String) -> Result<(), NeoTrixError> {
    let mut sources = PROJECT_SOURCES.lock().map_err(|e| e.to_string())?;
    for list in sources.values_mut() {
        if let Some(pos) = list.iter().position(|s| s.id == source_id) {
            list.remove(pos);
            return Ok(());
        }
    }
    Err(NeoTrixError::Brain("Source not found".to_string()))
}

#[command]
pub fn project_instruction_list(project_id: String) -> Result<Vec<ProjectInstruction>, String> {
    let instructions = PROJECT_INSTRUCTIONS.lock().map_err(|e| e.to_string())?;
    Ok(instructions.get(&project_id).cloned().unwrap_or_default())
}

#[command]
pub fn project_instruction_add(project_id: String, content: String) -> Result<ProjectInstruction, NeoTrixError> {
    let instruction = ProjectInstruction {
        id: generate_id("ins"),
        project_id: project_id.clone(),
        content,
        enabled: true,
        created_at: now_ts(),
        updated_at: now_ts(),
    };
    PROJECT_INSTRUCTIONS.lock().map_err(|e| e.to_string())?
        .entry(project_id).or_default().push(instruction.clone());
    Ok(instruction)
}

#[command]
pub fn project_instruction_update(
    instruction_id: String,
    content: Option<String>,
    enabled: Option<bool>,
) -> Result<ProjectInstruction, NeoTrixError> {
    let mut instructions = PROJECT_INSTRUCTIONS.lock().map_err(|e| e.to_string())?;
    for list in instructions.values_mut() {
        if let Some(ins) = list.iter_mut().find(|i| i.id == instruction_id) {
            if let Some(v) = content { ins.content = v; }
            if let Some(v) = enabled { ins.enabled = v; }
            ins.updated_at = now_ts();
            return Ok(ins.clone());
        }
    }
    Err(NeoTrixError::Brain("Instruction not found".to_string()))
}

#[command]
pub fn project_instruction_delete(instruction_id: String) -> Result<(), NeoTrixError> {
    let mut instructions = PROJECT_INSTRUCTIONS.lock().map_err(|e| e.to_string())?;
    for list in instructions.values_mut() {
        if let Some(pos) = list.iter().position(|i| i.id == instruction_id) {
            list.remove(pos);
            return Ok(());
        }
    }
    Err(NeoTrixError::Brain("Instruction not found".to_string()))
}

#[command]
pub fn project_scan_directory(path: String) -> Result<Project, String> {
    let path_buf = std::path::PathBuf::from(&path);
    if !path_buf.exists() {
        return Err("Path does not exist".to_string());
    }
    let name = path_buf.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();
    let project_type = detect_project_type(&path_buf);
    project_create(name, path, Some(project_type), None, None, None)
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

#[command]
pub fn cmd_project_open(path: String) -> Result<ProjectInfo, NeoTrixError> {
    detect_project(Some(path))
}

#[command]
pub fn cmd_scan_files(path: String) -> Result<Vec<FlatFileNode>, NeoTrixError> {
    read_dir_recursive(path, Some(1))
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