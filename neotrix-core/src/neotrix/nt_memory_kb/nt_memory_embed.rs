use serde::Deserialize;
use rusqlite::{params, Connection};

/// Configuration for the embedding API (OpenAI-compatible, incl. Gemini Embedding 2).
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub dimension: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("NEOTRIX_EMBEDDING_API_KEY")
                .or_else(|_| std::env::var("NEOTRIX_API_KEY"))
                .unwrap_or_default(),
            base_url: std::env::var("NEOTRIX_EMBEDDING_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            model: std::env::var("NEOTRIX_EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".to_string()),
            dimension: std::env::var("NEOTRIX_EMBEDDING_DIMENSION")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(768),
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

/// Generate embeddings for multiple texts in a single API call.
pub fn embed_text_batch(config: &EmbeddingConfig, texts: &[&str]) -> Result<Vec<Vec<f32>>, String> {
    if texts.is_empty() { return Ok(Vec::new()); }

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    let input: Vec<&str> = texts.iter().map(|t| *t).collect();
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

/// Load all (node_id, embedding) pairs from the database.
pub fn load_all_embeddings(conn: &Connection) -> rusqlite::Result<Vec<(String, Vec<f32>)>> {
    let mut stmt = conn.prepare(
        "SELECT e.node_id, e.vector FROM embeddings e JOIN nodes n ON n.id = e.node_id"
    )?;
    let rows = stmt.query_map([], |row| {
        let node_id: String = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        Ok((node_id, blob_to_vector(&blob)))
    })?;
    rows.collect()
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
