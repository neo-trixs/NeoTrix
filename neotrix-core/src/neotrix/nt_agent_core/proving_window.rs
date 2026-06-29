use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// How often proving periods roll over
#[derive(Debug, Clone, Copy)]
pub enum ProvingFrequency {
    /// Every cycle (~1 second)
    PerCycle,
    /// Every N cycles
    EveryNCycles(u32),
    /// Every M milliseconds
    EveryMs(u64),
}

/// A single proving window during which agents must be challenged
#[derive(Debug, Clone)]
pub struct ProvingWindow {
    /// Epoch number (monotonically increasing)
    pub epoch: u64,
    /// When this window started (unix nanos)
    pub started_at: u64,
    /// Duration of the window in nanos
    pub duration_ns: u64,
    /// How many challenges must be passed in this window
    pub required_challenges: u32,
    /// Challenges completed so far in this window
    pub challenges_completed: u32,
    /// Whether this window has been passed
    pub passed: bool,
}

/// Per-agent proving status
#[derive(Debug, Clone)]
pub struct AgentProvingStatus {
    pub agent_id: String,
    /// Proving windows this agent participated in
    pub windows: Vec<ProvingWindow>,
    /// Consecutive windows passed
    pub consecutive_passes: u32,
    /// Total windows passed
    pub total_passes: u64,
    /// Last window the agent failed
    pub last_failed_epoch: Option<u64>,
    /// Current challenge count (resets per window)
    pub current_challenge_count: u32,
}

/// Manages proving windows for all agents
pub struct ProvingWindowManager {
    /// Per-agent proving status
    pub agent_status: HashMap<String, AgentProvingStatus>,
    /// Current epoch number
    pub current_epoch: u64,
    /// Default window duration (default: 100 cycles = ~100 seconds)
    pub window_duration_cycles: u32,
    /// Challenges required per window
    pub challenges_required: u32,
    /// When the current window started
    pub window_start_cycle: u64,
}

impl ProvingWindowManager {
    pub fn new(window_duration_cycles: u32, challenges_required: u32) -> Self {
        Self {
            agent_status: HashMap::new(),
            current_epoch: 0,
            window_duration_cycles,
            challenges_required,
            window_start_cycle: 0,
        }
    }

    /// Register a new agent
    pub fn register_agent(&mut self, agent_id: &str) {
        self.agent_status
            .entry(agent_id.to_string())
            .or_insert(AgentProvingStatus {
                agent_id: agent_id.to_string(),
                windows: Vec::new(),
                consecutive_passes: 0,
                total_passes: 0,
                last_failed_epoch: None,
                current_challenge_count: 0,
            });
    }

    /// Tick the proving window system. Call this every cycle.
    /// Returns true if a new window just started.
    pub fn tick(&mut self, current_cycle: u64) -> bool {
        let cycles_since_start = current_cycle - self.window_start_cycle;
        if cycles_since_start >= self.window_duration_cycles as u64 {
            // Close current window for all agents
            self.close_window();
            // Start new window
            self.current_epoch += 1;
            self.window_start_cycle = current_cycle;
            true
        } else {
            false
        }
    }

    /// Record a successful challenge for an agent
    pub fn record_challenge(&mut self, agent_id: &str) -> Result<(), String> {
        let status = self
            .agent_status
            .get_mut(agent_id)
            .ok_or_else(|| format!("Agent {} not registered", agent_id))?;
        status.current_challenge_count += 1;
        Ok(())
    }

    /// Check if an agent has passed the current window
    pub fn has_passed(&self, agent_id: &str) -> bool {
        self.agent_status
            .get(agent_id)
            .map(|s| s.current_challenge_count >= self.challenges_required)
            .unwrap_or(false)
    }

    /// Close the current window: evaluate each agent
    fn close_window(&mut self) {
        let epoch = self.current_epoch;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        for (_, status) in self.agent_status.iter_mut() {
            let passed = status.current_challenge_count >= self.challenges_required;

            status.windows.push(ProvingWindow {
                epoch,
                started_at: now,
                duration_ns: self.window_duration_cycles as u64 * 1_000_000_000, // ~1s per cycle
                required_challenges: self.challenges_required,
                challenges_completed: status.current_challenge_count,
                passed,
            });

            if passed {
                status.consecutive_passes += 1;
                status.total_passes += 1;
            } else {
                status.consecutive_passes = 0;
                status.last_failed_epoch = Some(epoch);
            }

            status.current_challenge_count = 0;
        }
    }

