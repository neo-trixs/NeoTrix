use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use super::{ElementError, ElementId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventKind {
    CapabilityUpdated,
    MemoryStored,
    GoalStateChanged,
    TraceCompleted,
    RegistryStateChanged,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum EventPayload {
    Text(String),
    Number(f64),
    Map(HashMap<String, String>),
    None,
}

#[derive(Debug, Clone)]
pub struct BusMessage {
    pub source: ElementId,
    pub kind: EventKind,
    pub payload: EventPayload,
}

type SubscriptionMap = Arc<Mutex<HashMap<EventKind, Vec<(ElementId, mpsc::Sender<EventPayload>)>>>>;

#[derive(Clone, Debug)]
pub struct ElementBus {
    subscriptions: SubscriptionMap,
}

impl ElementBus {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn publish(&self, _source: ElementId, kind: EventKind, payload: EventPayload) {
        let subs = self.subscriptions.lock().expect("result");
        if let Some(subscribers) = subs.get(&kind) {
            for (_id, sender) in subscribers {
                let _ = sender.try_send(payload.clone());
            }
        }
    }

    pub fn subscribe(
        &self,
        subscriber: ElementId,
        kind: EventKind,
    ) -> mpsc::Receiver<EventPayload> {
        let (tx, rx) = mpsc::channel(64);
        let mut subs = self.subscriptions.lock().expect("result");
        subs.entry(kind)
            .or_default()
            .push((subscriber, tx));
        rx
    }

    pub fn send_command(
        &self,
        _target: ElementId,
        _command: &str,
        _payload: EventPayload,
    ) -> Result<EventPayload, ElementError> {
        Err(ElementError::BusError("direct command not yet supported".into()))
    }

    #[cfg(test)]
    pub fn mock() -> Self {
        Self::new()
    }
}

impl Default for ElementBus {
    fn default() -> Self {
        Self::new()
    }
}
