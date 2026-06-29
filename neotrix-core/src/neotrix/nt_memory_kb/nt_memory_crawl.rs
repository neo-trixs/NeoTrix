use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use rusqlite::Connection;

use super::nt_memory_store as store;
use super::nt_memory_types::*;
use crate::neotrix::nt_world_search::WebSearchEngine;

fn http_client() -> &'static reqwest::blocking::Client {
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .user_agent("NeoTrix/0.18 (KnowledgeBase nt_world_crawl)")
            .no_proxy()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("HTTP client")
    })
}

fn http_client_seed() -> &'static reqwest::blocking::Client {
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .user_agent("NeoTrix/0.18 (KnowledgeSeed)")
            .no_proxy()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("HTTP client")
    })
}

use crate::core::nt_core_time::unix_now;

fn domain(url: &str) -> String {
    url.split('/')
        .nth(2)
        .unwrap_or("")
        .trim_start_matches("www.")
        .to_string()
}

fn skip_url(url: &str) -> bool {
    if url.ends_with(".zip")
        || url.ends_with(".tar.gz")
        || url.ends_with(".gz")
        || url.ends_with(".mp3")
        || url.ends_with(".mp4")
        || url.ends_with(".avi")
        || url.ends_with(".mov")
        || url.ends_with(".epub")
        || url.ends_with(".dmg")
        || url.ends_with(".exe")
        || url.ends_with(".bin")
    {
        return true;
    }
    let d = domain(url);
    if d.is_empty() {
        return true;
    }
    if d.starts_with("api.") && d != "api.github.com" {
        return true;
    }
    if d == "link.springer.com"
        || d == "academic.oup.com"
        || d == "jstor.org"
        || d == "www.jstor.org"
        || d == "sciencedirect.com"
        || d == "www.sciencedirect.com"
        || d == "cambridge.org"
        || d == "www.cambridge.org"
        || d == "tandfonline.com"
        || d == "www.tandfonline.com"
        || d == "onlinelibrary.wiley.com"
        || d == "wiley.com"
        || d == "emerald.com"
        || d == "www.emerald.com"
        || d == "degruyter.com"
        || d == "www.degruyter.com"
        || d == "mdpi.com"
        || d == "www.mdpi.com"
        || d == "frontiersin.org"
        || d == "www.frontiersin.org"
        || d == "id.loc.gov"
        || d == "export.arxiv.org"
        || d == "doi.org"
        || d == "dx.doi.org"
        || d == "worldcat.org"
        || d == "d-nb.info"
        || d == "books.google.com"
        || d.starts_with("books.google.")
        || d.starts_with("google.")
        || d == "www.google.com"
        || d.starts_with("scholar.google.")
        || d == "facebook.com"
        || d == "reddit.com"
        || d == "linkedin.com"
        || d == "pinterest.com"
        || d == "youtube.com"
        || d == "tumblr.com"
        || d == "instagram.com"
        || d == "amazon.com"
        || d == "indiebound.org"
        || d == "booksamillion.com"
    {
        return true;
    }
    if d == "api.semanticscholar.org" && url.contains("/CorpusID:") {
        return true;
    }
    if d == "api.twitter.com" || d == "api.x.com" {
        return true;
    }
    if (d == "twitter.com" || d == "x.com") && url.contains("/i/api/") {
        return true;
    }
    false
}

