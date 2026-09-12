//! Trade Product Specification Knowledge Pack
//!
//! Provides pluggable product knowledge for the foreign trade full-cycle.
//! Each product type (Machinery, Textile, Food, Chemical, Electronics) has
//! its own knowledge pack with BOM, routing, certifications, HS codes,
//! inspection standards, document templates, risk rules, and compliance maps.

use nt_core_capability_tree::{
    CapabilityNode, CapabilityRegistry, Domain,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Product Knowledge Pack Trait - the plug mechanism
pub(crate) trait ProductKnowledgePack: Send + Sync {
    fn product_type(&self) -> ProductType;
    fn bom_template(&self) -> Vec<BomTemplateItem>;
    fn routing_template(&self) -> Vec<RoutingTemplateStep>;
    fn packaging_spec(&self) -> PackagingSpec;
    fn required_certifications(&self) -> Vec<String>;
    fn hs_code(&self) -> &str;
    fn tax_refund_rate(&self) -> f64;
    fn inspection_standard(&self) -> InspectionStandard;
    fn document_templates(&self) -> DocumentTemplates;
    fn risk_rules(&self) -> Vec<RiskRule>;
    fn compliance_map(&self) -> ComplianceMap;
}

/// Product Types Supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProductType {
    Machinery,
    Textile,
    Food,
    Chemical,
    Electronics,
    Other,
}

/// BOM Template Item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BomTemplateItem {
    pub category: String,
    pub name: String,
    pub specification: String,
    pub unit: String,
    pub qty_per_unit: f64,
    pub lead_time_days: u32,
    pub critical: bool,
    pub approved_suppliers: Vec<String>,
}

/// Routing Template Step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RoutingTemplateStep {
    pub step_id: String,
    pub name: String,
    pub work_center_type: String,
    pub duration_hours: f64,
    pub skill_level: SkillLevel,
    pub quality_checkpoints: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum SkillLevel {
    Unskilled,
    SemiSkilled,
    Skilled,
    Expert,
}

/// Packaging Specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagingSpec {
    pub package_type: String,
    pub dimensions_cm: (f64, f64, f64),
    pub gross_weight_kg: f64,
    pub net_weight_kg: f64,
    pub marks_required: Vec<String>,
    pub special_handling: Vec<String>,
}

/// Inspection Standard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InspectionStandard {
    pub aql_level: String,
    pub sampling_plan: String,
    pub critical_defects: Vec<String>,
    pub major_defects: Vec<String>,
    pub minor_defects: Vec<String>,
    pub test_methods: Vec<TestMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TestMethod {
    pub parameter: String,
    pub method: String,
    pub equipment: String,
    pub acceptance_criteria: String,
}

/// Document Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocumentTemplates {
    pub commercial_invoice: String,
    pub packing_list: String,
    pub certificate_of_origin: String,
    pub additional: HashMap<String, String>,
}

