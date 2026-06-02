use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use super::nt_memory_store as store;
use super::nt_memory_types::*;

fn http_client() -> &'static reqwest::blocking::Client {
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .user_agent("NeoTrix/0.18 (KnowledgeBase nt_world_crawl)")
            .no_proxy()
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
                let link_id = store::insert_or_get_node(
                    conn,
                    link_title,
                    NodeType::Concept,
                    None,
                    None,
                    Some("wikipedia.org"),
                )
                .map_err(|e| format!("DB error: {}", e))?;
                store::upsert_edge(
                    conn,
                    &node_id,
                    &link_id,
                    RelationType::References,
                    1.0,
                    Some("Wikipedia cross-reference"),
                )
                .map_err(|e| format!("DB error: {}", e))?;
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
    let summary_s = extract_xml_tag(&text, "summary").unwrap_or_default();
    let summary = summary_s.as_str();
    let authors_str = extract_xml_tag(&text, "author").unwrap_or_default();

    let paper_url = format!("https://arxiv.org/abs/{}", arxiv_id);

    let node_id = store::insert_or_get_node(
        conn,
        &title,
        NodeType::Paper,
        Some(summary),
        Some(&paper_url),
        Some("arxiv.org"),
    )
    .map_err(|e| format!("DB error: {}", e))?;

    for author in authors_str.split(", ") {
        let trimmed = author.trim();
        if !trimmed.is_empty() {
            let author_id = store::insert_or_get_node(
                conn,
                trimmed,
                NodeType::Person,
                None,
                None,
                Some("arxiv.org"),
            )
            .map_err(|e| format!("DB error: {}", e))?;
            store::upsert_edge(
                conn,
                &node_id,
                &author_id,
                RelationType::DevelopedBy,
                1.0,
                Some("Author"),
            )
            .map_err(|e| format!("DB error: {}", e))?;
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
    let stars = data["stargazers_count"].as_i64().unwrap_or(0);
    let topics: Vec<String> = data["topics"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let lang = data["language"].as_str().unwrap_or("unknown");

    let _metadata = serde_json::json!({
        "stars": stars,
        "topics": topics,
        "language": lang,
    });

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

pub fn run_crawl_cycle(conn: &Connection, max_items: usize) -> Result<CrawlCycleReport, String> {
    let mut report = CrawlCycleReport::default();

    for _ in 0..max_items {
        let item = store::claim_next_crawl_url(conn)
            .map_err(|e| format!("DB claim error: {}", e))?;

        let item = match item {
            Some(item) => item,
            None => break,
        };

        report.attempted += 1;
        let result = fetch_and_ingest_url(conn, &item.url);

        match result {
            Ok((nodes, edges)) => {
                store::mark_crawl_complete(conn, &item.id, true, None)
                    .map_err(|e| format!("DB error: {}", e))?;
                report.completed += 1;
                report.nodes_created += nodes;
                report.edges_created += edges;
                report.urls_processed.push(item.url.clone());

                let domain = item.domain.unwrap_or_else(|| "unknown".into());
                let entry = report.by_domain.entry(domain).or_insert(0);
                *entry += 1;
            }
            Err(e) => {
                let err_str = format!("{:?}", e);
                store::mark_crawl_complete(conn, &item.id, false, Some(&err_str[..std::cmp::min(err_str.len(), 500)]))
                    .map_err(|e| format!("DB error: {}", e))?;
                report.failed += 1;
                report.errors.push((item.url, err_str));
            }
        }
    }

    Ok(report)
}

fn fetch_and_ingest_url(conn: &Connection, url: &str) -> Result<(usize, usize), String> {
    let resp = http_client()
        .get(url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .map_err(|e| format!("Fetch error: {}", e))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status));
    }

    let _content_type = resp.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let html = resp.text().map_err(|e| format!("Read error: {}", e))?;
    let (title, text) = extract_html_content(&html);

    if text.is_empty() {
        return Err("Empty content".into());
    }

    let page_url = url.to_string();
    let domain = extract_domain(url);

    let node_id = store::insert_or_get_node(
        conn,
        &title,
        NodeType::Article,
        Some(&text.chars().take(2000).collect::<String>()),
        Some(&page_url),
        Some(&domain),
    )
    .map_err(|e| format!("DB error: {}", e))?;

    let nodes_created = 1;
    let mut edges_created = 0;

    let discovered_links = extract_links(&html, url);
    let ts = now();
    for link in discovered_links.iter().take(50) {
        let link_domain = extract_domain(link);
        if link_domain.is_empty() || link_domain == domain {
            continue;
        }

        store::upsert_crawl_queue(conn, link, 1, &link_domain, 0, ts)
            .map_err(|e| format!("DB queue error: {}", e))?;

        if let Ok(Some(linked_node)) = store::find_node_by_url(conn, link) {
            store::upsert_edge(
                conn,
                &node_id,
                &linked_node.id,
                RelationType::References,
                1.0,
                Some("Hyperlink"),
            )
            .map_err(|e| format!("DB edge error: {}", e))?;
            edges_created += 1;
        }
    }

    Ok((nodes_created, edges_created))
}

fn extract_html_content(html: &str) -> (String, String) {
    let title = if let Some(start) = html.find("<title>") {
        let start = start + 7;
        if let Some(end) = html[start..].find("</title>") {
            html[start..start + end].trim().to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let mut text = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut i = 0;
    let bytes = html.as_bytes();

    while i < bytes.len() {
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

    let text = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    (title, text)
}

fn extract_links(html: &str, _base_url: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut pos = 0;

    while let Some(start) = html[pos..].find("href=\"") {
        let start = pos + start + 6;
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

fn extract_domain(url: &str) -> String {
    url.split('/')
        .nth(2)
        .unwrap_or("")
        .trim_start_matches("www.")
        .to_string()
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    if let Some(start) = xml.find(&open) {
        let start = start + open.len();
        if let Some(end) = xml[start..].find(&close) {
            return Some(xml[start..start + end].trim().to_string());
        }
    }
    None
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
    let title_clean = title.replace(' ', "_");

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

    let links_url = format!("https://en.wikipedia.org/w/api.php?action=query&prop=links&titles={}&pllimit=50&format=json", title_clean);
    if let Ok(resp) = http_client().get(&links_url).send() {
        if let Ok(data) = resp.json::<serde_json::Value>() {
            if let Some(pages) = data["query"]["pages"].as_object() {
                for page in pages.values() {
                    if let Some(links) = page["links"].as_array() {
                        for link in links {
                            if let Some(link_title) = link["title"].as_str() {
                                let link_id = store::insert_or_get_node(
                                    conn,
                                    link_title,
                                    NodeType::Concept,
                                    None,
                                    None,
                                    Some("wikipedia.org"),
                                )
                                .ok();

                                if let Some(lid) = link_id {
                                    let _ = store::upsert_edge(
                                        conn,
                                        &node_id,
                                        &lid,
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


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
