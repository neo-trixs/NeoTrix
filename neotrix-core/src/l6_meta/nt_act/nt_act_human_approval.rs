//! Human Approval Workflow — Human Approval 审批工作流
//!
//! 吸收 KB 经验:
//! - GenAI_Agents 的 human-approval 模式
//! - 集成到 SEAL pipeline、goal loop、cleanup 操作
//! - 高风险操作添加人工审批点
//! - 审批状态管理

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Human Approval 审批工作流
pub struct HumanApprovalWorkflow {
    pending_approvals: Vec<ApprovalRequest>,
    completed_approvals: Vec<ApprovalResult>,
    config: ApprovalConfig,
    stats: ApprovalStats,
}

/// 审批配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalConfig {
    pub timeout_seconds: u64,
    pub require_justification: bool,
    pub auto_approve_low_risk: bool,
    pub high_risk_threshold: f64,
}

impl Default for ApprovalConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 300,
            require_justification: true,
            auto_approve_low_risk: false,
            high_risk_threshold: 0.7,
        }
    }
}

/// 审批请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub request_id: String,
    pub operation_type: OperationType,
    pub description: String,
    pub risk_level: RiskLevel,
    pub context: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: ApprovalStatus,
}

/// 操作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    SEALPhase,
    GoalLoop,
    Cleanup,
    CodeChange,
    SystemConfig,
    DataExport,
}

/// 风险等级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// 审批状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Timeout,
}

/// 审批结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResult {
    pub request_id: String,
    pub status: ApprovalStatus,
    pub approver: String,
    pub justification: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 审批统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalStats {
    pub total_requests: u64,
    pub approved: u64,
    pub rejected: u64,
    pub timed_out: u64,
    pub auto_approved: u64,
    pub avg_approval_time: f64,
}

impl HumanApprovalWorkflow {
    /// 创建新的 Human Approval 审批工作流
    pub fn new() -> Self {
        Self {
            pending_approvals: Vec::new(),
            completed_approvals: Vec::new(),
            config: ApprovalConfig::default(),
            stats: ApprovalStats {
                total_requests: 0,
                approved: 0,
                rejected: 0,
                timed_out: 0,
                auto_approved: 0,
                avg_approval_time: 0.0,
            },
        }
    }

    /// 创建审批请求
    pub fn create_request(&mut self, operation_type: OperationType, description: &str, risk_level: RiskLevel) -> ApprovalRequest {
        let request = ApprovalRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            operation_type,
            description: description.to_string(),
            risk_level: risk_level.clone(),
            context: String::new(),
            created_at: chrono::Utc::now(),
            status: ApprovalStatus::Pending,
        };

        // 低风险自动审批
        if self.config.auto_approve_low_risk && risk_level == RiskLevel::Low {
            self.auto_approve(&request.request_id);
        }

        self.pending_approvals.push(request.clone());
        self.stats.total_requests += 1;

        request
    }

    /// 审批
    pub fn approve(&mut self, request_id: &str, approver: &str, justification: Option<String>) -> bool {
        if let Some(pos) = self.pending_approvals.iter().position(|r| r.request_id == request_id) {
            let request = self.pending_approvals.remove(pos);

            let result = ApprovalResult {
                request_id: request.request_id.clone(),
                status: ApprovalStatus::Approved,
                approver: approver.to_string(),
                justification,
                timestamp: chrono::Utc::now(),
            };

            self.completed_approvals.push(result);
            self.stats.approved += 1;

            return true;
        }
        false
    }

    /// 拒绝
    pub fn reject(&mut self, request_id: &str, approver: &str, justification: Option<String>) -> bool {
        if let Some(pos) = self.pending_approvals.iter().position(|r| r.request_id == request_id) {
            let request = self.pending_approvals.remove(pos);

            let result = ApprovalResult {
                request_id: request.request_id.clone(),
                status: ApprovalStatus::Rejected,
                approver: approver.to_string(),
                justification,
                timestamp: chrono::Utc::now(),
            };

            self.completed_approvals.push(result);
            self.stats.rejected += 1;

            return true;
        }
        false
    }

    /// 自动审批
    fn auto_approve(&mut self, request_id: &str) {
        if let Some(request) = self.pending_approvals.iter_mut().find(|r| r.request_id == request_id) {
            request.status = ApprovalStatus::Approved;
            self.stats.auto_approved += 1;
        }
    }

    /// 检查操作是否需要审批
    pub fn requires_approval(&self, operation_type: &OperationType, risk_level: &RiskLevel) -> bool {
        match risk_level {
            RiskLevel::Low => !self.config.auto_approve_low_risk,
            RiskLevel::Medium => true,
            RiskLevel::High => true,
            RiskLevel::Critical => true,
        }
    }

    /// 获取所有待审批请求
    pub fn pending(&self) -> &[ApprovalRequest] {
        &self.pending_approvals
    }

    /// 获取所有已完成审批
    pub fn completed(&self) -> &[ApprovalResult] {
        &self.completed_approvals
    }

    /// 获取统计信息
    pub fn stats(&self) -> &ApprovalStats {
        &self.stats
    }
}
