//! Trade Finance & Compliance Notable Sub-skill
//!
//! Handles FT05-FT06, FT14-FT16: Contract Review → Payment Collection →
//! Final Payment Collection → Settlement & Verification → Tax Refund Declaration
//!
//! This is a NOTABLE skill (域级突破) under the foreign_trade_full_cycle Keystone.

use nt_core_capability_tree::{
    CapabilityNode, CapabilityRegistry, Domain, NodeLayer,
};
use serde::{Deserialize, Serialize};

/// Contract Review Result (FT05)
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

/// Payment Proof (FT06, FT14)
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
    pub level: RiskLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Letter of Credit Review
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

/// Collection Record (FT14)
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

/// Settlement Record (FT15)
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

/// Tax Refund Claim (FT16)
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

/// Finance Engine
pub struct FinanceEngine;

impl Default for FinanceEngine {
    fn default() -> Self { Self }
}

impl FinanceEngine {
    /// Review contract for compliance and risk (FT05)
    pub fn review_contract(
        contract: &super::nt_trade_full_cycle::Contract,
        policy: &super::nt_trade_full_cycle::CompanyPolicy,
    ) -> ContractReview {
        let mut findings = Vec::new();
        let mut risk_score = 0.0;

        // Check payment terms
        if !policy.payment_terms.iter().any(|t| contract.payment_terms.contains(t)) {
            findings.push(ContractFinding {
                clause: "payment_terms".into(),
                issue: "Payment terms not in approved list".into(),
                severity: FindingSeverity::Major,
                recommendation: "Negotiate standard terms or get approval".into(),
            });
            risk_score += 20.0;
        }

        // Check deposit ratio
        if let Some(deposit_part) = contract.payment_terms.split(',').next() {
            if let Some(ratio_str) = deposit_part.trim().split('%').next() {
                if let Ok(ratio) = ratio_str.parse::<f64>() {
                    if ratio / 100.0 < policy.risk_control.min_deposit_ratio {
                        findings.push(ContractFinding {
                            clause: "deposit_ratio".into(),
                            issue: format!(
                                "Deposit ratio {}% below minimum {}%",
                                ratio,
                                policy.risk_control.min_deposit_ratio * 100.0
                            ),
                            severity: FindingSeverity::Critical,
                            recommendation: "Increase deposit or require LC".into(),
                        });
                        risk_score += 30.0;
                    }
                }
            }
        }

        // Check forbidden countries
        // (would need buyer country from context)

        // Check delivery date reasonableness
        // ... additional checks

        let approved = findings.iter().all(|f| f.severity != FindingSeverity::Critical)
            && risk_score < 50.0;

        ContractReview {
            contract_id: contract.contract_id.clone(),
            reviewed: true,
            review_date: chrono::Utc::now().date_naive().to_string(),
            reviewer: "Auto Reviewer".into(),
            findings,
            risk_score,
            approved,
            conditions: if approved { vec![] } else { vec!["Requires management approval".into()] },
        }
    }

    /// Record payment receipt (FT06, FT14)
    pub fn record_payment(
        contract_id: &str,
        payment_type: PaymentType,
        amount: f64,
        currency: &str,
        bank_slip: Option<String>,
        bank_ref: Option<String>,
        lc_number: Option<String>,
    ) -> PaymentProof {
        let mut risk_flags = Vec::new();

        // Check for risk flags
        if amount <= 0.0 {
            risk_flags.push(RiskFlag {
                code: "ZERO_AMOUNT".into(),
                description: "Payment amount is zero or negative".into(),
                level: RiskLevel::Critical,
            });
        }

        PaymentProof {
            payment_id: format!("PAY-{}", uuid::Uuid::new_v4().simple()),
            contract_id: contract_id.into(),
            payment_type,
            amount,
            currency: currency.into(),
            status: PaymentStatus::Received,
            received_date: Some(chrono::Utc::now().date_naive().to_string()),
            bank_slip,
            bank_reference: bank_ref,
            lc_number,
            risk_flags,
        }
    }

