use crate::core::nt_core_meta::self_model::SelfModel;
use crate::core::nt_core_meta::weakness::{Weakness, WeaknessReport};
use crate::core::nt_core_meta::planner::PlannedEvolution;

/// 架构设计输出 — 自主设计的结果
#[derive(Clone, Debug)]
pub struct ArchitectureDesign {
    pub new_modules: Vec<ModuleBlueprint>,
    pub refactoring_plans: Vec<RefactoringPlan>,
    pub rationale: String,
}

/// 新模块蓝图
#[derive(Clone, Debug)]
pub struct ModuleBlueprint {
    pub name: String,
    pub parent_module: String,
    pub file_name: String,
    pub types: Vec<TypeBlueprint>,
    pub traits: Vec<TraitBlueprint>,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct TypeBlueprint {
    pub name: String,
    pub kind: TypeKind,
    pub fields: Vec<FieldBlueprint>,
}

#[derive(Clone, Debug)]
pub enum TypeKind {
    Struct,
    Enum,
    Newtype(String),
}

#[derive(Clone, Debug)]
pub struct FieldBlueprint {
    pub name: String,
    pub type_expr: String,
    pub is_pub: bool,
}

#[derive(Clone, Debug)]
pub struct TraitBlueprint {
    pub name: String,
    pub methods: Vec<MethodBlueprint>,
}

#[derive(Clone, Debug)]
pub struct MethodBlueprint {
    pub name: String,
    pub signature: String,
    pub body_template: String,
}

/// 重构计划
#[derive(Clone, Debug)]
pub struct RefactoringPlan {
    pub target_module: String,
    pub target_file: String,
    pub description: String,
    pub actions: Vec<CodeAction>,
}

#[derive(Clone, Debug)]
pub enum CodeAction {
    SplitModule {
        new_sub_modules: Vec<String>,
    },
    AddTests {
        test_file: String,
        test_functions: Vec<String>,
    },
    ReplaceUnwrap {
        line_hint: usize,
        pattern: String,
        replacement: String,
    },
    ExtractInterface {
        trait_name: String,
        methods: Vec<String>,
    },
    AddDocumentation {
        target: String,
    },
}

/// 基于规则的架构设计师 — 不依赖 LLM
pub struct ArchitectureDesigner {
    pub max_new_modules_per_cycle: usize,
    pub split_threshold_lines: usize,
}

impl Default for ArchitectureDesigner {
    fn default() -> Self {
        Self {
            max_new_modules_per_cycle: 2,
            split_threshold_lines: 600,
        }
    }
}

impl ArchitectureDesigner {
    pub fn new() -> Self {
        Self::default()
    }

    /// 根据弱点报告和演化计划生成架构设计
    pub fn design(
        &self,
        report: &WeaknessReport,
        plans: &[PlannedEvolution],
        model: &SelfModel,
    ) -> ArchitectureDesign {
        let mut new_modules = Vec::new();
        let mut refactoring_plans = Vec::new();

        for plan in plans {
            match plan.weakness.pattern_id.as_str() {
                "LARGE_FILE" => {
                    if let Some(plan) = self.design_split(&plan.weakness, model) {
                        refactoring_plans.push(plan);
                    }
                }
                "MISSING_TESTS" => {
                    if let Some(plan) = self.design_tests(&plan.weakness) {
                        refactoring_plans.push(plan);
                    }
                }
                "EXCESS_UNWRAP" => {
                    if let Some(plan) = self.design_error_handling(&plan.weakness) {
                        refactoring_plans.push(plan);
                    }
                }
                "CIRCULAR_DEP" => {
                    if let Some(plan) = self.design_extract_interface(&plan.weakness, model) {
                        refactoring_plans.push(plan);
                    }
                    if let Some(ref module) = plan.weakness.target_module {
                        let blueprint = ModuleBlueprint {
                            name: format!("{}_interface", module),
                            parent_module: module.split('.').next().unwrap_or("core").to_string(),
                            file_name: "interface.rs".to_string(),
                            types: vec![],
                            traits: vec![TraitBlueprint {
                                name: format!("{}Interface", to_pascal(module)),
                                methods: vec![],
                            }],
                            description: format!("Shared interface extracted from {} to break circular dependency", module),
                        };
                        if new_modules.len() < self.max_new_modules_per_cycle {
                            new_modules.push(blueprint);
                        }
                    }
                }
                "ORPHAN_MODULE" => {
                    if let Some(plan) = self.design_integration(&plan.weakness) {
                        refactoring_plans.push(plan);
                    }
                }
                _ => {}
            }
        }

        let rationale = if refactoring_plans.is_empty() && new_modules.is_empty() {
            "No actionable architecture changes identified".to_string()
        } else {
            format!(
                "Designing {} new modules and {} refactoring plans based on {} weaknesses",
                new_modules.len(),
                refactoring_plans.len(),
                report.weaknesses.len(),
            )
        };

        ArchitectureDesign { new_modules, refactoring_plans, rationale }
    }

