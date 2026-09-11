use std::collections::HashMap;
use crate::agents::sim_agent::AgentAction;

/// Status of a behavior tree node after ticking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtStatus {
    Success,
    Failure,
    Running,
}

/// Shared state between BT nodes (Blackboard pattern)
#[derive(Debug, Clone, Default)]
pub struct Blackboard {
    pub floats: HashMap<String, f32>,
    pub ints: HashMap<String, i64>,
    pub bools: HashMap<String, bool>,
    pub strings: HashMap<String, String>,
    pub vectors: HashMap<String, [f32; 2]>,
}

impl Blackboard {
    pub fn new() -> Self { Self::default() }

    pub fn get_float(&self, key: &str) -> f32 {
        self.floats.get(key).copied().unwrap_or(0.0)
    }
    pub fn set_float(&mut self, key: &str, val: f32) {
        self.floats.insert(key.to_string(), val);
    }
    pub fn get_bool(&self, key: &str) -> bool {
        self.bools.get(key).copied().unwrap_or(false)
    }
    pub fn set_bool(&mut self, key: &str, val: bool) {
        self.bools.insert(key.to_string(), val);
    }
    pub fn get_vec2(&self, key: &str) -> Option<[f32; 2]> {
        self.vectors.get(key).copied()
    }
    pub fn set_vec2(&mut self, key: &str, val: [f32; 2]) {
        self.vectors.insert(key.to_string(), val);
    }
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.strings.get(key).map(|s| s.as_str())
    }
    pub fn set_string(&mut self, key: &str, val: &str) {
        self.strings.insert(key.to_string(), val.to_string());
    }
}

/// Core behavior tree node trait
pub trait BehaviorNode: Send + Sync {
    fn tick(&mut self, bb: &mut Blackboard) -> BtStatus;
    fn name(&self) -> &str;
}

/// Selector: tries children in order, returns Success on first Success.
/// Returns Running if a child is Running. Returns Failure if all children fail.
pub struct Selector {
    name: String,
    children: Vec<Box<dyn BehaviorNode>>,
    running_idx: usize,
}

impl Selector {
    pub fn new(name: &str, children: Vec<Box<dyn BehaviorNode>>) -> Self {
        Self { name: name.to_string(), children, running_idx: 0 }
    }
}

impl BehaviorNode for Selector {
    fn tick(&mut self, bb: &mut Blackboard) -> BtStatus {
        for i in self.running_idx..self.children.len() {
            match self.children[i].tick(bb) {
                BtStatus::Success => {
                    self.running_idx = 0;
                    return BtStatus::Success;
                }
                BtStatus::Running => {
                    self.running_idx = i;
                    return BtStatus::Running;
                }
                BtStatus::Failure => continue,
            }
        }
        self.running_idx = 0;
        BtStatus::Failure
    }

    fn name(&self) -> &str { &self.name }
}

/// Sequence: executes children in order, returns Failure on first Failure.
/// Returns Running if a child is Running. Returns Success if all children succeed.
pub struct Sequence {
    name: String,
    children: Vec<Box<dyn BehaviorNode>>,
    running_idx: usize,
}

impl Sequence {
    pub fn new(name: &str, children: Vec<Box<dyn BehaviorNode>>) -> Self {
        Self { name: name.to_string(), children, running_idx: 0 }
    }
}

impl BehaviorNode for Sequence {
    fn tick(&mut self, bb: &mut Blackboard) -> BtStatus {
        for i in self.running_idx..self.children.len() {
            match self.children[i].tick(bb) {
                BtStatus::Success => continue,
                BtStatus::Running => {
                    self.running_idx = i;
                    return BtStatus::Running;
                }
                BtStatus::Failure => {
                    self.running_idx = 0;
                    return BtStatus::Failure;
                }
            }
        }
        self.running_idx = 0;
        BtStatus::Success
    }

    fn name(&self) -> &str { &self.name }
}

/// Inverter: inverts Success/Failure, passes Running through.
pub struct Inverter {
    name: String,
    child: Box<dyn BehaviorNode>,
}

impl Inverter {
    pub fn new(name: &str, child: Box<dyn BehaviorNode>) -> Self {
        Self { name: name.to_string(), child }
    }
}

impl BehaviorNode for Inverter {
    fn tick(&mut self, bb: &mut Blackboard) -> BtStatus {
        match self.child.tick(bb) {
            BtStatus::Success => BtStatus::Failure,
            BtStatus::Failure => BtStatus::Success,
            BtStatus::Running => BtStatus::Running,
        }
    }

    fn name(&self) -> &str { &self.name }
}

