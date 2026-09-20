//! Knowledge writer — persist mapped knowledge into KB (SQLite).
//!
//! Extracted from batch-absorb.sh store_repo()/store_concept()/store_edge()
//! and absorb_to_capability.py rust_update_node_metadata() patterns.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::domain_mapper::MappingResult;
use super::source_adapter::KnowledgeInput;

#[derive(Error, Debug)]
pub enum WriterError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRecord {
    pub id: String,
    pub node_type: String,
    pub title: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub language: String,
    pub confidence: f64,
    pub importance: f64,
    pub created_at: i64,
    pub updated_at: i64,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRecord {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub weight: f64,
    pub description: Option<String>,
    pub created_at: i64,
}

pub struct KnowledgeWriter<'a> {
    conn: &'a Connection,
}

impl<'a> KnowledgeWriter<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn ensure_schema(&self) -> Result<(), WriterError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS knowledge_nodes (
                id TEXT PRIMARY KEY,
                node_type TEXT NOT NULL,
                title TEXT NOT NULL,
                summary TEXT,
                content TEXT,
                url TEXT UNIQUE,
                domain TEXT,
                language TEXT DEFAULT 'en',
                confidence REAL DEFAULT 1.0,
                importance REAL DEFAULT 0.5,
                created_at INTEGER,
                updated_at INTEGER,
                access_count INTEGER DEFAULT 0,
                metadata TEXT
            );
            CREATE TABLE IF NOT EXISTS knowledge_edges (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL,
                target_id TEXT NOT NULL,
                relation_type TEXT NOT NULL,
                weight REAL DEFAULT 1.0,
                description TEXT,
                created_at INTEGER,
                metadata TEXT
            );
            CREATE TABLE IF NOT EXISTS ingest_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_type TEXT,
                source_id TEXT,
                action TEXT,
                timestamp INTEGER,
                status TEXT,
                error TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_nodes_url ON knowledge_nodes(url);
            CREATE INDEX IF NOT EXISTS idx_edges_source ON knowledge_edges(source_id);
            CREATE INDEX IF NOT EXISTS idx_edges_target ON knowledge_edges(target_id);",
        )?;
        Ok(())
    }

    pub fn generate_id(prefix: &str) -> String {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let uuid_short = Uuid::new_v4().to_string()[..8].to_string();
        format!("{}-{}-{}", prefix, ts, uuid_short)
    }

    pub fn find_existing_by_url(&self, url: &str) -> Result<Option<String>, WriterError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM knowledge_nodes WHERE url = ?1 LIMIT 1")?;
        let mut rows = stmt.query_map(params![url], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    pub fn write_node(
        &self,
        input: &KnowledgeInput,
        mapping: &MappingResult,
    ) -> Result<String, WriterError> {
        let id = Self::generate_id("kb");
        let now = chrono::Utc::now().timestamp();
        let node_type = format!("{:?}", input.source_kind);
        let domain_str = mapping.domain.as_str();

        let capability_json = serde_json::json!({
            "branch": mapping.domain.as_str(),
            "capability": mapping.capability,
            "source_core": mapping.source_core.as_str(),
            "evidence": mapping.evidence,
        });

        let mut meta_map = input.metadata.clone();
        meta_map.insert("absorbed_capability".into(), capability_json);

        self.conn.execute(
            "INSERT OR IGNORE INTO knowledge_nodes
             (id, node_type, title, summary, content, url, domain, language,
              confidence, importance, created_at, updated_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                id,
                node_type,
                input.title,
                input.summary,
                input.content,
                input.url,
                domain_str,
                input.language,
                input.confidence,
                input.importance,
                now,
                now,
                serde_json::to_string(&meta_map)?,
            ],
        )?;

        Ok(id)
    }

    pub fn write_edge(
        &self,
        source_id: &str,
        target_id: &str,
        relation: &str,
        weight: f64,
        description: Option<&str>,
    ) -> Result<String, WriterError> {
        let id = Self::generate_id("edge");
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT OR IGNORE INTO knowledge_edges
             (id, source_id, target_id, relation_type, weight, description, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, source_id, target_id, relation, weight, description, now],
        )?;
        Ok(id)
    }

    pub fn write_ingest_log(
        &self,
        source_type: &str,
        source_id: &str,
        status: &str,
        error: Option<&str>,
    ) -> Result<(), WriterError> {
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT INTO ingest_log (source_type, source_id, action, timestamp, status, error)
             VALUES (?1, ?2, 'absorb', ?3, ?4, ?5)",
            params![source_type, source_id, now, status, error],
        )?;
        Ok(())
    }

    pub fn batch_write_capability_mapping(
        &self,
        node_id: &str,
        mapping: &MappingResult,
    ) -> Result<(), WriterError> {
        let patch = serde_json::json!({
            "absorbed_capability": {
                "branch": mapping.domain.as_str(),
                "capability": mapping.capability,
                "evidence": mapping.evidence,
                "source_core": mapping.source_core.as_str(),
            }
        });

        self.conn.execute(
            "UPDATE knowledge_nodes
             SET metadata = json_patch(COALESCE(metadata, '{}'), ?1),
                 updated_at = ?2
             WHERE id = ?3",
            params![patch.to_string(), chrono::Utc::now().timestamp(), node_id,],
        )?;
        Ok(())
    }

    pub fn absorb(
        &self,
        input: &KnowledgeInput,
        mapping: &MappingResult,
    ) -> Result<(String, bool), WriterError> {
        if let Some(ref url) = input.url {
            if let Some(existing_id) = self.find_existing_by_url(url)? {
                return Ok((existing_id, false));
            }
        }
        let node_id = self.write_node(input, mapping)?;
        self.write_ingest_log(&format!("{:?}", input.source_kind), &node_id, "ok", None)?;
        Ok((node_id, true))
    }

    pub fn node_count(&self) -> Result<usize, WriterError> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM knowledge_nodes", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn edge_count(&self) -> Result<usize, WriterError> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM knowledge_edges", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id1 = KnowledgeWriter::generate_id("kb");
        let id2 = KnowledgeWriter::generate_id("kb");
        assert_ne!(id1, id2);
        assert!(id1.starts_with("kb-"));
    }
}
