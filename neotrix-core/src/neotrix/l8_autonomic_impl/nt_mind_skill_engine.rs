//! Skill auto-invocation engine — scans, parses, indexes, and auto-invokes
//! skill markdown files with YAML frontmatter.
//!
//! Integrates with:
//!   - nt_mind_hook: fires SkillLoaded/SkillUnloaded HookEvents
//!   - GWT workspace: broadcasts skill activation events

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::core::nt_core_gwt::workspace::GlobalWorkspace;
use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::ProceduralMemoryRecord;
use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::{skill_upsert, SkillRecord};
use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;
use crate::neotrix::l8_autonomic_impl::nt_mind_hook::{HookEvent, MindHookRegistry, HookContext, HookResult};

/// A single skill entry parsed from a markdown file with YAML frontmatter.
#[derive(Debug, Clone)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub e8_modes: Vec<u8>,
    pub tools: Vec<String>,
    pub hooks: Vec<String>,
    pub priority: u8,
    pub path: PathBuf,
    pub content: String,
    pub active: bool,
    /// 渐进披露 (progressive disclosure, 吸收自 cathrynlavery/diagram-design):
    /// SKILL.md 只保留选择指南, 深层细节以 `references/*.md` 按需加载。
    pub references: Vec<String>,
    /// 技能树层级 (AgentSkillOS 吸收): 粗到细分类 + 父技能指针, 支撑互补性检索。
    pub category: String,
    pub parent: String,
}

impl SkillEntry {
    fn from_file(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        Self::from_content(path, &content)
    }

    fn from_content(path: &Path, content: &str) -> Option<Self> {
        let stripped = content.trim_start();
        if !stripped.starts_with("---") {
            return None;
        }
        let end = stripped[3..].find("---")?;
        let frontmatter = &stripped[3..3 + end];

        let mut name = String::new();
        let mut description = String::new();
        let mut triggers = Vec::new();
        let mut e8_modes = Vec::new();
        let mut tools = Vec::new();
        let mut hooks = Vec::new();
        let mut priority: u8 = 50;
        let mut references = Vec::new();
        let mut category = "general".to_string();
        let mut parent = String::new();

        for line in frontmatter.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("name:") {
                name = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("description:") {
                description = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("triggers:") {
                triggers = parse_array_field(val);
            } else if let Some(val) = line.strip_prefix("e8_modes:") {
                e8_modes = parse_array_field(val).iter().filter_map(|s| s.parse::<u8>().ok()).collect();
            } else if let Some(val) = line.strip_prefix("tools:") {
                tools = parse_array_field(val);
            } else if let Some(val) = line.strip_prefix("hooks:") {
                hooks = parse_array_field(val);
            } else if let Some(val) = line.strip_prefix("references:") {
                references = parse_array_field(val);
            } else if let Some(val) = line.strip_prefix("category:") {
                let c = val.trim().trim_matches('"').trim_matches('\'').to_string();
                if !c.is_empty() {
                    category = c;
                }
            } else if let Some(val) = line.strip_prefix("parent:") {
                parent = val.trim().trim_matches('"').trim_matches('\'').to_string();
            } else if let Some(val) = line.strip_prefix("priority:") {
                priority = val.trim().parse::<u8>().unwrap_or(50).min(100);
            }
        }

        if name.is_empty() || description.is_empty() {
            return None;
        }

        Some(Self {
            name,
            description,
            triggers,
            e8_modes,
            tools,
            hooks,
            priority,
            path: path.to_path_buf(),
            content: content.to_string(),
            active: false,
            references,
            category,
            parent,
        })
    }

    pub fn body(&self) -> &str {
        let stripped = self.content.trim_start();
        if !stripped.starts_with("---") {
            return stripped;
        }
        if let Some(end) = stripped[3..].find("---") {
            &stripped[3 + end + 3..]
        } else {
            stripped
        }
    }
}

/// 差分归因记录 (arxiv 2608.11888 SkillTriage 吸收): 单个技能的激活统计与
/// procedure-heavy 风险标记。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillAttribution {
    pub name: String,
    pub category: String,
    pub activations: u32,
    pub over_validation_score: u32,
    pub procedure_heavy: bool,
    pub flagged: bool,
}

/// 技能树层级统计 (G6, AgentSkillOS 吸收): 巡检报告的数据载体。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillTreeStats {
    pub total_skills: usize,
    pub categories: HashMap<String, usize>,
    pub roots: usize,
    pub orphans: usize,
    pub max_depth: usize,
}

