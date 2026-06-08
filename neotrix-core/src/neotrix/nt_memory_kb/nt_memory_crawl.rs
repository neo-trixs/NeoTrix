use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use super::nt_memory_store as store;
use super::nt_memory_types::*;

fn http_client() -> &'static reqwest::blocking::Client {
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .user_agent("NeoTrix/0.18 (KnowledgeBase nt_world_crawl)")
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("HTTP client")
    })
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn domain(url: &str) -> String {
    url.split('/').nth(2).unwrap_or("").trim_start_matches("www.").to_string()
}

fn skip_url(url: &str) -> bool {
    if url.ends_with(".zip") || url.ends_with(".tar.gz") || url.ends_with(".gz")
        || url.ends_with(".mp3") || url.ends_with(".mp4") || url.ends_with(".avi")
        || url.ends_with(".mov") || url.ends_with(".epub") || url.ends_with(".dmg")
        || url.ends_with(".exe") || url.ends_with(".bin")
    { return true; }
    let d = domain(url);
    if d.is_empty() { return true; }
    if d.starts_with("api.") && d != "api.github.com" { return true; }
    if d == "link.springer.com" || d == "academic.oup.com"
        || d == "jstor.org" || d == "www.jstor.org"
        || d == "sciencedirect.com" || d == "www.sciencedirect.com"
        || d == "cambridge.org" || d == "www.cambridge.org"
        || d == "tandfonline.com" || d == "www.tandfonline.com"
        || d == "onlinelibrary.wiley.com" || d == "wiley.com"
        || d == "emerald.com" || d == "www.emerald.com"
        || d == "degruyter.com" || d == "www.degruyter.com"
        || d == "mdpi.com" || d == "www.mdpi.com"
        || d == "frontiersin.org" || d == "www.frontiersin.org"
        || d == "id.loc.gov" || d == "export.arxiv.org"
        || d == "doi.org" || d == "dx.doi.org"
        || d == "worldcat.org" || d == "d-nb.info"
        || d == "openlibrary.org"
        || d == "books.google.com" || d.starts_with("books.google.")
        || d.starts_with("google.") || d == "www.google.com"
        || d.starts_with("scholar.google.")
        || d == "facebook.com" || d == "twitter.com" || d == "x.com"
        || d == "reddit.com" || d == "linkedin.com"
        || d == "pinterest.com" || d == "youtube.com"
        || d == "tumblr.com" || d == "instagram.com"
        || d == "amazon.com" || d == "indiebound.org"
        || d == "booksamillion.com"
    { return true; }
    if d == "api.semanticscholar.org" && url.contains("/CorpusID:") { return true; }
    false
}