fn try_parse_metadata_url(conn: &Connection, url: &str) -> Option<Result<(usize, usize), String>> {
    let lower = url.to_lowercase();
    let is_google = lower.contains(".google.") && lower.contains("/search");
    let is_scholar = lower.contains("scholar.google.");

    if !is_google && !is_scholar {
        return None;
    }

    let raw_query = url.split('?').nth(1).unwrap_or("");
    let cleaned_query = raw_query
        .replace("&amp;", "&")
        .replace("&#x3D;", "=")
        .replace("&#38;", "&");
    let params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(cleaned_query.as_bytes())
            .map(|(k, v)| (k.to_lowercase(), v.into_owned()))
            .collect();

    if is_google {
        if let Some(q) = params.get("q") {
            let qt = q.trim();
            if !qt.is_empty() {
                match store::insert_or_get_node(
                    conn,
                    qt,
                    NodeType::Concept,
                    Some(&format!("Google search: {}", qt)),
                    Some(url),
                    Some("google.com"),
                ) {
                    Ok(_) => return Some(Ok((1, 0))),
                    Err(e) => return Some(Err(format!("DB error: {}", e))),
                }
            }
        }
        return None;
    }

    if !lower.contains("/scholar_lookup") {
        if let Some(q) = params.get("q") {
            let qt = q.trim();
            if !qt.is_empty() {
                match store::insert_or_get_node(
                    conn,
                    qt,
                    NodeType::Paper,
                    Some(&format!("Google Scholar search: {}", qt)),
                    Some(url),
                    Some("scholar.google.com"),
                ) {
                    Ok(_) => return Some(Ok((1, 0))),
                    Err(e) => return Some(Err(format!("DB error: {}", e))),
                }
            }
        }
        return None;
    }
    let title = params
        .get("title")
        .map(|s| s.trim())
        .unwrap_or("")
        .to_string();
    if title.is_empty() && params.get("doi").is_none() {
        return None;
    }
    let doi = params
        .get("doi")
        .map(|s| s.trim())
        .unwrap_or("")
        .to_string();
    let journal = params
        .get("journal")
        .map(|s| s.trim())
        .unwrap_or("")
        .to_string();
    let year = params
        .get("publication_year")
        .map(|s| s.trim())
        .unwrap_or("")
        .to_string();
    let volume = params
        .get("volume")
        .map(|s| s.trim())
        .unwrap_or("")
        .to_string();
    let pages = params
        .get("pages")
        .map(|s| s.trim())
        .unwrap_or("")
        .to_string();
    let authors_raw = params
        .get("author")
        .map(|s| s.trim())
        .unwrap_or("")
        .to_string();
    let node_title = if !title.is_empty() {
        title.clone()
    } else {
        format!("doi:{}", doi)
    };
    let summary = format!(
        "Journal: {}\nYear: {}\nVolume: {}\nPages: {}\nDOI: {}",
        journal, year, volume, pages, doi
    );
    let summary_opt = if summary == "Journal: \nYear: \nVolume: \nPages: \nDOI: " {
        None
    } else {
        Some(summary.as_str())
    };
    let node_id = match store::insert_or_get_node(
        conn,
        &node_title,
        NodeType::Paper,
        summary_opt,
        Some(url),
        Some("scholar.google.com"),
    ) {
        Ok(id) => id,
        Err(e) => return Some(Err(format!("DB error: {}", e))),
    };
    let mut edges = 0;
    for author_name in authors_raw
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        if let Ok(author_id) = store::insert_or_get_node(
            conn,
            author_name,
            NodeType::Person,
            None,
            None,
            Some("scholar.google.com"),
        ) {
            let _ = store::upsert_edge(
                conn,
                &node_id,
                &author_id,
                RelationType::DevelopedBy,
                1.0,
                Some("Author of paper"),
            );
            edges += 1;
        }
    }
    if !doi.is_empty() {
        let _ = store::upsert_crawl_queue(
            conn,
            &format!("https://doi.org/{}", doi),
            1,
            "doi.org",
            5,
            unix_now(),
        );
    }
    Some(Ok((1, edges)))
}

pub fn reset_stuck_items(conn: &Connection) -> Result<usize, String> {
    conn.execute(
        "UPDATE crawl_queue SET status = 'pending' WHERE status = 'processing'",
        [],
    )
    .map_err(|e| format!("Reset stuck error: {}", e))
}

pub fn purge_all_skip_patterns(conn: &Connection) -> Result<usize, String> {
    let mut total = 0usize;
    let urls: Vec<(String, String)> = conn
        .prepare("SELECT id, url FROM crawl_queue WHERE status='pending'")
        .map_err(|e| format!("Prepare error: {}", e))?
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let url: String = row.get(1)?;
            Ok((id, url))
        })
        .map_err(|e| format!("Query error: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    let mut batch: Vec<String> = Vec::new();
    for (id, url) in &urls {
        if skip_url(url) {
            batch.push(id.clone());
            if batch.len() >= 500 {
                let placeholders: Vec<String> = batch
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!("?{}", i + 1))
                    .collect();
                let sql = format!(
                    "DELETE FROM crawl_queue WHERE id IN ({})",
                    placeholders.join(",")
                );
                let mut stmt = conn
                    .prepare(&sql)
                    .map_err(|e| format!("Prepare batch: {}", e))?;
                let params: Vec<&dyn rusqlite::types::ToSql> = batch
                    .iter()
                    .map(|s| s as &dyn rusqlite::types::ToSql)
                    .collect();
                let n = stmt
                    .execute(params.as_slice())
                    .map_err(|e| format!("Delete batch: {}", e))?;
                total += n;
                batch.clear();
            }
        }
    }
    if !batch.is_empty() {
        let placeholders: Vec<String> = batch
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 1))
            .collect();
        let sql = format!(
            "DELETE FROM crawl_queue WHERE id IN ({})",
            placeholders.join(",")
        );
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Prepare batch: {}", e))?;
        let params: Vec<&dyn rusqlite::types::ToSql> = batch
            .iter()
            .map(|s| s as &dyn rusqlite::types::ToSql)
            .collect();
        let n = stmt
            .execute(params.as_slice())
            .map_err(|e| format!("Delete batch: {}", e))?;
        total += n;
    }
    Ok(total)
}

