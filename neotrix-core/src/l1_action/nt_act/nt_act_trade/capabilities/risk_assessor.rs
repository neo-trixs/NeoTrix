//! Risk Assessor — 风险评估能力
//!
//! 对外贸交易全链路进行风险评估，覆盖信用风险、物流风险、
//! 合规风险和汇率风险。输出结构化风险发现和综合风险等级。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ============================================================
// 配置
// ============================================================

/// 风险评估配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessConfig {
    /// 信用风险权重
    pub credit_weight: f64,
    /// 物流风险权重
    pub logistics_weight: f64,
    /// 合规风险权重
    pub compliance_weight: f64,
    /// 汇率风险权重
    pub fx_weight: f64,
    /// 高风险阈值 (综合分 >= 此值标记为高风险)
    pub high_risk_threshold: f64,
}

impl Default for RiskAssessConfig {
    fn default() -> Self {
        Self {
            credit_weight: 0.30,
            logistics_weight: 0.25,
            compliance_weight: 0.30,
            fx_weight: 0.15,
            high_risk_threshold: 0.7,
        }
    }
}

// ============================================================
// 请求 / 响应
// ============================================================

/// 风险等级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// 低风险
    Low,
    /// 中风险
    Medium,
    /// 高风险
    High,
    /// 极高风险
    Critical,
}

/// 风险维度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskDimension {
    /// 信用风险
    Credit,
    /// 物流风险
    Logistics,
    /// 合规风险
    Compliance,
    /// 汇率风险
    ForeignExchange,
}

/// 单条风险发现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFinding {
    /// 风险维度
    pub dimension: RiskDimension,
    /// 风险等级
    pub level: RiskLevel,
    /// 风险描述
    pub description: String,
    /// 建议缓解措施
    pub mitigation: String,
    /// 影响分数 (0.0 ~ 1.0)
    pub impact_score: f64,
}

/// 风险评估请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessRequest {
    /// 买家 ID
    pub buyer_id: Option<String>,
    /// 供应商 ID
    pub supplier_id: Option<String>,
    /// 目标国家/地区
    pub destination_country: Option<String>,
    /// 交易金额 (USD)
    pub transaction_amount_usd: f64,
    /// 贸易条款
    pub trade_term: String,
    /// 付款方式
    pub payment_method: String,
    /// 目标币种
    pub target_currency: String,
    /// 交易天数 (用于汇率风险窗口)
    pub settlement_days: u32,
    /// 是否涉及制裁国家
    pub sanctioned_country: bool,
}

/// 风险评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessResult {
    /// 综合风险等级
    pub overall_level: RiskLevel,
    /// 综合风险分数 (0.0 ~ 1.0)
    pub overall_score: f64,
    /// 风险发现列表
    pub findings: Vec<RiskFinding>,
    /// 是否阻断 (高风险自动阻断)
    pub blocking: bool,
    /// 评估耗时 (μs)
    pub elapsed_us: u64,
}

// ============================================================
// 核心能力结构体
// ============================================================

/// 风险评估器
///
/// 负责对单笔外贸交易进行多维度风险评估，
/// 输出结构化风险发现和综合风险等级判断。
#[derive(Debug, Clone)]
pub struct RiskAssessor {
    config: RiskAssessConfig,
}

impl RiskAssessor {
    /// 创建风险评估器
    pub fn new(config: RiskAssessConfig) -> Self {
        Self { config }
    }

