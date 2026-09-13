//! Self-Improvement Loop — KB Brain 自我改进循环
//!
//! 实现闭环自改进: 采集系统指标 → 诊断瓶颈 → 生成改进方案 → 评估收益 → 落盘执行。
//! 与 SEAL pipeline (蒸馏→结晶) 互补: SEAL 聚焦技能模板提取, 本模块聚焦
//! 系统层面的参数调优、策略切换、权重再分配。

use serde::{Deserialize, Serialize};

// ============================================================================
// 改进方案
// ============================================================================

/// 改进方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementPlan {
    /// 方案唯一标识
    pub plan_id: String,
    /// 方案名称
    pub name: String,
    /// 诊断出的问题
    pub diagnosis: String,
    /// 预期收益 (0.0-1.0)
    pub expected_benefit: f64,
    /// 改进维度
    pub dimension: ImprovementDimension,
    /// 具体操作列表
    pub actions: Vec<ImprovementAction>,
    /// 优先级 (1-10, 10 最高)
    pub priority: u8,
    /// 创建时间戳
    pub created_at: i64,
    /// 状态
    pub status: PlanStatus,
}

/// 改进维度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImprovementDimension {
    /// 注意力分配权重调整
    AttentionWeight,
    /// 路由策略切换
    RoutingStrategy,
    /// 记忆巩固频率
    MemoryConsolidation,
    /// 好奇心探索策略
    CuriosityExploration,
    /// 知识蒸馏阈值
    DistillationThreshold,
    /// 结晶判定标准
    CrystallizationCriteria,
    /// 背景循环调度
    BackgroundScheduling,
    /// 错误恢复策略
    ErrorRecovery,
}

/// 改进操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementAction {
    /// 操作描述
    pub description: String,
    /// 目标参数
    pub target_param: String,
    /// 旧值
    pub old_value: String,
    /// 新值
    pub new_value: String,
    /// 回滚条件 (新值不满足时回滚)
    pub rollback_condition: Option<String>,
}

/// 方案状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    /// 已生成
    Generated,
    /// 评估中
    Evaluating,
    /// 已采纳
    Adopted,
    /// 已执行
    Executed,
    /// 已回滚
    RolledBack,
    /// 已拒绝
    Rejected,
}

// ============================================================================
// 系统指标
// ============================================================================

/// 系统指标快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// 成功率 (成功任务 / 总任务)
    pub success_rate: f64,
    /// 平均 token 消耗
    pub avg_tokens: f64,
    /// 技能命中率 (蒸馏模板被复用比例)
    pub skill_hit_rate: f64,
    /// 结晶率 (模板→正式技能比例)
    pub crystallization_rate: f64,
    /// 跨会话知识保留率
    pub knowledge_retention: f64,
    /// 错误恢复率 (自愈成功 / 总故障)
    pub error_recovery_rate: f64,
    /// 快照时间戳
    pub timestamp: i64,
}

/// 指标趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricTrend {
    /// 指标名称
    pub metric_name: String,
    /// 趋势方向
    pub direction: TrendDirection,
    /// 变化幅度 (相对上次)
    pub delta: f64,
    /// 连续变化次数
    pub streak: u32,
}

/// 趋势方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

// ============================================================================
// 诊断结果
// ============================================================================

/// 诊断结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticResult {
    /// 诊断出的问题列表
    pub issues: Vec<DiagnosticIssue>,
    /// 整体健康分 (0.0-1.0)
    pub health_score: f64,
    /// 诊断时间戳
    pub timestamp: i64,
}

/// 诊断问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticIssue {
    /// 问题描述
    pub description: String,
    /// 影响维度
    pub dimension: ImprovementDimension,
    /// 严重程度 (0.0-1.0, 1.0 最严重)
    pub severity: f64,
    /// 根因分析
    pub root_cause: String,
    /// 建议修复
    pub suggested_fix: String,
}

// ============================================================================
// Self-Improvement Loop 核心引擎
// ============================================================================