pub fn purge_skip_domains(conn: &Connection) -> Result<usize, String> {
    let mut total = 0;
    for d in &[
        "doi.org",
        "dx.doi.org",
        "worldcat.org",
        "d-nb.info",
        "id.loc.gov",
        "link.springer.com",
        "academic.oup.com",
        "jstor.org",
        "sciencedirect.com",
        "cambridge.org",
        "tandfonline.com",
        "onlinelibrary.wiley.com",
        "emerald.com",
        "degruyter.com",
        "mdpi.com",
        "frontiersin.org",
        "export.arxiv.org",
        "facebook.com",
        "reddit.com",
        "linkedin.com",
        "pinterest.com",
        "youtube.com",
        "tumblr.com",
        "instagram.com",
        "amazon.com",
        "indiebound.org",
        "booksamillion.com",
        "api.semanticscholar.org",
    ] {
        if let Ok(n) = conn.execute(
            "DELETE FROM crawl_queue WHERE status='pending' AND domain=?1",
            rusqlite::params![d],
        ) {
            total += n;
        }
    }
    if let Ok(n) = conn.execute(
        "DELETE FROM crawl_queue WHERE status='pending' AND domain LIKE 'books.google.%'",
        [],
    ) {
        total += n;
    }
    Ok(total)
}

pub async fn validate_urls_parallel(
    main_conn: &Connection,
    db_path: &str,
    num_workers: usize,
) -> Result<(usize, usize), String> {
    let count: i64 = main_conn
        .query_row(
            "SELECT COUNT(*) FROM crawl_queue WHERE status='pending'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if count == 0 {
        return Ok((0, 0));
    }
    let items = store::claim_crawl_urls_batch(main_conn, count as usize)
        .map_err(|e| format!("Claim error: {}", e))?;
    if items.is_empty() {
        return Ok((0, 0));
    }
    let alive = Arc::new(AtomicUsize::new(0));
    let dead = Arc::new(AtomicUsize::new(0));
    let dead_ids = Arc::new(Mutex::new(Vec::<String>::new()));
    let db_path = db_path.to_string();
    let client = Arc::new(
        reqwest::blocking::Client::builder()
            .user_agent("NeoTrix/0.18 (Validator)")
            .no_proxy()
            .timeout(Duration::from_secs(1))
            .build()
            .expect("HTTP client"),
    );
    tokio::task::spawn_blocking(move || {
        std::thread::scope(|s| {
            let n = num_workers.max(1);
            for chunk in items.chunks(n) {
                let chunk: Vec<_> = chunk
                    .iter()
                    .map(|i| (i.id.clone(), i.url.clone()))
                    .collect();
                let db = db_path.clone();
                let c = Arc::clone(&client);
                let a = Arc::clone(&alive);
                let d = Arc::clone(&dead);
                let di = Arc::clone(&dead_ids);
                s.spawn(move || {
                    let conn = Connection::open(&db).expect("Worker DB connection");
                    for (id, url) in &chunk {
                        let ok = if try_parse_metadata_url(&conn, url).is_some() {
                            true
                        } else if skip_url(url) {
                            false
                        } else {
                            c.head(url)
                                .send()
                                .map(|r| r.status().is_success())
                                .unwrap_or(false)
                        };
                        if ok {
                            let _ = conn.execute(
                                "UPDATE crawl_queue SET status='pending' WHERE id=?1",
                                rusqlite::params![id],
                            );
                            a.fetch_add(1, Ordering::SeqCst);
                        } else {
                            let _ = conn.execute(
                                "DELETE FROM crawl_queue WHERE id=?1",
                                rusqlite::params![id],
                            );
                            d.fetch_add(1, Ordering::SeqCst);
                            di.lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .push(id.clone());
                        }
                    }
                });
            }
        });
        (alive.load(Ordering::SeqCst), dead.load(Ordering::SeqCst))
    })
    .await
    .map_err(|e| format!("Blocking scope error: {}", e))
}

pub fn enqueue_seed_urls(
    conn: &Connection,
    topic_urls: &[(&str, i64, &str)],
) -> rusqlite::Result<usize> {
    let ts = unix_now();
    let mut count = 0;
    for (url, priority, domain) in topic_urls {
        store::ensure_crawl_pending(conn, url, 0, domain, *priority, ts)?;
        count += 1;
    }
    Ok(count)
}

/// Search the web and enqueue discovered URLs into the crawl queue.
/// Uses WebSearchEngine (DuckDuckGo) to find URLs matching the query,
/// then calls ensure_crawl_pending for each result URL.
pub fn enqueue_search_results_from_engine(
    conn: &Connection,
    query: &str,
    max_results: usize,
    priority: i64,
) -> Result<usize, String> {
    let engine = WebSearchEngine::default();
    let results = engine.search(query, max_results)?;
    if results.is_empty() {
        return Ok(0);
    }
    let ts = unix_now();
    let mut count = 0usize;
    for result in &results {
        let d = domain(&result.url);
        if d.is_empty() || skip_url(&result.url) {
            continue;
        }
        if store::ensure_crawl_pending(conn, &result.url, 0, &d, priority, ts).is_ok() {
            count += 1;
        }
        // Also create a Concept node for the search result
        let _ = store::insert_or_get_node(
            conn,
            &result.title,
            NodeType::Concept,
            Some(&result.snippet),
            Some(&result.url),
            Some(&d),
        );
    }
    Ok(count)
}

