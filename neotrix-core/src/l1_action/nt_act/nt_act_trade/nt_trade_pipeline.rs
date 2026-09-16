//! Trade Pipeline — 销售管道/商机追踪模块
//!
//! 对标 TMS 平台的商机管理能力：
//! - 商机生命周期 (创建→跟进→报价→谈判→成交/流失)
//! - 管道视图 (按阶段分组)
//! - 赢率预测 + 加权金额
//! - 历史转化率分析
//! - 商机关联 (询盘→报价→订单)

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 1. 商机阶段
// ============================================================

/// 商机阶段 — 外贸销售管道标准阶段
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DealStage {
    /// 新线索 (刚获取)
    NewLead,
    /// 已联系 (首次沟通)
    Contacted,
    /// 需求确认 (了解买家需求)
    RequirementConfirmed,
    /// 已报价 (报价单已发送)
    Quoted,
    /// 谈判中 (价格/条款协商)
    Negotiating,
    /// 合同阶段 (合同审核/签署)
    Contracting,
    /// 已成交 (订单确认)
    Won,
    /// 已流失 (竞品/放弃)
    Lost,
}

impl DealStage {
    /// 阶段顺序 (用于排序和管道视图)
    pub fn order(&self) -> u32 {
        match self {
            Self::NewLead => 0,
            Self::Contacted => 1,
            Self::RequirementConfirmed => 2,
            Self::Quoted => 3,
            Self::Negotiating => 4,
            Self::Contracting => 5,
            Self::Won => 6,
            Self::Lost => 7,
        }
    }

    /// 默认赢率 (0.0 - 1.0)
    pub fn default_win_rate(&self) -> f64 {
        match self {
            Self::NewLead => 0.05,
            Self::Contacted => 0.10,
            Self::RequirementConfirmed => 0.20,
            Self::Quoted => 0.40,
            Self::Negotiating => 0.60,
            Self::Contracting => 0.80,
            Self::Won => 1.0,
            Self::Lost => 0.0,
        }
    }
}

impl std::fmt::Display for DealStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NewLead => write!(f, "新线索"),
            Self::Contacted => write!(f, "已联系"),
            Self::RequirementConfirmed => write!(f, "需求确认"),
            Self::Quoted => write!(f, "已报价"),
            Self::Negotiating => write!(f, "谈判中"),
            Self::Contracting => write!(f, "合同阶段"),
            Self::Won => write!(f, "已成交"),
            Self::Lost => write!(f, "已流失"),
        }
    }
}

/// 流失原因
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LossReason {
    /// 价格过高
    PriceTooHigh,
    /// 选择了竞品
    CompetitorWon,
    /// 需求取消
    RequirementCancelled,
    /// 无响应
    NoResponse,
    /// 质量不满足
    QualityMismatch,
    /// 交期不满足
    LeadTimeIssue,
    /// 其他
    Other(String),
}

// ============================================================
// 2. 商机记录
// ============================================================

/// 商机记录 — 销售管道中的单个商机
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    /// 商机 ID
    pub id: String,
    /// 商机标题 (通常为产品+客户)
    pub title: String,
    /// 关联客户 ID
    pub customer_id: String,
    /// 关联公司 ID
    pub company_id: String,
    /// 关联询盘 ID (可选)
    pub inquiry_id: Option<String>,
    /// 关联报价 ID (可选)
    pub quote_id: Option<String>,
    /// 关联订单 ID (可选)
    pub order_id: Option<String>,
    /// 当前阶段
    pub stage: DealStage,
    /// 预计金额 (USD)
    pub expected_amount: f64,
    /// 实际金额 (USD, 成交后填入)
    pub actual_amount: Option<f64>,
    /// 赢率 (0.0 - 1.0)
    pub win_rate: f64,
    /// 加权金额 = expected_amount * win_rate
    pub weighted_amount: f64,
    /// 货币
    pub currency: String,
    /// 负责业务员 ID
    pub owner_id: String,
    /// 预计成交日期
    pub expected_close_date: Option<String>,
    /// 实际成交/流失日期
    pub actual_close_date: Option<String>,
    /// 流失原因
    pub loss_reason: Option<LossReason>,
    /// 阶段历史 (每次阶段变更记录)
    pub stage_history: Vec<StageChange>,
    /// 跟进记录
    pub notes: Vec<DealNote>,
    /// 标签
    pub tags: Vec<String>,
    /// 创建时间
    pub created_at: u64,
    /// 更新时间
    pub updated_at: u64,
}

