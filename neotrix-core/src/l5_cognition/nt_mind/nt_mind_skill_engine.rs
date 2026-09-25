//! Skill auto-invocation engine — scans, parses, indexes, and auto-invokes
//! skill markdown files with YAML frontmatter.
//!
//! Integrates with:
//!   - nt_mind_hook: fires SkillLoaded/SkillUnloaded HookEvents
//!   - GWT workspace: broadcasts skill activation events

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;

use crate::l5_cognition::nt_core_gwt::workspace::GlobalWorkspace;
use crate::l5_cognition::l1_facade::KnowledgeBase;
use crate::l5_cognition::nt_mind::nt_mind_hook::{MindHookRegistry, HookContext, HookResult};

/// Core skill engine: scan, index, match, activate/deactivate.
pub struct SkillEngine {
    skills_dir: PathBuf,
    skills: Vec<SkillDocEntry>,
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
    /// 锚定-然后-promote (dsh-anchored-standard 吸收): session 首个请求锚定
    /// Minimal 工具集, durable 后提升到 Standard 工具集。
    pub disclosure: AnchorPromote,
    /// 可逆效应逆账本 (cordiverse F1 吸收): `install_id` → 加载序逆操作。
    /// uninstall 以 LIFO 派生 teardown, 非手写清理。
    pub inverse_ledger: InverseLedger,
    /// 技能 fiber 生命周期注册表 (cordiverse F5 吸收): skill 名 → fiber 状态机。
    pub fiber_lifecycles: HashMap<String, FiberLifecycle>,
    /// A5 五维质量门 (SkillNet absorb, R-P79): load_all 时对每个技能跑质量评分,
    /// 记录 "拒绝低质量技能" 统计, 生产检索路径可按需查询。
    pub quality_stats: std::collections::HashMap<String, SkillQualityScores>,
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
            disclosure: AnchorPromote::default(),
            inverse_ledger: InverseLedger::new(),
            fiber_lifecycles: HashMap::new(),
            quality_stats: std::collections::HashMap::new(),
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

    /// Find skills matching a query string and optional E8 mode.
    /// When `e8_mode` is `None`, the E8 mode filter is skipped.
    /// Matching is case-insensitive keyword match against triggers.
    /// Results are sorted by priority descending, then by trigger relevance.
    /// 反哺自 spec-kit/autoroute 吸收: 确定性优先级栈 (exact > substring) + 硬结果上限
    /// (open-code-review 预算纪律) — 防止路由返回无界候选淹没下游消费方。
    pub const MAX_ROUTE_RESULTS: usize = 8;

    pub fn find_matching(&self, query: &str, e8_mode: Option<u8>) -> Vec<&SkillDocEntry> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<String> = query_lower.split_whitespace()
            .map(|s| s.to_string())
            .chain(std::iter::once(query_lower.clone()))
            .collect();

