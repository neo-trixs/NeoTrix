use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::core::nt_core_event::CoreEvent;

/// Type-safe event bus using tokio broadcast channel.
/// No `dyn Any` downcasting — every event is `CoreEvent` enum.
pub struct EventBus {
    sender: broadcast::Sender<CoreEvent>,
    log_file: Option<std::sync::Mutex<std::fs::File>>,
    shutdown_flag: Arc<AtomicBool>,
    handles: std::sync::Mutex<Vec<std::thread::JoinHandle<()>>>,
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self { sender: self.sender.clone(), log_file: None, shutdown_flag: Arc::new(AtomicBool::new(false)), handles: std::sync::Mutex::new(Vec::new()) }
    }
}

impl Default for EventBus {
    fn default() -> Self { Self::new(1024) }
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender, log_file: None, shutdown_flag: Arc::new(AtomicBool::new(false)), handles: std::sync::Mutex::new(Vec::new()) }
    }

    pub fn with_persistence(path: PathBuf) -> Self {
        let file = match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            Ok(f) => Some(std::sync::Mutex::new(f)),
            Err(e) => {
                log::warn!("[event-bus] open log: {}", e);
                None
            }
        };
        let (sender, _) = broadcast::channel(1024);
        Self { sender, log_file: file, shutdown_flag: Arc::new(AtomicBool::new(false)), handles: std::sync::Mutex::new(Vec::new()) }
    }

    /// Emit an event to all subscribers.
    /// If logging is enabled, also persists as JSON line.
    pub fn emit(&self, event: CoreEvent) {
        if let Some(ref log_file) = self.log_file {
            if let Ok(mut file) = log_file.lock() {
                if let Ok(line) = serde_json::to_string(&event) {
                    use std::io::Write;
                    let _ = writeln!(file, "{}", line);
                }
            }
        }
        if let Err(e) = self.sender.send(event) {
            log::warn!("[event_bus] drop core event: {:?}", e);
        }
    }

    /// Subscribe to events. Returns a receiver that gets every emitted event.
    /// If the receiver is too slow, it will receive `Lagged(n)` on next recv.
    pub fn subscribe(&self) -> broadcast::Receiver<CoreEvent> {
        self.sender.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<CoreEvent> {
        self.sender.clone()
    }

    /// Set the shutdown flag and join all spawned subscriber threads.
    /// Each thread gets up to 2s to exit after the flag is set.
    pub fn shutdown(&self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);
        let handles = self.handles.lock().ok();
        if let Some(mut handles) = handles {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
            for h in handles.drain(..) {
                if std::time::Instant::now() > deadline {
                    log::warn!("[event-bus] shutdown timeout reached, detaching remaining threads");
                    break;
                }
                let tid = h.thread().id();
                if h.join().is_err() {
                    log::warn!("[event-bus] thread {:?} panicked", tid);
                }
            }
        }
    }
}

impl Drop for EventBus {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Replay events from a JSONL log file, calling `callback` for each.
pub fn replay<F>(path: &PathBuf, callback: F)
where
    F: Fn(CoreEvent) + Send + Sync + 'static,
{
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    for line in content.lines() {
        if let Ok(event) = serde_json::from_str::<CoreEvent>(line) {
            callback(event);
        }
    }
}

// ── Layer-aware subscriber registration ────────────────────────────────────
// Each NeoTrix layer (L1-L9) gets a dedicated event subscriber that logs
// relevant events and routes them to the layer's diagnostic subsystem.
// This implements the "淋巴循环" (lymphatic circulation) — events flow
// through all layers so every subsystem has awareness of system-wide state.

/// Layer identifier for event routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerId {
    L1Body,
    L2World,
    L3Memory,
    L4Knowledge,
    L5Reasoning,
    L6Self,
    L7Capability,
    L8Autonomic,
    L9Meta,
}

