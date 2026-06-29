use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    String,
    Number,
    Bool,
    Array,
    Object,
}

#[derive(Debug, Clone)]
pub struct ExtractionField {
    pub name: String,
    pub description: String,
    pub field_type: FieldType,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct ExtractionSchema {
    pub name: String,
    pub fields: Vec<ExtractionField>,
}

#[derive(Debug, Clone)]
pub struct ExtractedContent {
    pub url: String,
    pub markdown: String,
    pub metadata: HashMap<String, String>,
    pub structured: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ExtractPipeline {
    pub max_html_bytes: usize,
}

impl Default for ExtractPipeline {
    fn default() -> Self {
        Self {
            max_html_bytes: 1_048_576,
        }
    }
}

impl ExtractPipeline {
    pub fn new(max_html_bytes: usize) -> Self {
        Self { max_html_bytes }
    }

    pub fn html_to_markdown(html: &str) -> String {
        let html = if html.len() > 1_000_000 {
            &html[..1_000_000]
        } else {
            html
        };
        let html = decode_html_entities(html);

        let mut md = String::with_capacity(html.len());
        let mut in_tag = false;
        let mut tag_buf = String::new();
        let mut in_style = false;
        let mut in_script = false;
        let mut in_pre = false;
        let mut in_code = false;
        let mut code_accum = String::new();
        let mut link_href: Option<String> = None;
        let mut link_text = String::new();
        let mut is_ordered_list = false;
        let mut need_para_break = false;

        for ch in html.chars() {
            if ch == '<' {
                if in_code && !in_pre {
                    if !code_accum.is_empty() {
                        md.push('`');
                        md.push_str(&code_accum);
                        md.push('`');
                        code_accum.clear();
                    }
                    in_code = false;
                }
                in_tag = true;
                tag_buf.clear();
                continue;
            }

            if in_tag {
                if ch == '>' {
                    in_tag = false;
                    let raw = &tag_buf;
                    let lower = raw.to_lowercase();
                    let tag_start = lower.split_whitespace().next().unwrap_or("");
                    let is_close = tag_start.starts_with('/');
                    let base = if is_close { &tag_start[1..] } else { tag_start };

                    match base {
                        "style" => in_style = !is_close,
                        "script" => in_script = !is_close,
                        "pre" => {
                            in_pre = !is_close;
                            if !is_close {
                                md.push_str("```\n");
                            } else {
                                if md.ends_with('\n') {
                                    md.push_str("```\n");
                                } else {
                                    md.push_str("\n```\n");
                                }
                            }
                        }
                        "code" if !in_pre => {
                            if !is_close {
                                in_code = true;
                                code_accum.clear();
                            } else if !code_accum.is_empty() {
                                md.push('`');
                                md.push_str(&code_accum);
                                md.push('`');
                                code_accum.clear();
                            }
                        }
                        _ => {}
                    }

                    if in_style || in_script {
                        continue;
                    }

                    let is_block = matches!(
                        base,
                        "p" | "div"
                            | "section"
                            | "article"
                            | "blockquote"
                            | "h1"
                            | "h2"
                            | "h3"
                            | "h4"
                            | "h5"
                            | "h6"
                            | "tr"
                            | "th"
                            | "td"
                            | "caption"
                            | "figure"
                            | "figcaption"
                            | "header"
                            | "footer"
                            | "nav"
                            | "main"
                            | "aside"
                            | "form"
                            | "details"
                            | "summary"
                            | "hr"
                            | "br"
                            | "li"
                            | "ul"
                            | "ol"
                            | "dl"
                            | "dt"
                            | "dd"
                            | "table"
                            | "thead"
                            | "tbody"
                            | "tfoot"
                            | "colgroup"
                            | "col"
                    );

                    if is_block && !is_close {
                        ensure_trail_newline(&mut md);
                    }

                    match base {
                        "hr" | "br" => {
                            md.push_str("---\n");
                        }
                        "h1" if !is_close => md.push_str("# "),
                        "h2" if !is_close => md.push_str("## "),
                        "h3" if !is_close => md.push_str("### "),
                        "h4" if !is_close => md.push_str("#### "),
                        "h5" if !is_close => md.push_str("##### "),
                        "h6" if !is_close => md.push_str("###### "),
                        "li" if !is_close => {
                            if is_ordered_list {
                                md.push_str("1. ");
                            } else {
                                md.push_str("- ");
                            }
                        }
                        "ol" => is_ordered_list = !is_close,
                        "blockquote" if !is_close => {
                            need_para_break = true;
                        }
                        "a" if !is_close => {
                            link_href = extract_attr(raw, "href");
                            link_text.clear();
                        }
                        "a" if is_close => {
                            let text = link_text.trim().to_string();
                            if let Some(ref href) = link_href {
                                let href = href.trim();
                                if !text.is_empty() && !href.is_empty() {
                                    md.push('[');
                                    md.push_str(&text);
                                    md.push_str("](");
                                    md.push_str(href);
                                    md.push(')');
                                } else if !href.is_empty() {
                                    md.push_str(href);
                                } else {
                                    md.push_str(&text);
                                }
                            }
                            link_href = None;
                            link_text.clear();
                        }
                        "img" => {
                            let src = extract_attr(raw, "src").unwrap_or_default();
                            let alt = extract_attr(raw, "alt").unwrap_or_default();
                            md.push('!');
                            md.push('[');
                            md.push_str(&alt);
                            md.push_str("](");
                            md.push_str(&src);
                            md.push(')');
                        }
                        "strong" | "b" => md.push_str("**"),
                        "em" | "i" | "cite" | "dfn" => md.push('*'),
                        _ => {}
                    }

                    if matches!(
                        base,
                        "h1" | "h2"
                            | "h3"
                            | "h4"
                            | "h5"
                            | "h6"
                            | "p"
                            | "li"
                            | "div"
                            | "section"
                            | "article"
                            | "figcaption"
                            | "caption"
                    ) && is_close
                    {
                        ensure_trail_newline(&mut md);
                    }

                    continue;
                }
                tag_buf.push(ch);
                continue;
            }

            if in_style || in_script {
                continue;
            }

            if in_code && !in_pre {
                if ch == '<' {
                    code_accum.push(ch);
                } else {
                    code_accum.push(ch);
                }
                continue;
            }

            if link_href.is_some() {
                link_text.push(ch);
                continue;
            }

            if in_pre {
                md.push(ch);
                if ch == '\n' {
                    ensure_trail_newline(&mut md);
                }
                continue;
            }

            if ch.is_control() && ch != '\t' && ch != '\n' {
                continue;
            }

            if ch == ' ' && md.ends_with(' ') {
                continue;
            }

            if need_para_break && ch != '\n' {
                md.push('\n');
                need_para_break = false;
            }

            md.push(ch);
        }

        let re_blank = Regex::new(r"\n{3,}").unwrap();
        let result = re_blank.replace_all(md.trim(), "\n\n").to_string();
        result
    }