pub fn ingest_from_wikipedia(conn: &Connection, topic: &str) -> Result<usize, String> {
    let url = format!(
        "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
        topic
    );
    let resp = http_client_seed()
        .get(&url)
        .send()
        .map_err(|e| format!("Wikipedia fetch error: {}", e))?;
    let data: serde_json::Value = resp
        .json()
        .map_err(|e| format!("JSON parse error: {}", e))?;
    let title = data["title"].as_str().unwrap_or(topic);
    let summary = data["extract"].as_str().unwrap_or("");
    let page_url = format!("https://en.wikipedia.org/wiki/{}", topic);
    let node_id = store::insert_or_get_node(
        conn,
        title,
        NodeType::Concept,
        Some(summary),
        Some(&page_url),
        Some("wikipedia.org"),
    )
    .map_err(|e| format!("DB error: {}", e))?;
    if let Some(links) = data["links"].as_array() {
        for link in links {
            if let Some(link_title) = link.as_str() {
                if let Ok(link_id) = store::insert_or_get_node(
                    conn,
                    link_title,
                    NodeType::Concept,
                    None,
                    None,
                    Some("wikipedia.org"),
                ) {
                    let _ = store::upsert_edge(
                        conn,
                        &node_id,
                        &link_id,
                        RelationType::References,
                        1.0,
                        Some("Wikipedia cross-reference"),
                    );
                }
            }
        }
    }
    Ok(1)
}

pub fn ingest_from_arxiv(conn: &Connection, arxiv_id: &str) -> Result<usize, String> {
    let url = format!("https://export.arxiv.org/api/query?id_list={}", arxiv_id);
    let resp = http_client_seed()
        .get(&url)
        .send()
        .map_err(|e| format!("arXiv fetch error: {}", e))?;
    let text = resp.text().map_err(|e| format!("Text error: {}", e))?;
    let title = extract_xml_tag(&text, "title").unwrap_or_else(|| "Unknown".into());
    let summary = extract_xml_tag(&text, "summary").unwrap_or_default();
    let authors_str = extract_xml_tag(&text, "author").unwrap_or_default();
    let paper_url = format!("https://arxiv.org/abs/{}", arxiv_id);
    let node_id = store::insert_or_get_node(
        conn,
        &title,
        NodeType::Paper,
        Some(&summary),
        Some(&paper_url),
        Some("arxiv.org"),
    )
    .map_err(|e| format!("DB error: {}", e))?;
    for author in authors_str.split(", ") {
        let trimmed = author.trim();
        if !trimmed.is_empty() {
            if let Ok(author_id) = store::insert_or_get_node(
                conn,
                trimmed,
                NodeType::Person,
                None,
                None,
                Some("arxiv.org"),
            ) {
                let _ = store::upsert_edge(
                    conn,
                    &node_id,
                    &author_id,
                    RelationType::DevelopedBy,
                    1.0,
                    Some("Author"),
                );
            }
        }
    }
    Ok(1)
}

pub fn ingest_from_openlibrary_search(conn: &Connection, query: &str) -> Result<usize, String> {
    let url = format!(
        "https://openlibrary.org/search.json?q={}&limit=50",
        urlencode(query)
    );
    let resp = http_client_seed()
        .get(&url)
        .send()
        .map_err(|e| format!("OL fetch error: {}", e))?;
    let data: serde_json::Value = resp
        .json()
        .map_err(|e| format!("JSON parse error: {}", e))?;
    let docs = data["docs"]
        .as_array()
        .ok_or_else(|| "No docs in response".to_string())?;
    let mut count = 0;
    for doc in docs {
        let title = doc["title"].as_str().unwrap_or("Untitled");
        let author = doc["author_name"][0].as_str().unwrap_or("Unknown");
        let ol_id = doc["key"].as_str().unwrap_or("");
        let first_year = doc["first_publish_year"].as_i64().unwrap_or(0);
        let summary = Some(
            &format!(
                "Author: {} | First published: {} | OL ID: {}",
                author, first_year, ol_id
            )[..],
        );
        let ol_url = format!("https://openlibrary.org{}", ol_id);
        let node_id = match store::insert_or_get_node(
            conn,
            title,
            NodeType::Article,
            summary,
            Some(&ol_url),
            Some("openlibrary.org"),
        ) {
            Ok(id) => id,
            Err(_) => continue,
        };
        if !author.is_empty() && author != "Unknown" {
            if let Ok(author_id) = store::insert_or_get_node(
                conn,
                author,
                NodeType::Person,
                None,
                None,
                Some("openlibrary.org"),
            ) {
                let _ = store::upsert_edge(
                    conn,
                    &node_id,
                    &author_id,
                    RelationType::DevelopedBy,
                    1.0,
                    Some("Author"),
                );
            }
        }
        count += 1;
    }
    Ok(count)
}

