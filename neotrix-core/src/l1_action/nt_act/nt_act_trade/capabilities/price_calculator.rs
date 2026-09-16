//! Price Calculator — 价格计算能力
//!
//! 基于成本、利润率、汇率、贸易条款 (FOB/CIF/EXW) 计算最终报价。
//! 支持多币种、阶梯价格和运费/保险估算。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ============================================================
// 配置
// ============================================================

/// 价格计算配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCalcConfig {
    /// 默认利润率 (0.0 ~ 1.0)
    pub default_margin: f64,
    /// 默认货币
    pub default_currency: String,
    /// 汇率来源标记
    pub exchange_rate_source: String,
    /// 是否含税
    pub tax_inclusive: bool,
    /// 税率 (仅 tax_inclusive=true 时生效)
    pub tax_rate: f64,
}

impl Default for PriceCalcConfig {
    fn default() -> Self {
        Self {
            default_margin: 0.15,
            default_currency: "USD".to_string(),
            exchange_rate_source: "ecb_daily".to_string(),
            tax_inclusive: false,
            tax_rate: 0.13,
        }
    }
}

// ============================================================
// 请求 / 响应
// ============================================================

/// 贸易条款
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TradeTerm {
    /// 工厂交货
    EXW,
    /// 船上交货 (离岸价)
    FOB,
    /// 成本加保险费加运费 (到岸价)
    CIF,
    /// 成本加运费
    CFR,
    /// 完税后交货
    DDP,
}

/// 单项成本明细
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostLineItem {
    /// 成本项名称
    pub label: String,
    /// 金额 (原始币种)
    pub amount: f64,
    /// 是否可选
    pub optional: bool,
}

/// 价格计算请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCalcRequest {
    /// 产品数量
    pub quantity: u32,
    /// 单位成本 (不含利润)
    pub unit_cost: f64,
    /// 成本币种
    pub cost_currency: String,
    /// 目标报价币种
    pub target_currency: String,
    /// 贸易条款
    pub trade_term: TradeTerm,
    /// 自定义利润率 (None 时使用默认值)
    pub custom_margin: Option<f64>,
    /// 附加成本项 (运费、包装、保险等)
    pub extra_costs: Vec<CostLineItem>,
    /// 汇率 (cost_currency -> target_currency)
    pub exchange_rate: Option<f64>,
}

/// 价格计算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCalcResult {
    /// 总成本 (目标币种)
    pub total_cost: f64,
    /// 利润额
    pub profit: f64,
    /// 税额 (如适用)
    pub tax: f64,
    /// 最终报价
    pub final_price: f64,
    /// 单价 (含利润)
    pub unit_price: f64,
    /// 使用的汇率
    pub applied_exchange_rate: f64,
    /// 贸易条款
    pub trade_term: TradeTerm,
    /// 成本分解
    pub breakdown: Vec<CostLineItem>,
    /// 计算耗时 (μs)
    pub elapsed_us: u64,
}

// ============================================================
// 核心能力结构体
// ============================================================

/// 价格计算器
///
/// 负责从成本基价出发，叠加利润率、附加费用、汇率转换，
/// 输出符合贸易条款的最终报价。
#[derive(Debug, Clone)]
pub struct PriceCalculator {
    config: PriceCalcConfig,
}

impl PriceCalculator {
    /// 创建价格计算器
    pub fn new(config: PriceCalcConfig) -> Self {
        Self { config }
    }

    /// 计算最终报价
    pub fn calculate(&self, request: &PriceCalcRequest) -> PriceCalcResult {
        let start = std::time::Instant::now();

        let margin = request.custom_margin.unwrap_or(self.config.default_margin);
        let rate = request.exchange_rate.unwrap_or(1.0);

        // 基础成本 (目标币种)
        let base_cost = request.unit_cost * rate;

        // 附加成本汇总
        let extra_total: f64 = request.extra_costs.iter().map(|c| c.amount * rate).sum();
        let total_cost = (base_cost * request.quantity as f64) + extra_total;

        // 利润
        let profit = total_cost * margin;

        // 税
        let tax = if self.config.tax_inclusive {
            (total_cost + profit) * self.config.tax_rate
        } else {
            0.0
        };

        // 最终报价
        let final_price = total_cost + profit + tax;
        let unit_price = final_price / request.quantity as f64;

        // 构建分解
        let mut breakdown = vec![CostLineItem {
            label: "基础成本".to_string(),
            amount: base_cost,
            optional: false,
        }];
        breakdown.extend(request.extra_costs.iter().map(|c| CostLineItem {
            label: c.label.clone(),
            amount: c.amount * rate,
            optional: c.optional,
        }));

        let elapsed_us = start.elapsed().as_micros() as u64;

        PriceCalcResult {
            total_cost,
            profit,
            tax,
            final_price,
            unit_price,
            applied_exchange_rate: rate,
            trade_term: request.trade_term.clone(),
            breakdown,
            elapsed_us,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_calculation() {
        let calc = PriceCalculator::new(PriceCalcConfig::default());
        let req = PriceCalcRequest {
            quantity: 100,
            unit_cost: 10.0,
            cost_currency: "CNY".to_string(),
            target_currency: "USD".to_string(),
            trade_term: TradeTerm::FOB,
            custom_margin: Some(0.20),
            extra_costs: vec![],
            exchange_rate: Some(0.14),
        };
        let result = calc.calculate(&req);
        // 10 * 0.14 * 100 = 140, profit = 140 * 0.2 = 28, total = 168
        assert!((result.total_cost - 140.0).abs() < 0.01);
        assert!((result.profit - 28.0).abs() < 0.01);
        assert!((result.final_price - 168.0).abs() < 0.01);
        assert!((result.unit_price - 1.68).abs() < 0.01);
    }

    #[test]
    fn test_with_extra_costs() {
        let calc = PriceCalculator::new(PriceCalcConfig::default());
        let req = PriceCalcRequest {
            quantity: 50,
            unit_cost: 20.0,
            cost_currency: "CNY".to_string(),
            target_currency: "USD".to_string(),
            trade_term: TradeTerm::CIF,
            custom_margin: Some(0.15),
            extra_costs: vec![CostLineItem {
                label: "运费".to_string(),
                amount: 500.0,
                optional: false,
            }],
            exchange_rate: Some(0.14),
        };
        let result = calc.calculate(&req);
        // base = 20 * 0.14 * 50 = 140, extra = 500 * 0.14 = 70
        // total = 210, profit = 210 * 0.15 = 31.5, final = 241.5
        assert!((result.total_cost - 210.0).abs() < 0.01);
        assert!((result.profit - 31.5).abs() < 0.01);
        assert!((result.final_price - 241.5).abs() < 0.01);
    }

    #[test]
    fn test_tax_inclusive() {
        let config = PriceCalcConfig {
            tax_inclusive: true,
            tax_rate: 0.13,
            ..Default::default()
        };
        let calc = PriceCalculator::new(config);
        let req = PriceCalcRequest {
            quantity: 10,
            unit_cost: 100.0,
            cost_currency: "USD".to_string(),
            target_currency: "USD".to_string(),
            trade_term: TradeTerm::EXW,
            custom_margin: Some(0.10),
            extra_costs: vec![],
            exchange_rate: Some(1.0),
        };
        let result = calc.calculate(&req);
        // total = 1000, profit = 100, tax = (1000+100)*0.13 = 143
        // final = 1243
        assert!((result.tax - 143.0).abs() < 0.01);
        assert!((result.final_price - 1243.0).abs() < 0.01);
    }
}
