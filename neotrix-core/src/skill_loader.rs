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
    /// Explicit exclusions: cases this skill must NOT be used for (P0-2 三段式之二).
    #[serde(default)]
    pub exclusions: Vec<String>,
    /// Output contract: promised result shape (P0-2 三段式之三).
    #[serde(default)]
    pub output_contract: Option<String>,
    /// License identifier (T35 E轨；S7.1 install 侧已有 license，此处候选本体侧对齐）.
    #[serde(default)]
    pub license: String,
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
    /// Explicit exclusions (P0-2 三段式之二).
    pub exclusions: Vec<String>,
    /// Output contract (P0-2 三段式之三).
    pub output_contract: Option<String>,
    /// License identifier (T35 E轨；index 透传／legacy 空串）.
    pub license: String,
    /// Intake gate verdict (P0-2 门禁).
    pub admission: SkillAdmission,
}

/// Intake gate verdict for the three-part description rule (P0-2).
///
/// A skill is `Admitted` only when all three parts are present:
/// triggers + exclusions + output contract. Anything else is `NeedsWork`
/// and enters the maturity pipeline at Candidate (never Trusted).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillAdmission {
    Admitted,
    NeedsWork {
        missing_trigger: bool,
        missing_exclusion: bool,
        missing_contract: bool,
    },
}

impl SkillAdmission {
    /// Whether the skill passed the intake gate.
    pub fn is_admitted(&self) -> bool {
        *self == SkillAdmission::Admitted
    }
}

/// Run the three-part intake gate over an index entry (P0-2).
///
/// Pure function so the gate is testable without filesystem access.
pub fn gate_skill(
    triggers: &[String],
    exclusions: &[String],
    output_contract: &Option<String>,
) -> SkillAdmission {
    let missing_trigger = triggers.is_empty();
    let missing_exclusion = exclusions.is_empty();
    let missing_contract = output_contract
        .as_ref()
        .map(|c| c.trim().is_empty())
        .unwrap_or(true);
    if !missing_trigger && !missing_exclusion && !missing_contract {
        SkillAdmission::Admitted
    } else {
        SkillAdmission::NeedsWork {
            missing_trigger,
            missing_exclusion,
            missing_contract,
        }
    }
}

