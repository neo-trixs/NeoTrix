use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalStatus {
    Active,
    Paused,
    Completed,
    Abandoned,
}

#[derive(Debug, Clone)]
pub struct Goal {
    pub id: String,
    pub description: Vec<u8>,
    pub priority: f64,
    pub deadline: Option<u64>,
    pub status: GoalStatus,
    pub parent_id: Option<String>,
    pub subgoal_ids: Vec<String>,
    pub created_at: u64,
    pub expected_value: f64,
}

impl Goal {
    pub fn new(id: impl Into<String>, description: Vec<u8>, priority: f64) -> Self {
        Self {
            id: id.into(),
            description,
            priority,
            deadline: None,
            status: GoalStatus::Active,
            parent_id: None,
            subgoal_ids: Vec::new(),
            created_at: 0,
            expected_value: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkingMemoryGate {
    pub gate_signal: i8,
    pub last_signal_change: u64,
    pub total_updates: u64,
    pub total_clears: u64,
}

impl WorkingMemoryGate {
    pub fn new() -> Self {
        Self {
            gate_signal: 0,
            last_signal_change: 0,
            total_updates: 0,
            total_clears: 0,
        }
    }

    pub fn set_signal(&mut self, signal: i8) {
        let clamped = signal.clamp(-1, 1);
        if clamped != self.gate_signal {
            self.last_signal_change += 1;
        }
        self.gate_signal = clamped;
    }

    pub fn should_update(&self) -> bool {
        self.gate_signal == 1
    }

    pub fn should_clear(&self) -> bool {
        self.gate_signal == -1
    }
}

impl Default for WorkingMemoryGate {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct InterferenceController {
    pub conflict_threshold: f64,
    pub total_conflicts_detected: u64,
    pub total_resolved: u64,
}

impl InterferenceController {
    pub fn new(conflict_threshold: f64) -> Self {
        Self {
            conflict_threshold,
            total_conflicts_detected: 0,
            total_resolved: 0,
        }
    }

    pub fn detect_interference(&self, goal_a: &Goal, goal_b: &Goal) -> f64 {
        QuantizedVSA::similarity(&goal_a.description, &goal_b.description)
    }

    pub fn resolve_conflict(&self, goals: &[Goal]) -> Vec<Goal> {
        let mut sorted = goals.to_vec();
        sorted.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut selected: Vec<Goal> = Vec::new();
        for goal in sorted {
            let conflicts = selected.iter().any(|g| {
                let sim = self.detect_interference(&goal, g);
                sim > self.conflict_threshold
            });
            if !conflicts {
                selected.push(goal);
            }
        }
        selected
    }
}

impl Default for InterferenceController {
    fn default() -> Self {
        Self::new(0.7)
    }
}

#[derive(Debug, Clone)]
pub struct ImpulseGate {
    pub impulsivity: f64,
    pub gate_history: VecDeque<(u64, bool)>,
    max_history: usize,
}

impl ImpulseGate {
    pub fn new(impulsivity: f64) -> Self {
        Self {
            impulsivity,
            gate_history: VecDeque::with_capacity(100),
            max_history: 100,
        }
    }

    pub fn evaluate(&self, long_term_value: f64, immediate_reward: f64) -> bool {
        let gate = long_term_value - (self.impulsivity * immediate_reward);
        gate > 0.3
    }

    pub fn record_outcome(&mut self, allowed: bool) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.gate_history.push_back((timestamp, allowed));
        while self.gate_history.len() > self.max_history {
            self.gate_history.pop_front();
        }
    }

    pub fn gating_ratio(&self) -> f64 {
        if self.gate_history.is_empty() {
            return 0.0;
        }
        let allowed = self.gate_history.iter().filter(|(_, a)| *a).count() as f64;
        allowed / self.gate_history.len() as f64
    }
}

#[derive(Debug, Clone)]
pub struct ResourceBroker {
    pub total_budget: f64,
    pub allocated: f64,
    pub used: f64,
    pub overage_count: u64,
}

impl ResourceBroker {
    pub fn new(total_budget: f64) -> Self {
        Self {
            total_budget,
            allocated: 0.0,
            used: 0.0,
            overage_count: 0,
        }
    }

    pub fn allocate(&mut self, request: f64) -> f64 {
        let actual = request.min(self.remaining());
        self.allocated += actual;
        actual
    }

    pub fn use_resources(&mut self, amount: f64) {
        self.used += amount;
        if self.used > self.allocated {
            self.overage_count += 1;
        }
    }

    pub fn remaining(&self) -> f64 {
        (self.total_budget - self.allocated).max(0.0)
    }

    pub fn utilization(&self) -> f64 {
        if self.total_budget == 0.0 {
            return 0.0;
        }
        (self.used / self.total_budget).clamp(0.0, 1.0)
    }

    pub fn reset(&mut self) {
        self.allocated = 0.0;
        self.used = 0.0;
        self.overage_count = 0;
    }
}

#[derive(Debug, Clone)]
pub struct ExecutiveController {
    pub goal_stack: VecDeque<Goal>,
    pub wm_gate: WorkingMemoryGate,
    pub interference: InterferenceController,
    pub impulse: ImpulseGate,
    pub broker: ResourceBroker,
    max_goals: usize,
}

impl ExecutiveController {
    pub fn new() -> Self {
        Self {
            goal_stack: VecDeque::new(),
            wm_gate: WorkingMemoryGate::new(),
            interference: InterferenceController::new(0.7),
            impulse: ImpulseGate::new(0.5),
            broker: ResourceBroker::new(100.0),
            max_goals: 20,
        }
    }

    pub fn push_goal(&mut self, goal: Goal) {
        for existing in &self.goal_stack {
            if existing.status == GoalStatus::Active {
                let sim = self.interference.detect_interference(&goal, existing);
                if sim > self.interference.conflict_threshold {
                    self.interference.total_conflicts_detected += 1;
                }
            }
        }
        if self.goal_stack.len() >= self.max_goals {
            self.goal_stack.pop_back();
        }
        self.goal_stack.push_front(goal);
    }

    pub fn pop_goal(&mut self, id: &str) {
        if let Some(goal) = self.goal_stack.iter_mut().find(|g| g.id == id) {
            goal.status = GoalStatus::Completed;
        }
    }

    pub fn abandon_goal(&mut self, id: &str) {
        let ids_to_abandon = self.collect_descendants(id);
        for goal in self.goal_stack.iter_mut() {
            if ids_to_abandon.contains(&goal.id) {
                goal.status = GoalStatus::Abandoned;
            }
        }
    }

    fn collect_descendants(&self, parent_id: &str) -> Vec<String> {
        let mut result = vec![parent_id.to_string()];
        let mut stack: Vec<&str> = vec![parent_id];
        while let Some(current) = stack.pop() {
            for goal in &self.goal_stack {
                if goal.parent_id.as_deref() == Some(current) {
                    result.push(goal.id.clone());
                    stack.push(&goal.id);
                }
            }
        }
        result
    }

    pub fn current_goal(&self) -> Option<&Goal> {
        self.goal_stack.front()
    }

    pub fn active_goals(&self) -> Vec<&Goal> {
        self.goal_stack
            .iter()
            .filter(|g| g.status == GoalStatus::Active)
            .collect()
    }

    pub fn subgoal_count(&self, goal_id: &str) -> usize {
        self.goal_stack
            .iter()
            .filter(|g| g.parent_id.as_deref() == Some(goal_id))
            .count()
    }

    pub fn decompose_goal(&mut self, parent_id: &str, sub_descriptions: &[&str]) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let maybe_parent = self.goal_stack.iter().position(|g| g.id == parent_id);
        let parent_idx = match maybe_parent {
            Some(idx) => idx,
            None => return,
        };

        let sub_ids: Vec<String> = sub_descriptions
            .iter()
            .enumerate()
            .map(|(i, _desc)| format!("{}.sub.{}", parent_id, i))
            .collect();

        for (idx, desc) in sub_descriptions.iter().enumerate() {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            desc.hash(&mut hasher);
            let seed = hasher.finish();
            let vsa = QuantizedVSA::seeded_random(seed, 64);

            let subgoal = Goal {
                id: sub_ids[idx].clone(),
                description: vsa,
                priority: self.goal_stack[parent_idx].priority * 0.9,
                deadline: self.goal_stack[parent_idx].deadline,
                status: GoalStatus::Active,
                parent_id: Some(parent_id.to_string()),
                subgoal_ids: Vec::new(),
                created_at: now,
                expected_value: self.goal_stack[parent_idx].expected_value
                    / sub_descriptions.len() as f64,
            };
            self.goal_stack.push_front(subgoal);
        }

        if let Some(parent) = self.goal_stack.iter_mut().find(|g| g.id == parent_id) {
            parent.subgoal_ids = sub_ids;
        }
    }

    pub fn allocate_resources(&mut self, importance: f64) -> f64 {
        let request = importance.clamp(0.0, 1.0) * self.broker.total_budget;
        self.broker.allocate(request)
    }

    pub fn gate_wm_update(&self) -> bool {
        self.wm_gate.should_update()
    }

    pub fn suppress_action(&self, lt_value: f64, ir_reward: f64) -> bool {
        !self.impulse.evaluate(lt_value, ir_reward)
    }

    pub fn cognitive_load(&self) -> f64 {
        let active = self.active_goals().len() as f64 * 0.1;
        let broker = self.broker.utilization() * 0.5;
        (active + broker).clamp(0.0, 1.0)
    }
}

impl Default for ExecutiveController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vsa(seed: u64) -> Vec<u8> {
        QuantizedVSA::seeded_random(seed, 64)
    }

    fn make_goal(id: &str, seed: u64, priority: f64) -> Goal {
        Goal::new(id, test_vsa(seed), priority)
    }

    #[test]
    fn test_goal_creation() {
        let desc = test_vsa(42);
        let g = Goal::new("g1", desc.clone(), 0.8);
        assert_eq!(g.id, "g1");
        assert_eq!(g.description, desc);
        assert!((g.priority - 0.8).abs() < 1e-9);
        assert_eq!(g.status, GoalStatus::Active);
        assert!(g.deadline.is_none());
        assert!(g.parent_id.is_none());
        assert!(g.subgoal_ids.is_empty());
        assert!((g.expected_value - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_goal_status_transitions() {
        let desc = test_vsa(1);
        let mut g = Goal::new("g1", desc, 0.5);
        assert_eq!(g.status, GoalStatus::Active);
        g.status = GoalStatus::Paused;
        assert_eq!(g.status, GoalStatus::Paused);
        g.status = GoalStatus::Completed;
        assert_eq!(g.status, GoalStatus::Completed);
        g.status = GoalStatus::Abandoned;
        assert_eq!(g.status, GoalStatus::Abandoned);
    }

    #[test]
    fn test_wm_gate_signals() {
        let mut gate = WorkingMemoryGate::new();
        assert_eq!(gate.gate_signal, 0);

        gate.set_signal(5);
        assert_eq!(gate.gate_signal, 1);

        gate.set_signal(-10);
        assert_eq!(gate.gate_signal, -1);

        gate.set_signal(0);
        assert_eq!(gate.gate_signal, 0);

        gate.set_signal(0);
        assert_eq!(gate.last_signal_change, 2);
    }

    #[test]
    fn test_wm_gate_should_update() {
        let mut gate = WorkingMemoryGate::new();
        assert!(!gate.should_update());
        gate.set_signal(1);
        assert!(gate.should_update());
    }

    #[test]
    fn test_wm_gate_should_clear() {
        let mut gate = WorkingMemoryGate::new();
        assert!(!gate.should_clear());
        gate.set_signal(-1);
        assert!(gate.should_clear());
    }

    #[test]
    fn test_interference_detection() {
        let ic = InterferenceController::new(0.7);
        let same_desc = test_vsa(42);
        let goal_a = Goal::new("a", same_desc.clone(), 0.8);
        let goal_b = Goal::new("b", same_desc.clone(), 0.6);
        let goal_c = Goal::new("c", test_vsa(99), 0.4);

        let sim_same = ic.detect_interference(&goal_a, &goal_b);
        let sim_diff = ic.detect_interference(&goal_a, &goal_c);

        assert!(
            sim_same > sim_diff,
            "same-description goals should have higher similarity ({}) than different ones ({})",
            sim_same,
            sim_diff
        );
        assert!(
            (sim_same - 1.0).abs() < 1e-6,
            "identical descriptions should have similarity ~1.0, got {}",
            sim_same
        );
    }

    #[test]
    fn test_interference_resolution() {
        let ic = InterferenceController::new(0.5);
        let shared_desc = test_vsa(1);
        let goals = vec![
            Goal::new("high", shared_desc.clone(), 1.0),
            Goal::new("mid", shared_desc.clone(), 0.5),
            Goal::new("low", test_vsa(2), 0.1),
        ];

        let resolved = ic.resolve_conflict(&goals);
        assert_eq!(
            resolved.len(),
            2,
            "should keep high (priority 1.0) and low (no conflict)"
        );
        assert!(resolved.iter().any(|g| g.id == "high"));
        assert!(resolved.iter().any(|g| g.id == "low"));
        assert!(!resolved.iter().any(|g| g.id == "mid"));
    }

    #[test]
    fn test_impulse_evaluate() {
        let gate = ImpulseGate::new(0.5);
        assert!(gate.evaluate(1.0, 0.5));
        assert!(!gate.evaluate(0.1, 1.0));
    }

    #[test]
    fn test_impulse_evaluate_blocked() {
        let gate = ImpulseGate::new(0.9);
        assert!(!gate.evaluate(0.2, 0.8));
    }

    #[test]
    fn test_impulse_ratio() {
        let mut gate = ImpulseGate::new(0.5);
        assert!((gate.gating_ratio() - 0.0).abs() < 1e-9);
        gate.record_outcome(true);
        gate.record_outcome(true);
        gate.record_outcome(false);
        assert!((gate.gating_ratio() - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_resource_broker_allocate() {
        let mut broker = ResourceBroker::new(100.0);
        let got = broker.allocate(30.0);
        assert!((got - 30.0).abs() < 1e-9);
        assert!((broker.allocated - 30.0).abs() < 1e-9);
    }

    #[test]
    fn test_resource_broker_remaining() {
        let mut broker = ResourceBroker::new(100.0);
        assert!((broker.remaining() - 100.0).abs() < 1e-9);
        broker.allocate(40.0);
        assert!((broker.remaining() - 60.0).abs() < 1e-9);
        broker.allocate(70.0);
        assert!((broker.remaining() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_resource_broker_utilization() {
        let mut broker = ResourceBroker::new(100.0);
        assert!((broker.utilization() - 0.0).abs() < 1e-9);
        broker.allocate(50.0);
        broker.use_resources(25.0);
        assert!((broker.utilization() - 0.25).abs() < 1e-9);
        broker.use_resources(25.0);
        assert!((broker.utilization() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_resource_broker_overage() {
        let mut broker = ResourceBroker::new(100.0);
        broker.allocate(20.0);
        broker.use_resources(30.0);
        assert_eq!(broker.overage_count, 1);
    }

    #[test]
    fn test_executive_controller_new() {
        let ec = ExecutiveController::new();
        assert!(ec.goal_stack.is_empty());
        assert!((ec.broker.total_budget - 100.0).abs() < 1e-9);
        assert!((ec.interference.conflict_threshold - 0.7).abs() < 1e-9);
        assert_eq!(ec.max_goals, 20);
    }

    #[test]
    fn test_push_goal() {
        let mut ec = ExecutiveController::new();
        ec.push_goal(make_goal("g1", 1, 0.8));
        assert_eq!(ec.goal_stack.len(), 1);
        assert_eq!(ec.current_goal().unwrap().id, "g1");
    }

    #[test]
    fn test_push_goal_interference_detected() {
        let mut ec = ExecutiveController::new();
        let desc = test_vsa(10);
        ec.push_goal(Goal::new("g1", desc.clone(), 0.8));
        ec.push_goal(Goal::new("g2", desc.clone(), 0.6));
        assert!(ec.interference.total_conflicts_detected > 0);
    }

    #[test]
    fn test_pop_goal() {
        let mut ec = ExecutiveController::new();
        ec.push_goal(make_goal("g1", 1, 0.8));
        ec.pop_goal("g1");
        assert_eq!(ec.goal_stack[0].status, GoalStatus::Completed);
    }

    #[test]
    fn test_current_goal() {
        let mut ec = ExecutiveController::new();
        assert!(ec.current_goal().is_none());
        ec.push_goal(make_goal("g1", 1, 0.8));
        assert_eq!(ec.current_goal().unwrap().id, "g1");
        ec.push_goal(make_goal("g2", 2, 0.9));
        assert_eq!(ec.current_goal().unwrap().id, "g2");
    }

    #[test]
    fn test_active_goals() {
        let mut ec = ExecutiveController::new();
        ec.push_goal(make_goal("g1", 1, 0.8));
        ec.push_goal(make_goal("g2", 2, 0.9));
        assert_eq!(ec.active_goals().len(), 2);
        ec.pop_goal("g1");
        assert_eq!(ec.active_goals().len(), 1);
        assert_eq!(ec.active_goals()[0].id, "g2");
    }

    #[test]
    fn test_abandon_goal() {
        let mut ec = ExecutiveController::new();
        ec.push_goal(make_goal("g1", 1, 0.8));
        ec.push_goal(make_goal("g2", 2, 0.6));
        ec.abandon_goal("g1");
        assert_eq!(ec.goal_stack[1].status, GoalStatus::Abandoned);
        assert_eq!(ec.goal_stack[0].status, GoalStatus::Active);
    }

    #[test]
    fn test_subgoal_count() {
        let mut ec = ExecutiveController::new();
        ec.push_goal(make_goal("parent", 1, 0.8));
        assert_eq!(ec.subgoal_count("parent"), 0);

        let sub = Goal {
            parent_id: Some("parent".into()),
            ..make_goal("child", 2, 0.5)
        };
        ec.push_goal(sub);
        assert_eq!(ec.subgoal_count("parent"), 1);
    }

    #[test]
    fn test_decompose_goal() {
        let mut ec = ExecutiveController::new();
        ec.push_goal(make_goal("root", 1, 0.9));
        ec.decompose_goal("root", &["research", "implement", "test"]);

        let sub_ids: Vec<String> = (0..3).map(|i| format!("root.sub.{}", i)).collect();

        for sid in &sub_ids {
            assert!(
                ec.goal_stack.iter().any(|g| g.id == *sid),
                "subgoal {} should exist",
                sid
            );
        }

        let parent = ec.goal_stack.iter().find(|g| g.id == "root").unwrap();
        assert_eq!(parent.subgoal_ids.len(), 3);
        assert_eq!(parent.subgoal_ids[0], "root.sub.0");
        assert_eq!(ec.subgoal_count("root"), 3);
    }

    #[test]
    fn test_decompose_goal_vsa_determinism() {
        let mut ec = ExecutiveController::new();
        ec.push_goal(make_goal("root", 1, 0.9));
        ec.decompose_goal("root", &["task"]);

        let sub = ec.goal_stack.iter().find(|g| g.id == "root.sub.0").unwrap();
        let desc_1 = sub.description.clone();

        let mut ec2 = ExecutiveController::new();
        ec2.push_goal(make_goal("root", 1, 0.9));
        ec2.decompose_goal("root", &["task"]);
        let sub2 = ec2
            .goal_stack
            .iter()
            .find(|g| g.id == "root.sub.0")
            .unwrap();

        assert_eq!(
            desc_1, sub2.description,
            "same description should produce same VSA"
        );
    }

    #[test]
    fn test_allocate_resources() {
        let mut ec = ExecutiveController::new();
        let got = ec.allocate_resources(0.5);
        assert!(
            (got - 50.0).abs() < 1e-9,
            "0.5 importance * 100 budget = 50, got {}",
            got
        );
    }

    #[test]
    fn test_gate_wm_update() {
        let mut ec = ExecutiveController::new();
        assert!(!ec.gate_wm_update());
        ec.wm_gate.set_signal(1);
        assert!(ec.gate_wm_update());
    }

    #[test]
    fn test_suppress_action() {
        let ec = ExecutiveController::new();
        assert!(
            !ec.suppress_action(1.0, 0.5),
            "high LT value should not suppress"
        );
        assert!(
            ec.suppress_action(0.1, 1.0),
            "low LT + high reward should suppress"
        );
    }

    #[test]
    fn test_cognitive_load() {
        let mut ec = ExecutiveController::new();
        let load_empty = ec.cognitive_load();
        assert!(
            (load_empty - 0.0).abs() < 1e-9,
            "empty load should be 0, got {}",
            load_empty
        );

        ec.push_goal(make_goal("g1", 1, 0.8));
        ec.push_goal(make_goal("g2", 2, 0.6));
        let load = ec.cognitive_load();
        assert!(
            load > 0.0,
            "load should be >0 with active goals, got {}",
            load
        );
        assert!(load <= 1.0, "load should be <= 1.0, got {}", load);
    }

    #[test]
    fn test_max_goals_enforced() {
        let mut ec = ExecutiveController::new();
        ec.max_goals = 3;
        for i in 0..5 {
            ec.push_goal(make_goal(&format!("g{}", i), i as u64, 1.0));
        }
        assert_eq!(ec.goal_stack.len(), 3);
    }

    #[test]
    fn test_abandon_cascade() {
        let mut ec = ExecutiveController::new();
        ec.push_goal(make_goal("root", 1, 0.9));
        ec.decompose_goal("root", &["a", "b"]);
        ec.abandon_goal("root");
        assert_eq!(
            ec.goal_stack
                .iter()
                .find(|g| g.id == "root")
                .unwrap()
                .status,
            GoalStatus::Abandoned
        );
        assert_eq!(
            ec.goal_stack
                .iter()
                .find(|g| g.id == "root.sub.0")
                .unwrap()
                .status,
            GoalStatus::Abandoned
        );
        assert_eq!(
            ec.goal_stack
                .iter()
                .find(|g| g.id == "root.sub.1")
                .unwrap()
                .status,
            GoalStatus::Abandoned
        );
    }

    #[test]
    fn test_resource_broker_reset() {
        let mut broker = ResourceBroker::new(100.0);
        broker.allocate(70.0);
        broker.use_resources(50.0);
        assert!(broker.overage_count == 1 || broker.overage_count == 0);
        broker.reset();
        assert!((broker.allocated - 0.0).abs() < 1e-9);
        assert!((broker.used - 0.0).abs() < 1e-9);
        assert_eq!(broker.overage_count, 0);
    }
}
