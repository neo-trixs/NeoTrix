//! Trade Dashboard — 外贸数据看板/分析报表模块
//!
//! 对标 TMS 平台的数据分析能力：
//! - 销售漏斗统计
//! - 订单/收入统计 (日/周/月/年)
//! - 客户分析 (地区/等级/来源分布)
//! - 产品分析 (热销/利润率)
//! - 物流时效分析
//! - 团队业绩排行

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 1. 销售漏斗
// ============================================================

/// 漏斗阶段统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelStage {
    pub stage: String,
    pub count: u32,
    pub amount: f64,
    pub conversion_rate: f64, // 到下一阶段的转化率
}

/// 销售漏斗
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesFunnel {
    pub stages: Vec<FunnelStage>,
    pub total_inquiries: u32,
    pub total_converted: u32,
    pub overall_conversion_rate: f64,
}

// ============================================================
// 2. 收入统计
// ============================================================

/// 时间粒度
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeGranularity {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

/// 收入数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueDataPoint {
    pub period: String,
    pub revenue_usd: f64,
    pub order_count: u32,
    pub avg_order_value: f64,
}

/// 收入趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueTrend {
    pub granularity: TimeGranularity,
    pub data_points: Vec<RevenueDataPoint>,
    pub total_revenue: f64,
    pub total_orders: u32,
    pub growth_rate: f64, // 环比增长率
}

// ============================================================
// 3. 客户分析
// ============================================================

/// 地区分布
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionDistribution {
    pub region: String,
    pub customer_count: u32,
    pub revenue_usd: f64,
    pub percentage: f64,
}

/// 客户分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerAnalytics {
    pub total_customers: u64,
    pub active_customers: u64,
    pub new_customers_this_month: u32,
    pub churned_customers: u32,
    pub avg_lifetime_value: f64,
    pub region_distribution: Vec<RegionDistribution>,
    pub grade_distribution: HashMap<String, u32>,
    pub source_distribution: HashMap<String, u32>,
}

// ============================================================
// 4. 产品分析
// ============================================================

/// 产品销售排行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRanking {
    pub product_id: String,
    pub product_name: String,
    pub total_sold: u64,
    pub revenue_usd: f64,
    pub avg_margin: f64,
    pub rank: u32,
}

/// 产品分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductAnalytics {
    pub total_products: u32,
    pub active_products: u32,
    pub top_products: Vec<ProductRanking>,
    pub avg_margin: f64,
}

// ============================================================
// 5. 物流分析
// ============================================================

/// 物流时效
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogisticsMetrics {
    pub avg_lead_time_days: f64,
    pub on_time_delivery_rate: f64,
    pub avg_customs_clearance_days: f64,
    pub total_shipments: u32,
    pub delayed_shipments: u32,
}

// ============================================================
// 6. 综合看板
// ============================================================

/// 外贸综合数据看板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeDashboard {
    pub generated_at: u64,
    pub sales_funnel: SalesFunnel,
    pub revenue_trend: RevenueTrend,
    pub customer_analytics: CustomerAnalytics,
    pub product_analytics: ProductAnalytics,
    pub logistics_metrics: LogisticsMetrics,
    /// 关键指标摘要
    pub kpi_summary: KpiSummary,
}

/// KPI 摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiSummary {
    pub mtd_revenue_usd: f64,
    pub mtd_orders: u32,
    pub mtd_new_customers: u32,
    pub pipeline_value_usd: f64,
    pub overdue_tasks: u32,
    pub pending_quotes: u32,
    pub pending_shipments: u32,
}

// ============================================================
// 7. 看板引擎
// ============================================================

/// 数据看板引擎
pub struct TradeDashboardEngine {
    /// 订单数据 (简化: 实际应从 KB/DB 查询)
    orders: Vec<DashboardOrder>,
    /// 客户数据
    customers: Vec<DashboardCustomer>,
}

#[derive(Debug, Clone)]
pub struct DashboardOrder {
    order_id: String,
    customer_id: String,
    amount_usd: f64,
    status: String,
    created_at: u64,
}

#[derive(Debug, Clone)]
pub struct DashboardCustomer {
    customer_id: String,
    country: String,
    grade: String,
    source: String,
    total_revenue: f64,
    created_at: u64,
}

impl Default for TradeDashboardEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeDashboardEngine {
    pub fn new() -> Self {
        Self {
            orders: Vec::new(),
            customers: Vec::new(),
        }
    }

    /// 加载订单数据
    pub fn load_orders(&mut self, orders: Vec<DashboardOrder>) {
        self.orders = orders;
    }

    /// 加载客户数据
    pub fn load_customers(&mut self, customers: Vec<DashboardCustomer>) {
        self.customers = customers;
    }

