//! System Scanner - 系统垃圾扫描
//!
//! 扫描系统级缓存、日志、临时文件
//! 域: NT-WORLD (虚空探索者)
//! 层: L2 Perception

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanCategory {
    SystemCache,
    SystemLog,
    TempFile,
    BrowserCache,
    DeveloperCache,
}

/// System scan risk level — used for system scanner risk assessment.
/// 
/// For the unified RiskLevel, see `neotrix_types::RiskLevel`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanRiskLevel {
    Safe,
    Moderate,
    High,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub age_days: u32,
    pub category: ScanCategory,
    pub risk_level: ScanRiskLevel,
    pub description: String,
}

/// Calculate file age in days from metadata.
///
/// Note: Uses modified time. Returns 0 if modification time is unavailable.
/// Real implementation needs:
/// - Configurable age source (created vs modified vs accessed)
/// - Timezone-aware calculation
pub fn calculate_age_days(metadata: &std::fs::Metadata) -> u32 {
    metadata.modified()
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|d| (d.as_secs() / 86400) as u32)
        .unwrap_or(0)
}

/// 扫描路径配置
#[derive(Debug, Clone)]
pub struct ScanPath {
    pub path: PathBuf,
    pub category: ScanCategory,
    pub risk_level: ScanRiskLevel,
    pub recursive: bool,
}

pub struct SystemScanner {
    scan_paths: Vec<ScanPath>,
    min_age_days: u32,
    min_size_bytes: u64,
}

impl SystemScanner {
    /// Create system scanner with default scan paths.
    ///
    /// Note: Default paths include Library/Caches, Library/Logs, /tmp,
    /// /private/var/folders, Chrome cache, .cargo/registry, .npm, Homebrew cache.
    /// Minimum age is 7 days, minimum size is 1MB.
    pub fn new() -> Self {
        let mut scanner = Self { scan_paths: Vec::new(), min_age_days: 7, min_size_bytes: 1024 * 1024 };
        scanner.init_default_paths();
        scanner
    }

    fn init_default_paths(&mut self) {
        let home = dirs::home_dir().unwrap_or_default();
        let paths = vec![
            (home.join("Library/Caches"), ScanCategory::SystemCache, ScanRiskLevel::Safe),
            (home.join("Library/Logs"), ScanCategory::SystemLog, ScanRiskLevel::Safe),
            (PathBuf::from("/tmp"), ScanCategory::TempFile, ScanRiskLevel::Safe),
            (PathBuf::from("/private/var/folders"), ScanCategory::TempFile, ScanRiskLevel::Safe),
            (home.join("Library/Caches/Google/Chrome"), ScanCategory::BrowserCache, ScanRiskLevel::Safe),
            (home.join(".cargo/registry"), ScanCategory::DeveloperCache, ScanRiskLevel::Moderate),
            (home.join(".npm"), ScanCategory::DeveloperCache, ScanRiskLevel::Safe),
            (home.join("Library/Caches/Homebrew"), ScanCategory::DeveloperCache, ScanRiskLevel::Safe),
        ];
        for (path, category, risk_level) in paths {
            self.scan_paths.push(ScanPath { path, category, risk_level, recursive: true });
        }
    }

    /// Set minimum file age filter (in days).
    ///
    /// Note: Files older than this threshold are included in scan results.
    /// Default is 7 days. Real implementation needs:
    /// - Validation (days >= 0)
    /// - Per-category age thresholds (logs vs caches)
    pub fn set_min_age(&mut self, days: u32) { self.min_age_days = days; }

    /// Set minimum file size filter (in bytes).
    ///
    /// Note: Files larger than this threshold are included in scan results.
    /// Default is 1MB. Real implementation needs:
    /// - Validation (bytes > 0)
    /// - Human-readable size parsing (e.g., "10MB" → 10485760)
    pub fn set_min_size(&mut self, bytes: u64) { self.min_size_bytes = bytes; }

    /// Add a custom scan path.
    ///
    /// Note: Path is always added with recursive=true. Real implementation needs:
    /// - Duplicate path detection
    /// - Path existence validation
    /// - Recursive flag parameter
    pub fn _add_scan_path(&mut self, path: PathBuf, category: ScanCategory, risk_level: ScanRiskLevel) {
        self.scan_paths.push(ScanPath { path, category, risk_level, recursive: true });
    }

    /// Scan all configured paths for system垃圾.
    ///
    /// Note: Uses rayon parallel iterator over scan_paths. Filters by min_age_days
    /// and min_size_bytes. Results sorted by size descending.
    /// Real implementation needs:
    /// - Progress callback for long scans
    /// - Cancellation support
    /// - Rate limiting for I/O-bound scans
    pub fn scan(&self) -> Vec<ScanResult> {
        use rayon::prelude::*;
        let results: Vec<ScanResult> = self.scan_paths.par_iter()
            .filter(|sp| sp.path.exists())
            .flat_map(|sp| self.scan_directory(sp))
            .collect();
        let mut sorted = results;
        sorted.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        sorted
    }

    fn scan_directory(&self, scan_path: &ScanPath) -> Vec<ScanResult> {
        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&scan_path.path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(metadata) = std::fs::metadata(&path) {
                    let size = metadata.len();
                    let age = calculate_age_days(&metadata);
                    if size >= self.min_size_bytes && age >= self.min_age_days {
                        results.push(ScanResult {
                            path: path.clone(),
                            size_bytes: size,
                            age_days: age,
                            category: scan_path.category.clone(),
                            risk_level: scan_path.risk_level.clone(),
                            description: self.generate_description(&path, &scan_path.category),
                        });
                    }
                }
            }
        }
        results
    }

    fn generate_description(&self, path: &Path, category: &ScanCategory) -> String {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
        match category {
            ScanCategory::SystemCache => format!("系统缓存: {}", name),
            ScanCategory::SystemLog => format!("系统日志: {}", name),
            ScanCategory::TempFile => format!("临时文件: {}", name),
            ScanCategory::BrowserCache => format!("浏览器缓存: {}", name),
            ScanCategory::DeveloperCache => format!("开发者缓存: {}", name),
        }
    }
}

impl Default for SystemScanner { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_scanner_new() {
        let s = SystemScanner::new();
        assert!(!s.scan_paths.is_empty());
    }

    #[test]
    fn test_scan_empty() {
        let temp = TempDir::new().unwrap();
        let mut s = SystemScanner::new();
        s.scan_paths.clear();
        s.min_age_days = 0;
        s.min_size_bytes = 0;
        s.scan_paths.push(ScanPath { path: temp.path().to_path_buf(), category: ScanCategory::TempFile, risk_level: ScanRiskLevel::Safe, recursive: false });
        assert!(s.scan().is_empty());
    }

    #[test]
    fn test_scan_with_files() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("test.log"), "content").unwrap();
        let mut s = SystemScanner::new();
        s.scan_paths.clear();
        s.min_age_days = 0;
        s.min_size_bytes = 0;
        s.scan_paths.push(ScanPath { path: temp.path().to_path_buf(), category: ScanCategory::SystemLog, risk_level: ScanRiskLevel::Safe, recursive: false });
        let r = s.scan();
        assert_eq!(r.len(), 1);
    }
}
