use crate::agent::tools::McpRegistry;

pub fn handle_git_diff(args: &serde_json::Value) -> Result<String, String> {
    let staged = args
        .get("staged")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let output = if staged {
        std::process::Command::new("git")
            .args(["diff", "--cached"])
            .output()
    } else {
        std::process::Command::new("git").args(["diff"]).output()
    }
    .map_err(|e| format!("git diff failed: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn handle_git_status(_args: &serde_json::Value) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| format!("git status failed: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<serde_json::Value> = stdout
        .lines()
        .filter_map(|line| {
            if line.len() < 3 {
                return None;
            }
            let (status, rest) = line.split_at(2);
            Some(serde_json::json!({
                "status": status.trim(),
                "path": rest.trim()
            }))
        })
        .collect();
    Ok(serde_json::to_string_pretty(&entries).unwrap_or_else(|_| stdout.to_string()))
}

pub fn handle_git_commit(args: &serde_json::Value) -> Result<String, String> {
    let message = args
        .get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'message' argument".to_string())?;

    let add = std::process::Command::new("git")
        .args(["add", "-A"])
        .output()
        .map_err(|e| format!("git add failed: {}", e))?;
    if !add.status.success() {
        return Err(String::from_utf8_lossy(&add.stderr).to_string());
    }

    let output = std::process::Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .map_err(|e| format!("git commit failed: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn handle_git_log(args: &serde_json::Value) -> Result<String, String> {
    let max_count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(10);
    let output = std::process::Command::new("git")
        .args(["log", "--oneline", &format!("-{}", max_count)])
        .output()
        .map_err(|e| format!("git log failed: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn register_git_tools(registry: &mut McpRegistry) {
    registry.register_builtin("git_diff", handle_git_diff);
    registry.register_builtin("git_status", handle_git_status);
    registry.register_builtin("git_commit", handle_git_commit);
    registry.register_builtin("git_log", handle_git_log);
}
