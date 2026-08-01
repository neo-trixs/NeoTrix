use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoVisibility {
    pub readability_score: f64,
    pub citation_probability: f64,
    pub structured_data_level: f64,
    pub ai_platform_coverage: f64,
    pub last_measured: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for GeoVisibility {
    fn default() -> Self {
        Self {
            readability_score: 0.5,
            citation_probability: 0.3,
            structured_data_level: 0.0,
            ai_platform_coverage: 0.0,
            last_measured: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GwtGeoDimension {
    pub visibility: GeoVisibility,
    pub target_platforms: Vec<String>,
    pub content_format: ContentFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentFormat {
    Raw,
    JsonLd,
    Markdown,
    Structured,
}

impl Default for GwtGeoDimension {
    fn default() -> Self {
        Self {
            visibility: GeoVisibility::default(),
            target_platforms: vec!["deepseek".into(), "doubao".into(), "qwen".into(), "kimi".into(), "yuanbao".into()],
            content_format: ContentFormat::Markdown,
        }
    }
}

impl GwtGeoDimension {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_visibility(&mut self, score: super::super::nt_core_cap::CapabilityVector) {
        self.visibility.readability_score = (self.visibility.readability_score + score.analysis() * 0.3).min(1.0).max(0.0);
        self.visibility.citation_probability = (self.visibility.citation_probability + score.synthesis() * 0.2).min(1.0).max(0.0);
        self.visibility.structured_data_level = (self.visibility.structured_data_level + score.domain_specificity() * 0.25).min(1.0).max(0.0);
        self.visibility.last_measured = Some(chrono::Utc::now());
    }

    pub fn evaluate_output(&self, content: &str) -> f64 {
        let lower = content.to_lowercase();
        let has_jsonld = content.contains("application/ld+json");
        let has_schema = content.contains("schema.org");
        let has_headers = content.lines().any(|l| l.starts_with('#'));
        let has_lists = content.lines().any(|l| l.trim().starts_with('-') || l.trim().starts_with('*'));
        let readability = if has_jsonld { 0.3 } else { 0.0 } + if has_schema { 0.2 } else { 0.0 }
            + if has_headers { 0.15 } else { 0.0 } + if has_lists { 0.1 } else { 0.0 };
        let structure_score = self.visibility.structured_data_level;
        let citation_score = self.visibility.citation_probability;
        let has_citations = lower.contains("source") || lower.contains("data") || lower.contains("according to");
        let final_citation = if has_citations { citation_score * 1.2 } else { citation_score * 0.8 };
        (readability + structure_score + final_citation).min(1.0)
    }

    pub fn citation_format_suggestion(&self, content: &str) -> String {
        if content.contains("application/ld+json") {
            return "JSON-LD already present".to_string();
        }
        format!(
            r#"<!-- GEO: readability={:.2}, citation_prob={:.2} -->
<script type="application/ld+json">
{{"@context":"https://schema.org","@type":"TechArticle","citationProbability":{:.2}}}
</script>"#,
            self.visibility.readability_score,
            self.visibility.citation_probability,
            self.visibility.citation_probability
        )
    }

    pub fn distribution_channels(&self) -> &[String] {
        &self.target_platforms
    }

    pub fn add_platform(&mut self, platform: &str) {
        let p = platform.to_string();
        if !self.target_platforms.contains(&p) {
            self.target_platforms.push(p);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// GEO-SFE: SEO/Structure-First Extraction Scoring
// ═══════════════════════════════════════════════════════════════════

/// SFE structural features extracted from content
#[derive(Debug, Clone, Default)]
pub struct SfeFeatures {
    /// Number of h1-h6 headings
    pub heading_count: usize,
    /// Number of list items (ul/ol)
    pub list_items: usize,
    /// Number of tables
    pub table_count: usize,
    /// Number of code blocks
    pub code_blocks: usize,
    /// Number of images
    pub image_count: usize,
    /// Number of links
    pub link_count: usize,
    /// Whether document has a clear title/h1
    pub has_title: bool,
    /// Whether document has a description/meta
    pub has_description: bool,
    /// Content length in characters
    pub content_length: usize,
    /// Word count
    pub word_count: usize,
    /// Average paragraph length in words
    pub avg_paragraph_words: f64,
    /// Flesch reading ease score (0-100)
    pub reading_ease: f64,
    /// Whether content has structured data / JSON-LD
    pub has_structured_data: bool,
    /// Number of keyword-rich headings (contain target terms)
    pub keyword_headings: usize,
    /// Internal link ratio vs external
    pub internal_link_ratio: f64,
}

/// SFE scoring configuration
#[derive(Debug, Clone)]
pub struct SfeConfig {
    /// Weight for heading structure (default: 0.25)
    pub heading_weight: f64,
    /// Weight for list/table structure (default: 0.20)
    pub list_table_weight: f64,
    /// Weight for readability (default: 0.15)
    pub readability_weight: f64,
    /// Weight for metadata completeness (default: 0.15)
    pub metadata_weight: f64,
    /// Weight for keyword optimization (default: 0.15)
    pub keyword_weight: f64,
    /// Weight for linking (default: 0.10)
    pub link_weight: f64,
    /// Target keywords for optimization
    pub target_keywords: Vec<String>,
    /// Minimum heading count for good structure (default: 3)
    pub min_headings: usize,
    /// Minimum word count (default: 300)
    pub min_words: usize,
}

impl Default for SfeConfig {
    fn default() -> Self {
        Self {
            heading_weight: 0.25,
            list_table_weight: 0.20,
            readability_weight: 0.15,
            metadata_weight: 0.15,
            keyword_weight: 0.15,
            link_weight: 0.10,
            target_keywords: Vec::new(),
            min_headings: 3,
            min_words: 300,
        }
    }
}

/// SFE score result with breakdown
#[derive(Debug, Clone)]
pub struct SfeScore {
    /// Overall score [0.0, 1.0]
    pub overall: f64,
    /// Heading structure sub-score [0.0, 1.0]
    pub heading_score: f64,
    /// List/table sub-score [0.0, 1.0]
    pub list_table_score: f64,
    /// Readability sub-score [0.0, 1.0]
    pub readability_score: f64,
    /// Metadata sub-score [0.0, 1.0]
    pub metadata_score: f64,
    /// Keyword sub-score [0.0, 1.0]
    pub keyword_score: f64,
    /// Linking sub-score [0.0, 1.0]
    pub link_score: f64,
    /// Improvement suggestions
    pub suggestions: Vec<SfeSuggestion>,
}

/// A specific improvement suggestion from SFE analysis
#[derive(Debug, Clone)]
pub struct SfeSuggestion {
    pub category: String,
    pub severity: SfeSeverity,
    pub message: String,
    pub expected_improvement: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfeSeverity {
    Critical,
    Major,
    Minor,
    Info,
}

impl SfeSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            SfeSeverity::Critical => "critical",
            SfeSeverity::Major => "major",
            SfeSeverity::Minor => "minor",
            SfeSeverity::Info => "info",
        }
    }
}

/// GEO-SFE extractor: analyzes and scores content structure
pub struct SfeExtractor {
    pub config: SfeConfig,
}

impl SfeExtractor {
    pub fn new(config: SfeConfig) -> Self {
        Self { config }
    }

    /// Extract structural features from content text
    pub fn extract_features(&self, content: &str) -> SfeFeatures {
        let lines: Vec<&str> = content.lines().collect();
        let heading_count = lines.iter().filter(|l| {
            let t = l.trim();
            t.starts_with("# ") || t.starts_with("## ") || t.starts_with("### ")
                || t.starts_with("#### ") || t.starts_with("##### ") || t.starts_with("###### ")
        }).count();
        let list_items = lines.iter().filter(|l| {
            let t = l.trim();
            t.starts_with("- ") || t.starts_with("* ") || t.starts_with("+ ")
                || t.starts_with("1. ") || t.starts_with("1) ")
        }).count();
        let table_count = {
            let pipe_lines = lines.iter().filter(|l| l.trim().contains('|')).count();
            // Estimate tables: header + separator + at least 1 data row = 3+ pipe lines
            if pipe_lines >= 3 { pipe_lines.saturating_sub(1) / 2 } else { 0 }
        };
        let code_fence_count = lines.iter().filter(|l| l.trim().starts_with("```")).count();
        let code_blocks = if code_fence_count % 2 == 0 { code_fence_count / 2 } else { 0 };
        let has_title = lines.iter().any(|l| l.trim().starts_with("# "));
        let content_len = content.len();
        let word_count = content.split_whitespace().count();
        let has_structured_data = content.contains("```json") || content.contains("\"@context\"");
        let keyword_headings = if self.config.target_keywords.is_empty() { 0 } else {
            lines.iter().filter(|l| {
                let t = l.trim();
                (t.starts_with("# ") || t.starts_with("## ") || t.starts_with("### "))
                    && self.config.target_keywords.iter().any(|kw| t.to_lowercase().contains(&kw.to_lowercase()))
            }).count()
        };
        SfeFeatures {
            heading_count,
            list_items,
            table_count,
            code_blocks,
            image_count: 0,
            link_count: 0,
            has_title,
            has_description: false,
            content_length: content_len,
            word_count,
            avg_paragraph_words: if lines.is_empty() { 0.0 } else { word_count as f64 / lines.len() as f64 },
            reading_ease: Self::compute_reading_ease(content),
            has_structured_data,
            keyword_headings,
            internal_link_ratio: 0.0,
        }
    }

    /// Score content based on extracted features
    pub fn score(&self, features: &SfeFeatures) -> SfeScore {
        let denom = self.config.min_headings as f64 * 2.0;
        let heading_score = if denom <= 0.0 {
            if features.heading_count > 0 { 1.0 } else { 0.0 }
        } else {
            (features.heading_count as f64 / denom).min(1.0).max(0.0)
        };
        let heading_bonus = if features.has_title { 0.15 } else { 0.0 };
        let heading_score = (heading_score + heading_bonus).min(1.0);

        let list_table_raw = (features.list_items as f64 * 0.05 + features.table_count as f64 * 0.15).min(1.0);
        let list_table_score = list_table_raw;

        let reading = (features.reading_ease / 100.0).max(0.0).min(1.0);
        let word_bonus = if features.word_count >= self.config.min_words { 0.1 } else { 0.0 };
        let readability_score = (reading * 0.7 + word_bonus).min(1.0);

        let metadata_score = if features.has_title { 0.5 } else { 0.0 }
            + if features.has_description { 0.3 } else { 0.0 }
            + if features.has_structured_data { 0.2 } else { 0.0 };

        let keyword_score = if self.config.target_keywords.is_empty() {
            0.5
        } else {
            (features.keyword_headings as f64 / self.config.target_keywords.len() as f64).min(1.0)
        };

        let link_score = if features.link_count > 0 {
            (features.link_count as f64 * 0.1).min(1.0)
        } else {
            0.0
        };

        let overall = heading_score * self.config.heading_weight
            + list_table_score * self.config.list_table_weight
            + readability_score * self.config.readability_weight
            + metadata_score * self.config.metadata_weight
            + keyword_score * self.config.keyword_weight
            + link_score * self.config.link_weight;

        let mut suggestions = Vec::new();
        if features.heading_count < self.config.min_headings {
            suggestions.push(SfeSuggestion {
                category: "headings".into(),
                severity: SfeSeverity::Major,
                message: format!("Add more headings (have {}, need at least {})", features.heading_count, self.config.min_headings),
                expected_improvement: self.config.heading_weight * 0.2,
            });
        }
        if !features.has_title {
            suggestions.push(SfeSuggestion {
                category: "title".into(),
                severity: SfeSeverity::Critical,
                message: "Add an H1 title (# Title)".into(),
                expected_improvement: self.config.metadata_weight * 0.5,
            });
        }
        if features.word_count < self.config.min_words {
            suggestions.push(SfeSuggestion {
                category: "length".into(),
                severity: SfeSeverity::Major,
                message: format!("Content too short ({} words, need at least {})", features.word_count, self.config.min_words),
                expected_improvement: self.config.readability_weight * 0.1,
            });
        }
        if features.list_items == 0 {
            suggestions.push(SfeSuggestion {
                category: "lists".into(),
                severity: SfeSeverity::Minor,
                message: "Add bullet points or numbered lists for scannability".into(),
                expected_improvement: self.config.list_table_weight * 0.1,
            });
        }

        SfeScore {
            overall: overall.max(0.0).min(1.0),
            heading_score,
            list_table_score,
            readability_score,
            metadata_score,
            keyword_score,
            link_score,
            suggestions,
        }
    }

    /// Compute Flesch reading ease score (approximate)
    fn compute_reading_ease(text: &str) -> f64 {
        let words = text.split_whitespace().count();
        if words == 0 { return 0.0; }
        let sentences = text.split(['.', '!', '?']).count().max(1);
        let syllables = text.split_whitespace()
            .map(Self::count_syllables)
            .sum::<usize>();
        let avg_syllables = syllables as f64 / words as f64;
        let avg_words = words as f64 / sentences as f64;
        let score = 206.835 - 1.015 * avg_words - 84.6 * avg_syllables;
        score.max(0.0).min(100.0)
    }

    fn count_syllables(word: &str) -> usize {
        let word = word.trim_matches(|c: char| !c.is_alphabetic());
        if word.is_empty() { return 1; }
        let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];
        let lower = word.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();
        let mut count = 0;
        let mut prev_vowel = false;
        let mut last_c = ' ';
        for (i, &c) in chars.iter().enumerate() {
            let is_vowel = vowels.contains(&c);
            if is_vowel && !prev_vowel {
                count += 1;
            }
            // Handle silent-e at end: "make" → 1, "table" → 2
            if c == 'e' && !is_vowel && i + 1 == chars.len() && count > 1 {
                count -= 1;
            }
            // Handle diphthong splits: "quiet" → 2 (qui-et), "lion" → 2 (li-on)
            if (last_c == 'i' || last_c == 'u') && (c == 'a' || c == 'e' || c == 'o') {
                count += 1;
            }
            prev_vowel = is_vowel;
            last_c = c;
        }
        count.max(1)
    }
}

#[cfg(test)]
mod sfe_tests {
    use super::*;

    #[test]
    fn test_sfe_features_extraction() {
        let config = SfeConfig::default();
        let extractor = SfeExtractor::new(config);
        let content = "\
# Main Title
## Section 1
Some paragraph text here.
## Section 2
- item one
- item two
- item three
| col1 | col2 |
|------|------|
| a    | b    |
```rust
fn hello() {}
```
```rust
fn world() {}
```
More text with source citation.";
        let features = extractor.extract_features(content);
        assert_eq!(features.heading_count, 3);
        assert_eq!(features.list_items, 3);
        assert!(features.table_count >= 1);
        assert!(features.code_blocks >= 1);
        assert!(features.has_title);
        assert!(features.word_count > 0);
        assert!(features.reading_ease > 0.0);
    }

    #[test]
    fn test_sfe_scoring() {
        let config = SfeConfig::default();
        let extractor = SfeExtractor::new(config);
        let content = "\
# Title
## Section 1
Lots of words here. Plenty of sentences. This is a readable paragraph.
## Section 2
Another paragraph with enough text to make a decent score.
- list one
- list two";
        let features = extractor.extract_features(content);
        let score = extractor.score(&features);
        assert!(score.overall >= 0.0);
        assert!(score.overall <= 1.0);
        assert!(score.heading_score >= 0.0);
        assert!(score.heading_score <= 1.0);
        assert!(score.readability_score >= 0.0);
        assert!(score.readability_score <= 1.0);
    }

    #[test]
    fn test_sfe_suggestions() {
        let config = SfeConfig::default();
        let extractor = SfeExtractor::new(config);
        let features = extractor.extract_features("just some plain text with no structure");
        let score = extractor.score(&features);
        assert!(!score.suggestions.is_empty());
        let has_title_suggestion = score.suggestions.iter().any(|s| s.category == "title");
        assert!(has_title_suggestion);
    }

    #[test]
    fn test_sfe_reading_ease() {
        let ease = SfeExtractor::compute_reading_ease("The cat sat on the mat. The dog ran in the park. It was a sunny day.");
        assert!(ease > 0.0);
        assert!(ease <= 100.0);
    }

    #[test]
    fn test_sfe_empty_content() {
        let config = SfeConfig::default();
        let extractor = SfeExtractor::new(config);
        let features = extractor.extract_features("");
        assert_eq!(features.heading_count, 0);
        assert_eq!(features.word_count, 0);
        assert_eq!(features.content_length, 0);
        let score = extractor.score(&features);
        assert!(!score.suggestions.is_empty());
        assert!(score.overall >= 0.0);
    }

    #[test]
    fn test_sfe_keyword_matching() {
        let config = SfeConfig {
            target_keywords: vec!["Rust".into(), "SEO".into()],
            ..SfeConfig::default()
        };
        let extractor = SfeExtractor::new(config);
        let content = "\
# Rust Programming Guide
## SEO Best Practices
Content with Rust examples.
## Other Section
No keywords here.";
        let features = extractor.extract_features(content);
        assert!(features.keyword_headings >= 2);
    }

    #[test]
    fn test_sfe_config_defaults() {
        let config = SfeConfig::default();
        assert!((config.heading_weight - 0.25).abs() < 1e-10);
        assert_eq!(config.min_headings, 3);
        assert_eq!(config.min_words, 300);
        assert!(config.target_keywords.is_empty());
        let s = SfeSeverity::Critical;
        assert_eq!(s.as_str(), "critical");
    }

    #[test]
    fn test_sfe_score_improves_with_structure() {
        let config = SfeConfig::default();
        let extractor = SfeExtractor::new(config);
        let plain = extractor.score(&extractor.extract_features("plain unstructured text"));
        let structured = extractor.score(&extractor.extract_features("\
# Title
## Section 1
Some content here with enough words. More text for length.
## Section 2
Even more text to fill out the word count.
- item one
- item two
- item three"));
        assert!(structured.overall > plain.overall);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_visibility_default() {
        let geo = GwtGeoDimension::new();
        assert_eq!(geo.target_platforms.len(), 5);
        assert!((geo.visibility.readability_score - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_output_with_rich_content() {
        let geo = GwtGeoDimension::new();
        let content = "# Title\n<script type=\"application/ld+json\">{\"@context\":\"https://schema.org\"}</script>\n- item 1\n- item 2\nSource: data";
        let score = geo.evaluate_output(content);
        assert!(score > 0.0);
    }

    #[test]
    fn test_citation_format_suggestion_without_jsonld() {
        let geo = GwtGeoDimension::new();
        let suggestion = geo.citation_format_suggestion("Plain content without structured data.");
        assert!(suggestion.contains("application/ld+json"));
    }

    #[test]
    fn test_citation_format_suggestion_with_jsonld() {
        let geo = GwtGeoDimension::new();
        let suggestion = geo.citation_format_suggestion("Has <script type=\"application/ld+json\">{}</script> already.");
        assert_eq!(suggestion, "JSON-LD already present");
    }

    #[test]
    fn test_add_platform_dedup() {
        let mut geo = GwtGeoDimension::new();
        geo.add_platform("deepseek");
        assert_eq!(geo.target_platforms.len(), 5);
        geo.add_platform("new_platform");
        assert_eq!(geo.target_platforms.len(), 6);
    }

    #[test]
    fn test_update_visibility_from_vector() {
        let mut geo = GwtGeoDimension::new();
        let cv = crate::core::nt_core_cap::CapabilityVector::from_values(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.9, 0.8, 0.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        geo.update_visibility(cv);
        assert!(geo.visibility.readability_score > 0.5);
        assert!(geo.visibility.structured_data_level > 0.0);
    }

    #[test]
    fn test_distribution_channels_returns_platforms() {
        let geo = GwtGeoDimension::new();
        let channels = geo.distribution_channels();
        assert!(channels.contains(&"deepseek".to_string()));
    }

    #[test]
    fn test_evaluate_output_empty_content() {
        let geo = GwtGeoDimension::new();
        let score = geo.evaluate_output("");
        assert!(score >= 0.0);
    }

    #[test]
    fn test_content_format_default() {
        let geo = GwtGeoDimension::new();
        assert!(matches!(geo.content_format, ContentFormat::Markdown));
    }
}
