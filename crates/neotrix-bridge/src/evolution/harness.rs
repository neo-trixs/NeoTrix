use std::collections::{HashMap, VecDeque};
use crate::types::{Domain, IntentionVsa, WorldEffect};

#[derive(Debug, Clone)]
pub struct OutcomeRecord {
    pub action: String,
    pub domain: Domain,
    pub success: bool,
    pub latency_ms: u64,
    pub timestamp_ms: i64,
    pub recovery_attempted: bool,
    pub recovery_succeeded: bool,
}

#[derive(Debug, Clone)]
pub struct ActionProfile {
    pub domain: Domain,
    pub action: String,
    pub total_calls: u64,
    pub success_count: u64,
    pub recovery_count: u64,
    pub recovery_success_count: u64,
    pub avg_latency_ms: f64,
    pub consecutive_failures: u64,
}

#[derive(Debug)]
pub enum HarnessDecision {
    Proceed,
    Retry { reason: String, backoff_ms: u64 },
    Fallback { alternative: String, reason: String },
    Abort { reason: String },
}

#[derive(Debug)]
pub struct OutcomeDrivenHarness {
    pub profiles: Vec<ActionProfile>,
    pub recent_outcomes: VecDeque<OutcomeRecord>,
    pub total_interceptions: u64,
    pub total_recoveries: u64,
    pub recovery_successes: u64,
    max_recent: usize,
    max_retries: u64,
    consecutive_failure_limit: u64,
    backoff_base_ms: u64,
}

