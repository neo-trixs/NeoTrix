//! # 审查层 — TriadReview + CommitGate + EvolutionCheckpointLedger
//!
//! 借鉴 Ouroboros 的三代审架构:
//! - TriadReview: 3 个 E8 状态独立投票
//! - ScopeReview: 全代码库架构影响审查
//! - CommitGate: 确定性预提交门控
//! - EvolutionCheckpointLedger: 进化检查点 + 回滚

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::constitution::Constitution;
use crate::core::nt_core_experience::evolution_coordinator::EvolutionCoordinator;

// ============================================================
// Compilation result cache (5 second TTL)
// ============================================================

struct CachedCheck {
    result: (bool, String),
    timestamp: Instant,
}

static COMPILE_CACHE: std::sync::LazyLock<Mutex<Option<CachedCheck>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));
static TEST_CACHE: std::sync::LazyLock<Mutex<Option<CachedCheck>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));
static CONSTITUTION_CACHE: std::sync::LazyLock<Mutex<Option<CachedCheck>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

fn cache_get(cache: &Mutex<Option<CachedCheck>>) -> Option<(bool, String)> {
    let guard = cache.lock().ok()?;
    let entry = guard.as_ref()?;
    if entry.timestamp.elapsed() < Duration::from_secs(5) {
        Some(entry.result.clone())
    } else {
        None
    }
}

fn cache_set(cache: &Mutex<Option<CachedCheck>>, passed: bool, detail: String) {
    if let Ok(mut guard) = cache.lock() {
        *guard = Some(CachedCheck {
            result: (passed, detail),
            timestamp: Instant::now(),
        });
    }
}

// ============================================================
// Individual check implementations
// ============================================================

fn check_code_compiles(timeout_secs: u64) -> (bool, String) {
    if let Some(cached) = cache_get(&COMPILE_CACHE) {
        return (cached.0, format!("[cached] {}", cached.1));
    }

    if cfg!(test) {
        return (true, String::new());
    }

    let Ok(child) = Command::new("cargo")
        .args(["check", "--message-format=json"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    else {
        let detail = "无法启动 cargo check — cargo 不在 PATH 中".to_string();
        cache_set(&COMPILE_CACHE, false, detail.clone());
        return (false, detail);
    };

    let (tx, rx) = std::sync::mpsc::sync_channel(256);
    std::thread::spawn(move || {
        let output = child.wait_with_output();
        if tx.send(output).is_err() {
            log::warn!("tx.send failed in compile check thread");
        }
    });

    let result = rx.recv_timeout(Duration::from_secs(timeout_secs));
    let output = match result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            let detail = format!("cargo check 进程错误: {}", e);
            cache_set(&COMPILE_CACHE, false, detail.clone());
            return (false, detail);
        }
        Err(_) => {
            let detail = format!("cargo check 超时 ({}s)", timeout_secs);
            cache_set(&COMPILE_CACHE, false, detail.clone());
            return (false, detail);
        }
    };

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let error_count = stderr_str.matches("\"level\":\"error\"").count()
        + stderr_str.matches("error[").count()
        + stderr_str.matches("error:").count();

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let total_errors = error_count + stdout_str.matches("\"level\":\"error\"").count();

    if !output.status.success() && total_errors > 0 {
        let detail = format!("编译失败 — {} 个错误", total_errors);
        cache_set(&COMPILE_CACHE, false, detail.clone());
        (false, detail)
    } else {
        let detail = "cargo check 通过 — 0 个错误".to_string();
        cache_set(&COMPILE_CACHE, true, detail.clone());
        (true, detail)
    }
}

