use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use tauri::command;

// ============================================================================
// Structs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoworkSession {
    pub id: String,
    pub name: String,
    pub workspace_path: String,
    pub status: String,
    pub files_read: u32,
    pub files_created: u32,
    pub files_modified: u32,
    pub started_at: i64,
    pub last_active_at: i64,
    pub deliverables: Vec<String>,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoworkFile {
    pub path: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub kind: String,
    pub last_modified: i64,
    pub content_summary: String,
    pub is_deliverable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoworkAction {
    pub id: String,
    pub session_id: String,
    pub action_type: String,
    pub target_path: String,
    pub status: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub details: Option<String>,
    pub result_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoworkDeliverable {
    pub id: String,
    pub session_id: String,
    pub name: String,
    pub path: String,
    pub kind: String,
    pub created_at: i64,
    pub size_bytes: u64,
    pub description: String,
    pub quality_score: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoworkTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub steps: Vec<CoworkTemplateStep>,
    pub suggested_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoworkTemplateStep {
    pub order: u32,
    pub action: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoworkConfig {
    pub enabled: bool,
    pub max_files_per_scan: u32,
    pub max_file_size_kb: u32,
    pub auto_save: bool,
    pub deliverable_formats: Vec<String>,
    pub allow_file_create: bool,
    pub allow_file_modify: bool,
    pub allow_file_delete: bool,
}

impl Default for CoworkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_files_per_scan: 500,
            max_file_size_kb: 1024,
            auto_save: true,
            deliverable_formats: vec![
                "md".into(), "txt".into(), "json".into(), "yaml".into(),
                "csv".into(), "html".into(), "pdf".into(),
            ],
            allow_file_create: true,
            allow_file_modify: true,
            allow_file_delete: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoworkStats {
    pub total_sessions: u32,
    pub total_deliverables: u32,
    pub files_processed: u32,
    pub active_sessions: u32,
    pub avg_files_per_session: f64,
    pub top_category: String,
    pub top_template: String,
}

// ============================================================================
// State
// ============================================================================

struct CoworkState {
    sessions: HashMap<String, CoworkSession>,
    actions: HashMap<String, Vec<CoworkAction>>,
    deliverables: HashMap<String, Vec<CoworkDeliverable>>,
    config: CoworkConfig,
}

impl CoworkState {
    fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            actions: HashMap::new(),
            deliverables: HashMap::new(),
            config: CoworkConfig::default(),
        }
    }
}

static COWORK: std::sync::LazyLock<Mutex<CoworkState>> =
    std::sync::LazyLock::new(|| Mutex::new(CoworkState::new()));

const MAX_SESSIONS: usize = 50;

// ============================================================================
// Helpers
// ============================================================================

fn short_uid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let hash = (nanos as u64).wrapping_mul(6364136223846793005).wrapping_add(1);
    format!("{:08x}", hash)[..8].to_string()
}

fn now_ts() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn find_session(id: &str) -> Result<CoworkSession, String> {
    let state = COWORK.lock().map_err(|e| e.to_string())?;
    state.sessions.get(id).cloned().ok_or_else(|| format!("Session not found: {}", id))
}

/// 将用户提供的路径解析到工作区内：拒绝绝对路径、`..` 逃逸，且最终路径必须在 workspace 内。
fn resolve_workspace_path(session: &CoworkSession, path: &str) -> Result<std::path::PathBuf, String> {
    let workspace = std::path::Path::new(&session.workspace_path);
    let p = std::path::Path::new(path);
    if p.is_absolute() {
        let has_parent = p.components().any(|c| matches!(c, std::path::Component::ParentDir));
        if has_parent {
            return Err("Invalid path".into());
        }
        if p.starts_with(workspace) {
            return Ok(p.to_path_buf());
        }
        return Err("Absolute path escapes workspace".into());
    }
    let mut clean = std::path::PathBuf::new();
    for comp in p.components() {
        match comp {
            std::path::Component::Normal(c) => clean.push(c),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir | std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err("Invalid path".into());
            }
        }
    }
    if clean.as_os_str().is_empty() {
        return Err("Empty path".into());
    }
    let joined = workspace.join(clean);
    if joined.starts_with(workspace) {
        Ok(joined)
    } else {
        Err("Path escapes workspace".into())
    }
}

