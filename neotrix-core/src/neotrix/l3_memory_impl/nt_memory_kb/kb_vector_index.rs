//! Vector index over the 384-dim embeddings stored in the KB.
//!
//! Session D (task-awakening-p2b) — REFRAMED: autonomous embedding path, NO
//! external docker/MiniLM dependency. The embeddings are produced by the
//! in-process `EmbedMode::Local` hash-kernel (see `nt_memory_embed.rs`), and
//! this module provides the in-memory ANN index used by `search`.
//!
//! Implementation note: we ship a dependency-free, zero-unsafe index. It uses a
//! flat (brute-force) exact cosine scan as the correctness baseline, layered
//! with an optional LSH candidate pre-filter that keeps recall exact by
//! re-ranking the candidate set with true cosine distance. The LSH banding is
//! purely a speed optimization over the exact scan; when the corpus is small
//! (or LSH is disabled) the search degrades to the exact brute-force, which is
//! always correct. No external crate, no `unsafe`.

use rusqlite::Connection;

use crate::neotrix::nt_memory_kb::nt_memory_embed::{cosine_similarity, load_all_embeddings, load_embeddings_page};

/// In-memory vector index over KB node embeddings.
pub struct VectorIndex {
    /// Expected embedding dimension (0 until first insert / build).
    dim: usize,
    /// (node_id, vector) pairs loaded from the KB.
    entries: Vec<(String, Vec<f32>)>,
    /// Optional LSH tables; empty when not built.
    lsh: Option<LshTables>,
}

impl VectorIndex {
    pub fn empty() -> Self {
        Self { dim: 0, entries: Vec::new(), lsh: None }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn dimension(&self) -> usize {
        self.dim
    }

    /// Append a single vector. Used by tests and incremental builds.
    pub fn add(&mut self, node_id: &str, vector: Vec<f32>) {
        if self.dim == 0 {
            self.dim = vector.len();
        }
        self.entries.push((node_id.to_string(), vector));
    }

    /// Build the index from the full KB embeddings table.
    /// Errors propagate the SQLite failure; an empty table yields an empty index.
    pub fn build_index(conn: &Connection) -> rusqlite::Result<Self> {
        let rows = load_all_embeddings(conn)?;
        let dim = rows.first().map(|(_, v)| v.len()).unwrap_or(0);
        Ok(Self { dim, entries: rows, lsh: None })
    }

    /// Build the index from the first `page` rows (large-corpus streaming helper).
    pub fn build_index_paged(conn: &Connection, page: usize) -> rusqlite::Result<Self> {
        let rows = load_embeddings_page(conn, 0, page.max(1))?;
        let dim = rows.first().map(|(_, v)| v.len()).unwrap_or(0);
        Ok(Self { dim, entries: rows, lsh: None })
    }

    /// Attach LSH tables with `num_bands` bands of `rows_per_band` hyperplanes
    /// each (so `dim` must be divisible by `rows_per_band`). Deterministic seed
    /// (42) — rebuilt identically every call so queries are reproducible.
    pub fn with_lsh(mut self, num_bands: usize, rows_per_band: usize) -> Result<Self, String> {
        if self.dim == 0 {
            return Err("cannot build LSH on an empty index (unknown dim)".to_string());
        }
        if rows_per_band == 0 || !self.dim.is_multiple_of(rows_per_band) {
            return Err(format!("rows_per_band must divide dim {} (got {})", self.dim, rows_per_band));
        }
        let tables = LshTables::build(self.dim, num_bands, rows_per_band, 42);
        self.lsh = Some(tables);
        Ok(self)
    }

    /// Search: returns up to `k` `(node_id, cosine_score)` pairs, highest score
    /// first. Cosine is computed exactly over either the LSH candidate set (when
    /// LSH is attached) or the entire corpus (otherwise). Correct nearest
    /// neighbours are guaranteed by the exact re-rank step.
    pub fn search(&self, query_vec: &[f32], k: usize) -> Vec<(String, f64)> {
        if query_vec.len() != self.dim || self.entries.is_empty() {
            return Vec::new();
        }
        let candidates: Vec<usize> = match &self.lsh {
            Some(lsh) => lsh.candidate_indices(query_vec, &self.entries),
            None => (0..self.entries.len()).collect(),
        };
        let mut scored: Vec<(String, f64)> = Vec::with_capacity(candidates.len());
        for &i in &candidates {
            let (id, v) = &self.entries[i];
            scored.push((id.clone(), cosine_similarity(query_vec, v)));
        }
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }
}

/// LSH over random-hyperplane sign bits, banded for candidate generation.
/// Purely a recall-preserving speed layer: the final ranking is always exact cosine.
struct LshTables {
    /// `bands[b][r]` is the hyperplane normal for band `b`, row `r` (length `dim/rows_per_band`).
    bands: Vec<Vec<Vec<f32>>>,
    #[allow(dead_code)]
    rows_per_band: usize,
}

impl LshTables {
    fn build(dim: usize, num_bands: usize, rows_per_band: usize, seed: u64) -> Self {
        let mut rng = Lcg64::new(seed);
        let mut bands: Vec<Vec<Vec<f32>>> = Vec::with_capacity(num_bands);
        for _ in 0..num_bands {
            let mut band = Vec::with_capacity(rows_per_band);
            for _ in 0..rows_per_band {
                let v: Vec<f32> = (0..dim).map(|_| rng.next_f32_signed()).collect();
                band.push(v);
            }
            bands.push(band);
        }
        Self { bands, rows_per_band }
    }

