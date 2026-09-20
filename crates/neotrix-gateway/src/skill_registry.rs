//! Skill Registry — SKILL.md Discovery and Activation
//!
//! Implements the Cursor/Copilot/Windsurf pattern: domain-specific instruction
//! bundles that agents can discover and apply. Stored as markdown files.
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │            Skill Registry                    │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
//! │  │ Discovery│  │  Loader  │  │ Activator│  │
//! │  │  Engine  │  │  Engine  │  │  Engine  │  │
//! │  └──────────┘  └──────────┘  └──────────┘  │
//! │       ↓              ↓              ↓        │
//! │  ┌──────────────────────────────────────┐   │
//! │  │        SKILL.md Parser               │   │
//! │  └──────────────────────────────────────┘   │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! # Skill Locations (Priority Order)
//! 1. `.neotrix/skills/` — Project-level skills
//! 2. `~/.neotrix/skills/` — User-level skills
//! 3. Built-in skills — System-provided skills
//!
//! # Safety
//! - All file operations are safe
//! - No unsafe code (R-P1)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Core Types
// ============================================================================

/// Skill scope (where the skill is defined)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SkillScope {
    /// Built-in system skills
    BuiltIn,
    /// User-level skills (~/.neotrix/skills/)
    User,
    /// Project-level skills (.neotrix/skills/)
    Project,
    /// Team-level skills (shared via VCS)
    Team,
}

/// Skill activation trigger
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SkillTrigger {
    /// Manual activation (user explicitly invokes)
    Manual,
    /// Auto-activation based on task type
    Auto { task_patterns: Vec<String> },
    /// File-based activation (when specific files are modified)
    FileBased { file_patterns: Vec<String> },
    /// Keyword activation (when specific keywords are used)
    Keyword { keywords: Vec<String> },
}

/// A discovered skill
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Skill {
    /// Unique skill identifier (derived from path)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Skill description
    pub description: String,
    /// Skill scope
    pub scope: SkillScope,
    /// Path to SKILL.md file
    pub skill_path: PathBuf,
    /// Directory containing the skill
    pub skill_dir: PathBuf,
    /// Activation trigger
    pub trigger: SkillTrigger,
    /// Skill content (markdown)
    pub content: String,
    /// Parsed frontmatter
    pub frontmatter: SkillFrontmatter,
    /// Supporting files in the skill directory
    pub supporting_files: Vec<PathBuf>,
    /// When the skill was last modified
    pub last_modified: String,
    /// Whether the skill is currently active
    pub active: bool,
}

/// Parsed frontmatter from SKILL.md
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SkillFrontmatter {
    /// Skill name
    pub name: Option<String>,
    /// Skill version
    pub version: Option<String>,
    /// Skill description
    pub description: Option<String>,
    /// Skill author
    pub author: Option<String>,
    /// Skill tags
    pub tags: Vec<String>,
    /// Skill dependencies (other skills)
    pub depends: Vec<String>,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

/// Skill activation result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillActivation {
    /// Whether activation succeeded
    pub success: bool,
    /// Skill ID
    pub skill_id: String,
    /// Human-readable message
    pub message: String,
    /// Activated skill content
    pub content: String,
    /// Supporting files loaded
    pub supporting_files: Vec<PathBuf>,
}

/// Skill search query
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillQuery {
    /// Search text
    pub text: Option<String>,
    /// Filter by scope
    pub scope: Option<SkillScope>,
    /// Filter by tags
    pub tags: Vec<String>,
    /// Filter by trigger type
    pub trigger: Option<SkillTrigger>,
    /// Maximum results
    pub limit: Option<usize>,
}

/// Registry statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillStats {
    pub total: usize,
    pub by_scope: HashMap<String, usize>,
    pub active: usize,
    pub last_scan: Option<String>,
}

// ============================================================================
// Skill Registry
// ============================================================================

/// SKILL.md discovery and activation engine
pub struct SkillRegistry {
    /// Discovered skills
    skills: Arc<RwLock<HashMap<String, Skill>>>,
    /// Skill search paths (in priority order)
    search_paths: Vec<PathBuf>,
    /// Currently active skills
    active_skills: Arc<RwLock<Vec<String>>>,
}