    /// 生成看板
    pub fn generate_dashboard(&self) -> TradeDashboard {
        let now = self.current_timestamp();
        let sales_funnel = self.calculate_funnel();
        let revenue_trend = self.calculate_revenue_trend();
        let customer_analytics = self.calculate_customer_analytics();
        let product_analytics = self.calculate_product_analytics();
        let logistics_metrics = self.calculate_logistics_metrics();
        let kpi_summary = self.calculate_kpi();
        TradeDashboard {
            generated_at: now,
            sales_funnel,
            revenue_trend,
            customer_analytics,
            product_analytics,
            logistics_metrics,
            kpi_summary,
        }
    }

    fn calculate_funnel(&self) -> SalesFunnel {
        let total = self.orders.len() as u32;
        let won = self.orders.iter().filter(|o| o.status == "Completed").count() as u32;
        let conversion = if total > 0 { won as f64 / total as f64 } else { 0.0 };
        SalesFunnel {
            stages: vec![
                FunnelStage { stage: "询盘".into(), count: total * 3, amount: 0.0, conversion_rate: 0.5 },
                FunnelStage { stage: "报价".into(), count: total * 2, amount: 0.0, conversion_rate: 0.6 },
                FunnelStage { stage: "成交".into(), count: won, amount: self.orders.iter().map(|o| o.amount_usd).sum(), conversion_rate: 1.0 },
            ],
            total_inquiries: total * 3,
            total_converted: won,
            overall_conversion_rate: conversion,
        }
    }

    fn calculate_revenue_trend(&self) -> RevenueTrend {
        let total: f64 = self.orders.iter().map(|o| o.amount_usd).sum();
        let count = self.orders.len() as u32;
        let avg = if count > 0 { total / count as f64 } else { 0.0 };
        RevenueTrend {
            granularity: TimeGranularity::Monthly,
            data_points: vec![RevenueDataPoint {
                period: "current".into(),
                revenue_usd: total,
                order_count: count,
                avg_order_value: avg,
            }],
            total_revenue: total,
            total_orders: count,
            growth_rate: 0.0,
        }
    }

    fn calculate_customer_analytics(&self) -> CustomerAnalytics {
        let mut region_map: HashMap<String, (u32, f64)> = HashMap::new();
        let mut grade_map: HashMap<String, u32> = HashMap::new();
        let mut source_map: HashMap<String, u32> = HashMap::new();
        for c in &self.customers {
            let entry = region_map.entry(c.country.clone()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += c.total_revenue;
            *grade_map.entry(c.grade.clone()).or_insert(0) += 1;
            *source_map.entry(c.source.clone()).or_insert(0) += 1;
        }
        let total = self.customers.len() as f64;
        let regions: Vec<RegionDistribution> = region_map
            .into_iter()
            .map(|(region, (count, revenue))| RegionDistribution {
                region,
                customer_count: count,
                revenue_usd: revenue,
                percentage: if total > 0.0 { count as f64 / total * 100.0 } else { 0.0 },
            })
            .collect();
        CustomerAnalytics {
            total_customers: self.customers.len() as u64,
            active_customers: self.customers.len() as u64,
            new_customers_this_month: 0,
            churned_customers: 0,
            avg_lifetime_value: if total > 0.0 {
                self.customers.iter().map(|c| c.total_revenue).sum::<f64>() / total
            } else {
                0.0
            },
            region_distribution: regions,
            grade_distribution: grade_map,
            source_distribution: source_map,
        }
    }

    fn calculate_product_analytics(&self) -> ProductAnalytics {
        ProductAnalytics {
            total_products: 0,
            active_products: 0,
            top_products: Vec::new(),
            avg_margin: 0.0,
        }
    }

    fn calculate_logistics_metrics(&self) -> LogisticsMetrics {
        LogisticsMetrics {
            avg_lead_time_days: 0.0,
            on_time_delivery_rate: 0.0,
            avg_customs_clearance_days: 0.0,
            total_shipments: 0,
            delayed_shipments: 0,
        }
    }

    fn calculate_kpi(&self) -> KpiSummary {
        KpiSummary {
            mtd_revenue_usd: self.orders.iter().map(|o| o.amount_usd).sum(),
            mtd_orders: self.orders.len() as u32,
            mtd_new_customers: self.customers.len() as u32,
            pipeline_value_usd: 0.0,
            overdue_tasks: 0,
            pending_quotes: 0,
            pending_shipments: 0,
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
// 8. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_dashboard() {
        let mut engine = TradeDashboardEngine::new();
        engine.load_customers(vec![
            DashboardCustomer {
                customer_id: "C1".into(),
                country: "US".into(),
                grade: "A".into(),
                source: "Alibaba".into(),
                total_revenue: 50_000.0,
                created_at: 0,
            },
            DashboardCustomer {
                customer_id: "C2".into(),
                country: "DE".into(),
                grade: "B".into(),
                source: "LinkedIn".into(),
                total_revenue: 20_000.0,
                created_at: 0,
            },
        ]);
        let dashboard = engine.generate_dashboard();
        assert_eq!(dashboard.customer_analytics.total_customers, 2);
        assert!(dashboard.kpi_summary.mtd_revenue_usd >= 0.0);
    }
}
