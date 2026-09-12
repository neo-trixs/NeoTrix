use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};

/// Event trait with Any + Send + Sync
pub trait GameEvent: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

/// Event handler trait
pub trait GameEventHandler: Send + Sync {
    fn handle(&mut self, event: &dyn GameEvent, entity: Option<u64>);
    fn name(&self) -> &str;
    fn priority(&self) -> i32 { 0 }
}

/// Wrapper for type-erased events
struct EventEntry {
    type_id: TypeId,
    event: Box<dyn GameEvent>,
}

/// Typed event bus with priority handlers
pub struct TypedEventBus {
    queues: VecDeque<EventEntry>,
    handlers: HashMap<TypeId, Vec<Box<dyn GameEventHandler>>>,
    event_count: u64,
}

impl TypedEventBus {
    pub fn new() -> Self {
        Self {
            queues: VecDeque::new(),
            handlers: HashMap::new(),
            event_count: 0,
        }
    }

    /// Send an event (queue for processing)
    pub fn send<T: GameEvent>(&mut self, event: T) {
        self.queues.push_back(EventEntry {
            type_id: TypeId::of::<T>(),
            event: Box::new(event),
        });
        self.event_count += 1;
    }

    /// Register a handler for a specific event type
    pub fn register_handler<T: GameEvent>(&mut self, handler: Box<dyn GameEventHandler>) {
        let type_id = TypeId::of::<T>();
        self.handlers.entry(type_id)
            .or_insert_with(Vec::new)
            .push(handler);

        // Sort handlers by priority (higher priority first)
        if let Some(handlers) = self.handlers.get_mut(&type_id) {
            handlers.sort_by(|a, b| b.priority().cmp(&a.priority()));
        }
    }

    /// Process all queued events (calls handlers)
    pub fn process_all(&mut self) {
        let entries: Vec<EventEntry> = self.queues.drain(..).collect();

        for entry in entries {
            if let Some(handlers) = self.handlers.get_mut(&entry.type_id) {
                for handler in handlers.iter_mut() {
                    handler.handle(entry.event.as_ref(), None);
                }
            }
        }
    }

    /// Process events and get count processed
    pub fn process_all_with_count(&mut self) -> usize {
        let count = self.queues.len();
        self.process_all();
        count
    }

    /// Peek at queue length
    pub fn queue_len(&self) -> usize {
        self.queues.len()
    }

    /// Total events sent
    pub fn event_count(&self) -> u64 {
        self.event_count
    }

    /// Clear all queues
    pub fn clear(&mut self) {
        self.queues.clear();
    }
}

impl Default for TypedEventBus {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // Test event types
    struct DamageEvent { amount: i32 }
    impl GameEvent for DamageEvent {
        fn as_any(&self) -> &dyn Any { self }
    }

    struct HealEvent { amount: i32 }
    impl GameEvent for HealEvent {
        fn as_any(&self) -> &dyn Any { self }
    }

    // Test handler
    struct DamageHandler {
        log: Arc<Mutex<Vec<String>>>,
    }
    impl GameEventHandler for DamageHandler {
        fn handle(&mut self, event: &dyn GameEvent, _entity: Option<u64>) {
            if let Some(dmg) = event.as_any().downcast_ref::<DamageEvent>() {
                self.log.lock().unwrap().push(format!("damage:{}", dmg.amount));
            }
        }
        fn name(&self) -> &str { "DamageHandler" }
    }

    struct HighPriorityHandler {
        log: Arc<Mutex<Vec<String>>>,
    }
    impl GameEventHandler for HighPriorityHandler {
        fn handle(&mut self, event: &dyn GameEvent, _entity: Option<u64>) {
            if let Some(dmg) = event.as_any().downcast_ref::<DamageEvent>() {
                self.log.lock().unwrap().push(format!("high:{}", dmg.amount));
            }
        }
        fn name(&self) -> &str { "HighPriority" }
        fn priority(&self) -> i32 { 100 }
    }

    #[test]
    fn test_event_bus_send_receive() {
        let mut bus = TypedEventBus::new();
        bus.send(DamageEvent { amount: 10 });
        assert_eq!(bus.queue_len(), 1);
        bus.process_all();
        assert_eq!(bus.queue_len(), 0);
    }

    #[test]
    fn test_event_bus_handler_called() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut bus = TypedEventBus::new();

        bus.register_handler::<DamageEvent>(Box::new(DamageHandler { log: log.clone() }));
        bus.send(DamageEvent { amount: 25 });
        bus.process_all();

        let log = log.lock().unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0], "damage:25");
    }

    #[test]
    fn test_priority_ordering() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut bus = TypedEventBus::new();

        // Register low priority first, then high
        bus.register_handler::<DamageEvent>(Box::new(DamageHandler { log: log.clone() }));
        bus.register_handler::<DamageEvent>(Box::new(HighPriorityHandler { log: log.clone() }));

        bus.send(DamageEvent { amount: 5 });
        bus.process_all();

        let log = log.lock().unwrap();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0], "high:5"); // High priority first
        assert_eq!(log[1], "damage:5");
    }

    struct HealHandler {
        log: Arc<Mutex<Vec<String>>>,
    }
    impl GameEventHandler for HealHandler {
        fn handle(&mut self, event: &dyn GameEvent, _entity: Option<u64>) {
            if let Some(heal) = event.as_any().downcast_ref::<HealEvent>() {
                self.log.lock().unwrap().push(format!("heal:{}", heal.amount));
            }
        }
        fn name(&self) -> &str { "HealHandler" }
    }

    #[test]
    fn test_multiple_event_types() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut bus = TypedEventBus::new();

        bus.register_handler::<DamageEvent>(Box::new(DamageHandler { log: log.clone() }));
        bus.register_handler::<HealEvent>(Box::new(HealHandler { log: log.clone() }));

        bus.send(DamageEvent { amount: 10 });
        bus.send(HealEvent { amount: 5 });
        bus.process_all();

        let log = log.lock().unwrap();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0], "damage:10");
        assert_eq!(log[1], "heal:5");
    }
}
