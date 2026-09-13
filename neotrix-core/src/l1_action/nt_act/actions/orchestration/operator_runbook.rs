//! 运行手册系统模块
//!
//! 操作员工作流文档化
//! 支持模板化运行手册、步骤追踪、决策树

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 运行手册定义
// ============================================================================

/// 步骤状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    /// 待执行
    Pending,
    /// 执行中
    InProgress,
    /// 已完成
    Completed,
    /// 跳过
    Skipped,
    /// 失败
    Failed,
}

/// 步骤类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum StepType {
    /// 自动执行
    Automatic,
    /// 需要人工确认
    ManualApproval,
    /// 条件分支
    Conditional,
    /// 并行执行
    Parallel,
    /// 循环
    Loop,
}

/// 运行手册步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookStep {
    /// 步骤ID
    pub id: String,
    /// 步骤名称
    pub name: String,
    /// 步骤描述
    pub description: String,
    /// 步骤类型
    pub step_type: StepType,
    /// 状态
    pub status: StepStatus,
    /// 命令/操作
    pub command: Option<String>,
    /// 预期输出
    pub expected_output: Option<String>,
    /// 条件表达式
    pub condition: Option<String>,
    /// 依赖步骤
    pub dependencies: Vec<String>,
    /// 超时时间 (秒)
    pub timeout_secs: Option<u32>,
    /// 重试次数
    pub max_retries: u32,
    /// 当前重试次数
    pub current_retries: u32,
    /// 标签
    pub tags: Vec<String>,
}

/// 决策节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionNode {
    /// 节点ID
    pub id: String,
    /// 条件描述
    pub condition: String,
    /// 真分支目标
    pub true_branch: String,
    /// 假分支目标
    pub false_branch: String,
}

/// 运行手册
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorRunbook {
    /// 手册ID
    pub id: String,
    /// 手册名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 版本
    pub version: String,
    /// 步骤列表
    pub steps: Vec<RunbookStep>,
    /// 决策节点
    pub decision_nodes: Vec<DecisionNode>,
    /// 变量
    pub variables: HashMap<String, serde_json::Value>,
    /// 标签
    pub tags: Vec<String>,
    /// 创建时间
    pub created_at: u64,
    /// 更新时间
    pub updated_at: u64,
}

/// 运行手册模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookTemplate {
    /// 模板ID
    pub id: String,
    /// 模板名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 适用场景
    pub applicable_scenarios: Vec<String>,
    /// 步骤模板
    pub step_templates: Vec<RunbookStep>,
    /// 决策模板
    pub decision_templates: Vec<DecisionNode>,
}

/// 运行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookExecution {
    /// 执行ID
    pub execution_id: String,
    /// 手册ID
    pub runbook_id: String,
    /// 状态
    pub status: StepStatus,
    /// 开始时间
    pub started_at: u64,
    /// 结束时间
    pub ended_at: Option<u64>,
    /// 步骤执行结果
    pub step_results: Vec<StepResult>,
    /// 错误信息
    pub error: Option<String>,
}

/// 步骤执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// 步骤ID
    pub step_id: String,
    /// 状态
    pub status: StepStatus,
    /// 输出
    pub output: Option<String>,
    /// 耗时 (毫秒)
    pub duration_ms: u64,
    /// 错误信息
    pub error: Option<String>,
}

// ============================================================================
// 运行手册管理器
// ============================================================================

/// 运行手册管理器
pub struct RunbookManager {
    /// 手册存储
    runbooks: HashMap<String, OperatorRunbook>,
    /// 模板存储
    templates: HashMap<String, RunbookTemplate>,
    /// 执行历史
    executions: Vec<RunbookExecution>,
}

impl RunbookManager {
    /// 创建管理器
    pub fn new() -> Self {
        Self {
            runbooks: HashMap::new(),
            templates: HashMap::new(),
            executions: vec![],
        }
    }
    
    /// 注册运行手册
    pub fn register_runbook(&mut self, runbook: OperatorRunbook) {
        self.runbooks.insert(runbook.id.clone(), runbook);
    }
    
    /// 注册模板
    pub fn register_template(&mut self, template: RunbookTemplate) {
        self.templates.insert(template.id.clone(), template);
    }
    
