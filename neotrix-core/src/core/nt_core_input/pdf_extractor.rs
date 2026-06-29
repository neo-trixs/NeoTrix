use flate2::read::ZlibDecoder;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Read;

/// Extracted PDF document with page-level text content.
#[derive(Debug, Clone)]
pub struct PdfDocument {
    pub pages: Vec<PdfPage>,
}

impl PdfDocument {
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}

/// A single page with extracted text.
#[derive(Debug, Clone)]
pub struct PdfPage {
    pub page_num: usize,
    pub text: String,
}

/// PDF text extraction engine.
/// Core principle: scan raw PDF bytes for `stream...endstream` blocks,
/// decompress FlateDecode streams via zlib, parse PDF text operators
/// (Tj, TJ, ', ") to reconstruct textual content.
#[derive(Debug, Clone)]
pub struct PdfExtractor {
    max_stream_bytes: usize,
    max_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PdfError {
    InvalidHeader,
    NoTextFound,
    ParseError(String),
}

impl fmt::Display for PdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PdfError::InvalidHeader => write!(f, "not a valid PDF (bad header)"),
            PdfError::NoTextFound => write!(f, "no text found in PDF"),
            PdfError::ParseError(msg) => write!(f, "PDF parse error: {msg}"),
        }
    }
}

impl Default for PdfExtractor {
    fn default() -> Self {
        Self {
            max_stream_bytes: 10_000_000,
            max_pages: 100,
        }
    }
}

impl PdfExtractor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_stream_bytes(mut self, n: usize) -> Self {
        self.max_stream_bytes = n;
        self
    }

    pub fn with_max_pages(mut self, n: usize) -> Self {
        self.max_pages = n;
        self
    }

    /// Extract text from raw PDF bytes. Returns per-page text via page tree
    /// structure, falling back to flat stream extraction when pages cannot
    /// be navigated.
    pub fn extract(&self, data: &[u8]) -> Result<PdfDocument, PdfError> {
        if data.len() < 8 || &data[..5] != b"%PDF-" {
            return Err(PdfError::InvalidHeader);
        }
        // Locate all stream blocks and their enclosing object headers
        let stream_blocks = locate_streams(data);
        if stream_blocks.is_empty() {
            return Err(PdfError::ParseError("no streams found".into()));
        }
        // Decode and parse each stream, indexed by object ID
        let mut stream_texts: Vec<Vec<u8>> = Vec::new();
        let mut stream_obj_ids: Vec<u32> = Vec::new();
        for (obj_id, raw, filter) in &stream_blocks {
            if raw.len() > self.max_stream_bytes {
                continue;
            }
            if let Ok(decoded) = decode_stream_data(raw, filter) {
                stream_obj_ids.push(*obj_id);
                stream_texts.push(decoded);
            }
        }
        if stream_texts.is_empty() {
            return Err(PdfError::NoTextFound);
        }
        // Try page-tree partitioning
        let object_model = parse_objects(data);
        let page_texts = if let Ok(ref objs) = object_model {
            self.partition_by_pages(objs, &stream_texts, &stream_obj_ids)
        } else {
            Vec::new()
        };
        if page_texts.is_empty() {
            // fallback: all streams into one page
            let mut full = String::new();
            for st in &stream_texts {
                full.push_str(&parse_content_stream(st));
                full.push('\n');
            }
            let trimmed = full.trim();
            if trimmed.is_empty() {
                return Err(PdfError::NoTextFound);
            }
            return Ok(PdfDocument {
                pages: vec![PdfPage {
                    page_num: 1,
                    text: trimmed.to_string(),
                }],
            });
        }
        Ok(PdfDocument { pages: page_texts })
    }

    fn partition_by_pages(
        &self,
        objects: &[PdfObject],
        stream_texts: &[Vec<u8>],
        stream_obj_ids: &[u32],
    ) -> Vec<PdfPage> {
        // Build a map from object ID → decoded stream content
        let mut stream_map: std::collections::HashMap<u32, &[u8]> =
            std::collections::HashMap::new();
        for (i, obj_id) in stream_obj_ids.iter().enumerate() {
            if *obj_id > 0 && i < stream_texts.len() {
                stream_map.insert(*obj_id, &stream_texts[i]);
            }
        }

        // Walk page tree
        let catalog_id = match find_catalog(objects) {
            Some(id) => id,
            None => return Vec::new(),
        };
        let catalog_obj = match objects.iter().find(|o| o.id == catalog_id) {
            Some(o) => o,
            None => return Vec::new(),
        };
        let pages_ref = match catalog_obj
            .dict
            .get("/Pages")
            .and_then(|v| parse_obj_ref(v))
        {
            Some(r) => r,
            None => return Vec::new(),
        };

        let pages = collect_pages(objects, pages_ref);
        if pages.is_empty() {
            return Vec::new();
        }

        let mut result: Vec<PdfPage> = Vec::new();
        let mut page_num = 0usize;
        for (page_obj_id, content_refs) in &pages {
            if result.len() >= self.max_pages {
                break;
            }
            page_num += 1;
            let mut page_text = String::new();

            if content_refs.is_empty() {
                // Page with no /Contents: try the page object's own stream
                if let Some(stream) = stream_map.get(page_obj_id) {
                    let parsed = parse_content_stream(stream);
                    if !parsed.is_empty() {
                        page_text.push_str(&parsed);
                    }
                }
            } else {
                for (ref_obj_id, _) in content_refs {
                    if let Some(stream) = stream_map.get(ref_obj_id) {
                        let parsed = parse_content_stream(stream);
                        if !parsed.is_empty() {
                            if !page_text.is_empty() {
                                page_text.push('\n');
                            }
                            page_text.push_str(&parsed);
                        }
                    }
                }
            }

            if !page_text.trim().is_empty() {
                result.push(PdfPage {
                    page_num,
                    text: page_text.trim().to_string(),
                });
            }
        }

        if !result.is_empty() {
            return result;
        }

        // Fallback if page tree found no text: try streaming approach per page
        // by scanning objects for /Type /Page entries not reached from tree
        // This handles malformed page trees
        Vec::new()
    }
}

