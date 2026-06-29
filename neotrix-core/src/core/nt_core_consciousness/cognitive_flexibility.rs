use std::collections::{HashMap, VecDeque};

use crate::core::nt_core_consciousness::executive_controller::Goal;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_time::unix_now_ms;

const VSA_DIM: usize = 1024;
const PERSEVERATION_THRESHOLD: usize = 5;
const DIMINISHING_RETURN_THRESHOLD: f64 = 0.02;
const CURIOSITY_THRESHOLD: f64 = 0.6;
const MAX_COGNITIVE_LOAD: f64 = 0.9;
const MIN_COOLDOWN_STEPS: u64 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReasoningStrategy {
    Direct,
    Analogical,
    Counterfactual,
    Decomposition,
    Reversal,
    FirstPrinciples,
}

impl ReasoningStrategy {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Direct,
            Self::Analogical,
            Self::Counterfactual,
            Self::Decomposition,
            Self::Reversal,
            Self::FirstPrinciples,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Analogical => "analogical",
            Self::Counterfactual => "counterfactual",
            Self::Decomposition => "decomposition",
            Self::Reversal => "reversal",
            Self::FirstPrinciples => "first_principles",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskSet {
    pub id: String,
    pub focus_vsa: Vec<u8>,
    pub active_goals: Vec<Goal>,
    pub active_memories: Vec<Vec<u8>>,
    pub reasoning_strategy: ReasoningStrategy,
    pub created_at: u64,
    pub last_active_at: u64,
    pub total_steps: u64,
    pub output_history: VecDeque<String>,
    pub progress_history: VecDeque<f64>,
}

impl TaskSet {
    pub fn new(id: impl Into<String>, focus_vsa: Vec<u8>, strategy: ReasoningStrategy) -> Self {
        let now = unix_now_ms();
        Self {
            id: id.into(),
            focus_vsa,
            active_goals: Vec::new(),
            active_memories: Vec::new(),
            reasoning_strategy: strategy,
            created_at: now,
            last_active_at: now,
            total_steps: 0,
            output_history: VecDeque::with_capacity(20),
            progress_history: VecDeque::with_capacity(20),
        }
    }

