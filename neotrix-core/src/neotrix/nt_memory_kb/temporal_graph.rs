use std::collections::HashSet;

use rusqlite::{params, Connection};

use super::nt_memory_store::{get_edges_for_node, get_node};
use super::nt_memory_types::*;

#[derive(Debug, Clone)]
pub struct TemporalSnapshot {
    pub id: String,
    pub timestamp: i64,
    pub label: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub version: u64,
}

#[derive(Debug, Clone)]
pub struct VersionDiff {
    pub from_version: u64,
    pub to_version: u64,
    pub nodes_added: Vec<String>,
    pub nodes_removed: Vec<String>,
    pub nodes_modified: Vec<String>,
    pub edges_added: Vec<String>,
    pub edges_removed: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TemporalQueryOptions {
    pub as_of: Option<i64>,
    pub from_version: Option<u64>,
    pub to_version: Option<u64>,
    pub include_superseded: bool,
    pub node_types: Vec<NodeType>,
    pub max_depth: usize,
}

impl Default for TemporalQueryOptions {
    fn default() -> Self {
        Self {
            as_of: None,
            from_version: None,
            to_version: None,
            include_superseded: false,
            node_types: Vec::new(),
            max_depth: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TemporalStats {
    pub total_facts: usize,
    pub active_facts: usize,
    pub version_count: u64,
    pub earliest_timestamp: i64,
    pub latest_timestamp: i64,
    pub most_updated_subject: String,
}

// ─── Schema ─────────────────────────────────────────────────────────

fn ensure_temporal_metadata_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS temporal_metadata (
            key TEXT PRIMARY KEY,
            version INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS temporal_snapshots (
            id TEXT PRIMARY KEY,
            timestamp INTEGER NOT NULL,
            label TEXT NOT NULL,
            node_count INTEGER NOT NULL DEFAULT 0,
            edge_count INTEGER NOT NULL DEFAULT 0,
            version INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS temporal_edge_map (
            edge_id TEXT PRIMARY KEY,
            temporal_fact_id TEXT NOT NULL,
            valid_from INTEGER NOT NULL,
            valid_to INTEGER,
            FOREIGN KEY (edge_id) REFERENCES edges(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_temporal_edge_map_valid
            ON temporal_edge_map(valid_from, valid_to);

        INSERT OR IGNORE INTO temporal_metadata (key, version) VALUES ('schema_version', 1);",
    )
    .map_err(|e| format!("temporal metadata schema: {}", e))?;
    Ok(())
}

fn current_version(conn: &Connection) -> Result<u64, String> {
    let v: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM temporal_snapshots",
            [],
            |r| r.get(0),
        )
        .map_err(|e| format!("current_version: {}", e))?;
    Ok(v as u64)
}

// ─── Core Functions ─────────────────────────────────────────────────

pub fn create_snapshot(conn: &Connection, label: &str) -> Result<TemporalSnapshot, String> {
    ensure_temporal_metadata_schema(conn)?;
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp();
    let node_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))
        .map_err(|e| format!("count nodes: {}", e))?;
    let edge_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0))
        .map_err(|e| format!("count edges: {}", e))?;
    let version = current_version(conn)? + 1;

    conn.execute(
        "INSERT INTO temporal_snapshots (id, timestamp, label, node_count, edge_count, version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, timestamp, label, node_count, edge_count, version as i64],
    )
    .map_err(|e| format!("insert snapshot: {}", e))?;

    Ok(TemporalSnapshot {
        id,
        timestamp,
        label: label.to_string(),
        node_count: node_count as usize,
        edge_count: edge_count as usize,
        version,
    })
}

pub fn list_snapshots(conn: &Connection) -> Result<Vec<TemporalSnapshot>, String> {
    ensure_temporal_metadata_schema(conn)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, timestamp, label, node_count, edge_count, version
             FROM temporal_snapshots ORDER BY version DESC",
        )
        .map_err(|e| format!("list snaps prepare: {}", e))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(TemporalSnapshot {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                label: row.get(2)?,
                node_count: row.get::<_, i64>(3)? as usize,
                edge_count: row.get::<_, i64>(4)? as usize,
                version: row.get::<_, i64>(5)? as u64,
            })
        })
        .map_err(|e| format!("list snaps query: {}", e))?;
    let mut snapshots = Vec::new();
    for r in rows {
        snapshots.push(r.map_err(|e| format!("snap row: {}", e))?);
    }
    Ok(snapshots)
}

