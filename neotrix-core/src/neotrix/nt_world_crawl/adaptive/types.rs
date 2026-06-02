use std::collections::HashMap;

use super::helpers::fingerprint_from_dom_element;

#[derive(Debug, Clone)]
pub struct ElementSnapshot {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub text: String,
    pub xpath: String,
    pub css_selectors: Vec<String>,
    pub parent_tag: Option<String>,
    pub parent_classes: Vec<String>,
    pub sibling_index: usize,
    pub depth: u8,
}

impl ElementSnapshot {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tag: &str,
        attributes: HashMap<String, String>,
        text: &str,
        xpath: &str,
        css_selectors: Vec<String>,
        parent_tag: Option<&str>,
        parent_classes: Vec<String>,
        sibling_index: usize,
        depth: u8,
    ) -> Self {
        ElementSnapshot {
            tag: tag.to_string(),
            attributes,
            text: text.to_string(),
            xpath: xpath.to_string(),
            css_selectors,
            parent_tag: parent_tag.map(|s| s.to_string()),
            parent_classes,
            sibling_index,
            depth,
        }
    }

    pub fn primary_css_selector(&self) -> &str {
        self.css_selectors.first().map(|s| s.as_str()).unwrap_or(&self.tag)
    }

    pub fn all_css_selectors(&self) -> &[String] {
        &self.css_selectors
    }

    pub fn to_fingerprint(&self) -> ElementFingerprint {
        let mut class_names: Vec<String> = Vec::new();
        if let Some(classes_str) = self.attributes.get("class") {
            class_names = classes_str.split_whitespace().map(|s| s.to_string()).collect();
            class_names.sort();
        }

        let mut data_attrs = HashMap::new();
        for (k, v) in &self.attributes {
            if k.starts_with("data-") {
                data_attrs.insert(k.clone(), v.clone());
            }
        }

        ElementFingerprint {
            tag: self.tag.clone(),
            class_names,
            text_content: self.text.trim().to_string(),
            href: self.attributes.get("href").cloned(),
            src: self.attributes.get("src").cloned(),
            data_attrs,
            parent_tag: self.parent_tag.clone(),
            parent_classes: self.parent_classes.clone(),
            sibling_index: self.sibling_index,
            depth: self.depth,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ElementFingerprint {
    pub tag: String,
    pub class_names: Vec<String>,
    pub text_content: String,
    pub href: Option<String>,
    pub src: Option<String>,
    pub data_attrs: HashMap<String, String>,
    pub parent_tag: Option<String>,
    pub parent_classes: Vec<String>,
    pub sibling_index: usize,
    pub depth: u8,
}

#[derive(Debug, Clone)]
pub struct SavedElement {
    pub name: String,
    pub fingerprint: ElementFingerprint,
    pub original_selector: String,
    pub created_at: std::time::Instant,
    pub hit_count: usize,
}

#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    pub element: String,
    pub similarity: f64,
    pub selector: String,
}

#[derive(Debug, Clone)]
pub struct DomElement {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub text: String,
    pub parent_tag: Option<String>,
    pub parent_classes: Vec<String>,
    pub sibling_index: usize,
    pub depth: u8,
}

#[derive(Debug, Clone)]
pub struct DomSnapshot {
    pub elements: Vec<DomElement>,
}

impl DomSnapshot {
    pub fn new(elements: Vec<DomElement>) -> Self {
        DomSnapshot { elements }
    }
}

#[derive(Debug, Clone)]
pub struct FallbackSelectors {
    selectors: Vec<(String, f64)>,
}

impl FallbackSelectors {
    pub fn new(primary: &str) -> Self {
        FallbackSelectors {
            selectors: vec![(primary.to_string(), 1.0)],
        }
    }

    pub fn add(&mut self, selector: &str, confidence: f64) {
        if !self.selectors.iter().any(|(s, _)| s == selector) {
            self.selectors.push((selector.to_string(), confidence));
        }
    }

    pub fn sort(&mut self) {
        self.selectors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn truncate(&mut self, n: usize) {
        self.selectors.truncate(n);
    }

    pub fn primary(&self) -> &str {
        self.selectors.first().map(|(s, _)| s.as_str()).unwrap_or("")
    }

    pub fn fallbacks(&self) -> Vec<&str> {
        self.selectors.iter().skip(1).map(|(s, _)| s.as_str()).collect()
    }

    pub fn all(&self) -> &[(String, f64)] {
        &self.selectors
    }

    pub fn best_fingerprint(&self, dom: &DomSnapshot) -> Option<ElementFingerprint> {
        let primary = self.primary();
        for elem in &dom.elements {
            let sel = super::helpers::build_selector(elem);
            if sel == primary {
                return Some(fingerprint_from_dom_element(elem));
            }
        }
        dom.elements.first().map(fingerprint_from_dom_element)
    }
}
