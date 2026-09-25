//! 自进化循环 — 变异阶段 (`nt_loop_mutate`).
//!
//! 自动修复周期: 管线修复 → 独立 Auditor 裁决 (verify→recover) → 按诊断修复
//! (断路器保护)。从 `evolution_loop.rs` 纯搬移, 行为零变更。

use super::nt_loop_types::{EvolutionLoop, EvolutionReport, REPAIR_MAX_ROUNDS};
use crate::l1_action::nt_act::nt_act_code::PipelineAutoFixer;
use crate::l5_cognition::nt_mind::evolution::self_diagnose::{ActionExecutor, RepairCircuitBreaker};

impl EvolutionLoop {
    /// 自动修复周期 — 对所有 auto_fixable 问题执行真实修复并重新扫描
    pub(crate) fn _autofix_cycle(
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
        let verdict = self.auditor._verify_change(
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

    /// 重置停滞计数
    pub fn on_fix_applied(&mut self) {
        self.consecutive_stagnant = 0;
        self.fixed_history.push(1);
        if self.fixed_history.len() > 20 {
            self.fixed_history.remove(0);
        }
    }

    /// 基于诊断结果的自动修复 — 按优先级顺序执行 ActionPlan
    ///
    /// 循环断路器接线 (R-P79): 每次 autofix 会话持有一个 RepairCircuitBreaker,
    /// 逐 item 执行前检查断路器; 跳闸 (轮次超限或连续无进展) 即停止后续修复,
    /// 防止自愈循环空转 (retry cap / loop detection)。
    pub(crate) fn _autofix_by_diagnosis(&mut self) -> u32 {
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