pub fn diff_versions(
    conn: &Connection,
    from_version: u64,
    to_version: u64,
) -> Result<VersionDiff, String> {
    ensure_temporal_metadata_schema(conn)?;

    // Capture all node IDs at each version boundary by snapshot
    let nodes_at = |_ver: u64| -> Result<HashSet<String>, String> {
        // Simplified: use snapshot node_count as heuristic, but collect all nodes
        let mut set = HashSet::new();
        let mut rows = conn
            .prepare("SELECT id FROM nodes")
            .map_err(|e| format!("nodes query: {}", e))?;
        let iter = rows
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("nodes iter: {}", e))?;
        for r in iter {
            set.insert(r.map_err(|e| format!("node id: {}", e))?);
        }
        Ok(set)
    };

    let nodes_before = nodes_at(from_version)?;
    let nodes_after = nodes_at(to_version)?;

    let mut nodes_added = Vec::new();
    let mut nodes_removed = Vec::new();
    let mut nodes_modified = Vec::new();

    for id in &nodes_after {
        if !nodes_before.contains(id) {
            nodes_added.push(id.clone());
        } else {
            // Check if version changed
            let v: i64 = conn
                .query_row("SELECT version FROM nodes WHERE id=?1", params![id], |r| {
                    r.get(0)
                })
                .map_err(|e| format!("node version: {}", e))?;
            if (v as u64) > from_version {
                nodes_modified.push(id.clone());
            }
        }
    }
    for id in &nodes_before {
        if !nodes_after.contains(id) {
            nodes_removed.push(id.clone());
        }
    }

    // Edge changes
    let _edges_before: HashSet<String> = {
        let mut stmt = conn
            .prepare("SELECT id FROM edges")
            .map_err(|e| format!("edges prepare: {}", e))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("edges query: {}", e))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    // Since we don't track edge versions separately we use the temporal_edge_map
    let mut edges_added = Vec::new();
    let mut edges_removed = Vec::new();

    // Edges active in to_version but not in from_version
    let active_facts_from: HashSet<String> = active_edge_ids_at(conn, from_version as i64)?;
    let active_facts_to: HashSet<String> = active_edge_ids_at(conn, to_version as i64)?;

    for eid in &active_facts_to {
        if !active_facts_from.contains(eid) {
            edges_added.push(eid.clone());
        }
    }
    for eid in &active_facts_from {
        if !active_facts_to.contains(eid) {
            edges_removed.push(eid.clone());
        }
    }

    Ok(VersionDiff {
        from_version,
        to_version,
        nodes_added,
        nodes_removed,
        nodes_modified,
        edges_added,
        edges_removed,
    })
}

fn active_edge_ids_at(conn: &Connection, timestamp: i64) -> Result<HashSet<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT edge_id FROM temporal_edge_map
             WHERE valid_from <= ?1 AND (valid_to IS NULL OR valid_to > ?1)",
        )
        .map_err(|e| format!("active edge ids prepare: {}", e))?;
    let rows = stmt
        .query_map(params![timestamp], |row| row.get::<_, String>(0))
        .map_err(|e| format!("active edge ids query: {}", e))?;
    let mut ids = HashSet::new();
    for r in rows {
        ids.insert(r.map_err(|e| format!("active edge row: {}", e))?);
    }
    Ok(ids)
}

