//! # AGENTS.md Discovery System
//!
//! Reverse-engineered from Claude Code's CLAUDE.md system.
//! Key insight: CLAUDE.md is injected as `<system-reminder>`, NOT as system prompt.
//! This preserves the shared prompt cache (all users share cached system prefix).
//!
//! # Discovery Order (broadest → most specific)
//! 1. Organization policy: /etc/neotrix/AGENTS.md
//! 2. User global: ~/.neotrix/AGENTS.md
//! 3. Project root: ./AGENTS.md or ./.neotrix/AGENTS.md
//! 4. Subdirectories: ./src/AGENTS.md (on-demand when accessing files there)
//! 5. Project-scoped user: ~/.neotrix/projects/<hash>/AGENTS.md
//!
//! # Injection Mechanism
//! Content is wrapped in `<system-reminder>` XML and attached to conversation messages,
//! NOT injected into the system prompt. This preserves prompt cache sharing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// A discovered AGENTS.md entry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentsMdEntry {
    /// Absolute path to the file
    pub path: PathBuf,
    /// Scope level (0=org, 1=user, 2=project, 3=subdirectory, 4=project-user)
    pub scope: u8,
    /// File content
    pub content: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified timestamp
    pub modified: String,
    /// Whether this file is shared (in git) or private
    pub shared: bool,
}

/// Discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Organization-level search paths
    pub org_paths: Vec<PathBuf>,
    /// User-level search paths
    pub user_paths: Vec<PathBuf>,
    /// Project-level file names to search
    pub project_names: Vec<String>,
    /// Maximum file size (bytes, default 4MB)
    pub max_file_size: u64,
    /// Maximum expanded patterns (default 1000)
    pub max_patterns: usize,
    /// Whether to load subdirectory AGENTS.md on-demand
    pub lazy_subdirs: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            org_paths: vec![
                PathBuf::from("/etc/neotrix"),
                PathBuf::from("/Library/Application Support/NeoTrix"),
            ],
            user_paths: vec![
                dirs::home_dir()
                    .map(|h| h.join(".neotrix"))
                    .unwrap_or_default(),
            ],
            project_names: vec![
                "AGENTS.md".to_string(),
                ".neotrix/AGENTS.md".to_string(),
                ".neotrix/rules".to_string(),
            ],
            max_file_size: 4 * 1024 * 1024, // 4MB
            max_patterns: 1000,
            lazy_subdirs: true,
        }
    }
}

/// AGENTS.md Discovery System
pub struct AgentsMdDiscovery {
    config: DiscoveryConfig,
    /// Cache: working_dir → discovered entries
    cache: Arc<Mutex<HashMap<PathBuf, Vec<AgentsMdEntry>>>>,
}

impl AgentsMdDiscovery {
    pub fn new(config: DiscoveryConfig) -> Self {
        Self {
            config,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(DiscoveryConfig::default())
    }

    /// Discover all AGENTS.md files for a given working directory
    pub fn discover(&self, working_dir: &Path) -> Vec<AgentsMdEntry> {
        // Check cache
        {
            let cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(working_dir) {
                return cached.clone();
            }
        }

        let mut entries = Vec::new();

        // 1. Organization-level (scope 0)
        for org_path in &self.config.org_paths {
            for name in &["AGENTS.md"] {
                if let Some(entry) = self.read_entry(&org_path.join(name), 0, false) {
                    entries.push(entry);
                }
            }
        }

        // 2. User global (scope 1)
        for user_path in &self.config.user_paths {
            for name in &["AGENTS.md", "rules"] {
                let target = user_path.join(name);
                if target.is_dir() {
                    // Load all .md files in rules directory
                    if let Ok(rd) = std::fs::read_dir(&target) {
                        for entry in rd.flatten() {
                            let path = entry.path();
                            if path.extension().map(|e| e == "md").unwrap_or(false) {
                                if let Some(e) = self.read_entry(&path, 1, false) {
                                    entries.push(e);
                                }
                            }
                        }
                    }
                } else if let Some(e) = self.read_entry(&target, 1, false) {
                    entries.push(e);
                }
            }
        }

        // 3. Project root (scope 2)
        for name in &self.config.project_names {
            if let Some(entry) = self.read_entry(&working_dir.join(name), 2, true) {
                entries.push(entry);
            }
        }

        // 4. Project-level user config (scope 4)
        if let Some(user_base) = dirs::home_dir() {
            let project_hash = self.hash_path(working_dir);
            let user_project = user_base
                .join(".neotrix/projects")
                .join(&project_hash)
                .join("AGENTS.md");
            if let Some(entry) = self.read_entry(&user_project, 4, false) {
                entries.push(entry);
            }
        }

        // 5. Subdirectory AGENTS.md (lazy — only if already known)
        if !self.config.lazy_subdirs {
            self.discover_subdirs(working_dir, &mut entries);
        }

        // Cache results
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(working_dir.to_path_buf(), entries.clone());
        }

        entries
    }

