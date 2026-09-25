//! nt_memory_crawl · 解析原语: html/链接/xml/domain.
use super::nt_crawl_core::is_safe_fetch_url;
/// 单一 HTML→文本 原语：提取标题、剥离 script/style/tag、解码常见实体、归一空白。
/// 所有吸收器 (UnifiedAbsorber / KnowledgeAbsorptionPipeline / MemoryCrawl) 统一委托此处。
pub fn extract_html_content(html: &str) -> (String, String) {
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
pub fn extract_links(html: &str, _base_url: &str) -> Vec<String> {
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

pub(crate) fn extract_domain(url: &str) -> String {
    url.split('/')
        .nth(2)
        .unwrap_or("")
        .trim_start_matches("www.")
        .to_string()
}

pub(crate) fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
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
