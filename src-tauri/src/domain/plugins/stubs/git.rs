//! Git Plugin — Git 版本控制

use super::common::stub_action;
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use async_trait::async_trait;
use std::process::Command as StdCommand;

pub struct GitPlugin;

/// Push the accumulated hunk lines when inside a hunk.
fn flush_hunk(
    hunks: &mut Vec<serde_json::Value>,
    lines: &mut Vec<serde_json::Value>,
    in_hunk: bool,
) {
    if in_hunk {
        hunks.push(serde_json::json!({ "lines": std::mem::take(lines) }));
    }
}

impl GitPlugin {
    /// Parse `git diff` unified output into per-file hunks matching the
    /// frontend `DiffFile` contract: `{path, hunks: [{lines: [{t, o, n, s}]}]}`
    /// where `t` is `add`/`del`/`ctx` and `o`/`n` are old/new line numbers.
    fn parse_diff_files(output: &str) -> Vec<serde_json::Value> {
        let mut files: Vec<serde_json::Value> = Vec::new();
        let mut cur_path: Option<String> = None;
        let mut cur_hunks: Vec<serde_json::Value> = Vec::new();
        let mut cur_lines: Vec<serde_json::Value> = Vec::new();
        let mut old_no: u32 = 0;
        let mut new_no: u32 = 0;
        let mut in_hunk = false;

        // NOTE: free function (not a closure) — a closure would borrow
        // `in_hunk` for its whole lifetime and conflict with assignments.

        for line in output.lines() {
            if let Some(rest) = line.strip_prefix("diff --git ") {
                // Close previous file.
                if cur_path.is_some() {
                    let mut hunks = std::mem::take(&mut cur_hunks);
                    let mut lines = std::mem::take(&mut cur_lines);
                    flush_hunk(&mut hunks, &mut lines, in_hunk);
                    cur_hunks = hunks;
                    files.push(serde_json::json!({
                        "path": cur_path.take().unwrap_or_default(),
                        "hunks": cur_hunks,
                    }));
                    cur_hunks = Vec::new();
                    in_hunk = false;
                }
                // `diff --git a/<path> b/<path>` — take the b-side path.
                let path = rest
                    .rsplit_once(" b/")
                    .map(|(_, b)| b.to_string())
                    .unwrap_or_else(|| rest.to_string());
                let path = path
                    .trim_matches('"')
                    .strip_prefix("b/")
                    .unwrap_or(&path)
                    .to_string();
                cur_path = Some(path);
                in_hunk = false;
            } else if line.starts_with("@@") {
                // New hunk header: `@@ -old[,old_count] +new[,new_count] @@ ...`
                let mut hunks = std::mem::take(&mut cur_hunks);
                let mut lines = std::mem::take(&mut cur_lines);
                flush_hunk(&mut hunks, &mut lines, in_hunk);
                cur_hunks = hunks;
                in_hunk = true;
                let header = line.trim_start_matches('@').trim_end_matches('@').trim();
                let mut parts = header.split_whitespace();
                old_no = parts
                    .next()
                    .and_then(|p| p.trim_start_matches('-').split(',').next())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
                new_no = parts
                    .next()
                    .and_then(|p| p.trim_start_matches('+').split(',').next())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
            } else if in_hunk {
                if let Some(content) = line.strip_prefix('+') {
                    if !content.starts_with('+') {
                        cur_lines.push(serde_json::json!({
                            "t": "add", "o": null, "n": new_no, "s": line,
                        }));
                        new_no += 1;
                    }
                } else if let Some(_content) = line.strip_prefix('-') {
                    if !line.starts_with("---") {
                        cur_lines.push(serde_json::json!({
                            "t": "del", "o": old_no, "n": null, "s": line,
                        }));
                        old_no += 1;
                    }
                } else if line.starts_with(' ') {
                    cur_lines.push(serde_json::json!({
                        "t": "ctx", "o": old_no, "n": new_no, "s": line,
                    }));
                    old_no += 1;
                    new_no += 1;
                }
                // Else: `\ No newline...`, index/---/+++/Binary lines — skip.
            }
        }
        if cur_path.is_some() {
            let mut hunks = std::mem::take(&mut cur_hunks);
            let mut lines = std::mem::take(&mut cur_lines);
            flush_hunk(&mut hunks, &mut lines, in_hunk);
            files.push(serde_json::json!({
                "path": cur_path.unwrap_or_default(),
                "hunks": hunks,
            }));
        }
        files
    }

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
            "stage",
            "discard",
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
                let branch = Self::git_command(&["rev-parse", "--abbrev-ref", "HEAD"], cwd)
                    .map(|b| b.trim().to_string())
                    .unwrap_or_default();
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
                    "branch": branch,
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
                let files = Self::parse_diff_files(&output);
                // Flat hunks for GitDiffViewer ({file, additions, deletions, content}).
                let hunks: Vec<serde_json::Value> = files
                    .iter()
                    .flat_map(|f| {
                        let path = f.get("path").and_then(|v| v.as_str()).unwrap_or("");
                        f.get("hunks")
                            .and_then(|v| v.as_array())
                            .cloned()
                            .unwrap_or_default()
                            .into_iter()
                            .map(|h| {
                                let lines = h
                                    .get("lines")
                                    .and_then(|v| v.as_array())
                                    .cloned()
                                    .unwrap_or_default();
                                let additions = lines
                                    .iter()
                                    .filter(|l| {
                                        l.get("t").and_then(|v| v.as_str()) == Some("add")
                                    })
                                    .count();
                                let deletions = lines
                                    .iter()
                                    .filter(|l| {
                                        l.get("t").and_then(|v| v.as_str()) == Some("del")
                                    })
                                    .count();
                                let content = lines
                                    .iter()
                                    .filter_map(|l| l.get("s").and_then(|v| v.as_str()))
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                serde_json::json!({
                                    "file": path,
                                    "additions": additions,
                                    "deletions": deletions,
                                    "content": content,
                                })
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect();
                Ok(serde_json::json!({
                    "diff": output,
                    "lines": output.lines().count(),
                    "files": files,
                    "hunks": hunks,
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
            "stage" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 path 参数".into(),
                        recoverable: true,
                    })?;
                Self::git_command(&["add", "--", path], cwd)?;
                Ok(serde_json::json!({ "ok": true, "path": path }))
            }
            "discard" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 path 参数".into(),
                        recoverable: true,
                    })?;
                // `checkout -- <path>` restores working-tree file (universal,
                // unlike newer `git restore` which old installs lack).
                Self::git_command(&["checkout", "--", path], cwd)?;
                Ok(serde_json::json!({ "ok": true, "path": path }))
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}
