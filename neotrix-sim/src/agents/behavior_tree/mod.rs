use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum BTStatus {
    Success,
    Failure,
    Running,
}

pub trait BehaviorNode: Send + Sync {
    fn tick(&mut self, blackboard: &mut Blackboard) -> BTStatus;
    fn name(&self) -> &str;
}

pub struct Blackboard {
    pub data: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl Blackboard {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn set<T: Send + Sync + 'static>(&mut self, key: &str, value: T) {
        self.data.insert(key.to_string(), Box::new(value));
    }

    pub fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.data.get(key)?.downcast_ref::<T>()
    }

    pub fn get_mut<T: 'static>(&mut self, key: &str) -> Option<&mut T> {
        self.data.get_mut(key)?.downcast_mut::<T>()
    }
}

pub struct Selector {
    pub children: Vec<Box<dyn BehaviorNode>>,
    pub current: usize,
}

impl Selector {
    pub fn new(children: Vec<Box<dyn BehaviorNode>>) -> Self {
        Self { children, current: 0 }
    }
}

impl BehaviorNode for Selector {
    fn tick(&mut self, blackboard: &mut Blackboard) -> BTStatus {
        while self.current < self.children.len() {
            match self.children[self.current].tick(blackboard) {
                BTStatus::Success => {
                    self.current = 0;
                    return BTStatus::Success;
                }
                BTStatus::Running => return BTStatus::Running,
                BTStatus::Failure => {
                    self.current += 1;
                }
            }
        }
        self.current = 0;
        BTStatus::Failure
    }

    fn name(&self) -> &str {
        "Selector"
    }
}

pub struct Sequence {
    pub children: Vec<Box<dyn BehaviorNode>>,
    pub current: usize,
}

impl Sequence {
    pub fn new(children: Vec<Box<dyn BehaviorNode>>) -> Self {
        Self { children, current: 0 }
    }
}

impl BehaviorNode for Sequence {
    fn tick(&mut self, blackboard: &mut Blackboard) -> BTStatus {
        while self.current < self.children.len() {
            match self.children[self.current].tick(blackboard) {
                BTStatus::Failure => {
                    self.current = 0;
                    return BTStatus::Failure;
                }
                BTStatus::Running => return BTStatus::Running,
                BTStatus::Success => {
                    self.current += 1;
                }
            }
        }
        self.current = 0;
        BTStatus::Success
    }

    fn name(&self) -> &str {
        "Sequence"
    }
}

pub struct Condition {
    pub name: String,
    pub check: Box<dyn Fn(&Blackboard) -> bool + Send + Sync>,
}

impl Condition {
    pub fn new(name: &str, check: Box<dyn Fn(&Blackboard) -> bool + Send + Sync>) -> Self {
        Self {
            name: name.to_string(),
            check,
        }
    }
}

impl BehaviorNode for Condition {
    fn tick(&mut self, blackboard: &mut Blackboard) -> BTStatus {
        if (self.check)(blackboard) {
            BTStatus::Success
        } else {
            BTStatus::Failure
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct Action {
    pub name: String,
    pub execute: Box<dyn Fn(&mut Blackboard) -> BTStatus + Send + Sync>,
}

impl Action {
    pub fn new(name: &str, execute: Box<dyn Fn(&mut Blackboard) -> BTStatus + Send + Sync>) -> Self {
        Self {
            name: name.to_string(),
            execute,
        }
    }
}

impl BehaviorNode for Action {
    fn tick(&mut self, blackboard: &mut Blackboard) -> BTStatus {
        (self.execute)(blackboard)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct Decorator {
    pub name: String,
    pub child: Box<dyn BehaviorNode>,
    pub modify: Box<dyn Fn(BTStatus) -> BTStatus + Send + Sync>,
}

impl Decorator {
    pub fn new(
        name: &str,
        child: Box<dyn BehaviorNode>,
        modify: Box<dyn Fn(BTStatus) -> BTStatus + Send + Sync>,
    ) -> Self {
        Self {
            name: name.to_string(),
            child,
            modify,
        }
    }
}

impl BehaviorNode for Decorator {
    fn tick(&mut self, blackboard: &mut Blackboard) -> BTStatus {
        let status = self.child.tick(blackboard);
        (self.modify)(status)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct BehaviorTree {
    pub root: Box<dyn BehaviorNode>,
}

impl BehaviorTree {
    pub fn new(root: Box<dyn BehaviorNode>) -> Self {
        Self { root }
    }

    pub fn tick(&mut self, blackboard: &mut Blackboard) -> BTStatus {
        self.root.tick(blackboard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blackboard() {
        let mut bb = Blackboard::new();
        bb.set("health", 100i32);
        assert_eq!(bb.get::<i32>("health"), Some(&100));
    }

    #[test]
    fn test_condition() {
        let mut bb = Blackboard::new();
        bb.set("has_food", true);
        let mut cond = Condition::new("check_food", Box::new(|bb| {
            *bb.get::<bool>("has_food").unwrap_or(&false)
        }));
        assert_eq!(cond.tick(&mut bb), BTStatus::Success);
    }

    #[test]
    fn test_selector() {
        let mut bb = Blackboard::new();
        let mut seq = Selector::new(vec![
            Box::new(Condition::new("fail", Box::new(|_| false))),
            Box::new(Condition::new("success", Box::new(|_| true))),
        ]);
        assert_eq!(seq.tick(&mut bb), BTStatus::Success);
    }

    #[test]
    fn test_sequence() {
        let mut bb = Blackboard::new();
        let mut seq = Sequence::new(vec![
            Box::new(Condition::new("ok", Box::new(|_| true))),
            Box::new(Action::new("act", Box::new(|_| BTStatus::Success))),
        ]);
        assert_eq!(seq.tick(&mut bb), BTStatus::Success);
    }
}