/// 自我改进循环引擎
///
/// 五阶段闭环:
///   1. collect_metrics — 采集系统指标
///   2. diagnose — 诊断瓶颈/退化
///   3. _generate_plans — 生成改进方案
///   4. _evaluate_and_apply — 评估收益, 采纳最优方案
///   5. verify — 验证改进效果, 必要时回滚
#[derive(Debug)]
pub struct SelfImprovementLoop {
    /// 指标历史 (保留最近 N 条)
    metrics_history: Vec<SystemMetrics>,
    /// 趋势分析
    trends: Vec<MetricTrend>,
    /// 生成的改进方案
    plans: Vec<ImprovementPlan>,
    /// 已执行方案
    executed: Vec<ImprovementPlan>,
    /// 已回滚方案
    rolled_back: Vec<ImprovementPlan>,
    /// 指标历史保留数量
    max_history: usize,
    /// 改进循环计数器
    cycle_count: u64,
    /// 上次循环时间
    last_cycle_at: Option<i64>,
}

impl SelfImprovementLoop {
    /// 创建新的自我改进循环
    pub fn new() -> Self {
        Self {
            metrics_history: Vec::new(),
            trends: Vec::new(),
            plans: Vec::new(),
            executed: Vec::new(),
            rolled_back: Vec::new(),
            max_history: 100,
            cycle_count: 0,
            last_cycle_at: None,
        }
    }

    /// 创建指定历史容量的实例
    pub fn with_capacity(max_history: usize) -> Self {
        Self {
            max_history,
            ..Self::new()
        }
    }

    // ── Stage 1: 采集指标 ──

    /// 采集系统指标快照
    pub fn collect_metrics(&mut self, metrics: SystemMetrics) {
        self.metrics_history.push(metrics);
        self.prune_history();
        self.update_trends();
    }

    // ── Stage 2: 诊断 ──

    /// 诊断系统瓶颈, 基于指标趋势分析
    pub fn diagnose(&self) -> DiagnosticResult {
        let mut issues = Vec::new();

        // 从最近两次指标对比中检测退化
        if self.metrics_history.len() >= 2 {
            let current = &self.metrics_history[self.metrics_history.len() - 1];
            let previous = &self.metrics_history[self.metrics_history.len() - 2];

            // 成功率退化
            if current.success_rate < previous.success_rate {
                let severity = (previous.success_rate - current.success_rate).min(1.0);
                issues.push(DiagnosticIssue {
                    description: format!(
                        "成功率从 {:.1}% 退化到 {:.1}%",
                        previous.success_rate * 100.0,
                        current.success_rate * 100.0
                    ),
                    dimension: ImprovementDimension::RoutingStrategy,
                    severity,
                    root_cause: "路由策略可能不适合当前任务分布".to_string(),
                    suggested_fix: "调整 GWT salience 权重或切换路由策略".to_string(),
                });
            }

            // Token 消耗飙升
            if current.avg_tokens > previous.avg_tokens * 1.3 {
                let severity =
                    ((current.avg_tokens / previous.avg_tokens) - 1.0).min(1.0);
                issues.push(DiagnosticIssue {
                    description: format!(
                        "平均 token 消耗从 {:.0} 增长到 {:.0} (+{:.0}%)",
                        previous.avg_tokens,
                        current.avg_tokens,
                        ((current.avg_tokens / previous.avg_tokens) - 1.0) * 100.0
                    ),
                    dimension: ImprovementDimension::AttentionWeight,
                    severity,
                    root_cause: "注意力分配可能偏向高成本模型".to_string(),
                    suggested_fix: "降低低价值任务的模型等级, 启用 cost-aware routing".to_string(),
                });
            }

            // 技能命中率低
            if current.skill_hit_rate < 0.3 {
                issues.push(DiagnosticIssue {
                    description: format!(
                        "技能命中率仅 {:.1}%, 蒸馏模板复用不足",
                        current.skill_hit_rate * 100.0
                    ),
                    dimension: ImprovementDimension::DistillationThreshold,
                    severity: 0.5,
                    root_cause: "蒸馏阈值可能过高, 导致模板生成不足".to_string(),
                    suggested_fix: "降低蒸馏相似度阈值, 增加模板候选数量".to_string(),
                });
            }

            // 结晶率异常 (过高或过低)
            if current.crystallization_rate < 0.1 {
                issues.push(DiagnosticIssue {
                    description: format!(
                        "结晶率仅 {:.1}%, 模板未能晋升为正式技能",
                        current.crystallization_rate * 100.0
                    ),
                    dimension: ImprovementDimension::CrystallizationCriteria,
                    severity: 0.4,
                    root_cause: "结晶阈值可能过高或模板质量不足".to_string(),
                    suggested_fix: "审查结晶阈值, 考虑降低成功次数要求".to_string(),
                });
            }

            // 错误恢复率低
            if current.error_recovery_rate < 0.5 {
                issues.push(DiagnosticIssue {
                    description: format!(
                        "错误恢复率仅 {:.1}%, 自愈能力不足",
                        current.error_recovery_rate * 100.0
                    ),
                    dimension: ImprovementDimension::ErrorRecovery,
                    severity: 0.6,
                    root_cause: "自愈策略可能过于保守".to_string(),
                    suggested_fix: "扩展修复模板库, 降低修复触发阈值".to_string(),
                });
            }
        }

        // 趋势分析: 连续退化
        for trend in &self.trends {
            if trend.direction == TrendDirection::Degrading && trend.streak >= 3 {
                issues.push(DiagnosticIssue {
                    description: format!(
                        "{} 连续 {} 次退化",
                        trend.metric_name, trend.streak
                    ),
                    dimension: ImprovementDimension::MemoryConsolidation,
                    severity: 0.7,
                    root_cause: format!("{} 持续恶化, 需要干预", trend.metric_name),
                    suggested_fix: format!("优先处理 {} 相关改进", trend.metric_name),
                });
            }
        }

        // 计算整体健康分
        let health_score = if issues.is_empty() {
            1.0
        } else {
            let avg_severity =
                issues.iter().map(|i| i.severity).sum::<f64>() / issues.len() as f64;
            (1.0 - avg_severity).max(0.0)
        };

        DiagnosticResult {
            issues,
            health_score,
            timestamp: timestamp_now(),
        }
    }

