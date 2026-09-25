//! 自进化循环 — 共享类型根 (`nt_loop_types`).
//!
//! 常量阈值 + `Issue` / `EvolutionReport` + 独立 Auditor (G8 三角色异模型)
//! + `EvolutionLoop` 结构体与构造器。纯数据类型, 无循环逻辑;
//! `nt_loop_evaluate` / `nt_loop_select` / `nt_loop_mutate` / `nt_loop_core`
//! 均依赖本模块 (DAG 根)。从 `evolution_loop.rs` 纯搬移, 行为零变更。

// pub use crate::l1_action::nt_act::nt_l1_shared_types::IssueType;
use serde::{Deserialize, Serialize};
use crate::l5_cognition::nt_mind::evolution::evolution_daemon::IssueType;

// ============================================================
// 常量
// ============================================================

/// 大文件阈值 (行数)
pub const LARGE_FILE_THRESHOLD: usize = 800;

/// 无测试模块阈值 (行数)
pub const MISSING_TESTS_THRESHOLD: usize = 300;

/// 最大 unsafe 数量
pub const EXCESS_UNSAFE_THRESHOLD: usize = 5;

/// 最大 unwrap 数量
pub const EXCESS_UNWRAP_THRESHOLD: usize = 20;

/// 最大 TODO 残留数
pub const TODO_LEFTOVERS_THRESHOLD: usize = 3;

/// 停滞检测: 连续无改进次数上限
pub const STAGNATION_LIMIT: u32 = 10;

/// 自愈修复断路器轮次上限 (retry cap, 典型 1-10 次)
/// 防止 autofix 循环空转烧资源 (Claude Code 事故教训: 无上限导致 25 万 API 调用浪费)
pub const REPAIR_MAX_ROUNDS: usize = 10;

// ============================================================
// 问题类型 (re-exported from L1 nt_l1_shared_types via line 20)
// ============================================================

/// 检测到的具体问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub issue_type: IssueType,
    pub severity: u8,          // 1-10
    pub file: Option<String>,
    pub description: String,
    pub suggestion: String,
    pub auto_fixable: bool,
    pub cycle_discovered: u64,
}

pub use crate::l5_cognition::l1_facade::ProjectSnapshot;
// 拆分兼容：卫星件外迁，外部 `evolution_loop::MetaHarnessOptimizer` 路径保留。
pub use super::super::harness_optimizer::MetaHarnessOptimizer;

// ============================================================
// 进化报告
// ============================================================

/// 单次进化周期报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionReport {
    pub cycle: u64,
    pub issues_found: Vec<Issue>,
    pub issues_fixed: u32,
    pub snapshot: ProjectSnapshot,
    pub evolution_score: f64,       // 0-100 综合健康分
    pub free_energy: f64,           // 来自 ActiveInference
    pub phi: f64,                   // 来自 IIT
    pub suggestions: Vec<String>,
    pub new_patterns: Vec<String>,  // 新模式发现 (蒸馏结果)
    pub auto_fixes: u32,            // 自动修复计数
}

// ============================================================
// 独立 ground-truth Auditor (G8 — LongHorizon-Harness 吸收)
// 三角色异模型: Evidence(地面真值) / Consistency(无副作用) / Governance(治理合规)
// 独立于进化循环自身的裁决: verify→checkpoint→recover
// ============================================================

/// Auditor 三角色 — 三个独立评判视角, 全部通过才接受变更 (异模型共识)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum _AuditorRole {
    /// 地面真值: 触发问题的指标必须下降或持平
    Evidence,
    /// 一致性: 变更不引入副作用 (unsafe/todo/规模不回归)
    Consistency,
    /// 治理: 不引入新的违规热点文件
    Governance,
}

impl _AuditorRole {
    pub fn label(self) -> &'static str {
        match self {
            _AuditorRole::Evidence => "Evidence",
            _AuditorRole::Consistency => "Consistency",
            _AuditorRole::Governance => "Governance",
        }
    }
}

/// 单角色裁决
#[derive(Debug, Clone)]
pub struct _RoleVerdict {
    pub role: _AuditorRole,
    pub pass: bool,
    pub detail: String,
}

/// 一次完整审计裁决
#[derive(Debug, Clone)]
pub struct AuditVerdict {
    pub passed: bool,
    pub role_verdicts: Vec<_RoleVerdict>,
    pub recovered: bool,
    pub checkpoint_cycle: u64,
}

impl AuditVerdict {
    pub fn summary(&self) -> String {
        let roles: Vec<String> = self
            .role_verdicts
            .iter()
            .map(|v| format!("{}={}", v.role.label(), if v.pass { "PASS" } else { "FAIL" }))
            .collect();
        format!(
            "audit {} (roles: {}) recovered={} checkpoint_cycle={}",
            if self.passed { "PASS" } else { "REJECT" },
            roles.join(","),
            self.recovered,
            self.checkpoint_cycle,
        )
    }
}

/// 独立审计器 — 与进化循环自身裁决解耦, 维护 last-good 检查点
#[derive(Debug, Clone)]
pub struct Auditor {
    pub checkpoint_cycle: u64,
    pub verdict_history: Vec<(u64, bool, String)>,
    last_checkpoint: Option<ProjectSnapshot>,
}

impl Default for Auditor {
    fn default() -> Self {
        Self::new()
    }
}

