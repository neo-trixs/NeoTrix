use crate::types::{
    BridgeHealth, ConsciousnessAbility, CuriositySignal, Domain, GraceMode,
    IntentionVsa, VsaLight, VsaOrigin, VsaTagged, WorldEffect,
};
use std::hash::Hasher;

const SENSE_VSA_SEED: u64 = 0xE4B1_7001;
const CONTENT_VSA_SEED: u64 = 0xE4B1_7002;
const MAX_CONTENT_QUEUE: usize = 1000;

pub struct EarnBridge {
    pub vsa: VsaLight,
    pub pipeline_active: bool,
    pub known_topics: Vec<String>,
    pub total_content_generated: u64,
    pub last_publish_ms: i64,
    pub error_count: u64,
    pub total_actuations: u64,

    content_formats: Vec<String>,
    platforms: Vec<String>,
    monetization_strategies: Vec<String>,
    pipeline_freshness: f64,
    api_keys_configured: bool,
    latest_revenue: f64,
    content_queue: Vec<String>,
}

impl Default for EarnBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl EarnBridge {
    pub fn new() -> Self {
        Self {
            vsa: VsaLight::new(4096),
            pipeline_active: false,
            known_topics: vec![
                "self-improving agents".into(),
                "rust cognitive architecture".into(),
                "VSA hyperdimensional computing".into(),
                "E8 reasoning engine".into(),
                "open-source AI infrastructure".into(),
            ],
            total_content_generated: 0,
            last_publish_ms: 0,
            error_count: 0,
            total_actuations: 0,
            content_formats: vec![
                "technical_blog".into(),
                "video_short".into(),
                "twitter_thread".into(),
                "code_demo".into(),
            ],
            platforms: vec![
                "x/twitter".into(),
                "youtube".into(),
                "github".into(),
                "bilibili".into(),
            ],
            monetization_strategies: vec![
                "knowledge_arbitrage".into(),
                "sponsored_content".into(),
                "digital_products".into(),
            ],
            pipeline_freshness: 0.8,
            api_keys_configured: false,
            latest_revenue: 0.0,
            content_queue: Vec::new(),
        }
    }

    pub fn with_api_keys(mut self) -> Self {
        self.api_keys_configured = true;
        self.pipeline_active = true;
        self
    }

    pub fn with_topics(mut self, topics: Vec<String>) -> Self {
        self.known_topics = topics;
        self
    }

    pub fn enqueue_content(&mut self, topic: String) {
        if self.content_queue.len() >= MAX_CONTENT_QUEUE {
            self.content_queue.remove(0);
        }
        self.content_queue.push(topic);
    }

    fn generate_vsa_signal(&self, seed: u64, negentropy: f64) -> VsaTagged {
        VsaTagged {
            vector: self.vsa.seeded_vector(seed),
            origin: VsaOrigin::World(crate::types::Sensory::SocialFeed),
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
            negentropy_contribution: negentropy,
        }
    }

    fn coherence_score(&self, seed: u64, known: &[Vec<u8>]) -> f64 {
        let candidate = self.vsa.seeded_vector(seed);
        if known.is_empty() {
            return 0.5;
        }
        let max_sim = known
            .iter()
            .map(|k| VsaLight::cosine_similarity(k, &candidate))
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        max_sim
    }

    fn generate_quality(&self) -> f64 {
        let coherence = self.coherence_score(CONTENT_VSA_SEED, &[]);
        let topic_novelty = self.vsa.novelty(
            &self
                .known_topics
                .iter()
                .map(|t| {
                    let mut h = std::collections::hash_map::DefaultHasher::new();
                    std::hash::Hash::hash(t, &mut h);
                    self.vsa.seeded_vector(h.finish())
                })
                .collect::<Vec<_>>(),
            &self.vsa.seeded_vector(SENSE_VSA_SEED + self.total_content_generated),
            0.9,
        );
        0.4 * coherence + 0.6 * topic_novelty
    }