    /// Get proving status for an agent
    pub fn agent_summary(&self, agent_id: &str) -> String {
        match self.agent_status.get(agent_id) {
            Some(s) => format!(
                "Agent {}: {} consecutive passes, {} total, last_fail={:?}",
                agent_id, s.consecutive_passes, s.total_passes, s.last_failed_epoch
            ),
            None => format!("Agent {}: not registered", agent_id),
        }
    }

    /// Overall proving health (0.0-1.0)
    pub fn proving_health(&self) -> f64 {
        let total = self.agent_status.len();
        if total == 0 {
            return 1.0;
        }
        let passed = self
            .agent_status
            .values()
            .filter(|s| {
                s.current_challenge_count >= self.challenges_required || s.consecutive_passes > 0
            })
            .count();
        passed as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_registration_and_challenge() {
        let mut mgr = ProvingWindowManager::new(10, 3);
        mgr.register_agent("agent-a");
        mgr.register_agent("agent-b");

        assert_eq!(mgr.agent_status.len(), 2);
        assert!(!mgr.has_passed("agent-a"));

        mgr.record_challenge("agent-a").unwrap();
        mgr.record_challenge("agent-a").unwrap();
        mgr.record_challenge("agent-a").unwrap();
        assert!(mgr.has_passed("agent-a"));

        // agent-b has no challenges
        assert!(!mgr.has_passed("agent-b"));
    }

    #[test]
    fn test_window_tick_triggers_evaluation() {
        let mut mgr = ProvingWindowManager::new(5, 2);
        mgr.register_agent("agent-a");

        mgr.record_challenge("agent-a").unwrap();
        mgr.record_challenge("agent-a").unwrap();

        // Before window closes
        assert_eq!(mgr.current_epoch, 0);
        assert_eq!(mgr.agent_status.get("agent-a").unwrap().windows.len(), 0);

        // Tick at cycle 5 — closes window 0, opens window 1
        let new_window = mgr.tick(5);
        assert!(new_window);
        assert_eq!(mgr.current_epoch, 1);
        assert_eq!(mgr.agent_status.get("agent-a").unwrap().windows.len(), 1);
        assert!(mgr.agent_status.get("agent-a").unwrap().windows[0].passed);
    }

    #[test]
    fn test_insufficient_challenges_fails_window() {
        let mut mgr = ProvingWindowManager::new(5, 3);
        mgr.register_agent("agent-a");

        mgr.record_challenge("agent-a").unwrap(); // only 1 out of 3 required

        mgr.tick(5);

        let agent = mgr.agent_status.get("agent-a").unwrap();
        assert!(!agent.windows[0].passed);
        assert_eq!(agent.consecutive_passes, 0);
        assert_eq!(agent.last_failed_epoch, Some(0));
    }

    #[test]
    fn test_proving_health_calculation() {
        let mut mgr = ProvingWindowManager::new(5, 2);
        mgr.register_agent("good-agent");
        mgr.register_agent("bad-agent");

        mgr.record_challenge("good-agent").unwrap();
        mgr.record_challenge("good-agent").unwrap();

        // good-agent has passed current window, bad-agent has not
        let health = mgr.proving_health();
        assert!((health - 0.5).abs() < 0.01, "expected 0.5, got {}", health);

        // After tick, good-agent's pass is recorded in window history
        mgr.tick(5);

        // proving_health checks either current OR consecutive>0
        let health_after = mgr.proving_health();
        assert!(
            (health_after - 0.5).abs() < 0.01,
            "expected 0.5, got {}",
            health_after
        );
    }

    #[test]
    fn test_unregistered_agent_challenge_fails() {
        let mut mgr = ProvingWindowManager::new(10, 1);
        let result = mgr.record_challenge("unknown");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not registered"));
    }

    #[test]
    fn test_agent_summary_output() {
        let mut mgr = ProvingWindowManager::new(5, 1);
        mgr.register_agent("alice");

        let summary = mgr.agent_summary("alice");
        assert!(summary.contains("alice"));
        assert!(summary.contains("0 consecutive"));

        mgr.record_challenge("alice").unwrap();
        mgr.tick(5);

        let summary = mgr.agent_summary("alice");
        assert!(summary.contains("1 consecutive"));

        let unknown = mgr.agent_summary("bob");
        assert!(unknown.contains("not registered"));
    }
}
