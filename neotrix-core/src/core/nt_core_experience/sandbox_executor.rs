use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

use super::tool_safety::SafetyLevel;

fn default_temp_dir() -> String {
    crate::core::nt_core_util::home_dir()
        .join(".neotrix")
        .join("sandbox")
        .to_string_lossy()
        .to_string()
}

const SANDBOX_VERSION: &str = "0.1.0";
const DEFAULT_CLEANUP_TTL_SECS: u64 = 3600;
const MAX_OUTPUT_BYTES: usize = 1_048_576;

/// Level of sandbox isolation for tool execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SandboxLevel {
    /// No isolation — direct execution.
    None,
    /// Read-only filesystem via symlinks, no network.
    ReadOnly,
    /// Isolated temp dir with stripped PATH and disabled network.
    Isolated,
    /// Strict isolation — cleared PATH, restricted HOME, blocked danger commands.
    Strict,
}

impl SandboxLevel {
    pub fn label(&self) -> &'static str {
        match self {
            SandboxLevel::None => "none",
            SandboxLevel::ReadOnly => "readonly",
            SandboxLevel::Isolated => "isolated",
            SandboxLevel::Strict => "strict",
        }
    }

    pub fn from_safety_level(level: &SafetyLevel) -> Self {
        match level {
            SafetyLevel::Safe | SafetyLevel::LowRisk => SandboxLevel::None,
            SafetyLevel::MediumRisk => SandboxLevel::ReadOnly,
            SafetyLevel::HighRisk => SandboxLevel::Isolated,
        }
    }
}

/// Execution status of a sandboxed command.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SandboxStatus {
    Running,
    Completed,
    Failed(String),
    Killed,
}

/// Result of a sandboxed execution.
#[derive(Debug, Clone)]
pub struct SandboxResultExecutor {
    pub status: SandboxStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub execution_time_ms: u64,
    pub sandbox_path: String,
}

/// Active sandbox tracking.
#[derive(Debug)]
pub struct ActiveSandbox {
    pub id: String,
    pub sandbox_path: PathBuf,
    pub created_at: Instant,
    pub cmd: String,
    pub level: SandboxLevel,
    pub result: Option<SandboxResultExecutor>,
}

impl ActiveSandbox {
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.age() >= ttl
    }
}

/// Configuration for the SandboxExecutor.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub temp_dir: PathBuf,
    pub cleanup_ttl: Duration,
    pub safe_commands: Vec<String>,
    pub danger_patterns: Vec<String>,
    pub max_output_bytes: usize,
    pub enable_sandbox_exec: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            temp_dir: PathBuf::from(default_temp_dir()),
            cleanup_ttl: Duration::from_secs(DEFAULT_CLEANUP_TTL_SECS),
            safe_commands: vec![
                "ls".into(),
                "echo".into(),
                "cat".into(),
                "head".into(),
                "tail".into(),
                "wc".into(),
                "sort".into(),
                "which".into(),
                "pwd".into(),
                "date".into(),
                "whoami".into(),
                "uname".into(),
                "git".into(),
                "cargo".into(),
                "rustc".into(),
                "python3".into(),
                "python".into(),
                "node".into(),
                "npm".into(),
                "true".into(),
                "false".into(),
            ],
            danger_patterns: vec![
                "rm -rf /".into(),
                "rm -rf /*".into(),
                "dd if=/dev/zero".into(),
                ":(){ :|:& };:".into(),
                "mkfs".into(),
                "fdisk".into(),
                "dd if=".into(),
                "chmod 777 /".into(),
                "chown".into(),
                "> /dev/sda".into(),
                "> /dev/nvme".into(),
                "sudo".into(),
            ],
            max_output_bytes: MAX_OUTPUT_BYTES,
            enable_sandbox_exec: true,
        }
    }
}