impl LayerId {
    pub fn label(&self) -> &str {
        match self {
            LayerId::L1Body => "L1",
            LayerId::L2World => "L2",
            LayerId::L3Memory => "L3",
            LayerId::L4Knowledge => "L4",
            LayerId::L5Reasoning => "L5",
            LayerId::L6Self => "L6",
            LayerId::L7Capability => "L7",
            LayerId::L8Autonomic => "L8",
            LayerId::L9Meta => "L9",
        }
    }
}

fn filter_event_for_layer(event: &CoreEvent, layer: LayerId) -> bool {
    match (event, layer) {
        // L1 (Body/I-O): task submission and agent feedback
        (CoreEvent::TaskSubmitted { .. }, LayerId::L1Body) => true,
        (CoreEvent::AgentFeedback { .. }, LayerId::L1Body) => true,
        (CoreEvent::AgentTeam { .. }, LayerId::L1Body) => true,
        // L2 (World): external rewards
        (CoreEvent::ExternalReward { .. }, LayerId::L2World) => true,
        // L3 (Memory): goal completion and budget
        (CoreEvent::GoalCompleted { .. }, LayerId::L3Memory) => true,
        (CoreEvent::BudgetExceeded { .. }, LayerId::L3Memory) => true,
        // L4 (Knowledge): system errors (data integrity)
        (CoreEvent::SystemError { component, .. }, LayerId::L4Knowledge) => component.contains("kb") || component.contains("store"),
        // L5 (Reasoning): all events relevant to reasoning
        (CoreEvent::TaskSubmitted { .. }, LayerId::L5Reasoning) => true,
        (CoreEvent::GoalCompleted { .. }, LayerId::L5Reasoning) => true,
        (CoreEvent::ExternalReward { .. }, LayerId::L5Reasoning) => true,
        // L6 (Self): meta-cognitive events
        (CoreEvent::AgentFeedback { .. }, LayerId::L6Self) => true,
        (CoreEvent::GoalCompleted { .. }, LayerId::L6Self) => true,
        // L7 (Capability): agent team events
        (CoreEvent::AgentTeam { .. }, LayerId::L7Capability) => true,
        // L8 (Autonomic): system errors and global halt
        (CoreEvent::SystemError { .. }, LayerId::L8Autonomic) => true,
        (CoreEvent::GlobalHalt { .. }, LayerId::L8Autonomic) => true,
        (CoreEvent::BudgetExceeded { .. }, LayerId::L8Autonomic) => true,
        // L9 (Meta): all critical events
        (CoreEvent::GlobalHalt { .. }, LayerId::L9Meta) => true,
        (CoreEvent::SystemError { severity, .. }, LayerId::L9Meta) => severity == "critical",
        _ => false,
    }
}

/// Register a subscriber for a specific layer.
/// Returns the tokio task handle so the caller can keep it alive.
pub fn subscribe_layer(bus: &EventBus, layer: LayerId) -> tokio::task::JoinHandle<()> {
    let mut rx = bus.subscribe();
    let layer_label = layer.label().to_string();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if !filter_event_for_layer(&event, layer) {
                        continue;
                    }
                    match &event {
                        crate::core::nt_core_event::CoreEvent::SystemError { severity, component, error } if severity == "critical" => {
                            log::error!("[event-bus:{}] CRITICAL: {}: {}", layer_label, component, error);
                        }
                        crate::core::nt_core_event::CoreEvent::SystemError { severity, component, error } if severity == "error" => {
                            log::warn!("[event-bus:{}] ERROR: {}: {}", layer_label, component, error);
                        }
                        crate::core::nt_core_event::CoreEvent::GlobalHalt { reason, source } => {
                            log::error!("[event-bus:{}] GLOBAL HALT: {} from {}", layer_label, reason, source);
                        }
                            crate::core::nt_core_event::CoreEvent::ConsciousnessCritique { quality, .. } if *quality < crate::neotrix::l8_autonomic_impl::nt_mind_background_loop::CONSCIOUSNESS_THRESHOLDS.eventbus_critical => {
                            log::warn!("[event-bus:{}] consciousness quality LOW ({:.3})", layer_label, quality);
                        }
                        _ => {
                            log::trace!("[event-bus:{}] {:?}", layer_label, event);
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    log::warn!("[event-bus:{}] lagged {} events", layer_label, n);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    log::info!("[event-bus:{}] channel closed", layer_label);
                    break;
                }
            }
        }
    })
}

