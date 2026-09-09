//! Trade domain shared data types — used by both L1 (orchestrator) and L5 (engines).
//!
//! These are pure data types (structs/enums) with no domain-specific method implementations.
//! Engine types with methods stay in L5; L1 defines traits for the interfaces it needs.

use serde::{Deserialize, Serialize};

// ════════════════════════════════════════════════════════════════
// Quote & Negotiation types (from nt_trade_quote_negotiation)
// ════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NegotiationStrategy {
    Collaborative,
    Competitive,
    Compromise,
    Accommodating,
    Avoiding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectionCategory {
    Price,
    Quality,
    DeliveryTime,
    PaymentTerms,
    Specification,
    Trust,
    Competitor,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementConfirmation {
    pub confirmed: bool,
    pub confirmed_items: Vec<ConfirmedItem>,
    pub pending_clarifications: Vec<String>,
    pub intent_level: IntentLevel,
    pub risk_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmedItem {
    pub item: String,
    pub spec: String,
    pub qty: u64,
    pub unit: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentLevel {
    Low,
    Medium,
    High,
    Confirmed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteGenerator {
    pub base_cost: f64,
    pub margin_target: f64,
    pub incoterms: String,
    pub currency: String,
    pub validity_days: u32,
    pub cost_breakdown: CostBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub material: f64,
    pub labor: f64,
    pub overhead: f64,
    pub packaging: f64,
    pub logistics: f64,
    pub certification: f64,
    pub contingency: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteSheet {
    pub quote_id: String,
    pub version: u32,
    pub incoterms: String,
    pub unit_price: f64,
    pub total: f64,
    pub currency: String,
    pub validity_days: u32,
    pub risk_flag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationEngine {
    pub strategy: NegotiationStrategy,
    pub bottom_line: f64,
    pub current_quote: f64,
    pub round: u32,
    pub concessions_made: Vec<Concession>,
    pub competitor_data: Option<CompetitorData>,
}

impl Default for NegotiationEngine {
    fn default() -> Self {
        Self {
            strategy: NegotiationStrategy::Collaborative,
            bottom_line: 0.0,
            current_quote: 0.0,
            round: 0,
            concessions_made: Vec::new(),
            competitor_data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concession {
    pub round: u32,
    pub item: String,
    pub original: f64,
    pub conceded: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorData {
    pub competitor_name: String,
    pub quoted_price: f64,
    pub quoted_terms: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationRecord {
    pub round: u32,
    pub objection: Objection,
    pub response: String,
    pub concession: Option<Concession>,
    pub bottom_line_held: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objection {
    pub category: ObjectionCategory,
    pub description: String,
    pub customer_argument: String,
    pub severity: ObjectionSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectionSeverity {
    Low,
    Medium,
    High,
    Blocker,
}

// ════════════════════════════════════════════════════════════════
// Production & Logistics types (from nt_trade_production_logistics)
// ════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionOrder {
    pub production_order_id: String,
    pub contract_id: String,
    pub bom: Vec<BomRequirement>,
    pub routing: Vec<RoutingRequirement>,
    pub schedule: ProductionSchedule,
    pub supplier_orders: Vec<SupplierOrder>,
    pub milestones: Vec<ProductionMilestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomRequirement {
    pub item_id: String,
    pub name: String,
    pub required_qty: f64,
    pub allocated_qty: f64,
    pub supplier: Option<String>,
    pub expected_arrival: Option<String>,
    pub status: MaterialStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialStatus {
    Pending,
    Ordered,
    InTransit,
    Received,
    Shortage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRequirement {
    pub step_id: String,
    pub name: String,
    pub work_center: String,
    pub planned_hours: f64,
    pub actual_hours: Option<f64>,
    pub assigned_operator: Option<String>,
    pub status: RoutingStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingStatus {
    NotStarted,
    InProgress,
    Completed,
    OnHold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionSchedule {
    pub start_date: String,
    pub end_date: String,
    pub critical_path: Vec<String>,
    pub buffer_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierOrder {
    pub order_id: String,
    pub supplier: String,
    pub items: Vec<SupplierOrderItem>,
    pub order_date: String,
    pub promised_date: String,
    pub status: SupplierOrderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierOrderItem {
    pub item_id: String,
    pub qty: f64,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupplierOrderStatus {
    Draft,
    Sent,
    Confirmed,
    PartialShipped,
    Completed,
    Delayed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionMilestone {
    pub name: String,
    pub planned_date: String,
    pub actual_date: Option<String>,
    pub status: MilestoneStatus,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MilestoneStatus {
    Pending,
    InProgress,
    Completed,
    Delayed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressReport {
    pub report_date: String,
    pub overall_progress: f64,
    pub daily_progress: Vec<DailyProgress>,
    pub deviation: ScheduleDeviation,
    pub alerts: Vec<ProductionAlert>,
    pub escalated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyProgress {
    pub date: String,
    pub work_center: String,
    pub planned_hours: f64,
    pub actual_hours: f64,
    pub output_qty: u64,
    pub efficiency: f64,
    pub stage: String,
    pub progress_pct: f64,
    pub eta: Option<u64>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleDeviation {
    pub critical_path_delay_days: i32,
    pub milestone_delays: Vec<MilestoneDelay>,
    pub recovery_plan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneDelay {
    pub milestone: String,
    pub delay_days: i32,
    pub cause: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionAlert {
    pub alert_id: String,
    pub level: AlertLevel,
    pub message: String,
    pub affected_milestone: Option<String>,
    pub suggested_action: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionReport {
    pub inspection_id: String,
    pub production_order_id: String,
    pub aql_level: String,
    pub sample_size: u32,
    pub result: InspectionResult,
    pub defects: Vec<Defect>,
    pub measurements: Vec<Measurement>,
    pub photos: Vec<String>,
    pub inspector: String,
    pub inspection_date: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InspectionResult {
    Pass,
    ConditionalPass,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defect {
    pub code: String,
    pub description: String,
    pub severity: DefectSeverity,
    pub location: String,
    pub qty: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefectSeverity {
    Critical,
    Major,
    Minor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    pub parameter: String,
    pub spec_min: f64,
    pub spec_max: f64,
    pub actual: f64,
    pub unit: String,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiqCertificate {
    pub ciq_id: String,
    pub certificate_no: String,
    pub product: String,
    pub hs_code: String,
    pub qty: u64,
    pub weight_kg: f64,
    pub status: CiqStatus,
    pub issue_date: String,
    pub expiry_date: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CiqStatus {
    Applied,
    InProgress,
    Issued,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingConfirmation {
    pub booking_id: String,
    pub booking_ref: String,
    pub vessel: String,
    pub voyage: String,
    pub port_of_loading: String,
    pub port_of_discharge: String,
    pub eto: String,
    pub eta: String,
    pub container_no: Option<String>,
    pub container_type: String,
    pub packing_list: PackingList,
    pub status: BookingStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookingStatus {
    Requested,
    Confirmed,
    Allocated,
    Loaded,
    Sailed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackingList {
    pub items: Vec<PackingItem>,
    pub total_ctns: u32,
    pub total_cbm: f64,
    pub total_gross_kg: f64,
    pub total_net_kg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackingItem {
    pub product: String,
    pub description: String,
    pub qty: u32,
    pub ctns: u32,
    pub cbm_per_ctn: f64,
    pub gross_kg_per_ctn: f64,
    pub net_kg_per_ctn: f64,
    pub marks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomsDeclaration {
    pub declaration_id: String,
    pub customs_code: String,
    pub declaration_date: String,
    pub hs_code: String,
    pub qty: u64,
    pub unit: String,
    pub unit_price: f64,
    pub total_value: f64,
    pub currency: String,
    pub trade_terms: String,
    pub origin_country: String,
    pub destination_country: String,
    pub status: CustomsStatus,
    pub clearance_time: Option<String>,
    pub risk_level: TradeRiskLevel,
    pub tax_amount: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomsStatus {
    Draft,
    Submitted,
    UnderReview,
    Inspected,
    Released,
    Held,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillOfLading {
    pub bl_number: String,
    pub bl_type: BlType,
    pub shipper: String,
    pub consignee: String,
    pub notify_party: Option<String>,
    pub vessel: String,
    pub voyage: String,
    pub port_of_loading: String,
    pub port_of_discharge: String,
    pub date_of_issue: String,
    pub place_of_issue: String,
    pub goods_description: String,
    pub marks_numbers: String,
    pub packages: String,
    pub gross_weight_kg: f64,
    pub measurement_cbm: f64,
    pub status: BlStatus,
    pub original_count: u32,
    pub copies_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlType {
    Original,
    Surrendered,
    SeaWaybill,
    HouseBl,
    MasterBl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlStatus {
    Draft,
    Issued,
    Surrendered,
    Lost,
    Amended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoInfo {
    pub product: String,
    pub description: String,
    pub qty: u32,
    pub ctns: u32,
    pub cbm_per_ctn: f64,
    pub gross_kg_per_ctn: f64,
    pub net_kg_per_ctn: f64,
    pub marks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingRequirements {
    pub port_of_loading: String,
    pub port_of_discharge: String,
    pub container_type: String,
    pub earliest_departure: String,
    pub latest_arrival: String,
}

// ════════════════════════════════════════════════════════════════
// Finance & Compliance types (from nt_trade_finance_compliance)
// ════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractReview {
    pub contract_id: String,
    pub reviewed: bool,
    pub review_date: String,
    pub reviewer: String,
    pub findings: Vec<ContractFinding>,
    pub risk_score: f64,
    pub approved: bool,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractFinding {
    pub clause: String,
    pub issue: String,
    pub severity: FindingSeverity,
    pub recommendation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    Info,
    Minor,
    Major,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProof {
    pub payment_id: String,
    pub contract_id: String,
    pub payment_type: PaymentType,
    pub amount: f64,
    pub currency: String,
    pub status: PaymentStatus,
    pub received_date: Option<String>,
    pub bank_slip: Option<String>,
    pub bank_reference: Option<String>,
    pub lc_number: Option<String>,
    pub risk_flags: Vec<RiskFlag>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentType {
    Deposit,
    Balance,
    LetterOfCredit,
    DocumentaryCollection,
    AdvancePayment,
    Other,
}

impl std::fmt::Display for PaymentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentType::Deposit => write!(f, "Deposit"),
            PaymentType::Balance => write!(f, "Balance"),
            PaymentType::LetterOfCredit => write!(f, "LetterOfCredit"),
            PaymentType::DocumentaryCollection => write!(f, "DocumentaryCollection"),
            PaymentType::AdvancePayment => write!(f, "AdvancePayment"),
            PaymentType::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Received,
    Verified,
    Paid,
    Failed,
    Disputed,
    Refunded,
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentStatus::Pending => write!(f, "Pending"),
            PaymentStatus::Received => write!(f, "Received"),
            PaymentStatus::Verified => write!(f, "Verified"),
            PaymentStatus::Paid => write!(f, "Paid"),
            PaymentStatus::Failed => write!(f, "Failed"),
            PaymentStatus::Disputed => write!(f, "Disputed"),
            PaymentStatus::Refunded => write!(f, "Refunded"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFlag {
    pub code: String,
    pub description: String,
    pub level: FinanceRiskLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinanceRiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LcReview {
    pub lc_number: String,
    pub contract_id: String,
    pub issuing_bank: String,
    pub advising_bank: Option<String>,
    pub amount: f64,
    pub currency: String,
    pub expiry_date: String,
    pub expiry_place: String,
    pub soft_clauses: Vec<SoftClause>,
    pub discrepancies: Vec<Discrepancy>,
    pub risk_score: f64,
    pub recommendation: LcRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftClause {
    pub clause_text: String,
    pub risk: String,
    pub mitigation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discrepancy {
    pub field: String,
    pub required: String,
    pub presented: String,
    pub is_waivable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LcRecommendation {
    Accept,
    AcceptWithAmendment,
    Reject,
    RequestClarification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionRecord {
    pub collection_id: String,
    pub contract_id: String,
    pub bl_number: Option<String>,
    pub bl_sent: bool,
    pub bl_sent_date: Option<String>,
    pub documents: Vec<CollectionDocument>,
    pub payment_received: bool,
    pub payment_date: Option<String>,
    pub amount_received: f64,
    pub documents_released: bool,
    pub release_date: Option<String>,
    pub status: CollectionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionDocument {
    pub doc_type: String,
    pub originals: u32,
    pub copies: u32,
    pub status: DocumentStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentStatus {
    Prepared,
    Submitted,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollectionStatus {
    InProgress,
    AwaitingPayment,
    Completed,
    Disputed,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementRecord {
    pub settlement_id: String,
    pub contract_id: String,
    pub collection_id: String,
    pub bank_receipt: String,
    pub received_amount: f64,
    pub received_currency: String,
    pub settlement_amount: f64,
    pub settlement_currency: String,
    pub fx_rate: f64,
    pub bank_fees: f64,
    pub net_amount: f64,
    pub verification_status: VerificationStatus,
    pub verification_date: Option<String>,
    pub verification_officer: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Pending,
    Verified,
    Discrepancy,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRefundClaim {
    pub refund_id: String,
    pub contract_id: String,
    pub declaration_id: String,
    pub product_hs_code: String,
    pub export_value: f64,
    pub refund_rate: f64,
    pub claim_amount: f64,
    pub status: RefundStatus,
    pub application_date: String,
    pub approval_date: Option<String>,
    pub refund_received_date: Option<String>,
    pub actual_refund_amount: Option<f64>,
    pub documents: Vec<RefundDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundDocument {
    pub doc_type: String,
    pub doc_number: String,
    pub issue_date: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefundStatus {
    Draft,
    Submitted,
    UnderReview,
    Approved,
    Paid,
    Rejected,
    RequiresSupplement,
}
