//! # Consciousness Hooks — Lifecycle hook system for ConsciousnessIntegration
//!
//! Inspired by AgentMemory's 12 lifecycle hooks pattern. Provides pre/post
//! hook points around every handler, cycle boundaries, and special events.

use log;
use std::collections::HashMap;
use std::time::Instant;

// ── Hook point ──

/// Points in the consciousness cycle where hooks can fire
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HookPoint {
    CycleStart,
    BeforeHandler(String),
    AfterHandler(String),
    PreDecision,
    PostDecision,
    CycleEnd,
    OnSleep,
    OnWake,
    OnCuriositySpike,
    OnGoalComplete(u64),
    OnError(String),
    OnStagnation,
}

impl HookPoint {
    pub fn label(&self) -> String {
        match self {
            HookPoint::CycleStart => "cycle_start".into(),
            HookPoint::BeforeHandler(name) => format!("before_{}", name),
            HookPoint::AfterHandler(name) => format!("after_{}", name),
            HookPoint::PreDecision => "pre_decision".into(),
            HookPoint::PostDecision => "post_decision".into(),
            HookPoint::CycleEnd => "cycle_end".into(),
            HookPoint::OnSleep => "on_sleep".into(),
            HookPoint::OnWake => "on_wake".into(),
            HookPoint::OnCuriositySpike => "curiosity_spike".into(),
            HookPoint::OnGoalComplete(id) => format!("goal_complete_{}", id),
            HookPoint::OnError(e) => format!("on_error_{}", e),
            HookPoint::OnStagnation => "on_stagnation".into(),
        }
    }
}

// ── Hook action ──

/// The action a hook can return
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum HookAction {
    /// Continue normally
    Continue,
    /// Block with a reason message
    Block(String),
    /// Log a warning but continue
    Warn(String),
    /// Skip the current handler entirely
    Skip,
}

// ── Hook trait ──

/// A single lifecycle hook
pub trait ConsciousnessHook: Send + Sync {
    fn name(&self) -> &'static str;
    fn hook_points(&self) -> Vec<HookPoint>;
    fn execute(&self, point: &HookPoint, cycle: u64) -> HookAction;
}

// ── Registry ──

/// Registry of hooks indexed by hook point
pub struct ConsciousnessHookRegistry {
    hooks: Vec<Box<dyn ConsciousnessHook>>,
    index: HashMap<HookPoint, Vec<usize>>,
}

impl ConsciousnessHookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn register(&mut self, hook: Box<dyn ConsciousnessHook>) {
        self.hooks.push(hook);
        self.rebuild_index();
    }

    fn rebuild_index(&mut self) {
        self.index.clear();
        for (i, hook) in self.hooks.iter().enumerate() {
            for point in hook.hook_points() {
                self.index.entry(point).or_default().push(i);
            }
        }
    }

    /// Run all hooks for a point, collecting results
    pub fn execute_all(&self, point: &HookPoint, cycle: u64) -> Vec<(String, HookAction)> {
        let mut results = Vec::new();
        let Some(indices) = self.index.get(point) else {
            return results;
        };
        for &i in indices {
            if let Some(hook) = self.hooks.get(i) {
                let action = hook.execute(point, cycle);
                results.push((hook.name().to_string(), action));
            }
        }
        results
    }

    /// Run hooks, return first Block reason or None
    pub fn execute_until_block(&self, point: &HookPoint, cycle: u64) -> Option<String> {
        let Some(indices) = self.index.get(point) else {
            return None;
        };
        for &i in indices {
            if let Some(hook) = self.hooks.get(i) {
                match hook.execute(point, cycle) {
                    HookAction::Block(reason) => {
                        return Some(format!("[{}] {}", hook.name(), reason))
                    }
                    HookAction::Warn(msg) => {
                        log::warn!("[hook::warn] [{}] {}", hook.name(), msg);
                    }
                    HookAction::Continue | HookAction::Skip => {}
                }
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.hooks.len()
    }

    pub fn hook_names(&self) -> Vec<String> {
        self.hooks.iter().map(|h| h.name().to_string()).collect()
    }
}

impl Default for ConsciousnessHookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Built-in hook: PerformanceTracer ──

/// Tracks handler duration. Records Instant at BeforeHandler, computes elapsed at AfterHandler.
pub struct PerformanceTracer {
    starts: HashMap<&'static str, Instant>,
    records: Vec<(String, std::time::Duration)>,
}

impl PerformanceTracer {
    pub fn new() -> Self {
        Self {
            starts: HashMap::new(),
            records: Vec::new(),
        }
    }

    pub fn report(&self) -> Vec<(String, std::time::Duration)> {
        let mut sorted = self.records.clone();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted
    }
}

impl ConsciousnessHook for PerformanceTracer {
    fn name(&self) -> &'static str {
        "performance_tracer"
    }

    fn hook_points(&self) -> Vec<HookPoint> {
        vec![
            HookPoint::BeforeHandler("".into()),
            HookPoint::AfterHandler("".into()),
        ]
    }

    fn execute(&self, point: &HookPoint, _cycle: u64) -> HookAction {
        match point {
            HookPoint::BeforeHandler(_) | HookPoint::AfterHandler(_) => HookAction::Continue,
            _ => HookAction::Continue,
        }
    }
}

