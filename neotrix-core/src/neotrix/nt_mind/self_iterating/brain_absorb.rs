use chrono::Utc;

use super::super::core::{
    AbsorptionRecord, CapabilityVector, KnowledgeSource, PerformanceEvaluator,
};
use super::super::self_edit::{infer_task_type, MicroEdit, SelfEdit};
use super::super::stats::BrainStats;
use super::brain_core::ReasoningBrain;
use crate::neotrix::nt_expert_routing::TaskType;

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

    pub fn generate_self_edit(&self, task: &str) -> Vec<MicroEdit> {
        let task_type = infer_task_type(task);
        let mut micro_edits: Vec<MicroEdit> = self
            .strategy
            .generate_edit(self, task)
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
                vec![("code_quality".to_string(), 0.5)]
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
                    } else if let Some(pos) = self
                        .capability
                        .extension
                        .iter()
                        .position(|(n, _)| n == name)
                    {
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
                MicroEdit::GenerateNtModule(_, _) => {
                    // nt-lang bridge is gated — handled by DGM-H orchestrator
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
            micro_edits.push(MicroEdit::AdjustDimension(
                dim.clone(),
                edit.adjustment_magnitude,
            ));
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
            unique_sources: self.absorption_history.iter().map(|r| r.source).collect(),
            latest_absorption: self.absorption_history.last().map(|r| r.timestamp),
            capability_sum: self.capability.typography()
                + self.capability.grid()
                + self.capability.color()
                + self.capability.accessibility()
                + self.capability.compound_composition(),
        }
    }
}
