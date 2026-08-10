use std::sync::LazyLock;
use std::time::Duration;

use chrono::Datelike;
use rusqlite::Connection;
use serde_json::Value;

use super::nt_memory_resource_ingest::{ResourceDescriptor, ResourceIngester};
use super::nt_memory_store as store;

fn http_client() -> &'static reqwest::blocking::Client {
    static CLIENT: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
        super::nt_http::run_blocking(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (nt_discovery_sources)")
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

fn browser_client() -> &'static reqwest::blocking::Client {
    static CLIENT: LazyLock<reqwest::blocking::Client> = LazyLock::new(|| {
        super::nt_http::run_blocking(|| {
            reqwest::blocking::Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
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

#[derive(Debug, Clone, Default)]
pub struct ExternalDiscoveryStats {
    pub queries_executed: usize,
    pub resources_found: usize,
    pub resources_ingested: usize,
    pub errors: Vec<(String, String)>,
}

// ======================================================================
// 1. 古籍 / Classic Books — Project Gutenberg
// ======================================================================
pub fn discover_gutenberg(conn: &Connection, query: &str, limit: usize) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();
    let mut ingester = ResourceIngester::new(conn);

    let url = format!(
        "https://gutendex.com/books?search={}&languages=en,zh",
        urlencoding(query),
    );

    let resp = super::nt_http::run_blocking(|| {
        http_client()
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
    })
    .map_err(|e| format!("Gutenberg fetch error: {}", e))?;

    let data: serde_json::Value = resp.json().map_err(|e| format!("JSON error: {}", e))?;
    let results = data["results"].as_array().ok_or_else(|| "Missing results".to_string())?;
    stats.queries_executed = 1;

    for (i, book) in results.iter().enumerate() {
        if i >= limit { break; }
        let title = book["title"].as_str().unwrap_or("Unknown");
        let authors: Vec<String> = book["authors"].as_array()
            .map(|a| a.iter().filter_map(|v: &Value| v["name"].as_str().map(|s: &str| s.to_string())).collect())
            .unwrap_or_default();
        let author_str = authors.join(", ");
        let summary = format!("{} — Project Gutenberg classic book", title);
        let book_id = book["id"].as_i64().unwrap_or(0);
        let book_url = format!("https://www.gutenberg.org/ebooks/{}", book_id);

        let mut desc = ResourceDescriptor::article(title, &summary, &book_url)
            .with_tags(vec!["book", "gutenberg", "classic", &format!("absorbed-{}", today())])
            .with_importance(0.5)
            .with_confidence(0.8);

        if !author_str.is_empty() {
            desc = desc.with_content(&format!("Author(s): {}", author_str));
        }

        if let Ok(result) = ingester.ingest(&desc) {
            stats.resources_ingested += 1;
            let _ = store::update_node_metadata(conn, &result.node_id, &serde_json::json!({
                "source": "gutenberg", "book_id": book_id, "authors": authors
            }));
        }
        stats.resources_found += 1;
    }
    Ok(stats)
}

/// ======================================================================
/// 2. 学术论文 / Papers — Semantic Scholar API
/// ======================================================================
pub fn discover_semantic_scholar(conn: &Connection, query: &str, limit: usize) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();
    let mut ingester = ResourceIngester::new(conn);

    let url = format!(
        "https://api.semanticscholar.org/graph/v1/paper/search?query={}&limit={}&fields=title,abstract,url,venue,year,authors",
        urlencoding(query),
        limit.min(100)
    );

    let resp = super::nt_http::run_blocking(|| {
        http_client()
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
    })
    .map_err(|e| format!("Semantic Scholar error: {}", e))?;

    let data: serde_json::Value = resp.json().map_err(|e| format!("JSON error: {}", e))?;
    let papers = data["data"].as_array().ok_or_else(|| "Missing data".to_string())?;
    stats.queries_executed = 1;

    for paper in papers {
        let title = paper["title"].as_str().unwrap_or("Unknown");
        let _abstract_s = paper["abstract"].as_str().unwrap_or("");
        let paper_url = paper["url"].as_str().unwrap_or("");
        let venue = paper["venue"].as_str().unwrap_or("unknown");
        let year = paper["year"].as_i64().unwrap_or(0);
        let authors: Vec<String> = paper["authors"].as_array()
            .map(|a| a.iter().filter_map(|v| v["name"].as_str().map(|s| s.to_string())).collect::<Vec<_>>())
            .unwrap_or_default();

        let summary = format!("{} ({}, {}) — via Semantic Scholar", title, venue, year);

        let desc = ResourceDescriptor::article(title, &summary, paper_url)
            .with_tags(vec!["paper", "semantic-scholar", venue, &format!("absorbed-{}", today())])
            .with_importance(0.7)
            .with_confidence(0.7);

        match ingester.ingest(&desc) {
            Ok(r) => {
                stats.resources_ingested += 1;
                let _ = store::update_node_metadata(conn, &r.node_id, &serde_json::json!({
                    "source": "semantic_scholar", "venue": venue, "year": year, "authors": authors
                }));
            }
            Err(e) => stats.errors.push((title.to_string(), e)),
        }
        stats.resources_found += 1;
    }
    Ok(stats)
}

/// ======================================================================
/// 3. 古迹 / Historical Sites & 石碑 / Stone Inscriptions — Pleiades
/// ======================================================================
pub fn discover_historical_sites(conn: &Connection, query: &str, limit: usize) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();
    let mut ingester = ResourceIngester::new(conn);

    let url = format!(
        "https://pleiades.stoa.org/places/search?q={}&format=json&limit={}",
        urlencoding(query),
        limit.min(100)
    );

    let resp = super::nt_http::run_blocking(|| {
        http_client()
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .header("Accept", "application/json")
            .send()
    })
    .map_err(|e| format!("Pleiades error: {}", e))?;

    let data: serde_json::Value = resp.json().map_err(|e| format!("JSON error: {}", e))?;
    let features = data["features"].as_array().ok_or_else(|| "Missing features".to_string())?;
    stats.queries_executed = 1;

    for feature in features {
        let props = &feature["properties"];
        let title = props["title"].as_str().unwrap_or("Unknown site");
        let description = props["description"].as_str().unwrap_or("");
        let site_url = format!("https://pleiades.stoa.org/places/{}", props["uri"].as_str().unwrap_or(""));
        let site_type: String = props["placeTypes"].as_array()
            .map(|t| t.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>().join(","))
            .unwrap_or_default();

        let desc = ResourceDescriptor::article(title, description, &site_url)
            .with_tags(vec!["historical-site", "pleiades", "ancient-world", &format!("absorbed-{}", today())])
            .with_importance(0.6)
            .with_confidence(0.7);

        match ingester.ingest(&desc) {
            Ok(r) => {
                stats.resources_ingested += 1;
                let _ = store::update_node_metadata(conn, &r.node_id, &serde_json::json!({
                    "source": "pleiades", "site_type": site_type
                }));
            }
            Err(e) => stats.errors.push((title.to_string(), e)),
        }
        stats.resources_found += 1;
    }
    Ok(stats)
}

/// ======================================================================
/// 4. 博物馆藏品 / Museum Collections — Europeana
/// ======================================================================
pub fn discover_europeana(conn: &Connection, query: &str, limit: usize) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();
    let mut ingester = ResourceIngester::new(conn);

    let url = format!(
        "https://api.europeana.eu/record/v2/search.json?query={}&rows={}&wskey=apidemo",
        urlencoding(query),
        limit.min(100)
    );

    let resp = super::nt_http::run_blocking(|| {
        http_client()
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
    })
    .map_err(|e| format!("Europeana error: {}", e))?;

    let data: serde_json::Value = resp.json().map_err(|e| format!("JSON error: {}", e))?;
    let items = data["items"].as_array().ok_or_else(|| "Missing items".to_string())?;
    stats.queries_executed = 1;

    for item in items {
        let title = item["title"].as_array()
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let summary = item["dcDescription"].as_array()
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let item_url = item["guid"].as_str().unwrap_or("");
        let provider = item["dataProvider"].as_array()
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let desc = ResourceDescriptor::article(title, summary, item_url)
            .with_tags(vec!["museum", "europeana", "cultural-heritage", &format!("absorbed-{}", today())])
            .with_importance(0.6)
            .with_confidence(0.7);

        match ingester.ingest(&desc) {
            Ok(r) => {
                stats.resources_ingested += 1;
                let _ = store::update_node_metadata(conn, &r.node_id, &serde_json::json!({
                    "source": "europeana", "provider": provider
                }));
            }
            Err(e) => stats.errors.push((title.to_string(), e)),
        }
        stats.resources_found += 1;
    }
    Ok(stats)
}

/// ======================================================================
/// 5. 铭刻 / Inscriptions — PHI (Packard Humanities Institute) via DMMapper
/// ======================================================================
pub fn discover_inscriptions(conn: &Connection, query: &str, limit: usize) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();
    let mut ingester = ResourceIngester::new(conn);

    let url = format!(
        "https://www.dmmapper.com/api/search?q={}&type=inscription&limit={}",
        urlencoding(query),
        limit.min(50)
    );

    let resp = super::nt_http::run_blocking(|| {
        http_client()
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
    })
    .map_err(|e| format!("Inscription search error: {}", e))?;

    if resp.status().is_success() {
        if let Ok(data) = resp.json::<serde_json::Value>() {
            let results = data["results"].as_array().cloned().unwrap_or_default();
            stats.queries_executed = 1;
            for res in results.iter().take(limit) {
                let title = res["title"].as_str().unwrap_or("Unknown inscription");
                let desc_text = res["description"].as_str().unwrap_or("");
                let url_s = res["url"].as_str().unwrap_or("");

                let desc = ResourceDescriptor::article(title, desc_text, url_s)
                    .with_tags(vec!["inscription", "epigraphy", "ancient-writing", &format!("absorbed-{}", today())])
                    .with_importance(0.65);

                match ingester.ingest(&desc) {
                    Ok(_) => stats.resources_ingested += 1,
                    Err(e) => stats.errors.push((title.to_string(), e)),
                }
                stats.resources_found += 1;
            }
        }
    }

    if stats.resources_found == 0 {
        // Fallback: create a structured record from query
        let desc = ResourceDescriptor::concept(
            &format!("Inscription search: {}", query),
            &format!("Inscriptions related to '{}' — search query logged for later retrieval", query),
        ).with_tags(vec!["inscription", "epigraphy", "search-query", &format!("absorbed-{}", today())]);
        if ingester.ingest(&desc).is_ok() {
            stats.resources_ingested += 1;
        }
        stats.resources_found += 1;
    }

    Ok(stats)
}

