//! nt_mind_repair — 自愈修复计划与恢复验证 (R-P42 吸收, 非平行重造)
//!
//! 契约驱动实现:
//! - repair_plan: 基于根因假设的修复策略生成、风险评估、策略选择
//! - recovery_verify: 修复结果验证、模式学习、已知陷阱晋升
//!
//! 参考: GenProg/Prophet/Angelix/SelRepair/PathFix/RePair/T³/AdaPatcher 等 SoTA APR 系统

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use neotrix_types::shared::Severity;

/// 根因假设
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _RootCauseHypothesis {
    pub hypothesis: String,
    pub confidence: f32,           // 0.0-1.0
    pub evidence: Vec<String>,
    pub affected_modules: Vec<String>,
}

/// 诊断信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnosis {
    pub description: String,
    pub root_causes: Vec<_RootCauseHypothesis>,
    pub severity: Severity,
}

/// 严重度
/// 修复选项类型 (参考 APR 文献: GenProg/Prophet/Template/LLM-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum _RepairStrategyType {
    /// 回滚到已知良好版本
    Rollback,
    /// 补丁修复 (基于模式/模板/LLM)
    Patch,
    /// 重新配置 (特性开关/配置调整)
    Reconfigure,
    /// 重启/重载 (暂态故障)
    Restart,
    /// 编译期修复 (cargo fix / 自动导入/类型修正)
    CompileFix,
    /// 测试存根/忽略启用 (临时规避)
    TestStub,
}

impl std::str::FromStr for _RepairStrategyType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rollback" => Ok(_RepairStrategyType::Rollback),
            "patch" => Ok(_RepairStrategyType::Patch),
            "reconfigure" => Ok(_RepairStrategyType::Reconfigure),
            "restart" => Ok(_RepairStrategyType::Restart),
            "compilefix" => Ok(_RepairStrategyType::CompileFix),
            "teststub" => Ok(_RepairStrategyType::TestStub),
            _ => Err(format!("Unknown _RepairStrategyType: {}", s)),
        }
    }
}

/// 单个修复选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _RepairOption {
    pub strategy: _RepairStrategyType,
    pub description: String,
    pub target_files: Vec<String>,
    pub estimated_effort: _EffortLevel,
    pub risk: RepairRiskLevel,
    pub confidence: f32,           // 基于历史成功率 + 根因匹配度
    pub prerequisites: Vec<String>,
}

/// 风险等级 — 用于修复计划风险评估
/// 
/// For the unified RiskLevel, see `neotrix_types::RiskLevel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairRiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// 工作量估算
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum _EffortLevel {
    Trivial,
    Low,
    Medium,
    High,
}

/// 完整修复计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairPlan {
    pub strategy: String,               // 选定策略描述
    pub steps: Vec<_RepairStep>,
    pub risk: RiskAssessment,
    pub rollback_point: Option<String>, // git commit / snapshot id
    pub confidence: f32,
    pub source_pattern: Option<String>, // 匹配的历史模式
}

/// 修复步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _RepairStep {
    pub action: _RepairStrategyType,
    pub target: String,
    pub details: String,
    pub verification: String,           // 步骤后如何验证
}

/// 风险评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub level: RepairRiskLevel,
    pub impact: String,
    pub mitigations: Vec<String>,
    pub rollback_plan: String,
}

/// 修复结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResult {
    pub plan: RepairPlan,
    pub executed_steps: Vec<_StepExecution>,
    pub tests_pass: bool,
    pub health_delta: f32,
    pub artifacts_changed: Vec<String>,
}

/// 步骤执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _StepExecution {
    pub step: _RepairStep,
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,
}

/// 健康报告 (用于恢复验证)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub healthy: bool,
    pub dimensions: HashMap<String, f32>, // dimension -> score 0-1
    pub alerts: Vec<String>,
}