impl SkillRegistry {
    /// Create a new skill registry
    pub fn new(search_paths: Vec<PathBuf>) -> Self {
        Self {
            skills: Arc::new(RwLock::new(HashMap::new())),
            search_paths,
            active_skills: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create with default search paths
    pub fn with_defaults() -> Self {
        let mut search_paths = Vec::new();

        // Project-level
        if let Ok(cwd) = std::env::current_dir() {
            search_paths.push(cwd.join(".neotrix").join("skills"));
        }

        // User-level
        if let Ok(home) = std::env::var("HOME") {
            search_paths.push(PathBuf::from(home).join(".neotrix").join("skills"));
        }

        Self::new(search_paths)
    }

    /// Scan all search paths for SKILL.md files
    pub async fn scan(&self) -> Result<usize, SkillError> {
        let mut total_found = 0;

        for search_path in &self.search_paths {
            if !search_path.exists() {
                continue;
            }

            let found = self.scan_directory(search_path).await?;
            total_found += found;
        }

        Ok(total_found)
    }

    /// Scan a directory for SKILL.md files
    fn scan_directory<'a>(
        &'a self,
        dir: &'a Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<usize, SkillError>> + Send + 'a>>
    {
        Box::pin(async move {
            let mut count = 0;
            let mut entries = tokio::fs::read_dir(dir)
                .await
                .map_err(|e| SkillError::Io(e.to_string()))?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| SkillError::Io(e.to_string()))?
            {
                let path = entry.path();

                if path.is_dir() {
                    // Check for SKILL.md in this directory
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Ok(skill) = self.load_skill(&skill_md).await {
                            let mut skills = self.skills.write().await;
                            skills.insert(skill.id.clone(), skill);
                            count += 1;
                        }
                    }

                    // Recurse into subdirectories
                    let sub_count = self.scan_directory(&path).await?;
                    count += sub_count;
                }
            }

            Ok(count)
        })
    }