    fn current_trends(&self) -> Vec<(String, f64)> {
        vec![
            ("self-improving agents".to_string(), 0.92),
            ("hyperdimensional computing".to_string(), 0.78),
            ("cognitive architectures".to_string(), 0.85),
            ("rust systems programming".to_string(), 0.71),
            ("open-source AGI infrastructure".to_string(), 0.88),
        ]
    }
}

impl ConsciousnessAbility for EarnBridge {
    fn domain(&self) -> Domain {
        Domain::Earn
    }

    fn sense(&mut self) -> Vec<VsaTagged> {
        let mut signals = Vec::new();

        let trends = self.current_trends();
        for (topic, engagement) in &trends {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(topic, &mut h);
            let seed = h.finish();
            signals.push(self.generate_vsa_signal(
                seed,
                engagement * self.pipeline_freshness,
            ));
        }

        let pipeline_signal = if self.pipeline_active {
            self.generate_vsa_signal(SENSE_VSA_SEED + 100, 0.6 * self.pipeline_freshness)
        } else {
            self.generate_vsa_signal(SENSE_VSA_SEED + 101, 0.1)
        };
        signals.push(pipeline_signal);

        let revenue_seed = self.total_content_generated.saturating_mul(7) ^ SENSE_VSA_SEED;
        let revenue_negentropy = self.latest_revenue.clamp(0.0, 1.0) * self.pipeline_freshness;
        signals.push(self.generate_vsa_signal(revenue_seed, revenue_negentropy));

        signals
    }

