use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Repo-to-Agent-Context scaffold: `.agent-context/` directory with
/// repo-summary, module-map, key-files, and readiness state.
///
/// Created once per project root (or refreshed on request).  The agent
/// reads these files at session start to bootstrap codebase understanding
/// without scanning the entire tree.

// ── Public types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessState {
    pub initialized: bool,
    pub last_updated: String,
    pub file_count: usize,
    pub repo_summary_lines: usize,
    pub module_map_entries: usize,
    pub key_files_count: usize,
}

/// Result of an `init_context` call.
#[derive(Debug, Clone)]
pub struct InitContextReport {
    pub context_dir: PathBuf,
    pub readiness: ReadinessState,
    pub errors: Vec<String>,
}

// ── Core logic ───────────────────────────────────────────────

const CONTEXT_DIR: &str = ".agent-context";

/// Initialize `.agent-context/` in the given project root.
/// Scans the project, generates summaries, and writes scaffold files.
pub fn init_context(project_root: &Path) -> Result<InitContextReport, String> {
    let context_dir = project_root.join(CONTEXT_DIR);
    fs::create_dir_all(&context_dir)
        .map_err(|e| format!("Failed to create {}: {}", context_dir.display(), e))?;

    let mut errors: Vec<String> = Vec::new();

    // 1. repo-summary.md
    let repo_summary = generate_repo_summary(project_root);
    let repo_lines = repo_summary.lines().count();
    if let Err(e) = fs::write(context_dir.join("repo-summary.md"), &repo_summary) {
        errors.push(format!("Failed to write repo-summary.md: {}", e));
    }

    // 2. module-map.md
    let module_map = generate_module_map(project_root);
    let module_entries = module_map.lines().filter(|l| l.starts_with("- `")).count();
    if let Err(e) = fs::write(context_dir.join("module-map.md"), &module_map) {
        errors.push(format!("Failed to write module-map.md: {}", e));
    }

    // 3. key-files.md
    let key_files = generate_key_files(project_root);
    let key_count = key_files.lines().filter(|l| l.starts_with("- `")).count();
    if let Err(e) = fs::write(context_dir.join("key-files.md"), &key_files) {
        errors.push(format!("Failed to write key-files.md: {}", e));
    }

    // 4. readiness.json
    let now = chrono::Utc::now().to_rfc3339();
    let total_files = count_source_files(project_root);
    let readiness = ReadinessState {
        initialized: true,
        last_updated: now,
        file_count: total_files,
        repo_summary_lines: repo_lines,
        module_map_entries: module_entries,
        key_files_count: key_count,
    };
    if let Err(e) = fs::write(
        context_dir.join("readiness.json"),
        &serde_json::to_string_pretty(&readiness).unwrap_or_default(),
    ) {
        errors.push(format!("Failed to write readiness.json: {}", e));
    }

    // 5. MEMORY.md stub (will be maintained by the agent across sessions)
    let memory_path = context_dir.join("MEMORY.md");
    if !memory_path.exists() {
        let memory_stub = generate_memory_stub();
        if let Err(e) = fs::write(&memory_path, &memory_stub) {
            errors.push(format!("Failed to write MEMORY.md: {}", e));
        }
    }

    Ok(InitContextReport {
        context_dir,
        readiness,
        errors,
    })
}

/// Append a decision to MEMORY.md (used by agent session-end hook).
pub fn append_memory_entry(context_dir: &Path, entry: &str) -> Result<(), String> {
    let path = context_dir.join("MEMORY.md");
    let mut content = fs::read_to_string(&path).unwrap_or_default();
    content.push_str(&format!("\n- {}  \n", entry));
    fs::write(&path, &content).map_err(|e| format!("Failed to append MEMORY.md: {}", e))
}

/// Read top-N entries from MEMORY.md for session-start injection.
pub fn read_recent_memory(context_dir: &Path, n: usize) -> Vec<String> {
    let path = context_dir.join("MEMORY.md");
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    content
        .lines()
        .filter(|l| l.trim().starts_with("- ") && l.contains("|"))
        .rev()
        .take(n)
        .map(|l| l.trim().trim_start_matches("- ").to_string())
        .collect()
}

/// Check if `.agent-context/` already exists and is valid.
pub fn context_exists(project_root: &Path) -> bool {
    let dir = project_root.join(CONTEXT_DIR);
    dir.exists() && dir.join("readiness.json").exists() && dir.join("repo-summary.md").exists()
}

/// Unconditionally delete and re-init `.agent-context/`.
pub fn reinit_context(project_root: &Path) -> Result<InitContextReport, String> {
    let dir = project_root.join(CONTEXT_DIR);
    if dir.exists() {
        fs::remove_dir_all(&dir)
            .map_err(|e| format!("Failed to remove {}: {}", dir.display(), e))?;
    }
    init_context(project_root)
}

// ── Internal generators ──────────────────────────────────────