    fn design_split(&self, weakness: &Weakness, model: &SelfModel) -> Option<RefactoringPlan> {
        let module = model.modules.iter().find(|m| {
            weakness.target_module.as_ref().is_some_and(|tm| m.name == *tm)
        })?;
        if module.total_lines < self.split_threshold_lines {
            return None;
        }
        let sub_names = guess_sub_modules(&module.name, &module.path);
        let target_mod = weakness.target_module.clone().unwrap_or_default();
        Some(RefactoringPlan {
            target_module: target_mod,
            target_file: format!("{}/mod.rs", module.path.trim_end_matches("/mod.rs")),
            description: format!("Split {} ({}.{} lines) into {} sub-modules",
                module.name, module.name, module.total_lines, sub_names.len()),
            actions: vec![CodeAction::SplitModule { new_sub_modules: sub_names }],
        })
    }

    fn design_tests(&self, weakness: &Weakness) -> Option<RefactoringPlan> {
        let target_mod = weakness.target_module.clone().unwrap_or_default();
        let file_name = weakness.file.as_deref().unwrap_or("src/lib.rs");
        Some(RefactoringPlan {
            target_module: target_mod.clone(),
            target_file: format!("{}/tests.rs", file_name.trim_end_matches(".rs")),
            description: format!("Add test coverage for {}", target_mod),
            actions: vec![CodeAction::AddTests {
                test_file: format!("{}/tests.rs", file_name.trim_end_matches(".rs")),
                test_functions: vec![
                    "test_sanity".to_string(),
                    "test_edge_cases".to_string(),
                ],
            }],
        })
    }

    fn design_error_handling(&self, weakness: &Weakness) -> Option<RefactoringPlan> {
        let target_mod = weakness.target_module.clone().unwrap_or_default();
        let file_name = weakness.file.clone().unwrap_or_default();
        Some(RefactoringPlan {
            target_module: target_mod,
            target_file: file_name.clone(),
            description: format!("Replace .unwrap() calls with ? operator in {}", file_name),
            actions: vec![CodeAction::ReplaceUnwrap {
                line_hint: weakness.line.unwrap_or(0),
                pattern: ".unwrap()".to_string(),
                replacement: "? /* TODO: replace with proper error handling */".to_string(),
            }],
        })
    }

    fn design_extract_interface(&self, _weakness: &Weakness, _model: &SelfModel) -> Option<RefactoringPlan> {
        None
    }

