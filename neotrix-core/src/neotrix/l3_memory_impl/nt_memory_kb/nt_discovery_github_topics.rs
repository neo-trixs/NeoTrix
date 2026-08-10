use std::collections::HashSet;
use std::sync::LazyLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use uuid::Uuid;

use super::nt_memory_store as store;
use super::nt_memory_types::*;

fn http_client() -> &'static reqwest::blocking::Client {
    static CLIENT: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
        super::nt_http::run_blocking(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (nt_discovery_github_topics)")
                .timeout(Duration::from_secs(30))
                .connect_timeout(Duration::from_secs(15))
                .no_proxy()
                .build()
                .unwrap_or_else(|e| {
                    eprintln!("WARN: HTTP client init failed: {}", e);
                    reqwest::blocking::Client::new()
                })
        })
    });
    &CLIENT
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn github_token() -> Option<String> {
    std::env::var("GITHUB_TOKEN")
        .ok()
        .or_else(|| std::env::var("GH_TOKEN").ok())
        .filter(|t| !t.is_empty())
}

fn api_get(url: &str) -> Result<serde_json::Value, String> {
    let req = http_client().get(url);
    let req = if let Some(token) = github_token() {
        req.header("Authorization", format!("Bearer {}", token))
    } else {
        req
    };
    let resp = super::nt_http::run_blocking(|| req.send()).map_err(|e| format!("HTTP error: {}", e))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().unwrap_or_default();
        return Err(format!("GitHub API {}: {} ({})", status, url, body.chars().take(200).collect::<String>()));
    }
    resp.json().map_err(|e| format!("JSON error: {}", e))
}

const GITHUB_API: &str = "https://api.github.com";

#[derive(Debug, Clone, Default)]
pub struct GithubDiscoveryStats {
    pub topics_found: usize,
    pub repos_found: usize,
    pub repos_ingested: usize,
    pub repos_skipped_existing: usize,
    pub api_calls: usize,
    pub errors: Vec<(String, String)>,
}