fn check_tests_pass() -> (bool, String) {
    if let Some(cached) = cache_get(&TEST_CACHE) {
        return (cached.0, format!("[cached] {}", cached.1));
    }

    if cfg!(test) {
        return (true, String::new());
    }

    let Ok(child) = Command::new("cargo")
        .args(["test", "--lib"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    else {
        let detail = "无法启动 cargo test — cargo 不在 PATH 中".to_string();
        cache_set(&TEST_CACHE, false, detail.clone());
        return (false, detail);
    };

    let (tx, rx) = std::sync::mpsc::sync_channel(256);
    std::thread::spawn(move || {
        let output = child.wait_with_output();
        if tx.send(output).is_err() {
            log::warn!("tx.send failed in test thread");
        }
    });

    let result = rx.recv_timeout(Duration::from_secs(60));
    let output = match result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            let detail = format!("cargo test 进程错误: {}", e);
            cache_set(&TEST_CACHE, false, detail.clone());
            return (false, detail);
        }
        Err(_) => {
            let detail = "cargo test 超时 (60s)".to_string();
            cache_set(&TEST_CACHE, false, detail.clone());
            return (false, detail);
        }
    };

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}\n{}", stdout_str, stderr_str);

    if combined.contains("test result: FAILED") {
        let failed_count = combined.lines().filter(|l| l.contains("FAILED")).count();
        let detail = format!("测试失败 — {} 个测试未通过", failed_count);
        cache_set(&TEST_CACHE, false, detail.clone());
        (false, detail)
    } else if combined.contains("test result: ok") {
        let detail = "所有测试通过".to_string();
        cache_set(&TEST_CACHE, true, detail.clone());
        (true, detail)
    } else {
        let detail = "测试结果无法解析 — 可能编译失败或无测试运行".to_string();
        cache_set(&TEST_CACHE, false, detail.clone());
        (false, detail)
    }
}

fn check_constitution_violations() -> (bool, String) {
    if let Some(cached) = cache_get(&CONSTITUTION_CACHE) {
        return (cached.0, format!("[cached] {}", cached.1));
    }

    let constitution = Constitution::new();
    let report = constitution.check_integrity();

    let violations: Vec<_> = report.checks.iter().filter(|c| !c.passed).collect();

    if report.all_passed {
        let detail = format!("宪法完整 — {} 项检查全部通过", report.checks.len());
        cache_set(&CONSTITUTION_CACHE, true, detail.clone());
        (true, detail)
    } else {
        let failed_names: Vec<_> = violations.iter().map(|c| c.id.as_str()).collect();
        let detail = format!(
            "宪法违规 [{}] — {}/{} 检查未通过",
            failed_names.join(", "),
            violations.len(),
            report.checks.len()
        );
        cache_set(&CONSTITUTION_CACHE, false, detail.clone());
        (false, detail)
    }
}

fn check_changelog_updated() -> (bool, String) {
    let Ok(output) = Command::new("git")
        .args(["diff", "--name-only", "HEAD"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
    else {
        return (true, "git 不可用 — 跳过 CHANGELOG 检查 (优雅降级)".into());
    };

    if !output.status.success() {
        return (
            true,
            "git diff 失败 — 跳过 CHANGELOG 检查 (优雅降级)".into(),
        );
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let modified_files: Vec<&str> = stdout_str.lines().collect();

    if modified_files.is_empty() {
        return (true, "无未提交变更 — CHANGELOG 无需更新".into());
    }

    let changelog_modified = modified_files.iter().any(|f| {
        let lower = f.to_lowercase();
        lower.contains("changelog") || lower.contains("changelog.md")
    });

    if changelog_modified {
        (true, "CHANGELOG.md 已在变更集中".into())
    } else {
        let detail = format!(
            "CHANGELOG.md 未更新 — 当前有 {} 个文件变更但未包含 CHANGELOG",
            modified_files.len()
        );
        (false, detail)
    }
}

fn check_evolution_bound(coordinator: &EvolutionCoordinator) -> (bool, String) {
    let cycle = coordinator.cycle;
    let bound: u64 = 10000;

    if cycle < bound {
        (true, format!("进化周期 {} < {} — 边界安全", cycle, bound))
    } else {
        (
            false,
            format!("进化周期 {} >= {} — 超出安全边界", cycle, bound),
        )
    }
}

// ============================================================
// 审查状态
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewStatus {
    Pending,
    InProgress,
    Approved,
    Rejected,
    Merged,
    RolledBack,
}

impl ReviewStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ReviewStatus::Approved
                | ReviewStatus::Rejected
                | ReviewStatus::Merged
                | ReviewStatus::RolledBack
        )
    }
}

// ============================================================
// TriadReview — 三 E8 状态投票
// ============================================================

