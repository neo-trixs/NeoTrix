use std::collections::HashMap;
use std::time::Instant;

use super::skill_manifest::SkillManifest;

/// Lightweight metadata only — no full source code.
/// This is what gets searched and indexed.
#[derive(Debug, Clone)]
pub struct DisclosureManifest {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub trigger_patterns: Vec<String>,
    pub estimated_cost: usize,
    pub dependencies: Vec<String>,
}

impl DisclosureManifest {
    /// Build from a SkillManifest + synthetic cost from trigger count / description length.
    pub fn from_skill_manifest(m: &SkillManifest) -> Self {
        let estimated_cost = m.description.len()
            + m.trigger_words.iter().map(|t| t.len()).sum::<usize>()
            + m.tags.iter().map(|t| t.len()).sum::<usize>()
            + m.dependencies.iter().map(|d| d.len()).sum::<usize>();
        Self {
            skill_id: m.name.clone(),
            name: m.name.clone(),
            description: m.description.clone(),
            version: m.version.clone(),
            trigger_patterns: m.trigger_words.clone(),
            estimated_cost,
            dependencies: m.dependencies.clone(),
        }
    }
}

/// Full skill code — only loaded when triggered.
#[derive(Debug, Clone)]
pub struct FullSkill {
    pub skill_id: String,
    pub source_code: String,
    pub manifest_overrides: HashMap<String, String>,
    pub last_accessed: Instant,
    pub access_count: u64,
}

impl FullSkill {
    pub fn new(skill_id: String) -> Self {
        Self {
            skill_id,
            source_code: String::new(),
            manifest_overrides: HashMap::new(),
            last_accessed: Instant::now(),
            access_count: 0,
        }
    }

    /// Record an access touch — bump last_accessed and increment counter.
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }
}

/// Progressive Disclosure Layer.
///
/// Sits between skill search and skill execution:
///   search_metadata()  → returns lightweight manifests (zero code)
///   load_full()        → on trigger-match, lazily loads full code
///
/// Defaults: max_cache_size = 100, min_trigger_similarity = 0.5
#[derive(Debug)]
pub struct ProgressiveDisclosureLayer {
    /// Metadata-only index — always loaded, cheap to search.
    pub manifest_index: Vec<DisclosureManifest>,
    /// Full-code cache — only populated on trigger.
    pub full_cache: HashMap<String, FullSkill>,
    max_cache_size: usize,
    min_trigger_similarity: f64,
}

impl Default for ProgressiveDisclosureLayer {
    fn default() -> Self {
        Self {
            manifest_index: Vec::new(),
            full_cache: HashMap::new(),
            max_cache_size: 100,
            min_trigger_similarity: 0.5,
        }
    }
}

impl ProgressiveDisclosureLayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_cache_size(mut self, n: usize) -> Self {
        self.max_cache_size = n;
        self
    }

    pub fn with_min_trigger_similarity(mut self, s: f64) -> Self {
        self.min_trigger_similarity = s;
        self
    }

    // ── Manifest Management ──

    /// Register a metadata manifest into the search index.
    /// If a manifest with the same `skill_id` already exists, it is replaced.
    pub fn register_manifest(&mut self, manifest: DisclosureManifest) {
        if let Some(pos) = self
            .manifest_index
            .iter()
            .position(|m| m.skill_id == manifest.skill_id)
        {
            self.manifest_index[pos] = manifest;
        } else {
            self.manifest_index.push(manifest);
        }
    }

    /// Register from an existing SkillManifest (convenience).
    pub fn register_skill_manifest(&mut self, m: &SkillManifest) {
        self.register_manifest(DisclosureManifest::from_skill_manifest(m));
    }

    /// Remove a manifest from the index + full cache.
    pub fn unregister(&mut self, skill_id: &str) {
        self.manifest_index.retain(|m| m.skill_id != skill_id);
        self.full_cache.remove(skill_id);
    }

    /// Number of manifests in the index.
    pub fn manifest_count(&self) -> usize {
        self.manifest_index.len()
    }

    // ── Metadata Search (no code access) ──

    /// Search metadata index by keyword overlap.
    /// Returns manifests sorted by descending score that meet `min_trigger_similarity`.
    pub fn search_metadata(&self, query: &str, top_k: usize) -> Vec<&DisclosureManifest> {
        if query.is_empty() || top_k == 0 {
            return Vec::new();
        }

        let query_tokens: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .filter(|t| !t.is_empty())
            .map(|t| t.to_string())
            .collect();

        if query_tokens.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(&DisclosureManifest, f64)> = self
            .manifest_index
            .iter()
            .map(|m| {
                let score = compute_keyword_overlap(&query_tokens, m);
                (m, score)
            })
            .filter(|(_, s)| *s >= self.min_trigger_similarity)
            .collect();

        // Sort descending by score
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter().take(top_k).map(|(m, _)| m).collect()
    }

    // ── Full-Code Loading ──

    /// Lazily load full skill code on trigger match.
    /// Returns `None` if skill_id not found in index or not yet loaded.
    ///
    /// In production, this would read from disk / SKILL.md when the cache misses.
    /// For now this delegates to `load_or_stub`.
    pub fn load_full(&mut self, skill_id: &str) -> Option<&FullSkill> {
        self.load_or_stub(skill_id)
    }

    /// Lazily load or create a stub FullSkill entry.
    /// Useful in tests or when full skill data is available elsewhere.
    pub fn load_or_stub(&mut self, skill_id: &str) -> Option<&FullSkill> {
        // Ensure manifest exists
        if !self.manifest_index.iter().any(|m| m.skill_id == skill_id) {
            return None;
        }

        // If not cached, insert a stub entry
        if !self.full_cache.contains_key(skill_id) {
            self.full_cache
                .insert(skill_id.to_string(), FullSkill::new(skill_id.to_string()));
        }

        let entry = self.full_cache.get_mut(skill_id)?;
        entry.touch();
        // We need the borrow to live long enough; this is safe because we hold &mut self.
        // The returned reference borrows from the HashMap which lives as long as self.
        Some(&*entry)
    }

    // ── Confidence / Disclose Decision ──

    /// Compute confidence that `query` matches this manifest's triggers.
    /// Score = matched_tokens / total_unique_tokens across name + description + trigger_patterns.
    pub fn should_disclose(query: &str, manifest: &DisclosureManifest) -> f64 {
        if query.is_empty() {
            return 0.0;
        }

        let query_tokens: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .filter(|t| !t.is_empty())
            .map(|t| t.to_string())
            .collect();

        if query_tokens.is_empty() {
            return 0.0;
        }

        compute_keyword_overlap(&query_tokens, manifest)
    }

    // ── Cache Eviction ──

    /// Evict least-used entries when cache exceeds max_cache_size.
    /// Drops the entry with the lowest access_count; ties broken by oldest last_accessed.
    pub fn evict_least_used(&mut self) -> usize {
        if self.full_cache.len() <= self.max_cache_size {
            return 0;
        }

        let excess = self.full_cache.len() - self.max_cache_size;
        // Find the `excess` least-used entries.
        let mut entries: Vec<(String, u64, Instant)> = self
            .full_cache
            .iter()
            .map(|(id, skill)| (id.clone(), skill.access_count, skill.last_accessed))
            .collect();

        // Sort by access_count asc, then last_accessed asc (oldest first)
        entries.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.cmp(&b.2)));

        for (id, _, _) in entries.iter().take(excess) {
            self.full_cache.remove(id);
        }

        excess
    }

    /// Force eviction to a target size (e.g., after bulk load).
    pub fn shrink_to(&mut self, target: usize) -> usize {
        if self.full_cache.len() <= target {
            return 0;
        }
        let old = self.full_cache.len();
        while self.full_cache.len() > target {
            self.evict_least_used();
        }
        old - self.full_cache.len()
    }

    /// Current cache size.
    pub fn cache_size(&self) -> usize {
        self.full_cache.len()
    }

    /// Get a reference to the full cache entry (for testing / inspection).
    pub fn get_cached(&self, skill_id: &str) -> Option<&FullSkill> {
        self.full_cache.get(skill_id)
    }

    // ── Dependency Resolution ──

    /// Return all dependency skill IDs for a given manifest (direct only).
    pub fn dependencies_of(&self, skill_id: &str) -> Vec<String> {
        self.manifest_index
            .iter()
            .find(|m| m.skill_id == skill_id)
            .map(|m| m.dependencies.clone())
            .unwrap_or_default()
    }

    /// Recursively resolve all transitive dependencies for a skill.
    /// Returns a list ordered breadth-first, with the skill itself first.
    pub fn resolve_transitive_deps(&self, skill_id: &str) -> Vec<String> {
        let mut resolved = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(skill_id.to_string());

        while let Some(current) = queue.pop_front() {
            if !seen.insert(current.clone()) {
                continue;
            }
            resolved.push(current.clone());
            for dep in self.dependencies_of(&current) {
                if !seen.contains(&dep) {
                    queue.push_back(dep);
                }
            }
        }

        resolved
    }
}

