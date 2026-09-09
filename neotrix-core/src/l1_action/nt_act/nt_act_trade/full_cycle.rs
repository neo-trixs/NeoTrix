//! Foreign Trade Full-Cycle Keystone Skill Orchestrator
//!
//! 17-step standard process covering the complete export trade lifecycle:
//! Customer Acquisition → Quotation → Negotiation → Contract → Payment →
//! Production → Inspection → Logistics → Customs → Settlement → Tax Refund → Review
//!
//! This is a KEYSTONE skill (跨域变革级) that orchestrates 3 Notable sub-skills:
//! 1. trade_quote_negotiation — 报价谈判闭环 (FT01-FT04)
//! 2. trade_production_logistics — 生产物流闭环 (FT07-FT13)
//! 3. trade_finance_compliance — 财务合规闭环 (FT05-FT06, FT14-FT16)

use nt_core_capability_tree::{
    CapabilityNode, CapabilityRegistry, Domain, NodeLayer,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Foreign Trade Phase (FT01-FT17)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradePhase {
    // Phase 1: Customer Acquisition
    Ft01CustomerDevelopment,
    Ft02RequirementConfirmation,
    // Phase 2: Quotation & Negotiation
    Ft03DetailedQuotation,
    Ft04NegotiationObjectionHandling,
    // Phase 3: Contract & Payment
    Ft05ContractReviewSigning,
    Ft06PaymentCollection,
    // Phase 4: Production
    Ft07ProductionOrderMaterialPrep,
    Ft08ProductionTrackingAlerting,
    Ft09QualityInspectionRelease,
    // Phase 5: Logistics
    Ft10InspectionCertification,
    Ft11BookingPackingList,
    Ft12CustomsClearance,
    Ft13BillOfLadingManagement,
    // Phase 6: Settlement
    Ft14FinalPaymentCollection,
    Ft15SettlementVerification,
    Ft16TaxRefundDeclaration,
    // Phase 7: Review
    Ft17OrderReviewExperienceAbsorption,
}

impl TradePhase {
    pub fn all() -> Vec<TradePhase> {
        vec![
            TradePhase::Ft01CustomerDevelopment,
            TradePhase::Ft02RequirementConfirmation,
            TradePhase::Ft03DetailedQuotation,
            TradePhase::Ft04NegotiationObjectionHandling,
            TradePhase::Ft05ContractReviewSigning,
            TradePhase::Ft06PaymentCollection,
            TradePhase::Ft07ProductionOrderMaterialPrep,
            TradePhase::Ft08ProductionTrackingAlerting,
            TradePhase::Ft09QualityInspectionRelease,
            TradePhase::Ft10InspectionCertification,
            TradePhase::Ft11BookingPackingList,
            TradePhase::Ft12CustomsClearance,
            TradePhase::Ft13BillOfLadingManagement,
            TradePhase::Ft14FinalPaymentCollection,
            TradePhase::Ft15SettlementVerification,
            TradePhase::Ft16TaxRefundDeclaration,
            TradePhase::Ft17OrderReviewExperienceAbsorption,
        ]
    }

    pub fn phase_group(&self) -> TradePhaseGroup {
        match self {
            TradePhase::Ft01CustomerDevelopment | TradePhase::Ft02RequirementConfirmation => {
                TradePhaseGroup::CustomerAcquisition
            }
            TradePhase::Ft03DetailedQuotation | TradePhase::Ft04NegotiationObjectionHandling => {
                TradePhaseGroup::QuotationNegotiation
            }
            TradePhase::Ft05ContractReviewSigning | TradePhase::Ft06PaymentCollection => {
                TradePhaseGroup::ContractPayment
            }
            TradePhase::Ft07ProductionOrderMaterialPrep
            | TradePhase::Ft08ProductionTrackingAlerting
            | TradePhase::Ft09QualityInspectionRelease => TradePhaseGroup::Production,
            TradePhase::Ft10InspectionCertification
            | TradePhase::Ft11BookingPackingList
            | TradePhase::Ft12CustomsClearance
            | TradePhase::Ft13BillOfLadingManagement => TradePhaseGroup::Logistics,
            TradePhase::Ft14FinalPaymentCollection
            | TradePhase::Ft15SettlementVerification
            | TradePhase::Ft16TaxRefundDeclaration => TradePhaseGroup::Settlement,
            TradePhase::Ft17OrderReviewExperienceAbsorption => TradePhaseGroup::Review,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            TradePhase::Ft01CustomerDevelopment => "FT01",
            TradePhase::Ft02RequirementConfirmation => "FT02",
            TradePhase::Ft03DetailedQuotation => "FT03",
            TradePhase::Ft04NegotiationObjectionHandling => "FT04",
            TradePhase::Ft05ContractReviewSigning => "FT05",
            TradePhase::Ft06PaymentCollection => "FT06",
            TradePhase::Ft07ProductionOrderMaterialPrep => "FT07",
            TradePhase::Ft08ProductionTrackingAlerting => "FT08",
            TradePhase::Ft09QualityInspectionRelease => "FT09",
            TradePhase::Ft10InspectionCertification => "FT10",
            TradePhase::Ft11BookingPackingList => "FT11",
            TradePhase::Ft12CustomsClearance => "FT12",
            TradePhase::Ft13BillOfLadingManagement => "FT13",
            TradePhase::Ft14FinalPaymentCollection => "FT14",
            TradePhase::Ft15SettlementVerification => "FT15",
            TradePhase::Ft16TaxRefundDeclaration => "FT16",
            TradePhase::Ft17OrderReviewExperienceAbsorption => "FT17",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradePhaseGroup {
    CustomerAcquisition,
    QuotationNegotiation,
    ContractPayment,
    Production,
    Logistics,
    Settlement,
    Review,
}

/// Trade Context - Input to the capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeContext {
    pub buyer_profile: BuyerProfile,
    pub inquiry: InquiryDetail,
    pub product_spec: ProductSpec,
    pub company_policy: CompanyPolicy,
    pub market_env: MarketEnvironment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyerProfile {
    pub buyer_id: String,
    pub company: String,
    pub country: String,
    pub industry: String,
    pub scale: String,
    pub decision_maker: Option<String>,
    pub contact: Option<String>,
    pub credit_score: Option<f64>,
    pub history_orders: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InquiryDetail {
    pub inquiry_id: String,
    pub product: String,
    pub specs: HashMap<String, String>,
    pub qty: u64,
    pub deadline: String,
    pub destination: String,
    pub intent_level: IntentLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentLevel {
    Low,
    Medium,
    High,
    Confirmed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSpec {
    pub spec_id: String,
    pub product_type: ProductType,
    pub bom: Vec<BomItem>,
    pub routing: Vec<RoutingStep>,
    pub packaging: PackagingSpec,
    pub certifications: Vec<String>,
    pub hs_code: String,
    pub tax_refund_rate: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductType {
    Machinery,
    Textile,
    Food,
    Chemical,
    Electronics,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomItem {
    pub item_id: String,
    pub name: String,
    pub qty: f64,
    pub unit: String,
    pub supplier: Option<String>,
    pub lead_time_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingStep {
    pub step_id: String,
    pub name: String,
    pub work_center: String,
    pub duration_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagingSpec {
    pub package_type: String,
    pub dimensions_cm: (f64, f64, f64),
    pub gross_weight_kg: f64,
    pub net_weight_kg: f64,
    pub marks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyPolicy {
    pub payment_terms: Vec<String>,
    pub risk_control: RiskControl,
    pub authorization_matrix: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskControl {
    pub max_credit_days: u32,
    pub min_deposit_ratio: f64,
    pub forbidden_countries: Vec<String>,
    pub forbidden_products: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEnvironment {
    pub fx_rates: HashMap<String, f64>,
    pub freight_rates: HashMap<String, f64>,
    pub regulations: Vec<String>,
    pub shipping_lines: Vec<String>,
}

/// Trade Result - Output from the capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub quotes: Vec<QuoteSheet>,
    pub contracts: Vec<Contract>,
    pub production_schedule: Schedule,
    pub inspection_reports: Vec<InspectionReport>,
    pub logistics_docs: LogisticsDocSet,
    pub finance_docs: FinanceDocSet,
    pub risk_alerts: Vec<RiskAlert>,
    pub recommendations: Vec<ActionRecommendation>,
    pub lessons_learned: Vec<Lesson>,
    pub knowledge_updates: Vec<KnowledgeDelta>,
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
pub struct Contract {
    pub contract_id: String,
    pub pi_number: String,
    pub parties: (String, String),
    pub items: Vec<ContractItem>,
    pub price: f64,
    pub incoterms: String,
    pub payment_terms: String,
    pub delivery_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractItem {
    pub product: String,
    pub qty: u64,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub milestones: Vec<Milestone>,
    pub critical_path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub planned_date: String,
    pub actual_date: Option<String>,
    pub status: MilestoneStatus,
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
pub struct InspectionReport {
    pub inspection_id: String,
    pub result: InspectionResult,
    pub defects: Vec<Defect>,
    pub aql_level: String,
    pub pass: bool,
    pub photos: Vec<String>,
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
    pub qty: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefectSeverity {
    Critical,
    Major,
    Minor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogisticsDocSet {
    pub ciq_certificates: Vec<CiqCertificate>,
    pub booking_confirmation: Option<BookingConfirmation>,
    pub customs_declaration: Option<CustomsDeclaration>,
    pub bill_of_lading: Option<BillOfLading>,
    pub packing_list: Option<PackingList>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiqCertificate {
    pub ciq_id: String,
    pub certificate_no: String,
    pub product: String,
    pub qty: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingConfirmation {
    pub booking_id: String,
    pub vessel: String,
    pub voyage: String,
    pub container_no: String,
    pub packing_list: PackingList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomsDeclaration {
    pub declaration_id: String,
    pub customs_status: String,
    pub clearance_time: Option<String>,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillOfLading {
    pub bl_number: String,
    pub bl_draft: String,
    pub bl_original: String,
    pub status: String,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackingList {
    pub items: Vec<PackingItem>,
    pub total_ctns: u32,
    pub total_cbm: f64,
    pub total_gross_kg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackingItem {
    pub product: String,
    pub qty: u32,
    pub ctns: u32,
    pub cbm: f64,
    pub gross_kg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceDocSet {
    pub payment_proofs: Vec<PaymentProof>,
    pub lc_review: Option<LcReview>,
    pub collection_records: Vec<CollectionRecord>,
    pub settlement_records: Vec<SettlementRecord>,
    pub tax_refund_claims: Vec<TaxRefundClaim>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProof {
    pub payment_id: String,
    pub method: String,
    pub amount: f64,
    pub status: String,
    pub bank_slip: Option<String>,
    pub lc_text: Option<String>,
    pub risk_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LcReview {
    pub lc_number: String,
    pub soft_clauses: Vec<String>,
    pub discrepancies: Vec<String>,
    pub risk_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionRecord {
    pub collection_id: String,
    pub bl_sent: bool,
    pub payment_received: bool,
    pub documents_released: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementRecord {
    pub settlement_id: String,
    pub bank_receipt: String,
    pub verification_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRefundClaim {
    pub refund_id: String,
    pub amount: f64,
    pub status: String,
    pub timeline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAlert {
    pub alert_id: String,
    pub phase: TradePhase,
    pub level: RiskLevel,
    pub message: String,
    pub suggested_action: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Info,
    Warning,
    Critical,
    Blocker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecommendation {
    pub action_id: String,
    pub phase: TradePhase,
    pub description: String,
    pub priority: u8,
    pub automation_possible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub lesson_id: String,
    pub phase: TradePhase,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDelta {
    pub entity_type: String,
    pub entity_id: String,
    pub operation: KnowledgeOperation,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeOperation {
    Create,
    Update,
    Delete,
}

/// Capability Spec - the interface contract for this Keystone skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeCapabilitySpec {
    pub context: TradeContext,
    pub result: TradeResult,
}

/// State Machine for the 17-phase process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeStateMachine {
    pub current_phase: TradePhase,
    pub completed_phases: Vec<TradePhase>,
    pub context: TradeContext,
    pub partial_result: TradeResult,
    pub risk_flags: Vec<RiskAlert>,
}

impl TradeStateMachine {
    pub fn new(context: TradeContext) -> Self {
        Self {
            current_phase: TradePhase::Ft01CustomerDevelopment,
            completed_phases: Vec::new(),
            context,
            partial_result: TradeResult::default(),
            risk_flags: Vec::new(),
        }
    }

    pub fn advance(&mut self, next_phase: TradePhase) -> Result<(), String> {
        if self.can_advance_to(next_phase) {
            self.completed_phases.push(self.current_phase);
            self.current_phase = next_phase;
            Ok(())
        } else {
            Err(format!(
                "Cannot advance from {:?} to {:?}",
                self.current_phase, next_phase
            ))
        }
    }

    fn can_advance_to(&self, next: TradePhase) -> bool {
        // Simple linear progression for now
        let all = TradePhase::all();
        let current_idx = all.iter().position(|p| *p == self.current_phase).unwrap_or(0);
        let next_idx = all.iter().position(|p| *p == next).unwrap_or(0);
        next_idx == current_idx + 1
    }

    pub fn add_risk(&mut self, alert: RiskAlert) {
        self.risk_flags.push(alert);
    }
}

impl Default for TradeResult {
    fn default() -> Self {
        Self {
            quotes: Vec::new(),
            contracts: Vec::new(),
            production_schedule: Schedule {
                milestones: Vec::new(),
                critical_path: Vec::new(),
            },
            inspection_reports: Vec::new(),
            logistics_docs: LogisticsDocSet {
                ciq_certificates: Vec::new(),
                booking_confirmation: None,
                customs_declaration: None,
                bill_of_lading: None,
                packing_list: None,
            },
            finance_docs: FinanceDocSet {
                payment_proofs: Vec::new(),
                lc_review: None,
                collection_records: Vec::new(),
                settlement_records: Vec::new(),
                tax_refund_claims: Vec::new(),
            },
            risk_alerts: Vec::new(),
            recommendations: Vec::new(),
            lessons_learned: Vec::new(),
            knowledge_updates: Vec::new(),
        }
    }
}

/// Entry point for the Keystone capability
pub fn execute_trade_full_cycle(context: TradeContext) -> TradeResult {
    let mut machine = TradeStateMachine::new(context);

    // Phase 1: Customer Acquisition
    machine.current_phase = TradePhase::Ft01CustomerDevelopment;
    // ... execute FT01 logic
    machine.advance(TradePhase::Ft02RequirementConfirmation).ok();
    // ... execute FT02 logic

    // Phase 2: Quotation & Negotiation (delegates to sub-skill)
    machine.current_phase = TradePhase::Ft03DetailedQuotation;
    // ... execute FT03 logic (uses trade_quote_negotiation)
    machine.advance(TradePhase::Ft04NegotiationObjectionHandling).ok();
    // ... execute FT04 logic

    // Phase 3: Contract & Payment (delegates to sub-skill)
    machine.current_phase = TradePhase::Ft05ContractReviewSigning;
    // ... execute FT05 logic (uses trade_finance_compliance)
    machine.advance(TradePhase::Ft06PaymentCollection).ok();
    // ... execute FT06 logic

    // Phase 4: Production (delegates to sub-skill)
    machine.current_phase = TradePhase::Ft07ProductionOrderMaterialPrep;
    // ... execute FT07 logic (uses trade_production_logistics)
    machine.advance(TradePhase::Ft08ProductionTrackingAlerting).ok();
    // ... execute FT08 logic
    machine.advance(TradePhase::Ft09QualityInspectionRelease).ok();
    // ... execute FT09 logic

    // Phase 5: Logistics (delegates to sub-skill)
    machine.current_phase = TradePhase::Ft10InspectionCertification;
    // ... execute FT10 logic (uses trade_production_logistics)
    machine.advance(TradePhase::Ft11BookingPackingList).ok();
    // ... execute FT11 logic
    machine.advance(TradePhase::Ft12CustomsClearance).ok();
    // ... execute FT12 logic
    machine.advance(TradePhase::Ft13BillOfLadingManagement).ok();
    // ... execute FT13 logic

    // Phase 6: Settlement (delegates to sub-skill)
    machine.current_phase = TradePhase::Ft14FinalPaymentCollection;
    // ... execute FT14 logic (uses trade_finance_compliance)
    machine.advance(TradePhase::Ft15SettlementVerification).ok();
    // ... execute FT15 logic
    machine.advance(TradePhase::Ft16TaxRefundDeclaration).ok();
    // ... execute FT16 logic

    // Phase 7: Review
    machine.current_phase = TradePhase::Ft17OrderReviewExperienceAbsorption;
    // ... execute FT17 logic

    machine.partial_result
}

/// Register the Keystone capability node in the CapabilityRegistry
pub fn register_trade_full_cycle_capability(registry: &mut CapabilityRegistry) -> CapabilityNode {
    let node = CapabilityNode::new_constellation(
        "NT-MIND::trade::foreign_trade_full_cycle".to_string(),
        Domain::Mind,
        NodeLayer::L4Cognition,
        vec!["foreign_trade_full_cycle".to_string()],
        vec![
            "trade_quote_negotiation".to_string(),
            "trade_production_logistics".to_string(),
            "trade_finance_compliance".to_string(),
            "trade_product_spec".to_string(),
        ],
    );
    registry.register(node.clone()).expect("Failed to register trade_full_cycle capability");
    node
}

/// Get the capability specification for external consumers
pub fn capability_spec() -> TradeCapabilitySpec {
    TradeCapabilitySpec {
        context: TradeContext {
            buyer_profile: BuyerProfile {
                buyer_id: String::new(),
                company: String::new(),
                country: String::new(),
                industry: String::new(),
                scale: String::new(),
                decision_maker: None,
                contact: None,
                credit_score: None,
                history_orders: Vec::new(),
            },
            inquiry: InquiryDetail {
                inquiry_id: String::new(),
                product: String::new(),
                specs: HashMap::new(),
                qty: 0,
                deadline: String::new(),
                destination: String::new(),
                intent_level: IntentLevel::Low,
            },
            product_spec: ProductSpec {
                spec_id: String::new(),
                product_type: ProductType::Other,
                bom: Vec::new(),
                routing: Vec::new(),
                packaging: PackagingSpec {
                    package_type: String::new(),
                    dimensions_cm: (0.0, 0.0, 0.0),
                    gross_weight_kg: 0.0,
                    net_weight_kg: 0.0,
                    marks: Vec::new(),
                },
                certifications: Vec::new(),
                hs_code: String::new(),
                tax_refund_rate: 0.0,
            },
            company_policy: CompanyPolicy {
                payment_terms: Vec::new(),
                risk_control: RiskControl {
                    max_credit_days: 0,
                    min_deposit_ratio: 0.0,
                    forbidden_countries: Vec::new(),
                    forbidden_products: Vec::new(),
                },
                authorization_matrix: HashMap::new(),
            },
            market_env: MarketEnvironment {
                fx_rates: HashMap::new(),
                freight_rates: HashMap::new(),
                regulations: Vec::new(),
                shipping_lines: Vec::new(),
            },
        },
        result: TradeResult::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trade_phase_all() {
        let phases = TradePhase::all();
        assert_eq!(phases.len(), 17);
    }

    #[test]
    fn test_trade_phase_groups() {
        assert_eq!(
            TradePhase::Ft01CustomerDevelopment.phase_group(),
            TradePhaseGroup::CustomerAcquisition
        );
        assert_eq!(
            TradePhase::Ft03DetailedQuotation.phase_group(),
            TradePhaseGroup::QuotationNegotiation
        );
        assert_eq!(
            TradePhase::Ft07ProductionOrderMaterialPrep.phase_group(),
            TradePhaseGroup::Production
        );
        assert_eq!(
            TradePhase::Ft13BillOfLadingManagement.phase_group(),
            TradePhaseGroup::Logistics
        );
        assert_eq!(
            TradePhase::Ft16TaxRefundDeclaration.phase_group(),
            TradePhaseGroup::Settlement
        );
        assert_eq!(
            TradePhase::Ft17OrderReviewExperienceAbsorption.phase_group(),
            TradePhaseGroup::Review
        );
    }

    #[test]
    fn test_state_machine_linear_advance() {
        let ctx = TradeContext {
            buyer_profile: BuyerProfile {
                buyer_id: "test".into(),
                company: "test".into(),
                country: "US".into(),
                industry: "test".into(),
                scale: "SME".into(),
                decision_maker: None,
                contact: None,
                credit_score: None,
                history_orders: vec![],
            },
            inquiry: InquiryDetail {
                inquiry_id: "inq-1".into(),
                product: "widget".into(),
                specs: HashMap::new(),
                qty: 100,
                deadline: "2026-12-31".into(),
                destination: "US".into(),
                intent_level: IntentLevel::High,
            },
            product_spec: ProductSpec {
                spec_id: "spec-1".into(),
                product_type: ProductType::Machinery,
                bom: vec![],
                routing: vec![],
                packaging: PackagingSpec {
                    package_type: "carton".into(),
                    dimensions_cm: (30.0, 20.0, 15.0),
                    gross_weight_kg: 10.0,
                    net_weight_kg: 8.0,
                    marks: vec![],
                },
                certifications: vec![],
                hs_code: "8471".into(),
                tax_refund_rate: 0.13,
            },
            company_policy: CompanyPolicy {
                payment_terms: vec!["T/T 30% deposit".into()],
                risk_control: RiskControl {
                    max_credit_days: 60,
                    min_deposit_ratio: 0.3,
                    forbidden_countries: vec![],
                    forbidden_products: vec![],
                },
                authorization_matrix: HashMap::new(),
            },
            market_env: MarketEnvironment {
                fx_rates: HashMap::new(),
                freight_rates: HashMap::new(),
                regulations: vec![],
                shipping_lines: vec![],
            },
        };

        let mut machine = TradeStateMachine::new(ctx);
        assert_eq!(machine.current_phase, TradePhase::Ft01CustomerDevelopment);

        machine.advance(TradePhase::Ft02RequirementConfirmation).unwrap();
        assert_eq!(machine.current_phase, TradePhase::Ft02RequirementConfirmation);
        assert_eq!(machine.completed_phases.len(), 1);

        machine.advance(TradePhase::Ft03DetailedQuotation).unwrap();
        assert_eq!(machine.current_phase, TradePhase::Ft03DetailedQuotation);

        // Cannot skip phases
        assert!(machine.advance(TradePhase::Ft05ContractReviewSigning).is_err());
    }

    #[test]
    fn test_capability_spec_structure() {
        let spec = capability_spec();
        assert!(spec.context.buyer_profile.buyer_id.is_empty());
        assert!(spec.context.inquiry.intent_level == IntentLevel::Low);
        assert!(spec.result.quotes.is_empty());
    }

    #[test]
    fn test_trade_phase_to_str() {
        assert_eq!(TradePhase::Ft01CustomerDevelopment.to_str(), "FT01");
        assert_eq!(TradePhase::Ft17OrderReviewExperienceAbsorption.to_str(), "FT17");
    }
}