/// ======================================================================
/// 6. 古籍 / Chinese Ancient Texts — 国学大师 / Chinese Text Project
/// ======================================================================
pub fn discover_chinese_ancient(conn: &Connection, query: &str, limit: usize) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();
    let mut ingester = ResourceIngester::new(conn);

    let url = format!(
        "https://ctext.org/api/search?q={}&limit={}&format=json",
        urlencoding(query),
        limit.min(50)
    );

    let resp = super::nt_http::run_blocking(|| {
        http_client()
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
    });

    if let Ok(resp) = resp {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<serde_json::Value>() {
                let results = data["results"].as_array().cloned().unwrap_or_default();
                stats.queries_executed = 1;
                for res in results.iter().take(limit) {
                    let title = res["title"].as_str().unwrap_or("Unknown");
                    let excerpt = res["excerpt"].as_str().unwrap_or("");
                    let url_s = res["url"].as_str().unwrap_or("");

                    let desc = ResourceDescriptor::article(title, excerpt, url_s)
                        .with_tags(vec!["chinese", "ancient-text", "古籍", &format!("absorbed-{}", today())])
                        .with_importance(0.6);

                    match ingester.ingest(&desc) {
                        Ok(_) => stats.resources_ingested += 1,
                        Err(e) => stats.errors.push((title.to_string(), e)),
                    }
                    stats.resources_found += 1;
                }
            }
        }
    }

    if stats.resources_found == 0 {
        // Log as search concept
        let d = ResourceDescriptor::concept(
            &format!("Chinese ancient text search: {}", query),
            &format!("Query for classical Chinese texts related to '{}'", query),
        ).with_tags(vec!["chinese", "古籍", "search-query", &format!("absorbed-{}", today())]);
        if ingester.ingest(&d).is_ok() { stats.resources_ingested += 1; }
        stats.resources_found += 1;
    }

    Ok(stats)
}

