use serde::{Deserialize, Serialize};

/// Artifacts — Grok Bot durable output pattern.
///
/// Agents produce artifacts: documents, code files, data exports, reports.
/// Artifacts are versioned, typed, and stored persistently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub kind: ArtifactKind,
    pub title: String,
    pub content: ArtifactContent,
    pub author_id: String,
    pub version: u32,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactKind {
    Document,
    Code,
    Data,
    Report,
    Image,
    Markdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactContent {
    Text(String),
    Binary(Vec<u8>),
    Structured(serde_json::Value),
}

/// Artifact store — manages produced artifacts.
pub struct ArtifactStore {
    artifacts: Vec<Artifact>,
}

impl ArtifactStore {
    pub fn new() -> Self {
        Self {
            artifacts: Vec::new(),
        }
    }

    /// Create a new artifact.
    pub fn create(
        &mut self,
        kind: ArtifactKind,
        title: String,
        content: ArtifactContent,
        author_id: &str,
    ) -> Artifact {
        let artifact = Artifact {
            id: uuid::Uuid::new_v4().to_string(),
            kind,
            title: title.clone(),
            content,
            author_id: author_id.to_string(),
            version: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            tags: Vec::new(),
            path: None,
        };
        tracing::info!("created artifact: {title}");
        self.artifacts.push(artifact.clone());
        artifact
    }

    /// Update an artifact's content (creates new version).
    pub fn update(&mut self, artifact_id: &str, content: ArtifactContent) -> Option<&Artifact> {
        let artifact = self.artifacts.iter_mut().find(|a| a.id == artifact_id)?;
        artifact.content = content;
        artifact.version += 1;
        artifact.updated_at = chrono::Utc::now().to_rfc3339();
        Some(artifact)
    }

    /// Get an artifact by ID.
    pub fn get(&self, id: &str) -> Option<&Artifact> {
        self.artifacts.iter().find(|a| a.id == id)
    }

    /// List all artifacts.
    pub fn list_all(&self) -> &[Artifact] {
        &self.artifacts
    }

    /// List artifacts by kind.
    pub fn list_by_kind(&self, kind: &ArtifactKind) -> Vec<&Artifact> {
        self.artifacts.iter().filter(|a| &a.kind == kind).collect()
    }

    /// List artifacts by author.
    pub fn list_by_author(&self, author_id: &str) -> Vec<&Artifact> {
        self.artifacts
            .iter()
            .filter(|a| a.author_id == author_id)
            .collect()
    }
}

impl Default for ArtifactStore {
    fn default() -> Self {
        Self::new()
    }
}
