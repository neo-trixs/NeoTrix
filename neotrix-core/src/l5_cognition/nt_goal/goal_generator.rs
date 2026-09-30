//! AutoGoalGenerator — 自主进化目标生成器
//!
//! P4-01: 从 ProjectSnapshot 扫描数据 + SelfDiagnose 诊断结果
//! 自动生成"该进化什么"的目标，不再依赖 LLM 提出目标。
//!
//! 文献对齐 (2026):
//!   - SelfEvolve (arXiv, Apr 2026): 92.7% Pass@1 的自扩展框架
//!   - DGM-Hyperagents (arXiv, Mar 2026): 可编辑元层的自我改进
//!   - 核心差异: NeoTrix 的进化目标由诊断驱动而非 LLM 提议

pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;

use serde::{Deserialize, Serialize};

/// 目标优先级 —— 进化目标 / 目标库 / 目标循环 三处共用的同一类型
///
/// 2026-09-30 归并（重复类型台账 tier=1 组，3 份同层同 crate 定义）：
///   - `nt_goal::goal_generator`（本处，进化目标）—— 保留点：跨子系统 fanin 最高
///     （`nt_goal::conflict_resolver` + `nt_goal` 侧 `pub use` + `nt_mind::meta_goal_generator`）
///   - `nt_core::nt_consciousness_core::goal_setter`（目标库）→ 改 `pub use` 指向本处
///   - `nt_mind::nt_mind::evolution::goal_loop::types`（目标循环）→ 改 `pub use` 指向本处，
///     其 `rank()` / `label()` / `Ord` / `PartialOrd` impl 一并移入本处
///
/// 变体集合与序关系三处一致：Low < Medium < High < Critical。声明顺序保留本处原样
/// （Critical 优先），**排序语义由 `rank()` 承担、不依赖声明顺序** —— 故
/// `goal_loop` 的 `sort_by_key(|b| Reverse(b.priority))` 行为不变。
///
/// trait 取三处并集（各自所需能力都是子集）：`Serialize`/`Deserialize` 来自目标库的
/// `Goal`，`Hash` 同源，`Copy`/`Ord` 来自进化目标与目标循环。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GoalPriority {
    Critical,
    High,
    Medium,
    Low,
}

impl GoalPriority {
    pub fn rank(&self) -> u8 {
        match self {
            GoalPriority::Low => 0,
            GoalPriority::Medium => 1,
            GoalPriority::High => 2,
            GoalPriority::Critical => 3,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            GoalPriority::Low => "low",
            GoalPriority::Medium => "medium",
            GoalPriority::High => "high",
            GoalPriority::Critical => "critical",
        }
    }
}

impl PartialOrd for GoalPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GoalPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
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
    /// CRT 多尺度时间视野 (nt_core_crt 接线 — 意识体维度升维):
    /// 目标生成具备战术/运营/战略三层时间意识, 而非扁平化即时任务。
    pub time_scale: crate::l5_cognition::nt_core::nt_crt::CrtTimeScale,
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
                time_scale: crate::l5_cognition::nt_core::nt_crt::CrtTimeScale::Huntian,
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
                time_scale: crate::l5_cognition::nt_core::nt_crt::CrtTimeScale::Gaitian,
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
                time_scale: crate::l5_cognition::nt_core::nt_crt::CrtTimeScale::Gaitian,
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
                time_scale: crate::l5_cognition::nt_core::nt_crt::CrtTimeScale::Huntian,
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
                time_scale: crate::l5_cognition::nt_core::nt_crt::CrtTimeScale::Gaitian,
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
                time_scale: crate::l5_cognition::nt_core::nt_crt::CrtTimeScale::Xuanye,
            });
        }

        goals
    }

    /// 综合进化报告
    pub fn summarize(goals: &[EvolutionGoal]) -> String {
        if goals.is_empty() {
            return "✅ 无进化目标 — 项目状态健康".into();
        }
        let critical = goals.iter().filter(|g| g.priority == GoalPriority::Critical).count();
        let high = goals.iter().filter(|g| g.priority == GoalPriority::High).count();
        let medium = goals.iter().filter(|g| g.priority == GoalPriority::Medium).count();
        format!(
            "🎯 {} 个进化目标 (Critical: {}, High: {}, Medium: {})",
            goals.len(), critical, high, medium,
        )
    }

    /// 按优先级排序
    pub fn prioritize(goals: &mut Vec<EvolutionGoal>) {
        goals.sort_by(|a, b| {
            let p_a = priority_score(a.priority);
            let p_b = priority_score(b.priority);
            p_b.cmp(&p_a)
                .then_with(|| b.expected_impact.partial_cmp(&a.expected_impact).unwrap_or(std::cmp::Ordering::Equal))
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
        let split_goals: Vec<_> = goals.iter().filter(|g| g.id.starts_with("SPLIT-")).collect();
        assert_eq!(split_goals.len(), 2);
    }

    #[test]
    fn test_goals_have_crt_time_scales() {
        // 意识体维度升维: 目标应具备 CRT 多尺度时间视野 (战术/运营/战略)
        use crate::l5_cognition::nt_core::nt_crt::CrtTimeScale;
        let goals = AutoGoalGenerator::generate_from_snapshot(&sample_snapshot());
        assert!(!goals.is_empty());
        for g in &goals {
            let _ = g.time_scale; // 字段存在即可
        }
        // 编译错误 → 战术 (Gaitian)
        let fix_goal = goals.iter().find(|g| g.id == "FIX-COMPILE-ERRORS").unwrap();
        assert_eq!(fix_goal.time_scale, CrtTimeScale::Gaitian);
        // 大文件拆分 → 运营 (Huntian)
        let split_goal = goals.iter().find(|g| g.id.starts_with("SPLIT-")).unwrap();
        assert_eq!(split_goal.time_scale, CrtTimeScale::Huntian);
        // unsafe 审查 → 战略 (Xuanye)
        let audit_goal = goals.iter().find(|g| g.id == "AUDIT-UNSAFE").unwrap();
        assert_eq!(audit_goal.time_scale, CrtTimeScale::Xuanye);
        // 三种尺度应覆盖 (战术/运营/战略全链路)
        let scales: std::collections::HashSet<_> =
            goals.iter().map(|g| g.time_scale).collect();
        assert!(scales.contains(&CrtTimeScale::Gaitian), "应有战术尺度");
        assert!(scales.contains(&CrtTimeScale::Huntian), "应有运营尺度");
        assert!(scales.contains(&CrtTimeScale::Xuanye), "应有战略尺度");
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

#[cfg(test)]
mod dup_merge_tests {
    use super::*;

    /// 归并前编译不过：goal_setter / goal_loop 各自持有独立的同名类型。
    #[test]
    fn goal_priority_is_one_type_across_modules() {
        let a: GoalPriority = GoalPriority::High;
        let b: crate::l5_cognition::nt_core::nt_consciousness_core::goal_setter::GoalPriority = a;
        let c: crate::l5_cognition::nt_mind::nt_mind::evolution::goal_loop::types::GoalPriority =
            b;
        assert_eq!(c, GoalPriority::High);
    }
}
