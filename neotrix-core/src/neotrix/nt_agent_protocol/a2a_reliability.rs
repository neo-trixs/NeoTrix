use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn now_u64() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Circuit breaker state machine. (Arsenal-inspired: kavacha)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for A2A agent communication failures.
/// Opens after N consecutive failures, half-opens after cooldown.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    pub name: String,
    pub state: CircuitState,
    pub failure_count: u64,
    pub max_failures: u64,
    pub cooldown_secs: u64,
    pub last_failure_time: u64,
    pub success_count: u64,
    pub half_open_max_success: u64,
}

impl CircuitBreaker {
    pub fn new(name: &str, max_failures: u64, cooldown_secs: u64) -> Self {
        Self {
            name: name.to_string(),
            state: CircuitState::Closed,
            failure_count: 0,
            max_failures,
            cooldown_secs,
            last_failure_time: 0,
            success_count: 0,
            half_open_max_success: 3,
        }
    }

    /// Check if call is allowed through the circuit.
    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let elapsed = now_u64().saturating_sub(self.last_failure_time);
                if elapsed >= self.cooldown_secs {
                    self.state = CircuitState::HalfOpen;
                    self.success_count = 0;
                    log::info!(
                        "CIRCUIT: {} → half-open after {}s cooldown",
                        self.name,
                        elapsed
                    );
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => self.success_count < self.half_open_max_success,
        }
    }

    /// Record a successful call.
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        match self.state {
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.half_open_max_success {
                    self.state = CircuitState::Closed;
                    self.success_count = 0;
                    log::info!(
                        "CIRCUIT: {} → closed after {} successes",
                        self.name,
                        self.half_open_max_success
                    );
                }
            }
            CircuitState::Closed => {}
            _ => {
                self.state = CircuitState::Closed;
            }
        }
    }

    /// Record a failed call.
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = now_u64();
        if self.failure_count >= self.max_failures {
            self.state = CircuitState::Open;
            self.success_count = 0;
            log::warn!(
                "CIRCUIT: {} → OPEN after {} failures",
                self.name,
                self.failure_count
            );
        }
    }

    /// Reset to closed state.
    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
    }
}

/// Retry policy with exponential backoff + jitter. (Arsenal-inspired: punarjanma)
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub jitter_factor: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 200,
            max_delay_ms: 5000,
            jitter_factor: 0.25,
        }
    }
}

impl RetryPolicy {
    pub fn new(max_retries: u32, base_delay_ms: u64) -> Self {
        Self {
            max_retries,
            base_delay_ms,
            max_delay_ms: 5000,
            jitter_factor: 0.25,
        }
    }

    /// Compute delay for retry attempt (0-indexed).
    pub fn delay_ms(&self, attempt: u32) -> u64 {
        let exp = self.base_delay_ms * 2u64.saturating_pow(attempt);
        let capped = exp.min(self.max_delay_ms);
        let jitter = (capped as f64 * self.jitter_factor * fastrand::f64()) as u64;
        capped + jitter
    }

    /// Check if another retry is allowed.
    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }
}

/// Session persistence with TTL. (Arsenal-inspired: sanga)
#[derive(Debug, Clone)]
pub struct AgentSession {
    pub session_id: String,
    pub agent_name: String,
    pub created_at: u64,
    pub last_accessed: u64,
    pub ttl_secs: u64,
    pub context: HashMap<String, String>,
}

impl AgentSession {
    pub fn new(session_id: &str, agent_name: &str, ttl_secs: u64) -> Self {
        let now = now_u64();
        Self {
            session_id: session_id.to_string(),
            agent_name: agent_name.to_string(),
            created_at: now,
            last_accessed: now,
            ttl_secs,
            context: HashMap::new(),
        }
    }

    /// Check if session has expired.
    pub fn is_expired(&self) -> bool {
        let elapsed = now_u64().saturating_sub(self.last_accessed);
        elapsed >= self.ttl_secs
    }

    /// Touch to update last_accessed.
    pub fn touch(&mut self) {
        self.last_accessed = now_u64();
    }
}

/// A2A reliability layer combining circuit breaker, retry, and session management.
pub struct A2AReliabilityLayer {
    pub circuit_breakers: HashMap<String, CircuitBreaker>,
    pub sessions: HashMap<String, AgentSession>,
    pub retry_policy: RetryPolicy,
    pub stats: ReliabilityStats,
}

#[derive(Debug, Clone, Default)]
pub struct ReliabilityStats {
    pub total_calls: u64,
    pub blocked_calls: u64,
    pub retried_calls: u64,
    pub failed_calls: u64,
    pub session_evictions: u64,
}

impl A2AReliabilityLayer {
    pub fn new(retry_policy: RetryPolicy) -> Self {
        Self {
            circuit_breakers: HashMap::new(),
            sessions: HashMap::new(),
            retry_policy,
            stats: ReliabilityStats::default(),
        }
    }

