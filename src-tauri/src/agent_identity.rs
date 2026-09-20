use crate::atomic_io;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Agent identity — the persistent "who am I" for an agent.
///
/// Modeled after Cumora's `participants` table + persona files.
/// Each agent has a global identity (MEMORY.md) and scoped work
/// facts (projects/<id>/).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    /// Unique agent ID (UUID).
    pub id: String,
    /// Display name (e.g., "Nova", "Iris").
    pub name: String,
    /// Role description (e.g., "Engineer", "Designer").
    pub role: String,
    /// Single-character initial for avatar.
    pub initial: char,
    /// Avatar background hex color.
    pub avatar_color: String,
    /// Status for presence.
    pub status: AgentStatus,
    /// Allowed tool names (empty = all tools).
    pub tools: Vec<String>,
    /// System prompt / persona instructions.
    pub system_prompt: String,
    /// Skills this agent has loaded.
    pub skills: Vec<String>,
    /// Preferred engine for this agent.
    pub preferred_engine: Option<String>,
    /// Preferred model tier.
    pub preferred_tier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Available,
    Resting,
    Thinking,
    Busy,
    Offline,
}

impl AgentStatus {
    pub fn emoji(&self) -> &str {
        match self {
            Self::Available => "🟢",
            Self::Resting => "💤",
            Self::Thinking => "🧠",
            Self::Busy => "🔴",
            Self::Offline => "⚫",
        }
    }
}

impl AgentIdentity {
    /// Create a new agent with sensible defaults.
    pub fn new(name: impl Into<String>, role: impl Into<String>) -> Self {
        let name = name.into();
        let initial = name.chars().next().unwrap_or('A');
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.clone(),
            role: role.into(),
            initial,
            avatar_color: "#6366f1".into(), // indigo-500
            status: AgentStatus::Available,
            tools: Vec::new(),
            system_prompt: String::new(),
            skills: Vec::new(),
            preferred_engine: None,
            preferred_tier: None,
        }
    }

    /// Path to the agent's global memory file.
    pub fn memory_path(&self, workspace: &PathBuf) -> PathBuf {
        workspace.join("memory").join(format!("{}.md", self.id))
    }

    /// Path to the agent's project-scoped memory.
    pub fn project_memory_path(&self, workspace: &PathBuf, project_id: &str) -> PathBuf {
        workspace
            .join("memory")
            .join("projects")
            .join(project_id)
            .join(format!("{}.md", self.id))
    }

    /// Path to the agent's skills directory.
    pub fn skills_path(&self, workspace: &PathBuf) -> PathBuf {
        workspace.join("skills").join(&self.id)
    }
}

/// Memory scope — Cumora pattern of global + scoped memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryScope {
    /// Global agent identity (MEMORY.md).
    Global,
    /// Project-scoped work facts.
    Project(String),
    /// Conversation-scoped ephemeral context.
    Conversation(String),
}

/// Memory entry — a single piece of knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub scope: MemoryScope,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// The memory store — manages agent memories across scopes.
pub struct MemoryStore {
    workspace: PathBuf,
}

impl MemoryStore {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }

    /// Ensure the memory directory structure exists.
    pub fn init(&self) -> Result<(), std::io::Error> {
        let memory_dir = self.workspace.join("memory");
        std::fs::create_dir_all(&memory_dir)?;
        std::fs::create_dir_all(memory_dir.join("projects"))?;
        Ok(())
    }

    /// Read global memory for an agent.
    pub fn read_global(&self, agent_id: &str) -> Result<String, std::io::Error> {
        let path = self.workspace.join("memory").join(format!("{agent_id}.md"));
        if path.exists() {
            String::from_utf8(
                atomic_io::read_with_fallback(&path)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        } else {
            Ok(String::new())
        }
    }

    /// Write global memory for an agent.
    pub fn write_global(&self, agent_id: &str, content: &str) -> Result<(), std::io::Error> {
        let path = self.workspace.join("memory").join(format!("{agent_id}.md"));
        atomic_io::write_atomic(&path, content.as_bytes())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Read project-scoped memory for an agent.
    pub fn read_project(&self, agent_id: &str, project_id: &str) -> Result<String, std::io::Error> {
        let path = self
            .workspace
            .join("memory")
            .join("projects")
            .join(project_id)
            .join(format!("{agent_id}.md"));
        if path.exists() {
            String::from_utf8(
                atomic_io::read_with_fallback(&path)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        } else {
            Ok(String::new())
        }
    }

    /// Write project-scoped memory for an agent.
    pub fn write_project(
        &self,
        agent_id: &str,
        project_id: &str,
        content: &str,
    ) -> Result<(), std::io::Error> {
        let path = self
            .workspace
            .join("memory")
            .join("projects")
            .join(project_id)
            .join(format!("{agent_id}.md"));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        atomic_io::write_atomic(&path, content.as_bytes())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

/// Standing prompt — the Cumora GLANCE_YIELD_RULES pattern.
/// Kept to ~5KB, shape-level, never scenario-specific.
pub const STANDING_PROMPT: &str = r#"# NeoTrix Agent Coordination Protocol

## Core Principles

1. **Reply from real posted state** — never guess what peers will do.
2. **Post optimistically** — the system is your safety net.
3. **Don't repeat a peer** — completion measured by task items, not head count.
4. **Never claim a turn** — claims exist only for genuine shared deliverables.
5. **One named teammate at a time** — if it isn't you, stay out.

## Failure Taxonomy

- Coordination signals → fail-open (duplicate > stall)
- Data mutations → fail-closed (block > corrupt)
- Network calls → fail-open with retry

## Model Tiers

- **Fast**: Triage, classification, simple Q&A
- **Standard**: Normal conversation, code generation
- **Think**: Complex analysis, deep search, planning

Route to the right tier. Don't use Think for "hello"."#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_identity_defaults() {
        let agent = AgentIdentity::new("Nova", "Engineer");
        assert_eq!(agent.name, "Nova");
        assert_eq!(agent.role, "Engineer");
        assert_eq!(agent.initial, 'N');
        assert_eq!(agent.status, AgentStatus::Available);
        assert!(agent.tools.is_empty());
    }

    #[test]
    fn agent_status_emoji() {
        assert_eq!(AgentStatus::Available.emoji(), "🟢");
        assert_eq!(AgentStatus::Busy.emoji(), "🔴");
        assert_eq!(AgentStatus::Thinking.emoji(), "🧠");
    }

    #[test]
    fn memory_paths() {
        let workspace = PathBuf::from("/workspace");
        let agent = AgentIdentity::new("Nova", "Engineer");

        assert_eq!(
            agent.memory_path(&workspace),
            PathBuf::from("/workspace/memory/{id}.md")
                .to_string_lossy()
                .replace("{id}", &agent.id)
        );
    }
}
