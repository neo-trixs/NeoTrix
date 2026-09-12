use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use regex::Regex;

/// NT-WORLD 域插件 — 网页抓取/搜索/内容提取
pub struct WorldPlugin;

impl DomainPlugin for WorldPlugin {
    fn name(&self) -> &str {
        "world"
    }
    fn description(&self) -> &str {
        "世界感知：网页搜索、内容抓取、知识提取"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "web_search".into(),
                description: "网页搜索 (DDG→Wikipedia fallback)".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "fetch_url".into(),
                description: "抓取网页内容".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "extract_content".into(),
                description: "提取网页结构化内容".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "crawl_status".into(),
                description: "爬虫状态查询".into(),
                params: vec![],
                returns: "Value".into(),
            },
        ]
    }

    fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "web_search" => {
                let query =
                    args.get("query")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "missing 'query'".into(),
                            recoverable: true,
                        })?;
                let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
                web_search(query, count)
            }
            "fetch_url" => {
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'url'".into(),
                        recoverable: true,
                    })?;
                fetch_url(url)
            }
            "extract_content" => {
                let url = args
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "missing 'url'".into(),
                        recoverable: true,
                    })?;
                let format = args
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("text");
                extract_content(url, format)
            }
            "crawl_status" => Ok(serde_json::json!({
                "status": "ready",
                " backends": ["ddg", "wikipedia"],
                "features": ["web_search", "fetch_url", "extract_content"]
            })),
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}

fn web_search(query: &str, count: usize) -> Result<serde_json::Value, DomainError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| DomainError {
            code: "HTTP_ERROR".into(),
            message: e.to_string(),
            recoverable: true,
        })?;

    // DDG instant answer API
    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        urlencoding::encode(query)
    );

    let resp = client.get(&url).send().map_err(|e| DomainError {
        code: "HTTP_ERROR".into(),
        message: e.to_string(),
        recoverable: true,
    })?;

    let json: serde_json::Value = resp.json().map_err(|e| DomainError {
        code: "PARSE_ERROR".into(),
        message: e.to_string(),
        recoverable: true,
    })?;

    let mut results = Vec::new();

    // Abstract
    if let Some(abstract_text) = json.get("Abstract").and_then(|v| v.as_str()) {
        if !abstract_text.is_empty() {
            results.push(serde_json::json!({
                "title": json.get("Heading").and_then(|v| v.as_str()).unwrap_or(query),
                "url": json.get("AbstractURL").and_then(|v| v.as_str()).unwrap_or(""),
                "snippet": abstract_text,
                "source": "ddg_abstract"
            }));
        }
    }

    // Related topics
    if let Some(topics) = json.get("RelatedTopics").and_then(|v| v.as_array()) {
        for topic in topics.iter().take(count.saturating_sub(results.len())) {
            if let Some(text) = topic.get("Text").and_then(|v| v.as_str()) {
                results.push(serde_json::json!({
                    "title": text.split(" - ").next().unwrap_or(text),
                    "url": topic.get("FirstURL").and_then(|v| v.as_str()).unwrap_or(""),
                    "snippet": text,
                    "source": "ddg_related"
                }));
            }
        }
    }

    // Fallback: Wikipedia API
    if results.is_empty() {
        let wiki_url = format!(
            "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
            urlencoding::encode(query)
        );
        if let Ok(wiki_resp) = client.get(&wiki_url).send() {
            if let Ok(wiki_json) = wiki_resp.json::<serde_json::Value>() {
                if let Some(extract) = wiki_json.get("extract").and_then(|v| v.as_str()) {
                    results.push(serde_json::json!({
                        "title": wiki_json.get("title").and_then(|v| v.as_str()).unwrap_or(query),
                        "url": wiki_json.get("content_urls").and_then(|c| c.get("desktop")).and_then(|d| d.get("page")).and_then(|p| p.as_str()).unwrap_or(""),
                        "snippet": extract,
                        "source": "wikipedia"
                    }));
                }
            }
        }
    }

    Ok(serde_json::json!({
        "query": query,
        "count": results.len(),
        "results": results
    }))
}

fn fetch_url(url: &str) -> Result<serde_json::Value, DomainError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (compatible; NeoTrix/1.0)")
        .build()
        .map_err(|e| DomainError {
            code: "HTTP_ERROR".into(),
            message: e.to_string(),
            recoverable: true,
        })?;

    let resp = client.get(url).send().map_err(|e| DomainError {
        code: "HTTP_ERROR".into(),
        message: e.to_string(),
        recoverable: true,
    })?;

    let status = resp.status().as_u16();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let body = resp.text().map_err(|e| DomainError {
        code: "HTTP_ERROR".into(),
        message: e.to_string(),
        recoverable: true,
    })?;

    // 简单 HTML 清理
    let text = if content_type.contains("html") {
        // 移除 script/style 标签
        let re_script = regex::Regex::new(r"(?s)<script[^>]*>.*?</script>").unwrap_or_default();
        let re_style = regex::Regex::new(r"(?s)<style[^>]*>.*?</style>").unwrap_or_default();
        let re_tag = regex::Regex::new(r"<[^>]+>").unwrap_or_default();
        let re_space = regex::Regex::new(r"\s+").unwrap_or_default();

        let cleaned = re_script.replace_all(&body, "");
        let cleaned = re_style.replace_all(&cleaned, "");
        let cleaned = re_tag.replace_all(&cleaned, " ");
        let cleaned = re_space.replace_all(&cleaned, " ");
        cleaned.trim().to_string()
    } else {
        body
    };

    // 截断到合理长度
    let truncated = if text.len() > 10000 {
        format!("{}...", &text[..10000])
    } else {
        text
    };

    Ok(serde_json::json!({
        "url": url,
        "status": status,
        "content_type": content_type,
        "text": truncated,
        "length": truncated.len()
    }))
}

fn extract_content(url: &str, format: &str) -> Result<serde_json::Value, DomainError> {
    let fetched = fetch_url(url)?;
    let text = fetched.get("text").and_then(|v| v.as_str()).unwrap_or("");

    match format {
        "summary" => {
            // 简单摘要：取前500字符
            let summary = if text.len() > 500 {
                format!("{}...", &text[..500])
            } else {
                text.to_string()
            };
            Ok(serde_json::json!({
                "url": url,
                "format": "summary",
                "content": summary
            }))
        }
        "json" => {
            // 尝试提取结构化信息
            Ok(serde_json::json!({
                "url": url,
                "format": "json",
                "title": fetched.get("url").and_then(|v| v.as_str()).unwrap_or(""),
                "content_type": fetched.get("content_type").and_then(|v| v.as_str()).unwrap_or(""),
                "text": text
            }))
        }
        _ => Ok(serde_json::json!({
            "url": url,
            "format": "text",
            "content": text
        })),
    }
}
