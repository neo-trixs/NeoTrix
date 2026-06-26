//! # Self-Healing Selector (G301)
//!
//! Element fingerprinting using multi-factor structural similarity.
//! Fallback chain: CSS → XPath → Text content → DOM path.
//! VSA 4096-dim encoding for element fingerprints (truncated to [u64; 4]).

use regex::Regex;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Multi-factor structural fingerprint of a DOM element.
#[derive(Debug, Clone)]
pub struct ElementFingerprint {
    pub tag_name: String,
    pub class_path: Vec<String>,
    pub id: Option<String>,
    pub text_snippet: Option<String>,
    pub depth: u8,
    pub sibling_index: u32,
    pub attribute_signature: [u64; 2],
    pub vsa_fingerprint: [u64; 4],
}

impl ElementFingerprint {
    pub fn new(tag_name: &str) -> Self {
        Self {
            tag_name: tag_name.to_lowercase(),
            class_path: Vec::new(),
            id: None,
            text_snippet: None,
            depth: 0,
            sibling_index: 0,
            attribute_signature: [0; 2],
            vsa_fingerprint: [0; 4],
        }
    }
}

/// Strategy for generating and ordering fallback selectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorStrategy {
    CssFirst,
    XPathFirst,
    TextFirst,
    Adaptive,
}

/// Self-healing selector with fallback chain and adaptive learning.
#[derive(Debug, Clone)]
pub struct HealingSelector {
    pub primary_selector: String,
    pub fallback_selectors: Vec<String>,
    pub element_fingerprint: ElementFingerprint,
    pub success_count: u32,
    pub fail_count: u32,
    pub last_success_ms: u64,
}

/// Engine for managing multiple self-healing selectors.
#[derive(Debug, Clone)]
pub struct SelectorEngine {
    selectors: Vec<HealingSelector>,
}

/// Aggregated statistics for a SelectorEngine.
#[derive(Debug, Clone, Default)]
pub struct SelectorStats {
    pub total_selectors: usize,
    pub total_successes: u32,
    pub total_failures: u32,
    pub success_rate: f64,
}

// ---------------------------------------------------------------------------
// Hashing utilities
// ---------------------------------------------------------------------------

