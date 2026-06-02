use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Manages isolated git worktrees for parallel agent execution
pub struct WorktreeManager {
    base_dir: PathBuf,
    worktrees: Vec<Worktree>,
}

#[derive(Clone)]
pub struct Worktree {
    pub name: String,
    pub path: PathBuf,
    pub branch: String,
    pub created_at: u64,
    pub is_active: bool,
}

impl WorktreeManager {
    /// Create a new WorktreeManager rooted at the given directory
    pub fn new(base_dir: &Path) -> Self {
        WorktreeManager {
            base_dir: base_dir.to_path_buf(),
            worktrees: Vec::new(),
        }
    }

    /// Create a new worktree for the given feature/branch name.
    /// Runs: git branch <name> <base_branch> && git worktree add ../<name> <name>
    pub fn create(&mut self, name: &str, base_branch: &str) -> Result<Worktree, String> {
        let root_path = self.get_root()?;
        let parent = root_path.parent().unwrap_or(&root_path);
        let worktree_path = parent.join(name);

        let branch_out = Command::new("git")
            .args(["branch", name, base_branch])
            .current_dir(&root_path)
            .output()
            .map_err(|e| format!("Failed to run git branch: {}", e))?;
        if !branch_out.status.success() {
            let err = String::from_utf8_lossy(&branch_out.stderr);
            return Err(format!(
                "Failed to create branch '{}' from '{}': {}",
                name, base_branch, err
            ));
        }

        let wt_out = Command::new("git")
            .args([
                "worktree",
                "add",
                &worktree_path.to_string_lossy(),
                name,
            ])
            .current_dir(&root_path)
            .output()
            .map_err(|e| format!("Failed to run git worktree add: {}", e))?;
        if !wt_out.status.success() {
            let err = String::from_utf8_lossy(&wt_out.stderr);
            let _ = Command::new("git")
                .args(["branch", "-D", name])
                .current_dir(&root_path)
                .output();
            return Err(format!("Failed to add worktree '{}': {}", name, err));
        }

        let worktree = Worktree {
            name: name.to_string(),
            path: worktree_path,
            branch: name.to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            is_active: true,
        };
        self.worktrees.push(worktree.clone());
        Ok(worktree)
    }

    /// Create from an existing branch (does not create a new branch)
    pub fn create_from_branch(&mut self, name: &str, branch: &str) -> Result<Worktree, String> {
        let root_path = self.get_root()?;
        let parent = root_path.parent().unwrap_or(&root_path);
        let worktree_path = parent.join(name);

        let out = Command::new("git")
            .args([
                "worktree",
                "add",
                &worktree_path.to_string_lossy(),
                branch,
            ])
            .current_dir(&root_path)
            .output()
            .map_err(|e| format!("Failed to run git worktree add: {}", e))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(format!(
                "Failed to add worktree '{}' from branch '{}': {}",
                name, branch, err
            ));
        }

