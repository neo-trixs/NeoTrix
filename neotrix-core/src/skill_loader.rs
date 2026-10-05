//! Enhanced skill discovery and loading system.
//!
//! Provides index-based skill lookup with categories, tags, dependency tracking,
//! and fast search capabilities. Replaces the legacy filesystem-only scan with
//! a pre-built index that supports rich metadata queries.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
    // ── 调用策略（2026-09-28 吸收，见 SkillInvocationPolicy）──
    // 对齐官方 Agent Skills 的 `disable-model-invocation` / `user-invocable`：
    // 「存在」与「对模型暴露」是**正交两维**，不是一件事。
    /// 禁止模型自动加载本技能。缺省 false = 允许按 description 触发。
    #[serde(default)]
    pub disable_model_invocation: bool,
    /// 不在用户 `/` 菜单展示，只供程序化调用。缺省 true。
    #[serde(default = "default_true")]
    pub user_invocable: bool,
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

fn default_true() -> bool {
    true
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
    /// 调用策略（2026-09-28 吸收）：与「是否存在」「是否准入」正交的第三维。
    pub invocation: SkillInvocationPolicy,
}

/// 调用策略：技能**存在**与**对模型暴露**是两件独立的事。
///
/// 吸收源：官方 Agent Skills 规范的 `disable-model-invocation` 与
/// `user-invocable` 两个独立字段。四种组合全部合法，**包括两者皆假**
/// ——那种技能仍可被受信调用方按名字取用，只是不出现在任何自动发现面里。
///
/// NeoTrix 的具体动机：`skills/self-health/`、`skills/self-iteration-agent/`
/// 这类自检/自迭代技能**必须能被模型自动触发**（它们的价值就在于被触发），
/// 而 `external-absorption` 这类会**修改 KB 的技能**不应被模型随手唤起
/// ——此前两种诉求挤在同一个 `description` 里，无法分别表达。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillInvocationPolicy {
    /// 允许模型按 description/triggers 自动加载（`!disable_model_invocation`）。
    pub model_invocable: bool,
    /// 在用户 `/` 菜单中展示（`user_invocable`）。
    pub user_invocable: bool,
}

impl SkillInvocationPolicy {
    /// 是否应出现在**面向模型的目录**里（官方规范：目录只含
    /// name + description，且对不可调用的技能不可见）。
    pub fn visible_to_model(&self) -> bool {
        self.model_invocable
    }

    /// 是否应出现在**用户 `/` 菜单**里。
    pub fn visible_to_user(&self) -> bool {
        self.user_invocable
    }

    /// 是否仅供受信调用方按名取用（两个维度皆不暴露）。
    pub fn is_trusted_only(&self) -> bool {
        !self.model_invocable && !self.user_invocable
    }
}

impl Default for SkillInvocationPolicy {
    fn default() -> Self {
        // 缺省=最宽松：允许模型自动触发 + 出现在用户菜单。
        // 与 `SkillEntry` 的 serde default 一致，既有 index.json 行为不变。
        Self {
            model_invocable: true,
            user_invocable: true,
        }
    }
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

/// 谁在看这份技能列表 —— 决定 `SkillInvocationPolicy` 哪一维生效（A5 接线）。
///
/// 「存在」与「对谁可见」正交：同一批技能对**受信调用方**按名取用永远可用，
/// 但**自动面**（模型自动触发、用户 `/` 菜单）必须按策略裁掉。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkillAudience {
    /// 面向模型的**自动触发 / 广告面**（默认，见 [`SkillFilter::audience`]）。
    #[default]
    Model,
    /// 用户 `/` 菜单 / `user_invocable` 面。
    User,
    /// 受信调用方（按名 `load_skill`、依赖图、发现面）：不隐藏。
    Trusted,
}