// ─── Raw Stream Locator ────────────────────────────────────────────────────

/// Scan raw PDF bytes for all `stream`...`endstream` blocks and detect
/// the compression filter from the preceding object dictionary.
fn locate_streams(data: &[u8]) -> Vec<(u32, Vec<u8>, String)> {
    let mut streams = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        let stream_start = match find_subseq(&data[pos..], b"stream\n") {
            Some(p) => pos + p + 7,
            None => match find_subseq(&data[pos..], b"stream\r\n") {
                Some(p) => pos + p + 8,
                None => break,
            },
        };
        let stream_end = match find_subseq(&data[stream_start..], b"\nendstream") {
            Some(p) => stream_start + p,
            None => match find_subseq(&data[stream_start..], b"\rendstream") {
                Some(p) => stream_start + p,
                None => break,
            },
        };
        let raw = &data[stream_start..stream_end];

        // Scan backwards from stream_start to find the object header (N 0 obj)
        let before = if stream_start > 300 {
            &data[stream_start - 300..stream_start]
        } else {
            &data[..stream_start]
        };
        let before_str = String::from_utf8_lossy(before);

        // Find /Filter key
        let filter = if before_str.contains("/FlateDecode") {
            "FlateDecode"
        } else if before_str.contains("/ASCIIHexDecode") {
            "ASCIIHexDecode"
        } else if before_str.contains("/ASCII85Decode") {
            "ASCII85Decode"
        } else {
            ""
        };

        // Extract object ID from preceding header
        let obj_id = extract_preceding_obj_id(&before_str);

        streams.push((obj_id, raw.to_vec(), filter.to_string()));
        pos = stream_end + 10;
    }
    streams
}