fn try_parse_metadata_url(conn: &Connection, url: &str) -> Option<Result<(usize, usize), String>> {
    let lower = url.to_lowercase();
    let is_google = lower.contains(".google.") && lower.contains("/search");
    let is_scholar = lower.contains("scholar.google.");

    if !is_google && !is_scholar { return None; }

    let raw_query = url.split('?').nth(1).unwrap_or("");
    let cleaned_query = raw_query.replace("&amp;", "&").replace("&#x3D;", "=").replace("&#38;", "&");
    let params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(cleaned_query.as_bytes())
            .map(|(k, v)| (k.to_lowercase(), v.into_owned()))
            .collect();

    if is_google {
        if let Some(q) = params.get("q") {
            let qt = q.trim();
            if !qt.is_empty() {
                match store::insert_or_get_node(conn, qt, NodeType::Concept, Some(&format!("Google search: {}", qt)), Some(url), Some("google.com")) {
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
                match store::insert_or_get_node(conn, qt, NodeType::Paper, Some(&format!("Google Scholar search: {}", qt)), Some(url), Some("scholar.google.com")) {
                    Ok(_) => return Some(Ok((1, 0))),
                    Err(e) => return Some(Err(format!("DB error: {}", e))),
                }
            }
        }
        return None;
    }
    let title = params.get("title").map(|s| s.trim()).unwrap_or("").to_string();
    if title.is_empty() && params.get("doi").is_none() { return None; }
    let doi = params.get("doi").map(|s| s.trim()).unwrap_or("").to_string();
    let journal = params.get("journal").map(|s| s.trim()).unwrap_or("").to_string();
    let year = params.get("publication_year").map(|s| s.trim()).unwrap_or("").to_string();
    let volume = params.get("volume").map(|s| s.trim()).unwrap_or("").to_string();
    let pages = params.get("pages").map(|s| s.trim()).unwrap_or("").to_string();
    let authors_raw = params.get("author").map(|s| s.trim()).unwrap_or("").to_string();
    let node_title = if !title.is_empty() { title.clone() } else { format!("doi:{}", doi) };
    let summary = format!("Journal: {}\nYear: {}\nVolume: {}\nPages: {}\nDOI: {}", journal, year, volume, pages, doi);
    let summary_opt = if summary == "Journal: \nYear: \nVolume: \nPages: \nDOI: " { None } else { Some(summary.as_str()) };
    let node_id = match store::insert_or_get_node(conn, &node_title, NodeType::Paper, summary_opt, Some(url), Some("scholar.google.com")) {
        Ok(id) => id,
        Err(e) => return Some(Err(format!("DB error: {}", e))),
    };
    let mut edges = 0;
    for author_name in authors_raw.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        if let Ok(author_id) = store::insert_or_get_node(conn, author_name, NodeType::Person, None, None, Some("scholar.google.com")) {
            let _ = store::upsert_edge(conn, &node_id, &author_id, RelationType::DevelopedBy, 1.0, Some("Author of paper"));
            edges += 1;
        }
    }
    if !doi.is_empty() {
        let _ = store::upsert_crawl_queue(conn, &format!("https://doi.org/{}", doi), 1, "doi.org", 5, now());
    }
    Some(Ok((1, edges)))
}

pub fn reset_stuck_items(conn: &Connection) -> Result<usize, String> {
    conn.execute(
        "UPDATE crawl_queue SET status = 'pending' WHERE status = 'processing'",
        [],
    ).map_err(|e| format!("Reset stuck error: {}", e))
}

pub fn purge_skip_domains(conn: &Connection) -> Result<usize, String> {
    let mut total = 0;
    for d in &["doi.org", "dx.doi.org", "worldcat.org", "d-nb.info", "openlibrary.org",
               "id.loc.gov", "link.springer.com", "academic.oup.com", "jstor.org",
               "sciencedirect.com", "cambridge.org", "tandfonline.com",
               "onlinelibrary.wiley.com", "emerald.com", "degruyter.com",
               "mdpi.com", "frontiersin.org", "export.arxiv.org",
               "facebook.com", "twitter.com", "x.com", "reddit.com", "linkedin.com",
               "pinterest.com", "youtube.com", "tumblr.com", "instagram.com",
               "amazon.com", "indiebound.org", "booksamillion.com",
               "api.semanticscholar.org"] {
        if let Ok(n) = conn.execute("DELETE FROM crawl_queue WHERE status='pending' AND domain=?1", rusqlite::params![d]) { total += n; }
    }
    if let Ok(n) = conn.execute("DELETE FROM crawl_queue WHERE status='pending' AND domain LIKE 'books.google.%'", []) { total += n; }
    Ok(total)
}

pub fn enqueue_seed_urls(conn: &Connection, topic_urls: &[(&str, i64, &str)]) -> rusqlite::Result<usize> {
    let ts = now();
    let mut count = 0;
    for (url, priority, domain) in topic_urls {
        store::upsert_crawl_queue(conn, url, 0, domain, *priority, ts)?;
        count += 1;
    }
    Ok(count)
}