pub fn ingest_from_github(conn: &Connection, owner: &str, repo: &str) -> Result<usize, String> {
    let api_url = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let resp = http_client_seed()
        .get(&api_url)
        .send()
        .map_err(|e| format!("GitHub fetch error: {}", e))?;
    let data: serde_json::Value = resp
        .json()
        .map_err(|e| format!("JSON parse error: {}", e))?;
    let default_title = format!("{}/{}", owner, repo);
    let title = data["full_name"].as_str().unwrap_or(&default_title);
    let description = data["description"].as_str().unwrap_or("");
    let repo_url = data["html_url"].as_str().unwrap_or(&api_url);
    let topics: Vec<String> = data["topics"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
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
    if let Some(owner_data) = data["owner"].as_object() {
        if let Some(owner_login) = owner_data.get("login").and_then(|v| v.as_str()) {
            if let Ok(owner_id) = store::insert_or_get_node(
                conn,
                owner_login,
                NodeType::Organization,
                None,
                Some(&format!("https://github.com/{}", owner_login)),
                Some("github.com"),
            ) {
                let _ = store::upsert_edge(
                    conn,
                    &node_id,
                    &owner_id,
                    RelationType::DevelopedBy,
                    1.0,
                    Some("Repository owner"),
                );
            }
        }
    }
    for topic in &topics {
        if let Ok(topic_id) = store::insert_or_get_node(
            conn,
            topic,
            NodeType::Concept,
            None,
            None,
            Some("github.com"),
        ) {
            let _ = store::upsert_edge(
                conn,
                &node_id,
                &topic_id,
                RelationType::Related,
                1.0,
                Some("GitHub topic"),
            );
        }
    }
    Ok(1)
}

pub fn ingest_github_search(conn: &Connection, query: &str) -> Result<usize, String> {
    let url = format!(
        "https://api.github.com/search/repositories?q={}&sort=stars&per_page=30",
        urlencode(query)
    );
    let resp = http_client_seed()
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .map_err(|e| format!("GitHub search fetch error: {}", e))?;
    let data: serde_json::Value = resp
        .json()
        .map_err(|e| format!("JSON parse error: {}", e))?;
    let items = data["items"]
        .as_array()
        .ok_or_else(|| "No items in response".to_string())?;
    let mut count = 0;
    for item in items {
        let owner = item["owner"]["login"].as_str().unwrap_or("");
        let repo = item["name"].as_str().unwrap_or("");
        if owner.is_empty() || repo.is_empty() {
            continue;
        }
        let default_title = format!("{}/{}", owner, repo);
        let title = item["full_name"].as_str().unwrap_or(&default_title);
        let description = item["description"].as_str().unwrap_or("");
        let repo_url = item["html_url"].as_str().unwrap_or("");
        let topics: Vec<String> = item["topics"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let node_id = match store::insert_or_get_node(
            conn,
            title,
            NodeType::Repository,
            Some(description),
            Some(repo_url),
            Some("github.com"),
        ) {
            Ok(id) => id,
            Err(_) => continue,
        };
        if let Some(owner_login) = item["owner"]["login"].as_str() {
            if let Ok(owner_id) = store::insert_or_get_node(
                conn,
                owner_login,
                NodeType::Organization,
                None,
                Some(&format!("https://github.com/{}", owner_login)),
                Some("github.com"),
            ) {
                let _ = store::upsert_edge(
                    conn,
                    &node_id,
                    &owner_id,
                    RelationType::DevelopedBy,
                    1.0,
                    None,
                );
            }
        }
        for topic in &topics {
            if let Ok(topic_id) = store::insert_or_get_node(
                conn,
                topic,
                NodeType::Concept,
                None,
                None,
                Some("github.com"),
            ) {
                let _ = store::upsert_edge(
                    conn,
                    &node_id,
                    &topic_id,
                    RelationType::Related,
                    1.0,
                    Some("GitHub topic"),
                );
            }
        }
        count += 1;
    }
    Ok(count)
}

pub fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "%20".to_string(),
            c => format!("%{:02X}", c as u8),
        })
        .collect()
}

pub fn run_crawl_cycle(conn: &Connection, max_items: usize) -> Result<CrawlCycleReport, String> {
    run_crawl_cycle_with(conn, max_items, true)
}

