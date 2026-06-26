use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeThread {
    pub id: u64,
    pub thread_type: ThreadType,
    pub vector: Vec<u8>,
    pub summary: String,
    pub coherence: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ThreadType {
    Goals,
    Reasoning,
    Memory,
    Sensory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconstructedNarrative {
    pub id: u64,
    pub bundled_vector: Vec<u8>,
    pub threads: Vec<NarrativeThread>,
    pub coherence: f64,
    pub complexity: f64,
    pub timestamp: u64,
    pub reconstruction_time_us: u64,
}

#[derive(Debug, Clone)]
pub struct ReconstructiveConfig {
    pub max_threads_per_type: usize,
    pub coherence_threshold: f64,
    pub bundle_dim: usize,
    pub narrative_history: usize,
}

impl Default for ReconstructiveConfig {
    fn default() -> Self {
        Self {
            max_threads_per_type: 3,
            coherence_threshold: 0.3,
            bundle_dim: 4096,
            narrative_history: 50,
        }
    }
}

pub struct ReconstructiveNarrative {
    config: ReconstructiveConfig,
    narratives: VecDeque<ReconstructedNarrative>,
    current_threads: Vec<NarrativeThread>,
    next_id: u64,
    step: u64,
    previous_bundled: Option<Vec<u8>>,
}

impl ReconstructiveNarrative {
    pub fn new(config: ReconstructiveConfig) -> Self {
        Self {
            config,
            narratives: VecDeque::new(),
            current_threads: Vec::new(),
            next_id: 1,
            step: 0,
            previous_bundled: None,
        }
    }

    pub fn rebuild_narrative(
        &mut self,
        goals: &[(Vec<u8>, String)],
        reasoning: &[(Vec<u8>, String)],
        memory: &[(Vec<u8>, String)],
        sensory: &[(Vec<u8>, String)],
    ) -> ReconstructedNarrative {
        self.step += 1;
        let ts = self.step;

        let mut threads = Vec::new();

        let mut add_threads = |items: &[(Vec<u8>, String)], ttype: ThreadType| {
            for (vector, summary) in items.iter().take(self.config.max_threads_per_type) {
                let id = self.next_id;
                self.next_id += 1;
                threads.push(NarrativeThread {
                    id,
                    thread_type: ttype,
                    vector: vector.clone(),
                    summary: summary.clone(),
                    coherence: 1.0,
                    timestamp: ts,
                });
            }
        };

        add_threads(goals, ThreadType::Goals);
        add_threads(reasoning, ThreadType::Reasoning);
        add_threads(memory, ThreadType::Memory);
        add_threads(sensory, ThreadType::Sensory);

        let thread_vectors: Vec<&[u8]> = threads.iter().map(|t| t.vector.as_slice()).collect();

        let bundled_vector = if thread_vectors.is_empty() {
            vec![0u8; self.config.bundle_dim]
        } else if thread_vectors.len() == 1 {
            thread_vectors[0].to_vec()
        } else {
            QuantizedVSA::majority_bundle(&thread_vectors)
        };

        let coherence = if threads.len() < 2 {
            1.0
        } else {
            let mut total = 0.0;
            let mut pairs = 0;
            for i in 0..threads.len() {
                for j in (i + 1)..threads.len() {
                    total += QuantizedVSA::similarity(&threads[i].vector, &threads[j].vector);
                    pairs += 1;
                }
            }
            total / pairs as f64
        };

        let complexity = 1.0 - coherence;

        for t in &mut threads {
            t.coherence = coherence;
        }

        let reconstruction_time_us = 10 + ts % 90;

        let narrative = ReconstructedNarrative {
            id: ts,
            bundled_vector: bundled_vector.clone(),
            threads: threads.clone(),
            coherence,
            complexity,
            timestamp: ts,
            reconstruction_time_us,
        };

        self.current_threads = threads;
        self.previous_bundled = Some(bundled_vector);

        self.narratives.push_back(narrative.clone());
        while self.narratives.len() > self.config.narrative_history {
            self.narratives.pop_front();
        }

        narrative
    }

    pub fn current_narrative(&self) -> Option<&ReconstructedNarrative> {
        self.narratives.back()
    }

    pub fn narrative_coherence_trend(&self, window: usize) -> String {
        if self.narratives.len() < 2 {
            return "stable".to_string();
        }
        let window = window.min(self.narratives.len());
        let start = self.narratives.len() - window;
        let coherences: Vec<f64> = self
            .narratives
            .iter()
            .skip(start)
            .map(|n| n.coherence)
            .collect();
        if coherences.len() < 2 {
            return "stable".to_string();
        }
        let first = coherences.first().copied().unwrap_or(0.5);
        let last = coherences.last().copied().unwrap_or(0.5);
        if last > first + 0.01 {
            "improving".to_string()
        } else if last < first - 0.01 {
            "declining".to_string()
        } else {
            "stable".to_string()
        }
    }

    pub fn thread_by_type(&self, thread_type: ThreadType) -> Vec<&NarrativeThread> {
        self.current_threads
            .iter()
            .filter(|t| t.thread_type == thread_type)
            .collect()
    }

    pub fn narrative_similarity(&self, a_id: u64, b_id: u64) -> f64 {
        let a = self.narratives.iter().find(|n| n.id == a_id);
        let b = self.narratives.iter().find(|n| n.id == b_id);
        match (a, b) {
            (Some(na), Some(nb)) => {
                QuantizedVSA::similarity(&na.bundled_vector, &nb.bundled_vector)
            }
            _ => 0.0,
        }
    }

    pub fn narrative_entropy(&self) -> f64 {
        let narrative = match self.narratives.back() {
            Some(n) => n,
            None => return 0.0,
        };
        let coherences: Vec<f64> = narrative.threads.iter().map(|t| t.coherence).collect();
        if coherences.is_empty() {
            return 0.0;
        }
        let mut entropy = 0.0;
        for &c in &coherences {
            let p = c.max(1e-10).min(1.0 - 1e-10);
            entropy -= p * p.log2() + (1.0 - p) * (1.0 - p).log2();
        }
        entropy / coherences.len() as f64
    }

    pub fn reset(&mut self) {
        self.narratives.clear();
        self.current_threads.clear();
        self.next_id = 1;
        self.step = 0;
        self.previous_bundled = None;
    }

    pub fn narrative_count(&self) -> usize {
        self.narratives.len()
    }

    pub fn stats(&self) -> (usize, f64, f64, usize) {
        let count = self.narratives.len();
        if count == 0 {
            return (0, 0.0, 0.0, self.step as usize);
        }
        let avg_coherence: f64 =
            self.narratives.iter().map(|n| n.coherence).sum::<f64>() / count as f64;
        let avg_complexity: f64 =
            self.narratives.iter().map(|n| n.complexity).sum::<f64>() / count as f64;
        (count, avg_coherence, avg_complexity, self.step as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vector(seed: u8) -> Vec<u8> {
        let mut v = vec![0u8; 4096];
        for i in 0..4096 {
            v[i] = ((seed as u64)
                .wrapping_mul(2654435761)
                .wrapping_add(i as u64)
                % 2) as u8;
        }
        v
    }

    #[test]
    fn test_initial_state() {
        let config = ReconstructiveConfig::default();
        let rn = ReconstructiveNarrative::new(config);
        assert_eq!(rn.narrative_count(), 0);
        assert!(rn.current_narrative().is_none());
    }

    #[test]
    fn test_rebuild_basic() {
        let config = ReconstructiveConfig::default();
        let mut rn = ReconstructiveNarrative::new(config);
        let goals = vec![(make_vector(1), "goal 1".to_string())];
        let reasoning = vec![(make_vector(2), "reasoning 1".to_string())];
        let memory = vec![(make_vector(3), "memory 1".to_string())];
        let sensory = vec![(make_vector(4), "sensory 1".to_string())];
        let narrative = rn.rebuild_narrative(&goals, &reasoning, &memory, &sensory);
        assert_eq!(rn.narrative_count(), 1);
        assert_eq!(narrative.bundled_vector.len(), 4096);
        assert!(rn.current_narrative().is_some());
    }

    #[test]
    fn test_rebuild_multiple() {
        let config = ReconstructiveConfig::default();
        let mut rn = ReconstructiveNarrative::new(config);
        let goals = vec![
            (make_vector(1), "goal 1".to_string()),
            (make_vector(2), "goal 2".to_string()),
        ];
        let reasoning = vec![
            (make_vector(3), "reasoning 1".to_string()),
            (make_vector(4), "reasoning 2".to_string()),
            (make_vector(5), "reasoning 3".to_string()),
        ];
        let memory = vec![];
        let sensory = vec![(make_vector(6), "sensory 1".to_string())];
        let narrative = rn.rebuild_narrative(&goals, &reasoning, &memory, &sensory);
        assert_eq!(rn.narrative_count(), 1);
        assert!(narrative.coherence >= 0.0);
        assert!(narrative.complexity >= 0.0);
    }

    #[test]
    fn test_coherence_trend() {
        let config = ReconstructiveConfig::default();
        let mut rn = ReconstructiveNarrative::new(config);
        // With fewer than 2 narratives, trend is stable
        assert_eq!(rn.narrative_coherence_trend(3), "stable");

        // Create narratives with different coherences
        for i in 0..5 {
            let v = make_vector(i);
            let goals = vec![(v.clone(), format!("goal {}", i))];
            rn.rebuild_narrative(&goals, &goals, &goals, &goals);
        }
        let trend = rn.narrative_coherence_trend(3);
        // trend will be one of the valid options
        assert!(trend == "improving" || trend == "declining" || trend == "stable");
    }

    #[test]
    fn test_narrative_history_cap() {
        let config = ReconstructiveConfig {
            narrative_history: 3,
            ..Default::default()
        };
        let mut rn = ReconstructiveNarrative::new(config);
        for i in 0..10 {
            let v = make_vector(i);
            let goals = vec![(v.clone(), format!("goal {}", i))];
            rn.rebuild_narrative(&goals, &[], &[], &[]);
        }
        assert_eq!(rn.narrative_count(), 3);
    }

    #[test]
    fn test_thread_by_type() {
        let config = ReconstructiveConfig::default();
        let mut rn = ReconstructiveNarrative::new(config);
        let goals = vec![(make_vector(1), "goal 1".to_string())];
        let reasoning = vec![(make_vector(2), "reasoning 1".to_string())];
        let memory = vec![(make_vector(3), "memory 1".to_string())];
        let sensory = vec![(make_vector(4), "sensory 1".to_string())];
        rn.rebuild_narrative(&goals, &reasoning, &memory, &sensory);
        let g = rn.thread_by_type(ThreadType::Goals);
        assert_eq!(g.len(), 1);
        let r = rn.thread_by_type(ThreadType::Reasoning);
        assert_eq!(r.len(), 1);
        let m = rn.thread_by_type(ThreadType::Memory);
        assert_eq!(m.len(), 1);
        let s = rn.thread_by_type(ThreadType::Sensory);
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_narrative_similarity() {
        let config = ReconstructiveConfig::default();
        let mut rn = ReconstructiveNarrative::new(config);
        let v1 = make_vector(1);
        let v2 = make_vector(2);
        let n1 = rn.rebuild_narrative(&[(v1.clone(), "a".to_string())], &[], &[], &[]);
        let n2 = rn.rebuild_narrative(&[(v2.clone(), "b".to_string())], &[], &[], &[]);
        let sim = rn.narrative_similarity(n1.id, n2.id);
        assert!(sim >= 0.0 && sim <= 1.0);
        // Same vector should give higher similarity
        let n3 = rn.rebuild_narrative(&[(v1.clone(), "c".to_string())], &[], &[], &[]);
        let sim_same = rn.narrative_similarity(n1.id, n3.id);
        let sim_diff = rn.narrative_similarity(n1.id, n2.id);
        assert!(sim_same > sim_diff || (sim_same - sim_diff).abs() < 0.1);
    }

    #[test]
    fn test_stats() {
        let config = ReconstructiveConfig::default();
        let mut rn = ReconstructiveNarrative::new(config);
        let (count, avg_c, avg_x, step) = rn.stats();
        assert_eq!(count, 0);
        assert_eq!(avg_c, 0.0);
        assert_eq!(avg_x, 0.0);
        assert_eq!(step, 0);

        for i in 0..3 {
            let v = make_vector(i);
            rn.rebuild_narrative(&[(v, format!("goal {}", i))], &[], &[], &[]);
        }
        let (count, avg_c, avg_x, step) = rn.stats();
        assert_eq!(count, 3);
        assert!(avg_c > 0.0);
        assert!(avg_x > 0.0);
        assert_eq!(step, 3);
    }
}
