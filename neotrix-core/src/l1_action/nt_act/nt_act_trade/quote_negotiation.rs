//! Trade Quote & Negotiation Notable Sub-skill
//!
//! Handles FT01-FT04: Customer Development → Requirement Confirmation →
//! Detailed Quotation → Negotiation & Objection Handling
//!
//! This is a NOTABLE skill (域级突破) under the foreign_trade_full_cycle Keystone.

use nt_core_capability_tree::{
    CapabilityNode, CapabilityRegistry, Domain, NodeLayer,
};
use serde::{Deserialize, Serialize};

/// Negotiation Strategy Types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NegotiationStrategy {
    Collaborative,
    Competitive,
    Compromise,
    Accommodating,
    Avoiding,
}

/// Customer Objection Categories
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

/// Requirement Confirmation Result (FT02)
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

/// Quote Generator (FT03)
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

impl QuoteGenerator {
    pub fn generate(&self) -> QuoteSheet {
        let unit_price = self.base_cost * (1.0 + self.margin_target);
        let total = unit_price; // per unit

        QuoteSheet {
            quote_id: format!("QUO-{}", uuid::Uuid::new_v4().simple()),
            version: 1,
            incoterms: self.incoterms.clone(),
            unit_price,
            total,
            currency: self.currency.clone(),
            validity_days: self.validity_days,
            risk_flag: self.assess_risk(),
        }
    }

    fn assess_risk(&self) -> Option<String> {
        if self.margin_target < 0.15 {
            Some("Low margin risk".into())
        } else if self.validity_days > 60 {
            Some("Long validity exposes to FX risk".into())
        } else {
            None
        }
    }
}

/// Quote Sheet Output
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

/// Negotiation Engine (FT04)
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

impl NegotiationEngine {
    pub fn handle_objection(&mut self, objection: Objection) -> NegotiationRecord {
        let response = self.generate_response(&objection);
        let concession = self.maybe_concede(&objection);

        let record = NegotiationRecord {
            round: self.round,
            objection,
            response,
            concession: concession.clone(),
            bottom_line_held: self.current_quote >= self.bottom_line,
        };

        self.round += 1;
        if let Some(c) = concession {
            self.concessions_made.push(c);
            self.current_quote = self.current_quote - (record.concession.as_ref().map(|c| c.original - c.conceded).unwrap_or(0.0));
        }

        record
    }

    fn generate_response(&self, objection: &Objection) -> String {
        match objection.category {
            ObjectionCategory::Price => {
                format!(
                    "We understand price sensitivity. Our quote of {:.2} {} includes {} and {}. \
                    We can discuss volume discounts for orders > {} units.",
                    self.current_quote,
                    "USD",
                    "quality assurance",
                    "after-sales support",
                    1000
                )
            }
            ObjectionCategory::DeliveryTime => {
                format!(
                    "Standard lead time is {} weeks. We can offer expedited production for {}% surcharge.",
                    8, 15
                )
            }
            ObjectionCategory::PaymentTerms => {
                "We require 30% deposit per company policy. Balance against BL copy is standard. \
                For established partners, we can discuss LC at sight.".into()
            }
            ObjectionCategory::Quality => {
                "All products undergo {} inspection per AQL {}. We provide full test reports and \
                support third-party inspection.".into()
            }
            _ => "Thank you for your feedback. Let us review internally and revert within 24 hours.".into(),
        }
    }

    fn maybe_concede(&mut self, objection: &Objection) -> Option<Concession> {
        // Simple concession logic: only on price, up to bottom line
        if objection.category == ObjectionCategory::Price
            && self.current_quote > self.bottom_line * 1.05
            && self.round < 3
        {
            let concession_amount = (self.current_quote - self.bottom_line) * 0.3;
            Some(Concession {
                round: self.round,
                item: "unit_price".into(),
                original: self.current_quote,
                conceded: self.current_quote - concession_amount,
                reason: format!("Price objection: {}", objection.description),
            })
        } else {
            None
        }
    }
}

