//! Git Plugin — Git 版本控制

use super::common::stub_action;
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use async_trait::async_trait;
use std::process::Command as StdCommand;

pub struct GitPlugin;

impl GitPlugin {
    fn git_command(args: &[&str], cwd: Option<&str>) -> Result<String, DomainError> {
        let mut cmd = StdCommand::new("git");
        cmd.args(args);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        let output = cmd.output().map_err(|e| DomainError {
            code: "GIT_ERROR".into(),
            message: format!("git 执行失败: {}", e),
            recoverable: true,
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(DomainError {
                code: "GIT_ERROR".into(),
                message: stderr,
                recoverable: true,
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[async_trait]
impl DomainPlugin for GitPlugin {
    fn name(&self) -> &str {
        "git"
    }
    fn description(&self) -> &str {
        "Git 版本控制"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            "status",
            "diff",
            "staged_files",
            "branches",
            "checkout",
            "commit",
            "push",
            "apply_diff",
        ]
        .iter()
        .map(|a| stub_action(a))
        .collect()
    }
    async fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        let cwd = args.get("cwd").and_then(|v| v.as_str());
        match action {
            "status" => {
                let output = Self::git_command(&["status", "--porcelain"], cwd)?;
                let files: Vec<serde_json::Value> = output
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|l| {
                        let status = if l.len() >= 2 { &l[..2] } else { "  " };
                        let path = if l.len() > 3 { l[3..].trim() } else { "" };
                        serde_json::json!({
                            "status": status.trim(),
                            "path": path,
                        })
                    })
                    .collect();
                Ok(serde_json::json!({
                    "clean": files.is_empty(),
                    "files": files,
                    "count": files.len(),
                }))
            }
            "diff" => {
                let file = args.get("file").and_then(|v| v.as_str());
                let mut git_args = vec!["diff"];
                if let Some(f) = file {
                    git_args.extend_from_slice(&["HEAD", "--", f]);
                } else {
                    git_args.push("HEAD");
                }
                let output = Self::git_command(&git_args, cwd)?;
                Ok(serde_json::json!({
                    "diff": output,
                    "lines": output.lines().count(),
                }))
            }
            "staged_files" => {
                let output = Self::git_command(&["diff", "--cached", "--name-status"], cwd)?;
                let files: Vec<serde_json::Value> = output
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|l| {
                        let parts: Vec<&str> = l.splitn(2, '\t').collect();
                        let status = parts.first().unwrap_or(&"").trim();
                        let path = parts.get(1).unwrap_or(&"");
                        serde_json::json!({
                            "status": status,
                            "path": path,
                        })
                    })
                    .collect();
                Ok(serde_json::json!({
                    "files": files,
                    "count": files.len(),
                }))
            }
            "branches" => {
                let output = Self::git_command(&["branch", "-a"], cwd)?;
                let current = output
                    .lines()
                    .find(|l| l.starts_with('*'))
                    .map(|l| l.trim_start_matches("* ").trim().to_string())
                    .unwrap_or_default();
                let branches: Vec<String> = output
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|l| l.trim_start_matches("* ").trim().to_string())
                    .collect();
                Ok(serde_json::json!({
                    "current": current,
                    "branches": branches,
                    "count": branches.len(),
                }))
            }
            "checkout" => {
                let branch =
                    args.get("branch")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 branch 参数".into(),
                            recoverable: true,
                        })?;
                let output = Self::git_command(&["checkout", branch], cwd)?;
                Ok(serde_json::json!({ "ok": true, "branch": branch, "output": output.trim() }))
            }
            "commit" => {
                let message = args
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 message 参数".into(),
                        recoverable: true,
                    })?;
                let output = Self::git_command(&["commit", "-m", message], cwd)?;
                Ok(serde_json::json!({ "ok": true, "output": output.trim() }))
            }
            "push" => {
                let remote = args
                    .get("remote")
                    .and_then(|v| v.as_str())
                    .unwrap_or("origin");
                let branch = args.get("branch").and_then(|v| v.as_str());
                let mut git_args = vec!["push", remote];
                if let Some(b) = branch {
                    git_args.push(b);
                }
                let output = Self::git_command(&git_args, cwd)?;
                Ok(serde_json::json!({ "ok": true, "output": output.trim() }))
            }
            "apply_diff" => {
                let diff =
                    args.get("diff")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 diff 参数".into(),
                            recoverable: true,
                        })?;
                let mut cmd = StdCommand::new("git");
                cmd.args(["apply", "--check"]);
                if let Some(dir) = cwd {
                    cmd.current_dir(dir);
                }
                cmd.stdin(std::process::Stdio::piped());
                let mut child = cmd.spawn().map_err(|e| DomainError {
                    code: "GIT_ERROR".into(),
                    message: format!("启动 git apply 失败: {}", e),
                    recoverable: true,
                })?;
                if let Some(stdin) = child.stdin.take() {
                    use std::io::Write;
                    let mut stdin = stdin;
                    stdin.write_all(diff.as_bytes()).map_err(|e| DomainError {
                        code: "GIT_ERROR".into(),
                        message: format!("写入 diff 失败: {}", e),
                        recoverable: true,
                    })?;
                }
                let check = child.wait().map_err(|e| DomainError {
                    code: "GIT_ERROR".into(),
                    message: format!("git apply --check 失败: {}", e),
                    recoverable: true,
                })?;
                if !check.success() {
                    return Err(DomainError {
                        code: "GIT_ERROR".into(),
                        message: "diff 检查失败，无法应用".into(),
                        recoverable: true,
                    });
                }
                // Apply without --check
                let mut cmd2 = StdCommand::new("git");
                cmd2.args(["apply"]);
                if let Some(dir) = cwd {
                    cmd2.current_dir(dir);
                }
                cmd2.stdin(std::process::Stdio::piped());
                let mut child2 = cmd2.spawn().map_err(|e| DomainError {
                    code: "GIT_ERROR".into(),
                    message: format!("启动 git apply 失败: {}", e),
                    recoverable: true,
                })?;
                if let Some(stdin) = child2.stdin.take() {
                    use std::io::Write;
                    let mut stdin = stdin;
                    stdin.write_all(diff.as_bytes()).map_err(|e| DomainError {
                        code: "GIT_ERROR".into(),
                        message: format!("写入 diff 失败: {}", e),
                        recoverable: true,
                    })?;
                }
                let apply = child2.wait().map_err(|e| DomainError {
                    code: "GIT_ERROR".into(),
                    message: format!("git apply 失败: {}", e),
                    recoverable: true,
                })?;
                if apply.success() {
                    Ok(serde_json::json!({ "ok": true }))
                } else {
                    Err(DomainError {
                        code: "GIT_ERROR".into(),
                        message: "git apply 失败".into(),
                        recoverable: true,
                    })
                }
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}
