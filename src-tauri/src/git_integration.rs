use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub last_commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub file: String,
    pub added: u32,
    pub removed: u32,
    pub hunks: u32,
}

fn run_git(args: &[&str], cwd: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("Git command failed: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git error: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ========== D-302a: Git Worktree ==========

pub fn worktree_create(branch: &str, path: &str, repo_path: &str) -> Result<String, String> {
    let repo = Path::new(repo_path);
    let target = Path::new(path);
    let worktree_path = if target.is_absolute() {
        target.to_path_buf()
    } else {
        repo.join(target)
    };

    let exists = run_git(&["worktree", "list", "--porcelain"], repo_path)?;
    if exists.contains(&worktree_path.to_string_lossy().to_string()) {
        return Err(format!("Worktree already exists at {}", worktree_path.display()));
    }

    run_git(
        &["worktree", "add", &worktree_path.to_string_lossy(), branch],
        repo_path,
    )?;
    Ok(format!(
        "Created worktree at {} for branch {}",
        worktree_path.display(),
        branch
    ))
}

pub fn worktree_list(repo_path: &str) -> Result<Vec<WorktreeInfo>, String> {
    let output = run_git(&["worktree", "list", "--porcelain"], repo_path)?;
    let mut worktrees = Vec::new();
    let mut current = WorktreeInfo {
        path: String::new(),
        branch: String::new(),
        hash: String::new(),
    };

    for line in output.lines() {
        if line.starts_with("worktree ") {
            if !current.path.is_empty() {
                worktrees.push(WorktreeInfo {
                    path: std::mem::take(&mut current.path),
                    branch: std::mem::take(&mut current.branch),
                    hash: std::mem::take(&mut current.hash),
                });
            }
            current.path = line.strip_prefix("worktree ").unwrap_or("").to_string();
        } else if line.starts_with("HEAD ") {
            current.hash = line.strip_prefix("HEAD ").unwrap_or("").to_string();
        } else if line.starts_with("branch ") {
            let ref_str = line.strip_prefix("branch ").unwrap_or("");
            current.branch = ref_str
                .strip_prefix("refs/heads/")
                .unwrap_or(ref_str)
                .to_string();
        }
    }
    if !current.path.is_empty() {
        worktrees.push(current);
    }

    Ok(worktrees)
}

pub fn worktree_remove(worktree_path: &str, repo_path: &str) -> Result<(), String> {
    run_git(&["worktree", "remove", worktree_path], repo_path)?;
    Ok(())
}

// ========== D-302b: Branch Management ==========

pub fn branch_list(repo_path: &str) -> Result<Vec<BranchInfo>, String> {
    let output = run_git(
        &["branch", "--all", "--format=%(refname:short)|%(HEAD)|%(upstream:track)"],
        repo_path,
    )?;

    let mut branches = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        let name = parts.first().unwrap_or(&"").to_string();
        let is_head = parts.get(1).copied().unwrap_or("") == "*";
        let is_remote = name.starts_with("remotes/");

        let commit_output = run_git(
            &["log", "--oneline", "-1", &name, "--format=%s"],
            repo_path,
        )
        .unwrap_or_default();

        branches.push(BranchInfo {
            name,
            is_current: is_head,
            is_remote,
            last_commit: commit_output,
        });
    }

    Ok(branches)
}

pub fn branch_create(name: &str, from_branch: &str, repo_path: &str) -> Result<(), String> {
    run_git(&["branch", name, from_branch], repo_path)?;
    Ok(())
}

pub fn branch_switch(path: &str, name: &str) -> Result<(), String> {
    run_git(&["checkout", name], path)?;
    Ok(())
}

pub fn branch_diff(repo_path: &str, base: &str, head: &str) -> Result<Vec<DiffSummary>, String> {
    let output = run_git(
        &["diff", "--stat", "--numstat", &format!("{}..{}", base, head)],
        repo_path,
    )?;

    let mut diffs = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let added: u32 = parts[0].parse().unwrap_or(0);
            let removed: u32 = parts[1].parse().unwrap_or(0);
            let file = parts[2..].join("\t");

            let hunk_count = count_hunks(repo_path, base, head, &file);

            diffs.push(DiffSummary {
                file,
                added,
                removed,
                hunks: hunk_count,
            });
        }
    }

    Ok(diffs)
}

