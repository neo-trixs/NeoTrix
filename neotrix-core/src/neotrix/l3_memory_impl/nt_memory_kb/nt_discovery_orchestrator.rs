use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use uuid::Uuid;

use super::nt_discovery_github_topics::{self, DiscoveryPipelineConfig, GithubDiscoveryStats};
use super::nt_discovery_sources::{self, ExternalDiscoveryStats};

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Which discovery modules to run in a cycle
#[derive(Debug, Clone)]
pub struct DiscoveryCycleConfig {
    pub run_github_topics: bool,
    pub github_config: DiscoveryPipelineConfig,
    pub run_openlibrary: bool,
    pub openlibrary_queries: Vec<String>,
    pub run_arxiv: bool,
    pub arxiv_queries: Vec<String>,
    pub run_wikipedia: bool,
    pub wikipedia_topics: Vec<String>,
    pub run_annas_archive: bool,
    pub annas_archive_queries: Vec<String>,
    pub max_resources_per_source: usize,
}

impl Default for DiscoveryCycleConfig {
    fn default() -> Self {
        Self {
            run_github_topics: true,
            github_config: DiscoveryPipelineConfig::default(),
            run_openlibrary: false,
            openlibrary_queries: vec![],
            run_arxiv: false,
            arxiv_queries: vec![],
            run_wikipedia: false,
            wikipedia_topics: vec![],
            run_annas_archive: false,
            annas_archive_queries: vec![],
            max_resources_per_source: 20,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiscoveryCycleReport {
    pub cycle_id: String,
    pub started_at: i64,
    pub completed_at: i64,
    pub github: Option<GithubDiscoveryStats>,
    pub openlibrary: Option<ExternalDiscoveryStats>,
    pub arxiv: Option<ExternalDiscoveryStats>,
    pub wikipedia: Option<ExternalDiscoveryStats>,
    pub annas_archive: Option<ExternalDiscoveryStats>,
    pub total_resources_ingested: usize,
    pub total_errors: usize,
    pub errors: Vec<(String, String)>,
}

impl DiscoveryCycleReport {
    pub fn summary(&self) -> String {
        let id_short: String = self.cycle_id.chars().take(8).collect();
        let mut parts = vec![format!("Discovery Cycle {}:", id_short)];

        if let Some(ref g) = self.github {
            parts.push(format!("  GitHub: {} topics, {} repos ingested ({} skipped)", g.topics_found, g.repos_ingested, g.repos_skipped_existing));
        }
        if let Some(ref o) = self.openlibrary {
            parts.push(format!("  OpenLibrary: {} ingested", o.resources_ingested));
        }
        if let Some(ref a) = self.arxiv {
            parts.push(format!("  ArXiv: {} ingested", a.resources_ingested));
        }
        if let Some(ref w) = self.wikipedia {
            parts.push(format!("  Wikipedia: {} ingested", w.resources_ingested));
        }
        if let Some(ref a) = self.annas_archive {
            parts.push(format!("  Anna's Archive: {} ingested", a.resources_ingested));
        }

        parts.push(format!("  Total ingested: {}, errors: {}", self.total_resources_ingested, self.total_errors));
        parts.join("\n")
    }
}

/// Record a source's run status in discovery_sources table
fn record_source_run(conn: &Connection, source_name: &str, total_items: usize, status: &str, error: Option<&str>) {
    let now_ts = now();
    let _ = conn.execute(
        "INSERT INTO discovery_sources (source_name, last_run_at, total_items, status, error_message)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(source_name) DO UPDATE SET
           last_run_at=excluded.last_run_at,
           total_items=excluded.total_items,
           status=excluded.status,
           error_message=excluded.error_message",
        rusqlite::params![source_name, now_ts, total_items as i64, status, error],
    );
}

/// Run a full discovery cycle with all enabled sources
pub fn run_discovery_cycle(
    conn: &Connection,
    config: &DiscoveryCycleConfig,
) -> DiscoveryCycleReport {
    let cycle_id = Uuid::new_v4().to_string();
    let start_ts = now();
    let mut report = DiscoveryCycleReport {
        cycle_id,
        started_at: start_ts,
        ..Default::default()
    };

    let mut total_ingested = 0;
    let mut total_errors = 0;

    // Phase 1: GitHub Topics Discovery
    if config.run_github_topics {
        match nt_discovery_github_topics::run_github_topics_discovery(conn, &config.github_config) {
            Ok(stats) => {
                total_ingested += stats.repos_ingested;
                total_errors += stats.errors.len();
                for e in &stats.errors {
                    report.errors.push(e.clone());
                }
                let err_msg = if stats.errors.is_empty() { None } else { Some("See cycle errors") };
                record_source_run(conn, "github_topics", stats.repos_ingested, if stats.errors.is_empty() { "completed" } else { "completed_with_errors" }, err_msg);
                report.github = Some(stats);
            }
            Err(e) => {
                total_errors += 1;
                report.errors.push(("github_topics".into(), e.clone()));
                record_source_run(conn, "github_topics", 0, "failed", Some(&e));
            }
        }
    }

    // Phase 2: OpenLibrary book search
    if config.run_openlibrary {
        let mut combined = ExternalDiscoveryStats::default();
        for query in &config.openlibrary_queries {
            match nt_discovery_sources::discover_books(conn, query, config.max_resources_per_source) {
                Ok(stats) => {
                    combined.resources_ingested += stats.resources_ingested;
                    combined.resources_found += stats.resources_found;
                    combined.queries_executed += stats.queries_executed;
                    for e in &stats.errors {
                        report.errors.push(e.clone());
                    }
                }
                Err(e) => {
                    report.errors.push((format!("openlibrary:{}", query), e));
                }
            }
        }
        total_ingested += combined.resources_ingested;
        total_errors += combined.errors.len();
        let err_msg = if combined.errors.is_empty() { None } else { Some("See cycle errors") };
        record_source_run(conn, "openlibrary", combined.resources_ingested, if combined.errors.is_empty() { "completed" } else { "completed_with_errors" }, err_msg);
        report.openlibrary = Some(combined);
    }

    // Phase 3: ArXiv paper search
    if config.run_arxiv {
        let mut combined = ExternalDiscoveryStats::default();
        for query in &config.arxiv_queries {
            match nt_discovery_sources::discover_arxiv_papers(conn, query, config.max_resources_per_source) {
                Ok(stats) => {
                    combined.resources_ingested += stats.resources_ingested;
                    combined.resources_found += stats.resources_found;
                    combined.queries_executed += stats.queries_executed;
                    for e in &stats.errors {
                        report.errors.push(e.clone());
                    }
                }
                Err(e) => {
                    report.errors.push((format!("arxiv:{}", query), e));
                }
            }
        }
        total_ingested += combined.resources_ingested;
        total_errors += combined.errors.len();
        let err_msg = if combined.errors.is_empty() { None } else { Some("See cycle errors") };
        record_source_run(conn, "arxiv", combined.resources_ingested, if combined.errors.is_empty() { "completed" } else { "completed_with_errors" }, err_msg);
        report.arxiv = Some(combined);
    }

    // Phase 4: Wikipedia
    if config.run_wikipedia {
        let mut combined = ExternalDiscoveryStats::default();
        for topic in &config.wikipedia_topics {
            match nt_discovery_sources::discover_wikipedia_topic(conn, topic) {
                Ok(stats) => {
                    combined.resources_ingested += stats.resources_ingested;
                    combined.resources_found += stats.resources_found;
                    combined.queries_executed += stats.queries_executed;
                    for e in &stats.errors {
                        report.errors.push(e.clone());
                    }
                }
                Err(e) => {
                    report.errors.push((format!("wikipedia:{}", topic), e));
                }
            }
        }
        total_ingested += combined.resources_ingested;
        total_errors += combined.errors.len();
        let err_msg = if combined.errors.is_empty() { None } else { Some("See cycle errors") };
        record_source_run(conn, "wikipedia", combined.resources_ingested, if combined.errors.is_empty() { "completed" } else { "completed_with_errors" }, err_msg);
        report.wikipedia = Some(combined);
    }

    // Phase 5: Anna's Archive
    if config.run_annas_archive {
        let mut combined = ExternalDiscoveryStats::default();
        for query in &config.annas_archive_queries {
            match nt_discovery_sources::discover_annas_archive(conn, query, config.max_resources_per_source) {
                Ok(stats) => {
                    combined.resources_ingested += stats.resources_ingested;
                    combined.resources_found += stats.resources_found;
                    combined.queries_executed += stats.queries_executed;
                    for e in &stats.errors {
                        report.errors.push(e.clone());
                    }
                }
                Err(e) => {
                    report.errors.push((format!("annas_archive:{}", query), e));
                }
            }
        }
        total_ingested += combined.resources_ingested;
        total_errors += combined.errors.len();
        let err_msg = if combined.errors.is_empty() { None } else { Some("See cycle errors") };
        record_source_run(conn, "annas_archive", combined.resources_ingested, if combined.errors.is_empty() { "completed" } else { "completed_with_errors" }, err_msg);
        report.annas_archive = Some(combined);
    }

    report.total_resources_ingested = total_ingested;
    report.total_errors = total_errors;
    report.completed_at = now();

    // Log to ingest_log
    let log_id = Uuid::new_v4().to_string();
    let status = if total_errors == 0 { "completed" } else { "completed_with_errors" };
    let _ = conn.execute(
        "INSERT INTO ingest_log (id, source_type, source_url, status, items_count, started_at, completed_at, error)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            log_id,
            "discovery_cycle",
            "https://github.com/topics",
            status,
            total_ingested as i64,
            start_ts,
            report.completed_at,
            if total_errors > 0 {
                Some(format!("{} errors", total_errors))
            } else {
                None
            },
        ],
    );

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_schema;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_empty_cycle() {
        let conn = test_conn();
        let mut config = DiscoveryCycleConfig::default();
        config.run_github_topics = false;
        let report = run_discovery_cycle(&conn, &config);
        assert_eq!(report.total_resources_ingested, 0);
        assert!(report.cycle_id.len() > 10);
    }

    #[test]
    fn test_discovery_cycle_report_summary() {
        let report = DiscoveryCycleReport {
            cycle_id: "test-cycle-id".into(),
            started_at: 1000,
            completed_at: 2000,
            ..Default::default()
        };
        let summary = report.summary();
        assert!(summary.contains("test"));
    }

    #[test]
    fn test_summary_empty_cycle_id_no_panic() {
        // Regression: &self.cycle_id[..8] panicked on an empty/short
        // cycle_id (e.g. DiscoveryCycleReport::default()). chars().take(8)
        // is safe for any length.
        let report = DiscoveryCycleReport { ..Default::default() };
        let summary = report.summary();
        assert!(summary.contains("Discovery Cycle"));
    }
}
