//! Verified Pivoting — 吸收自 Argus 4-Role Architecture
//!
//! Manager/Planner/Engineer/Reviewer 四角色分离：
//! - Manager: 提交总体目标，确定阶段，决定推进/回滚
//! - Planner: 分解为有依赖的有界任务
//! - Engineer: 执行、修改、自审
//! - Reviewer: 独立检查，发出裁定
//!
//! 验证转向: 当证据矛盾时，Manager 承认修订，保持目标忠实。

use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// 角色类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Manager,
    Planner,
    Engineer,
    Reviewer,
}

/// 任务阶段
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CampaignStage {
    Research,
    Plan,
    Execute,
    Review,
    Submit,
}

/// 工作契约 — 随知识增长而变化的运行时细节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingContract {
    pub stable_intent: String,        // 稳定用户意图
    pub operational_objective: String, // 可修订的操作目标
    pub constraints: Vec<String>,
    pub verification_criteria: Vec<String>,
    pub stage: CampaignStage,
}

/// 验证裁定
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Done,
    Continue,
    Blocked,
    Revise, // 触发验证转向
}

/// 角色动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAction {
    pub role: Role,
    pub action_type: String,
    pub content: String,
    pub evidence: Option<String>,
    pub timestamp: u64,
}

/// 验证转向器
pub struct VerifiedPivoting {
    contract: WorkingContract,
    action_log: Vec<RoleAction>,
    revision_count: u32,
    max_revisions: u32,
}

impl VerifiedPivoting {
    pub fn new(intent: String) -> Self {
        Self {
            contract: WorkingContract {
                stable_intent: intent,
                operational_objective: String::new(),
                constraints: Vec::new(),
                verification_criteria: Vec::new(),
                stage: CampaignStage::Research,
            },
            action_log: Vec::new(),
            revision_count: 0,
            max_revisions: 5,
        }
    }

    /// Manager: 提交目标
    pub fn manager_commit(&mut self, objective: String, constraints: Vec<String>) {
        self.contract.operational_objective = objective;
        self.contract.constraints = constraints;
        self.log_action(Role::Manager, "commit".to_string(), 
            format!("Committed objective: {}", self.contract.operational_objective));
    }

    /// Planner: 分解任务
    pub fn planner_decompose(&mut self, tasks: Vec<String>) {
        self.contract.stage = CampaignStage::Plan;
        self.log_action(Role::Planner, "decompose".to_string(),
            format!("Decomposed into {} tasks", tasks.len()));
    }

    /// Engineer: 执行并自审
    pub fn engineer_execute(&mut self, result: String, self_review: String) {
        self.contract.stage = CampaignStage::Execute;
        self.log_action(Role::Engineer, "execute".to_string(),
            format!("Result: {}, Self-review: {}", result, self_review));
    }

    /// Reviewer: 发出裁定
    pub fn reviewer_inspect(&mut self, artifact: &str) -> Verdict {
        self.contract.stage = CampaignStage::Review;
        
        // 简单验证逻辑: 如果 artifact 包含 "error" 或 "fail"，标记为需要修订
        let verdict = if artifact.contains("error") || artifact.contains("fail") {
            Verdict::Revise
        } else if artifact.contains("done") || artifact.contains("complete") {
            Verdict::Done
        } else {
            Verdict::Continue
        };

        self.log_action(Role::Reviewer, "inspect".to_string(),
            format!("Verdict: {:?}", verdict));

        verdict
    }

    /// 验证转向: Manager 承认修订
    pub fn verified_pivot(&mut self, new_objective: String, evidence: String) -> bool {
        if self.revision_count >= self.max_revisions {
            self.log_action(Role::Manager, "reject_pivot".to_string(),
                "Max revisions reached".to_string());
            return false;
        }

        self.contract.operational_objective = new_objective;
        self.revision_count += 1;
        self.log_action(Role::Manager, "pivot".to_string(),
            format!("Revised objective (revision {}): {} | Evidence: {}", 
                self.revision_count, self.contract.operational_objective, evidence));

        true
    }

    /// 获取当前契约
    pub fn contract(&self) -> &WorkingContract {
        &self.contract
    }

    /// 获取修订次数
    pub fn revision_count(&self) -> u32 {
        self.revision_count
    }

    /// 获取动作日志
    pub fn action_log(&self) -> &[RoleAction] {
        &self.action_log
    }

    fn log_action(&mut self, role: Role, action_type: String, content: String) {
        self.action_log.push(RoleAction {
            role,
            action_type,
            content,
            evidence: None,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verified_pivoting_basic() {
        let mut vp = VerifiedPivoting::new("Build a web app".to_string());
        
        vp.manager_commit("Build login page".to_string(), vec!["Use React".to_string()]);
        assert_eq!(vp.contract().stage, CampaignStage::Research);
        
        vp.planner_decompose(vec!["Design UI".to_string(), "Implement auth".to_string()]);
        assert_eq!(vp.contract().stage, CampaignStage::Plan);
        
        vp.engineer_execute("Login page built".to_string(), "Looks good".to_string());
        assert_eq!(vp.contract().stage, CampaignStage::Execute);
        
        let verdict = vp.reviewer_inspect("Login page complete and tested");
        assert_eq!(verdict, Verdict::Done);
    }

    #[test]
    fn test_verified_pivot() {
        let mut vp = VerifiedPivoting::new("Build API".to_string());
        vp.manager_commit("REST API".to_string(), vec![]);
        
        // Reviewer finds error
        let verdict = vp.reviewer_inspect("Error: endpoint returns 500");
        assert_eq!(verdict, Verdict::Revise);
        
        // Manager pivots
        let pivoted = vp.verified_pivot("GraphQL API".to_string(), "REST has performance issues".to_string());
        assert!(pivoted);
        assert_eq!(vp.contract().operational_objective, "GraphQL API");
        assert_eq!(vp.revision_count(), 1);
    }

    #[test]
    fn test_max_revisions() {
        let mut vp = VerifiedPivoting::new("Test".to_string());
        for _ in 0..5 {
            vp.verified_pivot("new objective".to_string(), "evidence".to_string());
        }
        let pivoted = vp.verified_pivot("final".to_string(), "evidence".to_string());
        assert!(!pivoted); // Should fail after max revisions
    }
}
