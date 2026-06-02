use std::collections::HashMap;
use std::sync::Arc;

use super::config::{CrawlFormat, CrawlTopic};
use crate::neotrix::provider::{LlmProvider, LlmRequest};

#[derive(Debug, Clone)]
pub struct ClassifiedContent {
    pub url: String,
    pub title: String,
    pub topic: CrawlTopic,
    pub format: CrawlFormat,
    pub confidence: f64,
    pub summary: String,
    pub keywords: Vec<String>,
    pub content_length: usize,
}

pub struct ContentClassifier {
    provider: Option<Arc<dyn LlmProvider>>,
    topic_keywords: HashMap<CrawlTopic, Vec<String>>,
    format_heuristics: HashMap<CrawlFormat, Vec<String>>,
    classification_count: u64,
    topic_distribution: HashMap<CrawlTopic, u64>,
}

impl Default for ContentClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentClassifier {
    pub fn new() -> Self {
        let mut topic_keywords = HashMap::new();

        topic_keywords.insert(CrawlTopic::LawAndGovernance, vec![
            "law".into(), "legal".into(), "statute".into(), "regulation".into(),
            "constitution".into(), "code".into(), "act".into(), "decree".into(),
            "jurisprudence".into(), "court".into(), "judgment".into(), "treaty".into(),
            "convention".into(), "charter".into(), "amendment".into(), "rights".into(),
            "立法".into(), "法律".into(), "法规".into(), "宪法".into(),
        ]);

        topic_keywords.insert(CrawlTopic::PolicyAndRegulation, vec![
            "policy".into(), "governance".into(), "regulation".into(), "directive".into(),
            "framework".into(), "strategy".into(), "action plan".into(), "guideline".into(),
            "public administration".into(), "government".into(), "ministry".into(),
            "政策".into(), "治理".into(), "监管".into(), "指南".into(),
        ]);

        topic_keywords.insert(CrawlTopic::ScienceAndTechnology, vec![
            "science".into(), "technology".into(), "research".into(), "experiment".into(),
            "algorithm".into(), "data".into(), "AI".into(), "machine learning".into(),
            "quantum".into(), "biology".into(), "physics".into(), "chemistry".into(),
            "engineering".into(), "computation".into(), "neural".into(),
        ]);

        topic_keywords.insert(CrawlTopic::HumanitiesAndCulture, vec![
            "culture".into(), "society".into(), "humanity".into(), "art".into(),
            "language".into(), "tradition".into(), "heritage".into(), "civilization".into(),
            "anthropology".into(), "sociology".into(), "ethnic".into(), "identity".into(),
        ]);

        topic_keywords.insert(CrawlTopic::SocietyAndEconomics, vec![
            "economy".into(), "market".into(), "trade".into(), "finance".into(),
            "GDP".into(), "inflation".into(), "employment".into(), "tax".into(),
            "budget".into(), "investment".into(), "monetary".into(), "fiscal".into(),
        ]);

        topic_keywords.insert(CrawlTopic::HealthAndMedicine, vec![
            "health".into(), "medical".into(), "disease".into(), "treatment".into(),
            "clinical".into(), "patient".into(), "diagnosis".into(), "therapy".into(),
            "pharmaceutical".into(), "epidemiology".into(), "vaccine".into(),
        ]);

        topic_keywords.insert(CrawlTopic::EducationAndAcademia, vec![
            "education".into(), "academic".into(), "university".into(), "curriculum".into(),
            "pedagogy".into(), "learning".into(), "school".into(), "degree".into(),
            "scholarship".into(), "research".into(), "publish".into(), "journal".into(),
        ]);

        topic_keywords.insert(CrawlTopic::NewsAndMedia, vec![
            "news".into(), "report".into(), "breaking".into(), "update".into(),
            "press".into(), "media".into(), "journalism".into(), "headline".into(),
            "报道".into(), "新闻".into(), "快讯".into(),
        ]);

        topic_keywords.insert(CrawlTopic::PhilosophyAndEthics, vec![
            "philosophy".into(), "ethics".into(), "moral".into(), "logic".into(),
            "metaphysics".into(), "epistemology".into(), "ontology".into(), "reasoning".into(),
            "consciousness".into(), "free will".into(), "justice".into(), "virtue".into(),
        ]);

        topic_keywords.insert(CrawlTopic::HistoryAndArcheology, vec![
            "history".into(), "historical".into(), "ancient".into(), "archeology".into(),
            "medieval".into(), "empire".into(), "dynasty".into(), "civilization".into(),
            "artifacts".into(), "excavation".into(), "timeline".into(), "century".into(),
        ]);

        topic_keywords.insert(CrawlTopic::ArtsAndLiterature, vec![
            "art".into(), "literature".into(), "poetry".into(), "novel".into(),
            "painting".into(), "sculpture".into(), "music".into(), "dance".into(),
            "theater".into(), "cinema".into(), "aesthetic".into(), "creative".into(),
        ]);

        topic_keywords.insert(CrawlTopic::General, vec![]);

        let mut format_heuristics = HashMap::new();
        format_heuristics.insert(CrawlFormat::LegalDocument, vec![
            "§".into(), "article".into(), "subsection".into(), "paragraph".into(),
            "whereas".into(), "hereby".into(), "pursuant".into(), "notwithstanding".into(),
        ]);
        format_heuristics.insert(CrawlFormat::AcademicPaper, vec![
            "abstract".into(), "introduction".into(), "methodology".into(), "results".into(),
            "conclusion".into(), "references".into(), "doi".into(), "et al".into(),
        ]);
        format_heuristics.insert(CrawlFormat::GovernmentPortal, vec![
            "gov".into(), "government".into(), "official".into(), "agency".into(),
            "department".into(), "ministry".into(), ".gov".into(), ".gouv".into(),
        ]);
        format_heuristics.insert(CrawlFormat::NewsArticle, vec![
            "published".into(), "updated".into(), "reporter".into(), "correspondent".into(),
            "breaking news".into(), "exclusive".into(), "headline".into(),
        ]);
        format_heuristics.insert(CrawlFormat::Encyclopedia, vec![
            "encyclopedia".into(), "wik".into(), "overview".into(), "definition".into(),
            "history".into(), "etymology".into(), "also see".into(),
        ]);
        format_heuristics.insert(CrawlFormat::BlogPost, vec![
            "blog".into(), "posted by".into(), "comments".into(), "subscribe".into(),
            "share this".into(), "related posts".into(),
        ]);
        format_heuristics.insert(CrawlFormat::CodeRepository, vec![
            "repository".into(), "github".into(), "code".into(), "fork".into(),
            "pull request".into(), "commit".into(), "branch".into(),
        ]);
        format_heuristics.insert(CrawlFormat::ReferenceWork, vec![
            "reference".into(), "handbook".into(), "guide".into(), "manual".into(),
            "compendium".into(), "glossary".into(), "index".into(),
        ]);
        format_heuristics.insert(CrawlFormat::DiscussionForum, vec![]);
        format_heuristics.insert(CrawlFormat::OfficialDocument, vec![]);
        format_heuristics.insert(CrawlFormat::Multimedia, vec![]);
        format_heuristics.insert(CrawlFormat::Other, vec![]);

        ContentClassifier {
            provider: None,
            topic_keywords,
            format_heuristics,
            classification_count: 0,
            topic_distribution: HashMap::new(),
        }
    }

