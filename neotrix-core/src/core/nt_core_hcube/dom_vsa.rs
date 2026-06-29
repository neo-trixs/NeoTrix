// REVIVED Task 2 — dead_code removed
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DOM_VSA_DIM: usize = 64;

// ---------------------------------------------------------------------------
// Part 5: SimpleVsaVector — self-contained bipolar (-1/+1) VSA vector
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleVsaVector {
    pub bits: Vec<i8>,
}

impl SimpleVsaVector {
    pub fn new() -> Self {
        Self {
            bits: vec![-1; DOM_VSA_DIM],
        }
    }

    pub fn from_bits(bits: Vec<i8>) -> Self {
        Self { bits }
    }

    pub fn from_seed(seed: u64) -> Self {
        let mut state = seed;
        let mut bits = Vec::with_capacity(DOM_VSA_DIM);
        for _ in 0..DOM_VSA_DIM {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            bits.push(if (state >> 31) & 1 == 0 { -1 } else { 1 });
        }
        Self { bits }
    }

    pub fn all_ones() -> Self {
        Self {
            bits: vec![1; DOM_VSA_DIM],
        }
    }

    pub fn bind(&self, other: &Self) -> Self {
        let bits: Vec<i8> = self
            .bits
            .iter()
            .zip(other.bits.iter())
            .map(|(a, b)| a * b)
            .collect();
        Self { bits }
    }

    pub fn bundle(vectors: &[&Self]) -> Self {
        if vectors.is_empty() {
            return Self::all_ones();
        }
        let mut sums = vec![0i32; DOM_VSA_DIM];
        for v in vectors {
            for (s, &b) in sums.iter_mut().zip(v.bits.iter()) {
                *s += b as i32;
            }
        }
        let bits: Vec<i8> = sums.iter().map(|&s| if s >= 0 { 1 } else { -1 }).collect();
        Self { bits }
    }

    pub fn similarity(&self, other: &Self) -> f64 {
        let dot: i32 = self
            .bits
            .iter()
            .zip(other.bits.iter())
            .map(|(a, b)| (*a as i32) * (*b as i32))
            .sum();
        (dot as f64 / DOM_VSA_DIM as f64 + 1.0) / 2.0
    }
}

impl Default for SimpleVsaVector {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Part 1: Role vectors for DOM element types
// ---------------------------------------------------------------------------

static ROLE_CACHE: OnceLock<Mutex<HashMap<String, SimpleVsaVector>>> = OnceLock::new();

pub struct DomRoles;

impl DomRoles {
    pub fn role(name: &str) -> SimpleVsaVector {
        let cache = ROLE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        let mut cache = cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(v) = cache.get(name) {
            return v.clone();
        }
        let v = string_to_vsa(name);
        cache.insert(name.to_string(), v.clone());
        v
    }

