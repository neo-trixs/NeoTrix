#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    Success,
    Failure,
    Running,
}

pub trait BehaviorNode: Send + Sync {
    fn name(&self) -> &str;
    fn tick(&self, context: &mut bt::BehaviorContext) -> NodeStatus;
}

pub mod bt {
    use std::collections::HashMap;

    pub struct BehaviorContext {
        pub blackboard: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
        pub dt: f64,
    }

    impl BehaviorContext {
        pub fn new() -> Self {
            Self {
                blackboard: HashMap::new(),
                dt: 0.0,
            }
        }
        pub fn set<T: std::any::Any + Send + Sync + 'static>(&mut self, key: &str, value: T) {
            self.blackboard.insert(key.to_string(), Box::new(value));
        }
        pub fn get<T: std::any::Any + Send + Sync + 'static>(&self, key: &str) -> Option<&T> {
            self.blackboard.get(key).and_then(|v| v.downcast_ref::<T>())
        }
        pub fn get_mut<T: std::any::Any + Send + Sync + 'static>(
            &mut self,
            key: &str,
        ) -> Option<&mut T> {
            self.blackboard
                .get_mut(key)
                .and_then(|v| v.downcast_mut::<T>())
        }
    }

    impl Default for BehaviorContext {
        fn default() -> Self {
            Self::new()
        }
    }
}

pub struct BehaviorTree {
    root: Option<Box<dyn BehaviorNode>>,
    context: bt::BehaviorContext,
}

impl BehaviorTree {
    pub fn new() -> Self {
        Self {
            root: None,
            context: bt::BehaviorContext::new(),
        }
    }
    pub fn set_root(&mut self, node: Box<dyn BehaviorNode>) {
        self.root = Some(node);
    }
    pub fn tick(&mut self) -> NodeStatus {
        if let Some(ref root) = self.root {
            root.tick(&mut self.context)
        } else {
            NodeStatus::Failure
        }
    }
    pub fn context(&mut self) -> &mut bt::BehaviorContext {
        &mut self.context
    }
}

impl Default for BehaviorTree {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Sequence {
    children: Vec<Box<dyn BehaviorNode>>,
}

impl Sequence {
    pub fn new(children: Vec<Box<dyn BehaviorNode>>) -> Self {
        Self { children }
    }
}

impl BehaviorNode for Sequence {
    fn name(&self) -> &str {
        "Sequence"
    }
    fn tick(&self, ctx: &mut bt::BehaviorContext) -> NodeStatus {
        for child in &self.children {
            match child.tick(ctx) {
                NodeStatus::Failure => return NodeStatus::Failure,
                NodeStatus::Running => return NodeStatus::Running,
                _ => {}
            }
        }
        NodeStatus::Success
    }
}

pub struct Selector {
    children: Vec<Box<dyn BehaviorNode>>,
}

impl Selector {
    pub fn new(children: Vec<Box<dyn BehaviorNode>>) -> Self {
        Self { children }
    }
}

impl BehaviorNode for Selector {
    fn name(&self) -> &str {
        "Selector"
    }
    fn tick(&self, ctx: &mut bt::BehaviorContext) -> NodeStatus {
        for child in &self.children {
            match child.tick(ctx) {
                NodeStatus::Success => return NodeStatus::Success,
                NodeStatus::Running => return NodeStatus::Running,
                _ => {}
            }
        }
        NodeStatus::Failure
    }
}

pub struct Inverter {
    child: Box<dyn BehaviorNode>,
}

impl Inverter {
    pub fn new(child: Box<dyn BehaviorNode>) -> Self {
        Self { child }
    }
}

impl BehaviorNode for Inverter {
    fn name(&self) -> &str {
        "Inverter"
    }
    fn tick(&self, ctx: &mut bt::BehaviorContext) -> NodeStatus {
        match self.child.tick(ctx) {
            NodeStatus::Success => NodeStatus::Failure,
            NodeStatus::Failure => NodeStatus::Success,
            NodeStatus::Running => NodeStatus::Running,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysSuccess;

    impl BehaviorNode for AlwaysSuccess {
        fn name(&self) -> &str {
            "AlwaysSuccess"
        }
        fn tick(&self, _: &mut bt::BehaviorContext) -> NodeStatus {
            NodeStatus::Success
        }
    }

    struct AlwaysFailure;

    impl BehaviorNode for AlwaysFailure {
        fn name(&self) -> &str {
            "AlwaysFailure"
        }
        fn tick(&self, _: &mut bt::BehaviorContext) -> NodeStatus {
            NodeStatus::Failure
        }
    }

    #[test]
    fn test_sequence_success() {
        let mut bt = BehaviorTree::new();
        bt.set_root(Box::new(Sequence::new(vec![
            Box::new(AlwaysSuccess),
            Box::new(AlwaysSuccess),
        ])));
        assert_eq!(bt.tick(), NodeStatus::Success);
    }

    #[test]
    fn test_sequence_failure() {
        let mut bt = BehaviorTree::new();
        bt.set_root(Box::new(Sequence::new(vec![
            Box::new(AlwaysSuccess),
            Box::new(AlwaysFailure),
        ])));
        assert_eq!(bt.tick(), NodeStatus::Failure);
    }

    #[test]
    fn test_selector() {
        let mut bt = BehaviorTree::new();
        bt.set_root(Box::new(Selector::new(vec![
            Box::new(AlwaysFailure),
            Box::new(AlwaysSuccess),
        ])));
        assert_eq!(bt.tick(), NodeStatus::Success);
    }

    #[test]
    fn test_empty_tree() {
        let mut bt = BehaviorTree::new();
        assert_eq!(bt.tick(), NodeStatus::Failure);
    }
}
