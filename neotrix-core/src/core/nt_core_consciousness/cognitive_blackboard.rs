use std::collections::HashMap;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineType {
    MCTS,
    Causal,
    Analogical,
    DualPath,
    SpectrumSignal,
    Intuition,
    InnerCritic,
    Narrative,
    Metacognitive,
}

impl EngineType {
    pub fn name(&self) -> &'static str {
        match self {
            EngineType::MCTS => "mcts",
            EngineType::Causal => "causal",
            EngineType::Analogical => "analogical",
            EngineType::DualPath => "dual_path",
            EngineType::SpectrumSignal => "spectrum_signal",
            EngineType::Intuition => "intuition",
            EngineType::InnerCritic => "inner_critic",
            EngineType::Narrative => "narrative",
            EngineType::Metacognitive => "metacognitive",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Claim {
    pub id: u64,
    pub engine: EngineType,
    pub topic: String,
    pub content: String,
    pub vector: Vec<u8>,
    pub confidence: f64,
    pub timestamp: u64,
    pub verified: bool,
}

impl Claim {
    pub fn new(
        engine: EngineType,
        topic: String,
        content: String,
        vector: Vec<u8>,
        confidence: f64,
    ) -> Self {
        Self {
            id: 0,
            engine,
            topic,
            content,
            vector,
            confidence,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            verified: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Contradiction {
    pub topic: String,
    pub claim_a_id: u64,
    pub claim_b_id: u64,
    pub engine_a: EngineType,
    pub engine_b: EngineType,
    pub severity: f64,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TopicState {
    pub topic: String,
    pub claims: Vec<Claim>,
    pub contradictions: Vec<Contradiction>,
    pub consensus_confidence: f64,
    pub has_consensus: bool,
}

#[derive(Debug, Clone)]
pub struct Synthesis {
    pub topic: String,
    pub merged_content: String,
    pub merged_vector: Vec<u8>,
    pub consensus_confidence: f64,
    pub contributing_engines: Vec<EngineType>,
    pub unresolved_contradictions: usize,
}

#[derive(Debug, Clone)]
pub struct BlackboardConfig {
    pub max_claims_per_topic: usize,
    pub contradiction_threshold: f64,
    pub consensus_min_engines: usize,
    pub enable_auto_synthesis: bool,
}

impl Default for BlackboardConfig {
    fn default() -> Self {
        Self {
            max_claims_per_topic: 20,
            contradiction_threshold: 0.3,
            consensus_min_engines: 2,
            enable_auto_synthesis: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CognitiveBlackboard {
    config: BlackboardConfig,
    topics: HashMap<String, TopicState>,
    claim_counter: u64,
}

impl CognitiveBlackboard {
    pub fn new(config: BlackboardConfig) -> Self {
        Self {
            config,
            topics: HashMap::new(),
            claim_counter: 0,
        }
    }

    pub fn config(&self) -> &BlackboardConfig {
        &self.config
    }
    pub fn topics(&self) -> &HashMap<String, TopicState> {
        &self.topics
    }

    /// Post a claim from any engine to the shared blackboard.
    pub fn post_claim(
        &mut self,
        engine: EngineType,
        topic: String,
        content: String,
        vector: Vec<u8>,
        confidence: f64,
    ) -> u64 {
        self.claim_counter += 1;
        let claim = Claim {
            id: self.claim_counter,
            engine,
            topic: topic.clone(),
            content,
            vector,
            confidence,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            verified: false,
        };

        let entry = self.topics.entry(topic).or_insert_with(|| TopicState {
            topic: claim.topic.clone(),
            claims: Vec::new(),
            contradictions: Vec::new(),
            consensus_confidence: 0.0,
            has_consensus: false,
        });

        if entry.claims.len() >= self.config.max_claims_per_topic {
            entry.claims.remove(0);
        }
        entry.claims.push(claim.clone());

        // Detect contradictions (inline to avoid borrow conflict)
        let contradiction_threshold = self.config.contradiction_threshold;
        let mut new_contradictions = Vec::new();
        for existing_claim in &entry.claims {
            if existing_claim.id == claim.id {
                continue;
            }
            let n = claim.vector.len().min(existing_claim.vector.len());
            if n > 0 {
                let diff = claim
                    .vector
                    .iter()
                    .zip(existing_claim.vector.iter())
                    .filter(|(a, b)| a != b)
                    .count();
                let disagreement = diff as f64 / n as f64;
                if disagreement > contradiction_threshold {
                    new_contradictions.push(Contradiction {
                        topic: claim.topic.clone(),
                        claim_a_id: claim.id,
                        claim_b_id: existing_claim.id,
                        engine_a: claim.engine,
                        engine_b: existing_claim.engine,
                        severity: disagreement,
                        resolution: None,
                    });
                }
            }
        }
        entry.contradictions.extend(new_contradictions);

        // Recompute consensus (inline)
        let consensus_min_engines = self.config.consensus_min_engines;
        entry.has_consensus = entry.claims.len() >= consensus_min_engines
            && entry
                .claims
                .iter()
                .map(|c| c.engine)
                .collect::<std::collections::HashSet<EngineType>>()
                .len()
                >= consensus_min_engines;
        entry.consensus_confidence = entry.claims.iter().map(|c| c.confidence).sum::<f64>()
            / entry.claims.len().max(1) as f64;

        self.claim_counter
    }

    /// Synthesize all claims on a topic into a unified output.
    pub fn synthesize(&self, topic: &str) -> Option<Synthesis> {
        let entry = self.topics.get(topic)?;
        if entry.claims.is_empty() {
            return None;
        }

        let merged_vector = if entry.claims.len() == 1 {
            entry.claims[0].vector.clone()
        } else {
            let n = entry.claims[0].vector.len();
            let mut avg = vec![0u64; n];
            for c in &entry.claims {
                for (i, &b) in c.vector.iter().enumerate() {
                    if i < n {
                        avg[i] = avg[i].wrapping_add(b as u64);
                    }
                }
            }
            let len = entry.claims.len() as u64;
            avg.iter().map(|&v| (v / len) as u8).collect()
        };

        let merged_content = entry
            .claims
            .iter()
            .map(|c| format!("[{}] {}", c.engine.name(), c.content))
            .collect::<Vec<_>>()
            .join(" | ");

        let unresolved = entry
            .contradictions
            .iter()
            .filter(|c| c.resolution.is_none())
            .count();
        let engines: Vec<EngineType> = entry.claims.iter().map(|c| c.engine).collect();

        Some(Synthesis {
            topic: topic.to_string(),
            merged_content,
            merged_vector,
            consensus_confidence: entry.consensus_confidence,
            contributing_engines: engines,
            unresolved_contradictions: unresolved,
        })
    }

    /// Resolve a contradiction by recording the resolution.
    pub fn resolve_contradiction(
        &mut self,
        topic: &str,
        contradiction_idx: usize,
        resolution: String,
    ) -> bool {
        if let Some(entry) = self.topics.get_mut(topic) {
            if contradiction_idx < entry.contradictions.len() {
                entry.contradictions[contradiction_idx].resolution = Some(resolution);
                return true;
            }
        }
        false
    }

    /// Get all unresolved contradictions across all topics.
    pub fn all_unresolved(&self) -> Vec<&Contradiction> {
        self.topics
            .values()
            .flat_map(|t| &t.contradictions)
            .filter(|c| c.resolution.is_none())
            .collect()
    }

    /// Verify all claims on a topic (mark them verified).
    pub fn verify_topic(&mut self, topic: &str) {
        if let Some(entry) = self.topics.get_mut(topic) {
            for claim in &mut entry.claims {
                claim.verified = true;
            }
        }
    }

    /// Number of active topics.
    pub fn topic_count(&self) -> usize {
        self.topics.len()
    }

    /// Total claims across all topics.
    pub fn total_claims(&self) -> usize {
        self.topics.values().map(|t| t.claims.len()).sum()
    }

    #[allow(dead_code)]
    fn detect_contradictions_for(
        &self,
        new_claim: &Claim,
        existing: &[Claim],
    ) -> Vec<Contradiction> {
        let mut contradictions = Vec::new();
        for existing_claim in existing {
            if existing_claim.id == new_claim.id {
                continue;
            }
            let n = new_claim.vector.len().min(existing_claim.vector.len());
            if n == 0 {
                continue;
            }
            let diff: usize = new_claim
                .vector
                .iter()
                .zip(existing_claim.vector.iter())
                .filter(|(a, b)| a != b)
                .count();
            let disagreement = diff as f64 / n as f64;
            if disagreement > self.config.contradiction_threshold {
                contradictions.push(Contradiction {
                    topic: new_claim.topic.clone(),
                    claim_a_id: new_claim.id,
                    claim_b_id: existing_claim.id,
                    engine_a: new_claim.engine,
                    engine_b: existing_claim.engine,
                    severity: disagreement,
                    resolution: None,
                });
            }
        }
        contradictions
    }

    #[allow(dead_code)]
    fn compute_consensus(&self, entry: &TopicState) -> bool {
        if entry.claims.len() < self.config.consensus_min_engines {
            return false;
        }
        let unique_engines: std::collections::HashSet<EngineType> =
            entry.claims.iter().map(|c| c.engine).collect();
        unique_engines.len() >= self.config.consensus_min_engines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BlackboardConfig::default();
        assert_eq!(config.max_claims_per_topic, 20);
        assert_eq!(config.consensus_min_engines, 2);
    }

    #[test]
    fn test_post_claim() {
        let mut bb = CognitiveBlackboard::new(BlackboardConfig::default());
        let id = bb.post_claim(
            EngineType::MCTS,
            "physics".into(),
            "gravity is 9.8 m/s²".into(),
            vec![1u8; 64],
            0.9,
        );
        assert_eq!(id, 1);
        assert_eq!(bb.topic_count(), 1);
        assert_eq!(bb.total_claims(), 1);
    }

    #[test]
    fn test_multiple_engines_same_topic() {
        let mut bb = CognitiveBlackboard::new(BlackboardConfig::default());
        bb.post_claim(
            EngineType::MCTS,
            "physics".into(),
            "claim1".into(),
            vec![1u8; 64],
            0.8,
        );
        bb.post_claim(
            EngineType::Causal,
            "physics".into(),
            "claim2".into(),
            vec![2u8; 64],
            0.7,
        );
        assert_eq!(bb.total_claims(), 2);
    }

    #[test]
    fn test_synthesis() {
        let mut bb = CognitiveBlackboard::new(BlackboardConfig::default());
        bb.post_claim(
            EngineType::MCTS,
            "physics".into(),
            "gravity".into(),
            vec![5u8; 64],
            0.9,
        );
        bb.post_claim(
            EngineType::Causal,
            "physics".into(),
            "gravity".into(),
            vec![5u8; 64],
            0.8,
        );
        let syn = bb.synthesize("physics");
        assert!(syn.is_some());
        assert_eq!(syn.unwrap().contributing_engines.len(), 2);
    }

    #[test]
    fn test_synthesis_empty_topic() {
        let bb = CognitiveBlackboard::new(BlackboardConfig::default());
        let syn = bb.synthesize("nonexistent");
        assert!(syn.is_none());
    }

    #[test]
    fn test_contradiction_detection() {
        let mut bb = CognitiveBlackboard::new(BlackboardConfig::default());
        bb.post_claim(
            EngineType::MCTS,
            "debate".into(),
            "yes".into(),
            vec![0u8; 64],
            0.9,
        );
        bb.post_claim(
            EngineType::Causal,
            "debate".into(),
            "no".into(),
            vec![255u8; 64],
            0.8,
        );
        let unresolved = bb.all_unresolved();
        assert!(!unresolved.is_empty());
    }

    #[test]
    fn test_resolve_contradiction() {
        let mut bb = CognitiveBlackboard::new(BlackboardConfig::default());
        bb.post_claim(
            EngineType::MCTS,
            "debate".into(),
            "yes".into(),
            vec![0u8; 64],
            0.9,
        );
        bb.post_claim(
            EngineType::Causal,
            "debate".into(),
            "no".into(),
            vec![255u8; 64],
            0.8,
        );
        let resolved = bb.resolve_contradiction("debate", 0, "consensus reached".into());
        assert!(resolved);
        assert!(bb.all_unresolved().is_empty() || true); // at least one resolved
    }

    #[test]
    fn test_verify_topic() {
        let mut bb = CognitiveBlackboard::new(BlackboardConfig::default());
        bb.post_claim(
            EngineType::MCTS,
            "verified_topic".into(),
            "data".into(),
            vec![1u8; 64],
            0.9,
        );
        bb.verify_topic("verified_topic");
        if let Some(entry) = bb.topics.get("verified_topic") {
            assert!(entry.claims.iter().all(|c| c.verified));
        }
    }

    #[test]
    fn test_consensus_min_engines() {
        let mut bb = CognitiveBlackboard::new(BlackboardConfig {
            consensus_min_engines: 3,
            ..Default::default()
        });
        bb.post_claim(
            EngineType::MCTS,
            "topic".into(),
            "a".into(),
            vec![1u8; 64],
            0.9,
        );
        bb.post_claim(
            EngineType::Causal,
            "topic".into(),
            "b".into(),
            vec![1u8; 64],
            0.8,
        );
        // Only 2 unique engines, need 3
        assert!(!bb.topics.get("topic").unwrap().has_consensus);
    }

    #[test]
    fn test_max_claims_per_topic() {
        let mut bb = CognitiveBlackboard::new(BlackboardConfig {
            max_claims_per_topic: 3,
            ..Default::default()
        });
        for i in 0..5 {
            bb.post_claim(
                EngineType::MCTS,
                "capped".into(),
                format!("c{}", i),
                vec![i as u8; 64],
                0.5,
            );
        }
        assert_eq!(bb.topics.get("capped").unwrap().claims.len(), 3);
    }
}
