//! L5 外贸全流程编排器 — 编排 L1 能力网 → FT01-FT26 全链路
//!
//! 架构:
//! - L1 能力网提供通用基础: messaging(WhatsApp/Email), social_media, lead_management, kb
//! - L5 编排器将 L1 能力组合为外贸领域工作流
//! - 不重复造轮子: 每个 L1 能力可被其他域复用
//!
//! 流程分组 (7组26步):
//!   G1 获客运营 (FT01-FT05): 社交媒体 → 询盘 → 资质 → 跟进 → 沟通
//!   G2 报价谈判 (FT06-FT09): 需求确认 → 报价 → 谈判 → 合同
//!   G3 收款 (FT10-FT11): 付款方式 → 收款确认
//!   G4 生产 (FT12-FT15): 排产 → 跟单 → 质检 → 放行
//!   G5 物流 (FT16-FT20): 检证 → 订舱 → 报关 → 提单 → 运输
//!   G6 结算 (FT21-FT24): 尾款 → 结汇 → 退税 → 核销
//!   G7 复盘 (FT25-FT26): 订单复盘 → 经验吸收

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::l1_action::nt_io::nt_io_messaging::{MessagingBridge, MessagingRouter, MessagingRegistry, Channel};
use crate::l1_action::nt_act::nt_act_media::{ContentGenerator, ScheduleEngine, SocialAnalytics, Platform};
use crate::l1_action::nt_memory::nt_memory_lead::{LeadManager, Lead, LeadSource, LeadStage, LeadQuality};

// ════════════════════════════════════════════════════════════════
// 全链路状态机
// ════════════════════════════════════════════════════════════════

/// 外贸全链路阶段 (FT01-FT26)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradePhase {
    // ── G1: 获客运营 ──
    Ft01SocialMediaContent,          // 社交媒体内容创作与发布
    Ft02InquiryCapture,              // 询盘捕获 (多渠道统一入口)
    Ft03LeadQualification,           // 询盘资质评分与分级
    Ft04FollowUpNurturing,           // 跟进培育 (WhatsApp/Email)
    Ft05CommunicationEngagement,     // 沟通互动 (建立信任)

    // ── G2: 报价谈判 ──
    Ft06RequirementConfirmation,     // 需求确认
    Ft07DetailedQuotation,           // 详细报价
    Ft08NegotiationObjectionHandling,// 谈判异议处理
    Ft09ContractReviewSigning,       // 合同审核签署

    // ── G3: 收款 ──
    Ft10PaymentArrangement,          // 付款方式协商
    Ft11PaymentCollection,           // 收款确认

    // ── G4: 生产 ──
    Ft12ProductionOrderMaterialPrep, // 生产下单备料
    Ft13ProductionTrackingAlerting,  // 生产跟踪预警
    Ft14QualityInspectionRelease,    // 质量检验放行
    Ft15FinalQualityCheck,           // 出货前终检

    // ── G5: 物流 ──
    Ft16InspectionCertification,     // 检验检疫证书
    Ft17BookingPackingList,          // 订舱装箱
    Ft18CustomsClearance,            // 报关清关
    Ft19BillOfLadingManagement,      // 提单管理
    Ft20ShipmentTracking,            // 运输跟踪

    // ── G6: 结算 ──
    Ft21FinalPaymentCollection,      // 尾款收取
    Ft22SettlementVerification,      // 结汇核销
    Ft23TaxRefundDeclaration,        // 退税申报
    Ft24AccountReconciliation,       // 账务核对

    // ── G7: 复盘 ──
    Ft25OrderReview,                 // 订单复盘
    Ft26ExperienceAbsorption,        // 经验吸收入库
}

