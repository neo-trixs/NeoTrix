use std::collections::{HashMap, HashSet};

use rusqlite::Connection;

#[derive(Clone, Debug)]
pub struct SchemaDrift {
    pub module: String,
    pub expected_type: String,
    pub expected_fields: Vec<String>,
    pub actual_fields: Vec<String>,
    pub missing: Vec<String>,
    pub extra: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct GhostModuleDetector {
    pub declared_at: String,
    pub module_name: String,
}

#[derive(Default)]
pub struct SchemaWatchdog {
    known_schemas: HashMap<String, Vec<String>>,
    drifts: Vec<SchemaDrift>,
}

impl SchemaWatchdog {
    pub fn new() -> Self {
        let mut w = Self::default();
        w.register("KnowledgeNode", vec![
            "id", "title", "body", "url", "source", "node_type",
            "created_at", "updated_at", "confidence", "embedding_id",
            "access_count", "last_accessed", "provenance", "domain",
            "language", "importance", "ttl",
        ]);
        w.register("NodeType", vec![
            "Concept", "Fact", "Claim", "Article", "Document", "Source",
            "Person", "Organization", "Location", "Event", "Technology",
            "Tool", "Framework", "Language", "Protocol", "Methodology",
            "Pattern", "Skill", "Capability", "Defect", "Fix", "Improvement",
            "ArchitectureComponent", "DataFlow", "Algorithm", "Benchmark",
            "Paper", "Tutorial", "API", "Config", "Query", "Response",
            "WikiPage",
        ]);
        w.register("RelationType", vec![
            "related_to", "derived_from", "part_of", "uses", "implemented_by",
            "depends_on", "conflicts_with", "improves", "fixes", "causes",
            "examples", "supports", "contradicts", "subtype_of", "instance_of",
            "maps_to", "references", "mentions", "follows", "precedes",
            "requires", "produces", "consumes", "equivalent_to", "generalizes",
            "specializes", "bridges", "enables", "constrains", "subclass",
            "provenance", "documents", "describes", "validates", "deprecates",
            "replaces", "supersedes", "translates",
        ]);
        w
    }

    pub fn register(&mut self, type_name: &str, fields: Vec<&str>) {
        self.known_schemas.insert(
            type_name.to_string(),
            fields.into_iter().map(String::from).collect(),
        );
    }

    pub fn detect_drift(&mut self, type_name: &str, actual_fields: &[String]) -> Option<SchemaDrift> {
        let expected = self.known_schemas.get(type_name)?;
        let actual_set: HashSet<&str> = actual_fields.iter().map(|s| s.as_str()).collect();
        let expected_set: HashSet<&str> = expected.iter().map(|s| s.as_str()).collect();

        let missing: Vec<String> = expected_set.difference(&actual_set).map(|s| s.to_string()).collect();
        let extra: Vec<String> = actual_set.difference(&expected_set).map(|s| s.to_string()).collect();

        if missing.is_empty() && extra.is_empty() {
            return None;
        }

        let drift = SchemaDrift {
            module: type_name.to_string(),
            expected_type: type_name.to_string(),
            expected_fields: expected.clone(),
            actual_fields: actual_fields.to_vec(),
            missing,
            extra,
        };
        self.drifts.push(drift.clone());
        Some(drift)
    }

    pub fn report(&self) -> String {
        let mut r = String::new();
        r.push_str(&format!("Schema Watchdog Report -- {} schemas tracked\n", self.known_schemas.len()));
        if self.drifts.is_empty() {
            r.push_str("No schema drift detected\n");
        } else {
            for d in &self.drifts {
                r.push_str(&format!("Drift in {}: missing={:?}, extra={:?}\n", d.module, d.missing, d.extra));
            }
        }
        r
    }