/// Scan backwards through text for a pattern like "N 0 obj" to get the owning
/// object ID of a stream block.
fn extract_preceding_obj_id(before: &str) -> u32 {
    // Find the last occurrence of " obj" and work backwards
    if let Some(obj_pos) = before.rfind(" obj") {
        let pre = &before[..obj_pos].trim();
        if let Some(space) = pre.rfind(' ') {
            let num_str = pre[space + 1..].trim();
            if let Some(space2) = num_str.rfind(' ') {
                num_str[space2 + 1..].parse().unwrap_or(0)
            } else {
                num_str.parse().unwrap_or(0)
            }
        } else {
            pre.parse().unwrap_or(0)
        }
    } else {
        0
    }
}

fn find_subseq(data: &[u8], pat: &[u8]) -> Option<usize> {
    if pat.is_empty() || pat.len() > data.len() {
        return None;
    }
    for i in 0..=data.len() - pat.len() {
        if &data[i..i + pat.len()] == pat {
            return Some(i);
        }
    }
    None
}

// ─── Decompression ─────────────────────────────────────────────────────────

fn decode_stream_data(raw: &[u8], filter: &str) -> Result<Vec<u8>, PdfError> {
    match filter {
        "FlateDecode" => {
            let mut decoder = ZlibDecoder::new(raw);
            let mut out = Vec::with_capacity(raw.len() * 4);
            decoder
                .read_to_end(&mut out)
                .map_err(|_| PdfError::ParseError("zlib decompress failed".into()))?;
            if out.is_empty() {
                return Err(PdfError::ParseError("empty after decompress".into()));
            }
            Ok(out)
        }
        "" => {
            // no compression — maybe raw text
            let printable = raw.iter().all(|&b| {
                b.is_ascii_graphic() || b == b' ' || b == b'\n' || b == b'\r' || b == b'\t'
            });
            if printable {
                Ok(raw.to_vec())
            } else {
                Err(PdfError::ParseError(
                    "non-printable uncompressed stream".into(),
                ))
            }
        }
        other => Err(PdfError::ParseError(format!("unsupported filter: {other}"))),
    }
}

// ─── Content Stream Text Parser ────────────────────────────────────────────