/// 验证判定
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifyVerdict {
    Recovered,              // 完全恢复
    RetryWithDiagnosis,     // 需重新诊断
    Reconcile,              // 需人工介入
    Failed,                 // 修复失败
}

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    pub verdict: VerifyVerdict,
    pub recovered: bool,
    pub exit_reason: String,
    pub pattern_update: Option<PatternUpdate>,
    pub known_trap_promoted: Option<String>,
}

/// 模式更新
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternUpdate {
    pub promoted_trap: Option<String>,
    pub flaky_reason: Option<String>,
    pub new_pattern: Option<_RepairPattern>,
}

/// 修复模式 (用于模式库学习)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _RepairPattern {
    pub signature: String,              // 根因特征签名
    pub strategy: _RepairStrategyType,
    pub success_rate: f32,
    pub usage_count: u32,
    pub applicable_contexts: Vec<String>,
}

/// 修复计划生成器
pub struct _RepairPlanner {
    pattern_library: HashMap<String, _RepairPattern>,
    strategy_weights: HashMap<_RepairStrategyType, f32>,
}

impl _RepairPlanner {
    pub fn new() -> Self {
        let mut weights = HashMap::new();
        weights.insert(_RepairStrategyType::Rollback, 0.9);       // 高成功率、低风险
        weights.insert(_RepairStrategyType::Restart, 0.8);
        weights.insert(_RepairStrategyType::Reconfigure, 0.7);
        weights.insert(_RepairStrategyType::CompileFix, 0.6);
        weights.insert(_RepairStrategyType::Patch, 0.5);          // 需匹配模式
        weights.insert(_RepairStrategyType::TestStub, 0.3);       // 临时规避、最后手段

        Self {
            pattern_library: HashMap::new(),
            strategy_weights: weights,
        }
    }

    /// 核心入口: 从根因假设生成修复计划
    pub fn generate_plan(&self, diagnosis: &Diagnosis) -> RepairPlan {
        // 1. 选策略: 基于置信度、风险、历史模式
        let strategy = self.select_strategy(diagnosis);

        // 2. 生成步骤
        let steps = self.generate_steps(&strategy, diagnosis);

        // 3. 风险评估
        let risk = self.assess_risk(&strategy, diagnosis);

        // 4. 回滚点
        let rollback_point = self.suggest_rollback_point(diagnosis);

        RepairPlan {
            strategy: format!("{:?}", strategy),
            steps,
            risk,
            rollback_point,
            confidence: self.estimate_confidence(&strategy, diagnosis),
            source_pattern: self.find_matching_pattern(diagnosis),
        }
    }

