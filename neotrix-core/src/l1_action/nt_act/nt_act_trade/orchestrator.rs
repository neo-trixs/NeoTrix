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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::l1_action::nt_act::nt_act_media::{
    ContentGenerator, Platform, ScheduleEngine, SocialAnalytics,
};
use crate::l1_action::nt_act::nt_act_trade::finance_compliance::{
    FinanceEngine, PaymentType, RefundDocument, TaxRefundClaim as FCTaxRefundClaim,
};
use crate::l1_action::nt_act::nt_act_trade::production_logistics::{
    BillOfLading as PLBillOfLading, BomRequirement, BookingConfirmation as PLBookingConfirmation,
    CiqCertificate, CiqStatus, CustomsDeclaration as PLCustomsDeclaration, DailyProgress,
    LogisticsEngine, PackingList as PLPackingList, ProductionEngine, ProductionOrder,
    ProductionSchedule,
};
use crate::l1_action::nt_act::nt_act_trade::quote_negotiation::{
    ObjectionCategory, QuoteGenerator,
};
use crate::l1_action::nt_act::nt_act_trade::trade_core::{
    Concession, NegotiationEngine,
};
use crate::l1_action::nt_io::nt_io_messaging::{
    Channel, MessagingBridge, MessagingRegistry, MessagingRouter,
};
use crate::l1_action::nt_memory::nt_memory_lead::{Lead, LeadManager, LeadQuality, LeadSource};

// ════════════════════════════════════════════════════════════════
// 全链路状态机
// ════════════════════════════════════════════════════════════════

/// 外贸全链路阶段 (FT01-FT26) — 区分 full_cycle::TradePhase (FT01-FT17)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradePhase26 {
    // ── G1: 获客运营 ──
    Ft01SocialMediaContent,      // 社交媒体内容创作与发布
    Ft02InquiryCapture,          // 询盘捕获 (多渠道统一入口)
    Ft03LeadQualification,       // 询盘资质评分与分级
    Ft04FollowUpNurturing,       // 跟进培育 (WhatsApp/Email)
    Ft05CommunicationEngagement, // 沟通互动 (建立信任)

    // ── G2: 报价谈判 ──
    Ft06RequirementConfirmation,      // 需求确认
    Ft07DetailedQuotation,            // 详细报价
    Ft08NegotiationObjectionHandling, // 谈判异议处理
    Ft09ContractReviewSigning,        // 合同审核签署

    // ── G3: 收款 ──
    Ft10PaymentArrangement, // 付款方式协商
    Ft11PaymentCollection,  // 收款确认

    // ── G4: 生产 ──
    Ft12ProductionOrderMaterialPrep, // 生产下单备料
    Ft13ProductionTrackingAlerting,  // 生产跟踪预警
    Ft14QualityInspectionRelease,    // 质量检验放行
    Ft15FinalQualityCheck,           // 出货前终检

    // ── G5: 物流 ──
    Ft16InspectionCertification, // 检验检疫证书
    Ft17BookingPackingList,      // 订舱装箱
    Ft18CustomsClearance,        // 报关清关
    Ft19BillOfLadingManagement,  // 提单管理
    Ft20ShipmentTracking,        // 运输跟踪

    // ── G6: 结算 ──
    Ft21FinalPaymentCollection, // 尾款收取
    Ft22SettlementVerification, // 结汇核销
    Ft23TaxRefundDeclaration,   // 退税申报
    Ft24AccountReconciliation,  // 账务核对

    // ── G7: 复盘 ──
    Ft25OrderReview,          // 订单复盘
    Ft26ExperienceAbsorption, // 经验吸收入库
}

impl TradePhase26 {
    pub fn all() -> Vec<TradePhase26> {
        vec![
            TradePhase26::Ft01SocialMediaContent,
            TradePhase26::Ft02InquiryCapture,
            TradePhase26::Ft03LeadQualification,
            TradePhase26::Ft04FollowUpNurturing,
            TradePhase26::Ft05CommunicationEngagement,
            TradePhase26::Ft06RequirementConfirmation,
            TradePhase26::Ft07DetailedQuotation,
            TradePhase26::Ft08NegotiationObjectionHandling,
            TradePhase26::Ft09ContractReviewSigning,
            TradePhase26::Ft10PaymentArrangement,
            TradePhase26::Ft11PaymentCollection,
            TradePhase26::Ft12ProductionOrderMaterialPrep,
            TradePhase26::Ft13ProductionTrackingAlerting,
            TradePhase26::Ft14QualityInspectionRelease,
            TradePhase26::Ft15FinalQualityCheck,
            TradePhase26::Ft16InspectionCertification,
            TradePhase26::Ft17BookingPackingList,
            TradePhase26::Ft18CustomsClearance,
            TradePhase26::Ft19BillOfLadingManagement,
            TradePhase26::Ft20ShipmentTracking,
            TradePhase26::Ft21FinalPaymentCollection,
            TradePhase26::Ft22SettlementVerification,
            TradePhase26::Ft23TaxRefundDeclaration,
            TradePhase26::Ft24AccountReconciliation,
            TradePhase26::Ft25OrderReview,
            TradePhase26::Ft26ExperienceAbsorption,
        ]
    }

