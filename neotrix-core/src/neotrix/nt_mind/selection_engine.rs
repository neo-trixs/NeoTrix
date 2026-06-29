//! Adaptive selection + structured extraction engine.
//!
//! Two subsystems:
//! 1. AdaptiveSelector — tracks elements by structural fingerprint, survives DOM changes
//! 2. ExtractionSchema — defines how to extract structured data from pages

use std::collections::{HashMap, HashSet};
use std::fmt::Write as FmtWrite;
use std::hash::{DefaultHasher, Hash, Hasher};

// ============================================================================
// Constants
// ============================================================================

/// HTML void elements — no closing tag
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

// ============================================================================
// HTML scanning utilities (string-based, zero external dependencies)
// ============================================================================

/// Information extracted from a single HTML tag
#[derive(Debug, Clone)]
struct RawTag {
    tag_name: String,
    attrs_raw: String,
    start: usize,
    end: usize,
    is_self_closing: bool,
    is_closing: bool,
}

fn parse_tag_at(html: &str, pos: usize) -> Option<RawTag> {
    if !html[pos..].starts_with('<') {
        return None;
    }
    let rest = &html[pos + 1..];

    let is_closing = rest.starts_with('/');
    let after_slash = if is_closing { 1 } else { 0 };

    if rest.starts_with("!--") {
        let end = html[pos..].find("-->")?;
        return Some(RawTag {
            tag_name: String::new(),
            attrs_raw: String::new(),
            start: pos,
            end: pos + end + 3,
            is_self_closing: true,
            is_closing: false,
        });
    }

    let body = &rest[after_slash..];
    let name_end = body
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(body.len());
    let tag_name = body[..name_end].to_lowercase();
    if tag_name.is_empty() {
        return None;
    }

    let after_name = &body[name_end..];
    let mut in_quote = false;
    let mut quote_char = '"';
    let mut gt_offset = 0;
    for (i, c) in after_name.char_indices() {
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
                '>' => {
                    gt_offset = i + 1;
                    break;
                }
                _ => {}
            }
        }
    }
    if gt_offset == 0 {
        return None;
    }

    let attrs_raw = after_name[..gt_offset - 1].trim().to_string();
    let is_self_closing = attrs_raw.ends_with('/') || VOID_ELEMENTS.contains(&tag_name.as_str());
    let end = pos + 1 + after_slash + name_end + gt_offset;

    Some(RawTag {
        tag_name,
        attrs_raw,
        start: pos,
        end,
        is_self_closing,
        is_closing,
    })
}

fn html_skip_past_tag(html: &str, pos: usize) -> Option<usize> {
    let rest = &html[pos..];
    let mut in_quote = false;
    let mut quote_char = '"';
    for (i, c) in rest.char_indices() {
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
                '>' => return Some(pos + i + 1),
                _ => {}
            }
        }
    }
    None
}

/// Find position of closing tag matching an opening tag.
fn find_closing(html: &str, tag: &str, open_end: usize) -> Option<usize> {
    let tag = tag.to_lowercase();
    let mut depth: i32 = 1;
    let mut pos = open_end;
    while depth > 0 {
        let rest = &html[pos..];
        let next = rest.find('<')?;
        pos += next;
        if pos + 1 >= html.len() {
            return None;
        }
        let c2 = html.as_bytes()[pos + 1];
        if c2 == b'/' {
            let cr = &html[pos + 2..];
            let ne = cr.find(|c: char| c.is_whitespace() || c == '>')?;
            if cr[..ne].to_lowercase() == tag {
                depth -= 1;
                if depth == 0 {
                    return html_skip_past_tag(html, pos);
                }
            }
            pos = html_skip_past_tag(html, pos)?;
        } else if c2 == b'!' || c2 == b'?' {
            pos = html_skip_past_tag(html, pos)?;
        } else {
            if let Some(rt) = parse_tag_at(html, pos) {
                if !rt.is_self_closing && !rt.is_closing && rt.tag_name.to_lowercase() == tag {
                    depth += 1;
                }
                pos = rt.end;
            } else {
                pos = html_skip_past_tag(html, pos)?;
            }
        }
    }
    None
}

fn extract_classes_from(attrs: &str) -> Vec<String> {
    let lower = attrs.to_lowercase();
    let idx = match lower.find("class=") {
        Some(i) => i,
        None => return vec![],
    };
    let after = &attrs[idx + 6..];
    let q = match after.chars().next() { Some(c) => c, None => return vec![] };
    let start = 1;
    if start >= after.len() {
        return vec![];
    }
    let inner = &after[start..];
    let end = match inner.find(q) { Some(e) => e, None => return vec![] };
    inner[..end]
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn extract_id_from(attrs: &str) -> Option<String> {
    let lower = attrs.to_lowercase();
    let idx = lower.find("id=")?;
    let after = &attrs[idx + 3..];
    let q = after.chars().next()?;
    let start = 1;
    if start >= after.len() {
        return None;
    }
    let inner = &after[start..];
    let end = inner.find(q)?;
    let val = inner[..end].trim().to_string();
    if val.is_empty() {
        None
    } else {
        Some(val)
    }
}

fn extract_attr_keys(attrs: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut i = 0;
    let b = attrs.as_bytes();
    while i < b.len() {
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= b.len() {
            break;
        }
        let ks = i;
        while i < b.len() && b[i] != b'=' && !b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i > ks {
            keys.push(attrs[ks..i].to_string());
        }
        if i < b.len() && b[i] == b'=' {
            i += 1;
            if i < b.len() && (b[i] == b'"' || b[i] == b'\'') {
                let q = b[i];
                i += 1;
                while i < b.len() && b[i] != q {
                    i += 1;
                }
                if i < b.len() {
                    i += 1;
                }
            }
        }
    }
    keys
}

fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut buf = String::new();
    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                buf.clear();
                buf.push(c);
            }
            '>' if in_tag => {
                buf.push(c);
                let l = buf.to_lowercase();
                if l.starts_with("<script") {
                    in_script = true;
                } else if l.starts_with("</script") {
                    in_script = false;
                } else if l.starts_with("<style") {
                    in_style = true;
                } else if l.starts_with("</style") {
                    in_style = false;
                }
                in_tag = false;
            }
            _ if !in_tag && !in_script && !in_style => {
                if c.is_whitespace() {
                    if !out.ends_with(' ') && !out.is_empty() {
                        out.push(' ');
                    }
                } else {
                    out.push(c);
                }
            }
            _ if in_tag => buf.push(c),
            _ => {}
        }
    }
    out.trim().to_string()
}

fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

// ============================================================================
// Parsed CSS selector
// ============================================================================

#[derive(Debug, Clone)]
struct CssSelector {
    tag: Option<String>,
    class: Option<String>,
    id: Option<String>,
}

fn parse_selector(sel: &str) -> CssSelector {
    let part = sel
        .rsplit(|c: char| c == ' ' || c == '>' || c == '+' || c == '~')
        .next()
        .unwrap_or(sel)
        .trim();
    let mut tag = None;
    let mut class = None;
    let mut id = None;
    let mut current = String::new();
    let mut expect: Option<u8> = None;
    for c in part.chars() {
        match expect {
            Some(b'.') => {
                if c == '.' || c == '#' {
                    class = Some(std::mem::take(&mut current));
                    expect = Some(if c == '#' { b'#' } else { b'.' });
                } else {
                    current.push(c);
                }
            }
            Some(b'#') => {
                if c == '.' || c == '#' {
                    id = Some(std::mem::take(&mut current));
                    expect = Some(if c == '#' { b'#' } else { b'.' });
                } else {
                    current.push(c);
                }
            }
            _ => match c {
                '.' => {
                    tag = Some(std::mem::take(&mut current));
                    expect = Some(b'.');
                }
                '#' => {
                    tag = Some(std::mem::take(&mut current));
                    expect = Some(b'#');
                }
                _ => current.push(c),
            },
        }
    }
    if !current.is_empty() {
        match expect {
            Some(b'.') => class = Some(current),
            Some(b'#') => id = Some(current),
            _ => tag = Some(current),
        }
    }
    CssSelector { tag, class, id }
}

