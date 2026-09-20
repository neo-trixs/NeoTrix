//! Knowledge source adapter — unified interface for different knowledge origins.
//!
//! Extracted from batch-absorb.sh / absorb_to_capability.py essential patterns.
//! Each source type implements `SourceAdapter` to produce `KnowledgeInput`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceKind {
    GitHubRepo,
    GitHubTopic,
    ArxivPaper,
    Article,
    WebPage,
    KnowledgeNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeInput {
    pub source_kind: SourceKind,
    pub title: String,
    pub summary: String,
    pub content: Option<String>,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub language: String,
    pub confidence: f64,
    pub importance: f64,
    pub metadata: HashMap<String, serde_json::Value>,
    pub source_specific: SourceSpecific,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceSpecific {
    GitHubRepo {
        full_name: String,
        stars: u64,
        language: Option<String>,
        topics: Vec<String>,
    },
    GitHubTopic {
        topic_name: String,
        total_count: u64,
        top_repos: Vec<String>,
    },
    ArxivPaper {
        paper_id: String,
        authors: Vec<String>,
        categories: Vec<String>,
    },
    Generic,
}

#[derive(Error, Debug)]
pub enum SourceError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("API error: {0}")]
    ApiError(String),
}

pub trait SourceAdapter: Send + Sync {
    fn source_kind(&self) -> SourceKind;
    fn parse(&self, raw: &str) -> Result<KnowledgeInput, SourceError>;
    fn validate(&self, input: &KnowledgeInput) -> bool;
    fn default_domain(&self) -> &str;
    fn default_importance(&self) -> f64 {
        0.5
    }
    fn default_confidence(&self) -> f64 {
        1.0
    }
}

pub struct GitHubRepoAdapter;

impl SourceAdapter for GitHubRepoAdapter {
    fn source_kind(&self) -> SourceKind {
        SourceKind::GitHubRepo
    }
    fn default_domain(&self) -> &str {
        "github.com"
    }

    fn parse(&self, raw: &str) -> Result<KnowledgeInput, SourceError> {
        let input = raw.trim();
        let (owner, repo) = parse_github_url(input)?;
        let full_name = format!("{}/{}", owner, repo);
        let url = format!("https://github.com/{}/{}", owner, repo);

        let mut meta = HashMap::new();
        meta.insert("owner".into(), serde_json::Value::String(owner.clone()));
        meta.insert("repo".into(), serde_json::Value::String(repo.clone()));

        Ok(KnowledgeInput {
            source_kind: SourceKind::GitHubRepo,
            title: full_name.clone(),
            summary: format!("GitHub repository: {}", full_name),
            content: None,
            url: Some(url),
            domain: Some("github.com".into()),
            language: "en".into(),
            confidence: 1.0,
            importance: 0.5,
            metadata: meta,
            source_specific: SourceSpecific::GitHubRepo {
                full_name,
                stars: 0,
                language: None,
                topics: vec![],
            },
        })
    }

    fn validate(&self, input: &KnowledgeInput) -> bool {
        input.url.is_some() && input.source_kind == SourceKind::GitHubRepo
    }
}

pub struct ArxivAdapter;

impl SourceAdapter for ArxivAdapter {
    fn source_kind(&self) -> SourceKind {
        SourceKind::ArxivPaper
    }
    fn default_domain(&self) -> &str {
        "arxiv.org"
    }
    fn default_importance(&self) -> f64 {
        0.6
    }

    fn parse(&self, raw: &str) -> Result<KnowledgeInput, SourceError> {
        let input = raw.trim();
        let paper_id = parse_arxiv_id(input)?;
        let url = format!("https://arxiv.org/abs/{}", paper_id);

        let mut meta = HashMap::new();
        meta.insert(
            "paper_id".into(),
            serde_json::Value::String(paper_id.clone()),
        );

        Ok(KnowledgeInput {
            source_kind: SourceKind::ArxivPaper,
            title: format!("ArXiv {}", paper_id),
            summary: format!("Paper from arxiv: {}", paper_id),
            content: None,
            url: Some(url),
            domain: Some("arxiv.org".into()),
            language: "en".into(),
            confidence: 1.0,
            importance: 0.6,
            metadata: meta,
            source_specific: SourceSpecific::ArxivPaper {
                paper_id,
                authors: vec![],
                categories: vec![],
            },
        })
    }

    fn validate(&self, input: &KnowledgeInput) -> bool {
        input.source_kind == SourceKind::ArxivPaper
    }
}

pub struct GitHubTopicAdapter;