pub fn run_crawl_cycle_with(
    conn: &Connection,
    max_items: usize,
    fetch_links: bool,
) -> Result<CrawlCycleReport, String> {
    let mut report = CrawlCycleReport::default();
    for _ in 0..max_items {
        let item = match store::claim_next_crawl_url(conn)
            .map_err(|e| format!("DB claim error: {}", e))?
        {
            Some(item) => item,
            None => break,
        };
        report.attempted += 1;
        if let Some(result) = try_parse_metadata_url(conn, &item.url) {
            match result {
                Ok((nodes, edges)) => {
                    let _ = store::mark_crawl_complete(conn, &item.id, true, None);
                    report.completed += 1;
                    report.nodes_created += nodes;
                    report.edges_created += edges;
                }
                Err(e) => {
                    let truncated = &e[..std::cmp::min(e.len(), 500)];
                    let _ = store::mark_crawl_complete(conn, &item.id, false, Some(truncated));
                    report.failed += 1;
                    report.errors.push((item.url.clone(), e));
                }
            }
            continue;
        }
        if skip_url(&item.url) {
            let _ = store::mark_crawl_complete(
                conn,
                &item.id,
                false,
                Some("Skipped: known bad pattern"),
            );
            report.failed += 1;
            report
                .errors
                .push((item.url.clone(), "Skipped: known bad pattern".into()));
            continue;
        }
        match fetch_and_ingest_url(conn, &item.url, fetch_links) {
            Ok((nodes, edges)) => {
                let _ = store::mark_crawl_complete(conn, &item.id, true, None);
                report.completed += 1;
                report.nodes_created += nodes;
                report.edges_created += edges;
            }
            Err(e) => {
                let truncated = &e[..std::cmp::min(e.len(), 500)];
                let _ = store::mark_crawl_complete(conn, &item.id, false, Some(truncated));
                report.failed += 1;
                report.errors.push((item.url.clone(), e));
            }
        }
    }
    Ok(report)
}

pub async fn run_crawl_cycle_parallel(
    main_conn: &Connection,
    db_path: &str,
    max_items: usize,
    num_workers: usize,
    fetch_links: bool,
) -> Result<CrawlCycleReport, String> {
    let raw_items = if let Ok(items) = store::claim_crawl_urls_batch(main_conn, max_items) {
        items
    } else {
        return Ok(CrawlCycleReport::default());
    };
    if raw_items.is_empty() {
        return Ok(CrawlCycleReport::default());
    }
    let completed = Arc::new(AtomicUsize::new(0));
    let failed = Arc::new(AtomicUsize::new(0));
    let nodes_created = Arc::new(AtomicUsize::new(0));
    let edges_created = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
    let db_path = db_path.to_string();
    tokio::task::spawn_blocking(move || {
        std::thread::scope(|s| {
            let n = num_workers.max(1);
            for chunk in raw_items.chunks(n) {
                let chunk: Vec<_> = chunk
                    .iter()
                    .map(|i| (i.id.clone(), i.url.clone()))
                    .collect();
                let db = db_path.clone();
                let c = Arc::clone(&completed);
                let f = Arc::clone(&failed);
                let nc = Arc::clone(&nodes_created);
                let ec = Arc::clone(&edges_created);
                let errs = Arc::clone(&errors);
                s.spawn(move || {
                    let conn = Connection::open(&db).expect("Worker DB connection");
                    for (id, url) in &chunk {
                        if let Some(result) = try_parse_metadata_url(&conn, url) {
                            match result {
                                Ok((n, e)) => {
                                    let _ = store::mark_crawl_complete(&conn, id, true, None);
                                    c.fetch_add(1, Ordering::SeqCst);
                                    nc.fetch_add(n, Ordering::SeqCst);
                                    ec.fetch_add(e, Ordering::SeqCst);
                                }
                                Err(e_msg) => {
                                    let slice = &e_msg[..std::cmp::min(e_msg.len(), 500)];
                                    let _ =
                                        store::mark_crawl_complete(&conn, id, false, Some(slice));
                                    f.fetch_add(1, Ordering::SeqCst);
                                    errs.lock()
                                        .unwrap_or_else(|e| e.into_inner())
                                        .push((url.clone(), e_msg));
                                }
                            }
                            continue;
                        }
                        if skip_url(url) {
                            let _ = store::mark_crawl_complete(
                                &conn,
                                id,
                                false,
                                Some("Skipped: known bad pattern"),
                            );
                            f.fetch_add(1, Ordering::SeqCst);
                            errs.lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .push((url.clone(), "Skipped: known bad pattern".into()));
                            continue;
                        }
                        match fetch_and_ingest_url(&conn, url, fetch_links) {
                            Ok((n, e)) => {
                                let _ = store::mark_crawl_complete(&conn, id, true, None);
                                c.fetch_add(1, Ordering::SeqCst);
                                nc.fetch_add(n, Ordering::SeqCst);
                                ec.fetch_add(e, Ordering::SeqCst);
                            }
                            Err(e_msg) => {
                                let slice = &e_msg[..std::cmp::min(e_msg.len(), 500)];
                                let _ = store::mark_crawl_complete(&conn, id, false, Some(slice));
                                f.fetch_add(1, Ordering::SeqCst);
                                errs.lock()
                                    .unwrap_or_else(|e| e.into_inner())
                                    .push((url.clone(), e_msg));
                            }
                        }
                    }
                });
            }
        });
        CrawlCycleReport {
            attempted: raw_items.len(),
            completed: completed.load(Ordering::SeqCst),
            failed: failed.load(Ordering::SeqCst),
            nodes_created: nodes_created.load(Ordering::SeqCst),
            edges_created: edges_created.load(Ordering::SeqCst),
            urls_processed: Vec::new(),
            errors: errors.lock().map(|m| m.clone()).unwrap_or_default(),
            by_domain: std::collections::HashMap::new(),
        }
    })
    .await
    .map_err(|e| format!("Blocking scope error: {}", e))
}