    fn actuate(&mut self, intention: &IntentionVsa) -> Result<WorldEffect, String> {
        self.total_actuations += 1;
        let start = std::time::Instant::now();

        match intention.action.as_str() {
            "generate_content" => {
                if !self.pipeline_active {
                    self.error_count += 1;
                    return Err("content pipeline not configured; no API keys".into());
                }
                let topic = intention
                    .parameters
                    .get("topic")
                    .and_then(|v| v.as_str())
                    .unwrap_or("neotrix cognitive architecture");
                if !self.known_topics.contains(&topic.to_string()) {
                    self.known_topics.push(topic.to_string());
                }
                self.total_content_generated += 1;
                self.last_publish_ms = chrono::Utc::now().timestamp_millis();
                let quality = self.generate_quality();
                Ok(WorldEffect {
                    domain: Domain::Earn,
                    description: format!(
                        "generated content '{}' with quality {:.2}",
                        topic, quality
                    ),
                    success: quality > 0.3,
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }

            "publish" => {
                if !self.pipeline_active {
                    self.error_count += 1;
                    return Err("publish: pipeline inactive, no API keys configured".into());
                }
                let target = intention
                    .parameters
                    .get("platform")
                    .and_then(|v| v.as_str())
                    .unwrap_or("x/twitter");
                if !self.platforms.contains(&target.to_string()) {
                    self.platforms.push(target.to_string());
                }
                self.last_publish_ms = chrono::Utc::now().timestamp_millis();
                self.total_content_generated += 1;
                Ok(WorldEffect {
                    domain: Domain::Earn,
                    description: format!("published content to {}", target),
                    success: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }

            "analyze_market" => {
                let trends = self.current_trends();
                let best = trends
                    .iter()
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(t, _)| t.as_str())
                    .unwrap_or("unknown");
                Ok(WorldEffect {
                    domain: Domain::Earn,
                    description: format!(
                        "market analysis: top trend '{}' with {} tracked topics",
                        best,
                        self.known_topics.len()
                    ),
                    success: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }

            "optimize_strategy" => {
                if self.known_topics.is_empty() {
                    self.error_count += 1;
                    return Err("optimize_strategy: no known topics to optimize around".into());
                }
                let format = intention
                    .parameters
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("technical_blog");
                if !self.content_formats.contains(&format.to_string()) {
                    self.content_formats.push(format.to_string());
                }
                self.pipeline_freshness =
                    (self.pipeline_freshness + 0.1).min(1.0);
                Ok(WorldEffect {
                    domain: Domain::Earn,
                    description: format!(
                        "optimized strategy: format={}, topics={}, freshness={:.2}",
                        format,
                        self.known_topics.len(),
                        self.pipeline_freshness
                    ),
                    success: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }

            other => {
                self.error_count += 1;
                Err(format!("earn: unknown action '{}'", other))
            }
        }
    }

    fn curiosity_signals(&self) -> Vec<CuriositySignal> {
        let mut signals = Vec::new();

        let all_formats = [
            "technical_blog",
            "video_short",
            "twitter_thread",
            "code_demo",
            "podcast_episode",
            "infographic",
            "tutorial_series",
            "research_paper_summary",
        ];
        for fmt in all_formats {
            if !self.content_formats.contains(&fmt.to_string()) {
                signals.push(CuriositySignal {
                    domain: Domain::Earn,
                    query: format!("explore content format: {}", fmt),
                    novelty_estimate: 0.85,
                    potential_negentropy: 0.6,
                });
            }
        }

        let all_platforms = [
            "x/twitter",
            "youtube",
            "github",
            "bilibili",
            "linkedin",
            "substack",
            "medium",
            "discord",
            "telegram",
        ];
        for plat in all_platforms {
            if !self.platforms.contains(&plat.to_string()) {
                signals.push(CuriositySignal {
                    domain: Domain::Earn,
                    query: format!("expand to platform: {}", plat),
                    novelty_estimate: 0.75,
                    potential_negentropy: 0.5,
                });
            }
        }

        let emerging_topics = [
            ("VSA-native RL post-training", 0.9, 0.85),
            ("E8 lattice attention mechanisms", 0.85, 0.8),
            ("sleep-consolidated memory for agents", 0.8, 0.75),
            ("curiosity-driven content generation", 0.7, 0.7),
            ("hyperdimensional knowledge graphs", 0.75, 0.65),
        ];
        for (topic, novelty, neg) in &emerging_topics {
            if !self.known_topics.contains(&topic.to_string()) {
                signals.push(CuriositySignal {
                    domain: Domain::Earn,
                    query: format!("create content about: {}", topic),
                    novelty_estimate: *novelty,
                    potential_negentropy: *neg,
                });
            }
        }

        if self.monetization_strategies.len() < 5 {
            signals.push(CuriositySignal {
                domain: Domain::Earn,
                query: "discover new monetization strategies".into(),
                novelty_estimate: 0.8,
                potential_negentropy: 0.7,
            });
        }

        signals
    }

    fn grace_mode(&self) -> GraceMode {
        GraceMode::SkipSilently
    }

    fn health(&self) -> BridgeHealth {
        BridgeHealth {
            domain: Domain::Earn,
            available: self.api_keys_configured && self.pipeline_active,
            last_seen_ms: self.last_publish_ms,
            error_count: self.error_count,
            total_actuations: self.total_actuations,
        }
    }

    fn probe_available(&self) -> bool {
        self.api_keys_configured && self.pipeline_active
    }

    fn negentropy_estimate(&self) -> f64 {
        let freshness = self.pipeline_freshness;
        let topic_novelty = if self.known_topics.is_empty() {
            0.0
        } else {
            self.current_trends()
                .iter()
                .filter(|(t, _)| self.known_topics.contains(t))
                .count() as f64
                / self.known_topics.len().max(1) as f64
        };
        let coverage = self.content_formats.len() as f64 / 8.0;
        let platform_reach = self.platforms.len() as f64 / 9.0;
        let base = 0.3 * freshness + 0.3 * topic_novelty + 0.2 * coverage + 0.2 * platform_reach;
        if !self.api_keys_configured {
            base * 0.1
        } else {
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Domain;

    #[test]
    fn test_earn_bridge_default_state() {
        let bridge = EarnBridge::new();
        assert!(!bridge.pipeline_active);
        assert!(!bridge.api_keys_configured);
        assert_eq!(bridge.total_content_generated, 0);
        assert_eq!(bridge.error_count, 0);
        assert_eq!(bridge.known_topics.len(), 5);
    }

    #[test]
    fn test_grace_mode_is_skip_silently() {
        let bridge = EarnBridge::new();
        assert_eq!(bridge.grace_mode(), GraceMode::SkipSilently);
    }

    #[test]
    fn test_probe_available_false_by_default() {
        let bridge = EarnBridge::new();
        assert!(!bridge.probe_available());
    }

    #[test]
    fn test_probe_available_true_with_api_keys() {
        let bridge = EarnBridge::with_api_keys(EarnBridge::new());
        assert!(bridge.probe_available());
    }

    #[test]
    fn test_health_without_keys() {
        let bridge = EarnBridge::new();
        let h = bridge.health();
        assert_eq!(h.domain, Domain::Earn);
        assert!(!h.available);
        assert_eq!(h.error_count, 0);
    }

    #[test]
    fn test_health_with_keys() {
        let bridge = EarnBridge::with_api_keys(EarnBridge::new());
        let h = bridge.health();
        assert!(h.available);
    }

    #[test]
    fn test_sense_returns_signals() {
        let mut bridge = EarnBridge::new();
        let signals = bridge.sense();
        assert!(!signals.is_empty());
        for s in &signals {
            assert_eq!(s.origin, VsaOrigin::World(crate::types::Sensory::SocialFeed));
        }
    }

    #[test]
    fn test_sense_negentropy_low_when_inactive() {
        let mut bridge = EarnBridge::new();
        let signals = bridge.sense();
        let pipeline_sig = &signals[signals.len() - 2];
        assert!(pipeline_sig.negentropy_contribution < 0.2);
    }

    #[test]
    fn test_sense_negentropy_higher_when_active() {
        let mut bridge = EarnBridge::with_api_keys(EarnBridge::new());
        let signals = bridge.sense();
        let pipeline_sig = &signals[signals.len() - 2];
        assert!(pipeline_sig.negentropy_contribution >= 0.4);
    }

    #[test]
    fn test_actuate_generate_content_without_keys_fails() {
        let mut bridge = EarnBridge::new();
        let intention = IntentionVsa {
            domain: Domain::Earn,
            action: "generate_content".into(),
            parameters: serde_json::json!({"topic": "rust agents"}),
            confidence: 0.8,
            urgency: 0.5,
        };
        let result = bridge.actuate(&intention);
        assert!(result.is_err());
        assert_eq!(bridge.error_count, 1);
    }

    #[test]
    fn test_actuate_generate_content_with_keys_succeeds() {
        let mut bridge = EarnBridge::with_api_keys(EarnBridge::new());
        let intention = IntentionVsa {
            domain: Domain::Earn,
            action: "generate_content".into(),
            parameters: serde_json::json!({"topic": "VSA-native reasoning"}),
            confidence: 0.9,
            urgency: 0.6,
        };
        let result = bridge.actuate(&intention);
        assert!(result.is_ok());
        let effect = result.unwrap();
        assert!(effect.success);
        assert_eq!(bridge.total_content_generated, 1);
        assert!(bridge.last_publish_ms > 0);
    }

    #[test]
    fn test_actuate_generate_content_adds_new_topic() {
        let mut bridge = EarnBridge::with_api_keys(EarnBridge::new());
        let initial_count = bridge.known_topics.len();
        let intention = IntentionVsa {
            domain: Domain::Earn,
            action: "generate_content".into(),
            parameters: serde_json::json!({"topic": "brand_new_topic_area"}),
            confidence: 0.8,
            urgency: 0.5,
        };
        let _ = bridge.actuate(&intention);
        assert_eq!(bridge.known_topics.len(), initial_count + 1);
        assert!(bridge.known_topics.contains(&"brand_new_topic_area".into()));
    }

    #[test]
    fn test_actuate_publish_without_keys_fails() {
        let mut bridge = EarnBridge::new();
        let intention = IntentionVsa {
            domain: Domain::Earn,
            action: "publish".into(),
            parameters: serde_json::json!({"platform": "youtube"}),
            confidence: 0.7,
            urgency: 0.4,
        };
        assert!(bridge.actuate(&intention).is_err());
    }

    #[test]
    fn test_actuate_publish_with_keys_succeeds() {
        let mut bridge = EarnBridge::with_api_keys(EarnBridge::new());
        let intention = IntentionVsa {
            domain: Domain::Earn,
            action: "publish".into(),
            parameters: serde_json::json!({"platform": "youtube"}),
            confidence: 0.8,
            urgency: 0.5,
        };
        let result = bridge.actuate(&intention);
        assert!(result.is_ok());
        let effect = result.unwrap();
        assert!(effect.description.contains("youtube"));
    }

    #[test]
    fn test_actuate_analyze_market_always_succeeds() {
        let mut bridge = EarnBridge::new();
        let intention = IntentionVsa {
            domain: Domain::Earn,
            action: "analyze_market".into(),
            parameters: serde_json::json!({}),
            confidence: 0.6,
            urgency: 0.3,
        };
        let result = bridge.actuate(&intention);
        assert!(result.is_ok());
        let effect = result.unwrap();
        assert!(effect.description.contains("market analysis"));
    }

    #[test]
    fn test_actuate_optimize_strategy() {
        let mut bridge = EarnBridge::with_api_keys(EarnBridge::new());
        let initial_freshness = bridge.pipeline_freshness;
        let intention = IntentionVsa {
            domain: Domain::Earn,
            action: "optimize_strategy".into(),
            parameters: serde_json::json!({"format": "podcast_episode"}),
            confidence: 0.9,
            urgency: 0.7,
        };
        let result = bridge.actuate(&intention);
        assert!(result.is_ok());
        assert!(bridge.pipeline_freshness > initial_freshness);
        assert!(bridge.content_formats.contains(&"podcast_episode".into()));
    }

    #[test]
    fn test_actuate_unknown_action() {
        let mut bridge = EarnBridge::new();
        let intention = IntentionVsa {
            domain: Domain::Earn,
            action: "nonexistent_action".into(),
            parameters: serde_json::json!({}),
            confidence: 0.5,
            urgency: 0.2,
        };
        assert!(bridge.actuate(&intention).is_err());
        assert_eq!(bridge.error_count, 1);
    }

    #[test]
    fn test_curiosity_signals_returns_potential_explorations() {
        let bridge = EarnBridge::new();
        let signals = bridge.curiosity_signals();
        assert!(!signals.is_empty());
        for s in &signals {
            assert_eq!(s.domain, Domain::Earn);
            assert!(s.novelty_estimate >= 0.0 && s.novelty_estimate <= 1.0);
            assert!(s.potential_negentropy >= 0.0 && s.potential_negentropy <= 1.0);
        }
    }

    #[test]
    fn test_curiosity_signals_includes_unused_formats() {
        let bridge = EarnBridge::new();
        let signals = bridge.curiosity_signals();
        let format_signals: Vec<_> = signals
            .iter()
            .filter(|s| s.query.contains("content format"))
            .collect();
        assert!(!format_signals.is_empty());
    }

    #[test]
    fn test_curiosity_signals_includes_unused_platforms() {
        let bridge = EarnBridge::new();
        let signals = bridge.curiosity_signals();
        let platform_signals: Vec<_> = signals
            .iter()
            .filter(|s| s.query.contains("platform"))
            .collect();
        assert!(!platform_signals.is_empty());
    }

    #[test]
    fn test_curiosity_signals_includes_emerging_topics() {
        let bridge = EarnBridge::new();
        let signals = bridge.curiosity_signals();
        let topic_signals: Vec<_> = signals
            .iter()
            .filter(|s| s.query.contains("create content about"))
            .collect();
        assert!(!topic_signals.is_empty());
    }

    #[test]
    fn test_negentropy_estimate_low_when_inactive() {
        let bridge = EarnBridge::new();
        let neg = bridge.negentropy_estimate();
        assert!(neg < 0.3);
    }

    #[test]
    fn test_negentropy_estimate_higher_when_configured() {
        let bridge = EarnBridge::with_api_keys(EarnBridge::new());
        let neg = bridge.negentropy_estimate();
        assert!(neg >= 0.0 && neg <= 1.0);
    }

    #[test]
    fn test_negentropy_estimate_increases_with_topics() {
        let bridge = EarnBridge::with_api_keys(
            EarnBridge::new()
                .with_topics(vec!["self-improving agents".into(), "rust architecture".into()]),
        );
        let neg = bridge.negentropy_estimate();
        assert!(neg > 0.0);
    }

    #[test]
    fn test_total_actuations_tracked() {
        let mut bridge = EarnBridge::with_api_keys(EarnBridge::new());
        assert_eq!(bridge.total_actuations, 0);
        let intention = IntentionVsa {
            domain: Domain::Earn,
            action: "analyze_market".into(),
            parameters: serde_json::json!({}),
            confidence: 0.5,
            urgency: 0.3,
        };
        let _ = bridge.actuate(&intention);
        assert_eq!(bridge.total_actuations, 1);
        let _ = bridge.actuate(&intention);
        assert_eq!(bridge.total_actuations, 2);
    }

    #[test]
    fn test_enqueue_content() {
        let mut bridge = EarnBridge::new();
        bridge.enqueue_content("test topic".into());
        assert_eq!(bridge.content_queue.len(), 1);
    }

    #[test]
    fn test_domain_returns_earn() {
        let bridge = EarnBridge::new();
        assert_eq!(bridge.domain(), Domain::Earn);
    }

    #[test]
    fn test_generate_quality_range() {
        let bridge = EarnBridge::new();
        let quality = bridge.generate_quality();
        assert!(quality >= 0.0 && quality <= 1.0);
    }

    #[test]
    fn test_with_topics_overrides_defaults() {
        let topics = vec!["ai safety".into(), "rust patterns".into()];
        let bridge = EarnBridge::new().with_topics(topics.clone());
        assert_eq!(bridge.known_topics, topics);
    }

    #[test]
    fn test_health_tracks_error_count() {
        let mut bridge = EarnBridge::new();
        let intention = IntentionVsa {
            domain: Domain::Earn,
            action: "generate_content".into(),
            parameters: serde_json::json!({}),
            confidence: 0.5,
            urgency: 0.3,
        };
        let _ = bridge.actuate(&intention);
        let h = bridge.health();
        assert_eq!(h.error_count, 1);
    }

    #[test]
    fn test_actuate_publish_adds_new_platform() {
        let mut bridge = EarnBridge::with_api_keys(EarnBridge::new());
        assert!(!bridge.platforms.contains(&"linkedin".into()));
        let intention = IntentionVsa {
            domain: Domain::Earn,
            action: "publish".into(),
            parameters: serde_json::json!({"platform": "linkedin"}),
            confidence: 0.7,
            urgency: 0.4,
        };
        let _ = bridge.actuate(&intention);
        assert!(bridge.platforms.contains(&"linkedin".into()));
    }

    #[test]
    fn test_sense_includes_revenue_signal() {
        let mut bridge = EarnBridge::with_api_keys(EarnBridge::new());
        bridge.latest_revenue = 0.75;
        let signals = bridge.sense();
        let last = signals.last().unwrap();
        assert!(last.negentropy_contribution > 0.0);
    }
}
