use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Debouncer — the Cumora wake debounce pattern (2500ms coalesce).
///
/// Multiple rapid events on the same key get merged into a single
/// execution. This prevents thundering herd when N messages arrive
/// in quick succession.
pub struct Debouncer<K: Clone + Eq + std::hash::Hash + Send + Sync> {
    /// Coalesce window — events within this duration are merged.
    coalesce: Duration,
    /// Pending timers per key.
    pending: RwLock<HashMap<K, Instant>>,
}

impl<K: Clone + Eq + std::hash::Hash + Send + Sync> Debouncer<K> {
    /// Create a new debouncer with the given coalesce window.
    pub fn new(coalesce: Duration) -> Self {
        Self {
            coalesce,
            pending: RwLock::new(HashMap::new()),
        }
    }

    /// Check if a key is ready to fire (not within the coalesce window).
    pub async fn should_fire(&self, key: &K) -> bool {
        let pending = self.pending.read().await;
        match pending.get(key) {
            Some(last) => last.elapsed() >= self.coalesce,
            None => true,
        }
    }

    /// Mark a key as fired (start the coalesce window).
    pub async fn mark_fired(&self, key: K) {
        let mut pending = self.pending.write().await;
        pending.insert(key, Instant::now());
    }

    /// Check and mark atomically — returns true if this call should execute.
    pub async fn check_and_mark(&self, key: &K) -> bool {
        if self.should_fire(key).await {
            self.mark_fired(key.clone()).await;
            true
        } else {
            false
        }
    }

    /// Clear a key's debounce state.
    pub async fn clear(&self, key: &K) {
        let mut pending = self.pending.write().await;
        pending.remove(key);
    }

    /// Clear all pending state.
    pub async fn clear_all(&self) {
        let mut pending = self.pending.write().await;
        pending.clear();
    }
}

/// Coalescer — merges multiple events into a single batch.
///
/// The Cumora pattern: when N messages arrive within a window,
/// they get merged into ONE agent turn instead of N.
pub struct Coalescer<T: Clone> {
    /// Coalesce window.
    window: Duration,
    /// Buffered events.
    buffer: RwLock<Vec<(Instant, T)>>,
}

impl<T: Clone + Send + Sync> Coalescer<T> {
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            buffer: RwLock::new(Vec::new()),
        }
    }

    /// Add an event to the buffer.
    pub async fn push(&self, event: T) {
        let mut buffer = self.buffer.write().await;
        buffer.push((Instant::now(), event));
    }

    /// Drain events that are older than the coalesce window.
    /// Returns the batch of events to process.
    pub async fn drain(&self) -> Vec<T> {
        let mut buffer = self.buffer.write().await;
        let cutoff = Instant::now() - self.window;
        let mut kept = Vec::new();
        let mut drained = Vec::new();

        for (ts, event) in buffer.drain(..) {
            if ts < cutoff {
                drained.push(event);
            } else {
                kept.push((ts, event));
            }
        }

        *buffer = kept;
        drained
    }

    /// Check if there are events ready to drain.
    pub async fn has_ready(&self) -> bool {
        let buffer = self.buffer.read().await;
        let cutoff = Instant::now() - self.window;
        buffer.iter().any(|(ts, _)| *ts < cutoff)
    }

    /// Current buffer size.
    pub async fn len(&self) -> usize {
        self.buffer.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.buffer.read().await.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn debounce_check_and_mark() {
        let debouncer = Debouncer::new(Duration::from_millis(100));
        let key = "test";

        // First call should fire
        assert!(debouncer.check_and_mark(&key).await);

        // Immediate second call should NOT fire (within window)
        assert!(!debouncer.check_and_mark(&key).await);

        // After window, should fire again
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(debouncer.check_and_mark(&key).await);
    }

    #[tokio::test]
    async fn coalescer_drains_old_events() {
        let coalescer = Coalescer::new(Duration::from_millis(50));

        coalescer.push("event1").await;
        coalescer.push("event2").await;

        // Not ready yet
        assert!(!coalescer.has_ready().await);

        // Wait for window
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Now ready
        assert!(coalescer.has_ready().await);

        let batch = coalescer.drain().await;
        assert_eq!(batch.len(), 2);
        assert!(coalescer.is_empty().await);
    }
}
