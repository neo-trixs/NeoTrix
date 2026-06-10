use std::path::Path;
use std::collections::HashMap;
use crate::core::nt_core_hcube::cross_modal::CrossModalAligner;
use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;
use super::types::*;

pub trait DocumentParser: Send + Sync {
    fn name(&self) -> &str;
    fn supported_formats(&self) -> Vec<DocumentFormat>;
    fn can_parse(&self, path: &Path, format: &DocumentFormat) -> bool;
    fn parse(&self, path: &Path) -> Result<ParsedDocument, String>;
    fn parse_text(&self, text: &str, format: DocumentFormat) -> ParsedDocument;
}

fn text_to_vsa(text: &str) -> Vec<u8> {
    let aligner = CrossModalAligner::new(VSA_DIM, 42);
    aligner.text_to_vsa(text)
}

fn build_parsed(document: Document) -> ParsedDocument {
    let sections: Vec<&Section> = document.sections.iter().flat_map(|s| s.flatten()).collect();
    let vsa_vectors: Vec<Vec<u8>> = sections.iter().map(|s| text_to_vsa(&s.content)).collect();
    let combined_vector = super::engine::bundle_sections(&vsa_vectors);
    let section_count = sections.len();

    let total_words: usize = sections.iter().map(|s| s.content.split_whitespace().count()).sum();
    let estimated_reading_time = if total_words > 0 {
        (total_words as f64 / 200.0) * 60.0
    } else {
        0.0
    };

    ParsedDocument {
        document,
        vsa_vectors,
        combined_vector,
        section_count,
        estimated_reading_time,
    }
}

pub struct PlainTextParser;

impl PlainTextParser {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentParser for PlainTextParser {
    fn name(&self) -> &str {
        "plain_text_parser"
    }

    fn supported_formats(&self) -> Vec<DocumentFormat> {
        vec![DocumentFormat::PlainText, DocumentFormat::Rtf]
    }

    fn can_parse(&self, _path: &Path, format: &DocumentFormat) -> bool {
        matches!(format, DocumentFormat::PlainText | DocumentFormat::Rtf)
    }

    fn parse(&self, path: &Path) -> Result<ParsedDocument, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
        let format = DocumentFormat::from_extension(path).unwrap_or(DocumentFormat::PlainText);
        Ok(self.parse_text(&text, format))
    }

    fn parse_text(&self, text: &str, format: DocumentFormat) -> ParsedDocument {
        let raw_text = text.to_string();
        let title = text.lines().next().map(|l| l.trim().to_string()).filter(|l| !l.is_empty());

        let mut sections = Vec::new();
        for block in text.split("\n\n") {
            let trimmed = block.trim();
            if trimmed.is_empty() {
                continue;
            }
            sections.push(Section {
                heading: None,
                level: 0,
                content: trimmed.to_string(),
                bounding_box: None,
                subsections: vec![],
            });
        }

        let document = Document {
            format,
            title,
            sections,
            metadata: HashMap::new(),
            raw_text,
        };

        build_parsed(document)
    }
}

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }

    fn parse_heading_level(line: &str) -> Option<(u8, String)> {
        let trimmed = line.trim();
        let mut level = 0u8;
        for ch in trimmed.chars() {
            if ch == '#' {
                level += 1;
            } else if ch == ' ' {
                if level > 0 && level <= 6 {
                    let heading = trimmed[level as usize + 1..].trim().to_string();
                    return Some((level, heading));
                }
                return None;
            } else {
                return None;
            }
        }
        None
    }
}

fn build_section_tree(items: Vec<(u8, Option<String>, String)>) -> Vec<Section> {
    let mut root: Vec<Section> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();

    for (level, heading, content) in items {
        let section = Section {
            heading,
            level,
            content,
            bounding_box: None,
            subsections: vec![],
        };

        while let Some(&_unused) = stack.last() {
            if get_section_level(&root, &stack) >= level {
                stack.pop();
            } else {
                break;
            }
        }

        if stack.is_empty() {
            root.push(section);
            stack.push(root.len() - 1);
        } else {
            let section_ref = get_section_mut(&mut root, &stack);
            section_ref.subsections.push(section);
            let idx = section_ref.subsections.len() - 1;
            stack.push(idx);
        }
    }

    root
}

