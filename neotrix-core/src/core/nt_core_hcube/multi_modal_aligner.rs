use std::collections::{HashMap, VecDeque};

/// Modality types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modality {
    Text,
    VSA,
    Sensory,
    Action,
    Concept,
}

/// An alignment between two concepts in different modalities
#[derive(Debug, Clone)]
pub struct AlignmentPair {
    pub id: u64,
    pub source_modality: Modality,
    pub target_modality: Modality,
    pub source_fingerprint: u64,
    pub target_fingerprint: u64,
    pub source_label: String,
    pub target_label: String,
    pub strength: f64,
    pub co_occurrence_count: u64,
}

/// Multi-modal alignment engine
#[derive(Debug)]
pub struct MultiModalAligner {
    pub alignments: VecDeque<AlignmentPair>,
    pub by_source: HashMap<(Modality, u64), Vec<u64>>,
    pub by_target: HashMap<(Modality, u64), Vec<u64>>,
    max_alignments: usize,
    next_id: u64,
}

impl MultiModalAligner {
    pub fn new() -> Self {
        Self {
            alignments: VecDeque::with_capacity(5000),
            by_source: HashMap::new(),
            by_target: HashMap::new(),
            max_alignments: 5000,
            next_id: 1,
        }
    }

    /// Record or strengthen an alignment between two concepts
    pub fn align(
        &mut self,
        source_mod: Modality,
        source_fp: u64,
        source_label: &str,
        target_mod: Modality,
        target_fp: u64,
        target_label: &str,
    ) -> u64 {
        let existing: Vec<u64> = self
            .by_source
            .get(&(source_mod, source_fp))
            .map(|ids| {
                ids.iter()
                    .filter(|&&id| {
                        if let Some(pair) = self.alignments.iter().find(|a| a.id == id) {
                            pair.target_modality == target_mod
                                && pair.target_fingerprint == target_fp
                        } else {
                            false
                        }
                    })
                    .copied()
                    .collect()
            })
            .unwrap_or_default();

        if let Some(&id) = existing.first() {
            if let Some(pair) = self.alignments.iter_mut().find(|a| a.id == id) {
                pair.co_occurrence_count += 1;
                pair.strength = (pair.strength + 0.05).min(1.0);
            }
            return id;
        }

        let id = self.next_id;
        if self.alignments.len() >= self.max_alignments {
            if let Some(oldest) = self.alignments.pop_front() {
                self.by_source
                    .remove(&(oldest.source_modality, oldest.source_fingerprint));
                self.by_target
                    .remove(&(oldest.target_modality, oldest.target_fingerprint));
            }
        }

        let pair = AlignmentPair {
            id,
            source_modality: source_mod,
            target_modality: target_mod,
            source_fingerprint: source_fp,
            target_fingerprint: target_fp,
            source_label: source_label.to_string(),
            target_label: target_label.to_string(),
            strength: 0.5,
            co_occurrence_count: 1,
        };

        self.by_source
            .entry((source_mod, source_fp))
            .or_default()
            .push(id);
        self.by_target
            .entry((target_mod, target_fp))
            .or_default()
            .push(id);
        self.alignments.push_back(pair);
        self.next_id += 1;
        id
    }

    /// Find alignments from a given concept in a modality
    pub fn find_alignments(&self, modality: Modality, fingerprint: u64) -> Vec<&AlignmentPair> {
        let mut results = Vec::new();
        if let Some(ids) = self.by_source.get(&(modality, fingerprint)) {
            for id in ids {
                if let Some(pair) = self.alignments.iter().find(|a| a.id == *id) {
                    results.push(pair);
                }
            }
        }
        if let Some(ids) = self.by_target.get(&(modality, fingerprint)) {
            for id in ids {
                if let Some(pair) = self.alignments.iter().find(|a| a.id == *id) {
                    if !results.iter().any(|r: &&AlignmentPair| r.id == pair.id) {
                        results.push(pair);
                    }
                }
            }
        }
        results
    }

