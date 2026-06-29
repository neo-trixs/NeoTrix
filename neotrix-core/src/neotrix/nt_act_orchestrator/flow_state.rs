use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A typed state identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlowStateId(Uuid);

impl FlowStateId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Trait for types that can serve as flow state.
pub trait ConfigState: Serialize + std::fmt::Debug + Clone + Send + Sync {
    fn state_id(&self) -> FlowStateId;
    fn merge(&mut self, other: Self);
}

/// Manages state transitions with history tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateManager<T: ConfigState> {
    pub current: T,
    pub history: Vec<(FlowStateId, T)>,
    current_id: FlowStateId,
}

impl<T: ConfigState> StateManager<T> {
    pub fn new(initial: T) -> Self {
        let id = initial.state_id();
        Self {
            current: initial,
            history: vec![],
            current_id: id,
        }
    }

    const MAX_HISTORY: usize = 5000;

    pub fn push(&mut self, new_state: T) {
        let id = new_state.state_id();
        self.history
            .push((self.current_id.clone(), self.current.clone()));
        if self.history.len() > Self::MAX_HISTORY {
            self.history.drain(0..Self::MAX_HISTORY / 5);
        }
        self.current = new_state;
        self.current_id = id;
    }

    pub fn current_id(&self) -> &FlowStateId {
        &self.current_id
    }

    pub fn len(&self) -> usize {
        self.history.len() + 1
    }

    pub fn is_empty(&self) -> bool {
        false // always has at least initial state
    }

    pub fn at_index(&self, index: usize) -> Option<&T> {
        if index < self.history.len() {
            Some(&self.history[index].1)
        } else if index == self.history.len() {
            Some(&self.current)
        } else {
            None
        }
    }

    pub fn rollback(&mut self) -> Option<T> {
        let (id, prev) = self.history.pop()?;
        self.current_id = id;
        let old = std::mem::replace(&mut self.current, prev);
        Some(old)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestState {
        id: FlowStateId,
        value: i32,
        label: String,
    }

    impl ConfigState for TestState {
        fn state_id(&self) -> FlowStateId {
            self.id.clone()
        }
        fn merge(&mut self, other: Self) {
            self.value = other.value;
            self.label = other.label;
        }
    }

    #[test]
    fn test_state_serialize() {
        let state = TestState {
            id: FlowStateId::new(),
            value: 42,
            label: "test".into(),
        };
        let json = serde_json::to_string(&state).expect("state should serialize");
        let deserialized: TestState =
            serde_json::from_str(&json).expect("state should deserialize");
        assert_eq!(state.value, deserialized.value);
        assert_eq!(state.label, deserialized.label);
    }

    #[test]
    fn test_state_manager_push() {
        let initial = TestState {
            id: FlowStateId::new(),
            value: 0,
            label: "start".into(),
        };
        let mut mgr = StateManager::new(initial);
        assert_eq!(mgr.len(), 1);

        let next = TestState {
            id: FlowStateId::new(),
            value: 1,
            label: "step1".into(),
        };
        mgr.push(next);
        assert_eq!(mgr.len(), 2);
        assert_eq!(mgr.current.value, 1);
    }

    #[test]
    fn test_state_manager_history() {
        let initial = TestState {
            id: FlowStateId::new(),
            value: 0,
            label: "init".into(),
        };
        let mut mgr = StateManager::new(initial);

        mgr.push(TestState {
            id: FlowStateId::new(),
            value: 10,
            label: "a".into(),
        });
        mgr.push(TestState {
            id: FlowStateId::new(),
            value: 20,
            label: "b".into(),
        });

        assert_eq!(
            mgr.at_index(0)
                .expect("state at index 0 should exist")
                .value,
            0
        );
        assert_eq!(
            mgr.at_index(1)
                .expect("state at index 1 should exist")
                .value,
            10
        );
        assert_eq!(
            mgr.at_index(2)
                .expect("state at index 2 should exist")
                .value,
            20
        );
        assert!(mgr.at_index(3).is_none());
    }

    #[test]
    fn test_state_rollback() {
        let initial = TestState {
            id: FlowStateId::new(),
            value: 0,
            label: "init".into(),
        };
        let mut mgr = StateManager::new(initial);

        mgr.push(TestState {
            id: FlowStateId::new(),
            value: 1,
            label: "one".into(),
        });
        mgr.push(TestState {
            id: FlowStateId::new(),
            value: 2,
            label: "two".into(),
        });

        let rolled = mgr.rollback();
        assert!(rolled.is_some());
        assert_eq!(rolled.expect("rolled back state should exist").value, 2);
        assert_eq!(mgr.current.value, 1);
        assert_eq!(mgr.len(), 2);
    }

    #[test]
    fn test_uuid_unique() {
        let a = FlowStateId::new();
        let b = FlowStateId::new();
        assert_ne!(a, b);
    }
}