fn get_section_level<'a>(root: &'a [Section], stack: &[usize]) -> u8 {
    if stack.is_empty() {
        return 0;
    }
    let mut current = root;
    let last_idx = stack.len() - 1;
    for &idx in &stack[..last_idx] {
        if idx < current.len() {
            current = &current[idx].subsections;
        } else {
            return 0;
        }
    }
    if last_idx < stack.len() {
        let idx = stack[last_idx];
        if idx < current.len() {
            return current[idx].level;
        }
    }
    0
}

fn get_section_mut<'a>(root: &'a mut Vec<Section>, stack: &[usize]) -> &'a mut Section {
    let mut current = root;
    for &idx in stack.iter().take(stack.len().saturating_sub(1)) {
        current = &mut current[idx].subsections;
    }
    let last = *stack.last().unwrap();
    &mut current[last]
}

impl DocumentParser for MarkdownParser {
    fn name(&self) -> &str {
        "markdown_parser"
    }

    fn supported_formats(&self) -> Vec<DocumentFormat> {
        vec![DocumentFormat::Markdown]
    }

    fn can_parse(&self, _path: &Path, format: &DocumentFormat) -> bool {
        matches!(format, DocumentFormat::Markdown)
    }

    fn parse(&self, path: &Path) -> Result<ParsedDocument, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
        Ok(self.parse_text(&text, DocumentFormat::Markdown))
    }

    fn parse_text(&self, text: &str, format: DocumentFormat) -> ParsedDocument {
        let raw_text = text.to_string();
        let lines: Vec<&str> = text.lines().collect();

        let title = lines.first()
            .and_then(|l| Self::parse_heading_level(l))
            .filter(|(level, _)| *level == 1)
            .map(|(_, heading)| heading);

        let mut items: Vec<(u8, Option<String>, String)> = Vec::new();
        let mut current_heading: Option<(u8, String)> = None;
        let mut current_content = String::new();

        for line in &lines {
            if let Some((level, heading)) = Self::parse_heading_level(line) {
                if let Some((prev_level, ref prev_heading)) = current_heading {
                    let content = current_content.trim().to_string();
                    items.push((prev_level, Some(prev_heading.clone()), content));
                    current_content.clear();
                }
                current_heading = Some((level, heading));
            } else {
                if !current_content.is_empty() {
                    current_content.push('\n');
                }
                current_content.push_str(line);
            }
        }

        if let Some((level, heading)) = current_heading.take() {
            let content = current_content.trim().to_string();
            items.push((level, Some(heading), content));
        }

        let sections = if items.is_empty() && !raw_text.trim().is_empty() {
            vec![Section {
                heading: None,
                level: 0,
                content: raw_text.trim().to_string(),
                bounding_box: None,
                subsections: vec![],
            }]
        } else {
            build_section_tree(items)
        };

        let document = Document {
            format,
            title,
            sections,
            metadata: HashMap::new(),
            raw_text,
        };

        build_parsed(document)
    }
}

pub struct HtmlParser;

impl HtmlParser {
    pub fn new() -> Self {
        Self
    }

    fn strip_tags(html: &str) -> String {
        let mut result = String::with_capacity(html.len());
        let mut in_tag = false;
        let mut in_entity = false;
        let mut entity_buf = String::new();

        for ch in html.chars() {
            match ch {
                '<' => in_tag = true,
                '>' if in_tag => in_tag = false,
                '&' if !in_tag => {
                    in_entity = true;
                    entity_buf.clear();
                }
                ';' if in_entity => {
                    in_entity = false;
                    let decoded = match entity_buf.as_str() {
                        "amp" => "&",
                        "lt" => "<",
                        "gt" => ">",
                        "quot" => "\"",
                        "apos" => "'",
                        "nbsp" => " ",
                        _ => "",
                    };
                    result.push_str(decoded);
                }
                _ if in_entity => entity_buf.push(ch),
                _ if !in_tag && !in_entity => result.push(ch),
                _ => {}
            }
        }
        result
    }