fn count_hunks(repo_path: &str, base: &str, head: &str, file: &str) -> u32 {
    let output = run_git(
        &[
            "diff",
            "--unified=0",
            &format!("{}..{}", base, head),
            "--",
            file,
        ],
        repo_path,
    );
    match output {
        Ok(diff) => diff.lines().filter(|l| l.starts_with("@@")).count() as u32,
        Err(_) => 0,
    }
}

// ========== D-302c: AI Commit Messages ==========

pub fn generate_commit_message(path: &str) -> Result<String, String> {
    let staged = run_git(&["diff", "--cached", "--stat"], path).unwrap_or_default();
    let is_staged = !staged.is_empty();
    let diff_stat = if is_staged {
        staged.clone()
    } else {
        run_git(&["diff", "HEAD", "--stat"], path)?
    };

    if diff_stat.is_empty() {
        return Err("No changes to commit".to_string());
    }

    let flag = if is_staged { "--cached" } else { "HEAD" };
    let diff_files = run_git(&["diff", flag, "--name-status"], path)
        .unwrap_or_default();

    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut deleted = Vec::new();
    let mut total_files = 0u32;

    for line in diff_files.lines() {
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        if parts.len() < 2 {
            continue;
        }
        total_files += 1;
        let file = parts[1].to_string();
        match parts[0] {
            "A" | "C" | "R" => added.push(file),
            "D" => deleted.push(file),
            _ => modified.push(file),
        }
    }

    let msg = build_message(&added, &modified, &deleted, total_files);
    Ok(msg)
}

fn build_message(added: &[String], modified: &[String], deleted: &[String], total: u32) -> String {
    let mut parts = Vec::new();

    if !added.is_empty() {
        let files = summarize_files(added);
        parts.push(format!("add {}", files));
    }
    if !modified.is_empty() {
        let files = summarize_files(modified);
        parts.push(format!("update {}", files));
    }
    if !deleted.is_empty() {
        let files = summarize_files(deleted);
        parts.push(format!("remove {}", files));
    }

    if parts.is_empty() {
        return format!("update ({} files changed)", total);
    }

    let msg = parts.join("; ");
    if msg.len() > 72 {
        if let Some(first) = parts.first() {
            let rest_count = total as usize - count_files_in_part(first);
            if rest_count > 0 {
                return format!("{} +{} more", first, rest_count);
            }
            return first.clone();
        }
    }

    msg
}

fn summarize_files(files: &[String]) -> String {
    if files.len() == 1 {
        return files[0].clone();
    }
    let dirs = group_by_directory(files);
    if dirs.len() == 1 && files.len() <= 3 {
        return files.join(", ");
    }
    format!("{} files", files.len())
}

fn group_by_directory(files: &[String]) -> std::collections::HashMap<String, Vec<String>> {
    let mut map = std::collections::HashMap::new();
    for f in files {
        let dir = Path::new(f)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        map.entry(dir).or_insert_with(Vec::new).push(f.clone());
    }
    map
}