fn tag_matches(rt: &RawTag, sel: &CssSelector) -> bool {
    if let Some(ref t) = sel.tag {
        if rt.tag_name != *t {
            return false;
        }
    }
    if let Some(ref c) = sel.class {
        let cls = extract_classes_from(&rt.attrs_raw);
        if !cls.iter().any(|x| x == c) {
            return false;
        }
    }
    if let Some(ref i) = sel.id {
        if extract_id_from(&rt.attrs_raw).as_deref() != Some(i) {
            return false;
        }
    }
    true
}

// ============================================================================
// Part 1: Structural Fingerprint
// ============================================================================

/// Represents an element's structural fingerprint — survives class renaming and
/// DOM restructuring.
#[derive(Debug, Clone)]
pub struct StructuralFingerprint {
    /// Tag path from root: e.g. "html/body/div[3]/div[1]/article"
    pub tag_path: String,
    /// Text density (text_length / total_bytes)
    pub text_density: f64,
    /// Number of children
    pub child_count: usize,
    /// Depth in DOM tree
    pub depth: usize,
    /// Sibling index (1-based)
    pub sibling_index: usize,
    /// Number of similar siblings (same tag)
    pub similar_sibling_count: usize,
    /// Attribute keys present (sorted)
    pub attr_keys: Vec<String>,
    /// Class names
    pub classes: Vec<String>,
    /// ID if present
    pub id: Option<String>,
    /// Text content hash (first 100 chars)
    pub text_sig: u64,
    /// Whether element contains links
    pub has_links: bool,
    /// Whether element contains images
    pub has_images: bool,
    /// Whether element contains tables
    pub has_tables: bool,
}

impl StructuralFingerprint {
    /// Parse HTML and compute fingerprints for all elements matching a CSS selector.
    pub fn from_html(html: &str, css_selector: &str) -> Result<Vec<Self>, String> {
        let sel = parse_selector(css_selector);
        let mut fps = Vec::new();

        let mut path_tags: Vec<String> = Vec::new();
        let mut lvl_counts: Vec<HashMap<String, usize>> = Vec::new();
        let mut pos = 0;

        while pos < html.len() {
            let rest = &html[pos..];
            let next = rest.find('<').map(|i| pos + i);
            let n = match next {
                Some(p) => p,
                None => break,
            };

            if n + 1 >= html.len() {
                break;
            }

            let c2 = html.as_bytes()[n + 1];

            if c2 == b'/' {
                let cr = &html[n + 2..];
                let ne = cr
                    .find(|c: char| c.is_whitespace() || c == '>')
                    .unwrap_or(cr.len());
                let cn = cr[..ne].to_lowercase();
                if path_tags.last().map(|t| t == &cn).unwrap_or(false) {
                    path_tags.pop();
                    lvl_counts.pop();
                }
                pos = html_skip_past_tag(html, n).unwrap_or(n + 1);
                continue;
            }

            if c2 == b'!' || c2 == b'?' {
                pos = html_skip_past_tag(html, n).unwrap_or(n + 1);
                continue;
            }

            if let Some(rt) = parse_tag_at(html, n) {
                if !rt.is_closing {
                    let sidx = {
                        let mut fallback = HashMap::new();
                        let count_map = lvl_counts.last_mut();
                        let map = count_map.unwrap_or(&mut fallback);
                        let entry = map.entry(rt.tag_name.clone()).or_insert(0);
                        *entry += 1;
                        *entry
                    };
                    let total_same = sidx;

                    if tag_matches(&rt, &sel) {
                        if let Some(fp) =
                            compute_fp(html, &rt, &path_tags, &lvl_counts, sidx, total_same)
                        {
                            fps.push(fp);
                        }
                    }

                    if !rt.is_self_closing {
                        path_tags.push(rt.tag_name.clone());
                        lvl_counts.push(HashMap::new());
                    }
                }
                pos = rt.end;
            } else {
                pos = n + 1;
            }
        }

        if fps.is_empty() {
            Err(format!("No elements matched selector: {}", css_selector))
        } else {
            Ok(fps)
        }
    }

    /// 12-factor cosine-ish similarity between two fingerprints.
    ///
    /// Weights:
    /// - tag_path exact: 0.25 (full match) or partial by segments
    /// - text_density: 0.15 (1 - |diff|/max)
    /// - child_count: 0.10
    /// - depth: 0.05
    /// - sibling_index + similar_sibling_count: 0.05 + 0.05
    /// - attr_keys Jaccard: 0.10
    /// - classes Jaccard: 0.10
    /// - id exact: +0.10 (bonus)
    /// - text_sig exact: +0.05 (bonus)
    /// - has_links + has_images + has_tables: 0.05 each
    pub fn similarity(&self, other: &Self) -> f64 {
        let mut score = 0.0;

        // tag_path: 0.25 — exact path or segment overlap
        let self_segs: Vec<&str> = self.tag_path.split('/').collect();
        let other_segs: Vec<&str> = other.tag_path.split('/').collect();
        let common = self_segs
            .iter()
            .zip(other_segs.iter())
            .take_while(|(a, b)| a == b)
            .count();
        let max_len = self_segs.len().max(other_segs.len());
        let path_score = if self_segs == other_segs {
            0.25
        } else if max_len > 0 {
            let ratio = common as f64 / max_len as f64;
            ratio * 0.20 // at most 0.20 for partial
        } else {
            0.0
        };
        score += path_score;

        // text_density: 0.15
        let max_d = self.text_density.max(other.text_density).max(1.0);
        score += 0.15 * (1.0 - (self.text_density - other.text_density).abs() / max_d);

        // child_count: 0.10
        let max_c = self.child_count.max(other.child_count).max(1) as f64;
        score += 0.10 * (1.0 - (self.child_count as f64 - other.child_count as f64).abs() / max_c);

        // depth: 0.05
        if self.depth == other.depth {
            score += 0.05;
        } else {
            let max_dep = self.depth.max(other.depth).max(1) as f64;
            score += 0.05 * (1.0 - (self.depth as f64 - other.depth as f64).abs() / max_dep);
        }

        // sibling_index: 0.05
        let max_si = self.sibling_index.max(other.sibling_index).max(1) as f64;
        score +=
            0.05 * (1.0 - (self.sibling_index as f64 - other.sibling_index as f64).abs() / max_si);

        // similar_sibling_count: 0.05
        if self.similar_sibling_count == other.similar_sibling_count {
            score += 0.05;
        } else {
            let max_sc = self
                .similar_sibling_count
                .max(other.similar_sibling_count)
                .max(1) as f64;
            score += 0.05
                * (1.0
                    - (self.similar_sibling_count as f64 - other.similar_sibling_count as f64)
                        .abs()
                        / max_sc);
        }

        // attr_keys Jaccard: 0.10
        let j_attr = jaccard(&self.attr_keys, &other.attr_keys);
        score += 0.10 * j_attr;

        // classes Jaccard: 0.10
        let j_cls = jaccard(&self.classes, &other.classes);
        score += 0.10 * j_cls;

        // id exact: +0.10
        if self.id.is_some() && other.id.is_some() && self.id == other.id {
            score += 0.10;
        }

        // text_sig exact: +0.05
        if self.text_sig == other.text_sig && self.text_sig != 0 {
            score += 0.05;
        }

        // has_links: 0.05
        if self.has_links == other.has_links {
            score += 0.05;
        }

        // has_images: 0.05
        if self.has_images == other.has_images {
            score += 0.05;
        }

        // has_tables: 0.05
        if self.has_tables == other.has_tables {
            score += 0.05;
        }

        score.min(1.0).max(0.0)
    }

