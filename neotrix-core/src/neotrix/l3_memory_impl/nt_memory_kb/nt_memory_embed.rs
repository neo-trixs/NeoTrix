use serde::Deserialize;
use rusqlite::{params, Connection};
use std::sync::OnceLock;

/// Configuration for the embedding API (OpenAI-compatible, incl. MiniLM local server).
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub dimension: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        // Default to the local all-MiniLM-L6-v2 server (384-dim), matching the
        // existing embeddings written by scripts/kb-embed-local.py. This keeps
        // dimension consistent across the whole corpus and avoids silent
        // cosine-similarity degradation from mixed 384/768 vectors.
        Self {
            api_key: std::env::var("NEOTRIX_EMBEDDING_API_KEY")
                .or_else(|_| std::env::var("NEOTRIX_API_KEY"))
                .unwrap_or_else(|_| "local".to_string()),
            base_url: std::env::var("NEOTRIX_EMBEDDING_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8237/v1".to_string()),
            model: std::env::var("NEOTRIX_EMBEDDING_MODEL")
                .unwrap_or_else(|_| "all-MiniLM-L6-v2".to_string()),
            dimension: std::env::var("NEOTRIX_EMBEDDING_DIMENSION")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(384),
        }
    }
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

/// Generate a single embedding vector via OpenAI-compatible API.
pub fn embed_text(config: &EmbeddingConfig, text: &str) -> Result<Vec<f32>, String> {
    let mut results = embed_text_batch(config, &[text])?;
    results.pop().ok_or_else(|| "Empty batch response".to_string())
}

fn embedding_client() -> Result<&'static reqwest::blocking::Client, String> {
    static CLIENT: OnceLock<Result<reqwest::blocking::Client, String>> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| format!("embedding HTTP client: {}", e))
    })
    .as_ref()
    .map_err(|e| e.clone())
}

/// Generate embeddings for multiple texts in a single API call.
pub fn embed_text_batch(config: &EmbeddingConfig, texts: &[&str]) -> Result<Vec<Vec<f32>>, String> {
    if texts.is_empty() { return Ok(Vec::new()); }

    let client = embedding_client()?;

    let input: Vec<&str> = texts.to_vec();
    let body = serde_json::json!({
        "input": input,
        "model": config.model,
        "dimensions": config.dimension,
    });

    let resp = client
        .post(format!("{}/embeddings", config.base_url))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .json(&body)
        .send()
        .map_err(|e| format!("Embedding request: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err_text = resp.text().unwrap_or_default();
        return Err(format!("Embedding API {}: {}", status, err_text));
    }

    let data: EmbeddingResponse = resp.json().map_err(|e| format!("Parse response: {}", e))?;

    // Sort by index to preserve original order
    let mut indexed: Vec<(usize, Vec<f32>)> = data.data.into_iter()
        .map(|d| (d.index, d.embedding))
        .collect();
    indexed.sort_by_key(|(idx, _)| *idx);

    Ok(indexed.into_iter().map(|(_, v)| v).collect())
}

/// Cosine similarity between two equal-length vectors.
/// Returns 0.0 if vectors have different lengths (logs a warning).
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        log::warn!("cosine_similarity: dimension mismatch {} vs {}", a.len(), b.len());
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { 0.0 }
    else { (dot / (norm_a * norm_b)) as f64 }
}