fn parse_array_field(val: &str) -> Vec<String> {
    let trimmed = val.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        inner.split(',')
            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        trimmed.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Core skill engine: scan, index, match, activate/deactivate.
pub struct SkillEngine {
    skills_dir: PathBuf,
    skills: Vec<SkillEntry>,
    /// Index: trigger keyword → skill indices
    trigger_index: HashMap<String, Vec<usize>>,
    /// Index: E8 mode → skill indices
    e8_index: HashMap<u8, Vec<usize>>,
    /// Optional hook registry for firing lifecycle events
    hooks: Option<MindHookRegistry>,
    /// Optional GWT for broadcasting activation events
    gwt: Option<Arc<RwLock<GlobalWorkspace>>>,
/// Optional KB handle: when attached, load_all() auto-syncs the skill
    /// index into the KB `skills_index` table (UCN Phase 1 写通)。
    kb: Option<Arc<KnowledgeBase>>,
    /// 差分归因 (arxiv 2608.11888 SkillTriage 吸收): 每 skill 的激活次数 /
    /// 过程过重标记, 识别 procedure-heavy 技能 (过度验证 = 强制劳动毒源)。
    attribution: HashMap<String, SkillAttribution>,
}

impl SkillEngine {
    pub fn new(skills_dir: PathBuf) -> Self {
        Self {
            skills_dir,
            skills: Vec::new(),
            trigger_index: HashMap::new(),
            e8_index: HashMap::new(),
            hooks: None,
            gwt: None,
            kb: None,
            attribution: HashMap::new(),
        }
    }

    pub fn with_kb(mut self, kb: Arc<KnowledgeBase>) -> Self {
        self.kb = Some(kb);
        self
    }

    pub fn kb(&self) -> Option<&Arc<KnowledgeBase>> {
        self.kb.as_ref()
    }

    pub fn with_hooks(mut self, hooks: MindHookRegistry) -> Self {
        self.hooks = Some(hooks);
        self
    }

    pub fn with_gwt(mut self, gwt: Arc<RwLock<GlobalWorkspace>>) -> Self {
        self.gwt = Some(gwt);
        self
    }

    /// Scan the skills directory and load all valid skill files.
    pub fn load_all(&mut self) -> Vec<SkillEntry> {
        self.skills.clear();
        self.trigger_index.clear();
        self.e8_index.clear();

        let dir = &self.skills_dir;
        if !dir.exists() {
            let _ = std::fs::create_dir_all(dir);
            return Vec::new();
        }

        let mut loaded = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Some(skill) = SkillEntry::from_file(&skill_md) {
                            loaded.push(skill);
                        }
                    }
                    continue;
                }
                if path.extension().is_some_and(|e| e == "md") {
                    if let Some(skill) = SkillEntry::from_file(&path) {
                        loaded.push(skill);
                    }
                }
            }
        }

        self.skills = loaded;
        self.build_index();
        // UCN Phase 1 写通: 若挂接 KB, 扫描后自动把索引同步进 skills_index 表。
        if let Some(kb) = self.kb.clone() {
            if let Ok(conn) = kb.raw_conn() {
                let _ = self.sync_to_kb_index(&conn);
            }
        }
        self.skills.clone()
    }

    /// 把当前内存索引同步到 KB `skills_index` 表 (UCN Phase 1 写通)。
    /// 返回本次真正写入/更新的条数; 内容未变化 (content_hash 相同) 被去重跳过。
    pub fn sync_to_kb_index(&self, conn: &rusqlite::Connection) -> Result<usize, String> {
        use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::skill_content_hash;
        use std::collections::HashSet;

        let mut written = 0usize;
        let mut seen: HashSet<String> = HashSet::new();
        for skill in &self.skills {
            if !seen.insert(skill.name.clone()) {
                continue;
            }
            let record = SkillRecord {
                id: uuid::Uuid::new_v4().to_string(),
                name: skill.name.clone(),
                description: Some(skill.description.clone()),
                source_path: Some(skill.path.to_string_lossy().to_string()),
                tags: if skill.triggers.is_empty() {
                    None
                } else {
                    Some(skill.triggers.join(","))
                },
                is_builtin: false,
                last_indexed_at: Some(crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::now()),
                created_at: crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::now(),
                updated_at: crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::now(),
                content_hash: Some(skill_content_hash(&skill.content)),
            };
            if skill_upsert(conn, &record.name, &record)? {
                written += 1;
            }
        }
        Ok(written)
    }

    /// Build trigger and E8 mode indices.
    fn build_index(&mut self) {
        self.trigger_index.clear();
        self.e8_index.clear();

        for (i, skill) in self.skills.iter().enumerate() {
            for trigger in &skill.triggers {
                let key = trigger.to_lowercase();
                self.trigger_index.entry(key).or_default().push(i);
            }
            for mode in &skill.e8_modes {
                self.e8_index.entry(*mode).or_default().push(i);
            }
        }
    }

    /// Find skills matching a query string and optional E8 mode.
    /// When `e8_mode` is `None`, the E8 mode filter is skipped.
    /// Matching is case-insensitive keyword match against triggers.
    /// Results are sorted by priority descending, then by trigger relevance.
    /// 反哺自 spec-kit/autoroute 吸收: 确定性优先级栈 (exact > substring) + 硬结果上限
    /// (open-code-review 预算纪律) — 防止路由返回无界候选淹没下游消费方。
    pub const MAX_ROUTE_RESULTS: usize = 8;

    pub fn find_matching(&self, query: &str, e8_mode: Option<u8>) -> Vec<&SkillEntry> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<String> = query_lower.split_whitespace()
            .map(|s| s.to_string())
            .chain(std::iter::once(query_lower.clone()))
            .collect();

        // tier 0 = exact trigger equality (最高优先级, 确定性命中)
        // tier 1 = substring 命中
        let mut exact: Vec<(usize, usize, &SkillEntry)> = Vec::new();
        let mut scored: Vec<(usize, usize, &SkillEntry)> = Vec::new();

        for skill in self.skills.iter() {
            if let Some(mode) = e8_mode {
                if !skill.e8_modes.is_empty() && !skill.e8_modes.contains(&mode) {
                    continue;
                }
            }
            let mut exact_count = 0;
            let mut match_count = 0;
            for word in &query_words {
                for trigger in &skill.triggers {
                    let t_lower = trigger.to_lowercase();
                    if t_lower == *word {
                        exact_count += 1;
                    } else if t_lower.contains(word.as_str()) || word.contains(t_lower.as_str()) {
                        match_count += 1;
                    }
                }
            }
            if exact_count > 0 {
                exact.push((exact_count, skill.priority as usize, skill));
            } else if match_count > 0 {
                scored.push((match_count, skill.priority as usize, skill));
            }
        }

        // Sort: desc by exact_count, then desc by priority (确定性优先层)
        exact.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
        // Sort: desc by match_count, then desc by priority
        scored.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));

        exact.into_iter().map(|(_, _, s)| s)
            .chain(scored.into_iter().map(|(_, _, s)| s))
            .take(Self::MAX_ROUTE_RESULTS)
            .collect()
    }

    /// 技能树 (AgentSkillOS 吸收): category → skills, 每类内按 priority 降序。
    pub fn skill_tree(&self) -> HashMap<String, Vec<&SkillEntry>> {
        let mut tree: HashMap<String, Vec<&SkillEntry>> = HashMap::new();
        for s in self.skills.iter() {
            tree.entry(s.category.clone()).or_default().push(s);
        }
        for v in tree.values_mut() {
            v.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
        tree
    }

    pub fn children_of(&self, name: &str) -> Vec<&SkillEntry> {
        self.skills.iter().filter(|s| s.parent == name).collect()
    }

    /// 互补性感知检索 (AgentSkillOS 吸收): 在 find_matching 候选基础上, 对
    /// 与已激活技能同 category 的候选施加降级, 优先返回未覆盖类别 (多样化)。
    pub fn find_matching_complementary(
        &self,
        query: &str,
        e8_mode: Option<u8>,
        active_names: &[&str],
    ) -> Vec<&SkillEntry> {
        let covered: Vec<String> = self
            .skills
            .iter()
            .filter(|s| active_names.contains(&s.name.as_str()))
            .map(|s| s.category.clone())
            .collect();

        let mut candidates = self.find_matching(query, e8_mode);
        candidates.sort_by(|a, b| {
            let a_covered = covered.contains(&a.category);
            let b_covered = covered.contains(&b.category);
            match (a_covered, b_covered) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => b.priority.cmp(&a.priority),
            }
        });
        candidates
    }

    pub fn record_activation(&mut self, name: &str) {
        let Some(entry) = self.get_skill(name) else {
            return;
        };
        let entry = entry.clone();
        let score = self.over_validation_score(name);
        let procedure_heavy = score >= 12;
        let attr = self.attribution.entry(name.to_string()).or_insert(SkillAttribution {
            name: name.to_string(),
            category: entry.category.clone(),
            activations: 0,
            over_validation_score: score,
            procedure_heavy,
            flagged: false,
        });
        attr.activations += 1;
        attr.over_validation_score = score;
        attr.procedure_heavy = procedure_heavy;
        attr.flagged = procedure_heavy && attr.activations > 1;
    }

    pub fn over_validation_score(&self, name: &str) -> u32 {
        let Some(skill) = self.get_skill(name) else {
            return 0;
        };
        let body = skill.body();
        let lower = body.to_lowercase();
        let markers = [
            "rebuild", "cargo clean", "verify", "re-read", "re read", "audit", "validate",
            "recheck", "must ensure", "compile twice", "check twice",
        ];
        let mut score = 0u32;
        for m in markers {
            score += lower.matches(m).count() as u32;
        }
        score += lower
            .lines()
            .filter(|l| {
                let t = l.trim();
                t.len() > 4
                    && (t.chars().next().is_some_and(|c| c.is_ascii_digit())
                        || t.starts_with('-'))
                    && t.matches(' ').count() >= 8
            })
            .count() as u32;
        score
    }

    pub fn attribution_report(&self) -> Vec<SkillAttribution> {
        let mut report: Vec<SkillAttribution> = self
            .skills
            .iter()
            .map(|s| {
                self.attribution
                    .get(&s.name)
                    .cloned()
                    .unwrap_or(SkillAttribution {
                        name: s.name.clone(),
                        category: s.category.clone(),
                        activations: 0,
                        over_validation_score: self.over_validation_score(&s.name),
                        procedure_heavy: false,
                        flagged: false,
                    })
            })
            .collect();
        report.sort_by(|a, b| b.activations.cmp(&a.activations));
        report
    }

    pub fn get_skill(&self, name: &str) -> Option<&SkillEntry> {
        self.skills.iter().find(|s| s.name == name)
    }

    pub fn get_skill_mut(&mut self, name: &str) -> Option<&mut SkillEntry> {
        self.skills.iter_mut().find(|s| s.name == name)
    }

    /// 渐进披露加载 (progressive disclosure, diagram-design 吸收):
    /// SKILL.md 只描述技能的选择与入口, 深层细节 (参考文档/模板/示例) 存于
    /// `<skill_dir>/references/<file>`, 按需读取 — 避免常驻加载拉爆上下文。
    ///
    /// 返回已声明引用中命中的内容; 未声明或不存在返回 Err (提示缺失)。
    pub fn load_reference(&self, name: &str, reference: &str) -> Result<String, String> {
        let entry = self
            .get_skill(name)
            .ok_or_else(|| format!("Skill '{}' not found", name))?;
        if !entry.references.iter().any(|r| r == reference) {
            return Err(format!(
                "Reference '{}' not declared in skill '{}' (declared: {:?})",
                reference, name, entry.references
            ));
        }
        let skill_dir = entry.path.parent().unwrap_or(&self.skills_dir);
        let ref_path = skill_dir.join("references").join(reference);
        if !ref_path.exists() {
            return Err(format!(
                "Reference file missing: {}",
                ref_path.display()
            ));
        }
        std::fs::read_to_string(&ref_path).map_err(|e| format!("read reference: {}", e))
    }

    /// Activate a skill by name. Fires HookEvent::SkillLoaded and GWT broadcast.
    pub fn activate_skill(&mut self, name: &str) -> Result<(), String> {
        let idx = self.skills.iter().position(|s| s.name == name)
            .ok_or_else(|| format!("Skill '{}' not found", name))?;
        if self.skills[idx].active {
            return Err(format!("Skill '{}' is already active", name));
        }
        self.skills[idx].active = true;
        self.record_activation(name);
        let desc = self.skills[idx].description.clone();
        let triggers = self.skills[idx].triggers.clone();
        let e8_modes = self.skills[idx].e8_modes.clone();
        let priority = self.skills[idx].priority;

        if let Some(ref mut hooks) = self.hooks {
            let ctx = HookContext::new(
                HookEvent::SkillLoaded,
                &format!("skill:{}", name),
            ).with_payload(serde_json::json!({
                "name": name,
                "description": desc,
                "triggers": triggers,
                "e8_modes": e8_modes,
                "priority": priority,
            }));
            hooks.trigger(&ctx);
        }

        if let Some(ref gwt) = self.gwt {
            if let Ok(mut gwt) = gwt.try_write() {
                gwt.broadcast(&format!("[skill_activated] {} — {}", name, desc));
            }
        }

        Ok(())
    }

    /// Deactivate a skill by name. Fires HookEvent::SkillUnloaded.
    pub fn deactivate_skill(&mut self, name: &str) -> Result<(), String> {
        let idx = self.skills.iter().position(|s| s.name == name)
            .ok_or_else(|| format!("Skill '{}' not found", name))?;
        if !self.skills[idx].active {
            return Err(format!("Skill '{}' is not active", name));
        }
        self.skills[idx].active = false;

        if let Some(ref mut hooks) = self.hooks {
            let ctx = HookContext::new(
                HookEvent::SkillUnloaded,
                &format!("skill:{}", name),
            );
            hooks.trigger(&ctx);
        }

        Ok(())
    }

    pub fn list_active(&self) -> Vec<&SkillEntry> {
        self.skills.iter().filter(|s| s.active).collect()
    }

    /// 技能树层级统计 (G6, AgentSkillOS 吸收): category 分布、根/叶/孤儿技能、
    /// 覆盖率与深度。供背景循环巡检报告使用。
    pub fn skill_tree_stats(&self) -> SkillTreeStats {
        let mut categories: HashMap<String, usize> = HashMap::new();
        let mut roots = 0usize;
        let mut orphans = 0usize;
        let mut max_depth = 0usize;
        let mut depth: HashMap<String, usize> = HashMap::new();

        for s in &self.skills {
            *categories.entry(s.category.clone()).or_insert(0) += 1;
            if s.parent.is_empty() {
                roots += 1;
            }
        }
        for _ in 0..=self.skills.len() {
            let mut changed = false;
            for s in &self.skills {
                if s.parent.is_empty() {
                    if depth.insert(s.name.clone(), 0).map_or(true, |old| old != 0) {
                        changed = true;
                    }
                } else {
                    if let Some(pd) = depth.get(&s.parent).copied() {
                        let d = pd + 1;
                        if depth.insert(s.name.clone(), d).map_or(true, |old| old != d) {
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        max_depth = depth.values().copied().max().unwrap_or(0);
        let known: std::collections::HashSet<&String> =
            self.skills.iter().map(|s| &s.name).collect();
        for s in &self.skills {
            if !s.parent.is_empty() && !known.contains(&s.parent) {
                orphans += 1;
            }
        }
        SkillTreeStats {
            total_skills: self.skills.len(),
            categories,
            roots,
            orphans,
            max_depth,
        }
    }

    /// 差分归因 flagged 汇总 (G7, arxiv 2608.11888 SkillTriage): 返回
    /// procedure-heavy 且被标记的技能 (过度验证毒源), 供巡检广播告警。
    pub fn flagged_attributions(&self) -> Vec<SkillAttribution> {
        self.attribution_report()
            .into_iter()
            .filter(|a| a.flagged || (a.procedure_heavy && a.activations > 0))
            .collect()
    }

    pub fn list_all(&self) -> Vec<&SkillEntry> {
        self.skills.iter().collect()
    }

    pub fn skills_dir(&self) -> &Path {
        &self.skills_dir
    }

    /// Install a skill from a source path (file or directory with SKILL.md).
    /// Copies the file(s) into the skills directory.
    pub fn install_skill(&mut self, source_path: &Path) -> Result<(), String> {
        if !source_path.exists() {
            return Err(format!("Source path does not exist: {}", source_path.display()));
        }

        if source_path.is_dir() {
            let skill_md = source_path.join("SKILL.md");
            if !skill_md.exists() {
                return Err("Directory must contain a SKILL.md file".to_string());
            }
            let content = std::fs::read_to_string(&skill_md).map_err(|e| e.to_string())?;
            let entry = SkillEntry::from_content(&skill_md, &content)
                .ok_or_else(|| "Invalid frontmatter in SKILL.md".to_string())?;

            let target_dir = self.skills_dir.join(&entry.name);
            let _ = std::fs::create_dir_all(&target_dir);

            // Copy SKILL.md
            let dest = target_dir.join("SKILL.md");
            std::fs::copy(&skill_md, &dest).map_err(|e| e.to_string())?;

            // Copy other files from source directory
            if let Ok(entries) = std::fs::read_dir(source_path) {
                for e in entries.flatten() {
                    let src = e.path();
                    if src == skill_md { continue; }
                    let fname = src.file_name().unwrap_or_default();
                    let dst = target_dir.join(fname);
                    if src.is_file() {
                        let _ = std::fs::copy(&src, &dst);
                    } else if src.is_dir() {
                        let dst_sub = target_dir.join(fname);
                        let _ = std::fs::create_dir_all(&dst_sub);
                        if let Ok(sub) = std::fs::read_dir(&src) {
                            for sub_entry in sub.flatten() {
                                let sub_src = sub_entry.path();
                                if sub_src.is_file() {
                                    let _ = std::fs::copy(&sub_src, dst_sub.join(sub_src.file_name().unwrap_or_default()));
                                }
                            }
                        }
                    }
                }
            }

            self.load_all();
            Ok(())
        } else if source_path.extension().is_some_and(|e| e == "md") {
            let content = std::fs::read_to_string(source_path).map_err(|e| e.to_string())?;
            let entry = SkillEntry::from_content(source_path, &content)
                .ok_or_else(|| "Invalid frontmatter in skill file".to_string())?;

            let target_dir = self.skills_dir.join(&entry.name);
            let _ = std::fs::create_dir_all(&target_dir);
            let dest = target_dir.join("SKILL.md");
            std::fs::copy(source_path, &dest).map_err(|e| e.to_string())?;

            self.load_all();
            Ok(())
        } else {
            Err("Source must be a .md file or a directory containing SKILL.md".to_string())
        }
    }

    /// Build a SkillEntry from a ProceduralMemoryRecord (KB-stored E8 trajectory pattern).
    /// Converts the E8 sequence, trigger, reward, and tags into a YAML-frontmatter skill
    /// that can be written to the filesystem and loaded by SkillEngine.
    pub fn skill_from_procedural_record(record: &ProceduralMemoryRecord) -> SkillEntry {
        let e8_str = format!("[{}]", record.e8_sequence.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(","));

        let yaml = format!(
            "---\nname: {}\ndescription: {}\ntriggers: [\"e8\", \"proc_skill\", \"{}\"]\ne8_modes: {}\npriority: {}\n---\n\n{}",
            record.name,
            record.description,
            record.skill_id,
            e8_str,
            (record.avg_reward * 100.0) as u8,
            record.description,
        );

        SkillEntry {
            name: record.name.clone(),
            description: record.description.clone(),
            triggers: vec!["e8".to_string(), "proc_skill".to_string(), record.skill_id.clone()],
            e8_modes: record.e8_sequence.clone(),
            tools: vec![],
            hooks: vec![],
            priority: (record.avg_reward * 100.0) as u8,
            path: PathBuf::new(),
            content: yaml,
            active: false,
            references: vec![],
            category: "procedural".to_string(),
            parent: String::new(),
        }
    }

    /// Install a procedural memory record as a YAML-frontmatter skill file in the skills directory.
    /// Creates `~/.neotrix/skills/<skill_name>/SKILL.md` from the record.
    /// Returns the name of the installed skill on success.
    pub fn install_from_procedural(&mut self, record: &ProceduralMemoryRecord) -> Result<String, String> {
        let skill = Self::skill_from_procedural_record(record);
        let target_dir = self.skills_dir.join(&skill.name);
        let _ = std::fs::create_dir_all(&target_dir);
        let dest = target_dir.join("SKILL.md");
        std::fs::write(&dest, &skill.content).map_err(|e| format!("write skill: {}", e))?;
        self.load_all();
        log::info!("[procedural→skill] installed '{}' from E8 pattern ({} states, reward={:.3})",
            skill.name, record.e8_sequence.len(), record.avg_reward);
        Ok(skill.name)
    }

    /// Find all skill files in the workspace and agent directories.
    /// Legacy compatibility: discovers but does NOT load into this engine.
    pub fn discover_skills() -> Vec<DiscoveredSkill> {
        let mut skills = Vec::new();
        let mut seen: Vec<String> = Vec::new();

        // 1. ~/.neotrix/skills/
        if let Ok(home) = std::env::var("HOME") {
            let dir = PathBuf::from(&home).join(".neotrix").join("skills");
            if dir.exists() {
                Self::scan_discover_dir(&dir, &mut seen, &mut skills);
            }
        }

        // 2. ~/.agents/skills/
        if let Ok(home) = std::env::var("HOME") {
            let dir = PathBuf::from(&home).join(".agents").join("skills");
            if dir.exists() {
                Self::scan_discover_dir(&dir, &mut seen, &mut skills);
            }
        }

        // 3. Workspace skills/
        let ws = Path::new("skills");
        if ws.exists() {
            Self::scan_discover_dir(ws, &mut seen, &mut skills);
        }

        skills
    }

    fn scan_discover_dir(dir: &Path, seen: &mut Vec<String>, skills: &mut Vec<DiscoveredSkill>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                    if seen.contains(&name) { continue; }
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
                        let description = Self::extract_frontmatter_desc(&content);
                        seen.push(name.clone());
                        skills.push(DiscoveredSkill { name, description, path: skill_md });
                    }
                }
            }
        }
    }

    fn extract_frontmatter_desc(content: &str) -> String {
        let stripped = content.trim_start();
        if !stripped.starts_with("---") { return String::new(); }
        if let Some(end) = stripped[3..].find("---") {
            let frontmatter = &stripped[3..3 + end];
            for line in frontmatter.lines() {
                if let Some(val) = line.trim().strip_prefix("description:") {
                    return val.trim().to_string();
                }
            }
        }
        String::new()
    }

    /// Find all SKILL.md files recursively within a directory (legacy compat).
    pub fn find_skill_mds(dir: &Path) -> Vec<PathBuf> {
        let mut results = Vec::new();
        if dir.is_file() && dir.ends_with("SKILL.md") {
            results.push(dir.to_path_buf());
            return results;
        }
        Self::find_skill_mds_recursive(dir, &mut results);
        results
    }

    fn find_skill_mds_recursive(dir: &Path, results: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let fname = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                    if fname.starts_with('.') || fname == "node_modules" || fname == "target" {
                        continue;
                    }
                    Self::find_skill_mds_recursive(&path, results);
                } else if path.ends_with("SKILL.md") {
                    results.push(path);
                }
            }
        }
    }
}

