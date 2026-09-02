//! 成本控制模块
//!
//! 实现 Token 估算、预算管理、成本优化策略
//! 支持 AI 漫剧生产的成本控制

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 成本定义
// ============================================================================

/// 成本类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CostType {
    /// Token 消耗
    Token,
    /// API 调用
    ApiCall,
    /// GPU 算力
    GpuCompute,
    /// 存储
    Storage,
    /// 网络传输
    Network,
}

/// 成本记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecord {
    /// 记录ID
    pub id: String,
    /// 成本类型
    pub cost_type: CostType,
    /// 成本金额 (元)
    pub amount: f64,
    /// 成本描述
    pub description: String,
    /// 关联任务ID
    pub task_id: Option<String>,
    /// 关联镜头ID
    pub shot_id: Option<String>,
    /// 记录时间
    pub recorded_at: u64,
}

/// 预算配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// 项目总预算 (元)
    pub total_budget: f64,
    /// 单集预算 (元)
    pub per_episode_budget: f64,
    /// 单镜头预算 (元)
    pub per_shot_budget: f64,
    /// 预警阈值 (0.0-1.0)
    pub warning_threshold: f32,
    /// 硬性限制阈值 (0.0-1.0)
    pub hard_limit_threshold: f32,
    /// 是否启用自动暂停
    pub enable_auto_pause: bool,
}

/// 成本估算
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimation {
    /// 预估 Token 消耗
    pub estimated_tokens: u64,
    /// 预估 API 调用次数
    pub estimated_api_calls: u32,
    /// 预估 GPU 算力 (小时)
    pub estimated_gpu_hours: f32,
    /// 预估总成本 (元)
    pub estimated_total_cost: f64,
    /// 成本明细
    pub cost_breakdown: HashMap<String, f64>,
}

/// 成本优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimization {
    /// 优化类型
    pub optimization_type: String,
    /// 优化描述
    pub description: String,
    /// 预估节省 (元)
    pub estimated_savings: f64,
    /// 优先级
    pub priority: u32,
}

// ============================================================================
// 成本管理器
// ============================================================================

/// 成本管理器
pub struct CostManager {
    /// 预算配置
    budget_config: BudgetConfig,
    /// 成本记录
    records: Vec<CostRecord>,
    /// 累计成本
    total_cost: f64,
    /// 每日成本
    daily_costs: HashMap<String, f64>,
}