/// Serialize a Vec<f32> to a byte blob for SQLite storage (little-endian f32).
fn vector_to_blob(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

/// Deserialize a byte blob back to Vec<f32>.
fn blob_to_vector(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Store a single node's embedding.
pub fn store_embedding(conn: &Connection, node_id: &str, vector: &[f32], model: &str) -> rusqlite::Result<()> {
    let dim = vector.len() as i32;
    let blob = vector_to_blob(vector);
    conn.execute(
        "INSERT OR REPLACE INTO embeddings (node_id, vector, dimension, model) VALUES (?1, ?2, ?3, ?4)",
        params![node_id, blob, dim, model],
    )?;
    Ok(())
}

/// Retrieve a single node's embedding.
pub fn get_embedding(conn: &Connection, node_id: &str) -> rusqlite::Result<Option<Vec<f32>>> {
    let mut stmt = conn.prepare("SELECT vector, dimension FROM embeddings WHERE node_id=?1")?;
    let mut rows = stmt.query(params![node_id])?;
    if let Some(row) = rows.next()? {
        let blob: Vec<u8> = row.get(0)?;
        Ok(Some(blob_to_vector(&blob)))
    } else {
        Ok(None)
    }
}

pub fn embedding_count(conn: &Connection) -> rusqlite::Result<usize> {
    conn.query_row("SELECT COUNT(*) FROM embeddings JOIN nodes ON nodes.id = embeddings.node_id", [], |row| row.get(0))
}

pub fn load_embeddings_page(conn: &Connection, offset: usize, limit: usize) -> rusqlite::Result<Vec<(String, Vec<f32>)>> {
    let mut stmt = conn.prepare(
        "SELECT e.node_id, e.vector FROM embeddings e JOIN nodes n ON n.id = e.node_id ORDER BY e.node_id LIMIT ?1 OFFSET ?2"
    )?;
    let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
        let node_id: String = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        Ok((node_id, blob_to_vector(&blob)))
    })?;
    let mut result = Vec::with_capacity(limit.min(4096));
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

/// Load all (node_id, embedding) pairs from the database.
/// Warning: loads entire embedding table into memory — use `load_embeddings_page` for large datasets.
pub fn load_all_embeddings(conn: &Connection) -> rusqlite::Result<Vec<(String, Vec<f32>)>> {
    let mut stmt = conn.prepare(
        "SELECT e.node_id, e.vector FROM embeddings e JOIN nodes n ON n.id = e.node_id"
    )?;
    let mut results = Vec::new();
    let rows = stmt.query_map([], |row| {
        let node_id: String = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        Ok((node_id, blob_to_vector(&blob)))
    })?;
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Find nodes without embeddings.
pub fn find_nodes_missing_embeddings(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT id FROM nodes WHERE id NOT IN (SELECT node_id FROM embeddings)"
    )?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect()
}

/// Build text for embedding from a node's title + summary + content.
pub fn build_node_text(title: &str, summary: Option<&str>, content: Option<&str>) -> String {
    let mut text = title.to_string();
    if let Some(s) = summary {
        text.push_str(". ");
        text.push_str(s);
    }
    if let Some(c) = content {
        text.push_str(". ");
        text.push_str(&c[..c.len().min(500)]);
    }
    text
}

/// Product Quantization (PQ) approximate nearest neighbor support.
///
/// Reads the trained codebook (`pq_codebook`) + compressed codes (`embeddings_pq`)
/// written by `scripts/kb-embed-pq.py`. Scores a query vector against all
/// quantized vectors using asymmetric distance computation (ADC): each
/// sub-vector of the query is compared to the K centroids of its sub-space once,
/// then the per-code distances are summed — O(M×K) per query instead of O(D×N).
pub fn pq_ann_search(
    conn: &Connection,
    query_vec: &[f32],
    k: usize,
    codebook_id: Option<i64>,
) -> rusqlite::Result<Vec<(String, f64)>> {
    let cb = match load_latest_codebook(conn, codebook_id)? {
        Some(cb) => cb,
        None => return Ok(Vec::new()),
    };
    let m = cb.m as usize;
    let sub_dim = cb.sub_dim as usize;
    let ks = cb.ks as usize;
    let d = m * sub_dim;
    if query_vec.len() != d {
        log::warn!("pq_ann_search: query dim {} != codebook dim {}", query_vec.len(), d);
        return Ok(Vec::new());
    }

    // Per-subspace distance tables: for each subspace s and each centroid c,
    // store squared distance between query sub-vector and centroid.
    let mut tables: Vec<Vec<f64>> = Vec::with_capacity(m);
    for s in 0..m {
        let q_sub = &query_vec[s * sub_dim..(s + 1) * sub_dim];
        let mut row = Vec::with_capacity(ks);
        for c in 0..ks {
            let centroid = &cb.codewords[s][c];
            let dist: f64 = q_sub.iter().zip(centroid.iter())
                .map(|(a, b)| {
                    let diff = (a - b) as f64;
                    diff * diff
                })
                .sum();
            row.push(dist);
        }
        tables.push(row);
    }

    let mut stmt = conn.prepare(
        "SELECT node_id, pq_codes FROM embeddings_pq WHERE (?1 IS NULL OR codebook_id = ?1)"
    )?;
    let rows = stmt.query_map([codebook_id], |row| {
        let node_id: String = row.get(0)?;
        let codes: Vec<u8> = row.get(1)?;
        Ok((node_id, codes))
    })?;

    let mut scored: Vec<(String, f64)> = Vec::new();
    for row in rows.flatten() {
        let (node_id, codes) = row;
        if codes.len() != m {
            continue;
        }
        let dist: f64 = codes.iter().enumerate()
            .map(|(s, &c)| tables[s][c as usize])
            .sum();
        scored.push((node_id, -dist));
    }
    // Coarse candidate ranking: score = -dist, so nearest (smallest dist) has the
    // HIGHEST score. Sort descending to pull the closest candidates first.
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    // Re-rank the top coarse candidates by exact cosine on the stored vectors.
    // PQ centroid quantization loses fine-grained precision, so a handful of
    // unrelated candidates can rank above the true nearest neighbours (they sit
    // in a near-identical coarse ADC band). Re-scoring the promising subset with
    // the exact embedding recovers the correct order while keeping PQ as the
    // fast filter (no full-corpus scan).
    let coarse_count = (k * 16).min(scored.len());
    if coarse_count > 0 {
        let coarse: Vec<(String, f64)> = scored.into_iter().take(coarse_count).collect();
        let mut exact = Vec::with_capacity(coarse.len());
        for (node_id, _) in coarse {
            let sim = match exact_vector_cosine(conn, &node_id, query_vec) {
                Some(s) => s,
                None => continue,
            };
            exact.push((node_id, sim));
        }
        exact.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        exact.truncate(k);
        return Ok(exact);
    }
    Ok(Vec::new())
}

/// Load one vector blob from the `embeddings` table and compute exact cosine
/// against `query_vec`. Returns `None` if the node has no usable vector.
fn exact_vector_cosine(conn: &Connection, node_id: &str, query_vec: &[f32]) -> Option<f64> {
    let blob: Vec<u8> = conn.query_row(
        "SELECT vector FROM embeddings WHERE node_id = ?1",
        params![node_id],
        |row| row.get(0),
    ).ok()?;
    let dim_bytes = blob.len() / 4;
    let mut v = Vec::with_capacity(dim_bytes);
    for i in 0..dim_bytes {
        let off = i * 4;
        v.push(f32::from_le_bytes([blob[off], blob[off + 1], blob[off + 2], blob[off + 3]]));
    }
    Some(cosine_similarity(query_vec, &v))
}

struct PqCodebook {
    m: usize,
    ks: usize,
    sub_dim: usize,
    codewords: Vec<Vec<Vec<f32>>>,
}

fn load_latest_codebook(conn: &Connection, codebook_id: Option<i64>) -> rusqlite::Result<Option<PqCodebook>> {
    let row = if let Some(id) = codebook_id {
        conn.query_row(
            "SELECT m, ks, sub_dim, codewords FROM pq_codebook WHERE id = ?1",
            params![id],
            |row| Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?, row.get::<_, i32>(2)?, row.get::<_, Vec<u8>>(3)?)),
        ).ok()
    } else {
        conn.query_row(
            "SELECT m, ks, sub_dim, codewords FROM pq_codebook ORDER BY id DESC LIMIT 1",
            [],
            |row| Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?, row.get::<_, i32>(2)?, row.get::<_, Vec<u8>>(3)?)),
        ).ok()
    };
    let Some((m, ks, sub_dim, blob)) = row else {
        return Ok(None);
    };
    let m = m as usize;
    let ks = ks as usize;
    let sub_dim = sub_dim as usize;
    let expect = m * ks * sub_dim * 4;
    if blob.len() != expect {
        log::warn!("pq_codebook blob size {} != expected {}", blob.len(), expect);
        return Ok(None);
    }
    let mut codewords: Vec<Vec<Vec<f32>>> = Vec::with_capacity(m);
    for s in 0..m {
        let mut sub = Vec::with_capacity(ks);
        for c in 0..ks {
            let base = (s * ks + c) * sub_dim * 4;
            let mut v = Vec::with_capacity(sub_dim);
            for i in 0..sub_dim {
                let off = base + i * 4;
                let b = [blob[off], blob[off + 1], blob[off + 2], blob[off + 3]];
                v.push(f32::from_le_bytes(b));
            }
            sub.push(v);
        }
        codewords.push(sub);
    }
    Ok(Some(PqCodebook { m, ks, sub_dim, codewords }))
}