    pub fn with_provider(provider: Option<Arc<dyn LlmProvider>>) -> Self {
        let mut classifier = Self::new();
        classifier.provider = provider;
        classifier
    }

    pub async fn try_llm_classify(&self, url: &str, title: &str, text: &str) -> Option<ClassifiedContent> {
        let provider = self.provider.as_ref()?;

        let topic_names: Vec<&str> = CrawlTopic::all().iter().map(CrawlTopic::name).collect();
        let format_names = [
            "legal_document", "academic_paper", "government_portal", "news_article",
            "encyclopedia", "blog_post", "official_document", "discussion_forum",
            "code_repository", "reference_work", "multimedia", "other",
        ];

        let truncated: String = text.chars().take(2000).collect();

        let prompt = format!(
            "Classify this content into one topic from {topics:?} and one format from {formats:?}. \
             Return ONLY valid JSON with keys 'topic', 'format', 'confidence'.\n\n\
             URL: {url}\nTitle: {title}\nContent: {text}",
            topics = topic_names,
            formats = &format_names[..],
            url = url,
            title = title,
            text = truncated,
        );

        let request = LlmRequest::new("default", &prompt)
            .with_max_tokens(200);

        let response = provider.complete(&request).await.ok()?;

        #[derive(serde::Deserialize)]
        struct LlmClassification {
            topic: String,
            format: String,
            confidence: f64,
        }

        let llm: LlmClassification = serde_json::from_str(&response.content).ok()?;

        let topic = CrawlTopic::all().into_iter().find(|t| t.name() == llm.topic)?;

        let format = match llm.format.as_str() {
            "legal_document" => CrawlFormat::LegalDocument,
            "academic_paper" => CrawlFormat::AcademicPaper,
            "government_portal" => CrawlFormat::GovernmentPortal,
            "news_article" => CrawlFormat::NewsArticle,
            "encyclopedia" => CrawlFormat::Encyclopedia,
            "blog_post" => CrawlFormat::BlogPost,
            "official_document" => CrawlFormat::OfficialDocument,
            "discussion_forum" => CrawlFormat::DiscussionForum,
            "code_repository" => CrawlFormat::CodeRepository,
            "reference_work" => CrawlFormat::ReferenceWork,
            "multimedia" => CrawlFormat::Multimedia,
            "other" => CrawlFormat::Other,
            _ => return None,
        };

        let confidence = llm.confidence.clamp(0.0, 1.0);

        Some(ClassifiedContent {
            url: url.to_string(),
            title: title.to_string(),
            topic,
            format,
            confidence,
            summary: text.chars().take(200).collect(),
            keywords: vec![],
            content_length: text.len(),
        })
    }

