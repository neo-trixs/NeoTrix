//! nt_world_dom_agent — Flat DOM tree for AI agent page interaction.
//!
//! Compresses interactive DOM elements into a flat text map with stable indices,
//! inspired by page-agent (23.9k⭐) FlatDomTree. Self-contained, no external deps.
//!
//! # Design
//! - `compress_dom(html)` → `FlatDomTree` — flat text map, element index → tag/text/attrs
//! - `find_interactive(tree)` → `&[DomElement]` — buttons, links, inputs, selects, textareas
//! - Stable indices: index = elemidx counter in document order, deterministic between parses

use std::collections::HashMap;

/// A single DOM element in the flat tree.
#[derive(Debug, Clone)]
pub struct DomElement {
    /// Stable index (document-order counter, deterministic for same HTML)
    pub index: usize,
    /// HTML tag name (lowercase)
    pub tag: String,
    /// Inner text content (HTML-decoded, whitespace-normalised)
    pub text: String,
    /// Key attributes (id, class, name, type, href, value, placeholder, role, etc.)
    pub attributes: HashMap<String, String>,
    /// True if this is an interactive element
    pub interactive: bool,
}

/// Flat DOM tree: a flat array of interactive + structural elements with stable indices.
#[derive(Debug, Clone)]
pub struct FlatDomTree {
    /// All interactive elements in document order
    pub elements: Vec<DomElement>,
    /// Total interactive element count
    pub count: usize,
}

impl FlatDomTree {
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Look up an element by its stable index.
    pub fn get(&self, index: usize) -> Option<&DomElement> {
        self.elements.iter().find(|e| e.index == index)
    }

    /// Produce a flat text map à la page-agent: `[idx]<tag> "text" [key=val ...]`
    pub fn to_text_map(&self) -> String {
        let mut lines: Vec<String> = Vec::with_capacity(self.elements.len());
        for elem in &self.elements {
            let attrs: String = if elem.attributes.is_empty() {
                String::new()
            } else {
                let mut pairs: Vec<String> = Vec::with_capacity(elem.attributes.len());
                for (k, v) in &elem.attributes {
                    if k == "id" || k == "class" || k == "type" || k == "name" || k == "href" || k == "role" || k == "placeholder" || k == "value" {
                        if !v.is_empty() {
                            pairs.push(format!("{}=\"{}\"", k, v));
                        } else {
                            pairs.push(k.clone());
                        }
                    }
                }
                if pairs.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", pairs.join(" "))
                }
            };
            let text = if elem.text.is_empty() {
                String::new()
            } else {
                let truncated: String = elem.text.chars().take(80).collect();
                format!(" \"{}\"", truncated.replace('"', "'"))
            };
            lines.push(format!("[{}]<{}>{}{}", elem.index, elem.tag, text, attrs));
        }
        lines.join("\n")
    }
}

/// Interactive DOM element tags recognised by the agent.
const INTERACTIVE_TAGS: &[&str] = &[
    "a", "button", "input", "select", "textarea",
    "label", "option", "details", "summary", "menuitem",
];

/// HTML tags whose content we skip entirely (script, style, etc.).
const SKIP_TAGS: &[&str] = &["script", "style", "meta", "link", "noscript", "head", "svg"];

// ---------------------------------------------------------------------------
// HTML scanner — lightweight, tag-based, no external parser
// ---------------------------------------------------------------------------

/// Lightweight HTML scanner used internally by `DomAgent`.
struct HtmlScanner<'a> {
    html: &'a str,
    bytes: &'a [u8],
    pos: usize,
    len: usize,
}

impl<'a> HtmlScanner<'a> {
    fn new(html: &'a str) -> Self {
        Self { bytes: html.as_bytes(), html, pos: 0, len: html.len() }
    }

