//! Git Hook — Git 钩子工具
//!
//! 移植自 skills/dev-tools/git-hook/git-hook.sh
//! 支持 pre-commit 钩子，检查 TODO.md 和 TODO.yml 同步

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Git 钩子
pub struct GitHook {
    root: PathBuf,
}

impl GitHook {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    /// 执行 pre-commit 钩子
    pub fn pre_commit(&self) -> Result<String, String> {
        let todo_md = self.root.join("TODO.md");
        let todo_yml = self.root.join("TODO.yml");

        // 检查 TODO.md 和 TODO.yml 是否同时被修改但不同步
        let output = Command::new("git")
            .args(&["diff", "--cached", "--name-only"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| format!("Failed to run git diff: {}", e))?;

        if output.status.success() {
            let changed_files = String::from_utf8_lossy(&output.stdout);
            if changed_files.contains("TODO.md") || changed_files.contains("TODO.yml") {
                if todo_md.exists() && todo_yml.exists() {
                    // 简单校验：TODO.md 的 [ ] 数量与 TODO.yml 的 pending 总量大致对齐
                    let todo_content = fs::read_to_string(&todo_md)
                        .map_err(|e| format!("Failed to read TODO.md: {}", e))?;
                    let yml_content = fs::read_to_string(&todo_yml)
                        .map_err(|e| format!("Failed to read TODO.yml: {}", e))?;

                    let todo_count = todo_content.matches("[ ]").count();
                    let pending_count = yml_content.matches("status: pending").count();

                    if todo_count > 0 && pending_count == 0 {
                        return Ok(format!(
                            "[GIT-HOOK] 警告：TODO.md 有 {} 项待办，但 TODO.yml 无 pending 项\n\
                             [GIT-HOOK] 请先运行 `neotrix todo sync` 同步后再提交",
                            todo_count
                        ));
                    }
                }
            }
        }

        // 运行同步
        let output = Command::new("neotrix")
            .args(&["todo", "sync"])
            .current_dir(&self.root)
            .output()
            .ok();

        if let Some(output) = output {
            if output.status.success() {
                Ok("同步完成".to_string())
            } else {
                Ok("同步失败（静默跳过）".to_string())
            }
        } else {
            Ok("neotrix 未安装（静默跳过）".to_string())
        }
    }

    /// 安装 git 钩子
    pub fn install(&self) -> Result<String, String> {
        let hooks_dir = self.root.join(".git/hooks");
        if !hooks_dir.exists() {
            return Err("Git hooks directory not found".to_string());
        }

        let hook_script = self.root.join("skills/dev-tools/git-hook/no-main-direct.sh");
        if !hook_script.exists() {
            return Err("Hook script not found".to_string());
        }

        // 创建 pre-commit 链接
        let pre_commit = hooks_dir.join("pre-commit");
        if pre_commit.exists() {
            fs::remove_file(&pre_commit)
                .map_err(|e| format!("Failed to remove existing pre-commit: {}", e))?;
        }

        std::os::unix::fs::symlink(&hook_script, &pre_commit)
            .map_err(|e| format!("Failed to create pre-commit symlink: {}", e))?;

        // 创建 pre-push 链接
        let pre_push = hooks_dir.join("pre-push");
        if pre_push.exists() {
            fs::remove_file(&pre_push)
                .map_err(|e| format!("Failed to remove existing pre-push: {}", e))?;
        }

        std::os::unix::fs::symlink(&hook_script, &pre_push)
            .map_err(|e| format!("Failed to create pre-push symlink: {}", e))?;

        Ok("Git hooks installed".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_git_hook_new() {
        let hook = GitHook::new(&PathBuf::from("."));
        assert_eq!(hook.root, PathBuf::from("."));
    }
}