impl Deal {
    /// 计算加权金额
    pub fn calculate_weighted(&mut self) {
        self.weighted_amount = self.expected_amount * self.win_rate;
    }
}

/// 阶段变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageChange {
    pub from: DealStage,
    pub to: DealStage,
    pub changed_at: u64,
    pub changed_by: String,
    pub note: Option<String>,
}

/// 商机备注
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealNote {
    pub id: String,
    pub content: String,
    pub author_id: String,
    pub timestamp: u64,
}

// ============================================================
// 3. 管道视图
// ============================================================

/// 管道阶段汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStageSummary {
    pub stage: DealStage,
    pub deal_count: u32,
    pub total_amount: f64,
    pub weighted_amount: f64,
    pub deals: Vec<DealSummary>,
}

/// 商机摘要 (管道视图用)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealSummary {
    pub id: String,
    pub title: String,
    pub customer_name: String,
    pub expected_amount: f64,
    pub win_rate: f64,
    pub owner_id: String,
    pub days_in_stage: u32,
    pub updated_at: u64,
}

/// 完整管道视图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineView {
    pub stages: Vec<PipelineStageSummary>,
    pub total_deals: u32,
    pub total_amount: f64,
    pub total_weighted: f64,
    pub win_rate_overall: f64,
}

// ============================================================
// 4. 管道引擎
// ============================================================

/// 销售管道引擎
pub struct TradePipelineEngine {
    deals: HashMap<String, Deal>,
    /// 商机索引: customer_id → deal_ids
    customer_index: HashMap<String, Vec<String>>,
    /// 历史转化率统计
    conversion_stats: ConversionStats,
}

/// 转化率统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversionStats {
    pub total_leads: u64,
    pub total_won: u64,
    pub total_lost: u64,
    /// 按阶段的转化率
    pub stage_conversions: HashMap<String, f64>,
}

impl Default for TradePipelineEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TradePipelineEngine {
    pub fn new() -> Self {
        Self {
            deals: HashMap::new(),
            customer_index: HashMap::new(),
            conversion_stats: ConversionStats::default(),
        }
    }