impl Default for OutcomeDrivenHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl OutcomeDrivenHarness {
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            recent_outcomes: VecDeque::with_capacity(500),
            total_interceptions: 0,
            total_recoveries: 0,
            recovery_successes: 0,
            max_recent: 500,
            max_retries: 3,
            consecutive_failure_limit: 3,
            backoff_base_ms: 100,
        }
    }

    pub fn with_retry_limit(mut self, limit: u64) -> Self {
        self.max_retries = limit;
        self
    }

    pub fn with_failure_limit(mut self, limit: u64) -> Self {
        self.consecutive_failure_limit = limit;
        self
    }

    pub fn with_backoff_base(mut self, ms: u64) -> Self {
        self.backoff_base_ms = ms;
        self
    }

    fn profile_index(&mut self, domain: &Domain, action: &str) -> usize {
        let existing = self.profiles.iter().position(|p| p.domain == *domain && p.action == action);
        if let Some(i) = existing {
            i
        } else {
            self.profiles.push(ActionProfile {
                domain: *domain,
                action: action.to_string(),
                total_calls: 0,
                success_count: 0,
                recovery_count: 0,
                recovery_success_count: 0,
                avg_latency_ms: 0.0,
                consecutive_failures: 0,
            });
            self.profiles.len() - 1
        }
    }

    pub fn profile(&mut self, domain: &Domain, action: &str) -> &mut ActionProfile {
        let idx = self.profiles.iter().position(|p| p.domain == *domain && p.action == action);
        if let Some(i) = idx {
            &mut self.profiles[i]
        } else {
            self.profiles.push(ActionProfile {
                domain: *domain,
                action: action.to_string(),
                total_calls: 0,
                success_count: 0,
                recovery_count: 0,
                recovery_success_count: 0,
                avg_latency_ms: 0.0,
                consecutive_failures: 0,
            });
            self.profiles.last_mut().unwrap()
        }
    }

    /// Check whether this actuation should proceed, be retried, fallback, or abort
    pub fn evaluate_before(&mut self, intention: &IntentionVsa) -> HarnessDecision {
        let failure_limit = self.consecutive_failure_limit;
        let max_retries = self.max_retries;
        let (domain, action) = (intention.domain, intention.action.clone());
        let pidx = self.profile_index(&domain, &action);
        let consecutive_failures = self.profiles[pidx].consecutive_failures;
        let recovery_count = self.profiles[pidx].recovery_count;

        if consecutive_failures >= failure_limit {
            self.total_interceptions += 1;
            if recovery_count < max_retries {
                let backoff = self.backoff_base_ms * (1u64 << recovery_count.min(5));
                return HarnessDecision::Retry {
                    reason: format!("{} consecutive failures on {}.{}", consecutive_failures, domain, action),
                    backoff_ms: backoff.min(5000),
                };
            } else {
                // Find alternative action in same domain
                let alt = self.profiles.iter()
                    .filter(|p| p.domain == intention.domain && p.action != intention.action)
                    .max_by(|a, b| a.success_rate().partial_cmp(&b.success_rate()).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|p| p.action.clone());
                if let Some(alternative) = alt {
                    return HarnessDecision::Fallback {
                        alternative,
                        reason: format!("retries exhausted for {}:{}", intention.domain, intention.action),
                    };
                }
                return HarnessDecision::Abort {
                    reason: format!("{}:{} unrecoverable", intention.domain, intention.action),
                };
            }
        }

        HarnessDecision::Proceed
    }

    /// Record outcome and update profile
    pub fn record_outcome(&mut self, intention: &IntentionVsa, effect: &WorldEffect, latency_ms: u64, was_recovery: bool) {
        let (domain, action) = (intention.domain, intention.action.clone());
        // Use index-based access to avoid borrow conflicts with self.* fields
        let pidx = self.profile_index(&domain, &action);
        {
            let profile = &mut self.profiles[pidx];
            profile.total_calls += 1;
            profile.avg_latency_ms = profile.avg_latency_ms * 0.9 + latency_ms as f64 * 0.1;

            if effect.success {
                profile.success_count += 1;
                profile.consecutive_failures = 0;
                if was_recovery {
                    profile.recovery_success_count += 1;
                }
            } else {
                profile.consecutive_failures += 1;
            }

            if was_recovery {
                profile.recovery_count += 1;
            }
        } // drop profile borrow

        if was_recovery {
            self.total_recoveries += 1;
            if effect.success {
                self.recovery_successes += 1;
            }
        }

        let record = OutcomeRecord {
            action: intention.action.clone(),
            domain: intention.domain,
            success: effect.success,
            latency_ms,
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
            recovery_attempted: was_recovery,
            recovery_succeeded: was_recovery && effect.success,
        };

        if self.recent_outcomes.len() >= self.max_recent {
            self.recent_outcomes.pop_front();
        }
        self.recent_outcomes.push_back(record);
    }

    /// Execute a complete harness cycle: evaluate → (recovery loop) → record
    pub fn execute(&mut self, intention: &IntentionVsa, actuate_fn: &mut dyn FnMut(&IntentionVsa) -> Result<WorldEffect, String>) -> Result<WorldEffect, String> {
        let mut current_intent = intention.clone();
        let mut recovery_depth = 0;

        loop {
            match self.evaluate_before(&current_intent) {
                HarnessDecision::Proceed => {
                    let start = std::time::Instant::now();
                    let result = actuate_fn(&current_intent);
                    let elapsed = start.elapsed().as_millis() as u64;
                    match result {
                        Ok(effect) => {
                            self.record_outcome(&current_intent, &effect, elapsed, recovery_depth > 0);
                            return Ok(effect);
                        }
                        Err(e) => {
                            let failed_effect = WorldEffect {
                                domain: current_intent.domain,
                                description: e.clone(),
                                success: false,
                                latency_ms: elapsed,
                            };
                            self.record_outcome(&current_intent, &failed_effect, elapsed, recovery_depth > 0);
                            recovery_depth += 1;
                            // Fall through to next loop iteration for retry decision
                        }
                    }
                }
                HarnessDecision::Retry { reason: _, backoff_ms } => {
                    // Brief sleep to backoff
                    std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                    recovery_depth += 1;
                    continue;
                }
                HarnessDecision::Fallback { alternative, reason: _ } => {
                    let mut fallback_intent = current_intent.clone();
                    fallback_intent.action = alternative.clone();
                    current_intent = fallback_intent;
                    recovery_depth += 1;
                    continue;
                }
                HarnessDecision::Abort { reason } => {
                    return Err(reason);
                }
            }
        }
    }

    pub fn domain_health(&self, domain: &Domain) -> (u64, u64, f64) {
        let domain_profiles: Vec<&ActionProfile> = self.profiles.iter()
            .filter(|p| p.domain == *domain)
            .collect();
        let total: u64 = domain_profiles.iter().map(|p| p.total_calls).sum();
        let successes: u64 = domain_profiles.iter().map(|p| p.success_count).sum();
        let rate = if total > 0 { successes as f64 / total as f64 } else { 1.0 };
        (total, successes, rate)
    }

    pub fn summary(&self) -> HashMap<Domain, (u64, u64, f64)> {
        let mut m = HashMap::new();
        for d in &[Domain::Crypto, Domain::Earn, Domain::Network, Domain::Crawl,
                    Domain::Social, Domain::Browse, Domain::Vision] {
            m.insert(*d, self.domain_health(d));
        }
        m
    }
}

