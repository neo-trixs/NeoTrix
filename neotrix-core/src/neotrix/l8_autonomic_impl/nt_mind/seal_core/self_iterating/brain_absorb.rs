use chrono::Utc;

use crate::neotrix::nt_world_model::TaskType;
use super::brain_core::ReasoningBrain;
use super::super::core::{CapabilityVector, KnowledgeSource, AbsorptionRecord, PerformanceEvaluator};
use super::super::self_edit::{SelfEdit, MicroEdit, infer_task_type};
use super::super::stats::BrainStats;

impl ReasoningBrain {
    pub fn absorb(&mut self, source: KnowledgeSource) {
        let source_vector = source.capability_vector();
        let mut lr = self.learning_rate;

        let cur = self.capability.arr().to_vec();
        let mut raw_deltas = vec![0.0; cur.len()];
        for i in 0..cur.len() {
            let src = source_vector.arr.get(i).copied().unwrap_or(0.0);
            raw_deltas[i] = lr * (src - cur[i]);
        }
        if let Some(ref fisher) = self.fisher {
            let mut proposed = cur.clone();
            for (i, val) in proposed.iter_mut().enumerate() {
                *val += raw_deltas[i];
            }
            let penalty = fisher.ewc_penalty(&cur, &proposed);
            if penalty > self.ewc_lambda {
                let scale = self.ewc_lambda / penalty;
                lr *= scale;
                for d in &mut raw_deltas {
                    *d *= scale;
                }
            }
        }

        self.capability.update_from_other(&source_vector, lr);
        self.capability.normalize();

        if let Some(ref mut fisher) = self.fisher {
            fisher.update_raw(&raw_deltas);
        }

        self.source_access_tracker.record_access(&source);

        self.absorption_history.push(AbsorptionRecord {
            source,
            timestamp: Utc::now().timestamp() as u64,
            weight: lr,
        });

        self.total_absorb_count += 1;
    }

    /// P0-2 反思式吸收三步循环 (吸收 VideoLingo Translate-Reflect-Adaptation):
    /// 1. **Translate** — 吸收源知识 (现有 absorb 逻辑, 源向量 → 能力向量)
    /// 2. **Reflect** — 反思吸收质量: 计算吸收前后能力向量位移 (L2 距离),
    ///    位移过大 → 可能过拟合/灾难性遗忘 → 标记需适应
    /// 3. **Adaptation** — 适应: 若位移超阈值, 按比例回退 (缩放学习率),
    ///    防止单次吸收过度改写既有能力 (EWC 之外的轻量防线)
    ///
    /// VideoLingo 原文: "Translate → Reflect → Adaptation" 三步循环,
    /// 反思结果驱动适应, 而非一次性单向吸收。
    /// 返回 (是否触发适应, 位移量)。
    pub fn absorb_with_reflection(&mut self, source: KnowledgeSource) -> (bool, f64) {
        let before = self.capability.arr().to_vec();
        self.absorb(source);
        let after = self.capability.arr().to_vec();

        // Reflect: 能力向量位移 (L2 距离)
        let displacement: f64 = before
            .iter()
            .zip(after.iter())
            .map(|(a, b)| (b - a) * (b - a))
            .sum::<f64>()
            .sqrt();

        // Adaptation: 位移超阈值 → 回退 (缩放学习率, 下次吸收更保守)
        // 阈值 0.10: lr=0.05 × 高维强源位移 ≈0.15 (触发), 弱源 ≈0.05 (不触发)。
        const DISPLACEMENT_THRESHOLD: f64 = 0.10;
        if displacement > DISPLACEMENT_THRESHOLD {
            let scale = DISPLACEMENT_THRESHOLD / displacement;
            self.learning_rate *= scale;
            // 回退本次吸收: 从位移反推, 按比例还原
            let cur = self.capability.arr_mut();
            for (i, b) in before.iter().enumerate() {
                cur[i] = b + (cur[i] - b) * scale;
            }
            self.capability.normalize();
            (true, displacement)
        } else {
            (false, displacement)
        }
    }