    fn done(&self) -> bool {
        self.pos >= self.len
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn remaining(&self) -> &'a str {
        &self.html[self.pos..]
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.len && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    /// Find the next occurrence of byte `b` from current position.
    fn find_byte(&self, b: u8, start: usize) -> Option<usize> {
        self.html[start..].find(b as char).map(|i| start + i)
    }

    /// Find the next occurrence of string `s` from current position.
    fn find_str(&self, s: &str, start: usize) -> Option<usize> {
        self.html[start..].find(s).map(|i| start + i)
    }
}

// ---------------------------------------------------------------------------
// DomAgent
// ---------------------------------------------------------------------------

/// The DOM agent — converts HTML into a flat interactive-element text map.
pub struct DomAgent;

impl DomAgent {
    pub fn new() -> Self {
        Self
    }

    /// Compress HTML into a `FlatDomTree` of interactive elements.
    ///
    /// Non-interactive elements (divs, spans, paragraphs, headers, images, etc.)
    /// are **not** included in the output. Only buttons, links, inputs, selects,
    /// textareas, and role-mapped interactive elements appear.
    pub fn compress_dom(&self, html: &str) -> FlatDomTree {
        let mut scanner = HtmlScanner::new(html);
        let mut elements: Vec<DomElement> = Vec::new();
        let mut idx: usize = 0;

        loop {
            // Scan for next '<'
            while !scanner.done() && scanner.peek() != Some(b'<') {
                scanner.advance();
            }
            if scanner.done() {
                break;
            }
            // We're at '<'
            scanner.advance();

            // Skip comment: `<!-- ... -->`
            if scanner.remaining().starts_with("!--") {
                if let Some(end) = scanner.find_str("-->", scanner.pos) {
                    scanner.pos = end + 3;
                    continue;
                }
                break;
            }

            // Skip closing tag: `</tagname>`
            if scanner.peek() == Some(b'/') {
                scanner.advance();
                if let Some(end) = scanner.find_byte(b'>', scanner.pos) {
                    scanner.pos = end + 1;
                    continue;
                }
                break;
            }

            // Parse tag name
            let tag_start = scanner.pos;
            let tag_end = tag_start
                + scanner.html[tag_start..]
                    .find(|c: char| c == ' ' || c == '>' || c == '/' || c == '\t' || c == '\n' || c == '\r')
                    .unwrap_or(0);
            if tag_end <= tag_start {
                continue;
            }
            let tag: String = scanner.html[tag_start..tag_end].to_lowercase();
            scanner.pos = tag_end;

            // Check if we should skip this tag's subtree
            if SKIP_TAGS.contains(&tag.as_str()) {
                let close_tag = format!("</{}", tag);
                if let Some(end) = scanner.find_str(&close_tag, scanner.pos) {
                    let after_close = scanner.html[end..].find('>');
                    if let Some(ce) = after_close {
                        scanner.pos = end + ce + 1;
                        continue;
                    }
                }
                // Self-closing fallback
                if let Some(end) = scanner.find_byte(b'>', scanner.pos) {
                    scanner.pos = end + 1;
                    continue;
                }
                break;
            }

            // Parse attributes until '>' or '/>'
            let mut attrs: HashMap<String, String> = HashMap::new();
            let mut self_closing = false;
            let mut closed = false;

            loop {
                scanner.skip_whitespace();
                if scanner.done() {
                    break;
                }
                match scanner.peek() {
                    Some(b'>') => {
                        scanner.advance();
                        closed = true;
                        break;
                    }
                    Some(b'/') if scanner.pos + 1 < scanner.len && scanner.bytes[scanner.pos + 1] == b'>' => {
                        self_closing = true;
                        scanner.pos += 2;
                        closed = true;
                        break;
                    }
                    _ => {}
                }

                // Attribute name
                let a_start = scanner.pos;
                let a_end = a_start
                    + scanner.html[a_start..]
                        .find(|c: char| c == '=' || c == ' ' || c == '>' || c == '/' || c == '\t' || c == '\n' || c == '\r')
                        .unwrap_or(0);
                if a_end <= a_start {
                    scanner.advance();
                    continue;
                }
                let attr_name: String = scanner.html[a_start..a_end].to_lowercase();
                scanner.pos = a_end;

                scanner.skip_whitespace();

                // Parse value if '='
                if scanner.peek() == Some(b'=') {
                    scanner.advance();
                    scanner.skip_whitespace();
                    if scanner.peek() == Some(b'"') || scanner.peek() == Some(b'\'') {
                        let quote = scanner.peek().unwrap_or(b'"');
                        scanner.advance();
                        let v_start = scanner.pos;
                        if let Some(v_end) = scanner.find_byte(quote, v_start) {
                            attrs.insert(attr_name, scanner.html[v_start..v_end].to_string());
                            scanner.pos = v_end + 1;
                        }
                    } else {
                        let v_start = scanner.pos;
                        let v_end = v_start
                            + scanner.html[v_start..]
                                .find(|c: char| c == ' ' || c == '>' || c == '/' || c == '\t' || c == '\n' || c == '\r')
                                .unwrap_or(0);
                        if v_end > v_start {
                            attrs.insert(attr_name, scanner.html[v_start..v_end].to_string());
                            scanner.pos = v_end;
                        }
                    }
                } else {
                    attrs.insert(attr_name, String::new());
                }
            }

            if !closed {
                break;
            }

            // Determine if this element is interactive
            // Exclude <input type="hidden"> even though "input" is in INTERACTIVE_TAGS
            let is_interactive = DomAgent::is_interactive(&tag, &attrs)
                && !(tag == "input" && attrs.get("type").map(|v| v == "hidden").unwrap_or(false));

            // Non-interactive structural tags: continue scanning — their children
            // may contain interactive elements.

            if is_interactive {
                // Extract inner text
                let text = if self_closing {
                    String::new()
                } else {
                    let inner_start = scanner.pos;
                    let close_tag = format!("</{}", tag);
                    if let Some(close_pos) = scanner.find_str(&close_tag, inner_start) {
                        let raw = &scanner.html[inner_start..close_pos];
                        DomAgent::text_content(raw)
                    } else {
                        String::new()
                    }
                };

                let elem = DomElement {
                    index: idx,
                    tag: tag.clone(),
                    text,
                    attributes: attrs,
                    interactive: true,
                };
                elements.push(elem);
                idx += 1;
            }
        }

        FlatDomTree { count: idx, elements }
    }