/// ======================================================================
/// 7. Internet Archive / 互联网档案馆
/// ======================================================================
pub fn discover_internet_archive(conn: &Connection, query: &str, limit: usize) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();
    let mut ingester = ResourceIngester::new(conn);

    let url = format!(
        "https://archive.org/advancedsearch.php?q={}&fl[]=identifier,title,description,creator,mediatype&rows={}&output=json",
        urlencoding(query),
        limit.min(50)
    );

    let resp = super::nt_http::run_blocking(|| {
        http_client()
            .get(&url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
    })
    .map_err(|e| format!("Internet Archive error: {}", e))?;

    let data: serde_json::Value = resp.json().map_err(|e| format!("JSON error: {}", e))?;
    let docs = data["response"]["docs"].as_array().ok_or_else(|| "Missing docs".to_string())?;
    stats.queries_executed = 1;

    for doc in docs {
        let title = doc["title"].as_str().unwrap_or("Unknown");
        let desc_text = doc["description"].as_str().unwrap_or("");
        let identifier = doc["identifier"].as_str().unwrap_or("");
        let creator = doc["creator"].as_str().unwrap_or("");
        let mediatype = doc["mediatype"].as_str().unwrap_or("unknown");
        let item_url = format!("https://archive.org/details/{}", identifier);

        let mut desc = ResourceDescriptor::article(title, desc_text, &item_url)
            .with_tags(vec!["internet-archive", mediatype, &format!("absorbed-{}", today())])
            .with_importance(0.5)
            .with_confidence(0.7);

        if !creator.is_empty() {
            desc = desc.with_content(&format!("Creator: {}", creator));
        }

        match ingester.ingest(&desc) {
            Ok(r) => {
                stats.resources_ingested += 1;
                let _ = store::update_node_metadata(conn, &r.node_id, &serde_json::json!({
                    "source": "internet_archive", "mediatype": mediatype, "creator": creator
                }));
            }
            Err(e) => stats.errors.push((title.to_string(), e)),
        }
        stats.resources_found += 1;
    }
    Ok(stats)
}

