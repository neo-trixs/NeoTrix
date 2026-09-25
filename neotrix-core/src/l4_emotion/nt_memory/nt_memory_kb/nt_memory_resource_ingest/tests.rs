//! nt_memory_resource_ingest tests — 原内联 tests 整体搬移，行为零变更。

use super::*;
use super::super::nt_memory_store::{get_all_nodes, get_edges_for_node, get_node};
use super::super::nt_memory_types::{NodeType, RelationType};
use super::nt_resource_corpus::copy_resumable;
use super::nt_resource_ingester::find_node_by_title;
use rusqlite::Connection;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        super::super::nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_reclaim_nt_target_tmp_dry_run_safe() {
        // dry_run 只计数不删除, 对任意 /private/tmp 状态均安全; 验证返回 Ok 且不报错。
        assert!(reclaim_nt_target_tmp(9999, true).is_ok(), "reclaim dry-run");
    }

    #[test]
    fn test_ingest_github_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::github(
            "testowner",
            "testrepo",
            "Test Repo",
            "A test repository for testing.",
        );
        let result = ingester.ingest(&desc).unwrap();
        assert!(!result.node_id.is_empty());
        assert!(result.insight_ids.is_empty());

        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.title, "Test Repo");
        assert_eq!(fetched.node_type, NodeType::Repository);
    }

    #[test]
    fn test_ingest_with_insights() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::github("o", "r", "Repo With Insights", "Desc")
            .with_key_insights(vec!["Insight one", "Insight two", "Insight three"]);
        let result = ingester.ingest(&desc).unwrap();
        assert_eq!(result.insight_ids.len(), 3);

        for iid in &result.insight_ids {
            let fetched = get_node(&conn, iid).unwrap().unwrap();
            assert_eq!(fetched.node_type, NodeType::Insight);
        }

        let edges = get_edges_for_node(&conn, &result.node_id).unwrap();
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_ingest_paper_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::paper("1234.56789", "Test Paper", "A test paper abstract.")
            .with_tags(vec!["test", "paper"]);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.node_type, NodeType::Paper);
        assert!(fetched.url.unwrap().contains("arxiv.org"));
    }

    #[test]
    fn test_ingest_article_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc =
            ResourceDescriptor::article("Test Article", "Summary", "https://example.com/article");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.node_type, NodeType::Article);
        assert_eq!(fetched.url.unwrap(), "https://example.com/article");
    }

    #[test]
    fn test_ingest_concept_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::concept("Test Concept", "A conceptual insight.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.node_type, NodeType::Concept);
        assert!(fetched.url.is_none());
    }

    #[test]
    fn test_ingest_tool_resource() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::tool(
            "Test Tool",
            "A tool description.",
            ResourceSource::GitHub {
                owner: "test".into(),
                repo: "tool".into(),
            },
        );
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.node_type, NodeType::Tool);
    }

    #[test]
    fn test_ingest_with_content() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::paper("0000.00000", "Paper With Content", "Abstract")
            .with_content("Full paper content here\nwith multiple lines.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert!(fetched.content.unwrap().contains("Full paper content"));
    }

    #[test]
    fn test_ingest_metadata_includes_episode() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::concept("Episode Concept", "Has episode tracking.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        let meta = fetched.metadata.unwrap();
        assert_eq!(meta["episode_id"], serde_json::json!(ingester.episode_id()));
        assert_eq!(fetched.source_episode, None);
    }

    #[test]
    fn test_ingest_multiple_and_relate() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let r1 = ingester
            .ingest(&ResourceDescriptor::concept("Concept A", "First concept."))
            .unwrap();
        let r2 = ingester
            .ingest(&ResourceDescriptor::concept("Concept B", "Second concept."))
            .unwrap();

        ingester
            .relate(
                &r1.node_id,
                &r2.node_id,
                RelationType::References,
                0.8,
                Some("A references B"),
            )
            .unwrap();

        let edges = get_edges_for_node(&conn, &r1.node_id).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target_id, r2.node_id);
        assert_eq!(edges[0].relation_type, RelationType::References);
    }

    #[test]
    fn test_ingest_with_importance_and_confidence() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::concept("Important Concept", "Very important.")
            .with_importance(0.95)
            .with_confidence(0.99);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert!((fetched.importance - 0.95).abs() < 0.01);
        assert!((fetched.confidence - 0.99).abs() < 0.01);
    }

    #[test]
    fn test_ingest_report_format() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        ingester
            .ingest(&ResourceDescriptor::concept(
                "Report Test",
                "Testing report.",
            ))
            .unwrap();
        let report = ingester.report();
        assert!(report.contains("Report Test"));
        assert!(report.contains("Episode ID"));
        assert!(report.contains("1"));
    }

    #[test]
    fn test_ingest_domain_from_source() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::github("o", "r", "Domain Test", "Testing domain.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert_eq!(fetched.domain.unwrap(), "github.com");
    }

    #[test]
    fn test_ingest_source_url() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::github("owner", "repo", "URL Test", "Testing URL.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert!(fetched.url.unwrap().contains("github.com/owner/repo"));
    }

    #[test]
    fn test_ingest_with_tags_in_metadata() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::paper("9999.99999", "Tagged Paper", "Abstract")
            .with_tags(vec!["tag1", "tag2", "absorbed-2026-07-03"]);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        let meta = fetched.metadata.unwrap();
        let tags: Vec<String> = serde_json::from_value(meta["tags"].clone()).unwrap();
        assert!(tags.contains(&"tag1".to_string()));
        assert!(tags.contains(&"absorbed-2026-07-03".to_string()));
    }

    #[test]
    fn test_relate_by_title() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        ingester
            .ingest(&ResourceDescriptor::concept("Source Node", "Source."))
            .unwrap();
        ingester
            .ingest(&ResourceDescriptor::concept("Target Node", "Target."))
            .unwrap();
        ingester
            .relate_by_title(
                "Source Node",
                "Target Node",
                RelationType::DependsOn,
                0.9,
                Some("depends"),
            )
            .unwrap();

        let src = find_node_by_title(&conn, "Source Node").unwrap().unwrap();
        let edges = get_edges_for_node(&conn, &src.id).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].relation_type, RelationType::DependsOn);
    }

    #[test]
    fn test_find_node_by_title_not_found() {
        let conn = test_conn();
        let result = find_node_by_title(&conn, "NonExistentNode").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_ingest_session_resources_creates_all_nodes() {
        let conn = test_conn();
        let report = ingest_session_resources(&conn).unwrap();
        assert!(report.contains("Episode ID"));
        let all_nodes = get_all_nodes(&conn).unwrap();
        assert!(all_nodes.len() >= 17);
    }

    #[test]
    fn test_ingest_insight_counts_in_metadata() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::paper("8888.88888", "Insight Count Test", "Abstract")
            .with_key_insights(vec!["A", "B", "C", "D", "E"]);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        let meta = fetched.metadata.unwrap();
        assert_eq!(meta["insight_count"], 5);
    }

    #[test]
    fn test_ingest_concept_without_insights_no_content() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let desc = ResourceDescriptor::concept("No Insights", "Just a concept.");
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.node_id).unwrap().unwrap();
        assert!(fetched.content.is_none());
    }

    #[test]
    fn test_ingest_long_insight_title_truncated() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let long = "A very long insight text that exceeds eighty characters in total length for truncation testing purposes";
        let desc = ResourceDescriptor::github("o", "r", "Truncation Test", "Summary")
            .with_key_insights(vec![long]);
        let result = ingester.ingest(&desc).unwrap();
        let fetched = get_node(&conn, &result.insight_ids[0]).unwrap().unwrap();
        assert!(fetched.title.len() <= 80);
        assert!(fetched.title.ends_with("..."));
        assert_eq!(fetched.summary.unwrap(), long);
    }

    #[test]
    #[ignore = "requires downloaded project-nomad source at ~/.neotrix/downloads/project-nomad-main-src/project-nomad-main/collections/"]
    fn test_ingest_project_nomad_data_sources() {
        let conn = test_conn();
        let mut ingester = ResourceIngester::new(&conn);
        let src = std::path::Path::new(
            "/Users/neo/.neotrix/downloads/project-nomad-main-src/project-nomad-main/collections",
        );
        let mut ingested = 0;

        // kiwix-categories.json
        if let Ok(content) = std::fs::read_to_string(src.join("kiwix-categories.json")) {
            if let Ok(kiwix) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(cats) = kiwix["categories"].as_array() {
                    for cat in cats {
                        if let Some(resources) = cat["resources"].as_array() {
                            for res in resources {
                                if let Some(url) = res["url"].as_str() {
                                    let title = format!(
                                        "Kiwix: {} - {}",
                                        cat["name"].as_str().unwrap_or(""),
                                        res["title"].as_str().unwrap_or("")
                                    );
                                    let summary = res["description"].as_str().unwrap_or("");
                                    let tags = vec![
                                        "kiwix".to_string(),
                                        "data-source".to_string(),
                                        cat["slug"].as_str().unwrap_or("").to_string(),
                                    ];
                                    let desc = ResourceDescriptor::article(&title, summary, url)
                                        .with_tags(tags.iter().map(|s| s.as_str()).collect())
                                        .with_importance(0.7);
                                    if ingester.ingest(&desc).is_ok() {
                                        ingested += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // maps.json
        if let Ok(content) = std::fs::read_to_string(src.join("maps.json")) {
            if let Ok(maps) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(colls) = maps["collections"].as_array() {
                    for coll in colls {
                        if let Some(resources) = coll["resources"].as_array() {
                            for res in resources {
                                if let Some(url) = res["url"].as_str() {
                                    let title = format!(
                                        "Map: {} - {}",
                                        coll["name"].as_str().unwrap_or(""),
                                        res["title"].as_str().unwrap_or("")
                                    );
                                    let summary = res["description"].as_str().unwrap_or("");
                                    let tags = vec![
                                        "maps".to_string(),
                                        "pmtiles".to_string(),
                                        "data-source".to_string(),
                                        coll["slug"].as_str().unwrap_or("").to_string(),
                                    ];
                                    let desc = ResourceDescriptor::article(&title, summary, url)
                                        .with_tags(tags.iter().map(|s| s.as_str()).collect())
                                        .with_importance(0.7);
                                    if ingester.ingest(&desc).is_ok() {
                                        ingested += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // wikipedia.json
        if let Ok(content) = std::fs::read_to_string(src.join("wikipedia.json")) {
            if let Ok(wiki) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(opts) = wiki["options"].as_array() {
                    for opt in opts {
                        if let Some(url) = opt["url"].as_str() {
                            if !url.is_empty() && url != "null" {
                                let title =
                                    format!("Wikipedia: {}", opt["name"].as_str().unwrap_or(""));
                                let summary = opt["description"].as_str().unwrap_or("");
                                let tags = vec![
                                    "wikipedia".to_string(),
                                    "zim".to_string(),
                                    "data-source".to_string(),
                                ];
                                let desc = ResourceDescriptor::article(&title, summary, url)
                                    .with_tags(tags.iter().map(|s| s.as_str()).collect())
                                    .with_importance(0.7);
                                if ingester.ingest(&desc).is_ok() {
                                    ingested += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        assert!(
            ingested > 0,
            "at least one project-nomad data source should be ingested"
        );
        println!("project-nomad data sources ingested: {}", ingested);
    }

    #[test]
    fn test_register_cortex_brain_temp_dir() {
        let conn = test_conn();
        let dir = std::env::temp_dir().join(format!("cortex_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("cortex-archive").join("zim")).unwrap();
        std::fs::write(dir.join("cortex-archive").join("zim").join("x.zim"), b"z").unwrap();
        std::fs::create_dir_all(dir.join("working")).unwrap();
        std::fs::write(dir.join("working").join("causal_graph.json"), b"{}").unwrap();
        let baseline: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE node_type='cortex_brain'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let n = register_cortex_brain(&conn, &dir).unwrap();
        assert!(
            n >= 2,
            "should register zim dir + causal graph node, got {n}"
        );
        let cnt: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE node_type='cortex_brain'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            cnt,
            baseline + n as i64,
            "cortex_brain node count drift (baseline={baseline} n={n} cnt={cnt})"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_register_cortex_brain_missing_mount_is_noop() {
        let conn = test_conn();
        let missing = std::path::Path::new("/Volumes/NeoTrixBrain__definitely_not_mounted");
        assert_eq!(register_cortex_brain(&conn, missing).unwrap(), 0);
        let cnt: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM nodes WHERE node_type='cortex_brain'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cnt, 0);
    }

    #[test]
    fn test_prune_cortex_orphans_no_zim_errors() {
        let conn = test_conn();
        let dir = std::env::temp_dir().join(format!("cortex_prune_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let r = prune_cortex_orphans(&conn, &dir, true);
        assert!(r.is_err(), "should error when no ZIM files on volume");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Resumable copy must finish correctly when a partial copy + progress sidecar already exist.
    #[test]
    fn test_copy_resumable_resumes_from_partial() {
        let dir = std::env::temp_dir().join(format!("cortex_copy_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("src.bin");
        let dest = dir.join("dest.bin");
        let data: Vec<u8> = (0..10 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
        std::fs::write(&src, &data).unwrap();
        // simulate a prior interrupted run: 3 MiB written + .prog checkpoint at 3 MiB
        let partial = 3 * 1024 * 1024;
        std::fs::write(&dest, &data[..partial]).unwrap();
        std::fs::write(format!("{}.prog", dest.display()), partial.to_string()).unwrap();
        copy_resumable(&src, &dest).unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), data);
        assert!(!std::path::Path::new(&format!("{}.prog", dest.display())).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Fresh resumable copy (no prior partial) must produce a byte-identical destination.
    #[test]
    fn test_copy_resumable_fresh() {
        let dir = std::env::temp_dir().join(format!("cortex_copy2_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("src.bin");
        let dest = dir.join("dest.bin");
        let data: Vec<u8> = (0..7 * 1024 * 1024).map(|i| (i % 197) as u8).collect();
        std::fs::write(&src, &data).unwrap();
        copy_resumable(&src, &dest).unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), data);
        assert!(!std::path::Path::new(&format!("{}.prog", dest.display())).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
