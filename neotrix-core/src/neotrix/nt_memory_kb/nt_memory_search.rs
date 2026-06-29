use rusqlite::{params, Connection};

use super::nt_memory_types::*;

pub fn search_fts(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> rusqlite::Result<Vec<KnowledgeSearchResult>> {
    let mut stmt = conn.prepare(
        "SELECT n.id, n.node_type, n.title, n.summary, n.content, n.url, n.domain,
                n.language, n.confidence, n.importance, n.created_at, n.updated_at,
                n.access_count, n.metadata,
                rank, n.version, n.superseded_by
         FROM nodes_fts f
         JOIN nodes n ON n.rowid = f.rowid
         WHERE nodes_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(params![query, limit as i64], |row| {
        Ok(KnowledgeSearchResult {
            node: KnowledgeNode {
                id: row.get(0)?,
                node_type: NodeType::from_str(&row.get::<_, String>(1)?),
                title: row.get(2)?,
                summary: row.get(3)?,
                content: row.get(4)?,
                url: row.get(5)?,
                domain: row.get(6)?,
                language: row.get(7)?,
                confidence: row.get(8)?,
                importance: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
                access_count: row.get(12)?,
                metadata: row
                    .get::<_, Option<String>>(13)?
                    .and_then(|m| serde_json::from_str(&m).ok()),
                version: row.get::<_, i64>(15)? as u64,
                superseded_by: row.get(16)?,
            },
            score: 1.0 - row.get::<_, f64>(14)?,
            matched_on: vec![SearchMatchType::FtsTitle],
        })
    })?;

    rows.collect()
}

pub fn search_by_type(
    conn: &Connection,
    node_type: &NodeType,
    limit: usize,
) -> rusqlite::Result<Vec<KnowledgeNode>> {
    let mut stmt = conn.prepare(
        "SELECT id, node_type, title, summary, content, url, domain, language,
            confidence, importance, created_at, updated_at, access_count, metadata,
            version, superseded_by
         FROM nodes
         WHERE node_type=?1
         ORDER BY importance DESC, access_count DESC
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(params![node_type.as_str(), limit as i64], |row| {
        Ok(KnowledgeNode {
            id: row.get(0)?,
            node_type: NodeType::from_str(&row.get::<_, String>(1)?),
            title: row.get(2)?,
            summary: row.get(3)?,
            content: row.get(4)?,
            url: row.get(5)?,
            domain: row.get(6)?,
            language: row.get(7)?,
            confidence: row.get(8)?,
            importance: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            access_count: row.get(12)?,
            metadata: row
                .get::<_, Option<String>>(13)?
                .and_then(|m| serde_json::from_str(&m).ok()),
            version: row.get::<_, i64>(14)? as u64,
            superseded_by: row.get(15)?,
        })
    })?;

    rows.collect()
}

pub fn get_related(
    conn: &Connection,
    node_id: &str,
    relation_type: Option<&str>,
    limit: usize,
) -> rusqlite::Result<Vec<KnowledgeSearchResult>> {
    let (sql, has_relation) = if let Some(_rt) = relation_type {
        (
            format!(
                "SELECT n.id, n.node_type, n.title, n.summary, n.content, n.url, n.domain,
                n.language, n.confidence, n.importance, n.created_at, n.updated_at,
                n.access_count, n.metadata, e.weight as score,
                n.version, n.superseded_by
             FROM edges e
             JOIN nodes n ON n.id = CASE WHEN e.source_id=?1 THEN e.target_id ELSE e.source_id END
             WHERE (e.source_id=?1 OR e.target_id=?1) AND e.relation_type=?2
             ORDER BY e.weight DESC
             LIMIT ?3"
            ),
            true,
        )
    } else {
        (
            format!(
                "SELECT n.id, n.node_type, n.title, n.summary, n.content, n.url, n.domain,
                n.language, n.confidence, n.importance, n.created_at, n.updated_at,
                n.access_count, n.metadata, e.weight as score,
                n.version, n.superseded_by
             FROM edges e
             JOIN nodes n ON n.id = CASE WHEN e.source_id=?1 THEN e.target_id ELSE e.source_id END
             WHERE e.source_id=?1 OR e.target_id=?1
             ORDER BY e.weight DESC
             LIMIT ?2"
            ),
            false,
        )
    };

    let mut stmt = conn.prepare(&sql)?;
    let rows: Vec<KnowledgeSearchResult> = if has_relation {
        stmt.query_map(params![node_id, relation_type, limit as i64], |row| {
            Ok(KnowledgeSearchResult {
                node: KnowledgeNode {
                    id: row.get(0)?,
                    node_type: NodeType::from_str(&row.get::<_, String>(1)?),
                    title: row.get(2)?,
                    summary: row.get(3)?,
                    content: row.get(4)?,
                    url: row.get(5)?,
                    domain: row.get(6)?,
                    language: row.get(7)?,
                    confidence: row.get(8)?,
                    importance: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                    access_count: row.get(12)?,
                    metadata: row
                        .get::<_, Option<String>>(13)?
                        .and_then(|m| serde_json::from_str(&m).ok()),
                    version: row.get::<_, i64>(15)? as u64,
                    superseded_by: row.get(16)?,
                },
                score: row.get(14)?,
                matched_on: vec![SearchMatchType::GraphRelation],
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    } else {
        stmt.query_map(params![node_id, limit as i64], |row| {
            Ok(KnowledgeSearchResult {
                node: KnowledgeNode {
                    id: row.get(0)?,
                    node_type: NodeType::from_str(&row.get::<_, String>(1)?),
                    title: row.get(2)?,
                    summary: row.get(3)?,
                    content: row.get(4)?,
                    url: row.get(5)?,
                    domain: row.get(6)?,
                    language: row.get(7)?,
                    confidence: row.get(8)?,
                    importance: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                    access_count: row.get(12)?,
                    metadata: row
                        .get::<_, Option<String>>(13)?
                        .and_then(|m| serde_json::from_str(&m).ok()),
                    version: row.get::<_, i64>(15)? as u64,
                    superseded_by: row.get(16)?,
                },
                score: row.get(14)?,
                matched_on: vec![SearchMatchType::GraphRelation],
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    };

    Ok(rows)
}

pub fn hybrid_search(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> rusqlite::Result<Vec<KnowledgeSearchResult>> {
    let fts_results = search_fts(conn, query, limit)?;
    if fts_results.len() >= limit {
        return Ok(fts_results);
    }

    let remaining = limit - fts_results.len();
    let mut stmt = conn.prepare(
        "SELECT id, node_type, title, summary, content, url, domain, language,
            confidence, importance, created_at, updated_at, access_count, metadata,
            version, superseded_by
         FROM nodes
         WHERE title LIKE ?1
         ORDER BY importance DESC
         LIMIT ?2",
    )?;

    let pattern = format!("%{}%", query);
    let rows = stmt.query_map(params![pattern, remaining as i64], |row| {
        Ok(KnowledgeSearchResult {
            node: KnowledgeNode {
                id: row.get(0)?,
                node_type: NodeType::from_str(&row.get::<_, String>(1)?),
                title: row.get(2)?,
                summary: row.get(3)?,
                content: row.get(4)?,
                url: row.get(5)?,
                domain: row.get(6)?,
                language: row.get(7)?,
                confidence: row.get(8)?,
                importance: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
                access_count: row.get(12)?,
                metadata: row
                    .get::<_, Option<String>>(13)?
                    .and_then(|m| serde_json::from_str(&m).ok()),
                version: row.get::<_, i64>(14)? as u64,
                superseded_by: row.get(15)?,
            },
            score: 0.1,
            matched_on: vec![SearchMatchType::FtsTitle],
        })
    })?;

    let mut results = fts_results;
    for r in rows.collect::<Vec<_>>().into_iter().filter_map(|r| r.ok()) {
        if !results.iter().any(|res| res.node.id == r.node.id) {
            results.push(r);
        }
    }
    Ok(results)
}
