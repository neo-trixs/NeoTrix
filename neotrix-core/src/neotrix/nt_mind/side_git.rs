//! Side-git world memory — independent git snapshot system
//!
//! Maintains a separate git repository at ~/.neotrix/snapshots/
//! that tracks file-level changes made by NeoTrix, independent of
//! the user's own git history.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::nt_core_util;

/// Side-git snapshot manager
#[derive(Debug, Clone)]
pub struct SideGit {
    snapshots_dir: PathBuf,
    initialized: bool,
    snapshot_count: u64,
}

impl SideGit {
    pub fn new() -> Self {
        Self {
            snapshots_dir: nt_core_util::home_dir().join(".neotrix").join("snapshots"),
            initialized: false,
            snapshot_count: 0,
        }
    }

    /// Initialize the side-git repository if it doesn't exist
    pub fn init(&mut self) -> Result<(), String> {
        let dir = &self.snapshots_dir;
        std::fs::create_dir_all(dir).map_err(|e| format!("create snapshots dir: {}", e))?;

        if !dir.join(".git").exists() {
            let output = Command::new("git")
                .args(["init"])
                .current_dir(dir)
                .output()
                .map_err(|e| format!("git init failed: {}", e))?;
            if !output.status.success() {
                return Err(format!(
                    "git init error: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            let _ = Command::new("git")
                .args(["config", "user.name", "neotrix-snapshot"])
                .current_dir(dir)
                .output();
            let _ = Command::new("git")
                .args(["config", "user.email", "neotrix@snapshot"])
                .current_dir(dir)
                .output();
        }

        let output = Command::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .current_dir(dir)
            .output();
        self.snapshot_count = match output {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse()
                .unwrap_or(0),
            _ => 0,
        };
        self.initialized = true;
        Ok(())
    }

    /// Snapshot a file: copy it into the side-git repo and commit
    pub fn snapshot_file(
        &mut self,
        file_path: &Path,
        workspace_root: &Path,
    ) -> Result<String, String> {
        if !self.initialized {
            self.init()?;
        }

        let rel_path = file_path.strip_prefix(workspace_root).unwrap_or(file_path);
        let dest = self.snapshots_dir.join(rel_path);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("create parent: {}", e))?;
        }

        std::fs::copy(file_path, &dest).map_err(|e| format!("copy file: {}", e))?;

        let _add_output = Command::new("git")
            .args(["add", "."])
            .current_dir(&self.snapshots_dir)
            .output()
            .map_err(|e| format!("git add: {}", e))?;

        let msg = format!(
            "snapshot {}: {}",
            self.snapshot_count + 1,
            rel_path.display()
        );
        let commit_output = Command::new("git")
            .args(["commit", "-m", &msg, "--allow-empty"])
            .current_dir(&self.snapshots_dir)
            .output()
            .map_err(|e| format!("git commit: {}", e))?;

        if commit_output.status.success() {
            self.snapshot_count += 1;
            let hash_output = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&self.snapshots_dir)
                .output();
            let hash = hash_output
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            Ok(hash)
        } else {
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            if stderr.contains("nothing to commit") || stderr.contains("no changes") {
                Ok("no_changes".to_string())
            } else {
                Err(format!("git commit: {}", stderr))
            }
        }
    }

    /// Snapshot multiple files at once
    pub fn snapshot_files(
        &mut self,
        file_paths: &[PathBuf],
        workspace_root: &Path,
    ) -> Result<u64, String> {
        let mut count = 0;
        for path in file_paths {
            if path.exists() {
                match self.snapshot_file(path, workspace_root) {
                    Ok(h) if h != "no_changes" => count += 1,
                    _ => {}
                }
            }
        }
        Ok(count)
    }

    /// Restore a file from the latest snapshot
    pub fn restore_file(&self, rel_path: &Path, workspace_root: &Path) -> Result<(), String> {
        let source = self.snapshots_dir.join(rel_path);
        let dest = workspace_root.join(rel_path);
        if source.exists() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("create parent: {}", e))?;
            }
            std::fs::copy(&source, &dest).map_err(|e| format!("restore file: {}", e))?;
            Ok(())
        } else {
            Err(format!("no snapshot found for: {}", rel_path.display()))
        }
    }

    /// List all snapshot commits (newest first)
    pub fn log(&self, max_count: usize) -> Vec<(String, String)> {
        if !self.initialized {
            return vec![];
        }
        let output = Command::new("git")
            .args(["log", "--oneline", &format!("-{}", max_count)])
            .current_dir(&self.snapshots_dir)
            .output();
        match output {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.splitn(2, ' ').collect();
                        if parts.len() == 2 {
                            Some((parts[0].to_string(), parts[1].to_string()))
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            _ => vec![],
        }
    }

    pub fn snapshot_count(&self) -> u64 {
        self.snapshot_count
    }
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    pub fn snapshots_dir(&self) -> &Path {
        &self.snapshots_dir
    }
}

impl Default for SideGit {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let sg = SideGit::new();
        assert!(!sg.is_initialized());
        assert_eq!(sg.snapshot_count(), 0);
        assert!(sg
            .snapshots_dir()
            .to_string_lossy()
            .contains(".neotrix/snapshots"));
    }

    #[test]
    fn test_init_creates_dir() {
        let mut sg = SideGit::new();
        assert!(sg.init().is_ok());
        assert!(sg.is_initialized());
        assert!(sg.snapshots_dir().exists());
    }
}
