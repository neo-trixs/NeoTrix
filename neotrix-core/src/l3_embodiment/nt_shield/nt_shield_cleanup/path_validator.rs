//! Path Validator - 路径验证器
//!
//! 验证路径安全性，防止误删系统关键文件
//! 域: NT-SHIELD (影卫)
//! 层: L3 Embodiment

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub real_path: PathBuf,
    pub is_symlink: bool,
    pub is_protected: bool,
    pub error: Option<String>,
}

pub struct _PathValidator {
    protected_paths: Vec<PathBuf>,
    protected_prefixes: Vec<String>,
}

impl _PathValidator {
    pub fn new() -> Self {
        let mut v = Self { protected_paths: Vec::new(), protected_prefixes: Vec::new() };
        v.init_protection_rules();
        v
    }

    fn init_protection_rules(&mut self) {
        self.protected_paths = vec![
            PathBuf::from("/System"), PathBuf::from("/usr"), PathBuf::from("/bin"),
            PathBuf::from("/sbin"), PathBuf::from("/private/var/db"),
            PathBuf::from("/private/var/vm"), PathBuf::from("/private/var/run"), PathBuf::from("/etc"),
        ];
        self.protected_prefixes = vec![
            "/System/Library".into(), "/usr/lib".into(), "/usr/bin".into(), "/private/var/db".into(),
        ];
    }

    pub fn validate(&self, path: &Path) -> ValidationResult {
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(e) => return ValidationResult { is_valid: false, real_path: path.to_path_buf(), is_symlink: false, is_protected: false, error: Some(format!("无法读取元数据: {}", e)) },
        };
        let is_symlink = metadata.file_type().is_symlink();
        if is_symlink {
            if let Ok(target) = std::fs::read_link(path) {
                if self.is_protected(&target) {
                    return ValidationResult { is_valid: false, real_path: path.to_path_buf(), is_symlink: true, is_protected: true, error: Some(format!("符号链接指向保护目录: {} -> {}", path.display(), target.display())) };
                }
            }
        }
        let real_path = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(e) => return ValidationResult { is_valid: false, real_path: path.to_path_buf(), is_symlink, is_protected: false, error: Some(format!("无法解析路径: {}", e)) },
        };
        let is_protected = self.is_protected(&real_path);
        if is_protected {
            return ValidationResult { is_valid: false, real_path, is_symlink, is_protected, error: Some("路径受系统保护".into()) };
        }
        if let Some(home) = dirs::home_dir() {
            if !real_path.starts_with(&home) && !real_path.starts_with("/tmp") {
                return ValidationResult { is_valid: false, real_path, is_symlink, is_protected, error: Some("路径不在用户目录下".into()) };
            }
        }
        ValidationResult { is_valid: true, real_path, is_symlink, is_protected, error: None }
    }

    fn is_protected(&self, path: &Path) -> bool {
        if self.protected_paths.contains(&path.to_path_buf()) { return true; }
        let s = path.to_string_lossy();
        self.protected_prefixes.iter().any(|p| s.starts_with(p))
    }

    pub fn _add_protected_path(&mut self, path: PathBuf) { self.protected_paths.push(path); }
    pub fn _add_protected_prefix(&mut self, prefix: String) { self.protected_prefixes.push(prefix); }
}

impl Default for _PathValidator { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_existing() {
        let v = _PathValidator::new();
        let temp = TempDir::new().unwrap();
        let f = temp.path().join("test.txt");
        std::fs::write(&f, "x").unwrap();
        let r = v.validate(&f);
        assert!(r.is_valid);
    }

    #[test]
    fn test_validate_protected() {
        let v = _PathValidator::new();
        let r = v.validate(Path::new("/System/Library"));
        assert!(!r.is_valid);
        assert!(r.is_protected);
    }

    #[test]
    fn test_validate_nonexistent() {
        let v = _PathValidator::new();
        let r = v.validate(Path::new("/nonexistent"));
        assert!(!r.is_valid);
    }
}
