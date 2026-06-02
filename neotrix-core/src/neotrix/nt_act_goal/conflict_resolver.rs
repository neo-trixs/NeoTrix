//! GoalConflictResolver — 进化目标冲突检测与合并
//!
//! P4-06: 检测重叠/冲突的进化目标，合并优先级。
//! 当 AutoGoalGenerator 生成大量目标时，确保不会针对同一文件
//! 产生相互矛盾的修改指令。

use crate::neotrix::nt_act_goal::goal_generator::{EvolutionGoal, GoalCategory, GoalPriority};

/// 两个目标之间的冲突描述
#[derive(Debug, Clone)]
pub struct GoalConflict {
    pub goal_a_id: String,
    pub goal_b_id: String,
    pub reason: ConflictReason,
    pub severity: ConflictSeverity,
}

/// 冲突原因分类
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictReason {
    SameFile { file: String },
    SameCategory,
    OppositePriorities,
    ResourceContention { resource: String },
}

/// 冲突严重程度
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictSeverity {
    /// 同一文件 + 优先级对立 → 必须合并
    Critical,
    /// 同一类别超过阈值 → 可以重排
    Warning,
    /// 轻微重叠 → 可忽略
    Info,
}

/// 目标冲突解析器
pub struct GoalConflictResolver {
    max_goals_per_file: usize,
    max_goals_per_category: usize,
}

impl GoalConflictResolver {
    pub fn new(max_goals_per_file: usize) -> Self {
        Self {
            max_goals_per_file,
            max_goals_per_category: 3,
        }
    }

    /// 检测所有目标中的冲突
    pub fn detect_conflicts(&self, goals: &[EvolutionGoal]) -> Vec<GoalConflict> {
        let mut conflicts = Vec::new();
        conflicts.extend(self.detect_file_overlaps(goals));
        conflicts.extend(self.detect_category_overlaps(goals));
        conflicts.extend(self.detect_priority_clashes(goals));
        conflicts
    }

    /// 解析冲突：合并冲突对，返回被移除的目标
    pub fn resolve_conflicts(goals: &mut Vec<EvolutionGoal>, conflicts: &[GoalConflict]) -> Vec<EvolutionGoal> {
        let mut removed = Vec::new();
        let mut merged_ids: Vec<String> = Vec::new();

        for conflict in conflicts {
            // 跳过已合并的目标
            if merged_ids.contains(&conflict.goal_a_id) || merged_ids.contains(&conflict.goal_b_id) {
                continue;
            }

            let pos_a = goals.iter().position(|g| g.id == conflict.goal_a_id);
            let pos_b = goals.iter().position(|g| g.id == conflict.goal_b_id);

            match (pos_a, pos_b) {
                (Some(ia), Some(ib)) => {
                    let goal_a = goals[ia].clone();
                    let goal_b = goals[ib].clone();

                    let merged = Self::merge_goals(&goal_a, &goal_b);

                    // 移除较低优先级的目标
                    let (keep_idx, remove_idx) = if priority_score(goal_a.priority) >= priority_score(goal_b.priority) {
                        (ia, ib)
                    } else {
                        (ib, ia)
                    };

                    // 移除低优先级目标
                    removed.push(goals.remove(remove_idx));
                    // 更新保留的目标为合并结果
                    let keep_pos = if remove_idx < keep_idx { keep_idx - 1 } else { keep_idx };
                    removed.push(goals.remove(keep_pos));
                    // 添加合并目标
                    goals.push(merged);

                    merged_ids.push(conflict.goal_a_id.clone());
                    merged_ids.push(conflict.goal_b_id.clone());
                }
                _ => {
                    // 其中一个目标可能已被前面的合并移除，跳过
                }
            }
        }

        removed
    }

    /// 检测同一文件上的目标重叠
    fn detect_file_overlaps(&self, goals: &[EvolutionGoal]) -> Vec<GoalConflict> {
        let mut conflicts = Vec::new();

        // 按文件分组
        let mut file_groups: Vec<Vec<&EvolutionGoal>> = Vec::new();
        let mut seen: Vec<String> = Vec::new();

        for goal in goals {
            if let Some(ref file) = goal.target_file {
                if !seen.contains(file) {
                    seen.push(file.clone());
                    let group: Vec<&EvolutionGoal> = goals.iter().filter(|g| g.target_file.as_deref() == Some(file)).collect();
                    if group.len() > 1 {
                        file_groups.push(group);
                    }
                }
            }
        }

        for group in file_groups {
            if group.len() > self.max_goals_per_file {
                // 超过阈值，检查优先级冲突
                let has_critical = group.iter().any(|g| matches!(g.priority, GoalPriority::Critical | GoalPriority::High));
                let has_low = group.iter().any(|g| matches!(g.priority, GoalPriority::Medium | GoalPriority::Low));

                if has_critical && has_low {
                    // 找到一对矛盾优先级
                    let critical_goal = group.iter().find(|g| matches!(g.priority, GoalPriority::Critical | GoalPriority::High)).unwrap();
                    let low_goal = group.iter().find(|g| matches!(g.priority, GoalPriority::Medium | GoalPriority::Low)).unwrap();

                    conflicts.push(GoalConflict {
                        goal_a_id: critical_goal.id.clone(),
                        goal_b_id: low_goal.id.clone(),
                        reason: ConflictReason::SameFile { file: critical_goal.target_file.clone().unwrap() },
                        severity: ConflictSeverity::Critical,
                    });
                }
            }
        }

        conflicts
    }