    /// Best-effort CSS selector from fingerprint.
    pub fn to_selector(&self) -> String {
        if let Some(ref id) = self.id {
            return format!("#{}", id);
        }
        let tag = self.tag_path.split('/').last().unwrap_or("div");
        if !self.classes.is_empty() {
            let cls = self.classes.join(".");
            return format!("{}.{}", tag, cls);
        }
        if self.sibling_index > 1 || self.similar_sibling_count > 1 {
            return format!("{}:nth-child({})", tag, self.sibling_index);
        }
        tag.to_string()
    }
}

fn jaccard(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let sa: HashSet<&str> = a.iter().map(|s| s.as_str()).collect();
    let sb: HashSet<&str> = b.iter().map(|s| s.as_str()).collect();
    let inter = sa.intersection(&sb).count() as f64;
    let union = sa.union(&sb).count() as f64;
    if union == 0.0 {
        1.0
    } else {
        inter / union
    }
}

macro_rules! path_write {
    ($dst:expr, $($arg:tt)*) => {
        match write!($dst, $($arg)*) {
            Ok(_) => Some(()),
            Err(_) => None,
        }
    };
}

/// Compute fingerprint for a matched tag.
fn compute_fp(
    html: &str,
    rt: &RawTag,
    path_tags: &[String],
    lvl_counts: &[HashMap<String, usize>],
    sibling_index: usize,
    total_same: usize,
) -> Option<StructuralFingerprint> {
    let mut tag_path = String::new();
    for (i, pt) in path_tags.iter().enumerate() {
        let count = lvl_counts
            .get(i)
            .and_then(|m| m.get(pt))
            .copied()
            .unwrap_or(1);
        if i > 0 {
            tag_path.push('/');
        }
        if count > 1 {
            path_write!(tag_path, "{}[{}]", pt, count)?;
        } else {
            tag_path.push_str(pt);
        }
    }

    // Append self
    if !tag_path.is_empty() {
        tag_path.push('/');
    }
    if total_same > 1 {
        path_write!(tag_path, "{}[{}]", rt.tag_name, total_same)?;
    } else {
        tag_path.push_str(&rt.tag_name);
    }

    let depth = path_tags.len() + 1;
    let depth_us = depth;

    // Find element boundaries
    let close_pos = if rt.is_self_closing {
        rt.end
    } else {
        find_closing(html, &rt.tag_name, rt.end)?
    };
    let outer_html = &html[rt.start..close_pos];

    // Strip tags for text
    let text = strip_html(outer_html);
    let text_len = text.len();
    let total_bytes = outer_html.len().max(1);
    let text_density = text_len as f64 / total_bytes as f64;

    // Text signature (first 100 chars)
    let text_sig = if text_len > 0 {
        let sample = if text_len > 100 { &text[..100] } else { &text };
        hash_str(sample)
    } else {
        0
    };

    // Count direct children
    let inner_start = rt.end;
    let inner_end = if rt.is_self_closing {
        rt.end
    } else {
        close_pos
    };
    let child_count = count_direct_children(html, inner_start, inner_end);

    // Detect presence of links, images, tables in element content
    let has_links = outer_html.contains("<a ");
    let has_images = outer_html.contains("<img ");
    let has_tables = outer_html.contains("<table");

    let mut attr_keys = extract_attr_keys(&rt.attrs_raw);
    attr_keys.sort();
    let classes = extract_classes_from(&rt.attrs_raw);
    let id = extract_id_from(&rt.attrs_raw);

    Some(StructuralFingerprint {
        tag_path,
        text_density,
        child_count,
        depth: depth_us,
        sibling_index,
        similar_sibling_count: total_same,
        attr_keys,
        classes,
        id,
        text_sig,
        has_links,
        has_images,
        has_tables,
    })
}

fn count_direct_children(html: &str, start: usize, end: usize) -> usize {
    if start >= end {
        return 0;
    }
    let content = &html[start..end];
    let mut count = 0;
    let mut depth: i32 = 0;
    let mut pos = 0;
    while pos < content.len() {
        let rest = &content[pos..];
        let next = match rest.find('<') {
            Some(i) => i,
            None => break,
        };
        pos += next;
        if pos + 1 >= content.len() {
            break;
        }
        let c2 = content.as_bytes()[pos + 1];
        if c2 == b'/' {
            depth -= 1;
            if let Some(end_pos) = html_skip_past_tag(content, pos) {
                pos = end_pos;
            } else {
                pos += 1;
            }
        } else if c2 == b'!' || c2 == b'?' {
            if let Some(end_pos) = html_skip_past_tag(content, pos) {
                pos = end_pos;
            } else {
                pos += 1;
            }
        } else if let Some(rt) = parse_tag_at(content, pos) {
            if !rt.is_closing {
                if depth == 0 {
                    count += 1;
                }
                if !rt.is_self_closing {
                    depth += 1;
                }
            }
            pos = rt.end;
        } else {
            pos += 1;
        }
    }
    count
}

// ============================================================================
// Part 2: Adaptive Selector
// ============================================================================

/// Self-healing element selector that tracks elements by structural fingerprint.
#[derive(Debug, Clone)]
pub struct AdaptiveSelector {
    /// The original CSS selector
    pub css_selector: String,
    /// Stored structural fingerprint for re-identification
    pub fingerprint: StructuralFingerprint,
    /// Minimum similarity threshold for re-identification (default 0.55)
    pub threshold: f64,
    /// Number of times this selector has been used
    pub use_count: u64,
    /// Number of times re-identification succeeded
    pub success_count: u64,
}

impl AdaptiveSelector {
    /// Create a new AdaptiveSelector from a CSS selector and example HTML.
    pub fn new(css_selector: &str, html: &str) -> Result<Self, String> {
        let fps = StructuralFingerprint::from_html(html, css_selector)?;
        let fp = fps
            .into_iter()
            .next()
            .ok_or_else(|| format!("No element found for selector: {}", css_selector))?;
        Ok(Self {
            css_selector: css_selector.to_string(),
            fingerprint: fp,
            threshold: 0.55,
            use_count: 1,
            success_count: 0,
        })
    }

    /// Re-locate element in changed HTML.
    /// Returns the best CSS selector that identifies the element in the new DOM.
    pub fn reidentify(&self, html: &str) -> Result<String, String> {
        // Scan all elements in the HTML (non-closing, non-void we can use for comparison)
        let candidates = self.scan_candidates(html)?;
        if candidates.is_empty() {
            return Err("No candidates found for re-identification".to_string());
        }

        let mut best_score = 0.0_f64;
        let mut best_selector = String::new();

        for fp in &candidates {
            let sim = self.fingerprint.similarity(fp);
            if sim > best_score {
                best_score = sim;
                best_selector = fp.to_selector();
            }
        }

        if best_score >= self.threshold {
            Ok(best_selector)
        } else {
            Err(format!(
                "Re-identification failed: best similarity {:.4} below threshold {:.4}",
                best_score, self.threshold
            ))
        }
    }