    pub fn group(&self) -> TradeGroup {
        match self {
            TradePhase26::Ft01SocialMediaContent
            | TradePhase26::Ft02InquiryCapture
            | TradePhase26::Ft03LeadQualification
            | TradePhase26::Ft04FollowUpNurturing
            | TradePhase26::Ft05CommunicationEngagement => TradeGroup::Acquisition,

            TradePhase26::Ft06RequirementConfirmation
            | TradePhase26::Ft07DetailedQuotation
            | TradePhase26::Ft08NegotiationObjectionHandling
            | TradePhase26::Ft09ContractReviewSigning => TradeGroup::Negotiation,

            TradePhase26::Ft10PaymentArrangement | TradePhase26::Ft11PaymentCollection => {
                TradeGroup::Payment
            }

            TradePhase26::Ft12ProductionOrderMaterialPrep
            | TradePhase26::Ft13ProductionTrackingAlerting
            | TradePhase26::Ft14QualityInspectionRelease
            | TradePhase26::Ft15FinalQualityCheck => TradeGroup::Production,

            TradePhase26::Ft16InspectionCertification
            | TradePhase26::Ft17BookingPackingList
            | TradePhase26::Ft18CustomsClearance
            | TradePhase26::Ft19BillOfLadingManagement
            | TradePhase26::Ft20ShipmentTracking => TradeGroup::Logistics,

            TradePhase26::Ft21FinalPaymentCollection
            | TradePhase26::Ft22SettlementVerification
            | TradePhase26::Ft23TaxRefundDeclaration
            | TradePhase26::Ft24AccountReconciliation => TradeGroup::Settlement,

            TradePhase26::Ft25OrderReview | TradePhase26::Ft26ExperienceAbsorption => {
                TradeGroup::Review
            }
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            TradePhase26::Ft01SocialMediaContent => "FT01",
            TradePhase26::Ft02InquiryCapture => "FT02",
            TradePhase26::Ft03LeadQualification => "FT03",
            TradePhase26::Ft04FollowUpNurturing => "FT04",
            TradePhase26::Ft05CommunicationEngagement => "FT05",
            TradePhase26::Ft06RequirementConfirmation => "FT06",
            TradePhase26::Ft07DetailedQuotation => "FT07",
            TradePhase26::Ft08NegotiationObjectionHandling => "FT08",
            TradePhase26::Ft09ContractReviewSigning => "FT09",
            TradePhase26::Ft10PaymentArrangement => "FT10",
            TradePhase26::Ft11PaymentCollection => "FT11",
            TradePhase26::Ft12ProductionOrderMaterialPrep => "FT12",
            TradePhase26::Ft13ProductionTrackingAlerting => "FT13",
            TradePhase26::Ft14QualityInspectionRelease => "FT14",
            TradePhase26::Ft15FinalQualityCheck => "FT15",
            TradePhase26::Ft16InspectionCertification => "FT16",
            TradePhase26::Ft17BookingPackingList => "FT17",
            TradePhase26::Ft18CustomsClearance => "FT18",
            TradePhase26::Ft19BillOfLadingManagement => "FT19",
            TradePhase26::Ft20ShipmentTracking => "FT20",
            TradePhase26::Ft21FinalPaymentCollection => "FT21",
            TradePhase26::Ft22SettlementVerification => "FT22",
            TradePhase26::Ft23TaxRefundDeclaration => "FT23",
            TradePhase26::Ft24AccountReconciliation => "FT24",
            TradePhase26::Ft25OrderReview => "FT25",
            TradePhase26::Ft26ExperienceAbsorption => "FT26",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeGroup {
    Acquisition, // 获客运营
    Negotiation, // 报价谈判
    Payment,     // 收款
    Production,  // 生产
    Logistics,   // 物流
    Settlement,  // 结算
    Review,      // 复盘
}

impl std::fmt::Display for TradeGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Acquisition => write!(f, "获客运营"),
            Self::Negotiation => write!(f, "报价谈判"),
            Self::Payment => write!(f, "收款"),
            Self::Production => write!(f, "生产"),
            Self::Logistics => write!(f, "物流"),
            Self::Settlement => write!(f, "结算"),
            Self::Review => write!(f, "复盘"),
        }
    }
}

impl std::fmt::Display for TradePhase26 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ════════════════════════════════════════════════════════════════
// 编排上下文
// ════════════════════════════════════════════════════════════════

/// 全链路上下文 — 贯穿整个贸易周期 (orchestrator 专用，区分 full_cycle::TradeContext)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchTradeContext {
    pub order_id: String,
    pub current_phase: TradePhase26,

    // ── 获客层 (L1 lead + messaging) ──
    pub lead: Option<Lead>,
    pub conversations: Vec<String>, // conversation IDs

    // ── 谈判层 ──
    pub buyer_profile: Option<OrchBuyerProfile>,
    pub quotation: Option<Quotation>,
    pub contract: Option<OrchContract>,

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

/// Buyer profile (orchestrator 专用，区分 full_cycle::BuyerProfile)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchBuyerProfile {
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

/// Contract (orchestrator 专用，区分 full_cycle::Contract)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchContract {
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
    pub phase: TradePhase26,
    pub action: String,
    pub result: String,
    pub timestamp: u64,
}

// ════════════════════════════════════════════════════════════════
// 编排器 — 编排 L1 能力
// ════════════════════════════════════════════════════════════════

