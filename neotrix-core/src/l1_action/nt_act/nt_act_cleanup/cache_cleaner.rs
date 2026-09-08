//! Cache Cleaner - 缓存清理器
//!
//! 清理各类开发工具缓存
//! 域: NT-ACT (行动执行者)
//! 层: L1 Action

use super::shared::*;
use std::path::{Path, PathBuf};

pub struct CacheCleaner {
    config: CleanupConfig,
}

impl CacheCleaner {
    pub fn new() -> Self { Self { config: CleanupConfig::default() } }
    pub fn with_config(config: CleanupConfig) -> Self { Self { config } }
    pub fn set_dry_run(&mut self, dry_run: bool) { self.config.dry_run = dry_run; }

    pub fn clean_npm(&self) -> CleanResult {
        let home = dirs::home_dir().unwrap_or_default();
        let path = home.join(".npm");
        self.clean_directory(&path, "npm")
    }

    pub fn clean_pip(&self) -> CleanResult {
        let home = dirs::home_dir().unwrap_or_default();
        let path = home.join("Library/Caches/pip");
        self.clean_directory(&path, "pip")
    }

    pub fn clean_cargo(&self) -> CleanResult {
        let home = dirs::home_dir().unwrap_or_default();
        let path = home.join(".cargo/registry");
        self.clean_directory(&path, "cargo")
    }

    pub fn clean_brew(&self) -> CleanResult {
        let output = std::process::Command::new("brew").arg("--cache").output();
        match output {
            Ok(o) if o.status.success() => {
                let cache = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let path = PathBuf::from(&cache);
                self.clean_directory(&path, "homebrew")
            }
            _ => CleanResult { name: "homebrew".into(), success: false, items_removed: 0, bytes_freed: 0, errors: vec!["无法获取 brew 缓存路径".into()] }
        }
    }

    pub fn clean_chrome(&self) -> CleanResult {
        let home = dirs::home_dir().unwrap_or_default();
        let path = home.join("Library/Caches/Google/Chrome");
        self.clean_directory(&path, "chrome")
    }

    fn clean_directory(&self, path: &Path, name: &str) -> CleanResult {
        if !path.exists() {
            return CleanResult { name: name.into(), success: true, items_removed: 0, bytes_freed: 0, errors: vec![] };
        }
        let size = calculate_directory_size(path);
        let files = count_files(path);
        if self.config.dry_run {
            return CleanResult { name: name.into(), success: true, items_removed: files, bytes_freed: 0, errors: vec![] };
        }
        match std::fs::remove_dir_all(path) {
            Ok(_) => CleanResult { name: name.into(), success: true, items_removed: files, bytes_freed: size, errors: vec![] },
            Err(e) => CleanResult { name: name.into(), success: false, items_removed: 0, bytes_freed: 0, errors: vec![format!("清理失败: {}", e)] },
        }
    }
}

impl Default for CacheCleaner { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_clean_dry_run() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("a.txt"), "x").unwrap();
        let mut c = CacheCleaner::new();
        c.set_dry_run(true);
        let r = c.clean_directory(temp.path(), "test");
        assert!(r.success);
        assert_eq!(r.items_removed, 1);
        assert!(temp.path().exists());
    }
}