    /// Register a circuit breaker for an agent endpoint.
    pub fn register_circuit_breaker(
        &mut self,
        agent_name: &str,
        max_failures: u64,
        cooldown_secs: u64,
    ) {
        self.circuit_breakers.insert(
            agent_name.to_string(),
            CircuitBreaker::new(agent_name, max_failures, cooldown_secs),
        );
    }

    /// Check if a call to an agent should proceed.
    pub fn allow_call(&mut self, agent_name: &str) -> bool {
        let allowed = match self.circuit_breakers.get_mut(agent_name) {
            Some(cb) => cb.allow_request(),
            None => true,
        };
        if !allowed {
            self.stats.blocked_calls += 1;
        }
        self.stats.total_calls += 1;
        allowed
    }

    /// Record call outcome for an agent.
    pub fn record_outcome(&mut self, agent_name: &str, success: bool) {
        if let Some(cb) = self.circuit_breakers.get_mut(agent_name) {
            if success {
                cb.record_success();
            } else {
                cb.record_failure();
                self.stats.failed_calls += 1;
            }
        }
    }

    /// Execute a call with retry and circuit breaker protection.
    /// Returns (Ok(result), number_of_retries) or (Err, last_attempt).
    pub fn call_with_retry<F, T>(&mut self, agent_name: &str, mut f: F) -> Result<(T, u32), String>
    where
        F: FnMut() -> Result<T, String>,
    {
        if !self.allow_call(agent_name) {
            return Err(format!("circuit breaker OPEN for '{}'", agent_name));
        }

        let mut last_error = String::new();
        for attempt in 0..=self.retry_policy.max_retries {
            match f() {
                Ok(result) => {
                    self.record_outcome(agent_name, true);
                    return Ok((result, attempt));
                }
                Err(e) => {
                    last_error = e;
                    self.stats.retried_calls += 1;
                    if self.retry_policy.should_retry(attempt) {
                        let delay = self.retry_policy.delay_ms(attempt);
                        std::thread::sleep(Duration::from_millis(delay));
                    }
                }
            }
        }

        self.record_outcome(agent_name, false);
        Err(format!(
            "call to '{}' failed after {} retries: {}",
            agent_name, self.retry_policy.max_retries, last_error,
        ))
    }

    /// Create or retrieve an agent session.
    pub fn get_or_create_session(
        &mut self,
        session_id: &str,
        agent_name: &str,
        ttl_secs: u64,
    ) -> &mut AgentSession {
        // Evict expired sessions periodically
        self.evict_expired_sessions();

        if !self.sessions.contains_key(session_id) {
            self.sessions.insert(
                session_id.to_string(),
                AgentSession::new(session_id, agent_name, ttl_secs),
            );
        }
        let session = self.sessions.get_mut(session_id).unwrap();
        session.touch();
        session
    }

    /// Store context in a session.
    pub fn store_session_context(&mut self, session_id: &str, key: &str, value: &str) -> bool {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.context.insert(key.to_string(), value.to_string());
            session.touch();
            true
        } else {
            false
        }
    }

    /// Retrieve context from a session.
    pub fn get_session_context(&self, session_id: &str, key: &str) -> Option<&String> {
        self.sessions
            .get(session_id)
            .and_then(|s| s.context.get(key))
    }

    /// Remove expired sessions.
    pub fn evict_expired_sessions(&mut self) {
        let expired: Vec<String> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.is_expired())
            .map(|(id, _)| id.clone())
            .collect();
        self.stats.session_evictions += expired.len() as u64;
        for id in expired {
            self.sessions.remove(&id);
        }
    }

    /// Summary for logging.
    pub fn summary(&self) -> String {
        format!(
            "A2AReliability: calls={} blocked={} retried={} failed={} sessions={} cb={} evict={}",
            self.stats.total_calls,
            self.stats.blocked_calls,
            self.stats.retried_calls,
            self.stats.failed_calls,
            self.sessions.len(),
            self.circuit_breakers.len(),
            self.stats.session_evictions,
        )
    }
}

