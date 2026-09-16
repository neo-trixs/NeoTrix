//! Trade Documents — 外贸单证管理模块
//!
//! 对标 TMS 平台的单证能力：
//! - 标准贸易单证 (CI/PL/BL/CO/Form A/E/F)
//! - 单证生成 (从订单数据自动填充)
//! - 单证状态追踪
//! - 单证合规校验
//! - 一键生成单证套件

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 1. 单证类型
// ============================================================

/// 贸易单证类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TradeDocumentType {
    /// 商业发票 (Commercial Invoice)
    CommercialInvoice,
    /// 装箱单 (Packing List)
    PackingList,
    /// 提单 (Bill of Lading)
    BillOfLading,
    /// 原产地证 (Certificate of Origin)
    CertificateOfOrigin,
    /// Form A (普惠制原产地证)
    FormA,
    /// Form E (中国-东盟自贸区)
    FormE,
    /// Form F (中国-智利自贸区)
    FormF,
    /// 检验检疫证书
    InspectionCertificate,
    /// 保险单
    InsurancePolicy,
    /// 信用证
    LetterOfCredit,
    /// 报关单
    CustomsDeclaration,
    /// 装船通知
    ShippingAdvice,
    /// 受益人证明
    BeneficiaryCertificate,
    /// 自定义
    Custom(String),
}

impl std::fmt::Display for TradeDocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommercialInvoice => write!(f, "商业发票"),
            Self::PackingList => write!(f, "装箱单"),
            Self::BillOfLading => write!(f, "提单"),
            Self::CertificateOfOrigin => write!(f, "原产地证"),
            Self::FormA => write!(f, "Form A"),
            Self::FormE => write!(f, "Form E"),
            Self::FormF => write!(f, "Form F"),
            Self::InspectionCertificate => write!(f, "检验证书"),
            Self::InsurancePolicy => write!(f, "保险单"),
            Self::LetterOfCredit => write!(f, "信用证"),
            Self::CustomsDeclaration => write!(f, "报关单"),
            Self::ShippingAdvice => write!(f, "装船通知"),
            Self::BeneficiaryCertificate => write!(f, "受益人证明"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// 单证状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeDocStatus {
    /// 待生成
    Pending,
    /// 已生成
    Generated,
    /// 已审核
    Reviewed,
    /// 已提交
    Submitted,
    /// 已批准
    Approved,
    /// 已拒绝
    Rejected,
    /// 已归档
    Archived,
}

/// 单证数据字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocField {
    pub name: String,
    pub value: String,
    pub required: bool,
}

/// 贸易单证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeDocument {
    pub id: String,
    pub doc_type: TradeDocumentType,
    pub order_id: String,
    pub status: TradeDocStatus,
    pub fields: Vec<DocField>,
    pub file_path: Option<String>,
    pub version: u32,
    pub review_notes: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

// ============================================================
// 2. 单证模板
// ============================================================

/// 单证模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeDocTemplate {
    pub doc_type: TradeDocumentType,
    pub name: String,
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    /// 字段映射: 订单字段 → 单证字段
    pub field_mapping: HashMap<String, String>,
}

// ============================================================
// 3. 单证套件
// ============================================================

/// 单证套件 — 一笔订单所需的全部单证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSet {
    pub order_id: String,
    pub documents: Vec<TradeDocument>,
    pub completeness: f64, // 0.0 - 1.0
    pub all_approved: bool,
}

// ============================================================
// 4. 单证引擎
// ============================================================

/// 外贸单证管理引擎
pub struct TradeDocumentEngine {
    /// 单证模板库
    templates: HashMap<TradeDocumentType, TradeDocTemplate>,
    /// 单证记录
    documents: Vec<TradeDocument>,
}