    /// 创建商机
    pub fn create_deal(
        &mut self,
        title: &str,
        customer_id: &str,
        company_id: &str,
        expected_amount: f64,
        currency: &str,
        owner_id: &str,
    ) -> Deal {
        let now = self.current_timestamp();
        let deal = Deal {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            customer_id: customer_id.to_string(),
            company_id: company_id.to_string(),
            inquiry_id: None,
            quote_id: None,
            order_id: None,
            stage: DealStage::NewLead,
            expected_amount,
            actual_amount: None,
            win_rate: DealStage::NewLead.default_win_rate(),
            weighted_amount: expected_amount * DealStage::NewLead.default_win_rate(),
            currency: currency.to_string(),
            owner_id: owner_id.to_string(),
            expected_close_date: None,
            actual_close_date: None,
            loss_reason: None,
            stage_history: Vec::new(),
            notes: Vec::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        self.customer_index
            .entry(customer_id.to_string())
            .or_default()
            .push(deal.id.clone());
        self.deals.insert(deal.id.clone(), deal.clone());
        self.conversion_stats.total_leads += 1;
        deal
    }

    /// 推进商机到下一阶段
    pub fn advance_stage(
        &mut self,
        deal_id: &str,
        operator_id: &str,
        note: Option<String>,
    ) -> Result<&Deal, String> {
        let now = self.current_timestamp();
        let deal = self
            .deals
            .get_mut(deal_id)
            .ok_or_else(|| format!("Deal {} not found", deal_id))?;
        let current = deal.stage.clone();
        let next = match current {
            DealStage::NewLead => DealStage::Contacted,
            DealStage::Contacted => DealStage::RequirementConfirmed,
            DealStage::RequirementConfirmed => DealStage::Quoted,
            DealStage::Quoted => DealStage::Negotiating,
            DealStage::Negotiating => DealStage::Contracting,
            DealStage::Contracting => DealStage::Won,
            DealStage::Won | DealStage::Lost => return Err("Deal already closed".into()),
        };
        deal.stage_history.push(StageChange {
            from: current,
            to: next.clone(),
            changed_at: now,
            changed_by: operator_id.to_string(),
            note,
        });
        deal.stage = next.clone();
        deal.win_rate = next.default_win_rate();
        deal.calculate_weighted();
        deal.updated_at = now;
        Ok(self.deals.get(deal_id).unwrap())
    }

    /// 标记商机成交
    pub fn mark_won(
        &mut self,
        deal_id: &str,
        actual_amount: f64,
        operator_id: &str,
    ) -> Result<&Deal, String> {
        let now = self.current_timestamp();
        let deal = self
            .deals
            .get_mut(deal_id)
            .ok_or_else(|| format!("Deal {} not found", deal_id))?;
        deal.stage_history.push(StageChange {
            from: deal.stage.clone(),
            to: DealStage::Won,
            changed_at: now,
            changed_by: operator_id.to_string(),
            note: Some("Deal won".into()),
        });
        deal.stage = DealStage::Won;
        deal.win_rate = 1.0;
        deal.actual_amount = Some(actual_amount);
        deal.actual_close_date = Some(now.to_string());
        deal.calculate_weighted();
        deal.updated_at = now;
        self.conversion_stats.total_won += 1;
        Ok(self.deals.get(deal_id).unwrap())
    }

    /// 标记商机流失
    pub fn mark_lost(
        &mut self,
        deal_id: &str,
        reason: LossReason,
        operator_id: &str,
    ) -> Result<&Deal, String> {
        let now = self.current_timestamp();
        let deal = self
            .deals
            .get_mut(deal_id)
            .ok_or_else(|| format!("Deal {} not found", deal_id))?;
        deal.stage_history.push(StageChange {
            from: deal.stage.clone(),
            to: DealStage::Lost,
            changed_at: now,
            changed_by: operator_id.to_string(),
            note: Some(format!("Lost: {:?}", reason)),
        });
        deal.stage = DealStage::Lost;
        deal.win_rate = 0.0;
        deal.loss_reason = Some(reason);
        deal.actual_close_date = Some(now.to_string());
        deal.calculate_weighted();
        deal.updated_at = now;
        self.conversion_stats.total_lost += 1;
        Ok(self.deals.get(deal_id).unwrap())
    }

    /// 获取管道视图
    pub fn pipeline_view(&self) -> PipelineView {
        let stages: Vec<_> = DealStage::all_stages()
            .iter()
            .filter(|s| **s != DealStage::Won && **s != DealStage::Lost)
            .map(|stage| {
                let deals_in_stage: Vec<&Deal> = self
                    .deals
                    .values()
                    .filter(|d| d.stage == *stage)
                    .collect();
                let total_amount: f64 = deals_in_stage.iter().map(|d| d.expected_amount).sum();
                let weighted: f64 = deals_in_stage.iter().map(|d| d.weighted_amount).sum();
                let summaries: Vec<DealSummary> = deals_in_stage
                    .iter()
                    .map(|d| DealSummary {
                        id: d.id.clone(),
                        title: d.title.clone(),
                        customer_name: d.customer_id.clone(), // 实际应从 CRM 查
                        expected_amount: d.expected_amount,
                        win_rate: d.win_rate,
                        owner_id: d.owner_id.clone(),
                        days_in_stage: 0, // TODO: 计算天数
                        updated_at: d.updated_at,
                    })
                    .collect();
                PipelineStageSummary {
                    stage: stage.clone(),
                    deal_count: deals_in_stage.len() as u32,
                    total_amount,
                    weighted_amount: weighted,
                    deals: summaries,
                }
            })
            .collect();
        let total_deals: u32 = stages.iter().map(|s| s.deal_count).sum();
        let total_amount: f64 = stages.iter().map(|s| s.total_amount).sum();
        let total_weighted: f64 = stages.iter().map(|s| s.weighted_amount).sum();
        let won = self.conversion_stats.total_won as f64;
        let total_closed = won + self.conversion_stats.total_lost as f64;
        PipelineView {
            stages,
            total_deals,
            total_amount,
            total_weighted,
            win_rate_overall: if total_closed > 0.0 {
                won / total_closed
            } else {
                0.0
            },
        }
    }

    /// 获取客户的所有商机
    pub fn customer_deals(&self, customer_id: &str) -> Vec<&Deal> {
        self.customer_index
            .get(customer_id)
            .map(|ids| ids.iter().filter_map(|id| self.deals.get(id)).collect())
            .unwrap_or_default()
    }

    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl DealStage {
    fn all_stages() -> Vec<DealStage> {
        vec![
            DealStage::NewLead,
            DealStage::Contacted,
            DealStage::RequirementConfirmed,
            DealStage::Quoted,
            DealStage::Negotiating,
            DealStage::Contracting,
            DealStage::Won,
            DealStage::Lost,
        ]
    }
}

// ============================================================
// 5. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_deal() {
        let mut engine = TradePipelineEngine::new();
        let deal = engine.create_deal(
            "Widget A for Acme",
            "CUST-001",
            "COMP-001",
            50_000.0,
            "USD",
            "sales-01",
        );
        assert_eq!(deal.stage, DealStage::NewLead);
        assert!(deal.win_rate > 0.0);
        assert!(deal.weighted_amount > 0.0);
    }