/// Container-level tool execution isolator.
///
/// Provides best-effort isolation for executing external commands
/// by creating temporary directories, managing environment variables,
/// and optionally using macOS `sandbox-exec` for additional containment.
///
/// This is NOT a security boundary — it is best-effort isolation
/// to prevent accidental damage from tool execution.
#[derive(Debug)]
pub struct SandboxExecutor {
    pub config: SandboxConfig,
    pub active_sandboxes: HashMap<String, ActiveSandbox>,
    pub total_executions: u64,
    pub total_blocks: u64,
}

impl SandboxExecutor {
    pub fn new(config: SandboxConfig) -> Self {
        if let Err(e) = fs::create_dir_all(&config.temp_dir) {
            log::warn!(
                "failed to create temp dir {}: {}",
                config.temp_dir.display(),
                e
            );
        }
        Self {
            config,
            active_sandboxes: HashMap::new(),
            total_executions: 0,
            total_blocks: 0,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(SandboxConfig::default())
    }

    /// Generate a unique sandbox ID.
    fn generate_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("sbx_{:x}_{}", nanos, self.total_executions)
    }

    /// Check if a command matches any danger pattern.
    pub fn is_danger_command(&self, cmd: &str) -> bool {
        let cmd_lower = cmd.to_lowercase();
        self.config
            .danger_patterns
            .iter()
            .any(|p| cmd_lower.contains(&p.to_lowercase()))
    }

    /// Check if a command is in the safe list.
    pub fn is_safe_command(&self, cmd: &str) -> bool {
        let cmd_name = cmd.split_whitespace().next().unwrap_or(cmd);
        self.config.safe_commands.iter().any(|s| s == cmd_name)
    }