    pub fn generate_self_edit(&self, task: &str) -> Vec<MicroEdit> {
        let task_type = infer_task_type(task);
        let mut micro_edits: Vec<MicroEdit> = self.strategy.generate_edit(self, task)
            .into_iter()
            .map(|d| MicroEdit::AdjustDimension(d.dimension, d.delta))
            .collect();

        let extension_dims = Self::generate_extension_dims(&task_type);
        if !extension_dims.is_empty() {
            micro_edits.push(MicroEdit::AddExtension(extension_dims));
        }

        micro_edits.push(MicroEdit::UpdateLearningRate(self.learning_rate));
        micro_edits.push(MicroEdit::NormalizeVector);

        micro_edits
    }

    fn generate_extension_dims(task_type: &TaskType) -> Vec<(String, f64)> {
        match task_type {
            TaskType::Security => {
                vec![
                    ("penetration_testing".to_string(), 0.6),
                    ("vulnerability_analysis".to_string(), 0.7),
                ]
            }
            TaskType::Planning => {
                vec![
                    ("resource_allocation".to_string(), 0.6),
                    ("timeline_estimation".to_string(), 0.7),
                ]
            }
            TaskType::CodeReview => {
                vec![
                    ("code_smell_detection".to_string(), 0.6),
                    ("nt_shield_audit".to_string(), 0.5),
                ]
            }
            TaskType::CodeGeneration => {
                vec![
                    ("code_quality".to_string(), 0.5),
                ]
            }
            _ => Vec::new(),
        }
    }

    pub fn apply_micro_edits(&mut self, edits: &[MicroEdit]) -> Vec<usize> {
        let mut applied_indices = Vec::new();

        for (i, edit) in edits.iter().enumerate() {
            match edit {
                MicroEdit::AdjustDimension(dim, amount) => {
                    if let Some(idx) = CapabilityVector::index_from_name(dim) {
                        let new_val = (self.capability.arr()[idx] + amount).min(1.0);
                        self.capability.arr_mut()[idx] = new_val;
                        applied_indices.push(i);
                    }
                }
                MicroEdit::UpdateLearningRate(rate) => {
                    self.learning_rate = *rate;
                    applied_indices.push(i);
                }
                MicroEdit::NormalizeVector => {
                    self.capability.normalize();
                    applied_indices.push(i);
                }
                MicroEdit::AddExtension(new_dims) => {
                    self.capability.extend_named(new_dims);
                    self.capability.merge_similar(0.85);
                    self.capability.prune_extension();
                    applied_indices.push(i);
                }
                MicroEdit::SetProvenance(source) => {
                    self.capability.set_provenance(source.clone());
                    applied_indices.push(i);
                }
                MicroEdit::BatchAdjust(pairs) => {
                    for (dim, amount) in pairs {
                        if let Some(idx) = CapabilityVector::index_from_name(dim) {
                            let new_val = (self.capability.arr()[idx] + amount).min(1.0);
                            self.capability.arr_mut()[idx] = new_val;
                        }
                    }
                    applied_indices.push(i);
                }
                MicroEdit::AddedDimension(name, value) => {
                    if CapabilityVector::index_from_name(name).is_none() {
                        let exists = self.capability.extension.iter().any(|(n, _)| n == name);
                        if !exists {
                            self.capability.extension.push((name.clone(), *value));
                            self.capability.merge_similar(0.85);
                        }
                    }
                    applied_indices.push(i);
                }
                MicroEdit::ModifiedDimension(name, old_val, new_val) => {
                    if let Some(idx) = CapabilityVector::index_from_name(name) {
                        let current = self.capability.arr()[idx];
                        if (current - old_val).abs() < 0.001 {
                            self.capability.arr_mut()[idx] = *new_val;
                        }
                    } else if let Some(pos) = self.capability.extension.iter().position(|(n, _)| n == name) {
                        let current = self.capability.extension[pos].1;
                        if (current - old_val).abs() < 0.001 {
                            self.capability.extension[pos].1 = *new_val;
                        }
                    }
                    applied_indices.push(i);
                }
                MicroEdit::RemovedDimension(name) => {
                    if CapabilityVector::index_from_name(name).is_some() {
                        if let Some(idx) = CapabilityVector::index_from_name(name) {
                            self.capability.arr_mut()[idx] = 0.0;
                        }
                    } else {
                        self.capability.extension.retain(|(n, _)| n != name);
                    }
                    applied_indices.push(i);
                }
            }
        }

        applied_indices
    }

