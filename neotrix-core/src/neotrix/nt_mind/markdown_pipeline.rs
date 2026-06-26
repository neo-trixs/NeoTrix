//! Markdown conversion pipeline — HTML→Markdown, ExtractedContent→Markdown
//!
//! Part of G303.1-2 (Wave 4) evolution roadmap.
//! All parsing done with string matching (same approach as selection_engine.rs).
//! Zero new external crate dependencies.

use crate::neotrix::nt_mind::content_extractor::{ContentBlock, ExtractedContent};
use std::collections::HashMap;

// ============================================================================
// Constants — HTML void elements (no closing tag)
// ============================================================================

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

// ============================================================================
// ToMarkdown trait
// ============================================================================

/// Markdown conversion trait — all document types implement this
pub trait ToMarkdown {
    fn to_markdown(&self) -> Result<String, String>;
    fn source_type(&self) -> &'static str;
}

// ============================================================================
// BlockType — classification for extracted content blocks
// ============================================================================

/// Block type classification for content extraction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Heading,
    Paragraph,
    List,
    Table,
    Code,
    Image,
    Div,
    Blockquote,
}

// ============================================================================
// Part 1: HtmlToMarkdown
// ============================================================================

/// HTML to Markdown converter using string-based scanning (no external parser).
///
/// Converts common HTML elements to their Markdown equivalents:
/// - Headings: `<h1>`-`<h6>` → `#`-`######`
/// - Links: `<a href="..">` → `[text](url)`
/// - Images: `<img>` → `![alt](src)`
/// - Code: `<code>` → `` ` ``, `<pre><code>` → fenced code block
/// - Lists: `<ul>/<ol>` → `- ` / `1. ` (nested with indent)
/// - Tables: `<table>` → pipe tables
/// - Blockquotes: `<blockquote>` → `> `
/// - Inline: `<strong>/<b>` → `**`, `<em>/<i>` → `*`
pub struct HtmlToMarkdown {
    pub preserve_tables: bool,
    pub preserve_code_blocks: bool,
    pub preserve_links: bool,
    pub preserve_images: bool,
    pub preserve_lists: bool,
    pub max_heading_depth: u8,
    pub line_width: usize,
}

impl HtmlToMarkdown {
    /// Create a new converter with sensible defaults:
    /// - All content types preserved
    /// - Max heading depth: 6
    /// - Line width: 80
    pub fn new() -> Self {
        Self {
            preserve_tables: true,
            preserve_code_blocks: true,
            preserve_links: true,
            preserve_images: true,
            preserve_lists: true,
            max_heading_depth: 6,
            line_width: 80,
        }
    }

    /// Convert a full HTML string to Markdown.
    ///
    /// Uses character-by-character scanning with tag parsing (no external parser).
    /// Returns the Markdown string, or an error if the input is malformed.
    pub fn from_html(&self, html: &str) -> Result<String, String> {
        if html.is_empty() {
            return Ok(String::new());
        }
        let mut state = ConvState::new(self);
        let bytes = html.as_bytes();
        let n = bytes.len();
        let mut i = 0;

        while i < n {
            if bytes[i] == b'<' {
                match self.parse_tag_at(html, i) {
                    Some(tag) => {
                        if tag.is_comment {
                            i = tag.end;
                            continue;
                        }
                        if tag.is_closing {
                            self.handle_closing_tag(&tag, &mut state);
                        } else if tag.is_self_closing {
                            self.handle_self_closing_tag(&tag, &mut state);
                        } else {
                            self.handle_opening_tag(&tag, &mut state);
                        }
                        i = tag.end;
                    }
                    None => {
                        state.output.push('<');
                        i += 1;
                    }
                }
            } else {
                if state.in_pre {
                    state.pre_buffer.push(bytes[i] as char);
                } else {
                    let c = bytes[i] as char;
                    state.output.push(c);
                }
                i += 1;
            }
        }

        if state.in_table && !state.table_rows.is_empty() {
            self.emit_table(&mut state);
        }
        if state.in_pre && !state.pre_buffer.is_empty() {
            self.emit_pre_block(&mut state);
        }

        Ok(state.output)
    }

    /// Convert from existing ExtractedContent blocks.
    ///
    /// Uses block content and heading information to guide markdown formatting.
    pub fn from_extracted_content(&self, content: &ExtractedContent) -> Result<String, String> {
        let mut output = String::new();
        for block in &content.blocks {
            let block_type = self.infer_block_type(block);
            match block_type {
                BlockType::Heading => {
                    if let Some(ref h) = block.heading {
                        output.push_str(&format!("## {}\n\n", h));
                    }
                    output.push_str(&block.body);
                    output.push_str("\n\n");
                }
                BlockType::Code if self.preserve_code_blocks => {
                    output.push_str("```\n");
                    output.push_str(&block.body);
                    if !block.body.ends_with('\n') {
                        output.push('\n');
                    }
                    output.push_str("```\n\n");
                }
                BlockType::Blockquote => {
                    for line in block.body.lines() {
                        output.push_str("> ");
                        output.push_str(line);
                        output.push('\n');
                    }
                    output.push('\n');
                }
                BlockType::Table if self.preserve_tables => {
                    let rows: Vec<&str> = block.body.lines().collect();
                    if rows.len() >= 2 {
                        for (idx, row) in rows.iter().enumerate() {
                            output.push('|');
                            for cell in row.split('\t') {
                                output.push(' ');
                                output.push_str(cell.trim());
                                output.push_str(" |");
                            }
                            output.push('\n');
                            if idx == 0 {
                                output.push('|');
                                for _cell in row.split('\t') {
                                    output.push_str(" --- |");
                                }
                                output.push('\n');
                            }
                        }
                    } else {
                        output.push_str(&block.body);
                    }
                    output.push('\n');
                }
                _ => {
                    if let Some(ref h) = block.heading {
                        output.push_str(&format!("## {}\n\n", h));
                    }
                    output.push_str(&block.body);
                    output.push_str("\n\n");
                }
            }
        }
        Ok(output)
    }