    pub async fn classify_with_llm_fallback(&mut self, url: &str, title: &str, text: &str) -> ClassifiedContent {
        if let Some(result) = self.try_llm_classify(url, title, text).await {
            self.classification_count += 1;
            *self.topic_distribution.entry(result.topic).or_insert(0) += 1;
            return result;
        }
        self.classify(url, text)
    }

    pub fn classify(&mut self, url: &str, _text: &str) -> ClassifiedContent {
        self.classification_count += 1;

        let text_lower = _text.to_lowercase();
        let content_len = _text.len();

        let topic = self.classify_topic(&text_lower, url);
        let format = self.classify_format(&text_lower, url, &topic);
        let confidence = self.compute_confidence(&text_lower, &topic, &format);
        let keywords = self.extract_keywords(&text_lower, &topic);
        let title = self.extract_title(_text, url);

        *self.topic_distribution.entry(topic).or_insert(0) += 1;

        ClassifiedContent {
            url: url.to_string(),
            title,
            topic,
            format,
            confidence,
            summary: _text.chars().take(200).collect(),
            keywords,
            content_length: content_len,
        }
    }

    fn classify_topic(&self, text: &str, url: &str) -> CrawlTopic {
        let url_lower = url.to_lowercase();

        if url_lower.contains("legislation") || url_lower.contains("law") || url_lower.contains("court")
            || url_lower.contains("juris") || url_lower.contains("constitut")
        {
            return CrawlTopic::LawAndGovernance;
        }
        if url_lower.contains("policy") || url_lower.contains("govern") || url_lower.contains("regulation") {
            return CrawlTopic::PolicyAndRegulation;
        }
        if url_lower.contains("arxiv") || url_lower.contains("nature.com") || url_lower.contains("science") {
            return CrawlTopic::ScienceAndTechnology;
        }
        if url_lower.contains("plato.stanford") || url_lower.contains("philosophy") {
            return CrawlTopic::PhilosophyAndEthics;
        }
        if url_lower.contains("who.int") || url_lower.contains("health") || url_lower.contains("medic") {
            return CrawlTopic::HealthAndMedicine;
        }

        let mut best_topic = CrawlTopic::General;
        let mut best_score = 0usize;

        for (topic, keywords) in &self.topic_keywords {
            if keywords.is_empty() {
                continue;
            }
            let score = keywords.iter().filter(|kw| text.contains(&kw.to_lowercase())).count();
            if score > best_score {
                best_score = score;
                best_topic = *topic;
            }
        }

        best_topic
    }

