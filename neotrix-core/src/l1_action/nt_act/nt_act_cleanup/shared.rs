//! Cleanup Shared Types - 清理子系统共享类型
//!
//! 统一定义清理子系统的类型，消除跨模块冗余
//! 所有 cleanup 模块应从此处导入共享类型

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ═══════════════════════════════════════════════════
// 共享枚举
// ═══════════════════════════════════════════════════

/// 风险等级 (单一事实源)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RiskLevel {
    Safe,       // 自动重建的缓存
    Moderate,   // 可能需要重新下载/登录
    Risky,      // 可能包含用户数据
    Protected,  // 系统保护，不可删除
}

impl RiskLevel {
    pub fn score(&self) -> u8 {
        match self {
            RiskLevel::Safe => 10,
            RiskLevel::Moderate => 30,
            RiskLevel::Risky => 60,
            RiskLevel::Protected => 90,
        }
    }
}

/// 扫描类别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ScanCategory {
    SystemCache,
    SystemLog,
    TempFile,
    UserCache,
    UserLog,
    BrowserCache,
    DeveloperCache,
    ApplicationSupport,
    LargeFile,
    OldFile,
}

/// 缓存类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CacheType {
    System,
    Browser,
    NodeJs,
    Python,
    Rust,
    Go,
    Docker,
    Homebrew,
    Xcode,
    Application,
}

/// 删除方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeleteMethod {
    Trash,      // 移动到回收站 (via osascript)
    Archive,    // 归档到备份目录
    Permanent,  // 永久删除
    Skipped,    // 跳过
}

/// 清理策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupStrategy {
    Conservative, // 保守：只清理安全的缓存
    Balanced,     // 平衡：清理缓存和旧文件
    Aggressive,   // 激进：清理所有可清理的内容
    Custom,       // 自定义：基于规则
}

// ═══════════════════════════════════════════════════
// 共享结构体
// ═══════════════════════════════════════════════════

/// 统一清理结果 (消除 CleanResult/DevCleanResult 冗余)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanResult {
    pub name: String,
    pub success: bool,
    pub items_removed: usize,
    pub bytes_freed: u64,
    pub errors: Vec<String>,
}

/// 删除结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResult {
    pub path: PathBuf,
    pub success: bool,
    pub method: DeleteMethod,
    pub error: Option<String>,
    pub size_freed: u64,
}

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub age_days: u32,
    pub category: ScanCategory,
    pub risk_level: RiskLevel,
    pub description: String,
}

/// 缓存信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    pub name: String,
    pub cache_type: CacheType,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub file_count: usize,
    pub last_modified: Option<String>,
    pub safe_to_clean: bool,
}

/// 风险评估结果
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub level: RiskLevel,
    pub score: u8,
    pub reasons: Vec<String>,
    pub requires_confirmation: bool,
    pub recommendation: String,
}

/// 清理计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPlan {
    pub strategy: CleanupStrategy,
    pub categories: Vec<String>,
    pub max_risk_level: String,
    pub dry_run: bool,
    pub scheduled: bool,
    pub interval_hours: Option<u32>,
}

/// 清理统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStats {
    pub total_scanned: usize,
    pub total_cleaned: usize,
    pub total_skipped: usize,
    pub total_errors: usize,
    pub bytes_freed: u64,
    pub duration_ms: u64,
}

/// 清理报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupReport {
    pub strategy: CleanupStrategy,
    pub stats: CleanupStats,
    pub recommendations: Vec<String>,
}

/// 清理项目
#[derive(Debug, Clone)]
pub struct CleanupItem {
    pub path: String,
    pub category: String,
    pub size_bytes: u64,
    pub age_days: u32,
    pub risk_level: String,
}

// ═══════════════════════════════════════════════════
// 共享工具函数 (消除 DRY 违规)
// ═══════════════════════════════════════════════════

/// 计算目录大小 (单一事实源)
pub fn calculate_directory_size(path: &std::path::Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if let Ok(metadata) = std::fs::metadata(&entry_path) {
                if metadata.is_dir() {
                    total += calculate_directory_size(&entry_path);
                } else {
                    total += metadata.len();
                }
            }
        }
    }
    total
}

/// 统计文件数量 (单一事实源)
pub fn count_files(path: &std::path::Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if let Ok(metadata) = std::fs::metadata(&entry_path) {
                if metadata.is_dir() {
                    count += count_files(&entry_path);
                } else {
                    count += 1;
                }
            }
        }
    }
    count
}

/// 计算文件年龄 (天) (单一事实源)
pub fn calculate_age_days(metadata: &std::fs::Metadata) -> u32 {
    if let Ok(modified) = metadata.modified() {
        let elapsed = modified.elapsed().unwrap_or_default();
        (elapsed.as_secs() / 86400) as u32
    } else {
        0
    }
}

