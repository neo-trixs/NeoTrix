use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoScore {
    pub overall: f64,
    pub authority: f64,
    pub structure: f64,
    pub semantic_completeness: f64,
    pub citation_feasibility: f64,
    pub recency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationFormat {
    pub jsonld: Option<String>,
    pub schema_org: Option<String>,
    pub llms_txt: Option<String>,
    pub structured_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionPlan {
    pub channels: Vec<String>,
    pub priority: u8,
    pub schedule: Option<String>,
}

pub trait GeoCapability: Send + Sync {
    fn score_content(&self, content: &str, query: &str) -> GeoScore;
    fn optimize_content(&self, content: &str, target_score: f64) -> String;
    fn inject_structured_data(&self, content: &str) -> CitationFormat;
    fn plan_distribution(&self, content: &str, channels: &[String]) -> DistributionPlan;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoScorer {
    pub authority_weight: f64,
    pub structure_weight: f64,
    pub semantic_weight: f64,
    pub citation_weight: f64,
    pub recency_weight: f64,
}

impl Default for GeoScorer {
    fn default() -> Self {
        Self {
            authority_weight: 0.25,
            structure_weight: 0.25,
            semantic_weight: 0.20,
            citation_weight: 0.20,
            recency_weight: 0.10,
        }
    }
}

impl GeoScorer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn score(&self, content: &str, query: &str) -> GeoScore {
        let authority = self.compute_authority(content);
        let structure = self.compute_structure(content);
        let semantic = self.compute_semantic(content, query);
        let citation = self.compute_citation_feasibility(content);
        let recency = self.compute_recency(content);
        let overall = authority * self.authority_weight
            + structure * self.structure_weight
            + semantic * self.semantic_weight
            + citation * self.citation_weight
            + recency * self.recency_weight;
        GeoScore {
            overall,
            authority,
            structure,
            semantic_completeness: semantic,
            citation_feasibility: citation,
            recency,
        }
    }

    fn compute_authority(&self, content: &str) -> f64 {
        let mut score: f64 = 0.4;
        let lower = content.to_lowercase();
        if lower.contains("according to") || lower.contains("reported by") {
            score += 0.1;
        }
        if lower.contains("study") || lower.contains("research") || lower.contains("survey") {
            score += 0.1;
        }
        if lower.contains("doi:") || lower.contains("arxiv") || lower.contains("ieee") {
            score += 0.1;
        }
        if lower.contains("government") || lower.contains("official") || lower.contains("standard")
        {
            score += 0.1;
        }
        let citation_count = content.matches('[').count().min(10) as f64 / 10.0 * 0.1;
        score + citation_count
    }

    fn compute_structure(&self, content: &str) -> f64 {
        let mut score: f64 = 0.3;
        let lines: Vec<&str> = content.lines().collect();
        let h_count = lines.iter().filter(|l| l.starts_with('#')).count();
        let list_count = lines
            .iter()
            .filter(|l| {
                l.trim().starts_with('-')
                    || l.trim().starts_with('*')
                    || l.trim().starts_with(|c: char| c.is_ascii_digit())
            })
            .count();
        let table_count = lines.iter().filter(|l| l.contains('|')).count();
        if h_count >= 5 {
            score += 0.15;
        } else if h_count >= 3 {
            score += 0.1;
        } else if h_count >= 1 {
            score += 0.05;
        }
        if list_count >= 3 {
            score += 0.1;
        } else if list_count >= 1 {
            score += 0.05;
        }
        if table_count >= 3 {
            score += 0.1;
        } else if table_count >= 1 {
            score += 0.05;
        }
        let first_line = lines.first().map(|l| l.trim()).unwrap_or("");
        if first_line.len() > 20 && first_line.len() < 200 {
            score += 0.15;
        }
        score.min(1.0)
    }

    fn compute_semantic(&self, content: &str, query: &str) -> f64 {
        let lower_content = content.to_lowercase();
        let query_terms: Vec<&str> = query.split_whitespace().collect();
        let matched = query_terms
            .iter()
            .filter(|t| lower_content.contains(*t))
            .count();
        let coverage = if query_terms.is_empty() {
            0.5
        } else {
            matched as f64 / query_terms.len() as f64
        };
        let mut score = 0.3 + coverage * 0.3;
        let sections = ["definition", "reason", "method", "case", "trend"];
        let found_sections = sections
            .iter()
            .filter(|s| lower_content.contains(*s))
            .count();
        score += found_sections as f64 * 0.08;
        score.min(1.0)
    }

    fn compute_citation_feasibility(&self, content: &str) -> f64 {
        let mut score: f64 = 0.4;
        let lower = content.to_lowercase();
        if lower.contains("source") || lower.contains("reference") {
            score += 0.1;
        }
        if lower.contains("data") || lower.contains("statistic") || lower.contains("percent") {
            score += 0.1;
        }
        if lower.contains("quote") || lower.contains("expert") || lower.contains("according") {
            score += 0.1;
        }
        let commas = content.matches(',').count();
        let avg_sentence_len = if commas > 0 {
            content.len() as f64 / (commas as f64 + 1.0)
        } else {
            content.len() as f64
        };
        if (50.0..200.0).contains(&avg_sentence_len) {
            score += 0.1;
        }
        score.min(1.0)
    }

    fn compute_recency(&self, content: &str) -> f64 {
        let lower = content.to_lowercase();
        let mut score: f64 = 0.5;
        let current_year = 2026;
        let year_strs: Vec<String> = (current_year - 2..=current_year)
            .map(|y| y.to_string())
            .collect();
        for ys in &year_strs {
            if lower.contains(ys) {
                score += 0.1;
            }
        }
        if lower.contains("recent") || lower.contains("latest") || lower.contains("updated") {
            score += 0.1;
        }
        if lower.contains("new")
            || lower.contains("novel")
            || lower.contains("current")
            || lower.contains("ongoing")
        {
            score += 0.1;
        }
        score.min(1.0)
    }

    pub fn optimize(&self, content: &str, target_score: f64) -> String {
        let score = self.score(content, "");
        if score.overall >= target_score {
            return content.to_string();
        }
        let mut optimized = content.to_string();
        if score.authority < 0.6 {
            optimized.push_str("\n\nAccording to recent studies and industry reports, this approach has been validated across multiple domains.");
        }
        if score.structure < 0.6 {
            let header = "\n\n## Key Points\n- Structured data improves AI discoverability\n- Clear hierarchy enhances citation probability\n- List formats increase information gain";
            optimized.push_str(header);
        }
        if score.citation_feasibility < 0.6 {
            optimized.push_str(
                "\n\nSources: Industry data (2025-2026), Academic research, Expert interviews",
            );
        }
        if score.recency < 0.6 {
            optimized.push_str(&format!("\n\n*Updated: {}*", 2026));
        }
        optimized
    }

    pub fn inject_jsonld(&self, content: &str, title: &str, description: &str) -> String {
        let jsonld = format!(
            r#"<script type="application/ld+json">
{{
  "@context": "https://schema.org",
  "@type": "Article",
  "headline": "{}",
  "description": "{}",
  "datePublished": "{}",
  "author": {{
    "@type": "Organization",
    "name": "NeoTrix"
  }}
}}
</script>"#,
            title.replace('"', "'"),
            description.replace('"', "'"),
            chrono::Utc::now().format("%Y-%m-%d")
        );
        format!("{}\n\n{}", jsonld, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_content() -> String {
        "## AI-Powered Code Generation\n\nRecent studies show AI code generation improves developer productivity by 55%. According to GitHub's 2025 report, 46% of new code is AI-assisted. Key methods include:\n- Transformer-based models\n- Reinforcement learning from human feedback\n- Retrieval-augmented generation\n\nResearch by Stanford (DOI: 10.1234/ai-code) confirms these findings. Source: Industry data (2025-2026).".to_string()
    }

    #[test]
    fn test_geo_scorer_default_weights() {
        let scorer = GeoScorer::default();
        assert!((scorer.authority_weight - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_score_content_with_sample() {
        let scorer = GeoScorer::new();
        let score = scorer.score(&sample_content(), "AI code generation benefits");
        assert!(score.overall > 0.0);
        assert!(score.overall <= 1.0);
    }

    #[test]
    fn test_authority_score_high_with_citations() {
        let scorer = GeoScorer::new();
        let content = "According to recent studies reported by Stanford (DOI: 10.1234/test), government data shows [1] 50% improvement [2].";
        let score = scorer.compute_authority(content);
        assert!(score > 0.7, "authority score {} should be > 0.7", score);
    }

    #[test]
    fn test_structure_score_high_with_headers() {
        let scorer = GeoScorer::new();
        let content = "# This title line is intentionally long enough to exceed the twenty character threshold\n## Section 1\n### Subsection\n- Item 1\n- Item 2\n- Item 3\n| Col1 | Col2 |\n| Data | Data |\n| More | Items |";
        let score = scorer.compute_structure(content);
        assert!(score > 0.7, "structure score {} should be > 0.7", score);
    }

    #[test]
    fn test_semantic_score_reflects_query_coverage() {
        let scorer = GeoScorer::new();
        let content =
            "Neural networks improve machine learning accuracy through deep learning methods.";
        let score = scorer.compute_semantic(content, "neural networks deep learning");
        assert!(score > 0.6);
    }

    #[test]
    fn test_optimize_low_score_content() {
        let scorer = GeoScorer::new();
        let content = "This is a short text.";
        let optimized = scorer.optimize(content, 0.8);
        assert!(optimized.len() > content.len());
    }

    #[test]
    fn test_optimize_high_score_content_unchanged() {
        let scorer = GeoScorer::new();
        let optimized = scorer.optimize(&sample_content(), 0.3);
        assert_eq!(optimized, sample_content());
    }

    #[test]
    fn test_inject_jsonld() {
        let scorer = GeoScorer::new();
        let result = scorer.inject_jsonld("Hello world", "Test Article", "A test");
        assert!(result.contains("schema.org"));
        assert!(result.contains("Test Article"));
        assert!(result.contains("Hello world"));
    }

    #[test]
    fn test_recency_score_high_with_current_year() {
        let scorer = GeoScorer::new();
        let content = "According to 2026 data, recent studies show improvement. The latest research confirms this. Current evidence is novel and ongoing.";
        let score = scorer.compute_recency(content);
        assert!(score > 0.7, "recency score {} should be > 0.7", score);
    }

    #[test]
    fn test_geo_score_all_channels() {
        let scorer = GeoScorer::new();
        let s = scorer.score("", "");
        assert!(s.authority > 0.0);
        assert!(s.structure > 0.0);
        assert!(s.semantic_completeness > 0.0);
        assert!(s.citation_feasibility > 0.0);
    }
}