    pub fn extract_metadata(html: &str, url: &str) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("url".to_string(), url.to_string());

        let html = if html.len() > 512_000 {
            &html[..512_000]
        } else {
            html
        };

        if let Some(title) = extract_tag_content(html, "title") {
            meta.insert("title".to_string(), title.trim().to_string());
        }

        let mut pos = 0;
        let lower = html.to_lowercase();
        while let Some(start) = lower[pos..].find("<meta") {
            let abs_start = pos + start;
            if let Some(end) = html[abs_start..].find('>') {
                let tag = &html[abs_start..abs_start + end + 1];
                let tag_lower = tag.to_lowercase();
                let name = extract_meta_attr(&tag_lower, tag, "name")
                    .or_else(|| extract_meta_attr(&tag_lower, tag, "property"))
                    .or_else(|| extract_meta_attr(&tag_lower, tag, "http-equiv"))
                    .unwrap_or_default();
                let content = extract_attr(tag, "content").unwrap_or_default();
                if !name.is_empty() && !content.is_empty() {
                    meta.insert(name.to_lowercase(), content);
                }
                pos = abs_start + end + 1;
            } else {
                break;
            }
        }

        if let Some(canonical) = extract_link_rel(html, "canonical") {
            meta.insert("canonical".to_string(), canonical);
        }