    /// 策略选择 (核心决策逻辑)
    fn select_strategy(&self, diagnosis: &Diagnosis) -> _RepairStrategyType {
        let max_confidence = diagnosis.root_causes.iter()
            .map(|rc| rc.confidence)
            .fold(0.0, f32::max);

        // 置信度高 + 严重 -> Patch/CompileFix
        if max_confidence >= 0.7 && matches!(diagnosis.severity, Severity::Critical | Severity::High) {
            let hyp = diagnosis.root_causes.iter()
                .map(|rc| rc.hypothesis.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            if hyp.contains("compile") || hyp.contains("type") || hyp.contains("编译") || hyp.contains("类型") {
                return _RepairStrategyType::CompileFix;
            }
            return _RepairStrategyType::Patch;
        }

        // 低置信度或暂态 -> Rollback/Restart/Reconfigure
        if max_confidence < 0.5 {
            return _RepairStrategyType::Rollback;
        }

        // 中等置信度 -> 尝试 Reconfigure
        _RepairStrategyType::Reconfigure
    }

    fn generate_steps(&self, strategy: &_RepairStrategyType, _diagnosis: &Diagnosis) -> Vec<_RepairStep> {
        match strategy {
            _RepairStrategyType::Rollback => vec![
                _RepairStep {
                    action: *strategy,
                    target: "git".into(),
                    details: "回滚到最后已知良好提交".into(),
                    verification: "cargo test --lib 通过".into(),
                }
            ],
            _RepairStrategyType::Restart => vec![
                _RepairStep {
                    action: *strategy,
                    target: "process".into(),
                    details: "重启受影响服务/进程".into(),
                    verification: "健康检查通过".into(),
                }
            ],
            _RepairStrategyType::Reconfigure => vec![
                _RepairStep {
                    action: *strategy,
                    target: "config".into(),
                    details: "调整特性开关/降级配置".into(),
                    verification: "功能烟测通过".into(),
                }
            ],
            _RepairStrategyType::CompileFix => vec![
                _RepairStep {
                    action: *strategy,
                    target: "source".into(),
                    details: "运行 cargo fix / 自动导入修正".into(),
                    verification: "cargo check --lib 无错误".into(),
                }
            ],
            _RepairStrategyType::Patch => vec![
                _RepairStep {
                    action: *strategy,
                    target: "source".into(),
                    details: "应用模式匹配补丁 / LLM 生成修复".into(),
                    verification: "单测通过 + 相关集成测试通过".into(),
                }
            ],
            _RepairStrategyType::TestStub => vec![
                _RepairStep {
                    action: *strategy,
                    target: "test".into(),
                    details: "为失败测试添加存根 / 标记 ignored".into(),
                    verification: "测试套件绿色".into(),
                }
            ],
        }
    }

    fn assess_risk(&self, strategy: &_RepairStrategyType, _diagnosis: &Diagnosis) -> RiskAssessment {
        let (level, impact) = match strategy {
            _RepairStrategyType::Rollback => (RepairRiskLevel::Low, "仅回滚代码, 状态可恢复"),
            _RepairStrategyType::Restart => (RepairRiskLevel::Low, "暂态中断, 无数据风险"),
            _RepairStrategyType::Reconfigure => (RepairRiskLevel::Medium, "配置变更可能影响其他模块"),
            _RepairStrategyType::CompileFix => (RepairRiskLevel::Medium, "自动修正可能引入语义变更"),
            _RepairStrategyType::Patch => (RepairRiskLevel::High, "代码修改可能引入新缺陷"),
            _RepairStrategyType::TestStub => (RepairRiskLevel::Critical, "规避测试掩盖真实问题"),
        };

        RiskAssessment {
            level,
            impact: impact.into(),
            mitigations: self.suggest_mitigations(strategy),
            rollback_plan: "git reset --hard <rollback_point>".into(),
        }
    }

    fn suggest_mitigations(&self, strategy: &_RepairStrategyType) -> Vec<String> {
        match strategy {
            _RepairStrategyType::Patch => vec![
                "先在隔离分支验证".into(),
                "运行完整测试套件".into(),
                "准备快速回滚".into(),
            ],
            _RepairStrategyType::TestStub => vec![
                "记录技术债务工单".into(),
                "设定恢复期限".into(),
                "仅作临时过渡".into(),
            ],
            _ => vec!["标准验证流程".into()],
        }
    }

    fn suggest_rollback_point(&self, _diagnosis: &Diagnosis) -> Option<String> {
        // 简化: 返回最近成功构建的 commit (实际应查 CI 历史)
        Some("HEAD~1".into())
    }

    fn estimate_confidence(&self, strategy: &_RepairStrategyType, diagnosis: &Diagnosis) -> f32 {
        let base = self.strategy_weights.get(strategy).copied().unwrap_or(0.5);
        let rc_boost = diagnosis.root_causes.iter()
            .map(|rc| rc.confidence)
            .fold(0.0, f32::max);
        (base + rc_boost) / 2.0
    }

    fn find_matching_pattern(&self, diagnosis: &Diagnosis) -> Option<String> {
        // 简化: 基于根因描述的关键词匹配
        for (sig, _pattern) in &self.pattern_library {
            if diagnosis.root_causes.iter().any(|rc| rc.hypothesis.contains(sig)) {
                return Some(sig.clone());
            }
        }
        None
    }

    /// 学习新模式 (供 recovery_verify 调用)
    pub(crate) fn _learn_pattern(&mut self, pattern: _RepairPattern) {
        self.pattern_library.insert(pattern.signature.clone(), pattern);
    }
}

impl Default for _RepairPlanner {
    fn default() -> Self {
        Self::new()
    }
}

/// 恢复验证器
pub struct RecoveryVerifier {
    known_traps: HashMap<String, u32>,    // 陷阱签名 -> 触发次数
    _flaky_tests: HashMap<String, u32>,    // 测试名 -> 失败次数 (预留)
    promotion_threshold: u32,
}

impl RecoveryVerifier {
    pub fn new() -> Self {
        Self {
            known_traps: HashMap::new(),
            _flaky_tests: HashMap::new(),
            promotion_threshold: 3,        // 3 次同模式触发晋升
        }
    }

