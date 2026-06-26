use std::collections::VecDeque;
use crate::types::{Domain, IntentionVsa, WorldEffect, VsaLight};

const MAX_TRACES_PER_DOMAIN: usize = 500;

#[derive(Debug, Clone)]
pub struct TraceStep {
    pub intention: IntentionVsa,
    pub effect: WorldEffect,
    pub timestamp_ms: i64,
    pub duration_ms: u64,
    pub vsa_context: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TraceEpisode {
    pub domain: Domain,
    pub steps: Vec<TraceStep>,
    pub outcome_score: f64,
    pub start_ms: i64,
    pub end_ms: i64,
    pub total_steps: usize,
    pub success_count: usize,
}

#[derive(Debug)]
pub struct TraceRecorder {
    pub traces: Vec<TraceEpisode>,
    pub max_episodes: usize,
    pub vsa: VsaLight,
    pub total_recorded: u64,
    domain_buffers: Vec<VecDeque<TraceStep>>,
}

impl TraceRecorder {
    pub fn new(max_episodes: usize) -> Self {
        Self {
            traces: Vec::with_capacity(max_episodes),
            max_episodes,
            vsa: VsaLight::new(256),
            total_recorded: 0,
            domain_buffers: (0..7).map(|_| VecDeque::with_capacity(MAX_TRACES_PER_DOMAIN)).collect(),
        }
    }

    fn domain_index(d: &Domain) -> usize {
        match d {
            Domain::Crypto => 0,
            Domain::Earn => 1,
            Domain::Network => 2,
            Domain::Crawl => 3,
            Domain::Social => 4,
            Domain::Browse => 5,
            Domain::Vision => 6,
            Domain::System => 0,
        }
    }

    pub fn record_step(&mut self, intention: &IntentionVsa, effect: &WorldEffect, duration_ms: u64) {
        let now_ms = chrono::Utc::now().timestamp_millis();
        let step = TraceStep {
            intention: intention.clone(),
            effect: effect.clone(),
            timestamp_ms: now_ms,
            duration_ms,
            vsa_context: self.vsa.seeded_vector(self.total_recorded),
        };
        let idx = Self::domain_index(&intention.domain);
        if let Some(buf) = self.domain_buffers.get_mut(idx) {
            if buf.len() >= MAX_TRACES_PER_DOMAIN {
                buf.pop_front();
            }
            buf.push_back(step);
        }
        self.total_recorded += 1;
    }

    pub fn episode(&mut self, domain: Domain, steps: Vec<TraceStep>) {
        if steps.is_empty() {
            return;
        }
        let start_ms = steps.first().map(|s| s.timestamp_ms).unwrap_or(0);
        let end_ms = steps.last().map(|s| s.timestamp_ms).unwrap_or(0);
        let total_steps = steps.len();
        let success_count = steps.iter().filter(|s| s.effect.success).count();
        let outcome_score = if total_steps == 0 { 0.0 } else { success_count as f64 / total_steps as f64 };

        let episode = TraceEpisode {
            domain,
            steps,
            outcome_score,
            start_ms,
            end_ms,
            total_steps,
            success_count,
        };

        if self.traces.len() >= self.max_episodes {
            self.traces.remove(0);
        }
        self.traces.push(episode);
    }

    pub fn domain_buffer(&self, d: &Domain) -> &VecDeque<TraceStep> {
        let idx = Self::domain_index(d);
        &self.domain_buffers[idx]
    }

    pub fn recent_failures(&self, domain: &Domain, n: usize) -> Vec<&TraceStep> {
        let idx = Self::domain_index(domain);
        self.domain_buffers[idx].iter()
            .filter(|s| !s.effect.success)
            .rev()
            .take(n)
            .collect()
    }

    pub fn success_rate(&self, domain: &Domain) -> f64 {
        let idx = Self::domain_index(domain);
        let buf = &self.domain_buffers[idx];
        if buf.is_empty() {
            return 1.0;
        }
        let successes = buf.iter().filter(|s| s.effect.success).count();
        successes as f64 / buf.len() as f64
    }