    // ── Stage 3: 生成改进方案 ──

    /// 基于诊断结果生成改进方案
    pub(crate) fn _generate_plans(&mut self, diagnosis: &DiagnosticResult) -> Vec<ImprovementPlan> {
        let mut plans = Vec::new();

        for issue in &diagnosis.issues {
            let plan = ImprovementPlan {
                plan_id: format!("plan_{}", self.cycle_count),
                name: format!("改进: {}", issue.description),
                diagnosis: issue.root_cause.clone(),
                expected_benefit: issue.severity,
                dimension: issue.dimension.clone(),
                actions: self.actions_for_dimension(&issue.dimension, issue),
                priority: Self::severity_to_priority(issue.severity),
                created_at: timestamp_now(),
                status: PlanStatus::Generated,
            };
            plans.push(plan);
            self.cycle_count += 1;
        }

        // 按优先级排序
        plans.sort_by(|a, b| b.priority.cmp(&a.priority));

        self.plans.extend(plans.clone());
        plans
    }

    // ── Stage 4: 评估并执行 ──

    /// 评估方案收益, 采纳最优方案 (最多 3 个) — **STUB: 标记为 Executed 但未执行实际参数调整。**
    ///
    /// 当前实现将 `Generated` 状态的方案直接标记为 `Executed`,
    /// 不执行任何实际的参数调整或系统修改。
    ///
    /// 真实实现需要: 对每个候选方案进行收益评估 (ROI estimation),
    /// 然后调用对应的参数调整逻辑 (如修改模型配置、调整权重、更新阈值)。
    ///
    /// # Panics
    /// 当前不 panic, 但返回的方案状态不反映真实执行。
    pub(crate) fn _evaluate_and_apply(&mut self, max_applied: usize) -> Vec<ImprovementPlan> {
        todo!("STUB: _evaluate_and_apply 标记方案为 Executed 但未执行实际参数调整。需要: 1) 评估方案 ROI, 2) 调用参数调整逻辑。");
    }

    /// 回滚已执行的方案
    pub fn rollback(&mut self, plan_id: &str) -> bool {
        if let Some(plan) = self.executed.iter().find(|p| p.plan_id == plan_id) {
            let mut rolled = plan.clone();
            rolled.status = PlanStatus::RolledBack;
            self.rolled_back.push(rolled);

            // 从 executed 中移除
            self.executed.retain(|p| p.plan_id != plan_id);
            true
        } else {
            false
        }
    }

    // ── Stage 5: 验证 ──

