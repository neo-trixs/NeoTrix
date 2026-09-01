use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Human-in-the-Loop Approval — 人类审批工作流
///
/// 参考: GenAI_Agents "Human-in-the-Loop Approval"
/// 核心思想: 在关键操作前请求人类审批，支持
/// 审批策略、超时处理和审计追踪。

/// 审批请求
#[derive(Debug, Clone)]
pub struct ApprovalRequest {
    pub id: String,
    pub action: String,
    pub description: String,
    pub risk_level: RiskLevel,
    pub requested_at: Instant,
    pub expires_at: Instant,
    pub context: HashMap<String, String>,
    pub status: ApprovalStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,      // 自动批准
    Medium,   // 需要审批
    High,     // 必须审批
    Critical, // 多人审批
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
    Cancelled,
}

/// 审批决策
#[derive(Debug, Clone)]
pub struct ApprovalDecision {
    pub request_id: String,
    pub approver: String,
    pub decision: ApprovalStatus,
    pub reason: Option<String>,
    pub decided_at: Instant,
}

/// 审批策略
#[derive(Debug, Clone)]
pub struct ApprovalPolicy {
    /// 自动批准的风险级别
    pub auto_approve_below: RiskLevel,
    /// 审批超时时间
    pub timeout: Duration,
    /// 所需审批人数 (Critical 级别)
    pub required_approvers: usize,
    /// 可用审批人列表
    pub available_approvers: Vec<String>,
}

impl Default for ApprovalPolicy {
    fn default() -> Self {
        Self {
            auto_approve_below: RiskLevel::Low,
            timeout: Duration::from_secs(300), // 5 分钟
            required_approvers: 1,
            available_approvers: vec!["admin".to_string()],
        }
    }
}

/// 审批管理器
pub struct ApprovalManager {
    policy: ApprovalPolicy,
    pending_requests: HashMap<String, ApprovalRequest>,
    decisions: Vec<ApprovalDecision>,
    completed_requests: Vec<ApprovalRequest>,
}

impl ApprovalManager {
    pub fn new(policy: ApprovalPolicy) -> Self {
        Self {
            policy,
            pending_requests: HashMap::new(),
            decisions: Vec::new(),
            completed_requests: Vec::new(),
        }
    }

    /// 从 ConfirmationType 创建审批请求 (桥接 ConfirmationGate 协议)
    ///
    /// 将 ConfirmationGate 的浏览器/敏感操作确认请求转换为统一的 ApprovalRequest。
    /// 参考: browser-act/skills Confirmation Gate 协议
    pub fn from_confirmation_type(
        confirmation_type: &str,
        description: &str,
        details: HashMap<String, String>,
    ) -> ApprovalRequest {
        let risk_level = match confirmation_type {
            "browser-creation" | "browser-deletion" => RiskLevel::High,
            "login" | "form-submission" | "file-upload" => RiskLevel::High,
            "sensitive-navigation" => RiskLevel::Medium,
            "profile-switch" => RiskLevel::Low,
            _ => RiskLevel::Medium,
        };

        let timeout = match confirmation_type {
            "login" => Duration::from_secs(120),
            "browser-creation" | "browser-deletion" => Duration::from_secs(60),
            "form-submission" => Duration::from_secs(30),
            _ => Duration::from_secs(60),
        };

        let now = Instant::now();
        ApprovalRequest {
            id: format!("conf_{}", now.duration_since(Instant::now()).as_millis()),
            action: confirmation_type.to_string(),
            description: description.to_string(),
            risk_level,
            requested_at: now,
            expires_at: now + timeout,
            context: details,
            status: ApprovalStatus::Pending,
        }
    }

    /// 提交审批请求
    pub fn submit_request(&mut self, request: ApprovalRequest) -> Result<String, String> {
        // 检查是否自动批准
        if self.should_auto_approve(&request) {
            let approved_request = ApprovalRequest {
                status: ApprovalStatus::Approved,
                ..request.clone()
            };
            self.completed_requests.push(approved_request);
            return Ok(request.id);
        }

        let id = request.id.clone();
        self.pending_requests.insert(id.clone(), request);
        Ok(id)
    }