    /// Escape Markdown special characters in text.
    ///
    /// Escapes: `\ * _ [ ] ( ) # + - !`
    pub fn escape_markdown(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        for c in text.chars() {
            match c {
                '\\' => result.push_str("\\\\"),
                '*' => result.push_str("\\*"),
                '_' => result.push_str("\\_"),
                '[' => result.push_str("\\["),
                ']' => result.push_str("\\]"),
                '(' => result.push_str("\\("),
                ')' => result.push_str("\\)"),
                '#' => result.push_str("\\#"),
                '+' => result.push_str("\\+"),
                '-' => result.push_str("\\-"),
                '!' => result.push_str("\\!"),
                _ => result.push(c),
            }
        }
        result
    }

    // ========================================================================
    // Internal: Tag parsing
    // ========================================================================

    /// Parse a tag starting at position `pos` (which must point to '<').
    fn parse_tag_at(&self, html: &str, pos: usize) -> Option<RawTag> {
        if !html[pos..].starts_with('<') {
            return None;
        }
        let rest = &html[pos + 1..];

        // HTML comment
        if rest.starts_with("!--") {
            let end = html[pos..].find("-->")?;
            return Some(RawTag {
                tag_name: String::new(),
                attrs: HashMap::new(),
                start: pos,
                end: pos + end + 3,
                is_self_closing: true,
                is_closing: false,
                is_comment: true,
            });
        }

        let is_closing = rest.starts_with('/');
        let after_slash = if is_closing { 1 } else { 0 };
        let body = &rest[after_slash..];

        let name_end = body
            .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .unwrap_or(body.len());
        if name_end == 0 {
            return None;
        }
        let tag_name = body[..name_end].to_lowercase();

        let after_name = &body[name_end..];
        let gt_offset = self.skip_past_tag_body(after_name)?;
        let attrs_raw = after_name[..gt_offset - 1].trim();

        let is_self_closing =
            attrs_raw.ends_with('/') || VOID_ELEMENTS.contains(&tag_name.as_str());
        let attrs = self.parse_attrs(attrs_raw);
        let end = pos + 1 + after_slash + name_end + gt_offset;

        Some(RawTag {
            tag_name,
            attrs,
            start: pos,
            end,
            is_self_closing,
            is_closing,
            is_comment: false,
        })
    }

    /// Find the offset (from start of `s`) of the closing '>'.
    fn skip_past_tag_body(&self, s: &str) -> Option<usize> {
        let mut in_quote = false;
        let mut quote_char = '"';
        for (i, c) in s.char_indices() {
            if in_quote {
                if c == quote_char {
                    in_quote = false;
                }
            } else {
                match c {
                    '"' | '\'' => {
                        in_quote = true;
                        quote_char = c;
                    }
                    '>' => return Some(i + 1),
                    _ => {}
                }
            }
        }
        None
    }

