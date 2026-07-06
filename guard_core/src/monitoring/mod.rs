use std::collections::VecDeque;
use std::time::Instant;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPSubdomain { pub name: String, pub allowed: bool, pub rate_limit: u32 }

#[derive(Debug, Clone)]
pub struct CircuitBreaker { pub name: String, pub failures: u32, pub max_failures: u32, pub cooldown_until: Instant }

impl CircuitBreaker {
    pub fn new(name: &str) -> Self { CircuitBreaker { name: name.to_string(), failures: 0, max_failures: 5, cooldown_until: Instant::now() } }
    pub fn record_failure(&mut self) { self.failures += 1; if self.failures >= self.max_failures { self.cooldown_until = Instant::now(); } }
    pub fn is_open(&self) -> bool { self.failures >= self.max_failures }
    pub fn reset(&mut self) { self.failures = 0; }
}

#[derive(Debug, Clone)]
pub struct ToolMtfTracker { pub mtf_count: u32, pub recent_calls: VecDeque<String> }

impl ToolMtfTracker {
    pub fn new() -> Self { ToolMtfTracker { mtf_count: 0, recent_calls: VecDeque::new() } }
    pub fn record_call(&mut self, tool: &str) { self.mtf_count += 1; self.recent_calls.push_back(tool.to_string()); if self.recent_calls.len() > 100 { self.recent_calls.pop_front(); } }
    pub fn mtf_rate(&self) -> f64 { if self.mtf_count == 0 { 0.0 } else { self.recent_calls.len() as f64 / self.mtf_count as f64 } }
}

#[derive(Debug, Clone)]
pub struct ContextPressureGauge { pub max_depth: usize, pub current_depth: usize }

impl ContextPressureGauge {
    pub fn new(max_depth: usize) -> Self { ContextPressureGauge { max_depth, current_depth: 0 } }
    pub fn push(&mut self) { if self.current_depth < self.max_depth { self.current_depth += 1; } }
    pub fn pop(&mut self) { if self.current_depth > 0 { self.current_depth -= 1; } }
    pub fn pressure(&self) -> f64 { self.current_depth as f64 / self.max_depth as f64 }
}

#[derive(Debug, Clone)]
pub struct AgentHealthSnapshot { pub healthy: bool, pub error_rate: f64, pub latency_p99_ms: u64, pub circuit_breakers_open: u32, pub context_pressure: f64, pub mtf_rate: f64 }

impl AgentHealthSnapshot {
    pub fn healthy() -> Self { AgentHealthSnapshot { healthy: true, error_rate: 0.0, latency_p99_ms: 0, circuit_breakers_open: 0, context_pressure: 0.0, mtf_rate: 0.0 } }
    pub fn degraded(error_rate: f64) -> Self { AgentHealthSnapshot { healthy: false, error_rate, latency_p99_ms: 0, circuit_breakers_open: 0, context_pressure: 0.0, mtf_rate: 0.0 } }
}

pub fn health_snapshot(cb: &CircuitBreaker, gauge: &ContextPressureGauge, tracker: &ToolMtfTracker) -> AgentHealthSnapshot {
    AgentHealthSnapshot { healthy: !cb.is_open(), error_rate: if cb.is_open() { 1.0 } else { 0.0 }, latency_p99_ms: 0, circuit_breakers_open: if cb.is_open() { 1 } else { 0 }, context_pressure: gauge.pressure(), mtf_rate: tracker.mtf_rate() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_circuit_breaker_new() { let cb = CircuitBreaker::new("test"); assert!(!cb.is_open()); }
    #[test] fn test_circuit_breaker_trips() { let mut cb = CircuitBreaker::new("t"); for _ in 0..5 { cb.record_failure(); } assert!(cb.is_open()); }
    #[test] fn test_circuit_breaker_reset() { let mut cb = CircuitBreaker::new("t"); for _ in 0..5 { cb.record_failure(); } cb.reset(); assert!(!cb.is_open()); }
    #[test] fn test_tool_mtf_tracker_new() { let t = ToolMtfTracker::new(); assert_eq!(t.mtf_count, 0); }
    #[test] fn test_tool_mtf_tracker_record() { let mut t = ToolMtfTracker::new(); t.record_call("a"); assert_eq!(t.mtf_count, 1); }
    #[test] fn test_context_pressure_gauge_new() { let g = ContextPressureGauge::new(10); assert_eq!(g.current_depth, 0); }
    #[test] fn test_context_pressure_gauge_push_pop() { let mut g = ContextPressureGauge::new(10); g.push(); assert_eq!(g.current_depth, 1); g.pop(); assert_eq!(g.current_depth, 0); }
    #[test] fn test_context_pressure_gauge_max() { let mut g = ContextPressureGauge::new(2); g.push(); g.push(); g.push(); assert_eq!(g.current_depth, 2); }
    #[test] fn test_health_snapshot_healthy() { let cb = CircuitBreaker::new("t"); let g = ContextPressureGauge::new(10); let t = ToolMtfTracker::new(); let s = health_snapshot(&cb, &g, &t); assert!(s.healthy); }
    #[test] fn test_health_snapshot_degraded() { let mut cb = CircuitBreaker::new("t"); for _ in 0..5 { cb.record_failure(); } let g = ContextPressureGauge::new(10); let t = ToolMtfTracker::new(); let s = health_snapshot(&cb, &g, &t); assert!(!s.healthy); }
    #[test] fn test_mcp_subdomain_creation() { let s = MCPSubdomain { name: "test".into(), allowed: true, rate_limit: 10 }; assert!(s.allowed); }
    #[test] fn test_agent_health_snapshot_healthy_ctor() { let s = AgentHealthSnapshot::healthy(); assert!(s.healthy); }
    #[test] fn test_agent_health_snapshot_degraded_ctor() { let s = AgentHealthSnapshot::degraded(0.5); assert!(!s.healthy); assert!((s.error_rate - 0.5).abs() < 1e-6); }
}
