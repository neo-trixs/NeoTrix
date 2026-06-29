use crate::types::{
    BridgeHealth, ConsciousnessAbility, CuriositySignal, Domain, GraceMode, IntentionVsa, VsaLight,
    VsaOrigin, VsaTagged, WorldEffect, Sensory,
};

pub struct SocialBridge {
    pub vsa: VsaLight,
    pub social_configured: bool,
    pub tracked_topics: Vec<String>,
    pub posts_made: u64,
    pub total_actuations: u64,
    pub last_feed_check_ms: i64,
    pub error_count: u64,
}

impl Default for SocialBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl SocialBridge {
    pub fn new() -> Self {
        Self {
            vsa: VsaLight::new(4096),
            social_configured: false,
            tracked_topics: vec![
                "consciousness".into(),
                "ai_safety".into(),
                "machine_learning".into(),
                "vsa".into(),
                "hdc".into(),
                "negentropy".into(),
            ],
            posts_made: 0,
            total_actuations: 0,
            last_feed_check_ms: 0,
            error_count: 0,
        }
    }

    fn seed_for_topic(&self, topic: &str, idx: usize) -> u64 {
        let mut s: u64 = 0xdead_beef;
        for b in topic.bytes() {
            s = s.wrapping_mul(31).wrapping_add(b as u64);
        }
        s.wrapping_add(idx as u64)
    }

    fn known_vectors(&self) -> Vec<Vec<u8>> {
        self.tracked_topics
            .iter()
            .enumerate()
            .map(|(i, t)| self.vsa.seeded_vector(self.seed_for_topic(t, i)))
            .collect()
    }
}

impl ConsciousnessAbility for SocialBridge {
    fn domain(&self) -> Domain {
        Domain::Social
    }

    fn sense(&mut self) -> Vec<VsaTagged> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        self.last_feed_check_ms = now;

        if !self.social_configured {
            return Vec::new();
        }

        let known = self.known_vectors();
        let mut results = Vec::with_capacity(self.tracked_topics.len());

        for (i, topic) in self.tracked_topics.iter().enumerate() {
            let seed = self.seed_for_topic(topic, i);
            let vector = self.vsa.seeded_vector(seed);

            let novelty = self.vsa.novelty(&known, &vector, 0.85);
            let topic_depth = (topic.len() as f64).ln_1p() / 10.0;
            let negentropy_contribution = novelty * 0.6 + topic_depth * 0.4;

            results.push(VsaTagged {
                vector,
                origin: VsaOrigin::World(Sensory::SocialFeed),
                timestamp_ms: now,
                negentropy_contribution: (negentropy_contribution * 100.0).round() / 100.0,
            });
        }

        results
    }

    fn actuate(&mut self, intention: &IntentionVsa) -> Result<WorldEffect, String> {
        self.total_actuations += 1;

        if !self.social_configured {
            self.error_count += 1;
            return Err("social not configured".into());
        }

        let start = std::time::Instant::now();

        match intention.action.as_str() {
            "post_update" => {
                self.posts_made += 1;
                Ok(WorldEffect {
                    domain: Domain::Social,
                    description: format!(
                        "post_update conf={} urgency={}",
                        intention.confidence, intention.urgency
                    ),
                    success: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }
            "reply" => {
                self.posts_made += 1;
                Ok(WorldEffect {
                    domain: Domain::Social,
                    description: "reply posted to thread".into(),
                    success: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }
            "search_topic" => {
                let topic = intention
                    .parameters
                    .get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                Ok(WorldEffect {
                    domain: Domain::Social,
                    description: format!("search_topic: {}", topic),
                    success: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }
            "analyze_trends" => {
                Ok(WorldEffect {
                    domain: Domain::Social,
                    description: format!(
                        "trend_analysis over {} tracked topics",
                        self.tracked_topics.len()
                    ),
                    success: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }
            other => {
                self.error_count += 1;
                Err(format!("unknown social action: {}", other))
            }
        }
    }

    fn curiosity_signals(&self) -> Vec<CuriositySignal> {
        if !self.social_configured {
            return Vec::new();
        }

        self.tracked_topics
            .iter()
            .enumerate()
            .map(|(i, topic)| {
                let seed = self.seed_for_topic(topic, i);
                let known = self.known_vectors();
                let vector = self.vsa.seeded_vector(seed);
                let novelty = self.vsa.novelty(&known, &vector, 0.8);

                CuriositySignal {
                    domain: Domain::Social,
                    query: format!("trending:{}", topic),
                    novelty_estimate: (novelty * 100.0).round() / 100.0,
                    potential_negentropy: (novelty * 0.7 + 0.3 * (topic.len() as f64).ln_1p())
                        .min(1.0)
                        .round(),
                }
            })
            .collect()
    }

    fn grace_mode(&self) -> GraceMode {
        GraceMode::SkipSilently
    }

    fn health(&self) -> BridgeHealth {
        BridgeHealth {
            domain: Domain::Social,
            available: self.social_configured,
            last_seen_ms: self.last_feed_check_ms,
            error_count: self.error_count,
            total_actuations: self.total_actuations,
        }
    }

    fn probe_available(&self) -> bool {
        self.social_configured
    }

    fn negentropy_estimate(&self) -> f64 {
        if !self.social_configured {
            return 0.0;
        }
        let feed_relevance = 0.6;
        let engagement_potential = (self.posts_made as f64).clamp(0.0, 100.0) / 100.0 * 0.4;
        (feed_relevance + engagement_potential).min(1.0)
    }
}
