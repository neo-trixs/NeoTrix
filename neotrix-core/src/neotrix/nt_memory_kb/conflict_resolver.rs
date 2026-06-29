use std::collections::HashSet;

use rusqlite::Connection;

use super::nt_memory_types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ConflictPair {
    pub node_a_id: String,
    pub node_b_id: String,
    pub similarity: f64,
    pub confidence_a: f64,
    pub confidence_b: f64,
    pub timestamp_a: i64,
    pub timestamp_b: i64,
    pub source_a: String,
    pub source_b: String,
    pub severity: ConflictSeverity,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionStrategy {
    KeepNewer,
    KeepHigherConfidence,
    KeepSourcePriority(Vec<String>),
    Merge,
    MarkBoth,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionAction {
    SupersedeAbyB,
    SupersedeBbyA,
    MergeIntoNew(String),
    KeepBothFlagged,
    DeleteBoth,
}

#[derive(Debug, Clone)]
pub struct ConflictReport {
    pub pairs: Vec<ConflictPair>,
    pub total_count: usize,
    pub resolved_count: usize,
    pub auto_resolved_count: usize,
    pub pending_count: usize,
    pub strategies_used: Vec<ResolutionStrategy>,
}

pub struct ConflictResolver {
    pub max_similarity_threshold: f64,
    pub default_strategy: ResolutionStrategy,
    pub auto_resolve: bool,
}

impl ConflictResolver {
    pub fn new() -> Self {
        Self {
            max_similarity_threshold: 0.85,
            default_strategy: ResolutionStrategy::KeepHigherConfidence,
            auto_resolve: true,
        }
    }

    pub fn with_threshold(mut self, t: f64) -> Self {
        self.max_similarity_threshold = t;
        self
    }

    pub fn with_strategy(mut self, s: ResolutionStrategy) -> Self {
        self.default_strategy = s;
        self
    }

    pub fn with_auto_resolve(mut self, b: bool) -> Self {
        self.auto_resolve = b;
        self
    }
}

fn title_similarity(a: &str, b: &str) -> f64 {
    let words_a: HashSet<&str> = a
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty() && w.len() > 2)
        .collect();
    let words_b: HashSet<&str> = b
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty() && w.len() > 2)
        .collect();
    if words_a.is_empty() && words_b.is_empty() {
        return if a.to_lowercase() == b.to_lowercase() {
            1.0
        } else {
            0.0
        };
    }
    if words_a.is_empty() || words_b.is_empty() {
        return 0.0;
    }
    let intersection: HashSet<&&str> = words_a.intersection(&words_b).collect();
    let union: HashSet<&&str> = words_a.union(&words_b).collect();
    let jaccard = intersection.len() as f64 / union.len() as f64;
    let lower_a = a.to_lowercase();
    let lower_b = b.to_lowercase();
    if lower_a == lower_b {
        return 1.0f64.max(jaccard);
    }
    if lower_a.contains(&lower_b) || lower_b.contains(&lower_a) {
        return 0.9f64.max(jaccard);
    }
    jaccard
}

fn confidence_score(node: &KnowledgeNode) -> f64 {
    node.confidence * (1.0 + node.importance * 0.5)
}

fn conflict_severity(sim: f64, conf_a: f64, conf_b: f64) -> ConflictSeverity {
    if sim > 0.95 || (sim > 0.8 && (conf_a - conf_b).abs() > 0.3) {
        ConflictSeverity::Critical
    } else if sim > 0.7 {
        ConflictSeverity::Warning
    } else {
        ConflictSeverity::Info
    }
}

fn log_conflict(
    conn: &Connection,
    pair: &ConflictPair,
    action: &ResolutionAction,
) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO conflict_log
         (node_a_id, node_b_id, similarity, severity, action_taken, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            pair.node_a_id,
            pair.node_b_id,
            pair.similarity,
            format!("{:?}", pair.severity),
            format!("{:?}", action),
            chrono::Utc::now().timestamp(),
        ],
    )
    .map_err(|e| format!("log conflict: {}", e))?;
    Ok(())
}

fn ensure_conflict_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS conflict_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            node_a_id TEXT NOT NULL,
            node_b_id TEXT NOT NULL,
            similarity REAL NOT NULL,
            severity TEXT NOT NULL,
            action_taken TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );",
    )
    .map_err(|e| format!("conflict schema: {}", e))?;
    Ok(())
}

