use std::collections::{HashSet, VecDeque};

use rusqlite::Connection;

use super::types::{TemporalFact, TemporalQuery};

// ─── Temporal Schema ───────────────────────────────────────────────

fn ensure_temporal_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS temporal_facts (
            fact_id TEXT PRIMARY KEY,
            subject TEXT NOT NULL,
            predicate TEXT NOT NULL,
            object TEXT NOT NULL,
            valid_from INTEGER NOT NULL,
            valid_to INTEGER,
            confidence REAL DEFAULT 1.0,
            source TEXT DEFAULT 'neotrix',
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_temporal_subject ON temporal_facts(subject);
        CREATE INDEX IF NOT EXISTS idx_temporal_object ON temporal_facts(object);
        CREATE INDEX IF NOT EXISTS idx_temporal_valid ON temporal_facts(valid_from, valid_to);",
    )?;
    Ok(())
}

/// Insert a temporal fact.
pub fn insert_temporal_fact(conn: &Connection, fact: &TemporalFact) -> Result<(), String> {
    ensure_temporal_schema(conn).map_err(|e| format!("schema error: {}", e))?;
    conn.execute(
        "INSERT OR REPLACE INTO temporal_facts (fact_id, subject, predicate, object, valid_from, valid_to, confidence, source, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            fact.fact_id,
            fact.subject,
            fact.predicate,
            fact.object,
            fact.valid_from,
            fact.valid_to,
            fact.confidence,
            fact.source,
            chrono::Utc::now().timestamp(),
        ],
    ).map_err(|e| format!("insert temporal fact: {}", e))?;
    Ok(())
}

/// Invalidate a fact by setting its valid_to to now.
pub fn invalidate_fact(conn: &Connection, fact_id: &str) -> Result<(), String> {
    ensure_temporal_schema(conn).map_err(|e| format!("schema error: {}", e))?;
    let now = chrono::Utc::now().timestamp();
    let rows = conn
        .execute(
            "UPDATE temporal_facts SET valid_to=?1 WHERE fact_id=?2 AND valid_to IS NULL",
            rusqlite::params![now, fact_id],
        )
        .map_err(|e| format!("invalidate fact: {}", e))?;
    if rows == 0 {
        return Err(format!("fact {} not found or already invalidated", fact_id));
    }
    Ok(())
}

/// Query facts with temporal filtering.
/// If as_of is None, returns only currently valid facts (valid_to IS NULL).
/// If as_of is Some(timestamp), returns facts valid at that time.
pub fn query_temporal_facts(
    conn: &Connection,
    query: &TemporalQuery,
    limit: usize,
) -> Result<Vec<TemporalFact>, String> {
    ensure_temporal_schema(conn).map_err(|e| format!("schema error: {}", e))?;

    let mut sql = String::from("SELECT fact_id, subject, predicate, object, valid_from, valid_to, confidence, source FROM temporal_facts WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref subject) = query.subject {
        sql.push_str(" AND subject=?");
        params.push(Box::new(subject.clone()));
    }
    if let Some(ref predicate) = query.predicate {
        sql.push_str(" AND predicate=?");
        params.push(Box::new(predicate.clone()));
    }
    if let Some(ref object) = query.object {
        sql.push_str(" AND object=?");
        params.push(Box::new(object.clone()));
    }

    if let Some(as_of) = query.as_of {
        sql.push_str(" AND valid_from <= ? AND (valid_to IS NULL OR valid_to > ?)");
        params.push(Box::new(as_of));
        params.push(Box::new(as_of));
    } else {
        sql.push_str(" AND valid_to IS NULL");
    }

    sql.push_str(&format!(" ORDER BY valid_from DESC LIMIT {}", limit));

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("prepare: {}", e))?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(TemporalFact {
                fact_id: row.get(0)?,
                subject: row.get(1)?,
                predicate: row.get(2)?,
                object: row.get(3)?,
                valid_from: row.get(4)?,
                valid_to: row.get(5)?,
                confidence: row.get(6)?,
                source: row.get(7)?,
            })
        })
        .map_err(|e| format!("query: {}", e))?;

    let mut facts = Vec::new();
    for row in rows {
        facts.push(row.map_err(|e| format!("row: {}", e))?);
    }
    Ok(facts)
}