    pub fn record_step(&mut self, output: String, progress: f64) {
        self.total_steps += 1;
        self.last_active_at = unix_now_ms();
        self.output_history.push_back(output);
        if self.output_history.len() > 20 {
            self.output_history.pop_front();
        }
        self.progress_history.push_back(progress);
        if self.progress_history.len() > 20 {
            self.progress_history.pop_front();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchReason {
    Perseveration,
    DiminishingReturns,
    ExternalInterrupt,
    Curiosity,
    StrategyShift,
}

#[derive(Debug, Clone)]
pub struct SwitchEvent {
    pub from_task_id: String,
    pub to_task_id: String,
    pub reason: SwitchReason,
    pub switching_cost: f64,
    pub timestamp: u64,
}

pub trait ContextualMemory {
    /// Serialize a task's context into an opaque VSA vector for later restoration.
    fn save_context(&mut self, task: &TaskSet) -> Vec<u8>;

    /// Attempt to reconstruct a TaskSet from a saved context vector.
    fn restore_context(&mut self, context: &[u8]) -> Option<TaskSet>;

    /// Remove a saved context by task ID.
    fn forget_context(&mut self, task_id: &str);
}

#[derive(Debug, Clone)]
pub struct VsaContextMemory {
    saved: HashMap<String, Vec<u8>>,
}

impl Default for VsaContextMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl VsaContextMemory {
    pub fn new() -> Self {
        Self {
            saved: HashMap::new(),
        }
    }

    pub fn saved_count(&self) -> usize {
        self.saved.len()
    }

    pub fn contains(&self, task_id: &str) -> bool {
        self.saved.contains_key(task_id)
    }
}

impl ContextualMemory for VsaContextMemory {
    fn save_context(&mut self, task: &TaskSet) -> Vec<u8> {
        let mut ctx = task.focus_vsa.clone();
        for goal in &task.active_goals {
            ctx = QuantizedVSA::xor_bind(&ctx, &goal.description);
        }
        for mem in &task.active_memories {
            ctx = QuantizedVSA::bundle(&[&ctx, mem]);
        }
        let strategy_tag =
            QuantizedVSA::seeded_random(task.reasoning_strategy.name().len() as u64, VSA_DIM);
        ctx = QuantizedVSA::xor_bind(&ctx, &strategy_tag);
        self.saved.insert(task.id.clone(), ctx.clone());
        ctx
    }

    fn restore_context(&mut self, context: &[u8]) -> Option<TaskSet> {
        for (id, saved_ctx) in &self.saved {
            let sim = QuantizedVSA::similarity(saved_ctx, context);
            if sim > 0.7 {
                return Some(TaskSet {
                    id: id.clone(),
                    focus_vsa: saved_ctx.clone(),
                    active_goals: Vec::new(),
                    active_memories: Vec::new(),
                    reasoning_strategy: ReasoningStrategy::Direct,
                    created_at: 0,
                    last_active_at: 0,
                    total_steps: 0,
                    output_history: VecDeque::new(),
                    progress_history: VecDeque::new(),
                });
            }
        }
        None
    }

    fn forget_context(&mut self, task_id: &str) {
        self.saved.remove(task_id);
    }
}

#[derive(Debug, Clone)]
pub struct TaskSwitcher {
    perseveration_threshold: usize,
    diminishing_return_threshold: f64,
    curiosity_threshold: f64,
    last_switch_reason: Option<SwitchReason>,
    last_switch_at: u64,
    cooldown_steps: u64,
    total_switches: u64,
}

impl Default for TaskSwitcher {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskSwitcher {
    pub fn new() -> Self {
        Self {
            perseveration_threshold: PERSEVERATION_THRESHOLD,
            diminishing_return_threshold: DIMINISHING_RETURN_THRESHOLD,
            curiosity_threshold: CURIOSITY_THRESHOLD,
            last_switch_reason: None,
            last_switch_at: 0,
            cooldown_steps: MIN_COOLDOWN_STEPS,
            total_switches: 0,
        }
    }

    /// Evaluate whether the current task should be switched away from.
    /// Checks (in priority order): external interrupt, perseveration,
    /// diminishing returns, curiosity trigger.
    /// Respects a cooldown after the last switch.
    pub fn should_switch(
        &mut self,
        current_task: &TaskSet,
        novelty_signal: f64,
        external_interrupt: bool,
        now: u64,
    ) -> Option<SwitchReason> {
        if now < self.last_switch_at + self.cooldown_steps {
            return None;
        }

        if external_interrupt {
            self.total_switches += 1;
            self.last_switch_reason = Some(SwitchReason::ExternalInterrupt);
            self.last_switch_at = now;
            return Some(SwitchReason::ExternalInterrupt);
        }

        if self.detect_perseveration(current_task) {
            self.total_switches += 1;
            self.last_switch_reason = Some(SwitchReason::Perseveration);
            self.last_switch_at = now;
            return Some(SwitchReason::Perseveration);
        }

        if self.detect_diminishing_returns(current_task) {
            self.total_switches += 1;
            self.last_switch_reason = Some(SwitchReason::DiminishingReturns);
            self.last_switch_at = now;
            return Some(SwitchReason::DiminishingReturns);
        }

        if novelty_signal > self.curiosity_threshold {
            self.total_switches += 1;
            self.last_switch_reason = Some(SwitchReason::Curiosity);
            self.last_switch_at = now;
            return Some(SwitchReason::Curiosity);
        }

        None
    }

    /// Returns true if the last `perseveration_threshold` outputs are identical,
    /// indicating the system is stuck in a reasoning loop.
    fn detect_perseveration(&self, task: &TaskSet) -> bool {
        if task.output_history.len() < self.perseveration_threshold {
            return false;
        }
        let recent: Vec<&String> = task
            .output_history
            .iter()
            .rev()
            .take(self.perseveration_threshold)
            .collect();
        let first = recent[0];
        recent.iter().all(|o| *o == first)
    }

    /// Returns true if the average progress over the last 3 steps is below threshold,
    /// indicating the current strategy is not making headway.
    fn detect_diminishing_returns(&self, task: &TaskSet) -> bool {
        if task.progress_history.len() < 3 {
            return false;
        }
        let recent: Vec<&f64> = task.progress_history.iter().rev().take(3).collect();
        let sum: f64 = recent.iter().map(|&&v| v).sum();
        let avg = sum / recent.len() as f64;
        avg < self.diminishing_return_threshold
    }

    pub fn total_switches(&self) -> u64 {
        self.total_switches
    }

    pub fn set_perseveration_threshold(&mut self, threshold: usize) {
        self.perseveration_threshold = threshold;
    }
}

#[derive(Debug, Clone)]
pub struct StrategyShifter {
    strategy_sequence: Vec<ReasoningStrategy>,
    position: usize,
}

impl Default for StrategyShifter {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategyShifter {
    pub fn new() -> Self {
        Self {
            strategy_sequence: ReasoningStrategy::all(),
            position: 0,
        }
    }

    /// Propose a reasoning strategy different from the current one.
    /// Cycles through available strategies round-robin.
    pub fn propose_alternative(
        &mut self,
        current: &ReasoningStrategy,
    ) -> Option<ReasoningStrategy> {
        let others: Vec<&ReasoningStrategy> = self
            .strategy_sequence
            .iter()
            .filter(|s| *s != current)
            .collect();
        if others.is_empty() {
            return None;
        }
        let idx = self.position % others.len();
        let proposed = *others[idx];
        self.position = (self.position + 1) % self.strategy_sequence.len();
        Some(proposed)
    }

    pub fn reset(&mut self) {
        self.position = 0;
    }
}

#[derive(Debug, Clone)]
pub struct CognitiveFlexibility {
    pub tasks: HashMap<String, TaskSet>,
    pub active_task_id: Option<String>,
    pub memory: VsaContextMemory,
    pub switcher: TaskSwitcher,
    pub shifter: StrategyShifter,
    pub switch_history: VecDeque<SwitchEvent>,
    max_switch_history: usize,
    pub cognitive_load: f64,
    pub max_cognitive_load: f64,
}

impl Default for CognitiveFlexibility {
    fn default() -> Self {
        Self::new()
    }
}

impl CognitiveFlexibility {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            active_task_id: None,
            memory: VsaContextMemory::new(),
            switcher: TaskSwitcher::new(),
            shifter: StrategyShifter::new(),
            switch_history: VecDeque::with_capacity(100),
            max_switch_history: 100,
            cognitive_load: 0.0,
            max_cognitive_load: MAX_COGNITIVE_LOAD,
        }
    }

    pub fn register_task(&mut self, task: TaskSet) {
        let id = task.id.clone();
        self.tasks.insert(id, task);
    }

    /// Activate a task by ID. If another task is active, its context is saved.
    /// Returns None if the task is already active or cognitive load is too high.
    pub fn activate_task(&mut self, task_id: &str) -> Option<SwitchReason> {
        let now = unix_now_ms();
        if self.cognitive_load >= self.max_cognitive_load {
            return None;
        }
        if let Some(ref current) = self.active_task_id {
            if current == task_id {
                return None;
            }
            if let Some(curr) = self.tasks.get(current) {
                self.memory.save_context(curr);
            }
        }
        let from_id = self.active_task_id.clone();
        self.active_task_id = Some(task_id.to_string());
        let cost = self.compute_switching_cost(&from_id, task_id);
        self.cognitive_load = (self.cognitive_load + cost * 0.3).min(1.0);

        if let Some(ref fid) = from_id {
            let event = SwitchEvent {
                from_task_id: fid.clone(),
                to_task_id: task_id.to_string(),
                reason: SwitchReason::ExternalInterrupt,
                switching_cost: cost,
                timestamp: now,
            };
            self.switch_history.push_back(event);
            while self.switch_history.len() > self.max_switch_history {
                self.switch_history.pop_front();
            }
        }
        Some(SwitchReason::ExternalInterrupt)
    }

    pub fn current_task(&self) -> Option<&TaskSet> {
        self.active_task_id
            .as_ref()
            .and_then(|id| self.tasks.get(id))
    }

    pub fn current_task_mut(&mut self) -> Option<&mut TaskSet> {
        self.active_task_id
            .as_ref()
            .and_then(|id| self.tasks.get_mut(id))
    }

    pub fn update_load(&mut self, additional: f64) {
        self.cognitive_load = (self.cognitive_load + additional).clamp(0.0, 1.0);
    }

    pub fn reduce_load(&mut self, amount: f64) {
        self.cognitive_load = (self.cognitive_load - amount).max(0.0);
    }

    pub fn can_accept_new_task(&self) -> bool {
        self.cognitive_load < self.max_cognitive_load
    }

    /// Evaluate whether the current task should be switched away from,
    /// based on perseveration, diminishing returns, curiosity, or external interrupt.
    pub fn evaluate_switch(
        &mut self,
        novelty_signal: f64,
        external_interrupt: bool,
    ) -> Option<SwitchReason> {
        let now = unix_now_ms();
        let current = match self
            .active_task_id
            .as_ref()
            .and_then(|id| self.tasks.get(id))
        {
            Some(t) => t,
            None => return None,
        };
        let reason = self
            .switcher
            .should_switch(current, novelty_signal, external_interrupt, now);
        if reason.is_some() {
            self.cognitive_load = (self.cognitive_load + 0.1).min(1.0);
        }
        reason
    }

    /// Switch to another registered task by ID for the given reason.
    /// Saves current task context and records a SwitchEvent.
    /// Returns false if the target task does not exist or cognitive load is at capacity.
    pub fn switch_to(&mut self, task_id: &str, reason: SwitchReason) -> bool {
        let now = unix_now_ms();
        if !self.tasks.contains_key(task_id) {
            return false;
        }
        if self.cognitive_load >= self.max_cognitive_load {
            return false;
        }
        let from_id = self.active_task_id.clone();
        if let Some(ref fid) = from_id {
            if fid == task_id {
                return false;
            }
            if let Some(curr) = self.tasks.get_mut(fid) {
                self.memory.save_context(curr);
            }
        }
        let cost = self.compute_switching_cost(&from_id, task_id);
        self.cognitive_load = (self.cognitive_load + cost * 0.3).min(1.0);
        self.active_task_id = Some(task_id.to_string());

        if let Some(ref fid) = from_id {
            let event = SwitchEvent {
                from_task_id: fid.clone(),
                to_task_id: task_id.to_string(),
                reason,
                switching_cost: cost,
                timestamp: now,
            };
            self.switch_history.push_back(event);
            while self.switch_history.len() > self.max_switch_history {
                self.switch_history.pop_front();
            }
        }
        true
    }

    /// Propose an alternative reasoning strategy for the active task.
    pub fn shift_strategy(&mut self) -> Option<ReasoningStrategy> {
        let current_strategy = match self
            .active_task_id
            .as_ref()
            .and_then(|id| self.tasks.get(id))
        {
            Some(t) => t.reasoning_strategy,
            None => return None,
        };
        let proposed = self.shifter.propose_alternative(&current_strategy)?;
        if let Some(ref id) = self.active_task_id.clone() {
            if let Some(task) = self.tasks.get_mut(id) {
                task.reasoning_strategy = proposed;
            }
        }
        Some(proposed)
    }

    /// Manually assign a reasoning strategy to a task.
    pub fn apply_strategy(&mut self, task_id: &str, strategy: ReasoningStrategy) -> bool {
        self.tasks
            .get_mut(task_id)
            .map(|t| {
                t.reasoning_strategy = strategy;
            })
            .is_some()
    }

    pub fn total_switches(&self) -> u64 {
        self.switcher.total_switches()
    }

    pub fn recent_switches(&self, n: usize) -> Vec<&SwitchEvent> {
        let n = n.min(self.switch_history.len());
        self.switch_history.iter().rev().take(n).collect()
    }

    /// Compute switching cost between two tasks based on VSA similarity of
    /// their focus vectors. Higher similarity = lower cost.
    fn compute_switching_cost(&self, from: &Option<String>, to: &str) -> f64 {
        match from {
            Some(f) => {
                let sim = match (self.tasks.get(f), self.tasks.get(to)) {
                    (Some(a), Some(b)) => QuantizedVSA::similarity(&a.focus_vsa, &b.focus_vsa),
                    _ => 0.0,
                };
                1.0 - sim
            }
            None => 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(id: &str, strategy: ReasoningStrategy, seed: u64) -> TaskSet {
        let focus = QuantizedVSA::seeded_random(seed, VSA_DIM);
        TaskSet::new(id, focus, strategy)
    }

    #[test]
    fn test_perseveration_detection() {
        let mut flex = CognitiveFlexibility::new();
        let mut task = make_task("t1", ReasoningStrategy::Direct, 42);
        for _ in 0..6 {
            task.record_step("same output".to_string(), 0.5);
        }
        flex.register_task(task);
        flex.activate_task("t1");

        let should_switch = flex.evaluate_switch(0.0, false);
        assert_eq!(should_switch, Some(SwitchReason::Perseveration));
    }

    #[test]
    fn test_task_switch_preserves_context() {
        let mut flex = CognitiveFlexibility::new();
        let task_a = make_task("a", ReasoningStrategy::Direct, 1);
        let task_b = make_task("b", ReasoningStrategy::Analogical, 2);
        flex.register_task(task_a);
        flex.register_task(task_b);

        flex.activate_task("a");
        flex.switch_to("b", SwitchReason::Curiosity);

        let a_saved = flex.memory.contains("a");
        assert!(a_saved);
        assert!(flex.active_task_id.as_ref().map(|s| s.as_str()) == Some("b"));

        flex.switch_to("a", SwitchReason::ExternalInterrupt);
        assert_eq!(flex.active_task_id.as_ref().map(|s| s.as_str()), Some("a"));
    }

    #[test]
    fn test_strategy_shifting() {
        let mut flex = CognitiveFlexibility::new();
        let mut task = make_task("t1", ReasoningStrategy::Decomposition, 7);
        task.record_step("stuck".to_string(), 0.01);
        task.record_step("stuck".to_string(), 0.01);
        task.record_step("stuck".to_string(), 0.01);
        flex.register_task(task);
        flex.activate_task("t1");

        let new_strategy = flex.shift_strategy();
        assert!(new_strategy.is_some());
        assert_ne!(new_strategy.unwrap(), ReasoningStrategy::Decomposition);

        let current = flex.current_task().unwrap();
        assert_ne!(current.reasoning_strategy, ReasoningStrategy::Decomposition);
    }

    #[test]
    fn test_cognitive_load_caps() {
        let mut flex = CognitiveFlexibility::new();
        let task_a = make_task("a", ReasoningStrategy::Direct, 1);
        let task_b = make_task("b", ReasoningStrategy::Analogical, 2);
        flex.register_task(task_a);
        flex.register_task(task_b);

        flex.cognitive_load = 0.95;
        assert!(!flex.can_accept_new_task());

        let result = flex.activate_task("b");
        assert!(result.is_none());
    }

    #[test]
    fn test_curiosity_triggers_exploration() {
        let mut flex = CognitiveFlexibility::new();
        let task_a = make_task("a", ReasoningStrategy::Direct, 1);
        let task_b = make_task("b", ReasoningStrategy::Analogical, 2);
        flex.register_task(task_a);
        flex.register_task(task_b);
        flex.activate_task("a");

        let should_switch = flex.evaluate_switch(0.9, false);
        assert_eq!(should_switch, Some(SwitchReason::Curiosity));
    }

    #[test]
    fn test_no_switch_during_cooldown() {
        let mut flex = CognitiveFlexibility::new();
        let task = make_task("t1", ReasoningStrategy::Direct, 1);
        flex.register_task(task);
        flex.activate_task("t1");

        flex.switch_to("t1", SwitchReason::Perseveration);
        let no_switch = flex.evaluate_switch(0.0, false);
        assert!(no_switch.is_none());
    }

    #[test]
    fn test_diminishing_returns_trigger() {
        let mut flex = CognitiveFlexibility::new();
        let mut task = make_task("t1", ReasoningStrategy::Direct, 1);
        for _ in 0..4 {
            task.record_step("x".to_string(), 0.01);
        }
        flex.register_task(task);
        flex.activate_task("t1");

        let should_switch = flex.evaluate_switch(0.0, false);
        assert_eq!(should_switch, Some(SwitchReason::DiminishingReturns));
    }

    #[test]
    fn test_context_memory_save_restore() {
        let mut mem = VsaContextMemory::new();
        let task = make_task("t1", ReasoningStrategy::Decomposition, 42);
        let ctx = mem.save_context(&task);
        assert!(mem.contains("t1"));

        let restored = mem.restore_context(&ctx);
        assert!(restored.is_some());
        assert_eq!(restored.unwrap().id, "t1");

        mem.forget_context("t1");
        assert!(!mem.contains("t1"));
    }

    #[test]
    fn test_strategy_shifter_cycles() {
        let mut shifter = StrategyShifter::new();
        let first = shifter.propose_alternative(&ReasoningStrategy::Direct);
        assert!(first.is_some());
        let second = shifter.propose_alternative(&ReasoningStrategy::Direct);
        assert!(second.is_some());
        assert_ne!(first, second);
    }

    #[test]
    fn test_register_and_activate_new() {
        let mut flex = CognitiveFlexibility::new();
        let task = make_task("t1", ReasoningStrategy::FirstPrinciples, 1);
        flex.register_task(task);
        assert!(flex.current_task().is_none());

        let reason = flex.activate_task("t1");
        assert!(reason.is_some());
        assert!(flex.current_task().is_some());
        assert_eq!(
            flex.current_task().unwrap().reasoning_strategy,
            ReasoningStrategy::FirstPrinciples
        );
    }

    #[test]
    fn test_switch_to_nonexistent_task() {
        let mut flex = CognitiveFlexibility::new();
        let result = flex.switch_to("ghost", SwitchReason::Curiosity);
        assert!(!result);
    }

    #[test]
    fn test_switch_history_recorded() {
        let mut flex = CognitiveFlexibility::new();
        flex.register_task(make_task("a", ReasoningStrategy::Direct, 1));
        flex.register_task(make_task("b", ReasoningStrategy::Analogical, 2));
        flex.register_task(make_task("c", ReasoningStrategy::Counterfactual, 3));

        flex.activate_task("a");
        flex.switch_to("b", SwitchReason::Curiosity);
        flex.switch_to("c", SwitchReason::Perseveration);

        let recent = flex.recent_switches(10);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].to_task_id, "c");
        assert_eq!(recent[0].reason, SwitchReason::Perseveration);
        assert_eq!(recent[1].to_task_id, "b");
    }

    #[test]
    fn test_update_load_clamping() {
        let mut flex = CognitiveFlexibility::new();
        flex.update_load(0.3);
        assert!((flex.cognitive_load - 0.3).abs() < 1e-6);
        flex.reduce_load(0.1);
        assert!((flex.cognitive_load - 0.2).abs() < 1e-6);
        flex.update_load(2.0);
        assert!((flex.cognitive_load - 1.0).abs() < 1e-6);
        flex.reduce_load(5.0);
        assert!((flex.cognitive_load - 0.0).abs() < 1e-6);
    }
}