pub fn query_as_of(
    conn: &Connection,
    center_id: &str,
    opts: &TemporalQueryOptions,
) -> Result<(Vec<KnowledgeNode>, Vec<KnowledgeEdge>), String> {
    let as_of = opts.as_of.unwrap_or_else(|| chrono::Utc::now().timestamp());
    use std::collections::VecDeque;

    let mut node_ids = HashSet::new();
    let mut edge_ids = HashSet::new();
    let mut frontier = VecDeque::new();

    // If center_id matches a specific node type filter, check it
    if !opts.node_types.is_empty() {
        if let Some(center) = get_node(conn, center_id).map_err(|e| format!("get center: {}", e))? {
            if !opts.node_types.contains(&center.node_type) {
                return Ok((Vec::new(), Vec::new()));
            }
        }
    }

    node_ids.insert(center_id.to_string());
    frontier.push_back((center_id.to_string(), 0));

    while let Some((current_id, depth)) = frontier.pop_front() {
        if depth >= opts.max_depth {
            continue;
        }
        let edges =
            get_edges_for_node(conn, &current_id).map_err(|e| format!("get_edges: {}", e))?;

        for edge in &edges {
            // Check temporal validity
            let valid = is_temporal_edge_valid(conn, &edge.id, as_of)?;
            if !valid {
                continue;
            }

            let neighbor = if edge.source_id == current_id {
                &edge.target_id
            } else {
                &edge.source_id
            };

            edge_ids.insert(edge.id.clone());
            if node_ids.insert(neighbor.to_string()) {
                frontier.push_back((neighbor.to_string(), depth + 1));
            }
        }
    }

    let mut nodes = Vec::new();
    for nid in &node_ids {
        if let Some(node) = get_node(conn, nid).map_err(|e| format!("get_node: {}", e))? {
            if !opts.node_types.is_empty() && !opts.node_types.contains(&node.node_type) {
                continue;
            }
            nodes.push(node);
        }
    }

    let mut edges = Vec::new();
    for eid in &edge_ids {
        let mut stmt = conn
            .prepare(
                "SELECT id, source_id, target_id, relation_type, weight, description, created_at,
                        metadata, version, superseded_by
                 FROM edges WHERE id=?1",
            )
            .map_err(|e| format!("prepare edge: {}", e))?;
        let mut rows = stmt
            .query_map(params![eid], |row| {
                Ok(KnowledgeEdge {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    target_id: row.get(2)?,
                    relation_type: RelationType::from_str(&row.get::<_, String>(3)?),
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

fn is_temporal_edge_valid(conn: &Connection, edge_id: &str, as_of: i64) -> Result<bool, String> {
    // Check if edge has temporal mapping
    let any_temporal: bool = conn
        .query_row(
            "SELECT 1 FROM temporal_edge_map WHERE edge_id=?1 LIMIT 1",
            params![edge_id],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if !any_temporal {
        return Ok(true); // backward compatible
    }

    let valid: bool = conn
        .query_row(
            "SELECT 1 FROM temporal_edge_map
             WHERE edge_id=?1 AND valid_from <= ?2 AND (valid_to IS NULL OR valid_to > ?2)
             LIMIT 1",
            params![edge_id, as_of],
            |_| Ok(true),
        )
        .unwrap_or(false);

    Ok(valid)
}

pub fn node_history(conn: &Connection, node_id: &str) -> Result<Vec<KnowledgeNode>, String> {
    let node = get_node(conn, node_id)
        .map_err(|e| format!("get_node: {}", e))?
        .ok_or_else(|| format!("node not found: {}", node_id))?;

    let mut history = vec![node];

    // Walk superseded_by chain
    let mut current_id = node_id.to_string();
    loop {
        let sid: Option<String> = conn
            .query_row(
                "SELECT superseded_by FROM nodes WHERE id=?1",
                params![current_id],
                |r| r.get(0),
            )
            .map_err(|e| format!("superseded_by query: {}", e))?;

        match sid {
            Some(next_id) if !next_id.is_empty() && next_id != current_id => {
                if let Some(next_node) =
                    get_node(conn, &next_id).map_err(|e| format!("get_node chain: {}", e))?
                {
                    history.push(next_node);
                    current_id = next_id;
                } else {
                    break;
                }
            }
            _ => break,
        }
    }

    Ok(history)
}

pub fn temporal_stats(conn: &Connection) -> Result<TemporalStats, String> {
    ensure_temporal_metadata_schema(conn)?;

    let total_facts: i64 = conn
        .query_row("SELECT COUNT(*) FROM temporal_facts", [], |r| r.get(0))
        .map_err(|e| format!("total facts: {}", e))?;

    let active_facts: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM temporal_facts WHERE valid_to IS NULL",
            [],
            |r| r.get(0),
        )
        .map_err(|e| format!("active facts: {}", e))?;

    let version_count = current_version(conn)?;

    let earliest: i64 = conn
        .query_row(
            "SELECT COALESCE(MIN(valid_from), 0) FROM temporal_facts",
            [],
            |r| r.get(0),
        )
        .map_err(|e| format!("earliest: {}", e))?;

    let latest: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(valid_from), 0) FROM temporal_facts",
            [],
            |r| r.get(0),
        )
        .map_err(|e| format!("latest: {}", e))?;

    let most_updated: String = conn
        .query_row(
            "SELECT COALESCE(
                (SELECT subject FROM temporal_facts
                 GROUP BY subject ORDER BY COUNT(*) DESC LIMIT 1),
                ''
            )",
            [],
            |r| r.get(0),
        )
        .map_err(|e| format!("most_updated: {}", e))?;

    Ok(TemporalStats {
        total_facts: total_facts as usize,
        active_facts: active_facts as usize,
        version_count,
        earliest_timestamp: earliest,
        latest_timestamp: latest,
        most_updated_subject: most_updated,
    })
}

pub fn graph_at_version(
    conn: &Connection,
    version: u64,
) -> Result<(Vec<String>, Vec<String>), String> {
    ensure_temporal_metadata_schema(conn)?;

    // Get snapshot nearest to this version
    let snap: Option<(i64, i64)> = conn
        .query_row(
            "SELECT timestamp, version FROM temporal_snapshots
             WHERE version <= ?1 ORDER BY version DESC LIMIT 1",
            params![version as i64],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();

    let as_of = match snap {
        Some((ts, _)) => ts,
        None => chrono::Utc::now().timestamp(),
    };

    // Get all edges active at that time
    let active_edges = active_edge_ids_at(conn, as_of)?;

    let node_ids: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT id FROM nodes")
            .map_err(|e| format!("nodes prepare: {}", e))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("nodes query: {}", e))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    Ok((node_ids, active_edges.into_iter().collect()))
}

pub fn compact_temporal_facts(conn: &Connection, older_than: i64) -> Result<usize, String> {
    ensure_temporal_metadata_schema(conn)?;

    // Archive: move facts older than `older_than` to a separate table
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS temporal_facts_archive (
            fact_id TEXT PRIMARY KEY,
            subject TEXT NOT NULL,
            predicate TEXT NOT NULL,
            object TEXT NOT NULL,
            valid_from INTEGER NOT NULL,
            valid_to INTEGER,
            confidence REAL DEFAULT 1.0,
            source TEXT DEFAULT 'neotrix',
            created_at INTEGER NOT NULL,
            archived_at INTEGER NOT NULL
        );",
    )
    .map_err(|e| format!("archive table: {}", e))?;

    let now = chrono::Utc::now().timestamp();
    let moved = conn
        .execute(
            "INSERT OR IGNORE INTO temporal_facts_archive
             (fact_id, subject, predicate, object, valid_from, valid_to, confidence, source, created_at, archived_at)
             SELECT fact_id, subject, predicate, object, valid_from, valid_to, confidence, source, created_at, ?1
             FROM temporal_facts WHERE valid_to IS NOT NULL AND valid_to < ?2",
            params![now, older_than],
        )
        .map_err(|e| format!("archive insert: {}", e))?;

    conn.execute(
        "DELETE FROM temporal_facts WHERE valid_to IS NOT NULL AND valid_to < ?1",
        params![older_than],
    )
    .map_err(|e| format!("archive delete: {}", e))?;

    // Also compact temporal_edge_map
    let edge_moved = conn
        .execute(
            "DELETE FROM temporal_edge_map WHERE valid_to IS NOT NULL AND valid_to < ?1",
            params![older_than],
        )
        .map_err(|e| format!("compact edge map: {}", e))?;

    Ok((moved + edge_moved) as usize)
}

pub fn insert_temporal_edge(
    conn: &Connection,
    edge: &KnowledgeEdge,
    valid_from: i64,
    valid_to: Option<i64>,
) -> Result<(), String> {
    ensure_temporal_metadata_schema(conn)?;

    // Insert the edge into the standard edges table
    super::nt_memory_store::insert_edge(conn, edge).map_err(|e| format!("insert edge: {}", e))?;

    // Create a temporal fact
    let fact_id = format!("te_{}", edge.id);
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT OR REPLACE INTO temporal_facts
         (fact_id, subject, predicate, object, valid_from, valid_to, confidence, source, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            fact_id,
            edge.source_id,
            edge.relation_type.as_str(),
            edge.target_id,
            valid_from,
            valid_to,
            edge.weight,
            "temporal_graph",
            now,
        ],
    )
    .map_err(|e| format!("insert temporal fact: {}", e))?;

    // Map edge to temporal fact
    conn.execute(
        "INSERT OR REPLACE INTO temporal_edge_map (edge_id, temporal_fact_id, valid_from, valid_to)
         VALUES (?1, ?2, ?3, ?4)",
        params![edge.id, fact_id, valid_from, valid_to],
    )
    .map_err(|e| format!("insert edge map: {}", e))?;

    Ok(())
}

pub fn retire_edge(conn: &Connection, edge_id: &str) -> Result<(), String> {
    ensure_temporal_metadata_schema(conn)?;

    let now = chrono::Utc::now().timestamp();
    let affected = conn
        .execute(
            "UPDATE temporal_edge_map SET valid_to=?1
             WHERE edge_id=?2 AND valid_to IS NULL",
            params![now, edge_id],
        )
        .map_err(|e| format!("retire edge map: {}", e))?;

    // Also update the temporal_facts entry
    conn.execute(
        "UPDATE temporal_facts SET valid_to=?1
         WHERE fact_id=?2 AND valid_to IS NULL",
        params![now, format!("te_{}", edge_id)],
    )
    .map_err(|e| format!("retire temporal fact: {}", e))?;

    if affected == 0 {
        return Err(format!("edge {} not found or already retired", edge_id));
    }
    Ok(())
}

pub fn active_edges_at(conn: &Connection, timestamp: i64) -> Result<Vec<KnowledgeEdge>, String> {
    ensure_temporal_metadata_schema(conn)?;

    let active_ids = active_edge_ids_at(conn, timestamp)?;

    // Also include edges without temporal mapping (backward compat)
    // But if any edge has a temporal mapping, respect it
    let mapped_edges: HashSet<String> = {
        let mut stmt = conn
            .prepare("SELECT DISTINCT edge_id FROM temporal_edge_map")
            .map_err(|e| format!("mapped edges: {}", e))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("mapped edges query: {}", e))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let mut all_edges = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT id, source_id, target_id, relation_type, weight, description, created_at,
                    metadata, version, superseded_by
             FROM edges",
        )
        .map_err(|e| format!("all edges prepare: {}", e))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(KnowledgeEdge {
                id: row.get(0)?,
                source_id: row.get(1)?,
                target_id: row.get(2)?,
                relation_type: RelationType::from_str(&row.get::<_, String>(3)?),
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
        .map_err(|e| format!("all edges query: {}", e))?;

    for r in rows {
        let edge = r.map_err(|e| format!("edge row: {}", e))?;
        if mapped_edges.contains(&edge.id) {
            if active_ids.contains(&edge.id) {
                all_edges.push(edge);
            }
        } else {
            all_edges.push(edge);
        }
    }

    Ok(all_edges)
}

// ─── VSA Version Tracking ───────────────────────────────────────────

pub fn track_vsa_version(conn: &Connection, entity_id: &str, vsa_hash: &str) -> Result<(), String> {
    ensure_temporal_metadata_schema(conn)?;
    conn.execute(
        "INSERT OR REPLACE INTO temporal_metadata (key, version)
         VALUES (?1, ?2)",
        params![format!("vsa_{}", entity_id), vsa_hash],
    )
    .map_err(|e| format!("track vsa version: {}", e))?;
    Ok(())
}

pub fn get_vsa_version(conn: &Connection, entity_id: &str) -> Result<Option<String>, String> {
    let result: Result<String, _> = conn.query_row(
        "SELECT version FROM temporal_metadata WHERE key=?1",
        params![format!("vsa_{}", entity_id)],
        |r| r.get(0),
    );
    match result {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("get vsa version: {}", e)),
    }
}

// ─── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Connection {
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

            CREATE TABLE IF NOT EXISTS nodes (
                id TEXT PRIMARY KEY,
                node_type TEXT NOT NULL DEFAULT 'concept',
                title TEXT NOT NULL DEFAULT '',
                summary TEXT,
                content TEXT,
                url TEXT,
                domain TEXT,
                language TEXT DEFAULT 'en',
                confidence REAL DEFAULT 1.0,
                importance REAL DEFAULT 0.5,
                created_at INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL DEFAULT 0,
                access_count INTEGER DEFAULT 0,
                metadata TEXT,
                version INTEGER NOT NULL DEFAULT 1,
                superseded_by TEXT
            );

            CREATE TABLE IF NOT EXISTS edges (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL,
                target_id TEXT NOT NULL,
                relation_type TEXT NOT NULL,
                weight REAL DEFAULT 1.0,
                description TEXT,
                created_at INTEGER NOT NULL DEFAULT 0,
                metadata TEXT,
                version INTEGER NOT NULL DEFAULT 1,
                superseded_by TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id);
            CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id);",
        )
        .unwrap();
        ensure_temporal_metadata_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_create_snapshot_and_list() {
        let conn = setup_db();
        let snap = create_snapshot(&conn, "test-snap").unwrap();
        assert_eq!(snap.label, "test-snap");
        assert!(snap.version >= 1);

        let snaps = list_snapshots(&conn).unwrap();
        assert!(!snaps.is_empty());
        assert_eq!(snaps[0].id, snap.id);
    }

    #[test]
    fn test_diff_versions_identical() {
        let conn = setup_db();
        let diff = diff_versions(&conn, 0, 1).unwrap();
        assert_eq!(diff.from_version, 0);
        assert_eq!(diff.to_version, 1);
    }

    #[test]
    fn test_query_as_of_basic() {
        let conn = setup_db();
        conn.execute(
            "INSERT INTO nodes (id, node_type, title, created_at, updated_at, version)
             VALUES ('n1', 'concept', 'Node1', 1000, 1000, 1)",
            [],
        )
        .unwrap();

        let opts = TemporalQueryOptions {
            max_depth: 1,
            ..Default::default()
        };
        let (nodes, edges) = query_as_of(&conn, "n1", &opts).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "n1");
        assert!(edges.is_empty());
    }

    #[test]
    fn test_node_history_version_chain() {
        let conn = setup_db();
        conn.execute(
            "INSERT INTO nodes (id, node_type, title, created_at, updated_at, version, superseded_by)
             VALUES ('n1', 'concept', 'V1', 1000, 1000, 1, 'n2')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO nodes (id, node_type, title, created_at, updated_at, version)
             VALUES ('n2', 'concept', 'V2', 1000, 1000, 2)",
            [],
        )
        .unwrap();

        let history = node_history(&conn, "n1").unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].id, "n1");
        assert_eq!(history[1].id, "n2");
    }

    #[test]
    fn test_temporal_stats_empty() {
        let conn = setup_db();
        let stats = temporal_stats(&conn).unwrap();
        assert_eq!(stats.total_facts, 0);
        assert_eq!(stats.active_facts, 0);
    }

    #[test]
    fn test_insert_temporal_edge_and_retire() {
        let conn = setup_db();
        conn.execute(
            "INSERT INTO nodes (id, node_type, title, created_at, updated_at, version)
             VALUES ('s1', 'concept', 'Source', 1000, 1000, 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO nodes (id, node_type, title, created_at, updated_at, version)
             VALUES ('t1', 'concept', 'Target', 1000, 1000, 1)",
            [],
        )
        .unwrap();

        let edge = KnowledgeEdge {
            id: "e1".into(),
            source_id: "s1".into(),
            target_id: "t1".into(),
            relation_type: RelationType::References,
            weight: 1.0,
            description: Some("test edge".into()),
            created_at: 1000,
            metadata: None,
            version: 1,
            superseded_by: None,
        };

        insert_temporal_edge(&conn, &edge, 1000, None).unwrap();

        // Should be active at future time
        let active = active_edges_at(&conn, 2000).unwrap();
        assert!(active.iter().any(|e| e.id == "e1"));

        retire_edge(&conn, "e1").unwrap();

        // Should no longer be active
        let active = active_edges_at(&conn, 99999).unwrap();
        assert!(!active.iter().any(|e| e.id == "e1"));
    }

    #[test]
    fn test_active_edges_at() {
        let conn = setup_db();
        conn.execute(
            "INSERT INTO nodes (id, node_type, title, created_at, updated_at, version)
             VALUES ('a1', 'concept', 'A', 0, 0, 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO nodes (id, node_type, title, created_at, updated_at, version)
             VALUES ('a2', 'concept', 'B', 0, 0, 1)",
            [],
        )
        .unwrap();

        let edge = KnowledgeEdge {
            id: "e_active".into(),
            source_id: "a1".into(),
            target_id: "a2".into(),
            relation_type: RelationType::Related,
            weight: 1.0,
            description: None,
            created_at: 0,
            metadata: None,
            version: 1,
            superseded_by: None,
        };
        insert_temporal_edge(&conn, &edge, 500, Some(1500)).unwrap();

        let at_1000 = active_edges_at(&conn, 1000).unwrap();
        assert!(at_1000.iter().any(|e| e.id == "e_active"));

        let at_2000 = active_edges_at(&conn, 2000).unwrap();
        assert!(!at_2000.iter().any(|e| e.id == "e_active"));
    }

    #[test]
    fn test_compact_temporal_facts() {
        let conn = setup_db();
        conn.execute(
            "INSERT INTO temporal_facts (fact_id, subject, predicate, object, valid_from, valid_to, confidence, source, created_at)
             VALUES ('old', 'subj', 'pred', 'obj', 100, 200, 1.0, 'test', 100)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO temporal_facts (fact_id, subject, predicate, object, valid_from, valid_to, confidence, source, created_at)
             VALUES ('current', 'subj', 'pred', 'obj', 1000, NULL, 1.0, 'test', 1000)",
            [],
        )
        .unwrap();

        let count = compact_temporal_facts(&conn, 500).unwrap();
        assert!(count >= 1);

        // Old fact should be gone
        let remaining: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM temporal_facts WHERE fact_id='old'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(remaining, 0);

        // Current fact should remain
        let current: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM temporal_facts WHERE fact_id='current'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(current, 1);
    }

    #[test]
    fn test_graph_at_version() {
        let conn = setup_db();
        create_snapshot(&conn, "v1-snap").unwrap();
        // After snapshot, graph state is tracked
        let (nodes, edges) = graph_at_version(&conn, 1).unwrap();
        assert!(nodes.is_empty());
        assert!(edges.is_empty());
    }

    #[test]
    fn test_vsa_version_tracking() {
        let conn = setup_db();
        track_vsa_version(&conn, "entity_42", "abc123def").unwrap();
        let v = get_vsa_version(&conn, "entity_42").unwrap();
        assert_eq!(v, Some("abc123def".into()));

        let missing = get_vsa_version(&conn, "nonexistent").unwrap();
        assert_eq!(missing, None);
    }

    #[test]
    fn test_query_as_of_with_node_type_filter() {
        let conn = setup_db();
        conn.execute(
            "INSERT INTO nodes (id, node_type, title, created_at, updated_at, version)
             VALUES ('n_paper', 'paper', 'Paper1', 1000, 1000, 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO nodes (id, node_type, title, created_at, updated_at, version)
             VALUES ('n_concept', 'concept', 'Concept1', 1000, 1000, 1)",
            [],
        )
        .unwrap();

        let opts = TemporalQueryOptions {
            node_types: vec![NodeType::Paper],
            max_depth: 1,
            ..Default::default()
        };
        let (nodes, _) = query_as_of(&conn, "n_paper", &opts).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "n_paper");

        // Center node with wrong type should return empty
        let (nodes, _) = query_as_of(&conn, "n_concept", &opts).unwrap();
        assert_eq!(nodes.len(), 0);
    }
}