/// 官方认证去重门禁占位（T35 E轨；蓝图桌面清单 P2-8＋R-P100）。
///
/// 同名（大小写不敏感）且已有官方认证条目存在即 true。
/// `certified` 是 SkillCandidate（L6）侧概念；本文件 ResolvedSkill 有 `tags` 字段，
/// 故以 `tags` 含 `"official"` 近似（精确小写匹配）。P 轨接入真实 certified 透传后再收敛。
/// 纯函数，无 IO。
pub fn is_official_converged(name: &str, existing: &[ResolvedSkill]) -> bool {
    existing
        .iter()
        .any(|s| s.name.eq_ignore_ascii_case(name) && s.tags.iter().any(|t| t == "official"))
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
    /// If true, only return skills admitted by the three-part gate (P0-2).
    pub require_admitted: bool,
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
        let index = self.load_index()?;
        // borrow 结束后再用 self.skill_dirs，避免与 load_index 的 &mut 借用重叠
        let index = index.clone();
        Ok(self.resolve_from_index(&index))
    }

    /// Resolve every category entry into a `ResolvedSkill` with a real on-disk path.
    ///
    /// Pure w.r.t. disk reads (only `Path::exists` touches the FS) so tests can
    /// inject a synthetic [`SkillIndex`] without touching `skills/index.json`.
    pub fn resolve_from_index(&self, index: &SkillIndex) -> Vec<ResolvedSkill> {
        let skill_dirs = &self.skill_dirs;
        // Build path lookup from skill_index
        let mut path_map: HashMap<&str, &SkillIndexEntry> = HashMap::new();
        for (name, entry) in &index.skill_index {
            path_map.insert(name.as_str(), entry);
        }

        let mut skills = Vec::new();

        for (cat_name, category) in &index.categories {
            for (skill_name, entry) in &category.skills {
                // Resolve the full path from skill_index.
                //
                // 2026-09-28 bug fix: `skill_index` 的 key 是**路径式**
                // （`architecture-auditor/diagnose`），而 `categories.<cat>.skills`
                // 的 key 是**裸名**（`diagnose`）。旧代码只用裸名查 ⇒ 嵌套技能全部
                // 落空，退化成 `skills/<裸名>` 这种不存在的路径（实测 45/58 失败，
                // `ResolvedSkill.exists=false`）。正确查法：**先试 `<cat>/<skill>`
                // 路径式 key，再退回裸名**（顶层技能的 key 恰好等于裸名）。
                let qualified = format!("{}/{}", cat_name, skill_name);
                let file_path = path_map
                    .get(qualified.as_str())
                    .or_else(|| path_map.get(skill_name.as_str()))
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
                    exclusions: entry.exclusions.clone(),
                    output_contract: entry.output_contract.clone(),
                    license: entry.license.clone(),
                    admission: gate_skill(
                        &entry.triggers,
                        &entry.exclusions,
                        &entry.output_contract,
                    ),
                });
            }
        }

        skills
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

        if filter.require_admitted && !skill.admission.is_admitted() {
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

        // Bonus for passing the three-part intake gate (P0-2)
        if skill.admission.is_admitted() {
            score += 8.0;
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
                    exclusions: entry.exclusions.clone(),
                    output_contract: entry.output_contract.clone(),
                    license: entry.license.clone(),
                    admission: gate_skill(
                        &entry.triggers,
                        &entry.exclusions,
                        &entry.output_contract,
                    ),
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
                            exclusions: Vec::new(),
                            output_contract: None,
                            license: String::new(),
                            admission: SkillAdmission::NeedsWork {
                                missing_trigger: true,
                                missing_exclusion: true,
                                missing_contract: true,
                            },
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
                                exclusions: Vec::new(),
                                output_contract: None,
                                license: String::new(),
                                admission: SkillAdmission::NeedsWork {
                                    missing_trigger: true,
                                    missing_exclusion: true,
                                    missing_contract: true,
                                },
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
        // 原断言 `!skill_dirs.is_empty()` 依赖**运行机器**: SkillLoader::new() 只收录
        // 存在的目录($HOME/.neotrix/skills 等), 于是在没装 skill 的机器/CI 上必然失败,
        // 也会被并发改 HOME 的其它测试(全仓 6 处 set_var("HOME"))打中 —— 这不是本测试
        // 该关心的事。改为断言真正的不变量: **收录进来的目录都真实存在**。
        // 机器无关, 且仍能抓住「收录了不存在的路径」这种真 bug。
        for d in &loader.skill_dirs {
            assert!(d.is_dir(), "收录了不存在的目录: {}", d.display());
        }
        // 显式目录构造仍应原样保留
        let explicit = SkillLoader::with_dirs(vec![PathBuf::from("/tmp/x")]);
        assert_eq!(explicit.skill_dirs.len(), 1);
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

    // ── skill_index 路径式 key 解析（2026-09-28 bug fix 固化）──
    //
    // 现场：`skill_index` 的 key 是路径式（`architecture-auditor/diagnose`），
    // `categories.<cat>.skills` 的 key 是裸名（`diagnose`）。旧代码只用裸名查
    // ⇒ 嵌套技能全部落空，退化成 `skills/<裸名>` 这种不存在的路径。
    // 实测 45/58 解析失败，`ResolvedSkill.exists=false`。

    fn index_with_path_keys() -> SkillIndex {
        let mut skill_index = HashMap::new();
        for (key, file) in [
            ("architecture-auditor", "architecture-auditor/SKILL.md"),
            (
                "architecture-auditor/diagnose",
                "architecture-auditor/diagnose/SKILL.md",
            ),
        ] {
            skill_index.insert(
                key.to_string(),
                SkillIndexEntry {
                    category: "architecture-auditor".into(),
                    file: file.into(),
                },
            );
        }
        let mut skills = HashMap::new();
        for name in ["architecture-auditor", "diagnose"] {
            skills.insert(
                name.to_string(),
                SkillEntry {
                    description: "d".into(),
                    tags: vec![],
                    triggers: vec![],
                    dependencies: vec![],
                    exclusions: vec![],
                    output_contract: None,
                    license: String::new(),
                },
            );
        }
        let mut categories = HashMap::new();
        categories.insert(
            "architecture-auditor".to_string(),
            SkillCategory {
                description: "c".into(),
                tags: vec![],
                skills,
            },
        );
        SkillIndex {
            version: "1.0.0".into(),
            generated: "2026-09-28".into(),
            categories,
            skill_index,
        }
    }

    #[test]
    fn test_skill_index_path_key_resolution() {
        // 用真实临时目录当 skill 根，让 exists 判定有意义。
        let tmp = std::env::temp_dir().join("nt_skill_loader_pathkey");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("architecture-auditor/diagnose"))
            .expect("create nested skill dir");
        std::fs::write(tmp.join("architecture-auditor/SKILL.md"), "top").expect("write top");
        std::fs::write(
            tmp.join("architecture-auditor/diagnose/SKILL.md"),
            "nested",
        )
        .expect("write nested");

        let loader = SkillLoader::with_dirs(vec![tmp.clone()]);
        let resolved = loader.resolve_from_index(&index_with_path_keys());
        assert_eq!(resolved.len(), 2);

        // 关键断言：嵌套技能必须落在真实路径上，而不是 `skills/diagnose`。
        let diag = resolved
            .iter()
            .find(|s| s.name == "diagnose")
            .expect("diagnose 应被解析出来");
        assert_eq!(
            diag.path,
            tmp.join("architecture-auditor/diagnose/SKILL.md"),
            "路径式 key 未被解析 —— 回退到了裸名路径"
        );
        assert!(diag.exists, "嵌套技能应存在");

        // 顶层技能（key 恰好等于裸名）也必须正确
        let top = resolved
            .iter()
            .find(|s| s.name == "architecture-auditor")
            .expect("顶层技能应被解析出来");
        assert_eq!(top.path, tmp.join("architecture-auditor/SKILL.md"));
        assert!(top.exists, "顶层技能应存在");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // -- P0-2 三段式门禁 --

    fn gated_entry() -> SkillEntry {
        SkillEntry {
            description: "d".into(),
            tags: vec![],
            triggers: vec!["合并".into()],
            dependencies: vec![],
            exclusions: vec!["不用于删除".into()],
            output_contract: Some("JSON".into()),
            license: String::new(),
        }
    }

    #[test]
    fn test_gate_admitted() {
        let e = gated_entry();
        assert_eq!(
            gate_skill(&e.triggers, &e.exclusions, &e.output_contract),
            SkillAdmission::Admitted
        );
    }

    #[test]
    fn test_gate_missing_parts() {
        assert_eq!(
            gate_skill(&[], &["x".into()], &Some("y".into())),
            SkillAdmission::NeedsWork {
                missing_trigger: true,
                missing_exclusion: false,
                missing_contract: false,
            }
        );
        assert_eq!(
            gate_skill(&["x".into()], &[], &None),
            SkillAdmission::NeedsWork {
                missing_trigger: false,
                missing_exclusion: true,
                missing_contract: true,
            }
        );
        // 空白契约视同缺失
        assert!(
            gate_skill(&["x".into()], &["y".into()], &Some("  ".into()))
                == SkillAdmission::NeedsWork {
                    missing_trigger: false,
                    missing_exclusion: false,
                    missing_contract: true,
                }
        );
    }

    #[test]
    fn test_admission_scoring_bonus() {
        let loader = SkillLoader::new();
        let admitted = ResolvedSkill {
            name: "a".into(),
            description: String::new(),
            path: PathBuf::new(),
            category: String::new(),
            tags: Vec::new(),
            triggers: Vec::new(),
            dependencies: Vec::new(),
            exists: false,
            exclusions: Vec::new(),
            output_contract: None,
            license: String::new(),
            admission: SkillAdmission::Admitted,
        };
        let needs_work = ResolvedSkill {
            admission: SkillAdmission::NeedsWork {
                missing_trigger: true,
                missing_exclusion: true,
                missing_contract: true,
            },
            ..admitted.clone()
        };
        let filter = SkillFilter::default();
        let admitted_score = loader.compute_score(&admitted, &filter);
        let needs_work_score = loader.compute_score(&needs_work, &filter);
        assert!(admitted_score - needs_work_score >= 8.0);
    }

    // -- T35 E轨：license 透传＋官方去重占位 --

    fn official_skill(name: &str) -> ResolvedSkill {
        ResolvedSkill {
            name: name.to_string(),
            description: String::new(),
            path: PathBuf::new(),
            category: String::new(),
            tags: vec!["official".to_string()],
            triggers: Vec::new(),
            dependencies: Vec::new(),
            exists: true,
            exclusions: Vec::new(),
            output_contract: None,
            license: "MIT".to_string(),
            admission: SkillAdmission::NeedsWork {
                missing_trigger: true,
                missing_exclusion: true,
                missing_contract: true,
            },
        }
    }

    #[test]
    fn test_is_official_converged() {
        let existing = vec![official_skill("DataSync")];
        // 同名大小写不敏感＋official 近似即 true
        assert!(is_official_converged("datasync", &existing));
        assert!(is_official_converged("DATASYNC", &existing));
        // 不同名即 false
        assert!(!is_official_converged("other", &existing));
        // 同名但无 official tag 即 false
        let mut plain = official_skill("DataSync");
        plain.tags = Vec::new();
        assert!(!is_official_converged("datasync", &[plain]));
        // 空表即 false
        let empty: Vec<ResolvedSkill> = Vec::new();
        assert!(!is_official_converged("datasync", &empty));
    }

    #[test]
    fn test_skill_entry_t35_license_serde() {
        // 旧快照（无 license）兼容且落默认空串
        let old_json = r#"{"description":"d","tags":[],"triggers":["t"],"dependencies":[],"exclusions":["e"],"output_contract":"JSON"}"#;
        let parsed: SkillEntry = match serde_json::from_str(old_json) {
            Ok(v) => v,
            Err(e) => {
                assert!(false, "old snapshot must parse: {e}");
                return;
            }
        };
        assert!(parsed.license.is_empty());
        // 非默认往返
        let mut full = parsed;
        full.license = "Apache-2.0".to_string();
        let value = match serde_json::to_value(&full) {
            Ok(v) => v,
            Err(e) => {
                assert!(false, "serialize must succeed: {e}");
                return;
            }
        };
        let back: SkillEntry = match serde_json::from_value(value) {
            Ok(v) => v,
            Err(e) => {
                assert!(false, "round-trip must parse: {e}");
                return;
            }
        };
        assert_eq!(back.license, "Apache-2.0");
    }
}
