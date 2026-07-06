use std::any::Any;
use crate::core::nt_core_cap::CapabilityVector;
use crate::neotrix::nt_mind::core::KnowledgeSource;
use super::bus::ElementBus;
use super::{Element, ElementError, ElementType, CapabilityAccess, CapabilityOp};

#[derive(Debug)]
pub struct CapabilityElement {
    pub capability: CapabilityVector,
    init_called: bool,
    started: bool,
}

impl Default for CapabilityElement {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityElement {
    pub fn new() -> Self {
        Self {
            capability: CapabilityVector::default(),
            init_called: false,
            started: false,
        }
    }

    pub fn absorb(&mut self, source: KnowledgeSource) {
        let v = source.capability_vector();
        self.capability.update_from_other(&v, 0.3);
        self.capability.normalize();
    }

    pub fn vector(&self) -> &CapabilityVector {
        &self.capability
    }

    pub fn vector_mut(&mut self) -> &mut CapabilityVector {
        &mut self.capability
    }
}

impl Element for CapabilityElement {
    fn id(&self) -> &str { "element.capability" }
    fn name(&self) -> &str { "Capability Vector" }
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
                name: "capability.query",
                description: "Read the current capability vector",
                operations: vec![CapabilityOp::Query],
            },
            CapabilityAccess {
                name: "capability.absorb",
                description: "Absorb knowledge from a source",
                operations: vec![CapabilityOp::Command],
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
    fn test_new_capability_element() {
        let el = CapabilityElement::new();
        assert_eq!(el.id(), "element.capability");
        assert_eq!(el.version(), "0.2.0");
        assert_eq!(el.element_type(), ElementType::Core);
        assert!(!el.init_called);
    }

    #[test]
    fn test_init_and_start() {
        let bus = ElementBus::new();
        let mut el = CapabilityElement::new();
        el.init(&bus).expect("value should be ok in test");
        el.start().expect("value should be ok in test");
        assert!(el.init_called);
        assert!(el.started);
    }

    #[test]
    fn test_absorb_updates_vector() {
        let mut el = CapabilityElement::new();
        let initial: f64 = el.capability.arr.iter().sum();
        el.absorb(KnowledgeSource::HeroUI);
        let after: f64 = el.capability.arr.iter().sum();
        assert!((after - initial).abs() > 1e-10);
    }

    #[test]
    fn test_provides_has_expected_entries() {
        let el = CapabilityElement::new();
        let provides = el.provides();
        assert!(provides.iter().any(|c| c.name == "capability.query"));
        assert!(provides.iter().any(|c| c.name == "capability.absorb"));
    }

    #[test]
    fn test_vector_accessors() {
        let mut el = CapabilityElement::new();
        el.absorb(KnowledgeSource::HeroUI);
        let v = el.vector();
        assert_eq!(v.arr.len(), 23);
        let vm = el.vector_mut();
        vm.arr[0] = 0.99;
        assert!((el.capability.arr[0] - 0.99).abs() < 1e-10);
    }
}
