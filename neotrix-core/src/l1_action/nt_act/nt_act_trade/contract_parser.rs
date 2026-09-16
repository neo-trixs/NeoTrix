//! Contract Parser — 合同 Excel 解析器
//!
//! 从合同 Excel 文件中提取结构化数据：
//! - 自动识别合同模板类型
//! - 提取买卖双方信息、产品明细、价格条款
//! - 支持多种合同格式 (采购合同/销售确认书/形式发票)

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 合同类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractType {
    /// 采购合同
    PurchaseContract,
    /// 销售确认书
    SalesConfirmation,
    /// 形式发票 (Proforma Invoice)
    ProformaInvoice,
    /// 商业发票
    CommercialInvoice,
    /// 其他
    Other(String),
}

/// 合同解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedContract {
    pub contract_type: ContractType,
    pub contract_number: Option<String>,
    pub date: Option<String>,
    pub seller: Option<String>,
    pub buyer: Option<String>,
    pub items: Vec<ParsedContractItem>,
    pub total_amount: Option<f64>,
    pub currency: Option<String>,
    pub trade_terms: Option<String>,
    pub payment_terms: Option<String>,
    pub delivery_date: Option<String>,
    pub raw_data: HashMap<String, String>,
}

/// 合同明细项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedContractItem {
    pub seq: u32,
    pub description: String,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
    pub unit_price: Option<f64>,
    pub amount: Option<f64>,
    pub specs: Option<String>,
}

/// 合同 Excel 解析器
pub struct ContractParser;

impl ContractParser {
    /// 从 Excel 行数据解析合同
    pub fn parse_from_rows(rows: &[Vec<String>]) -> ParsedContract {
        let mut contract = ParsedContract {
            contract_type: ContractType::Other("Unknown".into()),
            contract_number: None,
            date: None,
            seller: None,
            buyer: None,
            items: Vec::new(),
            total_amount: None,
            currency: None,
            trade_terms: None,
            payment_terms: None,
            delivery_date: None,
            raw_data: HashMap::new(),
        };

        for row in rows {
            if row.len() < 2 {
                continue;
            }
            let key = row[0].trim().to_lowercase();
            let value = row[1..].join(" ").trim().to_string();

            match key.as_str() {
                k if k.contains("contract") || k.contains("合同") => {
                    contract.contract_number = Some(value.clone());
                    contract.raw_data.insert("contract_number".into(), value);
                }
                k if k.contains("date") || k.contains("日期") => {
                    contract.date = Some(value.clone());
                    contract.raw_data.insert("date".into(), value);
                }
                k if k.contains("seller") || k.contains("卖方") || k.contains("供应商") => {
                    contract.seller = Some(value.clone());
                    contract.raw_data.insert("seller".into(), value);
                }
                k if k.contains("buyer") || k.contains("买方") || k.contains("客户") => {
                    contract.buyer = Some(value.clone());
                    contract.raw_data.insert("buyer".into(), value);
                }
                k if k.contains("total") || k.contains("合计") || k.contains("总额") => {
                    if let Ok(amount) = value.replace(',', "").parse::<f64>() {
                        contract.total_amount = Some(amount);
                    }
                    contract.raw_data.insert("total".into(), value);
                }
                k if k.contains("currency") || k.contains("币种") => {
                    contract.currency = Some(value.clone());
                    contract.raw_data.insert("currency".into(), value);
                }
                k if k.contains("terms") || k.contains("条款") || k.contains("价格条件") => {
                    contract.trade_terms = Some(value.clone());
                    contract.raw_data.insert("trade_terms".into(), value);
                }
                k if k.contains("payment") || k.contains("付款") => {
                    contract.payment_terms = Some(value.clone());
                    contract.raw_data.insert("payment_terms".into(), value);
                }
                k if k.contains("delivery") || k.contains("交货") => {
                    contract.delivery_date = Some(value.clone());
                    contract.raw_data.insert("delivery_date".into(), value);
                }
                _ => {}
            }
        }

        // 尝试从合同号判断类型
        if let Some(ref num) = contract.contract_number {
            let num_lower = num.to_lowercase();
            if num_lower.contains("pc") || num_lower.contains("采购") {
                contract.contract_type = ContractType::PurchaseContract;
            } else if num_lower.contains("sc") || num_lower.contains("销售") {
                contract.contract_type = ContractType::SalesConfirmation;
            } else if num_lower.contains("pi") || num_lower.contains("proforma") {
                contract.contract_type = ContractType::ProformaInvoice;
            } else if num_lower.contains("ci") || num_lower.contains("invoice") {
                contract.contract_type = ContractType::CommercialInvoice;
            }
        }

        contract
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_contract() {
        let rows = vec![
            vec!["Contract No.".into(), "SC-2026-001".into()],
            vec!["Date".into(), "2026-09-15".into()],
            vec!["Seller".into(), "ABC Corp".into()],
            vec!["Buyer".into(), "XYZ Inc".into()],
            vec!["Total".into(), "50,000.00".into()],
            vec!["Currency".into(), "USD".into()],
        ];
        let contract = ContractParser::parse_from_rows(&rows);
        assert_eq!(contract.contract_number, Some("SC-2026-001".into()));
        assert_eq!(contract.seller, Some("ABC Corp".into()));
        assert_eq!(contract.buyer, Some("XYZ Inc".into()));
        assert_eq!(contract.total_amount, Some(50000.0));
    }

    #[test]
    fn test_parse_chinese_contract() {
        let rows = vec![
            vec!["合同编号".into(), "PC-2026-002".into()],
            vec!["卖方".into(), "供应商A".into()],
            vec!["买方".into(), "采购商B".into()],
            vec!["合计".into(), "100,000.00".into()],
        ];
        let contract = ContractParser::parse_from_rows(&rows);
        assert_eq!(contract.contract_type, ContractType::PurchaseContract);
        assert_eq!(contract.seller, Some("供应商A".into()));
    }
}
