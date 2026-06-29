use crate::types::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

const NOVELTY_THRESHOLD: f64 = 0.75;
const SENSE_BATCH: usize = 5;
const MAX_EXPLORED: usize = 50_000;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn topic_seed(name: &str) -> u64 {
    let mut h = 0x9e3779b97f4a7c15u64;
    for b in name.bytes() {
        h = h.wrapping_mul(6364136223846793005).wrapping_add(b as u64);
    }
    h
}

pub struct CrawlBridge {
    pub vsa: VsaLight,
    pub frontier: Vec<String>,
    pub explored: Vec<String>,
    pub explored_fingerprints: Vec<Vec<u8>>,
    pub knowledge_gaps: Vec<String>,
    pub total_explorations: u64,
    pub total_actuations: u64,
    pub last_exploration_ms: i64,
    pub errors: u64,
    pub kb_available: bool,
    pub info_density_cache: HashMap<String, f64>,
}

impl Default for CrawlBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl CrawlBridge {
    pub fn new() -> Self {
        Self {
            vsa: VsaLight::new(VSA_DIM),
            frontier: vec![
                "consciousness_phi_iit".into(),
                "hypercube_vsa_binding".into(),
                "negentropy_curiosity_drive".into(),
                "sleep_consolidation_dream".into(),
                "self_improving_pipeline".into(),
                "epistemic_self_model".into(),
                "cross_modal_alignment".into(),
                "theory_of_mind_bdi".into(),
                "physical_commonsense".into(),
                "goal_directed_execution".into(),
            ],
            explored: Vec::new(),
            explored_fingerprints: Vec::new(),
            knowledge_gaps: vec![
                "VSA_continuous_binding".into(),
                "negentropy_curvature_rl".into(),
                "sleep_scm_bridge".into(),
            ],
            total_explorations: 0,
            total_actuations: 0,
            last_exploration_ms: 0,
            errors: 0,
            kb_available: true,
            info_density_cache: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    fn pop_frontier(&mut self) -> Option<String> {
        let idx = self
            .frontier
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                let na = self.novelty_for(a);
                let nb = self.novelty_for(b);
                na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i);
        idx.map(|i| self.frontier.remove(i))
    }

    fn novelty_for(&self, topic: &str) -> f64 {
        let seed = topic_seed(topic);
        let fp = self.vsa.seeded_vector(seed);
        self.vsa
            .novelty(&self.explored_fingerprints, &fp, NOVELTY_THRESHOLD)
    }

    fn info_density_for(&self, topic: &str) -> f64 {
        self.info_density_cache
            .get(topic)
            .copied()
            .unwrap_or_else(|| {
                let base: f64 = match topic.len() {
                    0..=10 => 0.5,
                    11..=30 => 0.7,
                    _ => 0.9,
                };
                let gap_bonus: f64 = if self.knowledge_gaps.iter().any(|g| topic.contains(g)) {
                    0.2
                } else {
                    0.0
                };
                (base + gap_bonus).min(1.0)
            })
    }

    fn mark_explored(&mut self, topic: String) {
        let seed = topic_seed(&topic);
        let fp = self.vsa.seeded_vector(seed);
        self.explored.push(topic.clone());
        self.explored_fingerprints.push(fp);
        if self.explored.len() > MAX_EXPLORED {
            let drain_count = MAX_EXPLORED / 5;
            self.explored.drain(0..drain_count);
            self.explored_fingerprints.drain(0..drain_count);
        }
        self.frontier.retain(|t| t != &topic);
        self.total_explorations = self.total_explorations.wrapping_add(1);
        self.last_exploration_ms = now_ms();
    }

    fn discover_connections(&mut self, topic: &str) {
        let related: Vec<String> = match topic {
            t if t.contains("consciousness") || t.contains("phi") => {
                vec![
                    "integrated_info_theory".into(),
                    "gwt_attention".into(),
                    "qualia_space".into(),
                ]
            }
            t if t.contains("vsa") || t.contains("hypercube") => {
                vec![
                    "freq_binding".into(),
                    "spatial_ssp".into(),
                    "hadamard_hlb".into(),
                ]
            }
            t if t.contains("negentropy") || t.contains("curiosity") => {
                vec![
                    "curvature_rl".into(),
                    "intrinsic_motivation".into(),
                    "predictive_error".into(),
                ]
            }
            t if t.contains("sleep") || t.contains("dream") || t.contains("consolidation") => {
                vec![
                    "nrem_sequence_replay".into(),
                    "rem_cross_session".into(),
                    "synaptic_homeostasis".into(),
                ]
            }
            t if t.contains("epistemic") || t.contains("self") => {
                vec![
                    "meta_cognition_kpi".into(),
                    "confidence_calibration".into(),
                    "uncertainty_encoding".into(),
                ]
            }
            t if t.contains("cross_modal") || t.contains("alignment") => {
                vec![
                    "vsa_common_space".into(),
                    "modality_fusion".into(),
                    "semantic_gap".into(),
                ]
            }
            t if t.contains("theory_of_mind") || t.contains("bdi") => {
                vec![
                    "belief_tracking".into(),
                    "intention_recognition".into(),
                    "user_model_persistence".into(),
                ]
            }
            t if t.contains("physical") || t.contains("commonsense") => {
                vec![
                    "spatial_scene_encoding".into(),
                    "causal_reasoning".into(),
                    "affordance_detection".into(),
                ]
            }
            _ => vec!["general_knowledge_grounding".into()],
        };
        for r in related {
            let seed = topic_seed(&r);
            let fp = self.vsa.seeded_vector(seed);
            if self
                .vsa
                .novelty(&self.explored_fingerprints, &fp, NOVELTY_THRESHOLD)
                > 0.2
                && !self.frontier.contains(&r)
                && !self.explored.contains(&r)
            {
                self.frontier.push(r);
            }
        }
    }
}

impl ConsciousnessAbility for CrawlBridge {
    fn domain(&self) -> Domain {
        Domain::Crawl
    }

