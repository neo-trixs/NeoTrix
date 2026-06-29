use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

use super::block_search::{extract_blocks, BlockChunk};
use super::simple_index::{compute_lexical_score, detect_symbol_query, encode_vsa, tokenize_code};
use super::VSA_DIM;

#[derive(Debug, Clone)]
pub struct FileChunkEntry {
    pub file_path: PathBuf,
    pub vsa: Vec<u8>,
    pub tokens: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CodeChunkIndex {
    pub chunks: Vec<BlockChunk>,
    pub file_path: PathBuf,
    pub vsa: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ScoredFile {
    pub file_path: PathBuf,
    pub score: f64,
    pub matching_chunks: Vec<BlockChunk>,
}

fn encode_block_vsa(blocks: &[BlockChunk]) -> Vec<u8> {
    let tokens: Vec<String> = blocks.iter().map(|b| b.name.clone()).collect();
    if tokens.is_empty() {
        return vec![0u8; VSA_DIM];
    }
    encode_vsa(&tokens)
}

pub struct MultiResolutionIndex {
    pub file_chunks: Vec<FileChunkEntry>,
    pub function_chunks: Vec<CodeChunkIndex>,
    root_path: PathBuf,
}

impl MultiResolutionIndex {
    pub fn new(root_path: &Path) -> Self {
        Self {
            file_chunks: Vec::new(),
            function_chunks: Vec::new(),
            root_path: root_path.to_path_buf(),
        }
    }

    pub fn index_file(&mut self, path: &Path, source: &str) -> Result<(), String> {
        let rel = path.strip_prefix(&self.root_path).unwrap_or(path);

        let tokens = tokenize_code(source);
        let vsa = encode_vsa(&tokens);
        self.file_chunks.push(FileChunkEntry {
            file_path: rel.to_path_buf(),
            vsa,
            tokens,
        });

        let blocks = extract_blocks(source);
        if !blocks.is_empty() {
            let block_vsa = encode_block_vsa(&blocks);
            self.function_chunks.push(CodeChunkIndex {
                chunks: blocks,
                file_path: rel.to_path_buf(),
                vsa: block_vsa,
            });
        }

        Ok(())
    }

    pub fn search(&self, query: &str, top_k: usize) -> Vec<ScoredFile> {
        if self.file_chunks.is_empty() || query.trim().is_empty() {
            return Vec::new();
        }

        let query_tokens = tokenize_code(query);
        if query_tokens.is_empty() {
            return Vec::new();
        }

        let query_vector = encode_vsa(&query_tokens);
        let is_symbol_query = detect_symbol_query(&query_tokens);
        let top_k = top_k.max(1);

        let file_scores: Vec<(f64, usize)> = self
            .file_chunks
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let vsa_sim = QuantizedVSA::similarity(&query_vector, &entry.vsa);
                let lexical = compute_lexical_score(&query_tokens, &entry.tokens);
                let alpha = if is_symbol_query { 0.3 } else { 0.5 };
                let score = alpha * vsa_sim + (1.0 - alpha) * lexical;
                (score, idx)
            })
            .filter(|(s, _)| *s > 0.005)
            .collect();

        let mut func_hits: HashMap<PathBuf, Vec<BlockChunk>> = HashMap::new();
        for func_idx in &self.function_chunks {
            let mut matching: Vec<BlockChunk> = Vec::new();
            for block in &func_idx.chunks {
                let block_tokens = tokenize_code(&block.source);
                let block_vsa = encode_vsa(&block_tokens);
                let sim = QuantizedVSA::similarity(&query_vector, &block_vsa);
                let lex = compute_lexical_score(&query_tokens, &block_tokens);
                if sim > 0.15 || lex > 0.0 {
                    matching.push(block.clone());
                }
            }
            if !matching.is_empty() {
                func_hits
                    .entry(func_idx.file_path.clone())
                    .or_default()
                    .extend(matching);
            }
        }

        let mut merged: Vec<ScoredFile> = file_scores
            .into_iter()
            .filter_map(|(score, idx)| {
                let entry = &self.file_chunks[idx];
                let bonus = if func_hits.contains_key(&entry.file_path) {
                    0.2
                } else {
                    0.0
                };
                let chunks = func_hits.remove(&entry.file_path).unwrap_or_default();
                Some(ScoredFile {
                    file_path: entry.file_path.clone(),
                    score: score + bonus,
                    matching_chunks: chunks,
                })
            })
            .collect();

        for (path, chunks) in func_hits.drain() {
            merged.push(ScoredFile {
                file_path: path,
                score: 0.5,
                matching_chunks: chunks,
            });
        }

        merged.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        merged.truncate(top_k);
        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_resolution_index_search() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let mut mri = MultiResolutionIndex::new(&root);

        let file1 = root.join("math.rs");
        std::fs::write(
            &file1,
            "fn add(a: i32, b: i32) -> i32 { a + b }\nfn multiply(a: i32, b: i32) -> i32 { a * b }",
        )
        .unwrap();

        let file2 = root.join("strings.rs");
        std::fs::write(&file2, "fn to_upper(s: &str) -> String { s.to_uppercase() }\nfn to_lower(s: &str) -> String { s.to_lowercase() }").unwrap();

        mri.index_file(&file1, &std::fs::read_to_string(&file1).unwrap())
            .unwrap();
        mri.index_file(&file2, &std::fs::read_to_string(&file2).unwrap())
            .unwrap();

        assert_eq!(mri.file_chunks.len(), 2);
        assert_eq!(mri.function_chunks.len(), 2);

        let results = mri.search("multiply numbers", 5);
        assert!(!results.is_empty());
        let top = &results[0];
        assert!(top.score > 0.0);
    }

    #[test]
    fn test_multi_resolution_function_bonus() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let mut mri = MultiResolutionIndex::new(&root);

        let file1 = root.join("auth.rs");
        std::fs::write(
            &file1,
            "fn parse_token(s: &str) -> bool { true }\nfn validate_token(t: &str) -> bool { true }",
        )
        .unwrap();

        let file2 = root.join("utils.rs");
        std::fs::write(
            &file2,
            "mod helpers {\n    pub fn do_thing() {}\n}\nfn utility() {}",
        )
        .unwrap();

        mri.index_file(&file1, &std::fs::read_to_string(&file1).unwrap())
            .unwrap();
        mri.index_file(&file2, &std::fs::read_to_string(&file2).unwrap())
            .unwrap();

        let results = mri.search("parse_token validate", 5);
        assert!(!results.is_empty());
        let top_file = results
            .iter()
            .find(|r| r.file_path.ends_with("auth.rs"))
            .unwrap();
        assert!(
            top_file.score > 0.3,
            "auth.rs should have good score for token query"
        );
    }

    #[test]
    fn test_multi_resolution_empty_search() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let mri = MultiResolutionIndex::new(&root);
        let results = mri.search("anything", 5);
        assert!(results.is_empty());
    }
}