/// Repeater: repeats child N times or until failure.
pub struct Repeater {
    name: String,
    child: Box<dyn BehaviorNode>,
    max_repeats: usize,
    current: usize,
}

impl Repeater {
    pub fn new(name: &str, child: Box<dyn BehaviorNode>, max_repeats: usize) -> Self {
        Self { name: name.to_string(), child, max_repeats, current: 0 }
    }
}

impl BehaviorNode for Repeater {
    fn tick(&mut self, bb: &mut Blackboard) -> BtStatus {
        if self.current >= self.max_repeats {
            self.current = 0;
            return BtStatus::Success;
        }
        match self.child.tick(bb) {
            BtStatus::Running => BtStatus::Running,
            BtStatus::Success => {
                self.current += 1;
                if self.current >= self.max_repeats {
                    self.current = 0;
                    BtStatus::Success
                } else {
                    BtStatus::Running
                }
            }
            BtStatus::Failure => {
                self.current = 0;
                BtStatus::Failure
            }
        }
    }

    fn name(&self) -> &str { &self.name }
}

/// Condition: checks a predicate on the Blackboard, returns Success or Failure.
pub struct Condition {
    name: String,
    check: Box<dyn Fn(&Blackboard) -> bool + Send + Sync>,
}

impl Condition {
    pub fn new(name: &str, check: Box<dyn Fn(&Blackboard) -> bool + Send + Sync>) -> Self {
        Self { name: name.to_string(), check }
    }
}

impl BehaviorNode for Condition {
    fn tick(&mut self, bb: &mut Blackboard) -> BtStatus {
        if (self.check)(bb) { BtStatus::Success } else { BtStatus::Failure }
    }
    fn name(&self) -> &str { &self.name }
}

/// Action: executes a side-effect on the Blackboard, returns Success/Failure/Running.
pub struct Action {
    name: String,
    execute: Box<dyn FnMut(&mut Blackboard) -> BtStatus + Send + Sync>,
}

impl Action {
    pub fn new(name: &str, execute: Box<dyn FnMut(&mut Blackboard) -> BtStatus + Send + Sync>) -> Self {
        Self { name: name.to_string(), execute }
    }
}

impl BehaviorNode for Action {
    fn tick(&mut self, bb: &mut Blackboard) -> BtStatus {
        (self.execute)(bb)
    }
    fn name(&self) -> &str { &self.name }
}

/// A complete behavior tree with root node and blackboard
pub struct BehaviorTree {
    pub root: Box<dyn BehaviorNode>,
    pub blackboard: Blackboard,
}

impl BehaviorTree {
    pub fn new(root: Box<dyn BehaviorNode>) -> Self {
        Self { root, blackboard: Blackboard::new() }
    }

    pub fn tick(&mut self) -> BtStatus {
        self.root.tick(&mut self.blackboard)
    }
}