        // tier 0 = exact trigger equality (最高优先级, 确定性命中)
        // tier 1 = substring 命中
        let mut exact: Vec<(usize, usize, &SkillDocEntry)> = Vec::new();
        let mut scored: Vec<(usize, usize, &SkillDocEntry)> = Vec::new();

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
    pub fn skill_tree(&self) -> HashMap<String, Vec<&SkillDocEntry>> {
        let mut tree: HashMap<String, Vec<&SkillDocEntry>> = HashMap::new();
        for s in self.skills.iter() {
            tree.entry(s.category.clone()).or_default().push(s);
        }
        for v in tree.values_mut() {
            v.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
        tree
    }

    pub fn children_of(&self, name: &str) -> Vec<&SkillDocEntry> {
        self.skills.iter().filter(|s| s.parent == name).collect()
    }

    /// 互补性感知检索 (AgentSkillOS 吸收): 在 find_matching 候选基础上, 对
    /// 与已激活技能同 category 的候选施加降级, 优先返回未覆盖类别 (多样化)。
    pub fn find_matching_complementary(
        &self,
        query: &str,
        e8_mode: Option<u8>,
        active_names: &[&str],
    ) -> Vec<&SkillDocEntry> {
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
            success_count: 0,
            last_used_at: chrono::Utc::now().timestamp(),
        });
        attr.activations += 1;
        attr.over_validation_score = score;
        attr.procedure_heavy = procedure_heavy;
        attr.flagged = procedure_heavy && attr.activations > 1;
        attr.last_used_at = chrono::Utc::now().timestamp();
    }

    /// Phase 4 (库治理): 语义去重。复用 `SkillComposer::compose` 判定 `Substitute`
    /// (同类别 + 工具重叠≥0.75 + 触发低重叠) 的技能对, 保留 priority 高、分值高的,
    /// 移除冗余副本, 防止技能库膨胀 (Dark Forest: 连接而非堆积)。
    pub fn prune_semantic_duplicates(&mut self) {
        let n = self.skills.len();
        let mut keep = vec![true; n];
        for i in 0..n {
            if !keep[i] {
                continue;
            }
            for j in (i + 1)..n {
                if !keep[j] {
                    continue;
                }
                let rel = SkillComposer::compose(&self.skills[i], &self.skills[j]);
                if !matches!(rel, SkillRelationship::Substitute) {
                    continue;
                }
                // 保留 priority 高者; priority 相同则比较 quality.overall()
                let worse = if self.skills[i].priority != self.skills[j].priority {
                    if self.skills[i].priority > self.skills[j].priority {
                        j
                    } else {
                        i
                    }
                } else {
                    let qi = self
                        .quality_stats
                        .get(&self.skills[i].name)
                        .map(|s| s.overall())
                        .unwrap_or(0.0);
                    let qj = self
                        .quality_stats
                        .get(&self.skills[j].name)
                        .map(|s| s.overall())
                        .unwrap_or(0.0);
                    if qi >= qj {
                        j
                    } else {
                        i
                    }
                };
                keep[worse] = false;
            }
        }
        if keep.iter().any(|k| !k) {
            let mut kept = Vec::with_capacity(keep.iter().filter(|k| **k).count());
            for (idx, k) in keep.iter().enumerate() {
                if *k {
                    kept.push(self.skills[idx].clone());
                }
            }
            self.skills = kept;
            self.build_index();
        }
    }

    /// Phase 4 (库治理): 回收零调用技能。仅当该技能在当前会话/已知归因中 `activations==0`
    /// 时移除 (Dark Forest: 不连接即无存在意义)。`protected` 名集合永不被回收。
    pub fn retire_zero_call(&mut self, protected: &std::collections::HashSet<String>) {
        let before = self.skills.len();
        self.skills.retain(|s| {
            if protected.contains(&s.name) {
                return true;
            }
            match self.attribution.get(&s.name) {
                Some(a) => a.activations > 0,
                None => {
                    // 从未记录过激活: 视为零调用, 回收 (除非被保护)
                    false
                }
            }
        });
        if self.skills.len() != before {
            self.build_index();
        }
    }

    /// Phase 4 (self-benchmark 校准): 依据真实调用证据重新平衡技能库。
    /// - activations==0 → 降优先级 (priority 降 1, 不低于 1);
    /// - success_rate<0.5 且 activations>=3 → 标记 flagged (降级候选)。
    pub fn rebalance(&mut self) {
        for skill in self.skills.iter_mut() {
            let Some(a) = self.attribution.get_mut(&skill.name) else {
                continue;
            };
            if a.activations == 0 {
                skill.priority = skill.priority.saturating_sub(1).max(1);
                continue;
            }
            let success_rate = a.success_count as f64 / a.activations as f64;
            if success_rate < 0.5 && a.activations >= 3 {
                a.flagged = true;
            }
        }
    }

    /// Phase 4 维护入口 (T3): 周期性调用 — 去重 + 真实调用证据驱动的回收与校准。
    /// 不放在 `load_all` 内, 避免每次加载即把无归因的新技能当零调用清掉。
    pub fn maintain(&mut self) -> usize {
        let before = self.skills.len();
        self.prune_semantic_duplicates();
        self.retire_zero_call(&std::collections::HashSet::new());
        self.rebalance();
        before.saturating_sub(self.skills.len())
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
                        success_count: 0,
                        last_used_at: 0,
                    })
            })
            .collect();
        report.sort_by(|a, b| b.activations.cmp(&a.activations));
        report
    }

    pub fn get_skill(&self, name: &str) -> Option<&SkillDocEntry> {
        self.skills.iter().find(|s| s.name == name)
    }

    pub(crate) fn _get_skill_mut(&mut self, name: &str) -> Option<&mut SkillDocEntry> {
        self.skills.iter_mut().find(|s| s.name == name)
    }

    /// 技能树层级统计 (G6, AgentSkillOS 吸收): category 分布、根/叶/孤儿技能、
    /// 覆盖率与深度。供背景循环巡检报告使用。
    pub fn skill_tree_stats(&self) -> _SkillTreeStats {
        let mut categories: HashMap<String, usize> = HashMap::new();
        let mut roots = 0usize;
        let mut orphans = 0usize;
        
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
                    if depth.insert(s.name.clone(), 0).is_none_or(|old| old != 0) {
                        changed = true;
                    }
                } else {
                    if let Some(pd) = depth.get(&s.parent).copied() {
                        let d = pd + 1;
                        if depth.insert(s.name.clone(), d).is_none_or(|old| old != d) {
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        let max_depth = depth.values().copied().max().unwrap_or(0);
        let known: std::collections::HashSet<&String> =
            self.skills.iter().map(|s| &s.name).collect();
        for s in &self.skills {
            if !s.parent.is_empty() && !known.contains(&s.parent) {
                orphans += 1;
            }
        }
        _SkillTreeStats {
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

    /// P4 技能驻留成本审计 (asm absorbed 2026-08-19, R-P79): 从 load_all
    /// 收集的 quality_stats 派生降级候选排名 (resident 降序)。消费者:
    /// background-loop `handle_skill_scan` — LAZY LOAD 下 resident 最肥的
    /// 技能应优先降级为渐进披露薄入口。
    pub fn audit_residency(&self) -> Vec<ResidencyAuditRow> {
        crate::l5_cognition::nt_mind::nt_mind_skill_engine::audit_residency(&self.quality_stats)
    }

    pub fn list_all(&self) -> Vec<&SkillDocEntry> {
        self.skills.iter().collect()
    }

    pub fn skills_dir(&self) -> &Path {
        &self.skills_dir
    }
}

pub mod skill_doc;
pub use skill_doc::*;
pub mod skill_quality;
pub use skill_quality::*;
pub mod book_to_skill;
pub use book_to_skill::*;
pub mod skill_attribution;
pub use skill_attribution::*;
pub mod prompt_library;
pub mod skill_compose;
pub use skill_compose::*;
pub mod skill_revert;
pub use skill_revert::*;
pub mod skill_scan;
pub mod skill_activate;
pub use prompt_library::*;
pub mod skill_hooks {
    use super::*;

    pub(crate) struct _SkillActivationHook {
        pub engine: Arc<RwLock<SkillEngine>>,
    }

    impl crate::l5_cognition::nt_mind::nt_mind_hook::HookAction for _SkillActivationHook {
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

    /// 星辰唤醒钩子 (CSGN Galaxy wake, T3 生产接线): `SkillLoaded` 事件 →
    /// `galaxy_wake_star` 落盘星辰活跃度。技能名 `-`→`_` 映射 namespace,
    /// 无星辰身份的技能静默跳过 (如 experience-tree 加密 hub)。
    pub struct CsgnWakeHook {
        pub kb: Option<Arc<KnowledgeBase>>,
    }

    impl crate::l5_cognition::nt_mind::nt_mind_hook::HookAction for CsgnWakeHook {
        fn name(&self) -> &str {
            "csgn_wake_hook"
        }

        fn execute(&self, ctx: &HookContext) -> HookResult {
            let msg = &ctx.message;
            let name = msg.strip_prefix("skill:").unwrap_or(msg);
            if name.is_empty() {
                return HookResult::ok("csgn_wake: empty skill name");
            }
            let ns = if name == "experience-tree" {
                "experience".to_string()
            } else {
                name.replace('-', "_")
            };
            match &self.kb {
                Some(kb) => match kb.galaxy_wake_star(&ns) {
                    Ok(msg) => HookResult::ok(&msg).with_effect("galaxy_wake"),
                    Err(e) => {
                        // 非星辰技能无 hub → 静默跳过, 不报错不阻塞
                        if e.contains("不存在 hub") || e.contains("非星辰") {
                            HookResult::ok("csgn_wake: skip (no star hub)")
                        } else {
                            HookResult::err(&e)
                        }
                    }
                },
                None => HookResult::ok("csgn_wake: no KB attached"),
            }
        }
    }
}

pub mod true_replay;
pub mod skill_retrieval;

#[cfg(test)]
mod tests;
