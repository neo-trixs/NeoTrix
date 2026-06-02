use std::time::Instant;

use super::helpers::{build_selector, fingerprint_from_dom_element, jaccard_str, levenshtein_normalized};
use super::types::{DomSnapshot, ElementFingerprint, FallbackSelectors, FuzzyMatch, SavedElement};

pub struct AdaptiveTracker {
    pub saved: Vec<SavedElement>,
    pub max_saved: usize,
}

impl AdaptiveTracker {
    pub fn new() -> Self {
        AdaptiveTracker {
            saved: Vec::new(),
            max_saved: 100,
        }
    }

    pub fn new_with_limit(max_saved: usize) -> Self {
        AdaptiveTracker {
            saved: Vec::new(),
            max_saved,
        }
    }

    pub fn save_element(&mut self, name: &str, selector: &str, fingerprint: ElementFingerprint) {
        if self.saved.len() >= self.max_saved {
            let oldest_idx = self
                .saved
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.created_at)
                .map(|(i, _)| i);
            if let Some(idx) = oldest_idx {
                self.saved.remove(idx);
            }
        }
        self.saved.push(SavedElement {
            name: name.to_string(),
            fingerprint,
            original_selector: selector.to_string(),
            created_at: Instant::now(),
            hit_count: 0,
        });
    }

    pub fn locate(
        &mut self,
        name: &str,
        dom_snapshot: &DomSnapshot,
    ) -> Option<FuzzyMatch> {
        let pos = self.saved.iter().position(|e| e.name == name)?;
        let saved = &self.saved[pos];

        let mut best: Option<FuzzyMatch> = None;
        let mut best_score = 0.6;

        for dom_elem in &dom_snapshot.elements {
            let fp = fingerprint_from_dom_element(dom_elem);
            let score = Self::similarity(&saved.fingerprint, &fp);
            if score >= best_score {
                let selector = build_selector(dom_elem);
                let element_repr = format!(
                    "<{}>{}",
                    dom_elem.tag,
                    if dom_elem.text.len() > 50 {
                        &dom_elem.text[..50]
                    } else {
                        &dom_elem.text
                    },
                );
                best = Some(FuzzyMatch {
                    element: element_repr,
                    similarity: score,
                    selector,
                });
                best_score = score;
            }
        }

        if best.is_some() {
            self.saved[pos].hit_count += 1;
        }

        best
    }

    pub fn find_similar(
        &self,
        fingerprint: &ElementFingerprint,
        dom_snapshot: &DomSnapshot,
        threshold: f64,
    ) -> Vec<FuzzyMatch> {
        let mut results: Vec<FuzzyMatch> = dom_snapshot
            .elements
            .iter()
            .map(|e| {
                let fp = fingerprint_from_dom_element(e);
                let score = Self::similarity(fingerprint, &fp);
                let selector = build_selector(e);
                let element_repr = format!(
                    "<{}>{}",
                    e.tag,
                    if e.text.len() > 50 {
                        &e.text[..50]
                    } else {
                        &e.text
                    },
                );
                FuzzyMatch {
                    element: element_repr,
                    similarity: score,
                    selector,
                }
            })
            .filter(|m| m.similarity >= threshold)
            .collect();
        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    pub fn track_element(&mut self, name: &str, selector: &str, dom: &DomSnapshot) -> FallbackSelectors {
        let mut selectors = FallbackSelectors::new(selector);
        selectors.add(selector, 1.0);

        for elem in &dom.elements {
            let current_selector = build_selector(elem);
            if current_selector == selector {
                continue;
            }
            let fp = fingerprint_from_dom_element(elem);
            let sim = Self::similarity_advanced(&fp, selector, dom);
            if sim >= 0.6 {
                selectors.add(&current_selector, sim);
            }
        }

        selectors.sort();
        selectors.truncate(5);

        let fp = selectors.best_fingerprint(dom);
        if let Some(fp) = fp {
            self.save_element(name, selector, fp);
        }

        selectors
    }

    pub fn smart_selector(saved: &SavedElement, dom_snapshot: &DomSnapshot) -> String {
        if let Some(m) = Self::default().locate(&saved.name, dom_snapshot) {
            return m.selector;
        }

        let fp = &saved.fingerprint;
        let mut candidates: Vec<(String, f64)> = Vec::new();

        for elem in &dom_snapshot.elements {
            let current = fingerprint_from_dom_element(elem);
            let sim = AdaptiveTracker::similarity(fp, &current);
            if sim >= 0.5 {
                let sel = build_selector(elem);
                let stability = Self::selector_stability(fp, &current);
                candidates.push((sel, sim * 0.6 + stability * 0.4));
            }
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates.first().map(|(s, _)| s.clone()).unwrap_or_else(|| saved.original_selector.clone())
    }

    fn selector_stability(fp_a: &ElementFingerprint, fp_b: &ElementFingerprint) -> f64 {
        let id_stable = if fp_a.data_attrs.keys().any(|k| k == "id" || k.starts_with("data-")) { 0.4 } else { 0.0 };
        let tag_stable = if fp_a.tag == fp_b.tag { 0.3 } else { 0.0 };
        let depth_stable = if fp_a.depth == fp_b.depth { 0.2 } else { 0.0 };
        let parent_stable = if fp_a.parent_tag == fp_b.parent_tag { 0.1 } else { 0.0 };
        id_stable + tag_stable + depth_stable + parent_stable
    }

    fn similarity_advanced(fp: &ElementFingerprint, _selector: &str, _dom: &DomSnapshot) -> f64 {
        let baseline = Self::similarity(fp, fp);
        if baseline > 0.9 { 0.9 } else { baseline * 0.85 }
    }

    pub fn similarity(a: &ElementFingerprint, b: &ElementFingerprint) -> f64 {
        let tag_score = if a.tag == b.tag { 1.0 } else { 0.0 };

        let class_score = jaccard_str(&a.class_names, &b.class_names);

        let text_score = levenshtein_normalized(&a.text_content, &b.text_content);

        let a_keys: Vec<String> = a.data_attrs.keys().cloned().collect();
        let b_keys: Vec<String> = b.data_attrs.keys().cloned().collect();
        let data_score = jaccard_str(&a_keys, &b_keys);

        let parent_tag_score = if a.parent_tag == b.parent_tag {
            1.0
        } else {
            0.0
        };
        let parent_class_score = jaccard_str(&a.parent_classes, &b.parent_classes);
        let parent_score = 0.5 * parent_tag_score + 0.5 * parent_class_score;

        let max_idx = a.sibling_index.max(b.sibling_index).max(1) as f64;
        let diff = (a.sibling_index as isize - b.sibling_index as isize).unsigned_abs() as f64;
        let sibling_score = 1.0 - diff / max_idx;

        0.3 * tag_score
            + 0.2 * class_score
            + 0.15 * text_score
            + 0.15 * data_score
            + 0.1 * parent_score
            + 0.1 * sibling_score
    }
}

impl Default for AdaptiveTracker {
    fn default() -> Self {
        Self::new()
    }
}
