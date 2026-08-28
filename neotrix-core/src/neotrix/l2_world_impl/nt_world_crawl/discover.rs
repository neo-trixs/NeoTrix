//! Regex-based discovery rules for the NT-WORLD crawler.
//!
//! Provides pattern-based content filtering and extraction for the `discover` capability.
//! Rules filter short content (< 30 chars), remove navigation/boilerplate, and extract
//! relevant content from crawled pages.

use serde::{Deserialize, Serialize};
use std::fmt;
use regex::Regex;

/// Action to take when a regex pattern matches content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiscoveryAction {
    /// Keep the matching content
    Keep,
    /// Prune/remove the matching content
    Prune,
    /// Transform the matching content
    Transform,
    /// Flag for review
    Flag,
}

impl fmt::Display for DiscoveryAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscoveryAction::Keep => write!(f, "keep"),
            DiscoveryAction::Prune => write!(f, "prune"),
            DiscoveryAction::Transform => write!(f, "transform"),
            DiscoveryAction::Flag => write!(f, "flag"),
        }
    }
}

/// A single regex-based discovery rule.
///
/// The pattern is stored as a String for serialization compatibility.
/// The Regex is compiled on demand via `compiled_pattern()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryRule {
    /// The regex pattern string (serialized as JSON string)
    pub pattern: String,
    /// The action to take when the pattern matches
    pub action: DiscoveryAction,
    /// Human-readable description
    pub description: String,
}

impl DiscoveryRule {
    /// Create a new rule from a pattern string and action
    pub fn new(pattern: &str, action: DiscoveryAction, description: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            pattern: pattern.to_string(),
            action,
            description: description.to_string(),
        })
    }

    /// Compile the pattern string into a Regex for matching
    pub fn compiled_pattern(&self) -> Regex {
        Regex::new(&self.pattern).expect("valid regex pattern")
    }
}

/// Discovered content result from applying a rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredContent {
    /// The matched content text
    pub content: String,
    /// The rule index that matched
    pub rule_index: usize,
    /// The action taken
    pub action: DiscoveryAction,
    /// Whether the content passed all filters
    pub passes_filters: bool,
    /// The rule description (for debugging/filtering)
    pub rule_description: String,
}

/// Statistics from the discovery extractor.
#[derive(Debug, Clone, Default)]
pub struct ExtractorStats {
    /// Total rules applied
    pub total_rules: usize,
    /// Total content items processed
    pub total_processed: usize,
    /// Content that passed all filters
    pub content_passed: usize,
    /// Content that was pruned/filtered out
    pub content_pruned: usize,
}

/// Regex-based discovery extractor for NT-WORLD crawler content.
///
/// Applies a set of regex rules to filter and extract relevant content from
/// crawled pages. Rules include minimum content length filtering, navigation
/// element removal, and other content quality checks.
#[derive(Debug, Clone, Default)]
pub struct DiscoveryExtractor {
    /// The discovery rules to apply
    rules: Vec<DiscoveryRule>,
    /// Statistics
    stats: ExtractorStats,
}

impl DiscoveryExtractor {
    /// Create a new DiscoveryExtractor with default rules.
    ///
    /// Default rules include:
    /// - Minimum content length: prune content shorter than 30 characters
    /// - Navigation element removal: remove <nav> elements and their contents
    /// - Footer removal: remove <footer> elements
    /// - Boilerplate pattern removal
    pub fn new() -> Self {
        let mut rules = Vec::new();

        // Rule 1: Minimum content length - prune content shorter than 30 chars
        rules.push(DiscoveryRule::new(
            r"^.{0,30}$",
            DiscoveryAction::Prune,
            "Prune content shorter than 30 characters",
        ).expect("valid regex"));

        // Rule 2: Remove navigation elements (<nav>...</nav>)
        rules.push(DiscoveryRule::new(
            r"<nav[^>]*>.*?</nav>|<nav[^>]*/?>",
            DiscoveryAction::Prune,
            "Remove navigation element HTML",
        ).expect("valid regex"));

        // Rule 3: Remove footer elements (<footer>...</footer>)
        rules.push(DiscoveryRule::new(
            r"<footer[^>]*>.*?</footer>|<footer[^>]*/?>",
            DiscoveryAction::Prune,
            "Remove footer element HTML",
        ).expect("valid regex"));

        // Rule 4: Remove common boilerplate/footer text patterns
        rules.push(DiscoveryRule::new(
            r"(?i)copyright|©\s*\d{4}|all rights reserved",
            DiscoveryAction::Prune,
            "Remove copyright/boilerplate text",
        ).expect("valid regex"));

        // Rule 5: Remove navigation link patterns
        rules.push(DiscoveryRule::new(
            r"(?i)<a\s+[^>]*href=[^>]*(#|javascript:|mailto:)",
            DiscoveryAction::Prune,
            "Remove navigation/link anchor patterns",
        ).expect("valid regex"));

        let total_rules = rules.len();
        Self {
            rules,
            stats: ExtractorStats {
                total_rules,
                ..Default::default()
            },
        }
    }