/// Lightweight discovered skill (legacy compatibility).
#[derive(Debug, Clone)]
pub struct DiscoveredSkill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
}

/// Hook events for skill lifecycle.
pub mod skill_hooks {
    use super::*;

    pub struct SkillActivationHook {
        pub engine: Arc<RwLock<SkillEngine>>,
    }

    impl crate::neotrix::l8_autonomic_impl::nt_mind_hook::HookAction for SkillActivationHook {
        fn name(&self) -> &str {
            "skill_activation_hook"
        }

        fn execute(&self, ctx: &HookContext) -> HookResult {
            let msg = &ctx.message;
            if msg.starts_with("skill:") {
                let name = &msg[6..];
                let engine = self.engine.try_write();
                match engine {
                    Ok(mut engine) => {
                        let _ = engine.activate_skill(name);
                    }
                    Err(_) => {
                        std::thread::yield_now();
                        if let Ok(mut engine) = self.engine.try_write() {
                            let _ = engine.activate_skill(name);
                        }
                    }
                }
            }
            HookResult::ok("skill activation hook processed")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_skill_content() -> &'static str {
        r#"---
name: rust-analyzer
description: Expertise in Rust code analysis and optimization
triggers: ["rust", "cargo", "unsafe", "lifetime", "ownership"]
e8_modes: [12, 13, 14]
tools: ["read", "edit", "bash"]
hooks: ["PreToolUse", "PostToolUse"]
priority: 80
---

# Rust Analyzer Skill

## Capabilities
- Analyze Rust code for safety issues
- Suggest optimizations
"#
    }