    fn parse_headings(&self, html: &str) -> Vec<(u8, String, usize)> {
        let mut headings = Vec::new();
        for level in 1..=6 {
            let tag = format!("h{}", level);
            let mut search_start = 0;
            while let Some(start) = html[search_start..].find(&format!("<{}", tag)) {
                let abs_start = search_start + start;
                let tag_close = html[abs_start..].find('>');
                let content_start = match tag_close {
                    Some(pos) => abs_start + pos + 1,
                    None => break,
                };
                let end_tag = format!("</{}>", tag);
                let content_end = html[content_start..].find(&end_tag);
                let content = match content_end {
                    Some(pos) => {
                        let raw = &html[content_start..content_start + pos];
                        Self::strip_tags(raw)
                    }
                    None => break,
                };
                headings.push((level, content.trim().to_string(), abs_start));
                search_start = content_start + 1;
            }
        }
        headings.sort_by_key(|&(_, _, pos)| pos);
        headings
    }

    fn extract_text(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        let mut in_script = false;
        let mut in_style = false;
        let mut tag_buf = String::new();

        for ch in html.chars() {
            match ch {
                '<' => {
                    in_tag = true;
                    tag_buf.clear();
                }
                '>' if in_tag => {
                    in_tag = false;
                    let tag_lower = tag_buf.to_lowercase();
                    if tag_lower.starts_with("script") || tag_lower.starts_with("/script") {
                        in_script = tag_lower.starts_with("script") && !tag_lower.starts_with("/script");
                    }
                    if tag_lower.starts_with("style") || tag_lower.starts_with("/style") {
                        in_style = tag_lower.starts_with("style") && !tag_lower.starts_with("/style");
                    }
                    if !result.is_empty() && !result.ends_with(' ') {
                        result.push(' ');
                    }
                }
                _ if !in_tag && !in_script && !in_style => {
                    if !ch.is_control() {
                        result.push(ch);
                    }
                }
                _ if in_tag => tag_buf.push(ch),
                _ => {}
            }
        }

        let mut cleaned = String::new();
        let mut prev_space = false;
        for ch in result.chars() {
            if ch.is_whitespace() {
                if !prev_space {
                    cleaned.push(' ');
                    prev_space = true;
                }
            } else {
                cleaned.push(ch);
                prev_space = false;
            }
        }
        cleaned.trim().to_string()
    }
}

impl DocumentParser for HtmlParser {
    fn name(&self) -> &str {
        "html_parser"
    }

    fn supported_formats(&self) -> Vec<DocumentFormat> {
        vec![DocumentFormat::Html]
    }

    fn can_parse(&self, _path: &Path, format: &DocumentFormat) -> bool {
        matches!(format, DocumentFormat::Html)
    }