    fn classify_format(&self, text: &str, url: &str, _topic: &CrawlTopic) -> CrawlFormat {
        let url_lower = url.to_lowercase();

        if url_lower.contains(".gov") || url_lower.contains(".gouv") || url_lower.contains(".go.") {
            return CrawlFormat::GovernmentPortal;
        }
        if url_lower.contains("wikipedia") || url_lower.contains("britannica") || url_lower.contains("wiki") {
            return CrawlFormat::Encyclopedia;
        }
        if url_lower.contains("arxiv") || url_lower.contains("sciencedirect") || url_lower.contains(".edu/") {
            return CrawlFormat::AcademicPaper;
        }
        if url_lower.contains("github") {
            return CrawlFormat::CodeRepository;
        }
        if url_lower.contains("blog") || url_lower.contains("medium") {
            return CrawlFormat::BlogPost;
        }
        if url_lower.contains("news") || url_lower.contains("cnn") || url_lower.contains("bbc") {
            return CrawlFormat::NewsArticle;
        }

        let mut best_format = CrawlFormat::Other;
        let mut best_score = 0usize;

        for (format, heuristics) in &self.format_heuristics {
            if heuristics.is_empty() {
                continue;
            }
            let score = heuristics.iter().filter(|h| text.contains(&h.to_lowercase())).count();
            if score > best_score {
                best_score = score;
                best_format = *format;
            }
        }

        best_format
    }

    fn compute_confidence(&self, text: &str, topic: &CrawlTopic, _format: &CrawlFormat) -> f64 {
        let keywords = self.topic_keywords.get(topic).map_or(0, |kws| kws.len());
        if keywords == 0 {
            return 0.3;
        }

        let matches = self.topic_keywords.get(topic).map_or(0, |kws| {
            kws.iter().filter(|kw| text.contains(&kw.to_lowercase())).count()
        });

        let ratio = matches as f64 / keywords as f64;
        (ratio * 0.8 + 0.2).min(0.95)
    }

    fn extract_keywords(&self, text: &str, topic: &CrawlTopic) -> Vec<String> {
        self.topic_keywords
            .get(topic)
            .map(|kws| {
                let mut found: Vec<String> = kws
                    .iter()
                    .filter(|kw| text.contains(&kw.to_lowercase()))
                    .take(10)
                    .cloned()
                    .collect();
                found.dedup();
                found
            })
            .unwrap_or_default()
    }

    fn extract_title(&self, text: &str, url: &str) -> String {
        if let Some(title_start) = text.find("<title>") {
            let start = title_start + 7;
            if let Some(title_end) = text[start..].find("</title>") {
                return text[start..start + title_end].trim().to_string();
            }
        }
        if let Some(title_start) = text.find("# ") {
            let end = text[title_start + 2..].find('\n').unwrap_or(100);
            return text[title_start + 2..title_start + 2 + end].trim().to_string();
        }
        url.rsplit('/').next().unwrap_or(url).to_string()
    }

    pub fn summary(&self) -> ClassifierSummary {
        ClassifierSummary {
            total_classified: self.classification_count,
            topic_distribution: self.topic_distribution.clone(),
        }
    }
}

pub struct ClassifierSummary {
    pub total_classified: u64,
    pub topic_distribution: HashMap<CrawlTopic, u64>,
}