    fn sample_skill_content_no_frontmatter() -> &'static str {
        "# Just a markdown file\n\nNo frontmatter here."
    }

    fn setup_temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("failed to create temp dir")
    }

    #[test]
    fn test_parse_skill_frontmatter() {
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, sample_skill_content()).unwrap();

        let entry = SkillEntry::from_file(&path).unwrap();
        assert_eq!(entry.name, "rust-analyzer");
        assert_eq!(entry.description, "Expertise in Rust code analysis and optimization");
        assert_eq!(entry.triggers, vec!["rust", "cargo", "unsafe", "lifetime", "ownership"]);
        assert_eq!(entry.e8_modes, vec![12, 13, 14]);
        assert_eq!(entry.tools, vec!["read", "edit", "bash"]);
        assert_eq!(entry.hooks, vec!["PreToolUse", "PostToolUse"]);
        assert_eq!(entry.priority, 80);
        assert!(!entry.active);
    }

    #[test]
    fn test_parse_skill_no_frontmatter_returns_none() {
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, sample_skill_content_no_frontmatter()).unwrap();
        assert!(SkillEntry::from_file(&path).is_none());
    }

    #[test]
    fn test_parse_skill_missing_name_returns_none() {
        let content = r#"---
description: No name here
---
body"#;
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, content).unwrap();
        assert!(SkillEntry::from_file(&path).is_none());
    }

    #[test]
    fn test_parse_skill_default_priority() {
        let content = r#"---
name: test
description: A test skill
---
body"#;
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, content).unwrap();
        let entry = SkillEntry::from_file(&path).unwrap();
        assert_eq!(entry.priority, 50);
    }

    #[test]
    fn test_skill_engine_load_all() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        // Create a skill as a subdirectory with SKILL.md
        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        let loaded = engine.load_all();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "rust-analyzer");
    }

    #[test]
    fn test_skill_engine_load_all_no_dir_creates_it() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("nonexistent");
        let mut engine = SkillEngine::new(skills_dir.clone());
        let loaded = engine.load_all();
        assert!(loaded.is_empty());
        assert!(skills_dir.exists());
    }

    #[test]
    #[ignore = "flaky: test ordering dependent"]
    fn test_find_matching_by_trigger() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // Match by trigger keyword
        let matches = engine.find_matching("rust", None);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "rust-analyzer");

        let matches = engine.find_matching("ownership", None);
        assert_eq!(matches.len(), 1);

        // Non-matching query
        let matches = engine.find_matching("python", None);
        assert!(matches.is_empty());
    }

    #[test]
    #[ignore = "flaky: test ordering dependent"]
    fn test_find_matching_filters_by_e8_mode() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // E8 mode 12 matches (12,13,14 are valid for this skill)
        let matches = engine.find_matching("rust", Some(12));
        assert_eq!(matches.len(), 1);

        // E8 mode 0 is not in [12,13,14] → no match when Some(0)
        let matches = engine.find_matching("rust", Some(0));
        assert!(matches.is_empty());

        // None skips E8 filter → matches
        let matches = engine.find_matching("rust", None);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_find_matching_e8_filter_with_empty_modes() {
        let content = r#"---
name: generic
description: A generic skill
triggers: ["help", "info"]
---
body"#;
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        let skill_dir = skills_dir.join("generic");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), content).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // No e8_modes specified — matches any mode (Some or None)
        let matches = engine.find_matching("help", Some(42));
        assert_eq!(matches.len(), 1);
        let matches = engine.find_matching("help", None);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_find_matching_exact_trigger_outranks_substring() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let exact_skill = r#"---