impl ActionProfile {
    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 { 1.0 } else { self.success_count as f64 / self.total_calls as f64 }
    }

    pub fn recovery_rate(&self) -> f64 {
        if self.recovery_count == 0 { 1.0 } else { self.recovery_success_count as f64 / self.recovery_count as f64 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Domain, IntentionVsa, WorldEffect};

    fn intention(domain: Domain, action: &str) -> IntentionVsa {
        IntentionVsa {
            domain,
            action: action.into(),
            parameters: serde_json::json!({}),
            confidence: 0.9,
            urgency: 0.5,
        }
    }

    fn ok_effect() -> WorldEffect {
        WorldEffect { domain: Domain::System, description: "ok".into(), success: true, latency_ms: 5 }
    }

    fn fail_effect() -> WorldEffect {
        WorldEffect { domain: Domain::System, description: "fail".into(), success: false, latency_ms: 5 }
    }

    #[test]
    fn test_proceed_on_first_call() {
        let mut h = OutcomeDrivenHarness::new();
        let intent = intention(Domain::Crawl, "explore");
        let d = h.evaluate_before(&intent);
        assert!(matches!(d, HarnessDecision::Proceed));
    }

    #[test]
    fn test_retry_after_consecutive_failures() {
        let mut h = OutcomeDrivenHarness::new();
        let intent = intention(Domain::Network, "rotate");
        for _ in 0..3 {
            h.record_outcome(&intent, &fail_effect(), 10, false);
        }
        let d = h.evaluate_before(&intent);
        assert!(matches!(d, HarnessDecision::Retry { .. }));
    }

    #[test]
    fn test_abort_after_exhausting_retries() {
        let mut h = OutcomeDrivenHarness::with_retry_limit(OutcomeDrivenHarness::new(), 2)
            .with_failure_limit(2);
        let intent = intention(Domain::Crypto, "transfer");
        for _ in 0..8 {
            h.record_outcome(&intent, &fail_effect(), 10, true);
        }
        let d = h.evaluate_before(&intent);
        assert!(matches!(d, HarnessDecision::Abort { .. }));
    }

    #[test]
    fn test_fallback_to_alternative() {
        let mut h = OutcomeDrivenHarness::new();
        let bad = intention(Domain::Crawl, "bad_action");
        let good = intention(Domain::Crawl, "good_action");
        for _ in 0..10 {
            h.record_outcome(&bad, &fail_effect(), 10, true);
        }
        for _ in 0..10 {
            h.record_outcome(&good, &ok_effect(), 5, false);
        }
        let d = h.evaluate_before(&bad);
        match d {
            HarnessDecision::Fallback { alternative, .. } => {
                assert_eq!(alternative, "good_action");
            }
            _ => panic!("expected fallback, got {:?}", d),
        }
    }

    #[test]
    fn test_execute_success_path() {
        let mut h = OutcomeDrivenHarness::new();
        let intent = intention(Domain::Crawl, "explore");
        let mut call_count = 0;
        let result = h.execute(&intent, &mut |_| {
            call_count += 1;
            Ok(ok_effect())
        });
        assert!(result.is_ok());
        assert_eq!(call_count, 1);
    }

    #[test]
    fn test_execute_with_retry_recovery() {
        let mut h = OutcomeDrivenHarness::new()
            .with_failure_limit(2)
            .with_backoff_base(1);
        let intent = intention(Domain::Crawl, "explore");
        let mut attempts = 0;
        let result = h.execute(&intent, &mut |_| {
            attempts += 1;
            if attempts <= 2 {
                Err("transient".into())
            } else {
                Ok(ok_effect())
            }
        });
        assert!(result.is_ok());
        assert!(attempts >= 3);
    }

    #[test]
    fn test_domain_health() {
        let mut h = OutcomeDrivenHarness::new();
        let intent = intention(Domain::Crawl, "explore");
        for _ in 0..10 {
            h.record_outcome(&intent, &ok_effect(), 5, false);
        }
        let (total, successes, rate) = h.domain_health(&Domain::Crawl);
        assert_eq!(total, 10);
        assert_eq!(successes, 10);
        assert!((rate - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_success_rate_resets_consecutive_failures() {
        let mut h = OutcomeDrivenHarness::new();
        let intent = intention(Domain::Browse, "navigate");
        for _ in 0..2 { h.record_outcome(&intent, &fail_effect(), 10, false); }
        h.record_outcome(&intent, &ok_effect(), 5, false);
        let p = h.profile(&Domain::Browse, "navigate");
        assert_eq!(p.consecutive_failures, 0);
    }

    #[test]
    fn test_summary_covers_all_domains() {
        let h = OutcomeDrivenHarness::new();
        let s = h.summary();
        assert_eq!(s.len(), 7);
    }

    #[test]
    fn test_action_profile_recovery_rate() {
        let mut p = ActionProfile {
            domain: Domain::System,
            action: "test".into(),
            total_calls: 10,
            success_count: 5,
            recovery_count: 4,
            recovery_success_count: 3,
            avg_latency_ms: 0.0,
            consecutive_failures: 0,
        };
        assert!((p.recovery_rate() - 0.75).abs() < 0.01);
        p.recovery_count = 0;
        assert!((p.recovery_rate() - 1.0).abs() < 0.01);
    }
}
