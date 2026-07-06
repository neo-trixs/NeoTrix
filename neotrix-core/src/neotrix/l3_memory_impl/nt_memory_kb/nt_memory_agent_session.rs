use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A persistent, session-scoped agent memory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionEntry {
    pub id: String,
    pub agent_id: String,
    pub session_id: String,
    pub tier: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub created_at: i64,
    pub access_count: u64,
    pub superseded: bool,
    pub superseded_by: Option<String>,
}

/// A record of an agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub agent_id: String,
    pub label: String,
    pub created_at: i64,
    pub ended_at: Option<i64>,
    pub metadata: HashMap<String, String>,
}

/// Stateless persistent agent session & memory manager.
///
/// All methods take `&Connection` (the KB's shared connection), matching the
/// pattern used by `nt_memory_store`, `nt_memory_search`, etc.
/// Tables are session-level, not agent-level, enabling cross-session retrieval.
pub struct AgentSessionManager;

impl AgentSessionManager {
    pub fn ensure_tables(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS agent_sessions (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                label TEXT NOT NULL DEFAULT '',
                created_at INTEGER NOT NULL,
                ended_at INTEGER,
                metadata TEXT NOT NULL DEFAULT '{}'
            );
            CREATE TABLE IF NOT EXISTS agent_memory_entries (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                tier TEXT NOT NULL DEFAULT 'core',
                content TEXT NOT NULL,
                embedding BLOB,
                metadata TEXT NOT NULL DEFAULT '{}',
                created_at INTEGER NOT NULL,
                access_count INTEGER NOT NULL DEFAULT 1,
                superseded INTEGER NOT NULL DEFAULT 0,
                superseded_by TEXT,
                FOREIGN KEY (session_id) REFERENCES agent_sessions(id)
            );
            CREATE INDEX IF NOT EXISTS idx_ame_agent ON agent_memory_entries(agent_id);
            CREATE INDEX IF NOT EXISTS idx_ame_session ON agent_memory_entries(session_id);
            CREATE INDEX IF NOT EXISTS idx_ame_tier ON agent_memory_entries(tier);
            CREATE INDEX IF NOT EXISTS idx_as_agent ON agent_sessions(agent_id);"
        )?;
        Ok(())
    }

    pub fn begin_session(conn: &Connection, agent_id: &str, label: &str) -> rusqlite::Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = unix_now();
        conn.execute(
            "INSERT INTO agent_sessions (id, agent_id, label, created_at, metadata) VALUES (?1, ?2, ?3, ?4, '{}')",
            params![id, agent_id, label, now],
        )?;
        Ok(id)
    }

    pub fn end_session(conn: &Connection, session_id: &str) -> rusqlite::Result<()> {
        conn.execute(
            "UPDATE agent_sessions SET ended_at = ?1 WHERE id = ?2",
            params![unix_now(), session_id],
        )?;
        Ok(())
    }

    pub fn store(
        conn: &Connection,
        agent_id: &str,
        session_id: &str,
        content: &str,
        tier: &str,
        metadata: HashMap<String, String>,
        embedding: Option<&[f32]>,
    ) -> rusqlite::Result<String> {
        let id = Uuid::new_v4().to_string();
        let meta_json = serde_json::to_string(&metadata).unwrap_or_else(|_| "{}".to_string());
        let blob = embedding.map(|v| v.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<u8>>());
        conn.execute(
            "INSERT INTO agent_memory_entries (id, session_id, agent_id, tier, content, embedding, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, session_id, agent_id, tier, content, blob, meta_json, unix_now()],
        )?;
        Ok(id)
    }

    pub fn recall_by_agent(conn: &Connection, agent_id: &str, query: &str, limit: usize) -> rusqlite::Result<Vec<AgentSessionEntry>> {
        let pattern = format!("%{}%", query);
        let stmt = conn.prepare(
            "SELECT id, agent_id, session_id, tier, content, metadata, created_at, access_count, superseded, superseded_by
             FROM agent_memory_entries
             WHERE agent_id = ?1 AND superseded = 0 AND content LIKE ?2
             ORDER BY created_at DESC LIMIT ?3",
        )?;
        map_entries(stmt, params![agent_id, pattern, limit as i64])
    }

    pub fn recall_by_session(conn: &Connection, session_id: &str, query: &str, limit: usize) -> rusqlite::Result<Vec<AgentSessionEntry>> {
        let pattern = format!("%{}%", query);
        let stmt = conn.prepare(
            "SELECT id, agent_id, session_id, tier, content, metadata, created_at, access_count, superseded, superseded_by
             FROM agent_memory_entries
             WHERE session_id = ?1 AND superseded = 0 AND content LIKE ?2
             ORDER BY created_at DESC LIMIT ?3",
        )?;
        map_entries(stmt, params![session_id, pattern, limit as i64])
    }

    pub fn recall_similar(conn: &Connection, agent_id: &str, query_embedding: &[f32], limit: usize) -> rusqlite::Result<Vec<(AgentSessionEntry, f64)>> {
        let mut stmt = conn.prepare(
            "SELECT id, agent_id, session_id, tier, content, metadata, created_at, access_count, superseded, superseded_by, embedding
             FROM agent_memory_entries
             WHERE agent_id = ?1 AND superseded = 0 AND embedding IS NOT NULL",
        )?;
        let rows = stmt.query_map(params![agent_id], |row| {
            let meta_str: String = row.get(5)?;
            let meta: HashMap<String, String> = serde_json::from_str(&meta_str).unwrap_or_default();
            let blob: Option<Vec<u8>> = row.get(10)?;
            let emb = blob.map(|b| {
                b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect::<Vec<f32>>()
            });
            Ok((AgentSessionEntry {
                id: row.get(0)?, agent_id: row.get(1)?, session_id: row.get(2)?,
                tier: row.get(3)?, content: row.get(4)?, metadata: meta,
                created_at: row.get(6)?, access_count: row.get::<_, i64>(7)? as u64,
                superseded: row.get::<_, i64>(8)? != 0, superseded_by: row.get(9)?,
            }, emb))
        })?;
        let mut scored: Vec<(AgentSessionEntry, f64)> = Vec::new();
        for r in rows {
            if let Ok((entry, Some(emb))) = r {
                scored.push((entry, cosine_similarity(query_embedding, &emb)));
            }
        }
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        Ok(scored)
    }

    pub fn list_sessions(conn: &Connection, agent_id: &str) -> rusqlite::Result<Vec<AgentSession>> {
        let mut stmt = conn.prepare(
            "SELECT id, agent_id, label, created_at, ended_at, metadata FROM agent_sessions WHERE agent_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![agent_id], |row| {
            let meta_str: String = row.get(5)?;
            let meta: HashMap<String, String> = serde_json::from_str(&meta_str).unwrap_or_default();
            Ok(AgentSession { id: row.get(0)?, agent_id: row.get(1)?, label: row.get(2)?, created_at: row.get(3)?, ended_at: row.get(4)?, metadata: meta })
        })?;
        let mut sessions = Vec::new();
        for s in rows.flatten() { sessions.push(s); }
        Ok(sessions)
    }
}

