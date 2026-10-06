#![forbid(unsafe_code)]

//! D09: Self-Referential Code (L8 Autonomic) — 自我修改代码数据模型
//!
//! 记录、审批、回滚所有自我代码变更。

use std::collections::{HashMap, VecDeque};

/// 变体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationType {
    AddCode,
    RemoveCode,
    Replace,
    Refactor,
}

/// 变体风险等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MutationRisk {
    Low,
    Medium,
    High,
    Critical,
}

/// 代码变体
#[derive(Debug, Clone)]
pub struct CodeMutation {
    pub id: String,
    pub mutation_type: MutationType,
    pub target_path: String,
    pub target_pattern: String,
    pub replacement: String,
    pub risk: MutationRisk,
    pub impact_estimate: f64,
}

/// 回滚计划
#[derive(Debug, Clone)]
pub struct RollbackPlan {
    pub mutation_id: String,
    pub original_content: String,
    pub backup_path: String,
    pub rollback_steps: Vec<String>,
}

/// 变体请求
#[derive(Debug, Clone)]
pub struct MutationRequest {
    pub mutation: CodeMutation,
    pub reason: String,
    pub requires_review: bool,
    pub submitted_at: u64,
}

/// 变体结果
#[derive(Debug, Clone)]
pub struct MutationResult {
    pub mutation_id: String,
    pub success: bool,
    pub error: Option<String>,
    pub applied_at: u64,
    pub verification_score: f64,
}

/// 自引用统计
#[derive(Debug, Clone)]
pub struct SelfRefStats {
    pub total_mutations: u32,
    pub success_rate: f64,
    pub pending_count: u32,
    pub risk_distribution: HashMap<MutationRisk, u32>,
}

/// 自我代码监控器
#[derive(Debug, Clone)]
pub struct SelfCodeMonitor {
    active_mutations: HashMap<String, MutationResult>,
    rollback_plans: HashMap<String, RollbackPlan>,
    pending_requests: VecDeque<MutationRequest>,
    max_history: usize,
}

impl SelfCodeMonitor {
    pub fn new() -> Self {
        Self {
            active_mutations: HashMap::new(),
            rollback_plans: HashMap::new(),
            pending_requests: VecDeque::new(),
            max_history: 100,
        }
    }

    pub fn submit_request(&mut self, request: MutationRequest) {
        self.pending_requests.push_back(request);
    }

    /// 自动审批：**`requires_review` 的请求一律拒绝**。
    ///
    /// ⚠️ 首版无条件 `remove` 后直接返回，从不读 `requires_review`
    /// ⇒ 该字段是纯装饰，**自改代码的人工复核闸门实际不存在**
    /// （由 dead-config-flag 门与人工审计共同查出）。现在它真的生效。
    pub fn approve_request(&mut self, id: &str) -> Option<CodeMutation> {
        let pos = self
            .pending_requests
            .iter()
            .position(|r| r.mutation.id == id)?;
        if self.pending_requests[pos].requires_review {
            log::warn!(
                "[self_ref_code] 拒绝自动审批：mutation={} 需人工复核",
                id
            );
            return None;
        }
        self.pending_requests.remove(pos).map(|r| r.mutation)
    }

    /// 人工复核**通过后**的显式放行路径。
    ///
    /// 与 [`SelfRefCodeMonitor::approve_request`] 分开而不是加布尔参数：
    /// 让「谁批准的」在调用点即可见，避免再次退化成「参数传了就放行」。
    pub fn approve_request_after_review(&mut self, id: &str) -> Option<CodeMutation> {
        let pos = self
            .pending_requests
            .iter()
            .position(|r| r.mutation.id == id)?;
        self.pending_requests.remove(pos).map(|r| r.mutation)
    }

    pub fn record_result(&mut self, result: MutationResult) {
        self.active_mutations
            .insert(result.mutation_id.clone(), result);
        if self.active_mutations.len() > self.max_history {
            if let Some(oldest) = self
                .active_mutations
                .keys()
                .cloned()
                .min_by_key(|k| self.active_mutations[k].applied_at)
            {
                self.active_mutations.remove(&oldest);
            }
        }
    }

    pub fn store_rollback(&mut self, plan: RollbackPlan) {
        self.rollback_plans.insert(plan.mutation_id.clone(), plan);
    }

    pub fn execute_rollback(&mut self, mutation_id: &str) -> Option<RollbackPlan> {
        self.rollback_plans.remove(mutation_id)
    }

    pub fn get_status(&self, mutation_id: &str) -> Option<&MutationResult> {
        self.active_mutations.get(mutation_id)
    }

    pub fn stats(&self) -> SelfRefStats {
        let total_mutations = self.active_mutations.len() as u32;
        let successful = self.active_mutations.values().filter(|r| r.success).count() as u32;
        let success_rate = if total_mutations == 0 {
            0.0
        } else {
            (successful as f64 / total_mutations as f64)
                .max(0.0)
                .min(1.0)
        };
        let pending_count = self.pending_requests.len() as u32;

        let mut risk_distribution: HashMap<MutationRisk, u32> = HashMap::new();
        risk_distribution.insert(MutationRisk::Low, 0);
        risk_distribution.insert(MutationRisk::Medium, 0);
        risk_distribution.insert(MutationRisk::High, 0);
        risk_distribution.insert(MutationRisk::Critical, 0);

        SelfRefStats {
            total_mutations,
            success_rate,
            pending_count,
            risk_distribution,
        }
    }
}