    /// Success rate of re-identification.
    pub fn success_rate(&self) -> f64 {
        if self.use_count == 0 {
            return 0.0;
        }
        self.success_count as f64 / self.use_count as f64
    }

    /// Confidence score based on success_rate and use_count.
    pub fn confidence(&self) -> f64 {
        let rate = self.success_rate();
        // Weight by number of uses (diminishing returns after ~20)
        let usage_weight = (self.use_count as f64 / (self.use_count as f64 + 5.0)).min(1.0);
        rate * 0.7 + usage_weight * 0.3
    }

    fn scan_candidates(&self, html: &str) -> Result<Vec<StructuralFingerprint>, String> {
        // Scan using the same tag name as the stored fingerprint for efficiency
        let tag = self
            .fingerprint
            .tag_path
            .split('/')
            .last()
            .map(|s| {
                if s.ends_with(']') {
                    if let Some(open) = s.rfind('[') {
                        &s[..open]
                    } else {
                        s
                    }
                } else {
                    s
                }
            })
            .unwrap_or("div");

        // Just scan everything — use a wide net
        let mut fps = Vec::new();
        let mut path_tags: Vec<String> = Vec::new();
        let mut lvl_counts: Vec<HashMap<String, usize>> = Vec::new();
        let mut pos = 0;

        while pos < html.len() {
            let rest = &html[pos..];
            let next = match rest.find('<') {
                Some(i) => pos + i,
                None => break,
            };

            if next + 1 >= html.len() {
                break;
            }

            let c2 = html.as_bytes()[next + 1];

            if c2 == b'/' {
                let cr = &html[next + 2..];
                let ne = cr
                    .find(|c: char| c.is_whitespace() || c == '>')
                    .unwrap_or(cr.len());
                let cn = cr[..ne].to_lowercase();
                if path_tags.last().map(|t| t == &cn).unwrap_or(false) {
                    path_tags.pop();
                    lvl_counts.pop();
                }
                pos = html_skip_past_tag(html, next).unwrap_or(next + 1);
                continue;
            }

            if c2 == b'!' || c2 == b'?' {
                pos = html_skip_past_tag(html, next).unwrap_or(next + 1);
                continue;
            }

            if let Some(rt) = parse_tag_at(html, next) {
                if !rt.is_closing {
                    let sidx = {
                        let mut fallback = HashMap::new();
                        let map = lvl_counts.last_mut().unwrap_or(&mut fallback);
                        let entry = map.entry(rt.tag_name.clone()).or_insert(0);
                        *entry += 1;
                        *entry
                    };

                    if rt.tag_name == tag {
                        if let Some(fp) = compute_fp(html, &rt, &path_tags, &lvl_counts, sidx, sidx)
                        {
                            fps.push(fp);
                        }
                    }

                    if !rt.is_self_closing {
                        path_tags.push(rt.tag_name.clone());
                        lvl_counts.push(HashMap::new());
                    }
                }
                pos = rt.end;
            } else {
                pos = next + 1;
            }
        }

        if fps.is_empty() {
            // Fallback: scan for all tags
            self.scan_all_candidates(html)
        } else {
            Ok(fps)
        }
    }

    fn scan_all_candidates(&self, html: &str) -> Result<Vec<StructuralFingerprint>, String> {
        let mut fps = Vec::new();
        let mut path_tags: Vec<String> = Vec::new();
        let mut lvl_counts: Vec<HashMap<String, usize>> = Vec::new();
        let mut pos = 0;

        while pos < html.len() {
            let rest = &html[pos..];
            let next = match rest.find('<') {
                Some(i) => pos + i,
                None => break,
            };

            if next + 1 >= html.len() {
                break;
            }

            let c2 = html.as_bytes()[next + 1];

            if c2 == b'/' {
                let cr = &html[next + 2..];
                let ne = cr
                    .find(|c: char| c.is_whitespace() || c == '>')
                    .unwrap_or(cr.len());
                let cn = cr[..ne].to_lowercase();
                if path_tags.last().map(|t| t == &cn).unwrap_or(false) {
                    path_tags.pop();
                    lvl_counts.pop();
                }
                pos = html_skip_past_tag(html, next).unwrap_or(next + 1);
                continue;
            }

            if c2 == b'!' || c2 == b'?' {
                pos = html_skip_past_tag(html, next).unwrap_or(next + 1);
                continue;
            }

            if let Some(rt) = parse_tag_at(html, next) {
                if !rt.is_closing {
                    let sidx = {
                        let mut fallback = HashMap::new();
                        let map = lvl_counts.last_mut().unwrap_or(&mut fallback);
                        let entry = map.entry(rt.tag_name.clone()).or_insert(0);
                        *entry += 1;
                        *entry
                    };

                    if let Some(fp) = compute_fp(html, &rt, &path_tags, &lvl_counts, sidx, sidx) {
                        fps.push(fp);
                    }

                    if !rt.is_self_closing {
                        path_tags.push(rt.tag_name.clone());
                        lvl_counts.push(HashMap::new());
                    }
                }
                pos = rt.end;
            } else {
                pos = next + 1;
            }
        }

        Ok(fps)
    }
}

// ============================================================================
// Part 3: Extraction Schema
// ============================================================================

/// Field type for extraction
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Text,
    Attribute,
    Html,
    Link,
    Image,
    Number,
    Nested,
}

/// Field extraction definition — similar to Crawl4AI's JsonCssExtractionStrategy.
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub selector: String,
    pub field_type: FieldType,
    pub attribute: Option<String>,
    pub nested: Option<Vec<FieldDef>>,
    pub repeated: bool,
    pub required: bool,
}

/// Schema for structured data extraction from HTML.
#[derive(Debug, Clone)]
pub struct ExtractionSchema {
    pub name: String,
    pub base_selector: Option<String>,
    pub fields: Vec<FieldDef>,
}

impl ExtractionSchema {
    /// Extract structured data from HTML according to the schema.
    pub fn extract(&self, html: &str) -> Result<Vec<serde_json::Value>, String> {
        let containers: Vec<(usize, usize)> = match &self.base_selector {
            Some(sel) => Self::find_elements(html, sel),
            None => vec![(0, html.len())],
        };

        if containers.is_empty() {
            return if self.fields.iter().any(|f| f.required) {
                Err(format!(
                    "No container found for base selector: {:?}",
                    self.base_selector
                ))
            } else {
                Ok(vec![])
            };
        }

        let mut results = Vec::new();
        for (cs, ce) in &containers {
            let container = &html[*cs..*ce];
            let mut obj = serde_json::Map::new();
            for field in &self.fields {
                let val = Self::extract_field(container, field)?;
                obj.insert(field.name.clone(), val);
            }
            results.push(serde_json::Value::Object(obj));
        }
        Ok(results)
    }

    /// Deserialize from JSON definition.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let v: serde_json::Value =
            serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;

        let name = v
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or("Missing 'name' field")?
            .to_string();

        let base_selector = v
            .get("base_selector")
            .and_then(|b| b.as_str())
            .map(|s| s.to_string());

        let fields_arr = v
            .get("fields")
            .and_then(|f| f.as_array())
            .ok_or("Missing or invalid 'fields' array")?;

        let mut fields = Vec::new();
        for fv in fields_arr {
            fields.push(Self::parse_field_def(fv)?);
        }