    pub fn apply_self_edit(&mut self, edit: &SelfEdit, reward: Option<f64>) -> bool {
        let snapshot = self.capability.clone();
        let snapshot_lr = self.learning_rate;

        let mut micro_edits = Vec::new();

        for dim in &edit.target_dimensions {
            micro_edits.push(MicroEdit::AdjustDimension(dim.clone(), edit.adjustment_magnitude));
        }

        micro_edits.push(MicroEdit::NormalizeVector);

        self.apply_micro_edits(&micro_edits);

        if let Some(r) = reward {
            if r < 0.0 {
                self.capability = snapshot;
                self.learning_rate = snapshot_lr;
                return false;
            }
        }

        self.absorption_history.push(AbsorptionRecord {
            source: KnowledgeSource::DesignPhilosophy,
            timestamp: Utc::now().timestamp() as u64,
            weight: edit.adjustment_magnitude,
        });

        true
    }

    pub fn absorb_batch(&mut self, sources: &[KnowledgeSource]) {
        for &source in sources {
            self.absorb(source);
        }
    }

    pub fn cold_sources(&self, min_accesses: usize) -> Vec<KnowledgeSource> {
        self.source_access_tracker.prune_cold(min_accesses)
    }

    pub fn source_access_count(&self, source: &KnowledgeSource) -> usize {
        self.source_access_tracker.access_count(source)
    }

    pub fn is_source_hot(&self, source: &KnowledgeSource) -> bool {
        self.source_access_tracker.is_hot(source)
    }

    pub fn evaluate_capability(&self, task_type: TaskType) -> f64 {
        let mut score = PerformanceEvaluator::evaluate(&task_type, &self.capability);

        if let Some(&affinity) = self.task_affinity.get(&task_type) {
            score = (score + affinity * 0.3).min(1.0);
        }

        score
    }

    pub fn update_task_affinity(&mut self, task_type: TaskType, performance: f64) {
        let entry = self.task_affinity.entry(task_type).or_insert(0.5);
        *entry = *entry * 0.7 + performance * 0.3;
    }

    pub fn get_statistics(&self) -> BrainStats {
        BrainStats {
            total_absorbed: self.total_absorb_count,
            unique_sources: self.absorption_history.iter()
                .map(|r| r.source).collect(),
            latest_absorption: self.absorption_history.last().map(|r| r.timestamp),
            capability_sum: self.capability.typography() + self.capability.grid() + self.capability.color()
                + self.capability.accessibility() + self.capability.compound_composition(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_knowledge::KnowledgeSource;

    /// P0-2 三步循环: 极端源向量应触发 Adaptation (位移超阈值 → 回退 + 学习率缩放)。
    #[test]
    fn test_absorb_with_reflection_adapts_on_extreme_source() {
        let mut brain = ReasoningBrain::new();
        // 基线能力全 0 (默认), 吸收强源 ResearchFindings (0.85-0.9 高维)
        let lr_before = brain.learning_rate;
        let cap_before = brain.capability.arr().to_vec();

        // 极端源: 高维强向量 → 位移必然超阈值 → 触发适应
        let (adapted, displacement) = brain.absorb_with_reflection(KnowledgeSource::ResearchFindings);

        assert!(adapted, "extreme source should trigger adaptation");
        assert!(displacement > 0.10, "displacement should exceed threshold, got {}", displacement);
        assert!(
            brain.learning_rate < lr_before,
            "adaptation should scale down learning rate"
        );
        // 回退后能力不应偏离基线过远
        let drift: f64 = cap_before
            .iter()
            .zip(brain.capability.arr().iter())
            .map(|(a, b)| (b - a).abs())
            .sum();
        assert!(drift < 1.0, "post-adaptation drift should be bounded, got {}", drift);
    }

    /// P0-2 三步循环: 温和源不应触发适应 (位移在阈值内)。
    #[test]
    fn test_absorb_with_reflection_no_adapt_on_gentle_source() {
        let mut brain = ReasoningBrain::new();
        let lr_before = brain.learning_rate;
        // AdamsLaw: 基础维度多为 0, 位移 ≈ 0.05 (阈值内) → 不触发适应
        let (adapted, _) = brain.absorb_with_reflection(KnowledgeSource::AdamsLaw);
        assert!(!adapted, "gentle source should not trigger adaptation");
        assert_eq!(brain.learning_rate, lr_before, "learning rate unchanged");
    }
}
