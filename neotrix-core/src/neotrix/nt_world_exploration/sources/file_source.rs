use std::path::Path;

use crate::neotrix::nt_world_exploration::content::{ExplorationSourceType, SourceContent};
use crate::neotrix::nt_world_exploration::source_trait::ExplorationSource;

/// 本地文件探索源 — 扫描文件系统中的文档/代码
pub struct FileSource {
    paths: Vec<String>,
    extensions: Vec<String>,
}

impl FileSource {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            extensions: vec![
                "md".into(),
                "txt".into(),
                "rs".into(),
                "py".into(),
                "js".into(),
            ],
        }
    }

    pub fn add_path(&mut self, path: impl Into<String>) {
        self.paths.push(path.into());
    }

    fn walk_dir(
        dir: &Path,
        exts: &[String],
        out: &mut Vec<SourceContent>,
        depth: usize,
        max_depth: usize,
    ) -> Result<(), String> {
        if depth > max_depth {
            return Ok(());
        }
        let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                Self::walk_dir(&path, exts, out, depth + 1, max_depth)?;
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    if exts.iter().any(|e| e == &ext.to_string_lossy().to_string()) {
                        Self::read_file_content(&path, out)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn read_file_content(path: &Path, out: &mut Vec<SourceContent>) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let path_str = path.to_string_lossy().to_string();
        if content.len() > 10_000 {
            let truncated = content.chars().take(10_000).collect::<String>();
            out.push(
                SourceContent::new(path_str, truncated, ExplorationSourceType::FileSystem)
                    .with_title(filename),
            );
        } else {
            out.push(
                SourceContent::new(path_str, content, ExplorationSourceType::FileSystem)
                    .with_title(filename),
            );
        }
        Ok(())
    }
}

impl ExplorationSource for FileSource {
    fn name(&self) -> &'static str {
        "file_system"
    }

    fn confidence(&self) -> f64 {
        0.5
    }

    fn explore(&mut self) -> Result<Vec<SourceContent>, String> {
        let mut all = Vec::new();
        for path_str in &self.paths {
            let path = Path::new(path_str);
            if path.is_dir() {
                Self::walk_dir(path, &self.extensions, &mut all, 0, 3)?;
            } else if path.is_file() {
                Self::read_file_content(path, &mut all)?;
            }
        }
        Ok(all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_read_file_content() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "hello world").unwrap();

        let mut results = Vec::new();
        FileSource::read_file_content(&file_path, &mut results).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "hello world");
        assert_eq!(results[0].title, "test.txt");
    }

    #[test]
    fn test_walk_dir_finds_matching_extensions() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();
        std::fs::write(dir.path().join("b.py"), "print('hello')").unwrap();
        std::fs::write(dir.path().join("c.txt"), "content").unwrap();
        std::fs::write(dir.path().join("d.jpg"), "not a text file").unwrap();

        let exts = vec!["rs".into(), "py".into(), "txt".into()];
        let mut results = Vec::new();
        FileSource::walk_dir(dir.path(), &exts, &mut results, 0, 3).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_truncates_large_files() {
        let dir = tempdir().unwrap();
        let large = "x".repeat(15_000);
        let file_path = dir.path().join("large.txt");
        std::fs::write(&file_path, &large).unwrap();

        let mut results = Vec::new();
        FileSource::read_file_content(&file_path, &mut results).unwrap();
        assert_eq!(results[0].text.len(), 10_000);
    }

    #[test]
    fn test_explore_empty() {
        let mut src = FileSource::new();
        let results = src.explore().unwrap();
        assert!(results.is_empty());
    }
}