pub fn detect_node_conflicts(
    _conn: &Connection,
    nodes: &[KnowledgeNode],
    threshold: f64,
) -> Result<Vec<ConflictPair>, String> {
    let mut pairs = Vec::new();
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let a = &nodes[i];
            let b = &nodes[j];
            let sim = title_similarity(&a.title, &b.title);

            if sim < threshold && !(a.url.is_some() && b.url.is_some() && a.url == b.url) {
                continue;
            }

            let url_match = a.url.is_some() && b.url.is_some() && a.url == b.url;
            let effective_sim = if url_match { sim.max(0.9) } else { sim };

            let conf_a = confidence_score(a);
            let conf_b = confidence_score(b);
            let severity = conflict_severity(effective_sim, conf_a, conf_b);

            let description = if url_match {
                format!(
                    "Same URL '{}' with different content",
                    a.url.as_deref().unwrap_or("")
                )
            } else {
                format!(
                    "Similar titles: '{}' vs '{}' (sim={:.2})",
                    a.title, b.title, effective_sim
                )
            };

            pairs.push(ConflictPair {
                node_a_id: a.id.clone(),
                node_b_id: b.id.clone(),
                similarity: effective_sim,
                confidence_a: conf_a,
                confidence_b: conf_b,
                timestamp_a: a.created_at,
                timestamp_b: b.created_at,
                source_a: a.domain.clone().unwrap_or_default(),
                source_b: b.domain.clone().unwrap_or_default(),
                severity,
                description,
            });
        }
    }
    Ok(pairs)
}

pub fn detect_edge_conflicts(
    _conn: &Connection,
    edges: &[KnowledgeEdge],
) -> Result<Vec<ConflictPair>, String> {
    let mut pairs = Vec::new();
    for i in 0..edges.len() {
        for j in (i + 1)..edges.len() {
            let a = &edges[i];
            let b = &edges[j];
            if a.source_id == b.source_id && a.target_id == b.target_id {
                if a.relation_type != b.relation_type {
                    pairs.push(ConflictPair {
                        node_a_id: a.id.clone(),
                        node_b_id: b.id.clone(),
                        similarity: 1.0,
                        confidence_a: a.weight,
                        confidence_b: b.weight,
                        timestamp_a: a.created_at,
                        timestamp_b: b.created_at,
                        source_a: "edge".to_string(),
                        source_b: "edge".to_string(),
                        severity: ConflictSeverity::Warning,
                        description: format!(
                            "Edge conflict: {:?} vs {:?} on same nodes",
                            a.relation_type, b.relation_type
                        ),
                    });
                }
            }
            if a.source_id == b.target_id && a.target_id == b.source_id {
                pairs.push(ConflictPair {
                    node_a_id: a.id.clone(),
                    node_b_id: b.id.clone(),
                    similarity: 0.9,
                    confidence_a: a.weight,
                    confidence_b: b.weight,
                    timestamp_a: a.created_at,
                    timestamp_b: b.created_at,
                    source_a: "edge".to_string(),
                    source_b: "edge".to_string(),
                    severity: ConflictSeverity::Info,
                    description: format!(
                        "Reverse edges: {:?}({}) vs {:?}({})",
                        a.relation_type, a.source_id, b.relation_type, b.target_id
                    ),
                });
            }
        }
    }
    Ok(pairs)
}

