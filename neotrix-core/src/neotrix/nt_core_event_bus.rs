use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use crate::core::nt_core_event::CoreEvent;

/// 事件溯源信封 (D4 — maka 'Log is the Runtime' / buzz 事件日志 + 身份 + receipts)
/// 落盘时包裹在 CoreEvent 之外, 提供可重建事件链的溯源字段 (全局 seq + 来源身份 + 时间戳),
/// 且不改变 CoreEvent enum schema (R-P84: 避免波及全部变体的 schema 变更)。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventEnvelope {
    /// 全局单调递增序列号 — 事件链回放顺序
    pub seq: u64,
    /// 事件来源身份 (agent/进程/模块标识, 如 "nt_act::executor")
    pub source: String,
    /// 产生时刻 (unix 毫秒)
    pub timestamp_ms: i64,
    /// 载荷事件
    pub event: CoreEvent,
}

/// Type-safe event bus using tokio broadcast channel.
/// No `dyn Any` downcasting — every event is `CoreEvent` enum.
pub struct EventBus {
    sender: broadcast::Sender<CoreEvent>,
    log_file: Option<std::sync::Mutex<std::fs::File>>,
    shutdown_flag: Arc<AtomicBool>,
    handles: std::sync::Mutex<Vec<std::thread::JoinHandle<()>>>,
    seq: Arc<AtomicU64>,
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self { sender: self.sender.clone(), log_file: None, shutdown_flag: Arc::new(AtomicBool::new(false)), handles: std::sync::Mutex::new(Vec::new()), seq: self.seq.clone() }
    }
}

impl Default for EventBus {
    fn default() -> Self { Self::new(1024) }
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender, log_file: None, shutdown_flag: Arc::new(AtomicBool::new(false)), handles: std::sync::Mutex::new(Vec::new()), seq: Arc::new(AtomicU64::new(0)) }
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
        Self { sender, log_file: file, shutdown_flag: Arc::new(AtomicBool::new(false)), handles: std::sync::Mutex::new(Vec::new()), seq: Arc::new(AtomicU64::new(0)) }
    }

    /// 当前已发出的溯源序号 (供外部记录事件链高水位)
    pub fn seq_watermark(&self) -> u64 {
        self.seq.load(Ordering::SeqCst)
    }

    /// Emit an event to all subscribers.
    /// If logging is enabled, also persists as JSON line (secrets/PII redacted before write).
    pub fn emit(&self, event: CoreEvent) {
        self.emit_from("unknown", event);
    }

    /// Emit with 事件溯源身份 (D4) — 记录来源 agent/模块 + 全局 seq, 落盘为 envelope。
    /// 与导出的 emit() 行为一致, 仅多带身份与递增序号。
    pub fn emit_from(&self, source: &str, event: CoreEvent) {
        if let Some(ref log_file) = self.log_file {
            let seq = self.seq.fetch_add(1, Ordering::SeqCst);
            let env = EventEnvelope {
                seq,
                source: source.to_string(),
                timestamp_ms: chrono::Utc::now().timestamp_millis(),
                event: event.clone(),
            };
            if let Ok(mut file) = log_file.lock() {
                if let Ok(line) = serde_json::to_string(&env) {
                    use std::io::Write;
                    // 隐私脱敏挂载点: 落盘前净化 secrets/PII (R-P42 强化 nt_shield 节点)。
                    // 用 JSON 感知脱敏 — 只替换字符串值, 不破坏数值/结构 (R-P86 类教训)。
                    let redacted = crate::neotrix::nt_shield::redaction::redact_json_line(&line);
                    let _ = writeln!(file, "{}", redacted);
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
/// 兼容旧格式 (裸 CoreEvent JSON) 与新格式 (EventEnvelope)。
pub fn replay<F>(path: &PathBuf, callback: F)
where
    F: Fn(CoreEvent) + Send + Sync + 'static,
{
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    for line in content.lines() {
        // 优先新格式 enveloped; 失败回退旧格式裸事件
        if let Ok(env) = serde_json::from_str::<EventEnvelope>(line) {
            callback(env.event);
        } else if let Ok(event) = serde_json::from_str::<CoreEvent>(line) {
            callback(event);
        }
    }
}

/// 事件溯源回放 (D4) — 从 JSONL 重建带身份与 seq 的完整事件链。
/// 兼容新格式 envelope; 旧格式裸事件以 seq=0 / source="legacy" 包裹。
pub fn replay_enveloped(path: &PathBuf) -> Vec<EventEnvelope> {
    let mut chain = Vec::new();
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return chain,
    };
    let mut fallback_seq: u64 = 0;
    for line in content.lines() {
        if let Ok(env) = serde_json::from_str::<EventEnvelope>(line) {
            chain.push(env);
        } else if let Ok(event) = serde_json::from_str::<CoreEvent>(line) {
            chain.push(EventEnvelope {
                seq: fallback_seq,
                source: "legacy".into(),
                timestamp_ms: 0,
                event,
            });
            fallback_seq += 1;
        }
    }
    chain
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
    fn test_enveloped_persistence_and_replay_chain() {
        // D4 (maka/buzz): 事件溯源 — envelope 携带 seq + 来源身份, replay 可重建事件链
        let path = PathBuf::from(format!("/tmp/neotrix_test_enveloped_{}.jsonl", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let bus = EventBus::with_persistence(path.clone());
        bus.emit_from("nt_act::executor", CoreEvent::TaskSubmitted { task: "t".into(), task_type: "g".into(), priority: 1 });
        bus.emit_from("nt_core::gateway", CoreEvent::GoalCompleted { goal_id: "g1".into(), goal: "test".into(), iterations: 5, score: 0.8 });
        drop(bus);

        let chain = replay_enveloped(&path);
        assert_eq!(chain.len(), 2, "both events must be replayed with envelope");
        assert_eq!(chain[0].seq, 0, "first event seq = 0");
        assert_eq!(chain[1].seq, 1, "second event seq = 1 (monotonic chain)");
        assert_eq!(chain[0].source, "nt_act::executor", "identity must survive serialization");
        assert!(matches!(chain[1].event, CoreEvent::GoalCompleted { .. }), "payload must round-trip");
        let _ = std::fs::remove_file(&path);
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