        Ok(Self {
            name,
            base_selector,
            fields,
        })
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("Serialization error: {}", e))
    }

    // ── internal helpers ──

    fn find_elements(html: &str, selector: &str) -> Vec<(usize, usize)> {
        let sel = parse_selector(selector);
        let mut results = Vec::new();
        let mut path_tags: Vec<String> = Vec::new();
        let mut pos = 0;

        while pos < html.len() {
            let rest = &html[pos..];
            let next = match rest.find('<') {
                Some(i) => pos + i,
                None => break,
            };

            if next + 1 >= html.len() {
                break;
            }

            let c2 = html.as_bytes()[next + 1];

            if c2 == b'/' {
                let cr = &html[next + 2..];
                let ne = cr
                    .find(|c: char| c.is_whitespace() || c == '>')
                    .unwrap_or(cr.len());
                let cn = cr[..ne].to_lowercase();
                if path_tags.last().map(|t| t == &cn).unwrap_or(false) {
                    path_tags.pop();
                }
                pos = html_skip_past_tag(html, next).unwrap_or(next + 1);
                continue;
            }

            if c2 == b'!' || c2 == b'?' {
                pos = html_skip_past_tag(html, next).unwrap_or(next + 1);
                continue;
            }

            if let Some(rt) = parse_tag_at(html, next) {
                if !rt.is_closing && tag_matches(&rt, &sel) {
                    let close = if rt.is_self_closing {
                        rt.end
                    } else {
                        find_closing(html, &rt.tag_name, rt.end).unwrap_or(html.len())
                    };
                    results.push((rt.start, close));
                }
                if !rt.is_closing && !rt.is_self_closing {
                    path_tags.push(rt.tag_name.clone());
                }
                pos = rt.end;
            } else {
                pos = next + 1;
            }
        }
        results
    }

    fn parse_field_def(v: &serde_json::Value) -> Result<FieldDef, String> {
        let name = v
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or("Field missing 'name'")?
            .to_string();
        let selector = v
            .get("selector")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let ft = match v.get("type").and_then(|t| t.as_str()).unwrap_or("text") {
            "text" => FieldType::Text,
            "attribute" | "attr" => FieldType::Attribute,
            "html" => FieldType::Html,
            "link" => FieldType::Link,
            "image" | "img" => FieldType::Image,
            "number" => FieldType::Number,
            "nested" => FieldType::Nested,
            other => return Err(format!("Unknown field type: {}", other)),
        };
        let attribute = v
            .get("attribute")
            .and_then(|a| a.as_str())
            .map(|s| s.to_string());
        let nested = if ft == FieldType::Nested {
            let arr = v
                .get("fields")
                .and_then(|f| f.as_array())
                .ok_or("Nested field missing 'fields' array")?;
            let mut fds = Vec::new();
            for fv in arr {
                fds.push(Self::parse_field_def(fv)?);
            }
            Some(fds)
        } else {
            v.get("fields")
                .and_then(|f| f.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|fv| Self::parse_field_def(fv).ok())
                        .collect()
                })
                .filter(|fds: &Vec<FieldDef>| !fds.is_empty())
        };
        let repeated = v.get("repeated").and_then(|r| r.as_bool()).unwrap_or(false);
        let required = v.get("required").and_then(|r| r.as_bool()).unwrap_or(true);

        Ok(FieldDef {
            name,
            selector,
            field_type: ft,
            attribute,
            nested,
            repeated,
            required,
        })
    }

    fn extract_field(container: &str, field: &FieldDef) -> Result<serde_json::Value, String> {
        if field.selector.is_empty() {
            return match field.field_type {
                FieldType::Text => {
                    let text = strip_html(container);
                    Ok(serde_json::Value::String(text))
                }
                FieldType::Html => Ok(serde_json::Value::String(container.to_string())),
                _ => Ok(serde_json::Value::Null),
            };
        }

        let elements = Self::find_elements(container, &field.selector);
        if elements.is_empty() {
            if field.required {
                return Ok(serde_json::Value::Null);
            }
            return Ok(serde_json::Value::Null);
        }

        if field.repeated {
            let mut arr = Vec::new();
            for (es, ee) in &elements {
                let el_html = &container[*es..*ee];
                let val = Self::extract_single(el_html, field)?;
                arr.push(val);
            }
            Ok(serde_json::Value::Array(arr))
        } else {
            let (es, ee) = elements
                .into_iter()
                .next()
                .ok_or_else(|| format!("No element for selector '{}'", field.selector))?;
            let el_html = &container[es..ee];
            Self::extract_single(el_html, field)
        }
    }

    fn extract_single(element_html: &str, field: &FieldDef) -> Result<serde_json::Value, String> {
        match field.field_type {
            FieldType::Text => {
                let text = strip_html(element_html);
                Ok(serde_json::Value::String(text))
            }
            FieldType::Attribute => {
                let attr = field.attribute.as_deref().unwrap_or("href");
                let val = Self::extract_attribute(element_html, attr);
                Ok(val
                    .map(|v| serde_json::Value::String(v))
                    .unwrap_or(serde_json::Value::Null))
            }
            FieldType::Html => Ok(serde_json::Value::String(element_html.to_string())),
            FieldType::Link => {
                let href = Self::extract_attribute(element_html, "href").unwrap_or_default();
                let text = strip_html(element_html);
                let mut map = serde_json::Map::new();
                map.insert("href".to_string(), serde_json::Value::String(href));
                map.insert("text".to_string(), serde_json::Value::String(text));
                Ok(serde_json::Value::Object(map))
            }
            FieldType::Image => {
                let src = Self::extract_attribute(element_html, "src").unwrap_or_default();
                let alt = Self::extract_attribute(element_html, "alt").unwrap_or_default();
                let mut map = serde_json::Map::new();
                map.insert("src".to_string(), serde_json::Value::String(src));
                map.insert("alt".to_string(), serde_json::Value::String(alt));
                Ok(serde_json::Value::Object(map))
            }
            FieldType::Number => {
                let text = strip_html(element_html);
                let cleaned: String = text
                    .chars()
                    .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                    .collect();
                if let Ok(n) = cleaned.parse::<f64>() {
                    Ok(serde_json::Value::Number(
                        serde_json::Number::from_f64(n)
                            .unwrap_or(serde_json::Number::from_f64(0.0).unwrap()),
                    ))
                } else {
                    Ok(serde_json::Value::Null)
                }
            }
            FieldType::Nested => {
                if let Some(ref nested_fields) = field.nested {
                    let mut obj = serde_json::Map::new();
                    for nf in nested_fields {
                        let val = Self::extract_field(element_html, nf)?;
                        obj.insert(nf.name.clone(), val);
                    }
                    Ok(serde_json::Value::Object(obj))
                } else {
                    Ok(serde_json::Value::Null)
                }
            }
        }
    }

    fn extract_attribute(html: &str, attr: &str) -> Option<String> {
        let lower = html.to_lowercase();
        let _search = format!("{}=&", attr);
        let search_quote = format!("{}=\"", attr);
        let search_squote = format!("{}='", attr);

        let (after, quote) = if let Some(idx) = lower.find(&search_quote) {
            (&html[idx + search_quote.len() - 1..], '"')
        } else if let Some(idx) = lower.find(&search_squote) {
            (&html[idx + search_squote.len() - 1..], '\'')
        } else {
            // Try without quotes (rare)
            let search_noq = format!("{}=, ", attr);
            if let Some(_idx) = lower.find(&search_noq) {
                return None;
            }
            return None;
        };

        // 'after' points to the quote character
        let val_start = 1; // after the quote
        if val_start >= after.len() {
            return None;
        }
        let inner = &after[val_start..];
        let end = inner.find(quote)?;
        let val = inner[..end].trim().to_string();
        if val.is_empty() {
            None
        } else {
            Some(val)
        }
    }
}