/// Risk Rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskRule {
    pub rule_id: String,
    pub category: RiskCategory,
    pub condition: String,
    pub action: RiskAction,
    pub severity: RiskSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskCategory {
    Quality,
    Delivery,
    Compliance,
    Financial,
    Ip,
    SupplyChain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum RiskAction {
    Block,
    Warn,
    RequireApproval,
    RequireInspection,
    RequireDocumentation,
    Monitor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Compliance Map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ComplianceMap {
    pub target_countries: HashMap<String, CountryCompliance>,
    pub international_standards: Vec<String>,
    pub trade_agreements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CountryCompliance {
    pub country_code: String,
    pub required_certifications: Vec<String>,
    pub labeling_requirements: Vec<String>,
    pub restricted_substances: Vec<String>,
    pub import_license_required: bool,
    pub tariff_rate: f64,
    pub anti_dumping: bool,
    pub quota: Option<f64>,
}

/// Machinery Knowledge Pack
#[derive(Debug, Clone)]
pub(crate) struct MachineryKnowledgePack;

impl ProductKnowledgePack for MachineryKnowledgePack {
    fn product_type(&self) -> ProductType {
        ProductType::Machinery
    }

    fn bom_template(&self) -> Vec<BomTemplateItem> {
        vec![
            BomTemplateItem {
                category: "Frame".into(),
                name: "Steel Frame".into(),
                specification: "Q235B, welded".into(),
                unit: "SET".into(),
                qty_per_unit: 1.0,
                lead_time_days: 14,
                critical: true,
                approved_suppliers: vec!["Supplier A".into(), "Supplier B".into()],
            },
            BomTemplateItem {
                category: "Motor".into(),
                name: "AC Motor 5.5kW".into(),
                specification: "IE3, 380V/50Hz".into(),
                unit: "PCS".into(),
                qty_per_unit: 1.0,
                lead_time_days: 21,
                critical: true,
                approved_suppliers: vec!["Siemens".into(), "ABB".into(), "WEG".into()],
            },
            BomTemplateItem {
                category: "Control".into(),
                name: "PLC Controller".into(),
                specification: "Modbus/TCP, 16DI/16DO".into(),
                unit: "PCS".into(),
                qty_per_unit: 1.0,
                lead_time_days: 10,
                critical: true,
                approved_suppliers: vec!["Mitsubishi".into(), "Omron".into()],
            },
            BomTemplateItem {
                category: "Safety".into(),
                name: "Light Curtain".into(),
                specification: "Type 4, 300mm resolution".into(),
                unit: "SET".into(),
                qty_per_unit: 1.0,
                lead_time_days: 7,
                critical: true,
                approved_suppliers: vec!["Sick".into(), "Keyence".into()],
            },
        ]
    }

    fn routing_template(&self) -> Vec<RoutingTemplateStep> {
        vec![
            RoutingTemplateStep {
                step_id: "010".into(),
                name: "Frame Fabrication".into(),
                work_center_type: "Welding".into(),
                duration_hours: 16.0,
                skill_level: SkillLevel::Skilled,
                quality_checkpoints: vec!["Weld visual inspection".into(), "Dimensional check".into()],
            },
            RoutingTemplateStep {
                step_id: "020".into(),
                name: "Machining".into(),
                work_center_type: "CNC".into(),
                duration_hours: 8.0,
                skill_level: SkillLevel::Skilled,
                quality_checkpoints: vec!["Surface finish".into(), "Tolerance check".into()],
            },
            RoutingTemplateStep {
                step_id: "030".into(),
                name: "Assembly".into(),
                work_center_type: "Assembly".into(),
                duration_hours: 24.0,
                skill_level: SkillLevel::Skilled,
                quality_checkpoints: vec!["Torque verification".into(), "Alignment check".into()],
            },
            RoutingTemplateStep {
                step_id: "040".into(),
                name: "Wiring & Integration".into(),
                work_center_type: "Electrical".into(),
                duration_hours: 12.0,
                skill_level: SkillLevel::Skilled,
                quality_checkpoints: vec!["Continuity test".into(), "Insulation resistance".into()],
            },
            RoutingTemplateStep {
                step_id: "050".into(),
                name: "Functional Test".into(),
                work_center_type: "Test".into(),
                duration_hours: 8.0,
                skill_level: SkillLevel::Expert,
                quality_checkpoints: vec!["Performance test".into(), "Safety function test".into()],
            },
            RoutingTemplateStep {
                step_id: "060".into(),
                name: "Painting & Packing".into(),
                work_center_type: "Finish".into(),
                duration_hours: 8.0,
                skill_level: SkillLevel::SemiSkilled,
                quality_checkpoints: vec!["Paint thickness".into(), "Packing integrity".into()],
            },
        ]
    }

    fn packaging_spec(&self) -> PackagingSpec {
        PackagingSpec {
            package_type: "Wooden crate with steel frame".into(),
            dimensions_cm: (120.0, 80.0, 100.0),
            gross_weight_kg: 350.0,
            net_weight_kg: 300.0,
            marks_required: vec!["CE".into(), "MADE IN CHINA".into(), "FRAGILE".into(), "THIS SIDE UP".into()],
            special_handling: vec!["Keep dry".into(), "Do not stack".into()],
        }
    }

    fn required_certifications(&self) -> Vec<String> {
        vec!["CE".into(), "ISO 9001".into(), "ISO 12100".into()]
    }

    fn hs_code(&self) -> &str {
        "8479.89"
    }

    fn tax_refund_rate(&self) -> f64 {
        0.13
    }

    fn inspection_standard(&self) -> InspectionStandard {
        InspectionStandard {
            aql_level: "AQL 1.0/2.5".into(),
            sampling_plan: "ISO 2859-1".into(),
            critical_defects: vec![
                "Safety guard missing".into(),
                "Emergency stop non-functional".into(),
                "Electrical insulation failure".into(),
            ],
            major_defects: vec![
                "Dimensional out of tolerance".into(),
                "Motor performance below spec".into(),
                "Control system communication error".into(),
            ],
            minor_defects: vec![
                "Paint scratch".into(),
                "Label misaligned".into(),
                "Minor surface rust".into(),
            ],
            test_methods: vec![
                TestMethod {
                    parameter: "No-load current".into(),
                    method: "Clamp meter measurement".into(),
                    equipment: "Clamp meter".into(),
                    acceptance_criteria: "Within ±10% of nameplate".into(),
                },
                TestMethod {
                    parameter: "Vibration".into(),
                    method: "ISO 10816".into(),
                    equipment: "Vibration analyzer".into(),
                    acceptance_criteria: "Zone A (< 2.8 mm/s)".into(),
                },
            ],
        }
    }

    fn document_templates(&self) -> DocumentTemplates {
        let mut additional = HashMap::new();
        additional.insert("CE Declaration".into(), "ce_declaration_template".into());
        additional.insert("User Manual".into(), "user_manual_template".into());
        additional.insert("Maintenance Schedule".into(), "maintenance_schedule_template".into());

        DocumentTemplates {
            commercial_invoice: "ci_machinery_template".into(),
            packing_list: "pl_machinery_template".into(),
            certificate_of_origin: "co_generic_template".into(),
            additional,
        }
    }

    fn risk_rules(&self) -> Vec<RiskRule> {
        vec![
            RiskRule {
                rule_id: "MACH-001".into(),
                category: RiskCategory::Quality,
                condition: "Motor efficiency below IE3".into(),
                action: RiskAction::Block,
                severity: RiskSeverity::Critical,
            },
            RiskRule {
                rule_id: "MACH-002".into(),
                category: RiskCategory::Compliance,
                condition: "Missing CE certification".into(),
                action: RiskAction::Block,
                severity: RiskSeverity::Critical,
            },
            RiskRule {
                rule_id: "MACH-003".into(),
                category: RiskCategory::Ip,
                condition: "Custom design without NDA".into(),
                action: RiskAction::RequireApproval,
                severity: RiskSeverity::High,
            },
            RiskRule {
                rule_id: "MACH-004".into(),
                category: RiskCategory::SupplyChain,
                condition: "Single source for critical component".into(),
                action: RiskAction::Warn,
                severity: RiskSeverity::Medium,
            },
        ]
    }

    fn compliance_map(&self) -> ComplianceMap {
        let mut countries = HashMap::new();
        countries.insert("US".into(), CountryCompliance {
            country_code: "US".into(),
            required_certifications: vec!["UL".into(), "CE".into()],
            labeling_requirements: vec!["UL mark".into(), "Rating plate".into()],
            restricted_substances: vec!["PCB".into(), "Asbestos".into()],
            import_license_required: false,
            tariff_rate: 0.0,
            anti_dumping: false,
            quota: None,
        });
        countries.insert("EU".into(), CountryCompliance {
            country_code: "EU".into(),
            required_certifications: vec!["CE".into()],
            labeling_requirements: vec!["CE mark".into(), "RoHS compliance".into()],
            restricted_substances: vec!["Lead".into(), "Mercury".into(), "Cd".into(), "Cr6+".into()],
            import_license_required: false,
            tariff_rate: 0.0,
            anti_dumping: false,
            quota: None,
        });

        ComplianceMap {
            target_countries: countries,
            international_standards: vec!["ISO 9001".into(), "ISO 12100".into(), "IEC 60204-1".into()],
            trade_agreements: vec!["RCEP".into(), "CPTPP".into()],
        }
    }
}

/// Textile Knowledge Pack
#[derive(Debug, Clone)]
pub(crate) struct TextileKnowledgePack;

impl ProductKnowledgePack for TextileKnowledgePack {
    fn product_type(&self) -> ProductType {
        ProductType::Textile
    }

    fn bom_template(&self) -> Vec<BomTemplateItem> {
        vec![
            BomTemplateItem {
                category: "Fabric".into(),
                name: "Cotton Fabric 200gsm".into(),
                specification: "100% Cotton, OEKO-TEX Std 100".into(),
                unit: "MTR".into(),
                qty_per_unit: 2.5,
                lead_time_days: 10,
                critical: true,
                approved_suppliers: vec!["Fabric Mill A".into(), "Fabric Mill B".into()],
            },
            BomTemplateItem {
                category: "Trim".into(),
                name: "YKK Zipper #5".into(),
                specification: "Nylon, auto-lock".into(),
                unit: "PCS".into(),
                qty_per_unit: 1.0,
                lead_time_days: 7,
                critical: false,
                approved_suppliers: vec!["YKK".into(), "SBS".into()],
            },
            BomTemplateItem {
                category: "Thread".into(),
                name: "Polyester Thread 120D".into(),
                specification: "Color-matched".into(),
                unit: "CONE".into(),
                qty_per_unit: 3.0,
                lead_time_days: 3,
                critical: false,
                approved_suppliers: vec!["Coats".into(), "A&E".into()],
            },
            BomTemplateItem {
                category: "Label".into(),
                name: "Woven Label".into(),
                specification: "Damask, 100% polyester".into(),
                unit: "PCS".into(),
                qty_per_unit: 1.0,
                lead_time_days: 5,
                critical: false,
                approved_suppliers: vec!["LabelCo".into()],
            },
        ]
    }

    fn routing_template(&self) -> Vec<RoutingTemplateStep> {
        vec![
            RoutingTemplateStep {
                step_id: "010".into(),
                name: "Fabric Inspection".into(),
                work_center_type: "Inspection".into(),
                duration_hours: 2.0,
                skill_level: SkillLevel::Skilled,
                quality_checkpoints: vec!["Color matching".into(), "Defect mapping".into(), "Shrinkage test".into()],
            },
            RoutingTemplateStep {
                step_id: "020".into(),
                name: "Spreading & Cutting".into(),
                work_center_type: "Cutting".into(),
                duration_hours: 4.0,
                skill_level: SkillLevel::SemiSkilled,
                quality_checkpoints: vec!["Marker efficiency".into(), "Cut accuracy".into()],
            },
            RoutingTemplateStep {
                step_id: "030".into(),
                name: "Sewing".into(),
                work_center_type: "Sewing".into(),
                duration_hours: 12.0,
                skill_level: SkillLevel::Skilled,
                quality_checkpoints: vec!["Stitch density".into(), "Seam strength".into(), "Thread tension".into()],
            },
            RoutingTemplateStep {
                step_id: "040".into(),
                name: "Finishing".into(),
                work_center_type: "Finishing".into(),
                duration_hours: 3.0,
                skill_level: SkillLevel::SemiSkilled,
                quality_checkpoints: vec!["Trimming".into(), "Pressing".into(), "Final measurement".into()],
            },
            RoutingTemplateStep {
                step_id: "050".into(),
                name: "Final Inspection & Packing".into(),
                work_center_type: "QC".into(),
                duration_hours: 2.0,
                skill_level: SkillLevel::Skilled,
                quality_checkpoints: vec!["AQL inspection".into(), "Needle detection".into(), "Packing check".into()],
            },
        ]
    }

    fn packaging_spec(&self) -> PackagingSpec {
        PackagingSpec {
            package_type: "Polybag + Export Carton".into(),
            dimensions_cm: (50.0, 40.0, 30.0),
            gross_weight_kg: 12.0,
            net_weight_kg: 10.0,
            marks_required: vec!["MADE IN CHINA".into(), "CARE LABEL".into(), "SIZE".into()],
            special_handling: vec!["Keep dry".into()],
        }
    }

    fn required_certifications(&self) -> Vec<String> {
        vec!["OEKO-TEX Standard 100".into(), "BSCI".into()]
    }

    fn hs_code(&self) -> &str {
        "6109.10"
    }

    fn tax_refund_rate(&self) -> f64 {
        0.13
    }

    fn inspection_standard(&self) -> InspectionStandard {
        InspectionStandard {
            aql_level: "AQL 2.5/4.0".into(),
            sampling_plan: "ISO 2859-1".into(),
            critical_defects: vec![
                "Needle/foreign object".into(),
                "Fabric hole/tear".into(),
                "Color bleeding".into(),
            ],
            major_defects: vec![
                "Open seam".into(),
                "Broken stitch".into(),
                "Measurement out of spec > 1cm".into(),
            ],
            minor_defects: vec![
                "Loose thread".into(),
                "Slight shade variation".into(),
                "Label crooked".into(),
            ],
            test_methods: vec![
                TestMethod {
                    parameter: "Colorfastness".into(),
                    method: "ISO 105-C06".into(),
                    equipment: "Crockmeter".into(),
                    acceptance_criteria: "Grade 4 minimum".into(),
                },
                TestMethod {
                    parameter: "Dimensional stability".into(),
                    method: "ISO 6330".into(),
                    equipment: "Washing machine".into(),
                    acceptance_criteria: "Within ±3%".into(),
                },
            ],
        }
    }

    fn document_templates(&self) -> DocumentTemplates {
        let mut additional = HashMap::new();
        additional.insert("Composition Declaration".into(), "composition_declaration_template".into());
        additional.insert("OEKO-TEX Certificate".into(), "oekotex_certificate_template".into());

        DocumentTemplates {
            commercial_invoice: "ci_textile_template".into(),
            packing_list: "pl_textile_template".into(),
            certificate_of_origin: "co_generic_template".into(),
            additional,
        }
    }

    fn risk_rules(&self) -> Vec<RiskRule> {
        vec![
            RiskRule {
                rule_id: "TEXT-001".into(),
                category: RiskCategory::Compliance,
                condition: "AZO dyes detected".into(),
                action: RiskAction::Block,
                severity: RiskSeverity::Critical,
            },
            RiskRule {
                rule_id: "TEXT-002".into(),
                category: RiskCategory::Quality,
                condition: "Colorfastness below Grade 3".into(),
                action: RiskAction::RequireInspection,
                severity: RiskSeverity::High,
            },
            RiskRule {
                rule_id: "TEXT-003".into(),
                category: RiskCategory::Delivery,
                condition: "Fabric lead time > 14 days".into(),
                action: RiskAction::Warn,
                severity: RiskSeverity::Medium,
            },
        ]
    }

    fn compliance_map(&self) -> ComplianceMap {
        let mut countries = HashMap::new();
        countries.insert("US".into(), CountryCompliance {
            country_code: "US".into(),
            required_certifications: vec!["FTC Care Label".into(), "CPSIA".into()],
            labeling_requirements: vec!["Fiber content".into(), "Country of origin".into(), "Care instructions".into()],
            restricted_substances: vec!["Lead in paint".into(), "Phthalates".into()],
            import_license_required: false,
            tariff_rate: 0.165,
            anti_dumping: false,
            quota: None,
        });
        countries.insert("EU".into(), CountryCompliance {
            country_code: "EU".into(),
            required_certifications: vec!["REACH".into(), "Textile Regulation".into()],
            labeling_requirements: vec!["Fiber composition".into(), "Care symbols".into()],
            restricted_substances: vec!["AZO dyes".into(), "Nickel release".into(), "Formaldehyde".into()],
            import_license_required: false,
            tariff_rate: 0.12,
            anti_dumping: false,
            quota: None,
        });

        ComplianceMap {
            target_countries: countries,
            international_standards: vec!["OEKO-TEX".into(), "GOTS".into(), "Bluesign".into()],
            trade_agreements: vec!["RCEP".into()],
        }
    }
}

/// Food Knowledge Pack
#[derive(Debug, Clone)]
pub(crate) struct FoodKnowledgePack;

impl ProductKnowledgePack for FoodKnowledgePack {
    fn product_type(&self) -> ProductType {
        ProductType::Food
    }

    fn bom_template(&self) -> Vec<BomTemplateItem> {
        vec![
            BomTemplateItem {
                category: "Raw Material".into(),
                name: "Premium Green Tea".into(),
                specification: "Grade A, Organic certified".into(),
                unit: "KG".into(),
                qty_per_unit: 0.5,
                lead_time_days: 30,
                critical: true,
                approved_suppliers: vec!["Tea Estate A".into(), "Tea Estate B".into()],
            },
            BomTemplateItem {
                category: "Packaging".into(),
                name: "Tin Can 250g".into(),
                specification: "Food grade, vacuum seal".into(),
                unit: "PCS".into(),
                qty_per_unit: 1.0,
                lead_time_days: 14,
                critical: true,
                approved_suppliers: vec!["Can Maker A".into()],
            },
            BomTemplateItem {
                category: "Additive".into(),
                name: "Natural Flavor".into(),
                specification: "FEMA GRAS approved".into(),
                unit: "KG".into(),
                qty_per_unit: 0.01,
                lead_time_days: 7,
                critical: false,
                approved_suppliers: vec!["Flavor House A".into()],
            },
        ]
    }

    fn routing_template(&self) -> Vec<RoutingTemplateStep> {
        vec![
            RoutingTemplateStep {
                step_id: "010".into(),
                name: "Raw Material Inspection".into(),
                work_center_type: "QC Lab".into(),
                duration_hours: 4.0,
                skill_level: SkillLevel::Expert,
                quality_checkpoints: vec!["Pesticide residue".into(), "Heavy metals".into(), "Microbiological".into(), "Sensory evaluation".into()],
            },
            RoutingTemplateStep {
                step_id: "020".into(),
                name: "Processing".into(),
                work_center_type: "Processing".into(),
                duration_hours: 8.0,
                skill_level: SkillLevel::Skilled,
                quality_checkpoints: vec!["Temperature control".into(), "Time control".into(), "Hygiene monitoring".into()],
            },
            RoutingTemplateStep {
                step_id: "030".into(),
                name: "Filling & Sealing".into(),
                work_center_type: "Filling".into(),
                duration_hours: 4.0,
                skill_level: SkillLevel::SemiSkilled,
                quality_checkpoints: vec!["Fill weight".into(), "Vacuum level".into(), "Seal integrity".into()],
            },
            RoutingTemplateStep {
                step_id: "040".into(),
                name: "Sterilization".into(),
                work_center_type: "Sterilization".into(),
                duration_hours: 2.0,
                skill_level: SkillLevel::Skilled,
                quality_checkpoints: vec!["F0 value".into(), "Temperature profile".into()],
            },
            RoutingTemplateStep {
                step_id: "050".into(),
                name: "Cooling & Inspection".into(),
                work_center_type: "QC".into(),
                duration_hours: 3.0,
                skill_level: SkillLevel::Skilled,
                quality_checkpoints: vec!["Incubation test".into(), "Seal test".into(), "Label check".into()],
            },
            RoutingTemplateStep {
                step_id: "060".into(),
                name: "Packing".into(),
                work_center_type: "Packing".into(),
                duration_hours: 2.0,
                skill_level: SkillLevel::SemiSkilled,
                quality_checkpoints: vec!["Carton weight".into(), "Marking".into()],
            },
        ]
    }

    fn packaging_spec(&self) -> PackagingSpec {
        PackagingSpec {
            package_type: "Tin can + Master carton".into(),
            dimensions_cm: (40.0, 30.0, 20.0),
            gross_weight_kg: 8.0,
            net_weight_kg: 6.0,
            marks_required: vec![
                "MADE IN CHINA".into(),
                "NET WT 250G".into(),
                "KEEP COOL & DRY".into(),
                "BEST BEFORE".into(),
                "NUTRITION FACTS".into(),
            ],
            special_handling: vec!["Cold chain required".into(), "Do not freeze".into()],
        }
    }

    fn required_certifications(&self) -> Vec<String> {
        vec!["HACCP".into(), "ISO 22000".into(), "Organic (if applicable)".into(), "FDA Registration (US)".into()]
    }

    fn hs_code(&self) -> &str {
        "0902.10"
    }

    fn tax_refund_rate(&self) -> f64 {
        0.13
    }

    fn inspection_standard(&self) -> InspectionStandard {
        InspectionStandard {
            aql_level: "Zero tolerance for critical".into(),
            sampling_plan: "ISO 2859-1 + FDA BAM".into(),
            critical_defects: vec![
                "Pathogen detected".into(),
                "Pesticide > MRL".into(),
                "Heavy metal > limit".into(),
                "Foreign material".into(),
            ],
            major_defects: vec![
                "Seal failure".into(),
                "Underweight".into(),
                "Label error (allergen)".into(),
            ],
            minor_defects: vec![
                "Minor dent on can".into(),
                "Print smudge".into(),
            ],
            test_methods: vec![
                TestMethod {
                    parameter: "Total plate count".into(),
                    method: "FDA BAM Ch.3".into(),
                    equipment: "Incubator".into(),
                    acceptance_criteria: "< 1000 CFU/g".into(),
                },
                TestMethod {
                    parameter: "Pesticide residue".into(),
                    method: "GB 23200".into(),
                    equipment: "GC-MS/LC-MS".into(),
                    acceptance_criteria: "Below China/Import country MRL".into(),
                },
            ],
        }
    }

    fn document_templates(&self) -> DocumentTemplates {
        let mut additional = HashMap::new();
        additional.insert("Health Certificate".into(), "health_certificate_template".into());
        additional.insert("Phytosanitary Certificate".into(), "phyto_certificate_template".into());
        additional.insert("Certificate of Analysis".into(), "coa_template".into());
        additional.insert("Free Sale Certificate".into(), "free_sale_template".into());

        DocumentTemplates {
            commercial_invoice: "ci_food_template".into(),
            packing_list: "pl_food_template".into(),
            certificate_of_origin: "co_generic_template".into(),
            additional,
        }
    }

    fn risk_rules(&self) -> Vec<RiskRule> {
        vec![
            RiskRule {
                rule_id: "FOOD-001".into(),
                category: RiskCategory::Compliance,
                condition: "Missing health certificate".into(),
                action: RiskAction::Block,
                severity: RiskSeverity::Critical,
            },
            RiskRule {
                rule_id: "FOOD-002".into(),
                category: RiskCategory::Quality,
                condition: "Cold chain break detected".into(),
                action: RiskAction::Block,
                severity: RiskSeverity::Critical,
            },
            RiskRule {
                rule_id: "FOOD-003".into(),
                category: RiskCategory::Compliance,
                condition: "Label missing allergen declaration".into(),
                action: RiskAction::Block,
                severity: RiskSeverity::Critical,
            },
            RiskRule {
                rule_id: "FOOD-004".into(),
                category: RiskCategory::SupplyChain,
                condition: "Shelf life < 60% at shipment".into(),
                action: RiskAction::Warn,
                severity: RiskSeverity::High,
            },
        ]
    }

    fn compliance_map(&self) -> ComplianceMap {
        let mut countries = HashMap::new();
        countries.insert("US".into(), CountryCompliance {
            country_code: "US".into(),
            required_certifications: vec!["FDA Prior Notice".into(), "FSVP".into()],
            labeling_requirements: vec!["Nutrition Facts".into(), "Ingredient list".into(), "Allergen declaration".into()],
            restricted_substances: vec!["Melamine".into(), "Clenbuterol".into()],
            import_license_required: true,
            tariff_rate: 0.0,
            anti_dumping: false,
            quota: None,
        });
        countries.insert("EU".into(), CountryCompliance {
            country_code: "EU".into(),
            required_certifications: vec!["Health Certificate".into(), "Official Certificate".into()],
            labeling_requirements: vec!["EU Nutrition Label".into(), "Allergen highlighting".into()],
            restricted_substances: vec!["Pesticide MRLs".into(), "Veterinary drug residues".into()],
            import_license_required: false,
            tariff_rate: 0.0,
            anti_dumping: false,
            quota: None,
        });

        ComplianceMap {
            target_countries: countries,
            international_standards: vec!["Codex Alimentarius".into(), "HACCP".into(), "ISO 22000".into()],
            trade_agreements: vec!["RCEP".into()],
        }
    }
}

/// Factory function to get the appropriate knowledge pack
pub(crate) fn get_product_knowledge_pack(product_type: ProductType) -> Box<dyn ProductKnowledgePack> {
    match product_type {
        ProductType::Machinery => Box::new(MachineryKnowledgePack),
        ProductType::Textile => Box::new(TextileKnowledgePack),
        ProductType::Food => Box::new(FoodKnowledgePack),
        ProductType::Chemical => Box::new(ChemicalKnowledgePack),
        ProductType::Electronics => Box::new(ElectronicsKnowledgePack),
        ProductType::Other => Box::new(GenericKnowledgePack),
    }
}

/// Chemical Knowledge Pack (placeholder)
#[derive(Debug, Clone)]
pub(crate) struct ChemicalKnowledgePack;

impl ProductKnowledgePack for ChemicalKnowledgePack {
    fn product_type(&self) -> ProductType { ProductType::Chemical }
    fn bom_template(&self) -> Vec<BomTemplateItem> { vec![] }
    fn routing_template(&self) -> Vec<RoutingTemplateStep> { vec![] }
    fn packaging_spec(&self) -> PackagingSpec { PackagingSpec { package_type: "UN certified drum".into(), dimensions_cm: (58.0, 58.0, 88.0), gross_weight_kg: 200.0, net_weight_kg: 180.0, marks_required: vec!["UN MARK".into(), "HAZCHEM".into()], special_handling: vec!["DG handling".into()] } }
    fn required_certifications(&self) -> Vec<String> { vec!["REACH".into(), "SDS".into()] }
    fn hs_code(&self) -> &str { "2901.10" }
    fn tax_refund_rate(&self) -> f64 { 0.13 }
    fn inspection_standard(&self) -> InspectionStandard { InspectionStandard { aql_level: "AQL 0.65".into(), sampling_plan: "ISO 2859-1".into(), critical_defects: vec![], major_defects: vec![], minor_defects: vec![], test_methods: vec![] } }
    fn document_templates(&self) -> DocumentTemplates { DocumentTemplates { commercial_invoice: "ci_chem_template".into(), packing_list: "pl_chem_template".into(), certificate_of_origin: "co_generic_template".into(), additional: HashMap::from([("SDS".into(), "sds_template".into()), ("DG Declaration".into(), "dg_declaration_template".into())]) } }
    fn risk_rules(&self) -> Vec<RiskRule> { vec![] }
    fn compliance_map(&self) -> ComplianceMap { ComplianceMap { target_countries: HashMap::new(), international_standards: vec![], trade_agreements: vec![] } }
}

/// Electronics Knowledge Pack (placeholder)
#[derive(Debug, Clone)]
pub(crate) struct ElectronicsKnowledgePack;

impl ProductKnowledgePack for ElectronicsKnowledgePack {
    fn product_type(&self) -> ProductType { ProductType::Electronics }
    fn bom_template(&self) -> Vec<BomTemplateItem> { vec![] }
    fn routing_template(&self) -> Vec<RoutingTemplateStep> { vec![] }
    fn packaging_spec(&self) -> PackagingSpec { PackagingSpec { package_type: "Anti-static bag + Carton".into(), dimensions_cm: (40.0, 30.0, 20.0), gross_weight_kg: 5.0, net_weight_kg: 3.0, marks_required: vec!["ESD".into(), "CE".into(), "RoHS".into()], special_handling: vec!["Anti-static".into()] } }
    fn required_certifications(&self) -> Vec<String> { vec!["CE".into(), "RoHS".into(), "FCC".into()] }
    fn hs_code(&self) -> &str { "8542.31" }
    fn tax_refund_rate(&self) -> f64 { 0.13 }
    fn inspection_standard(&self) -> InspectionStandard { InspectionStandard { aql_level: "AQL 1.0".into(), sampling_plan: "ISO 2859-1".into(), critical_defects: vec![], major_defects: vec![], minor_defects: vec![], test_methods: vec![] } }
    fn document_templates(&self) -> DocumentTemplates { DocumentTemplates { commercial_invoice: "ci_elec_template".into(), packing_list: "pl_elec_template".into(), certificate_of_origin: "co_generic_template".into(), additional: HashMap::from([("RoHS DoC".into(), "roh_doc_template".into())]) } }
    fn risk_rules(&self) -> Vec<RiskRule> { vec![] }
    fn compliance_map(&self) -> ComplianceMap { ComplianceMap { target_countries: HashMap::new(), international_standards: vec![], trade_agreements: vec![] } }
}

/// Generic Knowledge Pack (fallback)
#[derive(Debug, Clone)]
pub(crate) struct GenericKnowledgePack;

impl ProductKnowledgePack for GenericKnowledgePack {
    fn product_type(&self) -> ProductType { ProductType::Other }
    fn bom_template(&self) -> Vec<BomTemplateItem> { vec![] }
    fn routing_template(&self) -> Vec<RoutingTemplateStep> { vec![] }
    fn packaging_spec(&self) -> PackagingSpec { PackagingSpec { package_type: "Standard export carton".into(), dimensions_cm: (40.0, 30.0, 20.0), gross_weight_kg: 10.0, net_weight_kg: 8.0, marks_required: vec!["MADE IN CHINA".into()], special_handling: vec![] } }
    fn required_certifications(&self) -> Vec<String> { vec![] }
    fn hs_code(&self) -> &str { "9999.99" }
    fn tax_refund_rate(&self) -> f64 { 0.13 }
    fn inspection_standard(&self) -> InspectionStandard { InspectionStandard { aql_level: "AQL 2.5".into(), sampling_plan: "ISO 2859-1".into(), critical_defects: vec![], major_defects: vec![], minor_defects: vec![], test_methods: vec![] } }
    fn document_templates(&self) -> DocumentTemplates { DocumentTemplates { commercial_invoice: "ci_generic_template".into(), packing_list: "pl_generic_template".into(), certificate_of_origin: "co_generic_template".into(), additional: HashMap::new() } }
    fn risk_rules(&self) -> Vec<RiskRule> { vec![] }
    fn compliance_map(&self) -> ComplianceMap { ComplianceMap { target_countries: HashMap::new(), international_standards: vec![], trade_agreements: vec![] } }
}

/// Register the ProductSpec capability node
pub(crate) fn register_product_spec_capability(registry: &mut CapabilityRegistry) -> CapabilityNode {
    let node = CapabilityNode::new_primitive(
        "NT-MEMORY::trade::trade_product_spec".to_string(),
        Domain::Memory,
        vec!["trade_product_spec".to_string()],
    );
    registry.register(node.clone()).expect("Failed to register product_spec capability");
    node
}

/// KB Persistence for ProductKnowledgePack
/// Uses the binary_assets table with namespace "domain_nt_trade"
pub mod kb_persistence {
    use super::*;
    use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
    use serde_json;

    const KB_NAMESPACE: &str = "domain_nt_trade";

    /// Store a ProductKnowledgePack as JSON in KB
    pub fn store_knowledge_pack(kb: &KnowledgeBase, pack: &dyn ProductKnowledgePack) -> Result<String, String> {
        let product_type = pack.product_type();
        let name = format!("product_spec_{:?}", product_type).to_lowercase();
        
        let json = serde_json::json!({
            "product_type": format!("{:?}", product_type),
            "bom_template": pack.bom_template(),
            "routing_template": pack.routing_template(),
            "packaging_spec": pack.packaging_spec(),
            "required_certifications": pack.required_certifications(),
            "hs_code": pack.hs_code(),
            "tax_refund_rate": pack.tax_refund_rate(),
            "inspection_standard": pack.inspection_standard(),
            "document_templates": pack.document_templates(),
            "risk_rules": pack.risk_rules(),
            "compliance_map": pack.compliance_map(),
        });
        
        let data = serde_json::to_vec(&json).map_err(|e| format!("serialize: {}", e))?;
        kb.asset_store(KB_NAMESPACE, &name, &data, Some("application/json"), Some(&json))
    }

    /// Load a ProductKnowledgePack from KB by product type
    pub fn load_knowledge_pack(kb: &KnowledgeBase, product_type: ProductType) -> Result<Option<serde_json::Value>, String> {
        let name = format!("product_spec_{:?}", product_type).to_lowercase();
        let id = format!("{}/{}", KB_NAMESPACE, name);
        
        match kb.asset_load(&id) {
            Ok(Some(result)) => {
                let data = &result.0;
                let json: serde_json::Value = serde_json::from_slice(data).map_err(|e| format!("deserialize: {}", e))?;
                Ok(Some(json))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// List all stored ProductKnowledgePacks
    pub fn list_knowledge_packs(kb: &KnowledgeBase) -> Result<Vec<String>, String> {
        let assets = kb.asset_list(KB_NAMESPACE)?;
        Ok(assets.into_iter().map(|(id, name, _, _, _, _)| format!("{}/{}", id, name)).collect())
    }

    /// Store all built-in knowledge packs to KB
    pub fn seed_knowledge_packs(kb: &KnowledgeBase) -> Result<Vec<String>, String> {
        let mut stored = Vec::new();
        for pt in [ProductType::Machinery, ProductType::Textile, ProductType::Food, ProductType::Chemical, ProductType::Electronics] {
            let pack = super::get_product_knowledge_pack(pt);
            let id = store_knowledge_pack(kb, pack.as_ref())?;
            stored.push(id);
        }
        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machinery_pack() {
        let pack = MachineryKnowledgePack;
        assert_eq!(pack.product_type(), ProductType::Machinery);
        assert_eq!(pack.hs_code(), "8479.89");
        assert_eq!(pack.tax_refund_rate(), 0.13);
        assert!(!pack.bom_template().is_empty());
        assert!(!pack.routing_template().is_empty());
        assert!(!pack.required_certifications().is_empty());
        assert!(!pack.risk_rules().is_empty());
    }

    #[test]
    fn test_textile_pack() {
        let pack = TextileKnowledgePack;
        assert_eq!(pack.product_type(), ProductType::Textile);
        assert_eq!(pack.hs_code(), "6109.10");
        assert!(!pack.bom_template().is_empty());
        assert!(!pack.inspection_standard().critical_defects.is_empty());
    }

    #[test]
    fn test_food_pack() {
        let pack = FoodKnowledgePack;
        assert_eq!(pack.product_type(), ProductType::Food);
        assert_eq!(pack.hs_code(), "0902.10");
        assert!(!pack.required_certifications().is_empty());
        assert!(pack.packaging_spec().special_handling.contains(&"Cold chain required".into()));
    }

    #[test]
    fn test_factory_function() {
        let machinery = get_product_knowledge_pack(ProductType::Machinery);
        assert_eq!(machinery.product_type(), ProductType::Machinery);

        let textile = get_product_knowledge_pack(ProductType::Textile);
        assert_eq!(textile.product_type(), ProductType::Textile);

        let food = get_product_knowledge_pack(ProductType::Food);
        assert_eq!(food.product_type(), ProductType::Food);

        let generic = get_product_knowledge_pack(ProductType::Other);
        assert_eq!(generic.product_type(), ProductType::Other);
    }

    #[test]
    fn test_compliance_map() {
        let pack = MachineryKnowledgePack;
        let compliance = pack.compliance_map();
        assert!(compliance.target_countries.contains_key("US"));
        assert!(compliance.target_countries.contains_key("EU"));
    }

    #[test]
    fn test_risk_rules() {
        let pack = MachineryKnowledgePack;
        let rules = pack.risk_rules();
        assert!(rules.iter().any(|r| r.rule_id == "MACH-001"));
        assert!(rules.iter().any(|r| r.severity == RiskSeverity::Critical));
    }

    #[test]
    fn test_machinery_bom_details() {
        let pack = MachineryKnowledgePack;
        let bom = pack.bom_template();
        assert!(bom.iter().any(|b| b.category == "Frame" && b.critical));
        assert!(bom.iter().any(|b| b.category == "Motor" && b.lead_time_days == 21));
        assert!(bom.iter().any(|b| b.approved_suppliers.contains(&"Siemens".into())));
    }

    #[test]
    fn test_machinery_routing_steps() {
        let pack = MachineryKnowledgePack;
        let routing = pack.routing_template();
        assert_eq!(routing.len(), 6);
        assert!(routing.iter().any(|r| r.step_id == "010" && r.work_center_type == "Welding"));
        assert!(routing.iter().any(|r| r.step_id == "050" && r.skill_level == SkillLevel::Expert));
        assert!(routing[0].quality_checkpoints.contains(&"Weld visual inspection".into()));
    }

    #[test]
    fn test_textile_inspection_standard() {
        let pack = TextileKnowledgePack;
        let std = pack.inspection_standard();
        assert_eq!(std.aql_level, "AQL 2.5/4.0");
        assert!(std.critical_defects.contains(&"Needle/foreign object".into()));
        assert!(std.test_methods.iter().any(|t| t.parameter == "Colorfastness"));
    }

    #[test]
    fn test_food_packaging_cold_chain() {
        let pack = FoodKnowledgePack;
        let pkg = pack.packaging_spec();
        assert!(pkg.special_handling.contains(&"Cold chain required".into()));
        assert!(pkg.marks_required.contains(&"NUTRITION FACTS".into()));
    }

    #[test]
    fn test_food_risk_rules() {
        let pack = FoodKnowledgePack;
        let rules = pack.risk_rules();
        assert!(rules.iter().any(|r| r.rule_id == "FOOD-001"));
        assert!(rules.iter().any(|r| r.rule_id == "FOOD-002" && r.severity == RiskSeverity::Critical));
        assert!(rules.iter().any(|r| r.category == RiskCategory::SupplyChain));
    }

    #[test]
    fn test_chemical_pack_placeholder() {
        let pack = ChemicalKnowledgePack;
        assert_eq!(pack.product_type(), ProductType::Chemical);
        assert_eq!(pack.hs_code(), "2901.10");
        assert!(pack.required_certifications().contains(&"REACH".into()));
        assert!(pack.packaging_spec().marks_required.contains(&"UN MARK".into()));
    }

    #[test]
    fn test_electronics_pack_placeholder() {
        let pack = ElectronicsKnowledgePack;
        assert_eq!(pack.product_type(), ProductType::Electronics);
        assert_eq!(pack.hs_code(), "8542.31");
        assert!(pack.required_certifications().contains(&"CE".into()));
        assert!(pack.required_certifications().contains(&"RoHS".into()));
        assert!(pack.required_certifications().contains(&"FCC".into()));
        assert!(pack.packaging_spec().marks_required.contains(&"ESD".into()));
    }

    #[test]
    fn test_generic_pack_fallback() {
        let pack = GenericKnowledgePack;
        assert_eq!(pack.product_type(), ProductType::Other);
        assert_eq!(pack.hs_code(), "9999.99");
        assert!(pack.bom_template().is_empty());
        assert!(pack.risk_rules().is_empty());
        assert!(pack.compliance_map().target_countries.is_empty());
    }

    #[test]
    fn test_compliance_map_us_eu() {
        let pack = MachineryKnowledgePack;
        let compliance = pack.compliance_map();
        
        let us = compliance.target_countries.get("US").unwrap();
        assert!(us.required_certifications.contains(&"UL".into()));
        assert!(us.restricted_substances.contains(&"PCB".into()));
        assert_eq!(us.tariff_rate, 0.0);

        let eu = compliance.target_countries.get("EU").unwrap();
        assert!(eu.required_certifications.contains(&"CE".into()));
        assert!(eu.restricted_substances.contains(&"Lead".into()));
        assert_eq!(eu.tariff_rate, 0.0);
    }

    #[test]
    fn test_document_templates() {
        let pack = MachineryKnowledgePack;
        let tmpl = pack.document_templates();
        assert!(!tmpl.commercial_invoice.is_empty());
        assert!(!tmpl.packing_list.is_empty());
        assert!(!tmpl.certificate_of_origin.is_empty());
        assert!(tmpl.additional.contains_key("CE Declaration"));
    }

    #[test]
    fn test_machinery_risk_categories() {
        let pack = MachineryKnowledgePack;
        let rules = pack.risk_rules();
        let categories: std::collections::HashSet<_> = rules.iter().map(|r| r.category).collect();
        assert!(categories.contains(&RiskCategory::Quality));
        assert!(categories.contains(&RiskCategory::Compliance));
        assert!(categories.contains(&RiskCategory::Ip));
        assert!(categories.contains(&RiskCategory::SupplyChain));
    }

    #[test]
    fn test_routing_skill_levels() {
        let pack = MachineryKnowledgePack;
        let routing = pack.routing_template();
        let levels: std::collections::HashSet<_> = routing.iter().map(|r| r.skill_level).collect();
        assert!(levels.contains(&SkillLevel::Skilled));
        assert!(levels.contains(&SkillLevel::Expert));
        assert!(levels.contains(&SkillLevel::SemiSkilled));
    }

    #[test]
    fn test_kb_persistence_serialization() {
        use super::kb_persistence;
        use serde_json;

        let pack = MachineryKnowledgePack;
        let json = serde_json::json!({
            "product_type": format!("{:?}", pack.product_type()),
            "bom_template": pack.bom_template(),
            "routing_template": pack.routing_template(),
            "packaging_spec": pack.packaging_spec(),
            "required_certifications": pack.required_certifications(),
            "hs_code": pack.hs_code(),
            "tax_refund_rate": pack.tax_refund_rate(),
            "inspection_standard": pack.inspection_standard(),
            "document_templates": pack.document_templates(),
            "risk_rules": pack.risk_rules(),
            "compliance_map": pack.compliance_map(),
        });

        let data = serde_json::to_vec(&json).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&data).unwrap();
        
        assert_eq!(parsed["product_type"], "Machinery");
        assert_eq!(parsed["hs_code"], "8479.89");
        assert!(parsed["bom_template"].is_array());
        assert!(parsed["risk_rules"].is_array());
    }
}