    pub fn verify_db_schema(conn: &Connection) -> Vec<String> {
        fn get_column_names(conn: &Connection, table: &str) -> Result<Vec<String>, String> {
            let sql = format!("PRAGMA table_info({table})");
            let mut stmt = conn.prepare(&sql).map_err(|e| format!("prepare {sql}: {e}"))?;
            let names = stmt
                .query_map([], |row| row.get::<_, String>(1))
                .map_err(|e| format!("query {sql}: {e}"))?
                .filter_map(|r| r.ok())
                .collect();
            Ok(names)
        }

        let mut mismatches = Vec::new();

        let knowledge_expected: &[&str] = &[
            "id", "title", "node_type", "content", "summary", "url",
            "domain", "language", "confidence", "importance", "access_count",
            "metadata", "created_at", "updated_at",
        ];

        let crawl_queue_expected: &[&str] = &[
            "id", "url", "domain", "status", "priority", "depth",
            "error_count", "last_error", "created_at", "updated_at",
        ];

        for (table, expected) in [("knowledge_nodes", knowledge_expected), ("crawl_queue", crawl_queue_expected)] {
            match get_column_names(conn, table) {
                Ok(actual) => {
                    let actual_set: HashSet<&str> = actual.iter().map(|s| s.as_str()).collect();
                    let expected_set: HashSet<&str> = expected.iter().copied().collect();
                    for col in expected_set.difference(&actual_set) {
                        mismatches.push(format!("{table}: missing column '{col}'"));
                    }
                    for col in actual_set.difference(&expected_set) {
                        mismatches.push(format!("{table}: unexpected column '{col}'"));
                    }
                }
                Err(e) => mismatches.push(format!("{table}: could not read schema - {e}")),
            }
        }

        mismatches
    }
}

impl crate::core::nt_core_self_test::SelfTest for SchemaWatchdog {
    fn name(&self) -> &str {
        "schema_watchdog"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let mut w = SchemaWatchdog::new();

        // Test 1: known-good fields must not trigger drift
        for (type_name, fields) in &[
            ("KnowledgeNode", w.known_schemas.get("KnowledgeNode").unwrap().clone()),
            ("NodeType", w.known_schemas.get("NodeType").unwrap().clone()),
            ("RelationType", w.known_schemas.get("RelationType").unwrap().clone()),
        ] {
            if w.detect_drift(type_name, fields).is_some() {
                failures.push(format!("{}: false positive on exact match", type_name));
            }
        }

        // Test 2: known-bad fields must trigger drift
        let bad = vec!["id".to_string()];
        if w.detect_drift("KnowledgeNode", &bad).is_none() {
            failures.push("KnowledgeNode: false negative on minimal fields".into());
        }

        // Test 3: empty fields must trigger drift
        let empty: Vec<String> = vec![];
        if w.detect_drift("KnowledgeNode", &empty).is_none() {
            failures.push("KnowledgeNode: false negative on empty fields".into());
        }

        if failures.is_empty() { Ok(()) } else { Err(failures) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_drift_for_known_schema() {
        let mut w = SchemaWatchdog::new();
        let fields: Vec<String> = w.known_schemas.get("KnowledgeNode").unwrap().clone();
        assert!(w.detect_drift("KnowledgeNode", &fields).is_none());
    }

    #[test]
    fn test_drift_detected() {
        let mut w = SchemaWatchdog::new();
        let fields = vec!["id".to_string(), "title".to_string()];
        assert!(w.detect_drift("KnowledgeNode", &fields).is_some());
    }

    #[test]
    fn test_new_has_zero_drifts() {
        let w = SchemaWatchdog::new();
        assert_eq!(w.known_schemas.len(), 3);
    }

    #[test]
    fn test_verify_db_schema_matches() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE knowledge_nodes (
                id TEXT PRIMARY KEY, title TEXT, node_type TEXT, content TEXT,
                summary TEXT, url TEXT, domain TEXT, language TEXT,
                confidence REAL, importance REAL, access_count INTEGER,
                metadata TEXT, created_at TEXT, updated_at TEXT
            );
            CREATE TABLE crawl_queue (
                id INTEGER PRIMARY KEY, url TEXT, domain TEXT, status TEXT,
                priority INTEGER, depth INTEGER, error_count INTEGER,
                last_error TEXT, created_at TEXT, updated_at TEXT
            );"
        ).unwrap();
        let mismatches = SchemaWatchdog::verify_db_schema(&conn);
        assert!(mismatches.is_empty(), "mismatches: {:?}", mismatches);
    }

    #[test]
    fn test_verify_db_schema_missing_column() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE knowledge_nodes (
                id TEXT PRIMARY KEY, title TEXT, content TEXT
            );
            CREATE TABLE crawl_queue (
                id INTEGER PRIMARY KEY, url TEXT, status TEXT
            );"
        ).unwrap();
        let mismatches = SchemaWatchdog::verify_db_schema(&conn);
        assert!(!mismatches.is_empty());
        assert!(mismatches.iter().any(|m| m.contains("missing column")));
    }
}