    /// 核心验证: 修复执行后评估是否真正恢复
    pub fn verify(
        &mut self,
        repair: &FixResult,
        verify: &VerificationInput,
        history: &RepairHistory,
    ) -> VerifyResult {
        // 1. 基础判定: 测试通过 + 健康度改善
        let tests_ok = verify.tests_pass;
        let health_improved = verify.health.healthy || verify.health.dimensions.values().any(|&v| v > 0.5);

        if tests_ok && health_improved {
            // 2. 学习模式
            let pattern_update = self.learn_from_success(&repair.plan, history.same_pattern_count);

            // 3. 检查是否触发陷阱晋升
            let trap_promoted = self.check_trap_promotion(&repair.plan, history.same_pattern_count);

            return VerifyResult {
                verdict: VerifyVerdict::Recovered,
                recovered: true,
                exit_reason: "测试通过且健康度恢复".into(),
                pattern_update,
                known_trap_promoted: trap_promoted,
            };
        }

        // 测试失败或健康度未恢复 -> 需重新诊断或人工介入
        if history.same_pattern_count >= self.promotion_threshold as usize {
            // 同模式重复失败 -> 晋升为已知陷阱
            let trap_sig = self.trap_signature(&repair.plan);
            *self.known_traps.entry(trap_sig.clone()).or_insert(0) += 1;

            return VerifyResult {
                verdict: VerifyVerdict::RetryWithDiagnosis,
                recovered: false,
                exit_reason: format!("同模式失败 {} 次, 需重新诊断", history.same_pattern_count),
                pattern_update: None,
                known_trap_promoted: Some(trap_sig),
            };
        }

        VerifyResult {
            verdict: VerifyVerdict::RetryWithDiagnosis,
            recovered: false,
            exit_reason: "测试未通过或健康度未恢复, 建议重新诊断".into(),
            pattern_update: None,
            known_trap_promoted: None,
        }
    }

    fn learn_from_success(&mut self, plan: &RepairPlan, count: usize) -> Option<PatternUpdate> {
        if count == 0 && plan.confidence > 0.7 {
            // 首次成功且高置信度 -> 记录新模式
            let pattern = _RepairPattern {
                signature: self.plan_signature(plan),
                strategy: plan.strategy.parse().unwrap_or(_RepairStrategyType::Patch),
                success_rate: plan.confidence,
                usage_count: 1,
                applicable_contexts: vec!["auto".into()],
            };
            Some(PatternUpdate {
                promoted_trap: None,
                flaky_reason: None,
                new_pattern: Some(pattern),
            })
        } else {
            None
        }
    }

    fn check_trap_promotion(&mut self, plan: &RepairPlan, count: usize) -> Option<String> {
        if count >= self.promotion_threshold as usize {
            let sig = self.trap_signature(plan);
            Some(sig)
        } else {
            None
        }
    }