    #[test]
    fn test_advance_stage() {
        let mut engine = TradePipelineEngine::new();
        let deal = engine.create_deal("Test", "C1", "COMP1", 10_000.0, "USD", "s1");
        let deal_id = deal.id.clone();
        let deal = engine.advance_stage(&deal_id, "s1", None).unwrap();
        assert_eq!(deal.stage, DealStage::Contacted);
        let deal = engine.advance_stage(&deal_id, "s1", None).unwrap();
        assert_eq!(deal.stage, DealStage::RequirementConfirmed);
    }

    #[test]
    fn test_mark_won() {
        let mut engine = TradePipelineEngine::new();
        let deal = engine.create_deal("Test", "C1", "COMP1", 10_000.0, "USD", "s1");
        let deal = engine.mark_won(&deal.id, 9_500.0, "s1").unwrap();
        assert_eq!(deal.stage, DealStage::Won);
        assert_eq!(deal.actual_amount, Some(9_500.0));
        assert_eq!(deal.win_rate, 1.0);
    }

    #[test]
    fn test_mark_lost() {
        let mut engine = TradePipelineEngine::new();
        let deal = engine.create_deal("Test", "C1", "COMP1", 10_000.0, "USD", "s1");
        let deal = engine.mark_lost(&deal.id, LossReason::PriceTooHigh, "s1").unwrap();
        assert_eq!(deal.stage, DealStage::Lost);
        assert_eq!(deal.win_rate, 0.0);
    }

    #[test]
    fn test_pipeline_view() {
        let mut engine = TradePipelineEngine::new();
        engine.create_deal("Deal 1", "C1", "COMP1", 10_000.0, "USD", "s1");
        engine.create_deal("Deal 2", "C2", "COMP2", 20_000.0, "USD", "s1");
        let view = engine.pipeline_view();
        assert_eq!(view.total_deals, 2);
        assert_eq!(view.total_amount, 30_000.0);
    }

    #[test]
    fn test_customer_deals() {
        let mut engine = TradePipelineEngine::new();
        engine.create_deal("Deal 1", "C1", "COMP1", 10_000.0, "USD", "s1");
        engine.create_deal("Deal 2", "C1", "COMP1", 20_000.0, "USD", "s1");
        engine.create_deal("Deal 3", "C2", "COMP2", 30_000.0, "USD", "s1");
        let deals = engine.customer_deals("C1");
        assert_eq!(deals.len(), 2);
    }
}
