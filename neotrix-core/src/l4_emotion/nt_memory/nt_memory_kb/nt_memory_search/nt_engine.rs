use rusqlite::Connection;

use super::nt_pure_fns::search_fts;
use crate::l1_action::traits::{
    CapabilityCategory, CapabilityError, CapabilityHealth, CapabilityStats, ConstellationLevel,
    Document, L1Capability, SearchEngine as SearchEngineTrait, SearchResult as UnifiedSearchResult,
};

// ── KbSearchEngine (moved from nt_memory_search.rs, pure move) ──

// ════════════════════════════════════════════════════════════════
// Unified Architecture: L1Capability + SearchEngine trait
// ════════════════════════════════════════════════════════════════


/// KB Search wrapper implementing unified SearchEngine trait
pub struct KbSearchEngine {
    conn: std::sync::Arc<std::sync::Mutex<Connection>>,
}

impl KbSearchEngine {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: std::sync::Arc::new(std::sync::Mutex::new(conn)),
        }
    }
}

impl L1Capability for KbSearchEngine {
    fn capability_id(&self) -> &str {
        "memory.kb_search"
    }
    fn category(&self) -> CapabilityCategory {
        CapabilityCategory::Search
    }
    fn constellation(&self) -> ConstellationLevel {
        ConstellationLevel::C2Integration
    }
    fn health_check(&self) -> CapabilityHealth {
        CapabilityHealth {
            healthy: self.conn.lock().is_ok(),
            latency_ms: None,
            error_rate: 0.0,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            message: None,
        }
    }
    fn description(&self) -> &str {
        "KB search with FTS5 + vector hybrid ranking"
    }
    fn stats(&self) -> CapabilityStats {
        CapabilityStats::default()
    }
}

impl SearchEngineTrait for KbSearchEngine {
    fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<UnifiedSearchResult>, CapabilityError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| CapabilityError::Internal(e.to_string()))?;
        let results = search_fts(&conn, query, limit)
            .map_err(|e| CapabilityError::ExecutionFailed(e.to_string()))?;
        Ok(results
            .into_iter()
            .map(|r| UnifiedSearchResult {
                id: r.node.id,
                score: r.score,
                title: r.node.title,
                snippet: r.node.summary.unwrap_or_default(),
            })
            .collect())
    }

    fn index(&self, _doc: &Document) -> Result<(), CapabilityError> {
        // KB indexing is handled by ingest pipeline
        Ok(())
    }
}
