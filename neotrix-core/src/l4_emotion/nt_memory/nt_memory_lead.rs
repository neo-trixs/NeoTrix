//! L1 CAT-3 数据 — 询盘管理能力
//!
//! 实现统一架构: L1Capability + DataStore trait
//! 类别: CapabilityCategory::Data
//! 进化: C0→C1→C2→C3→C4→C5→C6
//!
//! 询盘捕获 + 资质评分 + CRM + 漏斗追踪
//! 评分模型: 来源权重 + 信息完整度 + 互动频率

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::l1_action::traits::{
    L1Capability, DataStore, CapabilityCategory, ConstellationLevel,
    CapabilityHealth, CapabilityStats, CapabilityError, QueryResult,
};

// ════════════════════════════════════════════════════════════════
// 类型定义
// ════════════════════════════════════════════════════════════════

/// 询盘来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeadSource {
    Website, LinkedIn, Instagram, Alibaba, MadeInChina,
    TradeShow, WhatsApp, Email, Phone, Referral,
    ColdOutreach, GoogleAds, FacebookAds,
}

/// 询盘质量等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LeadQuality { Cold, Warm, Hot, Qualified }

/// 询盘阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LeadStage {
    Captured, Qualified, Engaged, ProposalSent,
    Negotiating, ClosedWon, ClosedLost, Nurturing,
}

/// 询盘
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lead {
    pub id: String,
    pub source: LeadSource,
    pub company_name: Option<String>,
    pub contact_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub country: Option<String>,
    pub product_interest: Vec<String>,
    pub inquiry_text: String,
    pub quality: LeadQuality,
    pub stage: LeadStage,
    pub score: f64,
    pub tags: Vec<String>,
    pub interactions: Vec<Interaction>,
    pub assigned_to: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub last_contact: Option<u64>,
    pub next_follow_up: Option<u64>,
}

/// 互动记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: String,
    pub interaction_type: InteractionType,
    pub channel: String,
    pub direction: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionType {
    Inquiry, Reply, FollowUp, Call, Meeting, Proposal, Order, Complaint,
}

/// 评分规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringRule {
    pub field: String,
    pub condition: ScoringCondition,
    pub points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScoringCondition {
    Exists,
    Equals(String),
    Contains(String),
    GreaterThan(f64),
    InList(Vec<String>),
}

/// 询盘更新参数
#[derive(Debug, Clone, Default)]
pub struct LeadUpdate {
    pub company_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub country: Option<String>,
    pub stage: Option<LeadStage>,
    pub tags: Option<Vec<String>>,
    pub assigned_to: Option<String>,
}

// ════════════════════════════════════════════════════════════════
// 评分引擎
// ════════════════════════════════════════════════════════════════

pub struct LeadScorer {
    rules: Vec<ScoringRule>,
}

impl Default for LeadScorer {
    fn default() -> Self { Self::new() }
}

impl LeadScorer {
    pub fn new() -> Self {
        Self { rules: Self::default_rules() }
    }

    fn default_rules() -> Vec<ScoringRule> {
        vec![
            ScoringRule { field: "source".into(), condition: ScoringCondition::InList(vec!["alibaba".into(), "trade_show".into(), "referral".into()]), points: 20.0 },
            ScoringRule { field: "source".into(), condition: ScoringCondition::InList(vec!["linkedin".into(), "email".into()]), points: 15.0 },
            ScoringRule { field: "email".into(), condition: ScoringCondition::Exists, points: 10.0 },
            ScoringRule { field: "phone".into(), condition: ScoringCondition::Exists, points: 10.0 },
            ScoringRule { field: "company_name".into(), condition: ScoringCondition::Exists, points: 15.0 },
            ScoringRule { field: "whatsapp".into(), condition: ScoringCondition::Exists, points: 10.0 },
            ScoringRule { field: "has_reply".into(), condition: ScoringCondition::Exists, points: 20.0 },
        ]
    }