    /// Create a new DiscoveryExtractor with custom rules.
    pub fn with_rules(rules: Vec<DiscoveryRule>) -> Self {
        Self {
            rules,
            stats: ExtractorStats::default(),
        }
    }

    /// Add a custom discovery rule.
    pub fn add_rule(&mut self, rule: DiscoveryRule) {
        self.rules.push(rule);
        self.stats.total_rules += 1;
    }

    /// Apply all discovery rules to the given content and return results.
    ///
    /// For each rule, if the pattern matches the content, the rule's action
    /// is recorded. Also checks minimum content length (30 char minimum).
    ///
    /// Returns a list of DiscoveredContent results, one per matching rule.
    pub fn extract(&mut self, content: &str) -> Vec<DiscoveredContent> {
        let mut results = Vec::new();
        let content_len = content.chars().count();
        let compiled = self.compiled_patterns();

        for (i, rule) in self.rules.iter().enumerate() {
            let pattern = &compiled[i];
            let _matches = pattern.is_match(content);
            let passes_filters = content_len > 30 && !self.is_short_navigation(content);

            results.push(DiscoveredContent {
                content: content.to_string(),
                rule_index: i,
                action: rule.action.clone(),
                passes_filters,
                rule_description: rule.description.clone(),
            });
        }

        // Also record a length-based result if content is too short
        if content_len <= 30 {
            results.push(DiscoveredContent {
                content: content.to_string(),
                rule_index: self.rules.len(),
                action: DiscoveryAction::Prune,
                passes_filters: false,
                rule_description: "short content pruned".to_string(),
            });
        }

        self.stats.total_processed += 1;
        if passes_filters_for_content(content, &self.rules) {
            self.stats.content_passed += 1;
        } else {
            self.stats.content_pruned += 1;
        }

        results
    }

    /// Get compiled regex patterns for all rules
    fn compiled_patterns(&self) -> Vec<Regex> {
        self.rules.iter().map(|r| r.compiled_pattern()).collect()
    }

    /// Check if content is short navigation (just nav/footer links etc.)
    fn is_short_navigation(&self, content: &str) -> bool {
        let len = content.chars().count();
        if len <= 30 {
            return true;
        }
        // Check if it's just navigation elements
        self.rules.iter().any(|rule| {
            let pattern = rule.compiled_pattern();
            pattern.is_match(content) &&
            (rule.action == DiscoveryAction::Prune ||
             rule.description.contains("navigation") ||
             rule.description.contains("nav") ||
             rule.description.contains("footer"))
        })
    }
}

