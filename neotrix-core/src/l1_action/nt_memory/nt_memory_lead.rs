//! L1 询盘管理 — 询盘捕获 + 资质评分 + CRM + 跟进漏斗
//!
//! 通用能力: 任何域(外贸/销售/市场)都可调用
//! 设计原则: Lead trait 抽象 → 多来源可插拔, 评分模型 + 漏斗追踪

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

// ════════════════════════════════════════════════════════════════
// 询盘模型
// ════════════════════════════════════════════════════════════════

/// 询盘来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeadSource {
    Website,
    LinkedIn,
    Instagram,
    Alibaba,
    MadeInChina,
    GlobalSources,
    TradeShow,
    WhatsApp,
    Email,
    Phone,
    Referral,
    ColdOutreach,
    GoogleAds,
    FacebookAds,
}

/// 询盘质量等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LeadQuality {
    Cold,       // 冷线索, 需培育
    Warm,       // 温线索, 有意向
    Hot,        // 热线索, 准备成交
    Qualified,  // 已验证, 高价值
}

/// 询盘阶段 (销售漏斗)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LeadStage {
    Captured,       // 初次捕获
    Qualified,      // 资质验证
    Engaged,        // 已建立联系
    ProposalSent,   // 已发报价
    Negotiating,    // 谈判中
    ClosedWon,      // 成交
    ClosedLost,     // 流失
    Nurturing,      // 培育中
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
    pub language: Option<String>,
    pub product_interest: Vec<String>,
    pub inquiry_text: String,
    pub quality: LeadQuality,
    pub stage: LeadStage,
    pub score: f64,
    pub tags: Vec<String>,
    pub interactions: Vec<Interaction>,
    pub notes: Vec<Note>,
    pub assigned_to: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub last_contact: Option<u64>,
    pub next_follow_up: Option<u64>,
    pub metadata: HashMap<String, String>,
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
    pub sentiment: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionType {
    Inquiry,
    Reply,
    FollowUp,
    Call,
    Meeting,
    Proposal,
    Order,
    Complaint,
    Feedback,
}

/// 备注
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub author: String,
    pub content: String,
    pub timestamp: u64,
    pub pinned: bool,
}

// ════════════════════════════════════════════════════════════════
// 评分模型
// ════════════════════════════════════════════════════════════════

/// 询盘评分规则
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

/// 评分引擎
pub struct LeadScorer {
    rules: Vec<ScoringRule>,
}

impl Default for LeadScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl LeadScorer {
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
        }
    }

    fn default_rules() -> Vec<ScoringRule> {
        vec![
            // 来源加分
            ScoringRule { field: "source".into(), condition: ScoringCondition::InList(vec!["alibaba".into(), "trade_show".into(), "referral".into()]), points: 20.0 },
            ScoringRule { field: "source".into(), condition: ScoringCondition::InList(vec!["linkedin".into(), "email".into()]), points: 15.0 },
            ScoringRule { field: "source".into(), condition: ScoringCondition::InList(vec!["website".into(), "google_ads".into()]), points: 10.0 },
            // 信息完整度
            ScoringRule { field: "email".into(), condition: ScoringCondition::Exists, points: 10.0 },
            ScoringRule { field: "phone".into(), condition: ScoringCondition::Exists, points: 10.0 },
            ScoringRule { field: "company_name".into(), condition: ScoringCondition::Exists, points: 15.0 },
            ScoringRule { field: "whatsapp".into(), condition: ScoringCondition::Exists, points: 10.0 },
            // 互动加分
            ScoringRule { field: "interaction_count".into(), condition: ScoringCondition::GreaterThan(3.0), points: 15.0 },
            ScoringRule { field: "has_reply".into(), condition: ScoringCondition::Exists, points: 20.0 },
        ]
    }

    pub fn add_rule(&mut self, rule: ScoringRule) {
        self.rules.push(rule);
    }

    pub fn score_lead(&self, lead: &Lead) -> f64 {
        let mut total = 0.0;
        for rule in &self.rules {
            if self.evaluate_rule(lead, rule) {
                total += rule.points;
            }
        }
        // 互动频率加分
        let interaction_bonus = (lead.interactions.len() as f64 * 2.0).min(20.0);
        total += interaction_bonus;
        // 最近联系加分
        if let Some(last) = lead.last_contact {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let days_since = (now - last) / 86400;
            if days_since <= 1 { total += 10.0; }
            else if days_since <= 7 { total += 5.0; }
        }
        total.min(100.0)
    }

    fn evaluate_rule(&self, lead: &Lead, rule: &ScoringRule) -> bool {
        match rule.field.as_str() {
            "source" => {
                let source_str = format!("{:?}", lead.source).to_lowercase();
                match &rule.condition {
                    ScoringCondition::InList(list) => list.contains(&source_str),
                    ScoringCondition::Equals(v) => source_str == *v,
                    _ => false,
                }
            }
            "email" => lead.email.is_some(),
            "phone" => lead.phone.is_some(),
            "company_name" => lead.company_name.is_some(),
            "whatsapp" => lead.whatsapp.is_some(),
            "interaction_count" => {
                match &rule.condition {
                    ScoringCondition::GreaterThan(threshold) => lead.interactions.len() as f64 > *threshold,
                    _ => false,
                }
            }
            "has_reply" => lead.interactions.iter().any(|i| i.direction == "outbound"),
            _ => false,
        }
    }

    /// 评分 → 质量等级
    pub fn score_to_quality(score: f64) -> LeadQuality {
        if score >= 80.0 { LeadQuality::Qualified }
        else if score >= 60.0 { LeadQuality::Hot }
        else if score >= 40.0 { LeadQuality::Warm }
        else { LeadQuality::Cold }
    }
}