/// Result of a PQ training run.
#[derive(Debug)]
pub struct PqTrainReport {
    pub codebook_id: i64,
    pub m: usize,
    pub ks: usize,
    pub sub_dim: usize,
    pub vector_count: usize,
}

/// Lloyd k-means over `rows` (n×d subspaces) with `k` centroids for `iters` iterations.
/// Faithful port of `scripts/kb-embed-pq.py:kmeans` (seed 42, random init w/o replacement).
fn pq_kmeans(rows: &[&[f32]], k: usize, iters: usize) -> Vec<Vec<f32>> {
    let n = rows.len();
    let d = rows[0].len();
    use std::collections::HashSet;
    let mut seed: u64 = 42;
    let mut next_u64 = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed.wrapping_mul(2685821657736338717)
    };
    let mut used: HashSet<usize> = HashSet::new();
    let mut chosen: Vec<usize> = Vec::new();
    let k = k.min(n);
    while chosen.len() < k {
        let idx = (next_u64() as usize) % n;
        if used.insert(idx) {
            chosen.push(idx);
        }
    }
    let mut centroids: Vec<Vec<f32>> = chosen.iter().map(|&i| rows[i].to_vec()).collect();
    for _ in 0..iters {
        let mut labels: Vec<usize> = Vec::with_capacity(n);
        for r in rows {
            let mut best = 0usize;
            let mut best_dist = f32::INFINITY;
            for (c, cen) in centroids.iter().enumerate() {
                let mut dist = 0.0f32;
                for (a, b) in r.iter().zip(cen.iter()) {
                    let diff = a - b;
                    dist += diff * diff;
                }
                if dist < best_dist {
                    best_dist = dist;
                    best = c;
                }
            }
            labels.push(best);
        }
        let mut sums: Vec<Vec<f32>> = vec![vec![0.0f32; d]; centroids.len()];
        let mut counts: Vec<usize> = vec![0usize; centroids.len()];
        for (i, r) in rows.iter().enumerate() {
            let c = labels[i];
            counts[c] += 1;
            for (j, v) in r.iter().enumerate() {
                sums[c][j] += v;
            }
        }
        for c in 0..centroids.len() {
            if counts[c] > 0 {
                for j in 0..d {
                    centroids[c][j] = sums[c][j] / counts[c] as f32;
                }
            }
        }
    }
    centroids
}

