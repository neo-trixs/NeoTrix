//! P3: Computer First-Class Abstraction — 一等计算设备抽象
//!
//! 吸收 cumora COMPUTER.md: "define Computer as an abstraction for the machine
//! the agent works on, rather than having it read files, check disk, etc."
//!
//! 核心: **抽象物理计算机为一等对象**, agent 通过 `NtComputer` trait 与机器交互,
//! 而非直接调用 std::fs / std::process / platform APIs。
//!
//! 好处:
//! - 可测试: mock NtComputer 即可在无文件系统环境测试 agent 逻辑
//! - 可替换: 切换 SSH 远程 / Docker / 物理机, agent 代码不变
//! - 可审计: 所有机器交互经过 trait, 可记录/限制/回放

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 文件系统操作结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    pub content: Vec<u8>,
    pub metadata: FileMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub size: u64,
    pub modified_secs: u64,
    pub is_dir: bool,
}

/// 进程执行结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// 系统信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub working_dir: PathBuf,
    pub home_dir: Option<PathBuf>,
    pub cpu_count: usize,
    pub total_memory_mb: u64,
}

/// 屏幕状态 (用于 GUI agent 场景)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenState {
    pub width: u32,
    pub height: u32,
    pub screenshot_png: Option<Vec<u8>>,
    pub cursor_position: Option<(u32, u32)>,
}

/// NtComputer trait — 一等计算设备抽象。
///
/// 对齐 cumora "Computer as abstraction" 原则:
/// agent 不直接 std::fs / std::process, 而是通过此 trait。
///
/// 设计:
/// - `&self` methods: 信息查询 (只读)
/// - `&mut self` methods: 状态修改 (文件写入/进程执行)
/// - 所有方法返回 Result, 不 panic
/// - 不使用 unsafe (R-P1)
pub trait NtComputer {
    // ─── 文件系统 ───

    /// 读取文件内容。
    fn read_file(&self, path: &Path) -> Result<FileResult, ComputerError>;

    /// 写入文件内容。
    fn write_file(&mut self, path: &Path, content: &[u8]) -> Result<(), ComputerError>;

    /// 列出目录内容。
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, ComputerError>;

    /// 检查路径是否存在。
    fn path_exists(&self, path: &Path) -> bool;

    /// 获取文件元数据。
    fn file_metadata(&self, path: &Path) -> Result<FileMetadata, ComputerError>;

    // ─── 进程 ───

    /// 执行命令并返回结果。
    fn run_command(
        &mut self,
        program: &str,
        args: &[String],
    ) -> Result<ProcessResult, ComputerError>;

    /// 获取当前环境变量。
    fn env_var(&self, key: &str) -> Option<String>;

    // ─── 系统信息 ───

    /// 获取系统信息。
    fn system_info(&self) -> SystemInfo;

    /// 获取当前时间戳 (epoch seconds)。
    fn current_time_secs(&self) -> u64;

    // ─── 屏幕 (可选, GUI agent) ───

    /// 截屏 (如平台支持)。默认返回 None。
    fn screenshot(&self) -> Option<ScreenState> {
        None
    }

    /// 点击屏幕坐标 (如平台支持)。默认无操作。
    fn click(&mut self, _x: u32, _y: u32) -> Result<(), ComputerError> {
        Ok(())
    }

    /// 输入文本 (如平台支持)。默认无操作。
    fn type_text(&mut self, _text: &str) -> Result<(), ComputerError> {
        Ok(())
    }
}

/// Computer 操作错误。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputerError {
    /// 文件不存在。
    FileNotFound(String),
    /// 权限不足。
    PermissionDenied(String),
    /// IO 错误。
    IoError(String),
    /// 命令执行失败。
    ProcessError { exit_code: i32, stderr: String },
    /// 不支持的操作 (如无 GUI 环境下的截屏)。
    Unsupported(String),
    /// 其他错误。
    Other(String),
}

impl std::fmt::Display for ComputerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(p) => write!(f, "file not found: {}", p),
            Self::PermissionDenied(p) => write!(f, "permission denied: {}", p),
            Self::IoError(e) => write!(f, "io error: {}", e),
            Self::ProcessError { exit_code, stderr } => {
                write!(f, "process exited {}: {}", exit_code, stderr)
            }
            Self::Unsupported(op) => write!(f, "unsupported: {}", op),
            Self::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ComputerError {}