/// Mutable variant for internal use by the pipeline wiring
pub struct PerformanceTracerMut {
    inner: PerformanceTracer,
}

impl PerformanceTracerMut {
    pub fn new() -> Self {
        Self {
            inner: PerformanceTracer::new(),
        }
    }

    pub fn record_start(&mut self, name: &'static str) {
        self.inner.starts.insert(name, Instant::now());
    }

    pub fn record_end(&mut self, name: &'static str) {
        if let Some(start) = self.inner.starts.remove(name) {
            self.inner.records.push((name.to_string(), start.elapsed()));
        }
    }

    pub fn report(&self) -> Vec<(String, std::time::Duration)> {
        self.inner.report()
    }
}

// ── Built-in hook: SafetyGateHook ──

/// Blocks if any handler enters an error loop (>3 errors in last 10 calls per handler).
pub struct SafetyGateHook {
    history: HashMap<String, Vec<bool>>,
}

impl SafetyGateHook {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
        }
    }

    pub fn record_result(&mut self, handler: &str, success: bool) {
        let entry = self.history.entry(handler.to_string()).or_default();
        entry.push(success);
        if entry.len() > 10 {
            entry.remove(0);
        }
    }

    fn error_rate(&self, handler: &str) -> f64 {
        let Some(history) = self.history.get(handler) else {
            return 0.0;
        };
        if history.is_empty() {
            return 0.0;
        }
        let failures = history.iter().filter(|&&s| !s).count();
        failures as f64 / history.len() as f64
    }
}

impl ConsciousnessHook for SafetyGateHook {
    fn name(&self) -> &'static str {
        "safety_gate"
    }

    fn hook_points(&self) -> Vec<HookPoint> {
        vec![
            HookPoint::BeforeHandler("".into()),
            HookPoint::AfterHandler("".into()),
        ]
    }

    fn execute(&self, point: &HookPoint, _cycle: u64) -> HookAction {
        match point {
            HookPoint::BeforeHandler(name) => {
                if *name == "" {
                    return HookAction::Continue;
                }
                let rate = self.error_rate(name);
                if rate > 0.3 {
                    HookAction::Block(format!(
                        "handler '{}' error rate {:.2} exceeds threshold",
                        name, rate
                    ))
                } else {
                    HookAction::Continue
                }
            }
            _ => HookAction::Continue,
        }
    }
}

// ── Built-in hook: CuriosityLogger ──

/// Logs curiosity spikes above threshold.
pub struct CuriosityLogger {
    spikes: Vec<(u64, f64)>,
}

impl CuriosityLogger {
    pub fn new() -> Self {
        Self { spikes: Vec::new() }
    }

    pub fn recent_spikes(&self, n: usize) -> Vec<(u64, f64)> {
        let len = self.spikes.len();
        if len == 0 {
            return Vec::new();
        }
        let start = len.saturating_sub(n);
        self.spikes[start..].to_vec()
    }
}

impl ConsciousnessHook for CuriosityLogger {
    fn name(&self) -> &'static str {
        "curiosity_logger"
    }

    fn hook_points(&self) -> Vec<HookPoint> {
        vec![HookPoint::OnCuriositySpike]
    }

    fn execute(&self, point: &HookPoint, _cycle: u64) -> HookAction {
        match point {
            HookPoint::OnCuriositySpike => HookAction::Continue,
            _ => HookAction::Continue,
        }
    }
}

