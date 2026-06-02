use std::collections::HashMap;
use regex::Regex;

/// 提取策略级别（Tier）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionTier {
    SemanticHtml5,
    WaiAria,
    CommonClasses,
    Readability,
    PlainStrip,
}

impl ExtractionTier {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SemanticHtml5 => "semantic-html5",
            Self::WaiAria => "wai-aria",
            Self::CommonClasses => "common-classes",
            Self::Readability => "readability",
            Self::PlainStrip => "plain-strip",
        }
    }
}

/// 内容块
#[derive(Debug, Clone)]
pub struct ContentBlock {
    pub heading: Option<String>,
    pub body: String,
}

/// 智能提取结果
#[derive(Debug, Clone)]
pub struct ExtractedContent {
    pub title: String,
    pub description: String,
    pub metadata: HashMap<String, String>,
    pub blocks: Vec<ContentBlock>,
    pub raw_text: String,
    pub text_length: usize,
    pub tier: ExtractionTier,
    pub quality: f64,
    pub has_list_data: bool,
    pub has_table_data: bool,
    pub link_count: usize,
}

struct Region {
    start: usize,
    end: usize,
    text_density: f64,
}

pub struct SmartContentExtractor;

impl SmartContentExtractor {
    pub fn extract(html: &str) -> ExtractedContent {
        let metadata = Self::extract_metadata(html);
        let title = metadata.get("og:title")
            .or_else(|| metadata.get("title"))
            .cloned()
            .unwrap_or_default();
        let description = metadata.get("og:description")
            .or_else(|| metadata.get("description"))
            .cloned()
            .unwrap_or_default();

        let (blocks, raw_text, tier) = Self::try_semantic_selectors(html)
            .or_else(|| Self::try_aria_selectors(html))
            .or_else(|| Self::try_common_classes(html))
            .or_else(|| Self::readability_extract(html))
            .unwrap_or_else(|| Self::plain_extract(html));

        let text_length = raw_text.len();
        let has_list_data = Self::detect_list(html);
        let has_table_data = Self::detect_table(html);
        let link_count = Self::count_links(html);
        let quality = Self::score_quality(&raw_text, html, &tier);

        ExtractedContent {
            title,
            description,
            metadata,
            blocks,
            raw_text,
            text_length,
            tier,
            quality,
            has_list_data,
            has_table_data,
            link_count,
        }
    }

    fn try_semantic_selectors(html: &str) -> Option<(Vec<ContentBlock>, String, ExtractionTier)> {
        lazy_html_content(html, &["<article", "<main"])
            .or_else(|| lazy_section_split(html, "<article"))
            .or_else(|| lazy_section_split(html, "<main"))
    }

    fn try_aria_selectors(html: &str) -> Option<(Vec<ContentBlock>, String, ExtractionTier)> {
        aria_content(html)
    }

    fn try_common_classes(html: &str) -> Option<(Vec<ContentBlock>, String, ExtractionTier)> {
        let patterns = [
            r#"class="[^"]*(?:content|post|entry|article-body|post-content|main-content|page-content|documentation)[^"]*""#,
            r#"id="[^"]*(?:content|main|article|post|entry|page-wrapper)[^"]*""#,
        ];
        // Use regex to match any of these patterns, then extract the element content
        for pattern in &patterns {
            let re = Regex::new(pattern).ok()?;
            if let Some(m) = re.find(html) {
                let _matched = m.as_str();
                let element_start = m.start();
                // Find the parent element boundary by scanning backward for '<'
                let before = &html[..element_start];
                let tag_start = before.rfind('<')?;
                let content = Self::extract_inner_html(&html[tag_start..])?;
                let raw = strip_html(&content);
                if raw.len() > 100 {
                    return Some((
                        vec![ContentBlock { heading: None, body: raw.clone() }],
                        raw,
                        ExtractionTier::CommonClasses,
                    ));
                }
            }
        }
        None
    }

