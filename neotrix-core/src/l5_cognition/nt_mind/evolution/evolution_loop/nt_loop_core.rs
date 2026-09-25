//! 自进化循环 — 主循环 (`nt_loop_core`).
//!
//! `run_cycle` / `run_cycle_in` 进化周期 + 仪表盘 + 自我诊断入口 +
//! 双 Provider 适配 (`self_diagnose::EvolutionLoopProvider`,
//! `l1_action::evolution_loop_provider::EvolutionLoopProvider`)。
//! 从 `evolution_loop.rs` 纯搬移, 行为零变更。

use super::nt_loop_types::{EvolutionLoop, EvolutionReport, Issue, STAGNATION_LIMIT};
use crate::l5_cognition::l1_facade::ProjectSnapshot;
use crate::l5_cognition::nt_mind::evolution::evolution_daemon::IssueType;
use crate::l5_cognition::nt_mind::evolution::self_diagnose::{
    ActionPlan, CodeUnderlyingIssue, DiagnosticItem, EvolutionLoopProvider, PriorityQueue,
    PrioritizedIssue, SelfDiagnose,
};

impl EvolutionLoop {
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
        let snapshot = self._scan_project_in(target);

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
        let snapshot = self._scan_project_in(None);
        SelfDiagnose::run_diagnosis(&snapshot, self.cycle)
    }
}

impl EvolutionLoopProvider for EvolutionLoop {
    fn get_snapshot(&self) -> ProjectSnapshot {
        self.last_snapshot.clone().unwrap_or_else(|| ProjectSnapshot {
            total_files: 0,
            total_lines: 0,
            large_files: Vec::new(),
            modules_without_tests: Vec::new(),
            file_unsafe_hotspots: Vec::new(),
            unsafe_count: 0,
            unwrap_count: 0,
            todo_count: 0,
            compile_errors: 0,
            compile_warnings: 0,
            test_count: 0,
            test_failures: 0,
        })
    }

    fn self_diagnose(&mut self) -> (Vec<String>, Vec<PrioritizedIssue>) {
        let (items, pq) = Self::self_diagnose(self);
        let issues: Vec<PrioritizedIssue> = pq.into_vec().into_iter().map(|di| {
            PrioritizedIssue {
                issue: di.underlying_issue.clone(),
                score: di.composite_score,
                plan: di.action.clone(),
                action: di.action,
                composite_score: di.composite_score,
                underlying_issue: CodeUnderlyingIssue {
                    file: di.underlying_issue.file.clone().unwrap_or_default(),
                    line: 0,
                    message: di.underlying_issue.description.clone(),
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

impl crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::EvolutionLoopProvider for EvolutionLoop {
    fn get_snapshot(&self) -> crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::ProjectSnapshotLite {
        let snap = self.last_snapshot.as_ref();
        crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::ProjectSnapshotLite {
            modules: snap.map(|s| {
                let mut m: Vec<String> = Vec::new();
                m.extend(s.large_files.iter().cloned());
                m.extend(s.modules_without_tests.iter().cloned());
                m
            }).unwrap_or_default(),
            health_score: snap.map(|s| {
                let _total = s.total_files.max(1) as f64;
                (1.0 - (s.unsafe_count as f64 / 10.0).min(1.0))
                    * (1.0 - (s.unwrap_count as f64 / 50.0).min(1.0))
                    * (1.0 - (s.todo_count as f64 / 10.0).min(1.0))
            }).unwrap_or(1.0),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn self_diagnose(&mut self) -> (Vec<String>, Vec<crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::PrioritizedIssue>) {
        let (items, pq) = Self::self_diagnose(self);
        let l1_issues: Vec<crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::PrioritizedIssue> = pq.into_vec().into_iter().map(|di| {
            crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::PrioritizedIssue {
                issue: crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::Issue {
                    issue_type: format!("{:?}", di.underlying_issue.issue_type),
                    file: di.underlying_issue.file.clone(),
                    description: di.underlying_issue.description.clone(),
                },
                score: di.composite_score,
                plan: match &di.action {
                    ActionPlan::AddTestStub { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::AddTestStub { file: file.clone() },
                    ActionPlan::RunCargoFix => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::RunCargoFix,
                    ActionPlan::RemoveTodo { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::RemoveTodo { file: file.clone() },
                    ActionPlan::SplitLargeFile { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::SplitLargeFile { file: file.clone() },
                    ActionPlan::ReviewUnsafe { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::ReviewUnsafe { file: file.clone() },
                    ActionPlan::ReplaceUnwrap { file } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::ReplaceUnwrap { file: file.clone() },
                    ActionPlan::HumanDecision { reason, options } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::HumanDecision { reason: reason.clone(), options: options.clone() },
                    ActionPlan::NoAction { reason } => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::NoAction { reason: reason.clone() },
                    ActionPlan::AutoFix(s) => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::AutoFix(s.clone()),
                    ActionPlan::ManualReview(s) => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::ManualReview(s.clone()),
                    ActionPlan::Skip(s) => crate::l1_action::nt_act::nt_act_code::evolution_loop_provider::DiagnoseActionPlan::Skip(s.clone()),
                },
            }
        }).collect();
        let messages: Vec<String> = items.into_iter()
            .map(|d| format!("[{:?}] {} (score={:.2})", d.underlying_issue.issue_type, d.underlying_issue.description, d.composite_score))
            .collect();
        (messages, l1_issues)
    }

    fn on_fix_applied(&mut self) {
        self.on_fix_applied()
    }
}
