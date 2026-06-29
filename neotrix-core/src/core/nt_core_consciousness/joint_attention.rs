use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_time::unix_now_ms;
use std::collections::VecDeque;

/// VSA representation of what another agent is attending to.
#[derive(Debug, Clone)]
pub struct AttendedTarget {
    pub target_vsa: Vec<u8>,
    pub inferred_focus: Vec<u8>,
    pub attention_confidence: f64,
    pub timestamp: u64,
    pub agent_id: String,
    pub is_shared: bool,
}

/// Current coordination state between self and others.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoordinationState {
    Solitary,
    MutualGaze,
    JointFocus,
    CommonGround,
}

/// VSA representation of shared understanding with another agent.
#[derive(Debug, Clone)]
pub struct CommonGround {
    pub topic_vsa: Vec<u8>,
    pub confidence: f64,
    pub established_at: u64,
    pub last_reinforced: u64,
}

/// Tracks joint attention between self and other agents using VSA representations.
#[derive(Debug, Clone)]
pub struct JointAttentionModule {
    pub attention_targets: VecDeque<AttendedTarget>,
    pub self_focus: Option<Vec<u8>>,
    pub coordination: CoordinationState,
    pub common_grounds: Vec<CommonGround>,
    pub max_targets: usize,
    pub gaze_similarity_threshold: f64,
    pub joint_focus_threshold: f64,
    pub common_ground_decay_ms: u64,
    pub cycle_count: u64,
    pub shared_intention_bias: f64,
}

impl JointAttentionModule {
    pub fn new() -> Self {
        Self {
            attention_targets: VecDeque::with_capacity(100),
            self_focus: None,
            coordination: CoordinationState::Solitary,
            common_grounds: Vec::new(),
            max_targets: 100,
            gaze_similarity_threshold: 0.5,
            joint_focus_threshold: 0.7,
            common_ground_decay_ms: 60_000,
            cycle_count: 0,
            shared_intention_bias: 0.6,
        }
    }

    /// Infer what another agent is attending to from their attention cues.
    /// Stores the target and returns the inferred focus VSA.
    pub fn infer_other_attention(
        &mut self,
        attention_cue: Vec<u8>,
        agent_id: &str,
        confidence: f64,
    ) -> Vec<u8> {
        let inferred_focus = attention_cue.clone();

        let is_shared = self
            .self_focus
            .as_ref()
            .map(|sf| {
                QuantizedVSA::similarity(sf, &inferred_focus) > self.gaze_similarity_threshold
            })
            .unwrap_or(false);

        let target = AttendedTarget {
            target_vsa: attention_cue,
            inferred_focus: inferred_focus.clone(),
            attention_confidence: confidence,
            timestamp: unix_now_ms(),
            agent_id: agent_id.to_string(),
            is_shared,
        };

        self.attention_targets.push_back(target);
        if self.attention_targets.len() > self.max_targets {
            self.attention_targets.pop_front();
        }

        inferred_focus
    }

    /// Set own attention focus.
    pub fn set_self_focus(&mut self, focus_vsa: Vec<u8>) {
        self.self_focus = Some(focus_vsa);
    }

    /// Evaluate current coordination state by comparing self_focus with known targets.
    pub fn update_coordination(&mut self) -> CoordinationState {
        self.cycle_count += 1;

        let self_focus = match &self.self_focus {
            Some(f) => f.clone(),
            None => {
                self.coordination = CoordinationState::Solitary;
                return CoordinationState::Solitary;
            }
        };

        for target in &self.attention_targets {
            let sim = QuantizedVSA::similarity(&self_focus, &target.inferred_focus);
            if sim > self.joint_focus_threshold {
                self.coordination = CoordinationState::JointFocus;
                return self.coordination;
            }
        }

        for target in &self.attention_targets {
            if target.is_shared {
                self.coordination = CoordinationState::MutualGaze;
                return self.coordination;
            }
        }

        if !self.common_grounds.is_empty() {
            self.coordination = CoordinationState::CommonGround;
            return self.coordination;
        }

        self.coordination = CoordinationState::Solitary;
        CoordinationState::Solitary
    }

    /// Add or reinforce a common ground topic.
    pub fn establish_common_ground(&mut self, topic_vsa: Vec<u8>, confidence: f64) {
        let now = unix_now_ms();
        if let Some(cg) = self.common_grounds.iter_mut().find(|cg| {
            QuantizedVSA::similarity(&cg.topic_vsa, &topic_vsa) > self.gaze_similarity_threshold
        }) {
            cg.confidence = cg.confidence.max(confidence);
            cg.last_reinforced = now;
        } else {
            self.common_grounds.push(CommonGround {
                topic_vsa,
                confidence,
                established_at: now,
                last_reinforced: now,
            });
        }
    }

    /// Find a common ground by VSA similarity to the given topic.
    pub fn check_common_ground(&self, topic_vsa: &[u8]) -> Option<&CommonGround> {
        self.common_grounds.iter().find(|cg| {
            QuantizedVSA::similarity(&cg.topic_vsa, topic_vsa) > self.gaze_similarity_threshold
        })
    }

