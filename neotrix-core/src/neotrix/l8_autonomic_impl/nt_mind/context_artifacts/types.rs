use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtifactType {
    DatabaseSchema,
    ApiSpec,
    ArchitectureDoc,
    ConfigFile,
    InfrastructureSpec,
}

impl ArtifactType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::DatabaseSchema => "database_schema",
            Self::ApiSpec => "api_spec",
            Self::ArchitectureDoc => "architecture_doc",
            Self::ConfigFile => "config_file",
            Self::InfrastructureSpec => "infrastructure_spec",
        }
    }

    pub fn parse_artifact_kind(s: &str) -> Option<Self> {
        match s {
            "database_schema" | "DatabaseSchema" => Some(Self::DatabaseSchema),
            "api_spec" | "ApiSpec" => Some(Self::ApiSpec),
            "architecture_doc" | "ArchitectureDoc" => Some(Self::ArchitectureDoc),
            "config_file" | "ConfigFile" => Some(Self::ConfigFile),
            "infrastructure_spec" | "InfrastructureSpec" => Some(Self::InfrastructureSpec),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub artifact_type: ArtifactType,
    pub content: String,
    pub tags: Vec<String>,
    pub source_path: Option<String>,
}

impl Artifact {
    pub fn new(name: &str, artifact_type: ArtifactType, content: &str) -> Self {
        Self {
            name: name.to_string(),
            artifact_type,
            content: content.to_string(),
            tags: Vec::new(),
            source_path: None,
        }
    }

    pub fn with_tags(mut self, tags: &[&str]) -> Self {
        self.tags = tags.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_source(mut self, path: &str) -> Self {
        self.source_path = Some(path.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactSourceConfig {
    pub name: String,
    pub path: String,
    pub artifact_type: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactsConfig {
    pub sources: Vec<ArtifactSourceConfig>,
}