    /// 做出审批决策
    pub fn make_decision(&mut self, decision: ApprovalDecision) -> Result<(), String> {
        let request = self.pending_requests.get_mut(&decision.request_id)
            .ok_or("Request not found")?;

        // 检查审批人权限
        if !self.policy.available_approvers.contains(&decision.approver) {
            return Err("Unauthorized approver".to_string());
        }

        // 记录决策
        self.decisions.push(decision.clone());

        // 更新请求状态
        request.status = decision.decision.clone();

        // 如果批准或拒绝，移动到完成列表
        if decision.decision == ApprovalStatus::Approved || decision.decision == ApprovalStatus::Rejected {
            let completed = self.pending_requests.remove(&decision.request_id).unwrap();
            self.completed_requests.push(completed);
        }

        Ok(())
    }

    /// 检查超时
    pub fn check_timeouts(&mut self) -> Vec<String> {
        let now = Instant::now();
        let mut timed_out = Vec::new();

        let expired_ids: Vec<String> = self.pending_requests.iter()
            .filter(|(_, req)| now > req.expires_at)
            .map(|(id, _)| id.clone())
            .collect();

        for id in expired_ids {
            if let Some(mut request) = self.pending_requests.remove(&id) {
                request.status = ApprovalStatus::Expired;
                self.completed_requests.push(request);
                timed_out.push(id);
            }
        }

        timed_out
    }

    /// 获取待审批请求
    pub fn get_pending(&self) -> Vec<&ApprovalRequest> {
        self.pending_requests.values().collect()
    }

    /// 获取已完成请求
    pub fn get_completed(&self) -> &[ApprovalRequest] {
        &self.completed_requests
    }

    /// 获取审批历史
    pub fn get_decisions(&self) -> &[ApprovalDecision] {
        &self.decisions
    }