    fn design_integration(&self, weakness: &Weakness) -> Option<RefactoringPlan> {
        let target_mod = weakness.target_module.clone().unwrap_or_default();
        let file_name = weakness.file.clone().unwrap_or_default();
        Some(RefactoringPlan {
            target_module: target_mod,
            target_file: file_name,
            description: format!("Integrate orphan module {:?}", weakness.target_module),
            actions: vec![],
        })
    }
}

fn guess_sub_modules(module_name: &str, _path: &str) -> Vec<String> {
    let candidates = vec![
        format!("{}_core", module_name),
        format!("{}_utils", module_name),
        format!("{}_types", module_name),
    ];
    candidates.into_iter().take(2).collect()
}

fn to_pascal(name: &str) -> String {
    name.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_meta::self_model::DebtSeverity;

    fn sample_weakness(pattern_id: &str, module: &str, severity: DebtSeverity) -> Weakness {
        Weakness {
            pattern_id: pattern_id.to_string(),
            target_module: Some(module.to_string()),
            file: Some(format!("src/{}/mod.rs", module)),
            line: Some(1),
            severity,
            description: format!("Test: {}", pattern_id),
            impact: "test".to_string(),
            suggestion: "fix it".to_string(),
        }
    }

    fn empty_report() -> WeaknessReport {
        WeaknessReport {
            timestamp: chrono::Utc::now(),
            weaknesses: vec![],
            summary: crate::core::nt_core_meta::weakness::WeaknessSummary {
                total_count: 0, critical_count: 0, major_count: 0,
                minor_count: 0, cosmetic_count: 0,
            },
        }
    }

    #[test]
    fn test_empty_report_produces_no_design() {
        let designer = ArchitectureDesigner::new();
        let report = empty_report();
        let model = SelfModel::new();
        let design = designer.design(&report, &[], &model);
        assert!(design.new_modules.is_empty());
        assert!(design.refactoring_plans.is_empty());
    }

    #[test]
    fn test_large_file_triggers_split_design() {
        let designer = ArchitectureDesigner { split_threshold_lines: 100, ..Default::default() };
        let w = sample_weakness("LARGE_FILE", "core.memory", DebtSeverity::Minor);
        let report = WeaknessReport {
            timestamp: chrono::Utc::now(),
            weaknesses: vec![w.clone()],
            summary: crate::core::nt_core_meta::weakness::WeaknessSummary {
                total_count: 1, critical_count: 0, major_count: 0,
                minor_count: 1, cosmetic_count: 0,
            },
        };
        let mut model = SelfModel::new();
        model.modules.push(crate::core::nt_core_meta::self_model::ModuleInfo {
            name: "core.memory".to_string(),
            path: "src/core/memory/".to_string(),
            file_count: 1,
            total_lines: 500,
            test_count: 0,
            has_tests: false,
            unsafe_count: 0,
            unwrap_count: 0,
            todo_count: 0,
            public_api_count: 0,
            description: "memory module".to_string(),
        });
        let plan = PlannedEvolution {
            id: "EVO-1".to_string(),
            priority: 1,
            weakness: w,
            target_module: Some("core.memory".to_string()),
            action: "split core.memory".to_string(),
            estimated_impact: crate::core::nt_core_meta::planner::ImpactEstimate {
                files_affected: 3,
                risk: crate::core::nt_core_meta::planner::RiskLevel::Low,
            },
            dependencies: vec![],
        };
        let design = designer.design(&report, &[plan], &model);
        assert!(design.refactoring_plans.len() >= 1);
    }

    #[test]
    fn test_missing_tests_triggers_test_design() {
        let designer = ArchitectureDesigner::new();
        let w = sample_weakness("MISSING_TESTS", "core.foo", DebtSeverity::Major);
        let plan = PlannedEvolution {
            id: "EVO-2".to_string(), priority: 1,
            weakness: w.clone(),
            target_module: Some("core.foo".to_string()),
            action: "add tests".to_string(),
            estimated_impact: crate::core::nt_core_meta::planner::ImpactEstimate {
                files_affected: 1, risk: crate::core::nt_core_meta::planner::RiskLevel::Low,
            },
            dependencies: vec![],
        };
        let report = WeaknessReport {
            timestamp: chrono::Utc::now(),
            weaknesses: vec![w],
            summary: crate::core::nt_core_meta::weakness::WeaknessSummary {
                total_count: 1, critical_count: 0, major_count: 1,
                minor_count: 0, cosmetic_count: 0,
            },
        };
        let model = SelfModel::new();
        let design = designer.design(&report, &[plan], &model);
        assert!(design.refactoring_plans.iter().any(|p| !p.actions.is_empty()));
    }

    #[test]
    fn test_circular_dep_creates_interface_module() {
        let designer = ArchitectureDesigner::new();
        let w = sample_weakness("CIRCULAR_DEP", "agent.team", DebtSeverity::Critical);
        let plan = PlannedEvolution {
            id: "EVO-3".to_string(), priority: 0,
            weakness: w.clone(),
            target_module: Some("agent.team".to_string()),
            action: "extract interface".to_string(),
            estimated_impact: crate::core::nt_core_meta::planner::ImpactEstimate {
                files_affected: 5, risk: crate::core::nt_core_meta::planner::RiskLevel::High,
            },
            dependencies: vec![],
        };
        let report = WeaknessReport {
            timestamp: chrono::Utc::now(),
            weaknesses: vec![w],
            summary: crate::core::nt_core_meta::weakness::WeaknessSummary {
                total_count: 1, critical_count: 1, major_count: 0,
                minor_count: 0, cosmetic_count: 0,
            },
        };
        let model = SelfModel::new();
        let design = designer.design(&report, &[plan], &model);
        let interface_mods: Vec<_> = design.new_modules.iter().filter(|m| m.name.contains("interface")).collect();
        assert!(!interface_mods.is_empty());
    }

    #[test]
    fn test_to_pascal() {
        assert_eq!(to_pascal("hello_world"), "HelloWorld");
        assert_eq!(to_pascal("foo"), "Foo");
        assert_eq!(to_pascal("a.b.c"), "ABC");
    }
}
