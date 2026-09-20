//! GlobalWorkspace — broadcast-based attention routing.
//!
//! Specialists subscribe to the workspace and receive broadcasts filtered by relevance.
//! Implements the core GWT mechanism: salient content is broadcast, non-relevant content
//! is suppressed.

use std::sync::atomic::{AtomicU64, Ordering};

/// Priority of a broadcast — higher priority is delivered first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BroadcastPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// A piece of content to be broadcast through the Global Workspace.
#[derive(Debug, Clone)]
pub struct BroadcastContent {
    /// Unique identifier for this broadcast.
    pub id: u64,
    /// The domain tag (e.g. "nt_core", "nt_mind", "nt_world") that produced this.
    pub source_domain: String,
    /// Topic/keyword tags for relevance matching.
    pub tags: Vec<String>,
    /// Priority level.
    pub priority: BroadcastPriority,
    /// The actual payload as opaque bytes (avoids generic type parameters).
    pub payload: Vec<u8>,
    /// Timestamp when broadcast was created (epoch millis).
    pub timestamp: u64,
}

/// A subscriber registered to receive broadcasts.
#[derive(Debug, Clone)]
pub struct Subscriber {
    /// Unique subscriber identifier.
    pub id: u64,
    /// Domain this subscriber belongs to.
    pub domain: String,
    /// Tags this subscriber is interested in.
    pub interest_tags: Vec<String>,
    /// Minimum priority threshold — broadcasts below this are not delivered.
    pub min_priority: BroadcastPriority,
    /// Whether this subscriber is active.
    pub active: bool,
}

/// A delivery record: what was sent to which subscriber.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryRecord {
    pub broadcast_id: u64,
    pub subscriber_id: u64,
    pub delivered: bool,
    pub reason: DeliveryReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryReason {
    TagMatch,
    PriorityThreshold,
    DomainMatch,
    Suppressed,
}

/// GlobalWorkspace — the broadcast hub for attention routing.
pub struct GlobalWorkspace {
    subscribers: Vec<Subscriber>,
    delivery_log: Vec<DeliveryRecord>,
    next_broadcast_id: AtomicU64,
    next_subscriber_id: AtomicU64,
}

impl GlobalWorkspace {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
            delivery_log: Vec::new(),
            next_broadcast_id: AtomicU64::new(1),
            next_subscriber_id: AtomicU64::new(1),
        }
    }

    /// Register a new subscriber. Returns the subscriber ID.
    pub fn subscribe(
        &mut self,
        domain: String,
        interest_tags: Vec<String>,
        min_priority: BroadcastPriority,
    ) -> u64 {
        let id = self.next_subscriber_id.fetch_add(1, Ordering::Relaxed);
        self.subscribers.push(Subscriber {
            id,
            domain,
            interest_tags,
            min_priority,
            active: true,
        });
        id
    }

    /// Remove a subscriber by ID.
    pub fn unsubscribe(&mut self, subscriber_id: u64) -> bool {
        if let Some(sub) = self.subscribers.iter_mut().find(|s| s.id == subscriber_id) {
            sub.active = false;
            true
        } else {
            false
        }
    }

    /// Broadcast content to all relevant subscribers.
    ///
    /// A subscriber receives the broadcast if:
    /// 1. It is active
    /// 2. Its min_priority <= content.priority
    /// 3. It has at least one matching interest tag OR the broadcast has no tags (broadcast to all)
    pub fn broadcast(&mut self, content: BroadcastContent) -> Vec<DeliveryRecord> {
        let broadcast_id = content.id;
        let mut records = Vec::new();

        for sub in &self.subscribers {
            if !sub.active {
                records.push(DeliveryRecord {
                    broadcast_id,
                    subscriber_id: sub.id,
                    delivered: false,
                    reason: DeliveryReason::Suppressed,
                });
                continue;
            }

            if content.priority < sub.min_priority {
                records.push(DeliveryRecord {
                    broadcast_id,
                    subscriber_id: sub.id,
                    delivered: false,
                    reason: DeliveryReason::PriorityThreshold,
                });
                continue;
            }

            let tag_match = content.tags.is_empty()
                || sub.interest_tags.is_empty()
                || content
                    .tags
                    .iter()
                    .any(|t| sub.interest_tags.contains(t));

            let domain_match = sub.domain == content.source_domain;

            let delivered = tag_match || domain_match;
            let reason = if domain_match {
                DeliveryReason::DomainMatch
            } else if tag_match {
                DeliveryReason::TagMatch
            } else {
                DeliveryReason::Suppressed
            };

            records.push(DeliveryRecord {
                broadcast_id,
                subscriber_id: sub.id,
                delivered,
                reason,
            });
        }

        self.delivery_log.extend(records.clone());
        records
    }

    /// Get delivery statistics for a broadcast.
    pub fn delivery_stats(&self, broadcast_id: u64) -> (usize, usize) {
        let records: Vec<_> = self
            .delivery_log
            .iter()
            .filter(|r| r.broadcast_id == broadcast_id)
            .collect();
        let delivered = records.iter().filter(|r| r.delivered).count();
        (delivered, records.len())
    }

    /// Get total active subscribers.
    pub fn active_subscriber_count(&self) -> usize {
        self.subscribers.iter().filter(|s| s.active).count()
    }

    /// Create a BroadcastContent with auto-incremented ID.
    pub fn next_broadcast(
        &self,
        source_domain: String,
        tags: Vec<String>,
        priority: BroadcastPriority,
        payload: Vec<u8>,
        timestamp: u64,
    ) -> BroadcastContent {
        BroadcastContent {
            id: self.next_broadcast_id.load(Ordering::Relaxed),
            source_domain,
            tags,
            priority,
            payload,
            timestamp,
        }
    }
}