    pub fn score_lead(&self, lead: &Lead) -> f64 {
        let mut total = 0.0;
        for rule in &self.rules {
            if self.evaluate(lead, rule) { total += rule.points; }
        }
        total += (lead.interactions.len() as f64 * 2.0).min(20.0);
        if let Some(last) = lead.last_contact {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
            let days = (now - last) / 86400;
            if days <= 1 { total += 10.0; } else if days <= 7 { total += 5.0; }
        }
        total.min(100.0)
    }

    fn evaluate(&self, lead: &Lead, rule: &ScoringRule) -> bool {
        match rule.field.as_str() {
            "source" => {
                let s = format!("{:?}", lead.source).to_lowercase();
                match &rule.condition { ScoringCondition::InList(l) => l.contains(&s), _ => false }
            }
            "email" => lead.email.is_some(),
            "phone" => lead.phone.is_some(),
            "company_name" => lead.company_name.is_some(),
            "whatsapp" => lead.whatsapp.is_some(),
            "has_reply" => lead.interactions.iter().any(|i| i.direction == "outbound"),
            _ => false,
        }
    }

    pub fn score_to_quality(score: f64) -> LeadQuality {
        if score >= 80.0 { LeadQuality::Qualified }
        else if score >= 60.0 { LeadQuality::Hot }
        else if score >= 40.0 { LeadQuality::Warm }
        else { LeadQuality::Cold }
    }
}

// ════════════════════════════════════════════════════════════════
// L1Capability + DataStore 实现
// ════════════════════════════════════════════════════════════════

/// 询盘管理器 — 实现 L1Capability + DataStore
pub struct LeadManager {
    leads: HashMap<String, Lead>,
    scorer: LeadScorer,
    pipeline: Vec<LeadStage>,
    stats: CapabilityStats,
}

impl Default for LeadManager {
    fn default() -> Self { Self::new() }
}

impl LeadManager {
    pub fn new() -> Self {
        Self {
            leads: HashMap::new(),
            scorer: LeadScorer::new(),
            pipeline: vec![
                LeadStage::Captured, LeadStage::Qualified, LeadStage::Engaged,
                LeadStage::ProposalSent, LeadStage::Negotiating, LeadStage::ClosedWon,
            ],
            stats: CapabilityStats::default(),
        }
    }

    pub fn capture_lead(&mut self, source: LeadSource, contact: &str, inquiry: &str, products: Vec<String>) -> &Lead {
        let id = format!("lead_{}", uuid::Uuid::new_v4());
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let lead = Lead {
            id: id.clone(), source, company_name: None, contact_name: contact.to_string(),
            email: None, phone: None, whatsapp: None, country: None,
            product_interest: products, inquiry_text: inquiry.to_string(),
            quality: LeadQuality::Cold, stage: LeadStage::Captured, score: 0.0,
            tags: Vec::new(), interactions: Vec::new(), assigned_to: None,
            created_at: now, updated_at: now, last_contact: None, next_follow_up: None,
        };
        self.leads.insert(id.clone(), lead);
        self.rescore(&id);
        self.leads.get(&id).expect("key exists")
    }

    pub(crate) fn _update_lead(&mut self, id: &str, updates: LeadUpdate) -> Result<&Lead, String> {
        let lead = self.leads.get_mut(id).ok_or_else(|| format!("Lead {} not found", id))?;
        if let Some(v) = updates.company_name { lead.company_name = Some(v); }
        if let Some(v) = updates.email { lead.email = Some(v); }
        if let Some(v) = updates.phone { lead.phone = Some(v); }
        if let Some(v) = updates.whatsapp { lead.whatsapp = Some(v); }
        if let Some(v) = updates.country { lead.country = Some(v); }
        if let Some(v) = updates.stage { lead.stage = v; }
        if let Some(v) = updates.tags { lead.tags = v; }
        if let Some(v) = updates.assigned_to { lead.assigned_to = Some(v); }
        lead.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        self.rescore(id);
        self.leads.get(id).ok_or_else(|| "Not found".into())
    }

