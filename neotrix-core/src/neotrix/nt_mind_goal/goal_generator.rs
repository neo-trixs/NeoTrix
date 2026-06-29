//! AutoGoalGenerator — 自主进化目标生成器
//!
//! P4-01: 从 ProjectSnapshot 扫描数据 + SelfDiagnose 诊断结果
//! 自动生成"该进化什么"的目标，不再依赖 LLM 提出目标。
//!
//! 文献对齐 (2026):
//!   - SelfEvolve (arXiv, Apr 2026): 92.7% Pass@1 的自扩展框架
//!   - DGM-Hyperagents (arXiv, Mar 2026): 可编辑元层的自我改进
//!   - 核心差异: NeoTrix 的进化目标由诊断驱动而非 LLM 提议

use crate::neotrix::nt_mind_evolution_loop::ProjectSnapshot;

/// 进化目标优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// 进化目标类别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalCategory {
    CodeHealth,
    TestCoverage,
    Architecture,
    Performance,
    Security,
    Knowledge,
}

/// 单个进化目标
#[derive(Debug, Clone)]
pub struct EvolutionGoal {
    pub id: String,
    pub category: GoalCategory,
    pub priority: GoalPriority,
    pub description: String,
    pub target_file: Option<String>,
    pub expected_impact: f64,
    pub effort_estimate: f64,
    pub dependencies: Vec<String>,
}

/// 目标生成器
#[derive(Debug, Clone)]
pub struct AutoGoalGenerator;

impl AutoGoalGenerator {
    /// 从项目快照生成进化目标
    pub fn generate_from_snapshot(snapshot: &ProjectSnapshot) -> Vec<EvolutionGoal> {
        let mut goals = Vec::new();

        // 大文件 → 拆分目标
        for file in &snapshot.large_files {
            goals.push(EvolutionGoal {
                id: format!("SPLIT-{}", file.replace('/', "_")),
                category: GoalCategory::Architecture,
                priority: GoalPriority::High,
                description: format!("拆分大文件 {}", file),
                target_file: Some(file.clone()),
                expected_impact: 0.6,
                effort_estimate: 0.4,
                dependencies: vec![],
            });
        }

        // 无测试模块 → 测试目标
        for file in &snapshot.modules_without_tests {
            goals.push(EvolutionGoal {
                id: format!("TEST-{}", file.replace('/', "_")),
                category: GoalCategory::TestCoverage,
                priority: GoalPriority::High,
                description: format!("为 {} 添加测试覆盖", file),
                target_file: Some(file.clone()),
                expected_impact: 0.7,
                effort_estimate: 0.3,
                dependencies: vec![],
            });
        }

        // 编译错误 → 紧急修复
        if snapshot.compile_errors > 0 {
            goals.push(EvolutionGoal {
                id: "FIX-COMPILE-ERRORS".into(),
                category: GoalCategory::CodeHealth,
                priority: GoalPriority::Critical,
                description: format!("修复 {} 个编译错误", snapshot.compile_errors),
                target_file: None,
                expected_impact: 1.0,
                effort_estimate: 0.8,
                dependencies: vec![],
            });
        }

        // 编译警告 → 渐进清理
        if snapshot.compile_warnings > 5 {
            goals.push(EvolutionGoal {
                id: "CLEAN-WARNINGS".into(),
                category: GoalCategory::CodeHealth,
                priority: GoalPriority::Medium,
                description: format!("清理 {} 个编译警告", snapshot.compile_warnings),
                target_file: None,
                expected_impact: 0.3,
                effort_estimate: 0.3,
                dependencies: vec![],
            });
        }

        // 过多 TODO → 清理目标
        if snapshot.todo_count > 5 {
            goals.push(EvolutionGoal {
                id: "CLEAN-TODOS".into(),
                category: GoalCategory::CodeHealth,
                priority: GoalPriority::Medium,
                description: format!("清理 {} 个遗留 TODO", snapshot.todo_count),
                target_file: None,
                expected_impact: 0.2,
                effort_estimate: 0.2,
                dependencies: vec![],
            });
        }

        // unsafe 热点 → 安全审查目标
        if snapshot.unsafe_count > 5 {
            goals.push(EvolutionGoal {
                id: "AUDIT-UNSAFE".into(),
                category: GoalCategory::Security,
                priority: GoalPriority::Medium,
                description: format!("审查 {} 个 unsafe 块", snapshot.unsafe_count),
                target_file: None,
                expected_impact: 0.5,
                effort_estimate: 0.6,
                dependencies: vec![],
            });
        }

        goals
    }