fn hash_string(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

// ---------------------------------------------------------------------------
// HTML extraction helpers
// ---------------------------------------------------------------------------

fn extract_attr(html: &str, attr: &str) -> Option<String> {
    let pat = format!(r#"\b{}\s*=\s*["']([^"']*)["']"#, regex::escape(attr));
    Regex::new(&pat)
        .ok()?
        .captures(html)?
        .get(1)
        .map(|m| m.as_str().to_string())
}

fn extract_classes(html: &str) -> Vec<String> {
    extract_attr(html, "class")
        .unwrap_or_default()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

fn extract_id(html: &str) -> Option<String> {
    extract_attr(html, "id")
}

fn extract_text_snippet(html: &str) -> Option<String> {
    let re = Regex::new(r">([^<]+)<").ok()?;
    re.captures(html)?
        .get(1)
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty())
}

fn estimate_depth(html: &str) -> u8 {
    let indent = html
        .lines()
        .next()
        .map(|l| l.len() - l.trim_start().len())
        .unwrap_or(0);
    (indent / 2).min(255) as u8
}

fn count_siblings(html: &str, tag_name: &str) -> u32 {
    let tag = regex::escape(tag_name);
    let pat = format!(
        r"<{tag}\b[^>]*>(?:[^<]*(?:<(?!\/{tag}>)[^>]*>[^<]*)*)*<\/{tag}>",
        tag = tag
    );
    Regex::new(&pat)
        .map(|re| re.find_iter(html).count().saturating_sub(1) as u32)
        .unwrap_or(0)
}

fn compute_attr_signature(html: &str) -> [u64; 2] {
    let re = Regex::new(r#"\b(\w+)\s*=\s*["'][^"']*["']"#).unwrap();
    let attrs: Vec<&str> = re
        .find_iter(html)
        .filter_map(|m| {
            let s = m.as_str();
            if s.starts_with("class=") || s.starts_with("id=") {
                None
            } else {
                Some(s)
            }
        })
        .collect();
    [
        hash_string(&attrs.join("|")),
        hash_string(&attrs.iter().rev().cloned().collect::<Vec<_>>().join("|")),
    ]
}

fn compute_vsa_fingerprint(fp: &ElementFingerprint) -> [u64; 4] {
    let t = hash_string(&format!("tag:{}", fp.tag_name));
    let c = hash_string(&format!("cls:{}", fp.class_path.join(",")));
    let i = hash_string(&format!("id:{:?}", fp.id));
    let x = hash_string(&format!("tx:{:?}", fp.text_snippet));
    let d = hash_string(&format!("dp:{}", fp.depth));
    let s = hash_string(&format!("si:{}", fp.sibling_index));
    [
        t ^ c,
        i ^ x,
        d ^ s,
        fp.attribute_signature[0] ^ fp.attribute_signature[1],
    ]
}

// ---------------------------------------------------------------------------
// Similarity functions
// ---------------------------------------------------------------------------

fn jaccard_similarity(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let intersection = a.iter().filter(|x| b.contains(x)).count();
    let union = a.len() + b.len() - intersection;
    if union == 0 {
        1.0
    } else {
        intersection as f64 / union as f64
    }
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (na, nb) = (a.len(), b.len());
    if na == 0 {
        return nb;
    }
    if nb == 0 {
        return na;
    }
    let mut prev: Vec<usize> = (0..=nb).collect();
    let mut curr = vec![0usize; nb + 1];
    for i in 0..na {
        curr[0] = i + 1;
        for j in 0..nb {
            let cost = if a[i] == b[j] { 0 } else { 1 };
            curr[j + 1] = (curr[j] + 1).min(prev[j + 1] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[nb]
}

fn text_snippet_similarity(a: &Option<String>, b: &Option<String>) -> f64 {
    match (a, b) {
        (Some(x), Some(y)) if x == y => 1.0,
        (Some(x), Some(y)) => {
            let len = x.len().max(y.len());
            if len == 0 {
                return 1.0;
            }
            let edits = levenshtein_distance(x, y);
            1.0 - (edits as f64 / len as f64)
        }
        (None, None) => 1.0,
        _ => 0.0,
    }
}

fn hamming_similarity(a: &[u64; 2], b: &[u64; 2]) -> f64 {
    let diff = a[0].wrapping_sub(b[0]).count_ones() + a[1].wrapping_sub(b[1]).count_ones();
    1.0 - (diff as f64 / 128.0)
}

fn vsa_cosine_similarity(a: &[u64; 4], b: &[u64; 4]) -> f64 {
    let dot: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (*x).wrapping_mul(*y).count_ones() as f64)
        .sum();
    let mag_a: f64 = a.iter().map(|x| x.count_ones() as f64).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|x| x.count_ones() as f64).sum::<f64>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    (dot / (mag_a * mag_b)).min(1.0)
}

/// Compute a multi-factor cosine-like similarity between two element fingerprints.
///
/// Weighted factors:
///   - tag_name (25%), class_path (20%), id (15%), text_snippet (15%),
///   - depth (10%), sibling_index (5%), attribute_signature (5%), vsa_fingerprint (5%).
pub fn similarity(a: &ElementFingerprint, b: &ElementFingerprint) -> f64 {
    const W_TAG: f64 = 0.25;
    const W_CLASS: f64 = 0.20;
    const W_ID: f64 = 0.15;
    const W_TEXT: f64 = 0.15;
    const W_DEPTH: f64 = 0.10;
    const W_SIBLING: f64 = 0.05;
    const W_ATTR: f64 = 0.05;
    const W_VSA: f64 = 0.05;

    let tag_sim = if a.tag_name == b.tag_name { 1.0 } else { 0.0 };

    let class_sim = jaccard_similarity(&a.class_path, &b.class_path);

    let id_sim = match (&a.id, &b.id) {
        (Some(x), Some(y)) if x == y => 1.0,
        (Some(_), Some(_)) => 0.0,
        (None, None) => 1.0,
        _ => 0.3,
    };

    let text_sim = text_snippet_similarity(&a.text_snippet, &b.text_snippet);

    let depth_diff = (a.depth as f64 - b.depth as f64).abs() / 255.0;
    let depth_sim = 1.0 - depth_diff.min(1.0);

    let sib_diff = ((a.sibling_index as f64 - b.sibling_index as f64).abs() / 100.0).min(1.0);
    let sibling_sim = 1.0 - sib_diff;

    let attr_sim = hamming_similarity(&a.attribute_signature, &b.attribute_signature);
    let vsa_sim = vsa_cosine_similarity(&a.vsa_fingerprint, &b.vsa_fingerprint);

    W_TAG * tag_sim
        + W_CLASS * class_sim
        + W_ID * id_sim
        + W_TEXT * text_sim
        + W_DEPTH * depth_sim
        + W_SIBLING * sibling_sim
        + W_ATTR * attr_sim
        + W_VSA * vsa_sim
}

// ---------------------------------------------------------------------------
// Fingerprint computation
// ---------------------------------------------------------------------------

/// Compute an `ElementFingerprint` from an HTML snippet containing the element.
pub fn compute_fingerprint(html_snippet: &str, tag_name: &str) -> ElementFingerprint {
    let tag = tag_name.to_lowercase();
    let classes = extract_classes(html_snippet);
    let id = extract_id(html_snippet);
    let text = extract_text_snippet(html_snippet);
    let depth = estimate_depth(html_snippet);
    let sibling_index = count_siblings(html_snippet, &tag);
    let attr_sig = compute_attr_signature(html_snippet);

    let mut fp = ElementFingerprint {
        tag_name: tag,
        class_path: classes,
        id,
        text_snippet: text,
        depth,
        sibling_index,
        attribute_signature: attr_sig,
        vsa_fingerprint: [0; 4],
    };
    fp.vsa_fingerprint = compute_vsa_fingerprint(&fp);
    fp
}

// ---------------------------------------------------------------------------
// Selector generation
// ---------------------------------------------------------------------------

fn format_css_selector(tag: &str, id: &Option<String>, classes: &[String]) -> String {
    let mut sel = tag.to_string();
    if let Some(id_val) = id {
        sel.push('#');
        sel.push_str(id_val);
    }
    for cls in classes {
        sel.push('.');
        sel.push_str(cls);
    }
    sel
}

fn format_xpath(tag: &str, id: &Option<String>, classes: &[String]) -> String {
    let mut preds = Vec::new();
    if let Some(id_val) = id {
        preds.push(format!("@id='{}'", id_val));
    }
    if let Some(first_class) = classes.first() {
        preds.push(format!("contains(@class,'{}')", first_class));
    }
    if preds.is_empty() {
        format!("//{}", tag)
    } else {
        format!("//{}[{}]", tag, preds.join(" and "))
    }
}

fn generate_fresh_selector(snippet: &str) -> String {
    let tag = Regex::new(r"<\s*(\w+)")
        .ok()
        .and_then(|re| re.captures(snippet))
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .unwrap_or("div");
    let id = extract_id(snippet);
    let classes = extract_classes(snippet);
    format_css_selector(tag, &id, &classes)
}

fn generate_fallback_selectors_from_fingerprint(
    fp: &ElementFingerprint,
    strategy: SelectorStrategy,
) -> Vec<String> {
    let css = format_css_selector(&fp.tag_name, &fp.id, &fp.class_path);
    let xpath = format_xpath(&fp.tag_name, &fp.id, &fp.class_path);
    let text = fp.text_snippet.as_deref().unwrap_or("").to_string();

    let mut fbs: Vec<String> = Vec::new();
    let tag = &fp.tag_name;

    match strategy {
        SelectorStrategy::CssFirst => {
            fbs.push(css.clone());
            fbs.push(xpath.clone());
            if !text.is_empty() {
                fbs.push(format!("//{}[contains(text(),'{}')]", tag, text));
            }
            fbs.push(format!("//{}", tag));
        }
        SelectorStrategy::XPathFirst => {
            fbs.push(xpath.clone());
            fbs.push(css.clone());
            if !text.is_empty() {
                fbs.push(format!("//{}[contains(text(),'{}')]", tag, text));
            }
            fbs.push(format!("//{}", tag));
        }
        SelectorStrategy::TextFirst => {
            if !text.is_empty() {
                fbs.push(format!("//{}[contains(text(),'{}')]", tag, text));
            }
            fbs.push(xpath.clone());
            fbs.push(css.clone());
            fbs.push(format!("//{}", tag));
        }
        SelectorStrategy::Adaptive => {
            fbs.push(css.clone());
            fbs.push(xpath.clone());
            if !text.is_empty() {
                fbs.push(format!("//{}[contains(text(),'{}')]", tag, text));
            }
            fbs.push(format!("//{}", tag));
        }
    }

    fbs
}

/// Generate fallback selectors for a given HTML snippet, ordered by strategy.
pub fn generate_fallback_selectors(html_snippet: &str, strategy: SelectorStrategy) -> Vec<String> {
    let tag = Regex::new(r"<\s*(\w+)")
        .ok()
        .and_then(|re| re.captures(html_snippet))
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .unwrap_or("div");

    let fp = compute_fingerprint(html_snippet, tag);
    generate_fallback_selectors_from_fingerprint(&fp, strategy)
}

// ---------------------------------------------------------------------------
// Selector matching helpers
// ---------------------------------------------------------------------------

fn find_element_by_selector(html: &str, selector: &str) -> Option<String> {
    if selector.starts_with("//") {
        find_by_xpath_simple(html, selector)
    } else {
        find_by_css_simple(html, selector)
    }
}

fn find_by_css_simple(html: &str, selector: &str) -> Option<String> {
    let has_id = selector.contains('#');
    let has_class = selector.contains('.');
    let tag = selector.split(&['#', '.'][..]).next().unwrap_or(selector);

    let pattern = if has_id {
        let id_part = selector
            .split('#')
            .nth(1)
            .and_then(|s| s.split('.').next())
            .unwrap_or("");
        format!(
            r#"<{tag}[^>]*\bid\s*=\s*["']{}["'][^>]*>[^<]*</{tag}>"#,
            regex::escape(id_part)
        )
    } else if has_class {
        let class_part = selector.split('.').nth(1).unwrap_or("");
        format!(
            r#"<{tag}[^>]*\bclass\s*=\s*["'][^"']*{}[^"']*["'][^>]*>[^<]*</{tag}>"#,
            regex::escape(class_part)
        )
    } else {
        format!(
            r"<{tag}[^>]*>[^<]*</{tag}>",
            tag = regex::escape(tag)
        )
    };

    Regex::new(&pattern)
        .ok()
        .and_then(|re| re.find(html))
        .map(|m| m.as_str().to_string())
}

fn find_by_xpath_simple(html: &str, xpath: &str) -> Option<String> {
    let stripped = xpath.trim_start_matches("//");
    let tag = stripped.split('[').next().unwrap_or(stripped);
    let pattern = format!(
        r"<{tag}[^>]*>[^<]*</{tag}>",
        tag = regex::escape(tag)
    );
    Regex::new(&pattern)
        .ok()
        .and_then(|re| re.find(html))
        .map(|m| m.as_str().to_string())
}

fn find_candidates_by_tag(html: &str, tag_name: &str) -> Vec<String> {
    let pattern = format!(
        r"<{tag_name}[^>]*>[^<]*</{tag_name}>",
        tag_name = regex::escape(tag_name)
    );
    match Regex::new(&pattern) {
        Ok(re) => re.find_iter(html).map(|m| m.as_str().to_string()).collect(),
        Err(_) => vec![],
    }
}

// ---------------------------------------------------------------------------
// HealingSelector impl
// ---------------------------------------------------------------------------

impl HealingSelector {
    pub fn new(primary_selector: &str, element_fingerprint: ElementFingerprint) -> Self {
        let fallbacks =
            generate_fallback_selectors_from_fingerprint(&element_fingerprint, SelectorStrategy::Adaptive);
        Self {
            primary_selector: primary_selector.to_string(),
            fallback_selectors: fallbacks,
            element_fingerprint,
            success_count: 0,
            fail_count: 0,
            last_success_ms: 0,
        }
    }

    /// Find the best matching element in `current_html` and return a healed selector.
    ///
    /// Phase 1: verify primary selector against stored fingerprint (threshold ≥ 0.7).
    /// Phase 2: try fallback selectors (threshold ≥ 0.6).
    /// Phase 3: scan all same-tag-name candidates and return the best match (threshold ≥ 0.5).
    pub fn heal_selector(&self, current_html: &str) -> Option<String> {
        // Phase 1 — try primary selector
        if let Some(snippet) = find_element_by_selector(current_html, &self.primary_selector) {
            let fp = compute_fingerprint(&snippet, &self.element_fingerprint.tag_name);
            let sim = similarity(&self.element_fingerprint, &fp);
            if sim >= 0.7 {
                log::debug!("heal: primary OK sim={:.3}", sim);
                return Some(self.primary_selector.clone());
            }
        }

        // Phase 2 — try fallbacks
        for fb in &self.fallback_selectors {
            if let Some(snippet) = find_element_by_selector(current_html, fb) {
                let fp = compute_fingerprint(&snippet, &self.element_fingerprint.tag_name);
                let sim = similarity(&self.element_fingerprint, &fp);
                if sim >= 0.6 {
                    log::debug!("heal: fallback '{}' OK sim={:.3}", fb, sim);
                    return Some(fb.clone());
                }
            }
        }

        // Phase 3 — exhaustive tag scan
        let candidates = find_candidates_by_tag(current_html, &self.element_fingerprint.tag_name);
        let mut best_sim = 0.5;
        let mut best_selector: Option<String> = None;

        for snippet in candidates {
            let fp = compute_fingerprint(&snippet, &self.element_fingerprint.tag_name);
            let sim = similarity(&self.element_fingerprint, &fp);
            if sim > best_sim {
                best_sim = sim;
                best_selector = Some(generate_fresh_selector(&snippet));
            }
        }

        let best_selector = best_selector;
        if let Some(ref sel) = best_selector {
            log::info!("heal: new '{}' sim={:.3}", sel, best_sim);
        } else {
            log::warn!("heal: no match for '{}'", self.primary_selector);
        }

        best_selector
    }

    /// Record the outcome of using a selector for adaptive learning.
    pub fn record_outcome(&mut self, selector_used: &str, success: bool) {
        if success {
            self.success_count += 1;
            self.last_success_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            if selector_used != self.primary_selector
                && !self.fallback_selectors.contains(&selector_used.to_string())
            {
                self.fallback_selectors.push(selector_used.to_string());
            }
        } else {
            self.fail_count += 1;
        }
    }

    /// Choose strategy based on historical success rate.
    ///
    /// - < 30% → TextFirst (liberal matching)
    /// - 30–60% → XPathFirst (structural)
    /// - ≥ 60% → CssFirst (precise, confident)
    pub fn adaptive_strategy(&self) -> SelectorStrategy {
        let total = self.success_count + self.fail_count;
        if total == 0 {
            return SelectorStrategy::CssFirst;
        }
        let rate = self.success_count as f64 / total as f64;
        if rate < 0.3 {
            SelectorStrategy::TextFirst
        } else if rate < 0.6 {
            SelectorStrategy::XPathFirst
        } else {
            SelectorStrategy::CssFirst
        }
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.fail_count;
        if total == 0 {
            0.0
        } else {
            self.success_count as f64 / total as f64
        }
    }
}

// ---------------------------------------------------------------------------
// SelectorEngine impl
// ---------------------------------------------------------------------------

impl SelectorEngine {
    pub fn new() -> Self {
        Self {
            selectors: Vec::new(),
        }
    }

    pub fn add_selector(&mut self, selector: HealingSelector) {
        self.selectors.push(selector);
    }

    pub fn get(&self, index: usize) -> Option<&HealingSelector> {
        self.selectors.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut HealingSelector> {
        self.selectors.get_mut(index)
    }

    pub fn remove(&mut self, index: usize) -> Option<HealingSelector> {
        if index < self.selectors.len() {
            Some(self.selectors.remove(index))
        } else {
            None
        }
    }

    /// Attempt to heal all selectors against `current_html`.
    /// Returns `(index, Option<healed_selector>)` for each selector.
    pub fn heal_all(&self, current_html: &str) -> Vec<(usize, Option<String>)> {
        self.selectors
            .iter()
            .enumerate()
            .map(|(i, hs)| (i, hs.heal_selector(current_html)))
            .collect()
    }

    pub fn statistics(&self) -> SelectorStats {
        let total_successes: u32 = self.selectors.iter().map(|s| s.success_count).sum();
        let total_failures: u32 = self.selectors.iter().map(|s| s.fail_count).sum();
        let total = total_successes + total_failures;
        let success_rate = if total == 0 {
            0.0
        } else {
            total_successes as f64 / total as f64
        };
        SelectorStats {
            total_selectors: self.selectors.len(),
            total_successes,
            total_failures,
            success_rate,
        }
    }

    pub fn len(&self) -> usize {
        self.selectors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.selectors.is_empty()
    }
}

impl Default for SelectorEngine {
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

    const SAMPLE_HTML: &str = r#"<div class="container main">
        <h1 id="title">Welcome</h1>
        <p class="description">This is a test</p>
        <span class="price" data-value="42.99">$42.99</span>
    </div>"#;

    // ---- fingerprint ----

    #[test]
    fn test_compute_fingerprint_h1() {
        let html = r#"<h1 id="title" class="heading">Welcome</h1>"#;
        let fp = compute_fingerprint(html, "h1");
        assert_eq!(fp.tag_name, "h1");
        assert_eq!(fp.id, Some("title".into()));
        assert!(fp.class_path.contains(&"heading".to_string()));
        assert_eq!(fp.text_snippet, Some("Welcome".into()));
    }

    #[test]
    fn test_compute_fingerprint_div() {
        let fp = compute_fingerprint(SAMPLE_HTML, "div");
        assert_eq!(fp.tag_name, "div");
        assert!(fp.class_path.contains(&"container".to_string()));
    }

    // ---- similarity ----

    #[test]
    fn test_similarity_identical() {
        let a = compute_fingerprint(r#"<p class="text">Hello</p>"#, "p");
        let b = compute_fingerprint(r#"<p class="text">Hello</p>"#, "p");
        let sim = similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.01, "identical ~1.0, got {}", sim);
    }

    #[test]
    fn test_similarity_different() {
        let a = compute_fingerprint(r#"<p class="text">Hello</p>"#, "p");
        let b = compute_fingerprint(r#"<div class="box">World</div>"#, "div");
        let sim = similarity(&a, &b);
        assert!(sim < 0.8, "different elements: sim={}", sim);
    }

    #[test]
    fn test_similarity_threshold() {
        let html_a = r#"<div class="card">Old content</div>"#;
        let html_b = r#"<div class="card">New content</div>"#;
        let a = compute_fingerprint(html_a, "div");
        let b = compute_fingerprint(html_b, "div");
        let sim = similarity(&a, &b);
        assert!(sim > 0.5, "similar content: {}", sim);
    }

    // ---- fallback generation ----

    #[test]
    fn test_generate_fallback_css_first() {
        let html = r#"<a class="link" id="main-link" href="/page">Click</a>"#;
        let fallbacks = generate_fallback_selectors(html, SelectorStrategy::CssFirst);
        assert!(!fallbacks.is_empty());
        assert!(fallbacks[0].contains("a"));
    }

    #[test]
    fn test_generate_fallback_xpath_first() {
        let html = r#"<span class="badge">New</span>"#;
        let fallbacks = generate_fallback_selectors(html, SelectorStrategy::XPathFirst);
        assert!(
            fallbacks[0].starts_with("//"),
            "first should be xpath: {:?}",
            fallbacks
        );
    }

    // ---- healing ----

    #[test]
    fn test_heal_selector_found() {
        let html = r#"<div class="old-class">Content</div>"#;
        let fp = compute_fingerprint(html, "div");
        let hs = HealingSelector::new("div.old-class", fp);
        let new_html = r#"<div class="new-class">Content</div>"#;
        let result = hs.heal_selector(new_html);
        assert!(result.is_some(), "heal should find element");
    }

    #[test]
    fn test_heal_selector_not_found() {
        let fp = ElementFingerprint::new("table");
        let hs = HealingSelector::new("table.data", fp);
        let new_html = r#"<div>No table here</div>"#;
        let result = hs.heal_selector(new_html);
        assert!(result.is_none(), "heal should return None");
    }

    #[test]
    fn test_heal_primary_used_when_good() {
        let html = r#"<button id="go" class="btn">Go</button>"#;
        let fp = compute_fingerprint(html, "button");
        let hs = HealingSelector::new("button#go", fp);
        let result = hs.heal_selector(html);
        assert_eq!(result, Some("button#go".into()));
    }

    // ---- learning ----

    #[test]
    fn test_record_outcome_success() {
        let fp = ElementFingerprint::new("div");
        let mut hs = HealingSelector::new("div.test", fp);
        hs.record_outcome("div.test", true);
        assert_eq!(hs.success_count, 1);
        assert!(hs.last_success_ms > 0);
    }

    #[test]
    fn test_record_outcome_failure() {
        let fp = ElementFingerprint::new("div");
        let mut hs = HealingSelector::new("div.test", fp);
        hs.record_outcome("div.test", false);
        assert_eq!(hs.fail_count, 1);
    }

    #[test]
    fn test_adaptive_strategy() {
        let fp = ElementFingerprint::new("div");
        let mut hs = HealingSelector::new("div.test", fp);
        assert_eq!(hs.adaptive_strategy(), SelectorStrategy::CssFirst);
        hs.fail_count = 10;
        hs.success_count = 1;
        assert_eq!(hs.adaptive_strategy(), SelectorStrategy::TextFirst);
    }

    #[test]
    fn test_success_rate() {
        let fp = ElementFingerprint::new("div");
        let mut hs = HealingSelector::new("div.test", fp);
        assert_eq!(hs.success_rate(), 0.0);
        hs.record_outcome("div.test", true);
        hs.record_outcome("div.test", true);
        hs.record_outcome("div.test", false);
        assert!((hs.success_rate() - 2.0 / 3.0).abs() < 0.01);
    }

    // ---- engine ----

    #[test]
    fn test_selector_engine() {
        let mut engine = SelectorEngine::new();
        assert!(engine.is_empty());

        let fp1 = ElementFingerprint::new("h1");
        let hs1 = HealingSelector::new("h1.title", fp1);
        engine.add_selector(hs1);

        let fp2 = ElementFingerprint::new("p");
        let hs2 = HealingSelector::new("p.desc", fp2);
        engine.add_selector(hs2);

        assert_eq!(engine.len(), 2);
    }

    #[test]
    fn test_selector_stats() {
        let mut engine = SelectorEngine::new();
        let fp = ElementFingerprint::new("a");
        let mut hs = HealingSelector::new("a.link", fp);
        hs.success_count = 5;
        hs.fail_count = 1;
        engine.add_selector(hs);

        let stats = engine.statistics();
        assert_eq!(stats.total_successes, 5);
        assert_eq!(stats.total_failures, 1);
    }

    #[test]
    fn test_engine_heal_all() {
        let mut engine = SelectorEngine::new();
        let html = r#"<span class="x">A</span>"#;
        let fp = compute_fingerprint(html, "span");
        engine.add_selector(HealingSelector::new("span.x", fp));
        let results = engine.heal_all(html);
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_some());
    }

    #[test]
    fn test_engine_remove() {
        let mut engine = SelectorEngine::new();
        let fp = ElementFingerprint::new("a");
        engine.add_selector(HealingSelector::new("a", fp));
        assert!(engine.remove(0).is_some());
        assert!(engine.remove(0).is_none());
    }

    // ---- VSA ----

    #[test]
    fn test_vsa_fingerprint_stability() {
        let html = r#"<button class="btn primary" id="submit">Submit</button>"#;
        let a = compute_fingerprint(html, "button");
        let b = compute_fingerprint(html, "button");
        assert_eq!(a.vsa_fingerprint, b.vsa_fingerprint);
    }

    #[test]
    fn test_element_fingerprint_new() {
        let fp = ElementFingerprint::new("span");
        assert_eq!(fp.tag_name, "span");
        assert!(fp.class_path.is_empty());
        assert!(fp.id.is_none());
    }

    #[test]
    fn test_heal_different_tag_scan() {
        let html = r#"<section id="a">First</section><section id="b">Second</section>"#;
        let target = r#"<section id="b">Second</section>"#;
        let fp = compute_fingerprint(target, "section");
        let hs = HealingSelector::new("section#b", fp);
        let result = hs.heal_selector(html);
        assert!(result.is_some());
    }
}
