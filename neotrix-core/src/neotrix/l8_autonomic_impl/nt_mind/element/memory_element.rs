use std::any::Any;
use std::fmt;
use crate::neotrix::nt_mind::memory::ReasoningBank;
use super::bus::ElementBus;
use super::{Element, ElementError, ElementType, CapabilityAccess, CapabilityOp};

pub struct MemoryElement {
    pub bank: ReasoningBank,
    init_called: bool,
    started: bool,
}

impl fmt::Debug for MemoryElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryElement")
            .field("bank_stats", &self.bank.stats())
            .field("init_called", &self.init_called)
            .field("started", &self.started)
            .finish()
    }
}

impl MemoryElement {
    pub fn new(capacity: usize) -> Self {
        Self {
            bank: ReasoningBank::new(capacity),
            init_called: false,
            started: false,
        }
    }

    pub fn store(&mut self, description: &str, reward: f64) {
        use crate::neotrix::nt_world_model::TaskType;
        let memory = crate::neotrix::nt_mind::memory::ReasoningMemory::new(
            description, TaskType::General, &[], reward,
        );
        self.bank.store(memory);
    }

    pub fn recall_count(&self) -> usize {
        self.bank.stats().total_memories
    }
}

impl Element for MemoryElement {
    fn id(&self) -> &str { "element.memory" }
    fn name(&self) -> &str { "Reasoning Bank Memory" }
    fn version(&self) -> &str { "0.2.0" }
    fn element_type(&self) -> ElementType { ElementType::Core }

    fn init(&mut self, _bus: &ElementBus) -> Result<(), ElementError> {
        self.init_called = true;
        Ok(())
    }

    fn start(&mut self) -> Result<(), ElementError> {
        self.started = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ElementError> {
        self.started = false;
        Ok(())
    }

    fn destroy(&mut self) -> Result<(), ElementError> {
        self.init_called = false;
        self.started = false;
        Ok(())
    }

    fn provides(&self) -> Vec<CapabilityAccess> {
        vec![
            CapabilityAccess {
                name: "memory.store",
                description: "Store a memory in the ReasoningBank",
                operations: vec![CapabilityOp::Command],
            },
            CapabilityAccess {
                name: "memory.recall",
                description: "Query stored memories",
                operations: vec![CapabilityOp::Query],
            },
        ]
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_memory_element() {
        let el = MemoryElement::new(100);
        assert_eq!(el.id(), "element.memory");
        assert_eq!(el.recall_count(), 0);
    }

    #[test]
    fn test_init_and_start() {
        let bus = ElementBus::new();
        let mut el = MemoryElement::new(100);
        el.init(&bus).expect("value should be ok in test");
        el.start().expect("value should be ok in test");
    }

    #[test]
    fn test_store_memory() {
        let mut el = MemoryElement::new(100);
        el.store("test memory", 0.9);
        assert_eq!(el.recall_count(), 1);
    }

    #[test]
    fn test_provides() {
        let el = MemoryElement::new(100);
        let provides = el.provides();
        assert!(provides.iter().any(|c| c.name == "memory.store"));
    }
}