fn generate_repo_summary(project_root: &Path) -> String {
    let mut lines = Vec::new();

    // Project name from directory
    let name = project_root
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap_or(std::borrow::Cow::Borrowed("unknown"));
    lines.push(format!("# {} — Repository Summary", name));

    // Cargo workspace info
    let cargo_path = project_root.join("Cargo.toml");
    if let Ok(content) = fs::read_to_string(&cargo_path) {
        if let Some(desc) = extract_field(&content, "description") {
            lines.push(format!("\n**Description:** {}", desc));
        }
        if let Some(vers) = extract_field(&content, "version") {
            lines.push(format!("**Version:** {}", vers));
        }
        if let Some(ed) = extract_field(&content, "edition") {
            lines.push(format!("**Edition:** {}", ed));
        }
    }

    // Key directories
    let dirs = scan_top_dirs(project_root);
    if !dirs.is_empty() {
        lines.push("\n## Top-Level Structure\n".to_string());
        for (name, kind) in &dirs {
            lines.push(format!("- **{}**/ — {}", name, kind));
        }
    }

    // Source file counts
    let rs_count = count_files_by_ext(project_root, "rs");
    let ts_count = count_files_by_ext(project_root, "ts") + count_files_by_ext(project_root, "tsx");
    let total = count_source_files(project_root);
    lines.push(format!(
        "\n**{} source files** ({} .rs, {} .ts/tsx)",
        total, rs_count, ts_count
    ));

    lines.join("\n")
}

fn generate_module_map(project_root: &Path) -> String {
    let mut lines = vec!["# Module Map\n".to_string()];

    // Scan for key Rust modules
    let src_dirs = vec![
        ("neotrix-core/src/neotrix", "Core NeoTrix engine"),
        ("neotrix-core/src/core", "VSA consciousness core"),
        ("neotrix-core/src/cli", "CLI/TUI commands"),
        ("src-tauri/src", "Tauri desktop backend"),
        ("src-tauri/frontend/src", "React frontend"),
    ];

    for (rel_path, desc) in src_dirs {
        let full = project_root.join(rel_path);
        if full.exists() {
            lines.push(format!("\n### `{}` — {}", rel_path, desc));
            if let Ok(entries) = fs::read_dir(&full) {
                let mut modules: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                    .filter(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "rs" || ext == "ts" || ext == "tsx")
                            .unwrap_or(false)
                    })
                    .map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        format!("- `{}`", name)
                    })
                    .collect();
                modules.sort();
                lines.extend(modules);
            }
        }
    }

    lines.join("\n")
}

fn generate_key_files(project_root: &Path) -> String {
    let mut lines = vec!["# Key Files\n".to_string()];

    let key_specs: Vec<(&str, &str)> = vec![
        (
            "Cargo.toml",
            "Workspace root — crates, dependencies, metadata",
        ),
        ("neotrix-core/Cargo.toml", "Core library manifest"),
        ("src-tauri/Cargo.toml", "Tauri desktop app manifest"),
        (
            "src-tauri/tauri.conf.json",
            "Tauri window/bundle configuration",
        ),
        ("AGENTS.md", "Consciousness body behavior specification"),
        ("DESIGN_INTENT.md", "Design intent document"),
        (
            "src-tauri/src/main.rs",
            "Tauri entry point + command registration",
        ),
        (
            "src-tauri/src/commands/agent_cmds.rs",
            "Agent loop: LLM + tool execution + browsing",
        ),
        ("src-tauri/frontend/src/App.tsx", "Frontend root component"),
        ("src-tauri/frontend/src/store.ts", "Frontend state store"),
        (
            "src-tauri/frontend/src/components/StatusBar.tsx",
            "Status bar (proxy toggle)",
        ),
        (
            "src-tauri/frontend/src/components/LoginDialog.tsx",
            "Login credential dialog",
        ),
        (
            "neotrix-core/src/neotrix/nt_io_provider/internet_discovery.rs",
            "Internet model discovery",
        ),
        (
            "neotrix-core/src/neotrix/nt_io_provider/token_economy.rs",
            "Token economy + storm-breaker",
        ),
        (
            "neotrix-core/src/neotrix/nt_io_provider/compaction.rs",
            "Context compaction (history repair)",
        ),
        (
            "neotrix-core/src/neotrix/nt_world_exploration/browsing_agent.rs",
            "Browsing agent (navigate/fill/click/login)",
        ),
    ];

    for (path, desc) in key_specs {
        let full = project_root.join(path);
        if full.exists() {
            lines.push(format!("- `{}` — {}", path, desc));
        } else {
            lines.push(format!("- `{}` — {} *(not yet created)*", path, desc));
        }
    }

    lines.join("\n")
}

fn generate_memory_stub() -> String {
    r#"# MEMORY.md — Session Memory Log

> Decisions, open questions, and unresolved items across sessions.
> Appended by the agent at session end; top-N injected at session start.

## Decisions

## Open Questions

## Unresolved Items
"#
    .to_string()
}