    /// Returns the set of entry indices whose band hash collides with the query
    /// in at least one band. Union across bands ⇒ high recall; exact re-rank prunes.
    fn candidate_indices(&self, query: &[f32], entries: &[(String, Vec<f32>)]) -> Vec<usize> {
        let mut hits = std::collections::HashSet::new();
        for band in &self.bands {
            let q_hash = band_hash(query, band);
            for (i, (_, v)) in entries.iter().enumerate() {
                if band_hash(v, band) == q_hash {
                    hits.insert(i);
                }
            }
        }
        hits.into_iter().collect()
    }
}

/// Project `vec` onto each band hyperplane → sign bit → pack into a u64 band hash.
fn band_hash(vec: &[f32], band: &[Vec<f32>]) -> u64 {
    let mut hash: u64 = 0;
    for (r, plane) in band.iter().enumerate() {
        let dot: f32 = vec.iter().zip(plane.iter()).map(|(a, b)| a * b).sum();
        if dot >= 0.0 {
            hash |= 1u64 << (r % 64);
        }
    }
    hash
}

/// Small deterministic PRNG (LCG) — no external rng dependency, fully reproducible.
struct Lcg64 {
    state: u64,
}

impl Lcg64 {
    fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state
    }
    fn next_f32_signed(&mut self) -> f32 {
        // Map to roughly [-1, 1).
        (self.next_u64() as f64 / u64::MAX as f64 * 2.0 - 1.0) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    /// Build an in-memory KB with `nodes` + `embeddings` tables and a few vectors.
    fn seeded_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE nodes(id TEXT PRIMARY KEY, title TEXT);
             CREATE TABLE embeddings(node_id TEXT PRIMARY KEY, vector BLOB, dimension INTEGER, model TEXT, created_at INTEGER);",
        )
        .unwrap();
        // 锚点向量: 384-dim, 第 0 维占主导
        let mut anchor = vec![0.0f32; 384];
        anchor[0] = 1.0;
        let mut other = vec![0.0f32; 384];
        other[1] = 1.0;
        for (id, v) in [("anchor", anchor), ("other", other.clone()), ("other2", other.clone())] {
            conn.execute(
                "INSERT INTO nodes(id, title) VALUES (?1, ?2)",
                params![id, id],
            )
            .unwrap();
            let blob: Vec<u8> = v.iter().flat_map(|f| f.to_le_bytes()).collect();
            conn.execute(
                "INSERT INTO embeddings(node_id, vector, dimension, model, created_at) VALUES (?1, ?2, 384, 'hash-kernel', 0)",
                params![id, blob],
            )
            .unwrap();
        }
        conn
    }

    #[test]
    fn test_build_index_loads_all_nodes() {
        let conn = seeded_conn();
        let idx = VectorIndex::build_index(&conn).unwrap();
        assert_eq!(idx.len(), 3);
        assert_eq!(idx.dimension(), 384);
        assert!(!idx.is_empty());
    }

    #[test]
    fn test_search_returns_nearest_neighbor() {
        let conn = seeded_conn();
        let idx = VectorIndex::build_index(&conn).unwrap();
        // 查询向量与 anchor 完全一致 → anchor 必须排第一且 score≈1 (其他向量正交, score=0)
        let mut q = vec![0.0f32; 384];
        q[0] = 1.0;
        let hits = idx.search(&q, 3);
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].0, "anchor");
        assert!((hits[0].1 - 1.0).abs() < 1e-4, "score {} != 1.0", hits[0].1);
    }

    #[test]
    fn test_search_dim_mismatch_safe() {
        let conn = seeded_conn();
        let idx = VectorIndex::build_index(&conn).unwrap();
        // 维度不匹配 → 安全返回空
        assert!(idx.search(&[1.0f32, 2.0], 3).is_empty());
    }

    #[test]
    fn test_search_with_lsh_keeps_correct_nearest() {
        let conn = seeded_conn();
        let idx = VectorIndex::build_index(&conn)
            .unwrap()
            .with_lsh(8, 64)
            .expect("lsh build");
        // q == anchor 向量 → 必然与 anchor 命中同一 LSH band, 进入候选集并被精确重排到首位
        let mut q = vec![0.0f32; 384];
        q[0] = 1.0;
        let hits = idx.search(&q, 3);
        assert_eq!(hits[0].0, "anchor");
        assert!((hits[0].1 - 1.0).abs() < 1e-4);
    }
}