    /// 执行风险评估
    pub fn assess(&self, request: &RiskAssessRequest) -> RiskAssessResult {
        let start = std::time::Instant::now();
        let mut findings = Vec::new();

        // ── 信用风险 ──
        if request.buyer_id.is_none() {
            findings.push(RiskFinding {
                dimension: RiskDimension::Credit,
                level: RiskLevel::Medium,
                description: "买家信息缺失，无法进行信用评估".to_string(),
                mitigation: "要求提供买家营业执照或信用报告".to_string(),
                impact_score: 0.5,
            });
        }

        // ── 合规风险 ──
        if request.sanctioned_country {
            findings.push(RiskFinding {
                dimension: RiskDimension::Compliance,
                level: RiskLevel::Critical,
                description: "目标国家在制裁名单中".to_string(),
                mitigation: "终止交易或咨询合规部门".to_string(),
                impact_score: 1.0,
            });
        }

        // ── 物流风险 ──
        if request.trade_term == "CIF" || request.trade_term == "CFR" {
            findings.push(RiskFinding {
                dimension: RiskDimension::Logistics,
                level: RiskLevel::Low,
                description: format!("贸易条款 {} 需卖方承担运输风险", request.trade_term),
                mitigation: "确认货运保险覆盖范围".to_string(),
                impact_score: 0.2,
            });
        }

        // ── 汇率风险 ──
        if request.target_currency != "USD" && request.settlement_days > 30 {
            findings.push(RiskFinding {
                dimension: RiskDimension::ForeignExchange,
                level: RiskLevel::Medium,
                description: format!(
                    "结算周期 {} 天，目标币种 {} 存在汇率波动风险",
                    request.settlement_days, request.target_currency
                ),
                mitigation: "考虑锁定远期汇率或使用外汇对冲工具".to_string(),
                impact_score: 0.4,
            });
        }

        // ── 大额交易风险 ──
        if request.transaction_amount_usd > 500_000.0 {
            findings.push(RiskFinding {
                dimension: RiskDimension::Credit,
                level: RiskLevel::High,
                description: "大额交易（>50万美元），信用敞口较大".to_string(),
                mitigation: "要求信用证付款或分期付款".to_string(),
                impact_score: 0.7,
            });
        }

        // 计算综合分数
        let overall_score = if findings.is_empty() {
            0.0
        } else {
            findings.iter().map(|f| f.impact_score).sum::<f64>() / findings.len() as f64
        };

        let overall_level = if overall_score >= self.config.high_risk_threshold {
            RiskLevel::High
        } else if overall_score >= 0.4 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        let blocking = overall_level == RiskLevel::Critical
            || (overall_level == RiskLevel::High && request.transaction_amount_usd > 1_000_000.0);

        let elapsed_us = start.elapsed().as_micros() as u64;

        RiskAssessResult {
            overall_level,
            overall_score,
            findings,
            blocking,
            elapsed_us,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_risk_clean_request() {
        let assessor = RiskAssessor::new(RiskAssessConfig::default());
        let req = RiskAssessRequest {
            buyer_id: Some("buyer_001".to_string()),
            supplier_id: Some("sup_001".to_string()),
            destination_country: Some("Germany".to_string()),
            transaction_amount_usd: 50_000.0,
            trade_term: "FOB".to_string(),
            payment_method: "T/T".to_string(),
            target_currency: "USD".to_string(),
            settlement_days: 15,
            sanctioned_country: false,
        };
        let result = assessor.assess(&req);
        assert!(result.overall_level <= RiskLevel::Medium);
        assert!(!result.blocking);
    }

    #[test]
    fn test_sanctioned_country_critical() {
        let assessor = RiskAssessor::new(RiskAssessConfig::default());
        let req = RiskAssessRequest {
            buyer_id: None,
            supplier_id: None,
            destination_country: None,
            transaction_amount_usd: 100_000.0,
            trade_term: "CIF".to_string(),
            payment_method: "L/C".to_string(),
            target_currency: "EUR".to_string(),
            settlement_days: 45,
            sanctioned_country: true,
        };
        let result = assessor.assess(&req);
        assert_eq!(result.overall_level, RiskLevel::Critical);
        assert!(result.blocking);
    }

    #[test]
    fn test_high_value_blocking() {
        let assessor = RiskAssessor::new(RiskAssessConfig::default());
        let req = RiskAssessRequest {
            buyer_id: None,
            supplier_id: None,
            destination_country: None,
            transaction_amount_usd: 2_000_000.0,
            trade_term: "FOB".to_string(),
            payment_method: "T/T".to_string(),
            target_currency: "USD".to_string(),
            settlement_days: 10,
            sanctioned_country: false,
        };
        let result = assessor.assess(&req);
        assert!(result.blocking);
    }
}
