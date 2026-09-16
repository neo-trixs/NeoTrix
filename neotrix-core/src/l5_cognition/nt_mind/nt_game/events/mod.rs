use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventId(u64);

pub trait Event: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn event_type(&self) -> &str;
}

pub type EventCallback = Box<dyn Fn(&dyn Event) + Send + Sync>;

pub struct EventBus {
    listeners: HashMap<TypeId, Vec<Box<dyn Fn(&dyn Event) + Send + Sync>>>,
    event_log: Vec<Box<dyn Event>>,
    max_log_size: usize,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
            event_log: Vec::new(),
            max_log_size: 100,
        }
    }

    pub fn subscribe<T: Event + 'static>(&mut self, callback: impl Fn(&T) + Send + Sync + 'static) {
        let wrapper = Box::new(move |event: &dyn Event| {
            if let Some(typed) = event.as_any().downcast_ref::<T>() {
                callback(typed);
            }
        });
        self.listeners
            .entry(TypeId::of::<T>())
            .or_default()
            .push(wrapper);
    }

    pub fn emit<T: Event + 'static>(&mut self, event: T) {
        if self.event_log.len() >= self.max_log_size {
            self.event_log.remove(0);
        }
        self.event_log.push(Box::new(event));

        if let Some(callbacks) = self.listeners.get(&TypeId::of::<T>()) {
            let event_ref = self.event_log.last().unwrap().as_ref();
            for callback in callbacks {
                callback(event_ref);
            }
        }
    }

    pub fn log_size(&self) -> usize {
        self.event_log.len()
    }

    pub fn clear_log(&mut self) {
        self.event_log.clear();
    }

    pub fn listener_count<T: Event + 'static>(&self) -> usize {
        self.listeners
            .get(&TypeId::of::<T>())
            .map_or(0, |l| l.len())
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct EntityCreatedEvent {
    pub entity_id: u64,
}
impl Event for EntityCreatedEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn event_type(&self) -> &str {
        "entity_created"
    }
}

#[derive(Debug, Clone)]
pub struct EntityDestroyedEvent {
    pub entity_id: u64,
}
impl Event for EntityDestroyedEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn event_type(&self) -> &str {
        "entity_destroyed"
    }
}

#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub entity_a: u64,
    pub entity_b: u64,
    pub x: f64,
    pub y: f64,
}
impl Event for CollisionEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn event_type(&self) -> &str {
        "collision"
    }
}

#[derive(Debug, Clone)]
pub struct DamageEvent {
    pub source: u64,
    pub target: u64,
    pub amount: f64,
}
impl Event for DamageEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn event_type(&self) -> &str {
        "damage"
    }
}

#[derive(Debug, Clone)]
pub struct GameStateChangedEvent {
    pub state_name: String,
    pub old_value: String,
    pub new_value: String,
}
impl Event for GameStateChangedEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn event_type(&self) -> &str {
        "game_state_changed"
    }
}

#[derive(Debug, Clone)]
pub struct TurnEndedEvent {
    pub turn: u64,
    pub player: String,
}
impl Event for TurnEndedEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn event_type(&self) -> &str {
        "turn_ended"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn test_emit_and_listen() {
        let mut bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        bus.subscribe::<DamageEvent>(move |_e| {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });
        bus.emit(DamageEvent {
            source: 1,
            target: 2,
            amount: 10.0,
        });
        bus.emit(DamageEvent {
            source: 1,
            target: 3,
            amount: 5.0,
        });
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_event_log() {
        let mut bus = EventBus::new();
        bus.emit(TurnEndedEvent {
            turn: 1,
            player: "A".into(),
        });
        assert_eq!(bus.log_size(), 1);
    }

    #[test]
    fn test_no_listeners() {
        let mut bus = EventBus::new();
        bus.emit(DamageEvent {
            source: 1,
            target: 2,
            amount: 10.0,
        });
        assert_eq!(bus.listener_count::<DamageEvent>(), 0);
    }
}
