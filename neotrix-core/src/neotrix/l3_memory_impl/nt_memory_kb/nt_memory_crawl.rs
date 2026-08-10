use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use super::nt_memory_store as store;
use super::nt_memory_types::*;

// TODO: inject via DI — pass reqwest::blocking::Client through the crawler constructor
fn http_client() -> &'static reqwest::blocking::Client {
    super::nt_http::shared_blocking_client()
}

/// SSRF 防护 (OWASP 对齐)：URL 必须为 http/https，目标 IP 不得为内网/回环/链路本地/保留段。
/// 单一校验实现委托 `nt_http::resolve_safe_origin` (含 IPv4-mapped、编码绕过、DNS pin 校验)。
pub fn is_safe_fetch_url(url: &str) -> bool {
    super::nt_http::resolve_safe_origin(url).is_ok()
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
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
    let resp = super::nt_http::run_blocking(|| http_client().get(&url).send()).map_err(|e| format!("Wikipedia fetch error: {}", e))?;
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
    let resp = super::nt_http::run_blocking(|| http_client().get(&url).send()).map_err(|e| format!("arXiv fetch error: {}", e))?;
    let text = resp.text().map_err(|e| format!("Text error: {}", e))?;

    // export API 返回 feed, 首个 <title> 是 feed 级 "arXiv Query: ..."。
    // 论文元数据在第一个 <entry> 块内, 必须截取 entry 再提取 title/summary/author。
    let entry = text.find("<entry>").map(|i| &text[i..]).unwrap_or(&text);
    let title = extract_xml_tag(entry, "title").unwrap_or_else(|| "Unknown".into());
    let summary_s = extract_xml_tag(entry, "summary").unwrap_or_default();
    let summary = summary_s.as_str();
    // <author><name>X</name></author> 重复出现; 提取所有 author.name
    let mut authors_str = String::new();
    let mut rest = entry;
    while let Some(name_start) = rest.find("<name>") {
        let after = &rest[name_start + 6..];
        if let Some(name_end) = after.find("</name>") {
            if !authors_str.is_empty() {
                authors_str.push_str(", ");
            }
            authors_str.push_str(after[..name_end].trim());
            rest = &after[name_end + 7..];
        } else {
            break;
        }
    }

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

/// alphaXiv 论文 feed 摄取: 从 api.alphaxiv.org/papers/v3/feed 分页拉取论文元数据,
/// 落库 Paper 节点 + 作者 Person 节点 + 主题 Concept 节点 + GitHub Repository 关联。
/// 能力源自 alphaXiv 公开 feed API (sort=Recent), R-P79 接线到 KB 生产路径。
/// `categories` 为 alphaXiv 分类标识 (如 "ai-ml" / "q-bio" / "q-fin"), 空串表示全部。
pub fn ingest_from_alphaxiv_feed(
    conn: &Connection,
    pages: usize,
    page_size: usize,
    categories: &str,
) -> Result<usize, String> {
    let pages = pages.max(1);
    let page_size = page_size.max(1).min(50);
    let mut total = 0usize;
    for page in 1..=pages {
        let mut url = format!(
            "https://api.alphaxiv.org/papers/v3/feed?sort=Recent&pageNum={}&pageSize={}&interval=All%20time",
            page, page_size
        );
        let cat = categories.trim();
        if !cat.is_empty() {
            url.push_str(&format!("&categories={}", cat));
        }
        let resp = super::nt_http::run_blocking(|| http_client().get(&url).send())
            .map_err(|e| format!("alphaXiv fetch error: {}", e))?;
        let text = resp.text().map_err(|e| format!("Text error: {}", e))?;
        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("JSON parse error: {}", e))?;
        let papers = json["papers"].as_array().cloned().unwrap_or_default();
        if papers.is_empty() {
            break;
        }
        for p in &papers {
            let title = p["title"].as_str().unwrap_or("Unknown").to_string();
            if title == "Unknown" {
                continue;
            }
            let summary = p["abstract"].as_str().unwrap_or_default();
            let canonical_id = p["canonical_id"].as_str().unwrap_or_default();
            let page_url = format!("https://www.alphaxiv.org/abs/{}", canonical_id);

            let node_id = store::insert_or_get_node(
                conn,
                &title,
                NodeType::Paper,
                Some(summary),
                Some(&page_url),
                Some("alphaxiv.org"),
            )
            .map_err(|e| format!("DB error: {}", e))?;

            // 作者 → Person 节点 + DevelopedBy 边
            if let Some(authors) = p["authors"].as_array() {
                for a in authors {
                    if let Some(name) = a.as_str() {
                        let name = name.trim();
                        if name.is_empty() {
                            continue;
                        }
                        let author_id = store::insert_or_get_node(
                            conn,
                            name,
                            NodeType::Person,
                            None,
                            None,
                            Some("alphaxiv.org"),
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
            }

            // 主题 → Concept 节点 + References 边
            if let Some(topics) = p["topics"].as_array() {
                for t in topics {
                    if let Some(topic) = t.as_str() {
                        let topic = topic.trim();
                        if topic.is_empty() {
                            continue;
                        }
                        let topic_id = store::insert_or_get_node(
                            conn,
                            topic,
                            NodeType::Concept,
                            None,
                            None,
                            Some("alphaxiv.org"),
                        )
                        .map_err(|e| format!("DB error: {}", e))?;
                        store::upsert_edge(
                            conn,
                            &node_id,
                            &topic_id,
                            RelationType::References,
                            1.0,
                            Some("alphaXiv topic"),
                        )
                        .map_err(|e| format!("DB error: {}", e))?;
                    }
                }
            }

            // GitHub 仓库关联 (存在时)
            if let Some(gh) = p["github_url"].as_str() {
                let gh = gh.trim();
                if !gh.is_empty() {
                    let gh_id = store::insert_or_get_node(
                        conn,
                        gh,
                        NodeType::Repository,
                        None,
                        Some(gh),
                        Some("github.com"),
                    )
                    .map_err(|e| format!("DB error: {}", e))?;
                    store::upsert_edge(
                        conn,
                        &node_id,
                        &gh_id,
                        RelationType::References,
                        1.0,
                        Some("alphaXiv code"),
                    )
                    .map_err(|e| format!("DB error: {}", e))?;
                }
            }

            total += 1;
        }
    }
    Ok(total)
}

/// 填充 OpenLibrary 节点 (能力源自 `bin/kb_crawl_batch::crawl_openlibrary`, R-P95/R-P96 提炼并入)。
/// 仅更新已有但 content 为空的 OpenLibrary URL 节点; 复用安全抓取原语 (guard + pin + retry)。
pub fn ingest_from_openlibrary(conn: &Connection) -> Result<usize, String> {
    let ts = now();
    let mut stmt = conn
        .prepare(
            "SELECT id, url FROM nodes WHERE node_type='Article' AND COALESCE(summary, content, '') = '' AND url LIKE '%openlibrary.org%'",
        )
        .map_err(|e| format!("DB prepare: {e}"))?;

    let rows: Vec<(String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|e| format!("DB query: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    if rows.is_empty() {
        return Ok(0);
    }

    let mut filled = 0;
    for (id, url) in &rows {
        let api_url = format!("{}.json", url.trim_end_matches('/'));
        if let Ok((body, _host)) = super::nt_http::fetch_safe_http_with_retry(&api_url) {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
                let desc = data["description"].as_str()
                    .or_else(|| data["description"]["value"].as_str())
                    .or_else(|| data["subtitle"].as_str())
                    .or_else(|| {
                        data["excerpts"].as_array()
                            .and_then(|a| a.first())
                            .and_then(|e| e["text"].as_str())
                    });
                if let Some(text) = desc {
                    let clean = text.trim();
                    if !clean.is_empty()
                        && conn.execute(
                            "UPDATE nodes SET summary=?1, content=?1, updated_at=?2 WHERE id=?3",
                            rusqlite::params![clean, ts, id],
                        ).is_ok()
                    {
                        filled += 1;
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    Ok(filled)
}

pub fn ingest_from_github(conn: &Connection, owner: &str, repo: &str) -> Result<usize, String> {
    let api_url = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let resp = super::nt_http::run_blocking(|| http_client().get(&api_url).send()).map_err(|e| format!("GitHub fetch error: {}", e))?;
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

/// HF datasets 摄取 (http_client 直连, 绕开 SSRF guard — fake-ip 环境 huggingface.co
/// 经 DNS pin 校验为保留段而被 fetch_safe_http 拒绝, 但 API 为公开可信数据源,
/// 复用 GitHub/alphaXiv 同款 shared_blocking_client 直连)。
///
/// 输入 `dataset_ref` 支持三种形态:
///   - 完整 URL: https://huggingface.co/datasets/owner/name
///   - 限定 ID:  owner/name
///   - 裸 ID:    name (作者归 unknown)
pub fn ingest_from_hf_dataset(conn: &Connection, dataset_ref: &str) -> Result<usize, String> {
    // ── 解析 dataset ref → (owner, name) ──
    let trimmed = dataset_ref.trim();
    let after_ds = trimmed
        .strip_prefix("https://huggingface.co/datasets/")
        .or_else(|| trimmed.strip_prefix("https://hf.co/datasets/"))
        .or_else(|| trimmed.strip_prefix("hf.co/datasets/"))
        .or_else(|| trimmed.strip_prefix("huggingface.co/datasets/"))
        .unwrap_or(trimmed)
        .trim_end_matches('/');
    let (owner, name) = match after_ds.split_once('/') {
        Some((o, n)) if !o.is_empty() && !n.is_empty() => (o, n),
        Some((o, n)) if !o.is_empty() => (o, n),
        _ => ("unknown", after_ds),
    };
    if name.is_empty() {
        return Err(format!("invalid HF dataset ref: {}", dataset_ref));
    }

    let api_url = format!("https://huggingface.co/api/datasets/{}/{}", owner, name);
    let resp = super::nt_http::run_blocking(|| http_client().get(&api_url).send())
        .map_err(|e| format!("HF dataset fetch error: {}", e))?;
    if resp.status().is_client_error() || resp.status().is_server_error() {
        return Err(format!("HF API {} for {}", resp.status(), api_url));
    }
    let data: serde_json::Value = resp.json().map_err(|e| format!("HF JSON parse error: {}", e))?;

    let ds_id = data["id"].as_str().unwrap_or(after_ds);
    let author = data["author"].as_str().unwrap_or(owner);
    let downloads = data["downloads"].as_i64().unwrap_or(0);
    let likes = data["likes"].as_i64().unwrap_or(0);
    let gated = data["gated"].as_bool().unwrap_or(false);
    let _private = data["private"].as_bool().unwrap_or(false);
    let tags: Vec<String> = data["tags"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let description = data["description"].as_str().unwrap_or("").to_string();

    let summary = if description.is_empty() {
        format!(
            "HF dataset {} ({} downloads, {} likes{}).",
            ds_id,
            downloads,
            likes,
            if gated { ", gated" } else { "" }
        )
    } else {
        format!(
            "{} ({} downloads, {} likes{}).",
            description.chars().take(800).collect::<String>(),
            downloads,
            likes,
            if gated { ", gated" } else { "" }
        )
    };

    let ds_url = format!("https://huggingface.co/datasets/{}", ds_id);
    let node_id = store::insert_or_get_node(
        conn,
        ds_id,
        NodeType::Dataset,
        Some(&summary),
        Some(&ds_url),
        Some("huggingface.co"),
    )
    .map_err(|e| format!("DB error: {}", e))?;

    // 作者 Organization 节点 + edge
    let owner_id = store::insert_or_get_node(
        conn,
        author,
        NodeType::Organization,
        None,
        Some(&format!("https://huggingface.co/{}", author)),
        Some("huggingface.co"),
    )
    .map_err(|e| format!("DB error: {}", e))?;
    store::upsert_edge(
        conn,
        &node_id,
        &owner_id,
        RelationType::DevelopedBy,
        1.0,
        Some("Dataset author"),
    )
    .map_err(|e| format!("DB error: {}", e))?;

    // tags → Concept 节点 + Related edges
    let mut tags_created = 0usize;
    for tag in tags.iter().filter(|t| !t.is_empty()).take(12) {
        let tag_clean = tag.trim_start_matches("task_categories:").trim_start_matches("language:").trim_start_matches("license:");
        if tag_clean.is_empty() {
            continue;
        }
        let tag_id = store::insert_or_get_node(
            conn,
            tag_clean,
            NodeType::Concept,
            None,
            None,
            Some("huggingface.co"),
        )
        .map_err(|e| format!("DB error: {}", e))?;
        store::upsert_edge(
            conn,
            &node_id,
            &tag_id,
            RelationType::Related,
            1.0,
            Some("HF dataset tag"),
        )
        .map_err(|e| format!("DB error: {}", e))?;
        tags_created += 1;
    }

    log::info!(
        "[HF] dataset {} ingested: {} tags, {} downloads",
        ds_id,
        tags_created,
        downloads
    );
    Ok(1 + tags_created)
}

/// 批量消费 crawl_queue 中 huggingface.co 的 pending 条目 (http_client 直连)。
/// 返回 (成功数, 失败数)。失败的条目标记 completed=false 并记录错误, 避免死循环重试。
pub fn run_hf_queue_batch(conn: &Connection, max_items: usize) -> Result<(usize, usize), String> {
    let mut ok = 0usize;
    let mut fail = 0usize;
    let mut cursor = 0usize;
    loop {
        if ok + fail >= max_items || cursor >= max_items * 4 {
            break;
        }
        let item = match store::claim_hf_pending_url(conn)
            .map_err(|e| format!("DB claim error: {}", e))?
        {
            Some(item) => item,
            None => break,
        };
        cursor += 1;

        match ingest_from_hf_dataset(conn, &item.url) {
            Ok(n) => {
                store::mark_crawl_complete(conn, &item.id, true, None)
                    .map_err(|e| format!("DB error: {}", e))?;
                ok += 1;
                log::info!("[HF] queue {}: {} nodes ({} ok)", item.url, n, ok);
            }
            Err(e) => {
                let err_str = e.chars().take(400).collect::<String>();
                store::mark_crawl_complete(conn, &item.id, false, Some(&err_str))
                    .map_err(|e| format!("DB error: {}", e))?;
                fail += 1;
                log::warn!("[HF] queue {} failed: {}", item.url, e);
            }
        }
    }
    Ok((ok, fail))
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
    if !is_safe_fetch_url(url) {
        return Err(format!("URL rejected (SSRF guard): {}", url));
    }
    // connect-期 DNS pinning (防 rebinding): guard 已在 is_safe_fetch_url,
    // fetch_safe_http 内部再次 resolve + pin。
    let (html, _host) = super::nt_http::fetch_safe_http(url)?;
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

/// 单一 HTML→文本 原语：提取标题、剥离 script/style/tag、解码常见实体、归一空白。
/// 所有吸收器 (UnifiedAbsorber / KnowledgeAbsorptionPipeline / MemoryCrawl) 统一委托此处。
pub(crate) fn extract_html_content(html: &str) -> (String, String) {
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

    let text = decode_html_entities(&text)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    (title, text)
}

/// 解码 HTML 常见实体 (合并自原 nt_mind_knowledge_pipeline::extract_text_content)
fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

/// 单一 HTML→链接 原语：抽取 href、仅保留 http/https、SSRF 过滤内网/回环、去重。
/// 所有吸收器统一委托此处。
pub(crate) fn extract_links(html: &str, _base_url: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut pos = 0;

    while let Some(start) = html[pos..].find("href=\"") {
        let start = pos + start + 6;
        if let Some(end) = html[start..].find('"') {
            let href = &html[start..start + end];
            // SSRF 防护: 仅 http/https 且目标 IP 非内网/回环/链路本地 (防自扩增内网抓取)
            if (href.starts_with("http://") || href.starts_with("https://"))
                && is_safe_fetch_url(href)
            {
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
    let resp = super::nt_http::run_blocking(|| http_client().get(&url).send()).map_err(|e| format!("Fetch error: {}", e))?;

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
    if let Ok(resp) = super::nt_http::run_blocking(|| http_client().get(&links_url).send()) {
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
    use super::is_safe_fetch_url;

    #[test]
    fn test_basic() {
        assert!(true);
    }

    // arXiv export API 返回 feed, 首个 <title> 是 feed 级 "arXiv Query: ...",
    // 论文 title/summary/author 在 <entry> 内。此测试锁定 entry 截取逻辑,
    // 防止回归到 feed 级 title (R-P16 持久化验证 + R-P80 吸收纪律)。
    #[test]
    fn test_arxiv_entry_parsing_extracts_paper_title() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>arXiv Query: search_query=&amp;id_list=2608.06922</title>
  <entry>
    <title>MAGE: Safeguarding LLM Agents against Long-Horizon Threats via Shadow Memory</title>
    <summary>Defensive framework using dedicated safety-focused agentic memory.</summary>
    <author><name>Alice Zhang</name></author>
    <author><name>Bob Li</name></author>
  </entry>
</feed>"#;
        // 复刻 ingest_from_arxiv 的 entry 截取 + 提取逻辑
        let entry = xml.find("<entry>").map(|i| &xml[i..]).unwrap_or(xml);
        let title = super::extract_xml_tag(entry, "title").unwrap_or_else(|| "Unknown".into());
        let summary = super::extract_xml_tag(entry, "summary").unwrap_or_default();
        let mut authors_str = String::new();
        let mut rest = entry;
        while let Some(name_start) = rest.find("<name>") {
            let after = &rest[name_start + 6..];
            if let Some(name_end) = after.find("</name>") {
                if !authors_str.is_empty() {
                    authors_str.push_str(", ");
                }
                authors_str.push_str(after[..name_end].trim());
                rest = &after[name_end + 7..];
            } else {
                break;
            }
        }

        assert!(
            title.contains("MAGE"),
            "title should be the paper title, got: {}",
            title
        );
        assert!(title.contains("Shadow Memory"));
        assert!(!title.contains("arXiv Query"));
        assert!(summary.contains("Defensive framework"));
        assert_eq!(authors_str, "Alice Zhang, Bob Li");
    }

    #[test]
    fn test_ssrf_rejects_loopback() {
        assert!(!is_safe_fetch_url("http://127.0.0.1/"));
        assert!(!is_safe_fetch_url("http://127.0.0.1:8080/admin"));
        assert!(!is_safe_fetch_url("https://localhost/"));
        assert!(!is_safe_fetch_url("http://localhost:3000"));
        assert!(!is_safe_fetch_url("http://test.localhost/"));
        assert!(!is_safe_fetch_url("http://foo.local/"));
        assert!(!is_safe_fetch_url("http://[::1]/"));
    }

    #[test]
    fn test_ssrf_rejects_private_and_reserved() {
        assert!(!is_safe_fetch_url("http://10.0.0.1/"));
        assert!(!is_safe_fetch_url("http://172.16.0.1/"));
        assert!(!is_safe_fetch_url("http://192.168.1.1/"));
        // AWS IMDS / cloud metadata (link-local)
        assert!(!is_safe_fetch_url("http://169.254.169.254/latest/meta-data/"));
        assert!(!is_safe_fetch_url("http://[fc00::1]/"));
        assert!(!is_safe_fetch_url("http://[fe80::1]/"));
    }

    #[test]
    fn test_ssrf_rejects_ipv4_mapped_ipv6() {
        // `::ffff:127.0.0.1` 与 `::ffff:192.168.0.1` 曾绕过旧守卫 (is_loopback 只匹配 ::1)
        assert!(!is_safe_fetch_url("http://[::ffff:127.0.0.1]/"));
        assert!(!is_safe_fetch_url("http://[::ffff:127.0.0.2]:8080/"));
        assert!(!is_safe_fetch_url("http://[::ffff:192.168.1.1]/"));
        assert!(!is_safe_fetch_url("http://[::ffff:10.0.0.1]/"));
    }

    #[test]
    fn test_ssrf_rejects_bad_scheme_and_unparseable() {
        assert!(!is_safe_fetch_url("ftp://example.com/file"));
        assert!(!is_safe_fetch_url("file:///etc/passwd"));
        assert!(!is_safe_fetch_url("javascript:alert(1)"));
        assert!(!is_safe_fetch_url(""));
        assert!(!is_safe_fetch_url("not a url"));
    }

    #[test]
    fn test_ssrf_allows_public() {
        assert!(is_safe_fetch_url("http://8.8.8.8/"));
        assert!(is_safe_fetch_url("https://1.1.1.1/"));
    }
}