    /// Ratio of shared targets to total.
    pub fn shared_attention_score(&self) -> f64 {
        if self.attention_targets.is_empty() {
            return 0.0;
        }
        let shared = self
            .attention_targets
            .iter()
            .filter(|t| t.is_shared)
            .count();
        shared as f64 / self.attention_targets.len() as f64
    }

    /// Unique agent IDs from all tracked targets.
    pub fn attended_agents(&self) -> Vec<String> {
        let mut agents: Vec<String> = self
            .attention_targets
            .iter()
            .map(|t| t.agent_id.clone())
            .collect();
        agents.sort();
        agents.dedup();
        agents
    }

    /// Remove targets whose timestamp is older than decay window.
    pub fn clear_old_targets(&mut self) {
        let now = unix_now_ms();
        self.attention_targets
            .retain(|t| now - t.timestamp <= self.common_ground_decay_ms);
    }

    /// Reset all tracked state.
    pub fn reset(&mut self) {
        self.attention_targets.clear();
        self.self_focus = None;
        self.coordination = CoordinationState::Solitary;
        self.common_grounds.clear();
        self.cycle_count = 0;
    }
}

impl Default for JointAttentionModule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vsa_dim() -> usize {
        4096
    }

    /// Create a vector with controlled similarity to `base`.
    /// Flips every `interval`-th bit. similarity ≈ 1 - 1/interval.
    fn flip_every(base: &[u8], interval: usize) -> Vec<u8> {
        base.iter()
            .enumerate()
            .map(|(i, &b)| if i % interval == 0 { b ^ 1 } else { b })
            .collect()
    }

    #[test]
    fn test_new_jam_defaults() {
        let jam = JointAttentionModule::new();
        assert!(jam.attention_targets.is_empty());
        assert!(jam.self_focus.is_none());
        assert_eq!(jam.coordination, CoordinationState::Solitary);
        assert!(jam.common_grounds.is_empty());
        assert_eq!(jam.max_targets, 100);
        assert!((jam.gaze_similarity_threshold - 0.5).abs() < 1e-9);
        assert!((jam.joint_focus_threshold - 0.7).abs() < 1e-9);
        assert_eq!(jam.common_ground_decay_ms, 60_000);
        assert_eq!(jam.cycle_count, 0);
        assert!((jam.shared_intention_bias - 0.6).abs() < 1e-9);
    }

    #[test]
    fn test_infer_other_attention_stores_target() {
        let mut jam = JointAttentionModule::new();
        let cue = QuantizedVSA::seeded_random(42, vsa_dim());
        let inferred = jam.infer_other_attention(cue.clone(), "agent_1", 0.8);

        assert_eq!(jam.attention_targets.len(), 1);
        let target = &jam.attention_targets[0];
        assert_eq!(target.agent_id, "agent_1");
        assert!((target.attention_confidence - 0.8).abs() < 1e-9);
        assert_eq!(inferred, cue);
    }

    #[test]
    fn test_set_self_focus() {
        let mut jam = JointAttentionModule::new();
        let focus = QuantizedVSA::seeded_random(99, vsa_dim());
        jam.set_self_focus(focus.clone());
        assert!(jam.self_focus.is_some());
        assert_eq!(jam.self_focus.as_ref().unwrap(), &focus);
    }

    #[test]
    fn test_coordination_solitary() {
        let mut jam = JointAttentionModule::new();
        let state = jam.update_coordination();
        assert_eq!(state, CoordinationState::Solitary);
    }

    #[test]
    fn test_coordination_mutual_gaze() {
        let mut jam = JointAttentionModule::new();
        let base = QuantizedVSA::seeded_random(101, vsa_dim());

        // Similar focus (flip every 3rd bit → ~67% similarity, above gaze_threshold 0.5 but below joint 0.7)
        let similar = flip_every(&base, 3);
        jam.set_self_focus(base);
        jam.infer_other_attention(similar, "agent_x", 0.6);

        let state = jam.update_coordination();
        assert_eq!(state, CoordinationState::MutualGaze);
    }

    #[test]
    fn test_coordination_joint_focus() {
        let mut jam = JointAttentionModule::new();
        let base = QuantizedVSA::seeded_random(202, vsa_dim());

        // Very similar focus (flip every 10th bit → ~90% similarity, above joint threshold 0.7)
        let very_similar = flip_every(&base, 10);
        jam.set_self_focus(base);
        jam.infer_other_attention(very_similar, "agent_y", 0.9);

        let state = jam.update_coordination();
        assert_eq!(state, CoordinationState::JointFocus);
    }

    #[test]
    fn test_establish_common_ground() {
        let mut jam = JointAttentionModule::new();
        let topic = QuantizedVSA::seeded_random(303, vsa_dim());
        jam.establish_common_ground(topic.clone(), 0.75);

        assert_eq!(jam.common_grounds.len(), 1);
        assert!((jam.common_grounds[0].confidence - 0.75).abs() < 1e-9);

        // Reinforce with higher confidence
        jam.establish_common_ground(topic.clone(), 0.9);
        assert_eq!(jam.common_grounds.len(), 1);
        assert!((jam.common_grounds[0].confidence - 0.9).abs() < 1e-9);
    }

    #[test]
    fn test_check_common_ground_by_similarity() {
        let mut jam = JointAttentionModule::new();
        let topic = QuantizedVSA::seeded_random(404, vsa_dim());
        jam.establish_common_ground(topic.clone(), 0.8);

        // Query with similar VSA (flip few bits → still above threshold)
        let similar = flip_every(&topic, 10);
        let found = jam.check_common_ground(&similar);
        assert!(found.is_some());
        assert!((found.unwrap().confidence - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_check_common_ground_not_found() {
        let jam = JointAttentionModule::new();
        let topic = QuantizedVSA::seeded_random(505, vsa_dim());
        let found = jam.check_common_ground(&topic);
        assert!(found.is_none());
    }

    #[test]
    fn test_shared_attention_score() {
        let mut jam = JointAttentionModule::new();
        let focus = QuantizedVSA::seeded_random(606, vsa_dim());
        jam.set_self_focus(focus.clone());

        // This target will be shared (similar to self_focus)
        let shared_cue = flip_every(&focus, 3);
        jam.infer_other_attention(shared_cue, "alice", 0.7);

        // This target will NOT be shared (different random vector)
        let not_shared = QuantizedVSA::seeded_random(999, vsa_dim());
        // We need to add a target with is_shared=false.
        // Create one directly by modifying the flag on an added target
        jam.infer_other_attention(not_shared, "bob", 0.5);
        // The second target may coincidentally match the self focus. Force it off.
        if let Some(t) = jam.attention_targets.iter_mut().last() {
            t.is_shared = false;
        }

        let score = jam.shared_attention_score();
        assert!((score - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_attended_agents() {
        let mut jam = JointAttentionModule::new();
        let v1 = QuantizedVSA::seeded_random(1, vsa_dim());
        let v2 = QuantizedVSA::seeded_random(2, vsa_dim());
        let v3 = QuantizedVSA::seeded_random(3, vsa_dim());

        jam.infer_other_attention(v1, "alice", 0.5);
        jam.infer_other_attention(v2, "bob", 0.5);
        jam.infer_other_attention(v3, "alice", 0.5);

        let agents = jam.attended_agents();
        assert_eq!(agents.len(), 2);
        assert!(agents.contains(&"alice".to_string()));
        assert!(agents.contains(&"bob".to_string()));
    }

    #[test]
    fn test_clear_old_targets() {
        let mut jam = JointAttentionModule::new();
        // Set decay to a very small window so old timestamps trigger removal
        jam.common_ground_decay_ms = 1;

        let v = QuantizedVSA::seeded_random(707, vsa_dim());
        jam.infer_other_attention(v.clone(), "agent_z", 0.5);

        // Set timestamp to a very old value
        if let Some(t) = jam.attention_targets.iter_mut().last() {
            t.timestamp = 0;
        }

        jam.clear_old_targets();
        assert!(jam.attention_targets.is_empty());
    }

    #[test]
    fn test_reset() {
        let mut jam = JointAttentionModule::new();
        let v = QuantizedVSA::seeded_random(808, vsa_dim());
        jam.set_self_focus(v.clone());
        jam.infer_other_attention(v, "agent", 0.5);
        jam.establish_common_ground(QuantizedVSA::seeded_random(909, vsa_dim()), 0.5);
        jam.cycle_count = 10;
        jam.coordination = CoordinationState::JointFocus;

        jam.reset();
        assert!(jam.attention_targets.is_empty());
        assert!(jam.self_focus.is_none());
        assert_eq!(jam.coordination, CoordinationState::Solitary);
        assert!(jam.common_grounds.is_empty());
        assert_eq!(jam.cycle_count, 0);
    }

    #[test]
    fn test_coordination_state_cycles() {
        let mut jam = JointAttentionModule::new();
        let base = QuantizedVSA::seeded_random(111, vsa_dim());

        // 1. No focus → Solitary
        assert_eq!(jam.update_coordination(), CoordinationState::Solitary);

        // 2. Focus set but no targets → Solitary still
        jam.set_self_focus(base.clone());
        assert_eq!(jam.update_coordination(), CoordinationState::Solitary);

        // 3. Add a moderately similar target → MutualGaze
        let similar = flip_every(&base, 3);
        jam.infer_other_attention(similar, "other", 0.6);
        assert_eq!(jam.update_coordination(), CoordinationState::MutualGaze);

        // 4. Same target, JointFocus takes priority (similarity > 0.7)
        // Replace target with a very similar one
        jam.reset();
        jam.set_self_focus(base.clone());
        let very_similar = flip_every(&base, 10);
        jam.infer_other_attention(very_similar, "other", 0.9);
        assert_eq!(jam.update_coordination(), CoordinationState::JointFocus);
    }
}