pub fn scan_all_conflicts(
    conn: &Connection,
    resolver: &ConflictResolver,
) -> Result<ConflictReport, String> {
    ensure_conflict_schema(conn)?;

    let all_nodes: Vec<KnowledgeNode> = {
        let mut stmt = conn
            .prepare(
                "SELECT id, node_type, title, summary, content, url, domain, language,
                    confidence, importance, created_at, updated_at, access_count,
                    metadata, version, superseded_by
             FROM nodes",
            )
            .map_err(|e| format!("prepare nodes: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
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
            })
            .map_err(|e| format!("query nodes: {}", e))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let pairs = detect_node_conflicts(conn, &all_nodes, resolver.max_similarity_threshold)?;

    let total_count = pairs.len();
    let mut resolved_count = 0;
    let mut auto_resolved_count = 0;
    let mut strategies_used = Vec::new();
    let mut remaining = Vec::new();

    if resolver.auto_resolve {
        for pair in &pairs {
            let action = resolve_conflict(conn, pair, &resolver.default_strategy)?;
            log_conflict(conn, pair, &action)?;
            resolved_count += 1;
            auto_resolved_count += 1;
            strategies_used.push(resolver.default_strategy.clone());
        }
    } else {
        remaining = pairs;
    }

    Ok(ConflictReport {
        pairs: remaining,
        total_count,
        resolved_count,
        auto_resolved_count,
        pending_count: total_count - resolved_count,
        strategies_used,
    })
}

pub fn resolve_conflict(
    conn: &Connection,
    pair: &ConflictPair,
    strategy: &ResolutionStrategy,
) -> Result<ResolutionAction, String> {
    match strategy {
        ResolutionStrategy::KeepNewer => {
            if pair.timestamp_a >= pair.timestamp_b {
                supersede_node(conn, &pair.node_b_id, &pair.node_a_id)?;
                Ok(ResolutionAction::SupersedeBbyA)
            } else {
                supersede_node(conn, &pair.node_a_id, &pair.node_b_id)?;
                Ok(ResolutionAction::SupersedeAbyB)
            }
        }
        ResolutionStrategy::KeepHigherConfidence => {
            if pair.confidence_a >= pair.confidence_b {
                supersede_node(conn, &pair.node_b_id, &pair.node_a_id)?;
                Ok(ResolutionAction::SupersedeBbyA)
            } else {
                supersede_node(conn, &pair.node_a_id, &pair.node_b_id)?;
                Ok(ResolutionAction::SupersedeAbyB)
            }
        }
        ResolutionStrategy::KeepSourcePriority(sources) => {
            let a_priority = sources
                .iter()
                .position(|s| s == &pair.source_a)
                .unwrap_or(usize::MAX);
            let b_priority = sources
                .iter()
                .position(|s| s == &pair.source_b)
                .unwrap_or(usize::MAX);
            if a_priority <= b_priority {
                supersede_node(conn, &pair.node_b_id, &pair.node_a_id)?;
                Ok(ResolutionAction::SupersedeBbyA)
            } else {
                supersede_node(conn, &pair.node_a_id, &pair.node_b_id)?;
                Ok(ResolutionAction::SupersedeAbyB)
            }
        }
        ResolutionStrategy::Merge => {
            let merged_title = format!("{} / {}", pair.node_a_id, pair.node_b_id);
            Ok(ResolutionAction::MergeIntoNew(merged_title))
        }
        ResolutionStrategy::MarkBoth => Ok(ResolutionAction::KeepBothFlagged),
    }
}

fn supersede_node(conn: &Connection, old_id: &str, new_id: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE nodes SET superseded_by=?1 WHERE id=?2",
        rusqlite::params![new_id, old_id],
    )
    .map_err(|e| format!("supersede node: {}", e))?;
    Ok(())
}

