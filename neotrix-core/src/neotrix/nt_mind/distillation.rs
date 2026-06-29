//! Experience Distillation + Contrastive Reflection
//!
//! Fix 2 (EvolveR): 将原始 MicroEdit 序列蒸馏为抽象策略原则
//! Fix 3 (Self-Consolidation): 从失败记忆中提取错误模式（对比反思）

use super::super::nt_expert_routing::TaskType;
use super::core::CapabilityVector;
use super::memory::ReasoningMemory;
use super::self_edit::MicroEdit;
use crate::core::nt_core_meta::{
    CodeScanner, MetaCognitiveLoop, MetaCycleResult, SelfModel, WeaknessAnalyzer,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 抽象策略原则 — 从经验中蒸馏的可复用洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicPrinciple {
    pub id: String,
    pub description: String,
    pub task_type: TaskType,
    /// 抽象后的维度调整模式（维度名 → 调整幅度）
    pub adjustment_pattern: HashMap<String, f64>,
    /// 该原则的历史平均奖励
    pub avg_reward: f64,
    /// 成功应用次数
    pub application_count: u32,
}

/// 反模式 — 从失败中学习的错误模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPattern {
    pub id: String,
    pub description: String,
    pub task_type: TaskType,
    /// 导致失败的维度调整模式
    pub harmful_pattern: HashMap<String, f64>,
    /// 观察到的失败次数
    pub failure_count: u32,
}

/// 经验蒸馏器 — 将原始记忆转化为抽象策略
pub struct ExperienceDistiller;

impl ExperienceDistiller {
    /// 从一组记忆中蒸馏出策略原则
    /// 1. 按 task_type 分组
    /// 2. 对每组，提取 MicroEdit 的公共模式
    /// 3. 加权平均维度调整值（以 reward 为权重）
    pub fn distill(memories: &[ReasoningMemory]) -> Vec<StrategicPrinciple> {
        let mut grouped: HashMap<TaskType, Vec<&ReasoningMemory>> = HashMap::new();
        for m in memories {
            grouped.entry(m.task_type).or_default().push(m);
        }

        let mut principles = Vec::new();
        for (task_type, group) in grouped {
            if group.len() < 2 {
                continue;
            }

            let mut pattern: HashMap<String, (f64, f64)> = HashMap::new(); // dim → (weighted_sum, total_weight)
            let mut total_reward = 0.0;

            for m in &group {
                let w = m.reward.max(0.0);
                total_reward += w;
                for edit in &m.micro_edits {
                    if let MicroEdit::AdjustDimension(dim, amount) = edit {
                        let entry = pattern.entry(dim.clone()).or_insert((0.0, 0.0));
                        entry.0 += amount * w;
                        entry.1 += w;
                    }
                    if let MicroEdit::BatchAdjust(pairs) = edit {
                        for (dim, amount) in pairs {
                            let entry = pattern.entry(dim.clone()).or_insert((0.0, 0.0));
                            entry.0 += amount * w;
                            entry.1 += w;
                        }
                    }
                }
            }

            let avg_reward = total_reward / group.len() as f64;
            let adjustment_pattern: HashMap<String, f64> = pattern
                .into_iter()
                .map(|(k, (sum, w))| (k, if w > 0.0 { sum / w } else { 0.0 }))
                .filter(|(_, v)| v.abs() > 0.01)
                .collect();

            if !adjustment_pattern.is_empty() {
                let desc = Self::describe_pattern(&adjustment_pattern, &task_type);
                principles.push(StrategicPrinciple {
                    id: uuid::Uuid::new_v4().to_string(),
                    description: desc,
                    task_type,
                    adjustment_pattern,
                    avg_reward,
                    application_count: group.len() as u32,
                });
            }
        }
        principles
    }

