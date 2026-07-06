//! Extensible resource type system for the unified pool.
//!
//! `ResourceKind` categorizes what a resource is (proxy, LLM provider, etc.).
//! `DiscoveredResource` is the intermediate format between discovery and pool registration.
//! `ResourceMeta` carries optional metadata for any resource.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Broad category of a pooled resource.
///
/// Extend this enum when adding new pool types.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceKind {
    /// SOCKS5 / HTTP / Shadowsocks / VLESS proxy node
    #[default]
    Proxy,
    /// LLM provider (free or paid API endpoint)
    LlmProvider,
    /// DNS server (UDP / TCP / TLS / HTTPS)
    DnsServer,
    /// IP resource (raw IP:port)
    IpResource,
    /// Public dataset URL
    Dataset,
    /// Knowledge source (Wikipedia, ArXiv, etc.)
    KnowledgeSource,
    /// Embedding provider
    EmbeddingProvider,
    /// Generic search engine
    SearchEngine,
    /// Public REST API
    PublicApi,
    /// Custom / user-defined type
    Custom(String),
}

impl ResourceKind {
    pub fn as_str(&self) -> &str {
        match self {
            ResourceKind::Proxy => "proxy",
            ResourceKind::LlmProvider => "llm_provider",
            ResourceKind::DnsServer => "dns",
            ResourceKind::IpResource => "ip",
            ResourceKind::Dataset => "dataset",
            ResourceKind::KnowledgeSource => "knowledge_source",
            ResourceKind::EmbeddingProvider => "embedding_provider",
            ResourceKind::SearchEngine => "search_engine",
            ResourceKind::PublicApi => "public_api",
            ResourceKind::Custom(name) => name,
        }
    }

    /// All built-in kinds (for CLI enumeration)
    pub fn all_builtin() -> Vec<ResourceKind> {
        vec![
            ResourceKind::Proxy,
            ResourceKind::LlmProvider,
            ResourceKind::DnsServer,
            ResourceKind::IpResource,
            ResourceKind::Dataset,
            ResourceKind::KnowledgeSource,
            ResourceKind::EmbeddingProvider,
            ResourceKind::SearchEngine,
            ResourceKind::PublicApi,
        ]
    }
}

impl std::fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A resource discovered from an external source (scraper, API, etc.),
/// before it's normalized and registered into the pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredResource {
    /// What kind of resource
    pub kind: ResourceKind,
    /// Unique identifier (URL for proxies, model ID for LLMs, etc.)
    pub resource_id: String,
    /// Human-readable label
    pub label: String,
    /// Source URL / description
    pub source_url: Option<String>,
    /// Whether the resource is free
    pub is_free: bool,
    /// Whether authentication is required
    pub requires_auth: bool,
    /// Optional metadata (protocol, geo, tier, etc.)
    pub meta: HashMap<String, String>,
}

impl DiscoveredResource {
    pub fn new(kind: ResourceKind, resource_id: &str, label: &str) -> Self {
        Self {
            kind,
            resource_id: resource_id.to_string(),
            label: label.to_string(),
            source_url: None,
            is_free: true,
            requires_auth: false,
            meta: HashMap::new(),
        }
    }

    pub fn with_source(mut self, url: &str) -> Self {
        self.source_url = Some(url.to_string());
        self
    }

    pub fn with_meta(mut self, key: &str, value: &str) -> Self {
        self.meta.insert(key.to_string(), value.to_string());
        self
    }
}

/// Persistent metadata stored alongside pooled resources.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceMeta {
    pub kind: ResourceKind,
    pub source: String,
    pub discovered_at: String,
    pub tags: Vec<String>,
    pub extra: HashMap<String, String>,
}
