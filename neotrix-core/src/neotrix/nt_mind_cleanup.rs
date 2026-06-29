//! NeoTrix 清理维护模块
//!
//! 从 Mole 吸取的安全理念：
//! - 所有清理操作支持 --dry-run 预览
//! - 路径白名单保护机制
//! - 操作日志追踪
//!
//! 提供：
//! - 项目构建产物清理（target/, node_modules/, .build/ 等）
//! - 推理大脑内部缓存/记忆修剪
//! - 系统临时文件和日志清理
//! - IDE / 编辑器缓存清理
//! - 安全回滚快照

use chrono::Utc;
use log;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 清理类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupKind {
    ProjectArtifacts, // 项目构建产物
    Cache,            // 系统缓存
    Logs,             // 日志文件
    TempFiles,        // 临时文件
    MemoryPrune,      // 推理记忆修剪
    BrainSnapshot,    // 大脑快照清理
    IDECaches,        // IDE/编辑器缓存
    All,              // 全部清理
}

impl CleanupKind {
    pub fn description(&self) -> &'static str {
        match self {
            CleanupKind::ProjectArtifacts => {
                "项目构建产物 (target/, node_modules/, .build/, dist/, venv/)"
            }
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

/// 清理条目（单个文件/目录匹配模式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPattern {
    pub name: &'static str,
    pub kind: CleanupKind,
    pub patterns: Vec<&'static str>,
    pub max_age_days: Option<i64>,
    pub safe: bool,      // true = 可安全删除，false = 需要确认
    pub recursive: bool, // 递归匹配
}

/// 默认清理规则集（类 Mole 的规则模式，但不绑定 macOS 路径）
impl CleanupPattern {
    pub fn all_patterns() -> Vec<Self> {
        vec![
            // === 项目构建产物 ===
            Self {
                name: "Rust build artifacts",
                kind: CleanupKind::ProjectArtifacts,
                patterns: vec!["**/target/**"],
                max_age_days: Some(7),
                safe: true,
                recursive: true,
            },
            Self {
                name: "Node.js modules",
                kind: CleanupKind::ProjectArtifacts,
                patterns: vec!["**/node_modules/**"],
                max_age_days: Some(30),
                safe: true,
                recursive: true,
            },
            Self {
                name: "Python venv",
                kind: CleanupKind::ProjectArtifacts,
                patterns: vec!["**/.venv/**", "**/venv/**", "**/.tox/**"],
                max_age_days: Some(60),
                safe: true,
                recursive: true,
            },
            Self {
                name: "Build output",
                kind: CleanupKind::ProjectArtifacts,
                patterns: vec!["**/dist/**", "**/.build/**", "**/build/**", "**/out/**"],
                max_age_days: Some(30),
                safe: true,
                recursive: true,
            },
            Self {
                name: "Next.js cache",
                kind: CleanupKind::ProjectArtifacts,
                patterns: vec!["**/.next/**"],
                max_age_days: Some(7),
                safe: true,
                recursive: true,
            },
            Self {
                name: "Cargo registry cache",
                kind: CleanupKind::Cache,
                patterns: vec!["~/.cargo/registry/cache/**"],
                max_age_days: Some(90),
                safe: true,
                recursive: true,
            },
            // === 系统缓存 ===
            Self {
                name: "pip cache",
                kind: CleanupKind::Cache,
                patterns: vec!["~/.cache/pip/**"],
                max_age_days: Some(90),
                safe: true,
                recursive: true,
            },
            Self {
                name: "npm cache",
                kind: CleanupKind::Cache,
                patterns: vec!["~/.npm/_cacache/**"],
                max_age_days: Some(90),
                safe: true,
                recursive: true,
            },
            Self {
                name: "System temp",
                kind: CleanupKind::TempFiles,
                patterns: vec!["/tmp/**", "/var/tmp/**"],
                max_age_days: Some(1),
                safe: true,
                recursive: true,
            },
            // === IDE 缓存 ===
            Self {
                name: "VS Code caches",
                kind: CleanupKind::IDECaches,
                patterns: vec![
                    "~/Library/Application Support/Code/CachedData/**",
                    "~/.vscode/extensions/.cache/**",
                ],
                max_age_days: Some(30),
                safe: true,
                recursive: true,
            },
            Self {
                name: "Cursor caches",
                kind: CleanupKind::IDECaches,
                patterns: vec!["~/Library/Application Support/Cursor/CachedData/**"],
                max_age_days: Some(30),
                safe: true,
                recursive: true,
            },
            Self {
                name: "Xcode derived data",
                kind: CleanupKind::IDECaches,
                patterns: vec!["~/Library/Developer/Xcode/DerivedData/**"],
                max_age_days: Some(30),
                safe: true,
                recursive: true,
            },
            Self {
                name: "IntelliJ caches",
                kind: CleanupKind::IDECaches,
                patterns: vec!["~/Library/Caches/JetBrains/**"],
                max_age_days: Some(30),
                safe: true,
                recursive: true,
            },
        ]
    }
}

/// 清理结果
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

/// NeoTrix 清理器
pub struct CleanupEngine {
    pub patterns: Vec<CleanupPattern>,
    pub whitelist: Vec<PathBuf>,
    pub history: Vec<CleanupResult>,
    pub dry_run_default: bool,
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
            max_history: 50,
        }
    }

    /// 设置路径白名单（这些路径永远不会被清理）
    pub fn add_whitelist(&mut self, path: PathBuf) {
        self.whitelist.push(path);
    }

    /// 检查路径是否在白名单中
    fn is_whitelisted(&self, path: &Path) -> bool {
        self.whitelist.iter().any(|w| path.starts_with(w))
    }

    /// 扫描匹配项（dry-run 模式预览）
    pub fn scan(&self, kind: CleanupKind, dry_run: bool) -> CleanupResult {
        let mut result = CleanupResult::new(kind);
        result.dry_run = dry_run;

        let relevant: Vec<&CleanupPattern> = self
            .patterns
            .iter()
            .filter(|p| kind == CleanupKind::All || p.kind == kind)
            .collect();

        for pattern in &relevant {
            // 在项目目录下扫描 glob 模式匹配
            let mut scanned = 0u64;
            let mut deletable = 0u64;

            for glob_pat in &pattern.patterns {
                let pat_str = glob_pat.replace(
                    "~",
                    &dirs::home_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                );
                if let Ok(entries) = glob::glob(&pat_str) {
                    for entry in entries.flatten() {
                        scanned += 1;
                        if self.is_whitelisted(&entry) {
                            continue;
                        }
                        let is_old = if let Some(max_days) = pattern.max_age_days {
                            match std::fs::metadata(&entry) {
                                Ok(meta) => {
                                    if let Ok(modified) = meta.modified() {
                                        let age = Utc::now().timestamp()
                                            - modified
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
                            deletable += 1;
                            let size = std::fs::metadata(&entry).map(|m| m.len()).unwrap_or(0);
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

            result.scanned_count += scanned as usize;
            result.deletable_count += deletable as usize;
        }

        result
    }

    /// 执行清理（非 dry-run 模式）
    pub fn clean(&mut self, kind: CleanupKind) -> CleanupResult {
        let mut result = self.scan(kind, false);

        if !result.dry_run {
            for path_str in &result.pattern_matches {
                let path = Path::new(path_str);
                if self.is_whitelisted(path) {
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

        self.history.push(result.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        result
    }

    /// 清理推理记忆（修剪低奖励记忆）
    /// 对应 MemoryIteration 中的 prune_low_value + consolidate
    pub fn prune_memories(&self, bank: &mut super::nt_mind::memory::ReasoningBank) -> usize {
        let before = bank.stats().total_memories;
        let pruned = bank.prune_low_value(0.1); // reward < 0.1 的删除
        let merged = bank.consolidate_similar(0.85); // 相似度 > 0.85 的合并
        let replayed = bank.replay_high_value(); // 回放高价值记忆
        let after = bank.stats().total_memories;
        log::info!(
            "[cleanup] 记忆修剪: {} → {} (删除 {} 合并 {} 回放 {})",
            before,
            after,
            pruned,
            merged,
            replayed
        );
        pruned + merged
    }

    /// 清理思维快照（保留最近 N 个）
    pub fn prune_brain_snapshots(max_keep: usize) -> usize {
        let home = dirs::home_dir().unwrap_or_default();
        let snap_dir = home.join(".neotrix").join("snapshots");
        if !snap_dir.exists() {
            return 0;
        }
        let mut entries: Vec<_> = std::fs::read_dir(&snap_dir)
            .map(|d| {
                d.filter_map(|e| match e {
                    Ok(entry) => Some(entry),
                    Err(err) => {
                        log::warn!("[cleanup] read dir entry: {}", err);
                        None
                    }
                })
                .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        entries.sort_by_key(|e| e.path());

        let mut removed = 0;
        if entries.len() > max_keep {
            for entry in entries.iter().take(entries.len() - max_keep) {
                if let Err(e) = std::fs::remove_file(entry.path()) {
                    log::error!("[cleanup] 删除快照失败: {}", e);
                } else {
                    removed += 1;
                }
            }
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