impl Default for GlobalWorkspace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscriber_receives_matching_broadcast() {
        let mut ws = GlobalWorkspace::new();
        ws.subscribe(
            "nt_core".into(),
            vec!["reasoning".into()],
            BroadcastPriority::Low,
        );
        let content = ws.next_broadcast(
            "nt_mind".into(),
            vec!["reasoning".into()],
            BroadcastPriority::Normal,
            vec![],
            0,
        );
        let records = ws.broadcast(content);
        assert!(records.iter().any(|r| r.delivered));
    }

    #[test]
    fn subscriber_ignores_non_matching_broadcast() {
        let mut ws = GlobalWorkspace::new();
        ws.subscribe(
            "nt_core".into(),
            vec!["reasoning".into()],
            BroadcastPriority::Low,
        );
        let content = ws.next_broadcast(
            "nt_mind".into(),
            vec!["perception".into()],
            BroadcastPriority::Normal,
            vec![],
            0,
        );
        let records = ws.broadcast(content);
        assert!(!records.iter().any(|r| r.delivered));
    }

    #[test]
    fn priority_threshold_filters() {
        let mut ws = GlobalWorkspace::new();
        ws.subscribe(
            "nt_core".into(),
            vec![],
            BroadcastPriority::High,
        );
        let content = ws.next_broadcast(
            "nt_mind".into(),
            vec![],
            BroadcastPriority::Low,
            vec![],
            0,
        );
        let records = ws.broadcast(content);
        assert!(records.iter().all(|r| !r.delivered));
    }

    #[test]
    fn domain_match_delivers_regardless_of_tags() {
        let mut ws = GlobalWorkspace::new();
        ws.subscribe(
            "nt_core".into(),
            vec!["unrelated_tag".into()],
            BroadcastPriority::Low,
        );
        let content = ws.next_broadcast(
            "nt_core".into(),
            vec!["completely_different".into()],
            BroadcastPriority::Normal,
            vec![],
            0,
        );
        let records = ws.broadcast(content);
        assert!(records.iter().any(|r| r.delivered && r.reason == DeliveryReason::DomainMatch));
    }

    #[test]
    fn unsubscribe_prevents_delivery() {
        let mut ws = GlobalWorkspace::new();
        let sub_id = ws.subscribe(
            "nt_core".into(),
            vec!["tag".into()],
            BroadcastPriority::Low,
        );
        ws.unsubscribe(sub_id);
        let content = ws.next_broadcast(
            "nt_mind".into(),
            vec!["tag".into()],
            BroadcastPriority::Normal,
            vec![],
            0,
        );
        let records = ws.broadcast(content);
        assert!(records.iter().all(|r| !r.delivered));
    }

    #[test]
    fn delivery_stats_count_correctly() {
        let mut ws = GlobalWorkspace::new();
        ws.subscribe("a".into(), vec![], BroadcastPriority::Low);
        ws.subscribe("b".into(), vec![], BroadcastPriority::Low);
        let content = ws.next_broadcast("a".into(), vec![], BroadcastPriority::Normal, vec![], 0);
        let bc_id = content.id;
        ws.broadcast(content);
        let (delivered, total) = ws.delivery_stats(bc_id);
        assert_eq!(delivered, 2);
        assert_eq!(total, 2);
    }
}