// ════════════════════════════════════════════════════════════════
// CRM — 询盘管理器
// ════════════════════════════════════════════════════════════════

/// CRM — 询盘管理 + 漏斗追踪
pub struct LeadManager {
    leads: HashMap<String, Lead>,
    scorer: LeadScorer,
    pipeline: Vec<LeadStage>,
}

impl Default for LeadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LeadManager {
    pub fn new() -> Self {
        Self {
            leads: HashMap::new(),
            scorer: LeadScorer::new(),
            pipeline: vec![
                LeadStage::Captured,
                LeadStage::Qualified,
                LeadStage::Engaged,
                LeadStage::ProposalSent,
                LeadStage::Negotiating,
                LeadStage::ClosedWon,
            ],
        }
    }

    /// 捕获新询盘
    pub fn capture_lead(
        &mut self,
        source: LeadSource,
        contact_name: &str,
        inquiry_text: &str,
        product_interest: Vec<String>,
    ) -> &Lead {
        let id = format!("lead_{}", uuid::Uuid::new_v4());
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let lead = Lead {
            id: id.clone(),
            source,
            company_name: None,
            contact_name: contact_name.to_string(),
            email: None,
            phone: None,
            whatsapp: None,
            country: None,
            language: None,
            product_interest,
            inquiry_text: inquiry_text.to_string(),
            quality: LeadQuality::Cold,
            stage: LeadStage::Captured,
            score: 0.0,
            tags: Vec::new(),
            interactions: Vec::new(),
            notes: Vec::new(),
            assigned_to: None,
            created_at: now,
            updated_at: now,
            last_contact: None,
            next_follow_up: None,
            metadata: HashMap::new(),
        };
        self.leads.insert(id.clone(), lead);
        // 自动评分
        self.rescore_lead(&id);
        self.leads.get(&id).unwrap()
    }

    /// 更新询盘信息
    pub fn update_lead(&mut self, lead_id: &str, updates: LeadUpdate) -> Result<&Lead, String> {
        let lead = self.leads.get_mut(lead_id)
            .ok_or_else(|| format!("Lead {} not found", lead_id))?;
        if let Some(name) = updates.company_name { lead.company_name = Some(name); }
        if let Some(email) = updates.email { lead.email = Some(email); }
        if let Some(phone) = updates.phone { lead.phone = Some(phone); }
        if let Some(whatsapp) = updates.whatsapp { lead.whatsapp = Some(whatsapp); }
        if let Some(country) = updates.country { lead.country = Some(country); }
        if let Some(stage) = updates.stage { lead.stage = stage; }
        if let Some(tags) = updates.tags { lead.tags = tags; }
        if let Some(assignee) = updates.assigned_to { lead.assigned_to = Some(assignee); }
        lead.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.rescore_lead(lead_id);
        self.leads.get(lead_id).ok_or_else(|| "Not found".into())
    }

    /// 记录互动
    pub fn record_interaction(
        &mut self,
        lead_id: &str,
        interaction_type: InteractionType,
        channel: &str,
        direction: &str,
        content: &str,
    ) -> Result<(), String> {
        let lead = self.leads.get_mut(lead_id)
            .ok_or_else(|| format!("Lead {} not found", lead_id))?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        lead.interactions.push(Interaction {
            id: format!("int_{}", uuid::Uuid::new_v4()),
            interaction_type,
            channel: channel.to_string(),
            direction: direction.to_string(),
            content: content.to_string(),
            timestamp: now,
            sentiment: None,
        });
        lead.last_contact = Some(now);
        lead.updated_at = now;
        self.rescore_lead(lead_id);
        Ok(())
    }

