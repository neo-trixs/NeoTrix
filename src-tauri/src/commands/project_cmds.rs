use crate::commands::types::{Project, ProjectChat, ProjectSource, ProjectInstruction};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use tauri::command;

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
pub fn project_delete(id: String) -> Result<(), String> {
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
pub fn project_chat_create(project_id: String, name: String, session_id: Option<String>) -> Result<ProjectChat, String> {
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
) -> Result<ProjectChat, String> {
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
    Err("Chat not found".to_string())
}

#[command]
pub fn project_chat_delete(chat_id: String) -> Result<(), String> {
    let mut chats = PROJECT_CHATS.lock().map_err(|e| e.to_string())?;
    for list in chats.values_mut() {
        if let Some(pos) = list.iter().position(|c| c.id == chat_id) {
            list.remove(pos);
            return Ok(());
        }
    }
    Err("Chat not found".to_string())
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
) -> Result<ProjectSource, String> {
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
) -> Result<ProjectSource, String> {
    let mut sources = PROJECT_SOURCES.lock().map_err(|e| e.to_string())?;
    for list in sources.values_mut() {
        if let Some(src) = list.iter_mut().find(|s| s.id == source_id) {
            if let Some(v) = enabled { src.enabled = v; }
            if let Some(v) = name { src.name = v; }
            return Ok(src.clone());
        }
    }
    Err("Source not found".to_string())
}

#[command]
pub fn project_source_delete(source_id: String) -> Result<(), String> {
    let mut sources = PROJECT_SOURCES.lock().map_err(|e| e.to_string())?;
    for list in sources.values_mut() {
        if let Some(pos) = list.iter().position(|s| s.id == source_id) {
            list.remove(pos);
            return Ok(());
        }
    }
    Err("Source not found".to_string())
}

#[command]
pub fn project_instruction_list(project_id: String) -> Result<Vec<ProjectInstruction>, String> {
    let instructions = PROJECT_INSTRUCTIONS.lock().map_err(|e| e.to_string())?;
    Ok(instructions.get(&project_id).cloned().unwrap_or_default())
}

#[command]
pub fn project_instruction_add(project_id: String, content: String) -> Result<ProjectInstruction, String> {
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
) -> Result<ProjectInstruction, String> {
    let mut instructions = PROJECT_INSTRUCTIONS.lock().map_err(|e| e.to_string())?;
    for list in instructions.values_mut() {
        if let Some(ins) = list.iter_mut().find(|i| i.id == instruction_id) {
            if let Some(v) = content { ins.content = v; }
            if let Some(v) = enabled { ins.enabled = v; }
            ins.updated_at = now_ts();
            return Ok(ins.clone());
        }
    }
    Err("Instruction not found".to_string())
}

#[command]
pub fn project_instruction_delete(instruction_id: String) -> Result<(), String> {
    let mut instructions = PROJECT_INSTRUCTIONS.lock().map_err(|e| e.to_string())?;
    for list in instructions.values_mut() {
        if let Some(pos) = list.iter().position(|i| i.id == instruction_id) {
            list.remove(pos);
            return Ok(());
        }
    }
    Err("Instruction not found".to_string())
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

fn detect_project_type(path: &std::path::Path) -> String {
    if path.join(".git").exists() { return "git".to_string(); }
    if path.join("Cargo.toml").exists() { return "rust".to_string(); }
    if path.join("package.json").exists() { return "node".to_string(); }
    if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() { return "python".to_string(); }
    if path.join("go.mod").exists() { return "go".to_string(); }
    if path.join("pom.xml").exists() || path.join("build.gradle").exists() { return "java".to_string(); }
    "folder".to_string()
}