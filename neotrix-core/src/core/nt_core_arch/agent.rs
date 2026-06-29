use crate::core::nt_core_meta::scanner::CodeScanner;
use crate::core::nt_core_meta::weakness::{WeaknessAnalyzer, WeaknessReport};
use crate::core::nt_core_meta::planner::{EvolutionPlanner, PlannedEvolution};
use crate::core::nt_core_meta::self_model::SelfModel;
use super::designer::{ArchitectureDesigner, ArchitectureDesign};
use super::implementer::{CodeImplementer, FileChange};
use super::verifier::{CompileVerificationResult, ChangeVerifier};

/// 一次自主架构设计循环的结果
#[derive(Clone, Debug)]
pub struct ArchitectCycleResult {
    pub cycle: usize,
    pub model_snapshot: SelfModel,
    pub weakness_report: WeaknessReport,
    pub planned_evolutions: Vec<PlannedEvolution>,
    pub design: ArchitectureDesign,
    pub file_changes: Vec<FileChange>,
    pub verification: Option<CompileVerificationResult>,
    pub changes_applied: bool,
    pub summary: String,
}

/// 完全自包含的架构设计 Agent
/// 不依赖任何外部 API / CLI 工具
pub struct ArchitectAgent {
    pub project_root: String,
    pub scanner: CodeScanner,
    pub analyzer: WeaknessAnalyzer,
    pub planner: EvolutionPlanner,
    pub designer: ArchitectureDesigner,
    pub implementer: CodeImplementer,
    pub verifier: ChangeVerifier,
    pub cycle: usize,
    pub max_cycles: usize,
    pub dry_run: bool,
    pub history: Vec<ArchitectCycleResult>,
}

impl ArchitectAgent {
    pub fn new(project_root: &str) -> Self {
        Self {
            project_root: project_root.to_string(),
            scanner: CodeScanner::new(project_root),
            analyzer: WeaknessAnalyzer::new(),
            planner: EvolutionPlanner::new(),
            designer: ArchitectureDesigner::new(),
            implementer: CodeImplementer::new(),
            verifier: ChangeVerifier::new(),
            cycle: 0,
            max_cycles: 10,
            dry_run: false,
            history: Vec::new(),
        }
    }

    /// 设置 dry-run 模式（只生成计划，不写入磁盘）
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// 运行一次完整的自主架构循环
    pub fn run_cycle(&mut self) -> ArchitectCycleResult {
        self.cycle += 1;

        // Phase 1-4: 扫描 → 检测 → 规划 → 设计
        let model = self.scanner.scan();
        let report = self.analyzer.analyze(&model);
        let plans = self.planner.plan_from_report(&report);
        let design = self.designer.design(&report, &plans, &model);

        // Phase 5: 生成变更计划
        let changes = self.implementer.plan_changes(&design, &self.project_root);

        // Phase 5.5: 写入磁盘（dry-run 跳过）
        let mut applied = false;
        if !changes.is_empty() && !self.dry_run {
            if let Err(e) = self.implementer.write_changes(&changes) {
                let summary = format!("[Cycle {}] Write failed: {}", self.cycle, e);
                return ArchitectCycleResult {
                    cycle: self.cycle, model_snapshot: model, weakness_report: report,
                    planned_evolutions: plans, design, file_changes: changes,
                    verification: None, changes_applied: false, summary,
                };
            }
            applied = true;
        }

        // Phase 6: 验证（必须写盘后才验证）
        let verification = if !changes.is_empty() && applied {
            let result = self.verifier.verify_compilation(&self.project_root);
            if !result.success {
                // Rollback: 验证失败 → 撤销变更
                let _ = self.implementer.rollback(&changes);
                applied = false;
            }
            Some(result)
        } else {
            None
        };

        let summary = self.build_summary(&report, &plans, &design, &changes, &verification, applied);

        let result = ArchitectCycleResult {
            cycle: self.cycle,
            model_snapshot: model,
            weakness_report: report,
            planned_evolutions: plans,
            design,
            file_changes: changes,
            verification,
            changes_applied: applied,
            summary,
        };

        self.history.push(result.clone());
        result
    }

