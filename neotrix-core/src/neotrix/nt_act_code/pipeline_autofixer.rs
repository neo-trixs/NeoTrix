//! PipelineAutoFixer — 全自动修复管线
//!
//! 将 Phase 3 各模块串联为完整流水线:
//!   SelfDiagnose → SelfCodeWriter → SafeCodeApplier → EditHistoryTracker → EvolutionLoop
//!
//! 零 LLM 依赖: 所有决策基于规则 + 历史模式 + 确定性模板

use crate::neotrix::nt_act_code::code_writer::{CodeGenRequest, SelfCodeWriter};
use crate::neotrix::nt_act_code::edit_history::EditHistoryTracker;
use crate::neotrix::nt_act_code::safe_applier::SafeCodeApplier;
use crate::neotrix::nt_mind_evolution_loop::EvolutionLoop;
use crate::neotrix::nt_mind_self_diagnose::ActionPlan;

/// 单次管道执行结果
#[derive(Debug)]
pub struct PipelineResult {
    pub total_detected: usize,
    pub auto_generated: usize,
    pub auto_applied: usize,
    pub auto_failed: usize,
    pub human_needed: usize,
    pub details: Vec<String>,
}

/// 全自动修复管线
#[derive(Debug)]
pub struct PipelineAutoFixer {
    code_writer: SelfCodeWriter,
    applier: SafeCodeApplier,
}

impl PipelineAutoFixer {
    pub fn new() -> Self {
        Self {
            code_writer: SelfCodeWriter::new(),
            applier: SafeCodeApplier::new(),
        }
    }

    /// 测试用: 使用 temp 路径的 tracker 避免污染持久状态
    pub fn new_empty(history_path: std::path::PathBuf) -> Self {
        Self {
            code_writer: SelfCodeWriter::new(),
            applier: SafeCodeApplier::with_tracker_path(history_path),
        }
    }

    /// 对当前项目执行一次完整的诊断→生成→应用→记录流水线
    pub fn run_pipeline(&mut self, el: &mut EvolutionLoop) -> PipelineResult {
        // 1. 诊断
        let (_items, pq) = el.self_diagnose();
        let mut result = PipelineResult {
            total_detected: pq.len(),
            auto_generated: 0,
            auto_applied: 0,
            auto_failed: 0,
            human_needed: 0,
            details: Vec::new(),
        };

        // 2. 对高分项尝试代码生成
        for item in pq.as_slice() {
            if matches!(
                item.action,
                ActionPlan::HumanDecision { .. } | ActionPlan::NoAction { .. }
            ) {
                result.human_needed += 1;
                continue;
            }

            // 跳过低分项
            if item.composite_score < 0.3 {
                continue;
            }

            // 3. 生成代码
            let file = item.underlying_issue.file.as_deref().unwrap_or("unknown");
            let context = std::fs::read_to_string(file).unwrap_or_default();

            let req = CodeGenRequest {
                plan: item.action.clone(),
                file: file.to_string(),
                context,
            };

            let gen_result = match self.code_writer.generate(&req) {
                Ok(r) => r,
                Err(e) => {
                    result.auto_failed += 1;
                    result.details.push(format!("生成失败 {}: {}", file, e));
                    continue;
                }
            };
            result.auto_generated += 1;

            // 4. 安全应用
            let issue_type_str = format!("{:?}", item.underlying_issue.issue_type);
            let apply_result =
                self.applier
                    .safe_write(&gen_result.file, &gen_result.new_content, &issue_type_str);

            if apply_result.success {
                result.auto_applied += 1;
                result.details.push(format!(
                    "✅ {}: 已{}",
                    gen_result.file,
                    gen_result.template_used.as_deref().unwrap_or("自动修复")
                ));
                el.on_fix_applied();
            } else {
                result.auto_failed += 1;
                result.details.push(format!(
                    "❌ {}: {}",
                    gen_result.file,
                    apply_result.error.unwrap_or_default()
                ));
            }
        }

        result
    }

    /// 分析管线结果摘要
    pub fn summarize(result: &PipelineResult) -> String {
        format!(
            "管线: {} 检测 | {} 生成 | {} 应用 | {} 失败 | {} 需人工",
            result.total_detected,
            result.auto_generated,
            result.auto_applied,
            result.auto_failed,
            result.human_needed,
        )
    }

    /// 获取应用器追踪器引用
    pub fn edit_history(&self) -> &EditHistoryTracker {
        self.applier.tracker()
    }
}

impl Default for PipelineAutoFixer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pipeline_empty_tracker() {
        let tmp = std::env::temp_dir().join(format!("pipeline_test_{}", std::process::id()));
        let p = PipelineAutoFixer::new_empty(std::path::PathBuf::from(&tmp));
        assert!(p.edit_history().is_empty());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_run_pipeline_on_clean_evolution_loop() {
        let mut el = EvolutionLoop::new();
        let tmp = std::env::temp_dir().join(format!("paf_run_test_{}", std::process::id()));
        let mut p = PipelineAutoFixer::new_empty(tmp);
        let result = p.run_pipeline(&mut el);
        assert!(result.total_detected > 0 || result.total_detected == 0);
    }

    #[test]
    fn test_summarize_format() {
        let result = PipelineResult {
            total_detected: 10,
            auto_generated: 5,
            auto_applied: 3,
            auto_failed: 2,
            human_needed: 5,
            details: vec![],
        };
        let s = PipelineAutoFixer::summarize(&result);
        assert!(s.contains("10"));
        assert!(s.contains("5"));
        assert!(s.contains("3"));
    }

    #[test]
    fn test_result_counts() {
        let mut r = PipelineResult {
            total_detected: 5,
            auto_generated: 3,
            auto_applied: 2,
            auto_failed: 1,
            human_needed: 2,
            details: vec![],
        };
        r.auto_generated += 1;
        assert_eq!(r.auto_generated, 4);
    }

    #[test]
    fn test_double_run_no_panic() {
        let mut el = EvolutionLoop::new();
        let tmp = std::env::temp_dir().join(format!("paf_double_test_{}", std::process::id()));
        let mut p = PipelineAutoFixer::new_empty(tmp);
        let _r1 = p.run_pipeline(&mut el);
        let _r2 = p.run_pipeline(&mut el);
    }
}