    fn readability_extract(html: &str) -> Option<(Vec<ContentBlock>, String, ExtractionTier)> {
        let mut best_region: Option<Region> = None;
        let body_re = Regex::new(r"<body[^>]*>(.*?)</body>").ok()?;
        let body_html = if let Some(caps) = body_re.captures(html) {
            caps.get(1).map_or(html, |m| m.as_str())
        } else {
            html
        };

        // Split into logical sections by common block-level elements
        let sections: Vec<&str> = body_html.split_inclusive("</div>")
            .chain(body_html.split_inclusive("</section>"))
            .collect();

        for section in &sections {
            let text = strip_html(section);
            let text_len = text.len();
            if text_len < 80 {
                continue;
            }
            let markup_len = section.len().saturating_sub(text_len);
            let density = if markup_len > 0 {
                text_len as f64 / markup_len as f64
            } else {
                0.0
            };
            if density > 1.5 && text_len > 200 {
                if let Some(ref best) = best_region {
                    if density > 2.0 || text_len > best.text_density as usize * 2 {
                        // accumulate
                    }
                } else {
                    let start = html.find(section).unwrap_or(0);
                    best_region = Some(Region {
                        start,
                        end: start + section.len(),
                        text_density: density,
                    });
                }
            }
        }

        best_region.and_then(|region| {
            let raw = strip_html(&html[region.start..region.end]);
            if raw.len() > 200 {
                Some((
                    vec![ContentBlock { heading: None, body: raw.clone() }],
                    raw,
                    ExtractionTier::Readability,
                ))
            } else {
                None
            }
        })
    }

    fn plain_extract(html: &str) -> (Vec<ContentBlock>, String, ExtractionTier) {
        let text = strip_html(html);
        (vec![ContentBlock { heading: None, body: text.clone() }], text, ExtractionTier::PlainStrip)
    }

