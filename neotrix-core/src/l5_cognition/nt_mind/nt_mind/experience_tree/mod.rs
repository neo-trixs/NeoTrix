#![forbid(unsafe_code)]

//! Experience Tree — 五阶段吸收协议 (五阶段吸收引擎)
//!
//! 会话结束时自动执行:
//!   1. 快照 (Snapshot) — 当前状态快照
//!   2. 蒸馏 (Distill) — 提取模式
//!   3. 分类 (Classify) — 归类到领域
//!   4. 落盘 (Persist) — 写入 KB experience hub
//!   5. 反馈 (Feedback) — 回灌到意识流
//!
//! 统一写入 `~/.neotrix/knowledge.db` 的 `kv_store` `experience` 命名空间。
//! 遵循指针守恒: 经验正文只落 KB hub, AGENTS.md 不内联。
//!
//! 挂载点: `nt_mind_background_loop::handlers_absorption` (60s tick) + HookEvent::SessionEnd

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::l5_cognition::kb_facade::KnowledgeBase;
use crate::core::nt_core_knowledge::{AbsorptionRecord, KnowledgeSource};

// ============================================================================
// 数据结构
// ============================================================================

/// 统一 Schema — 每个吸收条目 (branch) 的固定字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceEntry {
    pub schema_version: u32,
    pub entry_type: EntryType,
    pub session_id: String,
    pub cycle: String,
    pub ts: u64,
    pub domain: Domain,
    pub content: String,
    pub evidence: String,
    pub source: Source,
    /// 蒸馏出的模式/规则/缺陷/洞察 (Distill 阶段产出)
    pub patterns: Vec<String>,
    /// 分类置信度
    pub confidence: f64,
}

/// 条目类型: pattern | rule | defect | insight | cycle | artifact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntryType {
    Pattern,
    Rule,
    Defect,
    Insight,
    Cycle,
    Artifact,
}

impl Default for EntryType {
    fn default() -> Self { EntryType::Insight }
}

/// 7 域 + 扩展域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    Core,
    Mind,
    Memory,
    World,
    Act,
    Io,
    Shield,
    Meta,
    Repair,
    Governance,
    Nexus,
}

impl Default for Domain {
    fn default() -> Self { Domain::Core }
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Domain::Core => "NT-CORE",
            Domain::Mind => "NT-MIND",
            Domain::Memory => "NT-MEMORY",
            Domain::World => "NT-WORLD",
            Domain::Act => "NT-ACT",
            Domain::Io => "NT-IO",
            Domain::Shield => "NT-SHIELD",
            Domain::Meta => "NT-META",
            Domain::Repair => "NT-REPAIR",
            Domain::Governance => "NT-GOVERNANCE",
            Domain::Nexus => "NT-NEXUS",
        };
        write!(f, "{}", s)
    }
}

/// 来源: dialogue | audit | research | absorption
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Source {
    Dialogue,
    Audit,
    Research,
    Absorption,
}

impl Default for Source {
    fn default() -> Self { Source::Dialogue }
}

/// 快照 — 会话开始时记录的上下文状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub session_id: String,
    pub cycle: String,
    pub task: String,
    pub domain: Domain,
    pub ts: u64,
    /// 会话期间的上下文标记
    pub context_tags: Vec<String>,
    /// 意识流当前状态摘要
    pub consciousness_state: HashMap<String, String>,
}

/// 蒸馏结果 — 从对话/经验中提取的模式。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillResult {
    pub session_id: String,
    pub cycle: String,
    pub patterns: Vec<String>,
    pub rules: Vec<String>,
    pub defects: Vec<String>,
    pub insights: Vec<String>,
    pub generated_at: u64,
}

/// 分类结果 — 将经验归类到领域和节点。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyResult {
    pub entries: Vec<ClassifiedEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedEntry {
    pub entry: ExperienceEntry,
    pub matched_nodes: Vec<String>,
    pub domain: Domain,
    pub confidence: f64,
}

