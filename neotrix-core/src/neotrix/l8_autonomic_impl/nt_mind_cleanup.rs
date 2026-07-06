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

use std::fs;
use std::path::{Path, PathBuf};
use chrono::{Local, Utc};
use serde::{Deserialize, Serialize};

// ============================================================
// Cleanup 目录布局管理
// ============================================================

/// 项目清理系统目录
pub struct CleanupDirs {
    pub root: PathBuf,         // project/.cleanup/
    pub archive: PathBuf,      // project/.cleanup/archive/
    pub log: PathBuf,          // project/.cleanup/log/
    pub rules_file: PathBuf,   // project/.cleanup/rules.toml
    pub index_file: PathBuf,   // project/.cleanup/log/index.json
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
    pub source_path: String,       // 原始路径 (相对于项目根)
    pub archived_path: String,     // 归档后路径
    pub size_bytes: u64,
    pub is_dir: bool,
    pub archived_at: i64,          // Unix timestamp
    pub cleanup_kind: String,      // 清理类型标签
    pub sha256: Option<String>,    // 文件哈希 (可选)
}

/// 归档清单 (每个批次一个)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveManifest {
    pub batch_id: String,          // YYYY-MM-DD_HHMMSS
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
            if !src.exists() { continue; }

            let is_dir = src.is_dir();
            let size = fs::metadata(src).map(|m| m.len()).unwrap_or(0);
            let fname = src.file_name()
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
                fs::rename(src, &dest).or_else(|_| {
                    fs::copy(src, &dest).and_then(|_| fs::remove_file(src))
                })
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
        CleanupLog::log(&self.dirs.log, &CleanupLogEntry {
            action: "archive".into(),
            kind: kind.into(),
            items: manifest.total_items,
            bytes: manifest.total_bytes,
            batch_id: batch_id.clone(),
            success: true,
            error: None,
        });

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
        self.index.entries.iter()
            .filter(|e| e.source_path.to_lowercase().contains(&q)
                || e.cleanup_kind.to_lowercase().contains(&q))
            .collect()
    }
}

