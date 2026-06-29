pub mod block_search;
pub mod multi_resolution;
pub mod simple_index;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub use block_search::{extract_blocks, BlockChunk, BlockType};
pub use multi_resolution::{CodeChunkIndex, FileChunkEntry, MultiResolutionIndex, ScoredFile};
pub use simple_index::{
    chunk_code, compute_lexical_score, detect_symbol_query, detect_test_file, encode_vsa,
    tokenize_code, CodeChunk, CodeIndex, CodeIndexEntry, CodeSearchResult,
};

pub(crate) const VSA_DIM: usize = 4096;
pub(crate) const CHUNK_LINES: usize = 50;
pub(crate) const CHUNK_OVERLAP: usize = 10;
const CACHE_TTL: Duration = Duration::from_secs(300);
pub(crate) const FILE_SIZE_LIMIT: u64 = 1_048_576;
pub(crate) const FILE_COHERENCE_BOOST: f64 = 0.2;
pub(crate) const DEFINITION_BOOST: f64 = 3.0;
pub(crate) const TEST_PENALTY: f64 = 0.3;
pub(crate) const COMPAT_PENALTY: f64 = 0.3;
pub(crate) const EXAMPLE_PENALTY: f64 = 0.3;
pub(crate) const SATURATION_DECAY: f64 = 0.5;

pub struct CodeSearchEngine {
    index: Option<CodeIndex>,
    root_path: PathBuf,
    last_build: Option<Instant>,
}

impl Default for CodeSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeSearchEngine {
    pub fn new() -> Self {
        Self {
            index: None,
            root_path: PathBuf::from("."),
            last_build: None,
        }
    }

    pub fn with_root(root_path: &Path) -> Self {
        Self {
            index: None,
            root_path: root_path.to_path_buf(),
            last_build: None,
        }
    }

    pub fn ensure_index(&mut self) -> Result<(), String> {
        let needs_rebuild = match &self.index {
            Some(idx) => idx.build_time.elapsed() > CACHE_TTL || idx.needs_rebuild(),
            None => true,
        };

        if needs_rebuild {
            let mut new_index = CodeIndex::new(&self.root_path);
            new_index.build()?;
            let count = new_index.entry_count();
            log::info!(
                "[CodeSearch] Index rebuilt: {} chunks from {:?}",
                count,
                self.root_path
            );
            self.index = Some(new_index);
            self.last_build = Some(Instant::now());
        }

        Ok(())
    }

    pub fn search(&mut self, query: &str, top_k: usize) -> Result<Vec<CodeSearchResult>, String> {
        self.ensure_index()?;
        match &self.index {
            Some(idx) => Ok(idx.search(query, top_k)),
            None => Err("Index not available".into()),
        }
    }

    pub fn format_results(&mut self, query: &str, top_k: usize) -> Result<String, String> {
        let results = self.search(query, top_k)?;
        if results.is_empty() {
            return Ok(format!("No code search results for: {}", query));
        }

        let mut output = format!("Code search results for \"{}\":\n\n", query);
        for (i, r) in results.iter().enumerate() {
            let path_str = if r.chunk.file_path.len() > 80 {
                format!("...{}", &r.chunk.file_path[r.chunk.file_path.len() - 77..])
            } else {
                r.chunk.file_path.clone()
            };
            output.push_str(&format!(
                "{}. {}:{}–{} (score: {:.3})\n",
                i + 1,
                path_str,
                r.chunk.start_line,
                r.chunk.end_line,
                r.score
            ));

            let preview: String = r
                .chunk
                .content
                .lines()
                .take(8)
                .collect::<Vec<&str>>()
                .join("\n");
            output.push_str(&preview);
            if r.chunk.content.lines().count() > 8 {
                output.push_str("\n...");
            }
            output.push('\n');
            if i < results.len() - 1 {
                output.push('\n');
            }
        }

        Ok(output)
    }
}

pub fn code_search(workspace_path: &str, query: &str, top_k: usize) -> Result<String, String> {
    let mut engine = CodeSearchEngine::with_root(Path::new(workspace_path));
    engine.format_results(query, top_k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_temp_workspace() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();

        let mut file1 = std::fs::File::create(root.join("main.rs")).unwrap();
        writeln!(
            file1,
            "fn main() {{\n    println!(\"Hello\");\n}}\n\nfn add(a: i32, b: i32) -> i32 {{\n    a + b\n}}"
        )
        .unwrap();

        let mut file2 = std::fs::File::create(root.join("utils.py")).unwrap();
        writeln!(
            file2,
            "def parse_config(path: str) -> dict:\n    import json\n    with open(path) as f:\n        return json.load(f)\n\ndef validate(data: dict) -> bool:\n    return 'name' in data"
        )
        .unwrap();

        (dir, root)
    }

    #[test]
    fn test_code_search_engine_format() {
        let (_dir, root) = create_temp_workspace();
        let mut engine = CodeSearchEngine::with_root(&root);
        let result = engine.format_results("parse config", 5);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Code search results"));
        assert!(output.contains("utils.py"));
    }

    #[test]
    fn test_code_search_engine_empty_query() {
        let (_dir, root) = create_temp_workspace();
        let mut engine = CodeSearchEngine::with_root(&root);
        let result = engine.format_results("", 5);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("No code search results"));
    }
}
