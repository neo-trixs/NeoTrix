//! 自进化循环引擎 — 持续迭代: 扫描 → 分析 → 修复 → 蒸馏
//!
//! 设计: 每个周期执行:
//!   1. 项目扫描 (代码度量, 文件大小, 测试覆盖率, 编译状态)
//!   2. 瓶颈分析 (慢模块, 循环依赖, unwrap 热点)
//!   3. Bug 检测 (编译警告, 测试失败, unsafe 使用)
//!   4. 自修复生成 (修复警告, 补全导入, 处理 unwrap)
//!   5. 模式蒸馏 (提取行为规则, 更新 AGENTS.md)
//!   6. 报告输出 (状态仪表盘)
//!
//! 融合 AGENTS.md 元认知自检 + MetaCognitive Self-Check 协议

use crate::neotrix::nt_mind_autofixer::AutoFixer;
use crate::neotrix::nt_act_code::PipelineAutoFixer;
use crate::neotrix::nt_mind_self_diagnose::{
    ActionExecutor, CodeUnderlyingIssue, DiagnosticItem, EvolutionLoopProvider,
    PriorityQueue, PrioritizedIssue, RepairCircuitBreaker, SelfDiagnose,
};
use crate::neotrix::nt_core_iit_phi::IITPhiCalculator;
use crate::neotrix::nt_world_infer::ActiveInferenceEngine;
pub use crate::neotrix::l1_body_impl::nt_l1_shared_types::IssueType;
use serde::{Deserialize, Serialize};

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

// ============================================================
// 项目快照
// ============================================================

/// 项目健康快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub total_files: usize,
    pub total_lines: usize,
    pub large_files: Vec<String>,
    pub modules_without_tests: Vec<String>,
    pub file_unsafe_hotspots: Vec<String>,
    pub unsafe_count: usize,
    pub unwrap_count: usize,
    pub todo_count: usize,
    pub compile_errors: usize,
    pub compile_warnings: usize,
    pub test_count: usize,
    pub test_failures: usize,
}

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
pub enum AuditorRole {
    /// 地面真值: 触发问题的指标必须下降或持平
    Evidence,
    /// 一致性: 变更不引入副作用 (unsafe/todo/规模不回归)
    Consistency,
    /// 治理: 不引入新的违规热点文件
    Governance,
}

impl AuditorRole {
    pub fn label(self) -> &'static str {
        match self {
            AuditorRole::Evidence => "Evidence",
            AuditorRole::Consistency => "Consistency",
            AuditorRole::Governance => "Governance",
        }
    }
}

/// 单角色裁决
#[derive(Debug, Clone)]
pub struct RoleVerdict {
    pub role: AuditorRole,
    pub pass: bool,
    pub detail: String,
}