impl Default for TradeDocumentEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeDocumentEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            templates: HashMap::new(),
            documents: Vec::new(),
        };
        engine.register_default_templates();
        engine
    }

    /// 注册默认模板
    fn register_default_templates(&mut self) {
        // Commercial Invoice
        self.templates.insert(
            TradeDocumentType::CommercialInvoice,
            TradeDocTemplate {
                doc_type: TradeDocumentType::CommercialInvoice,
                name: "商业发票".into(),
                required_fields: vec![
                    "seller_name".into(),
                    "seller_address".into(),
                    "buyer_name".into(),
                    "buyer_address".into(),
                    "invoice_no".into(),
                    "invoice_date".into(),
                    "product_desc".into(),
                    "quantity".into(),
                    "unit_price".into(),
                    "total_amount".into(),
                    "currency".into(),
                    "trade_terms".into(),
                    "port_of_loading".into(),
                    "port_of_discharge".into(),
                ],
                optional_fields: vec![
                    "lc_number".into(),
                    "hs_code".into(),
                    "country_of_origin".into(),
                ],
                field_mapping: HashMap::new(),
            },
        );
        // Packing List
        self.templates.insert(
            TradeDocumentType::PackingList,
            TradeDocTemplate {
                doc_type: TradeDocumentType::PackingList,
                name: "装箱单".into(),
                required_fields: vec![
                    "seller_name".into(),
                    "buyer_name".into(),
                    "invoice_no".into(),
                    "packing_date".into(),
                    "product_desc".into(),
                    "quantity".into(),
                    "net_weight".into(),
                    "gross_weight".into(),
                    "dimensions".into(),
                ],
                optional_fields: vec!["carton_no".into(), "marks".into()],
                field_mapping: HashMap::new(),
            },
        );
        // Bill of Lading
        self.templates.insert(
            TradeDocumentType::BillOfLading,
            TradeDocTemplate {
                doc_type: TradeDocumentType::BillOfLading,
                name: "提单".into(),
                required_fields: vec![
                    "shipper".into(),
                    "consignee".into(),
                    "notify_party".into(),
                    "vessel_name".into(),
                    "voyage_no".into(),
                    "port_of_loading".into(),
                    "port_of_discharge".into(),
                    "description_of_goods".into(),
                    "gross_weight".into(),
                    "measurement".into(),
                ],
                optional_fields: vec!["container_no".into(), "seal_no".into()],
                field_mapping: HashMap::new(),
            },
        );
        // Certificate of Origin
        self.templates.insert(
            TradeDocumentType::CertificateOfOrigin,
            TradeDocTemplate {
                doc_type: TradeDocumentType::CertificateOfOrigin,
                name: "原产地证".into(),
                required_fields: vec![
                    "exporter".into(),
                    "consignee".into(),
                    "country_of_origin".into(),
                    "product_desc".into(),
                    "hs_code".into(),
                    "quantity".into(),
                ],
                optional_fields: vec!["invoice_no".into(), "remarks".into()],
                field_mapping: HashMap::new(),
            },
        );
    }

    /// 生成单证
    pub fn generate_document(
        &mut self,
        doc_type: TradeDocumentType,
        order_id: &str,
        fields: Vec<DocField>,
    ) -> Result<TradeDocument, String> {
        // 校验必填字段
        if let Some(template) = self.templates.get(&doc_type) {
            for required in &template.required_fields {
                if !fields.iter().any(|f| f.name == *required) {
                    return Err(format!("Missing required field: {}", required));
                }
            }
        }
        let now = self.current_timestamp();
        let doc = TradeDocument {
            id: uuid::Uuid::new_v4().to_string(),
            doc_type,
            order_id: order_id.to_string(),
            status: TradeDocStatus::Generated,
            fields,
            file_path: None,
            version: 1,
            review_notes: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        self.documents.push(doc.clone());
        Ok(doc)
    }

    /// 审核单证
    pub fn review_document(
        &mut self,
        doc_id: &str,
        approved: bool,
        notes: String,
    ) -> Result<(), String> {
        let now = self.current_timestamp();
        let doc = self
            .documents
            .iter_mut()
            .find(|d| d.id == doc_id)
            .ok_or_else(|| format!("Document {} not found", doc_id))?;
        doc.review_notes.push(notes);
        doc.status = if approved {
            TradeDocStatus::Reviewed
        } else {
            TradeDocStatus::Rejected
        };
        doc.updated_at = now;
        Ok(())
    }

    /// 获取订单的单证套件
    pub fn get_document_set(&self, order_id: &str) -> DocumentSet {
        let docs: Vec<TradeDocument> = self
            .documents
            .iter()
            .filter(|d| d.order_id == order_id)
            .cloned()
            .collect();
        let total = docs.len() as f64;
        let approved = docs
            .iter()
            .filter(|d| d.status == TradeDocStatus::Reviewed)
            .count() as f64;
        let completeness = if total > 0.0 {
            approved / total
        } else {
            0.0
        };
        DocumentSet {
            order_id: order_id.to_string(),
            documents: docs,
            completeness,
            all_approved: completeness >= 1.0,
        }
    }

    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

// ============================================================
// 5. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_commercial_invoice() {
        let mut engine = TradeDocumentEngine::new();
        let fields = vec![
            DocField { name: "seller_name".into(), value: "ABC Corp".into(), required: true },
            DocField { name: "seller_address".into(), value: "Shanghai".into(), required: true },
            DocField { name: "buyer_name".into(), value: "XYZ Inc".into(), required: true },
            DocField { name: "buyer_address".into(), value: "New York".into(), required: true },
            DocField { name: "invoice_no".into(), value: "INV-001".into(), required: true },
            DocField { name: "invoice_date".into(), value: "2026-09-15".into(), required: true },
            DocField { name: "product_desc".into(), value: "Widget A".into(), required: true },
            DocField { name: "quantity".into(), value: "1000".into(), required: true },
            DocField { name: "unit_price".into(), value: "10.00".into(), required: true },
            DocField { name: "total_amount".into(), value: "10000.00".into(), required: true },
            DocField { name: "currency".into(), value: "USD".into(), required: true },
            DocField { name: "trade_terms".into(), value: "FOB".into(), required: true },
            DocField { name: "port_of_loading".into(), value: "Shanghai".into(), required: true },
            DocField { name: "port_of_discharge".into(), value: "New York".into(), required: true },
        ];
        let doc = engine.generate_document(
            TradeDocumentType::CommercialInvoice,
            "ORD-001",
            fields,
        );
        assert!(doc.is_ok());
        assert_eq!(doc.unwrap().status, TradeDocStatus::Generated);
    }

    #[test]
    fn test_generate_missing_fields() {
        let mut engine = TradeDocumentEngine::new();
        let fields = vec![
            DocField { name: "seller_name".into(), value: "ABC".into(), required: true },
        ];
        let result = engine.generate_document(
            TradeDocumentType::CommercialInvoice,
            "ORD-001",
            fields,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required field"));
    }

    #[test]
    fn test_document_set() {
        let mut engine = TradeDocumentEngine::new();
        let fields = vec![
            DocField { name: "seller_name".into(), value: "ABC".into(), required: true },
            DocField { name: "seller_address".into(), value: "Shanghai".into(), required: true },
            DocField { name: "buyer_name".into(), value: "XYZ".into(), required: true },
            DocField { name: "buyer_address".into(), value: "NY".into(), required: true },
            DocField { name: "invoice_no".into(), value: "INV-001".into(), required: true },
            DocField { name: "invoice_date".into(), value: "2026-09-15".into(), required: true },
            DocField { name: "product_desc".into(), value: "Widget".into(), required: true },
            DocField { name: "quantity".into(), value: "100".into(), required: true },
            DocField { name: "unit_price".into(), value: "10".into(), required: true },
            DocField { name: "total_amount".into(), value: "1000".into(), required: true },
            DocField { name: "currency".into(), value: "USD".into(), required: true },
            DocField { name: "trade_terms".into(), value: "FOB".into(), required: true },
            DocField { name: "port_of_loading".into(), value: "Shanghai".into(), required: true },
            DocField { name: "port_of_discharge".into(), value: "NY".into(), required: true },
        ];
        let doc = engine.generate_document(
            TradeDocumentType::CommercialInvoice,
            "ORD-001",
            fields,
        ).unwrap();
        engine.review_document(&doc.id, true, "Looks good".into()).unwrap();
        let set = engine.get_document_set("ORD-001");
        assert_eq!(set.documents.len(), 1);
        assert!(set.all_approved);
    }
}
