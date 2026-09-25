//! nt_memory_crawl · 队列/周期核心: 防护、种子队列、爬取周期、检索发现.
use rusqlite::Connection;

use super::super::nt_memory_store as store;
use super::super::nt_memory_types::*;
use super::super::shared_utils::now;
use super::nt_crawl_parse::{extract_domain, extract_html_content, extract_links};
/// DI-aware HTTP 客户端访问 — 优先从容器解析，回退到共享客户端
pub(crate) fn http_client() -> &'static reqwest::blocking::Client {
    super::super::nt_http::shared_blocking_client()
}

/// 获取阻塞 HTTP 客户端 (DI-ready: 可从容器注入)
pub fn resolve_blocking_client() -> reqwest::blocking::Client {
    use crate::l0_substrate::nt_core_di;
    if let Some(v) = nt_core_di::resolve_global::<reqwest::blocking::Client>() {
        return v;
    }
    super::super::nt_http::shared_blocking_client().clone()
}

/// SSRF 防护 (OWASP 对齐)：URL 必须为 http/https，目标 IP 不得为内网/回环/链路本地/保留段。
/// 单一校验实现委托 `nt_http::resolve_safe_origin` (含 IPv4-mapped、编码绕过、DNS pin 校验)。
pub fn is_safe_fetch_url(url: &str) -> bool {
    super::super::nt_http::resolve_safe_origin(url).is_ok()
}

pub fn on_node_inserted(conn: &Connection, node: &KnowledgeNode) -> rusqlite::Result<()> {
    let ts = now();
    if let Some(ref url) = node.url {
        let domain = extract_domain(url);
        let priority = (node.importance * 10.0) as i64;
        store::upsert_crawl_queue(conn, url, 1, &domain, priority, ts)?;
    }
    Ok(())
}

pub fn enqueue_seed_urls(
    conn: &Connection,
    topic_urls: &[(&str, i64, &str)],
) -> rusqlite::Result<usize> {
    let ts = now();
    let mut count = 0;
    for (url, priority, domain) in topic_urls {
        store::upsert_crawl_queue(conn, url, 0, domain, *priority, ts)?;
        count += 1;
    }
    Ok(count)
}


pub fn run_crawl_cycle(conn: &Connection, max_items: usize) -> Result<CrawlCycleReport, String> {
    let mut report = CrawlCycleReport::default();

    for _ in 0..max_items {
        let item =
            store::claim_next_crawl_url(conn).map_err(|e| format!("DB claim error: {}", e))?;

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
                store::mark_crawl_complete(
                    conn,
                    &item.id,
                    false,
                    Some(&err_str[..std::cmp::min(err_str.len(), 500)]),
                )
                .map_err(|e| format!("DB error: {}", e))?;
                report.failed += 1;
                report.errors.push((item.url, err_str));
            }
        }
    }

    Ok(report)
}

fn fetch_and_ingest_url(conn: &Connection, url: &str) -> Result<(usize, usize), String> {
    if !is_safe_fetch_url(url) {
        return Err(format!("URL rejected (SSRF guard): {}", url));
    }
    // connect-期 DNS pinning (防 rebinding): guard 已在 is_safe_fetch_url,
    // fetch_safe_http 内部再次 resolve + pin。
    let (html, _host) = super::super::nt_http::fetch_safe_http(url)?;
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
    let url = format!(
        "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
        seed_topic
    );
    let resp = super::super::nt_http::run_blocking(|| http_client().get(&url).send())
        .map_err(|e| format!("Fetch error: {}", e))?;

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
    if let Ok(resp) = super::super::nt_http::run_blocking(|| http_client().get(&links_url).send()) {
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