/// Get the full history of a subject-predicate-object triple.
pub fn get_fact_history(
    conn: &Connection,
    subject: &str,
    predicate: &str,
    object: &str,
) -> Result<Vec<TemporalFact>, String> {
    ensure_temporal_schema(conn).map_err(|e| format!("schema error: {}", e))?;
    let mut stmt = conn
        .prepare(
            "SELECT fact_id, subject, predicate, object, valid_from, valid_to, confidence, source
         FROM temporal_facts
         WHERE subject=?1 AND predicate=?2 AND object=?3
         ORDER BY valid_from DESC",
        )
        .map_err(|e| format!("prepare: {}", e))?;

    let rows = stmt
        .query_map(rusqlite::params![subject, predicate, object], |row| {
            Ok(TemporalFact {
                fact_id: row.get(0)?,
                subject: row.get(1)?,
                predicate: row.get(2)?,
                object: row.get(3)?,
                valid_from: row.get(4)?,
                valid_to: row.get(5)?,
                confidence: row.get(6)?,
                source: row.get(7)?,
            })
        })
        .map_err(|e| format!("query: {}", e))?;

    let mut facts = Vec::new();
    for row in rows {
        facts.push(row.map_err(|e| format!("row: {}", e))?);
    }
    Ok(facts)
}

/// Get the subgraph as of a specific time. Filters edges that have temporal
/// constraints: if a temporal fact exists for (source_id, relation_type, target_id),
/// only includes the edge if the fact was valid at `as_of`. Edges without
/// a corresponding temporal fact are always included (backward compatible).
pub fn subgraph_as_of(
    conn: &Connection,
    center_id: &str,
    depth: usize,
    as_of: i64,
) -> Result<
    (
        Vec<super::super::nt_memory_types::KnowledgeNode>,
        Vec<super::super::nt_memory_types::KnowledgeEdge>,
    ),
    String,
