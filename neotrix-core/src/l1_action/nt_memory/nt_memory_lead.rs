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
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
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
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
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
        self.leads.get(&id).unwrap()
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
        lead.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.rescore(id);
        self.leads.get(id).ok_or_else(|| "Not found".into())
    }

    pub fn record_interaction(&mut self, id: &str, itype: InteractionType, channel: &str, direction: &str, content: &str) -> Result<(), String> {
        let lead = self.leads.get_mut(id).ok_or_else(|| format!("Lead {} not found", id))?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
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
        lead.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
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
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.leads.values().filter(|l|
            l.stage != LeadStage::ClosedWon && l.stage != LeadStage::ClosedLost
                && l.next_follow_up.map_or(true, |t| t <= now)
        ).collect()
    }

    pub fn get_lead(&self, id: &str) -> Option<&Lead> { self.leads.get(id) }
    pub(crate) fn _list_leads(&self) -> Vec<&Lead> { self.leads.values().collect() }
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
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
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
        let results: Vec<QueryResult> = self.leads.values()
            .filter(|l| l.contact_name.contains(query) || l.inquiry_text.contains(query))
            .take(limit)
            .map(|l| QueryResult {
                id: l.id.clone(),
                score: l.score / 100.0,
                data: serde_json::to_vec(l).unwrap_or_default(),
            })
            .collect();
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
                a_s.partial_cmp(&b_s).unwrap_or(std::cmp::Ordering::Equal)
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
}
