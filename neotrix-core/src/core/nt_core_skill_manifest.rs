#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════
// Error
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub enum SkillError {
    NotFound(String),
    AlreadyRegistered(String),
    InvalidManifest(String),
    Io(std::io::Error),
    Serialization(String),
    VersionConflict(String, String),
}

impl fmt::Display for SkillError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(n) => write!(f, "skill not found: {n}"),
            Self::AlreadyRegistered(n) => write!(f, "skill already registered: {n}"),
            Self::InvalidManifest(m) => write!(f, "invalid manifest: {m}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Serialization(m) => write!(f, "serialization error: {m}"),
            Self::VersionConflict(n, v) => write!(f, "version conflict for {n}: {v}"),
        }
    }
}

impl std::error::Error for SkillError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SkillError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for SkillError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════
// Platform
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    ClaudeCode,
    OpenClaw,
    CodexCLI,
    Cursor,
    OpenCode,
    Generic,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::OpenClaw => "open-claw",
            Self::CodexCLI => "codex-cli",
            Self::Cursor => "cursor",
            Self::OpenCode => "opencode",
            Self::Generic => "generic",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('-', "").replace('_', "").as_str() {
            "claudecode" => Some(Self::ClaudeCode),
            "openclaw" => Some(Self::OpenClaw),
            "codexcli" => Some(Self::CodexCLI),
            "cursor" => Some(Self::Cursor),
            "opencode" => Some(Self::OpenCode),
            "generic" => Some(Self::Generic),
            _ => None,
        }
    }

    pub fn all() -> &'static [Platform; 6] {
        use Platform::*;
        &[ClaudeCode, OpenClaw, CodexCLI, Cursor, OpenCode, Generic]
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════
// SkillDependency
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDependency {
    pub name: String,
    /// Semver version requirement (e.g. "^1.0.0", ">=2.3", "~0.4.5")
    pub version_req: String,
}