    /// Discover subdirectory AGENTS.md files (on-demand)
    pub fn discover_subdir(&self, subdir: &Path) -> Vec<AgentsMdEntry> {
        let mut entries = Vec::new();
        for name in &["AGENTS.md", ".neotrix/AGENTS.md"] {
            if let Some(entry) = self.read_entry(&subdir.join(name), 3, true) {
                entries.push(entry);
            }
        }
        entries
    }

    /// Inject discovered entries as system-reminder (Claude pattern)
    ///
    /// Key insight: NOT injected into system prompt. This preserves
    /// the shared prompt cache (all users share cached system prefix).
    pub fn inject_as_reminder(entries: &[AgentsMdEntry]) -> String {
        if entries.is_empty() {
            return String::new();
        }

        let content: String = entries
            .iter()
            .map(|e| {
                format!(
                    "Contents of {}:\n{}",
                    e.path.display(),
                    e.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        format!(
            "<system-reminder>\nagentsMd\n{}\ncurrentDate\n</system-reminder>",
            content
        )
    }

    /// Clear cache for a working directory
    pub fn clear_cache(&self, working_dir: &Path) {
        let mut cache = self.cache.lock().unwrap();
        cache.remove(working_dir);
    }

    /// Clear entire cache
    pub fn clear_all_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    // --- Private helpers ---

    fn read_entry(&self, path: &Path, scope: u8, shared: bool) -> Option<AgentsMdEntry> {
        let metadata = std::fs::metadata(path).ok()?;

        // Check file size
        if metadata.len() > self.config.max_file_size {
            return None;
        }

        // Check if it's a directory (rules directory)
        if metadata.is_dir() {
            return None;
        }

        let content = std::fs::read_to_string(path).ok()?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| {
                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                Some(datetime.to_rfc3339())
            })
            .unwrap_or_default();

        Some(AgentsMdEntry {
            path: path.to_path_buf(),
            scope,
            content,
            size: metadata.len(),
            modified,
            shared,
        })
    }

    fn discover_subdirs(&self, dir: &Path, entries: &mut Vec<AgentsMdEntry>) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for entry in rd.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    // Skip hidden dirs, node_modules, target, etc.
                    if name.starts_with('.')
                        || name == "node_modules"
                        || name == "target"
                        || name == "vendor"
                    {
                        continue;
                    }
                    if let Some(e) = self.read_entry(&path.join("AGENTS.md"), 3, true) {
                        entries.push(e);
                    }
                }
            }
        }
    }

    /// Simple path hash for project-scoped user config
    fn hash_path(&self, path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_discovery_config_defaults() {
        let config = DiscoveryConfig::default();
        assert!(!config.org_paths.is_empty());
        assert!(!config.user_paths.is_empty());
        assert_eq!(config.max_file_size, 4 * 1024 * 1024);
    }

    #[test]
    fn test_inject_as_reminder_empty() {
        let result = AgentsMdDiscovery::inject_as_reminder(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_inject_as_reminder_with_entries() {
        let entries = vec![AgentsMdEntry {
            path: PathBuf::from("/test/AGENTS.md"),
            scope: 2,
            content: "# Test Rules\n- Rule 1".to_string(),
            size: 20,
            modified: "2026-01-01T00:00:00Z".to_string(),
            shared: true,
        }];

        let result = AgentsMdDiscovery::inject_as_reminder(&entries);
        assert!(result.contains("<system-reminder>"));
        assert!(result.contains("agentsMd"));
        assert!(result.contains("# Test Rules"));
        assert!(result.contains("</system-reminder>"));
    }

    #[test]
    fn test_scope_ordering() {
        let entries = vec![
            AgentsMdEntry { scope: 2, ..Default::default() },
            AgentsMdEntry { scope: 0, ..Default::default() },
            AgentsMdEntry { scope: 1, ..Default::default() },
        ];

        let mut sorted = entries.clone();
        sorted.sort_by_key(|e| e.scope);

        assert_eq!(sorted[0].scope, 0); // org first
        assert_eq!(sorted[1].scope, 1); // user second
        assert_eq!(sorted[2].scope, 2); // project third
    }
}