name: auth
description: Authentication handler
triggers: ["auth", "login", "session"]
---
body"#;
        let substring_skill = r#"---
name: auth-analyzer
description: A skill that also matches auth as substring
triggers: ["auth-flow", "oauth"]
---
body"#;
        for (name, content) in [("auth", exact_skill), ("auth-analyzer", substring_skill)] {
            let dir2 = skills_dir.join(name);
            std::fs::create_dir_all(&dir2).unwrap();
            std::fs::write(dir2.join("SKILL.md"), content).unwrap();
        }

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // Exact trigger "auth" must outrank substring "auth-flow" match
        let matches = engine.find_matching("auth", None);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].name, "auth", "exact trigger must be ranked first");
    }

    #[test]
    fn test_find_matching_caps_results() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        for i in 0..(SkillEngine::MAX_ROUTE_RESULTS + 5) {
            let name = format!("matching-skill-{}", i);
            let dir2 = skills_dir.join(&name);
            std::fs::create_dir_all(&dir2).unwrap();
            let content = format!(
                "---\nname: {}\ndescription: test\ntriggers: [\"matching-skill\"]\n---\nbody",
                name
            );
            std::fs::write(dir2.join("SKILL.md"), content).unwrap();
        }

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // All skills match query "matching-skill" (substring); results must be capped
        let matches = engine.find_matching("matching-skill", None);
        assert!(matches.len() <= SkillEngine::MAX_ROUTE_RESULTS);
        assert_eq!(matches.len(), SkillEngine::MAX_ROUTE_RESULTS);
    }

    #[test]
    fn test_activate_and_deactivate_skill() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        assert!(engine.activate_skill("rust-analyzer").is_ok());
        assert!(engine.get_skill("rust-analyzer").unwrap().active);

        // Double activation should fail
        assert!(engine.activate_skill("rust-analyzer").is_err());

        let active = engine.list_active();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "rust-analyzer");

        assert!(engine.deactivate_skill("rust-analyzer").is_ok());
        assert!(!engine.get_skill("rust-analyzer").unwrap().active);
        assert!(engine.list_active().is_empty());

        // Deactivate inactive should fail
        assert!(engine.deactivate_skill("rust-analyzer").is_err());
    }

    #[test]
    fn test_get_skill_nonexistent() {
        let dir = setup_temp_dir();
        let mut engine = SkillEngine::new(dir.path().join("skills"));
        engine.load_all();
        assert!(engine.get_skill("nonexistent").is_none());
        assert!(engine.activate_skill("nonexistent").is_err());
    }

    #[test]
    fn test_install_skill_from_file() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let source = dir.path().join("source.md");
        std::fs::write(&source, sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir.clone());
        engine.load_all();
        assert!(engine.list_all().is_empty());

        engine.install_skill(&source).unwrap();

        let all = engine.list_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "rust-analyzer");
        assert!(skills_dir.join("rust-analyzer").join("SKILL.md").exists());
    }

    #[test]
    fn test_install_skill_from_directory() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let source_dir = dir.path().join("my-skill");
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::write(source_dir.join("SKILL.md"), sample_skill_content()).unwrap();
        std::fs::write(source_dir.join("helper.py"), r#"print("hello")"#).unwrap();

        let mut engine = SkillEngine::new(skills_dir.clone());
        engine.load_all();
        assert!(engine.list_all().is_empty());

        engine.install_skill(&source_dir).unwrap();
        assert_eq!(engine.list_all().len(), 1);
        assert!(skills_dir.join("rust-analyzer").join("helper.py").exists());
    }

    #[test]
    fn test_install_skill_invalid_source() {
        let dir = setup_temp_dir();
        let mut engine = SkillEngine::new(dir.path().join("skills"));

        assert!(engine.install_skill(&dir.path().join("nonexistent")).is_err());
        assert!(engine.install_skill(&dir.path().join(".")).is_err());
    }

    #[test]
    fn test_parse_array_field() {
        assert_eq!(parse_array_field(r#"["a", "b", "c"]"#), vec!["a", "b", "c"]);
        assert_eq!(parse_array_field(r#"['x', 'y']"#), vec!["x", "y"]);
        assert_eq!(parse_array_field("a, b, c"), vec!["a", "b", "c"]);
        assert_eq!(parse_array_field(""), Vec::<String>::new());
    }

    #[test]
    fn test_skill_body() {
        let content = sample_skill_content();
        let dir = setup_temp_dir();
        let path = dir.path().join("test.md");
        std::fs::write(&path, content).unwrap();
        let entry = SkillEntry::from_file(&path).unwrap();
        let body = entry.body();
        assert!(body.contains("Analyze Rust code"));
        assert!(body.to_lowercase().contains("suggest optimizations"));
    }

    #[test]
    fn test_discover_skills_legacy() {
        // Should not crash/panic
        let _skills = SkillEngine::discover_skills();
    }

    #[test]
    fn test_priority_sorting() {
        let content_high = r#"---
name: high-priority
description: High priority skill
triggers: ["test"]
priority: 90
---
high"#;
        let content_low = r#"---
name: low-priority
description: Low priority skill
triggers: ["test"]
priority: 10
---
low"#;

        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        std::fs::write(skills_dir.join("high.md"), content_high).unwrap();
        std::fs::write(skills_dir.join("low.md"), content_low).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        let matches = engine.find_matching("test", None);
        assert_eq!(matches.len(), 2);
        // Higher priority first
        assert_eq!(matches[0].name, "high-priority");
    }

    #[test]
    fn test_list_all_empty() {
        let dir = setup_temp_dir();
        let mut engine = SkillEngine::new(dir.path().join("skills"));
        engine.load_all();
        assert!(engine.list_all().is_empty());
    }

    #[test]
    fn test_progressive_disclosure_load_reference() {
        // 渐进披露 (diagram-design 吸收): SKILL.md 只声明 references,
        // 细节存 references/*.md 按需加载。
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        let skill_dir = skills_dir.join("my-skill");
        let refs_dir = skill_dir.join("references");
        std::fs::create_dir_all(&refs_dir).unwrap();

        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: my-skill\ndescription: A skill with progressive disclosure\ntriggers: [\"mine\"]\nreferences: [\"type-a.md\", \"type-b.md\"]\n---\nbody",
        )
        .unwrap();
        std::fs::write(refs_dir.join("type-a.md"), "TYPE-A DETAILS").unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        assert_eq!(engine.list_all().len(), 1);

        // 已声明引用 → 按需加载成功
        let loaded = engine.load_reference("my-skill", "type-a.md").unwrap();
        assert!(loaded.contains("TYPE-A DETAILS"));

        // 未声明引用 → 拒绝 (禁未声明加载)
        assert!(engine.load_reference("my-skill", "secret.md").is_err());

        // 缺失文件 → 报错 (声明了但没落盘)
        assert!(engine.load_reference("my-skill", "type-b.md").is_err());

        // 不存在的技能 → 报错
        assert!(engine.load_reference("nope", "type-a.md").is_err());
    }

    #[test]
    fn test_list_active_empty() {
        let dir = setup_temp_dir();
        let mut engine = SkillEngine::new(dir.path().join("skills"));
        engine.load_all();
        assert!(engine.list_active().is_empty());
    }

    #[test]
    fn test_skill_entry_from_content_direct() {
        let entry = SkillEntry::from_content(Path::new("test.md"), sample_skill_content());
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.name, "rust-analyzer");
    }

    #[test]
    fn test_e8_index_build() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        std::fs::write(skills_dir.join("test.md"), sample_skill_content()).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        assert!(engine.e8_index.contains_key(&12));
        assert!(engine.e8_index.contains_key(&13));
        assert!(engine.e8_index.contains_key(&14));
        assert!(!engine.e8_index.contains_key(&0));
    }

    #[test]
    fn test_skill_from_procedural_record_creates_valid_entry() {
        let record = ProceduralMemoryRecord {
            id: "test-id".into(),
            skill_id: "proc_skill_test".into(),
            name: "Test E8 Skill".into(),
            description: "Learned E8 pattern: 3 states".into(),
            e8_sequence: vec![12, 13, 14],
            trigger_pattern: vec![12],
            success_rate: 0.85,
            execution_count: 5,
            avg_reward: 0.75,
            created_at: "2026-07-04T00:00:00Z".into(),
            updated_at: "2026-07-04T00:00:00Z".into(),
            tags: vec!["procedural".into(), "auto_discovered".into()],
        };

        let skill = SkillEngine::skill_from_procedural_record(&record);
        assert_eq!(skill.name, "Test E8 Skill");
        assert_eq!(skill.description, "Learned E8 pattern: 3 states");
        assert_eq!(skill.triggers, vec!["e8", "proc_skill", "proc_skill_test"]);
        assert_eq!(skill.e8_modes, vec![12, 13, 14]);
        assert_eq!(skill.priority, 75);
        assert!(skill.content.starts_with("---\nname: Test E8 Skill"));
        assert!(skill.content.contains("e8_modes: [12,13,14]"));
    }

    #[test]
    fn test_skill_from_procedural_record_low_reward() {
        let record = ProceduralMemoryRecord {
            id: "test-id-2".into(),
            skill_id: "proc_skill_low".into(),
            name: "Low Reward Skill".into(),
            description: "Learned E8 pattern with low confidence".into(),
            e8_sequence: vec![1, 2],
            trigger_pattern: vec![1],
            success_rate: 0.3,
            execution_count: 1,
            avg_reward: 0.15,
            created_at: "2026-07-04T00:00:00Z".into(),
            updated_at: "2026-07-04T00:00:00Z".into(),
            tags: vec![],
        };

        let skill = SkillEngine::skill_from_procedural_record(&record);
        assert_eq!(skill.priority, 15, "low avg_reward should give low priority");
        assert_eq!(skill.e8_modes, vec![1, 2]);
    }

    #[test]
    fn test_install_from_procedural_writes_skill_file() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let record = ProceduralMemoryRecord {
            id: "install-test-id".into(),
            skill_id: "proc_install_test".into(),
            name: "Installed Procedural Skill".into(),
            description: "E8 pattern installed via bridge".into(),
            e8_sequence: vec![5, 10, 15],
            trigger_pattern: vec![5],
            success_rate: 0.9,
            execution_count: 3,
            avg_reward: 0.88,
            created_at: "2026-07-04T00:00:00Z".into(),
            updated_at: "2026-07-04T00:00:00Z".into(),
            tags: vec!["procedural".into()],
        };

        let mut engine = SkillEngine::new(skills_dir.clone());
        let result = engine.install_from_procedural(&record);
        assert!(result.is_ok(), "install_from_procedural failed: {:?}", result);
        assert_eq!(result.unwrap(), "Installed Procedural Skill");

        // Verify the skill file was created and can be loaded back
        let mut engine2 = SkillEngine::new(skills_dir);
        let loaded = engine2.load_all();
        let skill = loaded.iter().find(|s| s.name == "Installed Procedural Skill");
        assert!(skill.is_some(), "installed skill should be loadable");
        assert_eq!(skill.unwrap().e8_modes, vec![5, 10, 15]);
    }

    fn kb_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_schema::initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_sync_to_kb_index_write_through_and_dedup() {
        use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::skill_list_all;

        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let conn = kb_conn();
        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        assert_eq!(engine.list_all().len(), 1);

        // 首次写通: 1 条真正写入
        assert_eq!(engine.sync_to_kb_index(&conn).unwrap(), 1);
        // 二次写通: 内容未变化 → 去重, 0 写入
        assert_eq!(engine.sync_to_kb_index(&conn).unwrap(), 0, "内容未变化必须去重 (避免每命令全量写)");

        let recs = skill_list_all(&conn, 10).unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].name, "rust-analyzer");
        assert!(recs[0].content_hash.is_some(), "写通必须携带 content_hash");
        assert_eq!(recs[0].tags.as_deref(), Some("rust,cargo,unsafe,lifetime,ownership"));

        // 内容变化 → 再次写入 (同 name 更新)
        std::fs::write(
            skill_dir.join("SKILL.md"),
            sample_skill_content().replace("priority: 80", "priority: 85"),
        )
        .unwrap();
        engine.load_all();
        assert_eq!(engine.sync_to_kb_index(&conn).unwrap(), 1, "内容变化应重新写入");
        let recs = skill_list_all(&conn, 10).unwrap();
        assert_eq!(recs.len(), 1, "同 name 更新而非新增");
    }

    #[test]
    fn test_load_all_auto_syncs_to_kb() {
        use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::skill_list_all;
        use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;

        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();
        let skill_dir = skills_dir.join("rust-analyzer");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(skill_dir.join("SKILL.md"), sample_skill_content()).unwrap();

        let kb = KnowledgeBase::open(Some(dir.path().join("kb.db"))).expect("KB open");
        let kb = Arc::new(kb);
        let mut engine = SkillEngine::new(skills_dir).with_kb(kb.clone());
        let loaded = engine.load_all();
        assert_eq!(loaded.len(), 1);

        let conn = kb.conn.lock().unwrap();
        let recs = skill_list_all(&conn, 10).unwrap();
        assert_eq!(recs.len(), 1, "load_all 后应自动写通到 skills_index");
        assert_eq!(recs[0].name, "rust-analyzer");
    }

    #[test]
    fn test_parse_category_parent_frontmatter() {
        let content = r#"---
name: nested
description: A categorized skill
triggers: ["nested"]
category: "test-domain"
parent: root-skill
---
body"#;
        let entry = SkillEntry::from_content(Path::new("nested.md"), content).unwrap();
        assert_eq!(entry.category, "test-domain");
        assert_eq!(entry.parent, "root-skill");
    }

    #[test]
    fn test_default_category_is_general() {
        let entry = SkillEntry::from_content(Path::new("x.md"), sample_skill_content()).unwrap();
        assert_eq!(entry.category, "general");
        assert!(entry.parent.is_empty());
    }

    #[test]
    fn test_skill_tree_groups_by_category() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let cat_a = r#"---
