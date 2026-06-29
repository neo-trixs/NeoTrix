#![forbid(unsafe_code)]
// REVIVED Task 2 — dead_code removed

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Clone, Debug, PartialEq)]
pub struct ProceduralTrace {
    pub id: u64,
    pub state_vsa: Vec<u8>,
    pub action_vsa: Vec<u8>,
    pub outcome_vsa: Vec<u8>,
    pub timestamp_ns: u128,
    pub reward: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkillChunk {
    pub id: u64,
    pub trigger_pattern: Vec<u8>,
    pub action_sequence: Vec<Vec<u8>>,
    pub expected_outcome: Vec<u8>,
    pub frequency: u32,
    pub last_used_ns: u128,
    pub strength: f64,
}

pub struct OnlineSkillCompiler {
    pub traces: Vec<ProceduralTrace>,
    pub skills: Vec<SkillChunk>,
    pub max_traces: usize,
    pub max_skills: usize,
    pub min_sequence_reps: u32,
    pub similarity_threshold: f64,
    pub decay_rate: f64,
    next_trace_id: u64,
    next_skill_id: u64,
}

impl Default for OnlineSkillCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl OnlineSkillCompiler {
    pub fn new() -> Self {
        Self {
            traces: Vec::new(),
            skills: Vec::new(),
            max_traces: 200,
            max_skills: 50,
            min_sequence_reps: 3,
            similarity_threshold: 0.85,
            decay_rate: 0.1,
            next_trace_id: 1,
            next_skill_id: 1,
        }
    }

    pub fn observe(&mut self, state: &[u8], action: &[u8], outcome: &[u8], reward: f64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let trace = ProceduralTrace {
            id: self.next_trace_id,
            state_vsa: state.to_vec(),
            action_vsa: action.to_vec(),
            outcome_vsa: outcome.to_vec(),
            timestamp_ns: now,
            reward,
        };
        self.next_trace_id += 1;
        if self.traces.len() >= self.max_traces {
            self.traces.remove(0);
        }
        self.traces.push(trace);
    }