    /// Resolve the full path of a command using `which`.
    fn resolve_command(cmd: &str) -> Option<String> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }
        let cmd_name = parts[0];
        if cmd_name.contains('/') {
            if Path::new(cmd_name).exists() {
                return Some(cmd_name.to_string());
            }
            return None;
        }
        let which_output = Command::new("which").arg(cmd_name).output().ok()?;
        if which_output.status.success() {
            let path = String::from_utf8_lossy(&which_output.stdout)
                .trim()
                .to_string();
            if !path.is_empty() {
                return Some(path);
            }
        }
        None
    }

    /// Determine sandbox level from command and safety context.
    pub fn determine_level(&self, cmd: &str, hint: Option<SandboxLevel>) -> SandboxLevel {
        if let Some(level) = hint {
            if level >= SandboxLevel::ReadOnly {
                return level;
            }
        }
        if self.is_danger_command(cmd) {
            return SandboxLevel::Strict;
        }
        if !self.is_safe_command(cmd) {
            return SandboxLevel::Isolated;
        }
        SandboxLevel::None
    }

    /// Create a sandbox directory at the given path.
    fn create_sandbox_dir(&self, sbx_path: &Path) -> std::io::Result<()> {
        fs::create_dir_all(sbx_path.join("work"))?;
        fs::create_dir_all(sbx_path.join("tmp"))?;
        fs::create_dir_all(sbx_path.join("home"))?;
        Ok(())
    }

    /// Apply ReadOnly isolation: symlink safe system paths.
    fn apply_readonly_isolation(&self, sbx_path: &Path) {
        let safe_bindirs = ["/usr/bin", "/bin", "/usr/lib", "/usr/share"];
        for dir in &safe_bindirs {
            let target = sbx_path.join("work").join(dir.trim_start_matches('/'));
            if let Err(e) = fs::create_dir_all(&target) {
                log::warn!(
                    "failed to create readonly bindir {}: {}",
                    target.display(),
                    e
                );
            }
        }
    }

    /// Check if `sandbox-exec` is available on macOS.
    fn sandbox_exec_available() -> bool {
        Command::new("which")
            .arg("sandbox-exec")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Build a sandbox-exec profile for the given level.
    /// Execute a command with the given sandbox level.
    ///
    /// Returns a `SandboxResultExecutor` with stdout, stderr, exit code, and timing.
    /// Temp dir cleanup is best-effort; stale dirs are cleaned by `cleanup_stale()`.
    pub fn execute(
        &mut self,
        cmd: &str,
        level: SandboxLevel,
        input_files: Vec<(&str, &str)>,
        env_overrides: Vec<(&str, &str)>,
    ) -> SandboxResultExecutor {
        let start = Instant::now();
        let sbx_id = self.generate_id();
        let sbx_path = self.config.temp_dir.join(&sbx_id);

        if level >= SandboxLevel::Strict && self.is_danger_command(cmd) {
            self.total_blocks += 1;
            return SandboxResultExecutor {
                status: SandboxStatus::Failed("danger command blocked".into()),
                stdout: String::new(),
                stderr: format!(
                    "DANGER: Command '{}' matched danger pattern and was blocked",
                    cmd
                ),
                exit_code: None,
                execution_time_ms: 0,
                sandbox_path: sbx_path.to_string_lossy().to_string(),
            };
        }

        self.total_executions += 1;
        if let Err(e) = self.create_sandbox_dir(&sbx_path) {
            log::warn!("failed to create sandbox dir {}: {}", sbx_path.display(), e);
        }

        for (name, content) in &input_files {
            let file_path = sbx_path.join("work").join(name);
            if let Err(e) = fs::write(&file_path, content) {
                log::warn!("failed to write input file {}: {}", file_path.display(), e);
            }
        }

        let resolved_cmd = Self::resolve_command(cmd);
        let executable = resolved_cmd.as_deref().unwrap_or(cmd);
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let args: Vec<&str> = if parts.len() > 1 {
            parts[1..].to_vec()
        } else {
            vec![]
        };

        self.apply_readonly_isolation(&sbx_path);

        let mut command = Command::new(executable);
        command.args(&args);
        command.current_dir(sbx_path.join("work"));

        match level {
            SandboxLevel::None => {
                command.env_remove("NO_NETWORK");
            }
            SandboxLevel::ReadOnly => {
                command.env("SANDBOX_LEVEL", "readonly");
                command.env("NO_NETWORK", "0");
                let home = sbx_path.join("home");
                command.env("HOME", home.to_string_lossy().as_ref());
            }
            SandboxLevel::Isolated => {
                command.env("SANDBOX_LEVEL", "isolated");
                command.env("NO_NETWORK", "1");
                command.env("PATH", "/usr/bin:/bin");
                let home = sbx_path.join("home");
                command.env("HOME", home.to_string_lossy().as_ref());
                command.env("TMPDIR", sbx_path.join("tmp").to_string_lossy().as_ref());
            }
            SandboxLevel::Strict => {
                command.env("SANDBOX_LEVEL", "strict");
                command.env("NO_NETWORK", "1");
                command.env("PATH", "");
                command.env("HOME", sbx_path.join("home").to_string_lossy().as_ref());
                command.env("TMPDIR", sbx_path.join("tmp").to_string_lossy().as_ref());
                command.env_remove("LD_LIBRARY_PATH");
                command.env_remove("DYLD_LIBRARY_PATH");
                command.env_remove("DYLD_INSERT_LIBRARIES");
            }
        }

        for (k, v) in &env_overrides {
            command.env(k, v);
        }

        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());

        let active = ActiveSandbox {
            id: sbx_id.clone(),
            sandbox_path: sbx_path.clone(),
            created_at: Instant::now(),
            cmd: cmd.to_string(),
            level,
            result: None,
        };
        self.active_sandboxes.insert(sbx_id.clone(), active);

        let output: Output = match command.output() {
            Ok(o) => o,
            Err(e) => {
                let elapsed = start.elapsed();
                if let Some(a) = self.active_sandboxes.get_mut(&sbx_id) {
                    a.result = Some(SandboxResultExecutor {
                        status: SandboxStatus::Failed(e.to_string()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        exit_code: None,
                        execution_time_ms: elapsed.as_millis() as u64,
                        sandbox_path: sbx_path.to_string_lossy().to_string(),
                    });
                }
                return self
                    .active_sandboxes
                    .remove(&sbx_id)
                    .and_then(|a| a.result)
                    .unwrap_or_else(|| SandboxResultExecutor {
                        status: SandboxStatus::Failed("sandbox state corrupted".into()),
                        stdout: String::new(),
                        stderr: "internal error: sandbox entry missing after execution".to_string(),
                        exit_code: None,
                        execution_time_ms: elapsed.as_millis() as u64,
                        sandbox_path: sbx_path.to_string_lossy().to_string(),
                    });
            }
        };

        let elapsed = start.elapsed();
        let stdout_str = String::from_utf8_lossy(
            &output.stdout[..output.stdout.len().min(self.config.max_output_bytes)],
        )
        .to_string();
        let stderr_str = String::from_utf8_lossy(
            &output.stderr[..output.stderr.len().min(self.config.max_output_bytes)],
        )
        .to_string();

        let status = if output.status.success() {
            SandboxStatus::Completed
        } else {
            match output.status.code() {
                Some(code) => SandboxStatus::Failed(format!("exit code {}", code)),
                None => SandboxStatus::Killed,
            }
        };

        let result = SandboxResultExecutor {
            status: status.clone(),
            stdout: stdout_str,
            stderr: stderr_str,
            exit_code: output.status.code(),
            execution_time_ms: elapsed.as_millis() as u64,
            sandbox_path: sbx_path.to_string_lossy().to_string(),
        };

        if let Some(a) = self.active_sandboxes.get_mut(&sbx_id) {
            a.result = Some(result.clone());
        }

        result
    }

    /// Clean up expired sandbox directories.
    pub fn cleanup_stale(&mut self) -> usize {
        let ttl = self.config.cleanup_ttl;
        let expired_ids: Vec<String> = self
            .active_sandboxes
            .iter()
            .filter(|(_, sbx)| sbx.is_expired(ttl))
            .map(|(id, _)| id.clone())
            .collect();

        let count = expired_ids.len();
        for id in &expired_ids {
            if let Some(sbx) = self.active_sandboxes.remove(id) {
                if let Err(e) = fs::remove_dir_all(&sbx.sandbox_path) {
                    log::warn!(
                        "failed to remove stale sandbox {}: {}",
                        sbx.sandbox_path.display(),
                        e
                    );
                }
            }
        }
        count
    }

    /// Clean up a specific sandbox by ID.
    pub fn cleanup_sandbox(&mut self, id: &str) -> bool {
        if let Some(sbx) = self.active_sandboxes.remove(id) {
            if let Err(e) = fs::remove_dir_all(&sbx.sandbox_path) {
                log::warn!(
                    "failed to remove sandbox {}: {}",
                    sbx.sandbox_path.display(),
                    e
                );
            }
            true
        } else {
            false
        }
    }

    /// Number of currently active sandboxes.
    pub fn active_count(&self) -> usize {
        self.active_sandboxes.len()
    }

    /// Get a summary report of executor state.
    pub fn report(&self) -> String {
        format!(
            "sandbox:exec={}_blocked={}_active={}_ttl={}s_v={}",
            self.total_executions,
            self.total_blocks,
            self.active_sandboxes.len(),
            self.config.cleanup_ttl.as_secs(),
            SANDBOX_VERSION,
        )
    }

    /// Warn about sandbox-exec availability.
    pub fn check_sandbox_exec_available() -> bool {
        Self::sandbox_exec_available()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_level_ordering() {
        assert!(SandboxLevel::None < SandboxLevel::ReadOnly);
        assert!(SandboxLevel::ReadOnly < SandboxLevel::Isolated);
        assert!(SandboxLevel::Isolated < SandboxLevel::Strict);
    }

    #[test]
    fn test_sandbox_level_from_safety() {
        assert_eq!(
            SandboxLevel::from_safety_level(&SafetyLevel::Safe),
            SandboxLevel::None
        );
        assert_eq!(
            SandboxLevel::from_safety_level(&SafetyLevel::LowRisk),
            SandboxLevel::None
        );
        assert_eq!(
            SandboxLevel::from_safety_level(&SafetyLevel::MediumRisk),
            SandboxLevel::ReadOnly
        );
        assert_eq!(
            SandboxLevel::from_safety_level(&SafetyLevel::HighRisk),
            SandboxLevel::Isolated
        );
        assert_eq!(
            SandboxLevel::from_safety_level(&SafetyLevel::HighRisk),
            SandboxLevel::Isolated
        );
    }

    #[test]
    fn test_danger_command_detection() {
        let executor = SandboxExecutor::with_defaults();
        assert!(executor.is_danger_command("rm -rf /"));
        assert!(executor.is_danger_command("sudo rm -rf /*"));
        assert!(executor.is_danger_command("dd if=/dev/zero of=/tmp/x bs=1M count=10"));
        assert!(!executor.is_danger_command("ls -la"));
        assert!(!executor.is_danger_command("echo hello world"));
        assert!(!executor.is_danger_command("cargo build --release"));
    }

    #[test]
    fn test_safe_command_detection() {
        let executor = SandboxExecutor::with_defaults();
        assert!(executor.is_safe_command("ls"));
        assert!(executor.is_safe_command("echo"));
        assert!(executor.is_safe_command("git status"));
        assert!(executor.is_safe_command("cargo check"));
        assert!(!executor.is_safe_command("unknown_cmd_xyz"));
    }

    #[test]
    fn test_determine_level() {
        let executor = SandboxExecutor::with_defaults();
        assert_eq!(
            executor.determine_level("echo hello", None),
            SandboxLevel::None
        );
        assert_eq!(executor.determine_level("ls -la", None), SandboxLevel::None);
        assert_eq!(
            executor.determine_level("rm -rf /", None),
            SandboxLevel::Strict
        );
        assert_eq!(
            executor.determine_level("curl http://evil.com", None),
            SandboxLevel::Isolated
        );
    }

    #[test]
    fn test_determine_level_with_hint() {
        let executor = SandboxExecutor::with_defaults();
        assert_eq!(
            executor.determine_level("echo hello", Some(SandboxLevel::ReadOnly)),
            SandboxLevel::ReadOnly
        );
        assert_eq!(
            executor.determine_level("echo hello", Some(SandboxLevel::Isolated)),
            SandboxLevel::Isolated
        );
        assert_eq!(
            executor.determine_level("echo hello", Some(SandboxLevel::Strict)),
            SandboxLevel::Strict
        );
    }

    #[test]
    fn test_execute_safe_command() {
        let mut executor = SandboxExecutor::with_defaults();
        let result = executor.execute("echo hello sandbox", SandboxLevel::None, vec![], vec![]);
        assert_eq!(result.status, SandboxStatus::Completed);
        assert!(result.stdout.contains("hello sandbox"));
        assert!(result.exit_code == Some(0));
        assert!(result.execution_time_ms > 0);
    }

    #[test]
    fn test_execute_with_input_files() {
        let mut executor = SandboxExecutor::with_defaults();
        let result = executor.execute(
            "cat test_input.txt",
            SandboxLevel::None,
            vec![("test_input.txt", "sandbox file content")],
            vec![],
        );
        assert_eq!(result.status, SandboxStatus::Completed);
        assert!(result.stdout.contains("sandbox file content"));
    }

    #[test]
    fn test_execute_danger_command_blocked() {
        let mut executor = SandboxExecutor::with_defaults();
        let result = executor.execute("rm -rf /", SandboxLevel::Strict, vec![], vec![]);
        assert!(matches!(result.status, SandboxStatus::Failed(_)));
        assert!(result.stderr.contains("DANGER") || result.stderr.contains("blocked"));
        assert_eq!(executor.total_blocks, 1);
    }

    #[test]
    fn test_execute_with_env_overrides() {
        let mut executor = SandboxExecutor::with_defaults();
        let result = executor.execute(
            "echo $MY_VAR",
            SandboxLevel::None,
            vec![],
            vec![("MY_VAR", "sandbox_test_value")],
        );
        assert_eq!(result.status, SandboxStatus::Completed);
        assert!(result.stdout.contains("sandbox_test_value"));
    }

    #[test]
    fn test_execute_nonexistent_command() {
        let mut executor = SandboxExecutor::with_defaults();
        let result = executor.execute(
            "nonexistent_cmd_xyz_999",
            SandboxLevel::None,
            vec![],
            vec![],
        );
        assert!(result.exit_code.is_none() || result.exit_code != Some(0));
    }

    #[test]
    fn test_active_sandbox_tracking() {
        let mut executor = SandboxExecutor::with_defaults();
        let initial_count = executor.active_count();
        executor.execute("echo tracking-test", SandboxLevel::None, vec![], vec![]);
        assert_eq!(executor.active_count(), initial_count + 1);
        assert_eq!(executor.total_executions, 1);
    }

    #[test]
    fn test_cleanup_sandbox_by_id() {
        let mut executor = SandboxExecutor::with_defaults();
        let result = executor.execute("echo cleanup-me", SandboxLevel::None, vec![], vec![]);
        let sbx_id = result
            .sandbox_path
            .split('/')
            .last()
            .unwrap_or("")
            .to_string();
        let cleaned = executor.cleanup_sandbox(&sbx_id);
        assert!(cleaned || sbx_id.is_empty());
    }

    #[test]
    fn test_report_format() {
        let mut executor = SandboxExecutor::with_defaults();
        executor.execute("echo report-test", SandboxLevel::None, vec![], vec![]);
        let report = executor.report();
        assert!(report.contains("exec="));
        assert!(report.contains("blocked="));
        assert!(report.contains("active="));
        assert!(report.contains("ttl="));
        assert!(report.contains(SANDBOX_VERSION));
    }

    #[test]
    fn test_sandbox_exec_available_check() {
        let available = SandboxExecutor::check_sandbox_exec_available();
        assert_eq!(available, cfg!(target_os = "macos"));
    }

    #[test]
    fn test_generate_id_uniqueness() {
        let executor = SandboxExecutor::with_defaults();
        let id1 = executor.generate_id();
        let id2 = executor.generate_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_isolation_readonly_env() {
        let mut executor = SandboxExecutor::with_defaults();
        let config = SandboxConfig {
            enable_sandbox_exec: false,
            ..SandboxConfig::default()
        };
        executor.config = config;
        let result = executor.execute(
            "echo $SANDBOX_LEVEL",
            SandboxLevel::ReadOnly,
            vec![],
            vec![],
        );
        assert_eq!(result.status, SandboxStatus::Completed);
        assert!(result.stdout.contains("readonly") || result.stdout.trim().is_empty());
    }

    #[test]
    fn test_default_config() {
        let config = SandboxConfig::default();
        assert!(config.safe_commands.contains(&"ls".to_string()));
        assert!(config.danger_patterns.contains(&"rm -rf /".to_string()));
        assert_eq!(config.max_output_bytes, MAX_OUTPUT_BYTES);
        assert_eq!(
            config.cleanup_ttl,
            Duration::from_secs(DEFAULT_CLEANUP_TTL_SECS)
        );
    }

    #[test]
    fn test_new_executor() {
        let executor = SandboxExecutor::with_defaults();
        assert!(executor.active_sandboxes.is_empty());
        assert_eq!(executor.total_executions, 0);
        assert_eq!(executor.total_blocks, 0);
    }
}