        let worktree = Worktree {
            name: name.to_string(),
            path: worktree_path,
            branch: branch.to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            is_active: true,
        };
        self.worktrees.push(worktree.clone());
        Ok(worktree)
    }

    /// Remove a worktree (worktree + branch cleanup)
    pub fn remove(&mut self, name: &str) -> Result<(), String> {
        let root_path = self.get_root()?;

        let out = Command::new("git")
            .args(["worktree", "remove", "--force", name])
            .current_dir(&root_path)
            .output()
            .map_err(|e| format!("Failed to run git worktree remove: {}", e))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(format!("Failed to remove worktree '{}': {}", name, err));
        }

        let _ = Command::new("git")
            .args(["branch", "-D", name])
            .current_dir(&root_path)
            .output();

        self.worktrees.retain(|w| w.name != name);
        Ok(())
    }

    /// List all managed worktrees
    pub fn list(&self) -> &[Worktree] {
        &self.worktrees
    }

    /// Get a specific worktree by name
    pub fn get(&self, name: &str) -> Option<&Worktree> {
        self.worktrees.iter().find(|w| w.name == name)
    }

    /// Run a command inside a worktree's directory
    pub fn run_in(
        &self,
        name: &str,
        command: &str,
        args: &[&str],
    ) -> Result<String, String> {
        let wt = self
            .get(name)
            .ok_or_else(|| format!("Worktree '{}' not found", name))?;
        let out = Command::new(command)
            .args(args)
            .current_dir(&wt.path)
            .output()
            .map_err(|e| {
                format!("Failed to run '{}' in worktree '{}': {}", command, name, e)
            })?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).to_string())
        } else {
            let err = String::from_utf8_lossy(&out.stderr);
            Err(format!("Command '{}' failed in '{}': {}", command, name, err))
        }
    }

    /// Get diff between worktree HEAD and working tree
    pub fn diff(&self, name: &str) -> Result<String, String> {
        let wt = self
            .get(name)
            .ok_or_else(|| format!("Worktree '{}' not found", name))?;
        let out = Command::new("git")
            .args(["diff", "HEAD"])
            .current_dir(&wt.path)
            .output()
            .map_err(|e| format!("Failed to get diff for '{}': {}", name, e))?;
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    /// Stage all and commit in a worktree
    pub fn commit(&self, name: &str, message: &str) -> Result<(), String> {
        let wt = self
            .get(name)
            .ok_or_else(|| format!("Worktree '{}' not found", name))?;

        let add = Command::new("git")
            .args(["add", "-A"])
            .current_dir(&wt.path)
            .output()
            .map_err(|e| format!("Failed to stage in '{}': {}", name, e))?;
        if !add.status.success() {
            let err = String::from_utf8_lossy(&add.stderr);
            return Err(format!("Failed to stage in '{}': {}", name, err));
        }

        let cm = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&wt.path)
            .output()
            .map_err(|e| format!("Failed to commit in '{}': {}", name, e))?;
        if !cm.status.success() {
            let err = String::from_utf8_lossy(&cm.stderr);
            return Err(format!("Failed to commit in '{}': {}", name, err));
        }
        Ok(())
    }

    /// Clean up all stale worktrees (prune) and clear internal list
    pub fn prune_all(&mut self) -> Result<(), String> {
        let root_path = self.get_root()?;
        let out = Command::new("git")
            .args(["worktree", "prune"])
            .current_dir(&root_path)
            .output()
            .map_err(|e| format!("Failed to prune worktrees: {}", e))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(format!("Failed to prune: {}", err));
        }
        self.worktrees.clear();
        Ok(())
    }

    /// Number of active worktrees
    pub fn active_count(&self) -> usize {
        self.worktrees.iter().filter(|w| w.is_active).count()
    }

    fn get_root(&self) -> Result<PathBuf, String> {
        let out = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(&self.base_dir)
            .output()
            .map_err(|e| format!("Failed to find git root: {}", e))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(format!("Not inside a git repository: {}", err));
        }
        let root = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Ok(PathBuf::from(root))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static WT_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn unique_name(base: &str) -> String {
        let id = WT_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("{}-{}", base, id)
    }

    fn cleanup_stale_worktree(path: &Path, name: &str) {
        let stale = path.parent().unwrap().join(name);
        if stale.exists() {
            let _ = std::fs::remove_dir_all(&stale);
        }
    }

    fn setup_repo() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let path = dir.path().to_path_buf();

        Command::new("git")
            .args(["init", "--initial-branch=main"])
            .current_dir(&path)
            .output()
            .expect("failed to init git repo");
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&path)
            .output()
            .expect("failed to set git email");
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&path)
            .output()
            .expect("failed to set git user name");

        std::fs::write(path.join("README.md"), "# Test\n").expect("failed to write README");
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(&path)
            .output()
            .expect("failed to git add");
        Command::new("git")
            .args(["commit", "-m", "Initial"])
            .current_dir(&path)
            .output()
            .expect("failed to git commit");

        (dir, path)
    }

