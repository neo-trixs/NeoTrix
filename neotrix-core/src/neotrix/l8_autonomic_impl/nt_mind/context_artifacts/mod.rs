pub mod types;
pub mod store;
pub mod indexer;

pub use types::{ArtifactType, Artifact, ArtifactSourceConfig, ArtifactsConfig};
pub use store::{ArtifactStore, ArtifactBuilder};
pub use indexer::ArtifactIndexer;

pub fn inject_relevant_artifacts(
    store: &ArtifactStore,
    query: &str,
    max_results: usize,
) -> Vec<Artifact> {
    let mut scored: Vec<(f64, &Artifact)> = Vec::new();

    for artifact in store.all() {
        let mut score = 0.0;
        let q = query.to_lowercase();

        if artifact.content.to_lowercase().contains(&q) {
            score += 3.0;
        }
        if artifact.name.to_lowercase().contains(&q) {
            score += 2.0;
        }
        for tag in &artifact.tags {
            if q.contains(&tag.to_lowercase()) {
                score += 1.5;
            }
        }
        let query_words: Vec<&str> = q.split_whitespace().collect();
        let content_lower = artifact.content.to_lowercase();
        let match_count = query_words
            .iter()
            .filter(|w| content_lower.contains(*w))
            .count();
        if match_count > 0 {
            score += 0.5 * match_count as f64;
        }

        if score > 0.0 {
            scored.push((score, artifact));
        }
    }

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    scored
        .into_iter()
        .take(max_results)
        .map(|(_, a)| a.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve() {
        let mut store = ArtifactStore::new();
        let art = Artifact::new(
            "users_table",
            ArtifactType::DatabaseSchema,
            "CREATE TABLE users",
        )
        .with_tags(&["database", "sql"])
        .with_source("schema.sql");
        store.store(art);

        let retrieved = store.get("users_table").expect("should find artifact");
        assert_eq!(retrieved.name, "users_table");
        assert_eq!(retrieved.artifact_type, ArtifactType::DatabaseSchema);
        assert!(retrieved.tags.contains(&"database".to_string()));
        assert_eq!(retrieved.source_path, Some("schema.sql".to_string()));
    }

    #[test]
    fn test_search_by_tag() {
        let mut store = ArtifactStore::new();
        store.store(Artifact::new("a", ArtifactType::DatabaseSchema, "").with_tags(&["db"]));
        store.store(Artifact::new("b", ArtifactType::ApiSpec, "").with_tags(&["api"]));
        store.store(Artifact::new("c", ArtifactType::ConfigFile, "").with_tags(&["db", "config"]));

        assert_eq!(store.search_by_tag("db").len(), 2);
        assert_eq!(store.search_by_tag("api").len(), 1);
    }

    #[test]
    fn test_search_by_tags_and() {
        let mut store = ArtifactStore::new();
        store.store(
            Artifact::new("a", ArtifactType::DatabaseSchema, "").with_tags(&["db", "sql", "prod"]),
        );
        store.store(Artifact::new("b", ArtifactType::ConfigFile, "").with_tags(&["db", "config"]));

        let results = store.search_by_tags(&["db", "sql"]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "a");
    }

    #[test]
    fn test_search_keyword() {
        let mut store = ArtifactStore::new();
        store.store(Artifact::new(
            "payment_api",
            ArtifactType::ApiSpec,
            "POST /payments creates a payment",
        ));
        store.store(Artifact::new(
            "users_db",
            ArtifactType::DatabaseSchema,
            "CREATE TABLE users (id INT)",
        ));
        store.store(Artifact::new(
            "deploy_config",
            ArtifactType::ConfigFile,
            "host: production",
        ));

        assert_eq!(store.search_keyword("payment").len(), 1);
        assert_eq!(store.search_keyword("TABLE").len(), 1);
    }

    #[test]
    fn test_search_combined() {
        let mut store = ArtifactStore::new();
        store.store(
            Artifact::new("api_v1", ArtifactType::ApiSpec, "GET /v1/users")
                .with_tags(&["api", "v1"]),
        );
        store.store(
            Artifact::new("api_v2", ArtifactType::ApiSpec, "GET /v2/users")
                .with_tags(&["api", "v2"]),
        );
        store.store(
            Artifact::new("db_schema", ArtifactType::DatabaseSchema, "users table")
                .with_tags(&["db"]),
        );

        assert_eq!(
            store.search(Some(ArtifactType::ApiSpec), None, None).len(),
            2
        );
        assert_eq!(
            store
                .search(Some(ArtifactType::ApiSpec), Some("v2"), None)
                .len(),
            1
        );
        assert_eq!(store.search(None, Some("users"), Some("api")).len(), 2);
    }

    #[test]
    fn test_remove_artifact() {
        let mut store = ArtifactStore::new();
        store.store(Artifact::new("x", ArtifactType::ConfigFile, ""));
        assert!(store.get("x").is_some());
        store.remove("x");
        assert!(store.get("x").is_none());
    }

    #[test]
    fn test_parse_sql_create_table() {
        let sql = r#"
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    user_id INT REFERENCES users(id),
    total DECIMAL(10,2),
    status VARCHAR(50)
);
"#;
        let artifact = ArtifactBuilder::parse_sql("schema", sql, None).expect("should parse SQL");
        assert_eq!(artifact.artifact_type, ArtifactType::DatabaseSchema);
        assert!(artifact.content.contains("TABLE users"));
        assert!(artifact.content.contains("TABLE orders"));
        assert!(artifact.content.contains("email: VARCHAR"));
        assert!(artifact.tags.contains(&"sql".to_string()));
    }

    #[test]
    fn test_parse_sql_empty() {
        assert!(ArtifactBuilder::parse_sql("empty", "-- no tables", None).is_none());
    }

    #[test]
    fn test_parse_sql_with_if_not_exists() {
        let sql = "CREATE TABLE IF NOT EXISTS config (key TEXT, value TEXT);";
        let artifact =
            ArtifactBuilder::parse_sql("cfg", sql, None).expect("should parse IF NOT EXISTS");
        assert!(artifact.content.contains("TABLE config"));
        assert!(artifact.content.contains("key: TEXT"));
    }

    #[test]
    fn test_parse_openapi_basic() {
        let yaml = r#"
openapi: "3.0.0"
info:
  title: Payment API
  description: Process payments
paths:
  /payments:
    get:
      summary: List payments
      parameters:
        - name: page
          in: query
    post:
      summary: Create payment
  /users:
    get:
      summary: List users
"#;
        let artifact = ArtifactBuilder::parse_openapi_yaml("payment_api", yaml, None)
            .expect("should parse OpenAPI YAML");
        assert_eq!(artifact.artifact_type, ArtifactType::ApiSpec);
        assert!(artifact.content.contains("GET /payments"));
        assert!(artifact.content.contains("POST /payments"));
        assert!(artifact.content.contains("GET /users"));
        assert!(artifact.tags.contains(&"api".to_string()));
    }

    #[test]
    fn test_parse_openapi_fallback() {
        let yaml = r#"
swagger: "2.0"
info:
  title: Simple API
paths:
  /items:
    get:
      summary: Get items
    post:
      summary: Create item
"#;
        let artifact = ArtifactBuilder::parse_openapi_yaml("simple", yaml, None)
            .expect("should parse with line-based fallback");
        assert!(artifact.content.contains("/items"));
    }

    #[test]
    fn test_parse_openapi_no_endpoints() {
        assert!(ArtifactBuilder::parse_openapi_yaml("empty", "just: text", None).is_none());
    }

    #[test]
    fn test_parse_markdown_headings() {
        let md = r#"# Project Title

## Architecture

The system uses a microservices architecture.

## Database

PostgreSQL with 3 tables.

## Deployment

Uses Docker and Kubernetes.
"#;
        let artifact = ArtifactBuilder::parse_markdown("readme", md, None);
        assert_eq!(artifact.artifact_type, ArtifactType::ArchitectureDoc);
        assert!(artifact.content.contains("Architecture"));
        assert!(artifact.content.contains("Database"));
        assert!(artifact.content.contains("Deployment"));
    }

    #[test]
    fn test_parse_yaml_config() {
        let yaml = r#"
server:
  host: localhost
  port: 8080
database:
  url: postgres://db:5432
  pool: 10
"#;
        let artifact = ArtifactBuilder::parse_config("deploy", yaml, "yaml", None);
        assert_eq!(artifact.artifact_type, ArtifactType::ConfigFile);
        assert!(artifact.content.contains("host"));
        assert!(artifact.content.contains("port"));
        assert!(artifact.tags.contains(&"yaml".to_string()));
    }

    #[test]
    fn test_parse_toml_config() {
        let toml = r#"
[server]
host = "localhost"
port = 8080

[database]
url = "postgres://db:5432"
max_connections = 10
"#;
        let artifact = ArtifactBuilder::parse_config("config", toml, "toml", None);
        assert_eq!(artifact.artifact_type, ArtifactType::ConfigFile);
        assert!(artifact.content.contains("[server]"));
        assert!(artifact.content.contains("[database]"));
        assert!(artifact.content.contains("host ="));
        assert!(artifact.tags.contains(&"toml".to_string()));
    }

    #[test]
    fn test_indexer_invalid_config() {
        let tmp = std::env::temp_dir().join("nonexistent_artifacts.json");
        let mut indexer = ArtifactIndexer::new(&tmp);
        assert!(indexer.build().is_err());
    }

    #[test]
    fn test_indexer_with_valid_config() {
        let tmp_dir = std::env::temp_dir().join("neotrix_test_artifacts");
        let _ = std::fs::create_dir_all(&tmp_dir);

        let sql_path = tmp_dir.join("schema.sql");
        std::fs::write(
            &sql_path,
            "CREATE TABLE users (id INT, name TEXT);\nCREATE TABLE orders (id INT);",
        )
        .expect("write sql file");

        let config_path = tmp_dir.join("artifacts.json");
        let config = serde_json::json!({
            "sources": [
                {
                    "name": "db_schema",
                    "path": sql_path.to_string_lossy(),
                    "tags": ["database", "production"]
                }
            ]
        });
        std::fs::write(&config_path, serde_json::to_string_pretty(&config).expect("value should be ok in test"))
            .expect("write config");

        let mut indexer = ArtifactIndexer::new(&config_path);
        let count = indexer.build().expect("should build artifacts");
        assert_eq!(count, 1);

        let store = indexer.store();
        let artifact = store.get("db_schema").expect("should find artifact");
        assert_eq!(artifact.artifact_type, ArtifactType::DatabaseSchema);
        assert!(artifact.tags.contains(&"database".to_string()));

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_inject_relevant_artifacts() {
        let mut store = ArtifactStore::new();
        store.store(
            Artifact::new(
                "users_db",
                ArtifactType::DatabaseSchema,
                "CREATE TABLE users (id SERIAL, email VARCHAR, name TEXT)",
            )
            .with_tags(&["database", "sql"]),
        );
        store.store(
            Artifact::new(
                "payment_api",
                ArtifactType::ApiSpec,
                "POST /payments\nGET /payments/{id}",
            )
            .with_tags(&["api", "payments"]),
        );
        store.store(
            Artifact::new(
                "deploy_config",
                ArtifactType::ConfigFile,
                "host: production\nport: 443",
            )
            .with_tags(&["config", "production"]),
        );

        let results = inject_relevant_artifacts(&store, "payment", 2);
        assert!(!results.is_empty());
        assert!(results.iter().any(|a| a.name == "payment_api"));

        let results = inject_relevant_artifacts(&store, "xyznonexistent", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_build_from_file_integration() {
        let tmp_dir = std::env::temp_dir().join("neotrix_test_build");
        let _ = std::fs::create_dir_all(&tmp_dir);

        let sql_file = tmp_dir.join("schema.sql");
        std::fs::write(
            &sql_file,
            "CREATE TABLE products (id INT, price DECIMAL, name VARCHAR);",
        )
        .expect("write sql");

        let artifact =
            ArtifactBuilder::build_from_file("products", sql_file.to_string_lossy().as_ref())
                .expect("should build from .sql file");
        assert_eq!(artifact.artifact_type, ArtifactType::DatabaseSchema);

        let md_file = tmp_dir.join("arch.md");
        std::fs::write(
            &md_file,
            "# Architecture\n\n## Overview\nThis is the system.\n\n## Components\nA, B, C.",
        )
        .expect("write md");

        let md_artifact =
            ArtifactBuilder::build_from_file("arch", md_file.to_string_lossy().as_ref())
                .expect("should build from .md file");
        assert_eq!(md_artifact.artifact_type, ArtifactType::ArchitectureDoc);

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_store_len_and_is_empty() {
        let mut store = ArtifactStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);

        store.store(Artifact::new("a", ArtifactType::ConfigFile, ""));
        assert!(!store.is_empty());
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_search_by_type() {
        let mut store = ArtifactStore::new();
        store.store(Artifact::new("s1", ArtifactType::DatabaseSchema, ""));
        store.store(Artifact::new("s2", ArtifactType::DatabaseSchema, ""));
        store.store(Artifact::new("api1", ArtifactType::ApiSpec, ""));

        assert_eq!(store.search_by_type(ArtifactType::DatabaseSchema).len(), 2);
        assert_eq!(store.search_by_type(ArtifactType::ApiSpec).len(), 1);
        assert_eq!(store.search_by_type(ArtifactType::ConfigFile).len(), 0);
    }

    #[test]
    fn test_artifact_type_label_and_from_str() {
        assert_eq!(ArtifactType::DatabaseSchema.label(), "database_schema");
        assert_eq!(
            ArtifactType::parse_artifact_kind("database_schema"),
            Some(ArtifactType::DatabaseSchema)
        );
        assert!(ArtifactType::parse_artifact_kind("unknown").is_none());
    }

    #[test]
    fn test_parse_sql_no_match() {
        assert!(ArtifactBuilder::parse_sql("test", "SELECT * FROM users;", None).is_none());
    }

    #[test]
    fn test_inject_relevant_by_tag() {
        let mut store = ArtifactStore::new();
        store.store(
            Artifact::new("db", ArtifactType::DatabaseSchema, "users table")
                .with_tags(&["database"]),
        );
        store.store(
            Artifact::new("api", ArtifactType::ApiSpec, "REST endpoints").with_tags(&["api"]),
        );

        let results = inject_relevant_artifacts(&store, "database query", 5);
        assert!(results.iter().any(|a| a.name == "db"));
    }
}