    fn trap_signature(&self, plan: &RepairPlan) -> String {
        format!("trap::{}:{}", plan.strategy, plan.steps.len())
    }

    fn plan_signature(&self, plan: &RepairPlan) -> String {
        format!("pattern::{}:steps={}", plan.strategy, plan.steps.len())
    }
}

impl Default for RecoveryVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationInput {
    pub tests_pass: bool,
    pub health: HealthReport,
}

/// 修复历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairHistory {
    pub same_pattern_count: usize,
    pub total_attempts: usize,
}

/// SelfTest for _RepairPlanner
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repair_planner_basic() {
        let planner = _RepairPlanner::new();
        let diagnosis = Diagnosis {
            description: "编译错误: 类型不匹配".into(),
            root_causes: vec![_RootCauseHypothesis {
                hypothesis: "类型不匹配导致编译失败".into(),
                confidence: 0.9,
                evidence: vec!["cargo check 报错".into()],
                affected_modules: vec!["core".into()],
            }],
            severity: Severity::High,
        };

        let plan = planner.generate_plan(&diagnosis);
        assert_eq!(plan.strategy, "CompileFix");
        assert!(!plan.steps.is_empty());
        assert!(plan.confidence > 0.5);
    }

    #[test]
    fn test_repair_planner_low_confidence_rollback() {
        let planner = _RepairPlanner::new();
        let diagnosis = Diagnosis {
            description: "间歇性故障".into(),
            root_causes: vec![_RootCauseHypothesis {
                hypothesis: "未知根因".into(),
                confidence: 0.3,
                evidence: vec![],
                affected_modules: vec![],
            }],
            severity: Severity::Medium,
        };

        let plan = planner.generate_plan(&diagnosis);
        assert_eq!(plan.strategy, "Rollback");
    }

    #[test]
    fn test_recovery_verifier_success() {
        let mut verifier = RecoveryVerifier::new();
        let fix = FixResult {
            plan: RepairPlan {
                strategy: "Patch".into(),
                steps: vec![],
                risk: RiskAssessment { level: RiskLevel::Medium, impact: "".into(), mitigations: vec![], rollback_plan: "".into() },
                rollback_point: None,
                confidence: 0.8,
                source_pattern: None,
            },
            executed_steps: vec![],
            tests_pass: true,
            health_delta: 0.3,
            artifacts_changed: vec![],
        };
        let verify = VerificationInput {
            tests_pass: true,
            health: HealthReport { healthy: true, dimensions: HashMap::new(), alerts: vec![] },
        };
        let history = RepairHistory { same_pattern_count: 0, total_attempts: 1 };

        let result = verifier.verify(&fix, &verify, &history);
        assert_eq!(result.verdict, VerifyVerdict::Recovered);
        assert!(result.recovered);
    }

    #[test]
    fn test_recovery_verifier_trap_promotion() {
        let mut verifier = RecoveryVerifier::new();
        let fix = FixResult {
            plan: RepairPlan {
                strategy: "Patch".into(),
                steps: vec![],
                risk: RiskAssessment { level: RiskLevel::Medium, impact: "".into(), mitigations: vec![], rollback_plan: "".into() },
                rollback_point: None,
                confidence: 0.6,
                source_pattern: None,
            },
            executed_steps: vec![],
            tests_pass: false,
            health_delta: -0.1,
            artifacts_changed: vec![],
        };
        let verify = VerificationInput {
            tests_pass: false,
            health: HealthReport { healthy: false, dimensions: HashMap::new(), alerts: vec![] },
        };

        // 连续 3 次失败
        for _ in 0..2 {
            let history = RepairHistory { same_pattern_count: 1, total_attempts: 1 };
            let _ = verifier.verify(&fix, &verify, &history);
        }
        let history = RepairHistory { same_pattern_count: 3, total_attempts: 3 };
        let result = verifier.verify(&fix, &verify, &history);
        assert!(result.known_trap_promoted.is_some());
    }
}