/// Mutable variant for internal use
pub struct CuriosityLoggerMut {
    inner: CuriosityLogger,
}

impl CuriosityLoggerMut {
    pub fn record_spike(&mut self, cycle: u64, delta: f64) {
        if delta > 0.5 {
            self.inner.spikes.push((cycle, delta));
        }
    }
}

impl CuriosityLoggerMut {
    pub fn new() -> Self {
        Self {
            inner: CuriosityLogger::new(),
        }
    }

    pub fn recent_spikes(&self, n: usize) -> Vec<(u64, f64)> {
        self.inner.recent_spikes(n)
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHook {
        name: &'static str,
        points: Vec<HookPoint>,
        result: HookAction,
    }
    impl TestHook {
        fn new(name: &'static str, points: Vec<HookPoint>, result: HookAction) -> Self {
            Self {
                name,
                points,
                result,
            }
        }
    }
    impl ConsciousnessHook for TestHook {
        fn name(&self) -> &'static str {
            self.name
        }
        fn hook_points(&self) -> Vec<HookPoint> {
            self.points.clone()
        }
        fn execute(&self, _point: &HookPoint, _cycle: u64) -> HookAction {
            self.result.clone()
        }
    }

    #[test]
    fn test_registry_registration_and_execution() {
        let mut reg = ConsciousnessHookRegistry::new();
        assert_eq!(reg.len(), 0);
        reg.register(Box::new(TestHook::new(
            "test_hook",
            vec![HookPoint::CycleStart],
            HookAction::Continue,
        )));
        assert_eq!(reg.len(), 1);
        let results = reg.execute_all(&HookPoint::CycleStart, 0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "test_hook");
        assert!(matches!(results[0].1, HookAction::Continue));
    }

