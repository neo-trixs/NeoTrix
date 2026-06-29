use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    Html,
    Text,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ConvertedDoc {
    pub title: String,
    pub content: String,
    pub word_count: usize,
    pub format: SourceFormat,
    pub vsa_fingerprint: [u64; 4],
}

pub struct DocConverter;

impl DocConverter {
    pub fn html_to_markdown(html: &str) -> ConvertedDoc {
        let title = Self::extract_title(html);
        let format = SourceFormat::Html;
        let mut md = html.to_string();

        // Block: blockquote
        let re_blockquote = Regex::new(r"(?s)<blockquote[^>]*>(.*?)</blockquote>").unwrap();
        md = re_blockquote
            .replace_all(&md, |caps: &regex::Captures| {
                let inner = Self::strip_html(caps.get(1).unwrap().as_str());
                let lines: Vec<String> = inner
                    .lines()
                    .map(|l| format!("> {}", l.trim()))
                    .collect();
                format!("{}\n\n", lines.join("\n"))
            })
            .to_string();

        // Block: pre/code blocks (must handle before inline code)
        let re_pre = Regex::new(r"(?s)<pre[^>]*><code[^>]*>(.*?)</code></pre>").unwrap();
        md = re_pre
            .replace_all(&md, |caps: &regex::Captures| {
                format!("```\n{}\n```\n\n", caps.get(1).unwrap().as_str())
            })
            .to_string();

        // Block: unordered list
        let re_ul = Regex::new(r"(?s)<ul[^>]*>(.*?)</ul>").unwrap();
        md = re_ul
            .replace_all(&md, |caps: &regex::Captures| {
                Self::process_list_items(caps.get(1).unwrap().as_str(), "- ")
            })
            .to_string();

        // Block: ordered list
        let re_ol = Regex::new(r"(?s)<ol[^>]*>(.*?)</ol>").unwrap();
        md = re_ol
            .replace_all(&md, |caps: &regex::Captures| {
                Self::process_list_items(caps.get(1).unwrap().as_str(), "1. ")
            })
            .to_string();

        // Block: headings h1-h6
        for level in 1..=6 {
            let pattern = format!("(?s)<h{level}[^>]*>(.*?)</h{level}>");
            let re_h = Regex::new(&pattern).unwrap();
            md = re_h
                .replace_all(&md, |caps: &regex::Captures| {
                    let inner = Self::strip_html(caps.get(1).unwrap().as_str());
                    format!("{} {}\n\n", "#".repeat(level), inner.trim())
                })
                .to_string();
        }

        // Block: horizontal rule
        let re_hr = Regex::new(r"<hr[^>]*/?>").unwrap();
        md = re_hr.replace_all(&md, "\n---\n\n").to_string();

        // Block: paragraphs
        let re_p = Regex::new(r"(?s)<p[^>]*>(.*?)</p>").unwrap();
        md = re_p
            .replace_all(&md, |caps: &regex::Captures| {
                let inner = caps.get(1).unwrap().as_str().trim();
                format!("{}\n\n", inner)
            })
            .to_string();

        // Inline: anchor links
        let re_a =
            Regex::new(r#"<a[^>]*href="([^"]*)"[^>]*>(.*?)</a>"#).unwrap();
        md = re_a
            .replace_all(&md, |caps: &regex::Captures| {
                let url = caps.get(1).unwrap().as_str();
                let text = caps.get(2).unwrap().as_str();
                format!("[{}]({})", text, url)
            })
            .to_string();

        // Inline: images with alt
        let re_img_alt = Regex::new(r#"<img[^>]*src="([^"]*)"[^>]*alt="([^"]*)"[^>]*/?>"#).unwrap();
        md = re_img_alt
            .replace_all(&md, |caps: &regex::Captures| {
                format!("![{}]({})", caps.get(2).unwrap().as_str(), caps.get(1).unwrap().as_str())
            })
            .to_string();

        // Inline: images without alt
        let re_img = Regex::new(r#"<img[^>]*src="([^"]*)"[^>]*/?>"#).unwrap();
        md = re_img
            .replace_all(&md, |caps: &regex::Captures| {
                format!("![]( {})", caps.get(1).unwrap().as_str())
            })
            .to_string();

        // Inline: strong/bold
        let re_strong = Regex::new(r"<strong>(.*?)</strong>").unwrap();
        md = re_strong
            .replace_all(&md, |caps: &regex::Captures| {
                format!("**{}**", caps.get(1).unwrap().as_str())
            })
            .to_string();

        let re_b = Regex::new(r"<b>(.*?)</b>").unwrap();
        md = re_b
            .replace_all(&md, |caps: &regex::Captures| {
                format!("**{}**", caps.get(1).unwrap().as_str())
            })
            .to_string();

        // Inline: emphasis/italic
        let re_em = Regex::new(r"<em>(.*?)</em>").unwrap();
        md = re_em
            .replace_all(&md, |caps: &regex::Captures| {
                format!("*{}*", caps.get(1).unwrap().as_str())
            })
            .to_string();

        let re_i = Regex::new(r"<i>(.*?)</i>").unwrap();
        md = re_i
            .replace_all(&md, |caps: &regex::Captures| {
                format!("*{}*", caps.get(1).unwrap().as_str())
            })
            .to_string();

        // Inline: code
        let re_code = Regex::new(r"<code>(.*?)</code>").unwrap();
        md = re_code
            .replace_all(&md, |caps: &regex::Captures| {
                format!("`{}`", caps.get(1).unwrap().as_str())
            })
            .to_string();

        // Inline: line breaks
        let re_br = Regex::new(r"<br\s*/?>").unwrap();
        md = re_br.replace_all(&md, "\n").to_string();

        // Strip remaining HTML tags
        md = Self::strip_html(&md);

        // Normalize whitespace: collapse multiple blank lines
        let re_blank = Regex::new(r"\n{3,}").unwrap();
        md = re_blank.replace_all(&md, "\n\n").to_string();
        let md = md.trim().to_string();

        let word_count = Self::count_words(&md);
        let vsa_fingerprint = Self::compute_vsa_fingerprint(&md);

        ConvertedDoc {
            title,
            content: md,
            word_count,
            format,
            vsa_fingerprint,
        }
    }

    fn process_list_items(inner: &str, prefix: &str) -> String {
        let re_li = Regex::new(r"(?s)<li[^>]*>(.*?)</li>").unwrap();
        let items: Vec<String> = re_li
            .captures_iter(inner)
            .map(|caps| {
                let text = Self::strip_html(caps.get(1).unwrap().as_str());
                format!("{}{}", prefix, text.trim())
            })
            .collect();
        if items.is_empty() {
            String::new()
        } else {
            format!("{}\n\n", items.join("\n"))
        }
    }

    pub fn extract_title(html: &str) -> String {
        // Try <title> first
        if let Some(caps) =
            Regex::new(r"(?s)<title[^>]*>(.*?)</title>")
                .unwrap()
                .captures(html)
        {
            let t = caps.get(1).unwrap().as_str().trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }
        // Fall back to first <h1>
        if let Some(caps) = Regex::new(r"(?s)<h1[^>]*>(.*?)</h1>")
            .unwrap()
            .captures(html)
        {
            let t = Self::strip_html(caps.get(1).unwrap().as_str());
            let t = t.trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }
        String::new()
    }

    pub fn strip_html(html: &str) -> String {
        let re = Regex::new(r"<[^>]*>").unwrap();
        let result = re.replace_all(html, "");
        // Decode common HTML entities
        let result = result.replace("&amp;", "&");
        let result = result.replace("&lt;", "<");
        let result = result.replace("&gt;", ">");
        let result = result.replace("&quot;", "\"");
        let result = result.replace("&#39;", "'");
        let result = result.replace("&nbsp;", " ");
        result
    }

    pub fn detect_format(content: &str) -> SourceFormat {
        if content.contains('<')
            && (content.contains("</") || content.contains("/>"))
            && Regex::new(r"<[a-zA-Z/][^>]*>")
                .unwrap()
                .is_match(content)
        {
            SourceFormat::Html
        } else if content.contains('<') && Regex::new(r"<[a-zA-Z!?][^>]*>").unwrap().is_match(content) {
            SourceFormat::Html
        } else {
            SourceFormat::Text
        }
    }

    pub fn compute_vsa_fingerprint(content: &str) -> [u64; 4] {
        use std::hash::{Hash, Hasher};
        let bytes = content.as_bytes();
        let mut fp = [0u64; 4];
        for (i, slot) in fp.iter_mut().enumerate() {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            bytes.hash(&mut hasher);
            (i as u64).hash(&mut hasher);
            0xD0C0u64.hash(&mut hasher);
            *slot = hasher.finish();
        }
        fp
    }

    fn count_words(text: &str) -> usize {
        text.split_whitespace().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_markdown_headings() {
        let html = "<h1>Title</h1><h2>Section</h2><h3>Sub</h3>";
        let doc = DocConverter::html_to_markdown(html);
        assert!(doc.content.contains("# Title"), "content: {}", doc.content);
        assert!(doc.content.contains("## Section"));
        assert!(doc.content.contains("### Sub"));
    }

    #[test]
    fn test_html_to_markdown_paragraphs() {
        let html = "<p>Hello world</p><p>Second paragraph</p>";
        let doc = DocConverter::html_to_markdown(html);
        assert!(doc.content.contains("Hello world"));
        assert!(doc.content.contains("Second paragraph"));
    }

    #[test]
    fn test_html_to_markdown_links() {
        let html = r#"<a href="https://example.com">click here</a>"#;
        let doc = DocConverter::html_to_markdown(html);
        assert!(doc.content.contains("[click here](https://example.com)"));
    }

    #[test]
    fn test_html_to_markdown_images() {
        let html = r#"<img src="pic.jpg" alt="A photo">"#;
        let doc = DocConverter::html_to_markdown(html);
        assert!(doc.content.contains("![A photo](pic.jpg)"));
    }

    #[test]
    fn test_html_to_markdown_lists() {
        let html = "<ul><li>Item A</li><li>Item B</li></ul>";
        let doc = DocConverter::html_to_markdown(html);
        assert!(doc.content.contains("- Item A"));
        assert!(doc.content.contains("- Item B"));
    }

    #[test]
    fn test_html_to_markdown_ordered_list() {
        let html = "<ol><li>First</li><li>Second</li></ol>";
        let doc = DocConverter::html_to_markdown(html);
        assert!(doc.content.contains("1. First"));
        assert!(doc.content.contains("1. Second"));
    }

    #[test]
    fn test_html_to_markdown_inline_styles() {
        let html = "<p><strong>bold</strong> and <em>italic</em></p>";
        let doc = DocConverter::html_to_markdown(html);
        assert!(doc.content.contains("**bold**"));
        assert!(doc.content.contains("*italic*"));
    }

    #[test]
    fn test_html_to_markdown_code() {
        let html = "<p>Use <code>fn main()</code></p>";
        let doc = DocConverter::html_to_markdown(html);
        assert!(doc.content.contains("`fn main()`"));
    }

    #[test]
    fn test_html_to_markdown_blockquote() {
        let html = "<blockquote>Cited text</blockquote>";
        let doc = DocConverter::html_to_markdown(html);
        assert!(doc.content.contains("> Cited text"));
    }

    #[test]
    fn test_html_to_markdown_break() {
        let html = "<p>Line1<br>Line2</p>";
        let doc = DocConverter::html_to_markdown(html);
        assert!(doc.content.contains("Line1\nLine2") || doc.content.contains("Line1\n\nLine2"));
    }

    #[test]
    fn test_extract_title_from_tag() {
        let html = "<html><title>My Page</title><body></body></html>";
        assert_eq!(DocConverter::extract_title(html), "My Page");
    }

    #[test]
    fn test_extract_title_from_h1() {
        let html = "<h1>Document Title</h1><p>content</p>";
        assert_eq!(DocConverter::extract_title(html), "Document Title");
    }

    #[test]
    fn test_extract_title_empty() {
        assert_eq!(DocConverter::extract_title("<p>no title</p>"), "");
    }

    #[test]
    fn test_strip_html() {
        let html = "<p>Hello <b>world</b></p>";
        assert_eq!(DocConverter::strip_html(html), "Hello world");
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(DocConverter::detect_format("<p>hi</p>"), SourceFormat::Html);
        assert_eq!(
            DocConverter::detect_format("plain text"),
            SourceFormat::Text
        );
    }

    #[test]
    fn test_word_count() {
        let html = "<p>one two three</p><p>four five</p>";
        let doc = DocConverter::html_to_markdown(html);
        assert_eq!(doc.word_count, 5);
    }

    #[test]
    fn test_vsa_fingerprint_deterministic() {
        let fp1 = DocConverter::compute_vsa_fingerprint("hello world");
        let fp2 = DocConverter::compute_vsa_fingerprint("hello world");
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_vsa_fingerprint_different_inputs() {
        let fp1 = DocConverter::compute_vsa_fingerprint("hello");
        let fp2 = DocConverter::compute_vsa_fingerprint("world");
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_html_to_markdown_code_block() {
        let html = "<pre><code>let x = 1;</code></pre>";
        let doc = DocConverter::html_to_markdown(html);
        assert!(doc.content.contains("```"));
        assert!(doc.content.contains("let x = 1;"));
    }

    #[test]
    fn test_html_entities() {
        let html = "<p>AT&amp;T &lt;test&gt;</p>";
        let doc = DocConverter::html_to_markdown(html);
        assert!(doc.content.contains("AT&T"));
        assert!(doc.content.contains("<test>"));
    }

    #[test]
    fn test_horizontal_rule() {
        let html = "<hr>";
        let doc = DocConverter::html_to_markdown(html);
        assert!(doc.content.contains("---"));
    }
}
