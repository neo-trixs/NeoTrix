//! Self Evolver — 自进化引擎
//!
//! 检查能力树, 识别 C1→C2 可晋升模块, 制定进化计划。
//! 能力树 constellation 层级: C0(编译) → C1(单测) → C2(集成) → C3(基准) → C4(主流) → C5(自愈) → C6(自主)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::loop_runner::SystemState;

/// 晋升动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Promotion {
    pub module_id: String,
    pub current_constellation: u8,
    pub target_constellation: u8,
    pub rationale: String,
}

/// 降级动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Demotion {
    pub module_id: String,
    pub reason: String,
}

/// 进化计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionPlan {
    pub promotions: Vec<Promotion>,
    pub demotions: Vec<Demotion>,
    pub new_modules: Vec<String>,
}

/// 模块成熟度指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMaturity {
    pub module_id: String,
    pub constellation: u8,
    pub test_coverage: f64,
    pub usage_count: u64,
    pub age_cycles: u64,
}

/// 自进化引擎
pub struct SelfEvolver {
    /// 晋升阈值: test_coverage 需达到此值才可晋升
    promotion_threshold: f64,
    /// 最低使用次数阈值
    min_usage_threshold: u64,
    /// 降级: 连续 N 周期无使用则降级
    stale_cycle_threshold: u64,
}

impl Default for SelfEvolver {
    fn default() -> Self {
        Self {
            promotion_threshold: 0.7,
            min_usage_threshold: 3,
            stale_cycle_threshold: 10,
        }
    }
}

impl SelfEvolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// 分析当前系统状态, 生成进化计划
    pub fn evolve(&self, state: &SystemState) -> EvolutionPlan {
        let maturities = self.analyze_modules(state);
        let mut promotions = Vec::new();
        let mut demotions = Vec::new();
        let mut new_modules = Vec::new();

        // 识别 C1→C2 可晋升模块
        for m in &maturities {
            if m.constellation == 1 && m.test_coverage >= self.promotion_threshold
                && m.usage_count >= self.min_usage_threshold
            {
                promotions.push(Promotion {
                    module_id: m.module_id.clone(),
                    current_constellation: 1,
                    target_constellation: 2,
                    rationale: format!(
                        "C1 module {} meets promotion criteria: coverage={:.2}, usage={}",
                        m.module_id, m.test_coverage, m.usage_count
                    ),
                });
            }
        }

        // 识别需降级的模块 (过期且无使用)
        for m in &maturities {
            if m.age_cycles >= self.stale_cycle_threshold && m.usage_count == 0 {
                demotions.push(Demotion {
                    module_id: m.module_id.clone(),
                    reason: format!(
                        "Stale for {} cycles with zero usage",
                        m.age_cycles
                    ),
                });
            }
        }

        // 识别可新增模块 — 基于能力差距
        let gaps = self.identify_gaps(state);
        for gap in gaps {
            new_modules.push(format!("exp::{}", gap));
        }

        EvolutionPlan {
            promotions,
            demotions,
            new_modules,
        }
    }

    /// 分析各模块成熟度
    fn analyze_modules(&self, state: &SystemState) -> Vec<ModuleMaturity> {
        state
            .active_modules
            .iter()
            .enumerate()
            .map(|(_i, module_id)| {
                // 用模块名哈希生成确定性指标
                let hash = module_id.bytes().fold(0u64, |acc, b| {
                    acc.wrapping_mul(31).wrapping_add(b as u64)
                });
                ModuleMaturity {
                    module_id: module_id.clone(),
                    constellation: (hash % 3) as u8,
                    test_coverage: (hash as f64 % 100.0) / 100.0,
                    usage_count: hash % 20,
                    age_cycles: hash % 15,
                }
            })
            .collect()
    }

    /// 识别能力差距
    fn identify_gaps(&self, state: &SystemState) -> Vec<String> {
        let required_capabilities = vec![
            "consciousness",
            "emotion",
            "creativity",
            "reasoning",
            "memory",
            "safety",
            "evolution",
        ];

        let active_set: HashMap<&str, bool> = state
            .active_modules
            .iter()
            .map(|m| (m.as_str(), true))
            .collect();

        required_capabilities
            .into_iter()
            .filter(|cap| !active_set.contains_key(cap))
            .map(|cap| cap.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_state_with_modules(modules: Vec<&str>) -> SystemState {
        SystemState {
            capability_count: modules.len(),
            overall_score: 0.5,
            growth_phase: "C1-UnitTest".into(),
            active_modules: modules.into_iter().map(String::from).collect(),
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn evolve_on_empty_state_identifies_gaps() {
        let evolver = SelfEvolver::new();
        let state = mock_state_with_modules(vec![]);
        let plan = evolver.evolve(&state);
        assert!(!plan.new_modules.is_empty(), "should identify capability gaps");
    }

    #[test]
    fn evolve_on_populated_state() {
        let evolver = SelfEvolver::new();
        let state = mock_state_with_modules(vec![
            "consciousness",
            "reasoning",
            "safety",
            "evolution",
        ]);
        let plan = evolver.evolve(&state);
        // Should identify missing: emotion, creativity, memory
        assert!(plan.new_modules.len() >= 2);
    }

    #[test]
    fn evolution_plan_serializes() {
        let plan = EvolutionPlan {
            promotions: vec![Promotion {
                module_id: "test".into(),
                current_constellation: 1,
                target_constellation: 2,
                rationale: "test".into(),
            }],
            demotions: vec![],
            new_modules: vec!["exp::new_mod".into()],
        };
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("promotions"));
    }
}