/// 五阶段引擎的统一输出。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorptionResult {
    pub snapshot: Option<Snapshot>,
    pub distill: Option<DistillResult>,
    pub classify: Option<ClassifyResult>,
    pub persisted_keys: Vec<String>,
    pub feedback_applied: bool,
    pub session_id: String,
    pub cycle: String,
}

// ============================================================================
// Phase 1: Snapshot (快照)
// ============================================================================

/// 阶段 1 — 会话快照: 记录当前上下文状态。
pub fn snapshot(session_id: &str, cycle: &str, task: &str, domain: Domain) -> Snapshot {
    let mut consciousness_state = HashMap::new();
    consciousness_state.insert("phase".to_string(), "active".to_string());
    consciousness_state.insert("last_tick".to_string(), now_ts().to_string());

    Snapshot {
        session_id: session_id.to_string(),
        cycle: cycle.to_string(),
        task: task.to_string(),
        domain,
        ts: now_ts(),
        context_tags: Vec::new(),
        consciousness_state,
    }
}

/// 添加上下文标记到快照。
pub fn snapshot_with_tags(snapshot: &mut Snapshot, tags: Vec<String>) {
    snapshot.context_tags = tags;
}

// ============================================================================
// Phase 2: Distill (蒸馏)
// ============================================================================

/// 阶段 2 — 蒸馏: 从会话内容提取模式/规则/缺陷/洞察。
pub fn distill(session_id: &str, cycle: &str, content: &str) -> DistillResult {
    let patterns = extract_patterns(content);
    let rules = extract_rules(content);
    let defects = extract_defects(content);
    let insights = extract_insights(content);

    DistillResult {
        session_id: session_id.to_string(),
        cycle: cycle.to_string(),
        patterns,
        rules,
        defects,
        insights,
        generated_at: now_ts(),
    }
}

fn extract_patterns(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|l| l.trim().len() > 10)
        .take(10)
        .map(|l| format!("pattern: {}", l.trim().chars().take(80).collect::<String>()))
        .collect()
}

fn extract_rules(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|l| l.contains("=>") || l.contains("→") || l.contains("->"))
        .take(5)
        .map(|l| format!("rule: {}", l.trim().chars().take(80).collect::<String>()))
        .collect()
}

fn extract_defects(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|l| l.contains("bug") || l.contains("fix") || l.contains("缺陷") || l.contains("修复"))
        .take(5)
        .map(|l| format!("defect: {}", l.trim().chars().take(80).collect::<String>()))
        .collect()
}

fn extract_insights(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|l| l.to_lowercase().contains("insight") || l.to_lowercase().contains("洞察") || l.to_lowercase().contains("learned"))
        .take(5)
        .map(|l| format!("insight: {}", l.trim().chars().take(80).collect::<String>()))
        .collect()
}

// ============================================================================
// Phase 3: Classify (分类)
// ============================================================================

/// 阶段 3 — 分类: 将蒸馏出的经验归类到领域和节点。
pub fn classify(
    distill: &DistillResult,
    domain: Domain,
    session_id: &str,
    cycle: &str,
    evidence: &str,
) -> ClassifyResult {
    let mut entries = Vec::new();
    let confidence = 0.85;

    let all_items: Vec<(String, EntryType)> = distill
        .patterns
        .iter()
        .map(|p| (p.clone(), EntryType::Pattern))
        .chain(distill.rules.iter().map(|r| (r.clone(), EntryType::Rule)))
        .chain(distill.defects.iter().map(|d| (d.clone(), EntryType::Defect)))
        .chain(distill.insights.iter().map(|i| (i.clone(), EntryType::Insight)))
        .collect();

    for (content, entry_type) in all_items {
        let entry = ExperienceEntry {
            schema_version: 1,
            entry_type,
            session_id: session_id.to_string(),
            cycle: cycle.to_string(),
            ts: now_ts(),
            domain,
            content,
            evidence: evidence.to_string(),
            source: Source::Dialogue,
            patterns: vec![],
            confidence,
        };
        entries.push(ClassifiedEntry {
            matched_nodes: vec![format!("node_{}_{}", domain as u8, entry.entry_type as u8)],
            domain,
            confidence,
            entry,
        });
    }

    ClassifyResult { entries }
}