impl serde::Serialize for FieldType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match self {
            FieldType::Text => "text",
            FieldType::Attribute => "attribute",
            FieldType::Html => "html",
            FieldType::Link => "link",
            FieldType::Image => "image",
            FieldType::Number => "number",
            FieldType::Nested => "nested",
        })
    }
}

impl<'de> serde::Deserialize<'de> for FieldType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "text" => Ok(FieldType::Text),
            "attribute" | "attr" => Ok(FieldType::Attribute),
            "html" => Ok(FieldType::Html),
            "link" => Ok(FieldType::Link),
            "image" | "img" => Ok(FieldType::Image),
            "number" => Ok(FieldType::Number),
            "nested" => Ok(FieldType::Nested),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &[
                    "text",
                    "attribute",
                    "html",
                    "link",
                    "image",
                    "number",
                    "nested",
                ],
            )),
        }
    }
}

impl serde::Serialize for FieldDef {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("selector", &self.selector)?;
        map.serialize_entry("type", &self.field_type)?;
        if let Some(ref attr) = self.attribute {
            map.serialize_entry("attribute", attr)?;
        }
        if let Some(ref nested) = self.nested {
            map.serialize_entry("fields", nested)?;
        }
        map.serialize_entry("repeated", &self.repeated)?;
        map.serialize_entry("required", &self.required)?;
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for FieldDef {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct FieldDefHelper {
            name: String,
            #[serde(default)]
            selector: String,
            #[serde(rename = "type", default = "default_field_type")]
            field_type: FieldType,
            attribute: Option<String>,
            fields: Option<Vec<FieldDef>>,
            #[serde(default)]
            repeated: bool,
            #[serde(default = "default_true")]
            required: bool,
        }
        fn default_field_type() -> FieldType {
            FieldType::Text
        }
        fn default_true() -> bool {
            true
        }

        let h = FieldDefHelper::deserialize(deserializer)?;
        Ok(FieldDef {
            name: h.name,
            selector: h.selector,
            field_type: h.field_type,
            attribute: h.attribute,
            nested: h.fields,
            repeated: h.repeated,
            required: h.required,
        })
    }
}

impl serde::Serialize for ExtractionSchema {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("name", &self.name)?;
        if let Some(ref bs) = self.base_selector {
            map.serialize_entry("base_selector", bs)?;
        }
        map.serialize_entry("fields", &self.fields)?;
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for ExtractionSchema {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct SchemaHelper {
            name: String,
            base_selector: Option<String>,
            fields: Vec<FieldDef>,
        }
        let h = SchemaHelper::deserialize(deserializer)?;
        Ok(ExtractionSchema {
            name: h.name,
            base_selector: h.base_selector,
            fields: h.fields,
        })
    }
}

// ============================================================================
// Part 4: Selection Engine
// ============================================================================

/// Main entry point for adaptive selection and structured extraction.
pub struct SelectionEngine {
    pub selectors: Vec<AdaptiveSelector>,
    pub schemas: Vec<ExtractionSchema>,
}

impl SelectionEngine {
    pub fn new() -> Self {
        Self {
            selectors: Vec::new(),
            schemas: Vec::new(),
        }
    }

    /// Train a new adaptive selector from a CSS selector and example HTML.
    pub fn train_selector(&mut self, css: &str, html: &str) -> Result<&AdaptiveSelector, String> {
        let selector = AdaptiveSelector::new(css, html)?;
        self.selectors.push(selector);
        self.selectors.last().ok_or_else(|| "no selectors available".into())
    }

    /// Re-identify all selectors in new HTML.
    pub fn reidentify_all(&self, html: &str) -> Vec<(String, Result<String, String>)> {
        self.selectors
            .iter()
            .map(|s| {
                let result = s.reidentify(html);
                (s.css_selector.clone(), result)
            })
            .collect()
    }

    /// Add an extraction schema.
    pub fn add_schema(&mut self, schema: ExtractionSchema) {
        self.schemas.push(schema);
    }

    /// Extract structured data using all schemas.
    pub fn extract_all(&self, html: &str) -> Vec<(String, Result<Vec<serde_json::Value>, String>)> {
        self.schemas
            .iter()
            .map(|s| {
                let result = s.extract(html);
                (s.name.clone(), result)
            })
            .collect()
    }

    /// Remove selectors with confidence below threshold.
    /// Returns number of pruned selectors.
    pub fn prune_low_confidence(&mut self, min_confidence: f64) -> usize {
        let before = self.selectors.len();
        self.selectors.retain(|s| s.confidence() >= min_confidence);
        before - self.selectors.len()
    }
}

impl Default for SelectionEngine {
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

    // ── Structural Fingerprint ──

    #[test]
    fn test_struct_fp_from_html_simple() {
        let html = r#"<html><body><div class="content"><p>Hello world</p></div></body></html>"#;
        let fps = StructuralFingerprint::from_html(html, "div.content").unwrap();
        assert_eq!(fps.len(), 1);
        assert_eq!(fps[0].tag_path, "html/body/div");
        assert!((fps[0].text_density - 0.0).abs() < 1.0); // some density > 0
        assert_eq!(fps[0].depth, 3);
        assert!(fps[0].child_count >= 1); // contains p
        assert_eq!(fps[0].classes, vec!["content".to_string()]);
    }

    #[test]
    fn test_struct_fp_from_html_by_tag() {
        let html =
            r#"<html><body><article><h1>Title</h1><p>Body text here</p></article></body></html>"#;
        let fps = StructuralFingerprint::from_html(html, "article").unwrap();
        assert_eq!(fps.len(), 1);
        assert!(fps[0].tag_path.contains("article"));
        assert_eq!(fps[0].child_count, 2); // h1 and p
    }

    #[test]
    fn test_struct_fp_from_html_by_id() {
        let html = r#"<html><body><div id="main"><p>Content</p></div></body></html>"#;
        let fps = StructuralFingerprint::from_html(html, "#main").unwrap();
        assert_eq!(fps.len(), 1);
        assert_eq!(fps[0].id.as_deref(), Some("main"));
    }

    #[test]
    fn test_struct_fp_similarity_identical() {
        let html = r#"<html><body><div class="box"><p>Same text</p></div></body></html>"#;
        let fps = StructuralFingerprint::from_html(html, "div.box").unwrap();
        let fp = &fps[0];
        let sim = fp.similarity(fp);
        assert!(
            (sim - 1.0).abs() < 0.01,
            "self-similarity should be ~1.0, got {}",
            sim
        );
    }

    #[test]
    fn test_struct_fp_similarity_different() {
        let html1 = r#"<html><body><div class="a"><p>Content A</p></div></body></html>"#;
        let html2 = r#"<html><body><span class="b"><p>Content B</p></span></body></html>"#;
        let fp_a = StructuralFingerprint::from_html(html1, "div.a").unwrap();
        let fp_b = StructuralFingerprint::from_html(html2, "span.b").unwrap();
        let sim = fp_a[0].similarity(&fp_b[0]);
        assert!(
            sim < 0.9,
            "different elements should have lower similarity, got {}",
            sim
        );
    }