/// ======================================================================
/// 8. HathiTrust Digital Library / 数字图书馆
/// ======================================================================
pub fn discover_hathitrust(conn: &Connection, query: &str, limit: usize) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();
    let mut ingester = ResourceIngester::new(conn);

    let url = format!(
        "https://catalog.hathitrust.org/api/volumes/brief/json/search?q={}&rows={}",
        urlencoding(query),
        limit.min(50)
    );

    let resp = super::nt_http::run_blocking(|| {
        http_client()
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
    })
    .map_err(|e| format!("HathiTrust error: {}", e))?;

    let data: serde_json::Value = resp.json().map_err(|e| format!("JSON error: {}", e))?;
    let records = data["records"].as_array().cloned().unwrap_or_default();
    stats.queries_executed = 1;

    for rec in records.iter().take(limit) {
        let title = rec["title"].as_str().unwrap_or("Unknown");
        let author = rec["author"].as_str().unwrap_or("");
        let pub_info = rec["publisher"].as_str().unwrap_or("");
        let rec_url = format!("https://catalog.hathitrust.org/Record/{}", rec["recordID"].as_str().unwrap_or(""));

        let desc = ResourceDescriptor::article(title, &format!("{} — HathiTrust digital library", title), &rec_url)
            .with_tags(vec!["book", "hathitrust", "digital-library", &format!("absorbed-{}", today())])
            .with_importance(0.5);

        match ingester.ingest(&desc) {
            Ok(r) => {
                stats.resources_ingested += 1;
                let _ = store::update_node_metadata(conn, &r.node_id, &serde_json::json!({
                    "source": "hathitrust", "author": author, "publisher": pub_info
                }));
            }
            Err(e) => stats.errors.push((title.to_string(), e)),
        }
        stats.resources_found += 1;
    }
    Ok(stats)
}