impl SkillDependency {
    pub fn new(name: impl Into<String>, version_req: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version_req: version_req.into(),
        }
    }

    /// Basic semver check: returns true if `actual` satisfies `version_req`.
    /// Supports: exact ("1.0.0"), caret ("^1.0.0"), tilde ("~0.4.5"),
    /// and prefix ("1." / ">=2.3" / "<3").
    pub fn satisfies(&self, actual: &str) -> bool {
        let req = self.version_req.trim();
        let actual = actual.trim();

        // Exact match
        if req == actual {
            return true;
        }

        // Caret: ^X.Y.Z means compatible with X.Y.Z
        if let Some(base) = req.strip_prefix('^') {
            return Self::is_compatible(base, actual, true);
        }

        // Tilde: ~X.Y.Z means approximately equivalent
        if let Some(base) = req.strip_prefix('~') {
            return Self::is_compatible(base, actual, false);
        }

        // Prefix match: "1." matches "1.0.0", "1.2.3", etc.
        if req.ends_with('.') {
            return actual.starts_with(req);
        }

        // Prefix match without dot: "1" matches "1.x.x"
        if req.chars().all(|c| c.is_ascii_digit()) {
            return actual.starts_with(req) && (actual.len() == req.len() || actual.as_bytes().get(req.len()) == Some(&b'.'));
        }

        // Range: ">=2.3" / "<3" / ">=1.0 <2.0"
        let parts: Vec<&str> = req.split_whitespace().collect();
        if parts.len() == 2 {
            return self.eval_op(parts[0], actual) && self.eval_op(parts[1], actual);
        }
        if parts.len() == 1 && (parts[0].starts_with('>') || parts[0].starts_with('<') || parts[0].starts_with('=')) {
            return self.eval_op(parts[0], actual);
        }

        // Check if req looks like a bare semver (e.g. "1.2.3") — exact match only
        let is_bare_semver = |s: &str| -> bool {
            let parts: Vec<&str> = s.split('.').collect();
            parts.len() == 3 && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
        };
        if is_bare_semver(req) {
            return false;
        }

        // Fallback: attempt caret semantics
        Self::is_compatible(req, actual, true)
    }

    fn eval_op(&self, op: &str, actual: &str) -> bool {
        if let Some(v) = op.strip_prefix(">=") {
            Self::compare_versions(actual, v) >= 0
        } else if let Some(v) = op.strip_prefix(">") {
            Self::compare_versions(actual, v) > 0
        } else if let Some(v) = op.strip_prefix("<=") {
            Self::compare_versions(actual, v) <= 0
        } else if let Some(v) = op.strip_prefix('<') {
            Self::compare_versions(actual, v) < 0
        } else if let Some(v) = op.strip_prefix("=") {
            Self::compare_versions(actual, v) == 0
        } else {
            false
        }
    }

    fn is_compatible(base: &str, actual: &str, caret: bool) -> bool {
        let base_parts: Vec<u64> = base.split('.').filter_map(|p| p.parse().ok()).collect();
        let actual_parts: Vec<u64> = actual.split('.').filter_map(|p| p.parse().ok()).collect();
        if base_parts.is_empty() || actual_parts.is_empty() {
            return false;
        }
        if caret {
            // Caret: ^1.2.3 means >=1.2.3, <2.0.0
            // ^0.2.3 means >=0.2.3, <0.3.0
            // ^0.0.3 means >=0.0.3, <0.0.4
            let major = base_parts[0];
            if major != 0 {
                if actual_parts[0] == major {
                    return Self::compare_parts(&actual_parts, &base_parts) >= 0;
                }
                false
            } else if base_parts.len() > 1 {
                let minor = base_parts[1];
                if base_parts.len() > 2 {
                    // ^0.0.3
                    if actual_parts.len() > 1 && actual_parts[0] == 0 && actual_parts[1] == minor {
                        return actual_parts.len() > 2 && actual_parts[2] >= base_parts[2];
                    }
                    false
                } else {
                    // ^0.2
                    if actual_parts[0] == 0 && actual_parts.len() > 1 {
                        actual_parts[1] == minor
                    } else {
                        false
                    }
                }
            } else {
                false
            }
        } else {
            // Tilde: ~1.2.3 means >=1.2.3, <1.3.0
            let len = base_parts.len().min(actual_parts.len());
            for i in 0..len - 1 {
                if actual_parts[i] != base_parts[i] {
                    return false;
                }
            }
            actual_parts[len - 1] >= base_parts[len - 1]
        }
    }

    fn compare_versions(a: &str, b: &str) -> i32 {
        let a_parts: Vec<i32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
        let b_parts: Vec<i32> = b.split('.').filter_map(|p| p.parse().ok()).collect();
        let max_len = a_parts.len().max(b_parts.len());
        for i in 0..max_len {
            let av = a_parts.get(i).copied().unwrap_or(0);
            let bv = b_parts.get(i).copied().unwrap_or(0);
            if av != bv {
                return if av > bv { 1 } else { -1 };
            }
        }
        0
    }

    fn compare_parts(a: &[u64], b: &[u64]) -> i32 {
        let max_len = a.len().max(b.len());
        for i in 0..max_len {
            let av = a.get(i).copied().unwrap_or(0);
            let bv = b.get(i).copied().unwrap_or(0);
            if av != bv {
                return if av > bv { 1 } else { -1 };
            }
        }
        0
    }
}

// ═══════════════════════════════════════════════════════════════════
// SkillCompatibility
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCompatibility {
    /// Minimum agent platform version required
    pub min_agent_version: String,
    /// Tools that must be available in the runtime
    pub required_tools: Vec<String>,
}

impl SkillCompatibility {
    pub fn new(min_agent_version: impl Into<String>) -> Self {
        Self {
            min_agent_version: min_agent_version.into(),
            required_tools: Vec::new(),
        }
    }

    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.required_tools = tools;
        self
    }
}

