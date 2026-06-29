use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CycleStep {
    Gather,
    Reason,
    Evolve,
    Reflect,
    Sleep,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct CycleNode {
    pub name: &'static str,
    pub step: CycleStep,
    pub priority: usize,
    pub enabled: bool,
    pub call_count: u64,
}

pub struct CycleRegistry {
    nodes: HashMap<CycleStep, Vec<CycleNode>>,
    step_order: Vec<CycleStep>,
}

impl CycleRegistry {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            step_order: vec![
                CycleStep::Gather,
                CycleStep::Reason,
                CycleStep::Evolve,
                CycleStep::Reflect,
                CycleStep::Sleep,
            ],
        }
    }

    pub fn register(&mut self, node: CycleNode) {
        let step = node.step.clone();
        self.nodes.entry(step).or_default().push(node);
    }

    pub fn nodes_at(&self, step: &CycleStep) -> Vec<&CycleNode> {
        self.nodes.get(step)
            .map(|nodes| {
                let mut sorted: Vec<_> = nodes.iter().collect();
                sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
                sorted
            })
            .unwrap_or_default()
    }

    pub fn all_enabled(&self) -> Vec<&CycleNode> {
        let mut all = Vec::new();
        for step in &self.step_order {
            if let Some(nodes) = self.nodes.get(step) {
                for node in nodes {
                    if node.enabled {
                        all.push(node);
                    }
                }
            }
        }
        all
    }

    pub fn set_step_order(&mut self, order: Vec<CycleStep>) {
        self.step_order = order;
    }

    pub fn enable(&mut self, name: &str) -> bool {
        for nodes in self.nodes.values_mut() {
            for node in nodes.iter_mut() {
                if node.name == name {
                    node.enabled = true;
                    return true;
                }
            }
        }
        false
    }

    pub fn disable(&mut self, name: &str) -> bool {
        for nodes in self.nodes.values_mut() {
            for node in nodes.iter_mut() {
                if node.name == name {
                    node.enabled = false;
                    return true;
                }
            }
        }
        false
    }

    pub fn increment_call(&mut self, name: &str) {
        for nodes in self.nodes.values_mut() {
            for node in nodes.iter_mut() {
                if node.name == name {
                    node.call_count += 1;
                    return;
                }
            }
        }
    }

    pub fn step_count(&self) -> usize {
        self.step_order.len()
    }

    pub fn registered_count(&self) -> usize {
        self.nodes.values().map(|v| v.len()).sum()
    }

    pub fn add_custom_step(&mut self, name: &str) {
        let step = CycleStep::Custom(name.to_string());
        if !self.step_order.contains(&step) {
            self.step_order.push(step);
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.step_order = vec![
            CycleStep::Gather,
            CycleStep::Reason,
            CycleStep::Evolve,
            CycleStep::Reflect,
            CycleStep::Sleep,
        ];
    }
}

impl Default for CycleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_registry_initial() {
        let r = CycleRegistry::new();
        assert_eq!(r.step_count(), 5);
        assert_eq!(r.registered_count(), 0);
    }

    #[test]
    fn test_cycle_registry_register() {
        let mut r = CycleRegistry::new();
        r.register(CycleNode { name: "memory_load", step: CycleStep::Gather, priority: 10, enabled: true, call_count: 0 });
        r.register(CycleNode { name: "mcts_search", step: CycleStep::Reason, priority: 5, enabled: true, call_count: 0 });
        assert_eq!(r.registered_count(), 2);
        assert_eq!(r.nodes_at(&CycleStep::Gather).len(), 1);
    }

    #[test]
    fn test_cycle_registry_enable_disable() {
        let mut r = CycleRegistry::new();
        r.register(CycleNode { name: "test_node", step: CycleStep::Reason, priority: 1, enabled: true, call_count: 0 });
        assert!(r.disable("test_node"));
        assert!(!r.nodes_at(&CycleStep::Reason)[0].enabled);
        assert!(r.enable("test_node"));
        assert!(r.nodes_at(&CycleStep::Reason)[0].enabled);
    }

    #[test]
    fn test_custom_step() {
        let mut r = CycleRegistry::new();
        r.add_custom_step("harvest");
        assert!(r.step_order.contains(&CycleStep::Custom("harvest".into())));
    }
}