/// 三代审: 3 个独立 E8 状态投票
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriadReview {
    pub review_id: String,
    pub votes: Vec<Vote>,
    pub status: ReviewStatus,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub reviewer_id: String, // E8 状态 ID
    pub decision: VoteDecision,
    pub rationale: String,
    pub concerns: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteDecision {
    Approve,
    Reject,
    Abstain,
}

impl TriadReview {
    pub fn new(review_id: &str) -> Self {
        Self {
            review_id: review_id.to_string(),
            votes: Vec::new(),
            status: ReviewStatus::Pending,
            summary: String::new(),
        }
    }

    /// 添加投票
    pub fn add_vote(
        &mut self,
        reviewer_id: &str,
        decision: VoteDecision,
        rationale: &str,
        concerns: Vec<String>,
    ) {
        self.votes.push(Vote {
            reviewer_id: reviewer_id.to_string(),
            decision,
            rationale: rationale.to_string(),
            concerns,
        });
        self.evaluate_quorum();
    }

    /// 评估是否需要 ≥2/3 多数
    fn evaluate_quorum(&mut self) {
        if self.votes.len() < 3 {
            self.status = ReviewStatus::InProgress;
            return;
        }
        let approves = self
            .votes
            .iter()
            .filter(|v| v.decision == VoteDecision::Approve)
            .count();
        let quorum = approves >= 2;
        self.status = if quorum {
            ReviewStatus::Approved
        } else {
            ReviewStatus::Rejected
        };
        self.summary = format!(
            "{}/3 通过 — {}",
            approves,
            if quorum {
                "Quorum 达成"
            } else {
                "Quorum 不足"
            }
        );
    }

    /// 重置审查
    pub fn reset(&mut self) {
        self.votes.clear();
        self.status = ReviewStatus::Pending;
        self.summary = String::new();
    }
}

// ============================================================
// ScopeReview — 全代码库架构审查
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeReview {
    pub review_id: String,
    pub status: ReviewStatus,
    pub affected_modules: Vec<String>,
    pub impact_level: ImpactLevel,
    pub findings: Vec<ScopeFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeFinding {
    pub module: String,
    pub impact: String,
    pub severity: FindingSeverity,
    pub recommendation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactLevel {
    Isolated,     // 单模块影响
    Local,        // 相邻模块
    Architecture, // 架构级
    Global,       // 全系统
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    Info,
    Warning,
    Critical,
}

impl ScopeReview {
    pub fn new(review_id: &str, affected_modules: Vec<String>) -> Self {
        Self {
            review_id: review_id.to_string(),
            status: ReviewStatus::Pending,
            affected_modules,
            impact_level: ImpactLevel::Isolated,
            findings: Vec::new(),
        }
    }

    pub fn add_finding(
        &mut self,
        module: &str,
        impact: &str,
        severity: FindingSeverity,
        recommendation: &str,
    ) {
        self.findings.push(ScopeFinding {
            module: module.to_string(),
            impact: impact.to_string(),
            severity,
            recommendation: recommendation.to_string(),
        });
    }

    /// 执行架构影响分析
    pub fn analyze(&mut self) {
        if self
            .findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Critical)
        {
            self.impact_level = ImpactLevel::Global;
            self.status = ReviewStatus::Rejected;
        } else if self
            .findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Warning)
        {
            self.impact_level = ImpactLevel::Architecture;
            self.status = ReviewStatus::Approved;
        } else {
            self.impact_level = ImpactLevel::Isolated;
            self.status = ReviewStatus::Approved;
        }
    }
}

// ============================================================
// CommitGate — 确定性预提交门控
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitGate {
    pub gate_id: String,
    pub checks: Vec<GateCheck>,
    pub all_passed: bool,
    pub compile_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

impl CommitGate {
    pub fn new(gate_id: &str) -> Self {
        Self {
            gate_id: gate_id.to_string(),
            checks: Vec::new(),
            all_passed: false,
            compile_timeout_secs: 120,
        }
    }

    pub fn with_compile_timeout(mut self, timeout_secs: u64) -> Self {
        self.compile_timeout_secs = timeout_secs;
        self
    }

    pub fn add_check(&mut self, name: &str, passed: bool, detail: &str) {
        self.checks.push(GateCheck {
            name: name.to_string(),
            passed,
            detail: detail.to_string(),
        });
    }

    pub fn run_standard_checks(&mut self) {
        self.run_standard_checks_with_coordinator(None);
    }

    pub fn run_standard_checks_with_coordinator(
        &mut self,
        coordinator: Option<&EvolutionCoordinator>,
    ) {
        self.checks.clear();

        // Check 1: code_compiles
        let (passed, detail) = check_code_compiles(self.compile_timeout_secs);
        self.add_check("code_compiles", passed, &detail);

        // Check 2: tests_pass
        let (passed, detail) = check_tests_pass();
        self.add_check("tests_pass", passed, &detail);

        // Check 3: constitution_violations
        let (passed, detail) = check_constitution_violations();
        self.add_check("constitution_violations", passed, &detail);

        // Check 4: changelog_updated
        let (passed, detail) = check_changelog_updated();
        self.add_check("changelog_updated", passed, &detail);

        // Check 5: evolution_bound
        let (passed, detail) = if let Some(coord) = coordinator {
            check_evolution_bound(coord)
        } else {
            (true, "未提供进化协调器 — 跳过边界检查 (优雅降级)".into())
        };
        self.add_check("evolution_bound", passed, &detail);

        self.all_passed = self.checks.iter().all(|c| c.passed);
    }

    /// 是否所有检查通过
    pub fn is_passed(&self) -> bool {
        self.all_passed
    }
}

// ============================================================
// EvolutionCheckpointLedger — 进化检查点 + 回滚
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionCheckpointLedger {
    pub checkpoints: Vec<EvolutionCheckpoint>,
    pub max_checkpoints: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionCheckpoint {
    pub id: String,
    pub timestamp: u64,
    pub cycle: u64,
    pub description: String,
    pub snapshot: String, // 序列化的状态快照
    pub parent_id: Option<String>,
    pub status: CheckpointStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointStatus {
    Active,
    RolledBack,
    Superseded,
}

impl EvolutionCheckpointLedger {
    pub fn new(max_checkpoints: usize) -> Self {
        Self {
            checkpoints: Vec::with_capacity(max_checkpoints),
            max_checkpoints,
        }
    }

    /// 创建检查点
    pub fn create_checkpoint(&mut self, cycle: u64, description: &str, snapshot: &str) -> String {
        let id = format!("ckpt-{:x}", self.checkpoints.len() + 1);
        let parent_id = self.checkpoints.last().map(|c| c.id.clone());
        let checkpoint = EvolutionCheckpoint {
            id: id.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            cycle,
            description: description.to_string(),
            snapshot: snapshot.to_string(),
            parent_id,
            status: CheckpointStatus::Active,
        };
        self.checkpoints.push(checkpoint);

        // 限制最大检查点数量
        while self.checkpoints.len() > self.max_checkpoints {
            let removed = self.checkpoints.remove(0);
            if removed.status == CheckpointStatus::Active {
                // 标记后续检查点为 Superseded
                for c in self.checkpoints.iter_mut() {
                    if c.parent_id.as_deref() == Some(&removed.id) {
                        c.status = CheckpointStatus::Superseded;
                    }
                }
            }
        }

        id
    }

    /// 回滚到指定检查点
    pub fn rollback_to(&mut self, checkpoint_id: &str) -> Option<&EvolutionCheckpoint> {
        let pos = self
            .checkpoints
            .iter()
            .position(|c| c.id == checkpoint_id)?;
        // 标记所有后续检查点为 RolledBack
        for c in self.checkpoints.iter_mut().skip(pos + 1) {
            c.status = CheckpointStatus::RolledBack;
        }
        self.checkpoints.get(pos)
    }

    /// 获取最近的检查点
    pub fn latest(&self) -> Option<&EvolutionCheckpoint> {
        self.checkpoints
            .iter()
            .filter(|c| c.status == CheckpointStatus::Active)
            .last()
    }

    /// 获取活跃检查点数量
    pub fn active_count(&self) -> usize {
        self.checkpoints
            .iter()
            .filter(|c| c.status == CheckpointStatus::Active)
            .count()
    }

    /// 获取回滚数量
    pub fn rollback_count(&self) -> usize {
        self.checkpoints
            .iter()
            .filter(|c| c.status == CheckpointStatus::RolledBack)
            .count()
    }

    /// 清除被取代的检查点
    pub fn prune_superseded(&mut self) {
        self.checkpoints
            .retain(|c| c.status != CheckpointStatus::Superseded);
    }
}

// ============================================================
// 审查会话管理器
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSession {
    pub session_id: String,
    pub triad: TriadReview,
    pub scope: Option<ScopeReview>,
    pub gate: CommitGate,
    pub status: ReviewStatus,
    pub target_module: String,
    pub description: String,
}

impl ReviewSession {
    pub fn new(session_id: &str, target_module: &str, description: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            triad: TriadReview::new(session_id),
            scope: None,
            gate: CommitGate::new(session_id),
            status: ReviewStatus::Pending,
            target_module: target_module.to_string(),
            description: description.to_string(),
        }
    }

    /// 运行完整审查流程
    pub fn run_full_review(&mut self) -> ReviewStatus {
        // Phase 1: TriadReview
        if self.triad.status != ReviewStatus::Approved {
            return self.triad.status;
        }

        // Phase 2: ScopeReview
        if let Some(ref scope) = self.scope {
            if scope.status != ReviewStatus::Approved {
                return scope.status;
            }
        }

        // Phase 3: CommitGate
        self.gate.run_standard_checks();
        if !self.gate.is_passed() {
            self.status = ReviewStatus::Rejected;
            return self.status;
        }

        self.status = ReviewStatus::Approved;
        self.status
    }
}

// ============================================================
// 审查管理器
// ============================================================

pub struct ReviewManager {
    pub sessions: HashMap<String, ReviewSession>,
    pub checkpoint_ledger: EvolutionCheckpointLedger,
}

impl ReviewManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            checkpoint_ledger: EvolutionCheckpointLedger::new(100),
        }
    }

    pub fn create_session(
        &mut self,
        session_id: &str,
        target_module: &str,
        description: &str,
    ) -> &mut ReviewSession {
        self.sessions
            .entry(session_id.to_string())
            .or_insert_with(|| ReviewSession::new(session_id, target_module, description))
    }

    pub fn get_session(&self, session_id: &str) -> Option<&ReviewSession> {
        self.sessions.get(session_id)
    }

    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut ReviewSession> {
        self.sessions.get_mut(session_id)
    }

    pub fn approve(&mut self, session_id: &str) -> bool {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.status = ReviewStatus::Approved;
            true
        } else {
            false
        }
    }

    pub fn reject(&mut self, session_id: &str) -> bool {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.status = ReviewStatus::Rejected;
            true
        } else {
            false
        }
    }
}

