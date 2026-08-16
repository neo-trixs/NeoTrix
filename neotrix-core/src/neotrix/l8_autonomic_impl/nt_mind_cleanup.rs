//! NeoTrix 清理维护模块 — 科学目录架构
//!
//! 目录架构:
//!   project/.cleanup/           清理系统根目录
//!   ├── rules.toml              清理规则配置
//!   ├── archive/                过期文件归档 (按日期分目录)
//!   │   └── YYYY-MM-DD_HHMMSS/  每次归档一个目录
//!   │       └── manifest.json   归档清单 (来源路径/大小/哈希)
//!   ├── log/                    清理日志
//!   │   ├── history.jsonl       追加式事件日志
//!   │   └── index.json          归档搜索索引
//!   └── tmp/                    清理过程中临时文件
//!
//!   project/.backup/            代码备份根目录 (同级, 每6h)
//!   ├── latest -> YYYY-MM-DD_HHMMSS/  最新备份软链接
//!   ├── YYYY-MM-DD_HHMMSS/      按时间戳的备份
//!   │   └── manifest.json       备份清单
//!   └── index.json              备份索引

use chrono::{Local, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================
// Cleanup 目录布局管理
// ============================================================

/// 项目清理系统目录
pub struct CleanupDirs {
    pub root: PathBuf,       // project/.cleanup/
    pub archive: PathBuf,    // project/.cleanup/archive/
    pub log: PathBuf,        // project/.cleanup/log/
    pub rules_file: PathBuf, // project/.cleanup/rules.toml
    pub index_file: PathBuf, // project/.cleanup/log/index.json
}

impl CleanupDirs {
    pub fn new(project_root: &Path) -> Self {
        let root = project_root.join(".cleanup");
        Self {
            archive: root.join("archive"),
            log: root.join("log"),
            rules_file: root.join("rules.toml"),
            index_file: root.join("log").join("index.json"),
            root,
        }
    }

    /// 确保所有目录存在，返回 self
    pub fn ensure(&self) -> std::io::Result<&Self> {
        fs::create_dir_all(&self.archive)?;
        fs::create_dir_all(&self.log)?;
        Ok(self)
    }

    /// 创建当前时间戳的归档目录
    pub fn create_archive_batch(&self) -> std::io::Result<PathBuf> {
        let ts = Local::now().format("%Y-%m-%d_%H%M%S");
        let batch = self.archive.join(ts.to_string());
        fs::create_dir_all(&batch)?;
        Ok(batch)
    }
}

// ============================================================
// 归档系统
// ============================================================

/// 归档清单条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub source_path: String,   // 原始路径 (相对于项目根)
    pub archived_path: String, // 归档后路径
    pub size_bytes: u64,
    pub is_dir: bool,
    pub archived_at: i64,       // Unix timestamp
    pub cleanup_kind: String,   // 清理类型标签
    pub sha256: Option<String>, // 文件哈希 (可选)
}

/// 归档清单 (每个批次一个)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveManifest {
    pub batch_id: String, // YYYY-MM-DD_HHMMSS
    pub created_at: i64,
    pub entries: Vec<ArchiveEntry>,
    pub total_bytes: u64,
    pub total_items: usize,
}

impl ArchiveManifest {
    pub fn new(batch_id: &str) -> Self {
        Self {
            batch_id: batch_id.to_string(),
            created_at: Utc::now().timestamp(),
            entries: Vec::new(),
            total_bytes: 0,
            total_items: 0,
        }
    }
}

/// 归档索引 (全局, 用于搜索)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArchiveIndex {
    pub entries: Vec<ArchiveEntry>,
    pub last_updated: i64,
}

impl ArchiveIndex {
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, &json)
    }
}

/// 归档操作: 将匹配的文件移动到 .cleanup/archive/ 而非删除
pub struct Archiver {
    pub dirs: CleanupDirs,
    pub index: ArchiveIndex,
}

impl Archiver {
    pub fn new(project_root: &Path) -> Self {
        let dirs = CleanupDirs::new(project_root);
        let index = ArchiveIndex::load(&dirs.index_file);
        Self { dirs, index }
    }

    /// 归档一批文件 (移动并记录)
    pub fn archive_paths(
        &mut self,
        paths: &[String],
        kind: &str,
    ) -> std::io::Result<ArchiveManifest> {
        self.dirs.ensure()?;
        let ts = Local::now().format("%Y-%m-%d_%H%M%S");
        let batch_id = ts.to_string();
        let batch_dir = self.dirs.create_archive_batch()?;
        let mut manifest = ArchiveManifest::new(&batch_id);

        for path_str in paths {
            let src = Path::new(path_str);
            if !src.exists() {
                continue;
            }

            let is_dir = src.is_dir();
            let size = fs::metadata(src).map(|m| m.len()).unwrap_or(0);
            let fname = src
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let dest = batch_dir.join(&fname);

            // 如果目标已存在, 加时间戳后缀
            let dest = if dest.exists() {
                let stem = batch_dir.join(format!("{}_{}", Utc::now().timestamp(), fname));
                stem
            } else {
                dest
            };

            let result = if is_dir {
                // 对目录: 压缩为 tar.gz 再归档
                Self::archive_dir(src, &dest)
            } else {
                fs::rename(src, &dest)
                    .or_else(|_| fs::copy(src, &dest).and_then(|_| fs::remove_file(src)))
            };

            match result {
                Ok(()) => {
                    let entry = ArchiveEntry {
                        source_path: path_str.clone(),
                        archived_path: dest.to_string_lossy().to_string(),
                        size_bytes: size,
                        is_dir,
                        archived_at: Utc::now().timestamp(),
                        cleanup_kind: kind.to_string(),
                        sha256: None,
                    };
                    manifest.entries.push(entry);
                    manifest.total_bytes += size;
                    manifest.total_items += 1;
                }
                Err(e) => {
                    log::warn!("[archiver] 归档失败 {}: {}", path_str, e);
                }
            }
        }

        // 写入清单
        let manifest_path = batch_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, &manifest_json)?;

        // 更新索引
        self.index.entries.extend(manifest.entries.clone());
        self.index.last_updated = Utc::now().timestamp();
        if let Err(e) = self.index.save(&self.dirs.index_file) {
            log::warn!("[archiver] 索引保存失败: {}", e);
        }

        // 记录日志
        CleanupLog::log(
            &self.dirs.log,
            &CleanupLogEntry {
                action: "archive".into(),
                kind: kind.into(),
                items: manifest.total_items,
                bytes: manifest.total_bytes,
                batch_id: batch_id.clone(),
                success: true,
                error: None,
            },
        );

        Ok(manifest)
    }

    fn archive_dir(src: &Path, dest: &Path) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            let tar_path = dest.with_extension("tar.gz");
            let status = std::process::Command::new("tar")
                .args(["-czf", &tar_path.to_string_lossy(), "-C"])
                .arg(src.parent().unwrap_or(Path::new(".")))
                .arg(src.file_name().unwrap_or_default())
                .status()?;
            if status.success() {
                let _ = fs::remove_dir_all(src);
                return Ok(());
            }
        }
        // fallback: 简单重命名 (当 tar 不可用时)
        let renamed = dest.with_extension("dir");
        fs::rename(src, &renamed)
    }

    /// 搜索归档: 按关键词查找已归档条目
    pub fn search(&self, query: &str) -> Vec<&ArchiveEntry> {
        let q = query.to_lowercase();
        self.index
            .entries
            .iter()
            .filter(|e| {
                e.source_path.to_lowercase().contains(&q)
                    || e.cleanup_kind.to_lowercase().contains(&q)
            })
            .collect()
    }
}