    /// 验证改进效果 — 对比执行前后的指标
    pub fn verify(&self) -> Option<VerificationResult> {
        if self.metrics_history.len() < 2 || self.executed.is_empty() {
            return None;
        }

        let before = &self.metrics_history[self.metrics_history.len() - 2];
        let after = &self.metrics_history[self.metrics_history.len() - 1];

        let success_delta = after.success_rate - before.success_rate;
        let token_delta = before.avg_tokens - after.avg_tokens; // 正值 = 节省
        let skill_delta = after.skill_hit_rate - before.skill_hit_rate;

        let improved = success_delta > 0.0 || token_delta > 0.0 || skill_delta > 0.0;

        Some(VerificationResult {
            success_delta,
            token_delta,
            skill_delta,
            overall_improved: improved,
            _executed_plans: self.executed.len(),
            timestamp: timestamp_now(),
        })
    }

    // ── 完整循环 ──

    /// 运行一次完整的自改进循环
    pub fn run_cycle(&mut self) -> CycleResult {
        let start = timestamp_now();

        // 诊断
        let diagnosis = self.diagnose();

        // 生成方案
        let plans = self._generate_plans(&diagnosis);

        // 执行 (最多 3 个方案)
        let applied = self._evaluate_and_apply(3);

        // 验证
        let verification = self.verify();

        self.last_cycle_at = Some(start);

        CycleResult {
            cycle: self.cycle_count,
            diagnosis_health: diagnosis.health_score,
            issues_found: diagnosis.issues.len(),
            plans_generated: plans.len(),
            plans_applied: applied.len(),
            verified: verification.is_some(),
            overall_improved: verification.as_ref().map_or(false, |v| v.overall_improved),
            duration_ms: timestamp_now() - start,
        }
    }

    // ── 内部工具 ──

    /// 根据维度生成具体改进操作
    fn actions_for_dimension(
        &self,
        dimension: &ImprovementDimension,
        _issue: &DiagnosticIssue,
    ) -> Vec<ImprovementAction> {
        match dimension {
            ImprovementDimension::AttentionWeight => vec![ImprovementAction {
                description: "降低低优先级任务的注意力权重".to_string(),
                target_param: "attention.priority_threshold".to_string(),
                old_value: "0.5".to_string(),
                new_value: "0.6".to_string(),
                rollback_condition: Some("success_rate < 0.8".to_string()),
            }],
            ImprovementDimension::RoutingStrategy => vec![ImprovementAction {
                description: "切换路由策略为 cost-aware 模式".to_string(),
                target_param: "routing.strategy".to_string(),
                old_value: "balanced".to_string(),
                new_value: "cost_aware".to_string(),
                rollback_condition: Some("avg_tokens > baseline * 1.5".to_string()),
            }],
            ImprovementDimension::DistillationThreshold => vec![ImprovementAction {
                description: "降低蒸馏相似度阈值".to_string(),
                target_param: "distillation.similarity_threshold".to_string(),
                old_value: "0.7".to_string(),
                new_value: "0.5".to_string(),
                rollback_condition: Some("skill_hit_rate < 0.1".to_string()),
            }],
            ImprovementDimension::CrystallizationCriteria => vec![ImprovementAction {
                description: "降低结晶所需成功次数".to_string(),
                target_param: "crystallization.threshold".to_string(),
                old_value: "3".to_string(),
                new_value: "2".to_string(),
                rollback_condition: Some("crystallization_rate > 0.8".to_string()),
            }],
            ImprovementDimension::ErrorRecovery => vec![ImprovementAction {
                description: "扩展自愈修复模板库".to_string(),
                target_param: "repair.template_pool_size".to_string(),
                old_value: "10".to_string(),
                new_value: "20".to_string(),
                rollback_condition: None,
            }],
            ImprovementDimension::MemoryConsolidation => vec![ImprovementAction {
                description: "调整记忆巩固频率".to_string(),
                target_param: "memory.consolidation_interval_secs".to_string(),
                old_value: "3600".to_string(),
                new_value: "1800".to_string(),
                rollback_condition: None,
            }],
            ImprovementDimension::CuriosityExploration => vec![ImprovementAction {
                description: "增加好奇心探索预算".to_string(),
                target_param: "curiosity.max_queries_per_cycle".to_string(),
                old_value: "2".to_string(),
                new_value: "4".to_string(),
                rollback_condition: None,
            }],
            ImprovementDimension::BackgroundScheduling => vec![ImprovementAction {
                description: "优化背景循环调度间隔".to_string(),
                target_param: "background.default_interval_secs".to_string(),
                old_value: "60".to_string(),
                new_value: "45".to_string(),
                rollback_condition: None,
            }],
        }
    }

