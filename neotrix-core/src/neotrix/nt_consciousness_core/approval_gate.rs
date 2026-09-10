#![forbid(unsafe_code)]

//! 三级审批门 (Three-Level Approval Gate)
//!
//! 根据风险评分决定审批级别：
//! - Full-Auto (< 30): 自动应用所有变更，记录审计日志
//! - Suggest (30-60): 推荐变更，需要显式批准
//! - Human Review (> 60): 人工审批，阻止自动执行
//!
//! 每个变更请求经过评分后路由到对应级别，审批结果写入审计日志。

use std::fmt;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// Public Types
// ============================================================================

/// 审批级别
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ApprovalLevel {
    /// 自动应用 (< 30)
    FullAuto,
    /// 推荐变更，需批准 (30-60)
    Suggest,
    /// 人工审批 (> 60)
    HumanReview,
}

impl fmt::Display for ApprovalLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApprovalLevel::FullAuto => write!(f, "Full-Auto"),
            ApprovalLevel::Suggest => write!(f, "Suggest"),
            ApprovalLevel::HumanReview => write!(f, "Human Review"),
        }
    }
}

/// 变更类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeType {
    /// 文件写入
    FileWrite,
    /// 文件删除
    FileDelete,
    /// 代码修改
    CodeModify,
    /// 配置变更
    ConfigChange,
    /// 网络请求
    NetworkRequest,
    /// 系统调用
    SystemCall,
    /// 数据库操作
    DatabaseOp,
    /// 自定义
    Custom(String),
}

/// 变更请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRequest {
    /// 请求 ID
    pub id: Uuid,
    /// 变更类型
    pub change_type: ChangeType,
    /// 变更描述
    pub description: String,
    /// 变更目标（文件路径、URL 等）
    pub target: String,
    /// 变更前内容（可选）
    pub before: Option<String>,
    /// 变更后内容
    pub after: String,
    /// 风险评分 (0-100)
    pub risk_score: u32,
    /// 请求者
    pub requester: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl ChangeRequest {
    pub fn new(
        change_type: ChangeType,
        description: impl Into<String>,
        target: impl Into<String>,
        after: impl Into<String>,
        risk_score: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            change_type,
            description: description.into(),
            target: target.into(),
            before: None,
            after: after.into(),
            risk_score,
            requester: "system".into(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

/// 审批决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalDecision {
    /// 决策 ID
    pub id: Uuid,
    /// 关联的变更请求 ID
    pub request_id: Uuid,
    /// 审批级别
    pub level: ApprovalLevel,
    /// 是否批准
    pub approved: bool,
    /// 审批人（Auto / Human / System）
    pub approver: String,
    /// 审批意见
    pub comment: Option<String>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 处理耗时（毫秒）
    pub duration_ms: u64,
}

/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// 日志 ID
    pub id: Uuid,
    /// 变更请求
    pub request: ChangeRequest,
    /// 审批决策
    pub decision: ApprovalDecision,
    /// 实际执行结果
    pub execution_result: Option<ExecutionResult>,
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// 是否成功
    pub success: bool,
    /// 输出信息
    pub output: String,
    /// 执行耗时（毫秒）
    pub duration_ms: u64,
    /// 回滚可用
    pub rollback_available: bool,
}

/// 审批门统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GateStats {
    /// 总请求数
    pub total_requests: u64,
    /// 各级别请求数
    pub by_level: HashMap<ApprovalLevel, u64>,
    /// 批准数
    pub approved: u64,
    /// 拒绝数
    pub rejected: u64,
    /// 平均风险评分
    pub avg_risk_score: f64,
}

// ============================================================================
// RiskScorer
// ============================================================================

/// 风险评分器
pub struct RiskScorer {
    /// 变更类型基础风险
    pub type_base_risk: HashMap<ChangeType, u32>,
    /// 关键词风险加成
    pub keyword_risks: Vec<KeywordRisk>,
}

/// 关键词风险
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordRisk {
    pub keyword: String,
    pub risk_add: u32,
    pub description: String,
}