impl Default for SelfCodeMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mutation(id: &str, risk: MutationRisk) -> CodeMutation {
        CodeMutation {
            id: id.to_string(),
            mutation_type: MutationType::Replace,
            target_path: "src/core/nt_core_e8.rs".into(),
            target_pattern: "old_pattern".into(),
            replacement: "new_pattern".into(),
            risk,
            impact_estimate: 0.3,
        }
    }

    #[test]
    /// `requires_review: false` ⇒ 自动审批放行（闸门不误伤正常路径）。
    #[test]
    fn test_submit_and_approve_no_review_needed() {
        let mut monitor = SelfCodeMonitor::new();
        let req = MutationRequest {
            mutation: sample_mutation("MUT-001", MutationRisk::Low),
            reason: "Cosmetic rename".into(),
            requires_review: false,
            submitted_at: 20260701,
        };
        monitor.submit_request(req);
        assert_eq!(monitor.pending_requests.len(), 1);

        let approved = monitor.approve_request("MUT-001");
        assert!(approved.is_some());
        assert_eq!(approved.unwrap().id, "MUT-001");
        assert!(monitor.pending_requests.is_empty());
    }

    /// ⚠️ 闸门回归测试：`requires_review: true` 不得被自动放行。
    ///
    /// 首版的 `test_submit_and_approve` 用 `requires_review: true`
    /// 却断言 `approve_request` 返回 `Some` —— **该测试固化了缺陷行为**，
    /// 是这个闸门长期不存在却全绿的直接原因。
    #[test]
    fn test_requires_review_blocks_auto_approval() {
        let mut monitor = SelfCodeMonitor::new();
        monitor.submit_request(MutationRequest {
            mutation: sample_mutation("MUT-002", MutationRisk::High),
            reason: "Rewrite core state machine".into(),
            requires_review: true,
            submitted_at: 20260701,
        });

        assert!(
            monitor.approve_request("MUT-002").is_none(),
            "需人工复核的请求不得自动放行"
        );
        assert_eq!(
            monitor.pending_requests.len(),
            1,
            "被拒后请求应仍在pending 队列里（不得被静默丢弃）"
        );

        // 人工复核通过后的显式放行路径可用
        let approved = monitor.approve_request_after_review("MUT-002");
        assert!(approved.is_some());
        assert!(monitor.pending_requests.is_empty());
    }

    #[test]
    fn test_record_result_tracks_success() {
        let mut monitor = SelfCodeMonitor::new();
        let result = MutationResult {
            mutation_id: "MUT-002".into(),
            success: true,
            error: None,
            applied_at: 20260702,
            verification_score: 0.95,
        };
        monitor.record_result(result);
        let status = monitor.get_status("MUT-002");
        assert!(status.is_some());
        assert!(status.unwrap().success);
    }

    #[test]
    fn test_rollback_store_and_execute() {
        let mut monitor = SelfCodeMonitor::new();
        let plan = RollbackPlan {
            mutation_id: "MUT-003".into(),
            original_content: "fn old() {}".into(),
            backup_path: "/tmp/backup_mut_003.rs".into(),
            rollback_steps: vec!["restore from backup".into()],
        };
        monitor.store_rollback(plan);
        assert_eq!(monitor.rollback_plans.len(), 1);

        let executed = monitor.execute_rollback("MUT-003");
        assert!(executed.is_some());
        assert_eq!(executed.unwrap().backup_path, "/tmp/backup_mut_003.rs");
        assert!(monitor.rollback_plans.is_empty());
    }

    #[test]
    fn test_stats_success_rate() {
        let mut monitor = SelfCodeMonitor::new();
        for i in 0..5 {
            monitor.record_result(MutationResult {
                mutation_id: format!("MUT-{:03}", i),
                success: i < 3,
                error: if i >= 3 { Some("failure".into()) } else { None },
                applied_at: 20260700 + i,
                verification_score: 0.8,
            });
        }
        let stats = monitor.stats();
        assert_eq!(stats.total_mutations, 5);
        assert!((stats.success_rate - 0.6).abs() < 1e-10);
        assert_eq!(stats.pending_count, 0);
    }

    #[test]
    fn test_approve_nonexistent_returns_none() {
        let mut monitor = SelfCodeMonitor::new();
        assert!(monitor.approve_request("DOES_NOT_EXIST").is_none());
    }

    #[test]
    fn test_record_result_caps_history() {
        let mut monitor = SelfCodeMonitor::new();
        monitor.max_history = 2;
        for i in 0..5 {
            monitor.record_result(MutationResult {
                mutation_id: format!("MUT-{:03}", i),
                success: true,
                error: None,
                applied_at: i,
                verification_score: 1.0,
            });
        }
        assert_eq!(monitor.active_mutations.len(), 2);
    }
}
