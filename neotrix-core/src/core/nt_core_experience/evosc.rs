use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct ContrastiveInsight {
    pub id: u64,
    pub success_pattern: Vec<u8>,
    pub failure_pattern: Vec<u8>,
    pub divergence_vector: Vec<u8>,
    pub context_label: String,
    pub confidence: f64,
    pub success_count: u64,
    pub failure_count: u64,
    pub created_at: u64,
    pub last_activated: u64,
}

#[derive(Debug, Clone)]
pub struct ConsolidatedMemory {
    pub id: u64,
    pub compressed: Vec<u8>,
    pub source_ids: Vec<u64>,
    pub coherence: f64,
    pub importance: f64,
    pub created_at: u64,
    pub access_count: u64,
}

pub struct ContrastiveReflector {
    pub insights: Vec<ContrastiveInsight>,
    pub max_insights: usize,
    pub similarity_threshold: f64,
    next_id: u64,
    total_comparisons: u64,
}

impl ContrastiveReflector {
    pub fn new(max_insights: usize) -> Self {
        Self {
            insights: Vec::with_capacity(max_insights),
            max_insights,
            similarity_threshold: 0.65,
            next_id: 0,
            total_comparisons: 0,
        }
    }

    pub fn compare(
        &mut self,
        success_trace: &[Vec<u8>],
        failure_trace: &[Vec<u8>],
        context: &str,
        cycle: u64,
    ) -> Option<u64> {
        self.total_comparisons += 1;
        if success_trace.is_empty() || failure_trace.is_empty() {
            return None;
        }
        let s_slices: Vec<&[u8]> = success_trace.iter().map(|v| v.as_slice()).collect();
        let f_slices: Vec<&[u8]> = failure_trace.iter().map(|v| v.as_slice()).collect();
        let success_bundle = QuantizedVSA::bundle(&s_slices);
        let failure_bundle = QuantizedVSA::bundle(&f_slices);
        let divergence = QuantizedVSA::xor_bind(&success_bundle, &failure_bundle);
        let similarity = QuantizedVSA::similarity(&success_bundle, &failure_bundle);

        if similarity > self.similarity_threshold {
            return None;
        }

        let existing = self.insights.iter_mut().find(|i| {
            let sim = QuantizedVSA::similarity(&i.divergence_vector, &divergence);
            sim > self.similarity_threshold
        });

        if let Some(existing) = existing {
            existing.success_count += 1;
            existing.last_activated = cycle;
            existing.confidence = (existing.confidence + similarity).min(1.0);
            Some(existing.id)
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.insights.push(ContrastiveInsight {
                id,
                success_pattern: success_bundle,
                failure_pattern: failure_bundle,
                divergence_vector: divergence,
                context_label: context.to_string(),
                confidence: 1.0 - similarity,
                success_count: 1,
                failure_count: 1,
                created_at: cycle,
                last_activated: cycle,
            });
            if self.insights.len() > self.max_insights {
                self.insights.sort_by(|a, b| {
                    let a_score = a.confidence * (a.success_count as f64 + a.failure_count as f64);
                    let b_score = b.confidence * (b.success_count as f64 + b.failure_count as f64);
                    a_score
                        .partial_cmp(&b_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                self.insights.remove(0);
            }
            Some(id)
        }
    }

    pub fn error_patterns(&self, min_confidence: f64) -> Vec<&ContrastiveInsight> {
        self.insights
            .iter()
            .filter(|i| i.confidence >= min_confidence)
            .collect()
    }

    pub fn find_applicable(&self, current_context: &str) -> Vec<&ContrastiveInsight> {
        self.insights
            .iter()
            .filter(|i| {
                i.context_label.contains(current_context)
                    || current_context.contains(&i.context_label)
            })
            .collect()
    }

    pub fn stats(&self) -> ContrastiveStats {
        ContrastiveStats {
            total_insights: self.insights.len(),
            max_insights: self.max_insights,
            total_comparisons: self.total_comparisons,
            avg_confidence: if self.insights.is_empty() {
                0.0
            } else {
                self.insights.iter().map(|i| i.confidence).sum::<f64>() / self.insights.len() as f64
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ContrastiveStats {
    pub total_insights: usize,
    pub max_insights: usize,
    pub total_comparisons: u64,
    pub avg_confidence: f64,
}

pub struct SelfConsolidation {
    pub memories: VecDeque<ConsolidatedMemory>,
    pub max_memories: usize,
    pub merge_threshold: f64,
    next_id: u64,
    total_consolidations: u64,
}

impl SelfConsolidation {
    pub fn new(max_memories: usize) -> Self {
        Self {
            memories: VecDeque::with_capacity(max_memories),
            max_memories,
            merge_threshold: 0.72,
            next_id: 0,
            total_consolidations: 0,
        }
    }

    pub fn consolidate(&mut self, traces: &[Vec<u8>], importance: f64, cycle: u64) -> u64 {
        let slices: Vec<&[u8]> = traces.iter().map(|v| v.as_slice()).collect();
        let bundled = QuantizedVSA::bundle(&slices);
        let coherence = if traces.len() < 2 {
            1.0
        } else {
            let mut sim_sum = 0.0;
            let mut pair_count = 0;
            for i in 0..traces.len().min(10) {
                for j in (i + 1)..traces.len().min(10) {
                    sim_sum += QuantizedVSA::similarity(&traces[i], &traces[j]);
                    pair_count += 1;
                }
            }
            sim_sum / pair_count.max(1) as f64
        };

        let merged = self.try_merge(&bundled, coherence, importance);
        if let Some(merged_id) = merged {
            return merged_id;
        }

        let id = self.next_id;
        self.next_id += 1;
        self.memories.push_back(ConsolidatedMemory {
            id,
            compressed: bundled,
            source_ids: (0..traces.len()).map(|i| i as u64).collect(),
            coherence,
            importance,
            created_at: cycle,
            access_count: 0,
        });
        self.total_consolidations += 1;

        if self.memories.len() > self.max_memories {
            self.memories.pop_front();
        }
        id
    }

    fn try_merge(
        &mut self,
        new_compressed: &[u8],
        new_coherence: f64,
        new_importance: f64,
    ) -> Option<u64> {
        for mem in self.memories.iter_mut() {
            let sim = QuantizedVSA::similarity(&mem.compressed, new_compressed);
            if sim > self.merge_threshold {
                let merged = QuantizedVSA::bundle(&[&mem.compressed, new_compressed]);
                mem.compressed = merged;
                mem.coherence = (mem.coherence + new_coherence) / 2.0;
                mem.importance = mem.importance.max(new_importance);
                mem.access_count += 1;
                self.total_consolidations += 1;
                return Some(mem.id);
            }
        }
        None
    }

    pub fn retrieve(&mut self, query: &[u8], top_k: usize) -> Vec<u64> {
        let mut scored: Vec<(usize, f64)> = self
            .memories
            .iter()
            .enumerate()
            .map(|(i, m)| (i, QuantizedVSA::similarity(&m.compressed, query)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let indices: Vec<usize> = scored.into_iter().take(top_k).map(|(i, _)| i).collect();
        for &idx in &indices {
            if let Some(m) = self.memories.get_mut(idx) {
                m.access_count += 1;
            }
        }
        indices.iter().map(|&i| self.memories[i].id).collect()
    }

    pub fn stats(&self) -> ConsolidationStats {
        ConsolidationStats {
            total_memories: self.memories.len(),
            max_memories: self.max_memories,
            total_consolidations: self.total_consolidations,
            avg_coherence: if self.memories.is_empty() {
                0.0
            } else {
                self.memories.iter().map(|m| m.coherence).sum::<f64>() / self.memories.len() as f64
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConsolidationStats {
    pub total_memories: usize,
    pub max_memories: usize,
    pub total_consolidations: u64,
    pub avg_coherence: f64,
}

pub struct EvoSC {
    pub reflector: ContrastiveReflector,
    pub consolidator: SelfConsolidation,
}

impl EvoSC {
    pub fn new() -> Self {
        Self {
            reflector: ContrastiveReflector::new(200),
            consolidator: SelfConsolidation::new(300),
        }
    }

    pub fn tick(
        &mut self,
        success_trace: &[Vec<u8>],
        failure_trace: &[Vec<u8>],
        context: &str,
        cycle: u64,
    ) {
        if self
            .reflector
            .compare(success_trace, failure_trace, context, cycle)
            .is_some()
        {
            let mut all: Vec<Vec<u8>> =
                Vec::with_capacity(success_trace.len() + failure_trace.len());
            all.extend_from_slice(success_trace);
            all.extend_from_slice(failure_trace);
            let slices: Vec<&[u8]> = all.iter().map(|v| v.as_slice()).collect();
            let compressed: Vec<Vec<u8>> =
                slices.chunks(4).map(|c| QuantizedVSA::bundle(c)).collect();
            self.consolidator.consolidate(&compressed, 0.6, cycle);
        }
    }

    pub fn stats(&self) -> EvoSCStats {
        EvoSCStats {
            insights: self.reflector.stats(),
            consolidation: self.consolidator.stats(),
        }
    }
}

impl Default for EvoSC {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EvoSCStats {
    pub insights: ContrastiveStats,
    pub consolidation: ConsolidationStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

    fn make_trace(values: &[u8]) -> Vec<Vec<u8>> {
        values
            .iter()
            .map(|&v| QuantizedVSA::seeded_random(v as u64, VSA_DIM))
            .collect()
    }

    #[test]
    fn test_contrastive_reflector_new() {
        let r = ContrastiveReflector::new(100);
        assert_eq!(r.insights.len(), 0);
        assert_eq!(r.total_comparisons, 0);
    }

    #[test]
    fn test_contrastive_compare_different() {
        let mut r = ContrastiveReflector::new(100);
        let success = make_trace(&[1, 2, 3]);
        let failure = make_trace(&[4, 5, 6]);
        let id = r.compare(&success, &failure, "test", 1);
        assert!(id.is_some());
        assert_eq!(r.insights.len(), 1);
    }

    #[test]
    fn test_contrastive_merge_similar() {
        let mut r = ContrastiveReflector::new(100);
        let trace = make_trace(&[1, 2, 3]);
        let id1 = r.compare(&trace, &make_trace(&[4, 5, 6]), "ctx", 1);
        let id2 = r.compare(&trace, &make_trace(&[7, 8, 9]), "ctx", 2);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_error_patterns_filter() {
        let mut r = ContrastiveReflector::new(100);
        r.compare(&make_trace(&[1]), &make_trace(&[10]), "a", 1);
        r.compare(&make_trace(&[2]), &make_trace(&[20]), "b", 2);
        let patterns = r.error_patterns(0.0);
        assert_eq!(patterns.len(), 2);
    }

    #[test]
    fn test_self_consolidation_new() {
        let s = SelfConsolidation::new(100);
        assert_eq!(s.memories.len(), 0);
    }

    #[test]
    fn test_consolidate() {
        let mut s = SelfConsolidation::new(100);
        let traces = make_trace(&[1, 2, 3, 4]);
        let id = s.consolidate(&traces, 0.8, 1);
        assert_eq!(s.memories.len(), 1);
        assert_eq!(id, 0);
    }

    #[test]
    fn test_retrieve() {
        let mut s = SelfConsolidation::new(100);
        let t1 = make_trace(&[1, 2]);
        let t2 = make_trace(&[10, 20]);
        s.consolidate(&t1, 0.9, 1);
        s.consolidate(&t2, 0.9, 2);
        let query = QuantizedVSA::seeded_random(1, VSA_DIM);
        let results = s.retrieve(&query, 2);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_self_consolidation_merge() {
        let mut s = SelfConsolidation::new(100);
        let t1 = make_trace(&[1, 2]);
        let t2 = make_trace(&[1, 3]);
        s.consolidate(&t1, 0.8, 1);
        s.consolidate(&t2, 0.8, 2);
        assert!(s.memories.len() <= 2);
    }

    #[test]
    fn test_evosc_new() {
        let e = EvoSC::new();
        assert_eq!(e.reflector.insights.len(), 0);
        assert_eq!(e.consolidator.memories.len(), 0);
    }

    #[test]
    fn test_evosc_tick() {
        let mut e = EvoSC::new();
        let success = make_trace(&[1, 2, 3]);
        let failure = make_trace(&[4, 5, 6]);
        e.tick(&success, &failure, "test_context", 1);
        assert_eq!(e.reflector.insights.len(), 1);
    }
}