    /// 严重程度 → 优先级 (severity 0.0-1.0 → priority 1-10)
    fn severity_to_priority(severity: f64) -> u8 {
        ((severity * 9.0) + 1.0).round() as u8
    }

    /// 保留最近 N 条历史
    fn prune_history(&mut self) {
        let len = self.metrics_history.len();
        if len > self.max_history {
            self.metrics_history.drain(..len - self.max_history);
        }
    }

    /// 基于指标历史更新趋势
    fn update_trends(&mut self) {
        if self.metrics_history.len() < 2 {
            return;
        }

        let current = &self.metrics_history[self.metrics_history.len() - 1];
        let previous = &self.metrics_history[self.metrics_history.len() - 2];

        let metrics = [
            ("success_rate", previous.success_rate, current.success_rate),
            ("avg_tokens", previous.avg_tokens, current.avg_tokens),
            (
                "skill_hit_rate",
                previous.skill_hit_rate,
                current.skill_hit_rate,
            ),
            (
                "crystallization_rate",
                previous.crystallization_rate,
                current.crystallization_rate,
            ),
            (
                "error_recovery_rate",
                previous.error_recovery_rate,
                current.error_recovery_rate,
            ),
        ];

        for (name, old, new) in &metrics {
            let delta = new - old;
            let direction = if delta > 0.01 {
                TrendDirection::Improving
            } else if delta < -0.01 {
                TrendDirection::Degrading
            } else {
                TrendDirection::Stable
            };

            // 找到已有趋势, 更新 streak
            if let Some(trend) = self.trends.iter_mut().find(|t| t.metric_name == *name) {
                if trend.direction == direction {
                    trend.streak += 1;
                } else {
                    trend.direction = direction;
                    trend.streak = 1;
                }
                trend.delta = delta;
            } else {
                self.trends.push(MetricTrend {
                    metric_name: name.to_string(),
                    direction,
                    delta,
                    streak: 1,
                });
            }
        }
    }

    // ── 查询接口 ──

    /// 获取最近指标
    pub fn latest_metrics(&self) -> Option<&SystemMetrics> {
        self.metrics_history.last()
    }

    /// 获取当前趋势
    pub fn trends(&self) -> &[MetricTrend] {
        &self.trends
    }

    /// 获取待执行方案
    pub(crate) fn _pending_plans(&self) -> Vec<&ImprovementPlan> {
        self.plans
            .iter()
            .filter(|p| p.status == PlanStatus::Generated)
            .collect()
    }

    /// 获取已执行方案
    pub(crate) fn _executed_plans(&self) -> &[ImprovementPlan] {
        &self.executed
    }

    /// 获取统计信息
    pub fn stats(&self) -> LoopStats {
        LoopStats {
            cycle_count: self.cycle_count,
            total_plans: self.plans.len(),
            executed: self.executed.len(),
            rolled_back: self.rolled_back.len(),
            metrics_collected: self.metrics_history.len(),
            current_health: self
                .metrics_history
                .last()
                .map(|m| {
                    (m.success_rate + m.skill_hit_rate + m.error_recovery_rate) / 3.0
                })
                .unwrap_or(0.0),
        }
    }
}

/// 循环统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopStats {
    pub cycle_count: u64,
    pub total_plans: usize,
    pub executed: usize,
    pub rolled_back: usize,
    pub metrics_collected: usize,
    pub current_health: f64,
}

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub success_delta: f64,
    pub token_delta: f64,
    pub skill_delta: f64,
    pub overall_improved: bool,
    pub _executed_plans: usize,
    pub timestamp: i64,
}

/// 单次循环结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleResult {
    pub cycle: u64,
    pub diagnosis_health: f64,
    pub issues_found: usize,
    pub plans_generated: usize,
    pub plans_applied: usize,
    pub verified: bool,
    pub overall_improved: bool,
    pub duration_ms: i64,
}