pub fn ingest_from_wikipedia(conn: &Connection, topic: &str) -> Result<usize, String> {
    let url = format!("https://en.wikipedia.org/api/rest_v1/page/summary/{}", topic);
    let resp = http_client().get(&url).send().map_err(|e| format!("Wikipedia fetch error: {}", e))?;
    let data: serde_json::Value = resp.json().map_err(|e| format!("JSON parse error: {}", e))?;
    let title = data["title"].as_str().unwrap_or(topic);
    let summary = data["extract"].as_str().unwrap_or("");
    let page_url = format!("https://en.wikipedia.org/wiki/{}", topic);
    let node_id = store::insert_or_get_node(conn, title, NodeType::Concept, Some(summary), Some(&page_url), Some("wikipedia.org"))
        .map_err(|e| format!("DB error: {}", e))?;
    if let Some(links) = data["links"].as_array() {
        for link in links {
            if let Some(link_title) = link.as_str() {
                if let Ok(link_id) = store::insert_or_get_node(conn, link_title, NodeType::Concept, None, None, Some("wikipedia.org")) {
                    let _ = store::upsert_edge(conn, &node_id, &link_id, RelationType::References, 1.0, Some("Wikipedia cross-reference"));
                }
            }
        }
    }
    Ok(1)
}

pub fn ingest_from_arxiv(conn: &Connection, arxiv_id: &str) -> Result<usize, String> {
    let url = format!("https://export.arxiv.org/api/query?id_list={}", arxiv_id);
    let resp = http_client().get(&url).send().map_err(|e| format!("arXiv fetch error: {}", e))?;
    let text = resp.text().map_err(|e| format!("Text error: {}", e))?;
    let title = extract_xml_tag(&text, "title").unwrap_or_else(|| "Unknown".into());
    let summary = extract_xml_tag(&text, "summary").unwrap_or_default();
    let authors_str = extract_xml_tag(&text, "author").unwrap_or_default();
    let paper_url = format!("https://arxiv.org/abs/{}", arxiv_id);
    let node_id = store::insert_or_get_node(conn, &title, NodeType::Paper, Some(&summary), Some(&paper_url), Some("arxiv.org"))
        .map_err(|e| format!("DB error: {}", e))?;
    for author in authors_str.split(", ") {
        let trimmed = author.trim();
        if !trimmed.is_empty() {
            if let Ok(author_id) = store::insert_or_get_node(conn, trimmed, NodeType::Person, None, None, Some("arxiv.org")) {
                let _ = store::upsert_edge(conn, &node_id, &author_id, RelationType::DevelopedBy, 1.0, Some("Author"));
            }
        }
    }
    Ok(1)
}

pub fn ingest_from_github(conn: &Connection, owner: &str, repo: &str) -> Result<usize, String> {
    let api_url = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let resp = http_client().get(&api_url).send().map_err(|e| format!("GitHub fetch error: {}", e))?;
    let data: serde_json::Value = resp.json().map_err(|e| format!("JSON parse error: {}", e))?;
    let default_title = format!("{}/{}", owner, repo);
    let title = data["full_name"].as_str().unwrap_or(&default_title);
    let description = data["description"].as_str().unwrap_or("");
    let repo_url = data["html_url"].as_str().unwrap_or(&api_url);
    let topics: Vec<String> = data["topics"].as_array().map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()).unwrap_or_default();
    let node_id = store::insert_or_get_node(conn, title, NodeType::Repository, Some(description), Some(repo_url), Some("github.com"))
        .map_err(|e| format!("DB error: {}", e))?;
    if let Some(owner_data) = data["owner"].as_object() {
        if let Some(owner_login) = owner_data.get("login").and_then(|v| v.as_str()) {
            if let Ok(owner_id) = store::insert_or_get_node(conn, owner_login, NodeType::Organization, None, Some(&format!("https://github.com/{}", owner_login)), Some("github.com")) {
                let _ = store::upsert_edge(conn, &node_id, &owner_id, RelationType::DevelopedBy, 1.0, Some("Repository owner"));
            }
        }
    }
    for topic in &topics {
        if let Ok(topic_id) = store::insert_or_get_node(conn, topic, NodeType::Concept, None, None, Some("github.com")) {
            let _ = store::upsert_edge(conn, &node_id, &topic_id, RelationType::Related, 1.0, Some("GitHub topic"));
        }
    }
    Ok(1)
}

pub fn run_crawl_cycle(conn: &Connection, max_items: usize) -> Result<CrawlCycleReport, String> {
    run_crawl_cycle_with(conn, max_items, true)
}