impl CostManager {
    /// 创建管理器
    pub fn new() -> Self {
        Self {
            budget_config: BudgetConfig {
                total_budget: 10000.0,
                per_episode_budget: 1000.0,
                per_shot_budget: 100.0,
                warning_threshold: 0.8,
                hard_limit_threshold: 0.95,
                enable_auto_pause: true,
            },
            records: vec![],
            total_cost: 0.0,
            daily_costs: HashMap::new(),
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: BudgetConfig) -> Self {
        Self {
            budget_config: config,
            records: vec![],
            total_cost: 0.0,
            daily_costs: HashMap::new(),
        }
    }
    
    /// 记录成本
    pub fn record_cost(&mut self, record: CostRecord) {
        self.total_cost += record.amount;
        
        // 更新每日成本
        let date_key = format!("day_{}", record.recorded_at / 86400);
        *self.daily_costs.entry(date_key).or_insert(0.0) += record.amount;
        
        self.records.push(record);
    }
    
    /// 检查预算状态
    pub fn check_budget(&self) -> BudgetStatus {
        let usage_ratio = self.total_cost / self.budget_config.total_budget;
        
        let status = if usage_ratio >= self.budget_config.hard_limit_threshold as f64 {
            BudgetExceededStatus::Exceeded
        } else if usage_ratio >= self.budget_config.warning_threshold as f64 {
            BudgetExceededStatus::Warning
        } else {
            BudgetExceededStatus::Normal
        };
        
        BudgetStatus {
            total_budget: self.budget_config.total_budget,
            used_budget: self.total_cost,
            remaining_budget: self.budget_config.total_budget - self.total_cost,
            usage_ratio: usage_ratio as f32,
            status,
            should_pause: self.budget_config.enable_auto_pause && 
                          usage_ratio >= self.budget_config.hard_limit_threshold as f64,
        }
    }
    
    /// 估算成本
    pub fn estimate_cost(
        &self,
        shot_count: u32,
        avg_prompt_tokens: u32,
        avg_generation_tokens: u32,
    ) -> CostEstimation {
        let total_tokens = (shot_count as u64) * (avg_prompt_tokens as u64 + avg_generation_tokens as u64);
        let api_calls = shot_count * 2; // 图片生成 + 可能的重试
        let gpu_hours = shot_count as f32 * 0.05; // 假设每个镜头0.05小时
        
        let token_cost = total_tokens as f64 * 0.00002; // 假设每token 0.00002元
        let api_cost = api_calls as f64 * 0.1; // 假设每次API调用0.1元
        let gpu_cost = gpu_hours as f64 * 10.0; // 假设每小时10元
        
        let mut cost_breakdown = HashMap::new();
        cost_breakdown.insert("Token".to_string(), token_cost);
        cost_breakdown.insert("API".to_string(), api_cost);
        cost_breakdown.insert("GPU".to_string(), gpu_cost);
        
        CostEstimation {
            estimated_tokens: total_tokens,
            estimated_api_calls: api_calls,
            estimated_gpu_hours: gpu_hours,
            estimated_total_cost: token_cost + api_cost + gpu_cost,
            cost_breakdown,
        }
    }
    
    /// 生成优化建议
    pub fn generate_optimizations(&self) -> Vec<CostOptimization> {
        let mut optimizations = vec![];
        
        // 检查是否有高成本镜头
        let avg_shot_cost = if !self.records.is_empty() {
            self.total_cost / self.records.len() as f64
        } else {
            0.0
        };
        
        if avg_shot_cost > self.budget_config.per_shot_budget {
            optimizations.push(CostOptimization {
                optimization_type: "降低生成质量".to_string(),
                description: "降低图片生成质量以减少Token消耗".to_string(),
                estimated_savings: avg_shot_cost * 0.3,
                priority: 1,
            });
        }
        
        // 检查是否有重试成本
        let retry_costs: f64 = self.records.iter()
            .filter(|r| r.description.contains("重试"))
            .map(|r| r.amount)
            .sum();
        
        if retry_costs > 0.0 {
            optimizations.push(CostOptimization {
                optimization_type: "减少重试".to_string(),
                description: "优化提示词以减少生成失败重试".to_string(),
                estimated_savings: retry_costs * 0.5,
                priority: 2,
            });
        }
        
        optimizations
    }
    
    /// 获取成本统计
    pub fn statistics(&self) -> CostStats {
        let total_records = self.records.len();
        let avg_cost_per_record = if total_records > 0 {
            self.total_cost / total_records as f64
        } else {
            0.0
        };
        
        let cost_by_type: HashMap<String, f64> = self.records.iter()
            .fold(HashMap::new(), |mut acc, r| {
                let type_name = format!("{:?}", r.cost_type);
                *acc.entry(type_name).or_insert(0.0) += r.amount;
                acc
            });
        
        CostStats {
            total_records,
            total_cost: self.total_cost,
            avg_cost_per_record,
            cost_by_type,
            budget_status: self.check_budget(),
        }
    }
}

/// 预算状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    /// 总预算
    pub total_budget: f64,
    /// 已使用预算
    pub used_budget: f64,
    /// 剩余预算
    pub remaining_budget: f64,
    /// 使用比例
    pub usage_ratio: f32,
    /// 状态
    pub status: BudgetExceededStatus,
    /// 是否应该暂停
    pub should_pause: bool,
}

/// 预算超支状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BudgetExceededStatus {
    /// 正常
    Normal,
    /// 警告
    Warning,
    /// 超支
    Exceeded,
}

/// 成本统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostStats {
    /// 总记录数
    pub total_records: usize,
    /// 总成本
    pub total_cost: f64,
    /// 平均每记录成本
    pub avg_cost_per_record: f64,
    /// 按类型成本
    pub cost_by_type: HashMap<String, f64>,
    /// 预算状态
    pub budget_status: BudgetStatus,
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cost_manager() {
        let mut manager = CostManager::new();
        
        // 记录成本
        manager.record_cost(CostRecord {
            id: "cost_001".to_string(),
            cost_type: CostType::Token,
            amount: 100.0,
            description: "剧本生成".to_string(),
            task_id: None,
            shot_id: None,
            recorded_at: 0,
        });
        
        manager.record_cost(CostRecord {
            id: "cost_002".to_string(),
            cost_type: CostType::ApiCall,
            amount: 50.0,
            description: "图片生成".to_string(),
            task_id: None,
            shot_id: None,
            recorded_at: 0,
        });
        
        // 检查预算
        let status = manager.check_budget();
        assert_eq!(status.used_budget, 150.0);
        assert_eq!(status.status, BudgetExceededStatus::Normal);
        
        // 估算成本
        let estimation = manager.estimate_cost(10, 100, 200);
        assert!(estimation.estimated_total_cost > 0.0);
    }
    
    #[test]
    fn test_budget_warning() {
        let config = BudgetConfig {
            total_budget: 100.0,
            per_episode_budget: 50.0,
            per_shot_budget: 10.0,
            warning_threshold: 0.8,
            hard_limit_threshold: 0.95,
            enable_auto_pause: true,
        };
        
        let mut manager = CostManager::with_config(config);
        
        manager.record_cost(CostRecord {
            id: "cost_001".to_string(),
            cost_type: CostType::Token,
            amount: 85.0,
            description: "大量生成".to_string(),
            task_id: None,
            shot_id: None,
            recorded_at: 0,
        });
        
        let status = manager.check_budget();
        assert_eq!(status.status, BudgetExceededStatus::Warning);
    }
}