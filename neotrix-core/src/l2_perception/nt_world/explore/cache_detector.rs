//! Cache Detector - 缓存检测器
//!
//! 检测各类开发工具、浏览器、应用的缓存
//! 域: NT-WORLD (虚空探索者)
//! 层: L2 Perception

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheType {
    NodeJs,
    Python,
    Rust,
    Go,
    Xcode,
    Browser,
    Homebrew,
    Docker,
    System,
}

#[derive(Debug, Clone)]
pub struct CacheInfo {
    pub name: String,
    pub cache_type: CacheType,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub file_count: u64,
    pub last_modified: Option<std::time::SystemTime>,
    pub safe_to_clean: bool,
}

fn calculate_directory_size(path: &Path) -> u64 {
    std::fs::read_dir(path)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| {
                    let p = e.path();
                    if p.is_dir() {
                        calculate_directory_size(&p)
                    } else {
                        std::fs::metadata(&p).ok().map(|m| m.len()).unwrap_or(0)
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}

fn count_files(path: &Path) -> u64 {
    std::fs::read_dir(path)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| {
                    let p = e.path();
                    if p.is_dir() {
                        count_files(&p)
                    } else {
                        1
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}

fn get_last_modified(path: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(path).ok().and_then(|m| m.modified().ok())
}

struct CacheEntry {
    name: String,
    cache_type: CacheType,
    path_resolver: Box<dyn Fn() -> Option<PathBuf>>,
    safe_to_clean: bool,
}

pub struct CacheDetector {
    cache_registry: Vec<CacheEntry>,
}

impl CacheDetector {
    pub fn new() -> Self {
        let mut d = Self { cache_registry: Vec::new() };
        d.init_cache_registry();
        d
    }

    fn init_cache_registry(&mut self) {
        let home = dirs::home_dir().unwrap_or_default();
        let entries: Vec<(&str, CacheType, Box<dyn Fn() -> Option<PathBuf>>, bool)> = vec![
            ("npm", CacheType::NodeJs, Box::new(move || Some(home.join(".npm"))), true),
            ("yarn", CacheType::NodeJs, Box::new(move || Some(home.join(".cache/yarn"))), true),
            ("pip", CacheType::Python, Box::new(move || Some(home.join("Library/Caches/pip"))), true),
            ("cargo-registry", CacheType::Rust, Box::new(move || Some(home.join(".cargo/registry"))), true),
            ("go-build", CacheType::Go, Box::new(move || Some(home.join("Library/Caches/go-build"))), true),
            ("xcode-derived", CacheType::Xcode, Box::new(move || Some(home.join("Library/Developer/Xcode/DerivedData"))), true),
            ("chrome", CacheType::Browser, Box::new(move || Some(home.join("Library/Caches/Google/Chrome"))), true),
            ("safari", CacheType::Browser, Box::new(move || Some(home.join("Library/Caches/com.apple.Safari"))), true),
            ("firefox", CacheType::Browser, Box::new(move || Some(home.join("Library/Caches/Firefox"))), true),
            ("brew-cli", CacheType::Homebrew, Box::new(|| {
                std::process::Command::new("brew").arg("--cache").output().ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok()).map(|s| PathBuf::from(s.trim()))
            }), true),
            ("docker-cli", CacheType::Docker, Box::new(|| {
                std::process::Command::new("docker").args(["system", "info", "--format", "{{.DockerRootDir}}"]).output().ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok()).map(|s| PathBuf::from(s.trim()))
            }), false),
        ];
        for (name, cache_type, resolver, safe) in entries {
            self.cache_registry.push(CacheEntry { name: name.into(), cache_type, path_resolver: resolver, safe_to_clean: safe });
        }
    }

    pub fn detect_all(&self) -> Vec<CacheInfo> {
        use rayon::prelude::*;
        let mut caches: Vec<CacheInfo> = self.cache_registry.par_iter().filter_map(|entry| {
            let path = (entry.path_resolver)()?;
            if !path.exists() { return None; }
            let size = calculate_directory_size(&path);
            let file_count = count_files(&path);
            Some(CacheInfo {
                name: entry.name.clone(), cache_type: entry.cache_type.clone(),
                path: path.clone(), size_bytes: size, file_count,
                last_modified: get_last_modified(&path), safe_to_clean: entry.safe_to_clean,
            })
        }).collect();
        caches.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        caches
    }

    pub fn detect_by_type(&self, cache_type: &CacheType) -> Vec<CacheInfo> {
        self.detect_all().into_iter().filter(|c| &c.cache_type == cache_type).collect()
    }
}

impl Default for CacheDetector { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_new() {
        let d = CacheDetector::new();
        assert!(!d.cache_registry.is_empty());
    }

    #[test]
    fn test_detect_all() {
        let d = CacheDetector::new();
        let caches = d.detect_all();
        for c in &caches {
            assert!(!c.name.is_empty());
        }
    }
}
