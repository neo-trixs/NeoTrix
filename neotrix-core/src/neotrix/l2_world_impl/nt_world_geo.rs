use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeoCategory {
    BrandVisibility,
    ContentAuthority,
    TechnicalSeo,
    UserExperience,
    SocialProof,
}

impl GeoCategory {
    pub fn name(&self) -> &'static str {
        match self {
            GeoCategory::BrandVisibility => "Brand Visibility",
            GeoCategory::ContentAuthority => "Content Authority",
            GeoCategory::TechnicalSeo => "Technical SEO",
            GeoCategory::UserExperience => "User Experience",
            GeoCategory::SocialProof => "Social Proof",
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            GeoCategory::BrandVisibility => 0.25,
            GeoCategory::ContentAuthority => 0.30,
            GeoCategory::TechnicalSeo => 0.15,
            GeoCategory::UserExperience => 0.15,
            GeoCategory::SocialProof => 0.15,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeoFactor {
    pub name: String,
    pub category: GeoCategory,
    pub score: f64,
    pub max_score: f64,
    pub description: String,
}

impl GeoFactor {
    pub fn new(
        name: impl Into<String>,
        category: GeoCategory,
        score: f64,
        max_score: f64,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            score: score.max(0.0).min(max_score),
            max_score,
            description: description.into(),
        }
    }

    pub fn normalized_score(&self) -> f64 {
        if self.max_score == 0.0 {
            return 0.0;
        }
        (self.score / self.max_score).max(0.0).min(1.0)
    }
}

#[derive(Debug, Clone)]
pub struct GeoAudit {
    pub factors: Vec<GeoFactor>,
    pub keyword: String,
    pub url: String,
    pub timestamp: u64,
}

impl GeoAudit {
    pub fn new(keyword: impl Into<String>, url: impl Into<String>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        Self {
            factors: Vec::new(),
            keyword: keyword.into(),
            url: url.into(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn add_factor(&mut self, factor: GeoFactor) {
        self.factors.push(factor);
    }

    pub fn overall_score(&self) -> f64 {
        if self.factors.is_empty() {
            return 0.0;
        }
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;
        for f in &self.factors {
            let w = f.category.weight();
            weighted_sum += f.normalized_score() * w;
            total_weight += w;
        }
        if total_weight == 0.0 {
            return 0.0;
        }
        (weighted_sum / total_weight * 100.0).max(0.0).min(100.0)
    }

    pub fn category_scores(&self) -> HashMap<GeoCategory, f64> {
        let mut scores: HashMap<GeoCategory, Vec<f64>> = HashMap::new();
        for f in &self.factors {
            scores.entry(f.category).or_default().push(f.normalized_score());
        }
        scores
            .into_iter()
            .map(|(cat, vals)| {
                let avg: f64 = vals.iter().sum::<f64>() / vals.len() as f64;
                (cat, (avg * 100.0).max(0.0).min(100.0))
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    pub factor: String,
    pub category: GeoCategory,
    pub priority: String,
    pub expected_impact: f64,
    pub action: String,
}

pub struct GeoEngine;

impl GeoEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, keyword: &str, url: &str) -> GeoAudit {
        let mut audit = GeoAudit::new(keyword, url);
        audit.add_factor(GeoFactor::new(
            "keyword_in_title", GeoCategory::BrandVisibility,
            0.0, 10.0, "Whether the keyword appears in the page title",
        ));
        audit.add_factor(GeoFactor::new(
            "keyword_in_headings", GeoCategory::ContentAuthority,
            0.0, 10.0, "Keyword presence in H1/H2 headings",
        ));
        audit.add_factor(GeoFactor::new(
            "page_load_speed", GeoCategory::TechnicalSeo,
            0.0, 10.0, "Estimated page load performance",
        ));
        audit.add_factor(GeoFactor::new(
            "mobile_friendly", GeoCategory::UserExperience,
            0.0, 10.0, "Mobile responsiveness score",
        ));
        audit.add_factor(GeoFactor::new(
            "backlink_count", GeoCategory::SocialProof,
            0.0, 10.0, "Number of referring domains",
        ));
        audit
    }

    pub fn generate_suggestions(audit: &GeoAudit) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();
        for f in &audit.factors {
            if f.normalized_score() < 0.5 {
                let impact = (0.5 - f.normalized_score()) * 100.0;
                let priority = if impact > 30.0 {
                    "high"
                } else if impact > 15.0 {
                    "medium"
                } else {
                    "low"
                };
                suggestions.push(OptimizationSuggestion {
                    factor: f.name.clone(),
                    category: f.category,
                    priority: priority.to_string(),
                    expected_impact: impact,
                    action: format!("Improve '{}' — current score {:.0}% of target", f.name, f.normalized_score() * 100.0),
                });
            }
        }
        suggestions.sort_by(|a, b| b.expected_impact.partial_cmp(&a.expected_impact).unwrap_or(std::cmp::Ordering::Equal));
        suggestions
    }
}

impl Default for GeoEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub fn benchmark_score(score: f64) -> &'static str {
    if score >= 80.0 {
        "excellent"
    } else if score >= 60.0 {
        "good"
    } else if score >= 40.0 {
        "fair"
    } else {
        "poor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_category_name() {
        assert_eq!(GeoCategory::BrandVisibility.name(), "Brand Visibility");
    }

    #[test]
    fn test_geo_category_weights_sum() {
        let cats = [
            GeoCategory::BrandVisibility,
            GeoCategory::ContentAuthority,
            GeoCategory::TechnicalSeo,
            GeoCategory::UserExperience,
            GeoCategory::SocialProof,
        ];
        let total: f64 = cats.iter().map(|c| c.weight()).sum();
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_geo_factor_normalized_score() {
        let f = GeoFactor::new("test", GeoCategory::BrandVisibility, 5.0, 10.0, "test");
        assert_eq!(f.normalized_score(), 0.5);
    }

    #[test]
    fn test_geo_factor_clamp() {
        let f = GeoFactor::new("test", GeoCategory::BrandVisibility, 15.0, 10.0, "test");
        assert_eq!(f.score, 10.0);
    }

    #[test]
    fn test_empty_audit_score() {
        let audit = GeoAudit::new("test", "https://example.com");
        assert_eq!(audit.overall_score(), 0.0);
    }

    #[test]
    fn test_audit_overall_score() {
        let mut audit = GeoAudit::new("hello", "https://example.com");
        audit.add_factor(GeoFactor::new("a", GeoCategory::BrandVisibility, 10.0, 10.0, "a"));
        audit.add_factor(GeoFactor::new("b", GeoCategory::ContentAuthority, 10.0, 10.0, "b"));
        let score = audit.overall_score();
        assert!((score - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_category_scores() {
        let mut audit = GeoAudit::new("test", "https://example.com");
        audit.add_factor(GeoFactor::new("a", GeoCategory::BrandVisibility, 5.0, 10.0, "a"));
        audit.add_factor(GeoFactor::new("b", GeoCategory::BrandVisibility, 8.0, 10.0, "b"));
        let scores = audit.category_scores();
        let bv = scores.get(&GeoCategory::BrandVisibility).unwrap();
        assert!((*bv - 65.0).abs() < 0.001);
    }

    #[test]
    fn test_generate_suggestions() {
        let mut audit = GeoAudit::new("test", "https://example.com");
        audit.add_factor(GeoFactor::new("a", GeoCategory::BrandVisibility, 1.0, 10.0, "low"));
        audit.add_factor(GeoFactor::new("b", GeoCategory::ContentAuthority, 9.0, 10.0, "high"));
        let suggestions = GeoEngine::generate_suggestions(&audit);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].factor, "a");
    }

    #[test]
    fn test_generate_suggestions_all_good() {
        let mut audit = GeoAudit::new("test", "https://example.com");
        audit.add_factor(GeoFactor::new("a", GeoCategory::BrandVisibility, 9.0, 10.0, "a"));
        audit.add_factor(GeoFactor::new("b", GeoCategory::ContentAuthority, 8.0, 10.0, "b"));
        let suggestions = GeoEngine::generate_suggestions(&audit);
        assert_eq!(suggestions.len(), 0);
    }

    #[test]
    fn test_benchmark_score() {
        assert_eq!(benchmark_score(85.0), "excellent");
        assert_eq!(benchmark_score(70.0), "good");
        assert_eq!(benchmark_score(50.0), "fair");
        assert_eq!(benchmark_score(20.0), "poor");
    }

    #[test]
    fn test_geo_engine_analyze() {
        let engine = GeoEngine::new();
        let audit = engine.analyze("rust programming", "https://example.com");
        assert_eq!(audit.keyword, "rust programming");
        assert_eq!(audit.factors.len(), 5);
    }

    #[test]
    fn test_geo_factor_description() {
        let f = GeoFactor::new("test", GeoCategory::TechnicalSeo, 7.0, 10.0, "page speed");
        assert_eq!(f.description, "page speed");
    }

    #[test]
    fn test_suggestions_sorted_by_impact() {
        let mut audit = GeoAudit::new("test", "https://example.com");
        audit.add_factor(GeoFactor::new("low_impact", GeoCategory::BrandVisibility, 8.0, 10.0, "a"));
        audit.add_factor(GeoFactor::new("high_impact", GeoCategory::ContentAuthority, 1.0, 10.0, "b"));
        audit.add_factor(GeoFactor::new("medium_impact", GeoCategory::TechnicalSeo, 4.0, 10.0, "c"));
        let suggestions = GeoEngine::generate_suggestions(&audit);
        assert!(suggestions.len() >= 2);
        assert_eq!(suggestions[0].factor, "high_impact");
    }
}
