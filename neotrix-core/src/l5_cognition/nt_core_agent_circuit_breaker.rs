//! Agent Circuit Breaker — steer→constrain→stop 三级行为控制
//!
//! 防止 agent 循环/失控/超预算。不同于 service circuit breaker (Closed/Open/HalfOpen)，
//! 这是 agent-level 行为熔断：steer(引导修正) → constrain(约束范围) → stop(完全停止)。
//!
//! 设计启发: Munder Difflin circuit breaker + Cumora spend gate

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Agent Circuit Breaker ──────────────────────────────────────────────────

/// Agent-level circuit breaker — 三级行为控制
pub struct AgentCircuitBreaker {
    /// Per-agent breaker state
    agents: HashMap<String, AgentBreakerState>,
    /// Global config
    config: BreakerConfig,
}

/// Config for agent circuit breaker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakerConfig {
    /// Consecutive tool failures before steer
    pub steer_threshold: u32,
    /// Consecutive failures before constrain
    pub constrain_threshold: u32,
    /// Consecutive failures before stop
    pub stop_threshold: u32,
    /// Max cost (USD) before auto-stop
    pub max_cost: f64,
    /// Max tool rounds per message before constrain
    pub max_tool_rounds: u32,
    /// Cooldown after stop (seconds)
    pub stop_cooldown_secs: u64,
}

impl Default for BreakerConfig {
    fn default() -> Self {
        Self {
            steer_threshold: 3,
            constrain_threshold: 5,
            stop_threshold: 8,
            max_cost: 10.0,
            max_tool_rounds: 10,
            stop_cooldown_secs: 300, // 5 minutes
        }
    }
}

/// Per-agent breaker state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBreakerState {
    /// Agent ID
    pub agent_id: String,
    /// Current level
    pub level: BreakerLevel,
    /// Consecutive tool failures
    pub consecutive_failures: u32,
    /// Tool rounds in current message
    pub current_tool_rounds: u32,
    /// Session cost so far (USD)
    pub session_cost: f64,
    /// Total cost today (USD)
    pub daily_cost: f64,
    /// When the agent was stopped (epoch secs)
    pub stopped_at: Option<u64>,
    /// Reason for current level
    pub reason: String,
    /// History of level transitions
    pub transitions: Vec<BreakerTransition>,
}

/// Breaker level — 三级行为控制
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakerLevel {
    /// Normal operation — agent runs freely
    Normal,
    /// Steer — agent is guided to change approach (e.g. retry with different strategy)
    Steer,
    /// Constrain — agent is restricted to read-only / limited tools
    Constrain,
    /// Stop — agent is completely halted, requires human intervention
    Stop,
}

impl BreakerLevel {
    /// Whether the agent can still execute tools
    pub fn allows_tools(&self) -> bool {
        matches!(self, BreakerLevel::Normal | BreakerLevel::Steer)
    }

    /// Whether the agent can write/modify state
    pub fn allows_writes(&self) -> bool {
        matches!(self, BreakerLevel::Normal)
    }

    /// Whether the agent is fully halted
    pub fn is_stopped(&self) -> bool {
        matches!(self, BreakerLevel::Stop)
    }

    /// Numeric severity (higher = more restrictive)
    pub fn severity(&self) -> u8 {
        match self {
            BreakerLevel::Normal => 0,
            BreakerLevel::Steer => 1,
            BreakerLevel::Constrain => 2,
            BreakerLevel::Stop => 3,
        }
    }
}

/// Record of a level transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakerTransition {
    pub from: BreakerLevel,
    pub to: BreakerLevel,
    pub reason: String,
    pub timestamp: u64,
}

/// Verdict from the circuit breaker check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BreakerVerdict {
    /// Agent can proceed normally
    Proceed,
    /// Agent should be steered (try different approach)
    Steer { suggestion: String },
    /// Agent should be constrained (read-only / limited tools)
    Constrain { allowed_tools: Vec<String> },
    /// Agent must stop immediately
    Stop { reason: String, cooldown_secs: u64 },
}

// ─── Free transition helper (avoids borrow conflict) ────────────────────────

fn apply_transition(state: &mut AgentBreakerState, to: BreakerLevel, reason: &str, cooldown: u64) -> BreakerVerdict {
    let from = state.level;
    if from == to {
        return match to {
            BreakerLevel::Normal => BreakerVerdict::Proceed,
            BreakerLevel::Steer => BreakerVerdict::Steer { suggestion: reason.to_string() },
            BreakerLevel::Constrain => BreakerVerdict::Constrain {
                allowed_tools: vec![
                    "read_file".to_string(),
                    "list_dir".to_string(),
                    "search_code".to_string(),
                    "grep".to_string(),
                ],
            },
            BreakerLevel::Stop => BreakerVerdict::Stop { reason: reason.to_string(), cooldown_secs: cooldown },
        };
    }
    state.level = to;
    state.reason = reason.to_string();
    if to == BreakerLevel::Stop {
        state.stopped_at = Some(now_secs());
    }
    state.transitions.push(BreakerTransition {
        from,
        to,
        reason: reason.to_string(),
        timestamp: now_secs(),
    });
    match to {
        BreakerLevel::Normal => BreakerVerdict::Proceed,
        BreakerLevel::Steer => BreakerVerdict::Steer { suggestion: reason.to_string() },
        BreakerLevel::Constrain => BreakerVerdict::Constrain {
            allowed_tools: vec![
                "read_file".to_string(),
                "list_dir".to_string(),
                "search_code".to_string(),
                "grep".to_string(),
            ],
        },
        BreakerLevel::Stop => BreakerVerdict::Stop { reason: reason.to_string(), cooldown_secs: cooldown },
    }
}