fn fetch_and_ingest_url(
    conn: &Connection,
    url: &str,
    enqueue_links: bool,
) -> Result<(usize, usize), String> {
    let resp = http_client()
        .get(url)
        .send()
        .map_err(|e| format!("Fetch error: {}", e))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status));
    }
    let html = resp.text().map_err(|e| format!("Read error: {}", e))?;
    let (title, text) = extract_html_content(&html);
    if text.is_empty() {
        return Err("Empty content".into());
    }
    let d = domain(url);
    let node_id = store::insert_or_get_node(
        conn,
        &title,
        NodeType::Article,
        Some(&text.chars().take(2000).collect::<String>()),
        Some(url),
        Some(&d),
    )
    .map_err(|e| format!("DB error: {}", e))?;
    if !enqueue_links {
        return Ok((1, 0));
    }
    let mut edges_created = 0;
    let discovered_links = extract_links(&html, url);
    let ts = unix_now();
    for link in discovered_links.iter().take(50) {
        let link_domain = domain(link);
        if link_domain.is_empty() || link_domain == d {
            continue;
        }
        let _ = store::upsert_crawl_queue(conn, link, 1, &link_domain, 0, ts);
        if let Ok(Some(linked_node)) = store::find_node_by_url(conn, link) {
            let _ = store::upsert_edge(
                conn,
                &node_id,
                &linked_node.id,
                RelationType::References,
                1.0,
                Some("Hyperlink"),
            );
            edges_created += 1;
        }
    }
    Ok((1, edges_created))
}

fn extract_html_content(html: &str) -> (String, String) {
    let title = html
        .find("<title>")
        .and_then(|s| {
            let start = s + 7;
            html[start..]
                .find("</title>")
                .map(|e| html[start..start + e].trim().to_string())
        })
        .unwrap_or_default();
    let mut text = String::new();
    let bytes = html.as_bytes();
    let mut i = 0;
    let n = bytes.len();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    while i < n {
        let c = bytes[i] as char;
        if in_script {
            if c == '<' && html[i..].starts_with("</script") {
                in_script = false;
                i += 8;
                continue;
            }
            i += 1;
            continue;
        }
        if in_style {
            if c == '<' && html[i..].starts_with("</style") {
                in_style = false;
                i += 7;
                continue;
            }
            i += 1;
            continue;
        }
        if c == '<' {
            in_tag = true;
            if html[i..].to_lowercase().starts_with("<script") {
                in_script = true;
            }
            if html[i..].to_lowercase().starts_with("<style") {
                in_style = true;
            }
            i += 1;
            continue;
        }
        if c == '>' {
            in_tag = false;
            i += 1;
            continue;
        }
        if !in_tag && !in_script && !in_style {
            text.push(c);
        }
        i += 1;
    }
    (title, text.split_whitespace().collect::<Vec<_>>().join(" "))
}

fn extract_links(html: &str, _base: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut pos = 0;
    while let Some(s) = html[pos..].find("href=\"") {
        let start = pos + s + 6;
        if let Some(end) = html[start..].find('"') {
            let href = &html[start..start + end];
            if href.starts_with("http://") || href.starts_with("https://") {
                links.push(href.to_string());
            }
            pos = start + end + 1;
        } else {
            break;
        }
    }
    links.sort();
    links.dedup();
    links
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    xml.find(&open).and_then(|s| {
        let start = s + open.len();
        xml[start..]
            .find(&close)
            .map(|e| xml[start..start + e].trim().to_string())
    })
}

#[derive(Debug, Clone, Default)]
pub struct CrawlCycleReport {
    pub attempted: usize,
    pub completed: usize,
    pub failed: usize,
    pub nodes_created: usize,
    pub edges_created: usize,
    pub urls_processed: Vec<String>,
    pub errors: Vec<(String, String)>,
    pub by_domain: std::collections::HashMap<String, usize>,
}

/// Discover all pages and subcategories in a Wikipedia category.
/// Uses the `categorymembers` API to find pages belonging to Category:{name}.
/// Creates Concept nodes for each page + BelongsToCategory edges.
/// Optionally enqueues discovered URLs to the crawl queue and recurses into subcategories.
pub fn discover_wiki_category_members(
    conn: &Connection,
    category: &str,
    max_pages: usize,
    max_depth: u32,
    enqueue_urls: bool,
) -> Result<(usize, usize, usize), String> {
    _discover_wiki_category_recursive(conn, category, max_pages, max_depth, enqueue_urls, 0)
}