    /// Parse attributes from an attribute string (e.g. `href="https://x.com" class="link"`).
    fn parse_attrs(&self, s: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        let mut i = 0;
        let bytes = s.as_bytes();
        let n = bytes.len();
        while i < n {
            // Skip whitespace
            while i < n && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= n {
                break;
            }
            // Read attribute name
            let name_start = i;
            while i < n && bytes[i] != b'=' && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
                i += 1;
            }
            let name = s[name_start..i].trim().to_string();
            if name.is_empty() || name == "/" {
                break;
            }
            // Skip whitespace around =
            while i < n && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < n && bytes[i] == b'=' {
                i += 1;
                while i < n && bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                // Read value
                if i < n && (bytes[i] == b'"' || bytes[i] == b'\'') {
                    let quote = bytes[i];
                    i += 1;
                    let val_start = i;
                    while i < n && bytes[i] != quote {
                        i += 1;
                    }
                    let value = s[val_start..i].to_string();
                    if i < n {
                        i += 1; // skip closing quote
                    }
                    map.insert(name, value);
                } else {
                    // unquoted value
                    let val_start = i;
                    while i < n && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
                        i += 1;
                    }
                    let value = s[val_start..i].to_string();
                    map.insert(name, value);
                }
            } else {
                // Boolean attribute (no value)
                map.insert(name, String::new());
            }
        }
        map
    }

    // ========================================================================
    // Internal: Tag handlers
    // ========================================================================

    fn handle_opening_tag(&self, tag: &RawTag, state: &mut ConvState) {
        match tag.tag_name.as_str() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag.tag_name.as_bytes()[1] - b'0';
                if level <= self.max_heading_depth {
                    state.heading_level = level;
                    self.ensure_newline(state);
                    for _ in 0..level {
                        state.output.push('#');
                    }
                    state.output.push(' ');
                }
            }
            "a" if self.preserve_links => {
                let href = tag.attrs.get("href").cloned().unwrap_or_default();
                state.link_href = Some(href);
                state.output.push('[');
            }
            "img" if self.preserve_images => {
                let src = tag.attrs.get("src").cloned().unwrap_or_default();
                let alt = tag.attrs.get("alt").cloned().unwrap_or_default();
                state.output.push_str(&format!("![{}]({})", alt, src));
            }
            "strong" | "b" => {
                state.bold_depth += 1;
                state.output.push_str("**");
            }
            "em" | "i" => {
                state.italic_depth += 1;
                state.output.push('*');
            }
            "code" => {
                if !state.in_pre {
                    state.code_depth += 1;
                    state.output.push('`');
                }
            }
            "pre" => {
                state.in_pre = true;
                state.pre_buffer.clear();
            }
            "p" => {
                if !state.output.is_empty() && !state.output.ends_with('\n') {
                    state.output.push('\n');
                }
                if !state.output.ends_with('\n') {
                    state.output.push('\n');
                }
                state.in_paragraph = true;
            }
            "br" => {
                state.output.push('\n');
            }
            "ul" if self.preserve_lists => {
                state.list_stack.push(ListCtx {
                    list_type: ListType::Unordered,
                    counter: 0,
                });
            }
            "ol" if self.preserve_lists => {
                state.list_stack.push(ListCtx {
                    list_type: ListType::Ordered,
                    counter: 0,
                });
            }
            "li" if self.preserve_lists && !state.list_stack.is_empty() => {
                let indent = (state.list_stack.len() - 1) * 2;
                for _ in 0..indent {
                    state.output.push(' ');
                }
                match state.list_stack.last_mut() {
                    Some(ctx) if ctx.list_type == ListType::Ordered => {
                        ctx.counter += 1;
                        let _ = std::fmt::Write::write_fmt(
                            &mut state.output,
                            format_args!("{}. ", ctx.counter),
                        );
                    }
                    _ => {
                        state.output.push_str("- ");
                        if let Some(ctx) = state.list_stack.last_mut() {
                            if ctx.list_type == ListType::Unordered {
                                // mark that this list has started
                            }
                        }
                    }
                }
            }
            "blockquote" => {
                state.in_blockquote = true;
                self.ensure_newline(state);
                state.output.push_str("> ");
            }
            "table" if self.preserve_tables => {
                state.in_table = true;
                state.table_rows.clear();
                state.table_row.clear();
            }
            "tr" if state.in_table => {
                if !state.table_row.is_empty() {
                    let row = std::mem::take(&mut state.table_row);
                    state.table_rows.push(row);
                }
            }
            "th" | "td" if state.in_table => {
                state.table_cell.clear();
            }
            _ => {}
        }
    }

    fn handle_closing_tag(&self, tag: &RawTag, state: &mut ConvState) {
        match tag.tag_name.as_str() {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if state.heading_level > 0 {
                    if !state.output.ends_with('\n') {
                        state.output.push('\n');
                    }
                    state.output.push('\n');
                    state.heading_level = 0;
                }
            }
            "a" if self.preserve_links => {
                if let Some(href) = state.link_href.take() {
                    state.output.push_str(&format!("]({})", href));
                }
            }
            "strong" | "b" => {
                if state.bold_depth > 0 {
                    state.bold_depth -= 1;
                    state.output.push_str("**");
                }
            }
            "em" | "i" => {
                if state.italic_depth > 0 {
                    state.italic_depth -= 1;
                    state.output.push('*');
                }
            }
            "code" => {
                if state.code_depth > 0 {
                    state.code_depth -= 1;
                    state.output.push('`');
                }
            }
            "pre" => {
                if state.in_pre {
                    self.emit_pre_block(state);
                    state.in_pre = false;
                }
            }
            "p" => {
                if state.in_paragraph {
                    if !state.output.ends_with('\n') {
                        state.output.push('\n');
                    }
                    state.output.push('\n');
                    state.in_paragraph = false;
                }
            }
            "ul" | "ol" if self.preserve_lists => {
                state.list_stack.pop();
                if !state.output.ends_with('\n') {
                    state.output.push('\n');
                }
            }
            "li" if self.preserve_lists => {
                if !state.output.ends_with('\n') {
                    state.output.push('\n');
                }
            }
            "blockquote" => {
                state.in_blockquote = false;
                if !state.output.ends_with('\n') {
                    state.output.push('\n');
                }
                state.output.push('\n');
            }
            "table" if self.preserve_tables => {
                if !state.table_row.is_empty() {
                    let row = std::mem::take(&mut state.table_row);
                    state.table_rows.push(row);
                }
                if !state.table_rows.is_empty() {
                    self.emit_table(state);
                }
                state.in_table = false;
                state.table_rows.clear();
            }
            "tr" if state.in_table => {
                if !state.table_row.is_empty() {
                    let row = std::mem::take(&mut state.table_row);
                    state.table_rows.push(row);
                }
            }
            "th" | "td" if state.in_table => {
                let cell = state.table_cell.drain(..).collect::<String>();
                state.table_row.push(cell);
            }
            _ => {}
        }
    }

    fn handle_self_closing_tag(&self, tag: &RawTag, state: &mut ConvState) {
        match tag.tag_name.as_str() {
            "br" => {
                state.output.push('\n');
            }
            "hr" => {
                self.ensure_newline(state);
                state.output.push_str("---\n\n");
            }
            "img" if self.preserve_images => {
                let src = tag.attrs.get("src").cloned().unwrap_or_default();
                let alt = tag.attrs.get("alt").cloned().unwrap_or_default();
                state.output.push_str(&format!("![{}]({})", alt, src));
            }
            "input" | "meta" | "link" => {
                // skip — no markdown equivalent
            }
            _ => {}
        }
    }

    // ========================================================================
    // Internal: Output helpers
    // ========================================================================

    fn ensure_newline(&self, state: &mut ConvState) {
        if !state.output.is_empty() && !state.output.ends_with('\n') {
            state.output.push('\n');
        }
    }

    fn emit_pre_block(&self, state: &mut ConvState) {
        let code = std::mem::take(&mut state.pre_buffer);
        if !state.output.is_empty() && !state.output.ends_with('\n') {
            state.output.push('\n');
        }
        state.output.push_str("```\n");
        state.output.push_str(&code);
        if !code.ends_with('\n') {
            state.output.push('\n');
        }
        state.output.push_str("```\n\n");
    }

    fn emit_table(&self, state: &mut ConvState) {
        let rows = std::mem::take(&mut state.table_rows);
        if rows.is_empty() {
            return;
        }
        let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        if max_cols == 0 {
            return;
        }

        // Check if the first row looks like headers (has <th> cells)
        // We determine this by checking if we ever saw <th> — we store it as a flag
        let has_headers = state.table_has_headers;

        for (idx, row) in rows.iter().enumerate() {
            if !state.output.is_empty() && !state.output.ends_with('\n') {
                state.output.push('\n');
            }
            state.output.push('|');
            for cell in row.iter().take(max_cols) {
                state.output.push(' ');
                state.output.push_str(cell.trim());
                state.output.push_str(" |");
            }
            state.output.push('\n');
            if idx == 0 && has_headers {
                state.output.push('|');
                for _ in 0..max_cols {
                    state.output.push_str(" --- |");
                }
                state.output.push('\n');
            }
        }

        // Also emit a separator after first row if no header row
        if !has_headers && rows.len() >= 2 {
            // Insert separator before second row
            // This is tricky since we already built the output. Let's insert it.
            // Find the position after the first row's newline
            // Actually, we need to track this. Let's just do a simpler approach:
            // re-emit with separator inserted
            state.output.clear();
            for (idx, row) in rows.iter().enumerate() {
                state.output.push('|');
                for cell in row.iter().take(max_cols) {
                    state.output.push(' ');
                    state.output.push_str(cell.trim());
                    state.output.push_str(" |");
                }
                state.output.push('\n');
                if idx == 0 {
                    state.output.push('|');
                    for _ in 0..max_cols {
                        state.output.push_str(" --- |");
                    }
                    state.output.push('\n');
                }
            }
        }

        state.output.push('\n');
        state.table_has_headers = false;
    }

    /// Infer block type from a ContentBlock's heading and body
    fn infer_block_type(&self, block: &ContentBlock) -> BlockType {
        if block.heading.is_some() && block.body.len() < 80 && !block.body.contains('\n') {
            return BlockType::Heading;
        }
        let body_lower = block.body.to_lowercase();
        if block.body.starts_with("```")
            || block.body.starts_with('\t')
            || block.body.starts_with("    ")
        {
            return BlockType::Code;
        }
        if body_lower.contains("<table>") || body_lower.contains('|') {
            return BlockType::Table;
        }
        if block.body.starts_with('>') {
            return BlockType::Blockquote;
        }
        // Check for list-like content
        if block.body.lines().any(|l| {
            l.trim().starts_with('-')
                || l.trim().starts_with('*')
                || l.trim()
                    .starts_with(|c: char| c.is_ascii_digit() && l.trim().contains(". "))
        }) {
            return BlockType::List;
        }
        BlockType::Paragraph
    }
}

