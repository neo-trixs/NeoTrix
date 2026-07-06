use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Worktree {
    pub path: PathBuf,
    pub branch: String,
    pub created_at: String,
}

#[derive(Debug)]
pub struct WorktreeManager {
    pub base_dir: PathBuf,
    pub git_dir: PathBuf,
}

impl WorktreeManager {
    pub fn new(base_dir: PathBuf, git_dir: PathBuf) -> Self {
        Self { base_dir, git_dir }
    }

    pub fn create(&self, branch: &str, base_branch: &str) -> Result<Worktree, String> {
        let git_dir_str = self.git_dir.to_str().ok_or_else(|| format!("invalid UTF-8 in git_dir: {:?}", self.git_dir))?;
        let branch_result = std::process::Command::new("git")
            .args(["-C", git_dir_str])
            .args(["branch", branch, format!("origin/{}", base_branch).as_str()])
            .output();
        let _ = branch_result;

        let worktree_path = self.base_dir.join(format!("wt-{}", branch));
        let wt_path_str = worktree_path.to_str().ok_or_else(|| format!("invalid UTF-8 in worktree path: {:?}", worktree_path))?;
        let wt_result = std::process::Command::new("git")
            .args(["-C", git_dir_str])
            .args(["worktree", "add", wt_path_str, branch])
            .output();

        match wt_result {
            Ok(out) if out.status.success() => Ok(Worktree {
                path: worktree_path,
                branch: branch.to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            }),
            Ok(out) => Err(format!("git worktree add failed: {}", String::from_utf8_lossy(&out.stderr))),
            Err(e) => Err(format!("git worktree add error: {}", e)),
        }
    }

    pub fn remove(&self, worktree: &Worktree) -> Result<(), String> {
        let git_dir_str = self.git_dir.to_str().ok_or_else(|| format!("invalid UTF-8 in git_dir: {:?}", self.git_dir))?;
        let wt_path_str = worktree.path.to_str().ok_or_else(|| format!("invalid UTF-8 in worktree path: {:?}", worktree.path))?;
        let result = std::process::Command::new("git")
            .args(["-C", git_dir_str])
            .args(["worktree", "remove", wt_path_str])
            .output();

        match result {
            Ok(out) if out.status.success() => {
                let _ = std::process::Command::new("git")
                    .args(["-C", git_dir_str])
                    .args(["branch", "-D", &worktree.branch])
                    .output();
                Ok(())
            }
            Ok(out) => Err(format!("git worktree remove failed: {}", String::from_utf8_lossy(&out.stderr))),
            Err(e) => Err(format!("git worktree remove error: {}", e)),
        }
    }

    pub fn list_active(&self) -> Result<Vec<Worktree>, String> {
        let git_dir_str = self.git_dir.to_str().ok_or_else(|| format!("invalid UTF-8 in git_dir: {:?}", self.git_dir))?;
        let result = std::process::Command::new("git")
            .args(["-C", git_dir_str])
            .args(["worktree", "list"])
            .output();

        match result {
            Ok(out) if out.status.success() => {
                let output = String::from_utf8_lossy(&out.stdout);
                let trees = output.lines().filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        Some(Worktree {
                            path: PathBuf::from(parts[0]),
                            branch: parts[1].trim_start_matches('[').trim_end_matches(']').to_string(),
                            created_at: String::new(),
                        })
                    } else {
                        None
                    }
                }).collect();
                Ok(trees)
            }
            Ok(out) => Err(format!("git worktree list failed: {}", String::from_utf8_lossy(&out.stderr))),
            Err(e) => Err(format!("git worktree list error: {}", e)),
        }
    }

    pub fn prune_orphaned(&self) -> Result<u32, String> {
        let git_dir_str = self.git_dir.to_str().ok_or_else(|| format!("invalid UTF-8 in git_dir: {:?}", self.git_dir))?;
        let result = std::process::Command::new("git")
            .args(["-C", git_dir_str])
            .args(["worktree", "prune"])
            .output();

        match result {
            Ok(out) if out.status.success() => {
                let _ = std::process::Command::new("git")
                    .args(["-C", git_dir_str])
                    .args(["worktree", "list", "--porcelain"])
                    .output();
                Ok(0)
            }
            Ok(out) => Err(format!("git worktree prune failed: {}", String::from_utf8_lossy(&out.stderr))),
            Err(e) => Err(format!("git worktree prune error: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_manager_creation() {
        let base = PathBuf::from("/tmp/neotrix-test-wt");
        let git = PathBuf::from("/tmp/neotrix-test-repo/.git");
        let mgr = WorktreeManager::new(base, git);
        assert_eq!(mgr.base_dir.to_str().expect("value should be ok in test"), "/tmp/neotrix-test-wt");
    }
}