// ============================================================
// 清理日志系统 (.cleanup/log/history.jsonl)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupLogEntry {
    pub action: String,      // "scan" | "clean" | "archive" | "backup"
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
        if let Ok(mut f) = fs::OpenOptions::new()
            .create(true).append(true).open(&file)
        {
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
        content.lines()
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
                ".backup".into(), ".cleanup".into(), "target".into(),
                "node_modules".into(), ".git".into(),
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

        self.collect_files(&self.project_root, &backup_dir, &mut file_count, &mut total_bytes)?;

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
        CleanupLog::log(&log_dir, &CleanupLogEntry {
            action: "backup".into(),
            kind: "code".into(),
            items: file_count,
            bytes: total_bytes,
            batch_id: backup_id,
            success: true,
            error: None,
        });

        Ok(manifest)
    }

    fn collect_files(
        &self,
        src_dir: &Path,
        dst_dir: &Path,
        file_count: &mut usize,
        total_bytes: &mut u64,
    ) -> std::io::Result<()> {
        if !src_dir.is_dir() { return Ok(()); }

        for entry in fs::read_dir(src_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_string();

            // 跳过排除项
            if self.exclude_patterns.iter().any(|p| name_str == *p || name_str.starts_with(p)) {
                continue;
            }
            // 跳过隐藏目录 (除了 .cleanup 的 rules.toml)
            if name_str.starts_with('.') {
                continue;
            }

            let src_path = entry.path();
            let rel = src_path.strip_prefix(&self.project_root).unwrap_or(&src_path);
            let dst_path = dst_dir.join(rel);

            if src_path.is_dir() {
                // 递归
                self.collect_files(&src_path, dst_dir, file_count, total_bytes)?;
            } else {
                // 只备份代码文件
                let ext = src_path.extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let is_code = matches!(ext.as_str(),
                    "rs" | "toml" | "py" | "js" | "ts" | "tsx" | "jsx" | "json"
                    | "css" | "scss" | "html" | "md" | "sh" | "yml" | "yaml"
                    | "sql" | "proto" | "vue" | "svelte" | "rb" | "go" | "mod"
                    | "sum" | "lock" | "conf" | "cfg" | "ini" | "plist"
                );
                if !is_code { continue; }

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
        if index.backups.len() <= self.max_backups { return; }
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
    All,
}

impl CleanupKind {
    pub fn description(&self) -> &'static str {
        match self {
            CleanupKind::ProjectArtifacts => "项目构建产物 (target/, node_modules/, .build/, dist/, venv/)",
            CleanupKind::Cache => "系统缓存 (~/Library/Caches, .cache, pip, cargo)",
            CleanupKind::Logs => "日志文件 (*.log, *.out, 系统日志)",
            CleanupKind::TempFiles => "临时文件 (/tmp, /var/tmp, ~/tmp)",
            CleanupKind::MemoryPrune => "推理记忆修剪 (低奖励记忆, 过期轨迹)",
            CleanupKind::BrainSnapshot => "大脑快照清理 (保留最近 N 个快照)",
            CleanupKind::IDECaches => "IDE 缓存 (Cursor, VS Code, IntelliJ, Xcode derived data)",
            CleanupKind::All => "全部清理 (包含以上所有类别)",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPattern {
    pub name: &'static str,
    pub kind: CleanupKind,
    pub patterns: Vec<&'static str>,
    pub max_age_days: Option<i64>,
    pub safe: bool,
    pub recursive: bool,
}

impl CleanupPattern {
    pub fn all_patterns() -> Vec<Self> {
        vec![
            Self { name: "Rust build artifacts", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/target/**"], max_age_days: Some(7), safe: true, recursive: true },
            Self { name: "Node.js modules", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/node_modules/**"], max_age_days: Some(30), safe: true, recursive: true },
            Self { name: "Python venv", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/.venv/**", "**/venv/**", "**/.tox/**"], max_age_days: Some(60), safe: true, recursive: true },
            Self { name: "Build output", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/dist/**", "**/.build/**", "**/build/**", "**/out/**"], max_age_days: Some(30), safe: true, recursive: true },
            Self { name: "Next.js cache", kind: CleanupKind::ProjectArtifacts, patterns: vec!["**/.next/**"], max_age_days: Some(7), safe: true, recursive: true },
            Self { name: "Cargo registry cache", kind: CleanupKind::Cache, patterns: vec!["~/.cargo/registry/cache/**"], max_age_days: Some(90), safe: true, recursive: true },
            Self { name: "pip cache", kind: CleanupKind::Cache, patterns: vec!["~/.cache/pip/**"], max_age_days: Some(90), safe: true, recursive: true },
            Self { name: "npm cache", kind: CleanupKind::Cache, patterns: vec!["~/.npm/_cacache/**"], max_age_days: Some(90), safe: true, recursive: true },
            Self { name: "System temp", kind: CleanupKind::TempFiles, patterns: vec!["/tmp/**", "/var/tmp/**"], max_age_days: Some(1), safe: true, recursive: true },
            Self { name: "VS Code caches", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Application Support/Code/CachedData/**", "~/.vscode/extensions/.cache/**"], max_age_days: Some(30), safe: true, recursive: true },
            Self { name: "Cursor caches", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Application Support/Cursor/CachedData/**"], max_age_days: Some(30), safe: true, recursive: true },
            Self { name: "Xcode derived data", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Developer/Xcode/DerivedData/**"], max_age_days: Some(30), safe: true, recursive: true },
            Self { name: "IntelliJ caches", kind: CleanupKind::IDECaches, patterns: vec!["~/Library/Caches/JetBrains/**"], max_age_days: Some(30), safe: true, recursive: true },
        ]
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
            mode, self.kind, self.scanned_count, self.deletable_count,
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
    pub archive_on_clean: bool,       // true = 归档而非删除
    pub project_root: PathBuf,
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
            patterns: CleanupPattern::all_patterns(),
            whitelist: vec![
                PathBuf::from("~/.config"),
                PathBuf::from("~/.ssh"),
                PathBuf::from("~/.gnupg"),
            ],
            history: Vec::new(),
            dry_run_default: true,
            archive_on_clean: true,
            project_root: PathBuf::from("."),
            max_history: 50,
        }
    }

    pub fn with_project_root(mut self, root: PathBuf) -> Self {
        self.project_root = root;
        self
    }

    pub fn add_whitelist(&mut self, path: PathBuf) {
        self.whitelist.push(path);
    }

    fn is_whitelisted(&self, path: &Path) -> bool {
        self.whitelist.iter().any(|w| path.starts_with(w))
    }

    pub fn scan(&self, kind: CleanupKind, dry_run: bool) -> CleanupResult {
        let mut result = CleanupResult::new(kind);
        result.dry_run = dry_run;

        let relevant: Vec<&CleanupPattern> = self.patterns.iter()
            .filter(|p| kind == CleanupKind::All || p.kind == kind)
            .collect();

        for pattern in &relevant {
            for glob_pat in &pattern.patterns {
                let pat_str = glob_pat.replace("~", &dirs::home_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default());
                if let Ok(entries) = glob::glob(&pat_str) {
                    for entry in entries.flatten() {
                        if self.is_whitelisted(&entry) { continue; }
                        let is_old = if let Some(max_days) = pattern.max_age_days {
                            match std::fs::metadata(&entry) {
                                Ok(meta) => {
                                    if let Ok(modified) = meta.modified() {
                                        let age = Utc::now().timestamp() - modified.elapsed()
                                            .map(|d| d.as_secs() as i64)
                                            .unwrap_or(0);
                                        age > max_days * 86400
                                    } else { false }
                                }
                                Err(_) => false,
                            }
                        } else { true };

                        if is_old {
                            result.deletable_count += 1;
                            let size = std::fs::metadata(&entry)
                                .map(|m| m.len()).unwrap_or(0);
                            result.estimated_bytes += size;
                            if result.pattern_matches.len() < 20 {
                                result.pattern_matches.push(entry.to_string_lossy().to_string());
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
        let mut result = self.scan(kind, false);

        if !result.dry_run && !result.pattern_matches.is_empty() {
            if self.archive_on_clean {
                // 归档模式: 移动而非删除
                let mut archiver = Archiver::new(&self.project_root);
                match archiver.archive_paths(&result.pattern_matches, &format!("{:?}", kind)) {
                    Ok(manifest) => {
                        result.estimated_bytes = manifest.total_bytes;
                        log::info!("[cleanup] 已归档 {} 项到 .cleanup/archive/{}", manifest.total_items, manifest.batch_id);
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
        CleanupLog::log(&log_dir, &CleanupLogEntry {
            action: if self.archive_on_clean { "archive" } else { "clean" }.into(),
            kind: format!("{:?}", kind),
            items: result.deletable_count,
            bytes: result.estimated_bytes,
            batch_id: Local::now().format("%Y-%m-%d_%H%M%S").to_string(),
            success: result.errors.is_empty(),
            error: if result.errors.is_empty() { None } else { Some(result.errors.join("; ")) },
        });

        self.history.push(result.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        result
    }

    fn delete_paths(&self, result: &mut CleanupResult) {
        for path_str in &result.pattern_matches {
            let path = Path::new(path_str);
            if self.is_whitelisted(path) { continue; }
            if path.is_dir() {
                if let Err(e) = std::fs::remove_dir_all(path) {
                    result.errors.push(format!("删除目录失败 {}: {}", path_str, e));
                }
            } else if let Err(e) = std::fs::remove_file(path) {
                result.errors.push(format!("删除文件失败 {}: {}", path_str, e));
            }
        }
    }

    pub fn prune_memories(&self, _bank: &mut crate::neotrix::nt_mind::memory::ReasoningBank) -> usize {
        // 记忆修剪已迁移至 consolidate_memories
        0
    }

    pub fn prune_brain_snapshots(max_keep: usize) -> usize {
        let home = dirs::home_dir().unwrap_or_default();
        let snap_dir = home.join(".neotrix").join("snapshots");
        if !snap_dir.exists() { return 0; }
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
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_patterns() {
        let patterns = CleanupPattern::all_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|p| p.kind == CleanupKind::ProjectArtifacts));
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
        let tmp = std::env::temp_dir().join("neotrix_test_cleanup_dirs");
        let _ = fs::remove_dir_all(&tmp);
        let dirs = CleanupDirs::new(&tmp);
        dirs.ensure().expect("ensure cleanup dirs");
        assert!(dirs.archive.exists());
        assert!(dirs.log.exists());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_archiver_archive_paths() {
        let tmp = std::env::temp_dir().join("neotrix_test_archiver");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        // 创建测试文件
        let test_file = tmp.join("test.txt");
        fs::write(&test_file, b"hello world").unwrap();

        let mut archiver = Archiver::new(&tmp);
        let paths = vec![test_file.to_string_lossy().to_string()];
        let manifest = archiver.archive_paths(&paths, "test")
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
        let tmp = std::env::temp_dir().join("neotrix_test_backup");
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
        let tmp = std::env::temp_dir().join("neotrix_test_cleanup_log");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("log")).unwrap();

        CleanupLog::log(&tmp.join("log"), &CleanupLogEntry {
            action: "test".into(), kind: "test".into(), items: 1, bytes: 100,
            batch_id: "batch_1".into(), success: true, error: None,
        });

        let recent = CleanupLog::recent(&tmp.join("log"), 10);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].items, 1);

        let _ = fs::remove_dir_all(&tmp);
    }
}