#[test]
fn test_create_and_list() {
        let (_keep, path) = setup_repo();
        let mut mgr = WorktreeManager::new(&path);
        let wt_name = unique_name("feature");
        cleanup_stale_worktree(&path, &wt_name);
        let wt = mgr.create(&wt_name, "main").expect("failed to create worktree");
        assert_eq!(wt.name, wt_name);
        assert!(wt.is_active);
        assert_eq!(mgr.list().len(), 1);
        assert_eq!(mgr.active_count(), 1);
    }

    #[test]
    fn test_create_twice_fails() {
        let (_keep, path) = setup_repo();
        let mut mgr = WorktreeManager::new(&path);
        let wt_name = unique_name("dup-test");
        cleanup_stale_worktree(&path, &wt_name);
        mgr.create(&wt_name, "main").expect("failed to create worktree");
        let dup = mgr.create(&wt_name, "main");
        assert!(dup.is_err());
    }

    #[test]
    fn test_remove_worktree() {
        let (_keep, path) = setup_repo();
        let mut mgr = WorktreeManager::new(&path);
        let wt_name = unique_name("to-remove");
        cleanup_stale_worktree(&path, &wt_name);
        mgr.create(&wt_name, "main").expect("failed to create worktree");
        mgr.remove(&wt_name).expect("failed to remove worktree");
        assert!(mgr.get(&wt_name).is_none());
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn test_get_nonexistent() {
        let (_keep, path) = setup_repo();
        let mgr = WorktreeManager::new(&path);
        assert!(mgr.get(&unique_name("ghost")).is_none());
    }

    #[test]
    fn test_run_in_worktree() {
        let (_keep, path) = setup_repo();
        let mut mgr = WorktreeManager::new(&path);
        let wt_name = unique_name("runner");
        cleanup_stale_worktree(&path, &wt_name);
        mgr.create(&wt_name, "main").expect("failed to create worktree");
        let out = mgr
            .run_in(&wt_name, "git", &["rev-parse", "--show-toplevel"])
            .expect("failed to run in worktree");
        assert!(out.contains(&wt_name));
    }

    #[test]
    fn test_create_from_branch() {
        let (_keep, path) = setup_repo();
        let mut mgr = WorktreeManager::new(&path);
        let branch_name = unique_name("release");
        Command::new("git")
            .args(["checkout", "-b", &branch_name])
            .current_dir(&path)
            .output()
            .expect("failed to checkout branch");
        Command::new("git")
            .args(["checkout", "main"])
            .current_dir(&path)
            .output()
            .expect("failed to checkout main");

        let wt_name = unique_name("release-wt");
        cleanup_stale_worktree(&path, &wt_name);
        let wt = mgr.create_from_branch(&wt_name, &branch_name).expect("failed to create worktree from branch");
        assert_eq!(wt.branch, branch_name);
        assert_eq!(mgr.list().len(), 1);
    }

    #[test]
    fn test_prune_all() {
        let (_keep, path) = setup_repo();
        let mut mgr = WorktreeManager::new(&path);
        let p1 = unique_name("p1");
        let p2 = unique_name("p2");
        cleanup_stale_worktree(&path, &p1);
        cleanup_stale_worktree(&path, &p2);
        mgr.create(&p1, "main").expect("failed to create worktree");
        mgr.create(&p2, "main").expect("failed to create worktree");
        assert_eq!(mgr.list().len(), 2);
        mgr.prune_all().expect("failed to prune worktrees");
        assert_eq!(mgr.list().len(), 0);
    }

    #[test]
    fn test_commit_and_diff() {
        let (_keep, path) = setup_repo();
        let mut mgr = WorktreeManager::new(&path);
        let wt_name = unique_name("cd");
        cleanup_stale_worktree(&path, &wt_name);
        mgr.create(&wt_name, "main").expect("failed to create worktree");

        let wt = mgr.get(&wt_name).expect("worktree not found");
        std::fs::write(wt.path.join("new_file.txt"), "hello").expect("failed to write test file");
        Command::new("git")
            .args(["add", "new_file.txt"])
            .current_dir(&wt.path)
            .output()
            .expect("failed to git add in worktree");

        let diff = mgr.diff(&wt_name).expect("failed to get diff");
        assert!(diff.contains("new_file.txt") || diff.contains("hello"));

        mgr.commit(&wt_name, "add new_file.txt").expect("failed to commit");

        let clean_diff = mgr.diff(&wt_name).expect("failed to get diff");
        assert!(clean_diff.is_empty());
    }
}
