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
use crate::l5_cognition::l1_facade::{ProceduralMemoryRecord, skill_upsert, SkillRecord, KnowledgeBase};
use crate::l5_cognition::nt_mind::nt_mind_hook::{HookEvent, MindHookRegistry, HookContext, HookResult};

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

    /// Scan the skills directory and load all valid skill files.
    pub fn load_all(&mut self) -> Vec<SkillDocEntry> {
        self.skills.clear();
        self.trigger_index.clear();
        self.e8_index.clear();

        let dir = &self.skills_dir;
        if !dir.exists() {
            let _ = std::fs::create_dir_all(dir);
            return Vec::new();
        }

        let mut loaded = Vec::new();
        self.quality_stats.clear();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Some(skill) = SkillDocEntry::from_file(&skill_md) {
                            let scores = SkillQualityScorer::evaluate(&skill);
                            // P6 SkillTrustBench 安全门 (Tencent AIG absorbed, R-P79):
                            // 静态 T01-T09 扫描 — 命中任一攻击分类即拒收, 不进入生产索引。
                            let (trust_findings, trust_verdict) =
                                crate::l3_embodiment::nt_shield::shield_core::tool_inspection_stack::scan_skill_content(&skill.content);
                            let trust_rejected = !matches!(trust_verdict, crate::l3_embodiment::nt_shield::shield_core::tool_inspection_stack::InspectionResult::Allow);
                            // E6 防护层硬化 (src9 EVOMAL 毒化扫描): 折入 R-P108
                            // 五维门 — 命中毒化模式即拒收, 阻断 promote。Err 保守视为拒收。
                            let poison_ok = evomal_poison_scan(&skill).unwrap_or(false);
                            // A5 安全门 (SkillNet absorb, R-P79): 含危险命令
                            // (rm -rf 等) 的技能拒收, 不进入生产检索索引。
                            if scores.safety >= 0.8 && !trust_rejected && poison_ok {
                                self.quality_stats.insert(skill.name.clone(), scores);
                                loaded.push(skill);
                            } else if trust_rejected {
                                log::warn!(
                                    "SkillTrustBench 拒收技能 `{}` ({} 命中): {}",
                                    skill.name,
                                    trust_findings.len(),
                                    trust_findings.first().map(|f| f.id).unwrap_or("?")
                                );
                            } else if !poison_ok {
                                log::warn!(
                                    "EVOMAL 毒化扫描拒收技能 `{}` (src9 模式命中)",
                                    skill.name
                                );
                            }
                        }
                    }
                    continue;
                }
                if path.extension().is_some_and(|e| e == "md") {
                    if let Some(skill) = SkillDocEntry::from_file(&path) {
                        let scores = SkillQualityScorer::evaluate(&skill);
                        // P6 SkillTrustBench 安全门 (同目录型技能, R-P79)。
                        let (trust_findings, trust_verdict) =
                            crate::l3_embodiment::nt_shield::shield_core::tool_inspection_stack::scan_skill_content(&skill.content);
                        let trust_rejected = !matches!(trust_verdict, crate::l3_embodiment::nt_shield::shield_core::tool_inspection_stack::InspectionResult::Allow);
                        // E6 防护层硬化 (src9 EVOMAL 毒化扫描): 折入 R-P108
                        // 五维门 — 命中毒化模式即拒收, 阻断 promote。Err 保守视为拒收。
                        let poison_ok = evomal_poison_scan(&skill).unwrap_or(false);
                        if scores.safety >= 0.8 && !trust_rejected && poison_ok {
                            self.quality_stats.insert(skill.name.clone(), scores);
                            loaded.push(skill);
                        } else if trust_rejected {
                            log::warn!(
                                "SkillTrustBench 拒收技能 `{}` ({} 命中): {}",
                                skill.name,
                                trust_findings.len(),
                                trust_findings.first().map(|f| f.id).unwrap_or("?")
                            );
                        } else if !poison_ok {
                            log::warn!(
                                "EVOMAL 毒化扫描拒收技能 `{}` (src9 模式命中)",
                                skill.name
                            );
                        }
                    }
                }
            }
        }

        self.skills = loaded;
        self.build_index();
        // Phase 4 库治理: load 时自动去重 (T3 生产接线, Dark Forest; retire/rebalance 见 maintain())
        self.prune_semantic_duplicates();
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
        use crate::l5_cognition::l1_facade::skill_content_hash;
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
                last_indexed_at: Some(crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_unify::now()),
                created_at: crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_unify::now(),
                updated_at: crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_unify::now(),
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

    /// 推进渐进披露阶梯 (P4, dsh-anchored-standard 吸收): 若 session 已
    /// durable (首个 durable 工具/调用), 从 Minimal 提升到 Standard 工具集。
    /// 返回阶段是否发生变化。
    pub fn step_disclosure(&mut self) -> bool {
        self.disclosure.maybe_promote()
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

    pub fn list_active(&self) -> Vec<&SkillDocEntry> {
        self.skills.iter().filter(|s| s.active).collect()
    }

    /// 披露门控的活跃技能视图 (P4 行为接线): 披露预算 active_tool_count()
    /// 真实限制模型可见工具集 — stage 0 (Minimal) 时仅暴露预算数量的
    /// 高优先级技能, promote 到 Standard 后暴露全部活跃技能。
    /// 这是 active_tool_count() 从"展示"到"行为门控"的生产路径。
    pub fn visible_active(&self) -> Vec<&SkillDocEntry> {
        let mut active: Vec<&SkillDocEntry> = self.skills.iter().filter(|s| s.active).collect();
        let budget = self.disclosure.active_tool_count();
        if self.disclosure.stage == 0 && active.len() > budget {
            // Minimal 阶段: 按 priority 升序 (高优先级在前) 截断到预算
            active.sort_by_key(|s| s.priority);
            active.truncate(budget);
        }
        active
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
            let entry = SkillDocEntry::from_content(&skill_md, &content)
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
            self.register_install_effects(&entry.name, &target_dir)?;
            Ok(())
        } else if source_path.extension().is_some_and(|e| e == "md") {
            let content = std::fs::read_to_string(source_path).map_err(|e| e.to_string())?;
            let entry = SkillDocEntry::from_content(source_path, &content)
                .ok_or_else(|| "Invalid frontmatter in skill file".to_string())?;

            let target_dir = self.skills_dir.join(&entry.name);
            let _ = std::fs::create_dir_all(&target_dir);
            let dest = target_dir.join("SKILL.md");
            std::fs::copy(source_path, &dest).map_err(|e| e.to_string())?;

            self.load_all();
            self.register_install_effects(&entry.name, &target_dir)?;
            Ok(())
        } else {
            Err("Source must be a .md file or a directory containing SKILL.md".to_string())
        }
    }

    /// Build a SkillDocEntry from a ProceduralMemoryRecord (KB-stored E8 trajectory pattern).
    /// Converts the E8 sequence, trigger, reward, and tags into a YAML-frontmatter skill
    /// that can be written to the filesystem and loaded by SkillEngine.
    pub fn skill_from_procedural_record(record: &ProceduralMemoryRecord) -> SkillDocEntry {
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

        SkillDocEntry {
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
            verified: false,
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

    /// 把 install 的逆操作推入逆账本并派生 skill fiber (cordiverse F1+F5)。
    /// teardown 从加载序派生, 非手写清理 (paper §3.3.3 p.27)。
    fn register_install_effects(&mut self, name: &str, target_dir: &Path) -> Result<(), String> {
        let install_id = self.inverse_ledger.begin_install();
        let inv_target = target_dir.to_path_buf();
        let inv_label = format!("remove installed skill dir: {}", inv_target.display());
        self.inverse_ledger.push_inverse(
            install_id,
            RevertibleEffect::new(inv_label, move || {
                if inv_target.exists() {
                    std::fs::remove_dir_all(&inv_target)
                        .map_err(|e| format!("remove {}: {}", inv_target.display(), e))
                } else {
                    Ok(())
                }
            }),
        )?;
        self.fiber_lifecycles.insert(
            name.to_string(),
            FiberLifecycle::new(name.to_string(), install_id),
        );
        if let Some(fiber) = self.fiber_lifecycles.get_mut(name) {
            let _ = fiber.transition(FiberLifecycleState::Active);
        }
        Ok(())
    }

    /// 卸载技能 (cordiverse F1+F5): 按加载序的 LIFO 逆序执行该 install 的
    /// 全部逆操作, 完成后把 fiber 转入 Retired 终态。逆操作中的失败按 fiber
    /// 捕获, 不中断其余逆操作, 也不影响其他 fiber。
    pub fn uninstall_skill(&mut self, name: &str) -> Result<Vec<Result<(), String>>, String> {
        if self.fiber_lifecycles.get(name).map(|f| f.state) == Some(FiberLifecycleState::Retired) {
            return Err(format!("skill '{}' fiber already retired", name));
        }
        let install_id = self.fiber_lifecycles.get(name)
            .map(|f| f.install_id)
            .ok_or_else(|| format!("no installed fiber for skill '{}'", name))?;
        let results = self.inverse_ledger.teardown(install_id);
        if let Some(fiber) = self.fiber_lifecycles.get_mut(name) {
            let _ = fiber.transition(FiberLifecycleState::Retired);
        }
        if let Some(idx) = self.skills.iter().position(|s| s.name == name) {
            self.skills.remove(idx);
            self.build_index();
        }
        Ok(results)
    }

    /// 从 skill 名查 fiber 当前生命周期状态。
    pub fn fiber_state(&self, name: &str) -> Option<FiberLifecycleState> {
        self.fiber_lifecycles.get(name).map(|f| f.state)
    }

    /// 按 fiber 捕获失败并转入 Failed (不传播到 sibling)。
    pub(crate) fn _record_fiber_failure(&mut self, name: &str, message: impl Into<String>) -> bool {
        if let Some(fiber) = self.fiber_lifecycles.get_mut(name) {
            fiber.record_failure(message);
            true
        } else {
            false
        }
    }

    /// 释放悬挂所有权: fiber 仍标记 held (Loaded/Active/Suspended) 但其 install
    /// 逆账本事务已消失 (holder 失效) → 自动转入 Retired 终态。返回释放列表。
    pub fn release_dangling(&mut self) -> Vec<String> {
        use FiberLifecycleState::*;
        let dangling: Vec<String> = self
            .fiber_lifecycles
            .iter()
            .filter(|(_, f)| matches!(f.state, Loaded | Active | Suspended))
            .filter(|(_, f)| !self.inverse_ledger.has_transaction(f.install_id))
            .map(|(name, _)| name.clone())
            .collect();
        let mut released = Vec::new();
        for name in dangling {
            if let Some(fiber) = self.fiber_lifecycles.get_mut(&name) {
                let _ = fiber.transition(FiberLifecycleState::Retired);
                released.push(name);
            }
        }
        released
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