fn default_templates() -> Vec<CoworkTemplate> {
    vec![
        CoworkTemplate {
            id: "tpl-doc-report".into(),
            name: "Document Report".into(),
            description: "Generate a comprehensive document report from workspace files".into(),
            category: "writing".into(),
            steps: vec![
                CoworkTemplateStep { order: 1, action: "scan".into(), description: "Scan workspace for relevant files".into() },
                CoworkTemplateStep { order: 2, action: "analyze".into(), description: "Analyze file content and structure".into() },
                CoworkTemplateStep { order: 3, action: "create".into(), description: "Generate the report document".into() },
            ],
            suggested_prompt: "Scan the workspace and generate a comprehensive document report covering all files and their relationships.".into(),
        },
        CoworkTemplate {
            id: "tpl-code-review".into(),
            name: "Code Review".into(),
            description: "Perform a structured code review on source files".into(),
            category: "code".into(),
            steps: vec![
                CoworkTemplateStep { order: 1, action: "scan".into(), description: "Scan source files matching code patterns".into() },
                CoworkTemplateStep { order: 2, action: "analyze".into(), description: "Analyze code quality, patterns, and potential issues".into() },
                CoworkTemplateStep { order: 3, action: "review".into(), description: "Generate detailed code review findings".into() },
                CoworkTemplateStep { order: 4, action: "create".into(), description: "Write the code review report".into() },
            ],
            suggested_prompt: "Review the source code in this workspace. Analyze code quality, identify potential bugs, and suggest improvements.".into(),
        },
        CoworkTemplate {
            id: "tpl-translation".into(),
            name: "Translation".into(),
            description: "Translate documents or content to a target language".into(),
            category: "translation".into(),
            steps: vec![
                CoworkTemplateStep { order: 1, action: "read".into(), description: "Read source documents for translation".into() },
                CoworkTemplateStep { order: 2, action: "translate".into(), description: "Translate content to target language".into() },
                CoworkTemplateStep { order: 3, action: "create".into(), description: "Write the translated output file".into() },
            ],
            suggested_prompt: "Read the files in the workspace and translate their content to the specified target language.".into(),
        },
        CoworkTemplate {
            id: "tpl-data-analysis".into(),
            name: "Data Analysis".into(),
            description: "Analyze data files and generate insights".into(),
            category: "data".into(),
            steps: vec![
                CoworkTemplateStep { order: 1, action: "scan".into(), description: "Scan for data files (CSV, JSON, etc.)".into() },
                CoworkTemplateStep { order: 2, action: "read".into(), description: "Read and parse data files".into() },
                CoworkTemplateStep { order: 3, action: "analyze".into(), description: "Analyze data and compute statistics".into() },
                CoworkTemplateStep { order: 4, action: "create".into(), description: "Generate analysis report and visualizations".into() },
            ],
            suggested_prompt: "Scan the workspace for data files, analyze their content, and produce a comprehensive data analysis report.".into(),
        },
        CoworkTemplate {
            id: "tpl-research-summary".into(),
            name: "Research Summary".into(),
            description: "Consolidate research materials into a summary".into(),
            category: "research".into(),
            steps: vec![
                CoworkTemplateStep { order: 1, action: "scan".into(), description: "Scan for research documents and notes".into() },
                CoworkTemplateStep { order: 2, action: "read".into(), description: "Read and extract key findings".into() },
                CoworkTemplateStep { order: 3, action: "create".into(), description: "Generate consolidated research summary".into() },
            ],
            suggested_prompt: "Scan the workspace for research materials, extract key findings, and produce a structured research summary.".into(),
        },
        CoworkTemplate {
            id: "tpl-api-docs".into(),
            name: "Api Documentation".into(),
            description: "Generate API documentation from source code".into(),
            category: "writing".into(),
            steps: vec![
                CoworkTemplateStep { order: 1, action: "scan".into(), description: "Scan for API source files".into() },
                CoworkTemplateStep { order: 2, action: "analyze".into(), description: "Extract API endpoints and signatures".into() },
                CoworkTemplateStep { order: 3, action: "create".into(), description: "Generate API documentation file".into() },
            ],
            suggested_prompt: "Scan source files for API definitions, extract endpoints and parameters, and generate comprehensive API documentation.".into(),
        },
    ]
}