/// LocalComputer — 本地文件系统 + 进程实现。
///
/// 最简单的 NtComputer 实现, 直接调用 std::fs / std::process。
/// 用于开发/测试。生产环境可替换为 RemoteComputer (SSH) / DockerComputer 等。
pub struct LocalComputer {
    env_overrides: std::collections::HashMap<String, String>,
}

impl LocalComputer {
    pub fn new() -> Self {
        Self {
            env_overrides: std::collections::HashMap::new(),
        }
    }

    /// 设置环境变量覆盖 (用于测试)。
    pub fn set_env(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.env_overrides.insert(key.into(), value.into());
    }
}

impl Default for LocalComputer {
    fn default() -> Self {
        Self::new()
    }
}

impl NtComputer for LocalComputer {
    fn read_file(&self, path: &Path) -> Result<FileResult, ComputerError> {
        let content = std::fs::read(path)
            .map_err(|e| ComputerError::IoError(format!("{}: {}", path.display(), e)))?;
        let metadata = std::fs::metadata(path)
            .map_err(|e| ComputerError::IoError(format!("metadata {}: {}", path.display(), e)))?;
        Ok(FileResult {
            content,
            metadata: FileMetadata {
                size: metadata.len(),
                modified_secs: metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                is_dir: metadata.is_dir(),
            },
        })
    }