impl SkillAudience {
    /// 该受众是否应看到此技能。
    ///
    /// 纯函数、单点裁决：`SkillInvocationPolicy` 的两个 `visible_to_*`
    /// 由此**首次获得生产消费者**（此前只有测试引用，等于策略未被强制）。
    pub fn admits(self, policy: &SkillInvocationPolicy) -> bool {
        match self {
            SkillAudience::Model => policy.visible_to_model(),
            SkillAudience::User => policy.visible_to_user(),
            SkillAudience::Trusted => true,
        }
    }
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
    /// 谁在看这份结果；缺省 [`SkillAudience::Model`]。
    ///
    /// 选 Model 为缺省，是因为 `search_skills` 的**唯一生产消费者**就是
    /// `skill_registry::SkillRegistry::match_trigger`（`skill_registry.rs:41`）
    /// —— 触发词自动选技能，即「模型按 description/triggers 自动加载」。
    /// 要按名取用走 `load_skill`（Trusted，不隐藏），要浏览用户菜单走
    /// `list_visible_to_user`。
    pub audience: SkillAudience,
}

/// Search results with relevance ranking.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub skill: ResolvedSkill,
    /// Relevance score: higher = more relevant.
    pub score: f64,
}

/// 进程级 `index.json` 解析缓存（A5, 2026-10-05）。
///
/// **为什么需要进程级**：`SkillLoader` 的调用方每次都新建实例
/// （`consciousness_core::core::tick` 每 tick 一个、`SkillRegistry::new` 一个），
/// 所以 loader 内的 memo 字段永远命中不了 ⇒ 每 tick 重读 + 重解析 27KB JSON。
/// 代价落在 per-tick 热路径上，且 `consciousness_core/core.rs:265` 已有对应 TODO。
///
/// **失效判据**：`(绝对路径, mtime, len)`。三者任一变化即重解析 ⇒ 开发期
/// 重生成 index.json 立刻可见；len 一并纳入是为了在 mtime 粒度较粗的
/// 文件系统上不误判（mtime 相同但长度不同 ⇒ 内容必不同）。
///
/// **测试隔离**：按**绝对路径**分键。临时目录测试各自持有不同路径 ⇒ 天然
/// 不共享；`SkillLoader::with_dirs` 指向的目录亦同。已由
/// `test_index_cache_does_not_leak_between_dirs` /
/// `test_index_cache_invalidates_on_rewrite` 证明。
static INDEX_CACHE: std::sync::OnceLock<
    std::sync::Mutex<HashMap<PathBuf, (Option<std::time::SystemTime>, u64, Arc<SkillIndex>)>>,
> = std::sync::OnceLock::new();

fn index_cache() -> &'static std::sync::Mutex<
    HashMap<PathBuf, (Option<std::time::SystemTime>, u64, Arc<SkillIndex>)>,
