use tauri::command;
use neotrix::neotrix::nt_core_error::NeoTrixError;
use super::DiffBlock;

pub(crate) fn parse_git_diff(diff_str: &str) -> Vec<DiffBlock> {
    let mut blocks = Vec::new();
    for line in diff_str.lines() {
        if let Some(stripped) = line.strip_prefix("+") {
            if !stripped.starts_with("+") {
                blocks.push(DiffBlock { r#type: "added".into(), content: stripped.to_string(), line_start: 0 });
                continue;
            }
        }
        if let Some(stripped) = line.strip_prefix("-") {
            if !stripped.starts_with("-") {
                blocks.push(DiffBlock { r#type: "removed".into(), content: stripped.to_string(), line_start: 0 });
                continue;
            }
        }
        if !line.starts_with("diff") && !line.starts_with("index") && !line.starts_with("---") && !line.starts_with("+++") && !line.starts_with("@@") && !line.starts_with("\\ ") {
            blocks.push(DiffBlock { r#type: "unchanged".into(), content: line.to_string(), line_start: 0 });
        }
    }
    blocks
}

fn run_git_cmd(args: &[&str]) -> Result<String, NeoTrixError> {
    let output = std::process::Command::new("git")
        .args(args)
        .output()
        .map_err(|e| NeoTrixError::Command { cmd: format!("git {}", args.join(" ")), exit_code: None, stderr: e.to_string() })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(NeoTrixError::Command { cmd: format!("git {}", args.join(" ")), exit_code: output.status.code(), stderr: stderr.trim().to_string() });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[command]
pub fn cmd_diff_staged() -> Result<Vec<DiffBlock>, NeoTrixError> {
    run_git_cmd(&["diff", "--cached"]).map(|s| parse_git_diff(&s))
}

#[command]
pub fn cmd_diff_unstaged() -> Result<Vec<DiffBlock>, NeoTrixError> {
    run_git_cmd(&["diff"]).map(|s| parse_git_diff(&s))
}

#[command]
pub fn cmd_diff_file(path: String) -> Result<Vec<DiffBlock>, NeoTrixError> {
    run_git_cmd(&["diff", "HEAD", "--", &path]).map(|s| parse_git_diff(&s))
}

/// Stage the given paths (or all changes when `paths` is empty). Returns the
/// updated list of changed files for the review UI.
#[command]
pub fn cmd_diff_stage(paths: Option<Vec<String>>) -> Result<Vec<String>, NeoTrixError> {
    match paths {
        Some(p) if !p.is_empty() => {
            let mut args: Vec<String> = vec!["add".into(), "--".into()];
            args.extend(p);
            let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            run_git_cmd(&arg_refs)?;
        }
        _ => { run_git_cmd(&["add", "-A"])?; }
    };
    changed_files_porcelain()
}

fn changed_files_porcelain() -> Result<Vec<String>, NeoTrixError> {
    let out = run_git_cmd(&["status", "--porcelain"])?;
    Ok(out
        .lines()
        .filter_map(|l| {
            if l.len() <= 3 { return None; }
            let p = l[3..].trim().to_string();
            if p.is_empty() { None } else { Some(p) }
        })
        .collect())
}

/// Unstage the given paths (or everything when `paths` is empty). Returns the
/// updated porcelain file list.
#[command]
pub fn cmd_diff_unstage(paths: Option<Vec<String>>) -> Result<Vec<String>, NeoTrixError> {
    match paths {
        Some(p) if !p.is_empty() => {
            let mut args: Vec<String> = vec!["reset".into(), "HEAD".into(), "--".into()];
            args.extend(p);
            let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            run_git_cmd(&arg_refs)?;
        }
        _ => { run_git_cmd(&["reset"])?; }
    };
    changed_files_porcelain()
}

/// Commit staged changes with the given message (Codex "Review changes" parity).
#[command]
pub fn cmd_diff_commit(message: String) -> Result<(), NeoTrixError> {
    run_git_cmd(&["commit", "-m", &message]).map(|_| ())
}
