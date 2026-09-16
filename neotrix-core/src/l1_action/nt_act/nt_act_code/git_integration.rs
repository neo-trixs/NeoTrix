#[derive(Debug, Clone)]
pub struct GitStatus {
    pub branch: String,
    pub dirty: bool,
    pub staged: u32,
    pub unstaged: u32,
    pub untracked: u32,
}

#[derive(Debug, Clone)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: u64,
}

pub struct GitIntegration {
    repo_path: String,
}

impl GitIntegration {
    pub fn new(repo_path: &str) -> Self {
        Self {
            repo_path: repo_path.to_string(),
        }
    }

    pub fn status(&self) -> Result<GitStatus, String> {
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        let staged = lines
            .iter()
            .filter(|l| l.starts_with('M') || l.starts_with('A') || l.starts_with('D'))
            .count() as u32;
        let unstaged = lines
            .iter()
            .filter(|l| l.len() > 1 && l.as_bytes()[1] == b'M')
            .count() as u32;
        let untracked = lines.iter().filter(|l| l.starts_with("??")).count() as u32;

        let branch_output = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| e.to_string())?;
        let branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();

        Ok(GitStatus {
            branch,
            dirty: !lines.is_empty(),
            staged,
            unstaged,
            untracked,
        })
    }

    pub fn log(&self, count: u32) -> Result<Vec<GitCommit>, String> {
        let output = std::process::Command::new("git")
            .args([
                "log",
                &format!("-{}", count),
                "--pretty=format:%H|%s|%an|%at",
            ])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 4 {
                    Some(GitCommit {
                        hash: parts[0].to_string(),
                        message: parts[1].to_string(),
                        author: parts[2].to_string(),
                        timestamp: parts[3].parse().unwrap_or(0),
                    })
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn diff(&self) -> Result<String, String> {
        let output = std::process::Command::new("git")
            .args(["diff", "--stat"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| e.to_string())?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let g = GitIntegration::new("/tmp");
        assert_eq!(g.repo_path, "/tmp");
    }
}