    fn sense(&mut self) -> Vec<VsaTagged> {
        let mut results = Vec::new();
        let now = now_ms();

        for topic in self.frontier.iter().take(SENSE_BATCH) {
            let novelty = self.novelty_for(topic);
            let info_density = self.info_density_for(topic);
            let neg = novelty * info_density;
            if neg < 0.05 {
                continue;
            }
            let seed = topic_seed(topic);
            let vector = self.vsa.seeded_vector(seed);
            let gap_bonus = if self.knowledge_gaps.iter().any(|g| topic.contains(g)) {
                0.15
            } else {
                0.0
            };
            results.push(VsaTagged {
                vector,
                origin: VsaOrigin::World(Sensory::PageContent),
                timestamp_ms: now,
                negentropy_contribution: neg + gap_bonus,
            });
        }

        let topic_count = self.frontier.len() as f64;
        let total_frontier_neg = self
            .frontier
            .iter()
            .map(|t| self.novelty_for(t) * self.info_density_for(t))
            .sum::<f64>();
        let avg_health_neg = if topic_count > 0.0 {
            total_frontier_neg / topic_count
        } else {
            0.0
        };
        let health_vector = self.vsa.seeded_vector((avg_health_neg * 1e6) as u64);
        results.push(VsaTagged {
            vector: health_vector,
            origin: VsaOrigin::Bridge(Domain::Crawl),
            timestamp_ms: now,
            negentropy_contribution: avg_health_neg * 0.1,
        });

        results
    }