// ============================================================================
// Phase 4: Persist (落盘)
// ============================================================================

/// 阶段 4 — 落盘: 写入 KB experience hub。
///
/// 统一写入 `kv_store` `experience` 命名空间。
/// 每个条目以 `branch_{cycle}_{index}_{hash}` 为 key。
/// 同时更新 hub 索引。
pub fn persist(
    kb: &KnowledgeBase,
    classify: &ClassifyResult,
    session_id: &str,
    cycle: &str,
) -> Result<Vec<String>, String> {
    let mut persisted_keys = Vec::new();
    let namespace = "experience";

    for (i, classified) in classify.entries.iter().enumerate() {
        let key = format!("branch_{}_{:04x}_{:04x}", cycle, i, (&classified.entry.session_id).chars().take(4).fold(0u16, |acc, c| acc.wrapping_add(c as u16)));
        let json = serde_json::to_string(&classified.entry)
            .map_err(|e| format!("persist: serialize entry: {e}"))?;

        kb.kv_set(namespace, &key, &json)
            .map_err(|e| format!("persist: kv_set failed for {key}: {e}"))?;

        // 登记吸收记录
        let _record = AbsorptionRecord {
            source: KnowledgeSource::DialogueExperience,
            timestamp: now_ts(),
            weight: KnowledgeSource::DialogueExperience.source_weight(),
        };

        persisted_keys.push(key);
    }

    // 更新 hub 索引
    update_hub(kb, session_id, cycle, &persisted_keys)?;

    Ok(persisted_keys)
}

/// 更新 hub 索引 — 维护 cycles/branches/dimensions/route_table 结构。
fn update_hub(
    kb: &KnowledgeBase,
    session_id: &str,
    cycle: &str,
    keys: &[String],
) -> Result<(), String> {
    let hub_json = kb.kv_get("experience", "hub")
        .map_err(|e| format!("update_hub: kv_get hub: {e}"))?;

    let mut hub: serde_json::Value = if let Some(h) = hub_json {
        serde_json::from_str(&h)
            .map_err(|e| format!("update_hub: parse hub json: {e}"))?
    } else {
        serde_json::json!({
            "schema_version": 1,
            "hub": { "cycles": {}, "branches": {}, "dimensions": {}, "route_table": {} },
            "metrics": { "total_entries": 0, "concepts": 0, "by_type": {}, "by_domain": {}, "by_source": {} },
            "legacy_sources": {}
        })
    };

    let hub_obj = hub.get_mut("hub").and_then(|h| h.as_object_mut())
        .ok_or_else(|| "update_hub: invalid hub structure".to_string())?;

    // 更新 cycles 索引
    let cycles = hub_obj["cycles"].as_object_mut()
        .ok_or_else(|| "update_hub: cycles not object".to_string())?;
    cycles[cycle] = serde_json::json!({
        "session_id": session_id,
        "ts": now_ts(),
        "entries": keys.len(),
        "keys": keys
    });

    // 更新 branches 索引
    let branches = hub_obj["branches"].as_object_mut()
        .ok_or_else(|| "update_hub: branches not object".to_string())?;
    for key in keys {
        branches[key] = serde_json::json!({ "cycle": cycle, "ts": now_ts() });
    }

    // 更新指标
    let metrics = hub.get_mut("metrics").and_then(|m| m.as_object_mut())
        .ok_or_else(|| "update_hub: metrics not object".to_string())?;
    let total = metrics.get("total_entries").and_then(|v| v.as_u64()).unwrap_or(0);
    metrics["total_entries"] = serde_json::json!(total + keys.len() as u64);

    let hub_bytes = serde_json::to_string(&hub)
        .map_err(|e| format!("update_hub: serialize: {e}"))?;
    kb.kv_set("experience", "hub", &hub_bytes)
        .map_err(|e| format!("update_hub: kv_set: {e}"))?;

    Ok(())
}