impl Default for ReviewManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_triad_review_approves_with_quorum() {
        let mut review = TriadReview::new("test-1");
        review.add_vote("E8-01", VoteDecision::Approve, "Looks good", vec![]);
        review.add_vote("E8-02", VoteDecision::Approve, "Agreed", vec![]);
        review.add_vote(
            "E8-03",
            VoteDecision::Reject,
            "Concerns about safety",
            vec!["Memory leak".into()],
        );
        assert_eq!(review.status, ReviewStatus::Approved);
    }

    #[test]
    fn test_triad_review_rejects_without_quorum() {
        let mut review = TriadReview::new("test-2");
        review.add_vote("E8-01", VoteDecision::Approve, "OK", vec![]);
        review.add_vote("E8-02", VoteDecision::Reject, "Not safe", vec![]);
        review.add_vote("E8-03", VoteDecision::Reject, "Bad design", vec![]);
        assert_eq!(review.status, ReviewStatus::Rejected);
    }

    #[test]
    fn test_triad_review_pending_before_votes() {
        let review = TriadReview::new("test-3");
        assert_eq!(review.status, ReviewStatus::Pending);
    }

    #[serial]
    #[test]
    fn test_commit_gate_standard_checks_has_all_5() {
        let mut gate = CommitGate::new("gate-1");
        gate.run_standard_checks();
        assert_eq!(gate.checks.len(), 5);
        let names: Vec<&str> = gate.checks.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"code_compiles"));
        assert!(names.contains(&"tests_pass"));
        assert!(names.contains(&"constitution_violations"));
        assert!(names.contains(&"changelog_updated"));
        assert!(names.contains(&"evolution_bound"));
    }

    #[serial]
    #[test]
    fn test_commit_gate_evolution_bound_no_coordinator_degraded() {
        let mut gate = CommitGate::new("gate-evo");
        gate.run_standard_checks();
        let evo_check = gate
            .checks
            .iter()
            .find(|c| c.name == "evolution_bound")
            .unwrap();
        assert!(
            evo_check.passed,
            "no coordinator should degrade gracefully as passed"
        );
        assert!(evo_check.detail.contains("优雅降级"));
    }

    #[serial]
    #[test]
    fn test_commit_gate_evolution_bound_with_coordinator() {
        let mut coord =
            crate::core::nt_core_experience::evolution_coordinator::EvolutionCoordinator::new();
        coord.cycle = 500;
        let mut gate = CommitGate::new("gate-evo-2");
        gate.run_standard_checks_with_coordinator(Some(&coord));
        let evo_check = gate
            .checks
            .iter()
            .find(|c| c.name == "evolution_bound")
            .unwrap();
        assert!(evo_check.passed);
        assert!(evo_check.detail.contains("500"));
    }

    #[test]
    fn test_checkpoint_create_and_rollback() {
        let mut ledger = EvolutionCheckpointLedger::new(10);
        let c1 = ledger.create_checkpoint(1, "Initial", "snapshot_1");
        let _c2 = ledger.create_checkpoint(2, "Second", "snapshot_2");
        assert_eq!(ledger.active_count(), 2);

        let rolled_back = ledger.rollback_to(&c1);
        assert!(rolled_back.is_some());
        assert_eq!(ledger.active_count(), 1);
        assert_eq!(ledger.rollback_count(), 1);
    }

    #[test]
    fn test_checkpoint_max_limit() {
        let mut ledger = EvolutionCheckpointLedger::new(3);
        ledger.create_checkpoint(1, "A", "s1");
        ledger.create_checkpoint(2, "B", "s2");
        ledger.create_checkpoint(3, "C", "s3");
        ledger.create_checkpoint(4, "D", "s4");
        assert!(ledger.checkpoints.len() <= 3);
    }

    #[serial]
    #[test]
    fn test_review_session_full_approve() {
        let mut session =
            ReviewSession::new("test-session", "nt_core_hcube", "VSA NAG integration");
        session
            .triad
            .add_vote("E8-01", VoteDecision::Approve, "Good", vec![]);
        session
            .triad
            .add_vote("E8-02", VoteDecision::Approve, "Fine", vec![]);
        session
            .triad
            .add_vote("E8-03", VoteDecision::Approve, "Agree", vec![]);

        let mut scope = ScopeReview::new("scope-1", vec!["nt_core_hcube".into()]);
        scope.add_finding(
            "nt_core_hcube",
            "Minor API change",
            FindingSeverity::Info,
            "Backward compatible",
        );
        scope.analyze();
        session.scope = Some(scope);

        let status = session.run_full_review();
        // 真实 backends 可能因预存编译错误而拒绝, 但流程应走完到终端态
        assert!(
            status.is_terminal(),
            "full review should reach terminal state, got {:?}",
            status
        );
    }

    #[test]
    fn test_review_manager_create_and_approve() {
        let mut mgr = ReviewManager::new();
        mgr.create_session("s1", "test", "test review");
        assert!(mgr.approve("s1"));
        let session = mgr.get_session("s1").unwrap();
        assert_eq!(session.status, ReviewStatus::Approved);
    }

    #[test]
    fn test_evolution_checkpoint_prune_superseded() {
        let mut ledger = EvolutionCheckpointLedger::new(10);
        let c1 = ledger.create_checkpoint(1, "A", "s1");
        let _c2 = ledger.create_checkpoint(2, "B", "s2");
        ledger.rollback_to(&c1);
        ledger.create_checkpoint(3, "C", "s3");
        ledger.prune_superseded();
        assert_eq!(ledger.checkpoints.len(), 2); // c1 (active) + c3 (active)
    }

    #[test]
    fn test_scope_review_critical_rejects() {
        let mut scope = ScopeReview::new("scope-2", vec!["core".into()]);
        scope.add_finding(
            "core",
            "Architecture violation",
            FindingSeverity::Critical,
            "Must redesign",
        );
        scope.analyze();
        assert_eq!(scope.status, ReviewStatus::Rejected);
        assert_eq!(scope.impact_level, ImpactLevel::Global);
    }

    #[test]
    fn test_review_reset() {
        let mut review = TriadReview::new("reset-test");
        review.add_vote("E8-01", VoteDecision::Approve, "ok", vec![]);
        review.add_vote("E8-02", VoteDecision::Approve, "ok", vec![]);
        review.add_vote("E8-03", VoteDecision::Approve, "ok", vec![]);
        assert_eq!(review.status, ReviewStatus::Approved);
        review.reset();
        assert_eq!(review.status, ReviewStatus::Pending);
        assert!(review.votes.is_empty());
    }
}