impl Default for HtmlToMarkdown {
    fn default() -> Self {
        Self::new()
    }
}

impl ToMarkdown for HtmlToMarkdown {
    fn to_markdown(&self) -> Result<String, String> {
        Err(
            "HtmlToMarkdown requires an input — use from_html() or from_extracted_content()"
                .to_string(),
        )
    }

    fn source_type(&self) -> &'static str {
        "html/markdown"
    }
}

// ============================================================================
// Internal types for HTML parsing
// ============================================================================

#[derive(Debug)]
#[allow(dead_code)]
struct RawTag {
    tag_name: String,
    attrs: HashMap<String, String>,
    #[allow(dead_code)]
    start: usize,
    end: usize,
    is_self_closing: bool,
    is_closing: bool,
    is_comment: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListType {
    Unordered,
    Ordered,
}

#[derive(Debug, Clone)]
struct ListCtx {
    list_type: ListType,
    counter: u32,
}

/// Internal state for the HTML→Markdown converter
struct ConvState<'a> {
    #[allow(dead_code)]
    converter: &'a HtmlToMarkdown,
    output: String,
    link_href: Option<String>,
    bold_depth: usize,
    italic_depth: usize,
    code_depth: usize,
    heading_level: u8,
    in_pre: bool,
    pre_buffer: String,
    in_blockquote: bool,
    list_stack: Vec<ListCtx>,
    in_paragraph: bool,
    in_table: bool,
    table_rows: Vec<Vec<String>>,
    table_row: Vec<String>,
    table_cell: String,
    table_has_headers: bool,
}

impl<'a> ConvState<'a> {
    fn new(converter: &'a HtmlToMarkdown) -> Self {
        Self {
            converter,
            output: String::new(),
            link_href: None,
            bold_depth: 0,
            italic_depth: 0,
            code_depth: 0,
            heading_level: 0,
            in_pre: false,
            pre_buffer: String::new(),
            in_blockquote: false,
            list_stack: Vec::new(),
            in_paragraph: false,
            in_table: false,
            table_rows: Vec::new(),
            table_row: Vec::new(),
            table_cell: String::new(),
            table_has_headers: false,
        }
    }
}

// ============================================================================
// Part 2: ExtractedContentToMarkdown
// ============================================================================

/// Converts ExtractedContent to Markdown with optional YAML frontmatter.
pub struct ExtractedContentToMarkdown {
    pub include_metadata: bool,
    pub include_raw_text: bool,
}

impl ExtractedContentToMarkdown {
    pub fn new() -> Self {
        Self {
            include_metadata: true,
            include_raw_text: false,
        }
    }