impl TradePhase {
    pub fn all() -> Vec<TradePhase> {
        vec![
            TradePhase::Ft01SocialMediaContent,
            TradePhase::Ft02InquiryCapture,
            TradePhase::Ft03LeadQualification,
            TradePhase::Ft04FollowUpNurturing,
            TradePhase::Ft05CommunicationEngagement,
            TradePhase::Ft06RequirementConfirmation,
            TradePhase::Ft07DetailedQuotation,
            TradePhase::Ft08NegotiationObjectionHandling,
            TradePhase::Ft09ContractReviewSigning,
            TradePhase::Ft10PaymentArrangement,
            TradePhase::Ft11PaymentCollection,
            TradePhase::Ft12ProductionOrderMaterialPrep,
            TradePhase::Ft13ProductionTrackingAlerting,
            TradePhase::Ft14QualityInspectionRelease,
            TradePhase::Ft15FinalQualityCheck,
            TradePhase::Ft16InspectionCertification,
            TradePhase::Ft17BookingPackingList,
            TradePhase::Ft18CustomsClearance,
            TradePhase::Ft19BillOfLadingManagement,
            TradePhase::Ft20ShipmentTracking,
            TradePhase::Ft21FinalPaymentCollection,
            TradePhase::Ft22SettlementVerification,
            TradePhase::Ft23TaxRefundDeclaration,
            TradePhase::Ft24AccountReconciliation,
            TradePhase::Ft25OrderReview,
            TradePhase::Ft26ExperienceAbsorption,
        ]
    }

