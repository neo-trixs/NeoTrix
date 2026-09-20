//! Adaptive element relocation via similarity algorithms.

/// An adaptive CSS/XPath selector that self-heals when the DOM changes
#[derive(Debug, Clone)]
pub struct AdaptiveSelector {
    primary: String,
    fallbacks: Vec<String>,
    last_matched: Option<String>,
}

impl AdaptiveSelector {
    pub fn new(primary: &str) -> Self {
        Self {
            primary: primary.into(),
            fallbacks: Vec::new(),
            last_matched: None,
        }
    }

    pub fn with_fallback(mut self, fallback: &str) -> Self {
        self.fallbacks.push(fallback.into());
        self
    }

    /// Try to match the selector against HTML content
    pub fn match_against(&self, html: &str) -> Option<String> {
        // Try primary first
        if html.contains(&self.primary) {
            return Some(self.primary.clone());
        }
        // Try fallbacks
        for fallback in &self.fallbacks {
            if html.contains(fallback) {
                return Some(fallback.clone());
            }
        }
        None
    }
}