    /// Convert ExtractedContent to a Markdown string.
    ///
    /// * If `include_metadata` is true, metadata is rendered as YAML frontmatter.
    /// * Blocks are converted one by one.
    /// * If `include_raw_text` is true, raw_text is appended as a code block.
    pub fn convert(&self, content: &ExtractedContent) -> String {
        let mut out = String::new();

        if self.include_metadata && !content.metadata.is_empty() {
            out.push_str("---\n");
            let mut keys: Vec<&String> = content.metadata.keys().collect();
            keys.sort();
            for key in keys {
                if let Some(val) = content.metadata.get(key) {
                    let escaped = val.replace('\"', "\\\"");
                    if val.contains('\n') || val.contains(':') || val.contains('#') {
                        out.push_str(&format!("{}: \"{}\"\n", key, escaped));
                    } else {
                        out.push_str(&format!("{}: {}\n", key, val));
                    }
                }
            }
            out.push_str("---\n\n");
        }

        for block in &content.blocks {
            if let Some(ref heading) = block.heading {
                out.push_str(&format!("## {}\n\n", heading));
            }
            out.push_str(&block.body);
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push('\n');
        }

        if self.include_raw_text && !content.raw_text.is_empty() {
            out.push_str("---\n\n");
            out.push_str("```text\n");
            out.push_str(&content.raw_text);
            if !content.raw_text.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("```\n");
        }

        out
    }
}

impl Default for ExtractedContentToMarkdown {
    fn default() -> Self {
        Self::new()
    }
}

impl ToMarkdown for ExtractedContentToMarkdown {
    fn to_markdown(&self) -> Result<String, String> {
        Err("ExtractedContentToMarkdown requires input — use convert()".to_string())
    }

    fn source_type(&self) -> &'static str {
        "extracted-content/markdown"
    }
}

// ============================================================================
// Part 3: MarkdownPipelineBuilder
// ============================================================================

/// Builder for configuring a Markdown conversion pipeline.
///
/// Chainable methods configure both `HtmlToMarkdown` and `ExtractedContentToMarkdown`.
/// Call `build()` to get the configured converters.
pub struct MarkdownPipelineBuilder {
    html_converter: HtmlToMarkdown,
    content_converter: ExtractedContentToMarkdown,
}

impl MarkdownPipelineBuilder {
    pub fn new() -> Self {
        Self {
            html_converter: HtmlToMarkdown::new(),
            content_converter: ExtractedContentToMarkdown::new(),
        }
    }

    pub fn with_preserve_tables(mut self, v: bool) -> Self {
        self.html_converter.preserve_tables = v;
        self
    }

    pub fn with_preserve_code_blocks(mut self, v: bool) -> Self {
        self.html_converter.preserve_code_blocks = v;
        self
    }

    pub fn with_preserve_links(mut self, v: bool) -> Self {
        self.html_converter.preserve_links = v;
        self
    }

    pub fn with_preserve_images(mut self, v: bool) -> Self {
        self.html_converter.preserve_images = v;
        self
    }

    pub fn with_preserve_lists(mut self, v: bool) -> Self {
        self.html_converter.preserve_lists = v;
        self
    }

    pub fn with_max_heading_depth(mut self, d: u8) -> Self {
        self.html_converter.max_heading_depth = d;
        self
    }

    pub fn with_line_width(mut self, w: usize) -> Self {
        self.html_converter.line_width = w;
        self
    }

    pub fn with_include_metadata(mut self, v: bool) -> Self {
        self.content_converter.include_metadata = v;
        self
    }

    pub fn with_include_raw_text(mut self, v: bool) -> Self {
        self.content_converter.include_raw_text = v;
        self
    }

    /// Build both converters. Returns `(HtmlToMarkdown, ExtractedContentToMarkdown)`.
    pub fn build(self) -> (HtmlToMarkdown, ExtractedContentToMarkdown) {
        (self.html_converter, self.content_converter)
    }
}

