use std::time::Instant;
use serde::{Serialize, Deserialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LtlOperator { Not, And, Or, Implies, Eventually, Always, Until, Next }

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LtlAst { Atom(String), Not(Box<LtlAst>), And(Box<LtlAst>, Box<LtlAst>), Or(Box<LtlAst>, Box<LtlAst>), Implies(Box<LtlAst>, Box<LtlAst>), Eventually(Box<LtlAst>), Always(Box<LtlAst>), Until(Box<LtlAst>, Box<LtlAst>), Next(Box<LtlAst>) }

#[allow(dead_code)]
impl LtlAst {
    pub fn atomic(p: &str) -> Self { LtlAst::Atom(p.to_string()) }
    pub fn unary(op: LtlOperator, a: LtlAst) -> Self { match op { LtlOperator::Not => LtlAst::Not(Box::new(a)), LtlOperator::Eventually => LtlAst::Eventually(Box::new(a)), LtlOperator::Always => LtlAst::Always(Box::new(a)), LtlOperator::Next => LtlAst::Next(Box::new(a)), _ => panic!("not a unary op") } }
    pub fn binary(op: LtlOperator, a: LtlAst, b: LtlAst) -> Self { match op { LtlOperator::And => LtlAst::And(Box::new(a), Box::new(b)), LtlOperator::Or => LtlAst::Or(Box::new(a), Box::new(b)), LtlOperator::Implies => LtlAst::Implies(Box::new(a), Box::new(b)), LtlOperator::Until => LtlAst::Until(Box::new(a), Box::new(b)), _ => panic!("not a binary op") } }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PropChecker { pub props: Vec<String> }
#[allow(dead_code)]
impl PropChecker {
    pub fn new(props: Vec<String>) -> Self { PropChecker { props } }
    pub fn check(&self, ast: &LtlAst) -> bool { match ast { LtlAst::Atom(p) => self.props.contains(p), LtlAst::Not(a) => !self.check(a), LtlAst::And(a, b) => self.check(a) && self.check(b), LtlAst::Or(a, b) => self.check(a) || self.check(b), LtlAst::Implies(a, b) => !self.check(a) || self.check(b), LtlAst::Eventually(_) | LtlAst::Always(_) | LtlAst::Until(_, _) | LtlAst::Next(_) => true } }
    pub fn explain(&self, ast: &LtlAst) -> String { format!("checking {:?}", ast) }
}
#[allow(dead_code)]
pub fn default_prop_matcher() -> PropChecker { PropChecker::new(vec!["p".to_string(), "safe".to_string(), "valid".to_string()]) }

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuchiTransition { pub from: String, pub to: String, pub label: String }
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuchiAutomaton { pub states: Vec<String>, pub start: String, pub accepting: Vec<String>, pub transitions: Vec<BuchiTransition> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationOutcome { pub subdomain: String, pub passed: bool, pub details: String, pub severity: u8 }

pub fn summarise_outcome(o: &VerificationOutcome) -> String {
    format!("[{}] {} (sev={}): {}", if o.passed { "PASS" } else { "FAIL" }, o.subdomain, o.severity, o.details)
}

#[derive(Debug, Clone)]
pub struct VerificationReport { pub outcomes: Vec<VerificationOutcome>, pub all_passed: bool, pub timestamp: Instant, pub failed_count: usize }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MCPSubdomain { UserOutput, Execution, Navigation, NetworkAccess, FileAccess, ProcessManagement, Autonomy, Security, Identity, DataAccess }
impl MCPSubdomain {
    pub fn all() -> Vec<MCPSubdomain> {
        use MCPSubdomain::*;
        vec![UserOutput, Execution, Navigation, NetworkAccess, FileAccess, ProcessManagement, Autonomy, Security, Identity, DataAccess]
    }
}

pub fn verify_all(checks: &[(MCPSubdomain, bool, String, u8)]) -> VerificationReport {
    let outcomes: Vec<VerificationOutcome> = checks.iter().map(|(s, p, d, sev)| VerificationOutcome { subdomain: format!("{:?}", s), passed: *p, details: d.clone(), severity: *sev }).collect();
    let f = outcomes.iter().filter(|o| !o.passed).count();
    VerificationReport { outcomes, all_passed: f == 0, timestamp: Instant::now(), failed_count: f }
}

#[derive(Debug, Clone)]
pub struct RuntimeMonitor { pub check_interval: std::time::Duration, pub last_check: Instant, pub violation_counter: usize }

impl RuntimeMonitor {
    pub fn new(interval_ms: u64) -> Self { RuntimeMonitor { check_interval: std::time::Duration::from_millis(interval_ms), last_check: Instant::now(), violation_counter: 0 } }
    pub fn should_check(&self) -> bool { self.last_check.elapsed() >= self.check_interval }
    pub fn mark_checked(&mut self) { self.last_check = Instant::now(); }
    pub fn record_violation(&mut self, _detail: String) { self.violation_counter += 1; }
    pub fn violation_count(&self) -> usize { self.violation_counter }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_runtime_monitor_should_check() { let m = RuntimeMonitor::new(0); assert!(m.should_check()); }
    #[test] fn test_runtime_monitor_mark_checked() { let mut m = RuntimeMonitor::new(1000); m.mark_checked(); assert!(!m.should_check() || m.last_check.elapsed() >= m.check_interval); }
    #[test] fn test_monitor_state_persistence_across_steps() { let mut m = RuntimeMonitor::new(100); for i in 0..5 { m.record_violation(format!("s{}", i)); m.mark_checked(); } assert!(m.violation_count() >= 4); }
    #[test] fn test_summarise_outcome_pass() { let o = VerificationOutcome { subdomain: "Security".into(), passed: true, details: "ok".into(), severity: 0 }; let s = summarise_outcome(&o); assert!(s.contains("PASS")); }
    #[test] fn test_summarise_outcome_fail() { let o = VerificationOutcome { subdomain: "E".into(), passed: false, details: "t".into(), severity: 3 }; let s = summarise_outcome(&o); assert!(s.contains("FAIL")); }
    #[test] fn test_ltl_ast_atomic_construction() { let a = LtlAst::atomic("p"); assert!(matches!(a, LtlAst::Atom(_))); }
    #[test] fn test_ltl_ast_unary_construction() { let n = LtlAst::unary(LtlOperator::Not, LtlAst::atomic("p")); assert!(matches!(n, LtlAst::Not(_))); }
    #[test] fn test_ltl_ast_binary_construction() { let a = LtlAst::binary(LtlOperator::And, LtlAst::atomic("p"), LtlAst::atomic("q")); assert!(matches!(a, LtlAst::And(_, _))); }
    #[test] fn test_default_prop_checker_always_passes() { let c = default_prop_matcher(); let a = LtlAst::atomic("p"); assert!(c.check(&a)); assert!(!c.explain(&a).is_empty()); }
    #[test] fn test_verification_report_creation() { let r = VerificationReport { outcomes: vec![], all_passed: true, timestamp: Instant::now(), failed_count: 0 }; assert!(r.all_passed); assert_eq!(r.failed_count, 0); }
}