impl std::fmt::Display for ClassifierSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Classifier: total={}", self.total_classified)?;
        let mut topics: Vec<_> = self.topic_distribution.iter().collect();
        topics.sort_by(|a, b| b.1.cmp(a.1));
        for (topic, count) in topics.iter().take(5) {
            writeln!(f, "  {}: {}", topic.name(), count)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_legal_text() {
        let mut classifier = ContentClassifier::new();
        let legal_text = "This statute law article constitution amendment rights court judgment";
        let result = classifier.classify("https://example.com/law", legal_text);
        assert_eq!(result.topic, CrawlTopic::LawAndGovernance);
        assert!(result.confidence > 0.3);
    }

    #[test]
    fn test_classify_science_text() {
        let mut classifier = ContentClassifier::new();
        let science_text = "The experiment data shows algorithm machine learning quantum physics results";
        let result = classifier.classify("https://arxiv.org/paper", science_text);
        assert_eq!(result.topic, CrawlTopic::ScienceAndTechnology);
    }

    #[test]
    fn test_classify_by_url() {
        let mut classifier = ContentClassifier::new();
        let result = classifier.classify("https://www.constituteproject.org/constitution/Japan_1946", "some law text about constitution");
        assert_eq!(result.topic, CrawlTopic::LawAndGovernance);
    }

    #[test]
    fn test_format_detection() {
        let mut classifier = ContentClassifier::new();
        let result = classifier.classify("https://www.gov.uk/law", "official government act pursuant to section 5");
        assert_eq!(result.format, CrawlFormat::GovernmentPortal);
    }

    #[test]
    fn test_title_extraction() {
        let mut classifier = ContentClassifier::new();
        let html = "<html><title>Test Document</title><body>content</body></html>";
        let result = classifier.classify("https://example.com", html);
        assert_eq!(result.title, "Test Document");
    }

    #[test]
    fn test_keyword_extraction() {
        let mut classifier = ContentClassifier::new();
        let text = "law constitution amendment rights and policy regulation framework";
        let result = classifier.classify("https://example.com", text);
        assert!(!result.keywords.is_empty());
    }

    #[tokio::test]
    async fn test_try_llm_classify_returns_none_when_no_provider() {
        let classifier = ContentClassifier::new();
        let result = classifier.try_llm_classify("https://example.com", "Test", "some content").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_classify_fallback_to_keyword() {
        let mut classifier = ContentClassifier::new();
        let result = classifier.classify_with_llm_fallback("https://example.com/law", "Law Article", "This statute law article constitution amendment").await;
        assert_eq!(result.topic, CrawlTopic::LawAndGovernance);
        assert_eq!(result.format, CrawlFormat::LegalDocument);
    }

    struct MockLlmProvider;

    #[async_trait::async_trait]
    impl LlmProvider for MockLlmProvider {
        async fn complete(&self, _request: &LlmRequest) -> Result<crate::neotrix::provider::LlmResponse, crate::neotrix::provider::LlmError> {
            Ok(crate::neotrix::provider::LlmResponse {
                content: r#"{"topic":"science_and_technology","format":"academic_paper","confidence":0.92}"#.into(),
                model: "mock".into(),
                usage: crate::neotrix::provider::Usage::default(),
                finish_reason: crate::neotrix::provider::FinishReason::Stop,
            })
        }

        async fn stream_complete(&self, _request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<crate::neotrix::provider::LlmResponse, crate::neotrix::provider::LlmError>>, crate::neotrix::provider::LlmError> {
            let (_, rx) = tokio::sync::mpsc::channel(1);
            Ok(rx)
        }
    }

    #[tokio::test]
    async fn test_try_llm_classify_with_mock() {
        let provider: Option<Arc<dyn LlmProvider>> = Some(Arc::new(MockLlmProvider));
        let classifier = ContentClassifier::with_provider(provider);
        let result = classifier.try_llm_classify(
            "https://arxiv.org/abs/2301.00001",
            "Deep Learning Advances",
            "This paper presents a novel neural architecture for transformer networks.",
        ).await;
        assert!(result.is_some());
        let r = result.expect("result should be ok in test");
        assert_eq!(r.topic, CrawlTopic::ScienceAndTechnology);
        assert_eq!(r.format, CrawlFormat::AcademicPaper);
        assert!((r.confidence - 0.92).abs() < 0.01);
    }
}