/// 获取最后修改时间字符串
pub fn get_last_modified(path: &std::path::Path) -> Option<String> {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| {
            let elapsed = t.elapsed().ok()?;
            let days = elapsed.as_secs() / 86400;
            Some(format!("{} days ago", days))
        })
}

/// 格式化文件大小
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// 清理配置 (消除 dry_run 冗余)
#[derive(Debug, Clone)]
pub struct CleanupConfig {
    pub dry_run: bool,
    pub use_trash: bool,
    pub archive_before_delete: bool,
    pub verify_symlinks: bool,
    pub min_age_days: u32,
    pub min_size_bytes: u64,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            dry_run: false,
            use_trash: true,
            archive_before_delete: true,
            verify_symlinks: true,
            min_age_days: 7,
            min_size_bytes: 1024 * 1024, // 1MB
        }
    }
}

// ============================================================
// Glob 模式匹配 (旧系统 CleanupPattern 移植)
// ============================================================

/// 清理模式 — 定义清理规则
#[derive(Debug, Clone)]
pub struct CleanupPattern {
    pub name: &'static str,
    pub kind: ScanCategory,
    pub patterns: Vec<&'static str>,
    pub max_age_days: Option<i64>,
    pub safe: bool,
    pub recursive: bool,
}

impl CleanupPattern {
    /// 匹配路径是否命中模式
    pub fn matches(&self, path: &std::path::Path) -> bool {
        let path_str = path.to_string_lossy();
        for pattern in &self.patterns {
            if glob_match(pattern, &path_str) {
                return true;
            }
        }
        false
    }

    /// 按文件名匹配 (不递归)
    pub fn matches_name(&self, name: &str) -> bool {
        for pattern in &self.patterns {
            if glob_match(pattern, name) {
                return true;
            }
        }
        false
    }

    /// 返回所有内置清理模式
    pub fn all_patterns() -> Vec<Self> {
        vec![
            // Rust 构建产物
            Self {
                name: "Rust build artifacts",
                kind: ScanCategory::BuildArtifacts,
                patterns: vec!["**/target/**"],
                max_age_days: Some(7),
                safe: true,
                recursive: true,
            },
            // Node.js 模块
            Self {
                name: "Node.js modules",
                kind: ScanCategory::BuildArtifacts,
                patterns: vec!["**/node_modules/**"],
                max_age_days: Some(30),
                safe: true,
                recursive: true,
            },
            // Python 虚拟环境
            Self {
                name: "Python venv",
                kind: ScanCategory::BuildArtifacts,
                patterns: vec!["**/.venv/**", "**/venv/**", "**/.tox/**", "**/.nox/**"],
                max_age_days: Some(60),
                safe: true,
                recursive: true,
            },
            // 构建输出
            Self {
                name: "Build output",
                kind: ScanCategory::BuildArtifacts,
                patterns: vec!["**/dist/**", "**/.build/**", "**/build/**", "**/out/**"],
                max_age_days: Some(30),
                safe: true,
                recursive: true,
            },
            // Next.js 缓存
            Self {
                name: "Next.js cache",
                kind: ScanCategory::BuildArtifacts,
                patterns: vec!["**/.next/**", "**/.nuxt/**", "**/.output/**", "**/.svelte-kit/**"],
                max_age_days: Some(7),
                safe: true,
                recursive: true,
            },
            // Go 测试产物
            Self {
                name: "Go test artifacts",
                kind: ScanCategory::BuildArtifacts,
                patterns: vec!["**/*.test", "**/*.test.exe", "**/coverage.out"],
                max_age_days: Some(30),
                safe: true,
                recursive: true,
            },
            // 旧版本备份
            Self {
                name: "Old backups",
                kind: ScanCategory::BackupFiles,
                patterns: vec!["**/*_backup/**", "**/*_bak/**", "**/*.bak"],
                max_age_days: Some(14),
                safe: true,
                recursive: true,
            },
            // 临时文件
            Self {
                name: "Temp files",
                kind: ScanCategory::TempFiles,
                patterns: vec!["**/*.tmp", "**/*.temp", "**/tmp/**"],
                max_age_days: Some(7),
                safe: true,
                recursive: true,
            },
            // 日志文件
            Self {
                name: "Log files",
                kind: ScanCategory::OldLogs,
                patterns: vec!["**/*.log", "**/logs/**"],
                max_age_days: Some(30),
                safe: true,
                recursive: true,
            },
        ]
    }
}

/// 简易 glob 匹配 (支持 **, *, ?)
fn glob_match(pattern: &str, path: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();
    glob_match_parts(&pattern_parts, &path_parts)
}

