use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_memory_kb::{KnowledgeBase, DiscoveryPipelineConfig, DiscoveryCycleConfig};
use crate::neotrix::nt_mind::SelfIteratingBrain;

static KB: LazyLock<Mutex<Option<KnowledgeBase>>> = LazyLock::new(|| {
    let path = dirs::home_dir().map(|p| p.join(".neotrix").join("knowledge.db"));
    match KnowledgeBase::open(path) {
        Ok(kb) => Mutex::new(Some(kb)),
        Err(_) => Mutex::new(None),
    }
});

fn with_kb<F, R>(f: F) -> CommandOutput
where
    F: FnOnce(&KnowledgeBase) -> Result<R, String>,
    R: std::fmt::Display,
{
    let guard = KB.lock().unwrap_or_else(|e| e.into_inner());
    match guard.as_ref() {
        Some(kb) => match f(kb) {
            Ok(result) => CommandOutput::ok(&result.to_string()),
            Err(e) => CommandOutput::err(&format!("KB error: {}", e)),
        },
        None => CommandOutput::err("KnowledgeBase not available (open ~/.neotrix/knowledge.db failed)"),
    }
}

pub struct SourcesCmd;

impl CliCommand for SourcesCmd {
    fn name(&self) -> &str {
        "/sources"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/src", "/datasrc"]
    }