    #[test]
    fn test_performance_tracer_tracking() {
        let mut tracer = PerformanceTracerMut::new();
        tracer.record_start("foo");
        std::thread::sleep(std::time::Duration::from_millis(1));
        tracer.record_end("foo");
        let report = tracer.report();
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].0, "foo");
        assert!(report[0].1.as_nanos() > 0);
    }

    #[test]
    fn test_safety_gate_blocking() {
        let mut gate = SafetyGateHook::new();
        // Record 8 failures out of 10 → error rate 0.8 > 0.3
        for _ in 0..8 {
            gate.record_result("bad_handler", false);
        }
        for _ in 0..2 {
            gate.record_result("bad_handler", true);
        }
        let action = gate.execute(&HookPoint::BeforeHandler("bad_handler".into()), 0);
        assert!(matches!(action, HookAction::Block(_)));
    }

    #[test]
    fn test_safety_gate_allows_low_error_rate() {
        let mut gate = SafetyGateHook::new();
        for _ in 0..2 {
            gate.record_result("good_handler", false);
        }
        for _ in 0..8 {
            gate.record_result("good_handler", true);
        }
        let action = gate.execute(&HookPoint::BeforeHandler("good_handler".into()), 0);
        assert!(matches!(action, HookAction::Continue));
    }

    #[test]
    fn test_curiosity_logger_spike_recording() {
        let mut logger = CuriosityLoggerMut::new();
        logger.record_spike(1, 0.3);
        logger.record_spike(2, 0.6);
        logger.record_spike(3, 0.9);
        let spikes = logger.recent_spikes(10);
        assert_eq!(spikes.len(), 2);
        assert_eq!(spikes[0], (2, 0.6));
        assert_eq!(spikes[1], (3, 0.9));
    }

    #[test]
    fn test_execute_until_block_returns_first_block() {
        let mut reg = ConsciousnessHookRegistry::new();
        reg.register(Box::new(TestHook::new(
            "a",
            vec![HookPoint::CycleStart],
            HookAction::Continue,
        )));
        reg.register(Box::new(TestHook::new(
            "b",
            vec![HookPoint::CycleStart],
            HookAction::Block("reason_b".to_string()),
        )));
        reg.register(Box::new(TestHook::new(
            "c",
            vec![HookPoint::CycleStart],
            HookAction::Block("reason_c".to_string()),
        )));
        let blocked = reg.execute_until_block(&HookPoint::CycleStart, 0);
        assert!(blocked.is_some());
        assert!(blocked.unwrap().contains("reason_b"));
    }

    #[test]
    fn test_hook_point_labels() {
        assert_eq!(HookPoint::CycleStart.label(), "cycle_start");
        assert_eq!(HookPoint::CycleEnd.label(), "cycle_end");
        assert_eq!(HookPoint::BeforeHandler("foo".into()).label(), "before_foo");
        assert_eq!(HookPoint::AfterHandler("bar".into()).label(), "after_bar");
        assert_eq!(HookPoint::PreDecision.label(), "pre_decision");
        assert_eq!(HookPoint::PostDecision.label(), "post_decision");
        assert_eq!(HookPoint::OnSleep.label(), "on_sleep");
        assert_eq!(HookPoint::OnWake.label(), "on_wake");
        assert_eq!(HookPoint::OnCuriositySpike.label(), "curiosity_spike");
        assert_eq!(HookPoint::OnStagnation.label(), "on_stagnation");
    }

    #[test]
    fn test_multi_hook_execution_ordering() {
        let mut reg = ConsciousnessHookRegistry::new();
        reg.register(Box::new(TestHook::new(
            "first",
            vec![HookPoint::CycleStart],
            HookAction::Warn("first_warn".to_string()),
        )));
        reg.register(Box::new(TestHook::new(
            "second",
            vec![HookPoint::CycleStart],
            HookAction::Continue,
        )));
        let results = reg.execute_all(&HookPoint::CycleStart, 0);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "first");
        assert_eq!(results[1].0, "second");
    }

    #[test]
    fn test_empty_registry_returns_continue() {
        let reg = ConsciousnessHookRegistry::new();
        assert!(reg.execute_until_block(&HookPoint::CycleStart, 0).is_none());
        assert!(reg.execute_all(&HookPoint::CycleStart, 0).is_empty());
    }

    #[test]
    fn test_registry_hook_names() {
        let mut reg = ConsciousnessHookRegistry::new();
        reg.register(Box::new(TestHook::new(
            "alpha",
            vec![HookPoint::CycleStart],
            HookAction::Continue,
        )));
        reg.register(Box::new(TestHook::new(
            "beta",
            vec![HookPoint::CycleEnd],
            HookAction::Continue,
        )));
        let names = reg.hook_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"alpha".to_string()));
        assert!(names.contains(&"beta".to_string()));
    }

    #[test]
    fn test_safety_gate_empty_handler() {
        let gate = SafetyGateHook::new();
        let action = gate.execute(&HookPoint::BeforeHandler("".into()), 0);
        assert!(matches!(action, HookAction::Continue));
    }

    #[test]
    fn test_curiosity_logger_no_spikes_below_threshold() {
        let mut logger = CuriosityLoggerMut::new();
        logger.record_spike(1, 0.1);
        logger.record_spike(2, 0.4);
        logger.record_spike(3, 0.5);
        let spikes = logger.recent_spikes(10);
        assert_eq!(spikes.len(), 0);
    }

    #[test]
    fn test_warn_action_does_not_block() {
        let mut reg = ConsciousnessHookRegistry::new();
        reg.register(Box::new(TestHook::new(
            "warn_only",
            vec![HookPoint::CycleStart],
            HookAction::Warn("just a warning".to_string()),
        )));
        let blocked = reg.execute_until_block(&HookPoint::CycleStart, 0);
        assert!(blocked.is_none());
    }

    #[test]
    fn test_skip_action_does_not_block() {
        let mut reg = ConsciousnessHookRegistry::new();
        reg.register(Box::new(TestHook::new(
            "skip_only",
            vec![HookPoint::CycleStart],
            HookAction::Skip,
        )));
        let blocked = reg.execute_until_block(&HookPoint::CycleStart, 0);
        assert!(blocked.is_none());
    }

    #[test]
    fn test_on_error_label() {
        assert_eq!(HookPoint::OnError("crash".into()).label(), "on_error_crash");
    }

    #[test]
    fn test_on_goal_complete_label() {
        assert_eq!(HookPoint::OnGoalComplete(42).label(), "goal_complete_42");
    }
}
