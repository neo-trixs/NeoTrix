//! nt_resource_types — 资源描述子类型 (ResourceSource/ResourceDescriptor/ResourceIngestResult)，行为零变更纯搬移。

use super::super::nt_memory_types::NodeType;

#[derive(Debug, Clone)]
pub enum ResourceSource {
    GitHub { owner: String, repo: String },
    ArXiv { id: String },
    Web { url: String },
    Direct, // conceptual / built-in
}

impl ResourceSource {
    pub fn url(&self) -> Option<String> {
        match self {
            ResourceSource::GitHub { owner, repo } => {
                Some(format!("https://github.com/{}/{}", owner, repo))
            }
            ResourceSource::ArXiv { id } => Some(format!("https://arxiv.org/abs/{}", id)),
            ResourceSource::Web { url } => Some(url.clone()),
            ResourceSource::Direct => None,
        }
    }

    pub fn domain(&self) -> Option<String> {
        match self {
            ResourceSource::GitHub { .. } => Some("github.com".into()),
            ResourceSource::ArXiv { .. } => Some("arxiv.org".into()),
            ResourceSource::Web { url } => url
                .split('/')
                .nth(2)
                .or(url.split('/').nth(0))
                .map(|d| d.to_string()),
            ResourceSource::Direct => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceDescriptor {
    pub category: NodeType,
    pub title: String,
    pub summary: String,
    pub content: Option<String>,
    pub source: ResourceSource,
    pub key_insights: Vec<String>,
    pub tags: Vec<String>,
    pub importance: f64,
    pub confidence: f64,
}

impl ResourceDescriptor {
    pub fn github(owner: &str, repo: &str, title: &str, summary: &str) -> Self {
        Self {
            category: NodeType::Repository,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::GitHub {
                owner: owner.to_string(),
                repo: repo.to_string(),
            },
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.7,
            confidence: 0.9,
        }
    }

    pub fn paper(arxiv_id: &str, title: &str, summary: &str) -> Self {
        Self {
            category: NodeType::Paper,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::ArXiv {
                id: arxiv_id.to_string(),
            },
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.8,
            confidence: 0.9,
        }
    }

    pub fn article(title: &str, summary: &str, url: &str) -> Self {
        Self {
            category: NodeType::Article,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::Web {
                url: url.to_string(),
            },
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.6,
            confidence: 0.8,
        }
    }

    pub fn tool(name: &str, summary: &str, source: ResourceSource) -> Self {
        Self {
            category: NodeType::Tool,
            title: name.to_string(),
            summary: summary.to_string(),
            content: None,
            source,
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.6,
            confidence: 0.8,
        }
    }

    pub fn concept(title: &str, summary: &str) -> Self {
        Self {
            category: NodeType::Concept,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::Direct,
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.5,
            confidence: 0.7,
        }
    }

    pub fn insight(title: &str, summary: &str) -> Self {
        Self {
            category: NodeType::Insight,
            title: title.to_string(),
            summary: summary.to_string(),
            content: None,
            source: ResourceSource::Direct,
            key_insights: Vec::new(),
            tags: Vec::new(),
            importance: 0.5,
            confidence: 0.7,
        }
    }

    pub fn with_key_insights(mut self, insights: Vec<&str>) -> Self {
        self.key_insights = insights.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = importance;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }
}

#[derive(Debug)]
pub struct ResourceIngestResult {
    pub node_id: String,
    pub insight_ids: Vec<String>,
}
