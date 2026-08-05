use std::path::Path;
use std::process::Command;

use super::types::GitIntegration;

impl GitIntegration {
    pub fn is_git_repo(path: &Path) -> bool {
        path.join(".git").exists()
    }

    pub fn current_branch(path: &Path) -> Result<String, String> {
        let output = Command::new("git")
            .args(["-C", &path.to_string_lossy(), "branch", "--show-current"])
            .output()
            .map_err(|e| format!("git command error: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git branch --show-current failed: {}", stderr));
        }
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if branch.is_empty() {
            return Err("not on any branch (detached HEAD?)".to_string());
        }
        Ok(branch)
    }

    pub fn list_branches(path: &Path) -> Result<Vec<String>, String> {
        let output = Command::new("git")
            .args(["-C", &path.to_string_lossy(), "branch"])
            .output()
            .map_err(|e| format!("git command error: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git branch failed: {}", stderr));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .map(|l| {
                l.trim_start_matches('*')
                    .trim_start_matches(' ')
                    .trim()
                    .to_string()
            })
            .filter(|s| !s.is_empty())
            .collect())
    }

    pub fn create_branch(path: &Path, name: &str) -> Result<(), String> {
        let output = Command::new("git")
            .args(["-C", &path.to_string_lossy(), "checkout", "-b", name])
            .output()
            .map_err(|e| format!("git command error: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git checkout -b failed: {}", stderr));
        }
        Ok(())
    }

    pub fn switch_branch(path: &Path, name: &str) -> Result<(), String> {
        let output = Command::new("git")
            .args(["-C", &path.to_string_lossy(), "checkout", name])
            .output()
            .map_err(|e| format!("git command error: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git checkout failed: {}", stderr));
        }
        Ok(())
    }

    pub fn worktree_add(path: &Path, worktree_path: &Path, branch: &str) -> Result<(), String> {
        let output = Command::new("git")
            .args([
                "-C",
                &path.to_string_lossy(),
                "worktree",
                "add",
                &worktree_path.to_string_lossy(),
                branch,
            ])
            .output()
            .map_err(|e| format!("git command error: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git worktree add failed: {}", stderr));
        }
        Ok(())
    }

    pub fn diff_staged(path: &Path) -> Result<String, String> {
        let output = Command::new("git")
            .args(["-C", &path.to_string_lossy(), "diff", "--cached"])
            .output()
            .map_err(|e| format!("git command error: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git diff --cached failed: {}", stderr));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn diff_unstaged(path: &Path) -> Result<String, String> {
        let output = Command::new("git")
            .args(["-C", &path.to_string_lossy(), "diff"])
            .output()
            .map_err(|e| format!("git command error: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git diff failed: {}", stderr));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn commit(path: &Path, message: &str) -> Result<(), String> {
        let output = Command::new("git")
            .args(["-C", &path.to_string_lossy(), "commit", "-m", message])
            .output()
            .map_err(|e| format!("git command error: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git commit failed: {}", stderr));
        }
        Ok(())
    }

    pub fn generate_commit_message(diff: &str) -> String {
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut deleted = Vec::new();

        for line in diff.lines() {
            if let Some(file) = line.strip_prefix("+++ b/") {
                modified.push(file.to_string());
            }
            if let Some(_file) = line.strip_prefix("--- /dev/null") {
                if let Some(next) = diff.lines().skip_while(|l| l != &line).nth(1) {
                    if let Some(f) = next.strip_prefix("+++ b/") {
                        added.push(f.to_string());
                    }
                }
            }
            if let Some(f) = line.strip_prefix("--- a/") {
                if let Some(file) = f.strip_suffix('\t').or(Some(f)) {
                    deleted.push(file.to_string());
                }
            }
        }

        let mut parts: Vec<String> = Vec::new();
        if !added.is_empty() {
            parts.push(format!("add: {}", added.join(", ")));
        }
        if !modified.is_empty() {
            parts.push(format!("update: {}", modified.join(", ")));
        }
        if !deleted.is_empty() {
            parts.push(format!("remove: {}", deleted.join(", ")));
        }
        if parts.is_empty() {
            "chore: miscellaneous changes".to_string()
        } else {
            parts.join("; ")
        }
    }
}
