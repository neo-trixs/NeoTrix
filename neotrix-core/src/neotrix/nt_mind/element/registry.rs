use super::{Element, ElementError, ElementId};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryState {
    Constructed,
    Initialized,
    Started,
    Stopped,
}

pub struct ElementRegistry {
    elements: HashMap<ElementId, Box<dyn Element>>,
    load_order: Vec<ElementId>,
    state: RegistryState,
}

impl Default for ElementRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ElementRegistry {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            load_order: Vec::new(),
            state: RegistryState::Constructed,
        }
    }

    pub fn register(&mut self, element: Box<dyn Element>) -> Result<(), ElementError> {
        let id = element.id().to_string();
        if self.elements.contains_key(&id) {
            return Err(ElementError::AlreadyRegistered(id));
        }
        self.elements.insert(id, element);
        Ok(())
    }

    pub fn register_all(&mut self, elements: Vec<Box<dyn Element>>) -> Result<(), ElementError> {
        for element in elements {
            self.register(element)?;
        }
        Ok(())
    }

    pub fn resolve_and_init(&mut self) -> Result<(), ElementError> {
        if self.state != RegistryState::Constructed {
            return Err(ElementError::RuntimeError(
                "can only resolve from Constructed state".into(),
            ));
        }

        let order = self.resolve_dependency_order()?;
        let bus = self.bus();

        for id in &order {
            if let Some(element) = self.elements.get_mut(id) {
                element.init(&bus)?;
            }
        }

        self.load_order = order;
        self.state = RegistryState::Initialized;
        Ok(())
    }

    pub fn start_all(&mut self) -> Result<(), ElementError> {
        if self.state != RegistryState::Initialized {
            return Err(ElementError::RuntimeError(
                "can only start from Initialized state".into(),
            ));
        }

        for id in &self.load_order {
            if let Some(element) = self.elements.get_mut(id) {
                element.start()?;
            }
        }

        self.state = RegistryState::Started;
        Ok(())
    }

    pub fn bootstrap(&mut self, elements: Vec<Box<dyn Element>>) -> Result<(), ElementError> {
        self.register_all(elements)?;
        self.resolve_and_init()?;
        self.start_all()
    }

    pub fn shutdown(&mut self) -> Result<(), ElementError> {
        for id in self.load_order.iter().rev() {
            if let Some(element) = self.elements.get_mut(id) {
                let _ = element.stop();
            }
        }

        for id in self.load_order.iter().rev() {
            if let Some(element) = self.elements.get_mut(id) {
                let _ = element.destroy();
            }
        }

        self.state = RegistryState::Stopped;
        Ok(())
    }

    fn resolve_dependency_order(&self) -> Result<Vec<ElementId>, ElementError> {
        let mut visited: HashSet<String> = HashSet::new();
        let mut visiting: HashSet<String> = HashSet::new();
        let mut order: Vec<ElementId> = Vec::new();

        let ids: Vec<ElementId> = self.elements.keys().cloned().collect();

        for id in ids {
            self.visit_dfs(&id, &mut visited, &mut visiting, &mut order)?;
        }

        Ok(order)
    }

    fn visit_dfs(
        &self,
        id: &str,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        order: &mut Vec<ElementId>,
    ) -> Result<(), ElementError> {
        if visiting.contains(id) {
            return Err(ElementError::DependencyNotMet(format!(
                "circular dependency detected involving '{}'",
                id
            )));
        }

        if visited.contains(id) {
            return Ok(());
        }

        visiting.insert(id.to_string());

        if let Some(element) = self.elements.get(id) {
            for dep in element.depends_on() {
                if !self.elements.contains_key(dep) {
                    return Err(ElementError::DependencyNotMet(format!(
                        "element '{}' depends on '{}' which is not registered",
                        id, dep
                    )));
                }
                self.visit_dfs(dep, visited, visiting, order)?;
            }
        }

        visiting.remove(id);
        visited.insert(id.to_string());
        order.push(id.to_string());

        Ok(())
    }

    pub fn get<T: Element>(&self, id: &str) -> Option<&T> {
        self.elements
            .get(id)
            .and_then(|e| e.as_ref().as_any().downcast_ref::<T>())
    }

    pub fn get_mut<T: Element>(&mut self, id: &str) -> Option<&mut T> {
        self.elements
            .get_mut(id)
            .and_then(|e| e.as_mut().as_any_mut().downcast_mut::<T>())
    }

    pub fn bus(&self) -> super::bus::ElementBus {
        super::bus::ElementBus::new()
    }

    pub fn list(&self) -> Vec<&str> {
        self.elements.keys().map(|s| s.as_str()).collect()
    }

    pub fn state(&self) -> RegistryState {
        self.state
    }

    pub fn element_count(&self) -> usize {
        self.elements.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind::element::bus::{ElementBus, EventKind, EventPayload};
    use crate::neotrix::nt_mind::element::ElementType;

    #[derive(Debug)]
    struct TestElement {
        id: String,
        name: String,
        deps: Vec<String>,
        init_called: bool,
        start_called: bool,
        stop_called: bool,
        destroy_called: bool,
    }

    impl TestElement {
        fn new(id: &str, name: &str, deps: Vec<&str>) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string(),
                deps: deps.into_iter().map(|s| s.to_string()).collect(),
                init_called: false,
                start_called: false,
                stop_called: false,
                destroy_called: false,
            }
        }
    }

    impl Element for TestElement {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            &self.name
        }
        fn version(&self) -> &str {
            "0.1.0"
        }
        fn element_type(&self) -> ElementType {
            ElementType::Core
        }

        fn init(&mut self, _bus: &ElementBus) -> Result<(), ElementError> {
            self.init_called = true;
            Ok(())
        }
        fn start(&mut self) -> Result<(), ElementError> {
            self.start_called = true;
            Ok(())
        }
        fn stop(&mut self) -> Result<(), ElementError> {
            self.stop_called = true;
            Ok(())
        }
        fn destroy(&mut self) -> Result<(), ElementError> {
            self.destroy_called = true;
            Ok(())
        }

        fn depends_on(&self) -> Vec<&str> {
            self.deps.iter().map(|s| s.as_str()).collect()
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_empty_registry() {
        let reg = ElementRegistry::new();
        assert_eq!(reg.state(), RegistryState::Constructed);
        assert!(reg.list().is_empty());
        assert_eq!(reg.element_count(), 0);
    }

    #[test]
    fn test_register_and_bootstrap() {
        let mut reg = ElementRegistry::new();
        let e = Box::new(TestElement::new("test.1", "Test One", vec![]));
        reg.register(e)
            .expect("register first element should succeed");
        assert_eq!(reg.element_count(), 1);
    }

    #[test]
    fn test_bootstrap_lifecycle() {
        let mut reg = ElementRegistry::new();
        let e = Box::new(TestElement::new("test.lifecycle", "Lifecycle", vec![]));
        reg.bootstrap(vec![e])
            .expect("bootstrap lifecycle element should start registry");
        assert_eq!(reg.state(), RegistryState::Started);

        let element = reg
            .get::<TestElement>("test.lifecycle")
            .expect("test.lifecycle must exist after bootstrap");
        assert!(element.init_called);
        assert!(element.start_called);

        reg.shutdown()
            .expect("shutdown running registry should succeed");
        assert_eq!(reg.state(), RegistryState::Stopped);
    }

    #[test]
    fn test_shutdown_calls_stop_destroy() {
        let mut reg = ElementRegistry::new();
        let e = Box::new(TestElement::new("test.shutdown", "Shutdown", vec![]));
        reg.bootstrap(vec![e])
            .expect("bootstrap shutdown element should start registry");
        reg.shutdown()
            .expect("shutdown should stop and clean up registry");

        let element = reg
            .get::<TestElement>("test.shutdown")
            .expect("test.shutdown must exist after bootstrap");
        assert!(element.stop_called);
        assert!(element.destroy_called);
    }

    #[test]
    fn test_dependency_order() {
        let mut reg = ElementRegistry::new();
        let a = Box::new(TestElement::new("A", "Base", vec![]));
        let b = Box::new(TestElement::new("B", "Depends on A", vec!["A"]));
        reg.register_all(vec![a, b])
            .expect("register_all should register all elements");
        reg.resolve_and_init()
            .expect("resolve_and_init should initialize in dependency order");

        let order = reg.load_order.clone();
        let pos_a = order
            .iter()
            .position(|id| id == "A")
            .expect("element A must be in load_order");
        let pos_b = order
            .iter()
            .position(|id| id == "B")
            .expect("element B must be in load_order");
        assert!(pos_a < pos_b, "A must be before B in dependency order");
    }

    #[test]
    fn test_missing_dependency_errors() {
        let mut reg = ElementRegistry::new();
        let e = Box::new(TestElement::new("orphan", "Orphan", vec!["missing"]));
        reg.register(e)
            .expect("register orphan element should succeed");
        let result = reg.resolve_and_init();
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_registration_errors() {
        let mut reg = ElementRegistry::new();
        let a = Box::new(TestElement::new("dup", "First", vec![]));
        reg.register(a)
            .expect("register duplicate element should succeed on first attempt");
        let b = Box::new(TestElement::new("dup", "Second", vec![]));
        let result = reg.register(b);
        assert!(result.is_err());
    }

    #[test]
    fn test_bus_event_publish_subscribe() -> Result<(), String> {
        let bus = ElementBus::new();
        let mut rx = bus.subscribe("subscriber".into(), EventKind::CapabilityUpdated);

        bus.publish(
            "publisher".into(),
            EventKind::CapabilityUpdated,
            EventPayload::Text("test payload".into()),
        );

        let rt = tokio::runtime::Runtime::new().expect("create tokio runtime for bus test");
        let result = rt.block_on(async {
            tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await
        });

        match result {
            Ok(Some(EventPayload::Text(t))) => assert_eq!(t, "test payload"),
            other => return Err(format!("expected Text payload, got {:?}", other)),
        }
        Ok(())
    }

    #[test]
    fn test_get_by_type() {
        let mut reg = ElementRegistry::new();
        let e = Box::new(TestElement::new("test.get", "Get Test", vec![]));
        reg.bootstrap(vec![e])
            .expect("bootstrap element for get-by-type test");

        let found: Option<&TestElement> = reg.get("test.get");
        assert!(found.is_some());
        assert_eq!(
            found.expect("found must be Some after get").name(),
            "Get Test"
        );
    }

    #[test]
    fn test_start_from_wrong_state_fails() {
        let mut reg = ElementRegistry::new();
        // Try to start without initializing
        let e = Box::new(TestElement::new("bad-start", "Bad", vec![]));
        reg.register(e)
            .expect("register element for wrong-state test");
        let result = reg.start_all();
        assert!(result.is_err());
    }
}