impl RiskScorer {
    pub fn new() -> Self {
        let mut type_base_risk = HashMap::new();
        type_base_risk.insert(ChangeType::FileWrite, 20);
        type_base_risk.insert(ChangeType::FileDelete, 50);
        type_base_risk.insert(ChangeType::CodeModify, 35);
        type_base_risk.insert(ChangeType::ConfigChange, 25);
        type_base_risk.insert(ChangeType::NetworkRequest, 30);
        type_base_risk.insert(ChangeType::SystemCall, 60);
        type_base_risk.insert(ChangeType::DatabaseOp, 40);

        Self {
            type_base_risk,
            keyword_risks: vec![
                KeywordRisk { keyword: "rm -rf".into(), risk_add: 40, description: "递归删除".into() },
                KeywordRisk { keyword: "sudo".into(), risk_add: 35, description: "提权操作".into() },
                KeywordRisk { keyword: "chmod 777".into(), risk_add: 30, description: "权限开放".into() },
                KeywordRisk { keyword: "DROP TABLE".into(), risk_add: 45, description: "删表操作".into() },
                KeywordRisk { keyword: "DELETE FROM".into(), risk_add: 35, description: "批量删除".into() },
            ],
        }
    }

    /// 计算风险评分
    pub fn score(&self, request: &ChangeRequest) -> u32 {
        let base = self.type_base_risk.get(&request.change_type).copied().unwrap_or(25);

        let keyword_add: u32 = self.keyword_risks.iter()
            .filter(|kr| {
                request.description.to_uppercase().contains(&kr.keyword.to_uppercase())
                    || request.after.to_uppercase().contains(&kr.keyword.to_uppercase())
            })
            .map(|kr| kr.risk_add)
            .sum();

        (base + keyword_add).min(100)
    }
}

impl Default for RiskScorer {
    fn default() -> Self { Self::new() }
}

// ============================================================================
// ApprovalGate
// ============================================================================

/// 三级审批门
pub struct ApprovalGate {
    /// 风险评分器
    pub scorer: RiskScorer,
    /// 审计日志
    pub audit_log: Vec<AuditEntry>,
    /// 统计
    pub stats: GateStats,
    /// Full-Auto 阈值上限
    pub full_auto_threshold: u32,
    /// Suggest 阈值上限
    pub suggest_threshold: u32,
    /// 待审批队列
    pub pending: Vec<ChangeRequest>,
}

impl ApprovalGate {
    /// 创建默认审批门
    pub fn new() -> Self {
        Self {
            scorer: RiskScorer::new(),
            audit_log: Vec::new(),
            stats: GateStats::default(),
            full_auto_threshold: 30,
            suggest_threshold: 60,
            pending: Vec::new(),
        }
    }

    /// 根据风险评分确定审批级别
    pub fn classify(&self, risk_score: u32) -> ApprovalLevel {
        if risk_score < self.full_auto_threshold {
            ApprovalLevel::FullAuto
        } else if risk_score < self.suggest_threshold {
            ApprovalLevel::Suggest
        } else {
            ApprovalLevel::HumanReview
        }
    }

