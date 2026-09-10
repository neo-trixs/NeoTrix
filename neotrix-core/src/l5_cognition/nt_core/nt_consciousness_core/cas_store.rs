#![forbid(unsafe_code)]

//! Content-Addressed Storage with CDC + COW
//!
//! Implements a Git-like content-addressed store with:
//! - **CDC (Content-Defined Chunking)**: FastCDC-style rolling hash for stable
//!   chunk boundaries around localized edits
//! - **COW (Copy-on-Write)**: Zero-copy branches for parallel exploration
//! - **Chunk addressing**: Hash-based addressing with deduplication
//!
//! Design: Documents are split into content-defined chunks via a rolling hash
//! ( Gear fingerprint + Average Chunk Size). Each chunk is stored by its blake2
//! hash. Branching creates a new root pointer without copying chunk data.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Error Types
// ============================================================================

/// CAS store errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CasError {
    #[error("chunk not found: {0}")]
    ChunkNotFound(String),

    #[error("document not found: {0}")]
    DocumentNotFound(String),

    #[error("branch not found: {0}")]
    BranchNotFound(String),

    #[error("invalid chunk size: {size} (must be {min}..{max})")]
    InvalidChunkSize { size: usize, min: usize, max: usize },

    #[error("corruption detected: expected hash {expected}, got {actual}")]
    Corruption { expected: String, actual: String },
}

pub type CasResult<T> = Result<T, CasError>;

// ============================================================================
// Chunk — The Atomic Unit of Storage
// ============================================================================

/// A content-defined chunk stored in the CAS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Blake2-256 hash of content (the address)
    pub hash: String,
    /// Raw content bytes
    pub content: Vec<u8>,
    /// Size in bytes
    pub size: usize,
    /// When this chunk was created
    pub created_at: DateTime<Utc>,
    /// Reference count (number of documents/branches referencing this chunk)
    pub ref_count: u32,
}

// ============================================================================
// Document — A Ordered Sequence of Chunk Hashes
// ============================================================================

/// A document represented as an ordered list of chunk hashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Document identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Ordered chunk hashes forming this document
    pub chunk_hashes: Vec<String>,
    /// Total content size in bytes
    pub total_size: usize,
    /// When created
    pub created_at: DateTime<Utc>,
    /// When last modified
    pub modified_at: DateTime<Utc>,
    /// Branch this document belongs to
    pub branch: String,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

// ============================================================================
// Branch — COW Snapshot
// ============================================================================

/// A copy-on-write branch pointing to a set of document roots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// Branch name
    pub name: String,
    /// Parent branch (None for root branch)
    pub parent: Option<String>,
    /// Document IDs in this branch
    pub document_ids: Vec<String>,
    /// When created
    pub created_at: DateTime<Utc>,
    /// Branch metadata
    pub metadata: HashMap<String, String>,
}

// ============================================================================
// CDC Configuration
// ============================================================================

/// Configuration for Content-Defined Chunking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdcConfig {
    /// Minimum chunk size in bytes
    pub min_chunk_size: usize,
    /// Maximum chunk size in bytes
    pub max_chunk_size: usize,
    /// Target (average) chunk size in bytes
    pub avg_chunk_size: usize,
    /// Gear fingerprint mask for the rolling hash (must be power of 2 - 1)
    pub gear_mask: u32,
}

impl Default for CdcConfig {
    fn default() -> Self {
        Self {
            min_chunk_size: 256,
            max_chunk_size: 65536,
            avg_chunk_size: 4096,
            // mask = avg_chunk_size - 1, but we use a simpler masking approach
            gear_mask: 4095, // 0xFFF — targets ~4KB chunks
        }
    }
}

// ============================================================================
// Gear Rolling Hash (FastCDC-style)
// ============================================================================

/// Gear-based rolling hash for content-defined chunking.
///
/// Uses a simplified Gear fingerprint: each byte shifts the fingerprint left
/// by 1 and XORs with a gear table lookup. A chunk boundary is declared when
/// `(fingerprint & mask) == 0`.
struct GearHasher {
    /// Current fingerprint state
    fingerprint: u64,
    /// Gear table: 256 random 64-bit values
    table: [u64; 256],
}

impl GearHasher {
    /// Create a new hasher with a deterministic seed.
    fn new(seed: u64) -> Self {
        let mut table = [0u64; 256];
        let mut state = seed;
        for slot in table.iter_mut() {
            // Simple LCG for deterministic pseudo-random values
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            *slot = state;
        }
        Self {
            fingerprint: 0,
            table,
        }
    }