impl Default for SkillCompatibility {
    fn default() -> Self {
        Self {
            min_agent_version: "0.1.0".to_string(),
            required_tools: Vec::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// SkillManifest
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub tags: Vec<String>,
    pub dependencies: Vec<SkillDependency>,
    pub platforms: Vec<Platform>,
    /// Path to SKILL.md or main script
    pub entry: String,
    /// Additional resource files
    pub resources: Vec<String>,
    pub compatibility: SkillCompatibility,
    pub metadata: HashMap<String, String>,
}

impl SkillManifest {
    pub fn new(name: impl Into<String>, version: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: description.into(),
            author: String::new(),
            license: String::from("MIT"),
            tags: Vec::new(),
            dependencies: Vec::new(),
            platforms: vec![Platform::Generic],
            entry: String::from("SKILL.md"),
            resources: Vec::new(),
            compatibility: SkillCompatibility::default(),
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), SkillError> {
        if self.name.is_empty() {
            return Err(SkillError::InvalidManifest("name is required".into()));
        }
        if self.version.is_empty() {
            return Err(SkillError::InvalidManifest("version is required".into()));
        }
        if self.description.is_empty() {
            return Err(SkillError::InvalidManifest("description is required".into()));
        }
        if self.entry.is_empty() {
            return Err(SkillError::InvalidManifest("entry is required".into()));
        }
        Ok(())
    }

    /// Parse from a JSON string
    pub fn from_json(json: &str) -> Result<Self, SkillError> {
        let manifest: Self = serde_json::from_str(json)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, SkillError> {
        serde_json::to_string_pretty(self).map_err(|e| SkillError::Serialization(e.to_string()))
    }

    /// Check whether this manifest supports a given platform
    pub fn supports_platform(&self, platform: &Platform) -> bool {
        self.platforms.contains(platform) || self.platforms.contains(&Platform::Generic)
    }

    /// Check whether all dependency constraints are satisfied by the given versions
    pub fn check_dependencies(&self, available: &HashMap<String, String>) -> Result<(), SkillError> {
        for dep in &self.dependencies {
            let actual = available.get(&dep.name).ok_or_else(|| {
                SkillError::NotFound(format!("dependency '{}' not available", dep.name))
            })?;
            if !dep.satisfies(actual) {
                return Err(SkillError::VersionConflict(
                    dep.name.clone(),
                    format!("requires {} but {} is available", dep.version_req, actual),
                ));
            }
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════
// SkillEntry
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEntry {
    pub manifest: SkillManifest,
    /// Local path where the skill is installed
    pub install_path: Option<PathBuf>,
    /// URL from which this skill was sourced (if remote)
    pub source_url: Option<String>,
    /// Whether this skill was published locally
    pub published: bool,
}

impl SkillEntry {
    pub fn new(manifest: SkillManifest) -> Self {
        Self {
            manifest,
            install_path: None,
            source_url: None,
            published: false,
        }
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.install_path = Some(path);
        self
    }

    pub fn with_source(mut self, url: String) -> Self {
        self.source_url = Some(url);
        self
    }
}

// ═══════════════════════════════════════════════════════════════════
// SkillRegistry
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct SkillRegistry {
    skills: HashMap<String, SkillEntry>,
    pub install_dir: PathBuf,
    pub published: Vec<String>,
}

impl SkillRegistry {
    pub fn new(install_dir: PathBuf) -> Self {
        Self {
            skills: HashMap::new(),
            install_dir,
            published: Vec::new(),
        }
    }

    /// Register a skill in the registry
    pub fn register(&mut self, entry: SkillEntry) -> Result<(), SkillError> {
        let name = entry.manifest.name.clone();
        if self.skills.contains_key(&name) {
            return Err(SkillError::AlreadyRegistered(name));
        }
        entry.manifest.validate()?;
        self.skills.insert(name, entry);
        Ok(())
    }

    /// Unregister a skill
    pub fn unregister(&mut self, name: &str) -> Option<SkillEntry> {
        let entry = self.skills.remove(name);
        if entry.is_some() {
            self.published.retain(|n| n != name);
        }
        entry
    }

    /// Find a skill by exact name
    pub fn find(&self, name: &str) -> Option<&SkillEntry> {
        self.skills.get(name)
    }

    /// Search skills by query (case-insensitive name/description/tag match)
    pub fn search(&self, query: &str) -> Vec<&SkillEntry> {
        let q = query.to_lowercase();
        self.skills
            .values()
            .filter(|e| {
                let m = &e.manifest;
                m.name.to_lowercase().contains(&q)
                    || m.description.to_lowercase().contains(&q)
                    || m.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect()
    }

    /// Search skills by tag
    pub fn search_by_tag(&self, tag: &str) -> Vec<&SkillEntry> {
        let tag_lower = tag.to_lowercase();
        self.skills
            .values()
            .filter(|e| e.manifest.tags.iter().any(|t| t.to_lowercase() == tag_lower))
            .collect()
    }

    /// Search skills by platform
    pub fn search_by_platform(&self, platform: &Platform) -> Vec<&SkillEntry> {
        self.skills
            .values()
            .filter(|e| e.manifest.supports_platform(platform))
            .collect()
    }

    /// List all registered skills
    pub fn list(&self) -> Vec<&SkillEntry> {
        self.skills.values().collect()
    }

    /// Number of registered skills
    pub fn count(&self) -> usize {
        self.skills.len()
    }

    /// Get all skill names
    pub fn names(&self) -> Vec<String> {
        self.skills.keys().cloned().collect()
    }

    /// Install a skill: register and set install path
    pub fn install(&mut self, manifest: SkillManifest, path: PathBuf) -> Result<(), SkillError> {
        let mut entry = SkillEntry::new(manifest);
        entry.install_path = Some(path);
        self.register(entry)
    }

    /// Install a skill from a remote URL (stub — actual download is external)
    pub fn install_from_url(&mut self, url: &str, manifest_json: Option<&str>) -> Result<SkillEntry, SkillError> {
        if let Some(json) = manifest_json {
            let manifest = SkillManifest::from_json(json)?;
            let mut entry = SkillEntry::new(manifest);
            entry.source_url = Some(url.to_string());
            let name = entry.manifest.name.clone();
            if self.skills.contains_key(&name) {
                return Err(SkillError::AlreadyRegistered(name));
            }
            self.skills.insert(name.clone(), entry.clone());
            Ok(entry)
        } else {
            Err(SkillError::InvalidManifest("manifest JSON required for remote install".into()))
        }
    }

    /// Publish a skill (mark it as locally published)
    pub fn publish(&mut self, name: &str) -> Result<(), SkillError> {
        if let Some(entry) = self.skills.get_mut(name) {
            entry.published = true;
            if !self.published.contains(&name.to_string()) {
                self.published.push(name.to_string());
            }
            Ok(())
        } else {
            Err(SkillError::NotFound(name.to_string()))
        }
    }

    /// Export all manifests as a JSON array
    pub fn export_manifest(&self) -> Result<String, SkillError> {
        let manifests: Vec<&SkillManifest> = self.skills.values().map(|e| &e.manifest).collect();
        serde_json::to_string_pretty(&manifests).map_err(|e| SkillError::Serialization(e.to_string()))
    }

    /// Generate the `npx skills add` command string for installing a skill.
    /// Follows the pattern: `npx skills add <repo> [--skill <name>]`
    pub fn install_via_npx(&self, repo: &str, skill: Option<&str>) -> String {
        let mut cmd = format!("npx skills add {}", repo);
        if let Some(name) = skill {
            cmd.push_str(&format!(" --skill {}", name));
        }
        cmd
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new(PathBuf::from(".skills"))
    }
}

// ═══════════════════════════════════════════════════════════════════
// SkillMarket
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct SkillMarket {
    /// Remote registry URLs
    pub sources: Vec<String>,
    /// Cached remote manifests
    cache: Vec<SkillManifest>,
    /// Local registry reference
    pub registry: SkillRegistry,
}

impl SkillMarket {
    pub fn new(registry: SkillRegistry) -> Self {
        Self {
            sources: Vec::new(),
            cache: Vec::new(),
            registry,
        }
    }

    /// Add a remote source URL
    pub fn add_source(&mut self, url: impl Into<String>) {
        self.sources.push(url.into());
    }

    /// Discover skills from all sources (stub — would fetch from URLs)
    pub fn discover(&mut self) -> Vec<String> {
        let found = Vec::new();
        for _source in &self.sources {
            // In production, fetch from URL, parse registry index
        }
        found
    }

    /// Refresh the cache from remote sources (stub)
    pub fn refresh_cache(&mut self) -> Result<usize, SkillError> {
        let count = self.cache.len();
        // In production: fetch from each source, parse manifests, merge into cache
        Ok(count)
    }

    /// Search remote cached entries
    pub fn search_remote(&self, query: &str) -> Vec<&SkillManifest> {
        let q = query.to_lowercase();
        self.cache
            .iter()
            .filter(|m| {
                m.name.to_lowercase().contains(&q)
                    || m.description.to_lowercase().contains(&q)
                    || m.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect()
    }

    /// Import manifests from a ClawHub-style registry (simulated)
    pub fn import_from_clawhub(&mut self, manifests: Vec<SkillManifest>) {
        for m in manifests {
            if !self.cache.iter().any(|c| c.name == m.name) {
                self.cache.push(m);
            }
        }
    }

    /// Import manifests from a skills.sh-style registry (simulated)
    pub fn import_from_skills_sh(&mut self, manifests: Vec<SkillManifest>) {
        for m in manifests {
            if !self.cache.iter().any(|c| c.name == m.name) {
                self.cache.push(m);
            }
        }
    }

    /// Number of cached remote skills
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Number of remote sources
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> SkillManifest {
        SkillManifest {
            name: "test-skill".into(),
            version: "1.2.3".into(),
            description: "A test skill".into(),
            author: "NeoTrix".into(),
            license: "MIT".into(),
            tags: vec!["test".into(), "rust".into(), "cli".into()],
            dependencies: vec![
                SkillDependency::new("core-utils", "^1.0.0"),
            ],
            platforms: vec![Platform::OpenCode, Platform::ClaudeCode],
            entry: "SKILL.md".into(),
            resources: vec!["install.sh".into(), "config.toml".into()],
            compatibility: SkillCompatibility::new("0.5.0")
                .with_tools(vec!["cargo".into(), "node".into()]),
            metadata: HashMap::from([
                ("category".into(), "developer-tools".into()),
                ("icon".into(), "wrench".into()),
            ]),
        }
    }

    #[test]
    fn test_manifest_create_and_validate() {
        let m = sample_manifest();
        assert!(m.validate().is_ok());

        let empty = SkillManifest::new("", "", "");
        assert!(empty.validate().is_err());

        let no_name = SkillManifest::new("", "1.0.0", "desc");
        assert!(matches!(no_name.validate(), Err(SkillError::InvalidManifest(_))));

        let no_version = SkillManifest::new("foo", "", "desc");
        assert!(matches!(no_version.validate(), Err(SkillError::InvalidManifest(_))));
    }

    #[test]
    fn test_registry_register_find_unregister() {
        let mut reg = SkillRegistry::default();
        let m = sample_manifest();
        let entry = SkillEntry::new(m);
        assert!(reg.register(entry).is_ok());

        assert!(reg.find("test-skill").is_some());
        assert!(reg.find("nonexistent").is_none());

        // Duplicate registration
        let dup = SkillEntry::new(sample_manifest());
        assert!(matches!(reg.register(dup), Err(SkillError::AlreadyRegistered(_))));

        // Unregister
        let removed = reg.unregister("test-skill");
        assert!(removed.is_some());
        assert!(reg.find("test-skill").is_none());
    }

    #[test]
    fn test_registry_search_by_tag() {
        let mut reg = SkillRegistry::default();
        let m1 = SkillManifest {
            tags: vec!["database".into(), "sql".into()],
            ..sample_manifest()
        };
        let mut m2 = sample_manifest();
        m2.name = "other-skill".into();
        m2.tags = vec!["network".into(), "http".into()];

        reg.register(SkillEntry::new(m1)).unwrap();
        reg.register(SkillEntry::new(m2)).unwrap();

        let db_results = reg.search_by_tag("database");
        assert_eq!(db_results.len(), 1);
        assert_eq!(db_results[0].manifest.name, "test-skill");

        let net_results = reg.search_by_tag("network");
        assert_eq!(net_results.len(), 1);
        assert_eq!(net_results[0].manifest.name, "other-skill");

        let no_results = reg.search_by_tag("nonexistent");
        assert!(no_results.is_empty());
    }

    #[test]
    fn test_registry_search_by_platform() {
        let mut reg = SkillRegistry::default();
        let mut m1 = sample_manifest();
        m1.platforms = vec![Platform::OpenCode, Platform::ClaudeCode];
        let mut m2 = sample_manifest();
        m2.name = "cursor-only".into();
        m2.platforms = vec![Platform::Cursor];

        reg.register(SkillEntry::new(m1)).unwrap();
        reg.register(SkillEntry::new(m2)).unwrap();

        let code_results = reg.search_by_platform(&Platform::OpenCode);
        assert_eq!(code_results.len(), 1);
        assert_eq!(code_results[0].manifest.name, "test-skill");

        let cursor_results = reg.search_by_platform(&Platform::Cursor);
        assert_eq!(cursor_results.len(), 1);
        assert_eq!(cursor_results[0].manifest.name, "cursor-only");
    }

    #[test]
    fn test_install_via_npx_command() {
        let reg = SkillRegistry::default();

        let cmd1 = reg.install_via_npx("github.com/user/skills", None);
        assert_eq!(cmd1, "npx skills add github.com/user/skills");

        let cmd2 = reg.install_via_npx("github.com/user/skills", Some("my-skill"));
        assert_eq!(cmd2, "npx skills add github.com/user/skills --skill my-skill");
    }

    #[test]
    fn test_manifest_serialization() {
        let m = sample_manifest();
        let json = m.to_json().unwrap();
        assert!(json.contains("\"name\": \"test-skill\""));
        assert!(json.contains("\"version\": \"1.2.3\""));
        assert!(json.contains("\"OpenCode\""));
        assert!(json.contains("\"ClaudeCode\""));

        let parsed = SkillManifest::from_json(&json).unwrap();
        assert_eq!(parsed.name, m.name);
        assert_eq!(parsed.version, m.version);
        assert_eq!(parsed.platforms, m.platforms);
        assert_eq!(parsed.tags, m.tags);
        assert_eq!(parsed.dependencies.len(), 1);
        assert_eq!(parsed.metadata.get("category").unwrap(), "developer-tools");
    }

    #[test]
    fn test_dependency_resolution() {
        // Caret
        let dep = SkillDependency::new("foo", "^1.0.0");
        assert!(dep.satisfies("1.0.0"));
        assert!(dep.satisfies("1.2.3"));
        assert!(dep.satisfies("1.9.99"));
        assert!(!dep.satisfies("2.0.0"));
        assert!(!dep.satisfies("0.9.9"));

        // Tilde
        let dep2 = SkillDependency::new("foo", "~1.2.0");
        assert!(dep2.satisfies("1.2.0"));
        assert!(dep2.satisfies("1.2.5"));
        assert!(!dep2.satisfies("1.3.0"));
        assert!(!dep2.satisfies("0.9.9"));

        // Exact
        let dep3 = SkillDependency::new("foo", "2.0.0");
        assert!(dep3.satisfies("2.0.0"));
        assert!(!dep3.satisfies("2.0.1"));

        // Prefix
        let dep4 = SkillDependency::new("foo", "1.");
        assert!(dep4.satisfies("1.0.0"));
        assert!(dep4.satisfies("1.2.3"));
        assert!(!dep4.satisfies("2.0.0"));

        // Range
        let dep5 = SkillDependency::new("foo", ">=1.0 <2.0");
        assert!(dep5.satisfies("1.5.0"));
        assert!(!dep5.satisfies("2.0.0"));
        assert!(!dep5.satisfies("0.9.0"));

        // Complex: dependencies check
        let manifest = sample_manifest();
        let mut available = HashMap::new();
        available.insert("core-utils".to_string(), "1.5.0".to_string());
        assert!(manifest.check_dependencies(&available).is_ok());

        available.insert("core-utils".to_string(), "2.0.0".to_string());
        assert!(manifest.check_dependencies(&available).is_err());

        available.clear();
        assert!(manifest.check_dependencies(&available).is_err());
    }

    #[test]
    fn test_platform_display_and_parse() {
        assert_eq!(Platform::ClaudeCode.as_str(), "claude-code");
        assert_eq!(Platform::OpenCode.as_str(), "opencode");
        assert_eq!(Platform::Generic.as_str(), "generic");

        assert_eq!(Platform::from_str("claude-code"), Some(Platform::ClaudeCode));
        assert_eq!(Platform::from_str("opencode"), Some(Platform::OpenCode));
        assert_eq!(Platform::from_str("OPENCODE"), Some(Platform::OpenCode));
        assert_eq!(Platform::from_str("unknown"), None);

        assert_eq!(Platform::all().len(), 6);
    }

    #[test]
    fn test_marketplace_import() {
        let reg = SkillRegistry::default();
        let mut market = SkillMarket::new(reg);

        let m1 = sample_manifest();
        let mut m2 = sample_manifest();
        m2.name = "remote-skill".into();

        market.import_from_clawhub(vec![m1, m2]);
        assert_eq!(market.cache_size(), 2);

        // Dedup
        let m3 = sample_manifest();
        market.import_from_clawhub(vec![m3]);
        assert_eq!(market.cache_size(), 2);

        // search_remote
        let results = market.search_remote("remote");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "remote-skill");
    }

    #[test]
    fn test_publish_and_export() {
        let mut reg = SkillRegistry::default();
        reg.register(SkillEntry::new(sample_manifest())).unwrap();

        reg.publish("test-skill").unwrap();
        assert!(reg.published.contains(&"test-skill".to_string()));

        let json = reg.export_manifest().unwrap();
        assert!(json.contains("test-skill"));

        // Publishing nonexistent
        assert!(matches!(reg.publish("nope"), Err(SkillError::NotFound(_))));
    }

    #[test]
    fn test_install_from_url() {
        let mut reg = SkillRegistry::default();
        let manifest_json = sample_manifest().to_json().unwrap();

        let entry = reg.install_from_url("https://registry.example.com/skills/test-skill", Some(&manifest_json));
        assert!(entry.is_ok());
        assert_eq!(entry.unwrap().source_url.unwrap(), "https://registry.example.com/skills/test-skill");

        // Duplicate remote install
        let dup = reg.install_from_url("https://example.com/dup", Some(&manifest_json));
        assert!(matches!(dup, Err(SkillError::AlreadyRegistered(_))));
    }

    #[test]
    fn test_empty_registry_count() {
        let reg = SkillRegistry::default();
        assert_eq!(reg.count(), 0);
        assert!(reg.list().is_empty());
        assert!(reg.names().is_empty());
    }

    #[test]
    fn test_compatibility_default() {
        let compat = SkillCompatibility::default();
        assert_eq!(compat.min_agent_version, "0.1.0");
        assert!(compat.required_tools.is_empty());
    }
}
