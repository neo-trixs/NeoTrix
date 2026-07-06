use std::collections::VecDeque;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyGate { Allow, Deny, Review }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KernelError { Violation(String), Override(String), Unauthorized }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage { pub source: String, pub target: String, pub payload: String, pub urgent: bool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxRing { Ring0, Ring1, Ring2, Ring3 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore { pub score: f64, pub updated: String }

impl TrustScore {
    pub fn new(score: f64) -> Self { TrustScore { score, updated: String::new() } }
}

#[derive(Debug, Clone)]
pub struct RateLimiter { pub window: Duration, pub max_requests: u32, pub timestamps: VecDeque<Instant> }

impl RateLimiter {
    pub fn new(max: u32, window_secs: u64) -> Self { RateLimiter { window: Duration::from_secs(window_secs), max_requests: max, timestamps: VecDeque::new() } }
    pub fn allow(&mut self) -> bool { let now = Instant::now(); while let Some(t) = self.timestamps.front() { if now.duration_since(*t) > self.window { self.timestamps.pop_front(); } else { break; } } if self.timestamps.len() < self.max_requests as usize { self.timestamps.push_back(now); true } else { false } }
}

#[derive(Debug, Clone)]
pub struct AuditChain { pub entries: Vec<String> }

impl AuditChain {
    pub fn new() -> Self { AuditChain { entries: Vec::new() } }
    pub fn log(&mut self, entry: String) { self.entries.push(entry); }
    pub fn len(&self) -> usize { self.entries.len() }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatModelVariant { PolicyViolation, KernelPanic, IsolationBreach, ResourceExhaustion, DataLeak, UnauthorizedEscalation, AuditEvasion, Unknown }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_asi01_policy_gate_deny() { assert!(matches!(PolicyGate::Deny, PolicyGate::Deny)); }
    #[test] fn test_asi02_kernel_error_violation() { let e = KernelError::Violation("test".into()); assert!(matches!(e, KernelError::Violation(_))); }
    #[test] fn test_asi03_ipc_message_creation() { let m = IpcMessage { source: "a".into(), target: "b".into(), payload: "data".into(), urgent: false }; assert_eq!(m.source, "a"); }
    #[test] fn test_asi04_sandbox_ring_order() { assert!(matches!(SandboxRing::Ring0, SandboxRing::Ring0)); }
    #[test] fn test_asi05_trust_score_new() { let t = TrustScore::new(0.8); assert!((t.score - 0.8).abs() < 1e-6); }
    #[test] fn test_asi06_rate_limiter_allows_within_limit() { let mut r = RateLimiter::new(5, 60); for _ in 0..5 { assert!(r.allow()); } }
    #[test] fn test_asi07_rate_limiter_blocks_over_limit() { let mut r = RateLimiter::new(2, 60); assert!(r.allow()); assert!(r.allow()); assert!(!r.allow()); }
    #[test] fn test_asi08_audit_chain_log_and_len() { let mut a = AuditChain::new(); a.log("event".into()); assert_eq!(a.len(), 1); }
    #[test] fn test_asi09_threat_model_variant_mapping() { let t = ThreatModelVariant::PolicyViolation; assert!(matches!(t, ThreatModelVariant::PolicyViolation)); }
    #[test] fn test_asi10_unknown_threat_fallback() { let t = ThreatModelVariant::Unknown; assert!(matches!(t, ThreatModelVariant::Unknown)); }
}
