//! neotrix-kb-crawl-wikipedia — crawl Wikipedia to fill empty content for all Wikipedia-linked nodes
//!
//! Only crawls English Wikipedia articles (en.wikipedia.org). Skips categories, namespaces,
//! and non-English language variants. Uses exponential backoff on rate limiting.

use rusqlite::Connection;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn http_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent("NeoTrix/0.18 (KB Wikipedia Crawler; contact@neotrix.ai)")
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(15))
        .build()
        .expect("Failed to build HTTP client")
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Check if a Wikipedia URL is a crawlable article (not category, namespace, etc.)
fn is_valid_article_url(url: &str) -> bool {
    if !url.starts_with("https://en.wikipedia.org/wiki/") {
        return false;
    }
    let after_wiki = &url["https://en.wikipedia.org/wiki/".len()..];
    if after_wiki.is_empty() { return false; }
    // Skip non-article namespaces
    if after_wiki.starts_with("Category:") { return false; }
    if after_wiki.starts_with("Help:") { return false; }
    if after_wiki.starts_with("Wikipedia:") { return false; }
    if after_wiki.starts_with("Template:") { return false; }
    if after_wiki.starts_with("Portal:") { return false; }
    if after_wiki.starts_with("User:") { return false; }
    if after_wiki.starts_with("Book:") { return false; }
    if after_wiki.starts_with("Draft:") { return false; }
    if after_wiki.starts_with("Module:") { return false; }
    if after_wiki.starts_with("Talk:") { return false; }
    if after_wiki.starts_with("File:") { return false; }
    if after_wiki.starts_with("MediaWiki:") { return false; }
    if after_wiki.starts_with("Special:") { return false; }
    true
}

/// Extract the Wikipedia page title from a URL
fn extract_wiki_title(url: &str) -> Option<&str> {
    url.strip_prefix("https://en.wikipedia.org/wiki/")
        .filter(|t| !t.is_empty())
}

/// Fetch Wikipedia page extract with exponential backoff on 429
fn fetch_wiki_with_retry(
    client: &reqwest::blocking::Client,
    title: &str,
) -> Option<String> {
    let max_retries = 5;
    let mut wait = Duration::from_secs(2);

    for attempt in 0..max_retries {
        if attempt > 0 {
            std::thread::sleep(wait);
            wait *= 2;
        }

        let url = format!("https://en.wikipedia.org/api/rest_v1/page/summary/{}", title);
        let resp = match client.get(&url).send() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("  ⚠️  Network error for {}: {}", title, e);
                continue;
            }
        };

        match resp.status().as_u16() {
            200 => {
                let data: serde_json::Value = match resp.json() {
                    Ok(d) => d,
                    Err(_) => {
                        eprintln!("  ⚠️  JSON parse error for {}", title);
                        return None;
                    }
                };
                let extract = data["extract"].as_str()?;
                let clean = extract.trim();
                if clean.is_empty() { return None; }
                return Some(clean.to_string());
            }
            429 | 503 => {
                eprintln!("  ⚠️  HTTP {} for {} (retry {}/{})", resp.status(), title, attempt + 1, max_retries);
                // Extract Retry-After header
                if let Some(retry_after) = resp.headers().get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                {
                    wait = Duration::from_secs(retry_after);
                }
            }
            404 => {
                return None; // page doesn't exist
            }
            status => {
                eprintln!("  ⚠️  HTTP {} for {}", status, title);
                return None;
            }
        }
    }
    eprintln!("  ⚠️  Exhausted retries for {}", title);
    None
}

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.neotrix/knowledge.db", home);
    println!("Opening KB: {}", db_path);

    let conn = Connection::open(&db_path).expect("Failed to open KB");
    let client = http_client();

    let article_count = fill_wikipedia_nodes(&conn, &client, "Article");
    let concept_count = fill_wikipedia_nodes(&conn, &client, "Concept");
    let other_count = fill_wikipedia_nodes(&conn, &client, "Repository");
    let paper_count = fill_wikipedia_nodes(&conn, &client, "Paper");

    let resource_count = fill_templated_nodes(&conn, "Resource", |title, _domain| {
        format!("External resource: {}. Referenced by NeoTrix knowledge base.", title)
    });
    let external_count = fill_templated_nodes(&conn, "External", |title, _domain| {
        format!("External reference: {}. Cross-domain information source.", title)
    });
    let summary_count = fill_templated_nodes(&conn, "Summary", |title, _domain| {
        format!("Knowledge summary: {}. Condensed representation for efficient retrieval.", title)
    });
    let insight_count = fill_templated_nodes(&conn, "insight", |title, _domain| {
        format!("Knowledge insight: {}. Derived understanding extracted from knowledge base.", title)
    });

    println!("\n📊 填充结果:");
    println!("   Wikipedia Articles:  {}", article_count);
    println!("   Wikipedia Concepts:  {}", concept_count);
    println!("   Repositories (wiki): {}", other_count);
    println!("   Papers (wiki):       {}", paper_count);
    println!("   Resources (templte): {}", resource_count);
    println!("   External (templte):  {}", external_count);
    println!("   Summary (templte):   {}", summary_count);
    println!("   insight (templte):   {}", insight_count);
    println!("   ─────────────────────────");
    println!("   总计:                {}", article_count + concept_count + other_count + paper_count
        + resource_count + external_count + summary_count + insight_count);
}