    fn parse(&self, path: &Path) -> Result<ParsedDocument, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
        Ok(self.parse_text(&text, DocumentFormat::Html))
    }

    fn parse_text(&self, text: &str, format: DocumentFormat) -> ParsedDocument {
        let raw_text = Self::extract_text(text);
        let headings = self.parse_headings(text);

        let title = headings.first()
            .filter(|(level, _, _)| *level == 1)
            .map(|(_, heading, _)| heading.clone());

        let mut items: Vec<(u8, Option<String>, String)> = Vec::new();
        let mut last_heading: Option<(u8, String, usize)> = None;
        let mut content_buffer = String::new();

        for heading in &headings {
            if let Some((level, ref htext, _)) = last_heading {
                let content = content_buffer.trim().to_string();
                items.push((level, Some(htext.clone()), content));
                content_buffer.clear();
            }
            last_heading = Some(heading.clone());
        }

        if let Some((level, htext, _)) = last_heading {
            let content = content_buffer.trim().to_string();
            items.push((level, Some(htext), content));
        }

        let sections = if items.is_empty() && !raw_text.is_empty() {
            vec![Section {
                heading: None,
                level: 0,
                content: raw_text.clone(),
                bounding_box: None,
                subsections: vec![],
            }]
        } else {
            build_section_tree(items)
        };

        let document = Document {
            format,
            title,
            sections,
            metadata: HashMap::new(),
            raw_text,
        };

        build_parsed(document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text_parser_sections() {
        let parser = PlainTextParser::new();
        let text = "First paragraph of text.\nIt continues here.\n\nSecond paragraph here.\n\nThird one.";
        let result = parser.parse_text(text, DocumentFormat::PlainText);
        assert_eq!(result.section_count, 3);
        assert!(result.estimated_reading_time > 0.0);
    }

    #[test]
    fn test_plain_text_parser_title() {
        let parser = PlainTextParser::new();
        let result = parser.parse_text("Title Line\n\nContent here.", DocumentFormat::PlainText);
        assert_eq!(result.document.title.as_deref(), Some("Title Line"));
    }

    #[test]
    fn test_markdown_parser_headings() {
        let parser = MarkdownParser::new();
        let text = "# Main Title\n\nIntro paragraph.\n\n## Section One\n\nContent of section one.\n\n### Subsection\n\nDeep content.\n\n## Section Two\n\nFinal content.";
        let result = parser.parse_text(text, DocumentFormat::Markdown);
        assert_eq!(result.document.title.as_deref(), Some("Main Title"));
        assert_eq!(result.section_count, 4);
        assert!(result.estimated_reading_time > 0.0);
    }

    #[test]
    fn test_markdown_parser_hierarchy() {
        let parser = MarkdownParser::new();
        let text = "# Top\n\n## A\n\nA content.\n\n## B\n\nB content.\n\n### B1\n\nB1 content.\n\n# Next\n\nNext content.";
        let result = parser.parse_text(text, DocumentFormat::Markdown);
        assert_eq!(result.document.sections.len(), 2);
        let first = &result.document.sections[0];
        assert_eq!(first.heading.as_deref(), Some("Top"));
        assert_eq!(first.subsections.len(), 2);
        assert_eq!(first.subsections[0].heading.as_deref(), Some("A"));
        assert_eq!(first.subsections[1].subsections.len(), 1);
        assert_eq!(first.subsections[1].subsections[0].heading.as_deref(), Some("B1"));
    }

    #[test]
    fn test_html_parser_strip_tags() {
        assert_eq!(
            HtmlParser::strip_tags("<p>Hello <b>World</b></p>"),
            "Hello World"
        );
    }

    #[test]
    fn test_html_parser_extract_text() {
        let html = "<html><body><h1>Title</h1><p>Some text here.</p><script>var x=1;</script></body></html>";
        let text = HtmlParser::extract_text(html);
        assert!(text.contains("Title"));
        assert!(text.contains("Some text here."));
        assert!(!text.contains("var x=1;"));
    }

    #[test]
    fn test_html_parser_sections() {
        let parser = HtmlParser::new();
        let html = "<html><h1>Main</h1><p>Intro.</p><h2>Sub</h2><p>Detail.</p></html>";
        let result = parser.parse_text(html, DocumentFormat::Html);
        assert_eq!(result.document.title.as_deref(), Some("Main"));
        assert!(result.section_count >= 2);
    }

    #[test]
    fn test_vsa_vectors_non_empty() {
        let parser = PlainTextParser::new();
        let text = "Hello world.\n\nSecond section here.";
        let result = parser.parse_text(text, DocumentFormat::PlainText);
        for v in &result.vsa_vectors {
            assert_eq!(v.len(), VSA_DIM);
        }
        assert_eq!(result.combined_vector.len(), VSA_DIM);
    }

    #[test]
    fn test_supported_formats() {
        let mp = MarkdownParser::new();
        assert_eq!(mp.supported_formats(), vec![DocumentFormat::Markdown]);

        let pp = PlainTextParser::new();
        let formats = pp.supported_formats();
        assert!(formats.contains(&DocumentFormat::PlainText));
        assert!(formats.contains(&DocumentFormat::Rtf));
    }

    #[test]
    fn test_html_entity_decoding() {
        let decoded = HtmlParser::strip_tags("<p>AT&amp;T &amp; Co.</p>");
        assert_eq!(decoded, "AT&T & Co.");
    }
}