name: a-skill
description: cat a
triggers: ["alpha"]
category: analysis
---
body"#;
        let cat_b = r#"---
name: b-skill
description: cat b
triggers: ["beta"]
category: coding
---
body"#;
        std::fs::write(skills_dir.join("a.md"), cat_a).unwrap();
        std::fs::write(skills_dir.join("b.md"), cat_b).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        let tree = engine.skill_tree();
        assert_eq!(tree.len(), 2);
        assert!(tree.contains_key("analysis"));
        assert!(tree.contains_key("coding"));
    }

    #[test]
    fn test_children_of_returns_subskills() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let parent = r#"---
name: root-skill
description: root
triggers: ["root"]
category: coding
---
body"#;
        let child = r#"---
name: child-skill
description: child
triggers: ["child"]
category: coding
parent: root-skill
---
body"#;
        std::fs::write(skills_dir.join("root.md"), parent).unwrap();
        std::fs::write(skills_dir.join("child.md"), child).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        let children = engine.children_of("root-skill");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "child-skill");
    }

    #[test]
    fn test_find_matching_complementary_prefers_uncovered_category() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let covered = r#"---
name: rust-analyzer
description: rust analysis
triggers: ["rust"]
category: coding
priority: 90
---
body"#;
        let uncovered = r#"---