pub fn batch_resolve(
    conn: &Connection,
    pairs: &[ConflictPair],
    resolver: &ConflictResolver,
) -> Result<ConflictReport, String> {
    ensure_conflict_schema(conn)?;
    let mut resolved_count = 0;
    let mut strategies_used = Vec::new();

    for pair in pairs {
        let action = resolve_conflict(conn, pair, &resolver.default_strategy)?;
        log_conflict(conn, pair, &action)?;
        resolved_count += 1;
        strategies_used.push(resolver.default_strategy.clone());
    }

    Ok(ConflictReport {
        pairs: pairs.to_vec(),
        total_count: pairs.len(),
        resolved_count,
        auto_resolved_count: resolved_count,
        pending_count: 0,
        strategies_used,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_similarity_exact_match() {
        let sim = title_similarity("Machine Learning Basics", "Machine Learning Basics");
        assert!((sim - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_title_similarity_partial() {
        let sim = title_similarity("ML Basics", "Machine Learning Basics");
        assert!(sim > 0.0 && sim < 1.0);
    }

    #[test]
    fn test_title_similarity_no_match() {
        let sim = title_similarity("Quantum Physics", "Cooking Recipes");
        assert!(sim < 0.3);
    }

    #[test]
    fn test_confidence_score() {
        let node = KnowledgeNode {
            id: "test".into(),
            node_type: NodeType::Concept,
            title: "Test".into(),
            summary: None,
            content: None,
            url: None,
            domain: None,
            language: "en".into(),
            confidence: 0.8,
            importance: 0.5,
            created_at: 0,
            updated_at: 0,
            access_count: 0,
            metadata: None,
            version: 1,
            superseded_by: None,
        };
        let score = confidence_score(&node);
        assert!((score - 0.8 * 1.25).abs() < 0.01);
    }

    fn node(
        id: &str,
        title: &str,
        node_type: NodeType,
        url: Option<String>,
        domain: Option<String>,
        confidence: f64,
        importance: f64,
        created_at: i64,
    ) -> KnowledgeNode {
        KnowledgeNode {
            id: id.into(),
            node_type,
            title: title.into(),
            summary: None,
            content: None,
            url,
            domain,
            language: "en".into(),
            confidence,
            importance,
            created_at,
            updated_at: created_at,
            access_count: 0,
            metadata: None,
            version: 1,
            superseded_by: None,
        }
    }

    fn edge(
        id: &str,
        source_id: &str,
        target_id: &str,
        relation_type: RelationType,
        weight: f64,
        created_at: i64,
    ) -> KnowledgeEdge {
        KnowledgeEdge {
            id: id.into(),
            source_id: source_id.into(),
            target_id: target_id.into(),
            relation_type,
            weight,
            description: None,
            created_at,
            metadata: None,
            version: 1,
            superseded_by: None,
        }
    }

    #[test]
    fn test_detect_node_conflicts_no_match() {
        let nodes = vec![
            node(
                "a1",
                "Quantum Physics",
                NodeType::Concept,
                None,
                Some("wiki".into()),
                0.9,
                0.5,
                100,
            ),
            node(
                "b1",
                "Cooking Recipes",
                NodeType::Concept,
                None,
                Some("cook".into()),
                0.8,
                0.3,
                200,
            ),
        ];
        let conn = Connection::open_in_memory().unwrap();
        let pairs = detect_node_conflicts(&conn, &nodes, 0.85).unwrap();
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_detect_node_conflicts_same_url() {
        let nodes = vec![
            node(
                "a1",
                "Article v1",
                NodeType::Article,
                Some("https://example.com/article".into()),
                None,
                0.9,
                0.5,
                100,
            ),
            node(
                "b1",
                "Article v2",
                NodeType::Article,
                Some("https://example.com/article".into()),
                None,
                0.7,
                0.3,
                200,
            ),
        ];
        let conn = Connection::open_in_memory().unwrap();
        let pairs = detect_node_conflicts(&conn, &nodes, 0.85).unwrap();
        assert_eq!(pairs.len(), 1);
        assert!(pairs[0].description.contains("Same URL"));
    }

    #[test]
    fn test_resolve_keep_newer() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE nodes (id TEXT PRIMARY KEY, title TEXT, node_type TEXT, confidence REAL,
             importance REAL, created_at INTEGER, updated_at INTEGER, access_count INTEGER,
             version INTEGER DEFAULT 1, superseded_by TEXT, url TEXT, domain TEXT, language TEXT,
             summary TEXT, content TEXT, metadata TEXT);",
        )
        .unwrap();
        conn.execute("INSERT INTO nodes (id, title, node_type, created_at, version) VALUES ('old', 'Old', 'concept', 100, 1)", []).unwrap();
        conn.execute("INSERT INTO nodes (id, title, node_type, created_at, version) VALUES ('new', 'New', 'concept', 200, 1)", []).unwrap();

        let pair = ConflictPair {
            node_a_id: "old".into(),
            node_b_id: "new".into(),
            similarity: 0.9,
            confidence_a: 0.5,
            confidence_b: 0.8,
            timestamp_a: 100,
            timestamp_b: 200,
            source_a: "src1".into(),
            source_b: "src2".into(),
            severity: ConflictSeverity::Warning,
            description: "test".into(),
        };

        let action = resolve_conflict(&conn, &pair, &ResolutionStrategy::KeepNewer).unwrap();
        assert_eq!(action, ResolutionAction::SupersedeAbyB);

        let superseded: Option<String> = conn
            .query_row("SELECT superseded_by FROM nodes WHERE id='old'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(superseded, Some("new".to_string()));
    }

    #[test]
    fn test_resolve_keep_higher_confidence() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE nodes (id TEXT PRIMARY KEY, title TEXT, node_type TEXT, confidence REAL,
             importance REAL, created_at INTEGER, updated_at INTEGER, access_count INTEGER,
             version INTEGER DEFAULT 1, superseded_by TEXT, url TEXT, domain TEXT, language TEXT,
             summary TEXT, content TEXT, metadata TEXT);",
        )
        .unwrap();
        conn.execute("INSERT INTO nodes (id, title, node_type, confidence, version) VALUES ('low', 'Low', 'concept', 0.3, 1)", []).unwrap();
        conn.execute("INSERT INTO nodes (id, title, node_type, confidence, version) VALUES ('high', 'High', 'concept', 0.9, 1)", []).unwrap();

        let pair = ConflictPair {
            node_a_id: "low".into(),
            node_b_id: "high".into(),
            similarity: 0.9,
            confidence_a: 0.3,
            confidence_b: 0.9,
            timestamp_a: 100,
            timestamp_b: 200,
            source_a: "src1".into(),
            source_b: "src2".into(),
            severity: ConflictSeverity::Critical,
            description: "test".into(),
        };

        let action =
            resolve_conflict(&conn, &pair, &ResolutionStrategy::KeepHigherConfidence).unwrap();
        assert_eq!(action, ResolutionAction::SupersedeAbyB);
    }

    #[test]
    fn test_resolve_source_priority() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE nodes (id TEXT PRIMARY KEY, title TEXT, node_type TEXT, confidence REAL,
             importance REAL, created_at INTEGER, updated_at INTEGER, access_count INTEGER,
             version INTEGER DEFAULT 1, superseded_by TEXT, url TEXT, domain TEXT, language TEXT,
             summary TEXT, content TEXT, metadata TEXT);",
        )
        .unwrap();
        conn.execute("INSERT INTO nodes (id, title, node_type, version) VALUES ('from_web', 'Web', 'concept', 1)", []).unwrap();
        conn.execute("INSERT INTO nodes (id, title, node_type, version) VALUES ('from_paper', 'Paper', 'concept', 1)", []).unwrap();

        let pair = ConflictPair {
            node_a_id: "from_web".into(),
            node_b_id: "from_paper".into(),
            similarity: 0.9,
            confidence_a: 0.5,
            confidence_b: 0.5,
            timestamp_a: 100,
            timestamp_b: 200,
            source_a: "web".into(),
            source_b: "paper".into(),
            severity: ConflictSeverity::Info,
            description: "test".into(),
        };

        let action = resolve_conflict(
            &conn,
            &pair,
            &ResolutionStrategy::KeepSourcePriority(vec!["paper".into(), "web".into()]),
        )
        .unwrap();
        // paper has higher priority (index 0), so web should be superseded by paper
        assert_eq!(action, ResolutionAction::SupersedeAbyB);

        let superseded: Option<String> = conn
            .query_row(
                "SELECT superseded_by FROM nodes WHERE id='from_web'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(superseded, Some("from_paper".to_string()));
    }

    #[test]
    fn test_scan_all_conflicts_empty_db() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE nodes (id TEXT PRIMARY KEY, node_type TEXT, title TEXT, url TEXT, domain TEXT,
             language TEXT DEFAULT 'en', confidence REAL DEFAULT 1.0, importance REAL DEFAULT 0.5,
             created_at INTEGER DEFAULT 0, updated_at INTEGER DEFAULT 0, access_count INTEGER DEFAULT 0,
             metadata TEXT, version INTEGER DEFAULT 1, superseded_by TEXT,
             summary TEXT, content TEXT);"
        ).unwrap();

        let resolver = ConflictResolver::new();
        let report = scan_all_conflicts(&conn, &resolver).unwrap();
        assert_eq!(report.total_count, 0);
    }

    #[test]
    fn test_detect_edge_conflicts_different_relation() {
        let edges = vec![
            edge("e1", "n1", "n2", RelationType::References, 1.0, 100),
            edge("e2", "n1", "n2", RelationType::Contradicts, 0.5, 200),
        ];
        let conn = Connection::open_in_memory().unwrap();
        let pairs = detect_edge_conflicts(&conn, &edges).unwrap();
        assert!(!pairs.is_empty());
        assert!(pairs[0].description.contains("Edge conflict"));
    }
}
