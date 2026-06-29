use tauri::command;
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

fn run_git_cmd(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("Git command failed: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git error: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[command]
pub fn cmd_diff_staged() -> Result<Vec<DiffBlock>, String> {
    run_git_cmd(&["diff", "--cached"]).map(|s| parse_git_diff(&s))
}

#[command]
pub fn cmd_diff_unstaged() -> Result<Vec<DiffBlock>, String> {
    run_git_cmd(&["diff"]).map(|s| parse_git_diff(&s))
}

#[command]
pub fn cmd_diff_file(path: String) -> Result<Vec<DiffBlock>, String> {
    run_git_cmd(&["diff", "HEAD", "--", &path]).map(|s| parse_git_diff(&s))
}
