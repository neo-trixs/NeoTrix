//! nt_types — 共享类型 (CleanupKind/Platform/CleanupRiskLevel/CleanupPattern + 清理日志)，行为零变更纯搬移.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// ============================================================
// 清理日志系统 (.cleanup/log/history.jsonl)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _CleanupLogEntry {
    pub action: String, // "scan" | "clean" | "archive" | "backup"
    pub kind: String,
    pub items: usize,
    pub bytes: u64,
    pub batch_id: String,
    pub success: bool,
    pub error: Option<String>,
}

pub struct _CleanupLog;

impl _CleanupLog {
    pub fn log(log_dir: &Path, entry: &_CleanupLogEntry) {
        let file = log_dir.join("history.jsonl");
        let line = serde_json::to_string(&entry).unwrap_or_default();
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&file) {
            use std::io::Write;
            let _ = writeln!(f, "{}", line);
        }
    }

    /// 读取最近 N 条日志
    pub fn recent(log_dir: &Path, n: usize) -> Vec<_CleanupLogEntry> {
        let file = log_dir.join("history.jsonl");
        let content = match fs::read_to_string(&file) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        content
            .lines()
            .filter_map(|l| serde_json::from_str(l).ok())
            .rev()
            .take(n)
            .collect()
    }
}

// ============================================================
// 原有的 CleanupEngine 保留并增强
// ============================================================

/// 清理类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupKind {
    ProjectArtifacts,
    Cache,
    Logs,
    TempFiles,
    MemoryPrune,
    BrainSnapshot,
    IDECaches,
    SystemServices,
    ProjectMolting,
    All,
}

impl CleanupKind {
    pub fn description(&self) -> &'static str {
        match self {
            CleanupKind::ProjectArtifacts => {
                "项目构建产物 (target/, node_modules/, .build/, dist/, venv/)"
            }
            CleanupKind::Cache => "系统缓存 (~/Library/Caches, .cache, pip, cargo, AI 模型缓存)",
            CleanupKind::Logs => "日志文件 (*.log, *.out, 系统日志)",
            CleanupKind::TempFiles => "临时文件 (/tmp, /var/tmp, ~/tmp)",
            CleanupKind::MemoryPrune => "推理记忆修剪 (低奖励记忆, 过期轨迹)",
            CleanupKind::BrainSnapshot => "大脑快照清理 (保留最近 N 个快照)",
            CleanupKind::IDECaches => "IDE 缓存 (Cursor, VS Code, IntelliJ, Xcode derived data)",
            CleanupKind::SystemServices => {
                "系统服务命令清理 (brew cleanup, tmutil 快照, docker system prune)"
            }
            CleanupKind::ProjectMolting => {
                "项目蜕皮归档 (旧躯壳目录 → _archive/, 活动树只留最新态)"
            }
            CleanupKind::All => "全部清理 (包含以上所有类别)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    #[serde(rename = "macos")]
    MacOS,
    #[serde(rename = "windows")]
    Windows,
    #[serde(rename = "linux")]
    Linux,
    #[serde(rename = "all")]
    All,
}

impl Platform {
    pub fn current() -> Platform {
        #[cfg(target_os = "macos")]
        {
            return Platform::MacOS;
        }
        #[cfg(target_os = "windows")]
        {
            return Platform::Windows;
        }
        #[cfg(target_os = "linux")]
        {
            return Platform::Linux;
        }
        #[allow(unreachable_code)]
        {
            Platform::All
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Platform::MacOS => "macos",
            Platform::Windows => "windows",
            Platform::Linux => "linux",
            Platform::All => "all",
        }
    }

    pub fn matches(&self, p: Platform) -> bool {
        *self == Platform::All || p == Platform::All || *self == p
    }
}

/// 风险分级 — 驱动 CLI 交互 (MacBroom/DeepPurge Safe·Moderate·Advanced 参照)
/// 
/// For the unified RiskLevel, see `neotrix_types::RiskLevel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[derive(Default)]
pub enum CleanupRiskLevel {
    #[serde(rename = "low")]
    #[default]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
}

impl CleanupRiskLevel {
    pub fn label(&self) -> &'static str {
        match self {
            CleanupRiskLevel::Low => "低危",
            CleanupRiskLevel::Medium => "中危",
            CleanupRiskLevel::High => "高危",
        }
    }
}


/// 跨平台清理规则 — platform 门控 + risk 分级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPattern {
    pub name: &'static str,
    pub kind: CleanupKind,
    pub patterns: Vec<&'static str>,
    pub max_age_days: Option<i64>,
    pub safe: bool,
    pub recursive: bool,
    #[serde(default = "platform_all")]
    pub platform: Platform,
    #[serde(default)]
    pub risk: CleanupRiskLevel,
    pub description: Option<&'static str>,
}