    /// 检测同一类别的目标堆积
    fn detect_category_overlaps(&self, goals: &[EvolutionGoal]) -> Vec<GoalConflict> {
        let mut conflicts = Vec::new();

        for category in &[
            GoalCategory::CodeHealth,
            GoalCategory::TestCoverage,
            GoalCategory::Architecture,
            GoalCategory::Performance,
            GoalCategory::Security,
            GoalCategory::Knowledge,
        ] {
            let group: Vec<&EvolutionGoal> = goals.iter().filter(|g| g.category == *category).collect();
            if group.len() > self.max_goals_per_category {
                // 取前两个作为冲突对
                conflicts.push(GoalConflict {
                    goal_a_id: group[0].id.clone(),
                    goal_b_id: group[1].id.clone(),
                    reason: ConflictReason::SameCategory,
                    severity: ConflictSeverity::Warning,
                });
            }
        }

        conflicts
    }

    /// 检测依赖关系中的优先级倒挂
    fn detect_priority_clashes(&self, goals: &[EvolutionGoal]) -> Vec<GoalConflict> {
        let mut conflicts = Vec::new();

        for goal in goals {
            for dep_id in &goal.dependencies {
                if let Some(dep_goal) = goals.iter().find(|g| g.id == *dep_id) {
                    if goal.priority == GoalPriority::Critical && dep_goal.priority == GoalPriority::Medium {
                        conflicts.push(GoalConflict {
                            goal_a_id: goal.id.clone(),
                            goal_b_id: dep_goal.id.clone(),
                            reason: ConflictReason::OppositePriorities,
                            severity: ConflictSeverity::Info,
                        });
                    }
                }
            }
        }

        conflicts
    }