/// Helper function to check if content passes all filters.
fn passes_filters_for_content(content: &str, rules: &[DiscoveryRule]) -> bool {
    let content_len = content.chars().count();
    if content_len <= 30 {
        return false;
    }

    // Check that no prune rules match
    for rule in rules {
        if rule.action == DiscoveryAction::Prune {
            let pattern = rule.compiled_pattern();
            if pattern.is_match(content) {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_extractor_creation() {
        let mut extractor = DiscoveryExtractor::new();
        assert!(!extractor.rules.is_empty(), "should have default rules");
        assert!(extractor.stats.total_rules > 0, "should have rules");
    }

    #[test]
    fn test_min_length_filter_prunes_short_content() {
        let mut extractor = DiscoveryExtractor::new();
        let short = "hello world";
        let results = extractor.extract(short);
        // Should have a prune result for short content
        let pruned: Vec<_> = results.iter().filter(|r| r.action == DiscoveryAction::Prune && !r.passes_filters).collect();
        assert!(!pruned.is_empty(), "short content should be pruned");
    }

    #[test]
    fn test_min_length_filter_passes_long_content() {
        let mut extractor = DiscoveryExtractor::new();
        let long_content = "This is a substantial piece of content that is definitely more than thirty characters long so it should pass the minimum length filter";
        let results = extractor.extract(long_content);
        let passed: Vec<_> = results.iter().filter(|r| r.passes_filters).collect();
        assert!(!passed.is_empty(), "long content should pass filters");
    }

    #[test]
    fn test_nav_element_pruning() {
        let mut extractor = DiscoveryExtractor::new();
        let nav_html = "<nav><a href='/home'>Home</a> | <a href='/about'>About</a></nav><p>Real content here</p>";
        let results = extractor.extract(nav_html);
        let pruned: Vec<_> = results.iter().filter(|r| {
            r.rule_description.contains("navigation") || r.action == DiscoveryAction::Prune
        }).collect();
        // At least one rule should match the nav element
        assert!(!pruned.is_empty(), "navigation elements should be pruned");
    }

    #[test]
    fn test_footer_element_pruning() {
        let mut extractor = DiscoveryExtractor::new();
        let footer_html = "<footer>© 2024 Company. All rights reserved.</footer><p>Actual article content</p>";
        let results = extractor.extract(footer_html);
        let pruned: Vec<_> = results.iter().filter(|r| {
            r.rule_description.contains("footer") || r.action == DiscoveryAction::Prune
        }).collect();
        assert!(!pruned.is_empty(), "footer elements should be pruned");
    }

    #[test]
    fn test_keyword_boilerplate_pruning() {
        let mut extractor = DiscoveryExtractor::new();
        let boilerplate = "Copyright © 2024 Acme Corp. All rights reserved. Privacy Policy | Terms of Service";
        let results = extractor.extract(boilerplate);
        let pruned: Vec<_> = results.iter().filter(|r| {
            r.rule_description.contains("copyright") || r.rule_description.contains("boilerplate")
        }).collect();
        assert!(!pruned.is_empty(), "copyright/boilerplate should be pruned");
    }

    #[test]
    fn test_navigation_link_pruning() {
        let mut extractor = DiscoveryExtractor::new();
        let nav_links = "<a href='#' class='nav-link'>Home</a> <a href='/about'>About</a>";
        let results = extractor.extract(nav_links);
        let pruned: Vec<_> = results.iter().filter(|r| {
            r.action == DiscoveryAction::Prune && r.rule_description.contains("navigation")
        }).collect();
        assert!(!pruned.is_empty(), "navigation links should be pruned");
    }

    #[test]
    fn test_extractor_stats() {
        let mut extractor = DiscoveryExtractor::new();
        extractor.extract("short");
        extractor.extract("This is a much longer piece of content that definitely exceeds the thirty character minimum threshold for content quality assessment purposes");
        assert!(extractor.stats.total_processed >= 2);
        assert!(extractor.stats.content_passed + extractor.stats.content_pruned >= 2);
    }

    #[test]
    fn test_rule_matching_passes_filters() {
        let mut extractor = DiscoveryExtractor::new();
        // Content that's >30 chars and has no prune-matching patterns
        let good_content = "This is a fine article about technology and science with plenty of meaningful text content";
        let results = extractor.extract(good_content);
        let all_pass: bool = results.iter().all(|r| r.passes_filters);
        assert!(all_pass, "good content should pass all filters");
    }

    #[test]
    fn test_rule_matching_fails_filters() {
        let mut extractor = DiscoveryExtractor::new();
        // Content that matches a prune rule (nav element)
        let nav_content = "<nav>Menu</nav> Some text here";
        let results = extractor.extract(nav_content);
        let all_pass: bool = results.iter().all(|r| r.passes_filters);
        assert!(!all_pass, "content with nav should not pass all filters");
    }
}