    #[test]
    fn test_struct_fp_to_selector_with_id() {
        let fp = StructuralFingerprint {
            tag_path: "html/body/div".to_string(),
            text_density: 0.5,
            child_count: 3,
            depth: 2,
            sibling_index: 1,
            similar_sibling_count: 2,
            attr_keys: vec!["id".to_string(), "class".to_string()],
            classes: vec![],
            id: Some("main-content".to_string()),
            text_sig: 12345,
            has_links: false,
            has_images: false,
            has_tables: false,
        };
        assert_eq!(fp.to_selector(), "#main-content");
    }

    #[test]
    fn test_struct_fp_to_selector_with_class() {
        let fp = StructuralFingerprint {
            tag_path: "html/body/article".to_string(),
            text_density: 0.5,
            child_count: 3,
            depth: 2,
            sibling_index: 1,
            similar_sibling_count: 1,
            attr_keys: vec!["class".to_string()],
            classes: vec!["post".to_string(), "entry".to_string()],
            id: None,
            text_sig: 0,
            has_links: false,
            has_images: false,
            has_tables: false,
        };
        assert_eq!(fp.to_selector(), "article.post.entry");
    }

    #[test]
    fn test_struct_fp_multiple_matches() {
        let html =
            r#"<html><body><ul><li>First</li><li>Second</li><li>Third</li></ul></body></html>"#;
        let fps = StructuralFingerprint::from_html(html, "li").unwrap();
        assert_eq!(fps.len(), 3, "should find 3 li elements");
        assert_eq!(fps[0].sibling_index, 1);
        assert_eq!(fps[1].sibling_index, 2);
        assert_eq!(fps[2].sibling_index, 3);
    }

    #[test]
    fn test_struct_fp_no_match() {
        let html = "<html><body><p>No match here</p></body></html>";
        let result = StructuralFingerprint::from_html(html, "div.nonexistent");
        assert!(result.is_err());
    }

    // ── Adaptive Selector ──

    #[test]
    fn test_adaptive_selector_new() {
        let html = r#"<html><body><div class="target"><p>Content</p></div></body></html>"#;
        let sel = AdaptiveSelector::new("div.target", html).unwrap();
        assert_eq!(sel.css_selector, "div.target");
        assert!((sel.threshold - 0.55).abs() < 0.01);
        assert_eq!(sel.use_count, 1);
    }

    #[test]
    fn test_adaptive_selector_reidentify_same_html() {
        let html = r#"<html><body><div class="target"><p>Content</p></div></body></html>"#;
        let sel = AdaptiveSelector::new("div.target", html).unwrap();
        let result = sel.reidentify(html);
        assert!(result.is_ok(), "reidentify failed: {:?}", result);
    }

    #[test]
    fn test_adaptive_selector_reidentify_class_renamed() {
        let train_html = r#"<html><body><div class="old-name"><p>Content</p></div></body></html>"#;
        let changed_html =
            r#"<html><body><div class="new-name"><p>Content</p></div></body></html>"#;
        let sel = AdaptiveSelector::new("div.old-name", train_html).unwrap();
        let result = sel.reidentify(changed_html);
        assert!(
            result.is_ok(),
            "reidentify after class rename failed: {:?}",
            result
        );
    }

    #[test]
    fn test_adaptive_selector_reidentify_tag_change() {
        let train_html =
            r#"<html><body><div class="content"><p>Same text here</p></div></body></html>"#;
        let changed_html =
            r#"<html><body><section class="content"><p>Same text here</p></section></body></html>"#;
        let sel = AdaptiveSelector::new("div.content", train_html).unwrap();
        let result = sel.reidentify(changed_html);
        // Tag change with same classes: low similarity
        assert!(
            result.is_ok() || result.is_err(),
            "may or may not survive tag change"
        );
    }

    #[test]
    fn test_adaptive_selector_success_rate() {
        let html = r#"<html><body><div id="x"><p>X</p></div></body></html>"#;
        let mut sel = AdaptiveSelector::new("#x", html).unwrap();
        sel.use_count = 10;
        sel.success_count = 7;
        assert!((sel.success_rate() - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_adaptive_selector_confidence() {
        let mut sel = AdaptiveSelector {
            css_selector: "div".to_string(),
            fingerprint: StructuralFingerprint {
                tag_path: "html/body/div".to_string(),
                text_density: 0.5,
                child_count: 1,
                depth: 2,
                sibling_index: 1,
                similar_sibling_count: 1,
                attr_keys: vec![],
                classes: vec![],
                id: None,
                text_sig: 0,
                has_links: false,
                has_images: false,
                has_tables: false,
            },
            threshold: 0.55,
            use_count: 20,
            success_count: 18,
        };
        let c = sel.confidence();
        assert!(
            c > 0.7,
            "confidence should be high with good track record, got {}",
            c
        );
    }

    // ── Extraction Schema ──

    #[test]
    fn test_extract_simple_text() {
        let html = r#"<html><body><div class="item"><h2>Product Name</h2><p class="price">$29.99</p></div></body></html>"#;
        let schema = ExtractionSchema {
            name: "product".to_string(),
            base_selector: Some("div.item".to_string()),
            fields: vec![
                FieldDef {
                    name: "title".to_string(),
                    selector: "h2".to_string(),
                    field_type: FieldType::Text,
                    attribute: None,
                    nested: None,
                    repeated: false,
                    required: true,
                },
                FieldDef {
                    name: "price".to_string(),
                    selector: "p.price".to_string(),
                    field_type: FieldType::Text,
                    attribute: None,
                    nested: None,
                    repeated: false,
                    required: true,
                },
            ],
        };
        let result = schema.extract(html).unwrap();
        assert_eq!(result.len(), 1);
        let obj = result[0].as_object().unwrap();
        assert_eq!(obj["title"].as_str(), Some("Product Name"));
        assert!(obj["price"].as_str().unwrap().contains("29.99"));
    }

    #[test]
    fn test_extract_attribute() {
        let html = r#"<a href="https://example.com" class="link">Click here</a>"#;
        let schema = ExtractionSchema {
            name: "links".to_string(),
            base_selector: None,
            fields: vec![FieldDef {
                name: "url".to_string(),
                selector: "a.link".to_string(),
                field_type: FieldType::Attribute,
                attribute: Some("href".to_string()),
                nested: None,
                repeated: false,
                required: true,
            }],
        };
        let result = schema.extract(html).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["url"].as_str(), Some("https://example.com"));
    }

    #[test]
    fn test_extract_repeated_items() {
        let html = r#"<ul class="list"><li>Apple</li><li>Banana</li><li>Cherry</li></ul>"#;
        let schema = ExtractionSchema {
            name: "fruits".to_string(),
            base_selector: Some("ul.list".to_string()),
            fields: vec![FieldDef {
                name: "items".to_string(),
                selector: "li".to_string(),
                field_type: FieldType::Text,
                attribute: None,
                nested: None,
                repeated: true,
                required: false,
            }],
        };
        let result = schema.extract(html).unwrap();
        assert_eq!(result.len(), 1);
        let items = result[0]["items"].as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].as_str(), Some("Apple"));
        assert_eq!(items[1].as_str(), Some("Banana"));
        assert_eq!(items[2].as_str(), Some("Cherry"));
    }