fn count_files_in_part(part: &str) -> usize {
    let rest = part
        .strip_prefix("add ")
        .or_else(|| part.strip_prefix("update "))
        .or_else(|| part.strip_prefix("remove "))
        .unwrap_or(part);
    if rest.ends_with(" files") {
        rest.trim_end_matches(" files")
            .rsplit_once(' ')
            .map(|(_, n)| n.parse::<usize>().unwrap_or(1))
            .unwrap_or(1)
    } else {
        rest.split(", ").count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn init_test_repo(path: &str) {
        let _ = fs::remove_dir_all(path);
        fs::create_dir_all(path).expect("failed to create test repo directory");
        run_git(&["init"], path).expect("failed to git init test repo");
        run_git(&["config", "user.email", "test@test.com"], path).expect("failed to set git user.email");
        run_git(&["config", "user.name", "Test"], path).expect("failed to set git user.name");
        fs::write(Path::new(path).join("README.md"), "# Test").expect("failed to write README.md");
        run_git(&["add", "."], path).expect("failed to git add");
        run_git(&["commit", "-m", "initial"], path).expect("failed to create initial commit");
    }

    #[test]
    fn test_worktree_create_list_remove() {
        let repo = std::env::temp_dir().join("neotrix_git_worktree");
        let repo_str = repo.to_str().expect("temp path is not valid UTF-8").to_string();
        let wt = std::env::temp_dir().join("neotrix_git_wt");
        let wt_str = wt.to_str().expect("temp path is not valid UTF-8").to_string();
        let _ = fs::remove_dir_all(&wt);

        init_test_repo(&repo_str);
        run_git(&["checkout", "-b", "feature/test"], &repo_str).expect("failed to checkout feature/test branch");
        run_git(&["checkout", "main"], &repo_str).expect("failed to checkout main branch");

        let result = worktree_create("feature/test", &wt_str, &repo_str);
        assert!(result.is_ok(), "worktree create failed: {:?}", result);

        let canonical_wt = std::path::Path::new(&wt_str).canonicalize().unwrap_or_else(|_| wt.clone());
        let list = worktree_list(&repo_str).expect("failed to list worktrees");
        assert!(
            list.iter().any(|w| std::path::Path::new(&w.path) == canonical_wt),
            "worktree not found in list"
        );

        worktree_remove(&wt_str, &repo_str).expect("failed to remove worktree");
        let list = worktree_list(&repo_str).expect("failed to list worktrees after removal");
        assert!(!list.iter().any(|w| std::path::Path::new(&w.path) == canonical_wt));

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_branch_list_and_create() {
        let repo = std::env::temp_dir().join("neotrix_git_branch");
        let repo_str = repo.to_str().expect("temp path is not valid UTF-8").to_string();
        init_test_repo(&repo_str);

        run_git(&["checkout", "-b", "develop"], &repo_str).expect("failed to create develop branch");

        branch_create("feature/xyz", "develop", &repo_str).expect("failed to create feature branch");
        let branches = branch_list(&repo_str).expect("failed to list branches");
        assert!(branches.iter().any(|b| b.name == "feature/xyz"));

        branch_switch(&repo_str, "main").expect("failed to switch to main");
        let branches = branch_list(&repo_str).expect("failed to list branches after switch");
        let main = branches.iter().find(|b| b.name == "main").expect("main branch not found");
        assert!(main.is_current);

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_generate_commit_message() {
        let repo = std::env::temp_dir().join("neotrix_git_commitmsg");
        let repo_str = repo.to_str().expect("temp path is not valid UTF-8").to_string();
        init_test_repo(&repo_str);

        fs::create_dir_all(Path::new(&repo_str).join("src")).expect("failed to create src dir");
        fs::write(Path::new(&repo_str).join("src/main.rs"), "fn main() {}").expect("failed to write main.rs");
        fs::write(Path::new(&repo_str).join("README.md"), "# Updated").expect("failed to write README.md");
        run_git(&["add", "."], &repo_str).expect("failed to git add");

        let msg = generate_commit_message(&repo_str).expect("failed to generate commit message");
        assert!(!msg.is_empty());
        assert!(msg.contains("main") || msg.contains("README"));

        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn test_diff_summary() {
        let repo = std::env::temp_dir().join("neotrix_git_diff");
        let repo_str = repo.to_str().expect("temp path is not valid UTF-8").to_string();
        init_test_repo(&repo_str);

        run_git(&["checkout", "-b", "feature"], &repo_str).expect("failed to checkout feature branch");
        fs::create_dir_all(Path::new(&repo_str).join("src")).expect("failed to create src dir");
        fs::write(
            Path::new(&repo_str).join("src/lib.rs"),
            "pub fn hello() {}",
        )
        .expect("failed to write lib.rs");
        run_git(&["add", "."], &repo_str).expect("failed to git add");
        run_git(&["commit", "-m", "feature commit"], &repo_str).expect("failed to create feature commit");

        let diffs = branch_diff(&repo_str, "main", "feature").expect("failed to get branch diff");
        assert!(!diffs.is_empty());

        let _ = fs::remove_dir_all(&repo);
    }
}