impl SourceAdapter for GitHubTopicAdapter {
    fn source_kind(&self) -> SourceKind {
        SourceKind::GitHubTopic
    }
    fn default_domain(&self) -> &str {
        "github.com/topic"
    }

    fn parse(&self, raw: &str) -> Result<KnowledgeInput, SourceError> {
        let topic = raw.trim().to_string();
        if topic.is_empty() {
            return Err(SourceError::MissingField("topic".into()));
        }
        let url = format!("https://github.com/topics/{}", topic);

        let mut meta = HashMap::new();
        meta.insert("topic".into(), serde_json::Value::String(topic.clone()));

        Ok(KnowledgeInput {
            source_kind: SourceKind::GitHubTopic,
            title: topic.clone(),
            summary: format!("GitHub topic: {}", topic),
            content: None,
            url: Some(url),
            domain: Some("github.com/topic".into()),
            language: "en".into(),
            confidence: 1.0,
            importance: 0.5,
            metadata: meta,
            source_specific: SourceSpecific::GitHubTopic {
                topic_name: topic,
                total_count: 0,
                top_repos: vec![],
            },
        })
    }

    fn validate(&self, input: &KnowledgeInput) -> bool {
        input.source_kind == SourceKind::GitHubTopic
    }
}

pub struct GenericArticleAdapter;

impl SourceAdapter for GenericArticleAdapter {
    fn source_kind(&self) -> SourceKind {
        SourceKind::Article
    }
    fn default_domain(&self) -> &str {
        "unknown"
    }

    fn parse(&self, raw: &str) -> Result<KnowledgeInput, SourceError> {
        let title = raw.trim().to_string();
        if title.is_empty() {
            return Err(SourceError::MissingField("title".into()));
        }
        Ok(KnowledgeInput {
            source_kind: SourceKind::Article,
            title: title.clone(),
            summary: title,
            content: None,
            url: None,
            domain: None,
            language: "en".into(),
            confidence: 0.5,
            importance: 0.5,
            metadata: HashMap::new(),
            source_specific: SourceSpecific::Generic,
        })
    }

    fn validate(&self, input: &KnowledgeInput) -> bool {
        !input.title.is_empty()
    }
}

pub struct SourceAdapterFactory;

impl SourceAdapterFactory {
    pub fn detect(raw: &str) -> Box<dyn SourceAdapter> {
        let lower = raw.trim().to_lowercase();
        if lower.starts_with("https://github.com/")
            || (lower.contains('/') && lower.matches('/').count() == 1)
        {
            Box::new(GitHubRepoAdapter)
        } else if lower.starts_with("https://arxiv.org/") || lower.starts_with("arxiv:") {
            Box::new(ArxivAdapter)
        } else if lower.starts_with("https://github.com/topics/") {
            Box::new(GitHubTopicAdapter)
        } else {
            Box::new(GenericArticleAdapter)
        }
    }
}

fn parse_github_url(input: &str) -> Result<(String, String), SourceError> {
    let path = input
        .strip_prefix("https://github.com/")
        .unwrap_or(input)
        .trim_end_matches('/');
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        Err(SourceError::InvalidUrl(input.to_string()))
    }
}

fn parse_arxiv_id(input: &str) -> Result<String, SourceError> {
    if let Some(id) = input.strip_prefix("https://arxiv.org/abs/") {
        Ok(id.trim_end_matches('/').to_string())
    } else if let Some(id) = input.strip_prefix("arxiv:") {
        Ok(id.to_string())
    } else if input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        && !input.is_empty()
    {
        Ok(input.to_string())
    } else {
        Err(SourceError::InvalidUrl(input.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_parse() {
        let adapter = GitHubRepoAdapter;
        let input = adapter.parse("https://github.com/ollama/ollama").unwrap();
        assert_eq!(input.source_kind, SourceKind::GitHubRepo);
        assert_eq!(input.title, "ollama/ollama");
    }

    #[test]
    fn test_arxiv_parse() {
        let adapter = ArxivAdapter;
        let input = adapter.parse("2605.24517").unwrap();
        assert_eq!(input.source_kind, SourceKind::ArxivPaper);
        assert!(input.url.unwrap().contains("2605.24517"));
    }

    #[test]
    fn test_factory_detection() {
        let adapter = SourceAdapterFactory::detect("https://github.com/foo/bar");
        assert_eq!(adapter.source_kind(), SourceKind::GitHubRepo);

        let adapter = SourceAdapterFactory::detect("2301.12345");
        assert_eq!(adapter.source_kind(), SourceKind::ArxivPaper);
    }
}
