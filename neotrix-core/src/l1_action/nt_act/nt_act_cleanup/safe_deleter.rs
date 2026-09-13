//! Safe Deleter - 安全删除器
//!
//! 安全删除文件，支持回收站/归档/永久删除
//! 域: NT-ACT (行动执行者)
//! 层: L1 Action

use super::shared::*;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// 安全删除器
pub struct SafeDeleter {
    config: CleanupConfig,
    archive_path: PathBuf,
}

impl SafeDeleter {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            config: CleanupConfig::default(),
            archive_path: home.join(".cleanup/archive"),
        }
    }

    pub fn with_config(config: CleanupConfig) -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            config,
            archive_path: home.join(".cleanup/archive"),
        }
    }

    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.config.dry_run = dry_run;
    }

    pub fn set_use_trash(&mut self, use_trash: bool) {
        self.config.use_trash = use_trash;
    }

    pub(crate) fn _set_archive_before_delete(&mut self, archive: bool) {
        self.config.archive_before_delete = archive;
    }

    fn validate_symlink(&self, path: &Path) -> Result<(), String> {
        if !self.config.verify_symlinks {
            return Ok(());
        }
        if path.is_symlink() {
            let target = std::fs::read_link(path)
                .map_err(|e| format!("无法读取符号链接: {}", e))?;
            let protected = ["/System", "/usr", "/bin", "/sbin", "/private/var/db", "/etc"];
            let target_str = target.to_string_lossy();
            for p in &protected {
                if target_str.starts_with(p) {
                    return Err(format!("符号链接指向系统保护目录: {} -> {}", path.display(), target.display()));
                }
            }
            if let Some(home) = dirs::home_dir() {
                if !target.starts_with(&home) && !target.starts_with("/tmp") {
                    return Err(format!("符号链接指向非用户目录: {} -> {}", path.display(), target.display()));
                }
            }
        }
        Ok(())
    }

    fn trash_file(&self, path: &Path) -> Result<(), String> {
        let script = format!(
            r#"tell application "Finder"
                set theFile to POSIX file "{}" as alias
                move theFile to trash
            end tell"#,
            path.to_string_lossy()
        );
        let output = std::process::Command::new("osascript")
            .arg("-e").arg(&script)
            .output()
            .map_err(|e| format!("执行 osascript 失败: {}", e))?;
        if output.status.success() { Ok(()) }
        else { Err(format!("Finder 回收失败: {}", String::from_utf8_lossy(&output.stderr))) }
    }

    pub fn delete(&self, path: &Path) -> DeleteResult {
        if self.config.dry_run {
            return DeleteResult { path: path.to_path_buf(), success: true, method: DeleteMethod::Skipped, error: None, size_freed: 0 };
        }
        if let Err(e) = self.validate_symlink(path) {
            return DeleteResult { path: path.to_path_buf(), success: false, method: DeleteMethod::Skipped, error: Some(e), size_freed: 0 };
        }
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        if self.config.archive_before_delete {
            if let Err(e) = self.archive_file(path) {
                return DeleteResult { path: path.to_path_buf(), success: false, method: DeleteMethod::Archive, error: Some(format!("归档失败: {}", e)), size_freed: 0 };
            }
        }
        if self.config.use_trash {
            match self.trash_file(path) {
                Ok(_) => DeleteResult { path: path.to_path_buf(), success: true, method: DeleteMethod::Trash, error: None, size_freed: size },
                Err(_) => match std::fs::remove_file(path) {
                    Ok(_) => DeleteResult { path: path.to_path_buf(), success: true, method: DeleteMethod::Permanent, error: None, size_freed: size },
                    Err(e) => DeleteResult { path: path.to_path_buf(), success: false, method: DeleteMethod::Permanent, error: Some(format!("删除失败: {}", e)), size_freed: 0 },
                }
            }
        } else {
            match std::fs::remove_file(path) {
                Ok(_) => DeleteResult { path: path.to_path_buf(), success: true, method: DeleteMethod::Permanent, error: None, size_freed: size },
                Err(e) => DeleteResult { path: path.to_path_buf(), success: false, method: DeleteMethod::Permanent, error: Some(format!("删除失败: {}", e)), size_freed: 0 },
            }
        }
    }

    pub fn delete_directory(&self, path: &Path) -> DeleteResult {
        if self.config.dry_run {
            return DeleteResult { path: path.to_path_buf(), success: true, method: DeleteMethod::Skipped, error: None, size_freed: 0 };
        }
        if let Err(e) = self.validate_symlink(path) {
            return DeleteResult { path: path.to_path_buf(), success: false, method: DeleteMethod::Skipped, error: Some(e), size_freed: 0 };
        }
        let size = calculate_directory_size(path);
        if self.config.archive_before_delete {
            if let Err(e) = self.archive_directory(path) {
                return DeleteResult { path: path.to_path_buf(), success: false, method: DeleteMethod::Archive, error: Some(format!("归档失败: {}", e)), size_freed: 0 };
            }
        }
        if self.config.use_trash {
            match self.trash_file(path) {
                Ok(_) => DeleteResult { path: path.to_path_buf(), success: true, method: DeleteMethod::Trash, error: None, size_freed: size },
                Err(_) => match std::fs::remove_dir_all(path) {
                    Ok(_) => DeleteResult { path: path.to_path_buf(), success: true, method: DeleteMethod::Permanent, error: None, size_freed: size },
                    Err(e) => DeleteResult { path: path.to_path_buf(), success: false, method: DeleteMethod::Permanent, error: Some(format!("删除失败: {}", e)), size_freed: 0 },
                }
            }
        } else {
            match std::fs::remove_dir_all(path) {
                Ok(_) => DeleteResult { path: path.to_path_buf(), success: true, method: DeleteMethod::Permanent, error: None, size_freed: size },
                Err(e) => DeleteResult { path: path.to_path_buf(), success: false, method: DeleteMethod::Permanent, error: Some(format!("删除失败: {}", e)), size_freed: 0 },
            }
        }
    }

    fn archive_file(&self, path: &Path) -> Result<(), String> {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H%M%S");
        let archive_dir = self.archive_path.join(timestamp.to_string());
        std::fs::create_dir_all(&archive_dir).map_err(|e| format!("创建归档目录失败: {}", e))?;
        std::fs::copy(path, archive_dir.join(file_name)).map_err(|e| format!("复制文件失败: {}", e))?;
        Ok(())
    }

    fn archive_directory(&self, path: &Path) -> Result<(), String> {
        let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H%M%S");
        let archive_dir = self.archive_path.join(timestamp.to_string());
        std::fs::create_dir_all(&archive_dir).map_err(|e| format!("创建归档目录失败: {}", e))?;
        self.copy_dir_all(path, &archive_dir.join(dir_name)).map_err(|e| format!("复制目录失败: {}", e))?;
        Ok(())
    }

    fn copy_dir_all(&self, src: &Path, dst: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                self.copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
            } else {
                std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
            }
        }
        Ok(())
    }

    pub(crate) fn _create_backup_manifest(&self, paths: &[PathBuf]) -> Result<PathBuf, String> {
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H%M%S");
        let manifest_dir = self.archive_path.join("manifests");
        std::fs::create_dir_all(&manifest_dir).map_err(|e| format!("创建清单目录失败: {}", e))?;
        let manifest_path = manifest_dir.join(format!("backup_{}.json", timestamp));

        #[derive(Serialize)]
        struct BackupEntry { path: PathBuf, size_bytes: u64, backup_time: String }

        let entries: Vec<BackupEntry> = paths.iter().filter_map(|p| {
            let size = std::fs::metadata(p).ok().map(|m| m.len()).unwrap_or(0);
            Some(BackupEntry { path: p.clone(), size_bytes: size, backup_time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string() })
        }).collect();

        let manifest = serde_json::to_string_pretty(&entries).map_err(|e| format!("序列化失败: {}", e))?;
        std::fs::write(&manifest_path, manifest).map_err(|e| format!("写入清单失败: {}", e))?;
        Ok(manifest_path)
    }
}

impl Default for SafeDeleter {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_delete_dry_run() {
        let temp = TempDir::new().unwrap();
        let f = temp.path().join("test.txt");
        std::fs::write(&f, "x").unwrap();
        let mut d = SafeDeleter::new();
        d.set_dry_run(true);
        d.set_use_trash(false);
        d.config.archive_before_delete = false;
        let r = d.delete(&f);
        assert!(r.success);
        assert!(matches!(r.method, DeleteMethod::Skipped));
        assert!(f.exists());
    }

    #[test]
    fn test_delete_permanent() {
        let temp = TempDir::new().unwrap();
        let f = temp.path().join("test.txt");
        std::fs::write(&f, "x").unwrap();
        let mut d = SafeDeleter::new();
        d.set_dry_run(false);
        d.set_use_trash(false);
        d.config.archive_before_delete = false;
        let r = d.delete(&f);
        assert!(r.success);
        assert!(!f.exists());
    }

    #[test]
    fn test_delete_directory() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("dir");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), "a").unwrap();
        let mut d = SafeDeleter::new();
        d.set_use_trash(false);
        d.config.archive_before_delete = false;
        let r = d.delete_directory(&dir);
        assert!(r.success);
        assert!(!dir.exists());
    }
}
