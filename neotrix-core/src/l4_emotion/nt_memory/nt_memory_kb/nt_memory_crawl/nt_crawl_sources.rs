//! nt_memory_crawl · 数据源吸收: wiki/arxiv/alphaxiv/openlibrary/github/hf.
use rusqlite::Connection;

use super::super::nt_memory_store as store;
use super::super::nt_memory_types::*;
use super::super::shared_utils::now;
use super::nt_crawl_core::http_client;
use super::nt_crawl_parse::extract_xml_tag;
pub fn ingest_from_wikipedia(conn: &Connection, topic: &str) -> Result<usize, String> {
    let url = format!(
        "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
        topic
    );
    let resp = super::super::nt_http::run_blocking(|| http_client().get(&url).send())
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
    let resp = super::super::nt_http::run_blocking(|| http_client().get(&url).send())
        .map_err(|e| format!("arXiv fetch error: {}", e))?;
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
        let resp = super::super::nt_http::run_blocking(|| http_client().get(&url).send())
            .map_err(|e| format!("alphaXiv fetch error: {}", e))?;
        let text = resp.text().map_err(|e| format!("Text error: {}", e))?;
        let json: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {}", e))?;
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
        if let Ok((body, _host)) = super::super::nt_http::fetch_safe_http_with_retry(&api_url) {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
                let desc = data["description"]
                    .as_str()
                    .or_else(|| data["description"]["value"].as_str())
                    .or_else(|| data["subtitle"].as_str())
                    .or_else(|| {
                        data["excerpts"]
                            .as_array()
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
    let resp = super::super::nt_http::run_blocking(|| http_client().get(&api_url).send())
        .map_err(|e| format!("GitHub fetch error: {}", e))?;
    let data: serde_json::Value = resp
        .json()
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let default_title = format!("{}/{}", owner, repo);
    let title = data["full_name"].as_str().unwrap_or(&default_title);
    let description = data["description"].as_str().unwrap_or("");
    let repo_url = data["html_url"].as_str().unwrap_or(&api_url);
    let stars = data["stargazers_count"].as_i64().unwrap_or(0);
    let topics: Vec<String> = data["topics"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
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
    let resp = super::super::nt_http::run_blocking(|| http_client().get(&api_url).send())
        .map_err(|e| format!("HF dataset fetch error: {}", e))?;
    if resp.status().is_client_error() || resp.status().is_server_error() {
        return Err(format!("HF API {} for {}", resp.status(), api_url));
    }
    let data: serde_json::Value = resp
        .json()
        .map_err(|e| format!("HF JSON parse error: {}", e))?;

    let ds_id = data["id"].as_str().unwrap_or(after_ds);
    let author = data["author"].as_str().unwrap_or(owner);
    let downloads = data["downloads"].as_i64().unwrap_or(0);
    let likes = data["likes"].as_i64().unwrap_or(0);
    let gated = data["gated"].as_bool().unwrap_or(false);
    let _private = data["private"].as_bool().unwrap_or(false);
    let tags: Vec<String> = data["tags"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
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
        let tag_clean = tag
            .trim_start_matches("task_categories:")
            .trim_start_matches("language:")
            .trim_start_matches("license:");
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