/// Load all vectors from the `embeddings` table (optionally the first `limit` rows).
fn load_embeddings_raw(conn: &Connection, limit: Option<usize>) -> rusqlite::Result<(Vec<String>, Vec<Vec<f32>>)> {
    let sql = match limit {
        Some(l) => format!("SELECT node_id, vector FROM embeddings LIMIT {l}"),
        None => "SELECT node_id, vector FROM embeddings".to_string(),
    };
    let mut stmt = conn.prepare(&sql)?;
    let mut ids = Vec::new();
    let mut vecs = Vec::new();
    let rows = stmt.query_map([], |row| {
        let node_id: String = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        Ok((node_id, blob_to_vector(&blob)))
    })?;
    for row in rows {
        let (id, v) = row?;
        ids.push(id);
        vecs.push(v);
    }
    Ok((ids, vecs))
}

/// Train a PQ codebook from the `embeddings` table and persist both the codebook
/// (`pq_codebook`) and per-node compressed codes (`embeddings_pq`).
///
/// Faithful port of `scripts/kb-embed-pq.py` main flow: reshapes each vector into
/// `m` sub-spaces, trains one centroid set of size `ks` per subspace (Python
/// recommander defaults m=24/ks=256/iters=10 on 384-dim), then assigns each
/// vector's nearest centroid index per subspace into a packed `<m}B` byte code.
///
/// Errors (with a caller-facing message) if there aren't enough vectors to train
/// the requested `ks`, or if `m` doesn't divide `dimension`.
pub fn train_pq_codebook(
    conn: &Connection,
    m: usize,
    ks: usize,
    dimension: usize,
    limit: Option<usize>,
) -> Result<PqTrainReport, String> {
    if m == 0 || !dimension.is_multiple_of(m) {
        return Err(format!("m must be a positive divisor of {dimension}"));
    }
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM embeddings", [], |r| r.get(0))
        .map_err(|e| format!("PQ: count embeddings: {e}"))?;
    if (total as usize) < ks * 4 {
        return Err(format!(
            "Not enough vectors to train PQ: {total} < {}( = ks×4). Need more embeddings first.",
            ks * 4
        ));
    }
    let (ids, vecs) = load_embeddings_raw(conn, limit).map_err(|e| format!("PQ: load embeddings: {e}"))?;
    let n = vecs.len();
    let sub_dim = dimension / m;

    let mut codewords_all: Vec<Vec<Vec<f32>>> = Vec::with_capacity(m);
    for s in 0..m {
        let sub: Vec<&[f32]> = vecs.iter().map(|r| &r[s * sub_dim..(s + 1) * sub_dim]).collect();
        codewords_all.push(pq_kmeans(&sub, ks, 10));
    }

    let mut codeword_blob: Vec<u8> = Vec::new();
    for cw in &codewords_all {
        for centroid in cw {
            for v in centroid {
                codeword_blob.extend_from_slice(&v.to_le_bytes());
            }
        }
    }
    let now = crate::neotrix::nt_memory_kb::nt_memory_embed::unix_now();
    conn.execute(
        "INSERT INTO pq_codebook (m, ks, sub_dim, codewords, dimension, model, trained_at, num_vectors) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![m as i64, ks as i64, sub_dim as i64, codeword_blob, dimension as i64, "all-MiniLM-L6-v2", now, n as i64],
    ).map_err(|e| format!("insert pq_codebook: {e}"))?;
    let codebook_id = conn.last_insert_rowid();

    for i in 0..n {
        let mut codes: Vec<u8> = Vec::with_capacity(m);
        for s in 0..m {
            let sub = &vecs[i][s * sub_dim..(s + 1) * sub_dim];
            let mut best = 0usize;
            let mut best_dist = f32::INFINITY;
            for (c, centroid) in codewords_all[s].iter().enumerate() {
                let mut dist = 0.0f32;
                for (a, b) in sub.iter().zip(centroid.iter()) {
                    let diff = a - b;
                    dist += diff * diff;
                }
                if dist < best_dist {
                    best_dist = dist;
                    best = c;
                }
            }
            codes.push(best as u8);
        }
        conn.execute(
            "INSERT OR REPLACE INTO embeddings_pq (node_id, pq_codes, codebook_id) VALUES (?1,?2,?3)",
            params![ids[i], codes, codebook_id],
        ).map_err(|e| format!("insert embeddings_pq for {}: {e}", ids[i]))?;
    }
    Ok(PqTrainReport { codebook_id, m, ks, sub_dim, vector_count: n })
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_dimension_mismatch_safe() {
        // Mismatched dims must not panic and should yield 0.0 (silent-safe).
        let a: Vec<f32> = vec![1.0, 0.0];
        let b: Vec<f32> = vec![0.0, 1.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_cosine_identical() {
        let a: Vec<f32> = vec![0.5, 0.5, 0.0];
        let b: Vec<f32> = vec![0.5, 0.5, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vector_blob_roundtrip() {
        let v: Vec<f32> = vec![1.5, -2.0, 3.25, 100.5];
        let blob = vector_to_blob(&v);
        let back = blob_to_vector(&blob);
        assert_eq!(v, back);
    }

    #[test]
    fn test_pq_codebook_parse_in_memory() {
        // Build a synthetic codebook blob: m=2 subspaces, ks=2 centroids, sub_dim=2
        let m = 2;
        let ks = 2;
        let sub_dim = 2;
        let mut blob = Vec::new();
        // subspace 0: centroids [1,1],[9,9]; subspace 1: [2,2],[8,8]
        for cw in [1.0f32, 1.0, 9.0, 9.0, 2.0, 2.0, 8.0, 8.0] {
            blob.extend_from_slice(&cw.to_le_bytes());
        }
        // Build a temp DB with pq_codebook row and embeddings_pq entry
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE pq_codebook(id INTEGER PRIMARY KEY, m INTEGER, ks INTEGER, sub_dim INTEGER, codewords BLOB, dimension INTEGER, model TEXT, trained_at INTEGER, num_vectors INTEGER);
             CREATE TABLE embeddings_pq(node_id TEXT PRIMARY KEY, pq_codes BLOB, codebook_id INTEGER);
             CREATE TABLE embeddings(node_id TEXT PRIMARY KEY, vector BLOB, dimension INTEGER, model TEXT, created_at INTEGER);",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO pq_codebook (id, m, ks, sub_dim, codewords, dimension, model, trained_at, num_vectors) VALUES (1, 2, 2, 2, ?1, 4, 'mini', 0, 1)",
            params![blob],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO embeddings_pq VALUES ('n1', ?1, 1)",
            params![vec![0u8, 1u8]],
        )
        .unwrap();
        // Exact vector for n1 == query, so rerank lifts it to the top.
        let mut exact_blob = Vec::new();
        for f in [1.0f32, 1.0, 8.0, 8.0] {
            exact_blob.extend_from_slice(&f.to_le_bytes());
        }
        conn.execute(
            "INSERT INTO embeddings (node_id, vector, dimension, model, created_at) VALUES ('n1', ?1, 4, 'mini', 0)",
            params![exact_blob],
        )
        .unwrap();
        // query vector very close to sub0 centroid0 [1,1] and sub1 centroid1 [8,8]
        let q: Vec<f32> = vec![1.0, 1.0, 8.0, 8.0];
        let hits = pq_ann_search(&conn, &q, 1, Some(1)).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, "n1");
        // After exact re-ranking the score is cosine similarity (positive for a match).
        assert!(hits[0].1 > 0.0, "re-ranked score should be cosine similarity");
    }

    #[test]
    fn test_train_pq_codebook_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE pq_codebook(id INTEGER PRIMARY KEY AUTOINCREMENT, m INTEGER, ks INTEGER, sub_dim INTEGER, codewords BLOB, dimension INTEGER, model TEXT, trained_at INTEGER, num_vectors INTEGER);
             CREATE TABLE embeddings_pq(node_id TEXT PRIMARY KEY, pq_codes BLOB, codebook_id INTEGER);
             CREATE TABLE embeddings(node_id TEXT PRIMARY KEY, vector BLOB, dimension INTEGER, model TEXT, created_at INTEGER);",
        )
        .unwrap();
        // 8 vectors of dim 4 clustered so kmeans converges: two clusters of 4
        let vecs: Vec<Vec<f32>> = (0..8)
            .map(|i| if i < 4 { vec![1.0, 1.0, 1.0, 1.0] } else { vec![5.0, 5.0, 5.0, 5.0] })
            .collect();
        for (i, v) in vecs.iter().enumerate() {
            conn.execute(
                "INSERT INTO embeddings (node_id, vector, dimension, model, created_at) VALUES (?1, ?2, 4, 'mini', 0)",
                params![format!("n{i}"), vector_to_blob(v)],
            )
            .unwrap();
        }
        // m=2 sub-spaces × ks=2 centroids; 8 vectors >= ks*4
        let report = train_pq_codebook(&conn, 2, 2, 4, None).unwrap();
        assert_eq!(report.m, 2);
        assert_eq!(report.ks, 2);
        assert_eq!(report.sub_dim, 2);
        assert_eq!(report.vector_count, 8);
        assert!(report.codebook_id >= 1);

        let pq_count: i64 = conn.query_row("SELECT COUNT(*) FROM embeddings_pq", [], |r| r.get(0)).unwrap();
        assert_eq!(pq_count, 8);
        let cb_count: i64 = conn.query_row("SELECT COUNT(*) FROM pq_codebook", [], |r| r.get(0)).unwrap();
        assert_eq!(cb_count, 1);

        // The trained codebook must be loadable and searchable: query == cluster A
        let q: Vec<f32> = vec![1.0, 1.0, 1.0, 1.0];
        let hits = pq_ann_search(&conn, &q, 2, Some(report.codebook_id)).unwrap();
        assert!(!hits.is_empty());
        // n0/n1 (cluster A) must rank above n2/n3 (cluster B)
        let top = &hits[0].0;
        assert!(top == "n0" || top == "n1");
    }

    #[test]
    fn test_train_pq_too_few_vectors() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE pq_codebook(id INTEGER PRIMARY KEY AUTOINCREMENT, m INTEGER, ks INTEGER, sub_dim INTEGER, codewords BLOB, dimension INTEGER, model TEXT, trained_at INTEGER, num_vectors INTEGER);
             CREATE TABLE embeddings_pq(node_id TEXT PRIMARY KEY, pq_codes BLOB, codebook_id INTEGER);
             CREATE TABLE embeddings(node_id TEXT PRIMARY KEY, vector BLOB, dimension INTEGER, model TEXT, created_at INTEGER);",
        )
        .unwrap();
        for i in 0..3 {
            conn.execute(
                "INSERT INTO embeddings (node_id, vector, dimension, model, created_at) VALUES (?1, ?2, 4, 'mini', 0)",
                params![format!("n{i}"), vector_to_blob(&vec![0.0, 0.0, 0.0, 0.0])],
            )
            .unwrap();
        }
        let err = train_pq_codebook(&conn, 2, 2, 4, None).unwrap_err();
        assert!(err.contains("Not enough vectors"), "err: {err}");
    }
}
