#![forbid(unsafe_code)]

//! Muted / NoOp / Filtered wrappers for `EventBus`.
//!
//! Inspired by Qingjian's privacy-mode pattern:
//! - **MutedEventBus** — records all events without broadcasting. Audit/tracing mode.
//! - **NoOpEventBus** — silently drops everything. For tests and benchmarks.
//! - **FilteredEventBus** — only forwards events matching a predicate.

use std::sync::Mutex;

use tokio::sync::broadcast;

use super::nt_core_event_bus::EventBus;
use crate::core::nt_core_event::CoreEvent;

// ── MutedEventBus ──────────────────────────────────────────────────────────

/// Wraps a real `EventBus`, records every emitted event in-memory but never
/// broadcasts them to subscribers. Use this for audit/tracing modes where you
/// want a full event log without side effects on live subscribers.
pub struct MutedEventBus {
    inner: EventBus,
    recorded: Mutex<Vec<CoreEvent>>,
}

impl MutedEventBus {
    pub fn new(inner: EventBus) -> Self {
        Self {
            inner,
            recorded: Mutex::new(Vec::new()),
        }
    }

    /// Emit an event — recorded only, never broadcast.
    pub fn emit(&self, event: CoreEvent) {
        if let Ok(mut log) = self.recorded.lock() {
            log.push(event);
        }
    }

    /// Return all recorded events (consumes the buffer).
    pub fn replay(&self) -> Vec<CoreEvent> {
        self.recorded
            .lock()
            .map(|mut v| std::mem::take(&mut *v))
            .unwrap_or_default()
    }

    /// Return the number of events recorded so far.
    pub fn recorded_count(&self) -> usize {
        self.recorded.lock().map(|v| v.len()).unwrap_or(0)
    }

    /// Unwrap, returning the inner `EventBus`.
    pub fn into_inner(self) -> EventBus {
        self.inner
    }

    /// Borrow the inner `EventBus` (e.g. to subscribe or inspect seq_watermark).
    pub fn inner(&self) -> &EventBus {
        &self.inner
    }
}

// ── NoOpEventBus ───────────────────────────────────────────────────────────

/// Does nothing. All events are silently dropped. `subscribe()` returns a
/// receiver that will never receive any event. Useful for tests and benchmarks
/// where you need the type to satisfy an `impl EventBus`-like trait without
/// paying for channel overhead.
pub struct NoOpEventBus;

impl NoOpEventBus {
    pub fn new() -> Self {
        Self
    }

    /// No-op — event is silently discarded.
    pub fn emit(&self, _event: CoreEvent) {}

    /// Returns a broadcast receiver that will never receive any events
    /// (the sender is immediately dropped).
    pub fn subscribe(&self) -> broadcast::Receiver<CoreEvent> {
        // Create a channel, drop the sender — receiver will always return
        // `RecvError::Closed`.
        let (tx, rx) = broadcast::channel(1);
        drop(tx);
        rx
    }
}

impl Default for NoOpEventBus {
    fn default() -> Self {
        Self
    }
}

// ── FilteredEventBus ───────────────────────────────────────────────────────

/// Wraps a real `EventBus` and only forwards events for which the provided
/// predicate returns `true`. Events that don't match are silently dropped.
pub struct FilteredEventBus {
    inner: EventBus,
    filter: Box<dyn Fn(&CoreEvent) -> bool + Send + Sync>,
}

impl FilteredEventBus {
    pub fn new(inner: EventBus, filter: Box<dyn Fn(&CoreEvent) -> bool + Send + Sync>) -> Self {
        Self { inner, filter }
    }

    /// Emit an event — only forwarded to the inner bus if the filter accepts it.
    pub fn emit(&self, event: CoreEvent) {
        if (self.filter)(&event) {
            self.inner.emit(event);
        }
    }

    /// Subscribe to the inner bus (receives only events that pass the filter).
    pub fn subscribe(&self) -> broadcast::Receiver<CoreEvent> {
        self.inner.subscribe()
    }

    /// Unwrap, returning the inner `EventBus`.
    pub fn into_inner(self) -> EventBus {
        self.inner
    }

    /// Borrow the inner `EventBus`.
    pub fn inner(&self) -> &EventBus {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_event::CoreEvent;

    #[test]
    fn muted_records_and_replays() {
        let bus = EventBus::new(16);
        let muted = MutedEventBus::new(bus);

        muted.emit(CoreEvent::TaskSubmitted {
            task: "t1".into(),
            task_type: "g".into(),
            priority: 1,
        });
        muted.emit(CoreEvent::GoalCompleted {
            goal_id: "g1".into(),
            goal: "test".into(),
            iterations: 5,
            score: 0.8,
        });

        assert_eq!(muted.recorded_count(), 2);
        let events = muted.replay();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], CoreEvent::TaskSubmitted { .. }));
        assert!(matches!(events[1], CoreEvent::GoalCompleted { .. }));
        // Buffer is consumed after replay
        assert_eq!(muted.recorded_count(), 0);
    }

    #[test]
    fn muted_into_inner_returns_bus() {
        let bus = EventBus::new(16);
        let bus_ref = bus.subscribe(); // prove it works
        drop(bus_ref);
        let muted = MutedEventBus::new(bus);
        let _inner = muted.into_inner();
    }

    #[test]
    fn noop_emit_is_silent() {
        let noop = NoOpEventBus::new();
        noop.emit(CoreEvent::GlobalHalt {
            reason: "x".into(),
            source: "test".into(),
        });
        // No panic, no side effect
    }

    #[test]
    fn noop_subscribe_never_receives() {
        let noop = NoOpEventBus::new();
        let mut rx = noop.subscribe();
        noop.emit(CoreEvent::TaskSubmitted {
            task: "t".into(),
            task_type: "g".into(),
            priority: 1,
        });
        // Channel is closed — should get Closed error
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn filtered_passes_matching() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        let filtered = FilteredEventBus::new(
            bus,
            Box::new(|e| matches!(e, CoreEvent::TaskSubmitted { .. })),
        );

        filtered.emit(CoreEvent::TaskSubmitted {
            task: "t1".into(),
            task_type: "g".into(),
            priority: 1,
        });
        filtered.emit(CoreEvent::GlobalHalt {
            reason: "x".into(),
            source: "test".into(),
        });

        let received = rx.try_recv().expect("TaskSubmitted should pass filter");
        assert!(matches!(received, CoreEvent::TaskSubmitted { .. }));
        // GlobalHalt was filtered out
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn filtered_drops_non_matching() {
        let bus = EventBus::new(16);
        let filtered = FilteredEventBus::new(
            bus,
            Box::new(|_| false), // reject everything
        );

        filtered.emit(CoreEvent::TaskSubmitted {
            task: "t".into(),
            task_type: "g".into(),
            priority: 1,
        });
        // Nothing should reach the inner bus subscribers — the bus itself
        // still works, but no events were forwarded.
    }
}