// ── Keyword Overlap Scoring ──

/// Compute keyword overlap score between query tokens and a manifest.
///
/// Score = (number of query tokens that match ANY field) / (total unique query tokens).
/// Match fields: name, description, trigger_patterns.
/// Tokens are lowercased for comparison.
fn compute_keyword_overlap(query_tokens: &[String], manifest: &DisclosureManifest) -> f64 {
    let haystack: Vec<String> = manifest
        .trigger_patterns
        .iter()
        .chain(std::iter::once(&manifest.name))
        .chain(std::iter::once(&manifest.description))
        .flat_map(|s| {
            s.to_lowercase()
                .split_whitespace()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
        })
        .collect();

    if haystack.is_empty() {
        return 0.0;
    }

    let matched = query_tokens
        .iter()
        .filter(|qt| haystack.iter().any(|h| h.contains(&**qt) || qt.contains(h)))
        .count();

    matched as f64 / query_tokens.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest(id: &str, name: &str, triggers: &[&str], desc: &str) -> DisclosureManifest {
        DisclosureManifest {
            skill_id: id.to_string(),
            name: name.to_string(),
            description: desc.to_string(),
            version: "1.0.0".to_string(),
            trigger_patterns: triggers.iter().map(|s| s.to_string()).collect(),
            estimated_cost: 100,
            dependencies: vec![],
        }
    }

    fn manifest_with_deps(id: &str, deps: &[&str]) -> DisclosureManifest {
        DisclosureManifest {
            skill_id: id.to_string(),
            name: id.to_string(),
            description: String::new(),
            version: "1.0.0".to_string(),
            trigger_patterns: vec![],
            estimated_cost: 0,
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
        }
    }

    // ── Registration & Search ──

    #[test]
    fn test_register_and_search_metadata() {
        let mut layer = ProgressiveDisclosureLayer::new();
        layer.register_manifest(sample_manifest(
            "deploy",
            "Deploy",
            &["deploy", "release"],
            "Deploy releases",
        ));
        layer.register_manifest(sample_manifest(
            "search",
            "Search",
            &["search", "find"],
            "Search capabilities",
        ));
        assert_eq!(layer.manifest_count(), 2);

        let results = layer.search_metadata("deploy the app", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].skill_id, "deploy");
    }

    #[test]
    fn test_search_returns_metadata_only_no_code() {
        let mut layer = ProgressiveDisclosureLayer::new();
        layer.register_manifest(sample_manifest(
            "web",
            "Web",
            &["web", "internet"],
            "Web browsing",
        ));

        let results = layer.search_metadata("web", 10);
        assert_eq!(results.len(), 1);
        // Verify no source_code field exists on DisclosureManifest
        let m = results[0];
        assert_eq!(m.name, "Web");
        assert_eq!(m.description, "Web browsing");
        // Compile-time guarantee: DisclosureManifest has no source_code field
    }

    #[test]
    fn test_search_empty_query() {
        let layer = ProgressiveDisclosureLayer::new();
        assert!(layer.search_metadata("", 10).is_empty());
        assert!(layer.search_metadata("test", 0).is_empty());
    }

    #[test]
    fn test_search_no_match() {
        let mut layer = ProgressiveDisclosureLayer::new();
        layer.register_manifest(sample_manifest(
            "db",
            "Database",
            &["sql", "query"],
            "Database queries",
        ));
        let results = layer.search_metadata("cooking recipes", 10);
        assert!(results.is_empty());
    }

    // ── Score Computation ──

    #[test]
    fn test_score_exact_match() {
        let manifest = sample_manifest("git", "Git", &["git", "commit"], "Git operations");
        let score = ProgressiveDisclosureLayer::should_disclose("git commit", &manifest);
        assert!((score - 1.0).abs() < 1e-6, "expected 1.0, got {}", score);
    }

    #[test]
    fn test_score_partial() {
        let manifest = sample_manifest(
            "docker",
            "Docker",
            &["docker", "container"],
            "Docker container mgmt",
        );
        let score = ProgressiveDisclosureLayer::should_disclose("docker compose", &manifest);
        assert!(
            score > 0.0 && score < 1.0,
            "expected partial, got {}",
            score
        );
    }

    #[test]
    fn test_score_zero() {
        let manifest = sample_manifest(
            "terraform",
            "Terraform",
            &["terraform", "infra"],
            "Infrastructure",
        );
        let score = ProgressiveDisclosureLayer::should_disclose("cooking pasta", &manifest);
        assert!((score - 0.0).abs() < 1e-6, "expected 0.0, got {}", score);
    }

    #[test]
    fn test_score_empty_query() {
        let manifest = sample_manifest("x", "X", &["x"], "desc");
        let score = ProgressiveDisclosureLayer::should_disclose("", &manifest);
        assert!((score - 0.0).abs() < 1e-6);
    }

    // ── Trigger-Pattern Sensitivity ──

    #[test]
    fn test_trigger_pattern_match_strong() {
        let manifest = sample_manifest(
            "build",
            "Build",
            &["build", "compile", "make"],
            "Build system",
        );
        let score = ProgressiveDisclosureLayer::should_disclose("compile the project", &manifest);
        assert!(score > 0.0, "trigger keyword should match");
    }

    #[test]
    fn test_case_insensitive_search() {
        let mut layer = ProgressiveDisclosureLayer::new();
        layer.register_manifest(sample_manifest("deploy", "Deploy", &["DEPLOY"], "Deploy"));
        let results = layer.search_metadata("deploy", 10);
        assert_eq!(results.len(), 1);
    }

    // ── Cache Eviction ──

    #[test]
    fn test_cache_eviction_lru() {
        let mut layer = ProgressiveDisclosureLayer::new().with_max_cache_size(3);

        // Register 3 manifests
        for i in 0..3 {
            let id = format!("skill_{}", i);
            layer.register_manifest(sample_manifest(&id, &id, &[&id], &id));
            // Load/stub each into the cache
            layer.load_or_stub(&id);
        }
        assert_eq!(layer.cache_size(), 3);

        // Load a 4th -> we need to remove one, but currently our load_or_stub doesn't auto-evict.
        // Trigger manual eviction
        let id4 = "skill_4".to_string();
        layer.register_manifest(sample_manifest(&id4, &id4, &[&id4], &id4));
        // Touch skill_0 to make it more recently used
        if let Some(full) = layer.full_cache.get_mut("skill_0") {
            full.access_count = 10;
            full.last_accessed = Instant::now();
        }
        layer.load_or_stub(&id4);

        // Cache is now 4, max is 3
        let evicted = layer.evict_least_used();
        assert_eq!(evicted, 1);
        assert_eq!(layer.cache_size(), 3);

        // skill_0 has high access_count, should survive; the one with lowest should be gone
        assert!(layer.get_cached("skill_0").is_some());
    }

    #[test]
    fn test_eviction_noop_when_under_limit() {
        let mut layer = ProgressiveDisclosureLayer::new().with_max_cache_size(10);
        for i in 0..3 {
            let id = format!("s_{}", i);
            layer.register_manifest(sample_manifest(&id, &id, &[&id], &id));
            layer.load_or_stub(&id);
        }
        assert_eq!(layer.evict_least_used(), 0);
        assert_eq!(layer.cache_size(), 3);
    }

    #[test]
    fn test_shrink_to_target() {
        let mut layer = ProgressiveDisclosureLayer::new().with_max_cache_size(100);
        for i in 0..20 {
            let id = format!("s_{}", i);
            layer.register_manifest(sample_manifest(&id, &id, &[&id], &id));
            layer.load_or_stub(&id);
        }
        assert_eq!(layer.cache_size(), 20);
        let removed = layer.shrink_to(5);
        assert_eq!(removed, 15);
        assert_eq!(layer.cache_size(), 5);
    }

    // ── Dependency Tracking ──

    #[test]
    fn test_dependency_tracking() {
        let mut layer = ProgressiveDisclosureLayer::new();
        layer.register_manifest(manifest_with_deps("core", &[]));
        layer.register_manifest(manifest_with_deps("web", &["core"]));
        layer.register_manifest(manifest_with_deps("crawl", &["web", "core"]));

        assert!(layer.dependencies_of("core").is_empty());
        assert_eq!(layer.dependencies_of("web"), vec!["core"]);
        assert_eq!(layer.dependencies_of("crawl"), vec!["web", "core"]);
    }

    #[test]
    fn test_transitive_dependency_resolution() {
        let mut layer = ProgressiveDisclosureLayer::new();
        layer.register_manifest(manifest_with_deps("root", &["mid"]));
        layer.register_manifest(manifest_with_deps("mid", &["leaf"]));
        layer.register_manifest(manifest_with_deps("leaf", &[]));

        let deps = layer.resolve_transitive_deps("root");
        assert!(deps.contains(&"root".to_string()));
        assert!(deps.contains(&"mid".to_string()));
        assert!(deps.contains(&"leaf".to_string()));
        // root should be first
        assert_eq!(deps[0], "root");
    }

    #[test]
    fn test_dependency_circular_safety() {
        let mut layer = ProgressiveDisclosureLayer::new();
        layer.register_manifest(manifest_with_deps("a", &["b"]));
        layer.register_manifest(manifest_with_deps("b", &["c"]));
        layer.register_manifest(manifest_with_deps("c", &["a"]));

        let deps = layer.resolve_transitive_deps("a");
        assert_eq!(deps.len(), 3);
        // Should not infinite-loop
    }

    // ── From SkillManifest ──

    #[test]
    fn test_from_skill_manifest() {
        let sm = SkillManifest {
            name: "my-skill".to_string(),
            description: "Does X".to_string(),
            author: None,
            version: "2.0.0".to_string(),
            trigger_words: vec!["deploy".to_string(), "release".to_string()],
            tags: vec!["devops".to_string()],
            dependencies: vec!["core-utils".to_string()],
            permission_level:
                crate::neotrix::nt_mind::self_iterating::pipeline::PermissionLevel::Suggest,
            min_core_version: None,
            script_paths: vec![],
            reference_paths: vec![],
        };
        let dm = DisclosureManifest::from_skill_manifest(&sm);
        assert_eq!(dm.skill_id, "my-skill");
        assert_eq!(dm.name, "my-skill");
        assert_eq!(dm.version, "2.0.0");
        assert_eq!(dm.description, "Does X");
        assert_eq!(dm.trigger_patterns, vec!["deploy", "release"]);
        assert_eq!(dm.dependencies, vec!["core-utils"]);
    }

    // ── Unregister ──

    #[test]
    fn test_unregister_removes_from_index_and_cache() {
        let mut layer = ProgressiveDisclosureLayer::new();
        layer.register_manifest(sample_manifest("tmp", "Temp", &["temp"], "Temporary"));
        layer.load_or_stub("tmp");
        assert_eq!(layer.manifest_count(), 1);
        assert!(layer.get_cached("tmp").is_some());

        layer.unregister("tmp");
        assert_eq!(layer.manifest_count(), 0);
        assert!(layer.get_cached("tmp").is_none());
    }

    // ── Top-K Bounds ──

    #[test]
    fn test_search_respects_top_k() {
        let mut layer = ProgressiveDisclosureLayer::new();
        for i in 0..10 {
            let id = format!("skill_{}", i);
            layer.register_manifest(sample_manifest(&id, &id, &["common"], &id));
        }
        // All 10 match "common"
        let results = layer.search_metadata("common", 3);
        assert_eq!(results.len(), 3);
    }

    // ── Estimated Cost ──

    #[test]
    fn test_estimated_cost_on_register() {
        let mut layer = ProgressiveDisclosureLayer::new();
        layer.register_manifest(DisclosureManifest {
            skill_id: "costly".to_string(),
            name: "Costly".to_string(),
            description: "A".repeat(50),
            version: "1.0.0".to_string(),
            trigger_patterns: vec!["a".repeat(10)],
            estimated_cost: 999,
            dependencies: vec![],
        });
        assert_eq!(layer.manifest_index[0].estimated_cost, 999);
    }
}