    /// 检查是否应该自动批准
    fn should_auto_approve(&self, request: &ApprovalRequest) -> bool {
        match request.risk_level {
            RiskLevel::Low => true,
            RiskLevel::Medium => matches!(self.policy.auto_approve_below, RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical),
            _ => false,
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> ApprovalStats {
        let approved = self.completed_requests.iter().filter(|r| r.status == ApprovalStatus::Approved).count();
        let rejected = self.completed_requests.iter().filter(|r| r.status == ApprovalStatus::Rejected).count();
        let expired = self.completed_requests.iter().filter(|r| r.status == ApprovalStatus::Expired).count();

        ApprovalStats {
            pending: self.pending_requests.len(),
            approved,
            rejected,
            expired,
            total: self.completed_requests.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApprovalStats {
    pub pending: usize,
    pub approved: usize,
    pub rejected: usize,
    pub expired: usize,
    pub total: usize,
}

/// 审批请求构建器
pub struct ApprovalRequestBuilder {
    id: String,
    action: String,
    description: String,
    risk_level: RiskLevel,
    context: HashMap<String, String>,
    timeout: Duration,
}

impl ApprovalRequestBuilder {
    pub fn new(id: &str, action: &str) -> Self {
        Self {
            id: id.to_string(),
            action: action.to_string(),
            description: String::new(),
            risk_level: RiskLevel::Medium,
            context: HashMap::new(),
            timeout: Duration::from_secs(300),
        }
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn risk_level(mut self, level: RiskLevel) -> Self {
        self.risk_level = level;
        self
    }

    pub fn context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> ApprovalRequest {
        let now = Instant::now();
        ApprovalRequest {
            id: self.id,
            action: self.action,
            description: self.description,
            risk_level: self.risk_level,
            requested_at: now,
            expires_at: now + self.timeout,
            context: self.context,
            status: ApprovalStatus::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_approve_low_risk() {
        let mut manager = ApprovalManager::new(ApprovalPolicy::default());
        let request = ApprovalRequestBuilder::new("req_001", "read_file")
            .risk_level(RiskLevel::Low)
            .build();

        let id = manager.submit_request(request).unwrap();
        assert_eq!(id, "req_001");
        assert!(manager.get_pending().is_empty());
    }

    #[test]
    fn test_manual_approval() {
        let mut manager = ApprovalManager::new(ApprovalPolicy::default());
        let request = ApprovalRequestBuilder::new("req_002", "delete_file")
            .risk_level(RiskLevel::High)
            .build();

        let id = manager.submit_request(request).unwrap();
        assert_eq!(manager.get_pending().len(), 1);

        let decision = ApprovalDecision {
            request_id: id,
            approver: "admin".to_string(),
            decision: ApprovalStatus::Approved,
            reason: Some("Authorized".to_string()),
            decided_at: Instant::now(),
        };

        manager.make_decision(decision).unwrap();
        assert!(manager.get_pending().is_empty());
        assert_eq!(manager.get_completed().len(), 1);
    }

    #[test]
    fn test_rejection() {
        let mut manager = ApprovalManager::new(ApprovalPolicy::default());
        let request = ApprovalRequestBuilder::new("req_003", "execute_code")
            .risk_level(RiskLevel::Critical)
            .build();

        let id = manager.submit_request(request).unwrap();

        let decision = ApprovalDecision {
            request_id: id,
            approver: "admin".to_string(),
            decision: ApprovalStatus::Rejected,
            reason: Some("Too risky".to_string()),
            decided_at: Instant::now(),
        };

        manager.make_decision(decision).unwrap();
        assert!(manager.get_pending().is_empty());
        assert_eq!(manager.get_completed().len(), 1);
    }

    #[test]
    fn test_timeout() {
        let policy = ApprovalPolicy {
            timeout: Duration::from_millis(1),
            ..Default::default()
        };
        let mut manager = ApprovalManager::new(policy);

        let request = ApprovalRequestBuilder::new("req_004", "write_file")
            .risk_level(RiskLevel::Medium)
            .timeout(Duration::from_millis(1))
            .build();

        manager.submit_request(request).unwrap();
        std::thread::sleep(Duration::from_millis(10));

        let timed_out = manager.check_timeouts();
        assert_eq!(timed_out.len(), 1);
        assert_eq!(timed_out[0], "req_004");
    }

    #[test]
    fn test_unauthorized_approver() {
        let mut manager = ApprovalManager::new(ApprovalPolicy::default());
        let request = ApprovalRequestBuilder::new("req_005", "modify_config")
            .risk_level(RiskLevel::High)
            .build();

        let id = manager.submit_request(request).unwrap();

        let decision = ApprovalDecision {
            request_id: id,
            approver: "unauthorized_user".to_string(),
            decision: ApprovalStatus::Approved,
            reason: None,
            decided_at: Instant::now(),
        };

        assert!(manager.make_decision(decision).is_err());
    }

    #[test]
    fn test_stats() {
        let mut manager = ApprovalManager::new(ApprovalPolicy::default());

        // 自动批准
        let req1 = ApprovalRequestBuilder::new("req1", "action1").risk_level(RiskLevel::Low).build();
        manager.submit_request(req1).unwrap();

        // 手动批准
        let req2 = ApprovalRequestBuilder::new("req2", "action2").risk_level(RiskLevel::High).build();
        manager.submit_request(req2).unwrap();
        manager.make_decision(ApprovalDecision {
            request_id: "req2".to_string(),
            approver: "admin".to_string(),
            decision: ApprovalStatus::Approved,
            reason: None,
            decided_at: Instant::now(),
        }).unwrap();

        let stats = manager.stats();
        assert_eq!(stats.approved, 2);
        assert_eq!(stats.pending, 0);
    }
}
