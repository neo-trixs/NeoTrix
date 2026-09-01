use std::collections::{HashMap, HashSet};

use super::types::{DomElement, ElementFingerprint};

pub(crate) fn fingerprint_from_dom_element(e: &DomElement) -> ElementFingerprint {
    let mut data_attrs = HashMap::new();
    for (k, v) in &e.attributes {
        if k.starts_with("data-") {
            data_attrs.insert(k.clone(), v.clone());
        }
    }

    let mut class_names: Vec<String> = Vec::new();
    if let Some(classes_str) = e.attributes.get("class") {
        class_names = classes_str.split_whitespace().map(|s| s.to_string()).collect();
        class_names.sort();
    }

    ElementFingerprint {
        tag: e.tag.clone(),
        class_names,
        text_content: e.text.trim().to_string(),
        href: e.attributes.get("href").cloned(),
        src: e.attributes.get("src").cloned(),
        data_attrs,
        parent_tag: e.parent_tag.clone(),
        parent_classes: e.parent_classes.clone(),
        sibling_index: e.sibling_index,
        depth: e.depth,
    }
}

pub(crate) fn build_selector(e: &DomElement) -> String {
    let mut sel = e.tag.clone();
    if let Some(classes) = e.attributes.get("class") {
        for cls in classes.split_whitespace() {
            sel.push('.');
            sel.push_str(cls);
        }
    }
    if let Some(id) = e.attributes.get("id") {
        sel.push('#');
        sel.push_str(id);
    }
    sel
}

pub(crate) fn jaccard_str(a: &[String], b: &[String]) -> f64 {
    let set_a: HashSet<&str> = a.iter().map(|s| s.as_str()).collect();
    let set_b: HashSet<&str> = b.iter().map(|s| s.as_str()).collect();
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 {
        1.0
    } else {
        intersection as f64 / union as f64
    }
}

pub(crate) fn levenshtein_normalized(a: &str, b: &str) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    let dist = levenshtein_distance(&a_lower, &b_lower);
    let max_len = a_lower.len().max(b_lower.len()) as f64;
    1.0 - dist as f64 / max_len
}

pub(crate) fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr: Vec<usize> = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}