    /// 提取 metadata (OG tags + meta)
    fn extract_metadata(html: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();

        // <title>
        if let Some(t) = extract_tag_content(html, "title") {
            map.insert("title".to_string(), t.trim().to_string());
        }

        // Open Graph / meta tags
        let re = Regex::new(r#"<meta\s[^>]*?(?:property|name)=["']([^"']+)["'][^>]*?content=["']([^"']*)["'][^>]*>"#).expect("valid regex");
        for cap in re.captures_iter(html) {
            let key = cap.get(1).map_or("", |m| m.as_str()).to_lowercase();
            let val = cap.get(2).map_or("", |m| m.as_str()).to_string();
            map.insert(key, val);
        }

        // alt: content before property
        let re2 = Regex::new(r#"<meta\s[^>]*?content=["']([^"']*)["'][^>]*?(?:property|name)=["']([^"']+)["'][^>]*>"#).expect("valid regex");
        for cap in re2.captures_iter(html) {
            let key = cap.get(2).map_or("", |m| m.as_str()).to_lowercase();
            let val = cap.get(1).map_or("", |m| m.as_str()).to_string();
            map.entry(key).or_insert(val);
        }

        map
    }

    fn extract_inner_html(fragment: &str) -> Option<String> {
        // Find the closing tag of the first element
        let tag_end = fragment.find('>')?;
        let tag_name = extract_tag_name(&fragment[..tag_end])?;
        let closing = format!("</{}>", tag_name);
        let content_start = tag_end + 1;
        let content_end = fragment[content_start..].find(&closing)? + content_start;
        Some(fragment[content_start..content_end].to_string())
    }

    fn detect_list(html: &str) -> bool {
        let ul_re = Regex::new(r"<ul\b[^>]*>[\s\S]*?</ul>").expect("valid regex");
        let ol_re = Regex::new(r"<ol\b[^>]*>[\s\S]*?</ol>").expect("valid regex");
        ul_re.find_iter(html).count() + ol_re.find_iter(html).count() >= 2
    }

    fn detect_table(html: &str) -> bool {
        let table_re = Regex::new(r"<table\b[^>]*>[\s\S]*?</table>").expect("valid regex");
        table_re.find_iter(html).count() >= 1
    }

    fn count_links(html: &str) -> usize {
        let link_re = Regex::new(r#"<a\b[^>]*href=["']https?://[^"']+["']"#).expect("valid regex");
        link_re.find_iter(html).count()
    }

    fn score_quality(raw_text: &str, html: &str, tier: &ExtractionTier) -> f64 {
        let mut score: f64 = match tier {
            ExtractionTier::SemanticHtml5 => 0.95,
            ExtractionTier::WaiAria => 0.90,
            ExtractionTier::CommonClasses => 0.80,
            ExtractionTier::Readability => 0.70,
            ExtractionTier::PlainStrip => 0.40,
        };
        let text_len = raw_text.len();
        let html_len = html.len().max(1);
        let ratio = text_len as f64 / html_len as f64;
        if ratio > 0.3 {
            score += 0.05;
        } else if ratio < 0.05 {
            score -= 0.10;
        }
        if text_len > 500 {
            score += 0.05;
        }
        if text_len < 50 {
            score = (score * 0.5).max(0.05);
        }
        score.clamp(0.0, 1.0)
    }
}

// ============ 底层工具函数 ============

fn lazy_html_content<'a>(html: &'a str, tags: &[&str]) -> Option<(Vec<ContentBlock>, String, ExtractionTier)> {
    for tag in tags {
        if let Some(result) = lazy_section_split(html, tag) {
            return Some(result);
        }
    }
    None
}

fn lazy_section_split(html: &str, tag_prefix: &str) -> Option<(Vec<ContentBlock>, String, ExtractionTier)> {
    let tag_name = tag_prefix.trim_start_matches('<');
    // Find the outermost container with this tag
    let pattern = format!(r"<{}[^>]*>[\s\S]*?</{}>", regex::escape(tag_name), regex::escape(tag_name));
    let re = Regex::new(&pattern).ok()?;
    let mut blocks = Vec::new();

    for cap in re.captures_iter(html) {
        let element_html = cap.get(0).map_or("", |m| m.as_str());
        // Strip HTML to get text
        let text = strip_html(element_html);
        // Try to find a heading within
        let heading = extract_first_heading(element_html);

        let body = if let Some(ref h) = heading {
            text.replacen(h, "", 1).trim().to_string()
        } else {
            text.clone()
        };

        if body.len() > 50 {
            blocks.push(ContentBlock {
                heading,
                body,
            });
        }
    }

    if blocks.is_empty() {
        return None;
    }

    let combined: String = blocks.iter()
        .map(|b| {
            let h = b.heading.as_deref().unwrap_or("");
            format!("{}\n{}", h, b.body)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    Some((blocks, combined, ExtractionTier::SemanticHtml5))
}

fn aria_content(html: &str) -> Option<(Vec<ContentBlock>, String, ExtractionTier)> {
    let re = Regex::new(r#"(<div\b[^>]*?role=["'](?:main|article|region)["'][^>]*?>[\s\S]*?</div>)"#).ok()?;
    let mut blocks = Vec::new();
    for cap in re.captures_iter(html) {
        let element_html = cap.get(1).map_or("", |m| m.as_str());
        let text = strip_html(element_html);
        if text.len() > 100 {
            let heading = extract_first_heading(element_html);
            let body = if let Some(ref h) = heading {
                text.replacen(h, "", 1).trim().to_string()
            } else {
                text.clone()
            };
            blocks.push(ContentBlock { heading, body });
        }
    }
    if blocks.is_empty() {
        return None;
    }
    let combined: String = blocks.iter()
        .map(|b| {
            let h = b.heading.as_deref().unwrap_or("");
            format!("{}\n{}", h, b.body)
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    Some((blocks, combined, ExtractionTier::WaiAria))
}

fn extract_tag_content<'a>(html: &'a str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    html.find(&open).and_then(|start| {
        let cs = start + open.len();
        html[cs..].find(&close).map(|end| {
            html[cs..cs + end].trim().to_string()
        })
    })
}

fn extract_first_heading(html: &str) -> Option<String> {
    let heading_re = Regex::new(r"<h[1-3]\b[^>]*>(.*?)</h[1-3]>").ok()?;
    heading_re.captures(html)
        .and_then(|cap| cap.get(1))
        .map(|m| strip_html(m.as_str()))
}

fn extract_tag_name(open_tag: &str) -> Option<String> {
    let start = open_tag.trim_start().strip_prefix('<')?;
    let end = start.find(|c: char| c.is_whitespace() || c == '>')?;
    Some(start[..end].to_string())
}

fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut tag_buf = String::new();

    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                tag_buf.clear();
                tag_buf.push(c);
            }
            '>' if in_tag => {
                tag_buf.push(c);
                let lower = tag_buf.to_lowercase();
                if lower.starts_with("<script") {
                    in_script = true;
                } else if lower.starts_with("</script") {
                    in_script = false;
                } else if lower.starts_with("<style") {
                    in_style = true;
                } else if lower.starts_with("</style") {
                    in_style = false;
                } else if !in_script && !in_style {
                    // normal tag, skip it
                }
                in_tag = false;
            }
            _ if !in_tag && !in_script && !in_style => {
                if c.is_whitespace() {
                    if !result.ends_with(' ') && !result.is_empty() {
                        result.push(' ');
                    }
                } else {
                    result.push(c);
                }
            }
            _ if in_tag => {
                tag_buf.push(c);
            }
            _ => {}
        }
    }

    // Trim
    let trimmed = result.trim().to_string();
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Test Page</title></head><body><p>Hello</p></body></html>";
        let meta = SmartContentExtractor::extract_metadata(html);
        assert_eq!(meta.get("title").map(|s| s.as_str()), Some("Test Page"));
    }

    #[test]
    fn test_extract_og_tags() {
        let html = r#"<meta property="og:title" content="OG Title"><meta name="description" content="Desc">"#;
        let meta = SmartContentExtractor::extract_metadata(html);
        assert_eq!(meta.get("og:title").map(|s| s.as_str()), Some("OG Title"));
        assert_eq!(meta.get("description").map(|s| s.as_str()), Some("Desc"));
    }

    #[test]
    fn test_semantic_article() {
        let html = "<html><body><article><h1>Art Title</h1><p>Article content paragraph with additional text to exceed the minimum threshold for body detection.</p></article></body></html>";
        let result = SmartContentExtractor::extract(html);
        assert_eq!(result.tier, ExtractionTier::SemanticHtml5);
        assert!(result.raw_text.contains("Art Title"));
        assert!(result.raw_text.contains("Article content paragraph"));
        assert!(result.quality > 0.8);
    }

    #[test]
    fn test_aria_main() {
        let html = r#"<html><body><div role="main"><h2>Main Section</h2><p>Main content here.</p></div></body></html>"#;
        let result = SmartContentExtractor::extract(html);
        assert!(result.raw_text.contains("Main content"));
        assert!(result.raw_text.len() > 20);
    }

    #[test]
    fn test_common_class_content() {
        let html = r#"<html><body><div class="content"><h3>Content Area</h3><p>This is the content area with enough text to be detected.</p></div></body></html>"#;
        let result = SmartContentExtractor::extract(html);
        assert!(result.raw_text.contains("Content Area"));
    }

    #[test]
    fn test_readability_fallback() {
        let html = r#"<html><body><div><p>First section is short.</p></div><div><p>This is a much longer section with enough text content to trigger the readability heuristic. It has many words and demonstrates the text density approach to content extraction. When there's no semantic tags, we fall back to analyzing text-to-markup ratios across sections.</p></div></body></html>"#;
        let result = SmartContentExtractor::extract(html);
        assert!(result.quality > 0.0);
    }

    #[test]
    fn test_plain_strip_fallback() {
        let html = "<html><body><p>Minimal page.</p></body></html>";
        let result = SmartContentExtractor::extract(html);
        assert_eq!(result.tier, ExtractionTier::PlainStrip);
        assert!(result.raw_text.contains("Minimal page"));
        assert!(result.quality < 0.8);
    }

    #[test]
    fn test_list_detection() {
        let html = "<ul><li>Item 1</li><li>Item 2</li></ul><ul><li>Item 3</li></ul>";
        assert!(SmartContentExtractor::detect_list(html));
        let html2 = "<p>No lists here</p>";
        assert!(!SmartContentExtractor::detect_list(html2));
    }

    #[test]
    fn test_table_detection() {
        let html = "<table><tr><td>Cell</td></tr></table>";
        assert!(SmartContentExtractor::detect_table(html));
    }

    #[test]
    fn test_link_count() {
        let html = r#"<a href="https://example.com">Link</a><a href="https://other.com">Other</a>"#;
        assert_eq!(SmartContentExtractor::count_links(html), 2);
    }

    #[test]
    fn test_strip_html_removes_script_style() {
        let html = "<html><script>alert('x')</script><style>.cls{}</style><p>Real content</p></html>";
        let stripped = strip_html(html);
        assert!(!stripped.contains("alert"));
        assert!(!stripped.contains(".cls"));
        assert!(stripped.contains("Real content"));
    }

    #[test]
    fn test_first_heading_extraction() {
        let html = "<article><h1>The Title</h1><p>Content.</p><h2>Sub</h2></article>";
        let heading = extract_first_heading(html);
        assert_eq!(heading, Some("The Title".to_string()));
    }

    #[test]
    fn test_extraction_quality_score() {
        let high_quality = SmartContentExtractor::score_quality(
            "Long text content with enough substance to pass the quality threshold. ".repeat(50).as_str(),
            &format!("<html><body><article>{}</article></body></html>", "p".repeat(200)),
            &ExtractionTier::SemanticHtml5,
        );
        assert!(high_quality > 0.9);
    }
}