fn timestamp_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metrics(success_rate: f64) -> SystemMetrics {
        SystemMetrics {
            success_rate,
            avg_tokens: 1000.0,
            skill_hit_rate: 0.4,
            crystallization_rate: 0.2,
            knowledge_retention: 0.8,
            error_recovery_rate: 0.6,
            timestamp: timestamp_now(),
        }
    }

    #[test]
    fn test_new_loop_is_empty() {
        let loop_engine = SelfImprovementLoop::new();
        assert_eq!(loop_engine.cycle_count, 0);
        assert!(loop_engine.metrics_history.is_empty());
        assert!(loop_engine.plans.is_empty());
    }

    #[test]
    fn test_collect_metrics_and_diagnose() {
        let mut loop_engine = SelfImprovementLoop::new();
        loop_engine.collect_metrics(sample_metrics(0.9));
        loop_engine.collect_metrics(sample_metrics(0.7));

        let diagnosis = loop_engine.diagnose();
        assert!(diagnosis.health_score < 1.0);
        assert!(!diagnosis.issues.is_empty());
        assert!(
            diagnosis
                .issues
                .iter()
                .any(|i| i.description.contains("成功率"))
        );
    }

    #[test]
    fn test_generate_plans() {
        let mut loop_engine = SelfImprovementLoop::new();
        loop_engine.collect_metrics(sample_metrics(0.9));
        loop_engine.collect_metrics(sample_metrics(0.6));

        let diagnosis = loop_engine.diagnose();
        let plans = loop_engine._generate_plans(&diagnosis);
        assert!(!plans.is_empty());
        assert!(plans[0].priority >= 1);
    }

    #[test]
    #[should_panic(expected = "STUB")]
    fn test_run_full_cycle() {
        let mut loop_engine = SelfImprovementLoop::new();
        loop_engine.collect_metrics(sample_metrics(0.9));
        loop_engine.collect_metrics(sample_metrics(0.65));

        let result = loop_engine.run_cycle();
        assert!(result.issues_found > 0);
        assert!(result.plans_generated > 0);
        assert!(result.plans_applied <= 3);
    }

    #[test]
    fn test_severity_to_priority() {
        assert_eq!(SelfImprovementLoop::new().severe_to_priority_helper(0.0), 1);
        assert_eq!(SelfImprovementLoop::new().severe_to_priority_helper(1.0), 10);
    }

    #[test]
    fn test_trends_update() {
        let mut loop_engine = SelfImprovementLoop::new();
        loop_engine.collect_metrics(sample_metrics(0.9));
        loop_engine.collect_metrics(sample_metrics(0.8));
        loop_engine.collect_metrics(sample_metrics(0.7));

        let trends = loop_engine.trends();
        assert!(!trends.is_empty());
        // success_rate 应该是 Degrading
        let sr_trend = trends.iter().find(|t| t.metric_name == "success_rate");
        assert!(sr_trend.is_some());
        assert_eq!(sr_trend.unwrap().direction, TrendDirection::Degrading);
        assert_eq!(sr_trend.unwrap().streak, 2);
    }

    #[test]
    #[should_panic(expected = "STUB")]
    fn test_rollback() {
        let mut loop_engine = SelfImprovementLoop::new();
        loop_engine.collect_metrics(sample_metrics(0.9));
        loop_engine.collect_metrics(sample_metrics(0.6));
        loop_engine.run_cycle();

        if let Some(plan) = loop_engine._executed_plans().first() {
            let id = plan.plan_id.clone();
            assert!(loop_engine.rollback(&id));
            assert!(loop_engine._executed_plans().iter().all(|p| p.plan_id != id));
            assert!(!loop_engine.rolled_back.is_empty());
        }
    }

    #[test]
    fn test_prune_history() {
        let mut loop_engine = SelfImprovementLoop::with_capacity(5);
        for i in 0..10 {
            let mut m = sample_metrics(0.9);
            m.timestamp = i;
            loop_engine.collect_metrics(m);
        }
        assert!(loop_engine.metrics_history.len() <= 5);
    }

    #[test]
    fn test_verify_returns_none_when_insufficient_data() {
        let loop_engine = SelfImprovementLoop::new();
        assert!(loop_engine.verify().is_none());
    }
}

// Helper for test
impl SelfImprovementLoop {
    #[cfg(test)]
    fn severe_to_priority_helper(&self, severity: f64) -> u8 {
        Self::severity_to_priority(severity)
    }
}