    pub fn compile(&mut self) -> Vec<u64> {
        let mut new_ids = Vec::new();

        let mut used = vec![false; self.traces.len()];
        for i in 0..self.traces.len() {
            if used[i] {
                continue;
            }

            let mut cluster: Vec<usize> = vec![i];
            used[i] = true;

            for j in (i + 1)..self.traces.len() {
                if used[j] {
                    continue;
                }
                let sim =
                    QuantizedVSA::similarity(&self.traces[i].state_vsa, &self.traces[j].state_vsa);
                if sim >= self.similarity_threshold {
                    cluster.push(j);
                    used[j] = true;
                }
            }

            if (cluster.len() as u32) < self.min_sequence_reps {
                continue;
            }

            let mut clustered: Vec<&ProceduralTrace> =
                cluster.iter().map(|&idx| &self.traces[idx]).collect();
            clustered.sort_by_key(|t| t.timestamp_ns);

            let trigger_state_refs: Vec<&[u8]> =
                clustered.iter().map(|t| t.state_vsa.as_slice()).collect();
            let trigger_pattern = QuantizedVSA::bundle(&trigger_state_refs);

            let action_sequence: Vec<Vec<u8>> =
                clustered.iter().map(|t| t.action_vsa.clone()).collect();

            let outcome_refs: Vec<&[u8]> =
                clustered.iter().map(|t| t.outcome_vsa.as_slice()).collect();
            let expected_outcome = QuantizedVSA::bundle(&outcome_refs);

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();

            let skill = SkillChunk {
                id: self.next_skill_id,
                trigger_pattern,
                action_sequence,
                expected_outcome,
                frequency: cluster.len() as u32,
                last_used_ns: now,
                strength: 1.0,
            };
            self.next_skill_id += 1;

            if self.skills.len() >= self.max_skills {
                let weakest = self
                    .skills
                    .iter()
                    .enumerate()
                    .min_by(|a, b| {
                        a.1.strength
                            .partial_cmp(&b.1.strength)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(idx, _)| idx);
                if let Some(idx) = weakest {
                    self.skills.remove(idx);
                }
            }

            new_ids.push(skill.id);
            self.skills.push(skill);
        }

        new_ids
    }

    pub fn retrieve(&self, state_vsa: &[u8]) -> Option<&SkillChunk> {
        let mut best_sim = 0.0f64;
        let mut best_idx = None;
        for (i, skill) in self.skills.iter().enumerate() {
            let sim = QuantizedVSA::similarity(state_vsa, &skill.trigger_pattern);
            if sim > best_sim && sim >= self.similarity_threshold {
                best_sim = sim;
                best_idx = Some(i);
            }
        }
        best_idx.map(|i| &self.skills[i])
    }

    pub fn execute(&mut self, state_vsa: &[u8]) -> Option<(&SkillChunk, Vec<Vec<u8>>)> {
        let idx = self.best_match_index(state_vsa)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        self.skills[idx].last_used_ns = now;
        let seq = self.skills[idx].action_sequence.clone();
        Some((&self.skills[idx], seq))
    }

    fn best_match_index(&self, state_vsa: &[u8]) -> Option<usize> {
        let mut best_sim = 0.0f64;
        let mut best_idx = None;
        for (i, skill) in self.skills.iter().enumerate() {
            let sim = QuantizedVSA::similarity(state_vsa, &skill.trigger_pattern);
            if sim > best_sim && sim >= self.similarity_threshold {
                best_sim = sim;
                best_idx = Some(i);
            }
        }
        best_idx
    }

    pub fn decay(&mut self, now_ns: u128) {
        let days_ns: u128 = 86_400_000_000_000;
        self.skills.retain_mut(|skill| {
            let elapsed = if now_ns > skill.last_used_ns {
                now_ns - skill.last_used_ns
            } else {
                0
            };
            let days = (elapsed / days_ns) as f64;
            skill.strength *= 1.0 - self.decay_rate * days;
            if skill.strength < 0.0 {
                skill.strength = 0.0;
            }
            skill.strength >= 0.1
        });
    }

    pub fn skill_count(&self) -> usize {
        self.skills.len()
    }

    pub fn trace_count(&self) -> usize {
        self.traces.len()
    }

    pub fn clear_traces(&mut self) {
        self.traces.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_binary_vsa(val: u8) -> Vec<u8> {
        vec![if val == 0 { 0u8 } else { 1u8 }; 4096]
    }

    #[test]
    fn test_observe_adds_trace() {
        let mut compiler = OnlineSkillCompiler::new();
        assert_eq!(compiler.trace_count(), 0);
        compiler.observe(
            &make_binary_vsa(0),
            &make_binary_vsa(1),
            &make_binary_vsa(2),
            1.0,
        );
        assert_eq!(compiler.trace_count(), 1);
    }

    #[test]
    fn test_compile_creates_chunk_for_repeated_identical_traces() {
        let mut compiler = OnlineSkillCompiler::new();
        let state = make_binary_vsa(10);
        let action = make_binary_vsa(20);
        let outcome = make_binary_vsa(30);
        for _ in 0..3 {
            compiler.observe(&state, &action, &outcome, 1.0);
        }
        let ids = compiler.compile();
        assert_eq!(ids.len(), 1, "should create one skill chunk");
        assert_eq!(compiler.skill_count(), 1);
    }

    #[test]
    fn test_compile_ignores_unique_traces() {
        let mut compiler = OnlineSkillCompiler::new();
        compiler.observe(
            &make_binary_vsa(1),
            &make_binary_vsa(2),
            &make_binary_vsa(3),
            1.0,
        );
        compiler.observe(
            &make_binary_vsa(4),
            &make_binary_vsa(5),
            &make_binary_vsa(6),
            1.0,
        );
        let ids = compiler.compile();
        assert_eq!(ids.len(), 0, "unique traces should not compile");
        assert_eq!(compiler.skill_count(), 0);
    }

    #[test]
    fn test_compile_requires_min_reps() {
        let mut compiler = OnlineSkillCompiler::new();
        compiler.min_sequence_reps = 5;
        let state = make_binary_vsa(10);
        let action = make_binary_vsa(20);
        let outcome = make_binary_vsa(30);
        for _ in 0..4 {
            compiler.observe(&state, &action, &outcome, 1.0);
        }
        let ids = compiler.compile();
        assert_eq!(ids.len(), 0, "4 traces < min_sequence_reps=5");
    }

    #[test]
    fn test_retrieve_finds_matching_trigger() {
        let mut compiler = OnlineSkillCompiler::new();
        let state = make_binary_vsa(10);
        let action = make_binary_vsa(20);
        let outcome = make_binary_vsa(30);
        for _ in 0..3 {
            compiler.observe(&state, &action, &outcome, 1.0);
        }
        compiler.compile();
        let found = compiler.retrieve(&state);
        assert!(found.is_some(), "should retrieve matching state");
        assert_eq!(found.unwrap().action_sequence.len(), 3);
    }

    #[test]
    fn test_retrieve_returns_none_for_unknown_state() {
        let mut compiler = OnlineSkillCompiler::new();
        let state = make_binary_vsa(10);
        let action = make_binary_vsa(20);
        let outcome = make_binary_vsa(30);
        for _ in 0..3 {
            compiler.observe(&state, &action, &outcome, 1.0);
        }
        compiler.compile();
        let unknown = make_binary_vsa(99);
        let found = compiler.retrieve(&unknown);
        assert!(found.is_none(), "should return None for dissimilar state");
    }

    #[tokio::test]
    async fn test_execute_returns_action_sequence_and_updates_last_used() {
        let mut compiler = OnlineSkillCompiler::new();
        let state = make_binary_vsa(10);
        let action = make_binary_vsa(20);
        let outcome = make_binary_vsa(30);
        for _ in 0..3 {
            compiler.observe(&state, &action, &outcome, 1.0);
        }
        compiler.compile();
        let before = compiler.skills[0].last_used_ns;
        tokio::time::sleep(std::time::Duration::from_micros(1)).await;
        let (chunk, seq) = compiler.execute(&state).expect("should execute");
        assert_eq!(seq.len(), 3);
        assert!(chunk.last_used_ns > before, "last_used_ns should update");
    }

    #[test]
    fn test_decay_reduces_strength_over_time() {
        let mut compiler = OnlineSkillCompiler::new();
        for _ in 0..3 {
            compiler.observe(
                &make_binary_vsa(10),
                &make_binary_vsa(20),
                &make_binary_vsa(30),
                1.0,
            );
        }
        compiler.compile();
        assert!((compiler.skills[0].strength - 1.0).abs() < 1e-6);
        let future = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            + 86400_000_000_000 * 5; // 5 days later
        compiler.decay(future);
        assert!(compiler.skills[0].strength < 1.0, "strength should decay");
    }

    #[test]
    fn test_decay_removes_weak_skills() {
        let mut compiler = OnlineSkillCompiler::new();
        compiler.decay_rate = 1.0;
        for _ in 0..3 {
            compiler.observe(
                &make_binary_vsa(10),
                &make_binary_vsa(20),
                &make_binary_vsa(30),
                1.0,
            );
        }
        compiler.compile();
        let future = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            + 86400_000_000_000 * 2;
        compiler.decay(future);
        assert_eq!(
            compiler.skill_count(),
            0,
            "strength < 0.1 should be removed"
        );
    }

    #[test]
    fn test_compile_returns_new_chunk_ids() {
        let mut compiler = OnlineSkillCompiler::new();
        let ids = compiler.compile();
        assert!(ids.is_empty());
        for _ in 0..3 {
            compiler.observe(
                &make_binary_vsa(10),
                &make_binary_vsa(20),
                &make_binary_vsa(30),
                1.0,
            );
        }
        let ids = compiler.compile();
        assert_eq!(ids.len(), 1);
        assert!(ids[0] > 0);
    }

    #[test]
    fn test_clear_traces() {
        let mut compiler = OnlineSkillCompiler::new();
        for _ in 0..3 {
            compiler.observe(
                &make_binary_vsa(10),
                &make_binary_vsa(20),
                &make_binary_vsa(30),
                1.0,
            );
        }
        assert_eq!(compiler.trace_count(), 3);
        compiler.clear_traces();
        assert_eq!(compiler.trace_count(), 0);
    }

    #[test]
    fn test_similar_but_not_identical_traces_cluster() {
        let mut compiler = OnlineSkillCompiler::new();
        compiler.similarity_threshold = 0.90;
        let base = make_binary_vsa(10);
        let action = make_binary_vsa(20);
        let outcome = make_binary_vsa(30);

        for _ in 0..3 {
            compiler.observe(&base, &action, &outcome, 1.0);
        }
        let ids = compiler.compile();
        assert_eq!(ids.len(), 1, "identical traces cluster");
    }

    #[test]
    fn test_skill_count_and_trace_count() {
        let mut compiler = OnlineSkillCompiler::new();
        assert_eq!(compiler.skill_count(), 0);
        assert_eq!(compiler.trace_count(), 0);
        for _ in 0..3 {
            compiler.observe(
                &make_binary_vsa(10),
                &make_binary_vsa(20),
                &make_binary_vsa(30),
                1.0,
            );
        }
        assert_eq!(compiler.trace_count(), 3);
        compiler.compile();
        assert_eq!(compiler.skill_count(), 1);
    }

    #[test]
    fn test_execute_returns_none_for_unknown() {
        let mut compiler = OnlineSkillCompiler::new();
        for _ in 0..3 {
            compiler.observe(
                &make_binary_vsa(10),
                &make_binary_vsa(20),
                &make_binary_vsa(30),
                1.0,
            );
        }
        compiler.compile();
        let unknown = make_binary_vsa(99);
        let result = compiler.execute(&unknown);
        assert!(result.is_none());
    }

    #[test]
    fn test_default_values() {
        let compiler = OnlineSkillCompiler::new();
        assert_eq!(compiler.max_traces, 200);
        assert_eq!(compiler.max_skills, 50);
        assert_eq!(compiler.min_sequence_reps, 3);
        assert!((compiler.similarity_threshold - 0.85).abs() < 1e-6);
        assert!((compiler.decay_rate - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_trace_observe_eviction() {
        let mut compiler = OnlineSkillCompiler::new();
        compiler.max_traces = 2;
        compiler.observe(
            &make_binary_vsa(1),
            &make_binary_vsa(2),
            &make_binary_vsa(3),
            1.0,
        );
        compiler.observe(
            &make_binary_vsa(4),
            &make_binary_vsa(5),
            &make_binary_vsa(6),
            1.0,
        );
        compiler.observe(
            &make_binary_vsa(7),
            &make_binary_vsa(8),
            &make_binary_vsa(9),
            1.0,
        );
        assert_eq!(compiler.trace_count(), 2, "should evict oldest trace");
    }
}