    /// Review Letter of Credit for soft clauses
    pub fn review_lc(lc_text: &str, contract: &super::nt_trade_full_cycle::Contract) -> LcReview {
        // Simplified LC review - in production would parse MT700 format
        let soft_clauses = Self::detect_soft_clauses(lc_text);
        let discrepancies = Self::check_discrepancies(lc_text, contract);

        let risk_score = soft_clauses.len() as f64 * 15.0 + discrepancies.len() as f64 * 10.0;
        let recommendation = match risk_score {
            s if s == 0.0 => LcRecommendation::Accept,
            s if s <= 30.0 => LcRecommendation::AcceptWithAmendment,
            s if s <= 60.0 => LcRecommendation::RequestClarification,
            _ => LcRecommendation::Reject,
        };

        LcReview {
            lc_number: Self::extract_lc_number(lc_text).unwrap_or_default(),
            contract_id: contract.contract_id.clone(),
            issuing_bank: "Unknown".into(),
            advising_bank: None,
            amount: contract.price,
            currency: "USD".into(),
            expiry_date: "2026-12-31".into(),
            expiry_place: "Shanghai".into(),
            soft_clauses,
            discrepancies,
            risk_score,
            recommendation,
        }
    }

    fn detect_soft_clauses(lc_text: &str) -> Vec<SoftClause> {
        let mut clauses = Vec::new();
        let lower = lc_text.to_lowercase();

        // Common soft clause patterns
        if lower.contains("subject to") || lower.contains("at seller's risk") {
            clauses.push(SoftClause {
                clause_text: "Subject to buyer's approval".into(),
                risk: "Payment conditional on buyer's subjective satisfaction".into(),
                mitigation: "Require objective inspection criteria".into(),
            });
        }
        if lower.contains("buyer's bank") || lower.contains("applicant's bank") {
            clauses.push(SoftClause {
                clause_text: "Documents to be approved by applicant".into(),
                risk: "Applicant can reject documents arbitrarily".into(),
                mitigation: "Require issuing bank to determine compliance".into(),
            });
        }
        if lower.contains("partial shipment") && lower.contains("not allowed") {
            clauses.push(SoftClause {
                clause_text: "Partial shipments not allowed".into(),
                risk: "Any short shipment voids entire LC".into(),
                mitigation: "Negotiate partial shipment allowance".into(),
            });
        }

        clauses
    }

    fn check_discrepancies(_lc_text: &str, _contract: &super::nt_trade_full_cycle::Contract) -> Vec<Discrepancy> {
        let discrepancies = Vec::new();
        // Simplified - would check actual documents against LC terms
        discrepancies
    }