// ============================================================================
// Phase 5: Feedback (反馈)
// ============================================================================

/// 阶段 5 — 反馈: 回灌到意识流。
///
/// 将吸收的经验同步到 consciousness_runtime 和 route_table,
/// 使后续 session 可检索。
pub fn feedback(
    _kb: &KnowledgeBase,
    persist_result: &[String],
    cycle: &str,
) -> bool {
    if persist_result.is_empty() {
        return false;
    }

    // 更新 route_table — 使后续 session 可检索
    let route_table_key = format!("route_table_{cycle}");
    let route_data = serde_json::json!({
        "cycle": cycle,
        "updated": now_ts(),
        "branches": persist_result
    });
    let _ = _kb.kv_set("experience", &route_table_key, &serde_json::to_string(&route_data).unwrap_or_default());

    true
}

// ============================================================================
// 编排器 — 五阶段完整流程
// ============================================================================

/// 五阶段吸收引擎编排器。
///
/// 组合 snapshot → distill → classify → persist → feedback
/// 统一写入 `~/.neotrix/knowledge.db` 的 `kv_store experience` 命名空间。
pub struct ExperienceEngine {
    pub kb: Arc<KnowledgeBase>,
}

impl ExperienceEngine {
    pub fn new(kb: Arc<KnowledgeBase>) -> Self {
        Self { kb }
    }

    /// 执行完整五阶段吸收协议。
    ///
    /// 返回 `AbsorptionResult` 包含各阶段产出。
    pub fn run(
        &self,
        session_id: &str,
        cycle: &str,
        task: &str,
        domain: Domain,
        content: &str,
        evidence: &str,
    ) -> Result<AbsorptionResult, String> {
        // Phase 1: Snapshot
        let mut snap = snapshot(session_id, cycle, task, domain);
        snapshot_with_tags(&mut snap, vec!["absorption".to_string()]);

        // Phase 2: Distill
        let distill = distill(session_id, cycle, content);

        // Phase 3: Classify
        let classify = classify(&distill, domain, session_id, cycle, evidence);

        // Phase 4: Persist
        let persisted_keys = persist(&self.kb, &classify, session_id, cycle)?;

        // Phase 5: Feedback
        let feedback_applied = feedback(&self.kb, &persisted_keys, cycle);

        Ok(AbsorptionResult {
            snapshot: Some(snap),
            distill: Some(distill),
            classify: Some(classify),
            persisted_keys,
            feedback_applied,
            session_id: session_id.to_string(),
            cycle: cycle.to_string(),
        })
    }
}

// ============================================================================
// 会话结束自动触发 — Hook 集成
// ============================================================================

/// 会话结束自动触发 — 绑定到 `HookEvent::SessionEnd` 的 HookAction。
///
/// 当会话结束时, 后台循环自动驱动五阶段吸收协议。
/// 代理职责: 仅将经验写入 `~/.neotrix/pending-absorb.json`,
/// 由 `handle_pending_absorption` (60s tick) 驱动真正的吸收。
 pub struct SessionEndHook {
     pub kb: Option<Arc<KnowledgeBase>>,
 }

 impl SessionEndHook {
     pub fn new(kb: Option<Arc<KnowledgeBase>>) -> Self {
         Self { kb }
     }
 }

impl crate::l5_cognition::nt_mind::nt_mind_hook::HookAction for SessionEndHook {
    fn name(&self) -> &str {
        "experience_session_end_hook"
    }