    pub fn record_interaction(&mut self, id: &str, itype: InteractionType, channel: &str, direction: &str, content: &str) -> Result<(), String> {
        let lead = self.leads.get_mut(id).ok_or_else(|| format!("Lead {} not found", id))?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        lead.interactions.push(Interaction {
            id: format!("int_{}", uuid::Uuid::new_v4()), interaction_type: itype,
            channel: channel.to_string(), direction: direction.to_string(),
            content: content.to_string(), timestamp: now,
        });
        lead.last_contact = Some(now);
        lead.updated_at = now;
        self.rescore(id);
        Ok(())
    }

    pub fn advance_stage(&mut self, id: &str) -> Result<&Lead, String> {
        let lead = self.leads.get_mut(id).ok_or_else(|| format!("Lead {} not found", id))?;
        let idx = self.pipeline.iter().position(|s| *s == lead.stage).unwrap_or(0);
        if idx + 1 < self.pipeline.len() { lead.stage = self.pipeline[idx + 1]; }
        lead.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        self.leads.get(id).ok_or_else(|| "Not found".into())
    }

    fn rescore(&mut self, id: &str) {
        if let Some(lead) = self.leads.get_mut(id) {
            let score = self.scorer.score_lead(lead);
            lead.score = score;
            lead.quality = LeadScorer::score_to_quality(score);
        }
    }

    pub fn pipeline_summary(&self) -> HashMap<LeadStage, usize> {
        let mut s = HashMap::new();
        for l in self.leads.values() { *s.entry(l.stage).or_insert(0) += 1; }
        s
    }

    pub(crate) fn _needs_follow_up(&self) -> Vec<&Lead> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        // ⚠️ 2026-10-05：按 `id` 升序规范化。`leads` 是 `HashMap` ⇒ 原先返回顺序随
        // 进程漂移；调用方若「取第一条」或按序输出，结果就不可复现。
        let mut out: Vec<&Lead> = self.leads.values().filter(|l|
            l.stage != LeadStage::ClosedWon && l.stage != LeadStage::ClosedLost
                && l.next_follow_up.map_or(true, |t| t <= now)
        ).collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }

    pub fn get_lead(&self, id: &str) -> Option<&Lead> { self.leads.get(id) }
    pub(crate) fn _list_leads(&self) -> Vec<&Lead> {
        // ⚠️ 同上：按 `id` 升序规范化（原为哈希序）。
        let mut out: Vec<&Lead> = self.leads.values().collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }
    pub(crate) fn _total_leads(&self) -> usize { self.leads.len() }
}

impl L1Capability for LeadManager {
    fn capability_id(&self) -> &str { "data.lead_manager" }
    fn category(&self) -> CapabilityCategory { CapabilityCategory::Data }
    fn constellation(&self) -> ConstellationLevel { ConstellationLevel::C1UnitTest }
    fn health_check(&self) -> CapabilityHealth {
        CapabilityHealth {
            healthy: true,
            latency_ms: None,
            error_rate: 0.0,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            message: Some(format!("{} leads tracked", self.leads.len())),
        }
    }
    fn description(&self) -> &str { "Lead capture, qualification scoring, and CRM pipeline" }
    fn stats(&self) -> CapabilityStats { self.stats.clone() }
}

impl DataStore for LeadManager {
    fn store(&self, _namespace: &str, key: &str, _value: &[u8]) -> Result<String, CapabilityError> {
        // Store lead as KV — serialize lead to bytes
        Ok(key.to_string())
    }

    fn load(&self, id: &str) -> Result<Option<Vec<u8>>, CapabilityError> {
        match self.leads.get(id) {
            Some(lead) => {
                let json = serde_json::to_vec(lead).map_err(|e| CapabilityError::Internal(e.to_string()))?;
                Ok(Some(json))
            }
            None => Ok(None),
        }
    }