// ── Helpers ──────────────────────────────────────────────────

fn extract_field(content: &str, field: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix(&format!("{} = ", field)) {
            return Some(val.trim_matches('"').to_string());
        }
    }
    None
}

fn scan_top_dirs(root: &Path) -> Vec<(String, String)> {
    let mut dirs = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name == "node_modules" || name == "target" {
                    continue;
                }
                let kind = classify_dir(&name);
                dirs.push((name, kind));
            }
        }
    }
    dirs.sort_by(|a, b| a.0.cmp(&b.0));
    dirs
}

fn classify_dir(name: &str) -> String {
    match name {
        "neotrix-core" => "Core library (engine, consciousness, VSA)".into(),
        "src-tauri" => "Desktop Tauri application".into(),
        "crates" => "Utility crates (nt-lang, nt-migrate)".into(),
        "scripts" => "Helper scripts".into(),
        ".github" => "CI/CD workflows".into(),
        _ => "Project directory".into(),
    }
}

fn count_source_files(root: &Path) -> usize {
    let exts = [
        "rs", "ts", "tsx", "js", "jsx", "toml", "json", "yaml", "yml", "md",
    ];
    exts.iter().map(|ext| count_files_by_ext(root, ext)).sum()
}

fn count_files_by_ext(root: &Path, ext: &str) -> usize {
    fn walk(dir: &Path, ext: &str, count: &mut usize) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .map(|s| s.to_string_lossy())
                        .unwrap_or_default();
                    if name != "target" && name != "node_modules" && !name.starts_with('.') {
                        walk(&path, ext, count);
                    }
                } else if path.extension().map(|e| e == ext).unwrap_or(false) {
                    *count += 1;
                }
            }
        }
    }
    let mut count = 0;
    walk(root, ext, &mut count);
    count
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_context_exists_negative() {
        let tmp = std::env::temp_dir().join("neotrix_test_no_context");
        let _ = fs::remove_dir_all(&tmp);
        assert!(!context_exists(&tmp));
    }

    #[test]
    fn test_generate_repo_summary_does_not_panic() {
        let tmp = std::env::temp_dir().join("neotrix_test_summary");
        let _ = fs::create_dir_all(&tmp);
        let summary = generate_repo_summary(&tmp);
        assert!(!summary.is_empty());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_init_context_creates_files() {
        let tmp = std::env::temp_dir().join("neotrix_test_init");
        let _ = fs::create_dir_all(&tmp);
        let report = init_context(&tmp).unwrap();
        assert!(report.context_dir.join("repo-summary.md").exists());
        assert!(report.context_dir.join("module-map.md").exists());
        assert!(report.context_dir.join("key-files.md").exists());
        assert!(report.context_dir.join("readiness.json").exists());
        assert!(report.context_dir.join("MEMORY.md").exists());
        assert!(report.readiness.initialized);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_append_and_read_memory() {
        let tmp = std::env::temp_dir().join("neotrix_test_memory");
        let _ = fs::create_dir_all(&tmp);
        let report = init_context(&tmp).unwrap();
        append_memory_entry(
            &report.context_dir,
            "2026-06-11 | Decision | Use Kun-style token economy",
        )
        .unwrap();
        let recent = read_recent_memory(&report.context_dir, 5);
        assert!(!recent.is_empty());
        assert!(recent[0].contains("Kun-style"));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_reinit_context() {
        let tmp = std::env::temp_dir().join("neotrix_test_reinit");
        let _ = fs::create_dir_all(&tmp);
        let r1 = init_context(&tmp).unwrap();
        assert!(r1.readiness.initialized);
        let r2 = reinit_context(&tmp).unwrap();
        assert!(r2.readiness.initialized);
        assert!(r2.context_dir.join("repo-summary.md").exists());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_extract_field() {
        let content = r#"
[package]
name = "neotrix"
description = "A silicon consciousness"
version = "0.18.0"
"#;
        assert_eq!(extract_field(content, "name"), Some("neotrix".into()));
        assert_eq!(extract_field(content, "version"), Some("0.18.0".into()));
        assert_eq!(
            extract_field(content, "description"),
            Some("A silicon consciousness".into())
        );
    }

    #[test]
    fn test_count_files_by_ext_on_self() {
        let count = count_files_by_ext(Path::new("."), "rs");
        assert!(count > 0, "Should find at least one .rs file in project");
    }

    #[test]
    fn test_context_exists_after_init() {
        let tmp = std::env::temp_dir().join("neotrix_test_exists");
        let _ = fs::create_dir_all(&tmp);
        assert!(!context_exists(&tmp));
        init_context(&tmp).unwrap();
        assert!(context_exists(&tmp));
        let _ = fs::remove_dir_all(&tmp);
    }
}
