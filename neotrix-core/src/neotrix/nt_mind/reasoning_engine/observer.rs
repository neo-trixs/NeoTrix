//! +1 观察者：元认知轨迹分析 + 自迭代

use crate::core::CapabilityVector;
use crate::core::evolve_strategy_entry;
use crate::core::ObserverReport;
use crate::neotrix::nt_mind::reasoning_engine::ReasoningEngine;
use crate::neotrix::nt_mind::memory::ReasoningMemory;
use crate::neotrix::nt_mind::distillation::{ExperienceDistiller, apply_principles, avoid_anti_patterns};
use crate::neotrix::nt_mind::core::PerformanceEvaluator;
use crate::neotrix::nt_world_model::TaskType;

impl ReasoningEngine {
    pub fn observer_analyze(&mut self, task: &str) -> ObserverReport {
        let keywords: Vec<&str> = task.split_whitespace().collect();
        let report = self.observer.analyze(&self.state_trajectory, &keywords);

        if let Some(meta) = report.recommended_meta {
            self.current_state.meta = meta;
            self.state_trajectory.push(self.current_state);
        }

        if report.has_critical_pattern() {
            self.current_state = self.current_state.reflect();
            self.state_trajectory.push(self.current_state);
        }

        if report.has_actionable_insight && !report.capability_deltas.is_empty() {
            for (name, delta) in &report.capability_deltas {
                if let Some(idx) = CapabilityVector::index_from_name(name) {
                    let cur = self.brain.capability.arr()[idx];
                    self.brain.capability.arr_mut()[idx] = (cur + delta).clamp(0.0, 1.0);
                }
            }
            self.brain.capability.normalize();
        }

        let current_hex = self.current_state.mode;
        for pattern in &report.patterns {
            let name = format!("{:?}", pattern);
            let _evolved = evolve_strategy_entry(&mut self.strategy_matrix, current_hex, &name);
        }

        report
    }

    pub fn self_iterate(&mut self) {
        let before_score = PerformanceEvaluator::evaluate(&TaskType::General, &self.brain.capability);

        let new_principles = ExperienceDistiller::distill_traces(&self.traces);
        for p in new_principles {
            if !self.principles.iter().any(|existing| existing.description == p.description) {
                self.principles.push(p);
            }
        }

        let new_anti = ExperienceDistiller::contrastive_reflect_traces(&self.traces);
        for a in new_anti {
            if !self.anti_patterns.iter().any(|existing| existing.description == a.description) {
                self.anti_patterns.push(a);
            }
        }

        apply_principles(&mut self.brain.capability, &self.principles, 0.5);
        avoid_anti_patterns(&mut self.brain.capability, &self.anti_patterns);

        let type_coverage = self.traces.iter()
            .map(|t| t.reasoning_type)
            .collect::<std::collections::HashSet<_>>()
            .len() as f64 / 5.0;
        let analysis_idx = CapabilityVector::index_from_name("analysis").unwrap_or(10);
        let current_analysis = self.brain.capability.arr()[analysis_idx];
        self.brain.capability.arr_mut()[analysis_idx] = (current_analysis + type_coverage * 0.05).min(1.0);

        self.brain.capability.normalize();

        let after_score = PerformanceEvaluator::evaluate(&TaskType::General, &self.brain.capability);
        let improved = after_score - before_score;

        let summary = format!(
            "Self-iteration: {} principles, {} anti-patterns, capability improved by {:.3}",
            self.principles.len(), self.anti_patterns.len(), improved
        );
        let memory = ReasoningMemory::new(&summary, TaskType::General, &[], improved.max(0.0));
        self.bank.store(memory);
    }
}