    pub fn group(&self) -> TradeGroup {
        match self {
            TradePhase::Ft01SocialMediaContent
            | TradePhase::Ft02InquiryCapture
            | TradePhase::Ft03LeadQualification
            | TradePhase::Ft04FollowUpNurturing
            | TradePhase::Ft05CommunicationEngagement => TradeGroup::Acquisition,

            TradePhase::Ft06RequirementConfirmation
            | TradePhase::Ft07DetailedQuotation
            | TradePhase::Ft08NegotiationObjectionHandling
            | TradePhase::Ft09ContractReviewSigning => TradeGroup::Negotiation,

            TradePhase::Ft10PaymentArrangement
            | TradePhase::Ft11PaymentCollection => TradeGroup::Payment,

            TradePhase::Ft12ProductionOrderMaterialPrep
            | TradePhase::Ft13ProductionTrackingAlerting
            | TradePhase::Ft14QualityInspectionRelease
            | TradePhase::Ft15FinalQualityCheck => TradeGroup::Production,

            TradePhase::Ft16InspectionCertification
            | TradePhase::Ft17BookingPackingList
            | TradePhase::Ft18CustomsClearance
            | TradePhase::Ft19BillOfLadingManagement
            | TradePhase::Ft20ShipmentTracking => TradeGroup::Logistics,

            TradePhase::Ft21FinalPaymentCollection
            | TradePhase::Ft22SettlementVerification
            | TradePhase::Ft23TaxRefundDeclaration
            | TradePhase::Ft24AccountReconciliation => TradeGroup::Settlement,

            TradePhase::Ft25OrderReview
            | TradePhase::Ft26ExperienceAbsorption => TradeGroup::Review,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            TradePhase::Ft01SocialMediaContent => "FT01",
            TradePhase::Ft02InquiryCapture => "FT02",
            TradePhase::Ft03LeadQualification => "FT03",
            TradePhase::Ft04FollowUpNurturing => "FT04",
            TradePhase::Ft05CommunicationEngagement => "FT05",
            TradePhase::Ft06RequirementConfirmation => "FT06",
            TradePhase::Ft07DetailedQuotation => "FT07",
            TradePhase::Ft08NegotiationObjectionHandling => "FT08",
            TradePhase::Ft09ContractReviewSigning => "FT09",
            TradePhase::Ft10PaymentArrangement => "FT10",
            TradePhase::Ft11PaymentCollection => "FT11",
            TradePhase::Ft12ProductionOrderMaterialPrep => "FT12",
            TradePhase::Ft13ProductionTrackingAlerting => "FT13",
            TradePhase::Ft14QualityInspectionRelease => "FT14",
            TradePhase::Ft15FinalQualityCheck => "FT15",
            TradePhase::Ft16InspectionCertification => "FT16",
            TradePhase::Ft17BookingPackingList => "FT17",
            TradePhase::Ft18CustomsClearance => "FT18",
            TradePhase::Ft19BillOfLadingManagement => "FT19",
            TradePhase::Ft20ShipmentTracking => "FT20",
            TradePhase::Ft21FinalPaymentCollection => "FT21",
            TradePhase::Ft22SettlementVerification => "FT22",
            TradePhase::Ft23TaxRefundDeclaration => "FT23",
            TradePhase::Ft24AccountReconciliation => "FT24",
            TradePhase::Ft25OrderReview => "FT25",
            TradePhase::Ft26ExperienceAbsorption => "FT26",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeGroup {
    Acquisition,    // 获客运营
    Negotiation,    // 报价谈判
    Payment,        // 收款
    Production,     // 生产
    Logistics,      // 物流
    Settlement,     // 结算
    Review,         // 复盘
}

// ════════════════════════════════════════════════════════════════
// 编排上下文
// ════════════════════════════════════════════════════════════════

/// 全链路上下文 — 贯穿整个贸易周期
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeContext {
    pub order_id: String,
    pub current_phase: TradePhase,

    // ── 获客层 (L1 lead + messaging) ──
    pub lead: Option<Lead>,
    pub conversations: Vec<String>, // conversation IDs

    // ── 谈判层 ──
    pub buyer_profile: Option<BuyerProfile>,
    pub quotation: Option<Quotation>,
    pub contract: Option<Contract>,

    // ── 生产物流层 ──
    pub production_status: Option<ProductionStatus>,
    pub logistics: Option<LogisticsInfo>,

    // ── 财务层 ──
    pub payment: Option<PaymentInfo>,
    pub settlement: Option<SettlementInfo>,

    // ── 元数据 ──
    pub created_at: u64,
    pub updated_at: u64,
    pub events: Vec<TradeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyerProfile {
    pub name: String,
    pub company: String,
    pub country: String,
    pub language: String,
    pub channel: String,
    pub first_contact: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quotation {
    pub items: Vec<QuotationItem>,
    pub total_amount: f64,
    pub currency: String,
    pub incoterm: String,
    pub validity_days: u32,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotationItem {
    pub product: String,
    pub quantity: u32,
    pub unit_price: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub contract_number: String,
    pub signed_at: Option<u64>,
    pub terms: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionStatus {
    pub stage: String,
    pub progress_pct: f64,
    pub eta: Option<u64>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogisticsInfo {
    pub vessel: Option<String>,
    pub bl_number: Option<String>,
    pub pol: String,
    pub pod: String,
    pub etd: Option<u64>,
    pub eta: Option<u64>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInfo {
    pub method: String,
    pub total_amount: f64,
    pub paid_amount: f64,
    pub currency: String,
    pub lc_number: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementInfo {
    pub fx_rate: f64,
    pub settled_amount: f64,
    pub tax_refund: Option<f64>,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEvent {
    pub phase: TradePhase,
    pub action: String,
    pub result: String,
    pub timestamp: u64,
}

// ════════════════════════════════════════════════════════════════
// 编排器 — 编排 L1 能力
// ════════════════════════════════════════════════════════════════

/// 外贸全链路编排器 — 编排 L1 能力网
pub struct TradeOrchestrator {
    /// L1: 消息能力桥接
    pub messaging: MessagingBridge,
    /// L1: 社交媒体运营
    pub content_gen: ContentGenerator,
    pub schedule: ScheduleEngine,
    pub analytics: SocialAnalytics,
    /// L1: 询盘管理
    pub leads: LeadManager,
    /// 活跃交易上下文
    pub active_trades: HashMap<String, TradeContext>,
}

impl Default for TradeOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeOrchestrator {
    pub fn new() -> Self {
        let registry = MessagingRegistry::new();
        let router = MessagingRouter::new(registry);
        let messaging = MessagingBridge::new(router);
        Self {
            messaging,
            messaging: MessagingBus::new(),
            content_gen: ContentGenerator::new(crate::l1_action::nt_act::nt_act_media::ContentStrategy {
                name: "Foreign Trade Marketing".into(),
                target_platforms: vec![Platform::LinkedIn, Platform::Instagram, Platform::Alibaba],
                content_pillars: vec![],
                posting_frequency: crate::l1_action::nt_act::nt_act_media::PostingFrequency {
                    posts_per_week: 5,
                    best_times: vec![(9, 0), (14, 0)],
                    content_mix: HashMap::new(),
                },
                hashtag_strategy: crate::l1_action::nt_act::nt_act_media::HashtagStrategy {
                    branded: vec![],
                    industry: vec!["#manufacturing".into(), "#export".into(), "#trade".into()],
                    trending: vec![],
                    max_per_post: 10,
                },
                tone_of_voice: "Professional".into(),
                target_audience: "B2B buyers".into(),
            }),
            schedule: ScheduleEngine::new(),
            analytics: SocialAnalytics::new(),
            leads: LeadManager::new(),
            active_trades: HashMap::new(),
        }
    }

    // ════════════════════════════════════════════════════════════════
    // G1: 获客运营 (FT01-FT05)
    // ════════════════════════════════════════════════════════════════

    /// FT01: 社交媒体内容创作与发布
    pub fn create_social_content(&self, platform: Platform, body: &str) -> crate::l1_action::nt_act::nt_act_media::Post {
        // use crate::l1_action::nt_act::nt_act_media::{Post, ContentType, ContentStatus, EngagementMetrics};
        Post {
            id: uuid::Uuid::new_v4().to_string(),
            platform,
            content_type: ContentType::Text,
            title: None,
            body: body.to_string(),
            hashtags: vec!["#trade".into(), "#export".into()],
            media_urls: vec![],
            link_url: None,
            status: ContentStatus::Draft,
            scheduled_at: None,
            published_at: None,
            engagement: EngagementMetrics::default(),
            metadata: HashMap::new(),
        }
    }

    /// FT02: 询盘捕获
    pub fn capture_inquiry(
        &mut self,
        source: LeadSource,
        contact: &str,
        inquiry: &str,
        products: Vec<String>,
    ) -> String {
        let lead = self.leads.capture_lead(source, contact, inquiry, products);
        lead.id.clone()
    }

    /// FT03: 询盘资质评分
    pub fn qualify_lead(&self, lead_id: &str) -> Option<(f64, LeadQuality)> {
        self.leads.get_lead(lead_id).map(|l| (l.score, l.quality))
    }

    /// FT04: 跟进培育
    pub fn send_follow_up(
        &mut self,
        lead_id: &str,
        channel: Channel,
        template_id: &str,
        vars: &HashMap<String, String>,
    ) -> Result<String, String> {
        let lead = self.leads.get_lead(lead_id)
            .ok_or_else(|| format!("Lead {} not found", lead_id))?;
        let to = match channel {
            Channel::WhatsApp => lead.whatsapp.as_deref().unwrap_or(""),
            Channel::Email => lead.email.as_deref().unwrap_or(""),
            _ => "",
        };
        if to.is_empty() {
            return Err(format!("No {:?} contact for lead {}", channel, lead_id));
        }
        self.messaging.send_template(channel, template_id, to, vars)
    }

    /// FT05: 沟通互动
    pub fn record_communication(
        &mut self,
        lead_id: &str,
        channel: &str,
        direction: &str,
        content: &str,
    ) -> Result<(), String> {
        self.leads.record_interaction(
            lead_id,
            crate::l1_action::nt_memory::nt_memory_lead::InteractionType::Inquiry,
            channel,
            direction,
            content,
        )
    }

    // ════════════════════════════════════════════════════════════════
    // G2-G7: 下游流程 (FT06-FT26)
    // ════════════════════════════════════════════════════════════════

    /// 创建新交易上下文
    pub fn start_trade(&mut self, lead_id: &str) -> Result<String, String> {
        let lead = self.leads.get_lead(lead_id)
            .ok_or_else(|| format!("Lead {} not found", lead_id))?
            .clone();
        let order_id = format!("ORD-{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let ctx = TradeContext {
            order_id: order_id.clone(),
            current_phase: TradePhase::Ft06RequirementConfirmation,
            lead: Some(lead),
            conversations: Vec::new(),
            buyer_profile: None,
            quotation: None,
            contract: None,
            production_status: None,
            logistics: None,
            payment: None,
            settlement: None,
            created_at: now,
            updated_at: now,
            events: Vec::new(),
        };
        self.active_trades.insert(order_id.clone(), ctx);
        Ok(order_id)
    }

    /// 推进到下一阶段
    pub fn advance_phase(&mut self, order_id: &str) -> Result<&TradeContext, String> {
        let ctx = self.active_trades.get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        let phases = TradePhase::all();
        if let Some(idx) = phases.iter().position(|p| *p == ctx.current_phase) {
            if idx + 1 < phases.len() {
                ctx.current_phase = phases[idx + 1];
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                ctx.updated_at = now;
                ctx.events.push(TradeEvent {
                    phase: ctx.current_phase,
                    action: "advance".into(),
                    result: format!("Moved to {:?}", ctx.current_phase),
                    timestamp: now,
                });
            }
        }
        self.active_trades.get(order_id).ok_or_else(|| "Not found".into())
    }

    /// 获取漏斗统计
    pub fn pipeline_summary(&self) -> HashMap<crate::l1_action::nt_memory::nt_memory_lead::LeadStage, usize> {
        self.leads.pipeline_summary()
    }

    /// 活跃交易统计
    pub fn active_trade_count(&self) -> usize { self.active_trades.len() }
}

// ════════════════════════════════════════════════════════════════
// 测试
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trade_phases_count() {
        assert_eq!(TradePhase::all().len(), 26);
    }

    #[test]
    fn test_full_lifecycle() {
        let mut orch = TradeOrchestrator::new();

        // G1: 获客
        let lead_id = orch.capture_inquiry(
            LeadSource::LinkedIn,
            "John Smith",
            "Looking for industrial machinery",
            vec!["machinery".into()],
        );
        let (score, quality) = orch.qualify_lead(&lead_id).unwrap();
        assert!(score > 0.0);

        // 创建交易
        let order_id = orch.start_trade(&lead_id).unwrap();
        assert!(!order_id.is_empty());

        // 推进阶段
        let ctx = orch.advance_phase(&order_id).unwrap();
        assert_eq!(ctx.current_phase, TradePhase::Ft07DetailedQuotation);
    }

    #[test]
    fn test_capture_and_qualify() {
        let mut orch = TradeOrchestrator::new();
        let lead_id = orch.capture_inquiry(
            LeadSource::Alibaba,
            "Test Corp",
            "Need 500 units of widget A",
            vec!["widget".into()],
        );
        let (score, quality) = orch.qualify_lead(&lead_id).unwrap();
        assert!(score >= 20.0, "Alibaba lead should score >= 20: {}", score);
    }

    #[test]
    fn test_trade_groups() {
        assert_eq!(TradePhase::Ft01SocialMediaContent.group(), TradeGroup::Acquisition);
        assert_eq!(TradePhase::Ft07DetailedQuotation.group(), TradeGroup::Negotiation);
        assert_eq!(TradePhase::Ft12ProductionOrderMaterialPrep.group(), TradeGroup::Production);
        assert_eq!(TradePhase::Ft17BookingPackingList.group(), TradeGroup::Logistics);
        assert_eq!(TradePhase::Ft21FinalPaymentCollection.group(), TradeGroup::Settlement);
        assert_eq!(TradePhase::Ft25OrderReview.group(), TradeGroup::Review);
    }
}