    /// 综合进化报告
    pub fn summarize(goals: &[EvolutionGoal]) -> String {
        if goals.is_empty() {
            return "✅ 无进化目标 — 项目状态健康".into();
        }
        let critical = goals
            .iter()
            .filter(|g| g.priority == GoalPriority::Critical)
            .count();
        let high = goals
            .iter()
            .filter(|g| g.priority == GoalPriority::High)
            .count();
        let medium = goals
            .iter()
            .filter(|g| g.priority == GoalPriority::Medium)
            .count();
        format!(
            "🎯 {} 个进化目标 (Critical: {}, High: {}, Medium: {})",
            goals.len(),
            critical,
            high,
            medium,
        )
    }

    /// 按优先级排序
    pub fn prioritize(goals: &mut Vec<EvolutionGoal>) {
        goals.sort_by(|a, b| {
            let p_a = priority_score(a.priority);
            let p_b = priority_score(b.priority);
            p_b.cmp(&p_a).then_with(|| {
                b.expected_impact
                    .partial_cmp(&a.expected_impact)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });
    }
}

fn priority_score(p: GoalPriority) -> u8 {
    match p {
        GoalPriority::Critical => 4,
        GoalPriority::High => 3,
        GoalPriority::Medium => 2,
        GoalPriority::Low => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snapshot() -> ProjectSnapshot {
        ProjectSnapshot {
            total_files: 120,
            total_lines: 130000,
            large_files: vec!["big.rs".into(), "huge.rs".into()],
            modules_without_tests: vec!["untested.rs".into()],
            file_unsafe_hotspots: vec!["unsafe.rs".into()],
            unsafe_count: 8,
            unwrap_count: 2,
            todo_count: 12,
            compile_errors: 3,
            compile_warnings: 10,
            test_count: 1800,
            test_failures: 0,
        }
    }

    #[test]
    fn test_generates_split_goals_for_large_files() {
        let goals = AutoGoalGenerator::generate_from_snapshot(&sample_snapshot());
        let split_goals: Vec<_> = goals
            .iter()
            .filter(|g| g.id.starts_with("SPLIT-"))
            .collect();
        assert_eq!(split_goals.len(), 2);
    }

    #[test]
    fn test_generates_test_goals() {
        let goals = AutoGoalGenerator::generate_from_snapshot(&sample_snapshot());
        let test_goals: Vec<_> = goals.iter().filter(|g| g.id.starts_with("TEST-")).collect();
        assert_eq!(test_goals.len(), 1);
    }

    #[test]
    fn test_critical_compile_errors() {
        let goals = AutoGoalGenerator::generate_from_snapshot(&sample_snapshot());
        assert!(goals.iter().any(|g| g.id == "FIX-COMPILE-ERRORS"));
    }

    #[test]
    fn test_no_goals_for_healthy_project() {
        let healthy = ProjectSnapshot {
            large_files: vec![],
            modules_without_tests: vec![],
            file_unsafe_hotspots: vec![],
            unsafe_count: 0,
            unwrap_count: 0,
            todo_count: 0,
            compile_errors: 0,
            compile_warnings: 2,
            ..sample_snapshot()
        };
        let goals = AutoGoalGenerator::generate_from_snapshot(&healthy);
        assert!(goals.is_empty());
    }

    #[test]
    fn test_summarize_healthy() {
        let s = AutoGoalGenerator::summarize(&[]);
        assert!(s.contains("无进化目标"));
    }

    #[test]
    fn test_summarize_with_goals() {
        let goals = AutoGoalGenerator::generate_from_snapshot(&sample_snapshot());
        let s = AutoGoalGenerator::summarize(&goals);
        assert!(s.contains("进化目标"));
    }

    #[test]
    fn test_prioritize_critical_first() {
        let mut goals = AutoGoalGenerator::generate_from_snapshot(&sample_snapshot());
        AutoGoalGenerator::prioritize(&mut goals);
        if !goals.is_empty() {
            assert_eq!(goals[0].priority, GoalPriority::Critical);
        }
    }

    #[test]
    fn test_empty_snapshot_produces_no_goals() {
        let empty = ProjectSnapshot {
            total_files: 0,
            total_lines: 0,
            large_files: vec![],
            modules_without_tests: vec![],
            file_unsafe_hotspots: vec![],
            unsafe_count: 0,
            unwrap_count: 0,
            todo_count: 0,
            compile_errors: 0,
            compile_warnings: 0,
            test_count: 0,
            test_failures: 0,
        };
        assert!(AutoGoalGenerator::generate_from_snapshot(&empty).is_empty());
    }
}