        for (from, to) in &[
            ("og:title", "title"),
            ("og:description", "description"),
            ("og:image", "image"),
            ("og:url", "url"),
            ("description", "description"),
            ("keywords", "keywords"),
            ("author", "author"),
        ] {
            if meta.contains_key(*from) && !meta.contains_key(*to) {
                if let Some(val) = meta.get(*from).cloned() {
                    meta.insert(to.to_string(), val);
                }
            }
        }

        meta
    }

    pub fn extract_structured(
        markdown: &str,
        schema: &ExtractionSchema,
    ) -> Result<serde_json::Value, String> {
        let mut map = serde_json::Map::new();
        let md = markdown;

        for field in &schema.fields {
            let value = extract_field_from_markdown(md, field);
            match value {
                Some(v) => {
                    map.insert(field.name.clone(), v);
                }
                None => {
                    if field.required {
                        return Err(format!("Required field '{}' not found", field.name));
                    }
                    map.insert(field.name.clone(), serde_json::Value::Null);
                }
            }
        }

        Ok(serde_json::Value::Object(map))
    }

    pub fn process_url(
        &self,
        html: &str,
        url: &str,
        schema: Option<&ExtractionSchema>,
    ) -> ExtractedContent {
        let html = if html.len() > self.max_html_bytes {
            &html[..self.max_html_bytes]
        } else {
            html
        };

        let markdown = Self::html_to_markdown(html);
        let metadata = Self::extract_metadata(html, url);
        let structured = schema.and_then(|s| Self::extract_structured(&markdown, s).ok());

        ExtractedContent {
            url: url.to_string(),
            markdown,
            metadata,
            structured,
        }
    }

    pub fn guess_content_type(html: &str) -> &'static str {
        let lower = html.to_lowercase();
        let sample = if lower.len() > 100_000 {
            &lower[..100_000]
        } else {
            &lower
        };

        let is_api = sample.contains("swagger")
            || sample.contains("openapi")
            || sample.contains("api-reference")
            || sample.contains("endpoint")
            || sample.contains("rest api")
            || sample.contains("graphql")
            || sample.contains("api doc");
        if is_api {
            return "api_reference";
        }

        let is_forum = sample.contains("forum")
            || sample.contains("thread")
            || sample.contains("topic")
            || sample.contains("discuss")
            || sample.contains("stackoverflow")
            || (sample.contains("answer") && sample.contains("vote"));
        if is_forum {
            return "forum";
        }

        let is_doc = sample.contains("documentation")
            || sample.contains("docs")
            || sample.contains("manual")
            || sample.contains("guide")
            || sample.contains("tutorial")
            || sample.contains("getting started")
            || sample.contains("table of contents")
            || sample.contains("quickstart");
        if is_doc {
            return "documentation";
        }

        let is_blog = sample.contains("blog")
            || sample.contains("article")
            || sample.contains("published")
            || (sample.contains("author") && sample.contains("date"))
            || sample.contains("category")
            || sample.contains("read more");
        if is_blog {
            return "blog";
        }

        let is_repo = sample.contains("repository")
            || sample.contains("readme")
            || sample.contains("license")
            || sample.contains("contributing")
            || sample.contains("clone")
            || sample.contains("commit");
        if is_repo {
            return "repository";
        }

        "generic"
    }
}

