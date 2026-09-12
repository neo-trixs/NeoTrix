//! Dev Tool Cleaner - 开发工具清理器
//!
//! 清理 node_modules/target/venv 等开发产物
//! 域: NT-ACT (行动执行者)
//! 层: L1 Action

use super::shared::*;
use std::path::Path;

pub(crate) struct DevToolCleaner {
    config: CleanupConfig,
}

impl DevToolCleaner {
    pub fn new() -> Self { Self { config: CleanupConfig::default() } }
    pub fn with_config(config: CleanupConfig) -> Self { Self { config } }
    pub fn set_dry_run(&mut self, dry_run: bool) { self.config.dry_run = dry_run; }

    pub fn clean_node_modules(&self, project_dir: &Path) -> CleanResult {
        let path = project_dir.join("node_modules");
        self.clean_directory(&path, "node_modules")
    }

    pub fn clean_target(&self, project_dir: &Path) -> CleanResult {
        let path = project_dir.join("target");
        self.clean_directory(&path, "target")
    }

    pub fn clean_venv(&self, project_dir: &Path) -> CleanResult {
        let candidates = ["venv", ".venv", "__pycache__"];
        let mut total_freed = 0u64;
        let mut total_removed = 0usize;
        let mut errors = Vec::new();
        let mut success = true;

        for name in &candidates {
            let path = project_dir.join(name);
            if path.exists() {
                let r = self.clean_directory(&path, name);
                if r.success {
                    total_freed += r.bytes_freed;
                    total_removed += r.items_removed;
                } else {
                    success = false;
                    errors.extend(r.errors);
                }
            }
        }

        CleanResult { name: "python_artifacts".into(), success, items_removed: total_removed, bytes_freed: total_freed, errors }
    }

    pub fn clean_next(&self, project_dir: &Path) -> CleanResult {
        let path = project_dir.join(".next");
        self.clean_directory(&path, ".next")
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

impl Default for DevToolCleaner { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_clean_node_modules_dry_run() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir(temp.path().join("node_modules")).unwrap();
        let mut c = DevToolCleaner::new();
        c.set_dry_run(true);
        let r = c.clean_node_modules(temp.path());
        assert!(r.success);
        assert!(temp.path().join("node_modules").exists());
    }
}
