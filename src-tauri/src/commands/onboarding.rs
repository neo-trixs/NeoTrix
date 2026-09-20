//! Onboarding Wizard — 首次运行依赖检查 + 配置引导
//!
//! 检查系统依赖 (Node.js, git, cargo, etc.) 并返回状态给前端。

use crate::atomic_io;
use crate::ipc;
use crate::ipc::IpcResponse;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Prerequisite check result for a single tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prerequisite {
    /// Tool name
    pub name: String,
    /// Whether it's installed
    pub installed: bool,
    /// Detected version (if installed)
    pub version: Option<String>,
    /// Install command (if not installed)
    pub install_hint: Option<String>,
    /// Whether it's required (vs optional)
    pub required: bool,
}

/// Overall onboarding status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStatus {
    /// All prerequisites
    pub prerequisites: Vec<Prerequisite>,
    /// Whether all required tools are installed
    pub all_required_met: bool,
    /// Whether this is first run (no prior config)
    pub first_run: bool,
    /// System info
    pub system: SystemInfo,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub hostname: Option<String>,
}

/// Check all prerequisites
#[tauri::command]
pub fn onboarding_check_prereqs() -> IpcResponse<OnboardingStatus> {
    let prerequisites = vec![
        check_tool("git", "git", "--version", Some("brew install git"), true),
        check_tool("node", "node", "--version", Some("brew install node"), true),
        check_tool("npm", "npm", "--version", Some("brew install npm"), true),
        check_tool("cargo", "cargo", "--version", Some("curl https://sh.rustup.rs -sSf | sh"), true),
        check_tool("rustc", "rustc", "--version", None, true),
        check_tool("python3", "python3", "--version", Some("brew install python3"), false),
        check_tool("ffmpeg", "ffmpeg", "-version", Some("brew install ffmpeg"), false),
        check_tool("ollama", "ollama", "--version", Some("brew install ollama"), false),
    ];

    let all_required_met = prerequisites
        .iter()
        .filter(|p| p.required)
        .all(|p| p.installed);

    let system = SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        hostname: hostname::get().ok().map(|h| h.to_string_lossy().to_string()),
    };

    ipc::ok(OnboardingStatus {
        prerequisites,
        all_required_met,
        first_run: !config_dir_exists(),
        system,
    })
}

/// Check if a config directory exists (indicates not first run)
fn config_dir_exists() -> bool {
    dirs::home_dir()
        .map(|h| h.join(".neotrix").exists())
        .unwrap_or(false)
}

/// Check a single tool
fn check_tool(name: &str, cmd: &str, version_flag: &str, install_hint: Option<&str>, required: bool) -> Prerequisite {
    let output = Command::new(cmd)
        .arg(version_flag)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version = stdout
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            Prerequisite {
                name: name.to_string(),
                installed: true,
                version: Some(version),
                install_hint: None,
                required,
            }
        }
        _ => Prerequisite {
            name: name.to_string(),
            installed: false,
            version: None,
            install_hint: install_hint.map(|s| s.to_string()),
            required,
        },
    }
}

/// Get onboarding tips
#[tauri::command]
pub fn onboarding_get_tips() -> Vec<String> {
    vec![
        "使用 /help 查看所有可用命令".to_string(),
        "按 ⌘K 打开命令面板".to_string(),
        "按 ⌘3 切换终端面板".to_string(),
        "拖拽文件到聊天窗口可以上传".to_string(),
        "在设置中配置 API 密钥以使用完整功能".to_string(),
        "使用 /model 切换 AI 模型".to_string(),
    ]
}

/// Mark onboarding as completed
#[tauri::command]
pub fn onboarding_complete() -> IpcResponse<()> {
    let config_dir = match dirs::home_dir() {
        Some(h) => h.join(".neotrix"),
        None => return ipc::err("HOME_DIR_NOT_FOUND", "Cannot find home directory"),
    };
    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        return ipc::err("CONFIG_DIR_FAILED", format!("Failed to create config dir: {}", e));
    }
    let marker = config_dir.join(".onboarded");
    if let Err(e) = atomic_io::write_atomic(&marker, b"1") {
        return ipc::err("MARKER_WRITE_FAILED", format!("Failed to write onboarding marker: {}", e));
    }
    ipc::ok(())
}

/// Check if onboarding has been completed
#[tauri::command]
pub fn onboarding_is_completed() -> bool {
    dirs::home_dir()
        .map(|h| h.join(".neotrix").join(".onboarded").exists())
        .unwrap_or(false)
}