/// Parse PDF content stream operators to extract text.
/// Handles: (string) Tj, [(s) n (s) ...] TJ, ' (next-line), " (next-line with spacing)
fn parse_content_stream(data: &[u8]) -> String {
    let s = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    while i < len {
        // skip whitespace
        while i < len && chars[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= len {
            break;
        }
        match chars[i] {
            '(' => {
                // literal string: extract with escape handling
                i += 1;
                let mut depth = 1;
                let mut text = String::new();
                while i < len && depth > 0 {
                    match chars[i] {
                        '(' => {
                            depth += 1;
                            if depth > 1 {
                                text.push('(');
                            }
                        }
                        ')' => {
                            depth -= 1;
                            if depth > 0 {
                                text.push(')');
                            }
                        }
                        '\\' => {
                            i += 1;
                            if i < len {
                                match chars[i] {
                                    'n' => text.push('\n'),
                                    'r' => text.push('\r'),
                                    't' => text.push('\t'),
                                    '(' | ')' | '\\' => text.push(chars[i]),
                                    d if d.is_ascii_digit() => {
                                        let mut octal = String::from(d);
                                        for _ in 0..2 {
                                            if i + 1 < len && chars[i + 1].is_ascii_digit() {
                                                i += 1;
                                                octal.push(chars[i]);
                                            }
                                        }
                                        if let Ok(code) = u32::from_str_radix(&octal, 8) {
                                            if let Some(c) = char::from_u32(code) {
                                                text.push(c);
                                            }
                                        }
                                    }
                                    c => text.push(c),
                                }
                            }
                        }
                        c => text.push(c),
                    }
                    i += 1;
                }
                if !text.is_empty() {
                    result.push_str(&text);
                    result.push(' ');
                }
            }
            '[' => {
                // TJ array: [...]
                i += 1;
                let mut depth = 1;
                let mut arr_content = String::new();
                while i < len && depth > 0 {
                    match chars[i] {
                        '[' => depth += 1,
                        ']' => depth -= 1,
                        _ if depth == 1 => arr_content.push(chars[i]),
                        _ => {}
                    }
                    i += 1;
                }
                let extracted = extract_paren_strings(&arr_content);
                if !extracted.is_empty() {
                    result.push_str(&extracted);
                    result.push(' ');
                }
            }
            _ => {
                // operator or operand: skip to next whitespace
                let op_start = i;
                while i < len && !chars[i].is_ascii_whitespace() {
                    i += 1;
                }
                let op: String = chars[op_start..i].iter().collect();
                match op.as_str() {
                    "'" | "\"" => result.push('\n'),
                    _ => {} // skip structural operators
                }
            }
        }
    }
    result.trim().to_string()
}

fn extract_paren_strings(s: &str) -> String {
    let mut result = String::new();
    let mut in_paren = false;
    let mut depth = 0;
    let mut current = String::new();
    let mut i = 0;
    let chars: Vec<char> = s.chars().collect();
    while i < chars.len() {
        match chars[i] {
            '(' if !in_paren => {
                in_paren = true;
                depth = 1;
                current.clear();
            }
            '(' if in_paren => {
                depth += 1;
                current.push('(');
            }
            ')' if in_paren => {
                depth -= 1;
                if depth == 0 {
                    result.push_str(&current);
                    result.push(' ');
                    in_paren = false;
                } else {
                    current.push(')');
                }
            }
            '\\' if in_paren => {
                i += 1;
                if i < chars.len() {
                    match chars[i] {
                        'n' => current.push('\n'),
                        'r' => current.push('\r'),
                        't' => current.push('\t'),
                        '(' | ')' | '\\' => current.push(chars[i]),
                        c => current.push(c),
                    }
                }
            }
            c if in_paren => current.push(c),
            _ => {}
        }
        i += 1;
    }
    result
}

// ─── Object Model (for page tree navigation) ──────────────────────────────

#[derive(Debug)]
struct PdfObject {
    id: u32,
    _gen: u32,
    dict: std::collections::HashMap<String, String>,
    _is_stream: bool,
}

fn parse_objects(data: &[u8]) -> Result<Vec<PdfObject>, PdfError> {
    let s = std::str::from_utf8(data).map_err(|e| PdfError::ParseError(format!("utf8: {e}")))?;
    let mut objects = Vec::new();
    // Extract body before xref
    let body = if let Some(eof) = s.rfind("\n%%EOF") {
        if let Some(xp) = s[..eof].rfind("\nxref") {
            &s[..xp]
        } else {
            &s[..eof]
        }
    } else {
        s
    };
    let mut pos = 0;
    loop {
        let obj_end = match body[pos..].find("\nendobj") {
            Some(e) => pos + e + 7,
            None => break,
        };
        let obj_start = match body[..obj_end].rfind(&['\n', '\r'][..]) {
            Some(h) if h > pos => h + 1,
            _ => pos,
        };
        let block = &body[obj_start..obj_end];
        if let Some(obj) = parse_single_object(block) {
            objects.push(obj);
        }
        pos = obj_end;
    }
    if objects.is_empty() {
        Err(PdfError::ParseError("no objects found".into()))
    } else {
        Ok(objects)
    }
}

fn parse_single_object(block: &str) -> Option<PdfObject> {
    let header_end = block.find(" obj")?;
    let parts: Vec<&str> = block[..header_end].trim().split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let id: u32 = parts[0].parse().ok()?;
    let gen: u32 = parts[1].parse().ok()?;
    let content = block[header_end + 4..].trim();
    let dict_end = find_dict_end(content);
    let dict_str = if dict_end > 0 {
        &content[..dict_end]
    } else {
        content
    };
    let dict = parse_dict(dict_str);
    let is_stream = content.contains("\nstream\n") || content.contains("\nstream\r\n");
    Some(PdfObject {
        id,
        _gen: gen,
        dict,
        _is_stream: is_stream,
    })
}

fn find_dict_end(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut in_paren = false;
    let mut i = 0;
    while i < s.len() {
        match bytes[i] {
            b'(' => in_paren = true,
            b')' => in_paren = false,
            b'<' if !in_paren && i + 1 < s.len() && bytes[i + 1] == b'<' => {
                depth += 1;
                i += 1;
            }
            b'>' if !in_paren && depth > 0 && i + 1 < s.len() && bytes[i + 1] == b'>' => {
                depth -= 1;
                i += 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    0
}

fn parse_dict(s: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let inner = if s.starts_with("<<") && s.ends_with(">>") {
        &s[2..s.len() - 2]
    } else {
        s
    };
    let toks = tokenize_dict(inner.trim());
    let mut i = 0;
    while i + 1 < toks.len() {
        let key = toks[i].trim();
        let val = toks[i + 1].trim();
        if key.starts_with('/') {
            map.insert(key.to_string(), val.to_string());
        }
        i += 2;
    }
    map
}

fn tokenize_dict(s: &str) -> Vec<String> {
    let mut toks = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    let mut in_paren = false;
    let mut i = 0;
    let chars: Vec<char> = s.chars().collect();
    while i < chars.len() {
        match chars[i] {
            '(' => {
                current.push('(');
                let mut pdepth = 1;
                while i + 1 < chars.len() && pdepth > 0 {
                    i += 1;
                    match chars[i] {
                        '(' => {
                            pdepth += 1;
                            current.push('(');
                        }
                        ')' => {
                            pdepth -= 1;
                            if pdepth > 0 {
                                current.push(')');
                            }
                        }
                        '\\' => {
                            current.push('\\');
                            if i + 1 < chars.len() {
                                i += 1;
                                current.push(chars[i]);
                            }
                        }
                        c => current.push(c),
                    }
                }
                if pdepth == 0 {
                    current.push(')');
                }
                in_paren = false;
            }
            '<' if !in_paren && i + 1 < chars.len() && chars[i + 1] == '<' => {
                depth += 1;
                current.push_str("<<");
                i += 1;
            }
            '>' if depth > 0 && !in_paren && i + 1 < chars.len() && chars[i + 1] == '>' => {
                depth -= 1;
                current.push_str(">>");
                i += 1;
                if depth == 0 {
                    toks.push(current.clone());
                    current.clear();
                }
            }
            '[' if !in_paren && depth == 0 => {
                let mut adepth = 1;
                current.push('[');
                while i + 1 < chars.len() && adepth > 0 {
                    i += 1;
                    match chars[i] {
                        '[' => {
                            adepth += 1;
                            current.push('[');
                        }
                        ']' => {
                            adepth -= 1;
                            if adepth > 0 {
                                current.push(']');
                            }
                        }
                        c => current.push(c),
                    }
                }
                current.push(']');
                toks.push(current.clone());
                current.clear();
            }
            c if depth == 0 && !in_paren && c.is_ascii_whitespace() => {
                if !current.is_empty() {
                    toks.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(chars[i]),
        }
        i += 1;
    }
    if !current.is_empty() {
        toks.push(current);
    }
    toks
}

fn parse_obj_ref(s: &str) -> Option<u32> {
    let clean = s.trim().trim_end_matches('R').trim();
    clean
        .split_whitespace()
        .next()
        .and_then(|n| n.parse::<u32>().ok())
}

fn find_catalog(objects: &[PdfObject]) -> Option<u32> {
    for obj in objects {
        if let Some(t) = obj.dict.get("/Type") {
            let tt = t.trim().trim_end_matches(']');
            if tt == "/Catalog" {
                return Some(obj.id);
            }
        }
    }
    None
}

fn collect_pages(objects: &[PdfObject], pages_ref: u32) -> Vec<(u32, Vec<(u32, u32)>)> {
    let mut result = Vec::new();
    let mut visited = std::collections::HashSet::new();
    collect_pages_rec(objects, pages_ref, &mut result, &mut visited);
    result
}

fn collect_pages_rec(
    objects: &[PdfObject],
    node_ref: u32,
    result: &mut Vec<(u32, Vec<(u32, u32)>)>,
    visited: &mut std::collections::HashSet<u32>,
) {
    if !visited.insert(node_ref) {
        return;
    }
    let obj = match objects.iter().find(|o| o.id == node_ref) {
        Some(o) => o,
        None => return,
    };
    let type_val = obj
        .dict
        .get("/Type")
        .map(|s| s.trim().trim_end_matches(']'))
        .unwrap_or("");
    if type_val == "/Page" {
        let mut refs = Vec::new();
        if let Some(contents) = obj.dict.get("/Contents") {
            let c = contents.trim();
            if c.starts_with('[') {
                for r in extract_int_array(c) {
                    refs.push((r, 0));
                }
            } else if let Some(r) = parse_obj_ref(c) {
                refs.push((r, 0));
            }
        }
        result.push((node_ref, refs));
    } else if type_val == "/Pages" {
        if let Some(kids) = obj.dict.get("/Kids") {
            for kid in extract_int_array(kids) {
                collect_pages_rec(objects, kid, result, visited);
            }
        }
    }
}

fn extract_int_array(s: &str) -> Vec<u32> {
    let inner = s
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim_end_matches(']');
    let mut result = Vec::new();
    for part in inner.split_whitespace() {
        if let Some(rest) = part.strip_suffix("R") {
            if let Ok(n) = rest
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("")
                .parse::<u32>()
            {
                result.push(n);
            }
        } else if let Ok(n) = part.parse::<u32>() {
            result.push(n);
        }
    }
    result
}

// ─── Public API ────────────────────────────────────────────────────────────

pub fn extract_text_from_pdf(data: &[u8]) -> Result<String, PdfError> {
    let doc = PdfExtractor::new().extract(data)?;
    let mut full = String::new();
    for page in &doc.pages {
        full.push_str(&page.text);
        full.push_str("\n---\n");
    }
    Ok(full.trim().to_string())
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_header() {
        let r = extract_text_from_pdf(b"not a pdf");
        assert!(r.is_err());
    }

    #[test]
    fn test_empty_pdf_structure() {
        let r = extract_text_from_pdf(b"%PDF-1.4\n%%EOF\n");
        assert!(r.is_err());
    }

    #[test]
    fn test_parse_simple_stream() {
        let pdf = b"%PDF-1.4\n1 0 obj\n<< /Length 44 >>\nstream\nBT\n/F1 12 Tf\n(Hello World) Tj\nET\nendstream\nendobj\n%%EOF\n";
        let r = extract_text_from_pdf(pdf);
        assert!(r.is_ok(), "should extract text: {:?}", r.err());
        let text = r.unwrap();
        assert!(text.contains("Hello World"), "text: {text}");
    }

    #[test]
    fn test_tj_array() {
        let pdf = b"%PDF-1.4\n1 0 obj\n<<>>\nstream\nBT\n/F1 12 Tf\n[(Hello) 20 (World)] TJ\nET\nendstream\nendobj\n%%EOF\n";
        let r = extract_text_from_pdf(pdf);
        assert!(r.is_ok());
        let text = r.unwrap();
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn test_escape_sequences() {
        let stream = b"BT\n/F1 12 Tf\n(Line 1\\nLine 2\\tTab) Tj\nET\n";
        let text = parse_content_stream(stream);
        assert!(text.contains('\n'), "should contain newline");
        assert!(text.contains("Tab"), "should contain tab text");
    }

    #[test]
    fn test_parse_objects_detection() {
        let pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n%%EOF\n";
        let objs = parse_objects(pdf).unwrap();
        assert!(objs.len() >= 2);
        assert!(objs.iter().any(|o| o.id == 1));
    }

    #[test]
    fn test_find_catalog() {
        let objs = vec![PdfObject {
            id: 1,
            _gen: 0,
            dict: [("/Type".into(), "/Catalog".into())].into(),
            _is_stream: false,
        }];
        assert_eq!(find_catalog(&objs), Some(1));
    }

    #[test]
    fn test_locate_streams_simple() {
        let pdf = b"%PDF-1.4\nobj\n<<>>\nstream\nhello\nendstream\n%%EOF";
        let streams = locate_streams(pdf);
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].0, 0); // obj_id should be 0 (no header)
        assert_eq!(streams[0].2, ""); // no filter
    }

    #[test]
    fn test_locate_streams_with_obj_id() {
        let pdf = b"%PDF-1.4\n1 0 obj\n<< /Filter /FlateDecode >>\nstream\nhello\nendstream\nendobj\n%%EOF\n";
        let streams = locate_streams(pdf);
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].0, 1); // captured obj ID
    }

    #[test]
    fn test_partition_by_pages_single_page() {
        let pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R /Contents 4 0 R >>\nendobj\n4 0 obj\n<< /Length 44 >>\nstream\nBT\n/F1 12 Tf\n(Page One Content) Tj\nET\nendstream\nendobj\n%%EOF\n";
        let doc = PdfExtractor::new().extract(pdf).unwrap();
        assert_eq!(doc.pages.len(), 1, "should detect 1 page");
        assert!(doc.pages[0].text.contains("Page One Content"));
        assert_eq!(doc.pages[0].page_num, 1);
    }

    #[test]
    fn test_no_streams() {
        let r = extract_text_from_pdf(b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog >>\nendobj\n%%EOF\n");
        assert!(r.is_err());
    }

    #[test]
    fn test_pdf_extractor_configure() {
        let e = PdfExtractor::new()
            .with_max_pages(10)
            .with_max_stream_bytes(1_000);
        assert_eq!(e.max_pages, 10);
        assert_eq!(e.max_stream_bytes, 1_000);
    }

    #[test]
    fn test_empty_stream() {
        let r = decode_stream_data(b"", "FlateDecode");
        assert!(r.is_err());
    }

    #[test]
    fn test_extract_int_array_basic() {
        assert_eq!(extract_int_array("[3 0 R 4 0 R]"), vec![3, 4]);
    }

    #[test]
    fn test_parse_obj_ref_valid() {
        assert_eq!(parse_obj_ref("3 0 R"), Some(3));
    }

    #[test]
    fn test_find_dict_end_basic() {
        let s = "<< /Type /Page >>\nstream\n";
        let end = find_dict_end(s);
        assert!(end > 0);
        assert_eq!(&s[..end], "<< /Type /Page >>");
    }

    #[test]
    fn test_tokenize_simple() {
        let toks = tokenize_dict("/Type /Page /MediaBox [0 0 612 792]");
        assert!(toks.contains(&"/Type".to_string()));
        assert!(toks.contains(&"/Page".to_string()));
    }

    #[test]
    fn test_content_stream_empty() {
        assert_eq!(parse_content_stream(b""), "");
    }

    #[test]
    fn test_content_stream_multiline() {
        let data = b"BT\n/F1 12 Tf\n(First) Tj\nET\nBT\n/F1 14 Tf\n(Second) Tj\nET\n";
        let text = parse_content_stream(data);
        assert!(text.contains("First"));
        assert!(text.contains("Second"));
    }

    #[test]
    fn test_locate_flatedecode() {
        let pdf = b"%PDF-1.4\nobj\n<< /Filter /FlateDecode >>\nstream\nx\x9c\x0b\xc9\xc8,V\x00\xa2\x04\x00\x0c\xc8\x02k\nendstream\nendobj\n%%EOF\n";
        let streams = locate_streams(pdf);
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].2, "FlateDecode");
    }
}