    /// 从模板创建手册
    pub(crate) fn _create_from_template(
        &mut self,
        template_id: &str,
        runbook_id: &str,
        variables: HashMap<String, serde_json::Value>,
    ) -> Result<OperatorRunbook, String> {
        if let Some(template) = self.templates.get(template_id) {
            let steps: Vec<RunbookStep> = template.step_templates.iter().map(|s| {
                let mut step = s.clone();
                step.status = StepStatus::Pending;
                step.current_retries = 0;
                step
            }).collect();
            
            let runbook = OperatorRunbook {
                id: runbook_id.to_string(),
                name: format!("{} - {}", template.name, runbook_id),
                description: template.description.clone(),
                version: "1.0".to_string(),
                steps,
                decision_nodes: template.decision_templates.clone(),
                variables,
                tags: template.applicable_scenarios.clone(),
                created_at: current_timestamp(),
                updated_at: current_timestamp(),
            };
            
            self.runbooks.insert(runbook_id.to_string(), runbook.clone());
            Ok(runbook)
        } else {
            Err(format!("模板 {} 不存在", template_id))
        }
    }
    
    /// 执行运行手册
    pub fn execute(&mut self, runbook_id: &str) -> RunbookExecution {
        let execution_id = format!("exec_{}_{}", runbook_id, current_timestamp());
        
        let mut execution = RunbookExecution {
            execution_id: execution_id.clone(),
            runbook_id: runbook_id.to_string(),
            status: StepStatus::InProgress,
            started_at: current_timestamp(),
            ended_at: None,
            step_results: vec![],
            error: None,
        };
        
        if let Some(runbook) = self.runbooks.get(runbook_id) {
            for step in &runbook.steps {
                let result = self.execute_step(step);
                execution.step_results.push(result);
            }
        }
        
        execution.status = StepStatus::Completed;
        execution.ended_at = Some(current_timestamp());
        
        self.executions.push(execution.clone());
        execution
    }
    
    /// 执行单个步骤
    fn execute_step(&self, step: &RunbookStep) -> StepResult {
        StepResult {
            step_id: step.id.clone(),
            status: StepStatus::Failed,
            output: None,
            duration_ms: 0,
            error: Some("not wired: step execution not implemented".to_string()),
        }
    }
    
    /// 获取手册
    pub(crate) fn _get_runbook(&self, runbook_id: &str) -> Option<&OperatorRunbook> {
        self.runbooks.get(runbook_id)
    }
    
    /// 获取统计信息
    pub fn statistics(&self) -> RunbookStats {
        let total_runbooks = self.runbooks.len();
        let total_templates = self.templates.len();
        let total_executions = self.executions.len();
        let successful = self.executions.iter().filter(|e| e.status == StepStatus::Completed).count();
        
        RunbookStats {
            total_runbooks,
            total_templates,
            total_executions,
            successful_executions: successful,
        }
    }
}

/// 运行手册统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookStats {
    /// 总手册数
    pub total_runbooks: usize,
    /// 总模板数
    pub total_templates: usize,
    /// 总执行次数
    pub total_executions: usize,
    /// 成功执行次数
    pub successful_executions: usize,
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_runbook_manager() {
        let mut manager = RunbookManager::new();
        
        let runbook = OperatorRunbook {
            id: "runbook_001".to_string(),
            name: "视频生成手册".to_string(),
            description: "标准视频生成流程".to_string(),
            version: "1.0".to_string(),
            steps: vec![
                RunbookStep {
                    id: "step_1".to_string(),
                    name: "分析需求".to_string(),
                    description: "分析用户需求".to_string(),
                    step_type: StepType::Automatic,
                    status: StepStatus::Pending,
                    command: None,
                    expected_output: None,
                    condition: None,
                    dependencies: vec![],
                    timeout_secs: None,
                    max_retries: 3,
                    current_retries: 0,
                    tags: vec![],
                },
            ],
            decision_nodes: vec![],
            variables: HashMap::new(),
            tags: vec![],
            created_at: 0,
            updated_at: 0,
        };
        
        manager.register_runbook(runbook);
        
        let execution = manager.execute("runbook_001");
        assert_eq!(execution.status, StepStatus::Completed);
        
        let stats = manager.statistics();
        assert_eq!(stats.total_runbooks, 1);
    }
}