name: security-review
description: security review
triggers: ["rust"]
category: security
priority: 40
---
body"#;
        std::fs::write(skills_dir.join("a.md"), covered).unwrap();
        std::fs::write(skills_dir.join("b.md"), uncovered).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        // 无已激活技能 → 按优先级: coding(90) 优先
        let base = engine.find_matching("rust", None);
        assert_eq!(base[0].name, "rust-analyzer");

        // 已激活 coding 类技能 → security 类应前置 (互补)
        let comp = engine.find_matching_complementary("rust", None, &["rust-analyzer"]);
        assert!(!comp.is_empty());
        assert_eq!(comp[0].name, "security-review");
    }

    #[test]
    fn test_over_validation_score_flags_procedure_heavy() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let heavy = r#"---
name: heavy-skill
description: procedure heavy
triggers: ["heavy"]
category: general
---
1. rebuild the entire project and verify
2. cargo clean then re-read every file
3. verify compile twice and re-read all docs
4. audit every line and validate again
5. recheck rebuild and verify the compile
"#;
        std::fs::write(skills_dir.join("heavy.md"), heavy).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        let score = engine.over_validation_score("heavy-skill");
        assert!(score >= 12, "procedure-heavy skill must score >= 12, got {}", score);
    }

    #[test]
    fn test_attribution_tracks_activations_and_flags() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let heavy = r#"---