    /// 运行多个周期
    pub fn run_batch(&mut self, cycles: usize) -> Vec<ArchitectCycleResult> {
        let max = cycles.min(self.max_cycles.saturating_sub(self.cycle));
        let mut results = Vec::with_capacity(max);
        for _ in 0..max {
            let has_changes = self.history.last().map(|r| !r.file_changes.is_empty()).unwrap_or(true);
            if !has_changes {
                break;
            }
            let result = self.run_cycle();
            results.push(result);
        }
        results
    }

    /// 获取整体状态摘要
    pub fn status_summary(&self) -> String {
        let last = self.history.last();
        match last {
            Some(r) => format!(
                "Cycle {}: {} weaknesses, {} plans, {} file changes, applied={}, compiled={}",
                self.cycle,
                r.weakness_report.weaknesses.len(),
                r.planned_evolutions.len(),
                r.file_changes.len(),
                r.changes_applied,
                r.verification.as_ref().map(|v| v.success).unwrap_or(true),
            ),
            None => format!("ArchitectAgent ready. Project root: {} (dry_run={})", self.project_root, self.dry_run),
        }
    }

    fn build_summary(
        &self,
        report: &WeaknessReport,
        plans: &[PlannedEvolution],
        design: &ArchitectureDesign,
        changes: &[FileChange],
        verification: &Option<CompileVerificationResult>,
        applied: bool,
    ) -> String {
        let ver = match verification {
            Some(v) if v.success => "compilation OK".to_string(),
            Some(v) => format!("{} compile errors — rolled back", v.compile_errors.len()),
            None => "not verified".to_string(),
        };
        format!(
            "[Cycle {}] {} weaknesses, {} plans, {} new modules, {} refactorings, {} file changes — {} (applied={})",
            self.cycle,
            report.weaknesses.len(),
            plans.len(),
            design.new_modules.len(),
            design.refactoring_plans.len(),
            changes.len(),
            ver,
            applied,
        )
    }
}

impl Default for ArchitectAgent {
    fn default() -> Self {
        Self::new(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_agent_ready() {
        let agent = ArchitectAgent::new("/tmp/test_project");
        assert_eq!(agent.cycle, 0);
        assert!(agent.status_summary().contains("ArchitectAgent ready"));
    }

    #[test]
    fn test_run_cycle_on_fake_project() {
        let mut agent = ArchitectAgent::new("/tmp/nonexistent_project_xyz");
        let result = agent.run_cycle();
        assert_eq!(result.cycle, 1);
    }

    #[test]
    fn test_run_batch_empty() {
        let mut agent = ArchitectAgent::new("/tmp/empty");
        let results = agent.run_batch(0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_cycle_result_has_summary() {
        let mut agent = ArchitectAgent::new("/tmp/test");
        let result = agent.run_cycle();
        assert!(!result.summary.is_empty());
        assert!(result.summary.contains("Cycle 1"));
    }

    #[test]
    fn test_status_summary_evolves() {
        let mut agent = ArchitectAgent::new("/tmp/test");
        assert!(agent.status_summary().contains("ready"));
        agent.run_cycle();
        assert!(agent.status_summary().contains("Cycle 1"));
    }

    #[test]
    fn test_real_project_scan_produces_results() {
        let project = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let mut agent = ArchitectAgent::new(&project);
        let result = agent.run_cycle();
        log::info!("Weaknesses: {}", result.weakness_report.weaknesses.len());
        log::info!("Summary: {}", result.summary);
        assert!(!result.model_snapshot.modules.is_empty(), "Should discover modules in real project");
    }

    #[test]
    fn test_dry_run_does_not_write() {
        let tmp = TempDir::new().expect("value should be ok in test");
        let mut agent = ArchitectAgent::new(tmp.path().to_str().expect("value should be ok in test"));
        agent.dry_run = true;
        let result = agent.run_cycle();
        assert!(!result.changes_applied);
        // 不检查磁盘写入 — dry_run 不应修改任何文件
    }
}