    fn description(&self) -> &str {
        "Data source discovery: scan|github|books|arxiv|wiki|status|schedule"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let subcmd = args.first().map(|s| s.as_str()).unwrap_or("status");

        match subcmd {
            "scan" | "run" => {
                let mode = args.get(1).map(|s| s.as_str()).unwrap_or("all");
                let query = args.get(2).cloned().unwrap_or_default();
                let limit = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(10);

                let record_source = |kb: &KnowledgeBase, source_name: &str, ingested: usize, ok: bool, err: Option<&str>| {
                    if let Ok(conn) = kb.conn.lock() {
                        let _ = conn.execute(
                            "INSERT INTO discovery_sources (source_name, last_run_at, total_items, status, error_message)
                             VALUES (?1, ?2, ?3, ?4, ?5)
                             ON CONFLICT(source_name) DO UPDATE SET
                               last_run_at=excluded.last_run_at,
                               total_items=excluded.total_items,
                               status=excluded.status,
                               error_message=excluded.error_message",
                            rusqlite::params![
                                source_name,
                                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64,
                                ingested as i64,
                                if ok { "completed" } else { "failed" },
                                err,
                            ],
                        );
                    }
                };

                match mode {
                    "github" | "gh" => {
                        let min_stars = args.get(2).and_then(|s| s.parse::<i64>().ok()).unwrap_or(1000);
                        let repos_per = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(10);
                        let config = DiscoveryPipelineConfig {
                            min_stars_for_topic_discovery: min_stars,
                            repos_per_topic: repos_per,
                            ..Default::default()
                        };
                        with_kb(|kb| {
                            let stats = kb.run_github_topics_discovery(&config)?;
                            record_source(kb, "github_topics", stats.repos_ingested, true, None);
                            Ok(format!(
                                "GitHub Topics Discovery:\n  Topics found: {}\n  Repos ingested: {}\n  Repos skipped (existing): {}\n  API calls: {}\n  Errors: {}",
                                stats.topics_found, stats.repos_ingested, stats.repos_skipped_existing, stats.api_calls, stats.errors.len()
                            ))
                        })
                    }
                    "gutenberg" | "gut" => {
                        with_kb(|kb| {
                            let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                            let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_gutenberg(&conn, &query, limit)?;
                            record_source(kb, "gutenberg", stats.resources_ingested, stats.errors.is_empty(), None);
                            Ok(format!("Gutenberg books for '{}': {} found, {} ingested, {} errors", query, stats.resources_found, stats.resources_ingested, stats.errors.len()))
                        })
                    }
                    "semantic" | "s2" => {
                        with_kb(|kb| {
                            let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                            let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_semantic_scholar(&conn, &query, limit)?;
                            record_source(kb, "semantic_scholar", stats.resources_ingested, stats.errors.is_empty(), None);
                            Ok(format!("Semantic Scholar papers for '{}': {} found, {} ingested, {} errors", query, stats.resources_found, stats.resources_ingested, stats.errors.len()))
                        })
                    }
                    "pleiades" | "sites" => {
                        with_kb(|kb| {
                            let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                            let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_historical_sites(&conn, &query, limit)?;
                            record_source(kb, "pleiades", stats.resources_ingested, stats.errors.is_empty(), None);
                            Ok(format!("Historical sites for '{}': {} found, {} ingested, {} errors", query, stats.resources_found, stats.resources_ingested, stats.errors.len()))
                        })
                    }
                    "europeana" | "museum" => {
                        with_kb(|kb| {
                            let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                            let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_europeana(&conn, &query, limit)?;
                            record_source(kb, "europeana", stats.resources_ingested, stats.errors.is_empty(), None);
                            Ok(format!("Europeana museum items for '{}': {} found, {} ingested, {} errors", query, stats.resources_found, stats.resources_ingested, stats.errors.len()))
                        })
                    }
                    "inscriptions" | "epigraphy" => {
                        with_kb(|kb| {
                            let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                            let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_inscriptions(&conn, &query, limit)?;
                            record_source(kb, "inscriptions", stats.resources_ingested, stats.errors.is_empty(), None);
                            Ok(format!("Inscriptions for '{}': {} found, {} ingested, {} errors", query, stats.resources_found, stats.resources_ingested, stats.errors.len()))
                        })
                    }
                    "chinese" | "ctext" => {
                        with_kb(|kb| {
                            let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                            let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_chinese_ancient(&conn, &query, limit)?;
                            record_source(kb, "chinese_ancient", stats.resources_ingested, stats.errors.is_empty(), None);
                            Ok(format!("Chinese ancient texts for '{}': {} found, {} ingested, {} errors", query, stats.resources_found, stats.resources_ingested, stats.errors.len()))
                        })
                    }
                    "archive" | "internet-archive" => {
                        with_kb(|kb| {
                            let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                            let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_internet_archive(&conn, &query, limit)?;
                            record_source(kb, "internet_archive", stats.resources_ingested, stats.errors.is_empty(), None);
                            Ok(format!("Internet Archive items for '{}': {} found, {} ingested, {} errors", query, stats.resources_found, stats.resources_ingested, stats.errors.len()))
                        })
                    }
                    "hathitrust" => {
                        with_kb(|kb| {
                            let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                            let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_hathitrust(&conn, &query, limit)?;
                            record_source(kb, "hathitrust", stats.resources_ingested, stats.errors.is_empty(), None);
                            Ok(format!("HathiTrust records for '{}': {} found, {} ingested, {} errors", query, stats.resources_found, stats.resources_ingested, stats.errors.len()))
                        })
                    }
                    "opencontext" | "archaeology" => {
                        with_kb(|kb| {
                            let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                            let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_open_context(&conn, &query, limit)?;
                            record_source(kb, "open_context", stats.resources_ingested, stats.errors.is_empty(), None);
                            Ok(format!("Open Context archaeology for '{}': {} found, {} ingested, {} errors", query, stats.resources_found, stats.resources_ingested, stats.errors.len()))
                        })
                    }
                    "technical" | "techdocs" => {
                        with_kb(|kb| {
                            let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                            let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_technical_docs(&conn, &query)?;
                            record_source(kb, "technical_docs", stats.resources_ingested, stats.errors.is_empty(), None);
                            Ok(format!("Technical docs for '{}': {} ingested, {} errors", query, stats.resources_ingested, stats.errors.len()))
                        })
                    }
                    "gallica" | "bnf" => {
                        with_kb(|kb| {
                            let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                            let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_gallica(&conn, &query, limit)?;
                            record_source(kb, "gallica", stats.resources_ingested, stats.errors.is_empty(), None);
                            Ok(format!("Gallica BnF records for '{}': {} found, {} ingested, {} errors", query, stats.resources_found, stats.resources_ingested, stats.errors.len()))
                        })
                    }
                    "all" | "" => {
                        let config = DiscoveryCycleConfig {
                            run_github_topics: true,
                            ..Default::default()
                        };
                        with_kb(|kb| {
                            let report = kb.run_discovery_cycle(&config);
                            Ok(report.summary())
                        })
                    }
                    _ => CommandOutput::err(
                        "Usage: /sources scan [github|gutenberg|semantic|pleiades|europeana|inscriptions|chinese|archive|hathitrust|opencontext|technical|gallica|books|arxiv|wiki|annas|all] [query] [limit]"
                    ),
                }
            }

            "github" | "gh" => {
                let topic = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if topic.is_empty() {
                    return CommandOutput::err("Usage: /sources github <topic> [min_stars] [per_page]");
                }
                let min_stars = args.get(2).and_then(|s| s.parse::<i64>().ok()).unwrap_or(5000);
                let per_page = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(5);
                let config = DiscoveryPipelineConfig {
                    min_stars_for_topic_discovery: min_stars,
                    repos_per_topic: per_page,
                    scan_only_new_topics: false,
                    max_popular_repo_pages: 1,
                };
                with_kb(|kb| {
                    let stats = kb.run_github_topics_discovery(&config)?;
                    Ok(format!(
                        "GitHub scan for '{}': {} repos ingested ({} skipped, {} errors)",
                        topic, stats.repos_ingested, stats.repos_skipped_existing, stats.errors.len()
                    ))
                })
            }

            "books" | "openlibrary" => {
                let query = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if query.is_empty() {
                    return CommandOutput::err("Usage: /sources books <search query> [limit]");
                }
                let limit = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(10);
                with_kb(|kb| {
                    let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                    let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_books(&conn, query, limit)?;
                    let _ = conn.execute(
                        "INSERT INTO discovery_sources (source_name, last_run_at, total_items, status)
                         VALUES ('openlibrary', ?1, ?2, 'completed')
                         ON CONFLICT(source_name) DO UPDATE SET last_run_at=excluded.last_run_at, total_items=excluded.total_items, status=excluded.status",
                        rusqlite::params![std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64, stats.resources_ingested as i64],
                    );
                    Ok(format!(
                        "OpenLibrary books for '{}': {} found, {} ingested, {} errors",
                        query, stats.resources_found, stats.resources_ingested, stats.errors.len()
                    ))
                })
            }

            "arxiv" => {
                let query = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if query.is_empty() {
                    return CommandOutput::err("Usage: /sources arxiv <search query> [max_results]");
                }
                let limit = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(10);
                with_kb(|kb| {
                    let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                    let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_arxiv_papers(&conn, query, limit)?;
                    let _ = conn.execute(
                        "INSERT INTO discovery_sources (source_name, last_run_at, total_items, status)
                         VALUES ('arxiv', ?1, ?2, 'completed')
                         ON CONFLICT(source_name) DO UPDATE SET last_run_at=excluded.last_run_at, total_items=excluded.total_items, status=excluded.status",
                        rusqlite::params![std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64, stats.resources_ingested as i64],
                    );
                    Ok(format!(
                        "ArXiv papers for '{}': {} found, {} ingested, {} errors",
                        query, stats.resources_found, stats.resources_ingested, stats.errors.len()
                    ))
                })
            }

            "wiki" | "wikipedia" => {
                let topic = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if topic.is_empty() {
                    return CommandOutput::err("Usage: /sources wiki <topic>");
                }
                with_kb(|kb| {
                    let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                    let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_wikipedia_topic(&conn, topic)?;
                    let _ = conn.execute(
                        "INSERT INTO discovery_sources (source_name, last_run_at, total_items, status)
                         VALUES ('wikipedia', ?1, ?2, 'completed')
                         ON CONFLICT(source_name) DO UPDATE SET last_run_at=excluded.last_run_at, total_items=excluded.total_items, status=excluded.status",
                        rusqlite::params![std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64, stats.resources_ingested as i64],
                    );
                    Ok(format!(
                        "Wikipedia '{}': {} resources ingested, {} errors",
                        topic, stats.resources_ingested, stats.errors.len()
                    ))
                })
            }

            "annas" | "annas-archive" => {
                let query = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if query.is_empty() {
                    return CommandOutput::err("Usage: /sources annas <search query> [limit]");
                }
                let limit = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(5);
                with_kb(|kb| {
                    let conn = kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
                    let stats = crate::neotrix::nt_memory_kb::nt_discovery_sources::discover_annas_archive(&conn, query, limit)?;
                    let _ = conn.execute(
                        "INSERT INTO discovery_sources (source_name, last_run_at, total_items, status)
                         VALUES ('annas_archive', ?1, ?2, 'completed')
                         ON CONFLICT(source_name) DO UPDATE SET last_run_at=excluded.last_run_at, total_items=excluded.total_items, status=excluded.status",
                        rusqlite::params![std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64, stats.resources_ingested as i64],
                    );
                    Ok(format!(
                        "Anna's Archive books for '{}': {} found, {} ingested, {} errors",
                        query, stats.resources_found, stats.resources_ingested, stats.errors.len()
                    ))
                })
            }

            "schedule" => {
                let interval = args.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(86400);
                use crate::neotrix::nt_mind_background_loop::always_on::{
                    ScheduleExpr, ALWAYS_ON_ENGINE,
                };
                let mut engine = ALWAYS_ON_ENGINE.lock().unwrap_or_else(|e| e.into_inner());
                let id = engine.add_scheduled(
                    "data_source_discovery_cycle",
                    ScheduleExpr::Every { interval_secs: interval },
                );
                let _ = engine.save();
                CommandOutput::ok(&format!(
                    "Scheduled daily discovery cycle (id={}, interval={}s). Use '/schedule list' to view.",
                    id, interval
                ))
            }

            "migrate" | "import-all" => {
                with_kb(|kb| {
                    let report = kb.migrate_from_files();
                    Ok(report.summary())
                })
            }

            "status" | "stats" | "" => {
                with_kb(|kb| {
                    let stats = kb.stats()?;
                    let unified = kb.store_stats().unwrap_or_default();
                    let mut msg = format!(
                        "Knowledge Base Status:\n  Nodes: {}\n  Edges: {}\n  Crawl pending: {}\n  Crawl completed: {}\n",
                        stats.total_nodes, stats.total_edges, stats.crawl_pending, stats.crawl_completed,
                    );
                    if !stats.by_type.is_empty() {
                        msg.push_str("  By type:\n");
                        for (t, c) in &stats.by_type {
                            msg.push_str(&format!("    {}: {}\n", t, c));
                        }
                    }
                    msg.push_str("  Unified Store:\n");
                    for (table, count) in &unified {
                        msg.push_str(&format!("    {}: {}\n", table, count));
                    }
                    Ok(msg.trim().to_string())
                })
            }

            _ => CommandOutput::err(
                "Usage: /sources scan|migrate|github|books|arxiv|wiki|annas|schedule|status"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sources_cmd_name() {
        let cmd = SourcesCmd;
        assert_eq!(cmd.name(), "/sources");
    }

    #[test]
    fn test_sources_cmd_aliases() {
        let cmd = SourcesCmd;
        let aliases = cmd.aliases();
        assert!(aliases.contains(&"/src"));
        assert!(aliases.contains(&"/datasrc"));
    }

    #[test]
    fn test_sources_status_no_kb() {
        let cmd = SourcesCmd;
        let result = cmd.execute(&[], None);
        // Should handle gracefully even without KB
        assert!(!result.success || result.message.contains("Knowledge Base"));
    }
}