/// 复盘报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderReview {
    pub order_id: String,
    pub phases_completed: u32,
    pub total_phases: u32,
    pub success: bool,
    pub lessons_learned: Vec<String>,
    pub recommendations: Vec<String>,
}

/// 外贸全链路编排器 — 编排 L1 能力网 + 3 Notable 子技能
pub struct TradeOrchestrator {
    /// L1: 消息能力桥接
    pub messaging: MessagingBridge,
    /// L1: 社交媒体运营
    pub content_gen: ContentGenerator,
    pub schedule: ScheduleEngine,
    pub analytics: SocialAnalytics,
    /// L1: 询盘管理
    pub leads: LeadManager,
    /// Notable 子技能: 报价谈判 (G2)
    pub quote_engine: QuoteGenerator,
    pub negotiation_engine: NegotiationEngine,
    /// Notable 子技能: 生产物流 (G4-G5)
    pub production_engine: ProductionEngine,
    pub logistics_engine: LogisticsEngine,
    /// Notable 子技能: 财务合规 (G3, G6)
    pub finance_engine: FinanceEngine,
    /// 活跃交易上下文
    pub active_trades: HashMap<String, OrchTradeContext>,
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
            content_gen: ContentGenerator::new(
                crate::l1_action::nt_act::nt_act_media::ContentStrategy {
                    name: "Foreign Trade Marketing".into(),
                    target_platforms: vec![
                        Platform::LinkedIn,
                        Platform::Instagram,
                        Platform::Alibaba,
                    ],
                    content_pillars: vec![],
                    posting_frequency: crate::l1_action::nt_act::nt_act_media::PostingFrequency {
                        posts_per_week: 5,
                        best_times: vec![(9, 0), (14, 0)],
                    },
                    hashtag_strategy: crate::l1_action::nt_act::nt_act_media::HashtagStrategy {
                        branded: vec![],
                        industry: vec!["#manufacturing".into(), "#export".into(), "#trade".into()],
                        max_per_post: 10,
                    },
                    tone_of_voice: "Professional".into(),
                    target_audience: "B2B buyers".into(),
                },
            ),
            schedule: ScheduleEngine::new(),
            analytics: SocialAnalytics::new(),
            leads: LeadManager::new(),
            // Notable 子技能初始化
            quote_engine: QuoteGenerator {
                base_cost: 0.0,
                margin_target: 0.3,
                incoterms: "FOB".into(),
                currency: "USD".into(),
                validity_days: 30,
                cost_breakdown: super::trade_core::CostBreakdown {
                    material: 0.0,
                    labor: 0.0,
                    overhead: 0.0,
                    packaging: 0.0,
                    logistics: 0.0,
                    certification: 0.0,
                    contingency: 0.0,
                    total: 0.0,
                },
            },
            negotiation_engine: NegotiationEngine::default(),
            production_engine: ProductionEngine::default(),
            logistics_engine: LogisticsEngine::default(),
            finance_engine: FinanceEngine::default(),
            active_trades: HashMap::new(),
        }
    }

    // ════════════════════════════════════════════════════════════════
    // G1: 获客运营 (FT01-FT05)
    // ════════════════════════════════════════════════════════════════

    /// FT01: 社交媒体内容创作与发布
    pub(crate) fn _create_social_content(
        &self,
        platform: Platform,
        body: &str,
    ) -> crate::l1_action::traits::Post {
        crate::l1_action::traits::Post {
            id: uuid::Uuid::new_v4().to_string(),
            platform: format!("{:?}", platform).to_lowercase(),
            body: body.to_string(),
            hashtags: vec!["#trade".into(), "#export".into()],
            status: "draft".into(),
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
    pub(crate) fn _send_follow_up(
        &mut self,
        lead_id: &str,
        channel: Channel,
        template_id: &str,
        vars: &HashMap<String, String>,
    ) -> Result<String, String> {
        let lead = self
            .leads
            .get_lead(lead_id)
            .ok_or_else(|| format!("Lead {} not found", lead_id))?;
        let to = match channel {
            Channel::WhatsApp => lead.whatsapp.as_deref().unwrap_or(""),
            Channel::Email => lead.email.as_deref().unwrap_or(""),
            _ => "",
        };
        if to.is_empty() {
            return Err(format!("No {:?} contact for lead {}", channel, lead_id));
        }
        self.messaging
            .send_template(channel, template_id, to, vars)
            .map_err(|e| format!("{:?}", e))
    }

    /// FT05: 沟通互动
    pub(crate) fn _record_communication(
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
        let lead = self
            .leads
            .get_lead(lead_id)
            .ok_or_else(|| format!("Lead {} not found", lead_id))?
            .clone();
        let order_id = format!(
            "ORD-{}",
            uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
        );
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("system time error: {}", e))?
            .as_secs();
        let ctx = OrchTradeContext {
            order_id: order_id.clone(),
            current_phase: TradePhase26::Ft06RequirementConfirmation,
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
    pub fn advance_phase(&mut self, order_id: &str) -> Result<&OrchTradeContext, String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        let phases = TradePhase26::all();
        if let Some(idx) = phases.iter().position(|p| *p == ctx.current_phase) {
            if idx + 1 < phases.len() {
                ctx.current_phase = phases[idx + 1];
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| format!("system time error: {}", e))?
                    .as_secs();
                ctx.updated_at = now;
                ctx.events.push(TradeEvent {
                    phase: ctx.current_phase,
                    action: "advance".into(),
                    result: format!("Moved to {:?}", ctx.current_phase),
                    timestamp: now,
                });
            }
        }
        self.active_trades
            .get(order_id)
            .ok_or_else(|| "Not found".into())
    }

    /// 获取漏斗统计
    pub fn pipeline_summary(
        &self,
    ) -> HashMap<crate::l1_action::nt_memory::nt_memory_lead::LeadStage, usize> {
        self.leads.pipeline_summary()
    }

    /// 活跃交易统计
    pub(crate) fn _active_trade_count(&self) -> usize {
        self.active_trades.len()
    }

    // ════════════════════════════════════════════════════════════════
    // G2: 报价谈判 (FT06-FT09)
    // ════════════════════════════════════════════════════════════════

    /// FT06: 需求确认
    pub fn confirm_requirements(
        &mut self,
        order_id: &str,
        requirements: &[String],
    ) -> Result<(), String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        ctx.buyer_profile = Some(OrchBuyerProfile {
            name: ctx
                .buyer_profile
                .as_ref()
                .map(|b| b.name.clone())
                .unwrap_or_default(),
            company: ctx
                .buyer_profile
                .as_ref()
                .map(|b| b.company.clone())
                .unwrap_or_default(),
            country: ctx
                .buyer_profile
                .as_ref()
                .map(|b| b.country.clone())
                .unwrap_or_default(),
            language: ctx
                .buyer_profile
                .as_ref()
                .map(|b| b.language.clone())
                .unwrap_or_default(),
            channel: ctx
                .buyer_profile
                .as_ref()
                .map(|b| b.channel.clone())
                .unwrap_or_default(),
            first_contact: ctx
                .buyer_profile
                .as_ref()
                .map(|b| b.first_contact)
                .unwrap_or(0),
        });
        ctx.conversations.push(format!(
            "FT06_RequirementConfirmation:{}",
            requirements.join(",")
        ));
        ctx.quotation = None;
        ctx.contract = None;
        ctx.production_status = None;
        ctx.logistics = None;
        ctx.payment = None;
        ctx.settlement = None;
        ctx.current_phase = TradePhase26::Ft07DetailedQuotation;
        Ok(())
    }

    /// FT07: 详细报价
    pub fn generate_quotation(
        &mut self,
        order_id: &str,
        items: &[QuotationItem],
    ) -> Result<Quotation, String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        let total: f64 = items.iter().map(|i| i.amount).sum();
        let quotation = Quotation {
            items: items.to_vec(),
            total_amount: total,
            currency: "USD".into(),
            incoterm: "FOB".into(),
            validity_days: 30,
            notes: "Generated quotation".into(),
        };
        ctx.quotation = Some(quotation.clone());
        ctx.conversations.push(format!("FT07_Quotation:{}", total));
        Ok(quotation)
    }

    /// FT08: 谈判异议处理
    pub fn handle_objection(
        &mut self,
        order_id: &str,
        objection: &ObjectionCategory,
        _concession: &Concession,
    ) -> Result<(), String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        // TODO: convert ObjectionCategory to Objection type
        // self.negotiation_engine.handle_objection(objection);
        ctx.conversations
            .push(format!("FT08_Objection:{:?}", objection));
        // TODO: implement is_resolved check
        // if self.negotiation_engine.is_resolved() {
        //     ctx.current_phase = TradePhase26::Ft09ContractReviewSigning;
        // }
        Ok(())
    }

    /// FT09: 合同审核签署
    pub(crate) fn _review_and_sign_contract(
        &mut self,
        order_id: &str,
        terms: &str,
    ) -> Result<OrchContract, String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        let contract = OrchContract {
            contract_number: format!("CON-{:06}", ctx.events.len() + 1),
            signed_at: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| format!("system time error: {}", e))?
                    .as_secs(),
            ),
            terms: terms.to_string(),
        };
        ctx.contract = Some(contract.clone());
        ctx.conversations.push("FT09_ContractSigned".into());
        ctx.current_phase = TradePhase26::Ft10PaymentArrangement;
        Ok(contract)
    }

    // ════════════════════════════════════════════════════════════════
    // G3: 收款 (FT10-FT11)
    // ════════════════════════════════════════════════════════════════

    /// FT10: 付款方式协商
    pub fn arrange_payment(
        &mut self,
        order_id: &str,
        method: PaymentType,
        lc_number: Option<String>,
    ) -> Result<(), String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        ctx.payment = Some(PaymentInfo {
            method: format!("{:?}", method),
            total_amount: ctx.quotation.as_ref().map_or(0.0, |q| q.total_amount),
            paid_amount: 0.0,
            currency: ctx
                .quotation
                .as_ref()
                .map_or_else(|| "USD".to_string(), |q| q.currency.clone()),
            lc_number,
            status: "Pending".to_string(),
        });
        ctx.conversations
            .push(format!("FT10_PaymentArrangement:{:?}", method));
        ctx.current_phase = TradePhase26::Ft11PaymentCollection;
        Ok(())
    }

    /// FT11: 收款确认
    pub fn confirm_payment(&mut self, order_id: &str, paid_amount: f64) -> Result<(), String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        if let Some(payment) = ctx.payment.as_mut() {
            payment.paid_amount = paid_amount;
            if paid_amount >= payment.total_amount {
                payment.status = "Paid".into();
                ctx.current_phase = TradePhase26::Ft12ProductionOrderMaterialPrep;
            }
        }
        ctx.conversations
            .push(format!("FT11_PaymentConfirmed:{}", paid_amount));
        Ok(())
    }

    // ════════════════════════════════════════════════════════════════
    // G4: 生产 (FT12-FT15)
    // ════════════════════════════════════════════════════════════════

    /// FT12: 生产下单备料
    pub fn create_production_order(
        &mut self,
        order_id: &str,
        materials: &[BomRequirement],
    ) -> Result<ProductionOrder, String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        // TODO: ProductionEngine::create_production_order is a static method
        // let order = ProductionEngine::create_production_order(order_id, materials);
        let order = ProductionOrder {
            production_order_id: format!("PO-{}", uuid::Uuid::new_v4().simple()),
            contract_id: order_id.to_string(),
            bom: materials.iter().map(|m| m.clone()).collect(),
            routing: Vec::new(),
            schedule: ProductionSchedule {
                start_date: String::new(),
                end_date: String::new(),
                critical_path: Vec::new(),
                buffer_days: 0,
            },
            supplier_orders: Vec::new(),
            milestones: Vec::new(),
        };
        ctx.production_status = Some(ProductionStatus {
            stage: "Preparation".into(),
            progress_pct: 0.0,
            eta: None,
            issues: Vec::new(),
        });
        ctx.conversations.push("FT12_ProductionOrderCreated".into());
        ctx.current_phase = TradePhase26::Ft13ProductionTrackingAlerting;
        Ok(order)
    }

    /// FT13: 生产跟踪预警
    pub fn track_production(
        &mut self,
        order_id: &str,
        progress: &DailyProgress,
    ) -> Result<(), String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        // TODO: ProductionEngine.track_progress() method not yet implemented
        // self.production_engine.track_progress(progress)?;
        ctx.production_status = Some(ProductionStatus {
            stage: progress.stage.clone(),
            progress_pct: progress.progress_pct,
            eta: progress.eta,
            issues: progress.issues.clone(),
        });
        ctx.conversations.push(format!(
            "FT13_Progress:{:.1}%",
            progress.progress_pct * 100.0
        ));
        // TODO: DailyProgress.is_complete field not yet available
        // if progress.is_complete {
        //     ctx.current_phase = TradePhase26::Ft14QualityInspectionRelease;
        // }
        Ok(())
    }

    /// FT14: 质量检验放行
    pub(crate) fn _quality_inspect(&mut self, order_id: &str) -> Result<(), String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        ctx.current_phase = TradePhase26::Ft15FinalQualityCheck;
        ctx.conversations.push("FT14_QualityInspection".into());
        Ok(())
    }

    /// FT15: 出货前终检
    pub(crate) fn _final_quality_check(&mut self, order_id: &str) -> Result<bool, String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        ctx.conversations.push("FT15_FinalQualityCheck".into());
        // TODO: ProductionEngine.final_check() method not yet implemented
        // let passed = self.production_engine.final_check();
        let passed = true;
        if passed {
            ctx.current_phase = TradePhase26::Ft16InspectionCertification;
        }
        Ok(passed)
    }

    // ════════════════════════════════════════════════════════════════
    // G5: 物流 (FT16-FT20)
    // ════════════════════════════════════════════════════════════════

    /// FT16: 检验检疫证书
    pub fn apply_inspection_cert(&mut self, order_id: &str) -> Result<CiqCertificate, String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        // TODO: LogisticsEngine.apply_inspection_cert() method not yet implemented
        // let cert = self.logistics_engine.apply_inspection_cert()?;
        let cert = CiqCertificate {
            ciq_id: format!("CIQ-{}", uuid::Uuid::new_v4().simple()),
            certificate_no: format!("CN{:08}", rand::random::<u32>() % 100000000),
            product: "General merchandise".into(),
            hs_code: "9999".into(),
            qty: 1,
            weight_kg: 1.0,
            status: CiqStatus::Issued,
            issue_date: chrono::Utc::now().date_naive().to_string(),
            expiry_date: Some(
                (chrono::Utc::now() + chrono::Duration::days(365))
                    .date_naive()
                    .to_string(),
            ),
        };
        ctx.conversations.push("FT16_InspectionCertApplied".into());
        ctx.logistics = Some(LogisticsInfo {
            vessel: None,
            bl_number: None,
            pol: "Port of Loading".into(),
            pod: "Port of Discharge".into(),
            etd: None,
            eta: None,
            status: "Certified".into(),
        });
        ctx.current_phase = TradePhase26::Ft17BookingPackingList;
        Ok(cert)
    }

    /// FT17: 订舱装箱
    pub fn book_and_pack(
        &mut self,
        order_id: &str,
        booking: &PLBookingConfirmation,
        packing_list: &PLPackingList,
    ) -> Result<(), String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        self.logistics_engine.book_and_pack(booking, packing_list)?;
        ctx.logistics = Some(LogisticsInfo {
            vessel: Some(booking.vessel.clone()),
            bl_number: None,
            pol: booking.port_of_loading.clone(),
            pod: booking.port_of_discharge.clone(),
            etd: None,
            eta: None,
            status: "Booked".into(),
        });
        ctx.conversations
            .push("FT17_BookingPackingListCompleted".into());
        ctx.current_phase = TradePhase26::Ft18CustomsClearance;
        Ok(())
    }

    /// FT18: 报关清关
    pub fn customs_clearance(
        &mut self,
        order_id: &str,
        declaration: &PLCustomsDeclaration,
    ) -> Result<(), String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        self.logistics_engine.customs_clearance(declaration)?;
        ctx.conversations.push("FT18_CustomsCleared".into());
        ctx.current_phase = TradePhase26::Ft19BillOfLadingManagement;
        Ok(())
    }

    /// FT19: 提单管理
    pub(crate) fn _manage_bill_of_lading(
        &mut self,
        order_id: &str,
        bl: &PLBillOfLading,
    ) -> Result<(), String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        self.logistics_engine.manage_bl(bl)?;
        ctx.logistics = Some(
            ctx.logistics
                .as_ref()
                .map(|l| LogisticsInfo {
                    vessel: l.vessel.clone(),
                    bl_number: Some(bl.bl_number.clone()),
                    pol: l.pol.clone(),
                    pod: l.pod.clone(),
                    etd: l.etd,
                    eta: l.eta,
                    status: "Bill of Lading issued".into(),
                })
                .unwrap_or_else(|| LogisticsInfo {
                    vessel: None,
                    bl_number: Some(bl.bl_number.clone()),
                    pol: "Port of Loading".into(),
                    pod: "Port of Discharge".into(),
                    etd: None,
                    eta: None,
                    status: "Bill of Lading issued".into(),
                }),
        );
        ctx.conversations.push("FT19_BillOfLadingManaged".into());
        ctx.current_phase = TradePhase26::Ft20ShipmentTracking;
        Ok(())
    }

    /// FT20: 运输跟踪
    pub fn track_shipment(&mut self, order_id: &str) -> Result<String, String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        let tracking = self.logistics_engine.track_shipment()?;
        ctx.conversations
            .push(format!("FT20_ShipmentTrack:{}", tracking));
        Ok(tracking)
    }

    // ════════════════════════════════════════════════════════════════
    // G6: 结算 (FT21-FT24)
    // ════════════════════════════════════════════════════════════════

    /// FT21: 尾款收取
    pub(crate) fn _collect_final_payment(&mut self, order_id: &str) -> Result<(), String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        if let Some(payment) = ctx.payment.as_mut() {
            payment.paid_amount = payment.total_amount;
            payment.status = "Paid".into();
        }
        ctx.current_phase = TradePhase26::Ft22SettlementVerification;
        ctx.conversations.push("FT21_FinalPaymentCollected".into());
        Ok(())
    }

    /// FT22: 结汇核销
    pub fn verify_settlement(&mut self, order_id: &str) -> Result<SettlementInfo, String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        let record = self.finance_engine.verify_settlement()?;
        let settlement = SettlementInfo {
            fx_rate: record.fx_rate,
            settled_amount: record.settlement_amount,
            tax_refund: None,
            completed: true,
        };
        ctx.settlement = Some(settlement.clone());
        ctx.conversations.push("FT22_SettlementVerified".into());
        Ok(settlement)
    }

    /// FT23: 退税申报
    pub fn declare_tax_refund(
        &mut self,
        order_id: &str,
        claim: &FCTaxRefundClaim,
    ) -> Result<RefundDocument, String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        let doc = self.finance_engine.declare_tax_refund(claim)?;
        ctx.conversations.push("FT23_TaxRefundDeclared".into());
        ctx.current_phase = TradePhase26::Ft24AccountReconciliation;
        Ok(doc)
    }

    /// FT24: 账务核对
    pub fn reconcile_accounts(&mut self, order_id: &str) -> Result<(), String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        self.finance_engine.reconcile_accounts()?;
        ctx.conversations.push("FT24_AccountsReconciled".into());
        ctx.current_phase = TradePhase26::Ft25OrderReview;
        Ok(())
    }

    // ════════════════════════════════════════════════════════════════
    // G7: 复盘 (FT25-FT26)
    // ════════════════════════════════════════════════════════════════

    /// FT25: 订单复盘
    pub fn review_order(&mut self, order_id: &str) -> Result<OrderReview, String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        let review = OrderReview {
            order_id: ctx.order_id.clone(),
            phases_completed: ctx.events.len() as u32,
            total_phases: TradePhase26::all().len() as u32,
            success: ctx.current_phase == TradePhase26::Ft26ExperienceAbsorption,
            lessons_learned: Vec::new(),
            recommendations: Vec::new(),
        };
        ctx.conversations.push("FT25_OrderReviewed".into());
        ctx.current_phase = TradePhase26::Ft26ExperienceAbsorption;
        Ok(review)
    }

    /// FT26: 经验吸收入库
    pub fn absorb_experience(&mut self, order_id: &str) -> Result<(), String> {
        let ctx = self
            .active_trades
            .get_mut(order_id)
            .ok_or_else(|| format!("Trade {} not found", order_id))?;
        // 将交易经验转换为知识库条目
        let _experience_text = format!(
            "Trade {} completed: {} phases, final phase: {:?}",
            ctx.order_id,
            ctx.events.len(),
            ctx.current_phase
        );
        // 写入 KB (实际实现会使用 KB 写入函数)
        ctx.conversations.push("FT26_ExperienceAbsorbed".into());
        ctx.current_phase = TradePhase26::Ft25OrderReview; // 重置或标记完成
        Ok(())
    }
}