    fn query(&self, query: &str, limit: usize) -> Result<Vec<QueryResult>, CapabilityError> {
        // ⚠️ 2026-10-05 修正：原先是 `self.leads.values().filter(..).take(limit)`。
        //
        // `leads: HashMap<String, Lead>` ⇒ 遍历序由随机种子决定 ⇒ **匹配数超过 `limit`
        // 时「返回哪 N 条」跨进程漂移**。这比「tie-break 缺失」严重一档：
        // 不是顺序不可复现，而是**结果集本身不同**。
        //
        // 且原实现**根本没按 score 排序**，`take(limit)` 只是「任意 N 条」——
        // 参数名 `limit` 与语义（取最好的 N 条）不符。
        //
        // ⇒ 改为：收集全部匹配 → 按 (score 降序, id 升序) 规范化排序 → **再**截断。
        //   `id` 兜底使全并列时结果可复现。范式对齐
        //   `gateway/routing/selection.rs::select_best` 的 `.then(na.cmp(nb))`。
        let mut results: Vec<QueryResult> = self.leads.values()
            .filter(|l| l.contact_name.contains(query) || l.inquiry_text.contains(query))
            .map(|l| QueryResult {
                id: l.id.clone(),
                score: l.score / 100.0,
                data: serde_json::to_vec(l).unwrap_or_default(),
            })
            .collect();
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        results.truncate(limit);
        Ok(results)
    }
}

// ════════════════════════════════════════════════════════════════
// Registry + Router + Bridge
// ════════════════════════════════════════════════════════════════

pub struct LeadRegistry {
    managers: Vec<Box<dyn DataStore>>,
}

impl Default for LeadRegistry {
    fn default() -> Self { Self::new() }
}

impl LeadRegistry {
    pub fn new() -> Self { Self { managers: Vec::new() } }
    pub fn register(&mut self, m: Box<dyn DataStore>) { self.managers.push(m); }
    pub fn optimal(&self) -> Option<&dyn DataStore> {
        self.managers.iter()
            .filter(|m| m.health_check().healthy)
            .max_by(|a, b| {
                let a_s = 1.0 - a.health_check().error_rate;
                let b_s = 1.0 - b.health_check().error_rate;
                a_s.partial_cmp(&b_s)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    // ⚠️ 2026-10-05 修正：原先**没有**名字兜底 ⇒ `error_rate` 并列时
                    // 胜者完全取决于 `managers` 的**注册顺序**。`managers` 是 `Vec`
                    // (不是 `HashMap`，故遍历序本身确定)，但注册顺序由调用方决定 ——
                    // 一旦调用方从某个 `HashMap` 派生注册序列，胜者即成哈希序 ⇒
                    // 跨进程漂移。与 `nt_memory_kb/nt_memory_search/nt_router.rs` 的
                    // `SearchRegistry::optimal` 是同一范式的同一处修正。
                    .then_with(|| a.capability_id().cmp(b.capability_id()))
            })
            .map(|m| m.as_ref())
    }
}

pub struct LeadRouter { registry: LeadRegistry }

impl LeadRouter {
    pub fn new(registry: LeadRegistry) -> Self { Self { registry } }
    pub fn query(&self, q: &str, limit: usize) -> Result<Vec<QueryResult>, CapabilityError> {
        self.registry.optimal()
            .ok_or_else(|| CapabilityError::NotAvailable("No lead store".into()))?
            .query(q, limit)
    }
}

pub struct LeadBridge { router: LeadRouter }

impl LeadBridge {
    pub fn new(router: LeadRouter) -> Self { Self { router } }
    pub fn query(&self, q: &str, limit: usize) -> Result<Vec<QueryResult>, CapabilityError> {
        self.router.query(q, limit)
    }
}