impl Default for A2AReliabilityLayer {
    fn default() -> Self {
        Self::new(RetryPolicy::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closed() {
        let mut cb = CircuitBreaker::new("test-agent", 3, 10);
        assert!(cb.allow_request());
        assert_eq!(cb.state, CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_opens_after_failures() {
        let mut cb = CircuitBreaker::new("test-agent", 3, 60);
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.state, CircuitState::Open);
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_half_open_then_close() {
        let mut cb = CircuitBreaker::new("test-agent", 2, 1);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open);
        // Force half-open by setting last_failure_time far back
        cb.last_failure_time = 0;
        assert!(cb.allow_request());
        assert_eq!(cb.state, CircuitState::HalfOpen);
        cb.record_success();
        cb.record_success();
        cb.record_success();
        assert_eq!(cb.state, CircuitState::Closed);
    }

    #[test]
    fn test_retry_default_policy() {
        let p = RetryPolicy::default();
        assert!(p.should_retry(0));
        assert!(p.should_retry(2));
        assert!(!p.should_retry(3));
    }

    #[test]
    fn test_retry_delay_increases() {
        let p = RetryPolicy::new(3, 100);
        let d0 = p.delay_ms(0);
        let d1 = p.delay_ms(1);
        let d2 = p.delay_ms(2);
        assert!(d1 >= d0, "retry delay should increase: d0={} d1={}", d0, d1);
        assert!(d2 >= d1, "retry delay should increase: d1={} d2={}", d1, d2);
    }

    #[test]
    fn test_retry_delay_capped() {
        let p = RetryPolicy::new(10, 100000);
        let d = p.delay_ms(10);
        assert!(d <= p.max_delay_ms, "delay capped at max_delay_ms");
    }

    #[test]
    fn test_session_creation() {
        let mut layer = A2AReliabilityLayer::default();
        let session = layer.get_or_create_session("sess-1", "agent-a", 3600);
        assert_eq!(session.session_id, "sess-1");
        assert_eq!(session.agent_name, "agent-a");
    }

    #[test]
    fn test_session_context() {
        let mut layer = A2AReliabilityLayer::default();
        layer.get_or_create_session("sess-1", "agent-a", 3600);
        assert!(layer.store_session_context("sess-1", "key", "value"));
        assert_eq!(
            layer.get_session_context("sess-1", "key"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_session_expiry() {
        let mut layer = A2AReliabilityLayer::default();
        layer.get_or_create_session("sess-1", "agent-a", 0); // TTL=0 = already expired
        layer.evict_expired_sessions();
        assert!(layer.sessions.is_empty());
    }

    #[test]
    fn test_call_with_retry_success() {
        let mut layer = A2AReliabilityLayer::default();
        let result = layer.call_with_retry("agent-a", || Ok::<_, String>(42));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, 42);
    }

    #[test]
    fn test_call_with_retry_failure() {
        let mut layer = A2AReliabilityLayer::default();
        let result = layer.call_with_retry("agent-a", || Err::<i32, String>("fail".into()));
        assert!(result.is_err());
    }

    #[test]
    fn test_call_blocked_by_open_circuit() {
        let mut layer = A2AReliabilityLayer::default();
        layer.register_circuit_breaker("agent-a", 1, 60);
        // Fail once to open circuit
        let _ = layer.call_with_retry("agent-a", || Err::<i32, String>("fail".into()));
        // Circuit should be open now
        let result = layer.call_with_retry("agent-a", || Ok::<_, String>(42));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("circuit breaker OPEN"));
    }

    #[test]
    fn test_allow_call_tracking() {
        let mut layer = A2AReliabilityLayer::default();
        layer.register_circuit_breaker("agent-a", 1, 60);
        assert!(layer.allow_call("agent-a"));
        layer.record_outcome("agent-a", false);
        assert!(!layer.allow_call("agent-a"));
        assert_eq!(layer.stats.total_calls, 2);
        assert_eq!(layer.stats.blocked_calls, 1);
    }

    #[test]
    fn test_register_circuit_breaker() {
        let mut layer = A2AReliabilityLayer::default();
        layer.register_circuit_breaker("agent-a", 5, 30);
        assert!(layer.circuit_breakers.contains_key("agent-a"));
    }

    #[test]
    fn test_store_context_on_nonexistent_session() {
        let mut layer = A2AReliabilityLayer::default();
        assert!(!layer.store_session_context("no-such-session", "k", "v"));
    }

    #[test]
    fn test_get_context_on_nonexistent_session() {
        let layer = A2AReliabilityLayer::default();
        assert!(layer.get_session_context("no-such-session", "k").is_none());
    }

    #[test]
    fn test_summary_format() {
        let layer = A2AReliabilityLayer::default();
        let s = layer.summary();
        assert!(s.contains("A2AReliability:"));
    }

    #[test]
    fn test_retry_attempt_counts() {
        let mut layer = A2AReliabilityLayer::new(RetryPolicy::new(2, 10));
        let mut count = 0;
        let result = layer.call_with_retry("agent-b", || {
            count += 1;
            if count < 3 {
                Err::<i32, String>("retry".into())
            } else {
                Ok(42)
            }
        });
        assert!(result.is_ok());
        let (val, attempts) = result.unwrap();
        assert_eq!(val, 42);
        assert_eq!(attempts, 2);
    }

    #[test]
    fn test_session_touch_updates_last_accessed() {
        let mut layer = A2AReliabilityLayer::default();
        let session = layer.get_or_create_session("sess-1", "agent-a", 3600);
        let original = session.last_accessed;
        std::thread::sleep(Duration::from_millis(2));
        let session = layer.get_or_create_session("sess-1", "agent-a", 3600);
        assert!(session.last_accessed > original);
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let mut cb = CircuitBreaker::new("test", 3, 60);
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open);
        cb.reset();
        assert_eq!(cb.state, CircuitState::Closed);
        assert_eq!(cb.failure_count, 0);
    }
}