// ─── Implementation ─────────────────────────────────────────────────────────

impl AgentCircuitBreaker {
    pub fn new(config: BreakerConfig) -> Self {
        Self {
            agents: HashMap::new(),
            config,
        }
    }

    /// Ensure agent entry exists, return mutable ref
    fn ensure_agent(&mut self, agent_id: &str) -> &mut AgentBreakerState {
        self.agents
            .entry(agent_id.to_string())
            .or_insert_with(|| AgentBreakerState {
                agent_id: agent_id.to_string(),
                level: BreakerLevel::Normal,
                consecutive_failures: 0,
                current_tool_rounds: 0,
                session_cost: 0.0,
                daily_cost: 0.0,
                stopped_at: None,
                reason: String::new(),
                transitions: Vec::new(),
            })
    }

    /// Check agent state before allowing the next action
    pub fn check(&mut self, agent_id: &str, cost: f64) -> BreakerVerdict {
        self.ensure_agent(agent_id);
        // Extract what we need before mutable borrow
        let session_cost = self.agents[agent_id].session_cost + cost;
        let max_cost = self.config.max_cost;
        let stop_cooldown = self.config.stop_cooldown_secs;

        let state = self.agents.get_mut(agent_id).unwrap();
        state.session_cost = session_cost;

        // Check cost limit
        if session_cost >= max_cost {
            let reason = format!(
                "Session cost ${:.2} exceeds limit ${:.2}",
                session_cost, max_cost
            );
            return apply_transition(state, BreakerLevel::Stop, &reason, stop_cooldown);
        }

        let level = state.level;
        let reason_str = state.reason.clone();
        match level {
            BreakerLevel::Normal => BreakerVerdict::Proceed,
            BreakerLevel::Steer => BreakerVerdict::Steer { suggestion: reason_str },
            BreakerLevel::Constrain => BreakerVerdict::Constrain {
                allowed_tools: vec![
                    "read_file".to_string(),
                    "list_dir".to_string(),
                    "search_code".to_string(),
                    "grep".to_string(),
                ],
            },
            BreakerLevel::Stop => BreakerVerdict::Stop {
                reason: reason_str,
                cooldown_secs: stop_cooldown,
            },
        }
    }

    /// Record a tool failure
    pub fn record_failure(&mut self, agent_id: &str, reason: &str) -> BreakerVerdict {
        self.ensure_agent(agent_id);

        // Read thresholds and current state
        let (failures, rounds, steer_t, constrain_t, stop_t, max_rounds) = {
            let s = &self.agents[agent_id];
            (
                s.consecutive_failures + 1,
                s.current_tool_rounds + 1,
                self.config.steer_threshold,
                self.config.constrain_threshold,
                self.config.stop_threshold,
                self.config.max_tool_rounds,
            )
        };

        // Determine target level
        let target = if failures >= stop_t {
            BreakerLevel::Stop
        } else if failures >= constrain_t {
            BreakerLevel::Constrain
        } else if failures >= steer_t {
            BreakerLevel::Steer
        } else if rounds >= max_rounds {
            BreakerLevel::Constrain
        } else {
            BreakerLevel::Normal
        };

        let stop_cooldown = self.config.stop_cooldown_secs;
        let state = self.agents.get_mut(agent_id).unwrap();
        state.consecutive_failures = failures;
        state.current_tool_rounds = rounds;

        let msg = if failures >= stop_t {
            format!("{} consecutive failures (stop threshold)", failures)
        } else if failures >= constrain_t {
            format!("{} consecutive failures (constrain threshold)", failures)
        } else if failures >= steer_t {
            format!("{} consecutive failures (steer threshold): {}", failures, reason)
        } else if rounds >= max_rounds {
            format!("{} tool rounds in one message (max {})", rounds, max_rounds)
        } else {
            format!("failure: {}", reason)
        };

        apply_transition(state, target, &msg, stop_cooldown)
    }

    /// Record a tool success — resets consecutive failures
    pub fn record_success(&mut self, agent_id: &str) {
        if let Some(state) = self.agents.get_mut(agent_id) {
            state.consecutive_failures = 0;
            state.current_tool_rounds += 1;
        }
    }

    /// Record message boundary — resets tool rounds counter
    pub fn record_message_boundary(&mut self, agent_id: &str) {
        if let Some(state) = self.agents.get_mut(agent_id) {
            state.current_tool_rounds = 0;
        }
    }