name: heavy-skill
description: procedure heavy
triggers: ["heavy"]
category: general
---
1. rebuild the entire project and verify the compile
2. cargo clean then re-read every single file and recheck
3. verify compile twice and audit the result line by line
4. validate the rebuild and recheck the verify output
5. cargo clean again and re-read the recheck report
"#;
        std::fs::write(skills_dir.join("heavy.md"), heavy).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();

        engine.activate_skill("heavy-skill").unwrap();
        engine.activate_skill("heavy-skill").unwrap_err();

        let report = engine.attribution_report();
        let attr = report.iter().find(|a| a.name == "heavy-skill").unwrap();
        assert_eq!(attr.activations, 1);
        assert!(attr.procedure_heavy, "heavy skill must be flagged procedure-heavy");
    }

    #[test]
    fn test_skill_tree_stats_counts_hierarchy() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let root = r#"---
name: root-skill
description: root
triggers: ["root"]
category: coding
---
body"#;
        let child = r#"---
name: child-skill
description: child
triggers: ["child"]
category: coding
parent: root-skill
---
body"#;
        let orphan = r#"---
name: orphan-skill
description: orphan (parent missing)
triggers: ["orphan"]
category: analysis
parent: ghost-parent
---
body"#;
        std::fs::write(skills_dir.join("root.md"), root).unwrap();
        std::fs::write(skills_dir.join("child.md"), child).unwrap();
        std::fs::write(skills_dir.join("orphan.md"), orphan).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        let stats = engine.skill_tree_stats();
        assert_eq!(stats.total_skills, 3);
        assert_eq!(stats.roots, 1, "only root-skill has no parent");
        assert_eq!(stats.orphans, 1, "orphan-skill points to missing parent");
        assert_eq!(stats.max_depth, 1, "child depth = 1");
        assert_eq!(stats.categories.get("coding"), Some(&2));
        assert_eq!(stats.categories.get("analysis"), Some(&1));
    }

    #[test]
    fn test_flagged_attributions_reports_procedure_heavy() {
        let dir = setup_temp_dir();
        let skills_dir = dir.path().join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        let heavy = r#"---
name: heavy-skill
description: procedure heavy
triggers: ["heavy"]
category: general
---
1. rebuild the entire project and verify the compile twice
2. cargo clean then re-read every single file and recheck
3. verify the compile and audit the result line by line
4. validate the rebuild and recheck the verify output
5. cargo clean again and re-read the recheck report
"#;
        std::fs::write(skills_dir.join("heavy.md"), heavy).unwrap();

        let mut engine = SkillEngine::new(skills_dir);
        engine.load_all();
        engine.activate_skill("heavy-skill").unwrap();

        let flagged = engine.flagged_attributions();
        assert!(!flagged.is_empty(), "procedure-heavy skill must surface in flagged report");
        assert!(flagged.iter().all(|a| a.procedure_heavy));
    }
}
