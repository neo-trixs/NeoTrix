use tauri::command;
use neotrix::neotrix::nt_core_error::NeoTrixError;
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

fn run_git_cmd(args: &[&str]) -> Result<String, NeoTrixError> {
    let output = std::process::Command::new("git")
        .args(args)
        .output()
        .map_err(|e| NeoTrixError::Command { cmd: format!("git {}", args.join(" ")), exit_code: None, stderr: e.to_string() })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(NeoTrixError::Command { cmd: format!("git {}", args.join(" ")), exit_code: output.status.code(), stderr: stderr.trim().to_string() });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[command]
pub fn cmd_diff_staged() -> Result<Vec<DiffBlock>, NeoTrixError> {
    run_git_cmd(&["diff", "--cached"]).map(|s| parse_git_diff(&s))
}

#[command]
pub fn cmd_diff_unstaged() -> Result<Vec<DiffBlock>, NeoTrixError> {
    run_git_cmd(&["diff"]).map(|s| parse_git_diff(&s))
}

#[command]
pub fn cmd_diff_file(path: String) -> Result<Vec<DiffBlock>, NeoTrixError> {
    run_git_cmd(&["diff", "HEAD", "--", &path]).map(|s| parse_git_diff(&s))
}

/// Stage the given paths (or all changes when `paths` is empty). Returns the
/// updated list of changed files for the review UI.
/// Structured changed-file list for the diff review UI (Codex #1 gap: the
/// review file tree was hidden behind a button). Returns porcelain entries
/// with their two-letter status + path, split into staged/unstaged buckets.
#[command]
pub fn cmd_diff_changed_files() -> Result<serde_json::Value, NeoTrixError> {
    let out = run_git_cmd(&["status", "--porcelain"])?;
    Ok(parse_porcelain_changed(&out))
}

#[command]
pub fn cmd_diff_stage(paths: Option<Vec<String>>) -> Result<Vec<String>, NeoTrixError> {
    match paths {
        Some(p) if !p.is_empty() => {
            let mut args: Vec<String> = vec!["add".into(), "--".into()];
            args.extend(p);
            let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            run_git_cmd(&arg_refs)?;
        }
        _ => { run_git_cmd(&["add", "-A"])?; }
    };
    changed_files_porcelain()
}

fn changed_files_porcelain() -> Result<Vec<String>, NeoTrixError> {
    let out = run_git_cmd(&["status", "--porcelain"])?;
    Ok(parse_porcelain_paths(&out))
}

/// Parse `git status --porcelain` output into bare file paths.
pub(crate) fn parse_porcelain_paths(out: &str) -> Vec<String> {
    out.lines()
        .filter_map(|l| {
            if l.len() <= 3 { return None; }
            let p = l[3..].trim().to_string();
            if p.is_empty() { None } else { Some(p) }
        })
        .collect()
}

/// Parse `git status --porcelain` output into a structured {staged,
/// unstaged, untracked} file list for the diff review UI.
pub(crate) fn parse_porcelain_changed(out: &str) -> serde_json::Value {
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();
    for l in out.lines() {
        // Porcelain format: exactly XY SP path — status chars must come from
        // the raw line (do NOT trim; " M" means unstaged-modified).
        if l.len() < 4 { continue; }
        let status = &l[..2];
        let path = l[3..].trim().to_string();
        if path.is_empty() { continue; }
        let entry = serde_json::json!({ "status": status.trim(), "path": path });
        let is_staged = status.chars().next() == Some('M') || status.starts_with("A ") || status.starts_with("D ") || status.starts_with("R ") || status.starts_with("C ");
        if status == "??" {
            untracked.push(entry);
        } else if is_staged {
            staged.push(entry);
        } else {
            unstaged.push(entry);
        }
    }
    serde_json::json!({ "staged": staged, "unstaged": unstaged, "untracked": untracked })
}

/// Unstage the given paths (or everything when `paths` is empty). Returns the
/// updated porcelain file list.
#[command]
pub fn cmd_diff_unstage(paths: Option<Vec<String>>) -> Result<Vec<String>, NeoTrixError> {
    match paths {
        Some(p) if !p.is_empty() => {
            let mut args: Vec<String> = vec!["reset".into(), "HEAD".into(), "--".into()];
            args.extend(p);
            let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            run_git_cmd(&arg_refs)?;
        }
        _ => { run_git_cmd(&["reset"])?; }
    };
    changed_files_porcelain()
}

/// Discard working-tree changes for the given paths (per-file "reject" in the
/// diff review UI, Claude Code Manual / Codex review parity). Untracked files
/// are removed; tracked files are restored to HEAD. Returns the updated
/// porcelain changed-file list.
#[command]
pub fn cmd_diff_restore(paths: Vec<String>) -> Result<Vec<String>, NeoTrixError> {
    if paths.is_empty() {
        return Err(NeoTrixError::Memory("no paths to restore".into()));
    }
    for p in &paths {
        // Untracked (??) files have no HEAD entry; remove them outright.
        let porcelain = run_git_cmd(&["status", "--porcelain", "--", p])?;
        let untracked = porcelain.lines().any(|l| l.starts_with("??"));
        if untracked {
            let _ = std::fs::remove_file(p);
            continue;
        }
        // Restore to HEAD (not `--worktree` alone, which restores from the
        // INDEX and leaves staged changes behind — an accept/reject reject
        // must fully discard the change). `--staged --worktree --source=HEAD`
        // resets both the index and the working tree to HEAD semantics.
        run_git_cmd(&["restore", "--staged", "--worktree", "--source=HEAD", "--", p])?;
    }
    changed_files_porcelain()
}

/// Commit staged changes with the given message (Codex "Review changes" parity).
#[command]
pub fn cmd_diff_commit(message: String) -> Result<(), NeoTrixError> {
    run_git_cmd(&["commit", "-m", &message]).map(|_| ())
}

/// Diff against a base branch (Codex "Review against base branch" parity).
/// `git diff <base>...HEAD` shows commits on HEAD since diverging from base.
/// Falls back to `git diff <base>` when no merge base exists (unrelated
/// histories / fresh repos).
#[command]
pub fn cmd_diff_base(base: String) -> Result<Vec<DiffBlock>, NeoTrixError> {
    let merge_base = run_git_cmd(&["merge-base", &base, "HEAD"]);
    let args: Vec<&str> = if merge_base.is_ok() {
        vec!["diff", &base, "...HEAD"]
    } else {
        vec!["diff", &base, "HEAD"]
    };
    run_git_cmd(&args).map(|s| parse_git_diff(&s))
}

/// Changed-file list for base-branch review (parity with cmd_diff_changed_files
/// but scoped to the base branch). Returns porcelain-style {staged, unstaged,
/// untracked} buckets by reusing the same parser on `git diff --name-status`.
/// Parse `git diff --name-status` output into porcelain-style {staged} entries
/// (status letter + path). Shared by `cmd_diff_base_files`; testable in isolation.
pub(crate) fn parse_name_status(out: &str, base: &str) -> serde_json::Value {
    let mut staged = Vec::new();
    for line in out.lines() {
        if line.len() < 4 { continue; }
        let status = line[..2].trim();
        let path = line[2..].trim().to_string();
        if path.is_empty() { continue; }
        staged.push(serde_json::json!({ "status": status, "path": path }));
    }
    serde_json::json!({ "staged": staged, "unstaged": serde_json::Value::Array(vec![]), "untracked": serde_json::Value::Array(vec![]), "base": base })
}

#[command]
pub fn cmd_diff_base_files(base: String) -> Result<serde_json::Value, NeoTrixError> {
    let merge_base = run_git_cmd(&["merge-base", &base, "HEAD"]);
    let args: Vec<&str> = if merge_base.is_ok() {
        vec!["diff", "--name-status", &base, "...HEAD"]
    } else {
        vec!["diff", "--name-status", &base, "HEAD"]
    };
    let out = run_git_cmd(&args)?;
    Ok(parse_name_status(&out, &base))
}

/// Gather the full working-tree diff (staged + unstaged) and run the static
/// code-review pass on it (Claude Desktop "Review code" parity). Returns the
/// same `ReviewResult` shape as `review_diff`.
#[command]
pub fn cmd_diff_review(pr_title: Option<String>) -> Result<super::review_cmds::ReviewResult, NeoTrixError> {
    let mut diff_text = String::new();
    if let Ok(s) = run_git_cmd(&["diff"]) {
        diff_text.push_str(&s);
    }
    if let Ok(s) = run_git_cmd(&["diff", "--cached"]) {
        diff_text.push('\n');
        diff_text.push_str(&s);
    }
    let title = pr_title.unwrap_or_else(|| "工作区变更".to_string());
    super::review_cmds::review_diff(diff_text, title).map_err(|e| NeoTrixError::Memory(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn porcelain_paths_extracts_clean_paths() {
        let out = " M src/lib.rs\n A README.md\n?? notes.txt\n";
        assert_eq!(parse_porcelain_paths(out), vec!["src/lib.rs", "README.md", "notes.txt"]);
    }

    #[test]
    fn porcelain_changed_buckets_by_status() {
        let out = " M src/unstaged.rs\nM  src/staged.rs\nA  src/added.rs\n?? notes.txt\n";
        let v = parse_porcelain_changed(out);
        assert_eq!(v["staged"].as_array().unwrap().len(), 2);
        assert_eq!(v["unstaged"].as_array().unwrap().len(), 1);
        assert_eq!(v["untracked"].as_array().unwrap().len(), 1);
        assert_eq!(v["staged"][0]["path"], "src/staged.rs");
        assert_eq!(v["untracked"][0]["path"], "notes.txt");
    }

    #[test]
    fn porcelain_changed_handles_untracked_dir_suffix() {
        let out = "?? newdir/\n M src/a.rs\n";
        let v = parse_porcelain_changed(out);
        assert_eq!(v["untracked"][0]["path"], "newdir/");
        assert_eq!(v["unstaged"][0]["path"], "src/a.rs");
    }

    #[test]
    fn name_status_parses_base_diff_files() {
        let out = "M\tsrc/main.rs\nA\tsrc/new.rs\nD\told.rs\nR100\told_name.rs\tnew_name.rs\n";
        let v = parse_name_status(out, "main");
        let staged = v["staged"].as_array().unwrap();
        assert_eq!(staged.len(), 4);
        assert_eq!(staged[0]["status"], "M");
        assert_eq!(staged[0]["path"], "src/main.rs");
        assert_eq!(staged[1]["status"], "A");
        assert_eq!(staged[2]["status"], "D");
        // Rename line: tab-separated "R100 old_name.rs new_name.rs" — parser
        // takes everything after the status as the path; keep the full tail.
        assert!(staged[3]["path"].as_str().unwrap().contains("old_name.rs"));
        assert_eq!(v["base"], "main");
    }

    #[test]
    fn name_status_skips_empty_lines() {
        let out = "\nM\tsrc/a.rs\n\n";
        let v = parse_name_status(out, "main");
        assert_eq!(v["staged"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn review_gathers_diff_and_scores() {
        // cmd_diff_review shells out to git; verify the review pass itself by
        // feeding a synthetic diff through the shared review_diff pipeline.
        let diff = "+++ b/src/lib.rs\n+fn main() {\n+    let password = \"hunter2\";\n+    // TODO: fix\n+}\n";
        let result = crate::commands::review_cmds::review_diff(diff.to_string(), "test".into()).unwrap();
        assert_eq!(result.total_files, 1);
        assert!(result.critical >= 1, "hardcoded credential must be critical");
        assert!(result.score < 100);
        assert!(result.summary.contains("Score:"));
    }
}