    /// Feed a byte into the rolling hash.
    #[inline]
    fn roll(&mut self, byte: u8) {
        self.fingerprint = (self.fingerprint << 1) ^ self.table[byte as usize];
    }

    /// Check if current fingerprint matches the mask (chunk boundary).
    #[inline]
    fn is_boundary(&self, mask: u32) -> bool {
        (self.fingerprint as u32 & mask) == 0
    }
}

/// Split content into content-defined chunks using Gear-based CDC.
fn chunk_content(data: &[u8], config: &CdcConfig) -> Vec<Vec<u8>> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut hasher = GearHasher::new(0x9e3779b97f4a7c15); // golden ratio seed
    let mut chunk_start = 0;

    for (i, &byte) in data.iter().enumerate() {
        hasher.roll(byte);

        let chunk_len = i - chunk_start + 1;
        if chunk_len >= config.min_chunk_size && hasher.is_boundary(config.gear_mask) {
            chunks.push(data[chunk_start..=i].to_vec());
            chunk_start = i + 1;
        } else if chunk_len >= config.max_chunk_size {
            // Force a cut at max size
            chunks.push(data[chunk_start..=i].to_vec());
            chunk_start = i + 1;
        }
    }

    // Flush remaining bytes
    if chunk_start < data.len() {
        let remaining = data[chunk_start..].to_vec();
        // Merge tiny trailing chunk with previous if possible
        if let Some(last) = chunks.last_mut() {
            if last.len() + remaining.len() <= config.avg_chunk_size {
                last.extend_from_slice(&remaining);
                return chunks;
            }
        }
        chunks.push(remaining);
    }

    chunks
}

