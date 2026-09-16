//! Supplier Matcher — 供应商匹配能力
//!
//! 根据产品需求、地域偏好、信用评级等维度从供应商库中筛选最佳供应商。
//! 支持多条件组合查询和加权排序。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ============================================================
// 配置
// ============================================================

/// 供应商匹配配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierMatchConfig {
    /// 最低信用评级 (1-5)
    pub min_credit_rating: u8,
    /// 最大返回结果数
    pub max_results: usize,
    /// 地域权重 (0.0 ~ 1.0)
    pub region_weight: f64,
    /// 价格权重
    pub price_weight: f64,
    /// 交期权重
    pub delivery_weight: f64,
}

impl Default for SupplierMatchConfig {
    fn default() -> Self {
        Self {
            min_credit_rating: 3,
            max_results: 20,
            region_weight: 0.3,
            price_weight: 0.4,
            delivery_weight: 0.3,
        }
    }
}

// ============================================================
// 请求 / 响应
// ============================================================

/// 供应商匹配请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierMatchRequest {
    /// 产品分类
    pub product_category: Option<String>,
    /// 目标地域
    pub region: Option<String>,
    /// 最低产能 (件/月)
    pub min_capacity: Option<u32>,
    /// 目标价格上限 (USD)
    pub max_price_usd: Option<f64>,
    /// 最大交期 (天)
    pub max_lead_days: Option<u32>,
    /// 是否要求认证 (ISO / CE 等)
    pub require_certification: bool,
}

/// 单条供应商匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierMatchEntry {
    /// 供应商 ID
    pub supplier_id: String,
    /// 供应商名称
    pub name: String,
    /// 综合评分 (0.0 ~ 1.0)
    pub score: f64,
    /// 信用评级 (1-5)
    pub credit_rating: u8,
    /// 所在地域
    pub region: String,
    /// 月产能
    pub monthly_capacity: u32,
    /// 报价单价 (USD)
    pub quoted_price_usd: f64,
    /// 预计交期 (天)
    pub lead_days: u32,
    /// 命中的筛选维度
    pub matched_dimensions: Vec<String>,
}

/// 供应商匹配结果集
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierMatchResult {
    /// 匹配结果列表（按综合评分降序）
    pub entries: Vec<SupplierMatchEntry>,
    /// 总供应商候选数
    pub total_candidates: usize,
    /// 查询耗时 (μs)
    pub elapsed_us: u64,
}

// ============================================================
// 核心能力结构体
// ============================================================

/// 供应商匹配器
///
/// 负责从供应商库中筛选满足条件的供应商，
/// 按地域、价格、交期等维度加权排序输出候选列表。
#[derive(Debug, Clone)]
pub struct SupplierMatcher {
    config: SupplierMatchConfig,
}

impl SupplierMatcher {
    /// 创建供应商匹配器
    pub fn new(config: SupplierMatchConfig) -> Self {
        Self { config }
    }

    /// 执行供应商匹配
    pub fn match_suppliers(&self, request: &SupplierMatchRequest) -> SupplierMatchResult {
        let start = std::time::Instant::now();
        let mut entries = Vec::new();

        // 模拟供应商匹配逻辑 — 实际部署时对接知识库
        // 信用评级过滤
        if request.require_certification {
            entries.push(SupplierMatchEntry {
                supplier_id: "sup_001".to_string(),
                name: "示例供应商 A".to_string(),
                score: 0.85,
                credit_rating: 5,
                region: "华东".to_string(),
                monthly_capacity: 5000,
                quoted_price_usd: 120.0,
                lead_days: 15,
                matched_dimensions: vec!["certification".to_string(), "credit_rating".to_string()],
            });
        }

        // 地域匹配
        if let Some(ref region) = request.region {
            entries.push(SupplierMatchEntry {
                supplier_id: "sup_002".to_string(),
                name: format!("示例供应商 ({})", region),
                score: 0.75,
                credit_rating: 4,
                region: region.clone(),
                monthly_capacity: 3000,
                quoted_price_usd: 135.0,
                lead_days: 20,
                matched_dimensions: vec!["region".to_string()],
            });
        }

        // 价格过滤
        if let Some(max_price) = request.max_price_usd {
            entries.retain(|e| e.quoted_price_usd <= max_price);
        }

        // 交期过滤
        if let Some(max_days) = request.max_lead_days {
            entries.retain(|e| e.lead_days <= max_days);
        }

        // 产能过滤
        if let Some(min_cap) = request.min_capacity {
            entries.retain(|e| e.monthly_capacity >= min_cap);
        }

        // 按综合评分降序排序
        entries.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries.truncate(self.config.max_results);

        let total_candidates = entries.len();
        let elapsed_us = start.elapsed().as_micros() as u64;

        SupplierMatchResult {
            entries,
            total_candidates,
            elapsed_us,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_supplier_match() {
        let matcher = SupplierMatcher::new(SupplierMatchConfig::default());
        let req = SupplierMatchRequest {
            product_category: Some("Valve".to_string()),
            region: Some("华东".to_string()),
            min_capacity: None,
            max_price_usd: None,
            max_lead_days: None,
            require_certification: false,
        };
        let result = matcher.match_suppliers(&req);
        assert!(!result.entries.is_empty());
    }

    #[test]
    fn test_price_filter() {
        let matcher = SupplierMatcher::new(SupplierMatchConfig::default());
        let req = SupplierMatchRequest {
            product_category: None,
            region: None,
            min_capacity: None,
            max_price_usd: Some(130.0),
            max_lead_days: None,
            require_certification: true,
        };
        let result = matcher.match_suppliers(&req);
        for entry in &result.entries {
            assert!(entry.quoted_price_usd <= 130.0);
        }
    }

    #[test]
    fn test_empty_request() {
        let matcher = SupplierMatcher::new(SupplierMatchConfig::default());
        let req = SupplierMatchRequest {
            product_category: None,
            region: None,
            min_capacity: None,
            max_price_usd: None,
            max_lead_days: None,
            require_certification: false,
        };
        let result = matcher.match_suppliers(&req);
        assert!(result.entries.is_empty());
    }
}