    pub fn average_latency(&self, domain: &Domain) -> f64 {
        let idx = Self::domain_index(domain);
        let buf = &self.domain_buffers[idx];
        if buf.is_empty() {
            return 0.0;
        }
        let total: u64 = buf.iter().map(|s| s.duration_ms).sum();
        total as f64 / buf.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Domain, IntentionVsa, WorldEffect};

    fn make_intention(domain: Domain, action: &str) -> IntentionVsa {
        IntentionVsa {
            domain,
            action: action.into(),
            parameters: serde_json::json!({}),
            confidence: 0.9,
            urgency: 0.5,
        }
    }

    fn make_effect(success: bool) -> WorldEffect {
        WorldEffect {
            domain: Domain::Crawl,
            description: "test".into(),
            success,
            latency_ms: 10,
        }
    }

    #[test]
    fn test_record_and_retrieve() {
        let mut r = TraceRecorder::new(100);
        let intent = make_intention(Domain::Crawl, "explore");
        let effect = make_effect(true);
        r.record_step(&intent, &effect, 15);
        assert_eq!(r.total_recorded, 1);
        assert_eq!(r.domain_buffer(&Domain::Crawl).len(), 1);
    }

    #[test]
    fn test_success_rate() {
        let mut r = TraceRecorder::new(100);
        let intent = make_intention(Domain::Network, "rotate");
        for i in 0..10 {
            let ok = i < 7;
            r.record_step(&intent, &make_effect(ok), 5);
        }
        let rate = r.success_rate(&Domain::Network);
        assert!((rate - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_recent_failures() {
        let mut r = TraceRecorder::new(100);
        let intent = make_intention(Domain::Crypto, "transfer");
        for i in 0..5 {
            r.record_step(&intent, &make_effect(i != 2 && i != 4), 3);
        }
        let failures = r.recent_failures(&Domain::Crypto, 3);
        assert_eq!(failures.len(), 2);
    }

    #[test]
    fn test_episode() {
        let mut r = TraceRecorder::new(10);
        let intent = make_intention(Domain::Crawl, "explore");
        let steps: Vec<TraceStep> = (0..5).map(|i| {
            let effect = make_effect(i < 4);
            TraceStep {
                intention: intent.clone(),
                effect,
                timestamp_ms: 1000 + i as i64,
                duration_ms: 10,
                vsa_context: vec![],
            }
        }).collect();
        r.episode(Domain::Crawl, steps);
        assert_eq!(r.traces.len(), 1);
        assert!((r.traces[0].outcome_score - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_max_episodes() {
        let mut r = TraceRecorder::new(3);
        for n in 0..5 {
            let intent = make_intention(Domain::System, "test");
            let steps = vec![TraceStep {
                intention: intent,
                effect: make_effect(true),
                timestamp_ms: n * 1000,
                duration_ms: 5,
                vsa_context: vec![],
            }];
            r.episode(Domain::System, steps);
        }
        assert_eq!(r.traces.len(), 3);
    }

    #[test]
    fn test_average_latency() {
        let mut r = TraceRecorder::new(100);
        let intent = make_intention(Domain::Browse, "navigate");
        for ms in &[10, 20, 30] {
            r.record_step(&intent, &make_effect(true), *ms);
        }
        let avg = r.average_latency(&Domain::Browse);
        assert!((avg - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_empty_buffer() {
        let r = TraceRecorder::new(100);
        assert!((r.success_rate(&Domain::Vision) - 1.0).abs() < 0.01);
        assert!((r.average_latency(&Domain::Vision) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_buffer_capacity() {
        let mut r = TraceRecorder::new(100);
        let intent = make_intention(Domain::Social, "post");
        for _ in 0..600 {
            r.record_step(&intent, &make_effect(true), 1);
        }
        assert_eq!(r.domain_buffer(&Domain::Social).len(), 500);
    }
}