    pub fn reset_cache() {
        if let Some(cache) = ROLE_CACHE.get() {
            let mut cache = cache.lock().unwrap_or_else(|e| e.into_inner());
            cache.clear();
        }
    }
}

// ---------------------------------------------------------------------------
// Part 2: HTML parser (string-based, no deps)
// ---------------------------------------------------------------------------

/// Void elements that don't have children
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Raw elements whose content is treated as text (don't parse inner HTML)
const RAW_TEXT_ELEMENTS: &[&str] = &["script", "style", "textarea", "title"];

#[derive(Debug, Clone)]
struct HtmlNode {
    tag_name: String,
    attributes: Vec<(String, String)>,
    text_content: String,
    parent: Option<usize>,
    children: Vec<usize>,
    depth: usize,
    sibling_index: usize,
    total_siblings: usize,
}

#[derive(Debug, Clone)]
struct HtmlTree {
    nodes: Vec<HtmlNode>,
}

impl HtmlTree {
    fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    fn add_node(&mut self, node: HtmlNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }
}

fn parse_html(html: &str, max_depth: usize, max_children: usize) -> Result<HtmlTree, String> {
    let mut tree = HtmlTree::new();
    let root_idx = tree.add_node(HtmlNode {
        tag_name: "#document".into(),
        attributes: vec![],
        text_content: String::new(),
        parent: None,
        children: vec![],
        depth: 0,
        sibling_index: 0,
        total_siblings: 1,
    });

    let bytes = html.as_bytes();
    let len = bytes.len();
    let mut pos = 0;
    let mut stack: Vec<usize> = vec![root_idx];
    let mut raw_text_mode: Option<String> = None;
    let mut sibling_counters: HashMap<usize, usize> = HashMap::new();
    let mut text_buffer = String::new();

    macro_rules! flush_text {
        () => {
            if !text_buffer.is_empty() {
                if let Some(&current) = stack.last() {
                    if current < tree.nodes.len() {
                        tree.nodes[current].text_content.push_str(&text_buffer);
                    }
                }
                text_buffer.clear();
            }
        };
    }

    while pos < len {
        if let Some(ref raw_tag) = raw_text_mode {
            let closing = format!("</{}", raw_tag);
            if pos + closing.len() <= len && bytes[pos..].starts_with(closing.as_bytes()) {
                let mut end = pos + closing.len();
                while end < len && bytes[end] != b'>' {
                    end += 1;
                }
                if end < len {
                    end += 1;
                }
                raw_text_mode = None;
                pos = end;
                continue;
            }
            text_buffer.push(bytes[pos] as char);
            pos += 1;
            continue;
        }

        if bytes[pos] != b'<' {
            text_buffer.push(bytes[pos] as char);
            pos += 1;
            continue;
        }

        // < encountered
        flush_text!();

        // Check for comment <!-- -->
        if pos + 3 < len && bytes[pos..].starts_with(b"<!--") {
            if let Some(end) = find_subseq(&bytes, pos + 4, b"-->") {
                pos = end + 3;
                continue;
            }
            return Err("Unterminated comment".into());
        }

        // Check for closing </tag>
        if pos + 1 < len && bytes[pos + 1] == b'/' {
            if let Some(end) = find_subseq(&bytes, pos, b">") {
                let tag_content = std::str::from_utf8(&bytes[pos + 2..end])
                    .map_err(|_| "Invalid UTF-8 in closing tag".to_string())?
                    .trim();
                let closing_tag = tag_content
                    .split(|c: char| c.is_whitespace())
                    .next()
                    .unwrap_or(tag_content)
                    .to_lowercase();

                // Pop until we find the matching opening tag
                while let Some(&top) = stack.last() {
                    if top < tree.nodes.len()
                        && tree.nodes[top].tag_name.to_lowercase() == closing_tag
                    {
                        stack.pop();
                        break;
                    }
                    stack.pop();
                }
                pos = end + 1;
                continue;
            }
            return Err("Unterminated closing tag".into());
        }

        // Opening tag or self-closing tag
        if let Some(end) = find_subseq(&bytes, pos, b">") {
            let is_self_closing = end > 0 && bytes[end - 1] == b'/';
            let tag_bytes = if is_self_closing {
                &bytes[pos + 1..end - 1]
            } else {
                &bytes[pos + 1..end]
            };

            let tag_str =
                std::str::from_utf8(tag_bytes).map_err(|_| "Invalid UTF-8 in tag".to_string())?;

            let (name, attrs) = parse_tag_parts(tag_str);
            let tag_name_lower = name.to_lowercase();
            let is_void = VOID_ELEMENTS.contains(&tag_name_lower.as_str());

            if is_self_closing || is_void {
                if let Some(&current) = stack.last() {
                    let sib_idx = sibling_counters.entry(current).or_insert(0);
                    *sib_idx += 1;
                    let idx = tree.add_node(HtmlNode {
                        tag_name: name.clone(),
                        attributes: attrs,
                        text_content: String::new(),
                        parent: Some(current),
                        children: vec![],
                        depth: tree.nodes[current].depth + 1,
                        sibling_index: *sib_idx,
                        total_siblings: 0,
                    });
                    tree.nodes[current].children.push(idx);
                }
                pos = end + 1;
                continue;
            }

            if let Some(&current) = stack.last() {
                let depth = tree.nodes[current].depth + 1;
                if depth <= max_depth {
                    let sib_idx = sibling_counters.entry(current).or_insert(0);
                    *sib_idx += 1;
                    if tree.nodes[current].children.len() < max_children {
                        let idx = tree.add_node(HtmlNode {
                            tag_name: name.clone(),
                            attributes: attrs,
                            text_content: String::new(),
                            parent: Some(current),
                            children: vec![],
                            depth,
                            sibling_index: *sib_idx,
                            total_siblings: 0,
                        });
                        tree.nodes[current].children.push(idx);
                        stack.push(idx);

                        if RAW_TEXT_ELEMENTS.contains(&tag_name_lower.as_str()) {
                            raw_text_mode = Some(name.to_lowercase());
                        }
                    }
                }
            }

            pos = end + 1;
            continue;
        }

        return Err(format!("Unterminated tag at position {}", pos));
    }

    // Update total_siblings for all children
    for i in 0..tree.nodes.len() {
        let count = tree.nodes[i].children.len();
        for &child_idx in tree.nodes[i].children.clone().iter() {
            if child_idx < tree.nodes.len() {
                tree.nodes[child_idx].total_siblings = count;
            }
        }
    }

    Ok(tree)
}

fn find_subseq(haystack: &[u8], start: usize, needle: &[u8]) -> Option<usize> {
    haystack[start..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| start + p)
}

fn parse_tag_parts(tag: &str) -> (String, Vec<(String, String)>) {
    let tag = tag.trim();
    let mut parts = tag.split_whitespace();
    let name = parts.next().unwrap_or("").to_string();
    let mut attrs = Vec::new();

    for part in parts {
        let key;
        let value: String;

        if let Some(eq_pos) = part.find('=') {
            key = part[..eq_pos].to_string();
            let val_part = &part[eq_pos + 1..];

            if val_part.starts_with('"') || val_part.starts_with('\'') {
                let quote = val_part.chars().next().unwrap_or('"');
                let inner = &val_part[1..];
                value = inner.trim_end_matches(quote).to_string();
            } else {
                value = val_part.to_string();
            }
        } else {
            key = part.to_string();
            value = String::new();
        }

        if !key.is_empty() {
            attrs.push((key, value));
        }
    }

    (name, attrs)
}

fn strip_html_tags(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Part 5: Helper function — deterministic string-to-VSA
// ---------------------------------------------------------------------------

fn string_to_vsa(s: &str) -> SimpleVsaVector {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    let seed = hasher.finish();
    SimpleVsaVector::from_seed(seed)
}

// ---------------------------------------------------------------------------
// Part 2: DomVsaNode + DomVsaEncoder
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DomVsaNode {
    pub tag_role: SimpleVsaVector,
    pub content: SimpleVsaVector,
    pub attributes: SimpleVsaVector,
    pub children: Vec<DomVsaNode>,
    pub node_vector: SimpleVsaVector,
    pub depth: usize,
    pub position: (usize, usize),
}

pub struct DomVsaEncoder {
    max_depth: usize,
    max_children: usize,
    use_position: bool,
}

impl DomVsaEncoder {
    pub fn new(max_depth: usize, max_children: usize) -> Self {
        Self {
            max_depth,
            max_children,
            use_position: true,
        }
    }

    pub fn with_position(mut self, enable: bool) -> Self {
        self.use_position = enable;
        self
    }

    pub fn encode_html(&self, html: &str) -> Result<DomVsaNode, String> {
        let tree = parse_html(html, self.max_depth, self.max_children)?;
        if tree.nodes.len() <= 1 {
            return Err("Empty HTML document".into());
        }
        let root_node = &tree.nodes[0];
        if root_node.children.is_empty() {
            return Err("Empty HTML document".into());
        }
        Ok(self.tree_to_vsa(&tree, root_node.children[0]))
    }

    pub fn encode_page_fingerprint(
        &self,
        html: &str,
        url: &str,
    ) -> Result<DomPageFingerprint, String> {
        let root = self.encode_html(html)?;
        self.build_fingerprint(root, url, html)
    }

    pub fn encode_structure_only(&self, html: &str) -> Result<DomVsaNode, String> {
        let tree = parse_html(html, self.max_depth, self.max_children)?;
        if tree.nodes.len() <= 1 {
            return Err("Empty HTML document".into());
        }
        let root_node = &tree.nodes[0];
        if root_node.children.is_empty() {
            return Err("Empty HTML document".into());
        }
        Ok(self.tree_to_vsa_structure_only(&tree, root_node.children[0]))
    }

    fn tree_to_vsa(&self, tree: &HtmlTree, node_idx: usize) -> DomVsaNode {
        let node = &tree.nodes[node_idx];
        let tag_role = DomRoles::role(&node.tag_name);

        let content_v = if node.text_content.trim().is_empty() {
            None
        } else {
            let cleaned = strip_html_tags(&node.text_content);
            let trimmed = cleaned.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(SimpleVsaVector::bind(
                    &string_to_vsa("text"),
                    &string_to_vsa(trimmed),
                ))
            }
        };

        let attrs_v = if node.attributes.is_empty() {
            None
        } else {
            let attr_vecs: Vec<SimpleVsaVector> = node
                .attributes
                .iter()
                .map(|(k, v)| SimpleVsaVector::bind(&string_to_vsa(k), &string_to_vsa(v)))
                .collect();
            let refs: Vec<&SimpleVsaVector> = attr_vecs.iter().collect();
            Some(SimpleVsaVector::bundle(&refs))
        };

        let child_nodes: Vec<DomVsaNode> = node
            .children
            .iter()
            .map(|&ci| self.tree_to_vsa(tree, ci))
            .collect();

        let children_v = if child_nodes.is_empty() {
            None
        } else {
            let child_refs: Vec<&SimpleVsaVector> =
                child_nodes.iter().map(|n| &n.node_vector).collect();
            Some(SimpleVsaVector::bundle(&child_refs))
        };

        let mut inner_components: Vec<&SimpleVsaVector> = Vec::new();
        if let Some(ref c) = content_v {
            inner_components.push(c);
        }
        if let Some(ref a) = attrs_v {
            inner_components.push(a);
        }

        let inner = if inner_components.is_empty() {
            SimpleVsaVector::all_ones()
        } else {
            SimpleVsaVector::bundle(&inner_components)
        };

        let role_bound = SimpleVsaVector::bind(&tag_role, &inner);

        let node_vector = match children_v {
            Some(ref cv) => SimpleVsaVector::bundle(&[&role_bound, cv]),
            None => role_bound,
        };

        let position_offset = if self.use_position {
            node.sibling_index as f64 * 0.01
                + if node.total_siblings > 1 {
                    node.sibling_index as f64 / node.total_siblings as f64 * 0.1
                } else {
                    0.0
                }
        } else {
            0.0
        };

        let node_vector = if (position_offset - 0.0).abs() > 1e-9 {
            let pos_seed = string_to_vsa(&format!("pos_{}", position_offset));
            SimpleVsaVector::bind(&node_vector, &pos_seed)
        } else {
            node_vector
        };

        DomVsaNode {
            tag_role,
            content: content_v.unwrap_or_else(SimpleVsaVector::all_ones),
            attributes: attrs_v.unwrap_or_else(SimpleVsaVector::all_ones),
            children: child_nodes,
            node_vector,
            depth: node.depth,
            position: (node.sibling_index, node.total_siblings),
        }
    }

    fn tree_to_vsa_structure_only(&self, tree: &HtmlTree, node_idx: usize) -> DomVsaNode {
        let node = &tree.nodes[node_idx];
        let tag_role = DomRoles::role(&node.tag_name);

        let child_nodes: Vec<DomVsaNode> = node
            .children
            .iter()
            .map(|&ci| self.tree_to_vsa_structure_only(tree, ci))
            .collect();

        let children_v = if child_nodes.is_empty() {
            None
        } else {
            let child_refs: Vec<&SimpleVsaVector> =
                child_nodes.iter().map(|n| &n.node_vector).collect();
            Some(SimpleVsaVector::bundle(&child_refs))
        };

        let node_vector = match children_v {
            Some(ref cv) => SimpleVsaVector::bundle(&[&tag_role, cv]),
            None => tag_role.clone(),
        };

        DomVsaNode {
            tag_role,
            content: SimpleVsaVector::all_ones(),
            attributes: SimpleVsaVector::all_ones(),
            children: child_nodes,
            node_vector,
            depth: node.depth,
            position: (node.sibling_index, node.total_siblings),
        }
    }

    fn build_fingerprint(
        &self,
        root: DomVsaNode,
        url: &str,
        html: &str,
    ) -> Result<DomPageFingerprint, String> {
        let total_nodes = count_nodes(&root);
        let tag_histogram = self.build_tag_histogram(&root);

        let structure_root = self.encode_structure_only(html)?;

        Ok(DomPageFingerprint {
            url: url.to_string(),
            root_vector: root.node_vector,
            structure_vector: structure_root.node_vector,
            tag_histogram,
            depth: root.depth,
            total_nodes,
            created_at: Instant::now(),
        })
    }

    fn build_tag_histogram(&self, node: &DomVsaNode) -> HashMap<String, usize> {
        let mut hist = HashMap::new();
        self.collect_tags(node, &mut hist);
        hist
    }

    fn collect_tags(&self, node: &DomVsaNode, hist: &mut HashMap<String, usize>) {
        *hist.entry("unknown".into()).or_insert(0) += 1;
        for child in &node.children {
            self.collect_tags(child, hist);
        }
    }
}

fn count_nodes(node: &DomVsaNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

// ---------------------------------------------------------------------------
// Part 3: DomPageFingerprint
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DomPageFingerprint {
    pub url: String,
    pub root_vector: SimpleVsaVector,
    pub structure_vector: SimpleVsaVector,
    pub tag_histogram: HashMap<String, usize>,
    pub depth: usize,
    pub total_nodes: usize,
    pub created_at: Instant,
}

impl DomPageFingerprint {
    pub fn similarity(&self, other: &Self) -> f64 {
        self.root_vector.similarity(&other.root_vector)
    }

    pub fn structural_similarity(&self, other: &Self) -> f64 {
        self.structure_vector.similarity(&other.structure_vector)
    }

    pub fn is_duplicate(&self, other: &Self, threshold: f64) -> bool {
        self.similarity(other) > threshold
    }

    pub fn has_changed_significantly(&self, other: &Self, threshold: f64) -> bool {
        self.structural_similarity(other) < threshold
    }

    pub fn merge(&self, other: &Self) -> Self {
        let root_vector = SimpleVsaVector::bundle(&[&self.root_vector, &other.root_vector]);
        let structure_vector =
            SimpleVsaVector::bundle(&[&self.structure_vector, &other.structure_vector]);

        let mut tag_histogram = self.tag_histogram.clone();
        for (tag, count) in &other.tag_histogram {
            *tag_histogram.entry(tag.clone()).or_insert(0) += count;
        }

        DomPageFingerprint {
            url: format!("merged:{}|{}", self.url, other.url),
            root_vector,
            structure_vector,
            tag_histogram,
            depth: self.depth.max(other.depth),
            total_nodes: self.total_nodes + other.total_nodes,
            created_at: Instant::now(),
        }
    }
}

// ---------------------------------------------------------------------------
// Part 4: VsaHtmlIndex
// ---------------------------------------------------------------------------

pub struct VsaHtmlIndex {
    fingerprints: Vec<DomPageFingerprint>,
    threshold: f64,
}

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total: usize,
    pub unique_urls: usize,
    pub avg_similarity: f64,
}

impl VsaHtmlIndex {
    pub fn new(threshold: f64) -> Self {
        Self {
            fingerprints: Vec::new(),
            threshold,
        }
    }

    pub fn add(&mut self, fp: DomPageFingerprint) {
        self.fingerprints.push(fp);
    }

    pub fn find_similar(&self, fp: &DomPageFingerprint) -> Vec<&DomPageFingerprint> {
        self.fingerprints
            .iter()
            .filter(|existing| existing.similarity(fp) > self.threshold)
            .collect()
    }

    pub fn is_duplicate(&self, fp: &DomPageFingerprint) -> bool {
        self.fingerprints
            .iter()
            .any(|existing| existing.similarity(fp) > self.threshold)
    }

    pub fn prune_old(&mut self, max_count: usize) {
        if self.fingerprints.len() > max_count {
            let excess = self.fingerprints.len() - max_count;
            self.fingerprints.drain(0..excess);
        }
    }

    pub fn stats(&self) -> IndexStats {
        let total = self.fingerprints.len();
        let unique_urls = self
            .fingerprints
            .iter()
            .map(|fp| &fp.url)
            .collect::<HashSet<_>>()
            .len();
        let avg_similarity = if total <= 1 {
            0.0
        } else {
            let mut total_sim = 0.0;
            let mut count = 0;
            for i in 0..total {
                for j in (i + 1)..total {
                    total_sim += self.fingerprints[i].similarity(&self.fingerprints[j]);
                    count += 1;
                }
            }
            total_sim / count as f64
        };
        IndexStats {
            total,
            unique_urls,
            avg_similarity,
        }
    }

    pub fn len(&self) -> usize {
        self.fingerprints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fingerprints.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Part 6: Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── string_to_vsa tests ──

    #[test]
    fn test_string_to_vsa_deterministic() {
        let a = string_to_vsa("div");
        let b = string_to_vsa("div");
        assert_eq!(a, b);
    }

    #[test]
    fn test_string_to_vsa_different_inputs() {
        let a = string_to_vsa("div");
        let b = string_to_vsa("span");
        assert_ne!(a, b);
        let c = string_to_vsa("a");
        assert_ne!(a, c);
        assert_ne!(b, c);
    }

    // ── DomRoles tests ──

    #[test]
    fn test_dom_roles_consistent() {
        DomRoles::reset_cache();
        let a = DomRoles::role("div");
        let b = DomRoles::role("div");
        assert_eq!(a, b);
    }

    #[test]
    fn test_dom_roles_different() {
        DomRoles::reset_cache();
        let a = DomRoles::role("div");
        let b = DomRoles::role("span");
        assert_ne!(a, b);
        let c = DomRoles::role("h1");
        assert_ne!(a, c);
        assert_ne!(b, c);
    }

    // ── SimpleVsaVector tests ──

    #[test]
    fn test_simple_vsa_self_similarity_one() {
        let v = SimpleVsaVector::from_seed(42);
        let sim = v.similarity(&v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_simple_vsa_seed_deterministic() {
        let a = SimpleVsaVector::from_seed(42);
        let b = SimpleVsaVector::from_seed(42);
        assert_eq!(a, b);
    }

    #[test]
    fn test_simple_vsa_bind_distinct() {
        let a = SimpleVsaVector::from_seed(10);
        let b = SimpleVsaVector::from_seed(20);
        let bound = a.bind(&b);
        let sim_to_a = bound.similarity(&a);
        let sim_to_b = bound.similarity(&b);
        assert!(sim_to_a < 0.6);
        assert!(sim_to_b < 0.6);
    }

    #[test]
    fn test_simple_vsa_bundle_similarity() {
        let a = SimpleVsaVector::from_seed(10);
        let b = SimpleVsaVector::from_seed(20);
        let bundle = SimpleVsaVector::bundle(&[&a, &b]);
        let sim_a = bundle.similarity(&a);
        let sim_b = bundle.similarity(&b);
        assert!(sim_a > 0.4);
        assert!(sim_b > 0.4);
    }

    // ── HTML parser tests ──

    #[test]
    fn test_parse_simple_html() {
        let tree = parse_html("<div>hello</div>", 10, 10).unwrap();
        assert!(tree.nodes.len() >= 2);
        let root = &tree.nodes[0];
        assert_eq!(root.tag_name, "#document");
        assert_eq!(root.children.len(), 1);
        let child = &tree.nodes[root.children[0]];
        assert_eq!(child.tag_name, "div");
        assert!(child.text_content.contains("hello"));
    }

    #[test]
    fn test_parse_nested_html() {
        let html = "<div><span>text</span><p>para</p></div>";
        let tree = parse_html(html, 10, 10).unwrap();
        let root = &tree.nodes[0];
        assert_eq!(root.children.len(), 1);
        let div = &tree.nodes[root.children[0]];
        assert_eq!(div.tag_name, "div");
        assert_eq!(div.children.len(), 2);
    }

    #[test]
    fn test_parse_self_closing_tag() {
        let html = "<div><br/><img src=\"x\"/></div>";
        let tree = parse_html(html, 10, 10).unwrap();
        let div = &tree.nodes[tree.nodes[0].children[0]];
        assert_eq!(div.children.len(), 2);
        let br_tag = &tree.nodes[div.children[0]];
        assert_eq!(br_tag.tag_name, "br");
        let img_tag = &tree.nodes[div.children[1]];
        assert_eq!(img_tag.tag_name, "img");
    }

    #[test]
    fn test_parse_attributes() {
        let html = "<a href=\"https://x.com\" class=\"link\">click</a>";
        let tree = parse_html(html, 10, 10).unwrap();
        let a_tag = &tree.nodes[tree.nodes[0].children[0]];
        assert_eq!(a_tag.attributes.len(), 2);
        assert_eq!(a_tag.attributes[0].0, "href");
        assert_eq!(a_tag.attributes[1].0, "class");
    }

    #[test]
    fn test_parse_raw_text_elements() {
        let html = "<script>var x = 1 < 2;</script><div>ok</div>";
        let tree = parse_html(html, 10, 10).unwrap();
        let root = &tree.nodes[0];
        assert_eq!(root.children.len(), 2);
        let script = &tree.nodes[root.children[0]];
        assert_eq!(script.tag_name, "script");
        assert!(script.text_content.contains("var x = 1 < 2;"));
        let div = &tree.nodes[root.children[1]];
        assert_eq!(div.tag_name, "div");
    }

    // ── DomVsaEncoder tests ──

    #[test]
    fn test_encode_simple_html() {
        let encoder = DomVsaEncoder::new(10, 10);
        let node = encoder.encode_html("<div>hello world</div>").unwrap();
        assert_eq!(node.depth, 1);
        assert_eq!(node.position, (1, 1));
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_encode_nested_html() {
        let encoder = DomVsaEncoder::new(10, 10);
        let node = encoder
            .encode_html("<div><span>a</span><span>b</span></div>")
            .unwrap();
        assert_eq!(node.children.len(), 2);
        assert_eq!(node.children[0].depth, 2);
        assert_eq!(node.children[1].depth, 2);
    }

    #[test]
    fn test_encode_empty_html() {
        let encoder = DomVsaEncoder::new(10, 10);
        let result = encoder.encode_html("");
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_structure_only_no_text() {
        let encoder = DomVsaEncoder::new(10, 10);
        let full = encoder
            .encode_html("<div><span>hello</span></div>")
            .unwrap();
        let structure = encoder
            .encode_structure_only("<div><span>hello</span></div>")
            .unwrap();
        assert_eq!(full.children.len(), structure.children.len());
        assert_eq!(full.depth, structure.depth);
    }

    #[test]
    fn test_encode_page_fingerprint_creation() {
        let encoder = DomVsaEncoder::new(10, 10);
        let html = "<html><body><div>content</div></body></html>";
        let fp = encoder
            .encode_page_fingerprint(html, "https://test.com")
            .unwrap();
        assert_eq!(fp.url, "https://test.com");
        assert!(fp.total_nodes > 0);
        assert_eq!(fp.depth, 3);
    }

    // ── DomPageFingerprint tests ──

    #[test]
    fn test_page_fingerprint_similarity_identical() {
        let encoder = DomVsaEncoder::new(10, 10);
        let html = "<html><body><div>same page</div></body></html>";
        let fp1 = encoder
            .encode_page_fingerprint(html, "https://a.com")
            .unwrap();
        let fp2 = encoder
            .encode_page_fingerprint(html, "https://b.com")
            .unwrap();
        let sim = fp1.similarity(&fp2);
        assert!(
            sim > 0.8,
            "identical pages should have high similarity: {sim}"
        );
    }

    #[test]
    fn test_page_fingerprint_similarity_different() {
        let encoder = DomVsaEncoder::new(10, 10);
        let fp1 = encoder
            .encode_page_fingerprint("<div>page one</div>", "https://a.com")
            .unwrap();
        let fp2 = encoder
            .encode_page_fingerprint(
                "<article><h1>title</h1><p>paragraph</p></article>",
                "https://b.com",
            )
            .unwrap();
        let sim = fp1.similarity(&fp2);
        assert!(
            sim < 0.95,
            "different pages should have lower similarity: {sim}"
        );
    }

    #[test]
    fn test_page_fingerprint_structural_vs_full() {
        let encoder = DomVsaEncoder::new(10, 10);
        let html = "<div><span>text</span><p>more</p></div>";
        let full = encoder
            .encode_page_fingerprint(html, "https://test.com")
            .unwrap();
        let structure_only = encoder
            .encode_page_fingerprint(
                "<div><span>different</span><p>stuff</p></div>",
                "https://test2.com",
            )
            .unwrap();
        let struct_sim = full.structural_similarity(&structure_only);
        let full_sim = full.similarity(&structure_only);
        assert!(
            struct_sim > full_sim,
            "structural similarity ({struct_sim}) should be higher than full ({full_sim})"
        );
    }

    #[test]
    fn test_page_fingerprint_is_duplicate() {
        let encoder = DomVsaEncoder::new(10, 10);
        let html = "<div>same content</div>";
        let fp1 = encoder
            .encode_page_fingerprint(html, "https://a.com")
            .unwrap();
        let fp2 = encoder
            .encode_page_fingerprint(html, "https://b.com")
            .unwrap();
        assert!(fp1.is_duplicate(&fp2, 0.7));
    }

    #[test]
    fn test_page_fingerprint_has_changed() {
        let encoder = DomVsaEncoder::new(10, 10);
        let fp1 = encoder
            .encode_page_fingerprint("<div>original</div>", "https://a.com")
            .unwrap();
        let fp2 = encoder
            .encode_page_fingerprint(
                "<article>completely different structure</article>",
                "https://a.com",
            )
            .unwrap();
        assert!(fp1.has_changed_significantly(&fp2, 0.7));
    }

    #[test]
    fn test_page_fingerprint_merge() {
        let encoder = DomVsaEncoder::new(10, 10);
        let fp1 = encoder
            .encode_page_fingerprint("<div>first</div>", "https://a.com")
            .unwrap();
        let fp2 = encoder
            .encode_page_fingerprint("<div>second</div>", "https://b.com")
            .unwrap();
        let merged = fp1.merge(&fp2);
        assert!(merged.url.contains("merged:"));
        assert_eq!(merged.total_nodes, fp1.total_nodes + fp2.total_nodes);
        assert!(merged.depth >= fp1.depth);
        assert!(merged.depth >= fp2.depth);
    }

    // ── VsaHtmlIndex tests ──

    #[test]
    fn test_vsa_index_add_and_find() {
        let encoder = DomVsaEncoder::new(10, 10);
        let mut index = VsaHtmlIndex::new(0.5);
        let fp1 = encoder
            .encode_page_fingerprint("<div>page a</div>", "https://a.com")
            .unwrap();
        let fp2 = encoder
            .encode_page_fingerprint("<div>page b</div>", "https://b.com")
            .unwrap();
        index.add(fp1.clone());
        index.add(fp2.clone());

        assert_eq!(index.len(), 2);

        let similar = index.find_similar(&fp1);
        assert!(!similar.is_empty());
    }

    #[test]
    fn test_vsa_index_dedup() {
        let encoder = DomVsaEncoder::new(10, 10);
        let mut index = VsaHtmlIndex::new(0.7);
        let html = "<div>same content</div>";
        let fp1 = encoder
            .encode_page_fingerprint(html, "https://a.com")
            .unwrap();
        let fp2 = encoder
            .encode_page_fingerprint(html, "https://b.com")
            .unwrap();

        index.add(fp1.clone());
        assert!(index.is_duplicate(&fp2));
    }

    #[test]
    fn test_vsa_index_prune() {
        let encoder = DomVsaEncoder::new(10, 10);
        let mut index = VsaHtmlIndex::new(0.5);
        for i in 0..10 {
            let fp = encoder
                .encode_page_fingerprint(
                    &format!("<div>page {}</div>", i),
                    &format!("https://page{}.com", i),
                )
                .unwrap();
            index.add(fp);
        }
        assert_eq!(index.len(), 10);
        index.prune_old(3);
        assert_eq!(index.len(), 3);
    }

    #[test]
    fn test_vsa_index_empty_stats() {
        let index = VsaHtmlIndex::new(0.5);
        let stats = index.stats();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.unique_urls, 0);
        assert_eq!(stats.avg_similarity, 0.0);
    }

    #[test]
    fn test_vsa_index_stats() {
        let encoder = DomVsaEncoder::new(10, 10);
        let mut index = VsaHtmlIndex::new(0.3);
        for i in 0..3 {
            let fp = encoder
                .encode_page_fingerprint(
                    &format!("<div>page {}</div>", i),
                    &format!("https://page{}.com", i),
                )
                .unwrap();
            index.add(fp);
        }
        let stats = index.stats();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.unique_urls, 3);
        assert!(stats.avg_similarity > 0.0);
    }

    // ── Edge case tests ──

    #[test]
    fn test_parse_html_with_comments() {
        let html = "<div><!-- comment -->text</div>";
        let tree = parse_html(html, 10, 10).unwrap();
        let div = &tree.nodes[tree.nodes[0].children[0]];
        assert_eq!(div.text_content.trim(), "text");
    }

    #[test]
    fn test_parse_html_depths() {
        let html = "<div><span><a>link</a></span></div>";
        let tree = parse_html(html, 10, 10).unwrap();
        let div = &tree.nodes[tree.nodes[0].children[0]];
        assert_eq!(div.depth, 1);
        let span = &tree.nodes[div.children[0]];
        assert_eq!(span.depth, 2);
        let a = &tree.nodes[span.children[0]];
        assert_eq!(a.depth, 3);
    }

    #[test]
    fn test_simple_vsa_similarity_range() {
        let a = SimpleVsaVector::from_seed(100);
        let b = SimpleVsaVector::from_seed(200);
        let sim = a.similarity(&b);
        assert!(
            sim >= 0.0 && sim <= 1.0,
            "similarity should be in [0,1]: {sim}"
        );
    }

    #[test]
    fn test_dom_vsa_encoder_with_position() {
        let encoder = DomVsaEncoder::new(10, 10).with_position(false);
        let node = encoder
            .encode_html("<div><span>a</span><span>b</span></div>")
            .unwrap();
        assert_eq!(node.children.len(), 2);
    }
}
