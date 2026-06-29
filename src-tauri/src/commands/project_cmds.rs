use tauri::command;
use super::{FileNode, ProjectInfo};

#[command]
pub fn read_dir_recursive(path: String, max_depth: u32) -> Result<Vec<FileNode>, String> {
    let dir = std::path::Path::new(&path);
    if !dir.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }
    read_dir_inner(dir, 0, max_depth)
}

fn read_dir_inner(dir: &std::path::Path, depth: u32, max_depth: u32) -> Result<Vec<FileNode>, String> {
    if depth > max_depth { return Ok(Vec::new()); }
    let mut nodes = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();
        if name.starts_with('.') || name == "node_modules" || name == "target" { continue; }
        let is_dir = path.is_dir();
        let children = if is_dir { Some(read_dir_inner(&path, depth + 1, max_depth)?) } else { None };
        nodes.push(FileNode { name, path: path.to_string_lossy().to_string(), is_dir, children });
    }
    nodes.sort_by(|a, b| b.is_dir.cmp(&a.is_dir));
    Ok(nodes)
}

#[command]
pub fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, &content).map_err(|e| e.to_string())
}

#[command]
pub fn detect_project(path: String) -> Result<ProjectInfo, String> {
    let dir = std::path::Path::new(&path);
    let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();
    let language = if dir.join("Cargo.toml").exists() { "Rust" }
        else if dir.join("package.json").exists() { "JavaScript/TypeScript" }
        else if dir.join("pyproject.toml").exists() || dir.join("setup.py").exists() { "Python" }
        else if dir.join("go.mod").exists() { "Go" }
        else { "Unknown" };
    let file_count = count_files(dir, 0);
    Ok(ProjectInfo { name, path, language: language.into(), file_count })
}

#[command]
pub fn cmd_project_open(path: String) -> Result<String, String> {
    let dir = std::path::Path::new(&path);
    if !dir.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !dir.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }
    let name = dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let project_type = if dir.join("Cargo.toml").exists() { "Rust" }
        else if dir.join("package.json").exists() { "JavaScript/TypeScript" }
        else if dir.join("pyproject.toml").exists() || dir.join("setup.py").exists() { "Python" }
        else if dir.join("go.mod").exists() { "Go" }
        else { "Unknown" };
    let file_count = count_project_files(dir);
    let metadata = serde_json::json!({
        "name": name, "path": path, "type": project_type, "file_count": file_count,
    });
    Ok(metadata.to_string())
}

fn count_project_files(dir: &std::path::Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') || name == "node_modules" || name == "target" { continue; }
            if p.is_dir() { count += count_project_files(&p); }
            else { count += 1; }
        }
    }
    count
}

#[command]
pub fn cmd_scan_files(path: String, pattern: Option<String>) -> Result<Vec<String>, String> {
    let dir = std::path::Path::new(&path);
    if !dir.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    let mut files = Vec::new();
    scan_files_recursive(dir, &mut files, 3)?;
    if let Some(pat) = pattern {
        let pat_lower = pat.to_lowercase();
        files.retain(|f| {
            let p = std::path::Path::new(f);
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            name.contains(&pat_lower) || ext == pat_lower.trim_start_matches('.')
        });
    }
    Ok(files)
}

fn scan_files_recursive(dir: &std::path::Path, files: &mut Vec<String>, max_depth: u32) -> Result<(), String> {
    if max_depth == 0 { return Ok(()); }
    let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with('.') || name == "node_modules" || name == "target" { continue; }
        if path.is_dir() {
            scan_files_recursive(&path, files, max_depth - 1)?;
        } else {
            files.push(path.to_string_lossy().to_string());
        }
    }
    Ok(())
}

fn count_files(dir: &std::path::Path, depth: u32) -> usize {
    if depth > 5 { return 0; }
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') || name == "node_modules" || name == "target" { continue; }
            if path.is_dir() { count += count_files(&path, depth + 1); }
            else { count += 1; }
        }
    }
    count
}
