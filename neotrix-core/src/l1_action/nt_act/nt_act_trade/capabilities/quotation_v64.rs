//! v6.4 报价单数学 + V58 满足性/置信度（Layer 4 输出规则代码化）.
//!
//! 来源：外贸报价单英文版本 v6.4（11 列/单价=成本/0.85/FCA+1500/TERMS 8 条）
//!     ＋ V58 Layer 4（列 15 仅本地价/B 级强制未查到）＋ 附录 F 决策树。
//! 规则：无成本留空；B 级强制「未查到」；金额纯数字。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// 未查到标记（V58 BUG-06：数值列只许纯数字或此标记）
pub const UNMATCHED: &str = "未查到";

/// 默认国内运费（v6.4 §2.5）
pub const DEFAULT_FREIGHT: f64 = 1500.0;

/// 置信度（附录 F）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Confidence {
    /// 完全匹配 → 列 15 填价格
    A,
    /// 部分匹配 → 列 15 强制「未查到」
    B,
    /// 无匹配 → 列 15「未查到」，列 16「无法供应」
    C,
}

/// 单价 = 成本 / 0.85，保留 1 位小数；无成本/零成本 → None（留空）
pub fn unit_price(cost_incl_tax: Option<f64>) -> Option<f64> {
    match cost_incl_tax {
        Some(c) if c > 0.0 => Some((c / 0.85 * 10.0).round() / 10.0),
        _ => None,
    }
}

/// v6.4 报价行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRow {
    pub item: u32,
    pub code: String,
    pub name_en: String,
    pub desc_en: String,
    pub size_dn: String,
    pub qty: f64,
    pub nw: Option<f64>,
    pub gw: Option<f64>,
    pub unit_price: Option<f64>,
    pub total_price: Option<f64>,
}

/// v6.4 报价单汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quotation {
    pub rows: Vec<QuoteRow>,
    pub total_qty: f64,
    pub total_gw: f64,
    pub total_amount: f64,
    pub freight: f64,
    pub fca_shanghai: f64,
}

/// TERMS 8 条（v6.4 §3.2，逐字）
pub fn terms() -> Vec<String> {
    vec![
        "Country of Origin: China. WEIZIDOM VALVE GROUP.".to_string(),
        "Packing Details: Bubble bag, sponge, wooden case.".to_string(),
        "Shipment Term: FCA 上海港价格.".to_string(),
        "Payment Term: 50% deposit by T/T in advance for producing, 50% balanced to be paid before shipping.".to_string(),
        "Guarantee: 18 months after shipping.".to_string(),
        "Lead time: About 30 days after receipt of advanced payment.".to_string(),
        "Period of Validity: 20 days.".to_string(),
        "Shipping Documents: Packing List & Invoice by email. Documentation: Drawing, Material Certificates & Test Report by email.".to_string(),
    ]
}

/// 报价单输入行（调用方已做价格匹配，只传干净数值）
#[derive(Debug, Clone, Default)]
pub struct QuoteInput {
    pub code: String,
    pub name_en: String,
    pub desc_en: String,
    pub size_dn: String,
    pub qty: f64,
    pub cost_incl_tax: Option<f64>,
    pub nw: Option<f64>,
}

/// 构建 v6.4 报价单（Total = Σ；FCA = Total + 运费）
pub fn build_quotation(inputs: &[QuoteInput], freight: f64) -> Quotation {
    let mut rows = Vec::with_capacity(inputs.len());
    for (i, it) in inputs.iter().enumerate() {
        let unit = unit_price(it.cost_incl_tax);
        let total = unit.map(|u| (u * it.qty * 100.0).round() / 100.0);
        let gw = it.nw.map(|n| (n * it.qty * 100.0).round() / 100.0);
        rows.push(QuoteRow {
            item: (i + 1) as u32,
            code: it.code.clone(),
            name_en: it.name_en.clone(),
            desc_en: it.desc_en.clone(),
            size_dn: it.size_dn.clone(),
            qty: it.qty,
            nw: it.nw,
            gw,
            unit_price: unit,
            total_price: total,
        });
    }
    let total_qty: f64 = rows.iter().map(|r| r.qty).sum();
    let total_gw: f64 = rows.iter().filter_map(|r| r.gw).sum();
    let total_amount: f64 = rows.iter().filter_map(|r| r.total_price).sum();
    let total_amount = (total_amount * 100.0).round() / 100.0;
    Quotation {
        rows,
        total_qty,
        total_gw,
        total_amount,
        freight,
        fca_shanghai: ((total_amount + freight) * 100.0).round() / 100.0,
    }
}

/// 列 16 满足性（附录 F）：A→满足；B→否（差异进备注）；C→无法供应
pub fn satisfy_text(grade: Confidence) -> &'static str {
    match grade {
        Confidence::A => "满足",
        Confidence::B => "否",
        Confidence::C => "无法供应",
    }
}

/// 列 15 取值：A 级填价格（纯数字），B/C 级强制「未查到」（BUG-24）
pub fn price_cell(grade: Confidence, unit: Option<f64>) -> String {
    match (grade, unit) {
        (Confidence::A, Some(u)) => format!("{u:.2}"),
        _ => UNMATCHED.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_price_math() {
        assert_eq!(unit_price(Some(165.0)), Some(194.1));
        assert_eq!(unit_price(Some(48.0)), Some(56.5));
        assert_eq!(unit_price(None), None);
        assert_eq!(unit_price(Some(0.0)), None);
    }

    #[test]
    fn quotation_totals() {
        let q = build_quotation(
            &[
                QuoteInput {
                    code: "2".to_string(),
                    name_en: "Gate Valve".to_string(),
                    qty: 10.0,
                    cost_incl_tax: Some(165.0),
                    nw: Some(8.9),
                    ..Default::default()
                },
                QuoteInput {
                    code: "4668".to_string(),
                    name_en: "Butterfly Valve".to_string(),
                    qty: 20.0,
                    cost_incl_tax: Some(48.0),
                    nw: Some(3.1),
                    ..Default::default()
                },
            ],
            DEFAULT_FREIGHT,
        );
        assert!((q.total_amount - 3071.0).abs() < 0.01);
        assert!((q.fca_shanghai - 4571.0).abs() < 0.01);
        assert!((q.total_qty - 30.0).abs() < f64::EPSILON);
        assert_eq!(q.rows[0].gw, Some(89.0));
    }

    #[test]
    fn grade_gates_price_cell() {
        assert_eq!(price_cell(Confidence::A, Some(194.1)), "194.10");
        assert_eq!(price_cell(Confidence::B, Some(194.1)), UNMATCHED);
        assert_eq!(price_cell(Confidence::A, None), UNMATCHED);
        assert_eq!(satisfy_text(Confidence::C), "无法供应");
    }

    #[test]
    fn terms_count() {
        assert_eq!(terms().len(), 8);
    }
}