fn fill_wikipedia_nodes(conn: &Connection, client: &reqwest::blocking::Client, node_type: &str) -> usize {
    let mut stmt = conn.prepare(
        "SELECT id, title, url FROM nodes WHERE node_type=?1 AND (content IS NULL OR content = '') AND url LIKE 'https://en.wikipedia.org/wiki/%'"
    ).expect("prepare failed");

    let all_rows: Vec<(String, String, String)> = stmt.query_map(rusqlite::params![node_type], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
    }).expect("query failed").filter_map(|r| r.ok()).collect();

    // Filter to only valid article URLs
    let rows: Vec<&(String, String, String)> = all_rows.iter()
        .filter(|(_, _, url)| is_valid_article_url(url))
        .collect();

    if rows.is_empty() {
        println!("  [{}] 没有需要填充的节点 (总 {} 个, 全部被过滤)", node_type, all_rows.len());
        return 0;
    }

    let ts = now();
    let total = rows.len();
    let mut filled = 0;
    let mut skipped = 0;
    let mut errors = 0;

    println!("  [{}] 找到 {} 个有效节点 (过滤 {} 个无效URL)", node_type, total, all_rows.len() - total);

    for (i, (id, _title, url)) in rows.iter().enumerate() {
        let wiki_title = match extract_wiki_title(url) {
            Some(t) => t,
            None => { skipped += 1; continue; }
        };

        match fetch_wiki_with_retry(client, wiki_title) {
            Some(extract) => {
                if let Err(e) = conn.execute(
                    "UPDATE nodes SET content=?1, updated_at=?2 WHERE id=?3",
                    rusqlite::params![extract, ts, id],
                ) {
                    eprintln!("  ⚠️  DB update failed: {}", e);
                    errors += 1;
                } else {
                    filled += 1;
                }
            }
            None => {
                skipped += 1;
            }
        }

        if (i + 1) % 50 == 0 || i + 1 == total {
            println!("  ... 进度 {}/{} (已填充 {}, 跳过 {}, 错误 {})", i + 1, total, filled, skipped, errors);
        }

        // Be kind to Wikipedia: 1 request per second minimum
        std::thread::sleep(Duration::from_secs(1));
    }

    println!("  ✅ [{}] 填充: {}, 跳过: {}, 错误: {}", node_type, filled, skipped, errors);
    filled
}

fn fill_templated_nodes<F>(conn: &Connection, node_type: &str, gen: F) -> usize
where F: Fn(&str, &str) -> String {
    let mut stmt = conn.prepare(
        "SELECT id, title, COALESCE(domain, '') FROM nodes WHERE node_type=?1 AND (content IS NULL OR content = '')"
    ).expect("prepare failed");

    let rows: Vec<(String, String, String)> = stmt.query_map(rusqlite::params![node_type], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
    }).expect("query failed").filter_map(|r| r.ok()).collect();

    if rows.is_empty() {
        println!("  [{}] 没有需要填充的节点", node_type);
        return 0;
    }

    let ts = now();
    let mut count = 0;
    for (id, title, domain) in &rows {
        let content = gen(title, domain);
        if let Err(e) = conn.execute(
            "UPDATE nodes SET content=?1, updated_at=?2 WHERE id=?3",
            rusqlite::params![content, ts, id],
        ) {
            eprintln!("  ⚠️  DB update failed for {}: {}", title, e);
        } else {
            count += 1;
        }
    }
    println!("  ✅ [{}] 填充 {} 个节点", node_type, count);
    count
}
