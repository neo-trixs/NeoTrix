//! CleaningEngine — 安全删除引擎 (吸收 PureMac CleaningEngine + CLI Cleaner)
//!
//! 域: NT-ACT · 层: L1 Action
//! 流程: exclusion → allow-list → symlink → TOCTOU 复测 → unlink/remove_dir_all

use std::path::PathBuf;

use super::exclusions::excludes;
use super::locations::{is_safe_to_delete, normalize};
use super::safety::{can_remove, revalidate};
use super::scan_engine::CleanableItem;
use super::shared::calculate_directory_size;

/// 清理结果
#[derive(Debug, Clone, Default)]
pub struct CleaningResult {
    pub freed_bytes: u64,
    pub items_cleaned: usize,
    pub skipped: Vec<(PathBuf, String)>,
    pub failed: Vec<(PathBuf, String)>,
    pub excluded: Vec<PathBuf>,
}

impl CleaningResult {
    pub fn had_problems(&self) -> bool {
        !self.skipped.is_empty() || !self.failed.is_empty()
    }
}

/// 引擎配置
#[derive(Debug, Clone)]
pub struct CleaningConfig {
    pub dry_run: bool,
    pub use_allowlist: bool,
    pub home: PathBuf,
    pub excluded: Vec<PathBuf>,
}

impl Default for CleaningConfig {
    fn default() -> Self {
        Self {
            dry_run: false,
            use_allowlist: true,
            home: dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
            excluded: Vec::new(),
        }
    }
}

/// 安全清理引擎
pub struct CleaningEngine {
    config: CleaningConfig,
}

impl CleaningEngine {
    pub fn new(config: CleaningConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(CleaningConfig::default())
    }

    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.config.dry_run = dry_run;
    }

    /// 清理一批项目
    pub fn clean_items(&self, items: &[CleanableItem]) -> CleaningResult {
        let mut result = CleaningResult::default();
        for item in items {
            if excludes(&item.path, &self.config.excluded) {
                result.excluded.push(item.path.clone());
                continue;
            }
            self.clean_one(item, &mut result);
        }
        result
    }

    fn clean_one(&self, item: &CleanableItem, result: &mut CleaningResult) {
        let path = &item.path;
        if !path.exists() {
            // 扫描后已消失 — 视为已处理
            result.items_cleaned += 1;
            result.freed_bytes += item.size_bytes;
            return;
        }

        let first = path.canonicalize().unwrap_or_else(|_| normalize(path));
        let verdict = can_remove(
            path,
            &self.config.home,
            &self.config.excluded,
            self.config.use_allowlist,
        );
        if !verdict.is_ok() {
            let reason = verdict.reason().unwrap_or("rejected").to_string();
            if reason == "excluded" {
                result.excluded.push(path.clone());
            } else {
                result.skipped.push((path.clone(), reason));
            }
            return;
        }
        if !is_safe_to_delete(&first, &self.config.home) {
            result
                .skipped
                .push((path.clone(), "outside allow-list".into()));
            return;
        }

        // TOCTOU: 删除前复测解析结果
        let re = revalidate(path, &first);
        if !re.is_ok() {
            result.skipped.push((
                path.clone(),
                re.reason().unwrap_or("revalidate").to_string(),
            ));
            return;
        }

        let size = if item.size_bytes > 0 {
            item.size_bytes
        } else if first.is_dir() {
            calculate_directory_size(&first)
        } else {
            std::fs::metadata(&first).map(|m| m.len()).unwrap_or(0)
        };

        if self.config.dry_run {
            result.freed_bytes += size;
            result.items_cleaned += 1;
            return;
        }

        let rm = if first.is_dir() && !first.is_symlink() {
            std::fs::remove_dir_all(&first)
        } else {
            std::fs::remove_file(&first)
        };

        match rm {
            Ok(()) => {
                // 删后复查: 同 inode 存活 → 失败
                if std::fs::symlink_metadata(&first).is_ok() {
                    result
                        .failed
                        .push((path.clone(), "item survived removal".into()));
                } else {
                    result.freed_bytes += size;
                    result.items_cleaned += 1;
                }
            }
            Err(e) => {
                let hint = match e.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        "locked or in use — close the owning app and retry".to_string()
                    }
                    _ => e.to_string(),
                };
                result.failed.push((path.clone(), hint));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn item(path: &std::path::Path, category: &str) -> CleanableItem {
        CleanableItem {
            name: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            path: path.to_path_buf(),
            size_bytes: 0,
            category: category.into(),
            selected: true,
            age_days: 0,
            tool: "test".into(),
        }
    }

    #[test]
    fn test_dry_run_keeps_files() {
        let dir = TempDir::new().unwrap();
        let home = dir.path().to_path_buf();
        let cache = home.join("Library/Caches/app");
        std::fs::create_dir_all(&cache).unwrap();
        std::fs::write(cache.join("f"), vec![1u8; 2048]).unwrap();
        let eng = CleaningEngine::new(CleaningConfig {
            dry_run: true,
            use_allowlist: true,
            home: home.clone(),
            excluded: vec![],
        });
        let r = eng.clean_items(&[item(&cache, "dev")]);
        assert_eq!(r.items_cleaned, 1);
        assert!(cache.exists());
    }

    #[test]
    fn test_real_delete_inside_allowlist() {
        let dir = TempDir::new().unwrap();
        let home = dir.path().to_path_buf();
        let cache = home.join("Library/Caches/app");
        std::fs::create_dir_all(&cache).unwrap();
        std::fs::write(cache.join("f"), vec![1u8; 2048]).unwrap();
        let eng = CleaningEngine::new(CleaningConfig {
            dry_run: false,
            use_allowlist: true,
            home: home.clone(),
            excluded: vec![],
        });
        let r = eng.clean_items(&[item(&cache, "dev")]);
        assert_eq!(r.items_cleaned, 1, "failed={:?}", r.failed);
        assert!(!cache.exists());
        assert!(r.freed_bytes >= 2048);
    }

    #[test]
    fn test_refuses_outside_allowlist() {
        let dir = TempDir::new().unwrap();
        let home = dir.path().to_path_buf();
        let docs = home.join("Documents");
        std::fs::create_dir_all(&docs).unwrap();
        let secret = docs.join("secret.txt");
        std::fs::write(&secret, vec![0u8; 4096]).unwrap();
        let eng = CleaningEngine::new(CleaningConfig {
            dry_run: false,
            use_allowlist: true,
            home,
            excluded: vec![],
        });
        let r = eng.clean_items(&[item(&secret, "junk")]);
        assert_eq!(r.items_cleaned, 0);
        assert!(secret.exists());
        assert!(!r.skipped.is_empty() || !r.failed.is_empty());
    }

    #[test]
    fn test_exclusion_skips() {
        let dir = TempDir::new().unwrap();
        let home = dir.path().to_path_buf();
        let cache = home.join("Library/Caches/KeepMe");
        std::fs::create_dir_all(&cache).unwrap();
        std::fs::write(cache.join("f"), vec![1u8; 2048]).unwrap();
        let eng = CleaningEngine::new(CleaningConfig {
            dry_run: false,
            use_allowlist: true,
            home: home.clone(),
            excluded: vec![cache.clone()],
        });
        let r = eng.clean_items(&[item(&cache, "dev")]);
        assert_eq!(r.items_cleaned, 0);
        assert!(cache.exists());
        assert_eq!(r.excluded.len(), 1);
    }
}