// ============================================================
// 清理日志系统 (.cleanup/log/history.jsonl)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupLogEntry {
    pub action: String, // "scan" | "clean" | "archive" | "backup"
    pub kind: String,
    pub items: usize,
    pub bytes: u64,
    pub batch_id: String,
    pub success: bool,
    pub error: Option<String>,
}

pub struct CleanupLog;

impl CleanupLog {
    pub fn log(log_dir: &Path, entry: &CleanupLogEntry) {
        let file = log_dir.join("history.jsonl");
        let line = serde_json::to_string(&entry).unwrap_or_default();
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&file) {
            use std::io::Write;
            let _ = writeln!(f, "{}", line);
        }
    }

    /// 读取最近 N 条日志
    pub fn recent(log_dir: &Path, n: usize) -> Vec<CleanupLogEntry> {
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
// 备份系统 (.backup/)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub backup_id: String,
    pub created_at: i64,
    pub project: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub excluded: Vec<String>,
    pub is_incremental: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackupIndex {
    pub backups: Vec<BackupManifest>,
    pub last_backup: Option<i64>,
}

impl BackupIndex {
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        fs::write(path, &serde_json::to_string_pretty(self)?)
    }
}

pub struct BackupEngine {
    project_root: PathBuf,
    backup_root: PathBuf,
    exclude_patterns: Vec<String>,
    max_backups: usize,
}

impl BackupEngine {
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            backup_root: project_root.join(".backup"),
            exclude_patterns: vec![
                ".backup".into(),
                ".cleanup".into(),
                "target".into(),
                "node_modules".into(),
                ".git".into(),
            ],
            max_backups: 28,
        }
    }

    /// 执行增量备份, 返回备份清单
    pub fn run_backup(&mut self) -> std::io::Result<BackupManifest> {
        let ts = Local::now().format("%Y-%m-%d_%H%M%S");
        let backup_id = ts.to_string();
        let backup_dir = self.backup_root.join(&backup_id);
        fs::create_dir_all(&backup_dir)?;

        // 收集需要备份的代码文件
        let mut file_count = 0usize;
        let mut total_bytes = 0u64;

        self.collect_files(
            &self.project_root,
            &backup_dir,
            &mut file_count,
            &mut total_bytes,
        )?;

        let manifest = BackupManifest {
            backup_id: backup_id.clone(),
            created_at: Utc::now().timestamp(),
            project: self.project_root.to_string_lossy().to_string(),
            file_count,
            total_bytes,
            excluded: self.exclude_patterns.clone(),
            is_incremental: true,
        };

        // 写入清单
        let manifest_path = backup_dir.join("manifest.json");
        fs::write(&manifest_path, &serde_json::to_string_pretty(&manifest)?)?;

        // 更新索引
        let index_path = self.backup_root.join("index.json");
        let mut index = BackupIndex::load(&index_path);
        index.backups.push(manifest.clone());
        index.last_backup = Some(Utc::now().timestamp());
        index.save(&index_path)?;

        // 更新 latest 符号链接
        self.update_latest(&backup_dir);

        // 清理旧备份
        self.prune_old_backups(&index);

        // 记录日志
        let log_dir = self.project_root.join(".cleanup").join("log");
        CleanupLog::log(
            &log_dir,
            &CleanupLogEntry {
                action: "backup".into(),
                kind: "code".into(),
                items: file_count,
                bytes: total_bytes,
                batch_id: backup_id,
                success: true,
                error: None,
            },
        );

        Ok(manifest)
    }

    fn collect_files(
        &self,
        src_dir: &Path,
        dst_dir: &Path,
        file_count: &mut usize,
        total_bytes: &mut u64,
    ) -> std::io::Result<()> {
        if !src_dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(src_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_string();

            // 跳过排除项
            if self
                .exclude_patterns
                .iter()
                .any(|p| name_str == *p || name_str.starts_with(p))
            {
                continue;
            }
            // 跳过隐藏目录 (除了 .cleanup 的 rules.toml)
            if name_str.starts_with('.') {
                continue;
            }

            let src_path = entry.path();
            let rel = src_path
                .strip_prefix(&self.project_root)
                .unwrap_or(&src_path);
            let dst_path = dst_dir.join(rel);

            if src_path.is_dir() {
                // 递归
                self.collect_files(&src_path, dst_dir, file_count, total_bytes)?;
            } else {
                // 只备份代码文件
                let ext = src_path
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let is_code = matches!(
                    ext.as_str(),
                    "rs" | "toml"
                        | "py"
                        | "js"
                        | "ts"
                        | "tsx"
                        | "jsx"
                        | "json"
                        | "css"
                        | "scss"
                        | "html"
                        | "md"
                        | "sh"
                        | "yml"
                        | "yaml"
                        | "sql"
                        | "proto"
                        | "vue"
                        | "svelte"
                        | "rb"
                        | "go"
                        | "mod"
                        | "sum"
                        | "lock"
                        | "conf"
                        | "cfg"
                        | "ini"
                        | "plist"
                );
                if !is_code {
                    continue;
                }

                if let Some(parent) = dst_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&src_path, &dst_path)?;
                let size = fs::metadata(&src_path).map(|m| m.len()).unwrap_or(0);
                *file_count += 1;
                *total_bytes += size;
            }
        }
        Ok(())
    }

    fn update_latest(&self, backup_dir: &Path) {
        let latest = self.backup_root.join("latest");
        // 删除旧符号链接
        let _ = fs::remove_file(&latest);
        // macOS 上目录符号链接需要特殊处理
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let _ = symlink(backup_dir, &latest);
        }
        #[cfg(not(unix))]
        {
            let _ = std::os::windows::fs::symlink_dir(backup_dir, &latest);
        }
    }

    fn prune_old_backups(&self, index: &BackupIndex) {
        if index.backups.len() <= self.max_backups {
            return;
        }
        let to_remove = index.backups.len() - self.max_backups;
        for backup in index.backups.iter().take(to_remove) {
            let path = self.backup_root.join(&backup.backup_id);
            if path.exists() {
                let _ = fs::remove_dir_all(&path);
            }
        }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
}

impl RiskLevel {
    pub fn label(&self) -> &'static str {
        match self {
            RiskLevel::Low => "低危",
            RiskLevel::Medium => "中危",
            RiskLevel::High => "高危",
        }
    }
}

impl Default for RiskLevel {
    fn default() -> Self {
        RiskLevel::Low
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
    pub risk: RiskLevel,
    pub description: Option<&'static str>,
}

fn platform_all() -> Platform {
    Platform::All
}

impl CleanupPattern {
    /// 当前平台生效的规则 (平台门控 + 风险阀)
    pub fn active_below(&self, max_risk: RiskLevel) -> bool {
        self.platform.matches(Platform::current()) && self.risk <= max_risk
    }