    /// 合并两个冲突目标为一个
    pub fn merge_goals(a: &EvolutionGoal, b: &EvolutionGoal) -> EvolutionGoal {
        let merged_priority = if priority_score(a.priority) >= priority_score(b.priority) {
            a.priority
        } else {
            b.priority
        };

        let merged_file = match (&a.target_file, &b.target_file) {
            (Some(fa), Some(fb)) if fa == fb => Some(fa.clone()),
            (Some(fa), None) => Some(fa.clone()),
            (None, Some(fb)) => Some(fb.clone()),
            _ => None,
        };

        let mut merged_deps = a.dependencies.clone();
        for dep in &b.dependencies {
            if !merged_deps.contains(dep) {
                merged_deps.push(dep.clone());
            }
        }

        EvolutionGoal {
            id: format!("MERGED-{}-{}", &a.id, &b.id),
            category: if a.category == b.category { a.category } else { GoalCategory::Architecture },
            priority: merged_priority,
            description: format!("{}; {}", a.description, b.description),
            target_file: merged_file,
            expected_impact: (a.expected_impact + b.expected_impact) / 2.0,
            effort_estimate: a.effort_estimate + b.effort_estimate,
            dependencies: merged_deps,
        }
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

    fn make_goal(id: &str, priority: GoalPriority, category: GoalCategory, file: Option<&str>) -> EvolutionGoal {
        EvolutionGoal {
            id: id.to_string(),
            category,
            priority,
            description: format!("Goal {}", id),
            target_file: file.map(|s| s.to_string()),
            expected_impact: 0.5,
            effort_estimate: 0.3,
            dependencies: vec![],
        }
    }

    #[test]
    fn test_no_conflicts_empty_list() {
        let resolver = GoalConflictResolver::new(1);
        let conflicts = resolver.detect_conflicts(&[]);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_no_conflicts_single_goal() {
        let resolver = GoalConflictResolver::new(1);
        let goals = vec![make_goal("g1", GoalPriority::High, GoalCategory::CodeHealth, Some("src/main.rs"))];
        let conflicts = resolver.detect_conflicts(&goals);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_single_file_overlap_conflict() {
        let resolver = GoalConflictResolver::new(1);
        let goals = vec![
            make_goal("g1", GoalPriority::Critical, GoalCategory::CodeHealth, Some("src/main.rs")),
            make_goal("g2", GoalPriority::Low, GoalCategory::TestCoverage, Some("src/main.rs")),
        ];
        let conflicts = resolver.detect_conflicts(&goals);
        assert!(!conflicts.is_empty());
        assert!(conflicts.iter().any(|c| matches!(c.severity, ConflictSeverity::Critical)));
    }

    #[test]
    fn test_category_overlap_warning() {
        let resolver = GoalConflictResolver::new(3);
        let goals = vec![
            make_goal("g1", GoalPriority::High, GoalCategory::CodeHealth, None),
            make_goal("g2", GoalPriority::Medium, GoalCategory::CodeHealth, None),
            make_goal("g3", GoalPriority::Low, GoalCategory::CodeHealth, None),
            make_goal("g4", GoalPriority::Critical, GoalCategory::CodeHealth, None),
        ];
        let conflicts = resolver.detect_conflicts(&goals);
        assert!(conflicts.iter().any(|c| matches!(c.severity, ConflictSeverity::Warning)));
    }

    #[test]
    fn test_priority_clash_detected() {
        let resolver = GoalConflictResolver::new(3);
        let dep_goal = make_goal("dep-g", GoalPriority::Medium, GoalCategory::CodeHealth, None);
        let mut main_goal = make_goal("main", GoalPriority::Critical, GoalCategory::TestCoverage, None);
        main_goal.dependencies = vec!["dep-g".to_string()];
        let goals = vec![main_goal, dep_goal];
        let conflicts = resolver.detect_conflicts(&goals);
        assert!(conflicts.iter().any(|c| matches!(c.severity, ConflictSeverity::Info)));
    }

    #[test]
    fn test_merge_goals_produces_combined_goal() {
        let a = EvolutionGoal {
            id: "A1".into(),
            category: GoalCategory::CodeHealth,
            priority: GoalPriority::Critical,
            description: "Fix compile errors".into(),
            target_file: Some("src/main.rs".into()),
            expected_impact: 0.8,
            effort_estimate: 0.5,
            dependencies: vec![],
        };
        let b = EvolutionGoal {
            id: "B1".into(),
            category: GoalCategory::CodeHealth,
            priority: GoalPriority::Low,
            description: "Clean up warnings".into(),
            target_file: Some("src/main.rs".into()),
            expected_impact: 0.3,
            effort_estimate: 0.2,
            dependencies: vec![],
        };
        let merged = GoalConflictResolver::merge_goals(&a, &b);
        assert_eq!(merged.priority, GoalPriority::Critical);
        assert_eq!(merged.target_file, Some("src/main.rs".into()));
        assert!(merged.description.contains("Fix compile errors"));
        assert!(merged.description.contains("Clean up warnings"));
        assert_eq!(merged.id, "MERGED-A1-B1");
    }

    #[test]
    fn test_resolve_removes_conflicts() {
        let mut goals = vec![
            make_goal("g1", GoalPriority::Critical, GoalCategory::CodeHealth, Some("src/main.rs")),
            make_goal("g2", GoalPriority::Low, GoalCategory::TestCoverage, Some("src/main.rs")),
            make_goal("g3", GoalPriority::High, GoalCategory::Architecture, Some("src/lib.rs")),
        ];
        let resolver = GoalConflictResolver::new(1);
        let conflicts = resolver.detect_conflicts(&goals);
        let removed = GoalConflictResolver::resolve_conflicts(&mut goals, &conflicts);
        assert!(!removed.is_empty());
        // After merge, we should have 2 goals left: the merged one + the untouched g3
        assert_eq!(goals.len(), 2);
        // IDs of original conflicting goals should not appear
        assert!(!goals.iter().any(|g| g.id == "g1" || g.id == "g2"));
    }

    #[test]
    fn test_custom_max_goals_per_file_threshold() {
        let resolver = GoalConflictResolver::new(2);
        let goals = vec![
            make_goal("g1", GoalPriority::Critical, GoalCategory::CodeHealth, Some("src/main.rs")),
            make_goal("g2", GoalPriority::High, GoalCategory::TestCoverage, Some("src/main.rs")),
            make_goal("g3", GoalPriority::Low, GoalCategory::Architecture, Some("src/main.rs")),
        ];
        // With max=2, 3 goals on same file triggers a conflict
        let conflicts = resolver.detect_conflicts(&goals);
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn test_different_files_no_conflict() {
        let resolver = GoalConflictResolver::new(1);
        let goals = vec![
            make_goal("g1", GoalPriority::Critical, GoalCategory::CodeHealth, Some("src/main.rs")),
            make_goal("g2", GoalPriority::Low, GoalCategory::TestCoverage, Some("src/lib.rs")),
        ];
        let conflicts = resolver.detect_conflicts(&goals);
        let file_conflicts: Vec<_> = conflicts.iter().filter(|c| matches!(c.reason, ConflictReason::SameFile { .. })).collect();
        assert!(file_conflicts.is_empty());
    }
}
