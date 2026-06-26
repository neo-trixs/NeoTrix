use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSandbox {
    pub allowed_paths: Vec<PathBuf>,
    pub allowed_commands: Vec<String>,
    pub allowed_networks: Vec<String>,
    pub max_file_size: u64,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone)]
pub enum SandboxError {
    PathNotAllowed(String),
    CommandNotAllowed(String),
    NetworkNotAllowed(String),
    FileTooLarge(u64, u64),
    TimeoutExceeded(u64),
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SandboxError::PathNotAllowed(p) => write!(f, "path not allowed: {}", p),
            SandboxError::CommandNotAllowed(c) => write!(f, "command not allowed: {}", c),
            SandboxError::NetworkNotAllowed(h) => write!(f, "network host not allowed: {}", h),
            SandboxError::FileTooLarge(size, max) => write!(f, "file size {} exceeds max {}", size, max),
            SandboxError::TimeoutExceeded(t) => write!(f, "timeout {}s exceeded", t),
        }
    }
}

impl std::error::Error for SandboxError {}

impl Default for ToolSandbox {
    fn default() -> Self {
        Self {
            allowed_paths: Vec::new(),
            allowed_commands: Vec::new(),
            allowed_networks: Vec::new(),
            max_file_size: 10 * 1024 * 1024,
            timeout_secs: 30,
        }
    }
}

impl ToolSandbox {

    pub fn check_read(&self, path: &Path) -> Result<(), SandboxError> {
        let canonical = path.canonicalize().map_err(|_| {
            SandboxError::PathNotAllowed(path.display().to_string())
        })?;
        let allowed = self.allowed_paths.iter().any(|p| {
            p.canonicalize()
                .map(|a| canonical.starts_with(&a))
                .unwrap_or(false)
        });
        if allowed {
            Ok(())
        } else {
            Err(SandboxError::PathNotAllowed(path.display().to_string()))
        }
    }

    pub fn check_write(&self, path: &Path) -> Result<(), SandboxError> {
        let canonical = path.canonicalize().map_err(|_| {
            SandboxError::PathNotAllowed(path.display().to_string())
        })?;
        let allowed = self.allowed_paths.iter().any(|p| {
            p.canonicalize()
                .map(|a| canonical.starts_with(&a))
                .unwrap_or(false)
        });
        if allowed {
            Ok(())
        } else {
            Err(SandboxError::PathNotAllowed(path.display().to_string()))
        }
    }

    pub fn check_command(&self, cmd: &str) -> Result<(), SandboxError> {
        let base = cmd.split_whitespace().next().unwrap_or("");
        if self.allowed_commands.is_empty() || self.allowed_commands.iter().any(|a| a == base) {
            Ok(())
        } else {
            Err(SandboxError::CommandNotAllowed(cmd.to_string()))
        }
    }

    pub fn check_network(&self, host: &str) -> Result<(), SandboxError> {
        if self.allowed_networks.is_empty() || self.allowed_networks.iter().any(|a| a == host) {
            Ok(())
        } else {
            Err(SandboxError::NetworkNotAllowed(host.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_sandbox() {
        let s = ToolSandbox::default();
        assert!(s.allowed_paths.is_empty());
        assert_eq!(s.max_file_size, 10 * 1024 * 1024);
        assert_eq!(s.timeout_secs, 30);
    }

    #[test]
    fn test_check_command_allows_empty_allowlist() {
        let s = ToolSandbox::default();
        assert!(s.check_command("ls").is_ok());
    }

    #[test]
    fn test_check_command_denies_not_allowed() {
        let s = ToolSandbox {
            allowed_commands: vec!["ls".into(), "cat".into()],
            ..ToolSandbox::default()
        };
        assert!(s.check_command("rm -rf /").is_err());
    }

    #[test]
    fn test_check_command_allows_listed() {
        let s = ToolSandbox {
            allowed_commands: vec!["ls".into()],
            ..ToolSandbox::default()
        };
        assert!(s.check_command("ls -la").is_ok());
    }

    #[test]
    fn test_check_network_empty_allowlist() {
        let s = ToolSandbox::default();
        assert!(s.check_network("example.com").is_ok());
    }

    #[test]
    fn test_check_network_denies() {
        let s = ToolSandbox {
            allowed_networks: vec!["localhost".into()],
            ..ToolSandbox::default()
        };
        assert!(s.check_network("evil.com").is_err());
    }

    #[test]
    fn test_sandbox_error_display() {
        let e = SandboxError::PathNotAllowed("/tmp".into());
        assert_eq!(e.to_string(), "path not allowed: /tmp");
    }

    #[test]
    fn test_check_read_allowed_path() {
        let dir = std::env::temp_dir().join("neotrix_sandbox_test");
        fs::create_dir_all(&dir).ok();
        let s = ToolSandbox {
            allowed_paths: vec![dir.clone()],
            ..ToolSandbox::default()
        };
        let test_file = dir.join("test.txt");
        fs::write(&test_file, "data").ok();
        assert!(s.check_read(&test_file).is_ok());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_check_read_denied_path() {
        let s = ToolSandbox {
            allowed_paths: vec!["/tmp/allowed".into()],
            ..ToolSandbox::default()
        };
        let err = s.check_read(Path::new("/etc/passwd"));
        assert!(err.is_err());
    }
}