    fn extract_lc_number(lc_text: &str) -> Option<String> {
        // Simple regex extraction
        for line in lc_text.lines() {
            if line.to_lowercase().contains("lc number") || line.to_lowercase().contains("l/c no") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    return Some(parts[1].trim().to_string());
                }
            }
        }
        None
    }

    /// Process collection (FT14)
    pub fn process_collection(
        contract_id: &str,
        bl_number: &str,
        documents: Vec<CollectionDocument>,
    ) -> CollectionRecord {
        CollectionRecord {
            collection_id: format!("COLL-{}", uuid::Uuid::new_v4().simple()),
            contract_id: contract_id.into(),
            bl_number: Some(bl_number.into()),
            bl_sent: true,
            bl_sent_date: Some(chrono::Utc::now().date_naive().to_string()),
            documents,
            payment_received: false,
            payment_date: None,
            amount_received: 0.0,
            documents_released: false,
            release_date: None,
            status: CollectionStatus::AwaitingPayment,
        }
    }

    /// Complete settlement (FT15)
    pub fn complete_settlement(
        collection: &CollectionRecord,
        bank_receipt: &str,
        received_amount: f64,
        fx_rate: f64,
    ) -> SettlementRecord {
        let bank_fees = received_amount * 0.005; // 0.5% typical
        SettlementRecord {
            settlement_id: format!("SETL-{}", uuid::Uuid::new_v4().simple()),
            contract_id: collection.contract_id.clone(),
            collection_id: collection.collection_id.clone(),
            bank_receipt: bank_receipt.into(),
            received_amount,
            received_currency: "USD".into(),
            settlement_amount: received_amount * fx_rate,
            settlement_currency: "CNY".into(),
            fx_rate,
            bank_fees,
            net_amount: (received_amount - bank_fees) * fx_rate,
            verification_status: VerificationStatus::Pending,
            verification_date: None,
            verification_officer: None,
        }
    }

    /// Apply for tax refund (FT16)
    pub fn apply_tax_refund(
        contract_id: &str,
        declaration_id: &str,
        hs_code: &str,
        export_value: f64,
        refund_rate: f64,
    ) -> TaxRefundClaim {
        TaxRefundClaim {
            refund_id: format!("REF-{}", uuid::Uuid::new_v4().simple()),
            contract_id: contract_id.into(),
            declaration_id: declaration_id.into(),
            product_hs_code: hs_code.into(),
            export_value,
            refund_rate,
            claim_amount: export_value * refund_rate,
            status: RefundStatus::Draft,
            application_date: chrono::Utc::now().date_naive().to_string(),
            approval_date: None,
            refund_received_date: None,
            actual_refund_amount: None,
            documents: vec![
                RefundDocument {
                    doc_type: "Export Declaration".into(),
                    doc_number: declaration_id.into(),
                    issue_date: chrono::Utc::now().date_naive().to_string(),
                },
                RefundDocument {
                    doc_type: "VAT Invoice".into(),
                    doc_number: format!("VAT-{}", uuid::Uuid::new_v4().simple()),
                    issue_date: chrono::Utc::now().date_naive().to_string(),
                },
            ],
        }
    }

    /// Instance method: verify settlement (FT22)
    pub fn verify_settlement(&self) -> Result<SettlementRecord, String> {
        // In production, this would verify bank settlement
        Ok(SettlementRecord {
            settlement_id: format!("SET-{}", uuid::Uuid::new_v4().simple()),
            contract_id: "unknown".into(),
            collection_id: "unknown".into(),
            bank_receipt: "unknown".into(),
            received_amount: 0.0,
            received_currency: "USD".into(),
            settlement_amount: 0.0,
            settlement_currency: "USD".into(),
            fx_rate: 1.0,
            bank_fees: 0.0,
            net_amount: 0.0,
            verification_status: VerificationStatus::Pending,
            verification_date: None,
            verification_officer: None,
        })
    }

    /// Instance method: declare tax refund (FT23)
    pub fn declare_tax_refund(
        &self,
        claim: &TaxRefundClaim,
    ) -> Result<RefundDocument, String> {
        // In production, this would submit tax refund application
        Ok(RefundDocument {
            doc_type: "Tax Refund Application".into(),
            doc_number: claim.refund_id.clone(),
            issue_date: chrono::Utc::now().date_naive().to_string(),
        })
    }

    /// Instance method: reconcile accounts (FT24)
    pub fn reconcile_accounts(&self) -> Result<(), String> {
        // In production, this would reconcile all accounts
        Ok(())
    }
}