    fn actuate(&mut self, intention: &IntentionVsa) -> Result<WorldEffect, String> {
        let start = now_ms();
        self.total_actuations = self.total_actuations.wrapping_add(1);

        if intention.domain != Domain::Crawl {
            return Err(format!(
                "CrawlBridge received non-crawl domain: {:?}",
                intention.domain
            ));
        }

        let action = intention.action.as_str();
        match action {
            "explore_topic" => {
                let topic = intention
                    .parameters
                    .get("topic")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if topic.is_empty() {
                    return Err("explore_topic requires 'topic' parameter".into());
                }
                if self.explored.contains(&topic) {
                    return Ok(WorldEffect {
                        domain: Domain::Crawl,
                        description: format!("Topic '{}' already explored, skipping", topic),
                        success: true,
                        latency_ms: (now_ms() - start) as u64,
                    });
                }
                self.discover_connections(&topic);
                self.mark_explored(topic.clone());
                Ok(WorldEffect {
                    domain: Domain::Crawl,
                    description: format!("Explored topic: {}", topic),
                    success: true,
                    latency_ms: (now_ms() - start) as u64,
                })
            }
            "seed_frontier" => {
                let domains = intention
                    .parameters
                    .get("domains")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let count = domains.len();
                for d in domains {
                    if let Some(t) = d.as_str() {
                        let topic = t.to_string();
                        let seed = topic_seed(&topic);
                        let fp = self.vsa.seeded_vector(seed);
                        if !self.explored.contains(&topic)
                            && !self.frontier.contains(&topic)
                            && self
                                .vsa
                                .novelty(&self.explored_fingerprints, &fp, NOVELTY_THRESHOLD)
                                > 0.1
                        {
                            self.frontier.push(topic);
                        }
                    }
                }
                Ok(WorldEffect {
                    domain: Domain::Crawl,
                    description: format!("Seeded frontier with {} new domains", count),
                    success: true,
                    latency_ms: (now_ms() - start) as u64,
                })
            }
            "deepen" => {
                let topic = intention
                    .parameters
                    .get("topic")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let depth = intention
                    .parameters
                    .get("depth")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1);
                if topic.is_empty() {
                    return Err("deepen requires 'topic' parameter".into());
                }
                if depth > 0 {
                    self.discover_connections(&topic);
                    self.mark_explored(topic.clone());
                    self.info_density_cache.insert(topic.clone(), 0.95);
                }
                Ok(WorldEffect {
                    domain: Domain::Crawl,
                    description: format!("Deepened topic '{}' to depth {}", topic, depth),
                    success: true,
                    latency_ms: (now_ms() - start) as u64,
                })
            }
            "analyze_gaps" => {
                let gap_count = self.knowledge_gaps.len();
                let matched: Vec<&String> = self
                    .frontier
                    .iter()
                    .filter(|t| self.knowledge_gaps.iter().any(|g| t.contains(g)))
                    .collect();
                Ok(WorldEffect {
                    domain: Domain::Crawl,
                    description: format!(
                        "{} gaps pending, {} frontier topics match gaps",
                        gap_count,
                        matched.len()
                    ),
                    success: true,
                    latency_ms: (now_ms() - start) as u64,
                })
            }
            "seed_from_gaps" => {
                let mut seeded = 0u64;
                for gap in &self.knowledge_gaps {
                    let seed = topic_seed(gap);
                    let fp = self.vsa.seeded_vector(seed);
                    if !self.explored.contains(gap)
                        && !self.frontier.contains(gap)
                        && self
                            .vsa
                            .novelty(&self.explored_fingerprints, &fp, NOVELTY_THRESHOLD)
                            > 0.1
                    {
                        self.frontier.push(gap.clone());
                        seeded += 1;
                    }
                }
                Ok(WorldEffect {
                    domain: Domain::Crawl,
                    description: format!("Seeded {} knowledge gaps into frontier", seeded),
                    success: true,
                    latency_ms: (now_ms() - start) as u64,
                })
            }
            _ => Err(format!("Unknown crawl action: {}", action)),
        }
    }