/// ======================================================================
/// 9. 考古 / Archaeology — Open Context
/// ======================================================================
pub fn discover_open_context(conn: &Connection, query: &str, limit: usize) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();
    let mut ingester = ResourceIngester::new(conn);

    let url = format!(
        "https://opencontext.org/query.json?q={}&rows={}",
        urlencoding(query),
        limit.min(50)
    );

    let resp = super::nt_http::run_blocking(|| {
        http_client()
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
    });

    if let Ok(resp) = resp {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<serde_json::Value>() {
                let items = data["items"].as_array().cloned().unwrap_or_default();
                stats.queries_executed = 1;
                for item in items.iter().take(limit) {
                    let label = item["label"].as_str().unwrap_or("Unknown artifact");
                    let desc_text = item["description"].as_str().unwrap_or("");
                    let item_url = item["url"].as_str().unwrap_or("");

                    let desc = ResourceDescriptor::article(label, desc_text, item_url)
                        .with_tags(vec!["archaeology", "open-context", "artifact", &format!("absorbed-{}", today())])
                        .with_importance(0.6);

                    match ingester.ingest(&desc) {
                        Ok(_) => stats.resources_ingested += 1,
                        Err(e) => stats.errors.push((label.to_string(), e)),
                    }
                    stats.resources_found += 1;
                }
            }
        }
    }

    if stats.resources_found == 0 {
        let d = ResourceDescriptor::concept(
            &format!("Archaeology search: {}", query),
            &format!("Archaeological records related to '{}'", query),
        ).with_tags(vec!["archaeology", "search-query", &format!("absorbed-{}", today())]);
        if ingester.ingest(&d).is_ok() { stats.resources_ingested += 1; }
        stats.resources_found += 1;
    }

    Ok(stats)
}

/// ======================================================================
/// 10. 技术文档 / Technical Documentation — Wikipedia / Wikisource
/// ======================================================================
pub fn discover_technical_docs(conn: &Connection, topic: &str) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();

    let created = super::nt_memory_crawl::ingest_from_wikipedia(conn, topic)?;
    stats.queries_executed = 1;
    stats.resources_found = created;
    stats.resources_ingested = created;

    Ok(stats)
}

/// ======================================================================
/// 11. Gallica (Bibliothèque nationale de France) / 法国国家图书馆
/// ======================================================================
pub fn discover_gallica(conn: &Connection, query: &str, limit: usize) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();
    let mut ingester = ResourceIngester::new(conn);

    let url = format!(
        "https://gallica.bnf.fr/SRU?operation=searchRetrieve&query=keywords.any+all+%22{}%22&maximumRecords={}&recordSchema=dc",
        urlencoding(query),
        limit.min(50)
    );

    let resp = super::nt_http::run_blocking(|| {
        http_client()
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
    })
    .map_err(|e| format!("Gallica error: {}", e))?;

    let xml = resp.text().map_err(|e| format!("Read error: {}", e))?;
    stats.queries_executed = 1;

    for record in xml.split("<record>").skip(1).take(limit) {
        let title = extract_xml_tag(record, "dc:title")
            .or_else(|| extract_xml_tag(record, "title"))
            .unwrap_or_else(|| "Unknown".to_string());
        let creator = extract_xml_tag(record, "dc:creator").unwrap_or_default();
        let desc_text = extract_xml_tag(record, "dc:description").unwrap_or_default();

        let desc = ResourceDescriptor::article(&title, &desc_text, "")
            .with_tags(vec!["gallica", "bnf", "french-heritage", &format!("absorbed-{}", today())])
            .with_importance(0.5);

        match ingester.ingest(&desc) {
            Ok(r) => {
                stats.resources_ingested += 1;
                let _ = store::update_node_metadata(conn, &r.node_id, &serde_json::json!({
                    "source": "gallica", "creator": creator
                }));
            }
            Err(e) => stats.errors.push((title, e)),
        }
        stats.resources_found += 1;
    }
    Ok(stats)
}