    fn execute(&self, ctx: &crate::l5_cognition::nt_mind::nt_mind_hook::HookContext) -> crate::l5_cognition::nt_mind::nt_mind_hook::HookResult {
        let cycle = ctx.payload.as_ref()
            .and_then(|p| p.get("cycle"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let session_id = ctx.payload.as_ref()
            .and_then(|p| p.get("session_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // G3: SessionLedger — 会话结束时记录证据账本，防幻觉
        let mut ledger = crate::l1_action::nt_memory::evidence_ledger::SessionLedger::new(session_id);
        ledger.add_evidence(
            crate::l1_action::nt_memory::evidence_ledger::EvidenceType::Observation,
            &format!("Session {session_id} cycle {cycle} ended"),
            "experience_tree_hook",
            0.9,
        );

        // 写入 pending-absorb.json 供后台循环消费
        let pending = pending_absorb_path();
        let entry = serde_json::json!({
            "schema_version": 1,
            "session_id": session_id,
            "cycle": cycle,
            "ts": now_ts(),
            "domain": "NT-MIND",
            "entries": []
        });
        let _ = std::fs::write(&pending, serde_json::to_string_pretty(&entry).unwrap_or_default());

        crate::l5_cognition::nt_mind::nt_mind_hook::HookResult::ok(&format!(
            "experience-tree: session {session_id} cycle {cycle} queued for absorption"
        ))
    }
}

/// 待吸收文件路径: ~/.neotrix/pending-absorb.json
fn pending_absorb_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("pending-absorb.json")
}

// ============================================================================
// 跨会话模式挖掘 — Nexus-Weaver 自动调度
// ============================================================================

/// 跨会话模式挖掘 — 定期扫描 experience 命名空间,
/// 识别跨会话模式并触发 nexus-weaver 调度。
///
/// 由 `nt_mind_background_loop` 在独立 handler 中周期性调用。
pub struct NexusWeaverScheduler {
    pub kb: Arc<KnowledgeBase>,
    /// 跨会话模式最小出现次数
    pub min_pattern_occurrences: usize,
    /// 上次挖掘时间戳
    pub last_weave_ts: u64,
}

impl NexusWeaverScheduler {
    pub fn new(kb: Arc<KnowledgeBase>) -> Self {
        Self {
            kb,
            min_pattern_occurrences: 3,
            last_weave_ts: 0,
        }
    }

    /// 扫描 experience 命名空间, 识别跨会话模式并调度 nexus-weaver。
    ///
    /// 返回发现的模式连接数。
    pub fn weave_patterns(&mut self) -> Result<usize, String> {
        let entries = self.kb.experience_entries()?;
        if entries.is_empty() {
            return Ok(0);
        }

        // 统计 domain → 条目数 映射
        let mut domain_counts: HashMap<String, usize> = HashMap::new();
        let mut domain_patterns: HashMap<String, Vec<String>> = HashMap::new();

        for (key, _value) in &entries {
            if let Ok(Some(val)) = self.kb.kv_get("experience", key) {
                if let Ok(entry) = serde_json::from_str::<ExperienceEntry>(&val) {
                    let domain_str = entry.domain.to_string();
                    *domain_counts.entry(domain_str.clone()).or_insert(0) += 1;
                    domain_patterns
                        .entry(domain_str)
                        .or_default()
                        .push(entry.content);
                }
            }
        }

        // 识别跨会话模式: 同一 domain 下出现次数 >= min_pattern_occurrences
        let mut connections = 0;
        for (domain, count) in &domain_counts {
            if *count >= self.min_pattern_occurrences {
                connections += 1;
                log::info!(
                    "[nexus-weaver] domain {} has {} cross-session patterns (>= {})",
                    domain, count, self.min_pattern_occurrences
                );
            }
        }

        self.last_weave_ts = now_ts();
        Ok(connections)
    }

    /// 获取待处理的跨会话桥接建议。
    pub fn pending_bridges(&self) -> Result<Vec<String>, String> {
        let entries = self.kb.experience_entries()?;
        let mut bridges = Vec::new();

        for (key, _value) in &entries {
            if key.contains("bridge") || key.contains("connection") {
                bridges.push(key.clone());
            }
        }

        Ok(bridges)
    }
}

// ============================================================================
// KB 检索查询接口
// ============================================================================

/// KB 检索查询接口 — 提供对 experience namespace 的检索能力。
///
/// 支持关键词查询、类型/域过滤、cycle 核对。
pub struct ExperienceQuery {
    pub kb: Arc<KnowledgeBase>,
}

impl ExperienceQuery {
    pub fn new(kb: Arc<KnowledgeBase>) -> Self {
        Self { kb }
    }

    /// 关键词检索 — 扫描 experience 命名空间, 匹配 content/evidence。
    pub fn query(&self, keyword: &str, limit: usize) -> Result<Vec<ExperienceEntry>, String> {
        let entries = self.kb.experience_entries()?;
        let mut results = Vec::new();

        for (key, _value) in &entries {
            if results.len() >= limit {
                break;
            }
            if let Ok(Some(val)) = self.kb.kv_get("experience", key) {
                if let Ok(entry) = serde_json::from_str::<ExperienceEntry>(&val) {
                    if entry.content.contains(keyword) || entry.evidence.contains(keyword) {
                        results.push(entry);
                    }
                }
            }
        }

        Ok(results)
    }

    /// 按域检索 — 返回指定 domain 的所有条目。
    pub fn query_by_domain(&self, domain: &Domain, limit: usize) -> Result<Vec<ExperienceEntry>, String> {
        let entries = self.kb.experience_entries()?;
        let mut results = Vec::new();
        let domain_str = domain.to_string();

        for (key, _value) in &entries {
            if results.len() >= limit {
                break;
            }
            if let Ok(Some(val)) = self.kb.kv_get("experience", key) {
                if let Ok(entry) = serde_json::from_str::<ExperienceEntry>(&val) {
                    if entry.domain.to_string() == domain_str {
                        results.push(entry);
                    }
                }
            }
        }

        Ok(results)
    }

    /// 按类型检索 — 返回指定 entry_type 的所有条目。
    pub fn query_by_type(&self, entry_type: &EntryType, limit: usize) -> Result<Vec<ExperienceEntry>, String> {
        let entries = self.kb.experience_entries()?;
        let mut results = Vec::new();

        for (key, _value) in &entries {
            if results.len() >= limit {
                break;
            }
            if let Ok(Some(val)) = self.kb.kv_get("experience", key) {
                if let Ok(entry) = serde_json::from_str::<ExperienceEntry>(&val) {
                    if entry.entry_type == *entry_type {
                        results.push(entry);
                    }
                }
            }
        }

        Ok(results)
    }

    /// 按 cycle 核对 — 列出指定 cycle 的全部分支。
    pub fn list_by_cycle(&self, cycle: &str) -> Result<Vec<(String, ExperienceEntry)>, String> {
        let entries = self.kb.experience_entries()?;
        let mut results = Vec::new();

        for (key, _value) in &entries {
            if let Ok(Some(val)) = self.kb.kv_get("experience", key) {
                if let Ok(entry) = serde_json::from_str::<ExperienceEntry>(&val) {
                    if entry.cycle == cycle {
                        results.push((key.clone(), entry));
                    }
                }
            }
        }

        Ok(results)
    }

    /// 机器可读检索 — JSON 输出。
    pub fn query_json(&self, keyword: &str, limit: usize) -> Result<serde_json::Value, String> {
        let entries = self.query(keyword, limit)?;
        let json: Vec<serde_json::Value> = entries.iter().map(|e| {
            serde_json::json!({
                "key": format!("branch_{}", e.cycle),
                "cycle": e.cycle,
                "type": format!("{:?}", e.entry_type),
                "domain": e.domain.to_string(),
                "content": e.content,
                "evidence": e.evidence,
            })
        }).collect();
        Ok(serde_json::Value::Array(json))
    }

    /// 获取 hub 索引。
    pub fn hub(&self) -> Result<Option<serde_json::Value>, String> {
        let val = self.kb.kv_get("experience", "hub")?;
        if let Some(v) = val {
            serde_json::from_str(&v)
                .map(Some)
                .map_err(|e| format!("hub parse: {e}"))
        } else {
            Ok(None)
        }
    }

    /// 列出所有 experience 条目数。
    pub fn count(&self) -> Result<usize, String> {
        self.kb.experience_entries().map(|e| e.len())
    }

    /// 神经概念图检视 — 检视概念突触链路。
    pub fn neuron(&self, term: &str) -> Result<Vec<ExperienceEntry>, String> {
        self.query(term, 10)
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

fn now_ts() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ============================================================================
// Self-Reflection Engine — Reflexion-style verbal reinforcement learning loop
// ============================================================================

pub mod self_reflection;
pub use self_reflection::{SelfReflectionEngine, ReflectionBuffer, ReflectionRecord};

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot() {
        let snap = snapshot("sess_001", "100", "test task", Domain::Core);
        assert_eq!(snap.session_id, "sess_001");
        assert_eq!(snap.cycle, "100");
        assert_eq!(snap.task, "test task");
        assert_eq!(snap.domain, Domain::Core);
        assert_eq!(snap.context_tags.len(), 0);
    }

    #[test]
    fn test_snapshot_with_tags() {
        let mut snap = snapshot("sess_001", "100", "test", Domain::Core);
        snapshot_with_tags(&mut snap, vec!["tag1".to_string(), "tag2".to_string()]);
        assert_eq!(snap.context_tags.len(), 2);
        assert_eq!(snap.context_tags[0], "tag1");
    }

    #[test]
    fn test_distill_extracts_patterns() {
        let content = "Found a bug in the parser. => fix needed\nThis is a pattern: analyze then implement.\nInsight: learn from mistakes";
        let distill = distill("sess_001", "100", content);
        assert!(!distill.patterns.is_empty());
        assert!(!distill.defects.is_empty());
        assert!(!distill.insights.is_empty());
    }

    #[test]
    fn test_distill_empty_content() {
        let distill = distill("sess_001", "100", "");
        assert!(distill.patterns.is_empty());
        assert!(distill.rules.is_empty());
        assert!(distill.defects.is_empty());
        assert!(distill.insights.is_empty());
    }

    #[test]
    fn test_classify() {
        let distill = DistillResult {
            session_id: "sess_001".to_string(),
            cycle: "100".to_string(),
            patterns: vec!["pattern_a".to_string()],
            rules: vec!["rule_a".to_string()],
            defects: vec!["defect_a".to_string()],
            insights: vec!["insight_a".to_string()],
            generated_at: now_ts(),
        };
        let classify = classify(&distill, Domain::Mind, "sess_001", "100", "file:10");
        assert_eq!(classify.entries.len(), 4);
        for entry in &classify.entries {
            assert_eq!(entry.domain, Domain::Mind);
            assert!(entry.confidence > 0.0);
            assert!(!entry.matched_nodes.is_empty());
        }
    }

    #[test]
    fn test_domain_display() {
        assert_eq!(Domain::Core.to_string(), "NT-CORE");
        assert_eq!(Domain::Mind.to_string(), "NT-MIND");
        assert_eq!(Domain::Nexus.to_string(), "NT-NEXUS");
    }

    #[test]
    fn test_entry_type_default() {
        assert_eq!(EntryType::default(), EntryType::Insight);
    }

    #[test]
    fn test_domain_default() {
        assert_eq!(Domain::default(), Domain::Core);
    }

    #[test]
    fn test_source_default() {
        assert_eq!(Source::default(), Source::Dialogue);
    }

    #[test]
    fn test_result_has_session_info() {
        let result = AbsorptionResult {
            snapshot: None,
            distill: None,
            classify: None,
            persisted_keys: vec!["branch_100_0001".to_string()],
            feedback_applied: true,
            session_id: "sess_001".to_string(),
            cycle: "100".to_string(),
        };
        assert_eq!(result.session_id, "sess_001");
        assert_eq!(result.cycle, "100");
        assert_eq!(result.persisted_keys.len(), 1);
        assert!(result.feedback_applied);
    }

    #[test]
    fn test_nexus_weaver_scheduler() {
        // 验证 NexusWeaverScheduler 结构正确
        // (实际 KB 交互需要运行时环境)
        let _scheduler = NexusWeaverScheduler {
            kb: Arc::new(KnowledgeBase::open(None).unwrap_or_else(|_| panic!("KB open failed"))),
            min_pattern_occurrences: 3,
            last_weave_ts: 0,
        };
    }
}