/// Convert a BT Action result to an AgentAction for the existing decision pipeline.
/// This bridges BT output to the existing action execution system.
pub fn bt_action_to_agent_action(bt_action: &str, bb: &Blackboard) -> AgentAction {
    match bt_action {
        "explore" => {
            let dir = bb.get_vec2("explore_direction").unwrap_or([1.0, 0.0]);
            AgentAction::Explore { direction: crate::foundation::math_bridge::Vec2::new(dir[0], dir[1]) }
        }
        "rest" => AgentAction::Rest,
        "eat" => {
            let rid = bb.get_string("target_resource").unwrap_or("unknown").to_string();
            AgentAction::Eat { resource_id: rid }
        }
        "talk" => {
            let target = bb.get_string("talk_target").unwrap_or("").to_string();
            let msg = bb.get_string("talk_message").unwrap_or("hello").to_string();
            AgentAction::Talk { target_id: target, message: msg }
        }
        "attack" => {
            let target = bb.get_string("attack_target").unwrap_or("").to_string();
            AgentAction::Attack { target_id: target }
        }
        "trade" => {
            let target = bb.get_string("trade_target").unwrap_or("").to_string();
            AgentAction::Trade {
                target_id: target,
                item: bb.get_string("trade_item").unwrap_or("berries").to_string(),
                amount: bb.get_float("trade_amount") as u32,
            }
        }
        "build" => {
            let pos = bb.get_vec2("build_position").unwrap_or([0.0, 0.0]);
            let stype = bb.get_string("build_type").unwrap_or("shelter").to_string();
            AgentAction::Build {
                position: crate::foundation::math_bridge::Vec2::new(pos[0], pos[1]),
                structure_type: stype,
            }
        }
        "harvest" => {
            let rid = bb.get_string("target_resource").unwrap_or("unknown").to_string();
            AgentAction::Harvest { resource_id: rid }
        }
        "think" => AgentAction::Think,
        _ => AgentAction::Rest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blackboard_float_roundtrip() {
        let mut bb = Blackboard::new();
        bb.set_float("health", 0.8);
        assert_eq!(bb.get_float("health"), 0.8);
    }

    #[test]
    fn selector_picks_first_success() {
        let s = Selector::new("test", vec![
            Box::new(Condition::new("fail", Box::new(|_| false))),
            Box::new(Condition::new("pass", Box::new(|_| true))),
        ]);
        let mut bb = Blackboard::new();
        assert_eq!(s.tick(&mut bb), BtStatus::Success);
    }

    #[test]
    fn selector_fails_when_all_fail() {
        let s = Selector::new("test", vec![
            Box::new(Condition::new("fail1", Box::new(|_| false))),
            Box::new(Condition::new("fail2", Box::new(|_| false))),
        ]);
        let mut bb = Blackboard::new();
        assert_eq!(s.tick(&mut bb), BtStatus::Failure);
    }

    #[test]
    fn sequence_succeeds_when_all_pass() {
        let s = Sequence::new("test", vec![
            Box::new(Condition::new("pass1", Box::new(|_| true))),
            Box::new(Condition::new("pass2", Box::new(|_| true))),
        ]);
        let mut bb = Blackboard::new();
        assert_eq!(s.tick(&mut bb), BtStatus::Success);
    }

    #[test]
    fn sequence_fails_on_first_failure() {
        let s = Sequence::new("test", vec![
            Box::new(Condition::new("pass", Box::new(|_| true))),
            Box::new(Condition::new("fail", Box::new(|_| false))),
        ]);
        let mut bb = Blackboard::new();
        assert_eq!(s.tick(&mut bb), BtStatus::Failure);
    }

    #[test]
    fn inverter_flips_result() {
        let inv = Inverter::new("inv", Box::new(Condition::new("fail", Box::new(|_| false))));
        let mut bb = Blackboard::new();
        assert_eq!(inv.tick(&mut bb), BtStatus::Success);
    }

    #[test]
    fn condition_reads_blackboard() {
        let cond = Condition::new("check", Box::new(|bb| bb.get_float("hp") > 0.5));
        let mut bb = Blackboard::new();
        assert_eq!(cond.tick(&mut bb), BtStatus::Failure);
        bb.set_float("hp", 0.8);
        assert_eq!(cond.tick(&mut bb), BtStatus::Success);
    }

    #[test]
    fn bt_action_to_agent_action_explore() {
        let bb = Blackboard::new();
        let action = bt_action_to_agent_action("explore", &bb);
        assert!(matches!(action, AgentAction::Explore { .. }));
    }

    #[test]
    fn bt_action_to_agent_action_rest() {
        let bb = Blackboard::new();
        let action = bt_action_to_agent_action("rest", &bb);
        assert!(matches!(action, AgentAction::Rest));
    }

    #[test]
    fn bt_action_to_agent_action_eat() {
        let mut bb = Blackboard::new();
        bb.set_string("target_resource", "berry_bush_42");
        let action = bt_action_to_agent_action("eat", &bb);
        match action {
            AgentAction::Eat { resource_id } => assert_eq!(resource_id, "berry_bush_42"),
            _ => panic!("expected Eat"),
        }
    }

    #[test]
    fn bt_action_to_agent_action_talk() {
        let mut bb = Blackboard::new();
        bb.set_string("talk_target", "agent_7");
        bb.set_string("talk_message", "trade?");
        let action = bt_action_to_agent_action("talk", &bb);
        match action {
            AgentAction::Talk { target_id, message } => {
                assert_eq!(target_id, "agent_7");
                assert_eq!(message, "trade?");
            }
            _ => panic!("expected Talk"),
        }
    }

    #[test]
    fn full_tree_tick() {
        let tree = BehaviorTree::new(
            Box::new(Selector::new("root", vec![
                Box::new(Sequence::new("survival", vec![
                    Box::new(Condition::new("hungry", Box::new(|bb| bb.get_float("hunger") > 0.6))),
                    Box::new(Action::new("find_food", Box::new(|bb| {
                        bb.set_float("hunger", 0.2);
                        BtStatus::Success
                    }))),
                ])),
                Box::new(Action::new("wander", Box::new(|bb| {
                    bb.set_vec2("explore_direction", [1.0, 0.0]);
                    BtStatus::Success
                }))),
            ]))
        );
        let mut tree = tree;
        assert_eq!(tree.tick(), BtStatus::Success);
        assert_eq!(tree.blackboard.get_float("hunger"), 0.2);
    }
}