// ════════════════════════════════════════════════════════════════
// 测试
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l1_action::nt_act::nt_act_trade::production_logistics::{BookingStatus, MaterialStatus, PackingItem};

    #[test]
    fn test_trade_phases_count() {
        assert_eq!(TradePhase26::all().len(), 26);
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
        assert_eq!(ctx.current_phase, TradePhase26::Ft07DetailedQuotation);
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
        assert_eq!(
            TradePhase26::Ft01SocialMediaContent.group(),
            TradeGroup::Acquisition
        );
        assert_eq!(
            TradePhase26::Ft07DetailedQuotation.group(),
            TradeGroup::Negotiation
        );
        assert_eq!(
            TradePhase26::Ft12ProductionOrderMaterialPrep.group(),
            TradeGroup::Production
        );
        assert_eq!(
            TradePhase26::Ft17BookingPackingList.group(),
            TradeGroup::Logistics
        );
        assert_eq!(
            TradePhase26::Ft21FinalPaymentCollection.group(),
            TradeGroup::Settlement
        );
        assert_eq!(TradePhase26::Ft25OrderReview.group(), TradeGroup::Review);
    }

    #[test]
    fn test_g2_requirement_confirmation() {
        let mut orch = TradeOrchestrator::new();
        let lead_id = orch.capture_inquiry(
            LeadSource::Alibaba,
            "Test Corp",
            "Need 500 units of widget A",
            vec!["widget".into()],
        );
        let order_id = orch.start_trade(&lead_id).unwrap();

        let result = orch.confirm_requirements(&order_id, &["spec A".into(), "spec B".into()]);
        assert!(result.is_ok());

        let ctx = orch.active_trades.get(&order_id).unwrap();
        assert_eq!(ctx.current_phase, TradePhase26::Ft07DetailedQuotation);
        assert!(ctx.conversations.iter().any(|c| c.contains("FT06")));
    }

    #[test]
    fn test_g2_quotation_generation() {
        let mut orch = TradeOrchestrator::new();
        let lead_id = orch.capture_inquiry(
            LeadSource::LinkedIn,
            "Buyer Inc",
            "Looking for machinery",
            vec!["machinery".into()],
        );
        let order_id = orch.start_trade(&lead_id).unwrap();
        orch.confirm_requirements(&order_id, &["spec A".into()])
            .unwrap();

        let items = vec![
            QuotationItem {
                product: "Widget A".into(),
                quantity: 100,
                unit_price: 50.0,
                amount: 5000.0,
            },
            QuotationItem {
                product: "Widget B".into(),
                quantity: 200,
                unit_price: 25.0,
                amount: 5000.0,
            },
        ];
        let quotation = orch.generate_quotation(&order_id, &items).unwrap();

        assert_eq!(quotation.total_amount, 10000.0);
        assert_eq!(quotation.currency, "USD");
        assert_eq!(quotation.incoterm, "FOB");
    }

    #[test]
    fn test_g3_payment_arrangement() {
        let mut orch = TradeOrchestrator::new();
        let lead_id = orch.capture_inquiry(
            LeadSource::Alibaba,
            "Buyer",
            "Inquiry",
            vec!["product".into()],
        );
        let order_id = orch.start_trade(&lead_id).unwrap();
        orch.confirm_requirements(&order_id, &[]).unwrap();
        orch.generate_quotation(
            &order_id,
            &[QuotationItem {
                product: "A".into(),
                quantity: 10,
                unit_price: 100.0,
                amount: 1000.0,
            }],
        )
        .unwrap();

        let result = orch.arrange_payment(
            &order_id,
            PaymentType::LetterOfCredit,
            Some("LC12345".into()),
        );
        assert!(result.is_ok());

        let ctx = orch.active_trades.get(&order_id).unwrap();
        assert!(ctx.payment.is_some());
        let payment = ctx.payment.as_ref().unwrap();
        assert_eq!(payment.total_amount, 1000.0);
        assert_eq!(payment.lc_number, Some("LC12345".into()));
    }

    #[test]
    fn test_g3_payment_confirmation() {
        let mut orch = TradeOrchestrator::new();
        let lead_id = orch.capture_inquiry(
            LeadSource::Alibaba,
            "Buyer",
            "Inquiry",
            vec!["product".into()],
        );
        let order_id = orch.start_trade(&lead_id).unwrap();
        orch.confirm_requirements(&order_id, &[]).unwrap();
        orch.generate_quotation(
            &order_id,
            &[QuotationItem {
                product: "A".into(),
                quantity: 10,
                unit_price: 100.0,
                amount: 1000.0,
            }],
        )
        .unwrap();
        orch.arrange_payment(&order_id, PaymentType::LetterOfCredit, None)
            .unwrap();

        // Partial payment
        orch.confirm_payment(&order_id, 500.0).unwrap();
        let ctx = orch.active_trades.get(&order_id).unwrap();
        assert_eq!(ctx.payment.as_ref().unwrap().paid_amount, 500.0);

        // Full payment
        orch.confirm_payment(&order_id, 1000.0).unwrap();
        let ctx = orch.active_trades.get(&order_id).unwrap();
        assert_eq!(ctx.payment.as_ref().unwrap().paid_amount, 1000.0);
        assert_eq!(
            ctx.current_phase,
            TradePhase26::Ft12ProductionOrderMaterialPrep
        );
    }

    #[test]
    fn test_g4_production_flow() {
        let mut orch = TradeOrchestrator::new();
        let lead_id = orch.capture_inquiry(
            LeadSource::Alibaba,
            "Buyer",
            "Inquiry",
            vec!["product".into()],
        );
        let order_id = orch.start_trade(&lead_id).unwrap();
        orch.confirm_requirements(&order_id, &[]).unwrap();
        orch.generate_quotation(
            &order_id,
            &[QuotationItem {
                product: "A".into(),
                quantity: 10,
                unit_price: 100.0,
                amount: 1000.0,
            }],
        )
        .unwrap();
        orch.arrange_payment(&order_id, PaymentType::Deposit, None)
            .unwrap();
        orch.confirm_payment(&order_id, 1000.0).unwrap();

        // Create production order
        let materials = vec![BomRequirement {
            item_id: "M1".into(),
            name: "Steel".into(),
            required_qty: 50.0,
            allocated_qty: 0.0,
            supplier: None,
            expected_arrival: None,
            status: MaterialStatus::Pending,
        }];
        let order = orch.create_production_order(&order_id, &materials).unwrap();
        assert!(!order.production_order_id.is_empty());

        let ctx = orch.active_trades.get(&order_id).unwrap();
        assert_eq!(
            ctx.current_phase,
            TradePhase26::Ft13ProductionTrackingAlerting
        );

        // Track production
        let progress = DailyProgress {
            date: "2024-01-15".into(),
            work_center: "CNC-01".into(),
            planned_hours: 8.0,
            actual_hours: 6.0,
            output_qty: 100,
            efficiency: 0.85,
            stage: "Machining".into(),
            progress_pct: 0.5,
            eta: Some(1705296000),
            issues: vec!["Delay in raw material".into()],
        };
        orch.track_production(&order_id, &progress).unwrap();

        let ctx = orch.active_trades.get(&order_id).unwrap();
        assert_eq!(ctx.production_status.as_ref().unwrap().progress_pct, 0.5);
    }

    #[test]
    fn test_g5_logistics_flow() {
        let mut orch = TradeOrchestrator::new();
        let lead_id = orch.capture_inquiry(
            LeadSource::Alibaba,
            "Buyer",
            "Inquiry",
            vec!["product".into()],
        );
        let order_id = orch.start_trade(&lead_id).unwrap();
        orch.confirm_requirements(&order_id, &[]).unwrap();
        orch.generate_quotation(
            &order_id,
            &[QuotationItem {
                product: "A".into(),
                quantity: 10,
                unit_price: 100.0,
                amount: 1000.0,
            }],
        )
        .unwrap();
        orch.arrange_payment(&order_id, PaymentType::Deposit, None)
            .unwrap();
        orch.confirm_payment(&order_id, 1000.0).unwrap();
        orch.create_production_order(
            &order_id,
            &[BomRequirement {
                item_id: "M1".into(),
                name: "Steel".into(),
                required_qty: 50.0,
                allocated_qty: 0.0,
                supplier: None,
                expected_arrival: None,
                status: MaterialStatus::Pending,
            }],
        )
        .unwrap();

        // Apply inspection cert
        let cert = orch.apply_inspection_cert(&order_id).unwrap();
        assert!(!cert.certificate_no.is_empty());

        // Book and pack
        let booking = PLBookingConfirmation {
            booking_id: "BK-001".into(),
            booking_ref: "BK001".into(),
            vessel: "MSC SHANGHAI".into(),
            voyage: "2412E".into(),
            port_of_loading: "Shanghai".into(),
            port_of_discharge: "Los Angeles".into(),
            eto: "2024-01-15".into(),
            eta: "2024-02-15".into(),
            container_no: Some("B/L123456".into()),
            container_type: "40HQ".into(),
            packing_list: PLPackingList {
                items: vec![PackingItem {
                    product: "Widget A".into(),
                    description: "Widget A x 10".into(),
                    qty: 10,
                    ctns: 1,
                    cbm_per_ctn: 0.25,
                    gross_kg_per_ctn: 50.0,
                    net_kg_per_ctn: 45.0,
                    marks: vec![],
                }],
                total_ctns: 1,
                total_cbm: 0.25,
                total_gross_kg: 50.0,
                total_net_kg: 45.0,
            },
            status: BookingStatus::Confirmed,
        };
        orch.book_and_pack(&order_id, &booking, &booking.packing_list).unwrap();

        let ctx = orch.active_trades.get(&order_id).unwrap();
        assert_eq!(
            ctx.logistics.as_ref().unwrap().vessel,
            Some("MSC SHANGHAI".into())
        );
        assert_eq!(ctx.current_phase, TradePhase26::Ft18CustomsClearance);
    }

    #[test]
    fn test_g7_review_and_absorb() {
        let mut orch = TradeOrchestrator::new();
        let lead_id = orch.capture_inquiry(
            LeadSource::Alibaba,
            "Buyer",
            "Inquiry",
            vec!["product".into()],
        );
        let order_id = orch.start_trade(&lead_id).unwrap();
        orch.confirm_requirements(&order_id, &[]).unwrap();

        let review = orch.review_order(&order_id).unwrap();
        assert_eq!(review.order_id, order_id);
        assert_eq!(review.total_phases, 26);

        let result = orch.absorb_experience(&order_id);
        assert!(result.is_ok());
    }
}