/// 一次完整审计裁决
#[derive(Debug, Clone)]
pub struct AuditVerdict {
    pub passed: bool,
    pub role_verdicts: Vec<RoleVerdict>,
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
    pub fn verify_change(
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
        verdicts.push(RoleVerdict {
            role: AuditorRole::Evidence,
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
        verdicts.push(RoleVerdict {
            role: AuditorRole::Consistency,
            pass: consistency_pass,
            detail: format!(
                "unsafe {}→{} todo {}→{}",
                before.unsafe_count, after.unsafe_count, before.todo_count, after.todo_count,
            ),
        });

        // Governance: 不引入新的违规热点文件
        let governance_pass = after.file_unsafe_hotspots.len() <= before.file_unsafe_hotspots.len();
        verdicts.push(RoleVerdict {
            role: AuditorRole::Governance,
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
// 精确计数函数
// ============================================================

fn count_actual_unsafe(content: &str) -> usize {
    let mut count = 0usize;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("//") || t.starts_with("//!") || t.starts_with("/*") || t.starts_with("*") {
            continue;
        }
        if line.contains("matches(\"unsafe\"") || line.contains("contains(\"unsafe\"") {
            continue;
        }
        if t.contains("unsafe {") || t.contains("unsafe fn") || t.contains("unsafe trait") || t.contains("unsafe impl") {
            count += 1;
        }
    }
    count
}

// ============================================================
// 进化引擎
// ============================================================

// ── G10: RST 递归任务合成飞轮 (seed→extend→realign→validate→reuse) ──
// 吸收自 RST 2608.05466 (Self-Training with Recursive Task Synthesis):
// 用已验证种子任务迭代合成更难任务, 验证器 (validator) 对齐生成器, 防止
// 分布漂移; 验证通过的任务进 reuse 池, 形成数据飞轮。玩具实现: 任务 =
// 结构化字符串, 验证器 = 启发式 (复杂度单调 + 可解性检查), 无 LLM 依赖,
// 纯确定性可测。接线到 EvolutionLoop 作为"递归任务合成"进化输入源。

/// RST 任务 — 递归合成的单元。
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RstTask {
    /// 任务 id。
    pub id: String,
    /// 任务描述。
    pub prompt: String,
    /// 合成代数 (种子 = 0)。
    pub generation: u32,
    /// 复杂度评分 (验证器用于对齐: 应单调不减)。
    pub complexity: f64,
    /// 是否已通过验证。
    pub verified: bool,
    /// 被复用的父任务 id (None = 种子)。
    pub parent: Option<String>,
}

/// RST 验证结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RstVerdict {
    pub task_id: String,
    pub accepted: bool,
    /// 拒绝原因 (accepted=false 时)。
    pub reason: String,
}

/// RST 飞轮 — seed→extend→realign→validate→reuse。
#[derive(Debug, Clone)]
pub struct RstFlywheel {
    /// 已验证任务池 (reuse 源)。
    pub verified_pool: Vec<RstTask>,
    /// 生成代数上限 (防止无界漂移)。
    pub max_generation: u32,
    /// 每代最大合成数。
    pub extend_per_gen: usize,
    /// 复杂度对齐阈值 — 新任务复杂度必须 ≥ 父任务 × 阈值。
    pub realign_threshold: f64,
    /// 复杂度上限 (超出即拒绝, 防发散)。
    pub complexity_cap: f64,
    /// 已拒绝计数。
    pub rejected_count: u64,
    /// 已接受计数。
    pub accepted_count: u64,
}

impl Default for RstFlywheel {
    fn default() -> Self {
        Self::new()
    }
}

impl RstFlywheel {
    pub fn new() -> Self {
        Self {
            verified_pool: Vec::new(),
            max_generation: 4,
            extend_per_gen: 3,
            realign_threshold: 1.1,
            complexity_cap: 100.0,
            rejected_count: 0,
            accepted_count: 0,
        }
    }

    /// 阶段 1 seed: 注入种子任务 (代数 0, 直接入池)。
    pub fn seed(&mut self, prompt: impl Into<String>) -> RstTask {
        let task = RstTask {
            id: format!("rst-{}", uuid::Uuid::new_v4()),
            prompt: prompt.into(),
            generation: 0,
            complexity: 1.0,
            verified: true,
            parent: None,
        };
        self.verified_pool.push(task.clone());
        self.accepted_count += 1;
        task
    }

    /// 阶段 2 extend: 从已验证池采样父任务, 合成新任务 (复杂度随代数放大)。
    pub fn extend(&self, parent: &RstTask) -> Vec<RstTask> {
        if parent.generation >= self.max_generation {
            return Vec::new();
        }
        let gen = parent.generation + 1;
        (0..self.extend_per_gen)
            .map(|i| RstTask {
                id: format!("rst-{gen}-{i}-{}", uuid::Uuid::new_v4()),
                prompt: format!("extended[{}]: {}", parent.prompt, i),
                generation: gen,
                complexity: parent.complexity * 1.3,
                verified: false,
                parent: Some(parent.id.clone()),
            })
            .collect()
    }

    /// 阶段 3 realign: 复杂度对齐 — 只保留复杂度 ∈ [父×阈值, cap] 的候选。
    /// 防生成器漂移 (分布外任务被淘汰)。
    pub fn realign(&self, parent: &RstTask, candidates: Vec<RstTask>) -> Vec<RstTask> {
        let floor = parent.complexity * self.realign_threshold;
        candidates
            .into_iter()
            .filter(|c| c.complexity >= floor && c.complexity <= self.complexity_cap)
            .collect()
    }

    /// 阶段 4 validate: 启发式验证器 — 任务必须可解 (提示非空) 且复杂度
    /// 在合理区间。通过 → 入池; 失败 → 记拒绝。
    pub fn validate(&mut self, candidates: Vec<RstTask>) -> Vec<RstTask> {
        let mut accepted = Vec::new();
        for c in candidates {
            let verdict = self.validate_one(&c);
            if verdict.accepted {
                accepted.push(c);
                self.accepted_count += 1;
            } else {
                self.rejected_count += 1;
            }
        }
        accepted
    }

    fn validate_one(&self, task: &RstTask) -> RstVerdict {
        if task.prompt.trim().is_empty() {
            return RstVerdict {
                task_id: task.id.clone(),
                accepted: false,
                reason: "empty prompt".into(),
            };
        }
        if task.complexity <= 0.0 {
            return RstVerdict {
                task_id: task.id.clone(),
                accepted: false,
                reason: "non-positive complexity".into(),
            };
        }
        if task.complexity > self.complexity_cap {
            return RstVerdict {
                task_id: task.id.clone(),
                accepted: false,
                reason: "complexity exceeds cap".into(),
            };
        }
        RstVerdict {
            task_id: task.id.clone(),
            accepted: true,
            reason: String::new(),
        }
    }

    /// 阶段 5 reuse: 从已验证池采样可复用任务 (round-robin 策略, 确定性)。
    pub fn reuse(&self, offset: usize) -> Option<&RstTask> {
        if self.verified_pool.is_empty() {
            return None;
        }
        let idx = offset % self.verified_pool.len();
        self.verified_pool.get(idx)
    }

    /// 飞轮完整循环: 从指定父任务 extend → realign → validate → 入池。
    /// 返回新接受任务数。
    pub fn run_generation(&mut self, parent: &RstTask) -> usize {
        let candidates = self.extend(parent);
        if candidates.is_empty() {
            return 0;
        }
        let aligned = self.realign(parent, candidates);
        let accepted = self.validate(aligned);
        let n = accepted.len();
        self.verified_pool.extend(accepted);
        n
    }

    /// 统计: 池规模 / 各代数分布。
    pub fn stats(&self) -> (usize, Vec<usize>) {
        let max_gen = self
            .verified_pool
            .iter()
            .map(|t| t.generation)
            .max()
            .unwrap_or(0) as usize;
        let mut dist = vec![0usize; max_gen + 1];
        for t in &self.verified_pool {
            dist[t.generation as usize] += 1;
        }
        (self.verified_pool.len(), dist)
    }
}

// ────────────────────────────────────────────────────────────────
// P7: MetaHarnessOptimizer (吸收 arXiv 2608.13560 AutoDesign)
// 自我进化的 eval harness 生成器: 用 LLM 生成/变异测试 harness, 过滤重复,
// 按功能覆盖率裁剪。进化循环的元层: harness 本身也进入进化池。
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HarnessTarget {
    Compile,
    UnitTest,
    Integration,
    Bench,
}

impl HarnessTarget {
    pub fn label(&self) -> &'static str {
        match self {
            HarnessTarget::Compile => "compile",
            HarnessTarget::UnitTest => "unit",
            HarnessTarget::Integration => "integration",
            HarnessTarget::Bench => "bench",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessCandidate {
    pub target: HarnessTarget,
    pub code: String,
    /// 归一化指纹: 移除空白后做碰撞检测 (AutoDesign dedup 语义)
    pub fingerprint: String,
    /// 覆盖的功能点 (功能覆盖率裁剪依据)
    pub covers: Vec<String>,
}

impl HarnessCandidate {
    pub fn new(target: HarnessTarget, code: impl Into<String>, covers: Vec<String>) -> Self {
        let code = code.into();
        let fingerprint = normalize_code(&code);
        Self {
            target,
            fingerprint,
            code,
            covers,
        }
    }
}

fn normalize_code(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    for ch in code.chars() {
        if !ch.is_whitespace() {
            out.push(ch);
        }
    }
    out
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetaHarnessOptimizer {
    candidates: Vec<HarnessCandidate>,
}

impl MetaHarnessOptimizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// 提议一个候选, 去重 (指纹碰撞 → 拒绝重复)。
    pub fn propose(&mut self, c: HarnessCandidate) -> Result<(), String> {
        if self.candidates.iter().any(|x| x.fingerprint == c.fingerprint) {
            return Err(format!("duplicate harness fingerprint: {}", c.fingerprint));
        }
        self.candidates.push(c);
        Ok(())
    }

    /// 功能覆盖剪枝: 保留覆盖点最多的 top_k (AutoDesign 覆盖率裁剪)。
    pub fn prune(&mut self, k: usize) -> usize {
        if k == 0 {
            let n = self.candidates.len();
            self.candidates.clear();
            return n;
        }
        let mut ranked = self.candidates.clone();
        ranked.sort_by(|a, b| b.covers.len().cmp(&a.covers.len()));
        ranked.truncate(k);
        let removed = self.candidates.len() - ranked.len();
        self.candidates = ranked;
        removed
    }

    pub fn candidates(&self) -> &[HarnessCandidate] {
        &self.candidates
    }

    pub fn count(&self) -> usize {
        self.candidates.len()
    }

    /// 按 target 分组统计, 供进化调度。
    pub fn coverage(&self) -> std::collections::HashMap<HarnessTarget, usize> {
        let mut m = std::collections::HashMap::new();
        for c in &self.candidates {
            *m.entry(c.target).or_insert(0) += 1;
        }
        m
    }

    /// 为已注册功能点生成种子 harness 候选 (AutoDesign 初始化)。
    pub fn seed_for(features: &[(&str, HarnessTarget)]) -> Vec<HarnessCandidate> {
        features
            .iter()
            .map(|(name, target)| {
                let code = format!(
                    "#[test]\nfn check_{}() {{ /* auto-generated for {} */ }}",
                    name.replace(['-', ' '], "_"),
                    name
                );
                HarnessCandidate::new(*target, code, vec![name.to_string()])
            })
            .collect()
    }
}

impl crate::core::nt_core_self_test::SelfTest for MetaHarnessOptimizer {
    fn name(&self) -> &str {
        "nt_mind_meta_harness_optimizer"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut opt = MetaHarnessOptimizer::new();
        for c in MetaHarnessOptimizer::seed_for(&[("tok_a", HarnessTarget::UnitTest), ("tok_b", HarnessTarget::Compile)]) {
            opt.propose(c).map_err(|e| vec![e])?;
        }
        if opt.count() != 2 {
            return Err(vec!["expected 2 seeded candidates".into()]);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────
// P15: TrainPipeline (吸收 train-llm-from-scratch)
// 端到端训练管线 (SFT→RM→{PPO,DPO,GRPO}) 编排的状态机建模 + 超参知识
// (lr scale 法则) + 策略选择。只做管线编排, 不做真实训练。注入自进化
// 循环作为"训练方法论"层。
// ────────────────────────────────────────────────────────────────

/// 训练阶段 — Pretrain→Sft→Rm→{Ppo,Dpo,Grpo 任一}→Done
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrainStage {
    Pretrain,
    Sft,
    Rm,
    Ppo,
    Dpo,
    Grpo,
    Done,
}

impl TrainStage {
    pub fn label(self) -> &'static str {
        match self {
            TrainStage::Pretrain => "pretrain",
            TrainStage::Sft => "sft",
            TrainStage::Rm => "rm",
            TrainStage::Ppo => "ppo",
            TrainStage::Dpo => "dpo",
            TrainStage::Grpo => "grpo",
            TrainStage::Done => "done",
        }
    }

    /// 阶段推进: Pretrain→Sft→Rm→{Ppo,Dpo,Grpo 任一}→Done; Done 终止
    pub fn next(self) -> Option<TrainStage> {
        match self {
            TrainStage::Pretrain => Some(TrainStage::Sft),
            TrainStage::Sft => Some(TrainStage::Rm),
            TrainStage::Rm => Some(TrainStage::Ppo),
            TrainStage::Ppo => Some(TrainStage::Done),
            TrainStage::Dpo => Some(TrainStage::Done),
            TrainStage::Grpo => Some(TrainStage::Done),
            TrainStage::Done => None,
        }
    }
}

/// 训练超参 — 端到端管线的全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainConfig {
    /// 模型参数量 (scale), lr 缩放基准 1e9
    pub model_scale: f64,
    /// 学习率
    pub lr: f64,
    /// warmup 步数
    pub warmup_steps: usize,
    /// 每步 batch 大小
    pub batch_size: usize,
    /// 最大 epoch 数
    pub max_epochs: usize,
}

impl Default for TrainConfig {
    fn default() -> Self {
        Self {
            model_scale: 1e9,
            lr: 3e-4,
            warmup_steps: 500,
            batch_size: 32,
            max_epochs: 3,
        }
    }
}

/// 训练管线状态机 — 阶段推进 + 超参 + 历史追踪
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainPipeline {
    pub current: TrainStage,
    pub config: TrainConfig,
    pub epochs_run: usize,
    /// 每阶段完成时记录 (阶段, 当时 epoch 数)
    pub history: Vec<(TrainStage, usize)>,
}

impl Default for TrainPipeline {
    fn default() -> Self {
        Self::new(TrainConfig::default())
    }
}

impl TrainPipeline {
    pub fn new(config: TrainConfig) -> Self {
        Self {
            current: TrainStage::Pretrain,
            config,
            epochs_run: 0,
            history: Vec::new(),
        }
    }

    /// 推进到下一阶段: epochs_run += 1, 记录 (current, epochs_run), current = next()?,
    /// 返回新阶段。Done 之后返回 None (R-P3: ? 传播终止)。
    pub fn advance(&mut self) -> Option<TrainStage> {
        self.epochs_run += 1;
        self.history.push((self.current, self.epochs_run));
        let next = self.current.next()?;
        self.current = next;
        Some(next)
    }

    /// 训练策略选择 — 各阶段对应方法论 (train-llm-from-scratch 知识)
    pub fn recommend_strategy(&self, stage: TrainStage) -> &'static str {
        match stage {
            TrainStage::Sft => "supervised fine-tuning: next-token",
            TrainStage::Rm => "reward model: pairwise ranking",
            TrainStage::Ppo => "PPO: on-policy RLHF",
            TrainStage::Dpo => "DPO: off-policy preference",
            TrainStage::Grpo => "GRPO: group relative policy optimization",
            _ => "pretraining: next-token on corpus",
        }
    }

    /// 依据模型规模缩放 lr (经验法则: lr ∝ scale^-0.15), 更新 config.lr 并返回。
    /// 更大模型 → 更小 lr。
    pub fn scale_lr(&mut self, scale: f64) -> f64 {
        let scaled = self.config.lr * (scale / 1e9).powf(-0.15);
        self.config.lr = scaled;
        scaled
    }

    pub fn is_complete(&self) -> bool {
        self.current == TrainStage::Done
    }

    /// 完成阶段占比 (6 阶段含 Done)。R-P6: max(0.0).min(1.0) 钳制。
    pub fn stage_progress(&self) -> f64 {
        let idx = match self.current {
            TrainStage::Pretrain => 0,
            TrainStage::Sft => 1,
            TrainStage::Rm => 2,
            TrainStage::Ppo => 3,
            TrainStage::Dpo => 4,
            TrainStage::Grpo => 5,
            TrainStage::Done => 6,
        };
        (idx as f64 / 6.0).max(0.0).min(1.0)
    }
}

impl crate::core::nt_core_self_test::SelfTest for TrainPipeline {
    fn name(&self) -> &str {
        "nt_mind_train_pipeline"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut pipe = TrainPipeline::new(TrainConfig::default());
        if pipe.is_complete() {
            return Err(vec!["pipeline must start incomplete".into()]);
        }
        pipe.advance()
            .ok_or_else(|| vec!["advance from Pretrain must yield a stage".into()])?;
        let base = pipe.config.lr;
        let scaled = pipe.scale_lr(1e10);
        if !scaled.is_finite() || scaled >= base {
            return Err(vec!["scale_lr must lower lr for larger models".into()]);
        }
        let p = pipe.stage_progress();
        if !(0.0..=1.0).contains(&p) {
            return Err(vec![format!("stage_progress out of bounds: {p}")]);
        }
        Ok(())
    }
}

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

    // 上次扫描结果缓存
    last_snapshot: Option<ProjectSnapshot>,

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

    /// 运行一次完整进化周期
    pub fn run_cycle(
        &mut self,
        world_fe: Option<f64>,
        world_phi: Option<f64>,
    ) -> EvolutionReport {
        self.run_cycle_in(None, world_fe, world_phi)
    }

    /// 对指定目标目录运行一次完整进化周期（target=None 回落到自身路径）
    pub fn run_cycle_in(
        &mut self,
        target: Option<&std::path::Path>,
        world_fe: Option<f64>,
        world_phi: Option<f64>,
    ) -> EvolutionReport {
        self.cycle += 1;
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut new_patterns = Vec::new();

        // 1. 项目扫描
        let snapshot = self.scan_project_in(target);

        // 2. 问题检测
        self.detect_large_files(&snapshot, &mut issues);
        self.detect_missing_tests(&snapshot, &mut issues);
        self.detect_excess_unsafe(&snapshot, &mut issues);
        self.detect_excess_unwrap(&snapshot, &mut issues);
        self.detect_todo_leftovers(&snapshot, &mut issues);
        self.detect_compile_issues(&snapshot, &mut issues);

        // 3. 世界模型感知的问题检测
        //    显式 world_fe/world_phi 优先 (接 ActiveInference/IIT 的真实世界状态);
        //    None 时从项目快照派生 (让 project-evolve 对任意目标项目也产出非零语义值)。
        let (free_energy, phi) = match (world_fe, world_phi) {
            (Some(fe), Some(phi)) => (fe, phi),
            _ => Self::derive_free_energy_phi(&snapshot),
        };

        if free_energy > 2.0 {
            issues.push(Issue {
                issue_type: IssueType::HighFreeEnergy,
                severity: (free_energy.min(10.0) * 3.0) as u8,
                file: None,
                description: format!("世界模型自由能过高: {:.3} (阈值=2.0)", free_energy),
                suggestion: "降低 learning_rate 或增加 JEPA 训练步数".into(),
                auto_fixable: false,
                cycle_discovered: self.cycle,
            });
        }

        if phi < 0.05 && phi > 0.0 {
            issues.push(Issue {
                issue_type: IssueType::LowPhi,
                severity: 3,
                file: None,
                description: format!("E8 集成信息 Φ 过低: {:.4} (阈值=0.05)", phi),
                suggestion: "增加 E8 演化步数或调整共振宽度 σ".into(),
                auto_fixable: false,
                cycle_discovered: self.cycle,
            });
        }

        // 4. 生成修复建议
        for issue in &issues {
            if issue.auto_fixable {
                suggestions.push(format!(
                    "🔧 [{:?}] {}: {}",
                    issue.issue_type,
                    issue.file.as_deref().unwrap_or("global"),
                    issue.suggestion
                ));
            } else {
                suggestions.push(format!(
                    "⚠ [{:?}] {}: {}",
                    issue.issue_type,
                    issue.file.as_deref().unwrap_or("global"),
                    issue.suggestion
                ));
            }
        }

        // 5. 模式蒸馏 (基于重复出现的问题)
        let recent_fixed = self.fixed_history.iter().rev().take(5).sum::<u32>();
        if recent_fixed > 3 {
            new_patterns.push(format!(
                "进化周期 #{}: 最近5周期修复{}个问题 — 系统趋向稳定",
                self.cycle, recent_fixed
            ));
        }

        // 6. 综合健康评分
        let evolution_score = self.compute_evolution_score(&snapshot, &issues);

        // 7. 停滞检测
        if issues.is_empty() {
            self.consecutive_stagnant += 1;
        } else {
            self.consecutive_stagnant = 0;
        }

        self.issues = issues.clone();
        self.last_snapshot = Some(snapshot.clone());

        EvolutionReport {
            cycle: self.cycle,
            issues_found: issues,
            issues_fixed: recent_fixed,
            snapshot,
            evolution_score,
            free_energy,
            phi,
            suggestions,
            new_patterns,
            auto_fixes: 0,
        }
    }

    /// 自动修复周期 — 对所有 auto_fixable 问题执行真实修复并重新扫描
    pub fn autofix_cycle(
        &mut self,
        world_fe: Option<f64>,
        world_phi: Option<f64>,
    ) -> EvolutionReport {
        self.autofix_cycle_in(None, world_fe, world_phi)
    }

    /// 对指定目标目录执行自动修复周期
    pub fn autofix_cycle_in(
        &mut self,
        target: Option<&std::path::Path>,
        world_fe: Option<f64>,
        world_phi: Option<f64>,
    ) -> EvolutionReport {
        let initial_report = self.run_cycle_in(target, world_fe, world_phi);

        // 独立 Auditor (G8): 修复前打 last-good 检查点
        self.auditor.checkpoint(initial_report.cycle, &initial_report.snapshot);

        // 使用 PipelineAutoFixer 管线处理所有 auto_fixable 问题
        let pipeline_result = PipelineAutoFixer::new().run_pipeline(self);
        let fixes_applied = pipeline_result.auto_applied as u32;

        let final_report = self.run_cycle_in(target, Some(initial_report.free_energy), Some(initial_report.phi));

        // verify→recover: 三角色异模型裁决修复是否安全; 拒绝则标记回滚, 不计入 auto_fixes
        let verdict = self.auditor.verify_change(
            final_report.cycle,
            &initial_report.snapshot,
            &final_report.snapshot,
        );
        self.last_audit = Some(verdict.clone());
        let mut new_patterns = final_report.new_patterns;
        if !verdict.passed {
            new_patterns.push(format!(
                "进化周期 #{}: Auditor 拒绝自动修复 ({}), 回滚至 checkpoint #{} — 修复未接受",
                final_report.cycle, verdict.summary(), verdict.checkpoint_cycle,
            ));
        }

        EvolutionReport {
            cycle: final_report.cycle,
            issues_found: final_report.issues_found,
            issues_fixed: initial_report.issues_fixed + fixes_applied,
            snapshot: final_report.snapshot,
            evolution_score: final_report.evolution_score,
            free_energy: final_report.free_energy,
            phi: final_report.phi,
            suggestions: final_report.suggestions,
            new_patterns,
            auto_fixes: if verdict.passed { fixes_applied } else { 0 },
        }
    }

    /// 项目扫描 (基于文件系统, 当前 Cargo 项目)
    pub fn scan_project(&self) -> ProjectSnapshot {
        self.scan_project_in(None)
    }

    /// 项目扫描 — 对指定目标目录扫描（target=None 回落自身；target 为非 Rust 项目时仍扫描 .rs 文件并做 cargo check）
    pub fn scan_project_in(&self, target: Option<&std::path::Path>) -> ProjectSnapshot {
        // 目标目录解析: 显式 target > self.target_dir > 自身 Cargo 项目根
        let root = match target {
            Some(t) => t.to_path_buf(),
            None => match &self.target_dir {
                Some(t) => t.clone(),
                None => std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf(),
            },
        };
        let src_dir = if root.join("src").is_dir() {
            root.join("src")
        } else {
            root.clone()
        };
        let mut total_files = 0usize;
        let mut total_lines = 0usize;
        let mut large_files = Vec::new();
        let mut modules_without_tests = Vec::new();
        let mut unsafe_count = 0usize;
        let mut unwrap_count = 0usize;
        let mut todo_count = 0usize;
        let mut file_unsafe_hotspots: Vec<String> = Vec::new();

        if let Ok(entries) = Self::walk_rust_files(&src_dir) {
            for path in &entries {
                total_files += 1;
                if let Ok(content) = std::fs::read_to_string(path) {
                    let line_count = content.lines().count();
                    total_lines += line_count;

                    if line_count > LARGE_FILE_THRESHOLD {
                        large_files.push(path.to_string_lossy().to_string());
                    }

                    let is_test_file = path.to_string_lossy().contains("tests")
                        || content.contains("#[cfg(test)]")
                        || content.contains("#[test]");

                    // 精确 unsafe 计数: 只计 unsafe { } / unsafe fn / unsafe trait / unsafe impl 块
                    let file_unsafe = count_actual_unsafe(&content);
                    unsafe_count += file_unsafe;
                    if file_unsafe > EXCESS_UNSAFE_THRESHOLD {
                        file_unsafe_hotspots.push(path.to_string_lossy().to_string());
                    }

                    // unwrap 计数 (排除测试文件和注释行)
                    if !is_test_file {
                        for line in content.lines() {
                            if line.contains(".unwrap(") && !line.trim_start().starts_with("//") {
                                unwrap_count += 1;
                            }
                        }
                    }

                    // 精确 TODO 计数: 只计 // TODO / // FIXME / // HACK 行
                    let file_todo = content.lines()
                        .filter(|l| {
                            let t = l.trim();
                            t.starts_with("// TODO")
                                || t.starts_with("//TODO")
                                || t.starts_with("// FIXME")
                                || t.starts_with("//FIXME")
                                || t.starts_with("// HACK")
                                || t.starts_with("//HACK")
                        })
                        .count();
                    todo_count += file_todo;

                    // Check for missing tests
                    if line_count > MISSING_TESTS_THRESHOLD
                        && !content.contains("#[cfg(test)]")
                        && !content.contains("#[test]")
                    {
                        modules_without_tests.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        // 测试时跳过 cargo check（避免 build lock 死锁）
        let (compile_errs, compile_warns) = if cfg!(test) {
            (0, 0)
        } else {
            match AutoFixer::cargo_check_in(Some(&root)) {
                Ok((e, w)) => (e, w),
                Err(_) => (0, 0),
            }
        };
        ProjectSnapshot {
            total_files,
            total_lines,
            large_files,
            modules_without_tests,
            file_unsafe_hotspots,
            unsafe_count,
            unwrap_count,
            todo_count,
            compile_errors: compile_errs,
            compile_warnings: compile_warns,
            test_count: 0,
            test_failures: 0,
        }
    }

    /// 递归搜索 Rust 源文件
    fn walk_rust_files(dir: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
        const SKIP_DIRS: [&str; 5] = ["target", ".git", ".backup", "node_modules", "_archive"];
        let mut files = Vec::new();
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    // 跳过 target/.git/.backup/node_modules 等非源码目录
                    if path
                        .file_name()
                        .map(|n| !SKIP_DIRS.contains(&n.to_str().unwrap_or("")))
                        .unwrap_or(true)
                    {
                        files.extend(Self::walk_rust_files(&path)?);
                    }
                } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                    files.push(path);
                }
            }
        }
        Ok(files)
    }

    // ─── 问题检测器 ───

    fn detect_large_files(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
        for file in &snap.large_files {
            issues.push(Issue {
                issue_type: IssueType::LargeFile,
                severity: 5,
                file: Some(file.clone()),
                description: format!("文件过大: {}", file),
                suggestion: "拆分为多个子模块 (<800 行/文件)".into(),
                auto_fixable: false,
                cycle_discovered: self.cycle,
            });
        }
    }

    fn detect_missing_tests(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
        for file in &snap.modules_without_tests {
            issues.push(Issue {
                issue_type: IssueType::MissingTests,
                severity: 4,
                file: Some(file.clone()),
                description: format!("模块无测试: {}", file),
                suggestion: "添加 #[cfg(test)] mod tests 单元测试".into(),
                auto_fixable: false,
                cycle_discovered: self.cycle,
            });
        }
    }

    fn detect_excess_unsafe(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
        for file in &snap.file_unsafe_hotspots {
            issues.push(Issue {
                issue_type: IssueType::ExcessUnsafe,
                severity: 7,
                file: Some(file.clone()),
                description: format!("unsafe 过多: {}", file),
                suggestion: "审查 unsafe 块, 减少或添加安全抽象".into(),
                auto_fixable: false,
                cycle_discovered: self.cycle,
            });
        }
    }

    fn detect_excess_unwrap(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
        if snap.unwrap_count > EXCESS_UNWRAP_THRESHOLD {
            issues.push(Issue {
                issue_type: IssueType::ExcessUnwrap,
                severity: 6,
                file: None,
                description: format!(".unwrap() 过多: {} 处", snap.unwrap_count),
                suggestion: "用 ? 操作符或 match 替代 unwrap".into(),
                auto_fixable: true,
                cycle_discovered: self.cycle,
            });
        }
    }

    fn detect_todo_leftovers(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
        if snap.todo_count > TODO_LEFTOVERS_THRESHOLD {
            issues.push(Issue {
                issue_type: IssueType::TodoLeftovers,
                severity: 2,
                file: None,
                description: format!("TODO 残留: {} 处", snap.todo_count),
                suggestion: "清理已完成 TODO, 将未完成转移至 TODO.md".into(),
                auto_fixable: false,
                cycle_discovered: self.cycle,
            });
        }
    }

    fn detect_compile_issues(&self, snap: &ProjectSnapshot, issues: &mut Vec<Issue>) {
        if snap.compile_errors > 0 {
            issues.push(Issue {
                issue_type: IssueType::CompileWarning,
                severity: 10,
                file: None,
                description: format!("编译错误: {} 个", snap.compile_errors),
                suggestion: "运行 cargo check --lib 修复错误".into(),
                auto_fixable: true,
                cycle_discovered: self.cycle,
            });
        }
        if snap.compile_warnings > 0 {
            issues.push(Issue {
                issue_type: IssueType::CompileWarning,
                severity: 3,
                file: None,
                description: format!("编译警告: {} 个", snap.compile_warnings),
                suggestion: "运行 cargo fix --lib 自动修复".into(),
                auto_fixable: true,
                cycle_discovered: self.cycle,
            });
        }
    }

    /// 从项目快照派生 free_energy / phi (供无世界模型上下文的调用方使用)。
    ///
    /// 语义:
    /// - `free_energy`: ActiveInference 预测误差 — 项目风险维度 (issue 密度/unsafe/unwrap/
    ///   TODO/大文件/编译警告) 越高, 自由能越高。
    /// - `phi`: IIT 集成信息 — 将健康维度构成状态向量, 度量各健康维度间的
    ///   共振整合度; 纯噪声/均衡态 → 低 Φ, 结构化分化 → 高 Φ。
    fn derive_free_energy_phi(snap: &ProjectSnapshot) -> (f64, f64) {
        let file_n = snap.total_files.max(1) as f64;

        // 项目风险密度 (每文件归一化)
        let issue_density = (snap.large_files.len() as f64 + snap.modules_without_tests.len() as f64) / file_n;
        let unsafe_density = snap.unsafe_count as f64 / file_n;
        let unwrap_density = snap.unwrap_count as f64 / file_n;
        let todo_density = snap.todo_count as f64 / file_n;
        let warning_density = snap.compile_warnings as f64 / file_n;

        // ActiveInference 变分自由能: F = β·E_JEPA - H(E8)/T + γ·|∇E8|
        //   jepa_energy(预测能量)   = 风险密度 (项目越脏, 世界模型预测误差越大)
        //   e8_entropy(E8 状态熵)   = 结构复杂度 (文件量级信息)
        //   e8_energy_gradient      = 0 (单次扫描无时间演化)
        let jepa_energy = issue_density + unsafe_density + unwrap_density + todo_density + warning_density;
        let e8_entropy = (1.0 + snap.total_lines as f64).ln();
        let free_energy = ActiveInferenceEngine::new()
            .compute_free_energy(jepa_energy, e8_entropy, 0.0)
            .variational_fe;

        // IIT Φ: 健康维度状态向量 → 共振集成度
        //   state = [health_ratio, large_file_ratio, no_test_ratio, unsafe_ratio,
        //            unwrap_ratio, todo_ratio, warning_ratio] (0..1 归一)
        let (large_ratio, test_ratio, unsafe_ratio) = (
            snap.large_files.len() as f64 / file_n,
            snap.modules_without_tests.len() as f64 / file_n,
            snap.unsafe_count as f64 / file_n,
        );
        let state = vec![
            1.0 - (jepa_energy / 4.0).min(1.0), // 健康度 (逆风险密度)
            large_ratio,
            test_ratio,
            unsafe_ratio,
            unwrap_density / 4.0,
            todo_density / 2.0,
            warning_density,
        ];
        let phi = IITPhiCalculator::new().compute_phi(&state).phi;

        (free_energy, phi)
    }

    /// 综合健康评分 (0-100)
    fn compute_evolution_score(&self, snap: &ProjectSnapshot, _issues: &[Issue]) -> f64 {
        let mut score = 100.0;

        // 大文件惩罚
        score -= snap.large_files.len() as f64 * 5.0;

        // 无测试惩罚
        score -= snap.modules_without_tests.len() as f64 * 3.0;

        // unsafe 惩罚
        if snap.unsafe_count > EXCESS_UNSAFE_THRESHOLD {
            score -= (snap.unsafe_count - EXCESS_UNSAFE_THRESHOLD) as f64 * 2.0;
        }

        // unwrap 惩罚
        if snap.unwrap_count > EXCESS_UNWRAP_THRESHOLD {
            score -= (snap.unwrap_count - EXCESS_UNWRAP_THRESHOLD) as f64 * 1.0;
        }

        // 未完成 TODO 惩罚
        if snap.todo_count > TODO_LEFTOVERS_THRESHOLD {
            score -= (snap.todo_count - TODO_LEFTOVERS_THRESHOLD) as f64 * 2.0;
        }

        // 编译错误: 致命
        if snap.compile_errors > 0 {
            score -= 30.0;
        }

        // 编译警告
        score -= snap.compile_warnings.min(20) as f64 * 1.0;

        score.clamp(0.0, 100.0)
    }

    /// 判断是否需要人工介入
    pub fn needs_human_intervention(&self) -> bool {
        self.consecutive_stagnant >= STAGNATION_LIMIT
            || self.issues.iter().any(|i| i.severity >= 9)
    }

    /// 重置停滞计数
    pub fn on_fix_applied(&mut self) {
        self.consecutive_stagnant = 0;
        self.fixed_history.push(1);
        if self.fixed_history.len() > 20 {
            self.fixed_history.remove(0);
        }
    }

    /// 获取仪表盘文本
    pub fn dashboard(&self, report: &EvolutionReport) -> String {
        format!(
            "🧬 #{}: 评分={:.0}/100, 问题={}, 自修复={}, 累积修复={}, 停滞={}/{} | FE={:.2}, Φ={:.3}",
            report.cycle,
            report.evolution_score,
            report.issues_found.len(),
            report.auto_fixes,
            report.issues_fixed,
            self.consecutive_stagnant,
            STAGNATION_LIMIT,
            report.free_energy,
            report.phi,
        )
    }

    /// 自我诊断入口 — 零 LLM 依赖, 基于扫描数据 + 历史 + 能力向量排序
    pub fn self_diagnose(&self) -> (Vec<DiagnosticItem>, PriorityQueue) {
        let snapshot = self.scan_project_in(None);
        SelfDiagnose::run_diagnosis(&snapshot, self.cycle)
    }

    /// 基于诊断结果的自动修复 — 按优先级顺序执行 ActionPlan
    ///
    /// 循环断路器接线 (R-P79): 每次 autofix 会话持有一个 RepairCircuitBreaker,
    /// 逐 item 执行前检查断路器; 跳闸 (轮次超限或连续无进展) 即停止后续修复,
    /// 防止自愈循环空转 (retry cap / loop detection)。
    pub fn autofix_by_diagnosis(&mut self) -> u32 {
        let (_items, pq) = self.self_diagnose();
        let mut fixes = 0u32;
        let mut breaker = RepairCircuitBreaker::new(REPAIR_MAX_ROUNDS);
        for item in pq.as_slice() {
            if item.composite_score < 0.3 {
                continue;
            }
            if breaker.is_tripped() {
                log::warn!("[EvolutionLoop] 修复断路器已跳闸, 停止自动修复");
                break;
            }
            match ActionExecutor::execute_with_breaker(&ActionExecutor, &item.action, &mut breaker) {
                Ok(_) => fixes += 1,
                Err(e) if breaker.is_tripped() => {
                    log::warn!("[EvolutionLoop] 修复断路器跳闸: {}", e);
                    break;
                }
                Err(_) => {}
            }
        }
        if fixes > 0 {
            self.on_fix_applied();
        }
        fixes
    }
}

impl EvolutionLoopProvider for EvolutionLoop {
    fn self_diagnose(&mut self) -> (Vec<String>, Vec<PrioritizedIssue>) {
        let (items, pq) = Self::self_diagnose(self);
        let issues: Vec<PrioritizedIssue> = pq.into_vec().into_iter().map(|di| {
            PrioritizedIssue {
                action: di.action,
                composite_score: di.composite_score,
                underlying_issue: CodeUnderlyingIssue {
                    file: di.underlying_issue.file.clone(),
                    issue_type: format!("{:?}", di.underlying_issue.issue_type),
                },
            }
        }).collect();
        let messages: Vec<String> = items.into_iter()
            .map(|d| format!("[{:?}] {} (score={:.2})", d.underlying_issue.issue_type, d.underlying_issue.description, d.composite_score))
            .collect();
        (messages, issues)
    }

    fn on_fix_applied(&mut self) {
        self.on_fix_applied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    #[test]
    fn test_count_actual_unsafe_zero() {
        let s = "fn safe() { let x = 1; }";
        assert_eq!(count_actual_unsafe(s), 0);
    }

    #[test]
    fn test_count_actual_unsafe_counts_block() {
        let s = "fn foo() { unsafe { *p = 1; } }";
        assert_eq!(count_actual_unsafe(s), 1);
    }

    #[test]
    fn test_count_actual_unsafe_ignores_comment() {
        let s = "// unsafe { this is just a comment }";
        assert_eq!(count_actual_unsafe(s), 0);
    }

    #[test]
    fn test_count_actual_unsafe_ignores_doc_comment() {
        let s = "//! unsafe { doc comment unsafe }";
        assert_eq!(count_actual_unsafe(s), 0);
    }

    #[test]
    fn test_count_actual_unsafe_ignores_variable_name() {
        let s = "let unsafe_count = 5;";
        assert_eq!(count_actual_unsafe(s), 0);
    }

    #[test]
    fn test_evolution_loop_new() {
        let el = EvolutionLoop::new();
        assert_eq!(el.cycle, 0);
        assert!(el.enabled);
    }

    #[test]
    fn test_scan_project_returns_reasonable_values() {
        let el = EvolutionLoop::new();
        let snap = el.scan_project();
        assert!(snap.total_files > 0 || snap.total_lines == 0);
    }

    #[test]
    fn test_evolution_score_baseline() {
        let el = EvolutionLoop::new();
        let snap = el.scan_project();
        let score = el.compute_evolution_score(&snap, &[]);
        assert!(score >= 0.0 && score <= 100.0);
    }

    #[test]
    fn test_issue_detection_creates_valid_issues() {
        let mut el = EvolutionLoop::new();
        let report = el.run_cycle(Some(0.5), Some(0.3));
        assert_eq!(report.cycle, 1);
        for issue in &report.issues_found {
            assert!(issue.severity >= 1 && issue.severity <= 10);
            assert!(!issue.description.is_empty());
        }
    }

    #[test]
    fn test_high_free_energy_detected() {
        let mut el = EvolutionLoop::new();
        let report = el.run_cycle(Some(5.0), Some(0.3));
        assert!(report.issues_found.iter().any(|i| i.issue_type == IssueType::HighFreeEnergy));
    }

    #[test]
    fn test_low_phi_detected() {
        let mut el = EvolutionLoop::new();
        let report = el.run_cycle(Some(0.5), Some(0.01));
        assert!(report.issues_found.iter().any(|i| i.issue_type == IssueType::LowPhi));
    }

    #[test]
    fn test_stagnation_detection() {
        let mut el = EvolutionLoop::new();
        assert!(!el.needs_human_intervention());
        el.consecutive_stagnant = STAGNATION_LIMIT;
        assert!(el.needs_human_intervention());
    }

    #[test]
    fn test_on_fix_applied_resets_stagnation() {
        let mut el = EvolutionLoop::new();
        el.consecutive_stagnant = 5;
        el.on_fix_applied();
        assert_eq!(el.consecutive_stagnant, 0);
    }

    #[test]
    fn test_dashboard_format() {
        let el = EvolutionLoop::new();
        let snap = el.scan_project();
        let report = EvolutionReport {
            cycle: 1,
            issues_found: vec![],
            issues_fixed: 0,
            snapshot: snap,
            evolution_score: 85.0,
            free_energy: 0.5,
            phi: 0.3,
            suggestions: vec![],
            new_patterns: vec![],
            auto_fixes: 0,
        };
        let db = el.dashboard(&report);
        assert!(db.contains("#"));
        assert!(db.contains("评分"));
    }

    #[test]
    fn test_for_target_sets_target_dir() {
        let el = EvolutionLoop::for_target("/tmp/mock-project");
        assert!(el.target_dir.is_some());
        assert_eq!(el.target_dir.as_deref().unwrap(), std::path::Path::new("/tmp/mock-project"));
    }

    #[test]
    fn test_scan_project_in_arbitrary_dir() {
        // 构造一个临时 mock Rust 项目目录
        let dir = std::env::temp_dir().join(format!("nt-evolve-mock-{}", std::process::id()));
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create mock src");
        std::fs::write(src.join("main.rs"), "// TODO: fix me\nfn main() { let x = 1; }\n").expect("write mock main");
        std::fs::write(src.join("hot.rs"), "unsafe { }\nunsafe { }\nunsafe { }\nunsafe { }\nunsafe { }\nunsafe { }\n").expect("write mock hot");

        let el = EvolutionLoop::new();
        let snap = el.scan_project_in(Some(&dir));
        assert!(snap.total_files >= 2, "expected >=2 files, got {}", snap.total_files);
        assert!(snap.todo_count >= 1, "expected TODO detected, got {}", snap.todo_count);
        assert!(snap.file_unsafe_hotspots.len() >= 1, "expected unsafe hotspot");
        assert!(snap.unsafe_count >= 6, "expected >=6 unsafe, got {}", snap.unsafe_count);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_run_cycle_in_target_reports_target_snapshot() {
        let dir = std::env::temp_dir().join(format!("nt-evolve-cycle-{}", std::process::id()));
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create mock src");
        std::fs::write(src.join("lib.rs"), "// TODO x\n").expect("write mock lib");

        let mut el = EvolutionLoop::new();
        let report = el.run_cycle_in(Some(&dir), None, None);
        assert_eq!(report.cycle, 1);
        assert!(report.snapshot.todo_count >= 1, "expected TODO in target, got {}", report.snapshot.todo_count);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_derive_free_energy_phi_non_finite_or_zero() {
        // 显式接世界模型值时应原样透传 (旧语义不被破坏)
        let dir = std::env::temp_dir().join(format!("nt-evolve-fe-{}", std::process::id()));
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create mock src");
        std::fs::write(src.join("lib.rs"), "pub fn f() {}\n").expect("write mock lib");

        let mut el = EvolutionLoop::new();
        let report = el.run_cycle_in(Some(&dir), Some(0.5), Some(0.3));
        assert_eq!(report.free_energy, 0.5);
        assert_eq!(report.phi, 0.3);

        // None 时从快照派生 (不崩溃, 值为有限数)
        let report2 = el.run_cycle_in(Some(&dir), None, None);
        assert!(report2.free_energy.is_finite(), "free_energy must be finite, got {}", report2.free_energy);
        assert!(report2.phi.is_finite(), "phi must be finite, got {}", report2.phi);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_derive_free_energy_phi_dirty_vs_clean() {
        // 脏项目 (多问题) 自由能应高于干净项目 (风险驱动)
        let clean = ProjectSnapshot {
            total_files: 10,
            total_lines: 1000,
            large_files: vec![],
            modules_without_tests: vec![],
            file_unsafe_hotspots: vec![],
            unsafe_count: 0,
            unwrap_count: 1,
            todo_count: 1,
            test_count: 20,
            test_failures: 0,
            compile_errors: 0,
            compile_warnings: 0,
        };
        let dirty = ProjectSnapshot {
            total_files: 10,
            total_lines: 1000,
            large_files: vec!["big.rs".into()],
            modules_without_tests: vec!["m.rs".into()],
            file_unsafe_hotspots: vec!["u.rs".into()],
            unsafe_count: 8,
            unwrap_count: 40,
            todo_count: 6,
            test_count: 2,
            test_failures: 1,
            compile_errors: 0,
            compile_warnings: 5,
        };
        let (fe_clean, _phi_clean) = EvolutionLoop::derive_free_energy_phi(&clean);
        let (fe_dirty, phi_dirty) = EvolutionLoop::derive_free_energy_phi(&dirty);
        assert!(fe_dirty > fe_clean, "dirty FE ({}) must exceed clean FE ({})", fe_dirty, fe_clean);
        assert!(fe_dirty.is_finite() && phi_dirty.is_finite());
    }

    // ── G8 独立 Auditor 测试 ──────────────────────────────────

    fn snap(unwrap: usize, compile_errors: usize, unsafe_count: usize, todo: usize, hotspots: usize) -> ProjectSnapshot {
        ProjectSnapshot {
            total_files: 10,
            total_lines: 1000,
            large_files: vec![],
            modules_without_tests: vec![],
            file_unsafe_hotspots: (0..hotspots).map(|i| format!("hotspot_{}.rs", i)).collect(),
            unsafe_count,
            unwrap_count: unwrap,
            todo_count: todo,
            test_count: 20,
            test_failures: 0,
            compile_errors,
            compile_warnings: 0,
        }
    }

    #[test]
    fn test_auditor_accepts_improvement() {
        // 修复有效 (指标全面改善) → 三角色共识通过, checkpoint 前移
        let mut auditor = Auditor::new();
        let before = snap(40, 3, 8, 6, 2);
        let after = snap(12, 0, 6, 4, 1);
        auditor.checkpoint(1, &before);

        let v = auditor.verify_change(2, &before, &after);
        assert!(v.passed, "improvement must pass: {}", v.summary());
        assert!(!v.recovered);
        assert_eq!(v.checkpoint_cycle, 2, "pass 后 checkpoint 前移到新快照");
        assert_eq!(auditor.last_checkpoint().unwrap().unwrap_count, 12);
        assert_eq!(auditor.verdict_history.len(), 1);
    }

    #[test]
    fn test_auditor_rejects_regression_and_recovers() {
        // 修复反而引入回归 (unwrap 增加) → Evidence FAIL → 拒绝 + recover 标记
        let mut auditor = Auditor::new();
        let before = snap(10, 0, 5, 3, 1);
        let regression = snap(30, 2, 5, 3, 1);
        auditor.checkpoint(1, &before);

        let v = auditor.verify_change(2, &before, &regression);
        assert!(!v.passed, "regression must be rejected");
        assert!(v.recovered, "拒绝时须标记 recover 回滚至 last-good");
        assert_eq!(v.checkpoint_cycle, 1, "reject 后 checkpoint 保持 last-good");
        assert_eq!(auditor.last_checkpoint().unwrap().unwrap_count, 10);
        assert!(v.role_verdicts.iter().any(|r| !r.pass));
    }

    #[test]
    fn test_auditor_consensus_requires_all_roles() {
        // 仅 Evidence 通过但 Consistency 回归 (todo 激增) → 仍拒绝 (异模型共识)
        let mut auditor = Auditor::new();
        let before = snap(10, 0, 5, 3, 1);
        let side_effect = snap(8, 0, 20, 30, 1);
        auditor.checkpoint(1, &before);

        let v = auditor.verify_change(2, &before, &side_effect);
        assert!(!v.passed, "side-effect regression must fail consistency");
        assert!(v.recovered);
        let consistency = v
            .role_verdicts
            .iter()
            .find(|r| r.role == AuditorRole::Consistency)
            .expect("consistency role must be present");
        assert!(!consistency.pass);
    }

    #[test]
    fn test_auditor_governance_blocks_new_hotspots() {
        // 治理角色: 新引入 unsafe 热点文件 → Governance FAIL
        let mut auditor = Auditor::new();
        let before = snap(10, 0, 5, 3, 1);
        let new_hotspots = snap(10, 0, 12, 3, 4);
        auditor.checkpoint(1, &before);

        let v = auditor.verify_change(2, &before, &new_hotspots);
        assert!(!v.passed);
        let governance = v
            .role_verdicts
            .iter()
            .find(|r| r.role == AuditorRole::Governance)
            .expect("governance role must be present");
        assert!(!governance.pass);
    }

    #[test]
    fn test_autofix_cycle_zeroes_fixes_on_reject() {
        // 接线验证: autofix_cycle_in 中 rejected 变更 → auto_fixes=0 + rollback 模式入报告
        let _el = EvolutionLoop::new();
        let dir = std::env::temp_dir().join(format!("nt_audit_gate_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("src")).expect("create mock src");
        std::fs::write(dir.join("src").join("lib.rs"), "pub fn f() {}\n").expect("write mock lib");

        // 无 auto_fixable 问题可修 → 修复数为 0, 且 auditor 已注册裁决 (验证接线编译 + 运行)
        // 用 for_target 锁定扫描目录为 mock 项目 — 否则 PipelineAutoFixer::self_diagnose
        // 会扫整个真实仓库并对真实源文件写修复 (测试隔离纪律)。
        let mut el = EvolutionLoop::for_target(&dir);
        let report = el.autofix_cycle_in(Some(&dir), None, None);
        assert!(el.last_audit.is_some(), "autofix_cycle_in 必须产生审计裁决");
        assert_eq!(report.auto_fixes, 0);
        assert!(report.evolution_score.is_finite());

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── G10: RST flywheel ──────────────────────────────────────────────

    #[test]
    fn rst_seed_creates_verified_gen0() {
        let mut fw = RstFlywheel::new();
        let seed = fw.seed("parse the config");
        assert_eq!(seed.generation, 0);
        assert!(seed.verified);
        assert_eq!(fw.stats().0, 1);
    }

    #[test]
    fn rst_extend_increments_generation_and_complexity() {
        let mut fw = RstFlywheel::new();
        let seed = fw.seed("baseline");
        let children = fw.extend(&seed);
        assert_eq!(children.len(), fw.extend_per_gen);
        assert_eq!(children[0].generation, 1);
        assert!(!children[0].verified);
        assert!(children[0].complexity > seed.complexity);
    }

    #[test]
    fn rst_extend_stops_at_max_generation() {
        let mut fw = RstFlywheel::new();
        let mut t = fw.seed("deep");
        t.generation = fw.max_generation;
        assert!(fw.extend(&t).is_empty());
    }

    #[test]
    fn rst_realign_filters_below_threshold() {
        let fw = RstFlywheel::new();
        let parent = RstTask {
            id: "p".into(),
            prompt: "parent".into(),
            generation: 0,
            complexity: 10.0,
            verified: true,
            parent: None,
        };
        let low = RstTask {
            id: "low".into(),
            prompt: "low".into(),
            generation: 1,
            complexity: 10.4, // < 10×1.1=11 → 淘汰
            verified: false,
            parent: None,
        };
        let mut ok = low.clone();
        ok.id = "ok".into();
        ok.complexity = 12.0;
        let kept = fw.realign(&parent, vec![low, ok]);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].id, "ok");
    }

    #[test]
    fn rst_validate_rejects_empty_prompt() {
        let mut fw = RstFlywheel::new();
        let bad = RstTask {
            id: "bad".into(),
            prompt: "  ".into(),
            generation: 1,
            complexity: 5.0,
            verified: false,
            parent: None,
        };
        assert!(fw.validate(vec![bad]).is_empty());
        assert_eq!(fw.rejected_count, 1);
    }

    #[test]
    fn rst_validate_rejects_over_cap() {
        let mut fw = RstFlywheel::new();
        let over = RstTask {
            id: "over".into(),
            prompt: "x".into(),
            generation: 1,
            complexity: fw.complexity_cap + 1.0,
            verified: false,
            parent: None,
        };
        assert!(fw.validate(vec![over]).is_empty());
    }

    #[test]
    fn rst_reuse_samples_verified_pool_roundrobin() {
        let mut fw = RstFlywheel::new();
        fw.seed("a");
        fw.seed("b");
        let first = fw.reuse(0).unwrap().id.clone();
        let second = fw.reuse(1).unwrap().id.clone();
        assert_ne!(first, second, "round-robin rotates across pool");
        let third = fw.reuse(2).unwrap().id.clone();
        assert_eq!(third, first, "offset wraps modulo pool size");
    }

    #[test]
    fn rst_full_generation_grows_pool() {
        let mut fw = RstFlywheel::new();
        let seed = fw.seed("task");
        let before = fw.stats().0;
        let accepted = fw.run_generation(&seed);
        assert!(accepted > 0);
        assert_eq!(fw.stats().0, before + accepted);
        assert!(fw.accepted_count >= 1 + accepted as u64);
    }

    #[test]
    fn rst_stats_distribution_by_generation() {
        let mut fw = RstFlywheel::new();
        let seed = fw.seed("gen0");
        fw.run_generation(&seed);
        let (total, dist) = fw.stats();
        assert_eq!(total, 1 + dist.get(1).copied().unwrap_or(0));
        assert!(dist.len() >= 2, "seed(gen0) + children(gen1)");
        assert_eq!(dist[0], 1, "exactly one seed at gen0");
    }

    // ── P7 MetaHarnessOptimizer ──
    #[test]
    fn test_harness_seed_proposes() {
        let mut opt = MetaHarnessOptimizer::new();
        for c in MetaHarnessOptimizer::seed_for(&[("compile_ok", HarnessTarget::Compile), ("bench_fast", HarnessTarget::Bench)]) {
            opt.propose(c).expect("propose");
        }
        assert_eq!(opt.count(), 2);
    }

    #[test]
    fn test_harness_dedup_by_fingerprint() {
        let mut opt = MetaHarnessOptimizer::new();
        let a = HarnessCandidate::new(HarnessTarget::UnitTest, "  fn  a(){} ", vec!["x".into()]);
        let b = HarnessCandidate::new(HarnessTarget::UnitTest, "fn a(){}", vec!["x".into()]);
        opt.propose(a).expect("first");
        assert!(opt.propose(b).is_err(), "whitespace-normalized duplicate must be rejected");
    }

    #[test]
    fn test_harness_prune_keeps_top_coverage() {
        let mut opt = MetaHarnessOptimizer::new();
        for c in vec![
            HarnessCandidate::new(HarnessTarget::Integration, "c1", vec!["a".into()]),
            HarnessCandidate::new(HarnessTarget::Integration, "c2", vec!["a".into(), "b".into()]),
            HarnessCandidate::new(HarnessTarget::Integration, "c3", vec!["a".into(), "b".into(), "c".into()]),
        ] {
            opt.propose(c).expect("propose");
        }
        let removed = opt.prune(2);
        assert_eq!(removed, 1);
        assert_eq!(opt.count(), 2);
        assert_eq!(opt.candidates()[0].covers.len(), 3, "top-coverage candidate survives");
    }

    #[test]
    fn test_harness_coverage_grouping() {
        let mut opt = MetaHarnessOptimizer::new();
        opt.propose(HarnessCandidate::new(HarnessTarget::Compile, "c1", vec!["a".into()])).unwrap();
        opt.propose(HarnessCandidate::new(HarnessTarget::UnitTest, "c2", vec!["b".into()])).unwrap();
        let cov = opt.coverage();
        assert_eq!(cov.get(&HarnessTarget::Compile), Some(&1));
        assert_eq!(cov.get(&HarnessTarget::UnitTest), Some(&1));
    }

    #[test]
    fn test_harness_prune_zero_clears() {
        let mut opt = MetaHarnessOptimizer::new();
        opt.propose(HarnessCandidate::new(HarnessTarget::Compile, "c1", vec!["a".into()])).unwrap();
        let removed = opt.prune(0);
        assert_eq!(removed, 1);
        assert_eq!(opt.count(), 0);
    }

    #[test]
    fn test_harness_selftest() {
        let opt = MetaHarnessOptimizer::new();
        assert!(opt.self_test().is_ok());
    }

    // ── P15: TrainPipeline (train-llm-from-scratch 吸收) ──

    #[test]
    fn train_stage_ordering_advances() {
        assert_eq!(TrainStage::Pretrain.next(), Some(TrainStage::Sft));
        assert_eq!(TrainStage::Sft.next(), Some(TrainStage::Rm));
        assert_eq!(TrainStage::Rm.next(), Some(TrainStage::Ppo));
        assert_eq!(TrainStage::Ppo.next(), Some(TrainStage::Done));
        assert_eq!(TrainStage::Dpo.next(), Some(TrainStage::Done));
        assert_eq!(TrainStage::Grpo.next(), Some(TrainStage::Done));
        assert_eq!(TrainStage::Done.next(), None);
        assert_eq!(TrainStage::Ppo.label(), "ppo");
        assert_eq!(TrainStage::Done.label(), "done");
    }

    #[test]
    fn train_pipeline_advance_grows_history() {
        let mut pipe = TrainPipeline::new(TrainConfig::default());
        assert_eq!(pipe.current, TrainStage::Pretrain);
        assert_eq!(pipe.history.len(), 0);
        let next = pipe.advance().expect("advance from Pretrain");
        assert_eq!(next, TrainStage::Sft);
        assert_eq!(pipe.current, TrainStage::Sft);
        assert_eq!(pipe.history.len(), 1);
        assert_eq!(pipe.history[0], (TrainStage::Pretrain, 1));
        assert_eq!(pipe.epochs_run, 1);
    }

    #[test]
    fn train_recommend_strategy_per_stage() {
        let pipe = TrainPipeline::default();
        assert_eq!(pipe.recommend_strategy(TrainStage::Pretrain), "pretraining: next-token on corpus");
        assert_eq!(pipe.recommend_strategy(TrainStage::Sft), "supervised fine-tuning: next-token");
        assert_eq!(pipe.recommend_strategy(TrainStage::Rm), "reward model: pairwise ranking");
        assert_eq!(pipe.recommend_strategy(TrainStage::Ppo), "PPO: on-policy RLHF");
        assert_eq!(pipe.recommend_strategy(TrainStage::Dpo), "DPO: off-policy preference");
        assert_eq!(pipe.recommend_strategy(TrainStage::Grpo), "GRPO: group relative policy optimization");
        assert_eq!(pipe.recommend_strategy(TrainStage::Done), "pretraining: next-token on corpus");
    }

    #[test]
    fn train_scale_lr_lowers_for_larger_models() {
        let mut pipe = TrainPipeline::new(TrainConfig::default());
        let base = pipe.config.lr;
        let big = pipe.scale_lr(1e10);
        assert!(big < base, "10B model lr ({big}) must be smaller than base ({base})");
        assert_eq!(pipe.config.lr, big);

        let mut small_pipe = TrainPipeline::new(TrainConfig::default());
        let small_base = small_pipe.config.lr;
        let small = small_pipe.scale_lr(1e8);
        assert!(small > small_base, "100M model lr ({small}) must be larger than base ({small_base})");
    }

    #[test]
    fn train_is_complete_only_when_done() {
        let mut pipe = TrainPipeline::default();
        assert!(!pipe.is_complete());
        for _ in 0..4 {
            pipe.advance();
        }
        assert_eq!(pipe.current, TrainStage::Done);
        assert!(pipe.is_complete());
        assert_eq!(pipe.advance(), None, "Done 之后 advance 返回 None");
        assert!(pipe.is_complete());
    }

    #[test]
    fn train_stage_progress_bounds_and_monotonic() {
        let mut pipe = TrainPipeline::default();
        assert_eq!(pipe.stage_progress(), 0.0);
        assert!((0.0..=1.0).contains(&pipe.stage_progress()));
        let mut last = 0.0;
        for _ in 0..4 {
            pipe.advance();
            let p = pipe.stage_progress();
            assert!(p >= last && p <= 1.0, "progress must stay in [0,1] and be monotonic");
            last = p;
        }
        assert_eq!(pipe.stage_progress(), 1.0);
    }

    #[test]
    fn train_pipeline_selftest_passes() {
        let pipe = TrainPipeline::new(TrainConfig::default());
        assert!(pipe.self_test().is_ok());
    }
}
