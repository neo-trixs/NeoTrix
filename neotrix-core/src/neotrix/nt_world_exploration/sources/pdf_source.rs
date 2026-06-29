use std::collections::HashSet;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::core::nt_core_input::pdf_extractor::PdfExtractor;
use crate::neotrix::nt_mind::knowledge_engine::{
    KnowledgeEngine, KnowledgeEntry, KnowledgeSourceType,
};
use crate::neotrix::nt_world_exploration::content::{ExplorationSourceType, SourceContent};
use crate::neotrix::nt_world_exploration::source_trait::ExplorationSource;

/// PDF文件探索源 — 扫描本地PDF文档并提取文本内容
pub struct PdfSource {
    pub scan_paths: Vec<String>,
    pub max_file_size: usize,
    pub completed: HashSet<String>,
    pub processed_hashes: HashSet<String>,
}

impl PdfSource {
    pub fn new() -> Self {
        Self {
            scan_paths: Vec::new(),
            max_file_size: 50_000_000, // 50MB default
            completed: HashSet::new(),
            processed_hashes: HashSet::new(),
        }
    }

    pub fn add_path(&mut self, path: impl Into<String>) {
        self.scan_paths.push(path.into());
    }

    fn walk_dir(
        dir: &Path,
        max_file_size: usize,
        completed: &HashSet<String>,
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
                Self::walk_dir(&path, max_file_size, completed, out, depth + 1, max_depth)?;
            } else if path.is_file() {
                let ext = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                if ext == "pdf" {
                    let path_str = path.to_string_lossy().to_string();
                    if !completed.contains(&path_str) {
                        Self::extract_pdf_content(&path, max_file_size, out)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn extract_pdf_content(
        path: &Path,
        max_file_size: usize,
        out: &mut Vec<SourceContent>,
    ) -> Result<(), String> {
        let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
        if metadata.len() > max_file_size as u64 {
            return Ok(());
        }
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let path_str = path.to_string_lossy().to_string();

        let extractor = PdfExtractor::new();
        match extractor.extract(&data) {
            Ok(doc) => {
                let text = doc
                    .pages
                    .iter()
                    .map(|p| p.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n---\n");
                let source =
                    SourceContent::new(path_str.clone(), text, ExplorationSourceType::PdfDocument)
                        .with_title(filename)
                        .with_url(path_str)
                        .with_meta("page_count", doc.page_count().to_string())
                        .with_meta("file_size", metadata.len().to_string());
                out.push(source);
                Ok(())
            }
            Err(_) => Ok(()),
        }
    }

    /// Auto-scan local directories for PDFs, extract text, check knowledge engine for gaps.
    /// Returns count of new entries seeded.
    pub fn seed_from_pdf_gaps(
        &mut self,
        knowledge_engine: &mut KnowledgeEngine,
        max_per_cycle: usize,
    ) -> u64 {
        let home = dirs::home_dir().unwrap_or_default();
        let dirs = [home.join("Papers"), home.join("Downloads")];
        let mut seeded: u64 = 0;
        let mut pdf_paths: Vec<std::path::PathBuf> = Vec::new();

        // Collect PDFs from target directories
        for dir in &dirs {
            if dir.exists() {
                Self::collect_pdf_paths(dir, &mut pdf_paths, 0, 3);
            }
        }

        // Filter already completed (previously explored)
        pdf_paths.retain(|p| {
            let s = p.to_string_lossy().to_string();
            !self.completed.contains(&s)
        });

        for path in &pdf_paths {
            if seeded >= max_per_cycle as u64 {
                break;
            }

            let path_str = path.to_string_lossy().to_string();

            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if metadata.len() > self.max_file_size as u64 {
                continue;
            }
            let mod_time = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let hash_input = format!("{}:{}", path_str, mod_time);
            let content_hash = hex::encode(Sha256::digest(hash_input.as_bytes()));

            if self.processed_hashes.contains(&content_hash) {
                continue;
            }

            // Check if already in knowledge engine (by source_url or hash tag)
            let file_url = format!("file://{}", path_str);
            let hash_tag = format!("hash:{}", content_hash);
            let exists = knowledge_engine
                .entries
                .values()
                .any(|e| e.source_url == file_url || e.tags.iter().any(|t| *t == hash_tag));
            if exists {
                self.processed_hashes.insert(content_hash);
                continue;
            }

            let data = match std::fs::read(path) {
                Ok(d) => d,
                Err(_) => continue,
            };
            let extractor = PdfExtractor::new();
            let doc = match extractor.extract(&data) {
                Ok(d) => d,
                Err(_) => {
                    self.processed_hashes.insert(content_hash);
                    continue;
                }
            };
            let text = doc
                .pages
                .iter()
                .map(|p| p.text.as_str())
                .collect::<Vec<_>>()
                .join("\n---\n");
            let truncated: String = text.chars().take(10000).collect();

            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let title = filename.trim_end_matches(".pdf").to_string();

            let entry =
                KnowledgeEntry::new(&title, &truncated, KnowledgeSourceType::PdfLocal, &file_url)
                    .with_tags(vec![
                        "pdf".to_string(),
                        "local".to_string(),
                        format!("hash:{}", content_hash),
                    ])
                    .with_dimensions(vec![
                        format!("file_size:{}", metadata.len()),
                        format!("mod_time:{}", mod_time),
                    ])
                    .with_confidence(0.65);

            knowledge_engine.add_entry(entry);
            self.completed.insert(path_str);
            self.processed_hashes.insert(content_hash);
            seeded += 1;
        }

        seeded
    }

    fn collect_pdf_paths(
        dir: &Path,
        out: &mut Vec<std::path::PathBuf>,
        depth: usize,
        max_depth: usize,
    ) {
        if depth > max_depth {
            return;
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::collect_pdf_paths(&path, out, depth + 1, max_depth);
            } else if path.is_file() {
                let ext = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                if ext == "pdf" {
                    out.push(path);
                }
            }
        }
    }
}

impl ExplorationSource for PdfSource {
    fn name(&self) -> &'static str {
        "pdf_source"
    }

    fn confidence(&self) -> f64 {
        0.75
    }

    fn pending_count(&self) -> usize {
        self.scan_paths.len()
    }

    fn explore(&mut self) -> Result<Vec<SourceContent>, String> {
        let mut all = Vec::new();
        for path_str in &self.scan_paths {
            let path = Path::new(path_str);
            if path.is_dir() {
                Self::walk_dir(path, self.max_file_size, &self.completed, &mut all, 0, 3)?;
            } else if path.is_file() {
                let ext = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                if ext == "pdf" && !self.completed.contains(path_str) {
                    Self::extract_pdf_content(path, self.max_file_size, &mut all)?;
                }
            }
        }
        for item in &all {
            if let Some(ref url) = item.url {
                self.completed.insert(url.clone());
            }
        }
        Ok(all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_input::extract_text_from_pdf;

    fn create_minimal_pdf(text: &str) -> Vec<u8> {
        let content = format!(
            "%PDF-1.4\n1 0 obj\n<< /Length {len} >>\nstream\nBT\n/F1 12 Tf\n({text}) Tj\nET\nendstream\nendobj\n%%EOF\n",
            len = text.len() + 40
        );
        content.into_bytes()
    }

    #[test]
    fn test_extract_from_simple_pdf_bytes() {
        let pdf_bytes = create_minimal_pdf("Hello from PDF");
        let result = extract_text_from_pdf(&pdf_bytes).unwrap();
        assert!(result.contains("Hello from PDF"));
    }

    #[test]
    fn test_extract_pdf_content_via_source() {
        let dir = tempfile::tempdir().unwrap();
        let pdf_path = dir.path().join("test.pdf");
        let pdf_bytes = create_minimal_pdf("PDF content test");
        std::fs::write(&pdf_path, &pdf_bytes).unwrap();

        let mut results = Vec::new();
        PdfSource::extract_pdf_content(&pdf_path, 50_000_000, &mut results).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].text.contains("PDF content test"));
        assert_eq!(results[0].title, "test.pdf");
        assert_eq!(results[0].source_type, ExplorationSourceType::PdfDocument);
    }

    #[test]
    fn test_skip_non_pdf_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("readme.txt"), b"not a pdf").unwrap();
        std::fs::write(dir.path().join("doc.pdf"), &create_minimal_pdf("real pdf")).unwrap();

        let mut src = PdfSource::new();
        src.add_path(dir.path().to_string_lossy().to_string());
        let results = src.explore().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "doc.pdf");
    }

    #[test]
    fn test_skip_large_file() {
        let dir = tempfile::tempdir().unwrap();
        let pdf_path = dir.path().join("large.pdf");
        let pdf_bytes = create_minimal_pdf("small content");
        std::fs::write(&pdf_path, &pdf_bytes).unwrap();

        let mut results = Vec::new();
        PdfSource::extract_pdf_content(&pdf_path, 1, &mut results).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_completed_dedup() {
        let dir = tempfile::tempdir().unwrap();
        let pdf_path = dir.path().join("dedup.pdf");
        let pdf_bytes = create_minimal_pdf("dedup test");
        std::fs::write(&pdf_path, &pdf_bytes).unwrap();
        let path_str = pdf_path.to_string_lossy().to_string();

        let mut src = PdfSource::new();
        src.completed.insert(path_str.clone());
        src.add_path(path_str);
        let results = src.explore().unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_explore_empty() {
        let mut src = PdfSource::new();
        let results = src.explore().unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_confidence() {
        let src = PdfSource::new();
        assert!((src.confidence() - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_walk_finds_pdfs_case_insensitive_extension() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("doc.PDF"), &create_minimal_pdf("upper ext")).unwrap();
        std::fs::write(
            dir.path().join("paper.pdf"),
            &create_minimal_pdf("lower ext"),
        )
        .unwrap();
        std::fs::write(dir.path().join("notes.txt"), b"skip me").unwrap();

        let mut src = PdfSource::new();
        src.add_path(dir.path().to_string_lossy().to_string());
        let results = src.explore().unwrap();
        assert_eq!(results.len(), 2);
    }
}
