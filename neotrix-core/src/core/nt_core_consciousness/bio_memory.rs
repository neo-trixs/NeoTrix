// REVIVED Task 1 — dead_code removed 2026-06-24

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryMechanism {
    WorkingMemory(usize),
    Episodic(usize),
    Semantic(usize),
    Procedural,
    Priming,
    ClassicalConditioning,
    OperantConditioning,
    Habituation,
    Sensitization,
    LongTermPotentiation,
    LongTermDepression,
    SpikeTimingPlasticity,
    HippocampalReplay,
    PatternSeparation,
    PatternCompletion,
    HebbianCellAssembly,
    PredictiveCoding(usize),
    SuccessorRepresentation(usize),
    TemporalDifference(usize),
    EligibilityTrace,
    WorkingMemoryConsolidation,
    SleepConsolidation,
    RecallReconsolidation,
    Extinction,
    ContextReminding,
    SourceMemory,
}

pub fn mechanism_name(m: &MemoryMechanism) -> String {
    match m {
        MemoryMechanism::WorkingMemory(_) => "WorkingMemory".to_string(),
        MemoryMechanism::Episodic(_) => "Episodic".to_string(),
        MemoryMechanism::Semantic(_) => "Semantic".to_string(),
        MemoryMechanism::Procedural => "Procedural".to_string(),
        MemoryMechanism::Priming => "Priming".to_string(),
        MemoryMechanism::ClassicalConditioning => "ClassicalConditioning".to_string(),
        MemoryMechanism::OperantConditioning => "OperantConditioning".to_string(),
        MemoryMechanism::Habituation => "Habituation".to_string(),
        MemoryMechanism::Sensitization => "Sensitization".to_string(),
        MemoryMechanism::LongTermPotentiation => "LongTermPotentiation".to_string(),
        MemoryMechanism::LongTermDepression => "LongTermDepression".to_string(),
        MemoryMechanism::SpikeTimingPlasticity => "SpikeTimingPlasticity".to_string(),
        MemoryMechanism::HippocampalReplay => "HippocampalReplay".to_string(),
        MemoryMechanism::PatternSeparation => "PatternSeparation".to_string(),
        MemoryMechanism::PatternCompletion => "PatternCompletion".to_string(),
        MemoryMechanism::HebbianCellAssembly => "HebbianCellAssembly".to_string(),
        MemoryMechanism::PredictiveCoding(_) => "PredictiveCoding".to_string(),
        MemoryMechanism::SuccessorRepresentation(_) => "SuccessorRepresentation".to_string(),
        MemoryMechanism::TemporalDifference(_) => "TemporalDifference".to_string(),
        MemoryMechanism::EligibilityTrace => "EligibilityTrace".to_string(),
        MemoryMechanism::WorkingMemoryConsolidation => "WorkingMemoryConsolidation".to_string(),
        MemoryMechanism::SleepConsolidation => "SleepConsolidation".to_string(),
        MemoryMechanism::RecallReconsolidation => "RecallReconsolidation".to_string(),
        MemoryMechanism::Extinction => "Extinction".to_string(),
        MemoryMechanism::ContextReminding => "ContextReminding".to_string(),
        MemoryMechanism::SourceMemory => "SourceMemory".to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct BioMemorySlot {
    pub id: u64,
    pub mechanism: MemoryMechanism,
    pub key: Vec<f64>,
    pub value: Vec<f64>,
    pub strength: f64,
    pub created: u64,
    pub last_accessed: u64,
    pub access_count: u64,
}

#[derive(Debug, Clone)]
pub struct BioMemorySystem {
    pub slots: Vec<BioMemorySlot>,
    pub max_slots: usize,
    pub next_id: u64,
    pub tick: u64,
}

pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    (dot / (norm_a * norm_b)).clamp(0.0, 1.0)
}

impl BioMemorySystem {
    pub fn new(max_slots: usize) -> Self {
        Self {
            slots: Vec::with_capacity(max_slots),
            max_slots,
            next_id: 1,
            tick: 0,
        }
    }

    pub fn store(&mut self, mechanism: MemoryMechanism, key: Vec<f64>, value: Vec<f64>) -> u64 {
        self.tick += 1;
        let id = self.next_id;
        self.next_id += 1;

        if self.slots.len() >= self.max_slots {
            let oldest_idx = self
                .slots
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.last_accessed.cmp(&b.last_accessed))
                .map(|(i, _)| i);
            if let Some(idx) = oldest_idx {
                self.slots.swap_remove(idx);
            }
        }

        self.slots.push(BioMemorySlot {
            id,
            mechanism,
            key,
            value,
            strength: 1.0,
            created: self.tick,
            last_accessed: self.tick,
            access_count: 0,
        });
        id
    }

    pub fn recall(&self, key: &[f64], mechanism: Option<&MemoryMechanism>) -> Vec<&BioMemorySlot> {
        let mut results: Vec<&BioMemorySlot> = self
            .slots
            .iter()
            .filter(|s| {
                if let Some(m) = mechanism {
                    std::mem::discriminant(&s.mechanism) == std::mem::discriminant(m)
                } else {
                    true
                }
            })
            .filter(|s| cosine_similarity(key, &s.key) > 0.7)
            .collect();
        results.sort_by(|a, b| {
            b.strength
                .partial_cmp(&a.strength)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    pub fn recall_latest(
        &self,
        n: usize,
        mechanism: Option<&MemoryMechanism>,
    ) -> Vec<&BioMemorySlot> {
        let mut results: Vec<&BioMemorySlot> = self
            .slots
            .iter()
            .filter(|s| {
                if let Some(m) = mechanism {
                    std::mem::discriminant(&s.mechanism) == std::mem::discriminant(m)
                } else {
                    true
                }
            })
            .collect();
        results.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        results.truncate(n);
        results
    }

    pub fn reinforce(&mut self, id: u64) {
        self.tick += 1;
        if let Some(slot) = self.slots.iter_mut().find(|s| s.id == id) {
            slot.strength = (slot.strength + 0.1).min(1.0);
            slot.last_accessed = self.tick;
            slot.access_count += 1;
        }
    }

    pub fn decay(&mut self, rate: f64) {
        self.tick += 1;
        for slot in self.slots.iter_mut() {
            slot.strength = (slot.strength - rate).max(0.0);
            if slot.strength <= 0.0 {
                slot.last_accessed = self.tick;
            }
        }
        self.slots.retain(|s| s.strength > 0.0);
    }

    pub fn consolidate(
        &mut self,
        source_mechanism: &MemoryMechanism,
        target_mechanism: &MemoryMechanism,
    ) {
        let source_disc = std::mem::discriminant(source_mechanism);
        let source_slots: Vec<BioMemorySlot> = self
            .slots
            .iter()
            .filter(|s| std::mem::discriminant(&s.mechanism) == source_disc)
            .cloned()
            .collect();

        for mut slot in source_slots {
            slot.mechanism = target_mechanism.clone();
            slot.id = self.next_id;
            self.next_id += 1;
            slot.strength = (slot.strength * 0.8).max(0.1);
            self.slots.push(slot);
        }
    }

    pub fn pattern_complete(&self, partial: &[f64]) -> Option<Vec<f64>> {
        let mut best: Option<&BioMemorySlot> = None;
        let mut best_sim = 0.7_f64;

        for slot in &self.slots {
            let sim = cosine_similarity(partial, &slot.key);
            if sim > best_sim {
                best_sim = sim;
                best = Some(slot);
            }
        }
        best.map(|s| {
            let scale = best_sim * s.strength;
            s.value.iter().map(|v| v * scale).collect()
        })
    }

    pub fn count_by_mechanism(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for slot in &self.slots {
            let name = mechanism_name(&slot.mechanism);
            *counts.entry(name).or_insert(0) += 1;
        }
        counts
    }

    pub fn average_strength(&self) -> f64 {
        if self.slots.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.slots.iter().map(|s| s.strength).sum();
        sum / self.slots.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(key: f64, dim: usize) -> Vec<f64> {
        let mut v = vec![0.0_f64; dim];
        let idx = (key.abs() as usize) % dim;
        if idx < dim {
            v[idx] = 1.0;
        }
        v
    }

    #[test]
    fn test_store_and_recall() {
        let mut sys = BioMemorySystem::new(100);
        let id = sys.store(MemoryMechanism::Episodic(3), v(1.0, 64), v(42.0, 64));
        assert_eq!(id, 1);
        let results = sys.recall(&v(1.0, 64), None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value[1], 42.0);
    }

    #[test]
    fn test_recall_by_mechanism() {
        let mut sys = BioMemorySystem::new(100);
        sys.store(MemoryMechanism::Episodic(3), v(2.0, 64), v(10.0, 64));
        sys.store(MemoryMechanism::Semantic(5), v(2.0, 64), v(20.0, 64));
        let episodic_results = sys.recall(&v(2.0, 64), Some(&MemoryMechanism::Episodic(0)));
        assert_eq!(episodic_results.len(), 1);
        assert_eq!(episodic_results[0].value[10], 10.0);
    }

    #[test]
    fn test_reinforce_increases_strength() {
        let mut sys = BioMemorySystem::new(100);
        let id = sys.store(MemoryMechanism::Priming, v(3.0, 64), v(5.0, 64));
        let initial = sys.slots[0].strength;
        sys.reinforce(id);
        assert!(sys.slots[0].strength > initial);
        assert_eq!(sys.slots[0].access_count, 1);
    }

    #[test]
    fn test_decay_reduces_strength() {
        let mut sys = BioMemorySystem::new(100);
        sys.store(MemoryMechanism::Habituation, v(4.0, 64), v(7.0, 64));
        assert!((sys.slots[0].strength - 1.0).abs() < 1e-6);
        sys.decay(0.3);
        assert!((sys.slots[0].strength - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_consolidate_copies_slots() {
        let mut sys = BioMemorySystem::new(100);
        sys.store(
            MemoryMechanism::WorkingMemoryConsolidation,
            v(5.0, 64),
            v(8.0, 64),
        );
        let count_before = sys.slots.len();
        sys.consolidate(
            &MemoryMechanism::WorkingMemoryConsolidation,
            &MemoryMechanism::SleepConsolidation,
        );
        assert!(sys.slots.len() > count_before);
        let sleep_count = sys
            .slots
            .iter()
            .filter(|s| {
                std::mem::discriminant(&s.mechanism)
                    == std::mem::discriminant(&MemoryMechanism::SleepConsolidation)
            })
            .count();
        assert_eq!(sleep_count, 1);
    }

    #[test]
    fn test_pattern_complete() {
        let mut sys = BioMemorySystem::new(100);
        sys.store(MemoryMechanism::PatternCompletion, v(10.0, 64), v(99.0, 64));
        let result = sys.pattern_complete(&v(10.0, 64));
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r[1] > 0.0);
    }

    #[test]
    fn test_count_by_mechanism() {
        let mut sys = BioMemorySystem::new(100);
        sys.store(MemoryMechanism::Procedural, v(6.0, 64), v(1.0, 64));
        sys.store(MemoryMechanism::Procedural, v(7.0, 64), v(2.0, 64));
        sys.store(MemoryMechanism::Priming, v(8.0, 64), v(3.0, 64));
        let counts = sys.count_by_mechanism();
        assert_eq!(*counts.get("Procedural").unwrap(), 2);
        assert_eq!(*counts.get("Priming").unwrap(), 1);
    }

    #[test]
    fn test_average_strength() {
        let mut sys = BioMemorySystem::new(100);
        assert!((sys.average_strength() - 0.0).abs() < 1e-6);
        sys.store(MemoryMechanism::Sensitization, v(9.0, 64), v(4.0, 64));
        sys.store(MemoryMechanism::Extinction, v(10.0, 64), v(5.0, 64));
        assert!((sys.average_strength() - 1.0).abs() < 1e-6);
        sys.decay(0.5);
        assert!((sys.average_strength() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_max_slots_eviction() {
        let mut sys = BioMemorySystem::new(3);
        sys.store(MemoryMechanism::SourceMemory, v(1.0, 64), v(1.0, 64));
        sys.store(MemoryMechanism::SourceMemory, v(2.0, 64), v(2.0, 64));
        sys.store(MemoryMechanism::SourceMemory, v(3.0, 64), v(3.0, 64));
        sys.store(MemoryMechanism::SourceMemory, v(4.0, 64), v(4.0, 64));
        assert_eq!(sys.slots.len(), 3);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 1e-6);
        let d = vec![2.0, 2.0, 0.0];
        let sim = cosine_similarity(&a, &d);
        let expected = 2.0_f64 / (1.0_f64 * (8.0_f64).sqrt());
        assert!((sim - expected).abs() < 1e-6);
        assert!((cosine_similarity(&[], &[]) - 0.0).abs() < 1e-6);
    }
}
