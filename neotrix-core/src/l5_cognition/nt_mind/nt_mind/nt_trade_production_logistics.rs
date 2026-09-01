//! Trade Production & Logistics Notable Sub-skill
//!
//! Handles FT07-FT13: Production Order → Production Tracking → Quality Inspection →
//! Inspection/Certification → Booking → Customs Clearance → Bill of Lading
//!
//! This is a NOTABLE skill (域级突破) under the foreign_trade_full_cycle Keystone.

use crate::neotrix::nt_core_capability_tree::{
    CapabilityNode, CapabilityRegistry, ConstellationLevel, Domain, NodeLayer,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Production Order (FT07)
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

/// Production Progress Report (FT08)
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

/// Quality Inspection Report (FT09)
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

/// CIQ Certificate (FT10)
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

/// Booking Confirmation (FT11)
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

/// Customs Declaration (FT12)
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
    pub risk_level: RiskLevel,
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
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Bill of Lading (FT13)
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

/// Production Engine (FT07-FT09)
pub struct ProductionEngine;

impl ProductionEngine {
    pub fn create_production_order(
        contract_id: &str,
        product_spec: &super::nt_trade_full_cycle::ProductSpec,
    ) -> ProductionOrder {
        ProductionOrder {
            production_order_id: format!("PO-{}", uuid::Uuid::new_v4().simple()),
            contract_id: contract_id.into(),
            bom: product_spec
                .bom
                .iter()
                .map(|b| BomRequirement {
                    item_id: b.item_id.clone(),
                    name: b.name.clone(),
                    required_qty: b.qty,
                    allocated_qty: 0.0,
                    supplier: b.supplier.clone(),
                    expected_arrival: None,
                    status: MaterialStatus::Pending,
                })
                .collect(),
            routing: product_spec
                .routing
                .iter()
                .map(|r| RoutingRequirement {
                    step_id: r.step_id.clone(),
                    name: r.name.clone(),
                    work_center: r.work_center.clone(),
                    planned_hours: r.duration_hours,
                    actual_hours: None,
                    assigned_operator: None,
                    status: RoutingStatus::NotStarted,
                })
                .collect(),
            schedule: ProductionSchedule {
                start_date: chrono::Utc::now().date_naive().to_string(),
                end_date: (chrono::Utc::now() + chrono::Duration::weeks(8)).date_naive().to_string(),
                critical_path: vec!["cutting".into(), "assembly".into(), "testing".into()],
                buffer_days: 5,
            },
            supplier_orders: Vec::new(),
            milestones: vec![
                ProductionMilestone {
                    name: "Material Ready".into(),
                    planned_date: (chrono::Utc::now() + chrono::Duration::days(7)).date_naive().to_string(),
                    actual_date: None,
                    status: MilestoneStatus::Pending,
                    dependencies: vec![],
                },
                ProductionMilestone {
                    name: "Production Complete".into(),
                    planned_date: (chrono::Utc::now() + chrono::Duration::weeks(6)).date_naive().to_string(),
                    actual_date: None,
                    status: MilestoneStatus::Pending,
                    dependencies: vec!["Material Ready".into()],
                },
                ProductionMilestone {
                    name: "Inspection Pass".into(),
                    planned_date: (chrono::Utc::now() + chrono::Duration::weeks(7)).date_naive().to_string(),
                    actual_date: None,
                    status: MilestoneStatus::Pending,
                    dependencies: vec!["Production Complete".into()],
                },
            ],
        }
    }

    pub fn generate_progress_report(
        order: &ProductionOrder,
    ) -> ProgressReport {
        let completed = order.milestones.iter().filter(|m| m.status == MilestoneStatus::Completed).count();
        let total = order.milestones.len().max(1);
        let overall_progress = completed as f64 / total as f64;

        ProgressReport {
            report_date: chrono::Utc::now().date_naive().to_string(),
            overall_progress,
            daily_progress: Vec::new(),
            deviation: ScheduleDeviation {
                critical_path_delay_days: 0,
                milestone_delays: Vec::new(),
                recovery_plan: None,
            },
            alerts: Vec::new(),
            escalated: false,
        }
    }

    pub fn perform_inspection(
        order: &ProductionOrder,
        aql_level: &str,
    ) -> InspectionReport {
        InspectionReport {
            inspection_id: format!("INSP-{}", uuid::Uuid::new_v4().simple()),
            production_order_id: order.production_order_id.clone(),
            aql_level: aql_level.into(),
            sample_size: 80,
            result: InspectionResult::Pass,
            defects: Vec::new(),
            measurements: Vec::new(),
            photos: Vec::new(),
            inspector: "QC Team".into(),
            inspection_date: chrono::Utc::now().date_naive().to_string(),
        }
    }
}

/// Logistics Engine (FT10-FT13)
pub struct LogisticsEngine;

impl LogisticsEngine {
    pub fn apply_ciq(
        product: &str,
        hs_code: &str,
        qty: u64,
        weight_kg: f64,
    ) -> CiqCertificate {
        CiqCertificate {
            ciq_id: format!("CIQ-{}", uuid::Uuid::new_v4().simple()),
            certificate_no: format!("CN{:08}", rand::random::<u32>() % 100000000),
            product: product.into(),
            hs_code: hs_code.into(),
            qty,
            weight_kg,
            status: CiqStatus::Applied,
            issue_date: chrono::Utc::now().date_naive().to_string(),
            expiry_date: Some(
                (chrono::Utc::now() + chrono::Duration::days(365)).date_naive().to_string()
            ),
        }
    }

    pub fn create_booking(
        cargo_info: &CargoInfo,
        requirements: &BookingRequirements,
    ) -> BookingConfirmation {
        BookingConfirmation {
            booking_id: format!("BK-{}", uuid::Uuid::new_v4().simple()),
            booking_ref: format!("SO{:08}", rand::random::<u32>() % 100000000),
            vessel: "MSC EARTH".into(),
            voyage: "EC234".into(),
            port_of_loading: requirements.port_of_loading.clone(),
            port_of_discharge: requirements.port_of_discharge.clone(),
            eto: (chrono::Utc::now() + chrono::Duration::days(7)).date_naive().to_string(),
            eta: (chrono::Utc::now() + chrono::Duration::days(35)).date_naive().to_string(),
            container_no: None,
            container_type: requirements.container_type.clone(),
            packing_list: Self::generate_packing_list(cargo_info),
            status: BookingStatus::Confirmed,
        }
    }

    pub fn declare_customs(
        contract: &super::nt_trade_full_cycle::Contract,
        product_spec: &super::nt_trade_full_cycle::ProductSpec,
        booking: &BookingConfirmation,
    ) -> CustomsDeclaration {
        CustomsDeclaration {
            declaration_id: format!("CUST-{}", uuid::Uuid::new_v4().simple()),
            customs_code: "01".into(),
            declaration_date: chrono::Utc::now().date_naive().to_string(),
            hs_code: product_spec.hs_code.clone(),
            qty: contract.items.first().map(|i| i.qty).unwrap_or(0),
            unit: "PCS".into(),
            unit_price: contract.items.first().map(|i| i.unit_price).unwrap_or(0.0),
            total_value: contract.price,
            currency: "USD".into(),
            trade_terms: contract.incoterms.clone(),
            origin_country: "CN".into(),
            destination_country: "US".into(),
            status: CustomsStatus::Submitted,
            clearance_time: None,
            risk_level: RiskLevel::Low,
            tax_amount: 0.0,
        }
    }

    pub fn issue_bill_of_lading(
        shipper: &str,
        consignee: &str,
        booking: &BookingConfirmation,
        cargo: &CargoInfo,
    ) -> BillOfLading {
        BillOfLading {
            bl_number: format!("B/L-{:09}", rand::random::<u32>() % 1000000000),
            bl_type: BlType::Original,
            shipper: shipper.into(),
            consignee: consignee.into(),
            notify_party: None,
            vessel: booking.vessel.clone(),
            voyage: booking.voyage.clone(),
            port_of_loading: booking.port_of_loading.clone(),
            port_of_discharge: booking.port_of_discharge.clone(),
            date_of_issue: chrono::Utc::now().date_naive().to_string(),
            place_of_issue: "Shanghai".into(),
            goods_description: cargo.description.clone(),
            marks_numbers: cargo.marks.join(", "),
            packages: format!("{} CTNS", booking.packing_list.total_ctns),
            gross_weight_kg: booking.packing_list.total_gross_kg,
            measurement_cbm: booking.packing_list.total_cbm,
            status: BlStatus::Issued,
            original_count: 3,
            copies_count: 3,
        }
    }

    fn generate_packing_list(cargo: &CargoInfo) -> PackingList {
        PackingList {
            items: vec![PackingItem {
                product: cargo.product.clone(),
                description: cargo.description.clone(),
                qty: cargo.qty,
                ctns: cargo.ctns,
                cbm_per_ctn: cargo.cbm_per_ctn,
                gross_kg_per_ctn: cargo.gross_kg_per_ctn,
                net_kg_per_ctn: cargo.net_kg_per_ctn,
                marks: cargo.marks.clone(),
            }],
            total_ctns: cargo.ctns,
            total_cbm: cargo.cbm_per_ctn * cargo.ctns as f64,
            total_gross_kg: cargo.gross_kg_per_ctn * cargo.ctns as f64,
            total_net_kg: cargo.net_kg_per_ctn * cargo.ctns as f64,
        }
    }
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

/// Register the Notable capability node
pub fn register_production_logistics_capability(registry: &mut CapabilityRegistry) -> CapabilityNode {
    let node = CapabilityNode::new_composite(
        "NT-MIND::trade::trade_production_logistics".to_string(),
        Domain::Mind,
        NodeLayer::L3DomainService,
        vec!["trade_production_logistics".to_string()],
        vec!["trade_product_spec".to_string(), "trade_quote_negotiation".to_string()],
    );
    registry
        .register(node.clone())
        .expect("Failed to register production_logistics capability");
    node
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_order_creation() {
        use super::super::nt_trade_full_cycle::{ProductSpec, BomItem, RoutingStep, PackagingSpec};

        let spec = ProductSpec {
            spec_id: "test".into(),
            product_type: super::super::nt_trade_full_cycle::ProductType::Machinery,
            bom: vec![BomItem {
                item_id: "1".into(),
                name: "Steel".into(),
                qty: 100.0,
                unit: "kg".into(),
                supplier: Some("Supplier A".into()),
                lead_time_days: 7,
            }],
            routing: vec![RoutingStep {
                step_id: "1".into(),
                name: "Cutting".into(),
                work_center: "WC1".into(),
                duration_hours: 8.0,
            }],
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
        };

        let order = ProductionEngine::create_production_order("CONTRACT-1", &spec);
        assert!(!order.production_order_id.is_empty());
        assert_eq!(order.bom.len(), 1);
        assert_eq!(order.routing.len(), 1);
        assert_eq!(order.milestones.len(), 3);
    }

    #[test]
    fn test_progress_report() {
        use super::super::nt_trade_full_cycle::{ProductSpec, BomItem, RoutingStep, PackagingSpec};

        let spec = ProductSpec {
            spec_id: "test".into(),
            product_type: super::super::nt_trade_full_cycle::ProductType::Machinery,
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
        };

        let order = ProductionEngine::create_production_order("CONTRACT-1", &spec);
        let report = ProductionEngine::generate_progress_report(&order);
        assert!(report.overall_progress >= 0.0);
        assert!(report.overall_progress <= 1.0);
    }

    #[test]
    fn test_inspection_report() {
        use super::super::nt_trade_full_cycle::{ProductSpec, BomItem, RoutingStep, PackagingSpec};

        let spec = ProductSpec {
            spec_id: "test".into(),
            product_type: super::super::nt_trade_full_cycle::ProductType::Machinery,
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
        };

        let order = ProductionEngine::create_production_order("CONTRACT-1", &spec);
        let inspection = ProductionEngine::perform_inspection(&order, "AQL 2.5");
        assert_eq!(inspection.aql_level, "AQL 2.5");
        assert!(matches!(inspection.result, InspectionResult::Pass));
    }

    #[test]
    fn test_booking_creation() {
        let cargo = CargoInfo {
            product: "Widget".into(),
            description: "Precision Widget".into(),
            qty: 1000,
            ctns: 50,
            cbm_per_ctn: 0.05,
            gross_kg_per_ctn: 15.0,
            net_kg_per_ctn: 12.0,
            marks: vec!["MADE IN CHINA".into(), "WGT-1000".into()],
        };

        let req = BookingRequirements {
            port_of_loading: "Shanghai".into(),
            port_of_discharge: "Los Angeles".into(),
            container_type: "40HQ".into(),
            earliest_departure: "2026-01-15".into(),
            latest_arrival: "2026-02-20".into(),
        };

        let booking = LogisticsEngine::create_booking(&cargo, &req);
        assert!(!booking.booking_id.is_empty());
        assert_eq!(booking.container_type, "40HQ");
        assert_eq!(booking.packing_list.total_ctns, 50);
    }

    #[test]
    fn test_customs_declaration() {
        use super::super::nt_trade_full_cycle::{Contract, ContractItem, ProductSpec, BomItem, RoutingStep, PackagingSpec};

        let contract = Contract {
            contract_id: "CONTRACT-1".into(),
            pi_number: "PI-1".into(),
            parties: ("Seller".into(), "Buyer".into()),
            items: vec![ContractItem {
                product: "Widget".into(),
                qty: 1000,
                unit_price: 50.0,
            }],
            price: 50000.0,
            incoterms: "FOB Shanghai".into(),
            payment_terms: "T/T 30% deposit, 70% against BL copy".into(),
            delivery_date: "2026-03-01".into(),
        };

        let spec = ProductSpec {
            spec_id: "test".into(),
            product_type: super::super::nt_trade_full_cycle::ProductType::Machinery,
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
        };

        let booking = BookingConfirmation {
            booking_id: "BK-1".into(),
            booking_ref: "SO001".into(),
            vessel: "MSC EARTH".into(),
            voyage: "EC234".into(),
            port_of_loading: "Shanghai".into(),
            port_of_discharge: "Los Angeles".into(),
            eto: "2026-01-15".into(),
            eta: "2026-02-20".into(),
            container_no: None,
            container_type: "40HQ".into(),
            packing_list: PackingList {
                items: vec![],
                total_ctns: 50,
                total_cbm: 2.5,
                total_gross_kg: 750.0,
                total_net_kg: 600.0,
            },
            status: BookingStatus::Confirmed,
        };

        let declaration = LogisticsEngine::declare_customs(&contract, &spec, &booking);
        assert!(!declaration.declaration_id.is_empty());
        assert_eq!(declaration.hs_code, "8471");
        assert_eq!(declaration.total_value, 50000.0);
    }

    #[test]
    fn test_bill_of_lading_issuance() {
        use super::super::nt_trade_full_cycle::{Contract, ContractItem, ProductSpec, BomItem, RoutingStep, PackagingSpec};

        let contract = Contract {
            contract_id: "CONTRACT-1".into(),
            pi_number: "PI-1".into(),
            parties: ("Seller".into(), "Buyer".into()),
            items: vec![ContractItem { product: "Widget".into(), qty: 1000, unit_price: 50.0 }],
            price: 50000.0,
            incoterms: "FOB Shanghai".into(),
            payment_terms: "T/T 30% deposit, 70% against BL copy".into(),
            delivery_date: "2026-03-01".into(),
        };

        let spec = ProductSpec {
            spec_id: "test".into(),
            product_type: super::super::nt_trade_full_cycle::ProductType::Machinery,
            bom: vec![],
            routing: vec![],
            packaging: PackagingSpec { package_type: "carton".into(), dimensions_cm: (30.0, 20.0, 15.0), gross_weight_kg: 10.0, net_weight_kg: 8.0, marks: vec![] },
            certifications: vec![],
            hs_code: "8471".into(),
            tax_refund_rate: 0.13,
        };

        let booking = BookingConfirmation {
            booking_id: "BK-1".into(),
            booking_ref: "SO001".into(),
            vessel: "MSC EARTH".into(),
            voyage: "EC234".into(),
            port_of_loading: "Shanghai".into(),
            port_of_discharge: "Los Angeles".into(),
            eto: "2026-01-15".into(),
            eta: "2026-02-20".into(),
            container_no: None,
            container_type: "40HQ".into(),
            packing_list: PackingList { items: vec![], total_ctns: 50, total_cbm: 2.5, total_gross_kg: 750.0, total_net_kg: 600.0 },
            status: BookingStatus::Confirmed,
        };

        let cargo = CargoInfo {
            product: "Widget".into(),
            description: "Precision Widget".into(),
            qty: 1000,
            ctns: 50,
            cbm_per_ctn: 0.05,
            gross_kg_per_ctn: 15.0,
            net_kg_per_ctn: 12.0,
            marks: vec!["MADE IN CHINA".into()],
        };

        let bl = LogisticsEngine::issue_bill_of_lading("Seller Co.", "Buyer Inc.", &booking, &cargo);
        assert!(!bl.bl_number.is_empty());
        assert_eq!(bl.bl_type, BlType::Original);
        assert_eq!(bl.shipper, "Seller Co.");
        assert_eq!(bl.consignee, "Buyer Inc.");
        assert_eq!(bl.packages, "50 CTNS");
        assert_eq!(bl.original_count, 3);
    }

    #[test]
    fn test_milestone_dependencies() {
        use super::super::nt_trade_full_cycle::{ProductSpec, BomItem, RoutingStep, PackagingSpec};

        let spec = ProductSpec {
            spec_id: "test".into(),
            product_type: super::super::nt_trade_full_cycle::ProductType::Machinery,
            bom: vec![],
            routing: vec![],
            packaging: PackagingSpec { package_type: "carton".into(), dimensions_cm: (30.0, 20.0, 15.0), gross_weight_kg: 10.0, net_weight_kg: 8.0, marks: vec![] },
            certifications: vec![],
            hs_code: "8471".into(),
            tax_refund_rate: 0.13,
        };

        let order = ProductionEngine::create_production_order("CONTRACT-1", &spec);
        assert_eq!(order.milestones.len(), 3);
        assert_eq!(order.milestones[1].dependencies, vec!["Material Ready"]);
        assert_eq!(order.milestones[2].dependencies, vec!["Production Complete"]);
    }

    #[test]
    fn test_packing_list_calculations() {
        let cargo = CargoInfo {
            product: "Widget".into(),
            description: "Test".into(),
            qty: 2000,
            ctns: 100,
            cbm_per_ctn: 0.04,
            gross_kg_per_ctn: 10.0,
            net_kg_per_ctn: 8.0,
            marks: vec![],
        };

        let req = BookingRequirements {
            port_of_loading: "Shanghai".into(),
            port_of_discharge: "Rotterdam".into(),
            container_type: "20GP".into(),
            earliest_departure: "2026-01-01".into(),
            latest_arrival: "2026-02-01".into(),
        };

        let booking = LogisticsEngine::create_booking(&cargo, &req);
        let pl = &booking.packing_list;
        assert_eq!(pl.total_ctns, 100);
        assert!((pl.total_cbm - 4.0).abs() < 0.01);
        assert!((pl.total_gross_kg - 1000.0).abs() < 0.01);
        assert!((pl.total_net_kg - 800.0).abs() < 0.01);
    }

    #[test]
    fn test_ciq_certificate_expiry() {
        let ciq = LogisticsEngine::apply_ciq("Widget", "8471", 1000, 500.0);
        assert_eq!(ciq.status, CiqStatus::Applied);
        assert!(ciq.expiry_date.is_some());
        assert!(ciq.certificate_no.starts_with("CN"));
    }

    #[test]
    fn test_production_order_material_status() {
        use super::super::nt_trade_full_cycle::{ProductSpec, BomItem, RoutingStep, PackagingSpec};

        let spec = ProductSpec {
            spec_id: "test".into(),
            product_type: super::super::nt_trade_full_cycle::ProductType::Machinery,
            bom: vec![
                BomItem { item_id: "1".into(), name: "Steel".into(), qty: 100.0, unit: "kg".into(), supplier: Some("A".into()), lead_time_days: 7 },
                BomItem { item_id: "2".into(), name: "Motor".into(), qty: 1.0, unit: "PCS".into(), supplier: Some("B".into()), lead_time_days: 21 },
            ],
            routing: vec![],
            packaging: PackagingSpec { package_type: "carton".into(), dimensions_cm: (30.0, 20.0, 15.0), gross_weight_kg: 10.0, net_weight_kg: 8.0, marks: vec![] },
            certifications: vec![],
            hs_code: "8471".into(),
            tax_refund_rate: 0.13,
        };

        let order = ProductionEngine::create_production_order("CONTRACT-1", &spec);
        assert_eq!(order.bom.len(), 2);
        assert!(order.bom.iter().all(|b| b.status == MaterialStatus::Pending));
        assert_eq!(order.bom[0].supplier, Some("A".into()));
    }
}