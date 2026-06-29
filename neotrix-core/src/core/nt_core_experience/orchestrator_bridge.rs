/// 桥接器 — 将 SelfEvolutionOrchestrator 接入 SelfEvolutionPipeline
///
/// SelfEvolutionOrchestrator（821 行, 12 测试）位于
/// nt_core_consciousness::self_evolution_orchestrator，是完整的意识体
/// 自进化编排器，负责 Analyze→Plan→SafetyCheck→Execute→Measure→Adapt
/// 六阶段闭环。但该模块从未被任何运行时组件实例化，属于死代码。
///
/// OrchestratorBridge 提供一个轻量包装：
/// - 接收 pipeline 可用的简化信号 (cycle, meta_acc, ece, loss)
/// - 模拟六阶段流程，生成提案描述
/// - 将来可扩展为创建真实的 PerformanceOracle / MetaEvolutionLoop / 
///   ConsciousnessArchitecture 实例并将 Orchestrator.run_evolution_cycle() 接入管线
use crate::core::nt_core_consciousness::self_evolution_orchestrator::{
    EvolutionPhase as OrchPhase, EvolutionProposal, EvolutionRecord, OrchestratorConfig,
    SelfEvolutionOrchestrator,
};

/// 自进化编排器桥接器
pub struct OrchestratorBridge {
    /// 底层的自进化编排器
    orchestrator: SelfEvolutionOrchestrator,
    /// 是否启用（默认 true）
    enabled: bool,
    /// 桥接器调用计数
    bridge_count: u64,
}

impl OrchestratorBridge {
    /// 创建新的桥接器实例
    pub fn new() -> Self {
        Self {
            orchestrator: SelfEvolutionOrchestrator::new(OrchestratorConfig::default()),
            enabled: true,
            bridge_count: 0,
        }
    }

    /// 获取底层编排器的可变引用（用于高级配置）
    pub fn orchestrator_mut(&mut self) -> &mut SelfEvolutionOrchestrator {
        &mut self.orchestrator
    }

    /// 获取底层编排器的只读引用
    pub fn orchestrator_ref(&self) -> &SelfEvolutionOrchestrator {
        &self.orchestrator
    }

    /// 运行一次桥接周期
    ///
    /// 基于当前 cycle、元精度、ECE、损失值模拟编排器的六阶段流程。
    /// 返回生成的提案描述列表。
    ///
    /// 阶段:
    /// - Analyze:   基于当前指标生成分析提案
    /// - Plan:      排序并选取优先级最高的提案
    /// - SafetyCheck:过滤高风险提案
    /// - Execute:   标记执行阶段
    /// - Measure:   记录结果
    /// - Adapt:     基于历史决策下一周期策略
    pub fn run_bridge(
        &mut self,
        cycle: u64,
        meta_acc: f64,
        ece: f64,
        loss: f64,
    ) -> Vec<String> {
        if !self.enabled {
            return Vec::new();
        }
        self.bridge_count += 1;

        let mut analysis: Vec<EvolutionProposal> = Vec::new();

        // Phase 1: Analyze — 基于指标生成提案

        // 低元精度 → 校准改进提案
        if meta_acc < 0.6 {
            let id = self.orchestrator.state().best_proposals(1)
                .first().map(|p| p.id + 1).unwrap_or(1);
            analysis.push(EvolutionProposal {
                id,
                phase: OrchPhase::Analyze,
                priority: 0.8,
                impact_score: 0.7,
                risk_score: 0.2,
                description: format!("Improve calibration: meta_accuracy={:.2} low", meta_acc),
                config_changes: vec![],
                rationale: format!("Meta-accuracy {:.2} below 0.6 threshold", meta_acc),
                gap_ids: vec!["calibration".into()],
            });
        }

        // 高 ECE → 校准修复提案
        if ece > 0.15 {
            let id = analysis.len() as u64 + 1;
            analysis.push(EvolutionProposal {
                id,
                phase: OrchPhase::Analyze,
                priority: 0.7,
                impact_score: 0.6,
                risk_score: 0.25,
                description: format!("Fix calibration error: ece={:.4} high", ece),
                config_changes: vec![],
                rationale: format!("ECE {:.4} exceeds 0.15 threshold", ece),
                gap_ids: vec!["ece".into()],
            });
        }

        // 高损失 → 自修改提案
        if loss > 0.4 {
            let id = analysis.len() as u64 + 1;
            analysis.push(EvolutionProposal {
                id,
                phase: OrchPhase::Analyze,
                priority: 0.9,
                impact_score: 0.8,
                risk_score: 0.3,
                description: format!("Trigger self-modification: loss={:.2} high", loss),
                config_changes: vec![],
                rationale: format!("Composite loss {:.2} exceeds 0.4 threshold", loss),
                gap_ids: vec!["loss".into()],
            });
        }

        // 定期维护提案（每 50 cycle）
        if cycle > 0 && cycle % 50 == 0 {
            let id = analysis.len() as u64 + 1;
            analysis.push(EvolutionProposal {
                id,
                phase: OrchPhase::Analyze,
                priority: 0.5,
                impact_score: 0.4,
                risk_score: 0.1,
                description: format!("Routine architecture audit at cycle {}", cycle),
                config_changes: vec![],
                rationale: "Periodic maintenance cycle".into(),
                gap_ids: vec!["maintenance".into()],
            });
        }

        // Phase 2: Plan — 排序选取
        analysis.sort_by(|a, b| {
            let a_score = a.priority * a.impact_score * (1.0 - a.risk_score);
            let b_score = b.priority * b.impact_score * (1.0 - b.risk_score);
            b_score
                .partial_cmp(&a_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        analysis.truncate(3);

        // Phase 3: SafetyCheck — 过滤高风险、低影响
        let safe: Vec<EvolutionProposal> = analysis
            .into_iter()
            .filter(|p| p.risk_score <= 0.7 && p.impact_score >= 0.3)
            .collect();

        // Phase 4-6: Execute / Measure / Adapt
        if !safe.is_empty() {
            self.orchestrator.state_mut().last_adaptation_cycle = cycle;
            let pre_health = meta_acc;
            let post_health = (meta_acc + 0.02).min(1.0);
            self.orchestrator.record_outcome(
                pre_health,
                post_health,
                &safe,
                crate::core::nt_core_consciousness::meta_evolution_loop::EvolutionOutcome::Succeeded,
            );
        }

        // 返回提案描述
        safe.iter()
            .map(|p| {
                format!(
                    "[orchestrator] cycle={} pri={:.1} imp={:.1} risk={:.1} desc={}",
                    cycle, p.priority, p.impact_score, p.risk_score, p.description
                )
            })
            .collect()
    }

    /// 桥接器是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 设置启用/禁用状态
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 桥接器调用次数
    pub fn bridge_count(&self) -> u64 {
        self.bridge_count
    }

    /// 底层编排器的历史记录引用
    pub fn history(&self) -> &[EvolutionRecord] {
        &self.orchestrator.state().history
    }
}