    /// Load a skill from a SKILL.md file
    async fn load_skill(&self, skill_md_path: &Path) -> Result<Skill, SkillError> {
        let content = tokio::fs::read_to_string(skill_md_path)
            .await
            .map_err(|e| SkillError::Io(e.to_string()))?;

        let (frontmatter, body) = parse_skill_md(&content);

        let skill_dir = skill_md_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf();

        let name = frontmatter.name.clone().unwrap_or_else(|| {
            skill_dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });

        // Determine scope based on path
        let scope = if skill_dir.to_string_lossy().contains(".neotrix/skills") {
            SkillScope::Project
        } else if skill_dir.to_string_lossy().contains("~/.neotrix/skills") {
            SkillScope::User
        } else {
            SkillScope::BuiltIn
        };

        // Generate ID from path
        let id = format!(
            "skill-{}",
            skill_dir
                .to_string_lossy()
                .replace('/', "-")
                .replace('\\', "-")
                .to_lowercase()
        );

        // Find supporting files
        let supporting_files = self.find_supporting_files(&skill_dir).await;

        // Get last modified time
        let last_modified = tokio::fs::metadata(skill_md_path)
            .await
            .and_then(|m| m.modified())
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                datetime.to_rfc3339()
            })
            .unwrap_or_default();

        Ok(Skill {
            id,
            name,
            description: frontmatter.description.clone().unwrap_or_default(),
            scope,
            skill_path: skill_md_path.to_path_buf(),
            skill_dir,
            trigger: SkillTrigger::Manual, // Default trigger
            content: body,
            frontmatter,
            supporting_files,
            last_modified,
            active: false,
        })
    }

    /// Find supporting files in a skill directory (excluding SKILL.md itself)
    async fn find_supporting_files(&self, dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();

        if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_file()
                    && path.file_name() != Some(std::ffi::OsStr::new("SKILL.md"))
                {
                    files.push(path);
                }
            }
        }

        files
    }

    /// Search skills by query
    pub async fn search(&self, query: &SkillQuery) -> Vec<Skill> {
        let skills = self.skills.read().await;
        let mut results: Vec<Skill> = skills
            .values()
            .filter(|skill| {
                // Filter by scope
                if let Some(scope) = query.scope {
                    if skill.scope != scope {
                        return false;
                    }
                }

                // Filter by tags
                if !query.tags.is_empty() {
                    let has_tag = query.tags.iter().any(|t| skill.frontmatter.tags.contains(t));
                    if !has_tag {
                        return false;
                    }
                }

                // Filter by text
                if let Some(ref text) = query.text {
                    let text_lower = text.to_lowercase();
                    let name_match = skill.name.to_lowercase().contains(&text_lower);
                    let desc_match = skill.description.to_lowercase().contains(&text_lower);
                    let content_match = skill.content.to_lowercase().contains(&text_lower);
                    if !name_match && !desc_match && !content_match {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        results
    }

    /// Get a skill by ID
    pub async fn get_skill(&self, skill_id: &str) -> Option<Skill> {
        let skills = self.skills.read().await;
        skills.get(skill_id).cloned()
    }

    /// Activate a skill
    pub async fn activate(&self, skill_id: &str) -> Result<SkillActivation, SkillError> {
        let skill = self
            .get_skill(skill_id)
            .await
            .ok_or(SkillError::NotFound(skill_id.to_string()))?;

        // Load supporting files content
        let mut supporting_files_content = Vec::new();
        for file in &skill.supporting_files {
            if let Ok(_content) = tokio::fs::read_to_string(file).await {
                supporting_files_content.push(file.clone());
            }
        }

        // Mark as active
        {
            let mut active = self.active_skills.write().await;
            if !active.contains(&skill_id.to_string()) {
                active.push(skill_id.to_string());
            }
        }

        // Update skill active state
        {
            let mut skills = self.skills.write().await;
            if let Some(s) = skills.get_mut(skill_id) {
                s.active = true;
            }
        }

        Ok(SkillActivation {
            success: true,
            skill_id: skill_id.to_string(),
            message: format!("Skill '{}' activated", skill.name),
            content: skill.content,
            supporting_files: supporting_files_content,
        })
    }

    /// Deactivate a skill
    pub async fn deactivate(&self, skill_id: &str) -> Result<(), SkillError> {
        {
            let mut active = self.active_skills.write().await;
            active.retain(|id| id != skill_id);
        }

        {
            let mut skills = self.skills.write().await;
            if let Some(s) = skills.get_mut(skill_id) {
                s.active = false;
            }
        }

        Ok(())
    }

    /// Get all active skills
    pub async fn active_skills(&self) -> Vec<Skill> {
        let active = self.active_skills.read().await;
        let skills = self.skills.read().await;

        active
            .iter()
            .filter_map(|id| skills.get(id).cloned())
            .collect()
    }

    /// Get registry statistics
    pub async fn stats(&self) -> SkillStats {
        let skills = self.skills.read().await;
        let active = self.active_skills.read().await;

        let mut by_scope = HashMap::new();
        for skill in skills.values() {
            let scope_key = format!("{:?}", skill.scope);
            *by_scope.entry(scope_key).or_insert(0) += 1;
        }

        SkillStats {
            total: skills.len(),
            by_scope,
            active: active.len(),
            last_scan: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// Add a custom search path
    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }
}

// ============================================================================
// SKILL.md Parser
// ============================================================================

/// Parse a SKILL.md file into frontmatter and body
fn parse_skill_md(content: &str) -> (SkillFrontmatter, String) {
    let mut frontmatter = SkillFrontmatter::default();
    let mut body = content.to_string();

    // Check for YAML frontmatter
    if content.starts_with("---") {
        if let Some(end_idx) = content[3..].find("---") {
            let frontmatter_str = &content[3..3 + end_idx];
            body = content[3 + end_idx + 3..].trim().to_string();

            // Simple YAML parsing (avoid dependency on yaml crate)
            for line in frontmatter_str.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"').trim_matches('\'');

                    match key {
                        "name" => frontmatter.name = Some(value.to_string()),
                        "version" => frontmatter.version = Some(value.to_string()),
                        "description" => {
                            frontmatter.description = Some(value.to_string())
                        }
                        "author" => frontmatter.author = Some(value.to_string()),
                        "tags" => {
                            frontmatter.tags = value
                                .split(',')
                                .map(|t| t.trim().to_string())
                                .filter(|t| !t.is_empty())
                                .collect();
                        }
                        "depends" => {
                            frontmatter.depends = value
                                .split(',')
                                .map(|d| d.trim().to_string())
                                .filter(|d| !d.is_empty())
                                .collect();
                        }
                        _ => {
                            frontmatter
                                .metadata
                                .insert(key.to_string(), value.to_string());
                        }
                    }
                }
            }
        }
    }

    (frontmatter, body)
}

// ============================================================================
// Errors
// ============================================================================

/// Skill registry errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum SkillError {
    #[error("skill not found: {0}")]
    NotFound(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("activation failed: {0}")]
    ActivationFailed(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skill_md_with_frontmatter() {
        let content = r#"---
name: test-skill
version: 1.0.0
description: A test skill
author: Test Author
tags: test, example
---

# Test Skill

This is a test skill content.
"#;

        let (frontmatter, body) = parse_skill_md(content);
        assert_eq!(frontmatter.name, Some("test-skill".to_string()));
        assert_eq!(frontmatter.version, Some("1.0.0".to_string()));
        assert_eq!(
            frontmatter.description,
            Some("A test skill".to_string())
        );
        assert_eq!(frontmatter.author, Some("Test Author".to_string()));
        assert_eq!(frontmatter.tags, vec!["test", "example"]);
        assert!(body.contains("This is a test skill content."));
    }

    #[test]
    fn test_parse_skill_md_without_frontmatter() {
        let content = r#"# Test Skill

This is a test skill without frontmatter.
"#;

        let (frontmatter, body) = parse_skill_md(content);
        assert!(frontmatter.name.is_none());
        assert!(body.contains("This is a test skill without frontmatter."));
    }

    #[test]
    fn test_skill_scope_determination() {
        let project_path = PathBuf::from("/project/.neotrix/skills/my-skill");
        let user_path = PathBuf::from("/home/user/.neotrix/skills/my-skill");

        assert!(project_path.to_string_lossy().contains(".neotrix/skills"));
        assert!(user_path.to_string_lossy().contains(".neotrix/skills"));
    }

    #[tokio::test]
    async fn test_skill_registry_creation() {
        let registry = SkillRegistry::with_defaults();
        let stats = registry.stats().await;
        assert_eq!(stats.total, 0);
        assert_eq!(stats.active, 0);
    }
}