/// ======================================================================
/// 12. 书籍 / Books — OpenLibrary (existing, enhanced)
/// ======================================================================
pub fn discover_books(conn: &Connection, query: &str, limit: usize) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();
    let mut ingester = ResourceIngester::new(conn);

    let url = format!(
        "https://openlibrary.org/search.json?q={}&limit={}",
        urlencoding(query),
        limit.min(100)
    );

    let resp = super::nt_http::run_blocking(|| {
        http_client()
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
    })
    .map_err(|e| format!("OpenLibrary fetch error: {}", e))?;

    let data: serde_json::Value = resp.json().map_err(|e| format!("JSON error: {}", e))?;
    let docs = data["docs"].as_array().ok_or_else(|| "Missing docs".to_string())?;

    stats.queries_executed = 1;

    for doc in docs {
        let title = doc["title"].as_str().unwrap_or("Unknown");
        let author = doc["author_name"].as_array()
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown author");
        let first_publish = doc["first_publish_year"].as_i64().unwrap_or(0);
        let isbn = doc["isbn"].as_array()
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let cover_id = doc["cover_i"].as_i64();
        let summary = format!("{} by {} (published {})", title, author, first_publish);

        let book_url = if !isbn.is_empty() {
            format!("https://openlibrary.org/isbn/{}", isbn)
        } else {
            format!("https://openlibrary.org/search?q={}", urlencoding(title))
        };

        let mut desc = ResourceDescriptor::article(title, &summary, &book_url)
            .with_tags(vec![
                "book",
                "openlibrary",
                &format!("absorbed-{}", today()),
            ])
            .with_importance(0.5)
            .with_confidence(0.7);

        if let Some(cover) = cover_id {
            let cover_url = format!("https://covers.openlibrary.org/b/id/{}-L.jpg", cover);
            desc = desc.with_content(&format!("Cover: {}", cover_url));
        }

        match ingester.ingest(&desc) {
            Ok(result) => {
                stats.resources_ingested += 1;
                let _ = store::update_node_metadata(
                    conn,
                    &result.node_id,
                    &serde_json::json!({"source_type": "openlibrary", "author": author, "first_publish_year": first_publish}),
                );
            }
            Err(e) => {
                stats.errors.push((title.to_string(), e));
            }
        }
        stats.resources_found += 1;
    }

    Ok(stats)
}

/// ======================================================================
/// 13. Wikipedia topic ingest (leverages existing)
/// ======================================================================
pub fn discover_wikipedia_topic(conn: &Connection, topic: &str) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();
    let created = super::nt_memory_crawl::ingest_from_wikipedia(conn, topic)?;
    stats.queries_executed = 1;
    stats.resources_found = created;
    stats.resources_ingested = created;
    Ok(stats)
}

/// ======================================================================
/// 14. ArXiv papers (leverages existing, enhanced)
/// ======================================================================
pub fn discover_arxiv_papers(conn: &Connection, query: &str, max_results: usize) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();
    let mut ingester = ResourceIngester::new(conn);

    let url = format!(
        "https://export.arxiv.org/api/query?search_query=all:{}&start=0&max_results={}&sortBy=relevance&sortOrder=desc",
        urlencoding(query),
        max_results.min(100)
    );

    let resp = super::nt_http::run_blocking(|| {
        http_client()
            .get(&url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
    })
    .map_err(|e| format!("ArXiv fetch error: {}", e))?;

    let text = resp.text().map_err(|e| format!("Text error: {}", e))?;
    stats.queries_executed = 1;

    for entry in text.split("<entry>").skip(1) {
        let arxiv_id = extract_xml_tag(entry, "id")
            .unwrap_or_default()
            .trim()
            .trim_start_matches("http://arxiv.org/abs/")
            .trim_start_matches("https://arxiv.org/abs/")
            .to_string();
        let title = extract_xml_tag(entry, "title")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Unknown".into());
        let summary = extract_xml_tag(entry, "summary")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let authors_raw = extract_xml_tag(entry, "author")
            .unwrap_or_default();

        if arxiv_id.is_empty() || title == "Unknown" {
            continue;
        }

        stats.resources_found += 1;

        let mut desc = ResourceDescriptor::paper(&arxiv_id, &title, &summary)
            .with_tags(vec![
                "arxiv",
                &format!("absorbed-{}", today()),
            ])
            .with_importance(0.7)
            .with_confidence(0.8);

        if !authors_raw.is_empty() {
            desc = desc.with_content(&format!("Authors: {}", authors_raw));
        }

        match ingester.ingest(&desc) {
            Ok(r) => {
                stats.resources_ingested += 1;
                let _ = store::update_node_metadata(conn, &r.node_id, &serde_json::json!({
                    "source": "arxiv", "arxiv_id": arxiv_id
                }));
            }
            Err(e) => stats.errors.push((title, e)),
        }
    }

    Ok(stats)
}