    fn write_file(&mut self, path: &Path, content: &[u8]) -> Result<(), ComputerError> {
        // 创建父目录
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ComputerError::IoError(format!("mkdir {}: {}", parent.display(), e)))?;
        }
        std::fs::write(path, content)
            .map_err(|e| ComputerError::IoError(format!("write {}: {}", path.display(), e)))
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, ComputerError> {
        let entries = std::fs::read_dir(path)
            .map_err(|e| ComputerError::IoError(format!("readdir {}: {}", path.display(), e)))?;
        let mut result = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| ComputerError::IoError(e.to_string()))?;
            result.push(entry.path());
        }
        Ok(result)
    }

    fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn file_metadata(&self, path: &Path) -> Result<FileMetadata, ComputerError> {
        let m = std::fs::metadata(path)
            .map_err(|e| ComputerError::IoError(format!("metadata {}: {}", path.display(), e)))?;
        Ok(FileMetadata {
            size: m.len(),
            modified_secs: m
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0),
            is_dir: m.is_dir(),
        })
    }

    fn run_command(
        &mut self,
        program: &str,
        args: &[String],
    ) -> Result<ProcessResult, ComputerError> {
        let output = std::process::Command::new(program)
            .args(args)
            .output()
            .map_err(|e| ComputerError::ProcessError {
                exit_code: -1,
                stderr: format!("failed to exec {}: {}", program, e),
            })?;
        Ok(ProcessResult {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    fn env_var(&self, key: &str) -> Option<String> {
        if let Some(v) = self.env_overrides.get(key) {
            return Some(v.clone());
        }
        std::env::var(key).ok()
    }

    fn system_info(&self) -> SystemInfo {
        SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            hostname: std::env::var("HOSTNAME")
                .unwrap_or_else(|_| "localhost".into()),
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            home_dir: std::env::var("HOME")
                .ok()
                .map(PathBuf::from),
            cpu_count: num_cpus::get(),
            total_memory_mb: 0, // 需要 platform-specific 实现, 暂返回 0
        }
    }

    fn current_time_secs(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// MockComputer — 测试用 mock 实现。
///
/// 预编程所有响应, 用于单元测试 agent 逻辑而不依赖真实文件系统。
pub struct MockComputer {
    files: std::collections::HashMap<PathBuf, Vec<u8>>,
    commands: Vec<ProcessResult>,
    command_index: usize,
    pub system_info: SystemInfo,
}

impl MockComputer {
    pub fn new() -> Self {
        Self {
            files: std::collections::HashMap::new(),
            commands: Vec::new(),
            command_index: 0,
            system_info: SystemInfo {
                os: "mock".into(),
                arch: "mock".into(),
                hostname: "mock-host".into(),
                working_dir: PathBuf::from("/mock"),
                home_dir: Some(PathBuf::from("/mock/home")),
                cpu_count: 4,
                total_memory_mb: 8192,
            },
        }
    }

    /// 预置文件内容。
    pub fn with_file(mut self, path: impl Into<PathBuf>, content: Vec<u8>) -> Self {
        self.files.insert(path.into(), content);
        self
    }

    /// 预置命令返回值。
    pub fn with_command(mut self, result: ProcessResult) -> Self {
        self.commands.push(result);
        self
    }
}

impl Default for MockComputer {
    fn default() -> Self {
        Self::new()
    }
}

impl NtComputer for MockComputer {
    fn read_file(&self, path: &Path) -> Result<FileResult, ComputerError> {
        self.files
            .get(path)
            .map(|content| FileResult {
                content: content.clone(),
                metadata: FileMetadata {
                    size: content.len() as u64,
                    modified_secs: 0,
                    is_dir: false,
                },
            })
            .ok_or_else(|| ComputerError::FileNotFound(path.display().to_string()))
    }

    fn write_file(&mut self, path: &Path, content: &[u8]) -> Result<(), ComputerError> {
        self.files.insert(path.to_path_buf(), content.to_vec());
        Ok(())
    }

    fn list_dir(&self, _path: &Path) -> Result<Vec<PathBuf>, ComputerError> {
        Ok(self.files.keys().cloned().collect())
    }

    fn path_exists(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    fn file_metadata(&self, path: &Path) -> Result<FileMetadata, ComputerError> {
        self.files
            .get(path)
            .map(|c| FileMetadata {
                size: c.len() as u64,
                modified_secs: 0,
                is_dir: false,
            })
            .ok_or_else(|| ComputerError::FileNotFound(path.display().to_string()))
    }

    fn run_command(
        &mut self,
        _program: &str,
        _args: &[String],
    ) -> Result<ProcessResult, ComputerError> {
        if self.command_index < self.commands.len() {
            let result = self.commands[self.command_index].clone();
            self.command_index += 1;
            Ok(result)
        } else {
            Ok(ProcessResult {
                stdout: String::new(),
                stderr: "mock: no more pre-programmed responses".into(),
                exit_code: 0,
            })
        }
    }

    fn env_var(&self, key: &str) -> Option<String> {
        match key {
            "HOME" => self.system_info.home_dir.as_ref().map(|p| p.to_string_lossy().into_owned()),
            _ => None,
        }
    }

    fn system_info(&self) -> SystemInfo {
        self.system_info.clone()
    }

    fn current_time_secs(&self) -> u64 {
        1_700_000_000 // 固定时间戳, 测试可预测
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_read_write() {
        let mut mock = MockComputer::new()
            .with_file("/test.txt", b"hello".to_vec());

        let result = mock.read_file(Path::new("/test.txt")).unwrap();
        assert_eq!(result.content, b"hello");

        mock.write_file(Path::new("/new.txt"), b"world").unwrap();
        assert!(mock.path_exists(Path::new("/new.txt")));
    }

    #[test]
    fn test_mock_command() {
        let mut mock = MockComputer::new().with_command(ProcessResult {
            stdout: "ok".into(),
            stderr: String::new(),
            exit_code: 0,
        });

        let r = mock.run_command("echo", &["test".into()]).unwrap();
        assert_eq!(r.stdout, "ok");
        assert_eq!(r.exit_code, 0);

        // 第二次调用返回默认值
        let r2 = mock.run_command("echo", &["test".into()]).unwrap();
        assert_eq!(r2.exit_code, 0);
    }

    #[test]
    fn test_mock_env_var() {
        let mock = MockComputer::new();
        assert_eq!(mock.env_var("HOME"), Some("/mock/home".into()));
        assert_eq!(mock.env_var("NONEXISTENT"), None);
    }

    #[test]
    fn test_mock_system_info() {
        let mock = MockComputer::new();
        let info = mock.system_info();
        assert_eq!(info.os, "mock");
        assert_eq!(info.cpu_count, 4);
    }

    #[test]
    fn test_mock_file_not_found() {
        let mock = MockComputer::new();
        let r = mock.read_file(Path::new("/nonexistent"));
        assert!(matches!(r, Err(ComputerError::FileNotFound(_))));
    }

    #[test]
    fn test_error_display() {
        let e = ComputerError::FileNotFound("/foo".into());
        assert!(format!("{}", e).contains("/foo"));
    }
}