fn ensure_trail_newline(s: &mut String) {
    if !s.ends_with('\n') {
        s.push('\n');
    }
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let pattern = format!(r#"{}\s*=\s*"([^"]*?)""#, regex::escape(attr));
    if let Ok(re) = Regex::new(&pattern) {
        if let Some(caps) = re.captures(&lower) {
            let val = caps.get(1).map(|m| m.as_str().to_string())?;
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    let pattern2 = format!(r#"{}\s*=\s*'([^']*?)'"#, regex::escape(attr));
    if let Ok(re) = Regex::new(&pattern2) {
        if let Some(caps) = re.captures(tag) {
            return caps.get(1).map(|m| m.as_str().to_string());
        }
    }
    None
}

fn extract_meta_attr(tag_lower: &str, tag_orig: &str, attr: &str) -> Option<String> {
    let pattern = format!(r#"{}\s*=\s*"([^"]*?)""#, regex::escape(attr));
    if let Ok(re) = Regex::new(&pattern) {
        if let Some(caps) = re.captures(tag_lower) {
            let val = caps.get(1).map(|m| m.as_str().to_string())?;
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    let pattern2 = format!(r#"{}\s*=\s*'([^']*?)'"#, regex::escape(attr));
    if let Ok(re) = Regex::new(&pattern2) {
        if let Some(caps) = re.captures(tag_orig) {
            return caps.get(1).map(|m| m.as_str().to_string());
        }
    }
    None
}

fn extract_tag_content(html: &str, tag: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);

    let start = lower.find(&open)?;
    let tag_end = html[start..].find('>')?;
    let content_start = start + tag_end + 1;
    let close_start = lower[content_start..].find(&close)?;
    Some(html[content_start..content_start + close_start].to_string())
}

fn extract_link_rel(html: &str, rel: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let mut pos = 0;
    while let Some(start) = lower[pos..].find("<link") {
        let abs = pos + start;
        if let Some(end) = html[abs..].find('>') {
            let tag = &html[abs..abs + end + 1];
            let tag_lower = &lower[abs..abs + end + 1];
            if tag_lower.contains(&format!("rel=\"{}\"", rel))
                || tag_lower.contains(&format!("rel='{}'", rel))
            {
                if let Some(href) = extract_attr(tag, "href") {
                    return Some(href);
                }
            }
            pos = abs + end + 1;
        } else {
            break;
        }
    }
    None
}

fn extract_field_from_markdown(md: &str, field: &ExtractionField) -> Option<serde_json::Value> {
    let field_lower = field.name.to_lowercase();
    let desc_lower = field.description.to_lowercase();

    // Try 1: heading match — ## FieldName\n...content...
    let heading_pat = format!(r"(?m)^#{{2,6}}\s+.*?{}\s*$", regex::escape(&field_lower));
    if let Ok(re) = Regex::new(&heading_pat) {
        if let Some(m) = re.find(md) {
            let after = &md[m.end()..];
            let section_end = after.find("\n##").unwrap_or_else(|| after.len().min(500));
            let content = after[..section_end].trim();
            if !content.is_empty() {
                return Some(parse_field_value(content, field));
            }
        }
    }

    // Try 2: key: value pattern — Field Name: value
    let key_pat = format!(
        r"(?m)^\s*[\*]*{}[\*]*\s*[:\-–—]\s*(.+)$",
        regex::escape(&field_lower)
    );
    if let Ok(re) = Regex::new(&key_pat) {
        if let Some(caps) = re.captures(md) {
            let val = caps.get(1).map(|m| m.as_str().trim().to_string())?;
            if !val.is_empty() {
                return Some(parse_field_value(&val, field));
            }
        }
    }

    // Try 3: bold label pattern — **Field Name:** value
    let bold_pat = format!(
        r"(?m)^\s*\*\*.*?{}.*?\*\*\s*[:\-–—]\s*(.+)$",
        regex::escape(&field_lower)
    );
    if let Ok(re) = Regex::new(&bold_pat) {
        if let Some(caps) = re.captures(md) {
            let val = caps.get(1).map(|m| m.as_str().trim().to_string())?;
            if !val.is_empty() {
                return Some(parse_field_value(&val, field));
            }
        }
    }

    // Try 4: description keyword match — look for sentences containing field keywords
    let keywords: Vec<&str> = desc_lower
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .collect();
    if !keywords.is_empty() {
        for line in md.lines() {
            let line_lower = line.to_lowercase();
            let match_count = keywords.iter().filter(|k| line_lower.contains(**k)).count();
            if match_count >= keywords.len().min(3).max(1) && line.len() < 300 {
                let cleaned = line
                    .trim()
                    .trim_start_matches(&['-', '*', '1', '.', ' '][..])
                    .trim()
                    .to_string();
                if !cleaned.is_empty() {
                    return Some(parse_field_value(&cleaned, field));
                }
            }
        }
    }

    None
}

fn parse_field_value(raw: &str, field: &ExtractionField) -> serde_json::Value {
    let trimmed = raw.trim();
    match field.field_type {
        FieldType::Number => {
            let re = Regex::new(r"-?\d+(?:\.\d+)?").unwrap();
            if let Some(m) = re.find(trimmed) {
                let num_str = m.as_str();
                if num_str.contains('.') {
                    num_str
                        .parse::<f64>()
                        .map(serde_json::Value::from)
                        .unwrap_or(serde_json::Value::String(trimmed.to_string()))
                } else {
                    num_str
                        .parse::<i64>()
                        .map(serde_json::Value::from)
                        .unwrap_or(serde_json::Value::String(trimmed.to_string()))
                }
            } else {
                serde_json::Value::String(trimmed.to_string())
            }
        }
        FieldType::Bool => {
            let lower = trimmed.to_lowercase();
            if lower == "true" || lower == "yes" || lower == "1" || lower == "enabled" {
                serde_json::Value::Bool(true)
            } else if lower == "false" || lower == "no" || lower == "0" || lower == "disabled" {
                serde_json::Value::Bool(false)
            } else {
                serde_json::Value::String(trimmed.to_string())
            }
        }
        FieldType::Array => {
            let items: Vec<serde_json::Value> = trimmed
                .split(|c| c == ',' || c == ';' || c == '\n')
                .map(|s| serde_json::Value::String(s.trim().to_string()))
                .filter(|v| {
                    if let serde_json::Value::String(s) = v {
                        !s.is_empty()
                    } else {
                        true
                    }
                })
                .collect();
            if items.is_empty() {
                serde_json::Value::Array(vec![serde_json::Value::String(trimmed.to_string())])
            } else {
                serde_json::Value::Array(items)
            }
        }
        FieldType::Object => {
            let mut obj = serde_json::Map::new();
            for line in trimmed.lines() {
                if let Some((k, v)) = line
                    .split_once(':')
                    .or_else(|| line.split_once("–"))
                    .or_else(|| line.split_once("—"))
                {
                    obj.insert(
                        k.trim().to_string(),
                        serde_json::Value::String(v.trim().to_string()),
                    );
                }
            }
            if obj.is_empty() {
                serde_json::Value::String(trimmed.to_string())
            } else {
                serde_json::Value::Object(obj)
            }
        }
        FieldType::String => serde_json::Value::String(trimmed.to_string()),
    }
}

fn decode_html_entities(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '&' {
            let mut entity = String::new();
            i += 1;
            while i < chars.len() && chars[i] != ';' && entity.len() < 20 {
                entity.push(chars[i]);
                i += 1;
            }
            if i < chars.len() && chars[i] == ';' {
                let decoded = match entity.as_str() {
                    "amp" => "&".to_string(),
                    "lt" => "<".to_string(),
                    "gt" => ">".to_string(),
                    "quot" => "\"".to_string(),
                    "apos" => "'".to_string(),
                    "nbsp" => " ".to_string(),
                    "copy" => "©".to_string(),
                    "reg" => "®".to_string(),
                    "trade" => "™".to_string(),
                    _ => {
                        if entity.starts_with('#') {
                            let num_str = if entity.starts_with("#x") || entity.starts_with("#X") {
                                u32::from_str_radix(&entity[2..], 16).ok()
                            } else {
                                entity[1..].parse::<u32>().ok()
                            };
                            if let Some(codepoint) = num_str {
                                if let Some(c) = char::from_u32(codepoint) {
                                    c.to_string()
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }
                };
                result.push_str(&decoded);
                i += 1;
            } else {
                result.push('&');
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_markdown_headings() {
        let html = "<h1>Title</h1><h2>Section</h2><h3>Subsection</h3>";
        let md = ExtractPipeline::html_to_markdown(html);
        assert!(md.contains("# Title"));
        assert!(md.contains("## Section"));
        assert!(md.contains("### Subsection"));
    }

    #[test]
    fn test_html_to_markdown_paragraphs() {
        let html = "<p>First paragraph.</p><p>Second paragraph.</p>";
        let md = ExtractPipeline::html_to_markdown(html);
        assert!(md.contains("First paragraph."));
        assert!(md.contains("Second paragraph."));
    }

    #[test]
    fn test_html_to_markdown_lists() {
        let html = "<ul><li>Item A</li><li>Item B</li></ul>";
        let md = ExtractPipeline::html_to_markdown(html);
        assert!(md.contains("- Item A"));
        assert!(md.contains("- Item B"));
    }

    #[test]
    fn test_html_to_markdown_links() {
        let html = r#"<a href="https://example.com">click here</a>"#;
        let md = ExtractPipeline::html_to_markdown(html);
        assert!(md.contains("[click here](https://example.com)"));
    }

    #[test]
    fn test_html_to_markdown_images() {
        let html = r#"<img src="pic.png" alt="Photo">"#;
        let md = ExtractPipeline::html_to_markdown(html);
        assert!(md.contains("![Photo](pic.png)"));
    }

    #[test]
    fn test_html_to_markdown_code_block() {
        let html = "<pre><code>fn main() {}\n</code></pre>";
        let md = ExtractPipeline::html_to_markdown(html);
        assert!(md.contains("```"));
    }

    #[test]
    fn test_html_to_markdown_inline_code() {
        let html = "<p>Use <code>let x = 42;</code> in Rust.</p>";
        let md = ExtractPipeline::html_to_markdown(html);
        assert!(md.contains("`let x = 42;`"));
    }

    #[test]
    fn test_html_to_markdown_bold_italic() {
        let html = "<strong>Bold</strong> and <em>italic</em>";
        let md = ExtractPipeline::html_to_markdown(html);
        assert!(md.contains("**Bold**"));
        assert!(md.contains("*italic*"));
    }

    #[test]
    fn test_html_to_markdown_strip_scripts() {
        let html = "<p>Hello</p><script>alert('xss')</script><p>World</p>";
        let md = ExtractPipeline::html_to_markdown(html);
        assert!(md.contains("Hello"));
        assert!(md.contains("World"));
        assert!(!md.contains("alert"));
    }

    #[test]
    fn test_html_to_markdown_entities() {
        let html = "<p>AT&amp;T &lt;google&gt; &amp; more</p>";
        let md = ExtractPipeline::html_to_markdown(html);
        assert!(md.contains("AT&T"));
        assert!(md.contains("<google>"));
        assert!(md.contains("& more"));
    }

    #[test]
    fn test_extract_metadata_title() {
        let html = "<html><head><title>Test Page</title></head></html>";
        let meta = ExtractPipeline::extract_metadata(html, "https://example.com");
        assert_eq!(meta.get("title").map(|s| s.as_str()), Some("Test Page"));
    }

    #[test]
    fn test_extract_metadata_meta_tags() {
        let html = r#"
<html><head>
<meta name="description" content="A test page">
<meta name="keywords" content="rust, testing">
<meta property="og:image" content="https://example.com/img.png">
</head></html>"#;
        let meta = ExtractPipeline::extract_metadata(html, "https://example.com");
        assert_eq!(
            meta.get("description").map(|s| s.as_str()),
            Some("A test page")
        );
        assert_eq!(
            meta.get("keywords").map(|s| s.as_str()),
            Some("rust, testing")
        );
        assert_eq!(
            meta.get("og:image").map(|s| s.as_str()),
            Some("https://example.com/img.png")
        );
    }

    #[test]
    fn test_extract_metadata_url() {
        let meta = ExtractPipeline::extract_metadata("<html></html>", "https://example.com/page");
        assert_eq!(
            meta.get("url").map(|s| s.as_str()),
            Some("https://example.com/page")
        );
    }

    #[test]
    fn test_extract_structured_simple() {
        let md = "# Article\n\nTitle: My Article\nAuthor: John\nScore: 42";
        let schema = ExtractionSchema {
            name: "test".into(),
            fields: vec![
                ExtractionField {
                    name: "title".into(),
                    description: "Article title".into(),
                    field_type: FieldType::String,
                    required: true,
                },
                ExtractionField {
                    name: "author".into(),
                    description: "Writer name".into(),
                    field_type: FieldType::String,
                    required: true,
                },
                ExtractionField {
                    name: "score".into(),
                    description: "Score number".into(),
                    field_type: FieldType::Number,
                    required: false,
                },
            ],
        };
        let result = ExtractPipeline::extract_structured(md, &schema).unwrap();
        assert_eq!(
            result["title"],
            serde_json::Value::String("My Article".into())
        );
        assert_eq!(result["author"], serde_json::Value::String("John".into()));
        assert_eq!(
            result["score"],
            serde_json::Value::Number(serde_json::Number::from(42))
        );
    }

    #[test]
    fn test_extract_structured_required_missing() {
        let md = "Some random content without fields";
        let schema = ExtractionSchema {
            name: "test".into(),
            fields: vec![ExtractionField {
                name: "required_field".into(),
                description: "Must exist".into(),
                field_type: FieldType::String,
                required: true,
            }],
        };
        let result = ExtractPipeline::extract_structured(md, &schema);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("required_field"));
    }

    #[test]
    fn test_extract_structured_optional_missing() {
        let md = "Only title: hello";
        let schema = ExtractionSchema {
            name: "test".into(),
            fields: vec![
                ExtractionField {
                    name: "title".into(),
                    description: "Title".into(),
                    field_type: FieldType::String,
                    required: false,
                },
                ExtractionField {
                    name: "optional".into(),
                    description: "Optional field".into(),
                    field_type: FieldType::String,
                    required: false,
                },
            ],
        };
        let result = ExtractPipeline::extract_structured(md, &schema).unwrap();
        assert_eq!(result["title"], serde_json::Value::String("hello".into()));
        assert_eq!(result["optional"], serde_json::Value::Null);
    }

    #[test]
    fn test_extract_structured_bool() {
        let md = "Enabled: yes\nActive: true";
        let schema = ExtractionSchema {
            name: "test".into(),
            fields: vec![
                ExtractionField {
                    name: "enabled".into(),
                    description: "Is enabled".into(),
                    field_type: FieldType::Bool,
                    required: false,
                },
                ExtractionField {
                    name: "active".into(),
                    description: "Is active".into(),
                    field_type: FieldType::Bool,
                    required: false,
                },
            ],
        };
        let result = ExtractPipeline::extract_structured(md, &schema).unwrap();
        assert_eq!(result["enabled"], serde_json::Value::Bool(true));
        assert_eq!(result["active"], serde_json::Value::Bool(true));
    }

    #[test]
    fn test_extract_structured_array() {
        let md = "Tags: rust, web, testing";
        let schema = ExtractionSchema {
            name: "test".into(),
            fields: vec![ExtractionField {
                name: "tags".into(),
                description: "Tags list".into(),
                field_type: FieldType::Array,
                required: false,
            }],
        };
        let result = ExtractPipeline::extract_structured(md, &schema).unwrap();
        let arr = result["tags"].as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], "rust");
        assert_eq!(arr[1], "web");
        assert_eq!(arr[2], "testing");
    }

    #[test]
    fn test_process_url_full_pipeline() {
        let html = r#"<html><head><title>Test</title>
<meta name="description" content="A test pipeline">
</head><body><h1>Hello</h1><p>World</p></body></html>"#;
        let pipeline = ExtractPipeline::default();
        let schema = ExtractionSchema {
            name: "test".into(),
            fields: vec![ExtractionField {
                name: "hello".into(),
                description: "Greeting".into(),
                field_type: FieldType::String,
                required: false,
            }],
        };
        let content = pipeline.process_url(html, "https://example.com", Some(&schema));
        assert_eq!(content.url, "https://example.com");
        assert_eq!(
            content.metadata.get("title").map(|s| s.as_str()),
            Some("Test")
        );
        assert_eq!(
            content.metadata.get("description").map(|s| s.as_str()),
            Some("A test pipeline")
        );
        assert!(content.markdown.contains("# Hello"));
        assert!(content.markdown.contains("World"));
        assert!(content.structured.is_some());
    }

    #[test]
    fn test_guess_content_type_blog() {
        let html =
            "<html><article><h1>Blog Post</h1><p>Published on Jan 1 by Author</p></article></html>";
        let ctype = ExtractPipeline::guess_content_type(html);
        assert_eq!(ctype, "blog");
    }

    #[test]
    fn test_guess_content_type_documentation() {
        let html = "<html><h1>Documentation</h1><nav>Table of Contents</nav><p>Getting started guide</p></html>";
        let ctype = ExtractPipeline::guess_content_type(html);
        assert_eq!(ctype, "documentation");
    }

    #[test]
    fn test_guess_content_type_api() {
        let html =
            "<html><h1>API Reference</h1><p>REST API endpoints</p><code>GET /users</code></html>";
        let ctype = ExtractPipeline::guess_content_type(html);
        assert_eq!(ctype, "api_reference");
    }

    #[test]
    fn test_guess_content_type_forum() {
        let html = "<html><div class=\"thread\"><h2>Question</h2><div class=\"answer\">Answer</div><span class=\"vote\">5</span></div></html>";
        let ctype = ExtractPipeline::guess_content_type(html);
        assert_eq!(ctype, "forum");
    }

    #[test]
    fn test_guess_content_type_generic() {
        let html = "<html><p>Just some content</p></html>";
        let ctype = ExtractPipeline::guess_content_type(html);
        assert_eq!(ctype, "generic");
    }

    #[test]
    fn test_html_to_markdown_ordered_list() {
        let html = "<ol><li>First</li><li>Second</li><li>Third</li></ol>";
        let md = ExtractPipeline::html_to_markdown(html);
        assert!(md.contains("1. First"));
        assert!(md.contains("1. Second"));
        assert!(md.contains("1. Third"));
    }

    #[test]
    fn test_html_to_markdown_blockquote() {
        let html = "<blockquote><p>A citation</p></blockquote>";
        let md = ExtractPipeline::html_to_markdown(html);
        assert!(md.contains("A citation"));
    }

    #[test]
    fn test_html_to_markdown_horizontal_rule() {
        let html = "<hr>";
        let md = ExtractPipeline::html_to_markdown(html);
        assert!(md.contains("---"));
    }

    #[test]
    fn test_html_to_markdown_nested_structure() {
        let html =
            "<div><h1>Title</h1><p>Text with <strong>bold</strong> and <em>italic</em>.</p></div>";
        let md = ExtractPipeline::html_to_markdown(html);
        assert!(md.contains("# Title"));
        assert!(md.contains("**bold**"));
        assert!(md.contains("*italic*"));
    }

    #[test]
    fn test_extract_metadata_og_tags() {
        let html = r#"<html><head>
<meta property="og:title" content="OG Title">
<meta property="og:description" content="OG Desc">
</head></html>"#;
        let meta = ExtractPipeline::extract_metadata(html, "https://example.com");
        assert_eq!(meta.get("og:title").map(|s| s.as_str()), Some("OG Title"));
        assert_eq!(
            meta.get("og:description").map(|s| s.as_str()),
            Some("OG Desc")
        );
    }

    #[test]
    fn test_extract_structured_heading_section() {
        let md =
            "## Description\n\nThis is a long description of the item.\n\n## Other\n\nStuff here.";
        let schema = ExtractionSchema {
            name: "test".into(),
            fields: vec![ExtractionField {
                name: "description".into(),
                description: "Item description".into(),
                field_type: FieldType::String,
                required: true,
            }],
        };
        let result = ExtractPipeline::extract_structured(md, &schema).unwrap();
        assert_eq!(
            result["description"],
            "This is a long description of the item."
        );
    }

    #[test]
    fn test_decode_entities() {
        assert_eq!(decode_html_entities("AT&amp;T"), "AT&T");
        assert_eq!(decode_html_entities("&lt;tag&gt;"), "<tag>");
        assert_eq!(decode_html_entities("&quot;quote&quot;"), "\"quote\"");
        assert_eq!(decode_html_entities("&nbsp;"), " ");
        assert_eq!(decode_html_entities("&#65;"), "A");
        assert_eq!(decode_html_entities("&#x41;"), "A");
    }

    #[test]
    fn test_extract_pipeline_default() {
        let p = ExtractPipeline::default();
        assert_eq!(p.max_html_bytes, 1_048_576);
    }

    #[test]
    fn test_extract_pipeline_custom_max_bytes() {
        let p = ExtractPipeline::new(4096);
        assert_eq!(p.max_html_bytes, 4096);
    }

    #[test]
    fn test_guess_content_type_repository() {
        let html = "<html><h1>README</h1><p>License MIT</p><p>Clone the repository</p></html>";
        let ctype = ExtractPipeline::guess_content_type(html);
        assert_eq!(ctype, "repository");
    }
}
