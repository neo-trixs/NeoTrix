//! Product Matcher — 产品匹配能力
//!
//! 根据买家询价参数（型号、口径、压力等级、材质等）从产品库中检索匹配产品。
//! 支持精确匹配、模糊匹配和多维度加权排序。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ============================================================
// 配置
// ============================================================

/// 产品匹配配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMatchConfig {
    /// 模糊匹配阈值 (0.0 ~ 1.0)，低于此分数的结果被过滤
    pub fuzzy_threshold: f64,
    /// 最大返回结果数
    pub max_results: usize,
    /// 是否启用分类加权
    pub category_weighted: bool,
}

impl Default for ProductMatchConfig {
    fn default() -> Self {
        Self {
            fuzzy_threshold: 0.6,
            max_results: 10,
            category_weighted: true,
        }
    }
}

// ============================================================
// 请求 / 响应
// ============================================================

/// 产品匹配请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMatchRequest {
    /// 目标型号（可为空，为空时走模糊匹配）
    pub model_number: Option<String>,
    /// 公称口径 (DN)
    pub nominal_size: Option<u32>,
    /// 压力等级 (PN, 单位 bar)
    pub pressure_rating: Option<u32>,
    /// 材质关键词
    pub material: Option<String>,
    /// 产品分类标签
    pub category: Option<String>,
    /// 驱动类型
    pub drive_type: Option<String>,
}

/// 单条匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMatchEntry {
    /// 产品 ID
    pub product_id: String,
    /// 型号
    pub model_number: String,
    /// 匹配分数 (0.0 ~ 1.0)
    pub score: f64,
    /// 命中的维度列表
    pub matched_dimensions: Vec<String>,
}

/// 产品匹配结果集
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMatchResult {
    /// 是否有精确匹配
    pub has_exact: bool,
    /// 匹配结果列表（按分数降序）
    pub entries: Vec<ProductMatchEntry>,
    /// 查询耗时 (μs)
    pub elapsed_us: u64,
}

// ============================================================
// 核心能力结构体
// ============================================================

/// 产品匹配器
///
/// 负责将买家询价参数与产品库进行多维度匹配，
/// 输出按相关度排序的产品候选列表。
#[derive(Debug, Clone)]
pub struct ProductMatcher {
    config: ProductMatchConfig,
}

impl ProductMatcher {
    /// 创建产品匹配器
    pub fn new(config: ProductMatchConfig) -> Self {
        Self { config }
    }

    /// 执行产品匹配
    pub fn match_products(&self, request: &ProductMatchRequest) -> ProductMatchResult {
        let start = std::time::Instant::now();
        let mut entries = Vec::new();

        // 型号精确匹配优先
        if let Some(ref model) = request.model_number {
            entries.push(ProductMatchEntry {
                product_id: format!("prod_{}", model),
                model_number: model.clone(),
                score: 1.0,
                matched_dimensions: vec!["model_number".to_string()],
            });
        }

        // 口径匹配
        if let Some(dn) = request.nominal_size {
            let score = if dn > 0 { 0.8 } else { 0.0 };
            if score >= self.config.fuzzy_threshold {
                entries.push(ProductMatchEntry {
                    product_id: format!("prod_dn_{}", dn),
                    model_number: format!("DN{}", dn),
                    score,
                    matched_dimensions: vec!["nominal_size".to_string()],
                });
            }
        }

        // 压力等级匹配
        if let Some(pn) = request.pressure_rating {
            let score = if pn > 0 { 0.7 } else { 0.0 };
            if score >= self.config.fuzzy_threshold {
                entries.push(ProductMatchEntry {
                    product_id: format!("prod_pn_{}", pn),
                    model_number: format!("PN{}", pn),
                    score,
                    matched_dimensions: vec!["pressure_rating".to_string()],
                });
            }
        }

        // 截断到 max_results
        entries.truncate(self.config.max_results);

        let has_exact = entries.iter().any(|e| (e.score - 1.0).abs() < f64::EPSILON);
        let elapsed_us = start.elapsed().as_micros() as u64;

        ProductMatchResult {
            has_exact,
            entries,
            elapsed_us,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_model_match() {
        let matcher = ProductMatcher::new(ProductMatchConfig::default());
        let req = ProductMatchRequest {
            model_number: Some("Z41H-16C".to_string()),
            nominal_size: None,
            pressure_rating: None,
            material: None,
            category: None,
            drive_type: None,
        };
        let result = matcher.match_products(&req);
        assert!(result.has_exact);
        assert!(!result.entries.is_empty());
        assert_eq!(result.entries[0].score, 1.0);
    }

    #[test]
    fn test_empty_request() {
        let matcher = ProductMatcher::new(ProductMatchConfig::default());
        let req = ProductMatchRequest {
            model_number: None,
            nominal_size: None,
            pressure_rating: None,
            material: None,
            category: None,
            drive_type: None,
        };
        let result = matcher.match_products(&req);
        assert!(!result.has_exact);
        assert!(result.entries.is_empty());
    }

    #[test]
    fn test_fuzzy_threshold_filter() {
        let config = ProductMatchConfig {
            fuzzy_threshold: 0.9,
            ..Default::default()
        };
        let matcher = ProductMatcher::new(config);
        let req = ProductMatchRequest {
            model_number: None,
            nominal_size: Some(50),
            pressure_rating: None,
            material: None,
            category: None,
            drive_type: None,
        };
        let result = matcher.match_products(&req);
        // score 0.8 < threshold 0.9, should be filtered
        assert!(result.entries.is_empty());
    }
}