// ============================================================================
// Commands
// ============================================================================

#[command]
pub fn cowork_start(
    workspace_path: String,
    description: String,
    name: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<String, String> {
    let session_name = name.unwrap_or_else(|| {
        let parts: Vec<&str> = workspace_path.trim_end_matches('/').split('/').collect();
        parts.last().unwrap_or(&"workspace").to_string()
    });
    let id = format!("cw-{}", short_uid());
    let now = now_ts();
    let session = CoworkSession {
        id: id.clone(),
        name: session_name,
        workspace_path,
        status: "active".into(),
        files_read: 0,
        files_created: 0,
        files_modified: 0,
        started_at: now,
        last_active_at: now,
        deliverables: Vec::new(),
        description,
        tags: tags.unwrap_or_default(),
    };

    let mut state = COWORK.lock().map_err(|e| e.to_string())?;
    if state.sessions.len() >= MAX_SESSIONS {
        let oldest_key = state.sessions.keys().next().cloned();
        if let Some(key) = oldest_key {
            state.sessions.remove(&key);
            state.actions.remove(&key);
            state.deliverables.remove(&key);
        }
    }
    state.sessions.insert(id.clone(), session);
    state.actions.insert(id.clone(), Vec::new());
    state.deliverables.insert(id.clone(), Vec::new());
    Ok(id)
}

#[command]
pub fn cowork_list() -> Result<Vec<CoworkSession>, String> {
    let state = COWORK.lock().map_err(|e| e.to_string())?;
    let mut sessions: Vec<CoworkSession> = state.sessions.values().cloned().collect();
    sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    Ok(sessions)
}

#[command]
pub fn cowork_get(session_id: String) -> Result<CoworkSession, String> {
    find_session(&session_id)
}

#[command]
pub fn cowork_status(session_id: String) -> Result<CoworkSession, String> {
    let mut state = COWORK.lock().map_err(|e| e.to_string())?;
    let session = state.sessions.get_mut(&session_id).ok_or_else(|| format!("Session not found: {}", session_id))?;
    if session.status == "active" {
        session.files_read = session.files_read.saturating_add(1);
        session.last_active_at = now_ts();
    }
    Ok(session.clone())
}

#[command]
pub fn cowork_pause(session_id: String) -> Result<(), String> {
    let mut state = COWORK.lock().map_err(|e| e.to_string())?;
    let session = state.sessions.get_mut(&session_id).ok_or_else(|| format!("Session not found: {}", session_id))?;
    session.status = "paused".into();
    session.last_active_at = now_ts();
    Ok(())
}

#[command]
pub fn cowork_resume(session_id: String) -> Result<(), String> {
    let mut state = COWORK.lock().map_err(|e| e.to_string())?;
    let session = state.sessions.get_mut(&session_id).ok_or_else(|| format!("Session not found: {}", session_id))?;
    session.status = "active".into();
    session.last_active_at = now_ts();
    Ok(())
}

#[command]
pub fn cowork_stop(session_id: String) -> Result<CoworkSession, String> {
    let mut state = COWORK.lock().map_err(|e| e.to_string())?;
    let session = state.sessions.get_mut(&session_id).ok_or_else(|| format!("Session not found: {}", session_id))?;
    session.status = "completed".into();
    session.last_active_at = now_ts();
    Ok(session.clone())
}

#[command]
pub fn cowork_delete(session_id: String) -> Result<(), String> {
    let mut state = COWORK.lock().map_err(|e| e.to_string())?;
    if state.sessions.remove(&session_id).is_none() {
        return Err(format!("Session not found: {}", session_id));
    }
    // 级联清理该会话的行动记录与交付物索引
    state.actions.remove(&session_id);
    state.deliverables.remove(&session_id);
    Ok(())
}

#[command]
pub fn cowork_scan_files(session_id: String, pattern: Option<String>) -> Result<Vec<CoworkFile>, String> {
    let session = find_session(&session_id)?;
    let mut files = Vec::new();

    let dir = std::path::Path::new(&session.workspace_path);
    if !dir.is_dir() {
        return Err(format!("Workspace path is not a directory: {}", session.workspace_path));
    }

    let max_files = {
        let state = COWORK.lock().map_err(|e| e.to_string())?;
        state.config.max_files_per_scan as usize
    };

    let glob_pattern = pattern.unwrap_or_else(|| "**/*".into());
    // 防 glob 逃逸: pattern 含 `/` 开头、`..` 等越界组件时拒绝
    if glob_pattern.starts_with('/')
        || glob_pattern.split('/').any(|c| c == "..")
    {
        return Err("Pattern escapes workspace".into());
    }
    let full_pattern = format!("{}/{}", session.workspace_path.trim_end_matches('/'), glob_pattern);

    if let Ok(entries) = glob::glob(&full_pattern) {
        for (i, entry) in entries.flatten().enumerate() {
            if i >= max_files {
                break;
            }
            if let Ok(meta) = std::fs::metadata(&entry) {
                if meta.is_dir() {
                    continue;
                }
                let abs_path = entry.to_string_lossy().to_string();
                let rel_path = abs_path
                    .strip_prefix(&format!("{}/", session.workspace_path.trim_end_matches('/')))
                    .unwrap_or(&abs_path)
                    .to_string();
                let ext = std::path::Path::new(&abs_path)
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let kind = match ext.as_str() {
                    "rs" | "py" | "js" | "ts" | "go" | "java" | "cpp" | "c" | "h" | "hpp" | "rb" | "swift" | "kt" | "scala" => "source",
                    "md" | "txt" | "rst" | "adoc" | "tex" | "pdf" | "doc" | "docx" => "document",
                    "json" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" => "config",
                    "csv" | "tsv" | "xls" | "xlsx" | "xml" | "sqlite" | "db" => "data",
                    "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "bmp" => "image",
                    _ => "other",
                };
                let modified = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let content_summary = if meta.len() < 1024 * 100 {
                    std::fs::read_to_string(&abs_path)
                        .ok()
                        .map(|s| {
                            let trimmed = s.trim();
                            if trimmed.len() > 200 {
                                let end = trimmed.floor_char_boundary(200);
                                format!("{}...", &trimmed[..end])
                            } else {
                                trimmed.to_string()
                            }
                        })
                        .unwrap_or_default()
                } else {
                    format!("[{} bytes, preview suppressed]", meta.len())
                };

                files.push(CoworkFile {
                    path: abs_path,
                    relative_path: rel_path,
                    size_bytes: meta.len(),
                    kind: kind.into(),
                    last_modified: modified,
                    content_summary,
                    is_deliverable: false,
                });
            }
        }
    }

    let mut state = COWORK.lock().map_err(|e| e.to_string())?;
    if let Some(session) = state.sessions.get_mut(&session_id) {
        session.files_read = session.files_read.saturating_add(files.len() as u32);
        session.last_active_at = now_ts();
    }

    Ok(files)
}

#[command]
pub fn cowork_read_file(session_id: String, path: String) -> Result<String, String> {
    let session = find_session(&session_id)?;
    let abs_path = resolve_workspace_path(&session, &path)?;
    let content = std::fs::read_to_string(&abs_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mut state = COWORK.lock().map_err(|e| e.to_string())?;
    if let Some(session) = state.sessions.get_mut(&session_id) {
        session.files_read = session.files_read.saturating_add(1);
        session.last_active_at = now_ts();
    }

    Ok(content)
}

#[command]
pub fn cowork_write_file(session_id: String, path: String, content: String) -> Result<(), String> {
    let session = find_session(&session_id)?;
    let allowed = {
        let state = COWORK.lock().map_err(|e| e.to_string())?;
        state.config.allow_file_create || state.config.allow_file_modify
    };
    if !allowed {
        return Err("File creation/modification is disabled by cowork config".into());
    }
    let abs_path = resolve_workspace_path(&session, &path)?;

    let parent = std::path::Path::new(&abs_path)
        .parent()
        .ok_or_else(|| format!("Invalid workspace path (no parent): {}", abs_path.display()))?;
    std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    std::fs::write(&abs_path, &content).map_err(|e| format!("Failed to write file: {}", e))?;

    let mut state = COWORK.lock().map_err(|e| e.to_string())?;
    if let Some(session) = state.sessions.get_mut(&session_id) {
        session.files_created = session.files_created.saturating_add(1);
        session.last_active_at = now_ts();
    }

    let meta = std::fs::metadata(&abs_path).ok();
    let del_id = format!("del-{}", short_uid());
    let deliverable = CoworkDeliverable {
        id: del_id.clone(),
        session_id: session_id.clone(),
        name: std::path::Path::new(&abs_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".into()),
        path: abs_path.to_string_lossy().to_string(),
        kind: {
            let ext = std::path::Path::new(&path)
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            match ext.as_str() {
                "md" | "txt" | "rst" | "adoc" => "document",
                "json" | "csv" | "xml" | "yaml" | "yml" => "data",
                "rs" | "py" | "js" | "ts" | "go" => "code",
                "pdf" | "html" => "report",
                _ => "document",
            }
            .into()
        },
        created_at: now_ts(),
        size_bytes: meta.map(|m| m.len()).unwrap_or(0),
        description: format!("Deliverable created by cowork session {}", session_id),
        quality_score: None,
    };

    if let Some(session) = state.sessions.get_mut(&session_id) {
        session.deliverables.push(del_id);
    }
    if let Some(dels) = state.deliverables.get_mut(&session_id) {
        dels.push(deliverable);
    }

    Ok(())
}

#[command]
pub fn cowork_delete_file(session_id: String, path: String) -> Result<(), String> {
    let session = find_session(&session_id)?;
    let allowed = {
        let state = COWORK.lock().map_err(|e| e.to_string())?;
        state.config.allow_file_delete
    };
    if !allowed {
        return Err("File deletion is disabled by cowork config".into());
    }
    let abs_path = resolve_workspace_path(&session, &path)?;
    std::fs::remove_file(&abs_path).map_err(|e| format!("Failed to delete file: {}", e))?;

    let mut state = COWORK.lock().map_err(|e| e.to_string())?;
    if let Some(session) = state.sessions.get_mut(&session_id) {
        session.files_modified = session.files_modified.saturating_add(1);
        session.last_active_at = now_ts();
    }

    Ok(())
}

#[command]
pub fn cowork_list_deliverables(session_id: String) -> Result<Vec<CoworkDeliverable>, String> {
    let state = COWORK.lock().map_err(|e| e.to_string())?;
    state.deliverables.get(&session_id).cloned().ok_or_else(|| format!("Session not found: {}", session_id))
}

#[command]
pub fn cowork_get_deliverable(deliverable_id: String) -> Result<CoworkDeliverable, String> {
    let state = COWORK.lock().map_err(|e| e.to_string())?;
    for dels in state.deliverables.values() {
        if let Some(d) = dels.iter().find(|d| d.id == deliverable_id) {
            return Ok(d.clone());
        }
    }
    Err(format!("Deliverable not found: {}", deliverable_id))
}

#[command]
pub fn cowork_templates(category: Option<String>) -> Result<Vec<CoworkTemplate>, String> {
    let all = default_templates();
    match category {
        Some(cat) => Ok(all.into_iter().filter(|t| t.category == cat).collect()),
        None => Ok(all),
    }
}

#[command]
pub fn cowork_apply_template(session_id: String, template_id: String) -> Result<Vec<CoworkAction>, String> {
    find_session(&session_id)?;
    let templates = default_templates();
    let tpl = templates.into_iter().find(|t| t.id == template_id)
        .ok_or_else(|| format!("Template not found: {}", template_id))?;

    let now = now_ts();
    let actions: Vec<CoworkAction> = tpl.steps.into_iter().map(|step| {
        let action_id = format!("act-{}-{}", template_id, step.order);
        CoworkAction {
            id: action_id.clone(),
            session_id: session_id.clone(),
            action_type: step.action.clone(),
            target_path: session_id.clone(),
            status: "pending".into(),
            started_at: now,
            completed_at: None,
            details: Some(step.description),
            result_summary: None,
        }
    }).collect();

    let mut state = COWORK.lock().map_err(|e| e.to_string())?;
    let session_actions = state.actions.entry(session_id).or_default();
    for action in &actions {
        session_actions.push(action.clone());
    }

    Ok(actions)
}

#[command]
pub fn cowork_actions(session_id: String) -> Result<Vec<CoworkAction>, String> {
    let state = COWORK.lock().map_err(|e| e.to_string())?;
    state.actions.get(&session_id).cloned().ok_or_else(|| format!("Session not found: {}", session_id))
}

#[command]
pub fn cowork_config() -> Result<CoworkConfig, String> {
    let state = COWORK.lock().map_err(|e| e.to_string())?;
    Ok(state.config.clone())
}

#[command]
pub fn cowork_set_config(config: CoworkConfig) -> Result<(), String> {
    let mut state = COWORK.lock().map_err(|e| e.to_string())?;
    state.config = config;
    Ok(())
}

#[command]
pub fn cowork_stats() -> Result<CoworkStats, String> {
    let state = COWORK.lock().map_err(|e| e.to_string())?;
    let total_sessions = state.sessions.len() as u32;
    let active_sessions = state.sessions.values().filter(|s| s.status == "active").count() as u32;
    let total_deliverables: u32 = state.deliverables.values().map(|d| d.len() as u32).sum();
    let files_processed: u32 = state.sessions.values().map(|s| s.files_read).sum();
    let avg_files_per_session = if total_sessions > 0 {
        files_processed as f64 / total_sessions as f64
    } else {
        0.0
    };

    let templates = default_templates();
    let top_template = templates.first().map(|t| t.name.clone()).unwrap_or_default();
    let top_category = {
        let mut counts: HashMap<&str, u32> = HashMap::new();
        for t in &templates {
            *counts.entry(t.category.as_str()).or_insert(0) += 1;
        }
        counts.into_iter().max_by_key(|&(_, c)| c).map(|(k, _)| k.to_string()).unwrap_or_default()
    };

    Ok(CoworkStats {
        total_sessions,
        total_deliverables,
        files_processed,
        active_sessions,
        avg_files_per_session,
        top_category,
        top_template,
    })
}

#[command]
pub fn cowork_export_session(session_id: String, format: Option<String>) -> Result<String, String> {
    let state = COWORK.lock().map_err(|e| e.to_string())?;
    let session = state.sessions.get(&session_id).ok_or_else(|| format!("Session not found: {}", session_id))?;
    let actions = state.actions.get(&session_id).cloned().unwrap_or_default();
    let deliverables = state.deliverables.get(&session_id).cloned().unwrap_or_default();

    let fmt = format.unwrap_or_else(|| "json".into());
    match fmt.as_str() {
        "json" => {
            let export = serde_json::json!({
                "session": session,
                "actions": actions,
                "deliverables": deliverables,
                "exported_at": now_ts(),
            });
            serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
        }
        "markdown" | "md" => {
            let mut md = String::new();
            md.push_str(&format!("# Cowork Session: {}\n\n", session.name));
            md.push_str(&format!("- **ID**: {}\n", session.id));
            md.push_str(&format!("- **Status**: {}\n", session.status));
            md.push_str(&format!("- **Workspace**: {}\n", session.workspace_path));
            md.push_str(&format!("- **Files Read**: {}\n", session.files_read));
            md.push_str(&format!("- **Files Created**: {}\n", session.files_created));
            md.push_str(&format!("- **Files Modified**: {}\n", session.files_modified));
            md.push_str(&format!("- **Description**: {}\n", session.description));
            if !session.tags.is_empty() {
                md.push_str(&format!("- **Tags**: {}\n", session.tags.join(", ")));
            }
            md.push('\n');
            if !deliverables.is_empty() {
                md.push_str("## Deliverables\n\n");
                for d in &deliverables {
                    md.push_str(&format!("- **{}**: {} ({} bytes, {})\n", d.name, d.description, d.size_bytes, d.kind));
                }
                md.push('\n');
            }
            if !actions.is_empty() {
                md.push_str("## Actions\n\n");
                for a in &actions {
                    md.push_str(&format!("- [{}] {} → {} ({})\n", a.status, a.action_type, a.target_path, a.started_at));
                }
            }
            Ok(md)
        }
        _ => Err(format!("Unsupported export format: {}. Supported: json, markdown", fmt)),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn reset_state() {
        if let Ok(mut state) = COWORK.lock() {
            state.sessions.clear();
            state.actions.clear();
            state.deliverables.clear();
            state.config = CoworkConfig::default();
        }
    }

    #[test]
    fn test_cowork_start_and_get() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let id = cowork_start("/tmp".into(), "test session".into(), Some("test".into()), None).unwrap();
        assert!(id.starts_with("cw-"));
        let session = cowork_get(id.clone()).unwrap();
        assert_eq!(session.name, "test");
        assert_eq!(session.status, "active");
        assert_eq!(session.description, "test session");
    }

    #[test]
    fn test_cowork_pause_resume_stop() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let id = cowork_start("/tmp".into(), "pause test".into(), None, None).unwrap();
        cowork_pause(id.clone()).unwrap();
        assert_eq!(cowork_get(id.clone()).unwrap().status, "paused");
        cowork_resume(id.clone()).unwrap();
        assert_eq!(cowork_get(id.clone()).unwrap().status, "active");
        let session = cowork_stop(id.clone()).unwrap();
        assert_eq!(session.status, "completed");
    }

    #[test]
    fn test_cowork_list() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let _ = cowork_start("/tmp".into(), "list test".into(), None, None).unwrap();
        let sessions = cowork_list().unwrap();
        assert!(!sessions.is_empty());
    }

    #[test]
    fn test_cowork_delete() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let id = cowork_start("/tmp".into(), "delete test".into(), None, None).unwrap();
        cowork_delete(id.clone()).unwrap();
        // 删除后不可再获取，列表为空
        assert!(cowork_get(id.clone()).is_err());
        assert!(cowork_list().unwrap().is_empty());
        // 重复删除报错（幂等保护）
        assert!(cowork_delete(id.clone()).is_err());
    }

    #[test]
    fn test_cowork_config_default() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let cfg = cowork_config().unwrap();
        assert!(cfg.enabled);
        assert_eq!(cfg.max_files_per_scan, 500);
        assert!(cfg.allow_file_create);
        assert!(!cfg.allow_file_delete);
    }

    #[test]
    fn test_cowork_templates() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let templates = cowork_templates(None).unwrap();
        assert_eq!(templates.len(), 6);
        let doc_templates = cowork_templates(Some("writing".into())).unwrap();
        assert_eq!(doc_templates.len(), 2);
    }

    #[test]
    fn test_cowork_set_config() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let cfg = CoworkConfig {
            enabled: false,
            max_files_per_scan: 100,
            max_file_size_kb: 512,
            auto_save: false,
            deliverable_formats: vec!["md".into()],
            allow_file_create: false,
            allow_file_modify: false,
            allow_file_delete: true,
        };
        cowork_set_config(cfg).unwrap();
        let updated = cowork_config().unwrap();
        assert!(!updated.enabled);
        assert_eq!(updated.max_files_per_scan, 100);
        assert!(updated.allow_file_delete);
    }

    #[test]
    fn test_cowork_stats() {
        let _guard = TEST_LOCK.lock().unwrap();
        reset_state();
        let id = cowork_start("/tmp".into(), "stats test".into(), None, None).unwrap();
        let _ = cowork_write_file(id.clone(), "/tmp/cowork_test_stats.txt".into(), "hello".into());
        let stats = cowork_stats().unwrap();
        assert!(stats.total_sessions > 0);
        assert_eq!(stats.total_deliverables, 1);
        let _ = std::fs::remove_file("/tmp/cowork_test_stats.txt");
    }
}
