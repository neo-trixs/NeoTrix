#![allow(dead_code)]

use super::office_common::{list_zip_entries, read_zip_entry, xml_unescape};

/// docx extraction engine: reads raw .docx bytes (ZIP container) and extracts
/// paragraph text from `word/document.xml`.
#[derive(Debug, Clone)]
pub struct DocxExtractor {
    max_paragraphs: usize,
    include_headers: bool,
}

impl Default for DocxExtractor {
    fn default() -> Self {
        Self {
            max_paragraphs: 10000,
            include_headers: false,
        }
    }
}

impl DocxExtractor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_paragraphs(mut self, n: usize) -> Self {
        self.max_paragraphs = n;
        self
    }
    pub fn with_headers(mut self, yes: bool) -> Self {
        self.include_headers = yes;
        self
    }

    /// Extract text from a .docx file as a list of paragraphs.
    pub fn extract(&self, data: &[u8]) -> Result<Vec<String>, String> {
        let entries = list_zip_entries(data)?;

        // Find the main document XML
        let doc_entry = entries
            .iter()
            .find(|e| e.name == "word/document.xml" || e.name == "/word/document.xml")
            .ok_or_else(|| "word/document.xml not found in docx".to_string())?;
        let xml = read_zip_entry(data, doc_entry)?;

        self.parse_paragraphs(&xml)
    }

    /// Parse paragraphs from document.xml.
    /// In OOXML, paragraphs are `<w:p>` elements containing `<w:r>` runs with `<w:t>` text.
    fn parse_paragraphs(&self, xml: &[u8]) -> Result<Vec<String>, String> {
        let s = String::from_utf8_lossy(xml);
        let mut paragraphs = Vec::new();
        let mut count = 0;

        // Find all <w:p> elements (with namespace prefix)
        // The namespace is usually `w:` but could be different; find by suffix `:p>`
        let mut pos = 0;
        while count < self.max_paragraphs {
            // Look for opening w:p tag
            let p_start = match find_ns_tag(&s[pos..], "p") {
                Some(idx) => pos + idx,
                None => break,
            };

            // Find closing w:p tag
            let p_end = match find_ns_close(&s[p_start..], "p") {
                Some(idx) => p_start + idx,
                None => break,
            };

            let p_block = &s[p_start..p_end];

            // Extract text from all <w:t> elements within this paragraph
            let text_parts: Vec<String> = extract_all_ns_tags(p_block.as_bytes(), "t");
            let para_text = text_parts.join("").trim().to_string();

            if !para_text.is_empty() {
                paragraphs.push(xml_unescape(&para_text));
            } else {
                // Empty paragraphs become blank lines
                paragraphs.push(String::new());
            }

            count += 1;
            pos = p_end;
        }

        Ok(paragraphs)
    }

    /// Extract text as a single markdown document.
    pub fn to_markdown(&self, data: &[u8]) -> Result<String, String> {
        let paragraphs = self.extract(data)?;
        let mut md = String::new();
        for para in &paragraphs {
            if para.is_empty() {
                md.push('\n');
            } else {
                md.push_str(para);
                md.push('\n');
            }
        }
        // Try to extract headers/footers
        if self.include_headers {
            if let Ok(entries) = list_zip_entries(data) {
                for entry in &entries {
                    let name = entry.name.as_str();
                    if name.contains("header") && name.ends_with(".xml") {
                        if let Ok(xml) = read_zip_entry(data, entry) {
                            if let Ok(h_text) = self.parse_paragraphs(&xml) {
                                if !h_text.is_empty() {
                                    md.insert_str(0, "<!-- header -->\n");
                                    for h in &h_text {
                                        md.insert_str(0, &format!("> {h}\n"));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(md)
    }
}

/// Find a namespace-prefixed tag like `<w:p` or `<w:p ` or `</w:p>`.
fn find_ns_tag(s: &str, local_name: &str) -> Option<usize> {
    // Search for `<` followed by optional namespace prefix, then `:local_name`
    let mut pos = 0;
    while let Some(lt) = s[pos..].find('<') {
        let abs = pos + lt;
        let after_lt = &s[abs + 1..];
        // Skip comments and processing instructions
        if after_lt.starts_with("!") || after_lt.starts_with("?") {
            pos = abs + 1;
            continue;
        }
        // Check if this is an opening tag (<w:p, <w:p>, <p, etc.)
        let tag_end = after_lt
            .find(|c: char| c == '>' || c == ' ' || c == '/' || c == ':')
            .unwrap_or(0);
        let ns_end = after_lt.find(':');
        if let Some(ns_pos) = ns_end {
            if ns_pos <= tag_end || tag_end == 0 {
                let name_start = ns_pos + 1;
                let name_end = after_lt[name_start..]
                    .find(|c: char| c == '>' || c == ' ' || c == '/')
                    .unwrap_or(after_lt[name_start..].len());
                let name = &after_lt[name_start..name_start + name_end];
                if name == local_name {
                    return Some(abs);
                }
            }
        } else if tag_end > 0 {
            let name = &after_lt[..tag_end];
            if name == local_name {
                return Some(abs);
            }
        }
        pos = abs + 1;
    }
    None
}

/// Find closing tag like `</w:p>` or `</p>`.
fn find_ns_close(s: &str, local_name: &str) -> Option<usize> {
    let mut pos = 0;
    while let Some(lt) = s[pos..].find("</") {
        let abs = pos + lt + 2;
        let after_slash = &s[abs..];
        let tag_end = after_slash
            .find(|c: char| c == '>' || c == ' ' || c == ':')
            .unwrap_or(after_slash.len());
        let ns_pos = after_slash.find(':');
        let name = if let Some(ns_idx) = ns_pos {
            if ns_idx < tag_end {
                &after_slash[ns_idx + 1..tag_end]
            } else {
                &after_slash[..tag_end]
            }
        } else {
            &after_slash[..tag_end]
        };
        if name == local_name {
            // Find the closing >
            let close = s[pos + lt..].find('>')?;
            return Some(pos + lt + close + 1);
        }
        pos = abs;
    }
    None
}

/// Extract text content from all namespace-prefixed tags like `<w:t>`.
fn extract_all_ns_tags(xml: &[u8], local_name: &str) -> Vec<String> {
    let s = String::from_utf8_lossy(xml);
    let mut results = Vec::new();
    let mut pos = 0;

    // Find opening tag `<prefix:t>` or `<prefix:t ...>`
    while let Some(lt) = s[pos..].find('<') {
        let abs = pos + lt;
        let after_lt = &s[abs + 1..];
        // Check for closing tag pattern
        if after_lt.starts_with('/') {
            pos = abs + 1;
            continue;
        }
        // Extract tag name up to >, space, /, or :
        let tag_end = after_lt
            .find(|c: char| c == '>' || c == ' ' || c == '/' || c == ':')
            .unwrap_or(after_lt.len());
        let ns_pos = after_lt.find(':');
        let name = if let Some(ns_idx) = ns_pos {
            if ns_idx < tag_end {
                &after_lt[ns_idx + 1..tag_end]
            } else {
                &after_lt[..tag_end]
            }
        } else {
            &after_lt[..tag_end]
        };

        if name == local_name {
            // Find content start (after '>')
            let content_start = match after_lt.find('>') {
                Some(o) => abs + 1 + o + 1,
                None => {
                    pos = abs + 1;
                    continue;
                }
            };
            // Find closing tag
            let close_search = format!("</{name}>");
            let close_search_ns = if ns_pos.is_some() {
                let prefix = &after_lt[..ns_pos.unwrap()];
                format!("</{prefix}:{name}>")
            } else {
                close_search.clone()
            };

            let remaining = &s[content_start..];
            let close_loc = remaining
                .find(&close_search)
                .or_else(|| remaining.find(&close_search_ns));
            let close_loc = match close_loc {
                Some(o) => o,
                None => {
                    pos = abs + 1;
                    continue;
                }
            };
            results.push(remaining[..close_loc].to_string());
            pos = content_start + close_loc + close_search.len();
        } else {
            pos = abs + 1;
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_defaults() {
        let ex = DocxExtractor::new();
        assert_eq!(ex.max_paragraphs, 10000);
    }

    #[test]
    fn test_invalid_docx() {
        let ex = DocxExtractor::new();
        let result = ex.extract(b"not a zip file");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_ns_tag_inline() {
        let s =
            r#"<w:document><w:body><w:p><w:r><w:t>Hello</w:t></w:r></w:p></w:body></w:document>"#;
        let pos = find_ns_tag(s, "p");
        assert!(pos.is_some());
    }

    #[test]
    fn test_find_ns_close() {
        let s = r#"<w:p><w:r><w:t>Hello</w:t></w:r></w:p>"#;
        let pos = find_ns_close(s, "p");
        assert!(pos.is_some());
        assert_eq!(&s[pos.unwrap() - 6..pos.unwrap()], "</w:p>");
    }

    #[test]
    fn test_extract_all_ns_tags_simple() {
        let xml = br#"<w:document><w:t>Hello</w:t><w:t>World</w:t></w:document>"#;
        let texts = extract_all_ns_tags(xml, "t");
        assert_eq!(texts, vec!["Hello", "World"]);
    }
}