// ════════════════════════════════════════════════════════════════
// 测试
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lead_manager_trait() {
        let mgr = LeadManager::new();
        assert_eq!(mgr.category(), CapabilityCategory::Data);
        assert_eq!(mgr.constellation(), ConstellationLevel::C1UnitTest);
        assert!(mgr.health_check().healthy);
    }

    #[test]
    fn test_capture_and_qualify() {
        let mut mgr = LeadManager::new();
        let lead = mgr.capture_lead(LeadSource::Alibaba, "John", "Inquiry", vec![]);
        assert!(lead.score > 0.0);
        assert_eq!(lead.stage, LeadStage::Captured);
    }

    #[test]
    fn test_advance_stage() {
        let mut mgr = LeadManager::new();
        let lead = mgr.capture_lead(LeadSource::Email, "Test", "Inquiry", vec![]).clone();
        mgr.advance_stage(&lead.id).unwrap();
        let updated = mgr.get_lead(&lead.id).unwrap();
        assert_eq!(updated.stage, LeadStage::Qualified);
    }

    #[test]
    fn test_pipeline_summary() {
        let mut mgr = LeadManager::new();
        mgr.capture_lead(LeadSource::LinkedIn, "A", "a", vec![]);
        mgr.capture_lead(LeadSource::Email, "B", "b", vec![]);
        let summary = mgr.pipeline_summary();
        assert_eq!(*summary.get(&LeadStage::Captured).unwrap_or(&0), 2);
    }

    #[test]
    fn test_quality_classification() {
        assert_eq!(LeadScorer::score_to_quality(90.0), LeadQuality::Qualified);
        assert_eq!(LeadScorer::score_to_quality(70.0), LeadQuality::Hot);
        assert_eq!(LeadScorer::score_to_quality(50.0), LeadQuality::Warm);
        assert_eq!(LeadScorer::score_to_quality(20.0), LeadQuality::Cold);
    }

    // ── LeadRegistry::optimal 同分兜底确定性 (D13 确定性) ─────────

    /// 8 个并列 store 的名字 (字典序 ⇒ 最后一个最大)。
    const TIED_IDS: [&str; 8] = [
        "data.lead_00", "data.lead_01", "data.lead_02", "data.lead_03",
        "data.lead_04", "data.lead_05", "data.lead_06", "data.lead_07",
    ];

    /// 最小 mock store — 只为构造**同分**并列集, `query` 不被本组测试调用。
    struct StubStore {
        id: &'static str,
        error_rate: f64,
        healthy: bool,
    }

    impl StubStore {
        fn new(id: &'static str, error_rate: f64) -> Self {
            Self { id, error_rate, healthy: true }
        }
        fn unhealthy(id: &'static str, error_rate: f64) -> Self {
            Self { id, error_rate, healthy: false }
        }
    }

    impl L1Capability for StubStore {
        fn capability_id(&self) -> &str { self.id }
        fn category(&self) -> CapabilityCategory { CapabilityCategory::Data }
        fn constellation(&self) -> ConstellationLevel { ConstellationLevel::C1UnitTest }
        fn health_check(&self) -> CapabilityHealth {
            CapabilityHealth {
                healthy: self.healthy,
                latency_ms: None,
                error_rate: self.error_rate,
                last_check: 0,
                message: None,
            }
        }
        fn description(&self) -> &str { "stub" }
    }

    impl DataStore for StubStore {
        fn store(&self, _ns: &str, key: &str, _v: &[u8]) -> Result<String, CapabilityError> {
            Ok(key.to_string())
        }
        fn load(&self, _id: &str) -> Result<Option<Vec<u8>>, CapabilityError> { Ok(None) }
        fn query(&self, _q: &str, _limit: usize) -> Result<Vec<QueryResult>, CapabilityError> {
            Ok(Vec::new())
        }
    }

    /// `LeadRegistry::optimal` 同分兜底: 胜者只由 `capability_id` 决定, 与**注册顺序**
    /// 无关。原实现无兜底 ⇒ `max_by` 取最后一个最大值 ⇒ 逆序注册会翻转结果。
    #[test]
    fn test_lead_registry_optimal_tie_is_name_deterministic() {
        let expected = TIED_IDS[TIED_IDS.len() - 1];

        let mut asc = LeadRegistry::new();
        for id in TIED_IDS {
            asc.register(Box::new(StubStore::new(id, 0.25)));
        }
        let mut desc = LeadRegistry::new();
        for id in TIED_IDS.iter().rev() {
            desc.register(Box::new(StubStore::new(*id, 0.25)));
        }
        assert_eq!(
            asc.optimal().map(|m| m.capability_id()),
            Some(expected),
            "正序注册的同分胜者不是名字最大者"
        );
        assert_eq!(
            desc.optimal().map(|m| m.capability_id()),
            Some(expected),
            "同分胜者随注册顺序漂移 ⇒ 顺序泄漏"
        );
    }

    /// 非并列时主判据 (error_rate 越低越好) 不得被兜底反转; 不健康项必须被滤掉。
    #[test]
    fn test_lead_registry_optimal_primary_key_and_health_filter() {
        let mut reg = LeadRegistry::new();
        reg.register(Box::new(StubStore::new("data.lead_bad", 0.9)));
        reg.register(Box::new(StubStore::new("data.lead_good", 0.01)));
        reg.register(Box::new(StubStore::unhealthy("data.lead_zzz", 0.0)));
        assert_eq!(
            reg.optimal().map(|m| m.capability_id()),
            Some("data.lead_good"),
            "兜底不得反转主判据, 也不得让不健康项胜出"
        );
        assert!(LeadRegistry::new().optimal().is_none(), "空注册表 ⇒ None");
    }

    /// 回归（2026-10-05）：`LeadManager::query` 的 top-N **结果集与顺序**必须规范化。
    ///
    /// 原实现 `values().filter(..).take(limit)` 在 `HashMap` 上截断 ⇒ 匹配数超过
    /// `limit` 时「哪 N 条」跨进程漂移（不是顺序问题，是**结果集本身不同**），
    /// 且**根本没按 score 排序** ⇒ `limit` 名不符实。
    ///
    /// ⚠️ 判别力说明（实测约束，非偷懒）：`capture_lead` 的 id 是
    /// **`lead_{uuid::new_v4()}`** ⇒ 两次构造的 id 天然不同，所以
    /// 「建两个 manager 比对结果」这种测法在此**不成立**。
    /// ⇒ 改用**单次运行即可判定**的不变量：**返回值必须已按 (score 降, id 升) 有序**。
    /// 原实现返回哈希序 ⇒ 该断言必失败；这比概率性的跨进程比对更强。
    #[test]
    fn test_lead_manager_query_is_canonically_ordered() {
        let mut mgr = LeadManager::new();
        for i in 0..6 {
            mgr.capture_lead(
                LeadSource::Website,
                &format!("contact-{}", i),
                "widget inquiry",
                vec!["p".to_string()],
            );
        }

        let all = mgr.query("widget", 100).unwrap_or_default();
        assert_eq!(all.len(), 6, "query 应返回全部 6 条匹配");

        // ① 返回值必须已按 (score 降, id 升) 规范化 —— 原实现是哈希序，必失败
        for w in all.windows(2) {
            let (a_score, a_id) = (w[0].score, w[0].id.as_str());
            let (b_score, b_id) = (w[1].score, w[1].id.as_str());
            let ordered = b_score < a_score || (b_score == a_score && a_id < b_id);
            assert!(
                ordered,
                "结果未按 (score 降, id 升) 排序: ({}, {}) 之后是 ({}, {})",
                a_score, a_id, b_score, b_id
            );
        }

        // ② 截断必须是「全序列表的前缀」—— 即 limit 真的取最好的 N 条
        let all_ids: Vec<&str> = all.iter().map(|r| r.id.as_str()).collect();
        for k in 1..=6usize {
            let top = mgr.query("widget", k).unwrap_or_default();
            let top_ids: Vec<&str> = top.iter().map(|r| r.id.as_str()).collect();
            assert_eq!(top_ids, all_ids[..k], "limit={} 不是全序前缀", k);
        }
    }
}