    /// Human override — reset agent to Normal
    pub fn human_override(&mut self, agent_id: &str) {
        let stop_cooldown = self.config.stop_cooldown_secs;
        if let Some(state) = self.agents.get_mut(agent_id) {
            apply_transition(state, BreakerLevel::Normal, "Human override", stop_cooldown);
            state.stopped_at = None;
        }
    }

    /// Get current state for an agent
    pub fn state(&self, agent_id: &str) -> Option<&AgentBreakerState> {
        self.agents.get(agent_id)
    }

    /// Get all agent states
    pub fn all_states(&self) -> &HashMap<String, AgentBreakerState> {
        &self.agents
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

use crate::l0_substrate::nt_core_time::now_secs;

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_operation() {
        let mut cb = AgentCircuitBreaker::new(BreakerConfig::default());
        let v = cb.check("agent-1", 0.0);
        assert!(matches!(v, BreakerVerdict::Proceed));
    }

    #[test]
    fn test_steer_after_failures() {
        let mut cb = AgentCircuitBreaker::new(BreakerConfig {
            steer_threshold: 2,
            constrain_threshold: 4,
            stop_threshold: 6,
            ..Default::default()
        });

        cb.record_failure("a1", "timeout");
        let v = cb.record_failure("a1", "timeout");
        assert!(matches!(v, BreakerVerdict::Steer { .. }));
    }

    #[test]
    fn test_constrain_after_more_failures() {
        let mut cb = AgentCircuitBreaker::new(BreakerConfig {
            steer_threshold: 2,
            constrain_threshold: 3,
            stop_threshold: 5,
            ..Default::default()
        });

        cb.record_failure("a1", "err");
        cb.record_failure("a1", "err");
        let v = cb.record_failure("a1", "err");
        assert!(matches!(v, BreakerVerdict::Constrain { .. }));
    }

    #[test]
    fn test_stop_after_threshold() {
        let mut cb = AgentCircuitBreaker::new(BreakerConfig {
            steer_threshold: 2,
            constrain_threshold: 4,
            stop_threshold: 3,
            ..Default::default()
        });

        cb.record_failure("a1", "err");
        cb.record_failure("a1", "err");
        let v = cb.record_failure("a1", "err");
        assert!(matches!(v, BreakerVerdict::Stop { .. }));
    }

    #[test]
    fn test_success_resets_failures() {
        let mut cb = AgentCircuitBreaker::new(BreakerConfig {
            steer_threshold: 3,
            ..Default::default()
        });

        cb.record_failure("a1", "err");
        cb.record_failure("a1", "err");
        cb.record_success("a1"); // resets
        let v = cb.record_failure("a1", "err");
        assert!(matches!(v, BreakerVerdict::Proceed));
    }

    #[test]
    fn test_cost_limit_triggers_stop() {
        let mut cb = AgentCircuitBreaker::new(BreakerConfig {
            max_cost: 5.0,
            ..Default::default()
        });

        let v = cb.check("a1", 3.0);
        assert!(matches!(v, BreakerVerdict::Proceed));

        let v = cb.check("a1", 3.0); // total = 6.0 > 5.0
        assert!(matches!(v, BreakerVerdict::Stop { .. }));
    }

    #[test]
    fn test_human_override() {
        let mut cb = AgentCircuitBreaker::new(BreakerConfig {
            stop_threshold: 2,
            ..Default::default()
        });

        cb.record_failure("a1", "err");
        cb.record_failure("a1", "err");
        assert!(cb.state("a1").unwrap().level == BreakerLevel::Stop);

        cb.human_override("a1");
        assert!(cb.state("a1").unwrap().level == BreakerLevel::Normal);
    }

    #[test]
    fn test_tool_rounds_limit() {
        let mut cb = AgentCircuitBreaker::new(BreakerConfig {
            max_tool_rounds: 3,
            steer_threshold: 100,
            constrain_threshold: 100,
            stop_threshold: 100,
            ..Default::default()
        });

        cb.record_failure("a1", "err"); // round 1
        cb.record_failure("a1", "err"); // round 2
        let v = cb.record_failure("a1", "err"); // round 3 → constrain
        assert!(matches!(v, BreakerVerdict::Constrain { .. }));
    }

    #[test]
    fn test_message_boundary_resets_rounds() {
        let mut cb = AgentCircuitBreaker::new(BreakerConfig {
            max_tool_rounds: 3,
            steer_threshold: 100,
            constrain_threshold: 100,
            stop_threshold: 100,
            ..Default::default()
        });

        cb.record_failure("a1", "err");
        cb.record_failure("a1", "err");
        cb.record_message_boundary("a1"); // reset rounds
        let v = cb.record_failure("a1", "err"); // round 1 again
        assert!(matches!(v, BreakerVerdict::Proceed));
    }

    #[test]
    fn test_level_severity_ordering() {
        assert!(BreakerLevel::Normal.severity() < BreakerLevel::Steer.severity());
        assert!(BreakerLevel::Steer.severity() < BreakerLevel::Constrain.severity());
        assert!(BreakerLevel::Constrain.severity() < BreakerLevel::Stop.severity());
    }
}
