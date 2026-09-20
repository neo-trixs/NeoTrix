//! Enhanced skill discovery and loading system.
//!
//! Provides index-based skill lookup with categories, tags, dependency tracking,
//! and fast search capabilities. Replaces the legacy filesystem-only scan with
//! a pre-built index that supports rich metadata queries.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Metadata for a single skill entry in the index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEntry {
    /// Human-readable description.
    pub description: String,
    /// Searchable tags for filtering.
    pub tags: Vec<String>,
    /// Trigger words/phrases that activate this skill.
    #[serde(default)]
    pub triggers: Vec<String>,
    /// Skill names this skill depends on.
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Category grouping multiple skills.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCategory {
    pub description: String,
    pub tags: Vec<String>,
    pub skills: HashMap<String, SkillEntry>,
}

/// Index file location mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIndexEntry {
    pub category: String,
    pub file: String,
}

/// Root structure of `skills/index.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIndex {
    pub version: String,
    pub generated: String,
    pub categories: HashMap<String, SkillCategory>,
    pub skill_index: HashMap<String, SkillIndexEntry>,
}

/// Resolved skill with full path and metadata.
#[derive(Debug, Clone)]
pub struct ResolvedSkill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub category: String,
    pub tags: Vec<String>,
    pub triggers: Vec<String>,
    pub dependencies: Vec<String>,
    /// Whether the SKILL.md file actually exists on disk.
    pub exists: bool,
}

/// Search filter for querying skills.
#[derive(Debug, Clone, Default)]
pub struct SkillFilter {
    /// Filter by category name.
    pub category: Option<String>,
    /// Filter by tag (any match).
    pub tags: Vec<String>,
    /// Filter by trigger word (any match).
    pub triggers: Vec<String>,
    /// Text substring match on name or description.
    pub query: Option<String>,
    /// If true, only return skills whose SKILL.md exists on disk.
    pub require_exists: bool,
}

/// Search results with relevance ranking.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub skill: ResolvedSkill,
    /// Relevance score: higher = more relevant.
    pub score: f64,
}

/// Enhanced skill loader with index-based discovery.
pub struct SkillLoader {
    index: Option<SkillIndex>,
    skill_dirs: Vec<PathBuf>,
}

impl SkillLoader {
    /// Create a new loader that searches the given directories for `index.json`
    /// and SKILL.md files.
    pub fn new() -> Self {
        let mut dirs = Vec::new();

        // Workspace skills/
        let ws = PathBuf::from("skills");
        if ws.exists() {
            dirs.push(ws);
        }

        // ~/.neotrix/skills/
        if let Ok(home) = std::env::var("HOME") {
            let home_dir = PathBuf::from(&home).join(".neotrix").join("skills");
            if home_dir.exists() {
                dirs.push(home_dir);
            }
        }

        // ~/.agents/skills/
        if let Ok(home) = std::env::var("HOME") {
            let agents_dir = PathBuf::from(&home).join(".agents").join("skills");
            if agents_dir.exists() {
                dirs.push(agents_dir);
            }
        }

        Self {
            index: None,
            skill_dirs: dirs,
        }
    }

    /// Create a loader with explicit directories.
    pub fn with_dirs(dirs: Vec<PathBuf>) -> Self {
        Self {
            index: None,
            skill_dirs: dirs,
        }
    }

    /// Load the skill index from `skills/index.json` (first found directory).
    pub fn load_index(&mut self) -> Result<&SkillIndex, String> {
        if self.index.is_some() {
            return Ok(self.index.as_ref().unwrap());
        }

        for dir in &self.skill_dirs {
            let index_path = dir.join("index.json");
            if index_path.exists() {
                let content = std::fs::read_to_string(&index_path)
                    .map_err(|e| format!("Failed to read {}: {}", index_path.display(), e))?;
                let idx: SkillIndex = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse {}: {}", index_path.display(), e))?;
                self.index = Some(idx);
                return Ok(self.index.as_ref().unwrap());
            }
        }

        Err("No index.json found in any skill directory".to_string())
    }

    /// List all known skills from the index, optionally resolving paths.
    pub fn list_skills(&mut self) -> Result<Vec<ResolvedSkill>, String> {
        let skill_dirs = self.skill_dirs.clone();
        let index = self.load_index()?;

        // Build path lookup from skill_index
        let mut path_map: HashMap<&str, &SkillIndexEntry> = HashMap::new();
        for (name, entry) in &index.skill_index {
            path_map.insert(name.as_str(), entry);
        }

        let mut skills = Vec::new();

        for (cat_name, category) in &index.categories {
            for (skill_name, entry) in &category.skills {
                // Resolve the full path from skill_index
                let file_path = path_map
                    .get(skill_name.as_str())
                    .map(|idx| {
                        skill_dirs
                            .iter()
                            .map(|d| d.join(&idx.file))
                            .find(|p| p.exists())
                            .unwrap_or_else(|| skill_dirs[0].join(&idx.file))
                    })
                    .unwrap_or_else(|| PathBuf::from(format!("skills/{}", skill_name)));

                let exists = file_path.exists();

                skills.push(ResolvedSkill {
                    name: skill_name.clone(),
                    description: entry.description.clone(),
                    path: file_path,
                    category: cat_name.clone(),
                    tags: entry.tags.clone(),
                    triggers: entry.triggers.clone(),
                    dependencies: entry.dependencies.clone(),
                    exists,
                });
            }
        }

        Ok(skills)
    }