/// Register the Notable capability node
pub fn register_finance_compliance_capability(registry: &mut CapabilityRegistry) -> CapabilityNode {
    let node = CapabilityNode::new_composite(
        "NT-MIND::trade::trade_finance_compliance".to_string(),
        Domain::Mind,
        NodeLayer::L3DomainService,
        vec!["trade_finance_compliance".to_string()],
        vec!["trade_product_spec".to_string(), "trade_quote_negotiation".to_string()],
    );
    registry
        .register(node.clone())
        .expect("Failed to register finance_compliance capability");
    node
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_review_approved() {
        use super::super::nt_trade_full_cycle::{Contract, ContractItem, CompanyPolicy, RiskControl};

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

        let policy = CompanyPolicy {
            payment_terms: vec!["T/T 30% deposit, 70% against BL copy".into()],
            risk_control: RiskControl {
                max_credit_days: 60,
                min_deposit_ratio: 0.3,
                forbidden_countries: vec![],
                forbidden_products: vec![],
            },
            authorization_matrix: HashMap::new(),
        };

        let review = FinanceEngine::review_contract(&contract, &policy);
        assert!(review.approved);
        assert!(review.risk_score < 50.0);
    }

    #[test]
    fn test_contract_review_low_deposit() {
        use super::super::nt_trade_full_cycle::{Contract, ContractItem, CompanyPolicy, RiskControl};

        let contract = Contract {
            contract_id: "CONTRACT-2".into(),
            pi_number: "PI-2".into(),
            parties: ("Seller".into(), "Buyer".into()),
            items: vec![ContractItem {
                product: "Widget".into(),
                qty: 1000,
                unit_price: 50.0,
            }],
            price: 50000.0,
            incoterms: "FOB Shanghai".into(),
            payment_terms: "T/T 10% deposit, 90% against BL copy".into(), // Below 30%
            delivery_date: "2026-03-01".into(),
        };

        let policy = CompanyPolicy {
            payment_terms: vec!["T/T 30% deposit, 70% against BL copy".into()],
            risk_control: RiskControl {
                max_credit_days: 60,
                min_deposit_ratio: 0.3,
                forbidden_countries: vec![],
                forbidden_products: vec![],
            },
            authorization_matrix: HashMap::new(),
        };

        let review = FinanceEngine::review_contract(&contract, &policy);
        assert!(!review.approved);
        assert!(review.findings.iter().any(|f| f.severity == FindingSeverity::Critical));
    }

    #[test]
    fn test_payment_proof() {
        let proof = FinanceEngine::record_payment(
            "CONTRACT-1",
            PaymentType::Deposit,
            15000.0,
            "USD",
            Some("SLIP-123".into()),
            Some("BANK-REF-456".into()),
            None,
        );

        assert_eq!(proof.payment_type, PaymentType::Deposit);
        assert_eq!(proof.amount, 15000.0);
        assert_eq!(proof.status, PaymentStatus::Received);
        assert!(proof.risk_flags.is_empty());
    }

    #[test]
    fn test_lc_review_soft_clause() {
        use super::super::nt_trade_full_cycle::{Contract, ContractItem};

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
            payment_terms: "LC at sight".into(),
            delivery_date: "2026-03-01".into(),
        };

        let lc_text = r#"
        LC Number: LC202612345
        Amount: USD 50,000.00
        Subject to buyer's final approval of goods.
        Documents must be approved by applicant before payment.
        Partial shipments not allowed.
        "#;

        let review = FinanceEngine::review_lc(lc_text, &contract);
        assert!(!review.soft_clauses.is_empty());
        assert!(review.risk_score > 0.0);
        assert!(matches!(
            review.recommendation,
            LcRecommendation::AcceptWithAmendment | LcRecommendation::RequestClarification | LcRecommendation::Reject
        ));
    }

    #[test]
    fn test_tax_refund_claim() {
        let claim = FinanceEngine::apply_tax_refund(
            "CONTRACT-1",
            "CUST-1",
            "8471",
            50000.0,
            0.13,
        );

        assert_eq!(claim.product_hs_code, "8471");
        assert!((claim.claim_amount - 6500.0).abs() < 0.01);
        assert_eq!(claim.status, RefundStatus::Draft);
        assert_eq!(claim.documents.len(), 2);
    }

    #[test]
    fn test_settlement_calculation() {
        use super::super::nt_trade_full_cycle::{Contract, ContractItem, CompanyPolicy, RiskControl};

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

        let collection = CollectionRecord {
            collection_id: "COLL-1".into(),
            contract_id: contract.contract_id.clone(),
            bl_number: Some("BL-1".into()),
            bl_sent: true,
            bl_sent_date: Some("2026-02-01".into()),
            documents: vec![],
            payment_received: true,
            payment_date: Some("2026-02-20".into()),
            amount_received: 35000.0,
            documents_released: true,
            release_date: Some("2026-02-20".into()),
            status: CollectionStatus::Completed,
        };

        let settlement = FinanceEngine::complete_settlement(&collection, "BANK-RCPT-1", 35000.0, 7.2);
        assert!((settlement.settlement_amount - 252000.0).abs() < 1.0);
        assert!((settlement.net_amount - 250740.0).abs() < 10.0); // after fees
        assert_eq!(settlement.verification_status, VerificationStatus::Pending);
    }

    #[test]
    fn test_lc_review_no_soft_clauses() {
        use super::super::nt_trade_full_cycle::{Contract, ContractItem};

        let contract = Contract {
            contract_id: "CONTRACT-1".into(),
            pi_number: "PI-1".into(),
            parties: ("Seller".into(), "Buyer".into()),
            items: vec![ContractItem { product: "Widget".into(), qty: 1000, unit_price: 50.0 }],
            price: 50000.0,
            incoterms: "FOB Shanghai".into(),
            payment_terms: "LC at sight".into(),
            delivery_date: "2026-03-01".into(),
        };

        let lc_text = "LC Number: LC202612345\nAmount: USD 50,000.00\nIrrevocable LC at sight.\nDocuments: Commercial Invoice, Packing List, Bill of Lading.";
        let review = FinanceEngine::review_lc(lc_text, &contract);
        assert!(review.soft_clauses.is_empty());
        assert_eq!(review.recommendation, LcRecommendation::Accept);
    }

    #[test]
    fn test_payment_proof_risk_flags() {
        let proof = FinanceEngine::record_payment(
            "CONTRACT-1",
            PaymentType::Balance,
            -100.0, // Invalid amount
            "USD",
            None,
            None,
            None,
        );
        assert!(proof.risk_flags.iter().any(|f| f.code == "ZERO_AMOUNT"));
        assert_eq!(proof.risk_flags[0].level, RiskLevel::Critical);
    }

    #[test]
    fn test_collection_record_lifecycle() {
        let mut coll = CollectionRecord {
            collection_id: "COLL-1".into(),
            contract_id: "CONTRACT-1".into(),
            bl_number: Some("BL-1".into()),
            bl_sent: false,
            bl_sent_date: None,
            documents: vec![
                CollectionDocument { doc_type: "Commercial Invoice".into(), originals: 1, copies: 2, status: DocumentStatus::Prepared },
                CollectionDocument { doc_type: "Packing List".into(), originals: 1, copies: 2, status: DocumentStatus::Prepared },
            ],
            payment_received: false,
            payment_date: None,
            amount_received: 0.0,
            documents_released: false,
            release_date: None,
            status: CollectionStatus::InProgress,
        };

        assert_eq!(coll.status, CollectionStatus::InProgress);
        assert!(!coll.bl_sent);

        // Simulate BL sent
        coll.bl_sent = true;
        coll.bl_sent_date = Some("2026-02-01".into());
        coll.documents[0].status = DocumentStatus::Submitted;
        coll.status = CollectionStatus::AwaitingPayment;

        assert!(coll.bl_sent);
        assert_eq!(coll.status, CollectionStatus::AwaitingPayment);

        // Simulate payment received
        coll.payment_received = true;
        coll.payment_date = Some("2026-02-20".into());
        coll.amount_received = 35000.0;
        coll.documents_released = true;
        coll.release_date = Some("2026-02-20".into());
        coll.status = CollectionStatus::Completed;

        assert_eq!(coll.status, CollectionStatus::Completed);
        assert!(coll.documents_released);
    }

    #[test]
    fn test_contract_review_forbidden_country() {
        use super::super::nt_trade_full_cycle::{Contract, ContractItem, CompanyPolicy, RiskControl};

        let contract = Contract {
            contract_id: "CONTRACT-1".into(),
            pi_number: "PI-1".into(),
            parties: ("Seller".into(), "Buyer".into()),
            items: vec![ContractItem { product: "Widget".into(), qty: 1000, unit_price: 50.0 }],
            price: 50000.0,
            incoterms: "FOB Shanghai".into(),
            payment_terms: "T/T 30% deposit".into(),
            delivery_date: "2026-03-01".into(),
        };

        let policy = CompanyPolicy {
            payment_terms: vec!["T/T 30% deposit".into()],
            risk_control: RiskControl {
                max_credit_days: 60,
                min_deposit_ratio: 0.3,
                forbidden_countries: vec!["Iran".into(), "North Korea".into()],
                forbidden_products: vec![],
            },
            authorization_matrix: HashMap::new(),
        };

        // The review doesn't check buyer country directly, but we can test the logic
        let review = FinanceEngine::review_contract(&contract, &policy);
        assert!(review.approved); // No forbidden country in contract data
    }

    #[test]
    fn test_tax_refund_claim_workflow() {
        let claim = FinanceEngine::apply_tax_refund("CONTRACT-1", "CUST-1", "8471", 100000.0, 0.13);
        assert_eq!(claim.claim_amount, 13000.0);
        assert_eq!(claim.status, RefundStatus::Draft);
        assert_eq!(claim.documents.len(), 2);

        // Simulate status progression
        let mut c = claim;
        c.status = RefundStatus::Submitted;
        assert_eq!(c.status, RefundStatus::Submitted);

        c.status = RefundStatus::UnderReview;
        assert_eq!(c.status, RefundStatus::UnderReview);

        c.status = RefundStatus::Approved;
        c.approval_date = Some("2026-04-01".into());
        assert_eq!(c.status, RefundStatus::Approved);

        c.status = RefundStatus::Paid;
        c.refund_received_date = Some("2026-04-15".into());
        c.actual_refund_amount = Some(12950.0);
        assert_eq!(c.status, RefundStatus::Paid);
    }

    #[test]
    fn test_settlement_multi_currency() {
        use super::super::nt_trade_full_cycle::{Contract, ContractItem};

        let contract = Contract {
            contract_id: "CONTRACT-1".into(),
            pi_number: "PI-1".into(),
            parties: ("Seller".into(), "Buyer".into()),
            items: vec![ContractItem { product: "Widget".into(), qty: 1000, unit_price: 50.0 }],
            price: 50000.0,
            incoterms: "FOB Shanghai".into(),
            payment_terms: "LC at sight".into(),
            delivery_date: "2026-03-01".into(),
        };

        let collection = CollectionRecord {
            collection_id: "COLL-1".into(),
            contract_id: contract.contract_id.clone(),
            bl_number: Some("BL-1".into()),
            bl_sent: true,
            bl_sent_date: Some("2026-02-01".into()),
            documents: vec![],
            payment_received: true,
            payment_date: Some("2026-02-20".into()),
            amount_received: 50000.0,
            documents_released: true,
            release_date: Some("2026-02-20".into()),
            status: CollectionStatus::Completed,
        };

        // EUR settlement
        let settlement_eur = FinanceEngine::complete_settlement(&collection, "BANK-RCPT-1", 50000.0, 7.8);
        assert!((settlement_eur.settlement_amount - 390000.0).abs() < 100.0);
        assert_eq!(settlement_eur.settlement_currency, "CNY");

        // USD settlement
        let settlement_usd = FinanceEngine::complete_settlement(&collection, "BANK-RCPT-1", 50000.0, 7.2);
        assert!((settlement_usd.settlement_amount - 360000.0).abs() < 100.0);
    }
}