    /// Check whether a given tag+attrs combo is interactive.
    fn is_interactive(tag: &str, attrs: &HashMap<String, String>) -> bool {
        if INTERACTIVE_TAGS.contains(&tag) {
            return true;
        }
        if let Some(role) = attrs.get("role") {
            if role == "button" || role == "link" || role == "option" || role == "tab" || role == "menuitem" || role == "combobox" {
                return true;
            }
        }
        if attrs.get("contenteditable").map(|v| v == "true").unwrap_or(false) {
            return true;
        }
        if let Some(ti) = attrs.get("tabindex") {
            if ti.parse::<i32>().ok().unwrap_or(-1) >= 0 {
                return true;
            }
        }
        if let Some(ty) = attrs.get("type") {
            if ty == "checkbox" || ty == "radio" || ty == "submit" || ty == "button" || ty == "range" || ty == "file" {
                return true;
            }
        }
        false
    }

    /// Strip HTML tags and decode basic entities from a raw text slice.
    fn text_content(raw: &str) -> String {
        let mut out = String::with_capacity(raw.len());
        let bytes = raw.as_bytes();
        let mut i = 0;
        let len = raw.len();
        let mut in_tag = false;
        let mut prev_space = false;

        while i < len {
            match bytes[i] {
                b'<' => {
                    in_tag = true;
                    i += 1;
                }
                b'>' if in_tag => {
                    in_tag = false;
                    i += 1;
                }
                b'&' if !in_tag => {
                    // Decode HTML entities
                    let start = i;
                    i += 1;
                    while i < len && bytes[i] != b';' && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'#') {
                        i += 1;
                    }
                    if i < len && bytes[i] == b';' {
                        let entity = &raw[start + 1..i];
                        let ch = match entity {
                            "amp" => Some('&'),
                            "lt" => Some('<'),
                            "gt" => Some('>'),
                            "quot" => Some('"'),
                            "apos" => Some('\''),
                            "nbsp" => Some(' '),
                            _ => {
                                if let Some(num) = entity.strip_prefix('#') {
                                    if let Ok(cp) = if entity.starts_with("#x") || entity.starts_with("#X") {
                                        u32::from_str_radix(&entity[2..], 16)
                                    } else {
                                        num.parse::<u32>()
                                    } {
                                        char::from_u32(cp)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                        };
                        if let Some(c) = ch {
                            if c.is_whitespace() {
                                if !prev_space {
                                    out.push(' ');
                                    prev_space = true;
                                }
                            } else if !c.is_control() || c == '\n' {
                                out.push(c);
                                prev_space = false;
                            }
                        }
                        i += 1;
                    } else {
                        // Not a valid entity, emit as-is
                        out.push('&');
                        prev_space = false;
                    }
                }
                _ if !in_tag => {
                    let c = raw[i..].chars().next().unwrap_or(' ');
                    let c_len = c.len_utf8();
                    if c.is_whitespace() {
                        if !prev_space {
                            out.push(' ');
                            prev_space = true;
                        }
                    } else if !c.is_control() || c == '\n' {
                        out.push(c);
                        prev_space = false;
                    }
                    i += c_len;
                }
                _ => {
                    i += 1;
                }
            }
        }

        let trimmed = out.trim().to_string();
        trimmed
    }

    /// Return references to all interactive elements from a compressed tree.
    pub fn find_interactive(tree: &FlatDomTree) -> &[DomElement] {
        &tree.elements
    }
}

impl Default for DomAgent {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_dom_empty() {
        let agent = DomAgent::new();
        let tree = agent.compress_dom("");
        assert!(tree.is_empty());
        assert_eq!(tree.count, 0);
    }

    #[test]
    fn test_compress_dom_no_interactive() {
        let agent = DomAgent::new();
        let html = "<html><body><p>Hello</p><div>World</div></body></html>";
        let tree = agent.compress_dom(html);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_compress_dom_buttons_and_links() {
        let agent = DomAgent::new();
        let html = r#"<html><body>
            <button id="submit">Submit</button>
            <a href="/next">Next Page</a>
            <input type="text" name="search" placeholder="Search...">
            <select name="country"><option>US</option><option>UK</option></select>
            <textarea name="bio">Hello world</textarea>
        </body></html>"#;
        let tree = agent.compress_dom(html);
        // 5 primary + 2 options inside <select>
        assert_eq!(tree.count, 7);
        assert_eq!(tree.elements[0].tag, "button");
        assert_eq!(tree.elements[0].text, "Submit");
        assert_eq!(tree.elements[1].tag, "a");
        assert_eq!(tree.elements[2].tag, "input");
        assert_eq!(tree.elements[3].tag, "select");
        assert_eq!(tree.elements[4].tag, "option");
        assert_eq!(tree.elements[5].tag, "option");
        assert_eq!(tree.elements[6].tag, "textarea");
    }

    #[test]
    fn test_stable_indices() {
        let agent = DomAgent::new();
        let html = "<button>One</button><button>Two</button><button>Three</button>";
        let tree1 = agent.compress_dom(html);
        let tree2 = agent.compress_dom(html);
        assert_eq!(tree1.count, tree2.count);
        for i in 0..tree1.count {
            assert_eq!(tree1.elements[i].index, tree2.elements[i].index);
            assert_eq!(tree1.elements[i].text, tree2.elements[i].text);
        }
    }

    #[test]
    fn test_skips_script_and_style() {
        let agent = DomAgent::new();
        let html = r#"<html><head>
            <script>alert('xss')</script>
            <style>.btn{color:red}</style>
        </head><body><button id="real">Real</button></body></html>"#;
        let tree = agent.compress_dom(html);
        assert_eq!(tree.count, 1);
        assert_eq!(tree.elements[0].text, "Real");
    }

    #[test]
    fn test_text_decodes_entities() {
        let agent = DomAgent::new();
        let html = r#"<button>Save &amp; Close</button>"#;
        let tree = agent.compress_dom(html);
        assert_eq!(tree.elements[0].text, "Save & Close");
    }

    #[test]
    fn test_role_attributes_make_interactive() {
        let agent = DomAgent::new();
        let html = r#"<div role="button">Click</div><span role="link">Go</span>"#;
        let tree = agent.compress_dom(html);
        assert_eq!(tree.count, 2);
        assert_eq!(tree.elements[0].text, "Click");
        assert_eq!(tree.elements[1].text, "Go");
    }

    #[test]
    fn test_to_text_map_format() {
        let agent = DomAgent::new();
        let html = r#"<button id="go">Go</button><a href="/">Home</a>"#;
        let tree = agent.compress_dom(html);
        let map = tree.to_text_map();
        assert!(map.contains("[0]<button>"));
        assert!(map.contains("[1]<a>"));
        assert!(map.contains("id=\"go\""));
        assert!(map.contains("href=\"/\""));
    }

    #[test]
    fn test_get_by_index() {
        let agent = DomAgent::new();
        let html = "<button>First</button><button>Second</button>";
        let tree = agent.compress_dom(html);
        let second = tree.get(1);
        assert!(second.is_some());
        assert_eq!(second.unwrap().text, "Second");
        let missing = tree.get(99);
        assert!(missing.is_none());
    }

    #[test]
    fn test_find_interactive() {
        let agent = DomAgent::new();
        let html = r#"<input type="text"><textarea></textarea>"#;
        let tree = agent.compress_dom(html);
        let elems = DomAgent::find_interactive(&tree);
        assert_eq!(elems.len(), 2);
    }

    #[test]
    fn test_checkbox_and_radio_as_interactive() {
        let agent = DomAgent::new();
        let html = r#"<input type="checkbox"><input type="radio"><input type="hidden">"#;
        let tree = agent.compress_dom(html);
        assert_eq!(tree.count, 2, "checkbox and radio are interactive, hidden is not");
    }

    #[test]
    fn test_self_closing_input() {
        let agent = DomAgent::new();
        let html = r#"<input type="email" name="email" /><button>Send</button>"#;
        let tree = agent.compress_dom(html);
        assert_eq!(tree.count, 2);
        assert_eq!(tree.elements[0].tag, "input");
    }

    #[test]
    fn test_textarea_content() {
        let agent = DomAgent::new();
        let html = "<textarea name=\"msg\">Hello\nWorld</textarea>";
        let tree = agent.compress_dom(html);
        assert_eq!(tree.elements[0].text, "Hello World");
    }

    #[test]
    fn test_select_with_options() {
        let agent = DomAgent::new();
        let html = "<select name=\"lang\"><option value=\"en\">English</option><option value=\"fr\">French</option></select>";
        let tree = agent.compress_dom(html);
        assert_eq!(tree.count, 3);
        assert_eq!(tree.elements[0].tag, "select");
        assert_eq!(tree.elements[1].tag, "option");
        assert_eq!(tree.elements[2].tag, "option");
    }

    #[test]
    fn test_contenteditable_div() {
        let agent = DomAgent::new();
        let html = r#"<div contenteditable="true">Edit me</div>"#;
        let tree = agent.compress_dom(html);
        assert_eq!(tree.count, 1);
        assert_eq!(tree.elements[0].tag, "div");
    }

    #[test]
    fn test_tabindex_positive() {
        let agent = DomAgent::new();
        let html = r#"<div tabindex="0">Focusable</div><span tabindex="-1">Not</span>"#;
        let tree = agent.compress_dom(html);
        assert_eq!(tree.count, 1);
        assert_eq!(tree.elements[0].text, "Focusable");
    }

    #[test]
    fn test_comment_skipped() {
        let agent = DomAgent::new();
        let html = "<!-- comment --><button>OK</button><!-- another -->";
        let tree = agent.compress_dom(html);
        assert_eq!(tree.count, 1);
        assert_eq!(tree.elements[0].text, "OK");
    }

    #[test]
    fn test_nested_interactive() {
        let agent = DomAgent::new();
        let html = r#"<div><button><span>Nested</span></button></div>"#;
        let tree = agent.compress_dom(html);
        assert_eq!(tree.count, 1);
        assert_eq!(tree.elements[0].text, "Nested");
    }

    #[test]
    fn test_html_without_body() {
        let agent = DomAgent::new();
        let html = "<button>Alone</button>";
        let tree = agent.compress_dom(html);
        assert_eq!(tree.count, 1);
    }
}