pub fn run_crawl_cycle_with(conn: &Connection, max_items: usize, fetch_links: bool) -> Result<CrawlCycleReport, String> {
    let mut report = CrawlCycleReport::default();
    for _ in 0..max_items {
        let item = match store::claim_next_crawl_url(conn)
            .map_err(|e| format!("DB claim error: {}", e))? {
            Some(item) => item,
            None => break,
        };
        report.attempted += 1;
        if let Some(result) = try_parse_metadata_url(conn, &item.url) {
            match result {
                Ok((nodes, edges)) => {
                    let _ = store::mark_crawl_complete(conn, &item.id, true, None);
                    report.completed += 1; report.nodes_created += nodes; report.edges_created += edges;
                }
                Err(e) => {
                    let truncated = &e[..std::cmp::min(e.len(), 500)];
                    let _ = store::mark_crawl_complete(conn, &item.id, false, Some(truncated));
                    report.failed += 1; report.errors.push((item.url.clone(), e));
                }
            }
            continue;
        }
        if skip_url(&item.url) {
            let _ = store::mark_crawl_complete(conn, &item.id, false, Some("Skipped: known bad pattern"));
            report.failed += 1; report.errors.push((item.url.clone(), "Skipped: known bad pattern".into()));
            continue;
        }
        match fetch_and_ingest_url(conn, &item.url, fetch_links) {
            Ok((nodes, edges)) => {
                let _ = store::mark_crawl_complete(conn, &item.id, true, None);
                report.completed += 1; report.nodes_created += nodes; report.edges_created += edges;
            }
            Err(e) => {
                let truncated = &e[..std::cmp::min(e.len(), 500)];
                let _ = store::mark_crawl_complete(conn, &item.id, false, Some(truncated));
                report.failed += 1; report.errors.push((item.url.clone(), e));
            }
        }
    }
    Ok(report)
}

pub fn run_crawl_cycle_parallel(
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
    if raw_items.is_empty() { return Ok(CrawlCycleReport::default()); }
    let completed = Arc::new(AtomicUsize::new(0));
    let failed = Arc::new(AtomicUsize::new(0));
    let nodes_created = Arc::new(AtomicUsize::new(0));
    let edges_created = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
    let db_path = db_path.to_string();
    std::thread::scope(|s| {
        let n = num_workers.max(1);
        for chunk in raw_items.chunks(n) {
            let chunk: Vec<_> = chunk.iter().map(|i| (i.id.clone(), i.url.clone())).collect();
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
                                nc.fetch_add(n, Ordering::SeqCst); ec.fetch_add(e, Ordering::SeqCst);
                            }
                            Err(e_msg) => {
                                let slice = &e_msg[..std::cmp::min(e_msg.len(), 500)];
                                let _ = store::mark_crawl_complete(&conn, id, false, Some(slice));
                                f.fetch_add(1, Ordering::SeqCst);
                                errs.lock().unwrap().push((url.clone(), e_msg));
                            }
                        }
                        continue;
                    }
                    if skip_url(url) {
                        let _ = store::mark_crawl_complete(&conn, id, false, Some("Skipped: known bad pattern"));
                        f.fetch_add(1, Ordering::SeqCst);
                        errs.lock().unwrap().push((url.clone(), "Skipped: known bad pattern".into()));
                        continue;
                    }
                    match fetch_and_ingest_url(&conn, url, fetch_links) {
                        Ok((n, e)) => {
                            let _ = store::mark_crawl_complete(&conn, id, true, None);
                            c.fetch_add(1, Ordering::SeqCst);
                            nc.fetch_add(n, Ordering::SeqCst); ec.fetch_add(e, Ordering::SeqCst);
                        }
                        Err(e_msg) => {
                            let slice = &e_msg[..std::cmp::min(e_msg.len(), 500)];
                            let _ = store::mark_crawl_complete(&conn, id, false, Some(slice));
                            f.fetch_add(1, Ordering::SeqCst);
                            errs.lock().unwrap().push((url.clone(), e_msg));
                        }
                    }
                }
            });
        }
    });
    Ok(CrawlCycleReport {
        attempted: raw_items.len(),
        completed: completed.load(Ordering::SeqCst),
        failed: failed.load(Ordering::SeqCst),
        nodes_created: nodes_created.load(Ordering::SeqCst),
        edges_created: edges_created.load(Ordering::SeqCst),
        urls_processed: Vec::new(),
        errors: errors.lock().map(|m| m.clone()).unwrap_or_default(),
        by_domain: std::collections::HashMap::new(),
    })
}

