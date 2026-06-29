use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::sync::broadcast;

/// Global relay so the Tauri IPC bridge can receive events from all BrainEventBus instances.
fn global_sender() -> Option<broadcast::Sender<BrainEvent>> {
    GLOBAL_RELAY.get().cloned()
}

static GLOBAL_RELAY: OnceLock<broadcast::Sender<BrainEvent>> = OnceLock::new();

/// Get the global relay sender, if initialized, for one-shot emits.
pub fn get_global_sender() -> Option<broadcast::Sender<BrainEvent>> {
    GLOBAL_RELAY.get().cloned()
}

/// Get a helper that wraps the global sender for convenience
pub struct GlobalBus;

impl GlobalBus {
    pub fn emit(event: BrainEvent) {
        if let Some(tx) = get_global_sender() {
            if tx.send(event).is_err() {
                log::warn!("brain_event_bus GlobalBus::emit send failed: channel closed");
            }
        }
    }
}

/// Pipeline stage execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StageStatus {
    #[serde(rename = "start")]
    Start,
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "skip")]
    Skip,
    #[serde(rename = "error")]
    Error,
}

/// Cognitive load mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadMode {
    #[serde(rename = "fast")]
    Fast,
    #[serde(rename = "balanced")]
    Balanced,
    #[serde(rename = "deep")]
    Deep,
}

/// Degradation level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DegradationLevel {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "reduced")]
    Reduced,
    #[serde(rename = "limited")]
    Limited,
    #[serde(rename = "minimal")]
    Minimal,
}

/// Knowledge action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeAction {
    #[serde(rename = "absorbed")]
    Absorbed,
    #[serde(rename = "forgotten")]
    Forgotten,
    #[serde(rename = "archived")]
    Archived,
}

/// Tool execution mode (consciousness-invoked vs user-invoked)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolOrigin {
    #[serde(rename = "consciousness")]
    Consciousness,
    #[serde(rename = "user")]
    User,
}

/// A structured event emitted by the SEAL pipeline for frontend visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum BrainEvent {
    #[serde(rename = "stage")]
    Stage {
        name: String,
        status: StageStatus,
        duration_ms: Option<u64>,
    },
    #[serde(rename = "consciousness")]
    Consciousness {
        tag: String,
        coherence: f64,
        buffer_usage: f64,
        novelty: f64,
    },
    #[serde(rename = "health")]
    Health {
        health_score: f64,
        cognitive_load: LoadMode,
        degradation: DegradationLevel,
        curiosity_bonus: f64,
        iteration: u64,
    },
    #[serde(rename = "knowledge")]
    Knowledge {
        action: KnowledgeAction,
        domain: String,
        concept_count: usize,
    },
    #[serde(rename = "tool")]
    Tool {
        tool: String,
        success: bool,
        duration_ms: u64,
        origin: ToolOrigin,
        summary: String,
    },
    #[serde(rename = "webapp")]
    WebAppData {
        url: String,
        app_type: String,
        title: String,
        content_length: usize,
        extracted: bool,
    },
}

/// Event bus that emits brain events to subscribed listeners.
/// Uses a tokio broadcast channel — if no listeners, events are dropped.
pub struct BrainEventBus {
    sender: broadcast::Sender<BrainEvent>,
}

impl BrainEventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn sender(&self) -> broadcast::Sender<BrainEvent> {
        self.sender.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<BrainEvent> {
        self.sender.subscribe()
    }

    pub fn emit(&self, event: BrainEvent) {
        if self.sender.send(event.clone()).is_err() {
            log::warn!("brain_event_bus::emit local send failed: channel closed");
        }
        if let Some(tx) = global_sender() {
            if tx.send(event).is_err() {
                log::warn!("brain_event_bus::emit global relay send failed: channel closed");
            }
        }
    }

    /// Initialize the global relay. Returns a Receiver that gets a copy of every
    /// event emitted by any BrainEventBus. Call once at startup (e.g. from the Tauri app).
    pub fn init_global(capacity: usize) -> broadcast::Receiver<BrainEvent> {
        let (tx, rx) = broadcast::channel(capacity);
        let _ = GLOBAL_RELAY.set(tx);
        rx
    }
}

impl Default for BrainEventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_and_receive() -> Result<(), String> {
        let bus = BrainEventBus::new(16);
        let mut rx = bus.subscribe();
        bus.emit(BrainEvent::Stage {
            name: "Collate".into(),
            status: StageStatus::Start,
            duration_ms: None,
        });
        let received = rx.try_recv().expect("should receive event");
        match received {
            BrainEvent::Stage { name, status, .. } => {
                assert_eq!(name, "Collate");
                assert!(matches!(status, StageStatus::Start));
                Ok(())
            }
            _ => Err("expected stage event".into()),
        }
    }

    #[test]
    fn test_health_event_serialization() {
        let event = BrainEvent::Health {
            health_score: 85.0,
            cognitive_load: LoadMode::Balanced,
            degradation: DegradationLevel::Full,
            curiosity_bonus: 0.5,
            iteration: 42,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"health_score\":85.0"));
        assert!(json.contains("\"kind\":\"health\""));
    }

    #[test]
    fn test_stage_event_roundtrip() -> Result<(), String> {
        let event = BrainEvent::Stage {
            name: "DmnStage".into(),
            status: StageStatus::Done,
            duration_ms: Some(42),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: BrainEvent = serde_json::from_str(&json).unwrap();
        match parsed {
            BrainEvent::Stage {
                name,
                status,
                duration_ms,
            } => {
                assert_eq!(name, "DmnStage");
                assert!(matches!(status, StageStatus::Done));
                assert_eq!(duration_ms, Some(42));
                Ok(())
            }
            _ => Err("expected stage event".into()),
        }
    }

    #[test]
    fn test_broadcast_capacity_drops_old() {
        let bus = BrainEventBus::new(2);
        let mut rx = bus.subscribe();
        bus.emit(BrainEvent::Stage {
            name: "first".into(),
            status: StageStatus::Start,
            duration_ms: None,
        });
        bus.emit(BrainEvent::Stage {
            name: "second".into(),
            status: StageStatus::Done,
            duration_ms: None,
        });
        bus.emit(BrainEvent::Stage {
            name: "third".into(),
            status: StageStatus::Skip,
            duration_ms: None,
        });
        // Capacity=2, wrote 3 — first event is dropped.
        // After 3 writes to cap=2, the receiver may get Lagged(1)
        let received = rx.try_recv();
        match received {
            Ok(BrainEvent::Stage { name, .. }) => {
                // Might get "second" or "third" depending on broadcast internals
                assert!(name == "second" || name == "third");
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(n)) => {
                // Lagged is also valid — subscriber was too slow
                assert!(n >= 1);
            }
            other => panic!("unexpected result: {:?}", other),
        }
    }
}