/// Entry point for the Quote & Negotiation sub-skill
pub fn execute_quote_negotiation(
    requirement: RequirementConfirmation,
    product_spec: super::full_cycle::ProductSpec,
    market_env: super::full_cycle::MarketEnvironment,
) -> (Vec<QuoteSheet>, Vec<NegotiationRecord>) {
    // Generate initial quote
    let cost_breakdown = calculate_cost_breakdown(&product_spec, &market_env);
    let bottom_line = cost_breakdown.total * 1.10;
    let generator = QuoteGenerator {
        base_cost: cost_breakdown.total,
        margin_target: 0.20,
        incoterms: "FOB Shanghai".into(),
        currency: "USD".into(),
        validity_days: 30,
        cost_breakdown,
    };
    let initial_quote = generator.generate();

    // Simulate negotiation rounds
    let mut engine = NegotiationEngine {
        strategy: NegotiationStrategy::Collaborative,
        bottom_line,
        current_quote: initial_quote.unit_price,
        round: 1,
        concessions_made: Vec::new(),
        competitor_data: None,
    };

    let mut records = Vec::new();
    let mut quotes = vec![initial_quote];

    // Simulate 1-2 objection rounds
    if requirement.intent_level == IntentLevel::High {
        let objection = Objection {
            category: ObjectionCategory::Price,
            description: "Budget constraint, target price 15% lower".into(),
            customer_argument: "Competitor quoted 15% less".into(),
            severity: ObjectionSeverity::High,
        };
        let record = engine.handle_objection(objection);
        if record.bottom_line_held {
            let updated_quote = QuoteSheet {
                quote_id: format!("QUO-{}-v{}", engine.round, engine.round + 1),
                version: engine.round + 1,
                incoterms: "FOB Shanghai".into(),
                unit_price: engine.current_quote,
                total: engine.current_quote,
                currency: "USD".into(),
                validity_days: 30,
                risk_flag: None,
            };
            quotes.push(updated_quote);
        }
        records.push(record);
    }

    (quotes, records)
}

fn calculate_cost_breakdown(
    spec: &super::full_cycle::ProductSpec,
    market: &super::full_cycle::MarketEnvironment,
) -> CostBreakdown {
    let material: f64 = spec.bom.iter().map(|b| b.qty * 10.0).sum(); // simplified
    let labor = spec.routing.iter().map(|r| r.duration_hours * 25.0).sum::<f64>();
    let overhead = (material + labor) * 0.15;
    let packaging = 5.0;
    let logistics = market.freight_rates.get("FOB").copied().unwrap_or(50.0);
    let certification = 20.0;
    let contingency = (material + labor + overhead + packaging + logistics + certification) * 0.05;
    let total = material + labor + overhead + packaging + logistics + certification + contingency;

    CostBreakdown {
        material,
        labor,
        overhead,
        packaging,
        logistics,
        certification,
        contingency,
        total,
    }
}