> {
    INDEX_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

/// Enhanced skill loader with index-based discovery.
pub struct SkillLoader {
    /// `Arc` 而非 `SkillIndex`：多个 loader 共享同一份解析结果，克隆 O(1)。
    index: Option<Arc<SkillIndex>>,
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
    ///
    /// 读取走进程级缓存（见 [`INDEX_CACHE`]），故同一进程内重复调用不再读盘。
    pub fn load_index(&mut self) -> Result<&SkillIndex, String> {
        self.ensure_index()?;
        // `Arc<SkillIndex>` → `&SkillIndex`：deref 借用 self.index，生命周期与 &self 一致。
        let idx: &SkillIndex = self
            .index
            .as_deref()
            .ok_or_else(|| "skill index unavailable after load".to_string())?;
        Ok(idx)
    }

    /// 幂等：确保 `self.index` 已就位。已有则直接返回。
    ///
    /// 解析过一次后 `self.index` 命中，无需再 stat；未命中才按目录顺序找
    /// `index.json`，并经 [`INDEX_CACHE`] 复用其它 loader 的解析结果。
    fn ensure_index(&mut self) -> Result<(), String> {
        if self.index.is_some() {
            return Ok(());
        }

        for dir in &self.skill_dirs {
            let index_path = dir.join("index.json");
            if !index_path.exists() {
                continue;
            }
            // 绝对路径做键 ⇒ 相对 CWD 与临时目录互不串味。
            let key = std::fs::canonicalize(&index_path).unwrap_or_else(|_| index_path.clone());
            let meta = std::fs::metadata(&index_path).ok();
            let stamp = meta
                .as_ref()
                .map(|m| (m.modified().ok(), m.len()));

            if let Some((mtime, len)) = stamp {
                if let Ok(cache) = index_cache().lock() {
                    if let Some((hit_mtime, hit_len, idx)) = cache.get(&key) {
                        if *hit_mtime == mtime && *hit_len == len {
                            self.index = Some(idx.clone());
                            return Ok(());
                        }
                    }
                }
                // 锁被 poison 时 `if let Ok(..)` 落空 ⇒ 退化为直读，不 panic。
            }

            let content = std::fs::read_to_string(&index_path)
                .map_err(|e| format!("Failed to read {}: {}", index_path.display(), e))?;
            let parsed: SkillIndex = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse {}: {}", index_path.display(), e))?;
            let shared = Arc::new(parsed);
            if let Some((mtime, len)) = stamp {
                if let Ok(mut cache) = index_cache().lock() {
                    cache.insert(key, (mtime, len, shared.clone()));
                }
            }
            self.index = Some(shared);
            return Ok(());
        }

        Err("No index.json found in any skill directory".to_string())
    }

    /// List all known skills from the index, optionally resolving paths.
    ///
    /// **不做发现过滤**（A5, 2026-10-05）：本函数是**发现面**，被
    /// `consciousness_core::core::tick`（workspace_context.active_skills）与
    /// `agent::skills::SkillsEngine::init` 消费；调用策略（`visible_to_*`）在
    /// **投放面**强制（[`SkillLoader::list_visible_to`] 与
    /// [`SkillFilter::audience`]`search_skills`），不在这里丢条目 ——
    /// 隐藏项仍须可被发现、可按名取用，只是不被自动选中/不被列出。
    pub fn list_skills(&mut self) -> Result<Vec<ResolvedSkill>, String> {
        self.ensure_index()?;
        // `Arc::clone` 而非深克隆整棵 index：旧写法 `let index = index.clone()`
        // 为绕开 `load_index(&mut self)` 与 `resolve_from_index(&self, ..)` 的
        // 借用重叠而复制全部 categories/skill_index/Vec<String>，且**每 tick
        // 一次**（consciousness_core tick 路径）。现在 memo 里已是 `Arc`，
        // 克隆是引用计数加一。
        let index: Arc<SkillIndex> = self
            .index
            .clone()
            .ok_or_else(|| "skill index unavailable after load".to_string())?;
        Ok(self.resolve_from_index(&index))
    }

    /// **投放面**：可向某受众展示的技能清单。
    ///
    /// A5 的接线点在此处而非 `list_skills` —— `list_skills` 是**发现面**
    /// （`consciousness_core` 每 tick 取全量名字写 `active_skills`、
    /// `SkillsEngine::init` 靠它计数），在那里丢条目会改掉「有哪些技能存在」
    /// 这一发现语义；本函数只回答「**谁能看到哪些**」，发现面保持完整。
    pub fn list_visible_to(&mut self, audience: SkillAudience) -> Result<Vec<ResolvedSkill>, String> {
        Ok(self
            .list_skills()?
            .into_iter()
            .filter(|s| audience.admits(&s.invocation))
            .collect())
    }

    /// 面向模型的目录：`disable_model_invocation` 的技能不出现在这里。
    pub fn list_visible_to_model(&mut self) -> Result<Vec<ResolvedSkill>, String> {
        self.list_visible_to(SkillAudience::Model)
    }

    /// 用户 `/` 菜单：`user_invocable == false` 的技能不出现在这里。
    pub fn list_visible_to_user(&mut self) -> Result<Vec<ResolvedSkill>, String> {
        self.list_visible_to(SkillAudience::User)
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
                            // skill_dirs 为空（`with_dirs(vec![])` + 外部注入 index，
                            // resolve_from_index 是 pub）时不得索引 [0] ⇒ panic。
                            .or_else(|| skill_dirs.first().map(|d| d.join(&idx.file)))
                            .unwrap_or_else(|| PathBuf::from(&idx.file))
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
                    invocation: SkillInvocationPolicy {
                        model_invocable: !entry.disable_model_invocation,
                        user_invocable: entry.user_invocable,
                    },
                });
            }
        }

        skills
    }

    /// Load a specific skill by name.
    ///
    /// **受信面**（等价 [`SkillAudience::Trusted`]）：按名取用不受
    /// `disable_model_invocation` / `user_invocable` 限制 —— 这正是策略文档里
    /// 「仍可被受信调用方按名字取用」那一维。
    pub fn load_skill(&mut self, name: &str) -> Result<ResolvedSkill, String> {
        let skills = self.list_skills()?;
        skills
            .into_iter()
            .find(|s| s.name == name)
            .ok_or_else(|| format!("Skill '{}' not found", name))
    }

    /// Search skills using a filter. Returns results sorted by relevance score.
    ///
    /// 受众裁剪由 `filter.audience` 决定（缺省 [`SkillAudience::Model`]），
    /// 见 [`SkillFilter::audience`]。
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
        let index = self.load_index()?.clone(); // Arc clone：O(1)，非深克隆
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
    ///
    /// 与 `list_skills` 同为**发现面**（不做受众裁剪）：它是「按类目查全部」的
    /// 检索原语，调用方（`agent::skills::get_category`）自行决定投放给谁。
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
        // A5 接线：受众裁剪放**第一道** —— 策略是「能不能被看见」，
        // 先过策略再判内容匹配，省掉对不可见技能的无谓打分。
        if !filter.audience.admits(&skill.invocation) {
            return false;
        }

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
                    invocation: SkillInvocationPolicy {
                        model_invocable: !entry.disable_model_invocation,
                        user_invocable: entry.user_invocable,
                    },
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
                            invocation: SkillInvocationPolicy::default(),
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
                                invocation: SkillInvocationPolicy::default(),
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
///
/// **发现面**：不做受众裁剪（见 [`SkillLoader::list_skills`]）。
pub fn list_skills() -> Result<Vec<ResolvedSkill>, String> {
    SkillLoader::new().list_skills()
}

