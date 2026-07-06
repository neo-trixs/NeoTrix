//! Git 集成命令 — Git / Commit / Pr / Worktree

use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;

// ====== /git ======

pub struct GitCmd;
impl CliCommand for GitCmd {
    fn name(&self) -> &str { "/git" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str {
        "Git 命令: /git status | /git diff | /git log [--oneline -10] | /git worktree create|list|remove|pr|lock|unlock"
    }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::err("用法: /git <args>\n示例: /git status, /git diff, /git log --oneline -10");
        }
        if args[0] == "worktree" {
            return self.handle_worktree(&args[1..]);
        }
        let cmd_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let output = Command::new("git").args(&cmd_args).output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let combined = if stdout.is_empty() { stderr.trim().to_string() } else { stdout };
                if combined.is_empty() {
                    CommandOutput::ok("(无输出)")
                } else {
                    CommandOutput::ok(&combined)
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CommandOutput::err("git 未安装")
                } else {
                    CommandOutput::err(&format!("git 执行失败: {}", e))
                }
            }
        }
    }
}

impl GitCmd {
    fn handle_worktree(&self, args: &[String]) -> CommandOutput {
        let subcmd = args.first().map(|s| s.as_str()).unwrap_or("");
        match subcmd {
            "list" | "ls" => self.worktree_list(),
            "create" | "add" => self.worktree_create(&args[1..]),
            "remove" | "rm" => self.worktree_remove(&args[1..]),
            "pr" => self.worktree_pr(&args[1..]),
            "lock" => self.worktree_lock(&args[1..]),
            "unlock" => self.worktree_unlock(&args[1..]),
            "" => CommandOutput::ok(
                "用法:\n  /git worktree create <branch> [<path>]\n  /git worktree list\n  /git worktree remove <path>\n  /git worktree pr <number> [<path>]\n  /git worktree lock <path>\n  /git worktree unlock <path>"
            ),
            other => CommandOutput::err(&format!("未知 worktree 子命令: {other}. 可用: create, list, remove, pr, lock, unlock")),
        }
    }

    fn worktree_list(&self) -> CommandOutput {
        let output = Command::new("git").args(["worktree", "list"]).output();
        Self::map_git_output(output, |stdout| {
            let mut table = String::from("Worktrees:\n");
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let path = parts[0];
                    let commit = &parts[1][..7.min(parts[1].len())];
                    let branch = parts[2..].join(" ");
                    table.push_str(&format!("  {path:42} {commit} {branch}\n"));
                } else {
                    table.push_str(&format!("  {line}\n"));
                }
            }
            table
        })
    }

    fn worktree_create(&self, args: &[String]) -> CommandOutput {
        let branch = args.first().map(|s| s.as_str()).unwrap_or("");
        if branch.is_empty() {
            return CommandOutput::err("用法: /git worktree create <branch> [<path>]");
        }
        let path = args.get(1).map(|s| s.as_str()).unwrap_or(branch);
        let output = Command::new("git")
            .args(["worktree", "add", "-b", branch, path])
            .output();
        Self::map_git_output(output, |stdout| {
            format!("✅ 工作树已创建: {path} (分支: {branch})\n{stdout}")
        })
    }

    fn worktree_remove(&self, args: &[String]) -> CommandOutput {
        let path = args.first().map(|s| s.as_str()).unwrap_or("");
        if path.is_empty() {
            return CommandOutput::err("用法: /git worktree remove <path>");
        }
        let output = Command::new("git").args(["worktree", "remove", path]).output();
        Self::map_git_output(output, |stdout| {
            format!("🗑️  工作树已移除: {path}\n{stdout}")
        })
    }

    fn worktree_pr(&self, args: &[String]) -> CommandOutput {
        let pr_number = args.first().map(|s| s.as_str()).unwrap_or("");
        if pr_number.is_empty() {
            return CommandOutput::err("用法: /git worktree pr <pr-number> [<path>]");
        }
        let gh_check = Command::new("gh").args(["--version"]).output();
        if gh_check.is_err() {
            return CommandOutput::err("gh CLI 未安装。安装: brew install gh && gh auth login");
        }
        let branch_json = Command::new("gh")
            .args(["pr", "view", pr_number, "--json", "headRefName,headRepository"])
            .output();
        let branch_info = match branch_json {
            Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
            _ => return CommandOutput::err(&format!("无法获取 PR #{pr_number} 信息。请确认 PR 存在。")),
        };
        let branch_name = match serde_json::from_str::<serde_json::Value>(&branch_info) {
            Ok(v) => v["headRefName"].as_str().unwrap_or("").to_string(),
            Err(_) => return CommandOutput::err("解析 PR 信息失败"),
        };
        if branch_name.is_empty() {
            return CommandOutput::err("PR 分支名为空");
        }
        let worktree_path = args.get(1).map(|s| s.as_str()).unwrap_or(&branch_name);
        let fetch_out = Command::new("git")
            .args(["fetch", "origin", &format!("pull/{pr_number}/head:{branch_name}")])
            .output();
        if fetch_out.as_ref().map(|o| !o.status.success()).unwrap_or(true) {
            let _ = fetch_out;
            return CommandOutput::err("fetch PR 分支失败");
        }
        let output = Command::new("git")
            .args(["worktree", "add", worktree_path, &branch_name])
            .output();
        Self::map_git_output(output, |stdout| {
            format!("✅ PR #{pr_number} 工作树已创建: {worktree_path} (分支: {branch_name})\n{stdout}")
        })
    }

    fn worktree_lock(&self, args: &[String]) -> CommandOutput {
        let path = args.first().map(|s| s.as_str()).unwrap_or("");
        if path.is_empty() {
            return CommandOutput::err("用法: /git worktree lock <path>");
        }
        let output = Command::new("git").args(["worktree", "lock", path]).output();
        Self::map_git_output(output, |stdout| format!("🔒 已锁定: {path}\n{stdout}"))
    }

    fn worktree_unlock(&self, args: &[String]) -> CommandOutput {
        let path = args.first().map(|s| s.as_str()).unwrap_or("");
        if path.is_empty() {
            return CommandOutput::err("用法: /git worktree unlock <path>");
        }
        let output = Command::new("git").args(["worktree", "unlock", path]).output();
        Self::map_git_output(output, |stdout| format!("🔓 已解锁: {path}\n{stdout}"))
    }

    fn map_git_output(result: Result<std::process::Output, std::io::Error>, f: impl Fn(&str) -> String) -> CommandOutput {
        match result {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let combined = if stdout.is_empty() { stderr.trim().to_string() } else { stdout };
                if combined.is_empty() {
                    CommandOutput::ok("(完成)")
                } else if out.status.success() {
                    CommandOutput::ok(&f(&combined))
                } else {
                    CommandOutput::err(&combined)
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CommandOutput::err("git 未安装")
                } else {
                    CommandOutput::err(&format!("git worktree 执行失败: {e}"))
                }
            }
        }
    }
}