    /// Translate: find the best match in target modality
    pub fn translate(
        &self,
        source_mod: Modality,
        source_fp: u64,
        target_mod: Modality,
    ) -> Option<(u64, String, f64)> {
        let alignments = self.find_alignments(source_mod, source_fp);
        alignments
            .iter()
            .filter(|a| a.target_modality == target_mod)
            .max_by(|a, b| {
                a.strength
                    .partial_cmp(&b.strength)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|a| (a.target_fingerprint, a.target_label.clone(), a.strength))
            .or_else(|| {
                alignments
                    .iter()
                    .find(|a| a.source_modality == source_mod)
                    .and_then(|a| {
                        self.translate(a.target_modality, a.target_fingerprint, target_mod)
                    })
            })
    }

    /// Clean low-strength alignments
    pub fn prune(&mut self, min_strength: f64) -> usize {
        let before = self.alignments.len();
        self.alignments.retain(|a| a.strength >= min_strength);
        self.by_source.clear();
        self.by_target.clear();
        for pair in &self.alignments {
            self.by_source
                .entry((pair.source_modality, pair.source_fingerprint))
                .or_default()
                .push(pair.id);
            self.by_target
                .entry((pair.target_modality, pair.target_fingerprint))
                .or_default()
                .push(pair.id);
        }
        before - self.alignments.len()
    }

    pub fn stats(&self) -> String {
        format!(
            "MultiModalAligner: {} alignments, {} source indices, {} target indices",
            self.alignments.len(),
            self.by_source.len(),
            self.by_target.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_find_alignment() {
        let mut aligner = MultiModalAligner::new();
        let id = aligner.align(
            Modality::Text,
            42,
            "apple",
            Modality::VSA,
            1001,
            "vsa_apple",
        );
        assert_eq!(id, 1);
        let found = aligner.find_alignments(Modality::Text, 42);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].target_label, "vsa_apple");
        assert!((found[0].strength - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_strengthen_existing_alignment() {
        let mut aligner = MultiModalAligner::new();
        aligner.align(
            Modality::Text,
            10,
            "dog",
            Modality::Sensory,
            20,
            "sensor_dog",
        );
        let id = aligner.align(
            Modality::Text,
            10,
            "dog",
            Modality::Sensory,
            20,
            "sensor_dog",
        );
        let found = aligner.find_alignments(Modality::Text, 10);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].co_occurrence_count, 2);
        assert!((found[0].strength - 0.55).abs() < 1e-9);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_max_alignments_bounds() {
        let mut aligner = MultiModalAligner::new();
        aligner.max_alignments = 5;
        for i in 0..10 {
            aligner.align(
                Modality::Text,
                i as u64,
                &format!("word_{}", i),
                Modality::VSA,
                i as u64 + 100,
                &format!("vsa_{}", i),
            );
        }
        assert_eq!(aligner.alignments.len(), 5);
        assert_eq!(aligner.by_source.len(), 5);
    }

    #[test]
    fn test_translate_chain() {
        let mut aligner = MultiModalAligner::new();
        aligner.align(
            Modality::Text,
            1,
            "hello",
            Modality::VSA,
            10,
            "vsa_greeting",
        );
        aligner.align(
            Modality::VSA,
            10,
            "vsa_greeting",
            Modality::Sensory,
            100,
            "wave",
        );
        let result = aligner.translate(Modality::Text, 1, Modality::Sensory);
        assert!(result.is_some());
        let (fp, label, strength) = result.unwrap();
        assert_eq!(fp, 100);
        assert_eq!(label, "wave");
        assert!(strength > 0.0);
    }

    #[test]
    fn test_translate_direct() {
        let mut aligner = MultiModalAligner::new();
        aligner.align(Modality::Text, 99, "cat", Modality::Concept, 200, "feline");
        let result = aligner.translate(Modality::Text, 99, Modality::Concept);
        assert!(result.is_some());
        assert_eq!(result.unwrap().1, "feline");
    }

    #[test]
    fn test_prune_removes_low_strength() {
        let mut aligner = MultiModalAligner::new();
        aligner.align(Modality::Text, 1, "weak", Modality::Concept, 10, "low");
        let strong_id = aligner.align(Modality::Text, 2, "strong", Modality::Concept, 20, "high");
        if let Some(pair) = aligner.alignments.iter_mut().find(|a| a.id == strong_id) {
            pair.strength = 0.9;
        }
        let pruned = aligner.prune(0.6);
        assert_eq!(pruned, 1);
        assert_eq!(aligner.alignments.len(), 1);
        assert_eq!(aligner.by_source.len(), 1);
    }

    #[test]
    fn test_empty_aligner_returns_none() {
        let aligner = MultiModalAligner::new();
        let found = aligner.find_alignments(Modality::Text, 99);
        assert!(found.is_empty());
        let translated = aligner.translate(Modality::Text, 99, Modality::VSA);
        assert!(translated.is_none());
    }

    #[test]
    fn test_align_bidirectional_lookup() {
        let mut aligner = MultiModalAligner::new();
        aligner.align(
            Modality::Action,
            7,
            "jump",
            Modality::Sensory,
            8,
            "visual_jump",
        );
        let from_action = aligner.find_alignments(Modality::Action, 7);
        let from_sensory = aligner.find_alignments(Modality::Sensory, 8);
        assert_eq!(from_action.len(), 1);
        assert_eq!(from_sensory.len(), 1);
        assert_eq!(from_action[0].id, from_sensory[0].id);
    }

    #[test]
    fn test_prune_noop_when_all_above_threshold() {
        let mut aligner = MultiModalAligner::new();
        aligner.align(Modality::Text, 1, "a", Modality::VSA, 10, "va");
        let pruned = aligner.prune(0.3);
        assert_eq!(pruned, 0);
        assert_eq!(aligner.alignments.len(), 1);
    }

    #[test]
    fn test_stats_format() {
        let mut aligner = MultiModalAligner::new();
        aligner.align(Modality::Text, 1, "x", Modality::Concept, 2, "y");
        let s = aligner.stats();
        assert!(s.contains("1 alignments"));
        assert!(s.contains("1 source"));
        assert!(s.contains("1 target"));
    }

    #[test]
    fn test_next_id_increment() {
        let mut aligner = MultiModalAligner::new();
        let id1 = aligner.align(Modality::Text, 1, "one", Modality::VSA, 2, "v_one");
        let id2 = aligner.align(Modality::Text, 3, "two", Modality::VSA, 4, "v_two");
        assert_eq!(id2, id1 + 1);
    }
}