/// Convenience function: **投放面** —— 可向 `audience` 展示的技能。
pub fn list_visible_to(audience: SkillAudience) -> Result<Vec<ResolvedSkill>, String> {
    SkillLoader::new().list_visible_to(audience)
}

/// Convenience function: 面向模型的技能目录。
pub fn list_visible_to_model() -> Result<Vec<ResolvedSkill>, String> {
    SkillLoader::new().list_visible_to_model()
}

/// Convenience function: 用户 `/` 菜单的技能清单。
pub fn list_visible_to_user() -> Result<Vec<ResolvedSkill>, String> {
    SkillLoader::new().list_visible_to_user()
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
                    disable_model_invocation: false,
                    user_invocable: true,
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

    // ── 调用策略（2026-09-28 吸收）──

    #[test]
    fn test_invocation_policy_four_combinations() {
        // 四个组合全部合法，包括「两者皆不暴露」（仅受信调用方可取用）。
        // 语义恒等式，故期望值直接由输入推导 —— 手写第三四列反而容易把
        // `model_invocable` 与 `disable_model_invocation` 的极性写反（本轮就翻过一次）。
        for (disable_model, user_invocable) in
            [(false, true), (true, true), (true, false), (false, false)]
        {
            let p = SkillInvocationPolicy {
                model_invocable: !disable_model,
                user_invocable,
            };
            assert_eq!(p.visible_to_model(), !disable_model, "disable_model={disable_model}");
            assert_eq!(p.visible_to_user(), user_invocable);
            assert_eq!(
                p.is_trusted_only(),
                disable_model && !user_invocable,
                "is_trusted_only == (禁止模型 且 不对用户暴露)"
            );
        }
    }

    #[test]
    fn test_invocation_policy_default_is_permissive() {
        // 既有 index.json 无这两个 key ⇒ 行为必须与吸收前一致（最宽松）。
        let d = SkillInvocationPolicy::default();
        assert!(d.visible_to_model() && d.visible_to_user());
        assert!(!d.is_trusted_only());
    }

    #[test]
    fn test_entry_invocation_derives_from_serde_fields() {
        // serde 缺省：disable_model_invocation=false / user_invocable=true
        let e: SkillEntry =
            serde_json::from_str(r#"{"description":"d","tags":[]}"#).expect("parse");
        assert!(!e.disable_model_invocation);
        assert!(e.user_invocable, "user_invocable 缺省必须是 true");
    }

    #[test]
    fn test_entry_invocation_honors_explicit_false() {
        // 显式 false 必须被解析成 false（不能被 default 覆盖）——
        // 这是「对模型隐藏但用户可调」这一组合的表达力所在。
        let e: SkillEntry = serde_json::from_str(
            r#"{"description":"d","tags":[],"user_invocable":false}"#,
        )
        .expect("parse");
        assert!(!e.user_invocable);
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
            disable_model_invocation: false,
            user_invocable: true,
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
        
            invocation: SkillInvocationPolicy::default(),
};
        let needs_work = ResolvedSkill {
            admission: SkillAdmission::NeedsWork {
                missing_trigger: true,
                missing_exclusion: true,
                missing_contract: true,
            },
            invocation: SkillInvocationPolicy::default(),
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
            invocation: SkillInvocationPolicy::default(),
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

    // ── A5：调用策略接线（投放面强制，发现面保持完整）──

    /// 一条 index，含四个调用策略组合各一，外加各自独立的触发词。
    fn policy_index() -> SkillIndex {
        let mk = |desc: &str, disable_model: bool, user_invocable: bool| SkillEntry {
            description: desc.into(),
            tags: vec![],
            triggers: vec![format!("trig-{}", desc)],
            dependencies: vec![],
            exclusions: vec!["不用于删除".into()],
            output_contract: Some("JSON".into()),
            license: String::new(),
            disable_model_invocation: disable_model,
            user_invocable,
        };
        // name 即 desc，四组极性齐全
        let mut skills = HashMap::new();
        for (name, dm, ui) in [
            ("both", false, true),   // 两面都可见
            ("no-model", true, true), // 只对用户可见
            ("no-user", false, false), // 只对模型可见
            ("neither", true, false), // 仅受信调用方
        ] {
            skills.insert(name.to_string(), mk(name, dm, ui));
        }
        let mut skill_index = HashMap::new();
        for name in ["both", "no-model", "no-user", "neither"] {
            skill_index.insert(
                name.to_string(),
                SkillIndexEntry {
                    category: "pol".into(),
                    file: format!("{}/SKILL.md", name),
                },
            );
        }
        let mut categories = HashMap::new();
        categories.insert(
            "pol".to_string(),
            SkillCategory {
                description: "c".into(),
                tags: vec![],
                skills,
            },
        );
        SkillIndex {
            version: "1.0.0".into(),
            generated: "2026-10-05".into(),
            categories,
            skill_index,
        }
    }

    fn names(skills: &[ResolvedSkill]) -> Vec<String> {
        let mut v: Vec<String> = skills.iter().map(|s| s.name.clone()).collect();
        v.sort();
        v
    }

    #[test]
    fn test_discovery_surface_keeps_all_four_combinations() {
        // 发现面不得丢条目：隐藏项仍「存在」，否则 conscious tick 的
        // active_skills 与 SkillsEngine::init 的计数都会失真。
        let loader = SkillLoader::with_dirs(vec![PathBuf::from("/nonexistent-nt-dir")]);
        let all = loader.resolve_from_index(&policy_index());
        assert_eq!(
            names(&all),
            vec!["both", "neither", "no-model", "no-user"],
            "发现面必须返回全部四条"
        );
    }

    #[test]
    fn test_model_surface_excludes_disable_model_invocation() {
        let loader = SkillLoader::with_dirs(vec![PathBuf::from("/nonexistent-nt-dir")]);
        let model: Vec<ResolvedSkill> = loader
            .resolve_from_index(&policy_index())
            .into_iter()
            .filter(|s| s.invocation.visible_to_model())
            .collect();
        assert_eq!(
            names(&model),
            vec!["both", "no-user"],
            "disable_model_invocation 的技能不得对模型可见"
        );
    }

    #[test]
    fn test_user_surface_excludes_non_user_invocable() {
        let loader = SkillLoader::with_dirs(vec![PathBuf::from("/nonexistent-nt-dir")]);
        let user: Vec<ResolvedSkill> = loader
            .resolve_from_index(&policy_index())
            .into_iter()
            .filter(|s| s.invocation.visible_to_user())
            .collect();
        assert_eq!(
            names(&user),
            vec!["both", "no-model"],
            "user_invocable=false 的技能不得作为用户命令出现"
        );
    }

    #[test]
    fn test_trusted_surface_sees_everything() {
        let loader = SkillLoader::with_dirs(vec![PathBuf::from("/nonexistent-nt-dir")]);
        let trusted: Vec<ResolvedSkill> = loader
            .resolve_from_index(&policy_index())
            .into_iter()
            .filter(|s| SkillAudience::Trusted.admits(&s.invocation))
            .collect();
        assert_eq!(names(&trusted).len(), 4, "受信面按名取用不受策略限制");
    }

    #[test]
    fn test_audience_admits_is_the_single_verdict_point() {
        // 四组合 × 三受众的完整真值表（期望值按语义手推，与实现独立）。
        // (disable_model, user_invocable) -> (Model, User, Trusted)
        let cases = [
            (false, true, true, true, true),
            (true, true, false, true, true),
            (false, false, true, false, true),
            (true, false, false, false, true),
        ];
        for (dm, ui, want_model, want_user, want_trusted) in cases {
            let p = SkillInvocationPolicy {
                model_invocable: !dm,
                user_invocable: ui,
            };
            assert_eq!(
                SkillAudience::Model.admits(&p),
                want_model,
                "disable_model={dm}"
            );
            assert_eq!(SkillAudience::User.admits(&p), want_user, "user_invocable={ui}");
            assert_eq!(SkillAudience::Trusted.admits(&p), want_trusted);
        }
    }

    #[test]
    fn test_search_default_audience_hides_disable_model_skill() {
        // 端到端过 search_skills（唯一生产消费者是 SkillRegistry::match_trigger）。
        // 用真实临时目录 + 真实 index.json，避免依赖 resolve_from_index 的
        // 内部路径而漏掉「策略真的落到 search 上」这一层。
        let dir = std::env::temp_dir().join("nt_skill_loader_policy_search");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let json = serde_json::to_string(&policy_index()).expect("ser");
        std::fs::write(dir.join("index.json"), json).expect("write index");

        let mut loader = SkillLoader::with_dirs(vec![dir.clone()]);

        // 触发词命中全部四条；默认受众 = Model ⇒ 只能看到两条。
        let filter = SkillFilter {
            triggers: vec!["trig-".to_string()],
            ..Default::default()
        };
        let default_audience: Vec<String> = match loader.search_skills(&filter) {
            Ok(results) => results.into_iter().map(|r| r.skill.name).collect(),
            Err(e) => {
                assert!(false, "search must succeed: {e}");
                return;
            }
        };
        let mut sorted = default_audience.clone();
        sorted.sort();
        assert_eq!(sorted, vec!["both", "no-user"], "默认受众应裁掉 disable_model");

        // 显式 Trusted ⇒ 全部可见（同一批 index，只差受众参数）。
        let trusted = SkillFilter {
            triggers: vec!["trig-".to_string()],
            audience: SkillAudience::Trusted,
            ..Default::default()
        };
        let trusted_names: Vec<String> = match loader.search_skills(&trusted) {
            Ok(results) => results.into_iter().map(|r| r.skill.name).collect(),
            Err(e) => {
                assert!(false, "search must succeed: {e}");
                return;
            }
        };
        let mut sorted_trusted = trusted_names.clone();
        sorted_trusted.sort();
        assert_eq!(sorted_trusted.len(), 4, "受信受众应看到全部");

        // 发现面仍然完整（不受 search 的裁剪影响）。
        let discovered = match loader.list_skills() {
            Ok(v) => names(&v),
            Err(e) => {
                assert!(false, "list must succeed: {e}");
                return;
            }
        };
        assert_eq!(discovered.len(), 4, "list_skills 是发现面，不得裁剪");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_visible_to_applies_policy_end_to_end() {
        let dir = std::env::temp_dir().join("nt_skill_loader_policy_visible");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(
            dir.join("index.json"),
            serde_json::to_string(&policy_index()).expect("ser"),
        )
        .expect("write index");

        let mut loader = SkillLoader::with_dirs(vec![dir.clone()]);
        assert_eq!(
            names(&loader.list_visible_to_model().expect("model")),
            vec!["both", "no-user"]
        );
        assert_eq!(
            names(&loader.list_visible_to_user().expect("user")),
            vec!["both", "no-model"]
        );
        assert_eq!(
            names(&loader.list_visible_to(SkillAudience::Trusted).expect("trusted")),
            vec!["both", "neither", "no-model", "no-user"]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_skill_by_name_reaches_hidden_skill() {
        // 「两面都不暴露」的技能仍须能被受信调用方按名取到 —— 这是策略
        // 文档里明确的第三种合法状态（is_trusted_only）。
        let dir = std::env::temp_dir().join("nt_skill_loader_policy_byname");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(
            dir.join("index.json"),
            serde_json::to_string(&policy_index()).expect("ser"),
        )
        .expect("write index");

        let mut loader = SkillLoader::with_dirs(vec![dir.clone()]);
        let got = loader.load_skill("neither");
        assert!(
            got.as_ref().map(|s| s.invocation.is_trusted_only()) == Ok(true),
            "按名取用必须成功且标记为 trusted_only，实际 {:?}",
            got.as_ref().map(|s| &s.invocation)
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── A5：进程级 index 缓存的隔离与失效 ──

    fn write_min_index(dir: &Path, name: &str) {
        let mut skills = HashMap::new();
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
                disable_model_invocation: false,
                user_invocable: true,
            },
        );
        let mut skill_index = HashMap::new();
        skill_index.insert(
            name.to_string(),
            SkillIndexEntry {
                category: "c".into(),
                file: format!("{}/SKILL.md", name),
            },
        );
        let mut categories = HashMap::new();
        categories.insert(
            "c".to_string(),
            SkillCategory {
                description: "c".into(),
                tags: vec![],
                skills,
            },
        );
        let idx = SkillIndex {
            version: "1.0.0".into(),
            generated: "2026-10-05".into(),
            categories,
            skill_index,
        };
        std::fs::write(
            dir.join("index.json"),
            serde_json::to_string(&idx).expect("ser"),
        )
        .expect("write index");
    }

    #[test]
    fn test_index_cache_does_not_leak_between_dirs() {
        // 两个不同绝对路径 ⇒ 两条独立缓存条目。证明缓存键是路径而非内容/单例。
        let a = std::env::temp_dir().join("nt_skill_cache_iso_a");
        let b = std::env::temp_dir().join("nt_skill_cache_iso_b");
        for d in [&a, &b] {
            let _ = std::fs::remove_dir_all(d);
            std::fs::create_dir_all(d).expect("mkdir");
        }
        write_min_index(&a, "skill-a");
        write_min_index(&b, "skill-b");

        // 交替构造 loader，模拟「每次调用新建 SkillLoader」的真实用法。
        for _ in 0..3 {
            assert_eq!(
                names(&SkillLoader::with_dirs(vec![a.clone()]).list_skills().expect("a")),
                vec!["skill-a"]
            );
            assert_eq!(
                names(&SkillLoader::with_dirs(vec![b.clone()]).list_skills().expect("b")),
                vec!["skill-b"]
            );
        }
        let _ = std::fs::remove_dir_all(&a);
        let _ = std::fs::remove_dir_all(&b);
    }

    #[test]
    fn test_index_cache_invalidates_on_rewrite() {
        // 同路径重写 index.json ⇒ 新内容必须可见（缓存不得变成陈旧真相）。
        let dir = std::env::temp_dir().join("nt_skill_cache_invalidate");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        write_min_index(&dir, "v1");
        assert_eq!(
            names(&SkillLoader::with_dirs(vec![dir.clone()]).list_skills().expect("first")),
            vec!["v1"]
        );

        write_min_index(&dir, "v2-longer-name-to-change-len");
        assert_eq!(
            names(&SkillLoader::with_dirs(vec![dir.clone()]).list_skills().expect("second")),
            vec!["v2-longer-name-to-change-len"],
            "重写后必须读到新 index（len 变化 ⇒ 缓存失效）"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_missing_index_still_errors() {
        // 缓存不得把「找不到」也缓存住：目录不存在必须照旧报错。
        let mut loader = SkillLoader::with_dirs(vec![PathBuf::from("/nonexistent-nt-dir-xyz")]);
        assert!(loader.load_index().is_err());
        assert!(loader.list_skills().is_err());
    }

    #[test]
    fn test_resolve_from_index_with_empty_dirs_does_not_panic() {
        // `with_dirs(vec![])` + 外部注入 index：旧代码在此处索引 skill_dirs[0]
        // ⇒ panic。resolve_from_index 是 pub，属可达路径。
        let loader = SkillLoader::with_dirs(Vec::new());
        let resolved = loader.resolve_from_index(&policy_index());
        assert_eq!(resolved.len(), 4, "空目录列表下仍应解析出全部条目");
    }
}