    /// Load a specific skill by name.
    pub fn load_skill(&mut self, name: &str) -> Result<ResolvedSkill, String> {
        let skills = self.list_skills()?;
        skills
            .into_iter()
            .find(|s| s.name == name)
            .ok_or_else(|| format!("Skill '{}' not found", name))
    }

    /// Search skills using a filter. Returns results sorted by relevance score.
    pub fn search_skills(&mut self, filter: &SkillFilter) -> Result<Vec<SearchResult>, String> {
        let skills = self.list_skills()?;
        let mut results: Vec<SearchResult> = skills
            .into_iter()
            .filter(|skill| self.matches_filter(skill, filter))
            .map(|skill| {
                let score = self.compute_score(&skill, filter);
                SearchResult { skill, score }
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(results)
    }

    /// Get all skills that the given skill depends on (recursive).
    pub fn get_dependencies(&mut self, name: &str) -> Result<Vec<ResolvedSkill>, String> {
        let index = self.load_index()?.clone();
        let mut visited = HashSet::new();
        let mut result = Vec::new();

        self.collect_deps(name, &index, &mut visited, &mut result)?;
        Ok(result)
    }

    /// Get all skills that depend on the given skill (reverse dependencies).
    pub fn get_dependents(&mut self, name: &str) -> Result<Vec<ResolvedSkill>, String> {
        let skills = self.list_skills()?;
        let dependents: Vec<ResolvedSkill> = skills
            .into_iter()
            .filter(|s| s.dependencies.contains(&name.to_string()))
            .collect();
        Ok(dependents)
    }

    /// Get all skills in a category.
    pub fn get_category(&mut self, category: &str) -> Result<Vec<ResolvedSkill>, String> {
        let skills = self.list_skills()?;
        Ok(skills
            .into_iter()
            .filter(|s| s.category == category)
            .collect())
    }

    /// Fallback: scan filesystem without index (legacy compatibility).
    pub fn scan_legacy() -> Vec<ResolvedSkill> {
        let mut skills = Vec::new();
        let mut seen = HashSet::new();

        // Workspace skills/
        let ws = Path::new("skills");
        if ws.exists() {
            Self::scan_dir_legacy(ws, &mut seen, &mut skills);
        }

        // ~/.neotrix/skills/
        if let Ok(home) = std::env::var("HOME") {
            let home_dir = PathBuf::from(&home).join(".neotrix").join("skills");
            if home_dir.exists() {
                Self::scan_dir_legacy(&home_dir, &mut seen, &mut skills);
            }
        }

        // ~/.agents/skills/
        if let Ok(home) = std::env::var("HOME") {
            let agents_dir = PathBuf::from(&home).join(".agents").join("skills");
            if agents_dir.exists() {
                Self::scan_dir_legacy(&agents_dir, &mut seen, &mut skills);
            }
        }

        skills
    }

    // -- Private helpers --

    fn matches_filter(&self, skill: &ResolvedSkill, filter: &SkillFilter) -> bool {
        if let Some(ref cat) = filter.category {
            if &skill.category != cat {
                return false;
            }
        }

        if !filter.tags.is_empty() {
            let has_tag = filter.tags.iter().any(|t| skill.tags.contains(t));
            if !has_tag {
                return false;
            }
        }

        if !filter.triggers.is_empty() {
            let has_trigger = filter
                .triggers
                .iter()
                .any(|t| skill.triggers.iter().any(|st| st.contains(t)));
            if !has_trigger {
                return false;
            }
        }

        if let Some(ref query) = filter.query {
            let q = query.to_lowercase();
            let name_match = skill.name.to_lowercase().contains(&q);
            let desc_match = skill.description.to_lowercase().contains(&q);
            let tag_match = skill.tags.iter().any(|t| t.to_lowercase().contains(&q));
            if !name_match && !desc_match && !tag_match {
                return false;
            }
        }

        if filter.require_exists && !skill.exists {
            return false;
        }

        true
    }

    fn compute_score(&self, skill: &ResolvedSkill, filter: &SkillFilter) -> f64 {
        let mut score = 0.0;

        // Exact name match gets highest score
        if let Some(ref query) = filter.query {
            let q = query.to_lowercase();
            if skill.name.to_lowercase() == q {
                score += 100.0;
            } else if skill.name.to_lowercase().contains(&q) {
                score += 50.0;
            }
            if skill.description.to_lowercase().contains(&q) {
                score += 20.0;
            }
        }

        // Tag matches
        if !filter.tags.is_empty() {
            let tag_hits = filter
                .tags
                .iter()
                .filter(|t| skill.tags.contains(*t))
                .count();
            score += (tag_hits as f64) * 10.0;
        }

        // Trigger matches
        if !filter.triggers.is_empty() {
            let trigger_hits = filter
                .triggers
                .iter()
                .filter(|t| skill.triggers.iter().any(|st| st.contains(*t)))
                .count();
            score += (trigger_hits as f64) * 15.0;
        }

        // Bonus for existing on disk
        if skill.exists {
            score += 5.0;
        }

        // Bonus for fewer dependencies (simpler = more likely standalone)
        score += (10.0 - skill.dependencies.len() as f64).max(0.0);

        score
    }

    fn collect_deps(
        &self,
        name: &str,
        index: &SkillIndex,
        visited: &mut HashSet<String>,
        result: &mut Vec<ResolvedSkill>,
    ) -> Result<(), String> {
        if visited.contains(name) {
            return Ok(());
        }
        visited.insert(name.to_string());

        // Find the skill in the index
        for (cat_name, category) in &index.categories {
            if let Some(entry) = category.skills.get(name) {
                let file_path = index
                    .skill_index
                    .get(name)
                    .map(|idx| PathBuf::from(format!("skills/{}", idx.file)))
                    .unwrap_or_else(|| PathBuf::from(format!("skills/{}", name)));

                result.push(ResolvedSkill {
                    name: name.to_string(),
                    description: entry.description.clone(),
                    path: file_path,
                    category: cat_name.clone(),
                    tags: entry.tags.clone(),
                    triggers: entry.triggers.clone(),
                    dependencies: entry.dependencies.clone(),
                    exists: false, // Not resolving in this context
                });

                // Recurse into dependencies
                for dep in &entry.dependencies {
                    self.collect_deps(dep, index, visited, result)?;
                }
                break;
            }
        }

        Ok(())
    }

    fn scan_dir_legacy(dir: &Path, seen: &mut HashSet<String>, skills: &mut Vec<ResolvedSkill>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if seen.contains(&name) {
                        continue;
                    }

                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
                        let description = Self::extract_description_legacy(&content);
                        seen.insert(name.clone());

                        skills.push(ResolvedSkill {
                            name,
                            description,
                            path: skill_md,
                            category: String::new(),
                            tags: Vec::new(),
                            triggers: Vec::new(),
                            dependencies: Vec::new(),
                            exists: true,
                        });
                    }
                } else if let Some(ext) = path.extension() {
                    if ext == "json" {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            let name_owned = name.to_string();
                            if seen.contains(&name_owned) {
                                continue;
                            }
                            seen.insert(name_owned.clone());
                            skills.push(ResolvedSkill {
                                name: name_owned,
                                description: format!(".skill.json: {}", path.display()),
                                path,
                                category: String::new(),
                                tags: Vec::new(),
                                triggers: Vec::new(),
                                dependencies: Vec::new(),
                                exists: true,
                            });
                        }
                    }
                }
            }
        }
    }

    fn extract_description_legacy(content: &str) -> String {
        // Try YAML frontmatter first
        if content.starts_with("---") {
            if let Some(end) = content[3..].find("---") {
                let frontmatter = &content[3..3 + end];
                for line in frontmatter.lines() {
                    if let Some(val) = line.strip_prefix("description:") {
                        return val.trim().to_string().trim_matches('"').to_string();
                    }
                }
            }
        }
        // Fallback to line-by-line
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("description:") {
                return val.trim().to_string();
            }
        }
        String::new()
    }
}