impl Auditor {
    pub fn new() -> Self {
        Self {
            checkpoint_cycle: 0,
            verdict_history: Vec::new(),
            last_checkpoint: None,
        }
    }

    /// verify→checkpoint: 变更前保存 last-good 快照
    pub fn checkpoint(&mut self, cycle: u64, snap: &ProjectSnapshot) {
        self.last_checkpoint = Some(snap.clone());
        self.checkpoint_cycle = cycle;
    }

    /// 三角色异模型裁决 — 仅当前后快照均满足所有角色才接受; 否则标记 recover
    pub(crate) fn _verify_change(
        &mut self,
        cycle: u64,
        before: &ProjectSnapshot,
        after: &ProjectSnapshot,
    ) -> AuditVerdict {
        let mut verdicts = Vec::new();

        // Evidence: 触发问题指标下降或持平
        let evidence_pass = after.unwrap_count <= before.unwrap_count
            && after.compile_errors <= before.compile_errors
            && after.test_failures <= before.test_failures;
        verdicts.push(_RoleVerdict {
            role: _AuditorRole::Evidence,
            pass: evidence_pass,
            detail: format!(
                "unwrap {}→{} compile_errors {}→{} test_failures {}→{}",
                before.unwrap_count,
                after.unwrap_count,
                before.compile_errors,
                after.compile_errors,
                before.test_failures,
                after.test_failures,
            ),
        });

        // Consistency: 无副作用 — unsafe/todo 不回归 (容忍 ±1 噪声)
        let consistency_pass =
            after.unsafe_count <= before.unsafe_count + 1 && after.todo_count <= before.todo_count + 1;
        verdicts.push(_RoleVerdict {
            role: _AuditorRole::Consistency,
            pass: consistency_pass,
            detail: format!(
                "unsafe {}→{} todo {}→{}",
                before.unsafe_count, after.unsafe_count, before.todo_count, after.todo_count,
            ),
        });

        // Governance: 不引入新的违规热点文件
        let governance_pass = after.file_unsafe_hotspots.len() <= before.file_unsafe_hotspots.len();
        verdicts.push(_RoleVerdict {
            role: _AuditorRole::Governance,
            pass: governance_pass,
            detail: format!(
                "unsafe_hotspots {}→{}",
                before.file_unsafe_hotspots.len(),
                after.file_unsafe_hotspots.len(),
            ),
        });

        let passed = verdicts.iter().all(|v| v.pass);
        let mut recovered = false;
        if passed {
            self.last_checkpoint = Some(after.clone());
            self.checkpoint_cycle = cycle;
        } else if self.last_checkpoint.is_some() {
            // recover: 裁决拒绝 → 回滚到 last-good (last_checkpoint 保持不变)
            recovered = true;
        }

        let summary = format!(
            "{}: {}",
            if passed { "PASS" } else { "REJECT" },
            verdicts
                .iter()
                .map(|v| format!("{}={}", v.role.label(), if v.pass { "PASS" } else { "FAIL" }))
                .collect::<Vec<_>>()
                .join(",")
        );
        self.verdict_history.push((cycle, passed, summary.clone()));

        AuditVerdict {
            passed,
            role_verdicts: verdicts,
            recovered,
            checkpoint_cycle: self.checkpoint_cycle,
        }
    }

    /// 当前 last-good 检查点
    pub fn last_checkpoint(&self) -> Option<&ProjectSnapshot> {
        self.last_checkpoint.as_ref()
    }
}

// ============================================================
// 进化引擎
// ============================================================

// ── G10: RST 递归任务合成飞轮 (seed→extend→_realign→validate→reuse) ──
// 吸收自 RST 2608.05466 (Self-Training with Recursive Task Synthesis):
// 用已验证种子任务迭代合成更难任务, 验证器 (validator) 对齐生成器, 防止
// 分布漂移; 验证通过的任务进 reuse 池, 形成数据飞轮。玩具实现: 任务 =
// 结构化字符串, 验证器 = 启发式 (复杂度单调 + 可解性检查), 无 LLM 依赖,
// 纯确定性可测。接线到 EvolutionLoop 作为"递归任务合成"进化输入源。

/// 自进化循环引擎
#[derive(Debug, Clone)]
pub struct EvolutionLoop {
    pub cycle: u64,
    pub issues: Vec<Issue>,
    pub consecutive_stagnant: u32,
    pub fixed_history: Vec<u32>,
    pub enabled: bool,

    /// 被进化的目标项目目录（None = 自身/Cargo 项目根，保持旧行为）
    pub target_dir: Option<std::path::PathBuf>,

    // 上次扫描结果缓存 (跨模块 impl 访问 → pub(crate))
    pub(crate) last_snapshot: Option<ProjectSnapshot>,

    /// 独立 ground-truth Auditor (G8) — verify→checkpoint→recover, 三角色异模型
    pub auditor: Auditor,

    /// 最近一次审计裁决
    pub last_audit: Option<AuditVerdict>,
}

impl Default for EvolutionLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl EvolutionLoop {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            issues: Vec::new(),
            consecutive_stagnant: 0,
            fixed_history: Vec::new(),
            enabled: true,
            target_dir: None,
            last_snapshot: None,
            auditor: Auditor::new(),
            last_audit: None,
        }
    }

    /// 创建针对任意目标项目的进化循环
    pub fn for_target(target_dir: impl Into<std::path::PathBuf>) -> Self {
        let mut el = Self::new();
        el.target_dir = Some(target_dir.into());
        el
    }
}