/// Full pipeline: discover topics from popular repos, then for each topic discover and ingest repos.
/// 定向模式 (config.target_topics 非空): 跳过热门仓库主题发现, 直接按指定主题搜索并落库。
pub fn run_github_topics_discovery(
    conn: &Connection,
    config: &DiscoveryPipelineConfig,
) -> Result<GithubDiscoveryStats, String> {
    let mut stats = GithubDiscoveryStats::default();

    let known_topics = load_known_topics(conn);
    let mut known_repos: HashSet<String> = load_known_repos(conn);

    let ts = now();

    let topic_node_id = store::insert_or_get_node(
        conn,
        "GitHub Topics Source",
        NodeType::Source,
        Some("Auto-discovered GitHub topics and top repositories"),
        Some("https://github.com/topics"),
        Some("github.com"),
    ).map_err(|e| format!("DB: {}", e))?;

    // Phase 1: Discover topics from popular repos (定向模式跳过 — 直接使用 target_topics)
    let mut topic_set: HashSet<String> = known_topics.clone();
    let mut discovered_topics: Vec<String> = Vec::new();
    let mut total_new_topics = 0;

    if config.target_topics.is_empty() {
        let url = format!(
            "{}/search/repositories?q=stars:>{}&sort=stars&order=desc&per_page=100",
            GITHUB_API, config.min_stars_for_topic_discovery,
        );
        stats.api_calls += 1;

        let data = api_get(&url)?;
        let items = data["items"].as_array().ok_or_else(|| "Missing items".to_string())?;

        for item in items {
            if let Some(topics) = item["topics"].as_array() {
                for topic_val in topics {
                    if let Some(t) = topic_val.as_str() {
                        topic_set.insert(t.to_lowercase());
                    }
                }
            }
        }

        discovered_topics = topic_set.difference(&known_topics)
            .cloned()
            .collect();
        total_new_topics = discovered_topics.len();

        // Register discovered topics as Concept nodes
        for topic in &discovered_topics {
            let tid = store::insert_or_get_node(
                conn,
                topic,
                NodeType::Concept,
                Some(&format!("GitHub topic: {}", topic)),
                None,
                Some("github.com/topic"),
            ).ok();
            if let Some(tid) = tid {
                let _ = store::upsert_edge(
                    conn,
                    &tid,
                    &topic_node_id,
                    RelationType::InstanceOf,
                    0.8,
                    Some("GitHub topic discovered via topic scanner"),
                );
            }
        }
    } else {
        // 定向模式: 直接注册 target_topics 为 Concept 节点
        for topic in &config.target_topics {
            let t = topic.to_lowercase();
            topic_set.insert(t.clone());
            let tid = store::insert_or_get_node(
                conn,
                &t,
                NodeType::Concept,
                Some(&format!("GitHub topic: {}", t)),
                None,
                Some("github.com/topic"),
            ).ok();
            if let Some(tid) = tid {
                let _ = store::upsert_edge(
                    conn,
                    &tid,
                    &topic_node_id,
                    RelationType::InstanceOf,
                    0.8,
                    Some("GitHub topic targeted crawl"),
                );
            }
        }
    }

    // Phase 2: For each topic (including known), discover top repos
    let topics_to_scan: Vec<&String> = if config.scan_only_new_topics {
        discovered_topics.iter().collect()
    } else {
        topic_set.iter().collect()
    };

    for topic in &topics_to_scan {
        let per_page = config.repos_per_topic.min(100);
        let url = format!(
            "{}/search/repositories?q=topic:{}&sort=stars&order=desc&per_page={}",
            GITHUB_API, topic, per_page,
        );
        stats.api_calls += 1;

        match api_get(&url) {
            Ok(data) => {
                if let Some(items) = data["items"].as_array() {
                    for item in items {
                        if let Some(full_name) = item["full_name"].as_str() {
                            let repo_url = format!("https://github.com/{}", full_name);
                            if known_repos.insert(repo_url.clone()) {
                                // 直接从搜索结果落库 (零额外 API 调用)
                                match ingest_repo_from_search_item(conn, item) {
                                    Ok(_) => stats.repos_ingested += 1,
                                    Err(e) => stats.errors.push((full_name.to_string(), e)),
                                }
                            } else {
                                stats.repos_skipped_existing += 1;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                stats.errors.push((format!("topic:{}", topic), e));
            }
        }
    }

    // Log discovery event
    let log_id = Uuid::new_v4().to_string();
    let _ = conn.execute(
        "INSERT INTO ingest_log (id, source_type, source_url, status, items_count, started_at, completed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            log_id,
            "github_topics_discovery",
            "https://github.com/topics",
            if stats.errors.is_empty() { "completed" } else { "completed_with_errors" },
            stats.repos_ingested,
            ts,
            now(),
        ],
    );

    stats.topics_found = total_new_topics;
    Ok(stats)
}

/// 从 GitHub search API 结果 item 直接落库 (零额外 API 调用)。
/// 复用 ingest_from_github 的落库结构: Repository 节点 + Owner 组织 + Topic 概念边。
fn ingest_repo_from_search_item(conn: &Connection, item: &serde_json::Value) -> Result<usize, String> {
    let default_title = "unknown/unknown".to_string();
    let title = item["full_name"].as_str().unwrap_or(&default_title);
    let description = item["description"].as_str().unwrap_or("");
    let repo_url = item["html_url"].as_str().unwrap_or("");
    let topics: Vec<String> = item["topics"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let node_id = store::insert_or_get_node(
        conn,
        title,
        NodeType::Repository,
        Some(description),
        Some(repo_url),
        Some("github.com"),
    )
    .map_err(|e| format!("DB error: {}", e))?;

    if let Some(owner) = item["owner"].as_object() {
        if let Some(owner_login) = owner.get("login").and_then(|v| v.as_str()) {
            let owner_id = store::insert_or_get_node(
                conn,
                owner_login,
                NodeType::Organization,
                None,
                Some(&format!("https://github.com/{}", owner_login)),
                Some("github.com"),
            )
            .map_err(|e| format!("DB error: {}", e))?;
            store::upsert_edge(
                conn,
                &node_id,
                &owner_id,
                RelationType::DevelopedBy,
                1.0,
                Some("Repository owner"),
            )
            .map_err(|e| format!("DB error: {}", e))?;
        }
    }

    for topic in &topics {
        let topic_id = store::insert_or_get_node(
            conn,
            topic,
            NodeType::Concept,
            None,
            None,
            Some("github.com"),
        )
        .map_err(|e| format!("DB error: {}", e))?;
        store::upsert_edge(
            conn,
            &node_id,
            &topic_id,
            RelationType::Related,
            1.0,
            Some("GitHub topic"),
        )
        .map_err(|e| format!("DB error: {}", e))?;
    }

    Ok(1)
}

fn load_known_topics(conn: &Connection) -> HashSet<String> {
    let mut stmt = match conn.prepare(
        "SELECT title FROM nodes WHERE node_type='concept' AND domain='github.com/topic'"
    ) {
        Ok(s) => s,
        Err(_) => return HashSet::new(),
    };
    let rows: Vec<String> = match stmt.query_map([], |row| row.get(0)) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    };
    rows.into_iter().collect()
}

fn load_known_repos(conn: &Connection) -> HashSet<String> {
    let mut stmt = match conn.prepare(
        "SELECT url FROM nodes WHERE node_type='repository' AND url IS NOT NULL"
    ) {
        Ok(s) => s,
        Err(_) => return HashSet::new(),
    };
    let rows: Vec<String> = match stmt.query_map([], |row| row.get(0)) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    };
    rows.into_iter().collect()
}

#[derive(Debug, Clone)]
pub struct DiscoveryPipelineConfig {
    pub min_stars_for_topic_discovery: i64,
    pub repos_per_topic: usize,
    pub scan_only_new_topics: bool,
    pub max_popular_repo_pages: usize,
    /// 定向主题列表 — 非空时跳过 Phase 1 (热门仓库主题发现),
    /// 直接按指定主题搜索并落库 (定向爬取, 节省 API 配额)。
    pub target_topics: Vec<String>,
}

impl Default for DiscoveryPipelineConfig {
    fn default() -> Self {
        Self {
            min_stars_for_topic_discovery: 1000,
            repos_per_topic: 10,
            scan_only_new_topics: false,
            max_popular_repo_pages: 5,
            target_topics: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_schema;

    #[allow(dead_code)]
    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_discovery_config_defaults() {
        let cfg = DiscoveryPipelineConfig::default();
        assert_eq!(cfg.min_stars_for_topic_discovery, 1000);
        assert_eq!(cfg.repos_per_topic, 10);
        assert!(!cfg.scan_only_new_topics);
        assert!(cfg.target_topics.is_empty());
    }

    #[test]
    fn test_topic_discovery_empty_db() {
        let conn = test_conn();
        let topics = load_known_topics(&conn);
        assert!(topics.is_empty());
    }

    #[test]
    fn test_repo_discovery_empty_db() {
        let conn = test_conn();
        let repos = load_known_repos(&conn);
        assert!(repos.is_empty());
    }

    #[test]
    fn test_known_topic_tracking() {
        let conn = test_conn();
        let topic_id = store::insert_or_get_node(&conn, "rust", NodeType::Concept, None, None, Some("github.com/topic")).unwrap();
        assert!(!topic_id.is_empty());
        let topics = load_known_topics(&conn);
        assert!(topics.contains("rust"));
    }

    #[test]
    fn test_target_topics_registration() {
        let conn = test_conn();
        let cfg = DiscoveryPipelineConfig {
            target_topics: vec!["llm".to_string(), "web-crawler".to_string()],
            ..Default::default()
        };
        // 定向模式: 主题应注册为 Concept 节点 (github.com/topic)
        let topic_node_id = store::insert_or_get_node(
            &conn,
            "GitHub Topics Source",
            NodeType::Source,
            None,
            Some("https://github.com/topics"),
            Some("github.com"),
        ).unwrap();
        for t in &cfg.target_topics {
            let tid = store::insert_or_get_node(
                &conn,
                t,
                NodeType::Concept,
                None,
                None,
                Some("github.com/topic"),
            ).unwrap();
            let _ = store::upsert_edge(&conn, &tid, &topic_node_id, RelationType::InstanceOf, 0.8, None);
        }
        let topics = load_known_topics(&conn);
        assert!(topics.contains("llm"));
        assert!(topics.contains("web-crawler"));
    }

    #[test]
    fn test_ingest_repo_from_search_item() {
        let conn = test_conn();
        let item = serde_json::json!({
            "full_name": "openai/codex",
            "description": "desc",
            "html_url": "https://github.com/openai/codex",
            "topics": ["llm", "ai"],
            "owner": {"login": "openai"}
        });
        let n = ingest_repo_from_search_item(&conn, &item).unwrap();
        assert_eq!(n, 1);
        let repos = load_known_repos(&conn);
        assert!(repos.contains("https://github.com/openai/codex"));
    }
}