    fn curiosity_signals(&self) -> Vec<CuriositySignal> {
        let mut signals: Vec<CuriositySignal> = self
            .frontier
            .iter()
            .map(|topic| {
                let novelty = self.novelty_for(topic);
                let info_density = self.info_density_for(topic);
                CuriositySignal {
                    domain: Domain::Crawl,
                    query: topic.clone(),
                    novelty_estimate: novelty,
                    potential_negentropy: novelty * info_density * 0.8,
                }
            })
            .filter(|s| s.novelty_estimate > 0.3)
            .collect();
        signals.sort_by(|a, b| {
            b.novelty_estimate
                .partial_cmp(&a.novelty_estimate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        signals.truncate(10);
        signals
    }

    fn grace_mode(&self) -> GraceMode {
        GraceMode::SkipSilently
    }

    fn health(&self) -> BridgeHealth {
        BridgeHealth {
            domain: Domain::Crawl,
            available: self.kb_available,
            last_seen_ms: self.last_exploration_ms,
            error_count: self.errors,
            total_actuations: self.total_actuations,
        }
    }

    fn probe_available(&self) -> bool {
        true
    }

    fn negentropy_estimate(&self) -> f64 {
        let frontier_size = self.frontier.len() as f64;
        let avg_novelty = if self.frontier.is_empty() {
            0.0
        } else {
            self.frontier
                .iter()
                .map(|t| self.novelty_for(t))
                .sum::<f64>()
                / frontier_size
        };
        frontier_size * avg_novelty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain() {
        let b = CrawlBridge::new();
        assert_eq!(b.domain(), Domain::Crawl);
    }

    #[test]
    fn test_initial_state() {
        let b = CrawlBridge::new();
        assert!(!b.frontier.is_empty());
        assert!(b.explored.is_empty());
        assert_eq!(b.total_explorations, 0);
        assert_eq!(b.total_actuations, 0);
        assert!(b.last_exploration_ms == 0);
        assert!(b.kb_available);
    }

    #[test]
    fn test_probe_available() {
        let b = CrawlBridge::new();
        assert!(b.probe_available());
    }

    #[test]
    fn test_grace_mode() {
        let b = CrawlBridge::new();
        assert_eq!(b.grace_mode(), GraceMode::SkipSilently);
    }

    #[test]
    fn test_sense_returns_tagged_events() {
        let mut b = CrawlBridge::new();
        let events = b.sense();
        assert!(!events.is_empty());
        for e in &events {
            assert_eq!(e.vector.len(), VSA_DIM);
            assert!(e.negentropy_contribution >= 0.0);
            match &e.origin {
                VsaOrigin::World(Sensory::PageContent) => {}
                VsaOrigin::Bridge(Domain::Crawl) => {}
                _ => panic!("Unexpected origin: {:?}", e.origin),
            }
        }
    }

    #[test]
    fn test_actuate_explore_topic() {
        let mut b = CrawlBridge::new();
        let topic = "test_exploration".to_string();
        let intention = IntentionVsa {
            domain: Domain::Crawl,
            action: "explore_topic".into(),
            parameters: serde_json::json!({"topic": topic}),
            confidence: 1.0,
            urgency: 0.5,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(effect.success);
        assert!(b.explored.contains(&topic));
        assert_eq!(b.total_explorations, 1);
    }

    #[test]
    fn test_actuate_explore_topic_dedup() {
        let mut b = CrawlBridge::new();
        let topic = "dedup_test".to_string();
        let intention = IntentionVsa {
            domain: Domain::Crawl,
            action: "explore_topic".into(),
            parameters: serde_json::json!({"topic": topic}),
            confidence: 1.0,
            urgency: 0.5,
        };
        b.actuate(&intention).unwrap();
        let effect2 = b.actuate(&intention).unwrap();
        assert!(effect2.success);
        assert_eq!(b.total_explorations, 1);
    }

    #[test]
    fn test_actuate_explore_seeds_connections() {
        let mut b = CrawlBridge::new();
        let intention = IntentionVsa {
            domain: Domain::Crawl,
            action: "explore_topic".into(),
            parameters: serde_json::json!({"topic": "consciousness_phi_iit"}),
            confidence: 1.0,
            urgency: 0.5,
        };
        let before = b.frontier.len();
        b.actuate(&intention).unwrap();
        assert!(b.frontier.len() > before.saturating_sub(1));
    }

    #[test]
    fn test_actuate_seed_frontier() {
        let mut b = CrawlBridge::new();
        let intention = IntentionVsa {
            domain: Domain::Crawl,
            action: "seed_frontier".into(),
            parameters: serde_json::json!({"domains": ["new_topic_a", "new_topic_b"]}),
            confidence: 1.0,
            urgency: 0.5,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(effect.success);
        assert!(b.frontier.contains(&"new_topic_a".to_string()));
        assert!(b.frontier.contains(&"new_topic_b".to_string()));
    }

    #[test]
    fn test_actuate_deepen() {
        let mut b = CrawlBridge::new();
        let intention = IntentionVsa {
            domain: Domain::Crawl,
            action: "deepen".into(),
            parameters: serde_json::json!({"topic": "vsa_freq_binding", "depth": 3}),
            confidence: 1.0,
            urgency: 0.5,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(effect.success);
        assert!(b.explored.contains(&"vsa_freq_binding".to_string()));
    }

    #[test]
    fn test_actuate_analyze_gaps() {
        let mut b = CrawlBridge::new();
        let intention = IntentionVsa {
            domain: Domain::Crawl,
            action: "analyze_gaps".into(),
            parameters: serde_json::json!({}),
            confidence: 1.0,
            urgency: 0.3,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(effect.success);
        assert!(effect.description.contains("gaps"));
    }

    #[test]
    fn test_actuate_seed_from_gaps() {
        let mut b = CrawlBridge::new();
        let before = b.frontier.len();
        let intention = IntentionVsa {
            domain: Domain::Crawl,
            action: "seed_from_gaps".into(),
            parameters: serde_json::json!({}),
            confidence: 1.0,
            urgency: 0.7,
        };
        let effect = b.actuate(&intention).unwrap();
        assert!(effect.success);
        assert!(b.frontier.len() >= before);
    }

    #[test]
    fn test_actuate_wrong_domain() {
        let mut b = CrawlBridge::new();
        let intention = IntentionVsa {
            domain: Domain::System,
            action: "explore_topic".into(),
            parameters: serde_json::json!({"topic": "x"}),
            confidence: 1.0,
            urgency: 0.5,
        };
        assert!(b.actuate(&intention).is_err());
    }

    #[test]
    fn test_actuate_unknown_action() {
        let mut b = CrawlBridge::new();
        let intention = IntentionVsa {
            domain: Domain::Crawl,
            action: "nonexistent".into(),
            parameters: serde_json::json!({}),
            confidence: 1.0,
            urgency: 0.5,
        };
        assert!(b.actuate(&intention).is_err());
    }

    #[test]
    fn test_actuate_explore_topic_missing_param() {
        let mut b = CrawlBridge::new();
        let intention = IntentionVsa {
            domain: Domain::Crawl,
            action: "explore_topic".into(),
            parameters: serde_json::json!({}),
            confidence: 1.0,
            urgency: 0.5,
        };
        assert!(b.actuate(&intention).is_err());
    }

    #[test]
    fn test_curiosity_signals_returns_high_novelty() {
        let b = CrawlBridge::new();
        let signals = b.curiosity_signals();
        assert!(!signals.is_empty());
        for s in &signals {
            assert_eq!(s.domain, Domain::Crawl);
            assert!(s.novelty_estimate >= 0.0);
            assert!(s.potential_negentropy >= 0.0);
        }
    }

    #[test]
    fn test_curiosity_signals_sorted() {
        let b = CrawlBridge::new();
        let signals = b.curiosity_signals();
        for w in signals.windows(2) {
            assert!(w[0].novelty_estimate >= w[1].novelty_estimate);
        }
    }

    #[test]
    fn test_health_available() {
        let b = CrawlBridge::new();
        let h = b.health();
        assert_eq!(h.domain, Domain::Crawl);
        assert!(h.available);
    }

    #[test]
    fn test_health_tracks_actuations() {
        let mut b = CrawlBridge::new();
        let intention = IntentionVsa {
            domain: Domain::Crawl,
            action: "analyze_gaps".into(),
            parameters: serde_json::json!({}),
            confidence: 1.0,
            urgency: 0.3,
        };
        b.actuate(&intention).unwrap();
        let h = b.health();
        assert_eq!(h.total_actuations, 1);
    }

    #[test]
    fn test_negentropy_estimate_non_zero() {
        let b = CrawlBridge::new();
        let ne = b.negentropy_estimate();
        assert!(ne > 0.0);
    }

    #[test]
    fn test_negentropy_estimate_zero_when_empty() {
        let mut b = CrawlBridge::new();
        b.frontier.clear();
        assert_eq!(b.negentropy_estimate(), 0.0);
    }

    #[test]
    fn test_novelty_for_unexplored_is_high() {
        let b = CrawlBridge::new();
        let n = b.novelty_for("brand_new_topic");
        assert!(n > 0.5);
    }

    #[test]
    fn test_novelty_for_explored_is_low() {
        let mut b = CrawlBridge::new();
        let topic = "already_seen".to_string();
        let seed = topic_seed(&topic);
        let fp = b.vsa.seeded_vector(seed);
        b.explored.push(topic.clone());
        b.explored_fingerprints.push(fp);
        let n = b.novelty_for(&topic);
        assert!(n < 0.5);
    }

    #[test]
    fn test_mark_explored_removes_from_frontier() {
        let mut b = CrawlBridge::new();
        let topic = "consciousness_phi_iit".to_string();
        assert!(b.frontier.contains(&topic));
        b.mark_explored(topic.clone());
        assert!(!b.frontier.contains(&topic));
        assert!(b.explored.contains(&topic));
        assert_eq!(b.total_explorations, 1);
    }

    #[test]
    fn test_discover_connections_adds_related() {
        let mut b = CrawlBridge::new();
        let before = b.frontier.len();
        b.discover_connections("vsa_hypercube");
        assert!(b.frontier.len() >= before);
        assert!(b
            .frontier
            .iter()
            .any(|t| t.contains("freq_binding") || t.contains("hadamard")));
    }

    #[test]
    fn test_vsa_fingerprint_prevents_reexplore() {
        let mut b = CrawlBridge::new();
        let topic = "unique_topic".to_string();
        let seed = topic_seed(&topic);
        let fp = b.vsa.seeded_vector(seed);
        b.explored_fingerprints.push(fp);
        b.explored.push(topic.clone());
        let intention = IntentionVsa {
            domain: Domain::Crawl,
            action: "explore_topic".into(),
            parameters: serde_json::json!({"topic": topic}),
            confidence: 1.0,
            urgency: 0.5,
        };
        b.actuate(&intention).unwrap();
        assert_eq!(b.total_explorations, 0);
    }

    #[test]
    fn test_sense_respects_batch_size() {
        let mut b = CrawlBridge::new();
        b.frontier = (0..20).map(|i| format!("topic_{}", i)).collect();
        let events = b.sense();
        let content_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e.origin, VsaOrigin::World(Sensory::PageContent)))
            .collect();
        assert!(content_events.len() <= SENSE_BATCH);
    }

    #[test]
    fn test_sense_filters_low_negentropy() {
        let mut b = CrawlBridge::new();
        b.frontier.clear();
        b.frontier.push("xy".to_string());
        b.info_density_cache.insert("xy".to_string(), 0.01);
        let events = b.sense();
        let content_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e.origin, VsaOrigin::World(Sensory::PageContent)))
            .collect();
        assert!(content_events.is_empty());
    }

    #[test]
    fn test_info_density_cache_returns_cached() {
        let mut b = CrawlBridge::new();
        b.info_density_cache
            .insert("cached_topic".to_string(), 0.88);
        assert!((b.info_density_for("cached_topic") - 0.88).abs() < 1e-9);
    }

    #[test]
    fn test_info_density_gap_bonus() {
        let b = CrawlBridge::new();
        let gap_free = b.info_density_for("simple");
        let gap_hit = b.info_density_for("VSA_continuous_binding_demo");
        assert!(gap_hit >= gap_free);
    }

    #[test]
    fn test_pop_frontier_returns_highest_novelty() {
        let mut b = CrawlBridge::new();
        b.frontier = vec![
            "low_value".to_string(),
            "high_value_rich_topic_good".to_string(),
        ];
        let popped = b.pop_frontier();
        assert_eq!(popped.as_deref(), Some("high_value_rich_topic_good"));
    }

    #[test]
    fn test_seed_from_gaps_does_not_duplicate() {
        let mut b = CrawlBridge::new();
        b.frontier.clear();
        b.actuate(&IntentionVsa {
            domain: Domain::Crawl,
            action: "seed_from_gaps".into(),
            parameters: serde_json::json!({}),
            confidence: 1.0,
            urgency: 0.5,
        })
        .unwrap();
        let c1 = b.frontier.len();
        b.actuate(&IntentionVsa {
            domain: Domain::Crawl,
            action: "seed_from_gaps".into(),
            parameters: serde_json::json!({}),
            confidence: 1.0,
            urgency: 0.5,
        })
        .unwrap();
        assert_eq!(b.frontier.len(), c1);
    }
}