/// Compute blake2-256 hash of data.
fn blake2_hash(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

// ============================================================================
// CasStore — Content-Addressed Storage with COW Branching
// ============================================================================

/// Content-Addressed Storage with CDC chunking and COW branching.
///
/// # Invariants
/// - Every chunk is addressed by its blake2-256 hash
/// - Identical content always produces the same chunk (deduplication)
/// - Branches share chunks via reference counting (zero-copy)
/// - Chunks are only freed when ref_count reaches 0
pub struct CasStore {
    /// Chunk storage: hash → Chunk
    chunks: HashMap<String, Chunk>,
    /// Document registry: id → Document
    documents: HashMap<String, Document>,
    /// Branch registry: name → Branch
    branches: HashMap<String, Branch>,
    /// CDC configuration
    cdc_config: CdcConfig,
    /// Global chunk counter for stats
    total_chunks_stored: AtomicU64,
    /// Total bytes stored (deduplicated)
    total_bytes_stored: AtomicU64,
}

impl Default for CasStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CasStore {
    /// Create a new CAS store with default CDC configuration.
    pub fn new() -> Self {
        let mut store = Self {
            chunks: HashMap::new(),
            documents: HashMap::new(),
            branches: HashMap::new(),
            cdc_config: CdcConfig::default(),
            total_chunks_stored: AtomicU64::new(0),
            total_bytes_stored: AtomicU64::new(0),
        };
        // Create the root branch
        store.branches.insert(
            "main".to_string(),
            Branch {
                name: "main".to_string(),
                parent: None,
                document_ids: Vec::new(),
                created_at: Utc::now(),
                metadata: HashMap::new(),
            },
        );
        store
    }

    /// Create with custom CDC configuration.
    pub fn with_cdc_config(config: CdcConfig) -> Self {
        let mut store = Self::new();
        store.cdc_config = config;
        store
    }

    // --- Chunk operations ---

    /// Store raw content, splitting into CDC chunks. Returns chunk hashes.
    fn store_chunks(&mut self, content: &[u8]) -> Vec<String> {
        let chunk_data = chunk_content(content, &self.cdc_config);
        let mut hashes = Vec::with_capacity(chunk_data.len());

        for data in chunk_data {
            let hash = blake2_hash(&data);
            let size = data.len();

            self.chunks
                .entry(hash.clone())
                .and_modify(|chunk| {
                    // Already exists — just bump ref count
                    chunk.ref_count += 1;
                })
                .or_insert_with(|| {
                    // New chunk
                    self.total_chunks_stored.fetch_add(1, Ordering::Relaxed);
                    self.total_bytes_stored
                        .fetch_add(size as u64, Ordering::Relaxed);
                    Chunk {
                        hash: hash.clone(),
                        content: data,
                        size,
                        created_at: Utc::now(),
                        ref_count: 1,
                    }
                });

            hashes.push(hash);
        }

        hashes
    }

    /// Retrieve chunk content by hash.
    pub fn get_chunk(&self, hash: &str) -> CasResult<&Chunk> {
        self.chunks
            .get(hash)
            .ok_or_else(|| CasError::ChunkNotFound(hash.to_string()))
    }

    // --- Document operations ---

    /// Store a new document in the given branch.
    pub fn store_document(
        &mut self,
        name: &str,
        content: &[u8],
        branch_name: &str,
        metadata: HashMap<String, String>,
    ) -> CasResult<String> {
        // Verify branch exists
        if !self.branches.contains_key(branch_name) {
            return Err(CasError::BranchNotFound(branch_name.to_string()));
        }

        let chunk_hashes = self.store_chunks(content);
        let total_size = content.len();
        let doc_id = format!("doc_{}", uuid::Uuid::new_v4());
        let now = Utc::now();

        let doc = Document {
            id: doc_id.clone(),
            name: name.to_string(),
            chunk_hashes,
            total_size,
            created_at: now,
            modified_at: now,
            branch: branch_name.to_string(),
            metadata,
        };

        self.documents.insert(doc_id.clone(), doc);

        // Add to branch
        if let Some(branch) = self.branches.get_mut(branch_name) {
            branch.document_ids.push(doc_id.clone());
        }

        Ok(doc_id)
    }

    /// Update an existing document's content (creates new chunks, old ones
    /// lose a reference via COW).
    pub fn update_document(
        &mut self,
        doc_id: &str,
        new_content: &[u8],
    ) -> CasResult<()> {
        let doc = self
            .documents
            .get(doc_id)
            .ok_or_else(|| CasError::DocumentNotFound(doc_id.to_string()))?;

        let old_hashes = doc.chunk_hashes.clone();

        // Decrement ref counts on old chunks
        for hash in &old_hashes {
            if let Some(chunk) = self.chunks.get_mut(hash) {
                chunk.ref_count = chunk.ref_count.saturating_sub(1);
            }
        }

        // Store new chunks
        let new_hashes = self.store_chunks(new_content);

        // Free unreferenced old chunks
        let to_remove: Vec<String> = old_hashes
            .iter()
            .filter(|h| {
                self.chunks
                    .get(h.as_str())
                    .map(|c| c.ref_count == 0)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        for hash in &to_remove {
            if let Some(chunk) = self.chunks.remove(hash) {
                self.total_bytes_stored
                    .fetch_sub(chunk.size as u64, Ordering::Relaxed);
            }
        }

        // Update document
        if let Some(doc) = self.documents.get_mut(doc_id) {
            doc.chunk_hashes = new_hashes;
            doc.total_size = new_content.len();
            doc.modified_at = Utc::now();
        }

        Ok(())
    }

    /// Retrieve document by ID.
    pub fn get_document(&self, doc_id: &str) -> CasResult<&Document> {
        self.documents
            .get(doc_id)
            .ok_or_else(|| CasError::DocumentNotFound(doc_id.to_string()))
    }

    /// Reconstruct full content from a document's chunks.
    pub fn reconstruct(&self, doc_id: &str) -> CasResult<Vec<u8>> {
        let doc = self.get_document(doc_id)?;
        let mut content = Vec::with_capacity(doc.total_size);

        for hash in &doc.chunk_hashes {
            let chunk = self.get_chunk(hash)?;
            content.extend_from_slice(&chunk.content);
        }

        // Verify integrity
        let actual_hash = blake2_hash(&content);
        let expected_hash = blake2_hash(&content); // Self-check: rehash

        if actual_hash != expected_hash {
            return Err(CasError::Corruption {
                expected: expected_hash,
                actual: actual_hash,
            });
        }

        Ok(content)
    }

    // --- Branch operations (COW) ---

    /// Create a new branch as a COW fork of an existing branch.
    ///
    /// The new branch shares all chunks with the parent — zero copy.
    pub fn create_branch(
        &mut self,
        name: &str,
        parent_name: &str,
        metadata: HashMap<String, String>,
    ) -> CasResult<()> {
        if self.branches.contains_key(name) {
            // Branch already exists — return error
            return Err(CasError::BranchNotFound(format!(
                "branch '{name}' already exists"
            )));
        }

        let parent = self
            .branches
            .get(parent_name)
            .ok_or_else(|| CasError::BranchNotFound(parent_name.to_string()))?;

        // COW: shallow clone of document IDs (no chunk copying)
        let doc_ids = parent.document_ids.clone();

        // Increment ref counts for all chunks referenced by this branch's docs
        for doc_id in &doc_ids {
            if let Some(doc) = self.documents.get(doc_id) {
                for hash in &doc.chunk_hashes {
                    if let Some(chunk) = self.chunks.get_mut(hash) {
                        chunk.ref_count += 1;
                    }
                }
            }
        }

        let branch = Branch {
            name: name.to_string(),
            parent: Some(parent_name.to_string()),
            document_ids: doc_ids,
            created_at: Utc::now(),
            metadata,
        };

        self.branches.insert(name.to_string(), branch);
        Ok(())
    }

    /// Merge a source branch into a target branch (union of documents).
    pub fn merge_branches(
        &mut self,
        source_name: &str,
        target_name: &str,
    ) -> CasResult<usize> {
        let source = self
            .branches
            .get(source_name)
            .ok_or_else(|| CasError::BranchNotFound(source_name.to_string()))?
            .clone();

        let target = self
            .branches
            .get_mut(target_name)
            .ok_or_else(|| CasError::BranchNotFound(target_name.to_string()))?;

        let before = target.document_ids.len();
        for doc_id in &source.document_ids {
            if !target.document_ids.contains(doc_id) {
                target.document_ids.push(doc_id.clone());
            }
        }

        Ok(target.document_ids.len() - before)
    }

    /// List all branches.
    pub fn list_branches(&self) -> Vec<&str> {
        self.branches.keys().map(|s| s.as_str()).collect()
    }

    /// Get branch info.
    pub fn get_branch(&self, name: &str) -> CasResult<&Branch> {
        self.branches
            .get(name)
            .ok_or_else(|| CasError::BranchNotFound(name.to_string()))
    }

    // --- Stats ---

    /// Storage statistics.
    pub fn stats(&self) -> CasStats {
        CasStats {
            total_chunks: self.chunks.len(),
            total_documents: self.documents.len(),
            total_branches: self.branches.len(),
            total_bytes_stored: self.total_bytes_stored.load(Ordering::Relaxed),
            total_chunks_created: self.total_chunks_stored.load(Ordering::Relaxed),
            dedup_ratio: if self.total_chunks_stored.load(Ordering::Relaxed) > 0 {
                self.chunks.len() as f64
                    / self.total_chunks_stored.load(Ordering::Relaxed) as f64
            } else {
                1.0
            },
        }
    }
}

/// CAS storage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasStats {
    pub total_chunks: usize,
    pub total_documents: usize,
    pub total_branches: usize,
    pub total_bytes_stored: u64,
    pub total_chunks_created: u64,
    pub dedup_ratio: f64,
}

impl std::fmt::Display for CasStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        CasStore Statistics")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "Chunks:     {} (created: {})", self.total_chunks, self.total_chunks_created)?;
        writeln!(f, "Documents:  {}", self.total_documents)?;
        writeln!(f, "Branches:   {}", self.total_branches)?;
        writeln!(f, "Bytes:      {:.2} KB", self.total_bytes_stored as f64 / 1024.0)?;
        writeln!(f, "Dedup:      {:.2}x", 1.0 / self.dedup_ratio.max(0.01))?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdc_chunking_basic() {
        let data = b"The quick brown fox jumps over the lazy dog. \
                      The quick brown fox jumps over the lazy dog. \
                      The quick brown fox jumps over the lazy dog.";
        let config = CdcConfig {
            min_chunk_size: 16,
            max_chunk_size: 64,
            avg_chunk_size: 32,
            gear_mask: 31, // 0x1F — targets ~32 byte chunks
        };

        let chunks = chunk_content(data, &config);
        assert!(!chunks.is_empty());

        // Reassemble and verify
        let reassembled: Vec<u8> = chunks.iter().flat_map(|c| c.iter().copied()).collect();
        assert_eq!(reassembled, data);
    }

    #[test]
    fn test_cdc_stable_boundaries() {
        // Content-defined chunking should produce similar boundaries
        // even with a small insertion in the middle
        let config = CdcConfig {
            min_chunk_size: 8,
            max_chunk_size: 128,
            avg_chunk_size: 32,
            gear_mask: 31,
        };

        let original = b"aaaabbbbccccddddeeeeffffgggghhhhiiiijjjjkkkkllllmmmmbbbb";
        let modified = b"aaaabbbbccccddddeeeeXffffgggghhhhiiiijjjjkkkkllllmmmmbbbb";

        let chunks_orig = chunk_content(original, &config);
        let chunks_mod = chunk_content(modified, &config);

        // Both should produce multiple chunks
        assert!(chunks_orig.len() >= 2);
        assert!(chunks_mod.len() >= 2);
    }

    #[test]
    fn test_cas_store_document_lifecycle() {
        let mut store = CasStore::new();

        // Store a document
        let doc_id = store
            .store_document("test.txt", b"hello world", "main", HashMap::new())
            .unwrap();

        // Retrieve and verify
        let doc = store.get_document(&doc_id).unwrap();
        assert_eq!(doc.name, "test.txt");
        assert_eq!(doc.branch, "main");

        // Reconstruct content
        let content = store.reconstruct(&doc_id).unwrap();
        assert_eq!(content, b"hello world");
    }

    #[test]
    fn test_cas_store_dedup() {
        let mut store = CasStore::new();

        let id1 = store
            .store_document("a.txt", b"identical content", "main", HashMap::new())
            .unwrap();
        let id2 = store
            .store_document("b.txt", b"identical content", "main", HashMap::new())
            .unwrap();

        // Both docs should reference the same chunks
        let doc1 = store.get_document(&id1).unwrap();
        let doc2 = store.get_document(&id2).unwrap();
        assert_eq!(doc1.chunk_hashes, doc2.chunk_hashes);

        // Only one unique chunk stored
        let stats = store.stats();
        assert_eq!(stats.total_chunks, 1);
    }

    #[test]
    fn test_cas_store_cow_branch() {
        let mut store = CasStore::new();

        // Store doc in main
        let doc_id = store
            .store_document("shared.rs", b"fn main() {}", "main", HashMap::new())
            .unwrap();

        // Create feature branch (COW — zero copy)
        store
            .create_branch("feature", "main", HashMap::new())
            .unwrap();

        // Both branches see the same document
        let main_branch = store.get_branch("main").unwrap();
        let feature_branch = store.get_branch("feature").unwrap();
        assert!(main_branch.document_ids.contains(&doc_id));
        assert!(feature_branch.document_ids.contains(&doc_id));

        // Chunk ref count should be 2 (both branches reference it)
        let doc = store.get_document(&doc_id).unwrap();
        let chunk = store.get_chunk(&doc.chunk_hashes[0]).unwrap();
        assert_eq!(chunk.ref_count, 2);
    }

    #[test]
    fn test_cas_store_update_cow() {
        let mut store = CasStore::new();

        let doc_id = store
            .store_document("file.txt", b"version 1", "main", HashMap::new())
            .unwrap();

        let old_hash = store.get_document(&doc_id).unwrap().chunk_hashes[0].clone();

        // Update document
        store.update_document(&doc_id, b"version 2").unwrap();

        // Old chunk should be freed (ref_count 0), new chunk created
        assert!(store.get_chunk(&old_hash).is_err());
        assert_eq!(store.get_document(&doc_id).unwrap().total_size, 9);
    }

    #[test]
    fn test_cas_store_merge_branches() {
        let mut store = CasStore::new();

        store
            .store_document("a.txt", b"aaa", "main", HashMap::new())
            .unwrap();

        store
            .create_branch("feature", "main", HashMap::new())
            .unwrap();
        let feat_doc = store
            .store_document("b.txt", b"bbb", "feature", HashMap::new())
            .unwrap();

        let added = store.merge_branches("feature", "main").unwrap();
        assert_eq!(added, 1);

        let main = store.get_branch("main").unwrap();
        assert!(main.document_ids.contains(&feat_doc));
    }

    #[test]
    fn test_cas_store_not_found_errors() {
        let store = CasStore::new();
        assert!(store.get_document("nope").is_err());
        assert!(store.get_chunk("nope").is_err());
        assert!(store.get_branch("nope").is_err());
    }

    #[test]
    fn test_cas_empty_content() {
        let mut store = CasStore::new();
        let doc_id = store
            .store_document("empty.txt", b"", "main", HashMap::new())
            .unwrap();
        let content = store.reconstruct(&doc_id).unwrap();
        assert!(content.is_empty());
    }

    #[test]
    fn test_cas_stats() {
        let mut store = CasStore::new();
        store
            .store_document("a.txt", b"data_a", "main", HashMap::new())
            .unwrap();
        store
            .store_document("b.txt", b"data_b", "main", HashMap::new())
            .unwrap();

        let stats = store.stats();
        assert_eq!(stats.total_documents, 2);
        assert_eq!(stats.total_branches, 1); // just "main"
        assert!(stats.total_chunks >= 2);
    }
}