    /// 提交变更请求
    pub fn submit(&mut self, mut request: ChangeRequest) -> ApprovalDecision {
        let start = std::time::Instant::now();

        // 自动评分（如果未提供）
        if request.risk_score == 0 {
            request.risk_score = self.scorer.score(&request);
        }

        let risk_score = request.risk_score;
        let level = self.classify(risk_score);

        let decision = match level {
            ApprovalLevel::FullAuto => {
                // 自动批准
                self.stats.approved += 1;
                ApprovalDecision {
                    id: Uuid::new_v4(),
                    request_id: request.id,
                    level: ApprovalLevel::FullAuto,
                    approved: true,
                    approver: "system:full-auto".into(),
                    comment: Some("低风险，自动批准".into()),
                    timestamp: Utc::now(),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
            ApprovalLevel::Suggest => {
                // 推荐变更，加入待审批队列
                self.pending.push(request.clone());
                ApprovalDecision {
                    id: Uuid::new_v4(),
                    request_id: request.id,
                    level: ApprovalLevel::Suggest,
                    approved: false,
                    approver: "system:suggest".into(),
                    comment: Some("中风险，已推荐，等待批准".into()),
                    timestamp: Utc::now(),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
            ApprovalLevel::HumanReview => {
                // 人工审批，加入待审批队列
                self.pending.push(request.clone());
                ApprovalDecision {
                    id: Uuid::new_v4(),
                    request_id: request.id,
                    level: ApprovalLevel::HumanReview,
                    approved: false,
                    approver: "system:blocked".into(),
                    comment: Some("高风险，已阻止，等待人工审批".into()),
                    timestamp: Utc::now(),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
        };

        // 记录审计日志
        self.audit_log.push(AuditEntry {
            id: Uuid::new_v4(),
            request,
            decision: decision.clone(),
            execution_result: None,
        });

        // 更新统计
        self.stats.total_requests += 1;
        *self.stats.by_level.entry(level).or_insert(0) += 1;
        let n = self.stats.total_requests as f64;
        self.stats.avg_risk_score =
            (self.stats.avg_risk_score * (n - 1.0) + risk_score as f64) / n;

        decision
    }

    /// 批准待审批请求
    pub fn approve(&mut self, request_id: Uuid, approver: &str, comment: Option<String>) -> Option<ApprovalDecision> {
        if let Some(pos) = self.pending.iter().position(|r| r.id == request_id) {
            let request = self.pending.remove(pos);
            let start = std::time::Instant::now();

            let decision = ApprovalDecision {
                id: Uuid::new_v4(),
                request_id,
                level: self.classify(request.risk_score),
                approved: true,
                approver: format!("human:{}", approver),
                comment,
                timestamp: Utc::now(),
                duration_ms: start.elapsed().as_millis() as u64,
            };

            self.stats.approved += 1;
            self.audit_log.push(AuditEntry {
                id: Uuid::new_v4(),
                request,
                decision: decision.clone(),
                execution_result: None,
            });

            Some(decision)
        } else {
            None
        }
    }

    /// 拒绝待审批请求
    pub fn reject(&mut self, request_id: Uuid, approver: &str, reason: &str) -> Option<ApprovalDecision> {
        if let Some(pos) = self.pending.iter().position(|r| r.id == request_id) {
            let request = self.pending.remove(pos);
            let start = std::time::Instant::now();

            let decision = ApprovalDecision {
                id: Uuid::new_v4(),
                request_id,
                level: self.classify(request.risk_score),
                approved: false,
                approver: format!("human:{}", approver),
                comment: Some(reason.into()),
                timestamp: Utc::now(),
                duration_ms: start.elapsed().as_millis() as u64,
            };

            self.stats.rejected += 1;
            self.audit_log.push(AuditEntry {
                id: Uuid::new_v4(),
                request,
                decision: decision.clone(),
                execution_result: None,
            });

            Some(decision)
        } else {
            None
        }
    }

    /// 获取待审批列表
    pub fn pending_requests(&self) -> &[ChangeRequest] {
        &self.pending
    }

    /// 获取审计日志
    pub fn audit_trail(&self) -> &[AuditEntry] {
        &self.audit_log
    }

    /// 获取统计摘要
    pub fn stats_summary(&self) -> GateStatsSummary {
        GateStatsSummary {
            total: self.stats.total_requests,
            full_auto: self.stats.by_level.get(&ApprovalLevel::FullAuto).copied().unwrap_or(0),
            suggest: self.stats.by_level.get(&ApprovalLevel::Suggest).copied().unwrap_or(0),
            human_review: self.stats.by_level.get(&ApprovalLevel::HumanReview).copied().unwrap_or(0),
            approved: self.stats.approved,
            rejected: self.stats.rejected,
            pending: self.pending.len() as u64,
            avg_risk_score: self.stats.avg_risk_score,
        }
    }
}

impl Default for ApprovalGate {
    fn default() -> Self { Self::new() }
}

/// 审批门统计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateStatsSummary {
    pub total: u64,
    pub full_auto: u64,
    pub suggest: u64,
    pub human_review: u64,
    pub approved: u64,
    pub rejected: u64,
    pub pending: u64,
    pub avg_risk_score: f64,
}

impl fmt::Display for GateStatsSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        Approval Gate 统计")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "总请求:     {}", self.total)?;
        writeln!(f, "Full-Auto:  {}", self.full_auto)?;
        writeln!(f, "Suggest:    {}", self.suggest)?;
        writeln!(f, "Human Rev:  {}", self.human_review)?;
        writeln!(f, "───────────────────────────────────────────────")?;
        writeln!(f, "已批准:     {}", self.approved)?;
        writeln!(f, "已拒绝:     {}", self.rejected)?;
        writeln!(f, "待审批:     {}", self.pending)?;
        writeln!(f, "平均风险:   {:.1}", self.avg_risk_score)?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_full_auto() {
        let gate = ApprovalGate::new();
        assert_eq!(gate.classify(0), ApprovalLevel::FullAuto);
        assert_eq!(gate.classify(29), ApprovalLevel::FullAuto);
    }

    #[test]
    fn test_classify_suggest() {
        let gate = ApprovalGate::new();
        assert_eq!(gate.classify(30), ApprovalLevel::Suggest);
        assert_eq!(gate.classify(59), ApprovalLevel::Suggest);
    }

    #[test]
    fn test_classify_human_review() {
        let gate = ApprovalGate::new();
        assert_eq!(gate.classify(60), ApprovalLevel::HumanReview);
        assert_eq!(gate.classify(100), ApprovalLevel::HumanReview);
    }

    #[test]
    fn test_submit_low_risk_auto_approves() {
        let mut gate = ApprovalGate::new();
        let req = ChangeRequest::new(
            ChangeType::FileWrite,
            "write log file",
            "/tmp/log.txt",
            "hello",
            10,
        );
        let decision = gate.submit(req);
        assert!(decision.approved);
        assert_eq!(decision.level, ApprovalLevel::FullAuto);
        assert_eq!(gate.pending.len(), 0);
    }

    #[test]
    fn test_submit_medium_risk_pending() {
        let mut gate = ApprovalGate::new();
        let req = ChangeRequest::new(
            ChangeType::CodeModify,
            "modify function",
            "src/main.rs",
            "new code",
            45,
        );
        let decision = gate.submit(req);
        assert!(!decision.approved);
        assert_eq!(decision.level, ApprovalLevel::Suggest);
        assert_eq!(gate.pending.len(), 1);
    }

    #[test]
    fn test_submit_high_risk_blocked() {
        let mut gate = ApprovalGate::new();
        let req = ChangeRequest::new(
            ChangeType::SystemCall,
            "sudo rm -rf /",
            "/",
            "",
            90,
        );
        let decision = gate.submit(req);
        assert!(!decision.approved);
        assert_eq!(decision.level, ApprovalLevel::HumanReview);
        assert_eq!(gate.pending.len(), 1);
    }

    #[test]
    fn test_approve_pending() {
        let mut gate = ApprovalGate::new();
        let req = ChangeRequest::new(
            ChangeType::CodeModify,
            "update function",
            "src/lib.rs",
            "updated",
            40,
        );
        let decision = gate.submit(req);
        let req_id = decision.request_id;

        let approved = gate.approve(req_id, "alice", Some("looks good".into()));
        assert!(approved.is_some());
        assert!(approved.unwrap().approved);
        assert!(gate.pending.is_empty());
    }

    #[test]
    fn test_reject_pending() {
        let mut gate = ApprovalGate::new();
        let req = ChangeRequest::new(
            ChangeType::FileDelete,
            "delete config",
            "config.toml",
            "",
            55,
        );
        let decision = gate.submit(req);
        let req_id = decision.request_id;

        let rejected = gate.reject(req_id, "bob", "not allowed");
        assert!(rejected.is_some());
        assert!(!rejected.unwrap().approved);
        assert!(gate.pending.is_empty());
    }

    #[test]
    fn test_audit_trail_recorded() {
        let mut gate = ApprovalGate::new();
        let req = ChangeRequest::new(
            ChangeType::FileWrite,
            "write",
            "file.txt",
            "data",
            15,
        );
        gate.submit(req);
        assert_eq!(gate.audit_trail().len(), 1);
    }

    #[test]
    fn test_risk_scorer() {
        let scorer = RiskScorer::new();
        let req = ChangeRequest::new(
            ChangeType::SystemCall,
            "sudo rm -rf /tmp",
            "/tmp",
            "",
            0,
        );
        let score = scorer.score(&req);
        // SystemCall=60 + "sudo"=35 + "rm -rf"=40 = 135, capped at 100
        assert_eq!(score, 100);
    }

    #[test]
    fn test_stats_summary() {
        let mut gate = ApprovalGate::new();
        gate.submit(ChangeRequest::new(ChangeType::FileWrite, "w", "f", "d", 10));
        gate.submit(ChangeRequest::new(ChangeType::FileDelete, "d", "f", "", 50));
        let summary = gate.stats_summary();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.full_auto, 1);
        assert_eq!(summary.suggest, 1);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let gate = ApprovalGate::new();
        let json = serde_json::to_string(&gate.stats_summary()).unwrap();
        let _: GateStatsSummary = serde_json::from_str(&json).unwrap();
    }
}