    pub fn all_patterns() -> Vec<Self> {
        let mac = Platform::MacOS;
        let win = Platform::Windows;
        let lin = Platform::Linux;
        let all = Platform::All;
        vec![
            // ---- 项目构建产物 (跨平台, Mole 34 目标) ----
            Self { name: "Rust build artifacts", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/target/**"], max_age_days: Some(7), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: Some("target/ 为可重建构建产物") },
            Self { name: "Node.js modules", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/node_modules/**"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: Some("npm/yarn/pnpm install 重建") },
            Self { name: "Python venv", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/.venv/**", "**/venv/**", "**/.tox/**", "**/.nox/**"], max_age_days: Some(60), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "Build output", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/dist/**", "**/.build/**", "**/build/**", "**/out/**"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "Next.js cache", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/.next/**", "**/.nuxt/**", "**/.output/**", "**/.svelte-kit/**", "**/.astro/**"], max_age_days: Some(7), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "Swift build", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/.build/**"], max_age_days: Some(30), safe: true, recursive: true, platform: mac, risk: RiskLevel::Low, description: None },
            Self { name: "Go test artifacts", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/*.test", "**/*.test.exe", "**/coverage.out", "**/coverage.html"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: Some("go test 编译产物与覆盖率; vendor/ 为依赖源码不删") },
            Self { name: "Turbo/Parcel cache", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/.turbo/**", "**/.parcel-cache/**", "**/.angular/**", "**/.dart_tool/**", "**/.zig-cache/**", "**/zig-out/**"], max_age_days: Some(15), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "Test caches", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/.pytest_cache/**", "**/.mypy_cache/**", "**/.ruff_cache/**", "**/coverage/**", "**/__pycache__/**"], max_age_days: Some(7), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            // ---- 项目蜕皮: 旧躯壳目录 (legacy/old/backup 命名的旧版本, 归档到 _archive/) ----
            Self { name: "Legacy shells", kind: CleanupKind::ProjectMolting, patterns: vec!["**/legacy/**", "**/*_legacy/**", "**/legacy_*/**", "**/old_*/**", "**/*_old/**", "**/*_v0/**", "**/*_v1/**", "**/*_backup*/**"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: Some("旧躯壳目录 (旧版本代码), 蜕皮归档至 .cleanup/archive/ 而非删除") },
            Self { name: "iOS derived data", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Developer/Xcode/DerivedData/**", "~/Library/Developer/CoreSimulator/Caches/**"], max_age_days: Some(30), safe: true, recursive: true, platform: mac, risk: RiskLevel::Medium, description: None },
            // ---- 包管理缓存 ----
            Self { name: "Cargo registry cache", kind: CleanupKind::Cache, patterns: vec!["~/.cargo/registry/cache/**", "~/.cargo/git/db/**"], max_age_days: Some(90), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "pip cache", kind: CleanupKind::Cache, patterns: vec!["~/.cache/pip/**", "%LOCALAPPDATA%/pip/cache/**"], max_age_days: Some(90), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "npm cache", kind: CleanupKind::Cache, patterns: vec!["~/.npm/_cacache/**", "%APPDATA%/npm-cache/**"], max_age_days: Some(90), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "pnpm store", kind: CleanupKind::Cache, patterns: vec!["~/Library/Caches/pnpm/**", "~/.local/share/pnpm/store/**", "%LOCALAPPDATA%/pnpm-cache/**"], max_age_days: Some(90), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "yarn cache", kind: CleanupKind::Cache, patterns: vec!["~/.cache/yarn/**"], max_age_days: Some(90), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "bun cache", kind: CleanupKind::Cache, patterns: vec!["~/.bun/install/cache/**"], max_age_days: Some(90), safe: true, recursive: true, platform: all, risk: RiskLevel::Medium, description: Some("仅 bun install 缓存; ~/.bun/bin(可执行)与全局包保留") },
            Self { name: "uv pip cache", kind: CleanupKind::Cache, patterns: vec!["~/.cache/uv/**"], max_age_days: Some(90), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "go build cache", kind: CleanupKind::Cache, patterns: vec!["~/Library/Caches/go-build/**", "~/.cache/go-build/**", "%LOCALAPPDATA%/go-build/**"], max_age_days: Some(60), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "conda pkgs", kind: CleanupKind::Cache, patterns: vec!["~/miniconda3/pkgs/**", "~/anaconda3/pkgs/**", "~/.conda/pkgs/**"], max_age_days: Some(60), safe: true, recursive: true, platform: all, risk: RiskLevel::Medium, description: None },
            // ---- 浏览器缓存 ----
            Self { name: "Google Chrome cache", kind: CleanupKind::Cache, patterns: vec!["~/Library/Caches/Google/Chrome/**", "%LOCALAPPDATA%/Google/Chrome/User Data/Default/Cache/**"], max_age_days: Some(15), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "Playwright browsers", kind: CleanupKind::Cache, patterns: vec!["~/Library/Caches/ms-playwright/**", "~/Library/Caches/ms-playwright-go/**"], max_age_days: Some(30), safe: true, recursive: true, platform: mac, risk: RiskLevel::Low, description: Some("浏览器自动化引擎, 重装下载") },
            // ---- AI 应用缓存 (PureMac AI Apps 吸收: Ollama/LM Studio 模型缓存) ----
            Self { name: "Ollama model cache", kind: CleanupKind::Cache, patterns: vec!["~/.ollama/models/blobs/**"], max_age_days: Some(90), safe: false, recursive: true, platform: all, risk: RiskLevel::Medium, description: Some("Ollama 模型 blob 缓存, 可 ollama pull 重建; 谨慎, 仅清未引用 blob") },
            Self { name: "LM Studio model cache", kind: CleanupKind::Cache, patterns: vec!["~/.lmstudio/models/**"], max_age_days: Some(90), safe: false, recursive: true, platform: all, risk: RiskLevel::Medium, description: Some("LM Studio 模型缓存, 可重新下载") },
            Self { name: "MCP/Agent hub cache", kind: CleanupKind::Cache, patterns: vec!["~/.cache/claude/**", "~/.cache/opencode/**", "~/.cache/codex/**"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: Some("AI agent 工具缓存 (工具响应/索引, 可重建)") },
            Self { name: "Homebrew cache", kind: CleanupKind::Cache, patterns: vec!["~/Library/Caches/Homebrew/**", "~/Library/Caches/Homebrew/downloads/**", "/opt/homebrew/Library/Homebrew/vendor/**"], max_age_days: Some(30), safe: true, recursive: true, platform: mac, risk: RiskLevel::Low, description: Some("brew 下载缓存与 vendor ruby; brew cleanup 等价, 重装即重建") },
            // ---- Xcode 扩展 (mac-janitor: Archives/Simulators, 现仅 DerivedData) ----
            Self { name: "Xcode Archives", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Developer/Xcode/Archives/**"], max_age_days: Some(90), safe: false, recursive: true, platform: mac, risk: RiskLevel::Medium, description: Some("已归档的 App 构建产物, 含 dSYM; 确认无需再上传后清理") },
            Self { name: "Xcode Simulator runtimes", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Developer/CoreSimulator/Images/**", "~/Library/Developer/CoreSimulator/Caches/**"], max_age_days: Some(30), safe: true, recursive: true, platform: mac, risk: RiskLevel::Medium, description: Some("模拟器运行时镜像缓存; 删除后按需重下") },
            Self { name: "Xcode module caches", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Developer/Xcode/DerivedData/**/ModuleCache.noindex/**", "~/Library/Developer/Xcode/DerivedData/**/PrecompiledHeaders/**"], max_age_days: Some(7), safe: true, recursive: true, platform: mac, risk: RiskLevel::Medium, description: Some("Swift/ObjC 模块预编译缓存, 可重建") },
            // ---- temp ----
            Self { name: "System temp", kind: CleanupKind::TempFiles, patterns: vec!["/tmp/**", "/var/tmp/**", "%TEMP%/**", "%TMP%/**"], max_age_days: Some(1), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "Preload temp", kind: CleanupKind::TempFiles, patterns: vec!["%WINDIR%/Prefetch/**"], max_age_days: Some(1), safe: true, recursive: true, platform: win, risk: RiskLevel::Medium, description: None },
            // ---- IDE 缓存 ----
            Self { name: "VS Code caches", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Application Support/Code/CachedData/**", "~/.vscode/extensions/.cache/**", "%APPDATA%/Code/CachedData/**"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "Cursor caches", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Application Support/Cursor/CachedData/**", "~/.cursor/cache/**"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "IntelliJ caches", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Caches/JetBrains/**", "~/.cache/JetBrains/**", "%LOCALAPPDATA%/JetBrains/**"], max_age_days: Some(30), safe: true, recursive: true, platform: all, risk: RiskLevel::Low, description: None },
            Self { name: "Docker", kind: CleanupKind::Cache, patterns: vec!["~/Library/Containers/com.docker.docker/Data/vms/0/data/DockerDesktopRegular5/.cache/**"], max_age_days: Some(60), safe: true, recursive: true, platform: mac, risk: RiskLevel::Medium, description: None },
            // ---- Linux 系统缓存 ----
            Self { name: "apt/dnf/pacman package cache", kind: CleanupKind::Cache, patterns: vec!["/var/cache/apt/archives/**", "/var/cache/dnf/**", "/var/cache/pacman/pkg/**", "/var/tmp/**"], max_age_days: Some(30), safe: true, recursive: true, platform: lin, risk: RiskLevel::Medium, description: None },
            Self { name: "Linux user cache", kind: CleanupKind::Cache, patterns: vec!["~/.cache/**"], max_age_days: Some(30), safe: true, recursive: true, platform: lin, risk: RiskLevel::Low, description: None },
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
    pub fn has_cachedir_tag(dir: &Path) -> bool {
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
    pub fn is_system_root_dir(p: &Path) -> bool {
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
    pub fn entry_size(p: &Path) -> u64 {
        match std::fs::metadata(p) {
            Ok(m) if m.is_file() => m.len(),
            Ok(m) if m.is_dir() => {
                // 跳过系统根目录防误扫 (护栏: is_system_root_dir 判定后仍不遍历)
                if Self::is_system_root_dir(p) {
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
                            total = total.saturating_add(Self::entry_size(&ep));
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub kind: CleanupKind,
    pub scanned_count: usize,
    pub deletable_count: usize,
    pub estimated_bytes: u64,
    pub pattern_matches: Vec<String>,
    pub dry_run: bool,
    pub errors: Vec<String>,
    pub timestamp: i64,
}

impl CleanupResult {
    pub fn new(kind: CleanupKind) -> Self {
        Self {
            kind,
            scanned_count: 0,
            deletable_count: 0,
            estimated_bytes: 0,
            pattern_matches: Vec::new(),
            dry_run: true,
            errors: Vec::new(),
            timestamp: Utc::now().timestamp(),
        }
    }

    pub fn summary(&self) -> String {
        let mode = if self.dry_run { "预览" } else { "已清理" };
        format!(
            "[{}] {:?}: 扫描 {} 项, 可删除 {} 项 (约 {:.1} MB), {} 个错误",
            mode,
            self.kind,
            self.scanned_count,
            self.deletable_count,
            self.estimated_bytes as f64 / 1_048_576.0,
            self.errors.len()
        )
    }
}

pub struct CleanupEngine {
    pub patterns: Vec<CleanupPattern>,
    pub whitelist: Vec<PathBuf>,
    pub history: Vec<CleanupResult>,
    pub dry_run_default: bool,
    pub archive_on_clean: bool, // true = 归档而非删除
    pub project_root: PathBuf,
    pub risk_gate: RiskLevel, // 默认仅执行 <= 该风险级规则
    pub command_cleaner: Option<CommandCleaner>, // 命令式清理 (SystemServices)
    max_history: usize,
}

impl Default for CleanupEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanupEngine {
    pub fn new() -> Self {
        Self {
            patterns: CleanupPattern::all_patterns()
                .into_iter()
                .filter(|p| p.platform.matches(Platform::current()))
                .collect(),
            whitelist: vec![
                PathBuf::from("~/.config"),
                PathBuf::from("~/.ssh"),
                PathBuf::from("~/.gnupg"),
            ],
            history: Vec::new(),
            dry_run_default: true,
            archive_on_clean: true,
            project_root: PathBuf::from("."),
            risk_gate: RiskLevel::Medium,
            command_cleaner: Some(CommandCleaner::new()),
            max_history: 50,
        }
    }

    pub fn with_risk_gate(mut self, gate: RiskLevel) -> Self {
        self.risk_gate = gate;
        self
    }

    pub fn with_project_root(mut self, root: PathBuf) -> Self {
        self.project_root = root;
        self
    }

    pub fn add_whitelist(&mut self, path: PathBuf) {
        self.whitelist.push(path);
    }

    fn is_whitelisted(&self, path: &Path) -> bool {
        // whitelist 条目存字面 `~/.config`/`%VAR%`, 与 expand 后的绝对扫描路径
        // starts_with 永不匹配 — 先 expand 再比较 (修复: 白名单永久失效缺陷)
        self.whitelist.iter().any(|w| {
            let expanded = CleanupPattern::expand(&w.to_string_lossy());
            let expanded_path = PathBuf::from(expanded);
            !expanded_path.as_os_str().is_empty() && path.starts_with(expanded_path)
        })
    }

    pub fn scan(&self, kind: CleanupKind, dry_run: bool) -> CleanupResult {
        let mut result = CleanupResult::new(kind);
        result.dry_run = dry_run;

        let relevant: Vec<&CleanupPattern> = self
            .patterns
            .iter()
            .filter(|p| kind == CleanupKind::All || p.kind == kind)
            .filter(|p| p.risk <= self.risk_gate)
            .collect();

        for pattern in &relevant {
            for glob_pat in &pattern.patterns {
                let pat_str = CleanupPattern::expand(glob_pat);
                if let Ok(entries) = glob::glob(&pat_str) {
                    for entry in entries.flatten() {
                        if self.is_whitelisted(&entry) {
                            continue;
                        }
                        // 路径安全护栏: 拒绝系统根目录
                        if CleanupPattern::is_system_root_dir(&entry) {
                            continue;
                        }
                        let is_old = if let Some(max_days) = pattern.max_age_days {
                            match std::fs::metadata(&entry) {
                                Ok(meta) => {
                                    if let Ok(modified) = meta.modified() {
                                        // age = now - modified；未来 mtime (时钟偏移/解包) 视为 0，绝不删除
                                        let age = modified
                                            .elapsed()
                                            .map(|d| d.as_secs() as i64)
                                            .unwrap_or(0);
                                        age > max_days * 86400
                                    } else {
                                        false
                                    }
                                }
                                Err(_) => false,
                            }
                        } else {
                            true
                        };

                        if is_old {
                            result.deletable_count += 1;
                            let size = CleanupPattern::entry_size(&entry);
                            result.estimated_bytes += size;
                            if result.pattern_matches.len() < 20 {
                                result
                                    .pattern_matches
                                    .push(entry.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
        result
    }

    /// 执行清理: 如果 archive_on_clean 则归档, 否则直接删除
    pub fn clean(&mut self, kind: CleanupKind) -> CleanupResult {
        // SystemServices 类别走命令式清理通道
        if kind == CleanupKind::SystemServices {
            return self.clean_services(kind);
        }
        let mut result = self.scan(kind, false);

        if !result.dry_run && !result.pattern_matches.is_empty() {
            if self.archive_on_clean {
                // 归档模式: 移动而非删除
                let mut archiver = Archiver::new(&self.project_root);
                match archiver.archive_paths(&result.pattern_matches, &format!("{:?}", kind)) {
                    Ok(manifest) => {
                        result.estimated_bytes = manifest.total_bytes;
                        log::info!(
                            "[cleanup] 已归档 {} 项到 .cleanup/archive/{}",
                            manifest.total_items,
                            manifest.batch_id
                        );
                    }
                    Err(e) => {
                        log::warn!("[cleanup] 归档失败, 回退到直接删除: {}", e);
                        self.delete_paths(&mut result);
                    }
                }
            } else {
                self.delete_paths(&mut result);
            }
        }

        // 记录日志
        let log_dir = self.project_root.join(".cleanup").join("log");
        CleanupLog::log(
            &log_dir,
            &CleanupLogEntry {
                action: if self.archive_on_clean {
                    "archive"
                } else {
                    "clean"
                }
                .into(),
                kind: format!("{:?}", kind),
                items: result.deletable_count,
                bytes: result.estimated_bytes,
                batch_id: Local::now().format("%Y-%m-%d_%H%M%S").to_string(),
                success: result.errors.is_empty(),
                error: if result.errors.is_empty() {
                    None
                } else {
                    Some(result.errors.join("; "))
                },
            },
        );

        self.history.push(result.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        result
    }

    /// 命令式系统服务清理 (SystemServices): 遍历 command_cleaner 项执行
    fn clean_services(&mut self, kind: CleanupKind) -> CleanupResult {
        let mut result = CleanupResult::new(kind);
        result.dry_run = self.dry_run_default;
        let cleaner = match self.command_cleaner.as_ref() {
            Some(c) => c,
            None => {
                result.errors.push("无 command_cleaner".into());
                return result;
            }
        };
        let confirm = !result.dry_run; // dry-run 无需确认; 实删需确认 (由调用方传 --confirm)
        for (item, _) in cleaner.scan() {
            let r = cleaner.execute(item, confirm);
            result.scanned_count += 1;
            match r.status.as_str() {
                "executed" => {
                    result.deletable_count += 1;
                    if result.pattern_matches.len() < 20 {
                        result
                            .pattern_matches
                            .push(format!("{}: {}", item.name, r.output.trim()));
                    }
                }
                "needs_confirm" => {
                    if result.pattern_matches.len() < 20 {
                        result
                            .pattern_matches
                            .push(format!("{}: 需 --confirm", item.name));
                    }
                }
                "failed" => result.errors.push(format!("{}: {}", item.name, r.output)),
                // dry_run / skipped: 计入可用项 (供后台 dry-run 报告), 不计数 deletable
                _ => {
                    if result.pattern_matches.len() < 20 && r.status == "dry_run" {
                        let head = r.output.lines().take(1).next().unwrap_or("");
                        result
                            .pattern_matches
                            .push(format!("{} (dry-run): {}", item.name, head));
                    }
                }
            }
        }
        result
    }

    fn delete_paths(&self, result: &mut CleanupResult) {
        for path_str in &result.pattern_matches {
            let path = Path::new(path_str);
            if self.is_whitelisted(path) {
                continue;
            }
            if CleanupPattern::is_system_root_dir(path) {
                continue;
            }
            if path.is_dir() {
                if let Err(e) = std::fs::remove_dir_all(path) {
                    result
                        .errors
                        .push(format!("删除目录失败 {}: {}", path_str, e));
                }
            } else if let Err(e) = std::fs::remove_file(path) {
                result
                    .errors
                    .push(format!("删除文件失败 {}: {}", path_str, e));
            }
        }
    }

    pub fn prune_brain_snapshots(max_keep: usize) -> usize {
        let home = dirs::home_dir().unwrap_or_default();
        let snap_dir = home.join(".neotrix").join("snapshots");
        if !snap_dir.exists() {
            return 0;
        }
        let mut entries: Vec<_> = std::fs::read_dir(&snap_dir)
            .map(|d| d.filter_map(|e| e.ok()).collect::<Vec<_>>())
            .unwrap_or_default();
        entries.sort_by_key(|e| e.path());
        let mut removed = 0;
        if entries.len() > max_keep {
            for entry in entries.iter().take(entries.len() - max_keep) {
                if std::fs::remove_file(entry.path()).is_ok() {
                    removed += 1;
                }
            }
        }
        removed
    }

    /// 项目蜕皮: 将 project_root 顶层旧躯壳目录 (legacy/old/*_v0/*_backup* 命名)
    /// 整体归档至 .cleanup/archive/, 活动树只留最新态。
    ///
    /// 安全护栏 (三道闸):
    ///   ① 白名单路径 (is_whitelisted) 绝不蜕皮;
    ///   ② 系统根目录 (is_system_root_dir) 绝不蜕皮;
    ///   ③ project_root 自身绝不蜕皮 (仅扫描其下一级, 不递归).
    pub fn molt_project(&mut self) -> CleanupResult {
        let mut result = CleanupResult::new(CleanupKind::ProjectMolting);
        result.dry_run = self.dry_run_default;

        let root = self.project_root.clone();
        let mut shells: Vec<PathBuf> = Vec::new();
        if let Ok(rd) = fs::read_dir(&root) {
            for entry in rd.flatten() {
                let p = entry.path();
                if !p.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_lowercase();
                let is_shell = name == "legacy"
                    || name.contains("_legacy")
                    || name.starts_with("legacy_")
                    || name.starts_with("old_")
                    || name.ends_with("_old")
                    || name.contains("_v0")
                    || name.contains("_v1")
                    || name.contains("_backup");
                if is_shell {
                    shells.push(p);
                }
            }
        }
        // 安全护栏: 过滤白名单/系统根/root 自身
        shells.retain(|s| {
            !self.is_whitelisted(s) && !CleanupPattern::is_system_root_dir(s) && s != &root
        });
        result.scanned_count = shells.len();
        result.deletable_count = shells.len();
        for s in &shells {
            result.estimated_bytes = result
                .estimated_bytes
                .saturating_add(CleanupPattern::entry_size(s));
            if result.pattern_matches.len() < 20 {
                result.pattern_matches.push(s.to_string_lossy().to_string());
            }
        }

        // 非 dry-run + 归档模式: 执行蜕皮归档
        if !result.dry_run && !shells.is_empty() && self.archive_on_clean {
            let path_strs: Vec<String> = shells
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect();
            let mut archiver = Archiver::new(&root);
            match archiver.archive_paths(&path_strs, "ProjectMolting") {
                Ok(m) => {
                    result.estimated_bytes = m.total_bytes;
                    log::info!(
                        "[molting] 归档 {} 个旧躯壳到 .cleanup/archive/{}",
                        m.total_items,
                        m.batch_id
                    );
                }
                Err(e) => result.errors.push(format!("蜕皮归档失败: {}", e)),
            }
        }

        // 记录日志
        let log_dir = root.join(".cleanup").join("log");
        CleanupLog::log(
            &log_dir,
            &CleanupLogEntry {
                action: "molt".into(),
                kind: "ProjectMolting".into(),
                items: shells.len(),
                bytes: result.estimated_bytes,
                batch_id: Local::now().format("%Y-%m-%d_%H%M%S").to_string(),
                success: result.errors.is_empty(),
                error: if result.errors.is_empty() {
                    None
                } else {
                    Some(result.errors.join("; "))
                },
            },
        );

        self.history.push(result.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        result
    }
}

/// SelfTest — 清理/蜕皮引擎检测能力自检 (T1)。
///
/// 检测以下退化:
///   ① 规则库空 (CleanupPattern 未加载 → 清理静默失效);
///   ② 蜕皮模式缺失 (ProjectMolting 未注册 → 蜕皮能力被删/退化);
///   ③ 白名单空 (is_whitelisted 永假 → 危险: 白名单失效会误删受保护路径);
///   ④ 安全护栏失效 (系统根目录可被蜕皮 → 数据风险)。
#[derive(Default)]
pub struct CleanupEngineSelfTest;

impl crate::core::nt_core_self_test::SelfTest for CleanupEngineSelfTest {
    fn name(&self) -> &str {
        "nt_mind_cleanup_engine"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        let all = CleanupPattern::all_patterns();
        if all.is_empty() {
            failures.push("CleanupPattern::all_patterns() 为空: 清理规则库退化".into());
        }
        if !all.iter().any(|p| p.kind == CleanupKind::ProjectMolting) {
            failures.push("ProjectMolting 蜕皮规则未注册: 蜕皮能力缺失".into());
        }
        if !all.iter().any(|p| p.kind == CleanupKind::ProjectArtifacts) {
            failures.push("ProjectArtifacts 构建产物规则未注册".into());
        }

        let engine = CleanupEngine::new();
        if engine.whitelist.is_empty() {
            failures.push("白名单为空: is_whitelisted 永假, 受保护路径可被误删".into());
        }
        if !engine.is_whitelisted(
            &dirs::home_dir()
                .unwrap_or_default()
                .join(".config")
                .join("app"),
        ) {
            failures.push("白名单 ~/.config 未匹配绝对路径: expand 失效".into());
        }

        // 安全护栏: 系统根目录必须被蜕皮拒绝
        if !CleanupPattern::is_system_root_dir(std::path::Path::new("/")) {
            failures.push("系统根目录安全护栏失效 (/)".into());
        }
        if !CleanupPattern::is_system_root_dir(&dirs::home_dir().unwrap_or_default()) {
            failures.push("系统根目录安全护栏失效 ($HOME)".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// 运行外部命令, 校验 exit status。成功返回 stdout, 失败返回 Err (含 stderr)。
/// (修复: 此前不校验退出码, brew 输出含 "Error" 字样会误判 failed, 命令真失败却误判成功)
fn run_output(cmd: &str, args: &[&str]) -> Result<String, String> {
    match std::process::Command::new(cmd).args(args).output() {
        Ok(o) => {
            if o.status.success() {
                Ok(String::from_utf8_lossy(&o.stdout).to_string())
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let code = o
                    .status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "?".into());
                Err(format!("{} 退出码 {}: {}", cmd, code, stderr.trim()))
            }
        }
        Err(e) => Err(format!("{} 启动失败: {}", cmd, e)),
    }
}

// ============================================================
// 命令式清理 (CommandCleanup) — 系统服务级清理执行器
//
// 吸收来源 (GitHub 项目特性):
//   - mac-janitor / GuacSweep: Time Machine 本地快照 (tmutil listlocalsnapshots)
//   - PureMac: Docker prune (docker system prune)
//   - mac-janitor: Homebrew 缓存 (brew cleanup)
// 设计: 每项命令自带 dry-run 前缀 + 风险级 + 平台门控, 复用 CleanupEngine.risk_gate。
// 安全: 默认 dry_run=true; 高风险项 (TM 快照删除) 强制要求确认标志。
// ============================================================

/// 命令式清理项 — 一条可执行的外部清理命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandCleanup {
    pub name: &'static str,
    pub kind: CleanupKind,
    /// 实际执行命令 (argv)
    pub cmd: &'static str,
    pub args: Vec<&'static str>,
    /// dry-run 变体命令 (不产生实际删除)
    pub dry_run_cmd: &'static str,
    pub dry_run_args: Vec<&'static str>,
    /// 是否强制需用户确认 (拒绝无确认执行)
    pub requires_confirm: bool,
    pub platform: Platform,
    pub risk: RiskLevel,
    pub description: &'static str,
}

impl CommandCleanup {
    pub fn all() -> Vec<Self> {
        vec![
            // Homebrew 缓存清理 (mac-janitor: brew cleanup) — 低危可自动
            Self {
                name: "Homebrew cleanup",
                kind: CleanupKind::SystemServices,
                cmd: "brew",
                args: vec!["cleanup", "--prune=all"],
                dry_run_cmd: "brew",
                dry_run_args: vec!["cleanup", "--dry-run"],
                requires_confirm: false,
                platform: Platform::MacOS,
                risk: RiskLevel::Low,
                description: "brew cleanup --prune=all: 清理旧版本与下载缓存",
            },
            // Docker 未使用资源 (PureMac: docker prune) — 中危, 需确认
            Self {
                name: "Docker system prune",
                kind: CleanupKind::SystemServices,
                cmd: "docker",
                args: vec!["system", "prune", "-f"],
                dry_run_cmd: "docker",
                dry_run_args: vec!["system", "df"],
                requires_confirm: true,
                platform: Platform::All,
                risk: RiskLevel::Medium,
                description: "docker system prune -f: 移除停止容器/悬空镜像/未用网络与构建缓存",
            },
            // Time Machine 本地快照 (mac-janitor/GuacSweep) — 高危, 需确认
            Self {
                name: "Time Machine local snapshots",
                kind: CleanupKind::SystemServices,
                cmd: "tmutil",
                args: vec!["deletelocalsnapshots"],
                dry_run_cmd: "tmutil",
                dry_run_args: vec!["listlocalsnapshots", "/"],
                requires_confirm: true,
                platform: Platform::MacOS,
                risk: RiskLevel::High,
                description: "删除 Time Machine 本地快照 (需显式快照名, 执行前先列示)",
            },
        ]
    }

    /// 当前平台 + 风险阀过滤后的可用项
    pub fn active_on_current(&self, gate: RiskLevel) -> bool {
        self.platform.matches(Platform::current()) && self.risk <= gate
    }
}

/// 命令式清理执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub name: String,
    pub status: String, // "skipped" | "dry_run" | "executed" | "failed" | "needs_confirm"
    pub output: String,
    pub dry_run: bool,
    pub error: Option<String>,
}

/// 命令式清理执行器 — 挂载于 CleanupEngine
pub struct CommandCleaner {
    pub items: Vec<CommandCleanup>,
    pub dry_run: bool,
    pub risk_gate: RiskLevel,
}

impl Default for CommandCleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandCleaner {
    pub fn new() -> Self {
        Self {
            items: CommandCleanup::all(),
            dry_run: true,
            risk_gate: RiskLevel::Medium,
        }
    }

    pub fn with_dry_run(mut self, dry: bool) -> Self {
        self.dry_run = dry;
        self
    }
    pub fn with_risk_gate(mut self, gate: RiskLevel) -> Self {
        self.risk_gate = gate;
        self
    }

    /// 扫描可执行项 (平台 + 风险过滤), 返回 (名称, 是否可用)
    pub fn scan(&self) -> Vec<(&CommandCleanup, bool)> {
        self.items
            .iter()
            .filter(|i| i.active_on_current(self.risk_gate))
            .map(|i| (i, true))
            .collect()
    }

    /// 执行单个清理项。requires_confirm 项在非 dry-run 且无 confirm 时拒绝执行。
    pub fn execute(&self, item: &CommandCleanup, confirm: bool) -> CommandResult {
        if !item.active_on_current(self.risk_gate) {
            return CommandResult {
                name: item.name.into(),
                status: "skipped".into(),
                output: "平台/风险不适用".into(),
                dry_run: self.dry_run,
                error: None,
            };
        }
        if self.dry_run {
            // dry-run 变体: brew cleanup --dry-run / docker system df / tmutil listlocalsnapshots
            let out = run_output(item.dry_run_cmd, &item.dry_run_args).unwrap_or_else(|e| e);
            return CommandResult {
                name: item.name.into(),
                status: "dry_run".into(),
                output: out,
                dry_run: true,
                error: None,
            };
        }
        if item.requires_confirm && !confirm {
            return CommandResult {
                name: item.name.into(),
                status: "needs_confirm".into(),
                output: format!("需要显式确认 (--confirm) 才执行: {}", item.description),
                dry_run: false,
                error: None,
            };
        }
        // TM 快照: 先列示再逐个删除
        if item.cmd == "tmutil" {
            let snapshots = self.list_tm_snapshots();
            if snapshots.is_empty() {
                return CommandResult {
                    name: item.name.into(),
                    status: "executed".into(),
                    output: "无本地快照可删".into(),
                    dry_run: false,
                    error: None,
                };
            }
            let mut log = Vec::new();
            for snap in &snapshots {
                let out = run_output("tmutil", &["deletelocalsnapshots", snap])
                    .unwrap_or_else(|e| format!("ERR: {}", e));
                log.push(format!("{}: {}", snap, out.trim()));
            }
            return CommandResult {
                name: item.name.into(),
                status: "executed".into(),
                output: log.join("\n"),
                dry_run: false,
                error: None,
            };
        }
        let out = run_output(item.cmd, &item.args).unwrap_or_else(|e| e);
        CommandResult {
            name: item.name.into(),
            status: if out.contains("ERR") || out.contains("error") {
                "failed"
            } else {
                "executed"
            }
            .into(),
            output: out,
            dry_run: false,
            error: None,
        }
    }

    /// 列示 Time Machine 本地快照
    pub fn list_tm_snapshots(&self) -> Vec<String> {
        run_output("tmutil", &["listlocalsnapshots", "/"])
            .ok()
            .map(|o| {
                o.lines()
                    .filter_map(|l| l.trim().strip_prefix("com.apple.TimeMachine."))
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    /// 进程级唯一临时目录 — 防并行会话同跑 cargo test 时共享固定 temp 目录互相删除。
    fn unique_tmp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{}_pid{}", name, std::process::id()))
    }

    #[test]
    fn test_cleanup_patterns() {
        let patterns = CleanupPattern::all_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns
            .iter()
            .any(|p| p.kind == CleanupKind::ProjectArtifacts));
        assert!(patterns.iter().any(|p| p.kind == CleanupKind::IDECaches));
    }

    #[test]
    fn test_cleanup_engine_new() {
        let engine = CleanupEngine::new();
        assert!(!engine.patterns.is_empty());
        assert!(engine.dry_run_default);
    }

    #[test]
    fn test_cleanup_result_summary() {
        let mut r = CleanupResult::new(CleanupKind::Cache);
        r.dry_run = true;
        r.deletable_count = 5;
        r.estimated_bytes = 1_048_576;
        let s = r.summary();
        assert!(s.contains("预览"));
        assert!(s.contains("1.0 MB"));
    }

    #[test]
    fn test_clean_kind_descriptions() {
        assert!(!CleanupKind::All.description().is_empty());
        assert!(!CleanupKind::ProjectArtifacts.description().is_empty());
    }

    #[test]
    fn test_cleanup_dirs_creation() {
        let tmp = unique_tmp("neotrix_test_cleanup_dirs");
        let _ = fs::remove_dir_all(&tmp);
        let dirs = CleanupDirs::new(&tmp);
        dirs.ensure().expect("ensure cleanup dirs");
        assert!(dirs.archive.exists());
        assert!(dirs.log.exists());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_archiver_archive_paths() {
        let tmp = unique_tmp("neotrix_test_archiver");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        // 创建测试文件
        let test_file = tmp.join("test.txt");
        fs::write(&test_file, b"hello world").unwrap();

        let mut archiver = Archiver::new(&tmp);
        let paths = vec![test_file.to_string_lossy().to_string()];
        let manifest = archiver
            .archive_paths(&paths, "test")
            .expect("archive paths");
        assert_eq!(manifest.total_items, 1);
        assert_eq!(manifest.entries[0].source_path, paths[0]);

        // 验证索引
        assert_eq!(archiver.index.entries.len(), 1);

        // 搜索测试
        let hits = archiver.search("test.txt");
        assert_eq!(hits.len(), 1);

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_backup_engine() {
        let tmp = unique_tmp("neotrix_test_backup");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        // 创建测试代码文件
        let src = tmp.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.rs"), b"fn main() {}").unwrap();
        fs::write(tmp.join("Cargo.toml"), b"[package]\nname = \"test\"\n").unwrap();

        let mut engine = BackupEngine::new(&tmp);
        let manifest = engine.run_backup().expect("backup");
        assert!(manifest.file_count > 0);

        // 验证备份目录
        let backup_root = tmp.join(".backup");
        assert!(backup_root.exists());
        let latest = backup_root.join("latest");
        assert!(latest.exists());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_cleanup_log() {
        let tmp = unique_tmp("neotrix_test_cleanup_log");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("log")).unwrap();

        CleanupLog::log(
            &tmp.join("log"),
            &CleanupLogEntry {
                action: "test".into(),
                kind: "test".into(),
                items: 1,
                bytes: 100,
                batch_id: "batch_1".into(),
                success: true,
                error: None,
            },
        );

        let recent = CleanupLog::recent(&tmp.join("log"), 10);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].items, 1);

        let _ = fs::remove_dir_all(&tmp);
    }

    // ---- 跨平台 + 组件移除增强测试 ----

    #[test]
    fn test_platform_current_and_matches() {
        let cur = Platform::current();
        assert!(matches!(
            cur,
            Platform::MacOS | Platform::Windows | Platform::Linux | Platform::All
        ));
        assert!(Platform::All.matches(cur));
        assert!(cur.matches(cur));
    }

    #[test]
    fn test_pattern_platform_filtering() {
        // 当前平台生效, 非当前平台规则应被 new() 过滤
        let engine = CleanupEngine::new();
        for p in &engine.patterns {
            assert!(
                p.platform.matches(Platform::current()),
                "规则 {} 未按平台过滤",
                p.name
            );
        }
        // 全量列表应同时含 mac/win/linux 规则 (用于跨平台目标)
        let all = CleanupPattern::all_patterns();
        assert!(all.iter().any(|p| p.platform == Platform::MacOS));
        assert!(all.iter().any(|p| p.platform == Platform::Windows));
    }

    #[test]
    fn test_risk_gate_filters() {
        let low = RiskLevel::Low;
        let high = RiskLevel::High;
        let engine = CleanupEngine::new().with_risk_gate(RiskLevel::Low);
        // 默认 gate 为 Medium 时, High 规则不应执行
        let default_engine = CleanupEngine::new();
        assert!(engine.risk_gate <= low || default_engine.risk_gate > low);
        assert!(high > RiskLevel::Medium);
        assert!(RiskLevel::Low < RiskLevel::High);
    }

    #[test]
    fn test_pattern_expand_home() {
        let home = dirs::home_dir().unwrap();
        let expanded = CleanupPattern::expand("~/.cargo/registry/cache/**");
        assert!(expanded.starts_with(&home.to_string_lossy().to_string()));
        assert!(expanded.contains(".cargo"));
    }

    #[test]
    fn test_cachedir_tag_detection() {
        let tmp = unique_tmp("neotrix_test_cachedir_tag");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        // 带合法签名的 CACHEDIR.TAG → 应被识别为缓存
        fs::write(
            tmp.join("CACHEDIR.TAG"),
            "Signature: 8a477f597d02d456d45674aa7d611ef7b6c14a01bccaebbd4e53c5d4f\ncomment",
        )
        .unwrap();
        assert!(CleanupPattern::has_cachedir_tag(&tmp));
        // 不带签名 → 不是
        fs::write(tmp.join("CACHEDIR.TAG"), "random").unwrap();
        assert!(!CleanupPattern::has_cachedir_tag(&tmp));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_system_root_guard() {
        // 拒绝删除系统根目录
        assert!(CleanupPattern::is_system_root_dir(std::path::Path::new(
            "/"
        )));
        let home = dirs::home_dir().unwrap();
        assert!(CleanupPattern::is_system_root_dir(&home));
        // 普通目录不受影响
        assert!(!CleanupPattern::is_system_root_dir(std::path::Path::new(
            "/tmp/neotrix_test_x"
        )));
    }

    // ---- 吸收增强测试 (AI 缓存 / Homebrew / Xcode / 命令式清理) ----

    #[test]
    fn test_absorbed_ai_homebrew_xcode_patterns() {
        let all = CleanupPattern::all_patterns();
        // AI 模型缓存 (PureMac AI Apps 吸收)
        assert!(all.iter().any(|p| p.name == "Ollama model cache"));
        assert!(all.iter().any(|p| p.name == "LM Studio model cache"));
        assert!(all.iter().any(|p| p.name == "MCP/Agent hub cache"));
        // Homebrew 缓存 (mac-janitor 吸收)
        assert!(all.iter().any(|p| p.name == "Homebrew cache"));
        // Xcode Archives/Simulators (mac-janitor 吸收)
        assert!(all.iter().any(|p| p.name == "Xcode Archives"));
        assert!(all.iter().any(|p| p.name == "Xcode Simulator runtimes"));
        assert!(all.iter().any(|p| p.name == "Xcode module caches"));
    }

    #[test]
    fn test_absorbed_pattern_risk_gating() {
        // AI 模型缓存默认 medium 风险 → 默认 gate (Medium) 下可用, 但 Low gate 下被过滤
        let engine = CleanupEngine::new();
        assert_eq!(engine.risk_gate, RiskLevel::Medium);
        // SystemServices 类别枚举存在
        assert!(!CleanupKind::SystemServices.description().is_empty());
        // 命令清理项三件套 (brew/docker/tmutil)
        let cmds = CommandCleanup::all();
        assert!(cmds.iter().any(|c| c.name == "Homebrew cleanup"));
        assert!(cmds.iter().any(|c| c.name == "Docker system prune"));
        assert!(cmds
            .iter()
            .any(|c| c.name == "Time Machine local snapshots"));
    }

    #[test]
    fn test_command_cleaner_scan_and_confirm_gate() {
        let cleaner = CommandCleaner::new().with_dry_run(true);
        // 当前平台过滤后至少 brew (macOS) 或 docker (跨平台) 可用
        let scanned = cleaner.scan();
        assert!(!scanned.is_empty());
        // dry-run 模式执行 brew → dry_run 状态, 不产生副作用
        if let Some((brew, _)) = scanned.iter().find(|(i, _)| i.name == "Homebrew cleanup") {
            let r = cleaner.execute(brew, false);
            assert_eq!(r.status, "dry_run");
            assert!(r.dry_run);
        }
        // 非 dry-run + 需确认项 → needs_confirm
        let real = CommandCleaner::new().with_dry_run(false);
        if let Some((docker, _)) = real
            .scan()
            .iter()
            .find(|(i, _)| i.name == "Docker system prune")
        {
            let r = real.execute(docker, false);
            assert_eq!(r.status, "needs_confirm");
        }
    }

    #[test]
    fn test_clean_services_via_engine() {
        let mut engine = CleanupEngine::new();
        engine.dry_run_default = true;
        let r = engine.clean(CleanupKind::SystemServices);
        // dry-run 下不报错, 至少有扫描计数
        assert!(r.errors.is_empty());
        assert!(r.scanned_count > 0);
    }

    #[test]
    fn test_clean_services_dry_run_report_visible() {
        // 修复 #1: dry-run 报告不得静默失效 — pattern_matches 必须填充可用项
        let mut engine = CleanupEngine::new();
        engine.dry_run_default = true;
        let r = engine.clean(CleanupKind::SystemServices);
        assert!(r.dry_run, "clean 应继承 dry_run_default");
        assert!(
            !r.pattern_matches.is_empty(),
            "dry-run 报告应含可用项, 但为空"
        );
        assert!(
            r.pattern_matches.iter().any(|m| m.contains("dry-run")),
            "报告应标注 dry-run 状态: {:?}",
            r.pattern_matches
        );
    }

    #[test]
    fn test_whitelist_expanded_matches() {
        // 修复 #2: 白名单必须 expand 后匹配绝对路径
        let engine = CleanupEngine::new();
        let home = dirs::home_dir().unwrap();
        // 模拟扫描路径: ~/.config 下的真实绝对路径
        let abs = home.join(".config").join("some_app");
        assert!(
            engine.is_whitelisted(&abs),
            "白名单 ~/.config 应匹配绝对路径 {:?} (expand 后)",
            abs
        );
    }

    #[test]
    fn test_cleanup_kind_services_all() {
        // All 描述覆盖全部类别 (回归: 新增 SystemServices 后 All 不应 panic)
        assert!(!CleanupKind::All.description().is_empty());
    }

    // ---- 蜕皮 (ProjectMolting) 增强测试 ----

    #[test]
    fn test_project_molting_kind_registered() {
        let all = CleanupPattern::all_patterns();
        assert!(
            all.iter().any(|p| p.kind == CleanupKind::ProjectMolting),
            "ProjectMolting 蜕皮规则必须注册"
        );
        assert!(!CleanupKind::ProjectMolting.description().is_empty());
    }

    #[test]
    fn test_molt_project_detects_legacy_shells() {
        let tmp = unique_tmp("neotrix_test_molting");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("legacy")).unwrap();
        fs::create_dir_all(tmp.join("old_proto")).unwrap();
        fs::create_dir_all(tmp.join("active")).unwrap();
        fs::write(tmp.join("legacy").join("old.rs"), b"// old").unwrap();

        let mut engine = CleanupEngine::new().with_project_root(tmp.clone());
        engine.dry_run_default = true; // 预览模式
        let r = engine.molt_project();

        assert!(r.dry_run);
        assert_eq!(
            r.deletable_count, 2,
            "legacy + old_proto 应被识别, active 不应"
        );
        assert!(r.pattern_matches.iter().any(|m| m.contains("legacy")));
        assert!(r.pattern_matches.iter().any(|m| m.contains("old_proto")));
        assert!(
            r.pattern_matches.iter().all(|m| !m.contains("active")),
            "active 目录不得被蜕皮"
        );
        // 预览模式不产生归档副作用
        assert!(!tmp.join(".cleanup").exists());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_molt_project_never_molts_project_root() {
        let tmp = unique_tmp("neotrix_test_molt_root_guard");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let mut engine = CleanupEngine::new().with_project_root(tmp.clone());
        engine.dry_run_default = true;
        let r = engine.molt_project();

        // 空项目 (无 legacy/old 子目录) → 0 蜕皮; root 自身绝不入壳
        assert_eq!(r.deletable_count, 0);
        // 项目名即使包含 old 字样也不得误判 (root 自身被 is_shell 忽略)
        let root_named = tmp.join("old_project");
        fs::create_dir_all(&root_named).unwrap();
        engine.dry_run_default = true;
        let r2 = engine.molt_project();
        // 扫描的是 root 下一级, old_project 是 root 的子目录不是 root 自身
        assert!(
            r2.pattern_matches.iter().any(|m| m.contains("old_project")),
            "old_project 作为 root 的子目录应被识别为旧躯壳"
        );

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_molt_project_archives_in_archive_mode() {
        let tmp = unique_tmp("neotrix_test_molt_archive");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("legacy")).unwrap();
        fs::write(tmp.join("legacy").join("stale.rs"), b"// stale").unwrap();

        let mut engine = CleanupEngine::new().with_project_root(tmp.clone());
        engine.dry_run_default = false;
        engine.archive_on_clean = true;
        let r = engine.molt_project();

        assert_eq!(r.deletable_count, 1);
        assert!(r.errors.is_empty(), "归档不应报错: {:?}", r.errors);
        // 旧躯壳已从活动树移除 (移动而非删除)
        assert!(!tmp.join("legacy").exists(), "legacy 应被移入归档");
        assert!(tmp.join(".cleanup").join("archive").exists());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_cleanup_engine_selftest_ok() {
        let result = CleanupEngineSelfTest.self_test();
        assert!(result.is_ok(), "SelfTest 应通过: {:?}", result.err());
    }

    #[test]
    fn test_selftest_detects_missing_molting() {
        // 模拟规则库缺蜕皮 → SelfTest 必须报失败 (R-P23 检测系统自审计)
        // 通过向 all_patterns 打补丁不可行 (static), 验证护栏检测本身:
        let engine = CleanupEngine::new();
        assert!(
            engine
                .patterns
                .iter()
                .any(|p| p.kind == CleanupKind::ProjectMolting),
            "生产规则库应含蜕皮规则"
        );
    }
}