fn fetch_and_ingest_url(conn: &Connection, url: &str, enqueue_links: bool) -> Result<(usize, usize), String> {
    let resp = http_client().get(url).send().map_err(|e| format!("Fetch error: {}", e))?;
    let status = resp.status();
    if !status.is_success() { return Err(format!("HTTP {}", status)); }
    let html = resp.text().map_err(|e| format!("Read error: {}", e))?;
    let (title, text) = extract_html_content(&html);
    if text.is_empty() { return Err("Empty content".into()); }
    let d = domain(url);
    let node_id = store::insert_or_get_node(conn, &title, NodeType::Article, Some(&text.chars().take(2000).collect::<String>()), Some(url), Some(&d))
        .map_err(|e| format!("DB error: {}", e))?;
    if !enqueue_links { return Ok((1, 0)); }
    let mut edges_created = 0;
    let discovered_links = extract_links(&html, url);
    let ts = now();
    for link in discovered_links.iter().take(50) {
        let link_domain = domain(link);
        if link_domain.is_empty() || link_domain == d { continue; }
        let _ = store::upsert_crawl_queue(conn, link, 1, &link_domain, 0, ts);
        if let Ok(Some(linked_node)) = store::find_node_by_url(conn, link) {
            let _ = store::upsert_edge(conn, &node_id, &linked_node.id, RelationType::References, 1.0, Some("Hyperlink"));
            edges_created += 1;
        }
    }
    Ok((1, edges_created))
}

fn extract_html_content(html: &str) -> (String, String) {
    let title = html.find("<title>").and_then(|s| {
        let start = s + 7;
        html[start..].find("</title>").map(|e| html[start..start + e].trim().to_string())
    }).unwrap_or_default();
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
            if c == '<' && html[i..].starts_with("</script") { in_script = false; i += 8; continue; }
            i += 1; continue;
        }
        if in_style {
            if c == '<' && html[i..].starts_with("</style") { in_style = false; i += 7; continue; }
            i += 1; continue;
        }
        if c == '<' {
            in_tag = true;
            if html[i..].to_lowercase().starts_with("<script") { in_script = true; }
            if html[i..].to_lowercase().starts_with("<style") { in_style = true; }
            i += 1; continue;
        }
        if c == '>' { in_tag = false; i += 1; continue; }
        if !in_tag && !in_script && !in_style { text.push(c); }
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
            if href.starts_with("http://") || href.starts_with("https://") { links.push(href.to_string()); }
            pos = start + end + 1;
        } else { break; }
    }
    links.sort(); links.dedup(); links
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    xml.find(&open).and_then(|s| {
        let start = s + open.len();
        xml[start..].find(&close).map(|e| xml[start..start + e].trim().to_string())
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

pub fn discover_from_seed(conn: &Connection, seed_topic: &str) -> Result<usize, String> {
    let url = format!("https://en.wikipedia.org/api/rest_v1/page/summary/{}", seed_topic);
    let resp = http_client().get(&url).send().map_err(|e| format!("Fetch error: {}", e))?;
    let data: serde_json::Value = resp.json().map_err(|e| format!("JSON error: {}", e))?;
    let title = data["title"].as_str().unwrap_or(seed_topic);
    let extract = data["extract"].as_str().unwrap_or("");
    let page_url = format!("https://en.wikipedia.org/wiki/{}", seed_topic);
    let node_id = store::insert_or_get_node(conn, title, NodeType::Concept, Some(extract), Some(&page_url), Some("wikipedia.org"))
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
                                if let Ok(link_id) = store::insert_or_get_node(conn, link_title, NodeType::Concept, None, None, Some("wikipedia.org")) {
                                    let _ = store::upsert_edge(conn, &node_id, &link_id, RelationType::References, 1.0, Some("Wikipedia link"));
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic() { assert!(true); }
}