    /// 对比反思 — 从成功和失败记忆的对中提取反模式
    /// 匹配相同 task_type 的 (success, failure) 对
    /// 成功 = reward > 0.7, 失败 = reward < 0.3
    pub fn contrastive_reflect(memories: &[ReasoningMemory]) -> Vec<AntiPattern> {
        let successes: Vec<&ReasoningMemory> = memories.iter().filter(|m| m.reward > 0.7).collect();
        let failures: Vec<&ReasoningMemory> = memories.iter().filter(|m| m.reward < 0.3).collect();

        let mut anti_patterns = Vec::new();

        for fail in &failures {
            // 找同类型的成功案例做对比
            let similar_success = successes
                .iter()
                .filter(|s| s.task_type == fail.task_type)
                .max_by(|a, b| {
                    a.reward
                        .partial_cmp(&b.reward)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

            if let Some(success) = similar_success {
                // 提取成功和失败的 MicroEdit 差异
                let mut harmful = HashMap::new();
                for edit in &fail.micro_edits {
                    if let MicroEdit::AdjustDimension(dim, amount) = edit {
                        if amount.abs() > 0.05 {
                            let has_success = success.micro_edits.iter().any(
                                |se| matches!(se, MicroEdit::AdjustDimension(d, _) if d == dim),
                            );
                            if !has_success {
                                // 这个维度调整在成功案例中不存在 — 可能是反模式
                                *harmful.entry(dim.clone()).or_insert(0.0) += *amount;
                            }
                        }
                    }
                }

                if !harmful.is_empty() {
                    let desc = format!(
                        "Avoid adjusting {:?} for {:?} — present in failures but not in successful cases",
                        harmful.keys().collect::<Vec<_>>(),
                        fail.task_type
                    );
                    anti_patterns.push(AntiPattern {
                        id: uuid::Uuid::new_v4().to_string(),
                        description: desc,
                        task_type: fail.task_type,
                        harmful_pattern: harmful,
                        failure_count: 1,
                    });
                }
            }
        }
        anti_patterns
    }

    fn describe_pattern(pattern: &HashMap<String, f64>, task_type: &TaskType) -> String {
        let dims: Vec<&String> = pattern.keys().collect();
        format!(
            "Distilled strategy for {:?}: adjust {} dimensions ({:.2?})",
            task_type,
            dims.len(),
            dims
        )
    }
}

/// 应用策略原则到 CapabilityVector
pub fn apply_principles(
    cv: &mut CapabilityVector,
    principles: &[StrategicPrinciple],
    threshold: f64,
) {
    for p in principles {
        if p.avg_reward < threshold {
            continue;
        }
        for (dim, amount) in &p.adjustment_pattern {
            if let Some(idx) = CapabilityVector::index_from_name(dim) {
                cv.arr_mut()[idx] = (cv.arr()[idx] + amount * p.avg_reward).clamp(0.0, 1.0);
            }
        }
    }
}

/// 应用反模式到 CapabilityVector（反向调整，避免有害模式）
pub fn avoid_anti_patterns(cv: &mut CapabilityVector, anti_patterns: &[AntiPattern]) {
    for ap in anti_patterns {
        for (dim, amount) in &ap.harmful_pattern {
            if let Some(idx) = CapabilityVector::index_from_name(dim) {
                // 反向调整：如果反模式是增加，则避免（调低）
                cv.arr_mut()[idx] = (cv.arr()[idx] - amount.abs() * 0.5).max(0.0);
            }
        }
    }
}

// ============================================================================
// Metacognition Bridge — connects core::metacognition to nt_mind
// ============================================================================

#[derive(Debug, Clone)]
pub struct ModuleDelta {
    pub module: String,
    pub new_weaknesses: usize,
    pub resolved_weaknesses: usize,
    pub line_change: isize,
    pub compilation_errors: usize,
    pub compilation_warnings: usize,
}

/// Bridge that connects metacognition to the nt_mind layer.
/// Runs metacognitive cycles with continuity (no reset between cycles),
/// tracks per-module evolution, and exposes weakness data for B-Brain consumption.
pub struct MetaCognitionBridge {
    pub metacog_loop: MetaCognitiveLoop,
    pub weak_analyzer: WeaknessAnalyzer,
    pub project_root: String,
    pub last_scan: Option<SelfModel>,
    pub last_result: Option<MetaCycleResult>,
    pub delta_history: Vec<ModuleDelta>,
}

impl MetaCognitionBridge {
    pub fn new(project_root: &str) -> Self {
        let model = SelfModel::new();
        Self {
            metacog_loop: MetaCognitiveLoop::new(model),
            weak_analyzer: WeaknessAnalyzer::new(),
            project_root: project_root.to_string(),
            last_scan: None,
            last_result: None,
            delta_history: Vec::new(),
        }
    }

    pub fn run_full_cycle(&mut self) -> MetaCycleResult {
        let scanner = CodeScanner::new(&self.project_root);
        let model = scanner.scan();
        let previous = self.last_scan.take();
        self.metacog_loop.self_model = model.clone();
        let result = self.metacog_loop.run_cycle();
        if let Some(prev) = previous {
            let delta = self.compute_delta(&prev, &model, &result);
            self.delta_history.push(delta);
        }
        self.last_scan = Some(model);
        self.last_result = Some(result.clone());
        result
    }

    fn compute_delta(
        &self,
        prev: &SelfModel,
        current: &SelfModel,
        result: &MetaCycleResult,
    ) -> ModuleDelta {
        let _prev_err_count = prev.compilation.errors;
        let curr_errors = current.compilation.errors;
        let prev_lines: usize = prev.modules.iter().map(|m| m.total_lines).sum();
        let curr_lines: usize = current.modules.iter().map(|m| m.total_lines).sum();
        let top_module = result
            .plans
            .first()
            .and_then(|p| p.target_module.as_deref())
            .unwrap_or("unknown");
        ModuleDelta {
            module: top_module.to_string(),
            new_weaknesses: result
                .report
                .summary
                .total_count
                .saturating_sub(prev.tech_debt.total_count),
            resolved_weaknesses: prev
                .tech_debt
                .total_count
                .saturating_sub(result.report.summary.total_count),
            line_change: curr_lines as isize - prev_lines as isize,
            compilation_errors: curr_errors,
            compilation_warnings: current.compilation.warnings,
        }
    }

    pub fn quick_scan(&self) -> crate::core::nt_core_meta::WeaknessReport {
        let scanner = CodeScanner::new(&self.project_root);
        let model = scanner.scan();
        self.weak_analyzer.analyze(&model)
    }

    pub fn weakest_modules(&self, n: usize) -> Vec<(String, usize)> {
        let ranked = self.metacog_loop.planner.weakest_modules();
        ranked.into_iter().take(n).collect()
    }

    pub fn module_weakness_flags(&self, max_flags: usize) -> Vec<String> {
        let mut flags: Vec<String> = Vec::new();
        let weakest = self.weakest_modules(max_flags);
        for (module, score) in &weakest {
            flags.push(format!("metacog: {} weakness_score={}", module, score));
        }
        if let Some(result) = &self.last_result {
            if !result.health_check.compilation_ok {
                flags.push("metacog: compilation errors detected".into());
            }
            if let Some(delta) = self.delta_history.last() {
                if delta.new_weaknesses > 5 {
                    flags.push(format!(
                        "metacog: {} new weaknesses since last scan",
                        delta.new_weaknesses
                    ));
                }
            }
        }
        flags.truncate(max_flags);
        flags
    }

    pub fn module_roadmaps(&self) -> HashMap<String, Vec<String>> {
        let mut roadmaps: HashMap<String, Vec<String>> = HashMap::new();
        for (module, _score) in self.weakest_modules(10) {
            let plans = self.metacog_loop.planner.module_roadmap(&module);
            let steps: Vec<String> = plans
                .iter()
                .map(|p| {
                    let sev = match p.weakness.severity {
                        crate::core::nt_core_meta::DebtSeverity::Critical => "CRIT",
                        crate::core::nt_core_meta::DebtSeverity::Major => "MAJ",
                        crate::core::nt_core_meta::DebtSeverity::Minor => "MIN",
                        crate::core::nt_core_meta::DebtSeverity::Cosmetic => "COS",
                    };
                    format!("[{}] {}: {}", sev, p.weakness.pattern_id, p.action)
                })
                .collect();
            if !steps.is_empty() {
                roadmaps.insert(module, steps);
            }
        }
        roadmaps
    }

    pub fn enriched_status_summary(&self) -> String {
        let base = self.status_summary();
        let deltas: String = match self.delta_history.last() {
            None => "no prior scan".into(),
            Some(d) => format!(
                "delta: +{} new / -{} resolved weaknesses, {} lines, {} err/{} warn",
                d.new_weaknesses,
                d.resolved_weaknesses,
                d.line_change,
                d.compilation_errors,
                d.compilation_warnings,
            ),
        };
        let weakest: String = {
            let top = self.weakest_modules(3);
            if top.is_empty() {
                "no weaknesses".into()
            } else {
                let parts: Vec<String> = top.iter().map(|(m, s)| format!("{}={}", m, s)).collect();
                parts.join(", ")
            }
        };
        format!("{} | {} | weakest: {}", base, deltas, weakest)
    }

    pub fn status_summary(&self) -> String {
        self.metacog_loop.status_summary()
    }

    pub fn weakness_count(&self) -> usize {
        self.metacog_loop.self_model.tech_debt.total_count
    }

    pub fn critical_count(&self) -> usize {
        self.metacog_loop
            .self_model
            .tech_debt
            .items
            .iter()
            .filter(|i| i.severity == crate::core::nt_core_meta::DebtSeverity::Critical)
            .count()
    }

    pub fn pending_evolutions(&self) -> usize {
        self.metacog_loop.planner.pending_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distill_empty() {
        let principles = ExperienceDistiller::distill(&[]);
        assert!(principles.is_empty());
    }

    #[test]
    fn test_contrastive_reflect_empty() {
        let anti = ExperienceDistiller::contrastive_reflect(&[]);
        assert!(anti.is_empty());
    }

    #[test]
    fn test_bridge_new() {
        let bridge = MetaCognitionBridge::new("/tmp");
        assert_eq!(bridge.weakness_count(), 0);
        assert_eq!(bridge.pending_evolutions(), 0);
        assert!(bridge.status_summary().contains("MetaCognition Cycle"));
    }

    #[test]
    fn test_bridge_full_cycle_no_reset() {
        let mut bridge = MetaCognitionBridge::new(".");
        let r1 = bridge.run_full_cycle();
        assert_eq!(r1.iteration, 1);
        let r2 = bridge.run_full_cycle();
        assert_eq!(r2.iteration, 2);
        assert!(bridge.delta_history.len() >= 1);
    }

    #[test]
    fn test_weakest_modules() {
        let mut bridge = MetaCognitionBridge::new(".");
        bridge.run_full_cycle();
        let weakest = bridge.weakest_modules(3);
        assert!(weakest.len() <= 3);
    }

    #[test]
    fn test_module_weakness_flags() {
        let mut bridge = MetaCognitionBridge::new(".");
        bridge.run_full_cycle();
        let flags = bridge.module_weakness_flags(5);
        assert!(flags.len() <= 5);
    }

    #[test]
    fn test_module_roadmaps() {
        let mut bridge = MetaCognitionBridge::new(".");
        bridge.run_full_cycle();
        let roadmaps = bridge.module_roadmaps();
        for (_module, steps) in &roadmaps {
            for step in steps {
                assert!(step.starts_with('['));
            }
        }
    }

    #[test]
    fn test_enriched_status_summary() {
        let mut bridge = MetaCognitionBridge::new(".");
        let summary = bridge.enriched_status_summary();
        assert!(summary.contains("MetaCognition Cycle"));
        bridge.run_full_cycle();
        let enriched = bridge.enriched_status_summary();
        assert!(enriched.contains("no prior scan") || enriched.contains("delta:"));
    }

    #[test]
    fn test_quick_scan_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let bridge = MetaCognitionBridge::new(dir.path().to_str().unwrap());
        let report = bridge.quick_scan();
        assert_eq!(report.weaknesses.len(), 0);
    }

    #[test]
    fn test_module_delta_creation() {
        let delta = ModuleDelta {
            module: "test_mod".to_string(),
            new_weaknesses: 3,
            resolved_weaknesses: 1,
            line_change: 50,
            compilation_errors: 0,
            compilation_warnings: 2,
        };
        assert_eq!(delta.module, "test_mod");
        assert_eq!(delta.new_weaknesses, 3);
        assert_eq!(delta.resolved_weaknesses, 1);
        assert_eq!(delta.line_change, 50);
        assert_eq!(delta.compilation_errors, 0);
        assert_eq!(delta.compilation_warnings, 2);
    }

    #[test]
    fn test_module_delta_zero_values() {
        let delta = ModuleDelta {
            module: "empty_mod".to_string(),
            new_weaknesses: 0,
            resolved_weaknesses: 0,
            line_change: 0,
            compilation_errors: 0,
            compilation_warnings: 0,
        };
        assert_eq!(delta.module, "empty_mod");
        assert_eq!(delta.new_weaknesses, 0);
        assert_eq!(delta.line_change, 0);
    }

    #[test]
    fn test_module_weakness_flags_before_cycle() {
        let bridge = MetaCognitionBridge::new(".");
        let flags = bridge.module_weakness_flags(5);
        assert!(flags.is_empty());
    }

    #[test]
    fn test_distill_single_memory_filtered() {
        let mem = ReasoningMemory::new(
            "single test",
            TaskType::CodeGeneration,
            &[MicroEdit::AdjustDimension(
                "coding_ability".to_string(),
                0.1,
            )],
            0.8,
        );
        let principles = ExperienceDistiller::distill(&[mem]);
        assert!(principles.is_empty());
    }

    #[test]
    fn test_distill_with_zero_reward() {
        let mems: Vec<ReasoningMemory> = (0..3)
            .map(|i| {
                ReasoningMemory::new(
                    &format!("mem {}", i),
                    TaskType::CodeGeneration,
                    &[MicroEdit::AdjustDimension(
                        "coding_ability".to_string(),
                        0.1,
                    )],
                    0.0,
                )
            })
            .collect();
        let principles = ExperienceDistiller::distill(&mems);
        assert!(principles.is_empty());
    }

    #[test]
    fn test_contrastive_reflect_no_failures() {
        let mem = ReasoningMemory::new("great success", TaskType::CodeGeneration, &[], 0.95);
        let anti = ExperienceDistiller::contrastive_reflect(&[mem]);
        assert!(anti.is_empty());
    }

    #[test]
    fn test_apply_principles_empty_no_effect() {
        let mut cv = CapabilityVector::default();
        let original = cv.clone();
        apply_principles(&mut cv, &[], 0.5);
        assert_eq!(cv.arr(), original.arr());
    }

    #[test]
    fn test_avoid_anti_patterns_empty_no_effect() {
        let mut cv = CapabilityVector::default();
        let original = cv.clone();
        avoid_anti_patterns(&mut cv, &[]);
        assert_eq!(cv.arr(), original.arr());
    }

    #[test]
    fn test_weakness_count_initial() {
        let bridge = MetaCognitionBridge::new("/tmp");
        assert_eq!(bridge.weakness_count(), 0);
        assert_eq!(bridge.critical_count(), 0);
        assert_eq!(bridge.pending_evolutions(), 0);
    }

    #[test]
    fn test_bridge_status_summary_format() {
        let bridge = MetaCognitionBridge::new(".");
        let summary = bridge.status_summary();
        assert!(summary.contains("MetaCognition Cycle"));
    }

    #[test]
    fn test_contrastive_reflect_no_successes_no_failures() {
        let mems = vec![
            ReasoningMemory::new("neutral", TaskType::CodeGeneration, &[], 0.5),
            ReasoningMemory::new("neutral2", TaskType::CodeGeneration, &[], 0.5),
        ];
        let anti = ExperienceDistiller::contrastive_reflect(&mems);
        assert!(anti.is_empty());
    }

    #[test]
    fn test_bridge_weakest_modules_initial() {
        let bridge = MetaCognitionBridge::new(".");
        let weakest = bridge.weakest_modules(5);
        assert!(weakest.is_empty());
    }
}