fn _discover_wiki_category_recursive(
    conn: &Connection,
    category: &str,
    max_pages: usize,
    max_depth: u32,
    enqueue_urls: bool,
    depth: u32,
) -> Result<(usize, usize, usize), String> {
    if depth > max_depth {
        return Ok((0, 0, 0));
    }

    let encoded = urlencode(category);
    let api_url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&list=categorymembers\
         &cmtitle=Category%3A{}&cmtype=page%7Csubcat&cmlimit=50&format=json",
        encoded
    );

    let resp = http_client_seed()
        .get(&api_url)
        .send()
        .map_err(|e| format!("Wikipedia category fetch error: {}", e))?;
    let data: serde_json::Value = resp
        .json()
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let members = data["query"]["categorymembers"]
        .as_array()
        .ok_or_else(|| "No categorymembers in response".to_string())?;

    let mut nodes = 0usize;
    let mut edges = 0usize;
    let mut enqueued = 0usize;
    let ts = unix_now();
    let cat_node_id = store::insert_or_get_node(
        conn,
        &format!("Category:{}", category),
        NodeType::Concept,
        Some(&format!("Wikipedia category: {}", category)),
        Some(&format!(
            "https://en.wikipedia.org/wiki/Category:{}",
            category
        )),
        Some("wikipedia.org"),
    )
    .map_err(|e| format!("DB error: {}", e))?;

    for member in members.iter().take(max_pages) {
        let title = member["title"].as_str().unwrap_or("");
        let member_type = member["type"].as_str().unwrap_or("page");
        if title.is_empty() {
            continue;
        }

        if member_type == "subcat" {
            // Strip "Category:" prefix and recurse
            let subcat = title.strip_prefix("Category:").unwrap_or(title);
            let (n, e, eq) = _discover_wiki_category_recursive(
                conn,
                subcat,
                max_pages,
                max_depth,
                enqueue_urls,
                depth + 1,
            )?;
            nodes += n;
            edges += e;
            enqueued += eq;
            continue;
        }

        // Create page node
        let page_url = format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_"));
        let page_id = match store::insert_or_get_node(
            conn,
            title,
            NodeType::Concept,
            None,
            Some(&page_url),
            Some("wikipedia.org"),
        ) {
            Ok(id) => id,
            Err(_) => continue,
        };
        nodes += 1;

        // Create BelongsToCategory edge
        if store::upsert_edge(
            conn,
            &page_id,
            &cat_node_id,
            RelationType::BelongsToCategory,
            1.0,
            Some(&format!("Wikipedia category member: {}", category)),
        )
        .is_ok()
        {
            edges += 1;
        }

        // Optionally enqueue for crawling
        if enqueue_urls {
            let d = domain(&page_url);
            if store::ensure_crawl_pending(conn, &page_url, 1, &d, 3, ts).is_ok() {
                enqueued += 1;
            }
        }
    }

    Ok((nodes, edges, enqueued))
}

pub fn discover_from_seed(conn: &Connection, seed_topic: &str) -> Result<usize, String> {
    let url = format!(
        "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
        seed_topic
    );
    let resp = http_client()
        .get(&url)
        .send()
        .map_err(|e| format!("Fetch error: {}", e))?;
    let data: serde_json::Value = resp.json().map_err(|e| format!("JSON error: {}", e))?;
    let title = data["title"].as_str().unwrap_or(seed_topic);
    let extract = data["extract"].as_str().unwrap_or("");
    let page_url = format!("https://en.wikipedia.org/wiki/{}", seed_topic);
    let node_id = store::insert_or_get_node(
        conn,
        title,
        NodeType::Concept,
        Some(extract),
        Some(&page_url),
        Some("wikipedia.org"),
    )
    .map_err(|e| format!("DB error: {}", e))?;
    let mut count = 1;
    let links_url = format!("https://en.wikipedia.org/w/api.php?action=query&prop=links&titles={}&pllimit=50&format=json", title.replace(' ', "_"));
    if let Ok(resp) = http_client().get(&links_url).send() {
        if let Ok(data) = resp.json::<serde_json::Value>() {
            if let Some(pages) = data["query"]["pages"].as_object() {
                for page in pages.values() {
                    if let Some(links) = page["links"].as_array() {
                        for link in links {
                            if let Some(link_title) = link["title"].as_str() {
                                if let Ok(link_id) = store::insert_or_get_node(
                                    conn,
                                    link_title,
                                    NodeType::Concept,
                                    None,
                                    None,
                                    Some("wikipedia.org"),
                                ) {
                                    let _ = store::upsert_edge(
                                        conn,
                                        &node_id,
                                        &link_id,
                                        RelationType::References,
                                        1.0,
                                        Some("Wikipedia link"),
                                    );
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(count)
}
