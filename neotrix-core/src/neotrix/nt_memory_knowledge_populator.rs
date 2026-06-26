use crate::core::nt_core_bank::{ReasoningBank, ReasoningMemory};
use crate::core::nt_core_edit::MicroEdit;
use crate::core::{CapabilityVector, KnowledgeSource, TaskType};
use std::collections::HashSet;

const LEARNING_RATE: f64 = 0.15;

pub struct PopulateReport {
    pub sources_processed: usize,
    pub dimensions_updated: usize,
    pub extension_keys_added: HashSet<String>,
    pub non_zero_before: usize,
    pub non_zero_after: usize,
}

pub struct KnowledgePopulator;

impl KnowledgePopulator {
    pub fn populate_brain(brain: &mut CapabilityVector) -> PopulateReport {
        let non_zero_before = brain.arr.iter().filter(|&&v| v.abs() > 1e-6).count();
        let mut updated_dims = HashSet::new();
        let mut ext_keys = HashSet::new();

        let sources = KnowledgeSource::all();
        for source in &sources {
            let sv = source.capability_vector();
            let weight = source.source_weight();
            for (i, &val) in sv.arr.iter().enumerate() {
                if val.abs() > 1e-6 && i < brain.arr.len() {
                    let delta = val * weight * LEARNING_RATE;
                    if delta.abs() > 1e-6 {
                        brain.arr[i] = (brain.arr[i] + delta).min(1.0);
                        updated_dims.insert(i);
                    }
                }
            }
            for (name, val) in &sv.extension {
                if val.abs() > 1e-6 {
                    let delta = val * weight * LEARNING_RATE;
                    if delta.abs() > 1e-6 {
                        brain.add_extension_dim(
                            name,
                            (brain
                                .extension
                                .iter()
                                .find(|(n, _)| n == name)
                                .map(|(_, v)| v + delta)
                                .unwrap_or(delta))
                            .min(1.0),
                        );
                        ext_keys.insert(name.clone());
                    }
                }
            }
        }

        let non_zero_after = brain.arr.iter().filter(|&&v| v.abs() > 1e-6).count();
        PopulateReport {
            sources_processed: sources.len(),
            dimensions_updated: updated_dims.len(),
            extension_keys_added: ext_keys,
            non_zero_before,
            non_zero_after,
        }
    }

    pub fn populate_reasoning_bank(bank: &mut ReasoningBank, seed_per_source: usize) -> usize {
        let mut total = 0;
        let sources = KnowledgeSource::all();
        for source in &sources {
            let name = source.name();
            let mut micro_edits = Vec::new();
            for (ext_name, ext_val) in &source.capability_vector().extension {
                micro_edits.push(MicroEdit::AddedDimension(ext_name.clone(), *ext_val));
            }
            if micro_edits.is_empty() {
                micro_edits.push(MicroEdit::AdjustDimension("domain_specificity".into(), 0.5));
            }
            for _ in 0..seed_per_source {
                let mem = ReasoningMemory::new(
                    &format!("Knowledge seed: {}", name),
                    TaskType::Learning,
                    &micro_edits,
                    0.8,
                );
                bank.store(mem);
                total += 1;
            }
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_populate_all_sources() {
        let mut brain = CapabilityVector::default();
        let report = KnowledgePopulator::populate_brain(&mut brain);
        assert_eq!(report.sources_processed, 63);
        assert!(report.dimensions_updated > 0);
    }

    #[test]
    fn test_populate_increases_dimensions() {
        let mut brain = CapabilityVector::default();
        let before = brain.arr.iter().filter(|&&v| v.abs() > 1e-6).count();
        let report = KnowledgePopulator::populate_brain(&mut brain);
        assert!(report.non_zero_after > before, "dimensions should increase");
        assert!(report.non_zero_after > 0);
    }

    #[test]
    fn test_extension_keys_added() {
        let mut brain = CapabilityVector::default();
        let report = KnowledgePopulator::populate_brain(&mut brain);
        assert!(!report.extension_keys_added.is_empty());
        assert!(
            report.extension_keys_added.contains("long_term_memory")
                || report.extension_keys_added.contains("secret_detection")
        );
    }

    #[test]
    fn test_inject_seeds() {
        let mut bank = ReasoningBank::new(200);
        let total = KnowledgePopulator::populate_reasoning_bank(&mut bank, 2);
        assert_eq!(total, 126);
        let stats = bank.stats();
        assert_eq!(stats.total_memories, 126);
    }

    #[test]
    fn test_populate_non_zero_before_tracking() {
        let mut brain = CapabilityVector::default();
        brain.arr[0] = 0.5;
        let report = KnowledgePopulator::populate_brain(&mut brain);
        assert_eq!(report.non_zero_before, 1);
        assert!(report.non_zero_after > 1);
    }
}