/// ======================================================================
/// 15. Anna's Archive / 安娜档案 (shadow library)
/// ======================================================================
pub fn discover_annas_archive(conn: &Connection, query: &str, limit: usize) -> Result<ExternalDiscoveryStats, String> {
    let mut stats = ExternalDiscoveryStats::default();
    let mut ingester = ResourceIngester::new(conn);

    let domains = ["annas-archive.gl", "annas-archive.org"];
    let mut last_err = String::new();

    for domain in &domains {
        let url = format!(
            "https://{}/search?q={}",
            domain,
            urlencoding(query),
        );

        match super::nt_http::run_blocking(|| {
            browser_client()
                .get(&url)
                .timeout(std::time::Duration::from_secs(15))
                .send()
        })
        {
            Ok(resp) => {
                let html = resp.text().map_err(|e| format!("Read error: {}", e))?;
                stats.queries_executed = 1;
                let parsed = parse_annas_page(&mut ingester, &html, limit);
                stats.resources_found += parsed.resources_found;
                stats.resources_ingested += parsed.resources_ingested;
                stats.errors.extend(parsed.errors);
                return Ok(stats);
            }
            Err(e) => {
                last_err = format!("Anna's Archive {}: {}", domain, e);
            }
        }
    }

    Err(last_err)
}

fn parse_annas_page(ingester: &mut ResourceIngester, html: &str, limit: usize) -> ExternalDiscoveryStats {
    let mut stats = ExternalDiscoveryStats::default();
    let mut count = 0;
    let mut pos = 0;
    while let Some(start) = html[pos..].find("<div class=\"h-[1px]\"></div>") {
        if count >= limit { break; }
        let block_start = pos + start;
        let block_end = html[block_start..].find("<div class=\"flex justify-center items-center gap-4\">")
            .map(|e| block_start + e)
            .unwrap_or(html.len());
        let block = &html[block_start..block_end];

        let title = extract_between(block, "aria-label=\"", "\"")
            .or_else(|| extract_between(block, "<h3>", "</h3>"))
            .unwrap_or_else(|| "Unknown".to_string());
        let link = extract_between(block, "href=\"", "\"")
            .unwrap_or_default();
        let page_url = if link.starts_with("http") {
            link
        } else {
            format!("https://annas-archive.gl{}", link)
        };

        let desc = ResourceDescriptor::article(&title, &format!("Book from Anna's Archive: {}", title), &page_url)
            .with_tags(vec![
                "book",
                "annas-archive",
                &format!("absorbed-{}", today()),
            ])
            .with_importance(0.5);

        match ingester.ingest(&desc) {
            Ok(_) => stats.resources_ingested += 1,
            Err(e) => stats.errors.push((title, e)),
        }
        stats.resources_found += 1;
        count += 1;
        pos = block_end + 1;
    }

    stats
}

/// ======================================================================
/// Helper functions
/// ======================================================================
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

fn extract_between(text: &str, start_delim: &str, end_delim: &str) -> Option<String> {
    let start = text.find(start_delim)?;
    let value_start = start + start_delim.len();
    let end = text[value_start..].find(end_delim)?;
    Some(text[value_start..value_start + end].to_string())
}

fn urlencoding(s: &str) -> String {
    s.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        ' ' => "+".to_string(),
        _ => format!("%{:02X}", c as u8),
    }).collect()
}

fn today() -> String {
    let now = chrono::Utc::now();
    format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day())
}

#[cfg(test)]
mod tests {
    use super::*;



    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("hello world"), "hello+world");
        assert_eq!(urlencoding("rust/lang"), "rust%2Flang");
    }

    #[test]
    fn test_extract_xml_tag() {
        let xml = "<title>Hello</title>";
        assert_eq!(extract_xml_tag(xml, "title"), Some("Hello".into()));
    }

    #[test]
    fn test_extract_between() {
        let html = "<a href=\"https://example.com\">link</a>";
        assert_eq!(extract_between(html, "href=\"", "\""), Some("https://example.com".into()));
    }

    #[test]
    fn test_today_format() {
        let t = today();
        assert_eq!(t.len(), 10);
        assert!(t.contains('-'));
    }

    #[test]
    fn test_discovery_stats_default() {
        let stats = ExternalDiscoveryStats::default();
        assert_eq!(stats.queries_executed, 0);
        assert_eq!(stats.resources_found, 0);
    }
}