// ====== /commit ======

pub struct CommitCmd;
impl CliCommand for CommitCmd {
    fn name(&self) -> &str { "/commit" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str { "暂存全部并提交: /commit <message>" }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::err("用法: /commit <message>");
        }
        let msg = args.join(" ");

        let add_result = Command::new("git").args(["add", "-A"]).output();
        let _ = match add_result {
            Ok(out) => {
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    return CommandOutput::err(&format!("git add 失败: {}", stderr.trim()));
                }
                out
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return CommandOutput::err("git 未安装");
                }
                return CommandOutput::err(&format!("git add 失败: {}", e));
            }
        };

        let commit = Command::new("git").args(["commit", "-m", &msg]).output();
        match commit {
            Ok(out) => {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    CommandOutput::ok(&format!("✅ {}", stdout.trim()))
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let stderr_str = stderr.trim();
                    if stderr_str.contains("nothing to commit") {
                        CommandOutput::ok("✅ 无变更需要提交")
                    } else {
                        CommandOutput::err(&format!("commit 失败: {}", stderr_str))
                    }
                }
            }
            Err(e) => CommandOutput::err(&format!("commit 失败: {}", e)),
        }
    }
}

// ====== /pr ======

pub struct PrCmd;
impl CliCommand for PrCmd {
    fn name(&self) -> &str { "/pr" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str { "创建 GitHub PR (使用 gh CLI)" }
    fn execute(&self, _args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let gh_check = Command::new("gh").args(["--version"]).output();
        match gh_check {
            Ok(_) => {
                let output = Command::new("gh").args(["pr", "create", "--fill"]).output();
                match output {
                    Ok(out) => {
                        if out.status.success() {
                            let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
                            CommandOutput::ok(&format!("✅ PR 已创建: {}", url))
                        } else {
                            let stderr = String::from_utf8_lossy(&out.stderr);
                            CommandOutput::err(&format!("PR 创建失败: {}", stderr.trim()))
                        }
                    }
                    Err(e) => CommandOutput::err(&format!("gh 执行失败: {}", e)),
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    CommandOutput::err("gh CLI 未安装。安装: brew install gh && gh auth login")
                } else {
                    CommandOutput::err(&format!("gh 检查失败: {}", e))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_worktree_no_args() {
        let cmd = GitCmd;
        // worktree with no subcommand should show help
        let result = cmd.handle_worktree(&[]);
        assert!(result.success);
        assert!(result.message.contains("create"));
        assert!(result.message.contains("list"));
        assert!(result.message.contains("remove"));
        assert!(result.message.contains("pr"));
    }

    #[test]
    fn test_git_worktree_unknown_subcmd() {
        let cmd = GitCmd;
        let result = cmd.handle_worktree(&["nonexistent".to_string()]);
        assert!(!result.success);
        assert!(result.message.contains("未知"));
    }

    #[test]
    fn test_git_worktree_create_no_args() {
        let cmd = GitCmd;
        let result = cmd.worktree_create(&[]);
        assert!(!result.success);
        assert!(result.message.contains("用法"));
        assert!(result.message.contains("create"));
    }

    #[test]
    fn test_git_worktree_create_with_branch() {
        let cmd = GitCmd;
        // This may fail if not in a git repo, but should run without panic
        let _result = cmd.worktree_create(&["test-worktree-branch".to_string()]);
        // Not asserting pass/fail since it depends on git repo state
    }

    #[test]
    fn test_git_worktree_remove_no_args() {
        let cmd = GitCmd;
        let result = cmd.worktree_remove(&[]);
        assert!(!result.success);
        assert!(result.message.contains("用法"));
    }

    #[test]
    fn test_git_worktree_lock_unlock_no_args() {
        let cmd = GitCmd;
        let r1 = cmd.worktree_lock(&[]);
        assert!(!r1.success);
        let r2 = cmd.worktree_unlock(&[]);
        assert!(!r2.success);
    }

    #[test]
    fn test_git_worktree_pr_no_args() {
        let cmd = GitCmd;
        let result = cmd.worktree_pr(&[]);
        assert!(!result.success);
        assert!(result.message.contains("用法"));
    }

    #[test]
    fn test_git_execute_worktree_unknown() {
        let cmd = GitCmd;
        let result = cmd.execute(&["worktree".to_string(), "bogus".to_string()], None);
        assert!(!result.success);
        assert!(result.message.contains("未知"));
    }
}