impl Default for MarkdownPipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // HtmlToMarkdown tests
    // ------------------------------------------------------------------

    #[test]
    fn test_h1_to_heading() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("<h1>Title</h1>").unwrap();
        assert!(result.starts_with("# "));
        assert!(result.contains("Title"));
    }

    #[test]
    fn test_h2_to_heading() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("<h2>Section</h2>").unwrap();
        assert!(result.starts_with("## "));
        assert!(result.contains("Section"));
    }

    #[test]
    fn test_h6_to_heading() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("<h6>Tiny</h6>").unwrap();
        assert!(result.starts_with("###### "));
        assert!(result.contains("Tiny"));
    }

    #[test]
    fn test_link_conversion() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html(r#"<a href="https://x.com">X</a>"#).unwrap();
        assert_eq!(result, "[X](https://x.com)");
    }

    #[test]
    fn test_link_without_href() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("<a>text</a>").unwrap();
        assert_eq!(result, "[text]()");
    }

    #[test]
    fn test_link_preserve_off() {
        let conv = HtmlToMarkdown {
            preserve_links: false,
            ..HtmlToMarkdown::new()
        };
        let result = conv.from_html(r#"<a href="https://x.com">X</a>"#).unwrap();
        assert_eq!(result, "X");
    }

    #[test]
    fn test_image_conversion() {
        let conv = HtmlToMarkdown::new();
        let result = conv
            .from_html(r#"<img src="pic.png" alt="Photo">"#)
            .unwrap();
        assert_eq!(result, "![Photo](pic.png)");
    }

    #[test]
    fn test_image_no_alt() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html(r#"<img src="pic.png">"#).unwrap();
        assert_eq!(result, "![](pic.png)");
    }

    #[test]
    fn test_code_inline() {
        let conv = HtmlToMarkdown::new();
        let result = conv
            .from_html("<p>Use <code>fn main()</code> here</p>")
            .unwrap();
        let normalized = result.trim();
        assert!(normalized.contains("`fn main()`"));
    }

    #[test]
    fn test_code_block() {
        let conv = HtmlToMarkdown::new();
        let result = conv
            .from_html("<pre><code>fn main() {\n    println!(\"hello\");\n}</code></pre>")
            .unwrap();
        assert!(result.contains("```"));
        assert!(result.contains("fn main()"));
        assert!(result.contains("println!"));
    }

    #[test]
    fn test_bold_conversion() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("<strong>bold text</strong>").unwrap();
        assert_eq!(result, "**bold text**");
    }

    #[test]
    fn test_bold_with_b_tag() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("<b>bold</b>").unwrap();
        assert_eq!(result, "**bold**");
    }

    #[test]
    fn test_italic_conversion() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("<em>italic text</em>").unwrap();
        assert_eq!(result, "*italic text*");
    }

    #[test]
    fn test_italic_with_i_tag() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("<i>italic</i>").unwrap();
        assert_eq!(result, "*italic*");
    }

    #[test]
    fn test_br_tag() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("line1<br>line2").unwrap();
        assert_eq!(result, "line1\nline2");
    }

    #[test]
    fn test_self_closing_br() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("line1<br/>line2").unwrap();
        assert_eq!(result, "line1\nline2");
    }

    #[test]
    fn test_blockquote() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("<blockquote>citation</blockquote>").unwrap();
        assert!(result.contains("> citation") || result.contains(">citation"));
    }

    #[test]
    fn test_table_conversion() {
        let conv = HtmlToMarkdown::new();
        let html =
            "<table><tr><th>Name</th><th>Age</th></tr><tr><td>Alice</td><td>30</td></tr></table>";
        let result = conv.from_html(html).unwrap();
        assert!(result.contains('|'));
        assert!(result.contains("Name"));
        assert!(result.contains("Age"));
        assert!(result.contains("Alice"));
        assert!(result.contains("30"));
    }

    #[test]
    fn test_table_multiple_rows() {
        let conv = HtmlToMarkdown::new();
        let html = "<table><tr><th>Name</th><th>Age</th></tr><tr><td>Alice</td><td>30</td></tr><tr><td>Bob</td><td>25</td></tr></table>";
        let result = conv.from_html(html).unwrap();
        assert!(result.contains("Alice"));
        assert!(result.contains("Bob"));
        assert!(result.contains("---"));
    }

    #[test]
    fn test_table_preserve_off() {
        let conv = HtmlToMarkdown {
            preserve_tables: false,
            ..HtmlToMarkdown::new()
        };
        let html = "<table><tr><td>data</td></tr></table>";
        let result = conv.from_html(html).unwrap();
        // Should not produce pipe table, just raw text
        assert!(!result.contains('|'));
    }

    #[test]
    fn test_unordered_list() {
        let conv = HtmlToMarkdown::new();
        let html = "<ul><li>Item A</li><li>Item B</li></ul>";
        let result = conv.from_html(html).unwrap();
        assert!(result.contains("- Item A"));
        assert!(result.contains("- Item B"));
    }

    #[test]
    fn test_ordered_list() {
        let conv = HtmlToMarkdown::new();
        let html = "<ol><li>First</li><li>Second</li></ol>";
        let result = conv.from_html(html).unwrap();
        assert!(result.contains("1. First"));
        assert!(result.contains("2. Second"));
    }

    #[test]
    fn test_nested_list() {
        let conv = HtmlToMarkdown::new();
        let html = "<ul><li>Item 1<ul><li>Nested</li></ul></li><li>Item 2</li></ul>";
        let result = conv.from_html(html).unwrap();
        assert!(result.contains("- Item 1"));
        assert!(result.contains("  - Nested"));
        assert!(result.contains("- Item 2"));
    }

    #[test]
    fn test_inline_formatting() {
        let conv = HtmlToMarkdown::new();
        let result = conv
            .from_html("<p><strong>bold</strong> and <em>italic</em></p>")
            .unwrap();
        let trimmed = result.trim();
        assert!(trimmed.contains("**bold**"));
        assert!(trimmed.contains("*italic*"));
    }

    #[test]
    fn test_nested_formatting() {
        let conv = HtmlToMarkdown::new();
        let result = conv
            .from_html("<strong><em>bold italic</em></strong>")
            .unwrap();
        assert_eq!(result, "***bold italic***");
    }

    #[test]
    fn test_link_with_bold() {
        let conv = HtmlToMarkdown::new();
        let html = r#"<a href="https://x.com"><strong>X</strong></a>"#;
        let result = conv.from_html(html).unwrap();
        assert_eq!(result, "[**X**](https://x.com)");
    }

    #[test]
    fn test_complex_html() {
        let conv = HtmlToMarkdown::new();
        let html = "<h1>Title</h1><p>Paragraph with <a href=\"https://x.com\">a link</a> and <strong>bold</strong>.</p><ul><li>A</li><li>B</li></ul>";
        let result = conv.from_html(html).unwrap();
        assert!(result.starts_with("# "));
        assert!(result.contains("[a link](https://x.com)"));
        assert!(result.contains("**bold**"));
        assert!(result.contains("- A"));
        assert!(result.contains("- B"));
    }

    #[test]
    fn test_hr_tag() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("<hr>").unwrap();
        assert!(result.contains("---"));
    }

    #[test]
    fn test_empty_input() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_plain_text_no_html() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("just plain text").unwrap();
        assert_eq!(result, "just plain text");
    }

    #[test]
    fn test_escape_markdown_chars() {
        assert_eq!(HtmlToMarkdown::escape_markdown(r"\"), r"\\");
        assert_eq!(HtmlToMarkdown::escape_markdown("*"), r"\*");
        assert_eq!(HtmlToMarkdown::escape_markdown("_"), r"\_");
        assert_eq!(HtmlToMarkdown::escape_markdown("["), r"\[");
        assert_eq!(HtmlToMarkdown::escape_markdown("]"), r"\]");
        assert_eq!(HtmlToMarkdown::escape_markdown("("), r"\(");
        assert_eq!(HtmlToMarkdown::escape_markdown(")"), r"\)");
        assert_eq!(HtmlToMarkdown::escape_markdown("#"), r"\#");
        assert_eq!(HtmlToMarkdown::escape_markdown("+"), r"\+");
        assert_eq!(HtmlToMarkdown::escape_markdown("-"), r"\-");
        assert_eq!(HtmlToMarkdown::escape_markdown("!"), r"\!");
        assert_eq!(
            HtmlToMarkdown::escape_markdown("normal text"),
            "normal text"
        );
    }

    #[test]
    fn test_escape_markdown_combined() {
        let result = HtmlToMarkdown::escape_markdown("Hello *world* [test] (yes) #1 +2 -3 !4");
        assert_eq!(result, r"Hello \*world\* \[test\] \(yes\) \#1 \+2 \-3 \!4");
    }

    #[test]
    fn test_heading_max_depth() {
        let conv = HtmlToMarkdown {
            max_heading_depth: 3,
            ..HtmlToMarkdown::new()
        };
        let h4_result = conv.from_html("<h4>deep</h4>").unwrap();
        // h4 shouldn't produce heading because max_heading_depth=3
        assert!(!h4_result.contains("####"));
        let h3_result = conv.from_html("<h3>ok</h3>").unwrap();
        assert!(h3_result.contains("###"));
    }

    #[test]
    fn test_images_preserve_off() {
        let conv = HtmlToMarkdown {
            preserve_images: false,
            ..HtmlToMarkdown::new()
        };
        let result = conv
            .from_html(r#"<img src="pic.png" alt="Photo">"#)
            .unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_lists_preserve_off() {
        let conv = HtmlToMarkdown {
            preserve_lists: false,
            ..HtmlToMarkdown::new()
        };
        let html = "<ul><li>Item</li></ul>";
        let result = conv.from_html(html).unwrap();
        // List tags should be stripped, text preserved
        assert_eq!(result.trim(), "Item");
    }

    #[test]
    fn test_strip_unknown_tag() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("<unknown>text</unknown>").unwrap();
        assert_eq!(result, "text");
    }

    #[test]
    fn test_paragraph_spacing() {
        let conv = HtmlToMarkdown::new();
        let html = "<p>first</p><p>second</p>";
        let result = conv.from_html(html).unwrap();
        assert!(result.contains("first"));
        assert!(result.contains("second"));
    }

    #[test]
    fn test_html_comment_stripped() {
        let conv = HtmlToMarkdown::new();
        let html = "before<!-- comment -->after";
        let result = conv.from_html(html).unwrap();
        assert_eq!(result, "beforeafter");
    }

    #[test]
    fn test_void_element_br() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("line1<br>line2").unwrap();
        assert_eq!(result, "line1\nline2");
    }

    #[test]
    fn test_mixed_content() {
        let conv = HtmlToMarkdown::new();
        let html = "<div><h1>Title</h1><p>Body with <code>code</code>.</p></div>";
        let result = conv.from_html(html).unwrap();
        assert!(result.contains("# Title"));
        assert!(result.contains("`code`"));
    }

    #[test]
    fn test_element_with_attrs() {
        let conv = HtmlToMarkdown::new();
        let html = r#"<strong class="highlight" id="h1">text</strong>"#;
        let result = conv.from_html(html).unwrap();
        assert_eq!(result, "**text**");
    }

    // ------------------------------------------------------------------
    // ExtractedContentToMarkdown tests
    // ------------------------------------------------------------------

    #[test]
    fn test_content_converter_empty() {
        let conv = ExtractedContentToMarkdown::new();
        let content = ExtractedContent {
            title: String::new(),
            description: String::new(),
            metadata: HashMap::new(),
            blocks: vec![],
            raw_text: String::new(),
            text_length: 0,
            tier: crate::neotrix::nt_mind::content_extractor::ExtractionTier::PlainStrip,
            quality: 0.0,
            has_list_data: false,
            has_table_data: false,
            link_count: 0,
        };
        let result = conv.convert(&content);
        assert_eq!(result, "");
    }

    #[test]
    fn test_content_converter_with_metadata() {
        let conv = ExtractedContentToMarkdown::new();
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Test".to_string());
        metadata.insert("author".to_string(), "NeoTrix".to_string());
        let content = ExtractedContent {
            title: "Test".to_string(),
            description: String::new(),
            metadata,
            blocks: vec![ContentBlock {
                heading: None,
                body: "Body text".to_string(),
            }],
            raw_text: String::new(),
            text_length: 9,
            tier: crate::neotrix::nt_mind::content_extractor::ExtractionTier::SemanticHtml5,
            quality: 1.0,
            has_list_data: false,
            has_table_data: false,
            link_count: 0,
        };
        let result = conv.convert(&content);
        assert!(result.starts_with("---"));
        assert!(result.contains("title: Test"));
        assert!(result.contains("author: NeoTrix"));
        assert!(result.contains("Body text"));
    }

    #[test]
    fn test_content_converter_no_metadata() {
        let conv = ExtractedContentToMarkdown {
            include_metadata: false,
            ..ExtractedContentToMarkdown::new()
        };
        let content = ExtractedContent {
            title: String::new(),
            description: String::new(),
            metadata: HashMap::new(),
            blocks: vec![ContentBlock {
                heading: Some("Section".to_string()),
                body: "Content".to_string(),
            }],
            raw_text: String::new(),
            text_length: 7,
            tier: crate::neotrix::nt_mind::content_extractor::ExtractionTier::SemanticHtml5,
            quality: 1.0,
            has_list_data: false,
            has_table_data: false,
            link_count: 0,
        };
        let result = conv.convert(&content);
        assert!(!result.starts_with("---"));
        assert!(result.contains("## Section"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_content_converter_raw_text() {
        let conv = ExtractedContentToMarkdown {
            include_raw_text: true,
            ..ExtractedContentToMarkdown::new()
        };
        let content = ExtractedContent {
            title: String::new(),
            description: String::new(),
            metadata: HashMap::new(),
            blocks: vec![],
            raw_text: "raw data".to_string(),
            text_length: 9,
            tier: crate::neotrix::nt_mind::content_extractor::ExtractionTier::PlainStrip,
            quality: 0.5,
            has_list_data: false,
            has_table_data: false,
            link_count: 0,
        };
        let result = conv.convert(&content);
        assert!(result.contains("```text"));
        assert!(result.contains("raw data"));
    }

    // ------------------------------------------------------------------
    // HtmlToMarkdown::from_extracted_content tests
    // ------------------------------------------------------------------

    #[test]
    fn test_html_converter_from_extracted() {
        let conv = HtmlToMarkdown::new();
        let content = ExtractedContent {
            title: String::new(),
            description: String::new(),
            metadata: HashMap::new(),
            blocks: vec![
                ContentBlock {
                    heading: Some("Title".to_string()),
                    body: "Short body".to_string(),
                },
                ContentBlock {
                    heading: None,
                    body: "Longer paragraph text here for testing purposes.".to_string(),
                },
            ],
            raw_text: "raw".to_string(),
            text_length: 0,
            tier: crate::neotrix::nt_mind::content_extractor::ExtractionTier::SemanticHtml5,
            quality: 1.0,
            has_list_data: false,
            has_table_data: false,
            link_count: 0,
        };
        let result = conv.from_extracted_content(&content).unwrap();
        assert!(result.contains("## Title"));
        assert!(result.contains("Longer paragraph text"));
    }

    // ------------------------------------------------------------------
    // Builder tests
    // ------------------------------------------------------------------

    #[test]
    fn test_builder_default() {
        let builder = MarkdownPipelineBuilder::new();
        let (html, content) = builder.build();
        assert!(html.preserve_tables);
        assert!(html.preserve_links);
        assert!(content.include_metadata);
    }

    #[test]
    fn test_builder_chain() {
        let builder = MarkdownPipelineBuilder::new()
            .with_preserve_tables(false)
            .with_preserve_links(false)
            .with_preserve_images(false)
            .with_preserve_lists(false)
            .with_preserve_code_blocks(false)
            .with_max_heading_depth(3)
            .with_line_width(120)
            .with_include_metadata(false)
            .with_include_raw_text(true);
        let (html, content) = builder.build();
        assert!(!html.preserve_tables);
        assert!(!html.preserve_links);
        assert!(!html.preserve_images);
        assert!(!html.preserve_lists);
        assert!(!html.preserve_code_blocks);
        assert_eq!(html.max_heading_depth, 3);
        assert_eq!(html.line_width, 120);
        assert!(!content.include_metadata);
        assert!(content.include_raw_text);
    }

    // ------------------------------------------------------------------
    // ToMarkdown trait tests
    // ------------------------------------------------------------------

    #[test]
    fn test_html_trait_no_input() {
        let conv = HtmlToMarkdown::new();
        assert!(conv.to_markdown().is_err());
        assert_eq!(conv.source_type(), "html/markdown");
    }

    #[test]
    fn test_content_trait_no_input() {
        let conv = ExtractedContentToMarkdown::new();
        assert!(conv.to_markdown().is_err());
        assert_eq!(conv.source_type(), "extracted-content/markdown");
    }

    // ------------------------------------------------------------------
    // Error handling / edge cases
    // ------------------------------------------------------------------

    #[test]
    fn test_unclosed_tag() {
        let conv = HtmlToMarkdown::new();
        // An unclosed <div> should be tolerated — just strip
        let result = conv.from_html("<div>unclosed").unwrap();
        assert_eq!(result, "unclosed");
    }

    #[test]
    fn test_whitespace_only() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("   \n  \t  ").unwrap();
        assert_eq!(result.trim(), "");
    }

    #[test]
    fn test_multiple_links() {
        let conv = HtmlToMarkdown::new();
        let html = r#"<a href="https://a.com">A</a> and <a href="https://b.com">B</a>"#;
        let result = conv.from_html(html).unwrap();
        assert_eq!(result, "[A](https://a.com) and [B](https://b.com)");
    }

    #[test]
    fn test_bold_with_multiple_words() {
        let conv = HtmlToMarkdown::new();
        let result = conv.from_html("<b>first second third</b>").unwrap();
        assert_eq!(result, "**first second third**");
    }

    #[test]
    fn test_code_block_preserve_off() {
        let conv = HtmlToMarkdown {
            preserve_code_blocks: false,
            ..HtmlToMarkdown::new()
        };
        let result = conv.from_html("<pre><code>code</code></pre>").unwrap();
        assert!(!result.contains("```"));
        assert!(result.contains("code"));
    }

    #[test]
    fn test_all_headings() {
        let conv = HtmlToMarkdown::new();
        for level in 1..=6 {
            let html = format!("<h{level}>H{level}</h{level}>");
            let result = conv.from_html(&html).unwrap();
            let prefix = "#".repeat(level as usize);
            assert!(
                result.starts_with(&prefix),
                "h{level} should start with {prefix}"
            );
        }
    }

    #[test]
    fn test_comment_with_embedded_gt() {
        let conv = HtmlToMarkdown::new();
        // HTML comment with > inside
        let html = "text<!-- a > b -->more";
        let result = conv.from_html(html).unwrap();
        assert_eq!(result, "textmore");
    }

    #[test]
    fn test_trait_source_types() {
        let html_conv = HtmlToMarkdown::new();
        let content_conv = ExtractedContentToMarkdown::new();
        assert_eq!(html_conv.source_type(), "html/markdown");
        assert_eq!(content_conv.source_type(), "extracted-content/markdown");
    }
}
