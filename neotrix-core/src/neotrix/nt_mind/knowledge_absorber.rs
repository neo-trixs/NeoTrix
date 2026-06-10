use crate::core::nt_core_cap::CapabilityVector;
use crate::core::nt_core_edit::MicroEdit;
use crate::core::TaskType;
use crate::core::nt_core_bank::{ReasoningMemory, ReasoningBank};
use super::self_iterating::ReasoningBrain;

pub struct KnowledgeAbsorber;

impl KnowledgeAbsorber {
    pub fn absorb(
        brain: &mut ReasoningBrain,
        bank: &mut ReasoningBank,
        edits: &[(String, f64)],
        source_name: &str,
        task_type: TaskType,
        confidence: f64,
        description: &str,
    ) {
        if confidence < 0.3 {
            return;
        }

        let mut vector = CapabilityVector::default();
        for (dim_name, delta) in edits {
            if let Some(idx) = CapabilityVector::index_from_name(dim_name) {
                vector.arr_mut()[idx] = (*delta).min(1.0);
            }
        }
        brain.register_knowledge_source(source_name, vector);

        for (dim_name, delta) in edits {
            if let Some(idx) = CapabilityVector::index_from_name(dim_name) {
                let current = brain.capability.arr_mut()[idx];
                brain.capability.arr_mut()[idx] = (current + delta).clamp(0.0, 1.0);
            }
        }

        brain.capability.normalize();

        let micro_edits: Vec<MicroEdit> = edits.iter()
            .map(|(d, v)| MicroEdit::AdjustDimension(d.clone(), *v))
            .collect();

        let reward = (edits.len() as f64 * 0.02).min(1.0);
        let mem = ReasoningMemory::new(description, task_type, &micro_edits, reward);
        bank.store(mem);
    }
}