fn glob_match_parts(pattern: &[&str], path: &[&str]) -> bool {
    if pattern.is_empty() {
        return path.is_empty();
    }

    if pattern[0] == "**" {
        // ** 匹配任意数量的路径段
        for i in 0..=path.len() {
            if glob_match_parts(&pattern[1..], &path[i..]) {
                return true;
            }
        }
        return false;
    }

    if path.is_empty() {
        return false;
    }

    if pattern[0] == "*" {
        // * 匹配单个路径段 (不含 /)
        return glob_match_parts(&pattern[1..], &path[1..]);
    }

    if pattern[0].contains('*') {
        // 带通配符的段匹配
        if glob_segment_match(pattern[0], path[0]) {
            return glob_match_parts(&pattern[1..], &path[1..]);
        }
        return false;
    }

    // 精确匹配
    if pattern[0] == path[0] {
        return glob_match_parts(&pattern[1..], &path[1..]);
    }

    false
}

/// 段内 glob 匹配 (支持 *, ?)
fn glob_segment_match(pattern: &str, text: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    glob_segment_match_chars(&pattern_chars, &text_chars)
}

fn glob_segment_match_chars(pattern: &[char], text: &[char]) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }

    if pattern[0] == '*' {
        // * 匹配任意字符序列
        for i in 0..=text.len() {
            if glob_segment_match_chars(&pattern[1..], &text[i..]) {
                return true;
            }
        }
        return false;
    }

    if text.is_empty() {
        return false;
    }

    if pattern[0] == '?' || pattern[0] == text[0] {
        return glob_segment_match_chars(&pattern[1..], &text[1..]);
    }

    false
}

/// 按模式扫描目录
pub fn scan_by_patterns(root: &Path, patterns: &[CleanupPattern]) -> Vec<ScanResult> {
    let mut results = Vec::new();

    for pattern in patterns {
        for entry in walkdir::WalkDir::new(root)
            .max_depth(if pattern.recursive { 10 } else { 1 })
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if !pattern.matches(path) {
                continue;
            }

            if let Ok(meta) = std::fs::metadata(path) {
                let age_days = calculate_age_days(&meta);
                let max_age = pattern.max_age_days.unwrap_or(i64::MAX);
                if age_days <= max_age {
                    let category = match pattern.kind {
                        ScanCategory::BuildArtifacts => ScanCategory::BuildArtifacts,
                        ScanCategory::BackupFiles => ScanCategory::BackupFiles,
                        ScanCategory::TempFiles => ScanCategory::TempFiles,
                        ScanCategory::OldLogs => ScanCategory::OldLogs,
                        _ => ScanCategory::Other,
                    };
                    results.push(ScanResult {
                        path: path.to_path_buf(),
                        size_bytes: meta.len(),
                        age_days,
                        category,
                        last_modified: get_last_modified(&meta),
                    });
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match_simple() {
        assert!(glob_match("*.log", "test.log"));
        assert!(glob_match("**/target/**", "/foo/bar/target/debug/app"));
        assert!(glob_match("**/.next/**", "/app/.next/cache"));
        assert!(!glob_match("*.log", "test.txt"));
    }

    #[test]
    fn test_glob_match_recursive() {
        assert!(glob_match("**/node_modules/**", "/app/node_modules/package/dist/index.js"));
        assert!(!glob_match("**/node_modules/**", "/app/package.json"));
    }

    #[test]
    fn test_cleanup_pattern_matches() {
        let pattern = CleanupPattern {
            name: "test",
            kind: ScanCategory::BuildArtifacts,
            patterns: vec!["**/target/**"],
            max_age_days: None,
            safe: true,
            recursive: true,
        };
        assert!(pattern.matches(Path::new("/app/target/debug/app")));
        assert!(!pattern.matches(Path::new("/app/src/main.rs")));
    }

    #[test]
    fn test_scan_by_patterns() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("test.log"), "log content").unwrap();
        std::fs::write(temp.path().join("test.txt"), "text content").unwrap();

        let patterns = vec![CleanupPattern {
            name: "logs",
            kind: ScanCategory::OldLogs,
            patterns: vec!["*.log"],
            max_age_days: None,
            safe: true,
            recursive: false,
        }];

        let results = scan_by_patterns(temp.path(), &patterns);
        assert_eq!(results.len(), 1);
        assert!(results[0].path.to_string_lossy().contains("test.log"));
    }

    use tempfile::TempDir;

    #[test]
    fn test_calculate_directory_size() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("a.txt"), "hello").unwrap();
        std::fs::write(temp.path().join("b.txt"), "world!").unwrap();
        let size = calculate_directory_size(temp.path());
        assert!(size > 0);
    }

    #[test]
    fn test_count_files() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("a.txt"), "a").unwrap();
        std::fs::write(temp.path().join("b.txt"), "b").unwrap();
        assert_eq!(count_files(temp.path()), 2);
    }

    #[test]
    fn test_calculate_age_days() {
        let meta = std::fs::metadata(tempfile::NamedTempFile::new().unwrap().path()).unwrap();
        let age = calculate_age_days(&meta);
        assert_eq!(age, 0);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert!(format_size(1024).contains("KB"));
        assert!(format_size(1024 * 1024).contains("MB"));
        assert!(format_size(1024 * 1024 * 1024).contains("GB"));
    }
}