fn unix_now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

fn map_entries(mut stmt: rusqlite::Statement, params: impl rusqlite::Params) -> rusqlite::Result<Vec<AgentSessionEntry>> {
    let rows = stmt.query_map(params, |row| {
        let meta_str: String = row.get(5)?;
        let meta: HashMap<String, String> = serde_json::from_str(&meta_str).unwrap_or_default();
        Ok(AgentSessionEntry {
            id: row.get(0)?, agent_id: row.get(1)?, session_id: row.get(2)?,
            tier: row.get(3)?, content: row.get(4)?, metadata: meta,
            created_at: row.get(6)?, access_count: row.get::<_, i64>(7)? as u64,
            superseded: row.get::<_, i64>(8)? != 0, superseded_by: row.get(9)?,
        })
    })?;
    let mut results = Vec::new();
    for e in rows.flatten() { results.push(e); }
    Ok(results)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 { return 0.0; }
    (dot / (mag_a * mag_b)) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mgr() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        AgentSessionManager::ensure_tables(&conn).unwrap();
        conn
    }

    #[test]
    fn test_begin_and_end_session() {
        let conn = mgr();
        let sid = AgentSessionManager::begin_session(&conn, "agent-1", "test").unwrap();
        let sessions = AgentSessionManager::list_sessions(&conn, "agent-1").unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, sid);
        assert!(sessions[0].ended_at.is_none());
        AgentSessionManager::end_session(&conn, &sid).unwrap();
        let sessions = AgentSessionManager::list_sessions(&conn, "agent-1").unwrap();
        assert!(sessions[0].ended_at.is_some());
    }

    #[test]
    fn test_store_and_recall() {
        let conn = mgr();
        let sid = AgentSessionManager::begin_session(&conn, "agent-1", "test").unwrap();
        let mut meta = HashMap::new();
        meta.insert("type".into(), "observation".into());
        AgentSessionManager::store(&conn, "agent-1", &sid, "the sky is blue", "core", meta.clone(), None).unwrap();
        AgentSessionManager::store(&conn, "agent-1", &sid, "the grass is green", "core", meta, None).unwrap();
        let results = AgentSessionManager::recall_by_agent(&conn, "agent-1", "sky", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("sky"));
    }

    #[test]
    fn test_recall_session_scoped() {
        let conn = mgr();
        let sid1 = AgentSessionManager::begin_session(&conn, "agent-1", "s1").unwrap();
        let sid2 = AgentSessionManager::begin_session(&conn, "agent-1", "s2").unwrap();
        AgentSessionManager::store(&conn, "agent-1", &sid1, "data from session 1", "core", HashMap::new(), None).unwrap();
        AgentSessionManager::store(&conn, "agent-1", &sid2, "data from session 2", "core", HashMap::new(), None).unwrap();
        let r1 = AgentSessionManager::recall_by_session(&conn, &sid1, "data", 10).unwrap();
        assert_eq!(r1.len(), 1);
        assert!(r1[0].content.contains("session 1"));
    }

    #[test]
    fn test_recall_similar_with_embeddings() {
        let conn = mgr();
        let sid = AgentSessionManager::begin_session(&conn, "agent-1", "emb").unwrap();
        let emb1 = vec![1.0, 0.0, 0.0, 0.0];
        let emb2 = vec![0.0, 1.0, 0.0, 0.0];
        let emb_query = vec![0.9, 0.1, 0.0, 0.0];
        AgentSessionManager::store(&conn, "agent-1", &sid, "rust performance", "core", HashMap::new(), Some(&emb1)).unwrap();
        AgentSessionManager::store(&conn, "agent-1", &sid, "python simplicity", "core", HashMap::new(), Some(&emb2)).unwrap();
        let results = AgentSessionManager::recall_similar(&conn, "agent-1", &emb_query, 2).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].0.content.contains("rust"));
        assert!(results[0].1 > results[1].1);
    }

    #[test]
    fn test_multiple_agents_isolated() {
        let conn = mgr();
        let s1 = AgentSessionManager::begin_session(&conn, "agent-alpha", "a").unwrap();
        let s2 = AgentSessionManager::begin_session(&conn, "agent-beta", "b").unwrap();
        AgentSessionManager::store(&conn, "agent-alpha", &s1, "alpha's data", "core", HashMap::new(), None).unwrap();
        AgentSessionManager::store(&conn, "agent-beta", &s2, "beta's data", "core", HashMap::new(), None).unwrap();
        assert_eq!(AgentSessionManager::recall_by_agent(&conn, "agent-alpha", "data", 10).unwrap().len(), 1);
        assert_eq!(AgentSessionManager::recall_by_agent(&conn, "agent-beta", "data", 10).unwrap().len(), 1);
    }
}