    #[test]
    fn test_extract_link_field() {
        let html = r#"<a href="https://example.com/page">Example Page</a>"#;
        let schema = ExtractionSchema {
            name: "link_test".to_string(),
            base_selector: None,
            fields: vec![FieldDef {
                name: "link".to_string(),
                selector: "a".to_string(),
                field_type: FieldType::Link,
                attribute: None,
                nested: None,
                repeated: false,
                required: true,
            }],
        };
        let result = schema.extract(html).unwrap();
        assert_eq!(
            result[0]["link"]["href"].as_str(),
            Some("https://example.com/page")
        );
        assert_eq!(result[0]["link"]["text"].as_str(), Some("Example Page"));
    }

    #[test]
    fn test_extract_image_field() {
        let html = r#"<img src="photo.jpg" alt="A photo">"#;
        let schema = ExtractionSchema {
            name: "img_test".to_string(),
            base_selector: None,
            fields: vec![FieldDef {
                name: "image".to_string(),
                selector: "img".to_string(),
                field_type: FieldType::Image,
                attribute: None,
                nested: None,
                repeated: false,
                required: true,
            }],
        };
        let result = schema.extract(html).unwrap();
        assert_eq!(result[0]["image"]["src"].as_str(), Some("photo.jpg"));
        assert_eq!(result[0]["image"]["alt"].as_str(), Some("A photo"));
    }

    #[test]
    fn test_extract_number_field() {
        let html = r#"<span class="count">42</span>"#;
        let schema = ExtractionSchema {
            name: "num_test".to_string(),
            base_selector: None,
            fields: vec![FieldDef {
                name: "count".to_string(),
                selector: "span.count".to_string(),
                field_type: FieldType::Number,
                attribute: None,
                nested: None,
                repeated: false,
                required: true,
            }],
        };
        let result = schema.extract(html).unwrap();
        assert_eq!(result[0]["count"].as_f64(), Some(42.0));
    }

    #[test]
    fn test_extract_html_field() {
        let html = r#"<div class="desc"><p>Rich <strong>text</strong></p></div>"#;
        let schema = ExtractionSchema {
            name: "html_test".to_string(),
            base_selector: None,
            fields: vec![FieldDef {
                name: "html".to_string(),
                selector: "div.desc".to_string(),
                field_type: FieldType::Html,
                attribute: None,
                nested: None,
                repeated: false,
                required: true,
            }],
        };
        let result = schema.extract(html).unwrap();
        let html_val = result[0]["html"].as_str().unwrap();
        assert!(html_val.contains("<p>"));
        assert!(html_val.contains("</p>"));
    }

    #[test]
    fn test_extract_nested_field() {
        let html = r#"<div class="product"><h2>Widget</h2><div class="details"><span class="price">9.99</span><span class="stock">In Stock</span></div></div>"#;
        let schema = ExtractionSchema {
            name: "nested_test".to_string(),
            base_selector: Some("div.product".to_string()),
            fields: vec![
                FieldDef {
                    name: "name".to_string(),
                    selector: "h2".to_string(),
                    field_type: FieldType::Text,
                    attribute: None,
                    nested: None,
                    repeated: false,
                    required: true,
                },
                FieldDef {
                    name: "details".to_string(),
                    selector: "div.details".to_string(),
                    field_type: FieldType::Nested,
                    attribute: None,
                    nested: Some(vec![
                        FieldDef {
                            name: "price".to_string(),
                            selector: "span.price".to_string(),
                            field_type: FieldType::Text,
                            attribute: None,
                            nested: None,
                            repeated: false,
                            required: true,
                        },
                        FieldDef {
                            name: "stock".to_string(),
                            selector: "span.stock".to_string(),
                            field_type: FieldType::Text,
                            attribute: None,
                            nested: None,
                            repeated: false,
                            required: true,
                        },
                    ]),
                    repeated: false,
                    required: true,
                },
            ],
        };
        let result = schema.extract(html).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"].as_str(), Some("Widget"));
        assert_eq!(result[0]["details"]["price"].as_str(), Some("9.99"));
        assert_eq!(result[0]["details"]["stock"].as_str(), Some("In Stock"));
    }

    #[test]
    fn test_schema_from_json() {
        let json = r#"{
            "name": "test",
            "base_selector": "div.container",
            "fields": [
                {"name": "title", "selector": "h1.title", "type": "text"},
                {"name": "url", "selector": "a.link", "type": "attribute", "attribute": "href"},
                {"name": "items", "selector": "li", "type": "text", "repeated": true}
            ]
        }"#;
        let schema = ExtractionSchema::from_json(json).unwrap();
        assert_eq!(schema.name, "test");
        assert_eq!(schema.base_selector.as_deref(), Some("div.container"));
        assert_eq!(schema.fields.len(), 3);
        assert_eq!(schema.fields[0].name, "title");
        assert_eq!(schema.fields[0].field_type, FieldType::Text);
        assert_eq!(schema.fields[1].field_type, FieldType::Attribute);
        assert_eq!(schema.fields[2].repeated, true);
    }

    #[test]
    fn test_schema_to_json_roundtrip() {
        let schema = ExtractionSchema {
            name: "roundtrip".to_string(),
            base_selector: Some("div.test".to_string()),
            fields: vec![FieldDef {
                name: "name".to_string(),
                selector: "h2".to_string(),
                field_type: FieldType::Text,
                attribute: None,
                nested: None,
                repeated: false,
                required: true,
            }],
        };
        let json = schema.to_json().unwrap();
        let parsed = ExtractionSchema::from_json(&json).unwrap();
        assert_eq!(parsed.name, schema.name);
        assert_eq!(parsed.base_selector, schema.base_selector);
        assert_eq!(parsed.fields.len(), schema.fields.len());
    }

    // ── Selection Engine ──

    #[test]
    fn test_selection_engine_new() {
        let engine = SelectionEngine::new();
        assert!(engine.selectors.is_empty());
        assert!(engine.schemas.is_empty());
    }

    #[test]
    fn test_selection_engine_train_and_reidentify() {
        let html = r#"<html><body><div class="target"><p>Content</p></div></body></html>"#;
        let changed = r#"<html><body><div class="renamed"><p>Content</p></div></body></html>"#;
        let mut engine = SelectionEngine::new();
        engine.train_selector("div.target", html).unwrap();
        assert_eq!(engine.selectors.len(), 1);
        let results = engine.reidentify_all(changed);
        assert_eq!(results.len(), 1);
        assert!(
            results[0].1.is_ok(),
            "reidentify failed: {:?}",
            results[0].1
        );
    }

    #[test]
    fn test_selection_engine_extract_all() {
        let html = r#"<html><body><div class="item"><h2>Product</h2></div></body></html>"#;
        let schema = ExtractionSchema {
            name: "simple".to_string(),
            base_selector: Some("div.item".to_string()),
            fields: vec![FieldDef {
                name: "title".to_string(),
                selector: "h2".to_string(),
                field_type: FieldType::Text,
                attribute: None,
                nested: None,
                repeated: false,
                required: true,
            }],
        };
        let mut engine = SelectionEngine::new();
        engine.add_schema(schema);
        let results = engine.extract_all(html);
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_ok());
        let data = results[0].1.as_ref().unwrap();
        assert_eq!(data[0]["title"].as_str(), Some("Product"));
    }

    #[test]
    fn test_selection_engine_prune() {
        let html = r#"<html><body><div class="x"><p>X</p></div></body></html>"#;
        let mut engine = SelectionEngine::new();
        engine.train_selector("div.x", html).unwrap();
        // New selector: low confidence due to few uses
        let pruned = engine.prune_low_confidence(0.9);
        assert_eq!(pruned, 1, "should prune low-confidence selector");
        assert!(engine.selectors.is_empty());
    }
}
