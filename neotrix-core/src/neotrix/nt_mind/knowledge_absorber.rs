use neotrix_types::CapabilityVector;
use neotrix_types::edit::MicroEdit;
use neotrix_types::knowledge::{TaskType, RewardSource};
use neotrix_types::memory::{ReasoningMemory, T3Views, MemoryTier, MemoryLifecycle};
use crate::ReasoningBank;

use super::self_iterating::ReasoningBrain;

/// Shared absorption logic: register source + apply edits + store memory.
/// Eliminates duplicate code between WebKnowledgeMiner, KnowledgeMapper,
/// KnowledgeMiner, and SelfEvolver.
pub struct KnowledgeAbsorber;

impl KnowledgeAbsorber {
    /// Absorb knowledge from dimension edits into brain + bank.
    ///
    /// `edits`: `(dimension_name, delta)` pairs (e.g. `("inference_depth", 0.12)`)
    /// `source_name`: unique identifier for the knowledge source
    /// `task_type`: used for the stored ReasoningMemory
    /// `confidence`: reward / confidence score
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

        // Register knowledge source
        let mut vector = CapabilityVector::default();
        for (dim_name, delta) in edits {
            if let Some(idx) = CapabilityVector::index_from_name(dim_name) {
                vector.arr_mut()[idx] = (*delta).min(1.0);
            }
        }
        brain.register_knowledge_source(source_name, vector);

        // Apply edits to capability vector
        for (dim_name, delta) in edits {
            if let Some(idx) = CapabilityVector::index_from_name(dim_name) {
                let current = brain.capability.arr_mut()[idx];
                brain.capability.arr_mut()[idx] = (current + delta).clamp(0.0, 1.0);
            }
        }

        brain.capability.normalize();

        // Store memory
        let micro_edits: Vec<MicroEdit> = edits.iter()
            .map(|(d, v)| MicroEdit::AdjustDimension(d.clone(), *v))
            .collect();

        let reward = (edits.len() as f64 * 0.02).min(1.0);
        let mem = ReasoningMemory {
            id: format!("absorb-{}-{}", source_name, std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()),
            task_description: description.to_string(),
            task_type,
            micro_edits,
            reward,
            reward_source: RewardSource::Internal,
            success: confidence > 0.5,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            embedding: None,
            tier: MemoryTier::Working,
            lifecycle: MemoryLifecycle::new(confidence),
            t3_views: T3Views::new(),
            confidence: 1.0,
            source: crate::core::nt_core_bank::MemorySource::User,
            last_used_at: 0,
            conflict_group: String::new(),
            verification_time: 0,
        };

        bank.store(mem);
    }
}