> {
    let mut node_ids = HashSet::new();
    let mut edge_ids = HashSet::new();
    let mut frontier = VecDeque::new();

    node_ids.insert(center_id.to_string());
    frontier.push_back((center_id.to_string(), 0));

    while let Some((current_id, current_depth)) = frontier.pop_front() {
        if current_depth >= depth {
            continue;
        }
        let edges = super::super::nt_memory_store::get_edges_for_node(conn, &current_id)
            .map_err(|e| format!("get_edges: {}", e))?;
        for edge in &edges {
            // Check temporal validity of this edge
            let mut stmt = conn
                .prepare(
                    "SELECT 1 FROM temporal_facts
                 WHERE subject=?1 AND predicate=?2 AND object=?3
                   AND valid_from <= ?4 AND (valid_to IS NULL OR valid_to > ?4)
                 LIMIT 1",
                )
                .map_err(|e| format!("prepare temporal check: {}", e))?;

            let valid_now: bool = stmt
                .query_map(
                    rusqlite::params![
                        edge.source_id,
                        edge.relation_type.as_str(),
                        edge.target_id,
                        as_of
                    ],
                    |_| Ok(true),
                )
                .map_err(|e| format!("query temporal: {}", e))?
                .next()
                .is_some();

            // Check if ANY temporal fact exists for this triple (regardless of time)
            let any_temporal: bool = {
                let mut s = conn.prepare(
                    "SELECT 1 FROM temporal_facts WHERE subject=?1 AND predicate=?2 AND object=?3 LIMIT 1"
                ).map_err(|e| format!("prepare any_temporal: {}", e))?;
                let result = s
                    .query_map(
                        rusqlite::params![
                            edge.source_id,
                            edge.relation_type.as_str(),
                            edge.target_id
                        ],
                        |_| Ok(true),
                    )
                    .map_err(|e| format!("query any_temporal: {}", e))?
                    .next()
                    .is_some();
                result
            };

            // Include edge if:
            //   - no temporal facts exist for this triple (backward compat), OR
            //   - the fact is valid at as_of
            if !any_temporal || valid_now {
                let neighbor = if edge.source_id == current_id {
                    &edge.target_id
                } else {
                    &edge.source_id
                };
                edge_ids.insert(edge.id.clone());
                if node_ids.insert(neighbor.to_string()) {
                    frontier.push_back((neighbor.to_string(), current_depth + 1));
                }
            }
        }
    }

    let mut nodes = Vec::new();
    for nid in &node_ids {
        if let Some(node) = super::super::nt_memory_store::get_node(conn, nid)
            .map_err(|e| format!("get_node: {}", e))?
        {
            nodes.push(node);
        }
    }

    let mut edges = Vec::new();
    for eid in &edge_ids {
        let mut stmt = conn.prepare(
            "SELECT id, source_id, target_id, relation_type, weight, description, created_at, metadata,
                    version, superseded_by
             FROM edges WHERE id=?1",
        ).map_err(|e| format!("prepare edge: {}", e))?;
        let mut rows = stmt
            .query_map(rusqlite::params![eid], |row| {
                Ok(super::super::nt_memory_types::KnowledgeEdge {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    target_id: row.get(2)?,
                    relation_type: super::super::nt_memory_types::RelationType::from_str(
                        &row.get::<_, String>(3)?,
                    ),
                    weight: row.get(4)?,
                    description: row.get(5)?,
                    created_at: row.get(6)?,
                    metadata: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|m| serde_json::from_str(&m).ok()),
                    version: row.get::<_, i64>(8)? as u64,
                    superseded_by: row.get(9)?,
                })
            })
            .map_err(|e| format!("query edge: {}", e))?;
        if let Some(row) = rows.next() {
            edges.push(row.map_err(|e| format!("edge row: {}", e))?);
        }
    }

    Ok((nodes, edges))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS temporal_facts (
                fact_id TEXT PRIMARY KEY,
                subject TEXT NOT NULL,
                predicate TEXT NOT NULL,
                object TEXT NOT NULL,
                valid_from INTEGER NOT NULL,
                valid_to INTEGER,
                confidence REAL DEFAULT 1.0,
                source TEXT DEFAULT 'neotrix',
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_temporal_subject ON temporal_facts(subject);
            CREATE INDEX IF NOT EXISTS idx_temporal_object ON temporal_facts(object);
            CREATE INDEX IF NOT EXISTS idx_temporal_valid ON temporal_facts(valid_from, valid_to);",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_insert_and_query_valid_fact() {
        let conn = setup_test_db();
        let fact = TemporalFact {
            fact_id: "f1".into(),
            subject: "Einstein".into(),
            predicate: "developed".into(),
            object: "Relativity".into(),
            valid_from: 1000,
            valid_to: None,
            confidence: 0.95,
            source: "test".into(),
        };
        insert_temporal_fact(&conn, &fact).unwrap();

        let results = query_temporal_facts(
            &conn,
            &TemporalQuery {
                subject: Some("Einstein".into()),
                predicate: None,
                object: None,
                as_of: None,
            },
            10,
        )
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].fact_id, "f1");
        assert_eq!(results[0].object, "Relativity");
    }

    #[test]
    fn test_invalidate_fact() {
        let conn = setup_test_db();
        let fact = TemporalFact {
            fact_id: "f2".into(),
            subject: "Newton".into(),
            predicate: "wrote".into(),
            object: "Principia".into(),
            valid_from: 500,
            valid_to: None,
            confidence: 0.9,
            source: "test".into(),
        };
        insert_temporal_fact(&conn, &fact).unwrap();

        invalidate_fact(&conn, "f2").unwrap();

        let results = query_temporal_facts(
            &conn,
            &TemporalQuery {
                subject: Some("Newton".into()),
                predicate: None,
                object: None,
                as_of: None,
            },
            10,
        )
        .unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_query_as_of_timestamp() {
        let conn = setup_test_db();
        let fact = TemporalFact {
            fact_id: "f3".into(),
            subject: "Darwin".into(),
            predicate: "proposed".into(),
            object: "Evolution".into(),
            valid_from: 1500,
            valid_to: None,
            confidence: 0.8,
            source: "test".into(),
        };
        insert_temporal_fact(&conn, &fact).unwrap();

        invalidate_fact(&conn, "f3").unwrap();

        let results = query_temporal_facts(
            &conn,
            &TemporalQuery {
                subject: Some("Darwin".into()),
                predicate: None,
                object: None,
                as_of: Some(1600),
            },
            10,
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].fact_id, "f3");
    }

    #[test]
    fn test_query_as_of_after_invalidation() {
        let conn = setup_test_db();
        let fact = TemporalFact {
            fact_id: "f4".into(),
            subject: "Darwin".into(),
            predicate: "proposed".into(),
            object: "Evolution".into(),
            valid_from: 1500,
            valid_to: None,
            confidence: 0.8,
            source: "test".into(),
        };
        insert_temporal_fact(&conn, &fact).unwrap();

        invalidate_fact(&conn, "f4").unwrap();

        let now = chrono::Utc::now().timestamp() + 100;
        let results = query_temporal_facts(
            &conn,
            &TemporalQuery {
                subject: Some("Darwin".into()),
                predicate: None,
                object: None,
                as_of: Some(now),
            },
            10,
        )
        .unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_get_fact_history() {
        let conn = setup_test_db();

        let fact_v1 = TemporalFact {
            fact_id: "f5_v1".into(),
            subject: "Mendeleev".into(),
            predicate: "created".into(),
            object: "PeriodicTable".into(),
            valid_from: 1800,
            valid_to: Some(1850),
            confidence: 0.7,
            source: "test".into(),
        };
        let fact_v2 = TemporalFact {
            fact_id: "f5_v2".into(),
            subject: "Mendeleev".into(),
            predicate: "created".into(),
            object: "PeriodicTable".into(),
            valid_from: 1850,
            valid_to: None,
            confidence: 0.95,
            source: "test".into(),
        };
        insert_temporal_fact(&conn, &fact_v1).unwrap();
        insert_temporal_fact(&conn, &fact_v2).unwrap();

        let history = get_fact_history(&conn, "Mendeleev", "created", "PeriodicTable").unwrap();
        assert_eq!(history.len(), 2);
        assert!(history[0].valid_from > history[1].valid_from);
    }

    #[test]
    fn test_multiple_facts_same_subject() {
        let conn = setup_test_db();

        insert_temporal_fact(
            &conn,
            &TemporalFact {
                fact_id: "m1".into(),
                subject: "Tesla".into(),
                predicate: "invented".into(),
                object: "ACMotor".into(),
                valid_from: 1880,
                valid_to: None,
                confidence: 0.9,
                source: "test".into(),
            },
        )
        .unwrap();

        insert_temporal_fact(
            &conn,
            &TemporalFact {
                fact_id: "m2".into(),
                subject: "Tesla".into(),
                predicate: "invented".into(),
                object: "Radio".into(),
                valid_from: 1890,
                valid_to: None,
                confidence: 0.85,
                source: "test".into(),
            },
        )
        .unwrap();

        insert_temporal_fact(
            &conn,
            &TemporalFact {
                fact_id: "m3".into(),
                subject: "Tesla".into(),
                predicate: "worked_at".into(),
                object: "Edison".into(),
                valid_from: 1882,
                valid_to: Some(1885),
                confidence: 0.75,
                source: "test".into(),
            },
        )
        .unwrap();

        // Filter by predicate "invented"
        let invented = query_temporal_facts(
            &conn,
            &TemporalQuery {
                subject: Some("Tesla".into()),
                predicate: Some("invented".into()),
                object: None,
                as_of: None,
            },
            10,
        )
        .unwrap();
        assert_eq!(invented.len(), 2);

        // Filter by object "Radio"
        let radio = query_temporal_facts(
            &conn,
            &TemporalQuery {
                subject: Some("Tesla".into()),
                predicate: None,
                object: Some("Radio".into()),
                as_of: None,
            },
            10,
        )
        .unwrap();
        assert_eq!(radio.len(), 1);
        assert_eq!(radio[0].fact_id, "m2");
    }
}