impl Default for SkillLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function: load skill by name using default loader.
pub fn load_skill(name: &str) -> Result<ResolvedSkill, String> {
    SkillLoader::new().load_skill(name)
}

/// Convenience function: list all skills using default loader.
pub fn list_skills() -> Result<Vec<ResolvedSkill>, String> {
    SkillLoader::new().list_skills()
}

/// Convenience function: search skills using default loader.
pub fn search_skills(filter: &SkillFilter) -> Result<Vec<SearchResult>, String> {
    SkillLoader::new().search_skills(filter)
}

/// Convenience function: get dependencies of a skill.
pub fn get_dependencies(name: &str) -> Result<Vec<ResolvedSkill>, String> {
    SkillLoader::new().get_dependencies(name)
}

/// Convenience function: get skills in a category.
pub fn get_category(category: &str) -> Result<Vec<ResolvedSkill>, String> {
    SkillLoader::new().get_category(category)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_loader_new() {
        let loader = SkillLoader::new();
        assert!(!loader.skill_dirs.is_empty());
    }

    #[test]
    fn test_scan_legacy() {
        let skills = SkillLoader::scan_legacy();
        // At least the workspace skills should be found
        assert!(!skills.is_empty() || !Path::new("skills").exists());
    }

    #[test]
    fn test_extract_description_legacy() {
        let content = "---\nname: test\ndescription: A test skill\n---\n# Title";
        assert_eq!(
            SkillLoader::extract_description_legacy(content),
            "A test skill"
        );
    }

    #[test]
    fn test_extract_description_fallback() {
        let content = "# Title\ndescription: Fallback description";
        assert_eq!(
            SkillLoader::extract_description_legacy(content),
            "Fallback description"
        );
    }
}