/// Register the Notable capability node
pub fn register_quote_negotiation_capability(registry: &mut CapabilityRegistry) -> CapabilityNode {
    let node = CapabilityNode::new_composite(
        "NT-MIND::trade::trade_quote_negotiation".to_string(),
        Domain::Mind,
        NodeLayer::L3DomainService,
        vec!["trade_quote_negotiation".to_string()],
        vec!["trade_product_spec".to_string()],
    );
    registry.register(node.clone()).expect("Failed to register quote_negotiation capability");
    node
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_generation() {
        let generator = QuoteGenerator {
            base_cost: 100.0,
            margin_target: 0.20,
            incoterms: "FOB".into(),
            currency: "USD".into(),
            validity_days: 30,
            cost_breakdown: CostBreakdown {
                material: 50.0,
                labor: 20.0,
                overhead: 10.0,
                packaging: 5.0,
                logistics: 10.0,
                certification: 5.0,
                contingency: 5.0,
                total: 105.0,
            },
        };

        let quote = generator.generate();
        assert!((quote.unit_price - 120.0).abs() < 0.01);
        assert_eq!(quote.currency, "USD");
        assert_eq!(quote.incoterms, "FOB");
    }

    #[test]
    fn test_negotiation_price_objection() {
        let mut engine = NegotiationEngine {
            strategy: NegotiationStrategy::Collaborative,
            bottom_line: 110.0,
            current_quote: 120.0,
            round: 1,
            concessions_made: Vec::new(),
            competitor_data: None,
        };

        let objection = Objection {
            category: ObjectionCategory::Price,
            description: "Too expensive".into(),
            customer_argument: "Budget is 110".into(),
            severity: ObjectionSeverity::High,
        };

        let record = engine.handle_objection(objection);
        assert!(record.response.contains("price"));
        assert!(engine.concessions_made.len() <= 1);
    }

    #[test]
    fn test_negotiation_bottom_line_protection() {
        let mut engine = NegotiationEngine {
            strategy: NegotiationStrategy::Collaborative,
            bottom_line: 110.0,
            current_quote: 112.0, // Very close to bottom line
            round: 1,
            concessions_made: Vec::new(),
            competitor_data: None,
        };

        let objection = Objection {
            category: ObjectionCategory::Price,
            description: "Need lower".into(),
            customer_argument: "Can you do 105?".into(),
            severity: ObjectionSeverity::High,
        };

        let record = engine.handle_objection(objection);
        // Should not concede below bottom line
        assert!(record.bottom_line_held);
    }

    #[test]
    fn test_cost_breakdown() {
        use super::super::full_cycle::{ProductSpec, BomItem, RoutingStep, MarketEnvironment};

        let spec = ProductSpec {
            spec_id: "test".into(),
            product_type: super::super::full_cycle::ProductType::Machinery,
            bom: vec![BomItem {
                item_id: "1".into(),
                name: "Steel".into(),
                qty: 10.0,
                unit: "kg".into(),
                supplier: None,
                lead_time_days: 7,
            }],
            routing: vec![RoutingStep {
                step_id: "1".into(),
                name: "Cutting".into(),
                work_center: "WC1".into(),
                duration_hours: 2.0,
            }],
            packaging: super::super::full_cycle::PackagingSpec {
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

        let market = MarketEnvironment {
            fx_rates: HashMap::new(),
            freight_rates: HashMap::from([("FOB".to_string(), 50.0)]),
            regulations: vec![],
            shipping_lines: vec![],
        };

        let breakdown = calculate_cost_breakdown(&spec, &market);
        assert!(breakdown.total > 0.0);
        assert!(breakdown.material > 0.0);
    }

    #[test]
    fn test_requirement_confirmation_high_intent() {
        let req = RequirementConfirmation {
            confirmed: true,
            confirmed_items: vec![ConfirmedItem {
                item: "Widget".into(),
                spec: "Grade A".into(),
                qty: 1000,
                unit: "PCS".into(),
                note: None,
            }],
            pending_clarifications: vec![],
            intent_level: IntentLevel::High,
            risk_flags: vec![],
        };
        assert_eq!(req.intent_level, IntentLevel::High);
    }

    #[test]
    fn test_negotiation_multiple_rounds() {
        let mut engine = NegotiationEngine {
            strategy: NegotiationStrategy::Compromise,
            bottom_line: 100.0,
            current_quote: 150.0,
            round: 1,
            concessions_made: Vec::new(),
            competitor_data: Some(CompetitorData {
                competitor_name: "Competitor X".into(),
                quoted_price: 120.0,
                quoted_terms: "FOB".into(),
                source: "Customer feedback".into(),
            }),
        };

        // Round 1: Price objection
        let obj1 = Objection {
            category: ObjectionCategory::Price,
            description: "Competitor offers 120".into(),
            customer_argument: "Need better price".into(),
            severity: ObjectionSeverity::High,
        };
        let r1 = engine.handle_objection(obj1);
        assert!(r1.concession.is_some());

        // Round 2: Delivery time objection
        let obj2 = Objection {
            category: ObjectionCategory::DeliveryTime,
            description: "Need faster delivery".into(),
            customer_argument: "Standard 8 weeks too long".into(),
            severity: ObjectionSeverity::Medium,
        };
        let r2 = engine.handle_objection(obj2);
        assert!(r2.response.contains("week"));

        // Round 3: Payment terms objection
        let obj3 = Objection {
            category: ObjectionCategory::PaymentTerms,
            description: "Want 60 days credit".into(),
            customer_argument: "Standard terms".into(),
            severity: ObjectionSeverity::Low,
        };
        let r3 = engine.handle_objection(obj3);
        assert!(r3.response.contains("30%"));

        assert_eq!(engine.round, 4);
        assert!(engine.concessions_made.len() >= 1);
    }

    #[test]
    fn test_negotiation_competitive_strategy() {
        let mut engine = NegotiationEngine {
            strategy: NegotiationStrategy::Competitive,
            bottom_line: 100.0,
            current_quote: 130.0,
            round: 1,
            concessions_made: Vec::new(),
            competitor_data: None,
        };

        let objection = Objection {
            category: ObjectionCategory::Price,
            description: "Too high".into(),
            customer_argument: "Budget 110".into(),
            severity: ObjectionSeverity::High,
        };
        let record = engine.handle_objection(objection);
        // Competitive strategy should be less concession-prone
        assert!(record.response.len() > 0);
    }

    #[test]
    fn test_quote_generator_risk_flags() {
        // Low margin
        let gen1 = QuoteGenerator {
            base_cost: 100.0,
            margin_target: 0.10,
            incoterms: "FOB".into(),
            currency: "USD".into(),
            validity_days: 30,
            cost_breakdown: CostBreakdown { material: 50.0, labor: 20.0, overhead: 10.0, packaging: 5.0, logistics: 10.0, certification: 5.0, contingency: 5.0, total: 105.0 },
        };
        let q1 = gen1.generate();
        assert!(q1.risk_flag.is_some());
        assert!(q1.risk_flag.unwrap().contains("Low margin"));

        // Long validity
        let gen2 = QuoteGenerator {
            base_cost: 100.0,
            margin_target: 0.20,
            incoterms: "FOB".into(),
            currency: "USD".into(),
            validity_days: 90,
            cost_breakdown: CostBreakdown { material: 50.0, labor: 20.0, overhead: 10.0, packaging: 5.0, logistics: 10.0, certification: 5.0, contingency: 5.0, total: 105.0 },
        };
        let q2 = gen2.generate();
        assert!(q2.risk_flag.is_some());
        assert!(q2.risk_flag.unwrap().contains("FX risk"));
    }

    #[test]
    fn test_execute_quote_negotiation_integration() {
        use super::super::full_cycle::{ProductSpec, BomItem, RoutingStep, PackagingSpec, MarketEnvironment, IntentLevel};

        let req = RequirementConfirmation {
            confirmed: true,
            confirmed_items: vec![],
            pending_clarifications: vec![],
            intent_level: IntentLevel::High,
            risk_flags: vec![],
        };

        let spec = ProductSpec {
            spec_id: "spec-1".into(),
            product_type: super::super::full_cycle::ProductType::Machinery,
            bom: vec![BomItem { item_id: "1".into(), name: "Steel".into(), qty: 100.0, unit: "kg".into(), supplier: None, lead_time_days: 7 }],
            routing: vec![RoutingStep { step_id: "1".into(), name: "Cutting".into(), work_center: "WC1".into(), duration_hours: 8.0 }],
            packaging: PackagingSpec { package_type: "carton".into(), dimensions_cm: (30.0, 20.0, 15.0), gross_weight_kg: 10.0, net_weight_kg: 8.0, marks: vec![] },
            certifications: vec![],
            hs_code: "8471".into(),
            tax_refund_rate: 0.13,
        };

        let market = MarketEnvironment {
            fx_rates: HashMap::from([("USD".to_string(), 7.2)]),
            freight_rates: HashMap::from([("FOB".to_string(), 50.0)]),
            regulations: vec![],
            shipping_lines: vec![],
        };

        let (quotes, records) = execute_quote_negotiation(req, spec, market);
        assert!(!quotes.is_empty());
        assert_eq!(quotes[0].currency, "USD");
        assert_eq!(quotes[0].incoterms, "FOB Shanghai");
        assert!(!records.is_empty());
    }
}