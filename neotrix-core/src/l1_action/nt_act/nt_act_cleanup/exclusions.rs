//! Exclusions — 用户排除表 (吸收 PureMac CleanupExclusions)
//!
//! 域: NT-ACT · 层: L1 Action
//! 持久化于 `~/.config/neotrix/cleanup-exclusions.txt` (每行一个绝对路径)。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::locations::normalize;

/// 排除表存取
pub struct CleanupExclusions {
    file: PathBuf,
    paths: Vec<PathBuf>,
}

impl CleanupExclusions {
    /// 默认: `~/.config/neotrix/cleanup-exclusions.txt`
    pub fn load_default() -> Self {
        let file = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("neotrix")
            .join("cleanup-exclusions.txt");
        Self::load(file)
    }

    pub fn load(file: PathBuf) -> Self {
        let paths = read_exclusions(&file).unwrap_or_default();
        Self { file, paths }
    }

    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    /// 添加排除 (绝对路径，不含 NUL；已存在则 no-op)
    pub fn add(&mut self, path: &Path) -> io::Result<bool> {
        let raw = path.to_string_lossy();
        if !raw.starts_with('/') || raw.contains('\0') {
            return Ok(false);
        }
        let standardized = normalize(path);
        if self.paths.contains(&standardized) {
            return Ok(false);
        }
        self.paths.push(standardized);
        self.persist()
    }

    /// 移除排除；返回是否确实移除了项
    pub fn remove(&mut self, path: &Path) -> io::Result<bool> {
        let standardized = normalize(path);
        let before = self.paths.len();
        self.paths.retain(|p| p != &standardized);
        if self.paths.len() == before {
            return Ok(false);
        }
        self.persist()
    }

    fn persist(&self) -> io::Result<bool> {
        if let Some(parent) = self.file.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = String::new();
        for p in &self.paths {
            out.push_str(&p.to_string_lossy());
            out.push('\n');
        }
        fs::write(&self.file, out)?;
        Ok(true)
    }
}

fn read_exclusions(file: &Path) -> io::Result<Vec<PathBuf>> {
    let content = fs::read_to_string(file)?;
    Ok(content
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('/') && !l.is_empty())
        .map(PathBuf::from)
        .collect())
}

/// path 是否被排除表命中 (祖先/后代双向，对齐 PureMac excludes)
pub fn excludes(path: &Path, excluded: &[PathBuf]) -> bool {
    if excluded.is_empty() {
        return false;
    }
    let std = normalize(path);
    let forms = [path.to_path_buf(), std.clone()];
    for root in excluded {
        let root_norm = normalize(root);
        let root_forms = [root.to_path_buf(), root_norm.clone()];
        for rf in &root_forms {
            if rf.as_os_str() == "/" {
                return true;
            }
            for c in &forms {
                if c == rf || c.starts_with(rf.as_path()) || rf.starts_with(c.as_path()) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_excludes_exact_and_descendant() {
        let root = PathBuf::from("/Users/neo/Library/Caches/KeepMe");
        let list = vec![root.clone()];
        assert!(excludes(
            Path::new("/Users/neo/Library/Caches/KeepMe"),
            &list
        ));
        assert!(excludes(
            Path::new("/Users/neo/Library/Caches/KeepMe/sub"),
            &list
        ));
        assert!(!excludes(
            Path::new("/Users/neo/Library/Caches/Other"),
            &list
        ));
    }

    #[test]
    fn test_add_and_remove_persists() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("ex.txt");
        let mut ex = CleanupExclusions::load(file.clone());
        assert!(ex.add(Path::new("/tmp/junk")).unwrap());
        assert!(!ex.add(Path::new("/tmp/junk")).unwrap());
        let reloaded = CleanupExclusions::load(file);
        assert!(reloaded.paths().contains(&PathBuf::from("/tmp/junk")));
        let mut ex2 = reloaded;
        assert!(ex2.remove(Path::new("/tmp/junk")).unwrap());
        assert!(ex2.paths().is_empty());
    }

    #[test]
    fn test_reject_non_absolute() {
        let dir = TempDir::new().unwrap();
        let mut ex = CleanupExclusions::load(dir.path().join("ex.txt"));
        assert!(!ex.add(Path::new("relative/path")).unwrap());
    }
}