    /// 推进阶段
    pub fn advance_stage(&mut self, lead_id: &str) -> Result<&Lead, String> {
        let lead = self.leads.get_mut(lead_id)
            .ok_or_else(|| format!("Lead {} not found", lead_id))?;
        let current_idx = self.pipeline.iter().position(|s| *s == lead.stage).unwrap_or(0);
        if current_idx + 1 < self.pipeline.len() {
            lead.stage = self.pipeline[current_idx + 1];
        }
        lead.updated_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.leads.get(lead_id).ok_or_else(|| "Not found".into())
    }

    fn rescore_lead(&mut self, lead_id: &str) {
        if let Some(lead) = self.leads.get_mut(lead_id) {
            let score = self.scorer.score_lead(lead);
            lead.score = score;
            lead.quality = LeadScorer::score_to_quality(score);
        }
    }

    /// 漏斗统计
    pub fn pipeline_summary(&self) -> HashMap<LeadStage, usize> {
        let mut summary = HashMap::new();
        for lead in self.leads.values() {
            *summary.entry(lead.stage).or_insert(0) += 1;
        }
        summary
    }

    /// 按质量筛选
    pub fn leads_by_quality(&self, quality: LeadQuality) -> Vec<&Lead> {
        self.leads.values().filter(|l| l.quality == quality).collect()
    }

    /// 按阶段筛选
    pub fn leads_by_stage(&self, stage: LeadStage) -> Vec<&Lead> {
        self.leads.values().filter(|l| l.stage == stage).collect()
    }

    /// 获取需跟进的询盘
    pub fn needs_follow_up(&self) -> Vec<&Lead> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.leads.values()
            .filter(|l| {
                l.stage != LeadStage::ClosedWon && l.stage != LeadStage::ClosedLost
                    && l.next_follow_up.map_or(true, |t| t <= now)
            })
            .collect()
    }

    /// 获取所有询盘
    pub fn list_leads(&self) -> Vec<&Lead> {
        self.leads.values().collect()
    }

    pub fn get_lead(&self, lead_id: &str) -> Option<&Lead> {
        self.leads.get(lead_id)
    }

    pub fn total_leads(&self) -> usize { self.leads.len() }
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
// 测试
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_lead() {
        let mut mgr = LeadManager::new();
        let lead = mgr.capture_lead(
            LeadSource::Alibaba,
            "John Smith",
            "Looking for industrial machinery",
            vec!["machinery".into()],
        );
        assert_eq!(lead.stage, LeadStage::Captured);
        assert!(lead.score > 0.0);
    }

    #[test]
    fn test_lead_scoring() {
        let mut mgr = LeadManager::new();
        let lead = mgr.capture_lead(
            LeadSource::TradeShow,
            "John",
            "Inquiry",
            vec![],
        ).clone();

        // Add info to boost score
        mgr.update_lead(&lead.id, LeadUpdate {
            email: Some("john@example.com".into()),
            phone: Some("+1234567890".into()),
            company_name: Some("ACME Corp".into()),
            ..Default::default()
        }).unwrap();

        let updated = mgr.get_lead(&lead.id).unwrap();
        assert!(updated.score >= 60.0, "Score should be >= 60 with contact info: {}", updated.score);
    }

    #[test]
    fn test_advance_stage() {
        let mut mgr = LeadManager::new();
        let lead = mgr.capture_lead(
            LeadSource::Email,
            "Test",
            "Inquiry",
            vec![],
        ).clone();

        mgr.advance_stage(&lead.id).unwrap();
        let updated = mgr.get_lead(&lead.id).unwrap();
        assert_eq!(updated.stage, LeadStage::Qualified);
    }

    #[test]
    fn test_pipeline_summary() {
        let mut mgr = LeadManager::new();
        mgr.capture_lead(LeadSource::LinkedIn, "A", "a", vec![]);
        mgr.capture_lead(LeadSource::Email, "B", "b", vec![]);
        mgr.capture_lead(LeadSource::WhatsApp, "C", "c", vec![]);

        let summary = mgr.pipeline_summary();
        assert_eq!(*summary.get(&LeadStage::Captured).unwrap_or(&0), 3);
    }

    #[test]
    fn test_needs_follow_up() {
        let mut mgr = LeadManager::new();
        let lead = mgr.capture_lead(
            LeadSource::Website,
            "Test",
            "Inquiry",
            vec![],
        ).clone();

        // Set follow-up to past
        mgr.update_lead(&lead.id, LeadUpdate {
            ..Default::default()
        }).unwrap();

        let pending = mgr.needs_follow_up();
        assert!(!pending.is_empty());
    }

    #[test]
    fn test_lead_quality_classification() {
        assert_eq!(LeadScorer::score_to_quality(90.0), LeadQuality::Qualified);
        assert_eq!(LeadScorer::score_to_quality(70.0), LeadQuality::Hot);
        assert_eq!(LeadScorer::score_to_quality(50.0), LeadQuality::Warm);
        assert_eq!(LeadScorer::score_to_quality(20.0), LeadQuality::Cold);
    }
}