/// Convenience: register one subscriber for every layer (L1-L9).
/// Returns handles so the caller can keep them alive for the process lifetime.
pub fn subscribe_all_layers(bus: &EventBus) -> Vec<tokio::task::JoinHandle<()>> {
    vec![
        subscribe_layer(bus, LayerId::L1Body),
        subscribe_layer(bus, LayerId::L2World),
        subscribe_layer(bus, LayerId::L3Memory),
        subscribe_layer(bus, LayerId::L4Knowledge),
        subscribe_layer(bus, LayerId::L5Reasoning),
        subscribe_layer(bus, LayerId::L6Self),
        subscribe_layer(bus, LayerId::L7Capability),
        subscribe_layer(bus, LayerId::L8Autonomic),
        subscribe_layer(bus, LayerId::L9Meta),
    ]
}

/// Synchronous version for environments without tokio runtime.
/// Spawns a std::thread per layer subscriber. Threads check an `Arc<AtomicBool>`
/// shutdown flag and exit cleanly when `EventBus::shutdown()` is called.
pub fn subscribe_all_layers_sync(bus: &EventBus) {
    let layers = [
        LayerId::L1Body, LayerId::L2World, LayerId::L3Memory,
        LayerId::L4Knowledge, LayerId::L5Reasoning, LayerId::L6Self,
        LayerId::L7Capability, LayerId::L8Autonomic, LayerId::L9Meta,
    ];
    let handles: Vec<_> = layers.into_iter().map(|layer| {
        let mut rx = bus.subscribe();
        let layer_label = layer.label().to_string();
        let shutdown = Arc::clone(&bus.shutdown_flag);
        std::thread::spawn(move || {
            loop {
                if shutdown.load(Ordering::SeqCst) {
                    log::trace!("[event-bus:{}] shutdown signal received", layer_label);
                    break;
                }
                match rx.try_recv() {
                    Ok(event) => {
                        if !filter_event_for_layer(&event, layer) {
                            continue;
                        }
                        match &event {
                            crate::core::nt_core_event::CoreEvent::SystemError { severity, component, error } if severity == "critical" => {
                                log::error!("[event-bus:{}] CRITICAL: {}: {}", layer_label, component, error);
                            }
                            crate::core::nt_core_event::CoreEvent::SystemError { severity, component, error } if severity == "error" => {
                                log::warn!("[event-bus:{}] ERROR: {}: {}", layer_label, component, error);
                            }
                            crate::core::nt_core_event::CoreEvent::GlobalHalt { reason, source } => {
                                log::error!("[event-bus:{}] GLOBAL HALT: {} from {}", layer_label, reason, source);
                            }
                        crate::core::nt_core_event::CoreEvent::ConsciousnessCritique { quality, .. } if *quality < crate::neotrix::l8_autonomic_impl::nt_mind_background_loop::CONSCIOUSNESS_THRESHOLDS.eventbus_critical => {
                                log::warn!("[event-bus:{}] consciousness quality LOW ({:.3})", layer_label, quality);
                            }
                            _ => {
                                log::trace!("[event-bus:{}] {:?}", layer_label, event);
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                        log::info!("[event-bus:{}] channel closed", layer_label);
                        break;
                    }
                    Err(tokio::sync::broadcast::error::TryRecvError::Lagged(n)) => {
                        log::warn!("[event-bus:{}] lagged {} events", layer_label, n);
                    }
                }
            }
        })
    }).collect();
    if let Ok(mut guard) = bus.handles.lock() {
        guard.extend(handles);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_event::CoreEvent;

    #[test]
    fn test_event_bus_new() {
        let bus = EventBus::new(16);
        let _rx = bus.subscribe();
    }

    #[test]
    fn test_event_bus_emit_no_subscribers() {
        let bus = EventBus::new(16);
        bus.emit(CoreEvent::TaskSubmitted { task: "t".into(), task_type: "g".into(), priority: 1 });
    }

    #[test]
    fn test_event_bus_subscribe_and_receive() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        bus.emit(CoreEvent::GoalCompleted { goal_id: "g1".into(), goal: "test".into(), iterations: 5, score: 0.8 });
        let received = rx.try_recv().expect("should receive event");
        match received {
            CoreEvent::GoalCompleted { goal_id, .. } => {
                assert_eq!(goal_id, "g1");
            }
            _ => panic!("expected GoalCompleted"),
        }
    }

    #[test]
    fn test_event_bus_sender_clone() {
        let bus = EventBus::new(16);
        let s1 = bus.sender();
        let s2 = bus.sender();
        let mut rx = bus.subscribe();
        s1.send(CoreEvent::GlobalHalt { reason: "err".into(), source: "test".into() }).ok();
        let received = rx.try_recv().expect("should receive from s1");
        assert!(matches!(received, CoreEvent::GlobalHalt { .. }));
        s2.send(CoreEvent::SystemError { component: "db".into(), error: "timeout".into(), severity: "critical".into() }).ok();
        let received = rx.try_recv().expect("should receive from s2");
        assert!(matches!(received, CoreEvent::SystemError { .. }));
    }

    #[test]
    fn test_replay_empty_file() {
        let path = PathBuf::from("/tmp/neotrix_test_replay.jsonl");
        let _ = std::fs::remove_file(&path);
        replay(&path, |e| { drop(e); });
        // No panic = pass
    }

    #[test]
    fn test_layer_filter() {
        // L1Body should accept TaskSubmitted
        let e = CoreEvent::TaskSubmitted { task: "t".into(), task_type: "g".into(), priority: 1 };
        assert!(filter_event_for_layer(&e, LayerId::L1Body));
        assert!(!filter_event_for_layer(&e, LayerId::L4Knowledge));
        // L2World should accept ExternalReward
        let e2 = CoreEvent::ExternalReward { reward: 1.0, source: "env".into() };
        assert!(filter_event_for_layer(&e2, LayerId::L2World));
        assert!(!filter_event_for_layer(&e2, LayerId::L1Body));
        // L8Autonomic should accept SystemError
        let e3 = CoreEvent::SystemError { component: "db".into(), error: "timeout".into(), severity: "critical".into() };
        assert!(filter_event_for_layer(&e3, LayerId::L8Autonomic));
        // L9Meta should accept only critical severity
        let e4 = CoreEvent::SystemError { component: "db".into(), error: "warn".into(), severity: "warning".into() };
        assert!(!filter_event_for_layer(&e4, LayerId::L9Meta));
    }

    #[test]
    fn test_subscribe_all_layers_sync() {
        let bus = EventBus::new(1024);
        subscribe_all_layers_sync(&bus);
        // Emit events to all layers
        bus.emit(CoreEvent::TaskSubmitted { task: "t1".into(), task_type: "g".into(), priority: 1 });
        bus.emit(CoreEvent::GoalCompleted { goal_id: "g1".into(), goal: "test".into(), iterations: 5, score: 0.8 });
        bus.emit(CoreEvent::GlobalHalt { reason: "test".into(), source: "test".into() });
        // Give threads time to process
        std::thread::sleep(std::time::Duration::from_millis(50));
        // Drop bus — triggers Drop → shutdown() → joins all threads cleanly
        drop(bus);
    }
}