fn platform_all() -> Platform {
    Platform::All
}

impl CleanupPattern {
    /// 当前平台生效的规则 (平台门控 + 风险阀)
    pub(crate) fn _active_below(&self, max_risk: CleanupRiskLevel) -> bool {
        self.platform.matches(Platform::current()) && self.risk <= max_risk
    }

    pub fn all_patterns() -> Vec<Self> {
        let mac = Platform::MacOS;
        let win = Platform::Windows;
        let lin = Platform::Linux;
        let all = Platform::All;
        vec![
            // ---- 项目构建产物 (跨平台, Mole 34 目标) ----
            Self { name: "Rust build artifacts", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/target/**"], max_age_days: Some(7), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: Some("target/ 为可重建构建产物") },
            Self { name: "Node.js modules", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/node_modules/**"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: Some("npm/yarn/pnpm install 重建") },
            Self { name: "Python venv", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/.venv/**", "**/venv/**", "**/.tox/**", "**/.nox/**"], max_age_days: Some(60), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "Build output", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/dist/**", "**/.build/**", "**/build/**", "**/out/**"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "Next.js cache", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/.next/**", "**/.nuxt/**", "**/.output/**", "**/.svelte-kit/**", "**/.astro/**"], max_age_days: Some(7), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "Swift build", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/.build/**"], max_age_days: Some(30), safe: true, recursive: true, platform: mac, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "Go test artifacts", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/*.test", "**/*.test.exe", "**/coverage.out", "**/coverage.html"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: Some("go test 编译产物与覆盖率; vendor/ 为依赖源码不删") },
            Self { name: "Turbo/Parcel cache", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/.turbo/**", "**/.parcel-cache/**", "**/.angular/**", "**/.dart_tool/**", "**/.zig-cache/**", "**/zig-out/**"], max_age_days: Some(15), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "Test caches", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/.pytest_cache/**", "**/.mypy_cache/**", "**/.ruff_cache/**", "**/coverage/**", "**/__pycache__/**"], max_age_days: Some(7), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            // ---- 项目蜕皮: 旧躯壳目录 (legacy/old/backup 命名的旧版本, 归档到 _archive/) ----
            Self { name: "Legacy shells", kind: CleanupKind::ProjectMolting, patterns: vec!["**/legacy/**", "**/*_legacy/**", "**/legacy_*/**", "**/old_*/**", "**/*_old/**", "**/*_v0/**", "**/*_v1/**", "**/*_backup*/**"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: Some("旧躯壳目录 (旧版本代码), 蜕皮归档至 .cleanup/archive/ 而非删除") },
            Self { name: "iOS derived data", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Developer/Xcode/DerivedData/**", "~/Library/Developer/CoreSimulator/Caches/**"], max_age_days: Some(30), safe: true, recursive: true, platform: mac, risk: CleanupRiskLevel::Medium, description: None },
            // ---- 包管理缓存 ----
            Self { name: "Cargo registry cache", kind: CleanupKind::Cache, patterns: vec!["~/.cargo/registry/cache/**", "~/.cargo/git/db/**"], max_age_days: Some(90), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "pip cache", kind: CleanupKind::Cache, patterns: vec!["~/.cache/pip/**", "%LOCALAPPDATA%/pip/cache/**"], max_age_days: Some(90), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "npm cache", kind: CleanupKind::Cache, patterns: vec!["~/.npm/_cacache/**", "%APPDATA%/npm-cache/**"], max_age_days: Some(90), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "pnpm store", kind: CleanupKind::Cache, patterns: vec!["~/Library/Caches/pnpm/**", "~/.local/share/pnpm/store/**", "%LOCALAPPDATA%/pnpm-cache/**"], max_age_days: Some(90), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "yarn cache", kind: CleanupKind::Cache, patterns: vec!["~/.cache/yarn/**"], max_age_days: Some(90), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "bun cache", kind: CleanupKind::Cache, patterns: vec!["~/.bun/install/cache/**"], max_age_days: Some(90), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Medium, description: Some("仅 bun install 缓存; ~/.bun/bin(可执行)与全局包保留") },
            Self { name: "uv pip cache", kind: CleanupKind::Cache, patterns: vec!["~/.cache/uv/**"], max_age_days: Some(90), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "go build cache", kind: CleanupKind::Cache, patterns: vec!["~/Library/Caches/go-build/**", "~/.cache/go-build/**", "%LOCALAPPDATA%/go-build/**"], max_age_days: Some(60), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "conda pkgs", kind: CleanupKind::Cache, patterns: vec!["~/miniconda3/pkgs/**", "~/anaconda3/pkgs/**", "~/.conda/pkgs/**"], max_age_days: Some(60), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Medium, description: None },
            // ---- 浏览器缓存 ----
            Self { name: "Google Chrome cache", kind: CleanupKind::Cache, patterns: vec!["~/Library/Caches/Google/Chrome/**", "%LOCALAPPDATA%/Google/Chrome/User Data/Default/Cache/**"], max_age_days: Some(15), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "Playwright browsers", kind: CleanupKind::Cache, patterns: vec!["~/Library/Caches/ms-playwright/**", "~/Library/Caches/ms-playwright-go/**"], max_age_days: Some(30), safe: true, recursive: true, platform: mac, risk: CleanupRiskLevel::Low, description: Some("浏览器自动化引擎, 重装下载") },
            // ---- AI 应用缓存 (PureMac AI Apps 吸收: Ollama/LM Studio 模型缓存) ----
            Self { name: "Ollama model cache", kind: CleanupKind::Cache, patterns: vec!["~/.ollama/models/blobs/**"], max_age_days: Some(90), safe: false, recursive: true, platform: all, risk: CleanupRiskLevel::Medium, description: Some("Ollama 模型 blob 缓存, 可 ollama pull 重建; 谨慎, 仅清未引用 blob") },
            Self { name: "LM Studio model cache", kind: CleanupKind::Cache, patterns: vec!["~/.lmstudio/models/**"], max_age_days: Some(90), safe: false, recursive: true, platform: all, risk: CleanupRiskLevel::Medium, description: Some("LM Studio 模型缓存, 可重新下载") },
            Self { name: "MCP/Agent hub cache", kind: CleanupKind::Cache, patterns: vec!["~/.cache/claude/**", "~/.cache/opencode/**", "~/.cache/codex/**"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: Some("AI agent 工具缓存 (工具响应/索引, 可重建)") },
            Self { name: "Homebrew cache", kind: CleanupKind::Cache, patterns: vec!["~/Library/Caches/Homebrew/**", "~/Library/Caches/Homebrew/downloads/**", "/opt/homebrew/Library/Homebrew/vendor/**"], max_age_days: Some(30), safe: true, recursive: true, platform: mac, risk: CleanupRiskLevel::Low, description: Some("brew 下载缓存与 vendor ruby; brew cleanup 等价, 重装即重建") },
            // ---- Xcode 扩展 (mac-janitor: Archives/Simulators, 现仅 DerivedData) ----
            Self { name: "Xcode Archives", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Developer/Xcode/Archives/**"], max_age_days: Some(90), safe: false, recursive: true, platform: mac, risk: CleanupRiskLevel::Medium, description: Some("已归档的 App 构建产物, 含 dSYM; 确认无需再上传后清理") },
            Self { name: "Xcode Simulator runtimes", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Developer/CoreSimulator/Images/**", "~/Library/Developer/CoreSimulator/Caches/**"], max_age_days: Some(30), safe: true, recursive: true, platform: mac, risk: CleanupRiskLevel::Medium, description: Some("模拟器运行时镜像缓存; 删除后按需重下") },
            Self { name: "Xcode module caches", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Developer/Xcode/DerivedData/**/ModuleCache.noindex/**", "~/Library/Developer/Xcode/DerivedData/**/PrecompiledHeaders/**"], max_age_days: Some(7), safe: true, recursive: true, platform: mac, risk: CleanupRiskLevel::Medium, description: Some("Swift/ObjC 模块预编译缓存, 可重建") },
            // ---- temp ----
            Self { name: "System temp", kind: CleanupKind::TempFiles, patterns: vec!["/tmp/**", "/var/tmp/**", "%TEMP%/**", "%TMP%/**"], max_age_days: Some(1), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "Preload temp", kind: CleanupKind::TempFiles, patterns: vec!["%WINDIR%/Prefetch/**"], max_age_days: Some(1), safe: true, recursive: true, platform: win, risk: CleanupRiskLevel::Medium, description: None },
            // ---- IDE 缓存 ----
            Self { name: "VS Code caches", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Application Support/Code/CachedData/**", "~/.vscode/extensions/.cache/**", "%APPDATA%/Code/CachedData/**"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "Cursor caches", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Application Support/Cursor/CachedData/**", "~/.cursor/cache/**"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "IntelliJ caches", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Caches/JetBrains/**", "~/.cache/JetBrains/**", "%LOCALAPPDATA%/JetBrains/**"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: CleanupRiskLevel::Low, description: None },
            Self { name: "Docker", kind: CleanupKind::Cache, patterns: vec!["~/Library/Containers/com.docker.docker/Data/vms/0/data/DockerDesktopRegular5/.cache/**"], max_age_days: Some(60), safe: true, recursive: true, platform: mac, risk: CleanupRiskLevel::Medium, description: None },
            // ---- Linux 系统缓存 ----
            Self { name: "apt/dnf/pacman package cache", kind: CleanupKind::Cache, patterns: vec!["/var/cache/apt/archives/**", "/var/cache/dnf/**", "/var/cache/pacman/pkg/**", "/var/tmp/**"], max_age_days: Some(30), safe: true, recursive: true, platform: lin, risk: CleanupRiskLevel::Medium, description: None },
            Self { name: "Linux user cache", kind: CleanupKind::Cache, patterns: vec!["~/.cache/**"], max_age_days: Some(30), safe: true, recursive: true, platform: lin, risk: CleanupRiskLevel::Low, description: None },
        ]
    }

    /// 展开路径占位符 (~, %OS% 专用变量), 供 scan 使用
    pub fn expand(pat: &str) -> String {
        let home = dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        #[allow(unused_mut)] // windows 块被 cfg 移除时无需 mut
        let mut s = pat.replace("~", &home);
        #[cfg(target_os = "windows")]
        {
            s = s.replace(
                "%LOCALAPPDATA%",
                &std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
                    std::env::var("USERPROFILE")
                        .map(|u| format!("{}\\AppData\\Local", u))
                        .unwrap_or_default()
                }),
            );
            s = s.replace("%APPDATA%", &std::env::var("APPDATA").unwrap_or_default());
            s = s.replace(
                "%TEMP%",
                &std::env::var("TEMP").unwrap_or_else(|_| std::env::var("TMP").unwrap_or_default()),
            );
            s = s.replace(
                "%WINDIR%",
                &std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into()),
            );
        }
        s
    }

    /// 检查目录是否带 CACHEDIR.TAG 缓存签名 (Mole: 以 Signature: 开头的文件即缓存)
    pub(crate) fn _has_cachedir_tag(dir: &Path) -> bool {
        let tag = dir.join("CACHEDIR.TAG");
        if let Ok(content) = fs::read_to_string(&tag) {
            if let Some(first) = content.lines().next() {
                return first.trim_start()
                    == "Signature: 8a477f597d02d456d45674aa7d611ef7b6c14a01bccaebbd4e53c5d4f";
            }
        }
        false
    }

    /// 路径安全护栏: 拒绝系统根/project_root 自身 (Mole 禁删 /, $HOME, $HOME/Library)
    pub(crate) fn _is_system_root_dir(p: &Path) -> bool {
        let home = dirs::home_dir().unwrap_or_default();
        let canonical = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
        // 两侧都 canonicalize: /var → /private/var (macOS symlink), 否则保护失效 (误删风险)
        let canonical_home = std::fs::canonicalize(&home).unwrap_or(home.clone());
        canonical == canonical_home
            || canonical.starts_with(canonical_home.join("Library"))
            || p == std::path::Path::new("/")
            || p == std::path::Path::new("\\")
    }

    /// 估算路径体积: 目录递归累加子项 (受安全护栏约束), 文件取其 len
    pub(crate) fn _entry_size(p: &Path) -> u64 {
        match std::fs::metadata(p) {
            Ok(m) if m.is_file() => m.len(),
            Ok(m) if m.is_dir() => {
                // 跳过系统根目录防误扫 (护栏: _is_system_root_dir 判定后仍不遍历)
                if Self::_is_system_root_dir(p) {
                    return m.len();
                }
                let mut total = m.len();
                if let Ok(rd) = std::fs::read_dir(p) {
                    let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
                    entries.truncate(256);
                    for e in entries {
                        let ep = e.path();
                        // 符号链接不跟随 (防循环), 仅累加真实子目录/文件
                        if std::fs::symlink_metadata(&ep)
                            .map(|sm| sm.file_type().is_symlink())
                            .unwrap_or(false)
                        {
                            continue;
                        }
                        if ep.is_dir() {
                            total = total.saturating_add(Self::_entry_size(&ep));
                        } else if let Ok(em) = std::fs::metadata(&ep) {
                            total = total.saturating_add(em.len());
                        }
                    }
                }
                total
            }
            _ => 0,
        }
    }
}
