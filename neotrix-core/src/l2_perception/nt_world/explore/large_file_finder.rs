//! Large File Finder - 大文件查找器
//!
//! 查找系统中的大文件和旧文件
//! 域: NT-WORLD (虚空探索者)
//! 层: L2 Perception

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub age_days: u32,
    pub category: String,
    pub risk_level: String,
    pub description: String,
}

pub fn calculate_age_days(metadata: &std::fs::Metadata) -> u32 {
    metadata.modified()
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|d| (d.as_secs() / 86400) as u32)
        .unwrap_or(0)
}

pub struct LargeFileFinder {
    search_paths: Vec<PathBuf>,
    min_size_bytes: u64,
    min_age_days: u32,
}

impl LargeFileFinder {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            search_paths: vec![home.join("Downloads"), home.join("Desktop"), home.join("Documents")],
            min_size_bytes: 100 * 1024 * 1024, // 100MB
            min_age_days: 30,
        }
    }

    pub fn set_min_size(&mut self, bytes: u64) { self.min_size_bytes = bytes; }
    pub fn set_min_age(&mut self, days: u32) { self.min_age_days = days; }
    pub fn _add_search_path(&mut self, path: PathBuf) { self.search_paths.push(path); }

    pub fn find(&self) -> Vec<ScanResult> {
        use rayon::prelude::*;
        let results: Vec<ScanResult> = self.search_paths.par_iter()
            .filter(|p| p.exists())
            .flat_map(|p| self.scan_dir(p))
            .collect();
        let mut sorted = results;
        sorted.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        sorted
    }

    fn scan_dir(&self, dir: &Path) -> Vec<ScanResult> {
        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if metadata.is_dir() {
                        results.extend(self.scan_dir(&path));
                    } else {
                        let size = metadata.len();
                        let age = calculate_age_days(&metadata);
                        if size >= self.min_size_bytes && age >= self.min_age_days {
                            results.push(ScanResult {
                                path: path.clone(), size_bytes: size, age_days: age,
                                category: "LargeFile".to_string(), risk_level: "Moderate".to_string(),
                                description: format!("大文件: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                            });
                        }
                    }
                }
            }
        }
        results
    }
}

impl Default for LargeFileFinder { fn default() -> Self { Self::new() } }
