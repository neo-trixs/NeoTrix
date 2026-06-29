use std::collections::{HashMap, VecDeque};

/// Source of work discovery signals
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DiscoverySource {
    /// Knowledge gap detected by the metacognitive system
    KnowledgeGap,
    /// Curiosity drive signal (negentropy deficit)
    Curiosity,
    /// External event (scheduler tick, webhook, CI result)
    ExternalEvent,
    /// Internal reflection (self-audit finding)
    InternalReflection,
    /// User-initiated request
    UserRequest,
    /// Periodic knowledge enrichment cycle
    KnowledgeEnrichment,
}

/// Priority level for discovered work items
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum WorkPriority {
    Critical,
    High,
    Medium,
    Low,
    Background,
}

/// A discovered work item — something the consciousness should consider acting on
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkItem {
    pub id: u64,
    pub source: DiscoverySource,
    pub priority: WorkPriority,
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub urgency: f64,
    pub created_at: u64,
    pub handler_hint: Option<String>,
    pub context_tags: Vec<String>,
    pub triage_decision: Option<TriageDecision>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TriageDecision {
    Accept,
    Defer,
    Escalate,
    Discard,
}

impl WorkItem {
    pub fn composite_score(&self) -> f64 {
        let priority_weight = match self.priority {
            WorkPriority::Critical => 1.0,
            WorkPriority::High => 0.8,
            WorkPriority::Medium => 0.5,
            WorkPriority::Low => 0.3,
            WorkPriority::Background => 0.1,
        };
        priority_weight * self.confidence * (0.5 + 0.5 * self.urgency)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkDiscoveryStats {
    pub total_discovered: u64,
    pub accepted: u64,
    pub deferred: u64,
    pub escalated: u64,
    pub discarded: u64,
    pub pending_items: usize,
    pub queue_depth: usize,
    pub discovery_rate: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkDiscoveryConfig {
    pub max_queue_size: usize,
    pub max_pending_items: usize,
    pub min_confidence_threshold: f64,
    pub enable_knowledge_gap_discovery: bool,
    pub enable_curiosity_discovery: bool,
    pub enable_external_event_discovery: bool,
    pub stale_item_ttl_secs: u64,
    pub discovery_interval_cycles: u64,
}

impl Default for WorkDiscoveryConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 200,
            max_pending_items: 50,
            min_confidence_threshold: 0.3,
            enable_knowledge_gap_discovery: true,
            enable_curiosity_discovery: true,
            enable_external_event_discovery: true,
            stale_item_ttl_secs: 3600,
            discovery_interval_cycles: 3,
        }
    }
}

/// The WorkDiscoveryLoop implements the "Triage" primitive of loop engineering.
///
/// It discovers work from multiple sources (knowledge gaps, curiosity, external events),
/// prioritizes them, and feeds them into the LoopEngine's Observe phase.
///
/// This is NeoTrix's equivalent of the "morning triage loop" — the automated
/// discovery pipeline that finds what needs attention without waiting for a prompt.
#[derive(Debug)]
pub struct WorkDiscoveryLoop {
    config: WorkDiscoveryConfig,
    /// Pending items waiting for triage
    pending_queue: VecDeque<WorkItem>,
    /// Items that have been accepted and are being worked on
    active_items: Vec<WorkItem>,
    /// Items deferred for later
    deferred_items: Vec<WorkItem>,
    /// Registry of known signal sources
    signal_registry: HashMap<String, DiscoverySource>,
    /// Next ID counter
    next_id: u64,
    /// Stats
    total_discovered: u64,
    accepted: u64,
    deferred: u64,
    escalated: u64,
    discarded: u64,
    last_discovery_cycle: u64,
    discovery_events: VecDeque<u64>,
}

impl WorkDiscoveryLoop {
    pub fn new(config: WorkDiscoveryConfig) -> Self {
        Self {
            config,
            pending_queue: VecDeque::new(),
            active_items: Vec::new(),
            deferred_items: Vec::new(),
            signal_registry: HashMap::new(),
            next_id: 1,
            total_discovered: 0,
            accepted: 0,
            deferred: 0,
            escalated: 0,
            discarded: 0,
            last_discovery_cycle: 0,
            discovery_events: VecDeque::new(),
        }
    }

    pub fn register_signal_source(&mut self, name: &str, source: DiscoverySource) {
        self.signal_registry.insert(name.to_string(), source);
    }

    pub fn discover(
        &mut self,
        source: DiscoverySource,
        priority: WorkPriority,
        title: &str,
        description: &str,
        confidence: f64,
        urgency: f64,
    ) -> Option<u64> {
        if self.pending_queue.len() >= self.config.max_queue_size {
            return None;
        }
        if confidence < self.config.min_confidence_threshold {
            self.discarded += 1;
            return None;
        }

        let id = self.next_id;
        self.next_id += 1;

        let item = WorkItem {
            id,
            source: source.clone(),
            priority,
            title: title.to_string(),
            description: description.to_string(),
            confidence,
            urgency,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            handler_hint: None,
            context_tags: vec![format!("{:?}", source)],
            triage_decision: None,
        };

        self.pending_queue.push_back(item);
        self.total_discovered += 1;
        self.discovery_events.push_back(id);
        if self.discovery_events.len() > 100 {
            self.discovery_events.pop_front();
        }

        Some(id)
    }

    pub fn run_discovery_tick(&mut self, current_cycle: u64) -> Vec<WorkItem> {
        if current_cycle - self.last_discovery_cycle < self.config.discovery_interval_cycles {
            return Vec::new();
        }
        self.last_discovery_cycle = current_cycle;

        // Knowledge gap discovery would integrate with metacognitive gap detector
        if self.config.enable_knowledge_gap_discovery {
            // Signal source: metacognitive loop
        }

        // Curiosity-driven discovery
        if self.config.enable_curiosity_discovery {
            // Signal source: curiosity drive
        }

        Vec::new()
    }

    /// Triage the next pending item — decide Accept/Defer/Escalate/Discard
    pub fn triage_next(&mut self) -> Option<(u64, TriageDecision)> {
        let item = self.pending_queue.pop_front()?;
        let score = item.composite_score();

        let decision = if score > 0.7 {
            TriageDecision::Accept
        } else if score > 0.4 {
            TriageDecision::Defer
        } else if score > 0.2 {
            TriageDecision::Escalate
        } else {
            TriageDecision::Discard
        };

        match decision {
            TriageDecision::Accept => {
                self.accepted += 1;
            }
            TriageDecision::Defer => {
                self.deferred += 1;
                self.deferred_items.push(item.clone());
            }
            TriageDecision::Escalate => {
                self.escalated += 1;
            }
            TriageDecision::Discard => {
                self.discarded += 1;
            }
        }

        Some((item.id, decision))
    }

    /// Triage all pending items, returning accepted items
    pub fn triage_all(&mut self) -> Vec<WorkItem> {
        let mut accepted = Vec::new();
        while let Some(mut item) = self.pending_queue.pop_front() {
            let score = item.composite_score();
            let decision = if score > 0.7 {
                TriageDecision::Accept
            } else if score > 0.4 {
                TriageDecision::Defer
            } else if score > 0.2 {
                TriageDecision::Escalate
            } else {
                TriageDecision::Discard
            };

            item.triage_decision = Some(decision.clone());
            match decision {
                TriageDecision::Accept => {
                    self.accepted += 1;
                    accepted.push(item);
                }
                TriageDecision::Defer => {
                    self.deferred += 1;
                    self.deferred_items.push(item);
                }
                TriageDecision::Escalate => {
                    self.escalated += 1;
                }
                TriageDecision::Discard => {
                    self.discarded += 1;
                }
            }
        }
        accepted
    }

    /// Get the highest-priority accepted item
    pub fn next_work_item(&mut self) -> Option<WorkItem> {
        if self.active_items.is_empty() {
            return None;
        }
        self.active_items.sort_by(|a, b| {
            b.composite_score()
                .partial_cmp(&a.composite_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Some(self.active_items.remove(0))
    }

    pub fn mark_completed(&mut self, id: u64) -> bool {
        if let Some(pos) = self.active_items.iter().position(|w| w.id == id) {
            self.active_items.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn stats(&self) -> WorkDiscoveryStats {
        let total = self.total_discovered.max(1);
        WorkDiscoveryStats {
            total_discovered: self.total_discovered,
            accepted: self.accepted,
            deferred: self.deferred,
            escalated: self.escalated,
            discarded: self.discarded,
            pending_items: self.pending_queue.len(),
            queue_depth: self.pending_queue.len() + self.deferred_items.len(),
            discovery_rate: self.accepted as f64 / total as f64,
        }
    }

    pub fn pending_count(&self) -> usize {
        self.pending_queue.len()
    }

    pub fn active_count(&self) -> usize {
        self.active_items.len()
    }

    pub fn config(&self) -> &WorkDiscoveryConfig {
        &self.config
    }
}

impl Default for WorkDiscoveryLoop {
    fn default() -> Self {
        Self::new(WorkDiscoveryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_discovery_new() {
        let wd = WorkDiscoveryLoop::default();
        assert_eq!(wd.pending_count(), 0);
        assert_eq!(wd.active_count(), 0);
        assert_eq!(wd.total_discovered, 0);
    }

    #[test]
    fn test_work_discovery_discover() {
        let mut wd = WorkDiscoveryLoop::default();
        let id = wd.discover(
            DiscoverySource::KnowledgeGap,
            WorkPriority::High,
            "test gap",
            "a knowledge gap was detected",
            0.8,
            0.6,
        );
        assert!(id.is_some());
        assert_eq!(wd.pending_count(), 1);
        assert_eq!(wd.total_discovered, 1);
    }

    #[test]
    fn test_work_discovery_low_confidence_filtered() {
        let mut wd = WorkDiscoveryLoop::default();
        let id = wd.discover(
            DiscoverySource::Curiosity,
            WorkPriority::Low,
            "noise",
            "below threshold",
            0.1,
            0.0,
        );
        assert!(id.is_none());
        assert_eq!(wd.pending_count(), 0);
    }

    #[test]
    fn test_work_discovery_triage_accept() {
        let mut wd = WorkDiscoveryLoop::default();
        wd.discover(
            DiscoverySource::ExternalEvent,
            WorkPriority::Critical,
            "critical item",
            "urgent fix needed",
            0.9,
            1.0,
        );
        let decision = wd.triage_next();
        assert!(decision.is_some());
        let (_, d) = decision.unwrap();
        assert!(matches!(d, TriageDecision::Accept));
        assert_eq!(wd.accepted, 1);
    }

    #[test]
    fn test_work_discovery_triage_discard() {
        let mut wd = WorkDiscoveryLoop::default();
        wd.discover(
            DiscoverySource::InternalReflection,
            WorkPriority::Background,
            "low priority",
            "minor observation",
            0.3,
            0.1,
        );
        let decision = wd.triage_next();
        assert!(decision.is_some());
        let (_, d) = decision.unwrap();
        assert!(matches!(d, TriageDecision::Discard));
        assert_eq!(wd.discarded, 1);
    }

    #[test]
    fn test_work_discovery_triage_all() {
        let mut wd = WorkDiscoveryLoop::default();
        wd.discover(
            DiscoverySource::KnowledgeGap,
            WorkPriority::High,
            "gap1",
            "important gap",
            0.9,
            0.8,
        );
        wd.discover(
            DiscoverySource::Curiosity,
            WorkPriority::Low,
            "curiosity",
            "mild curiosity",
            0.5,
            0.3,
        );
        wd.discover(
            DiscoverySource::UserRequest,
            WorkPriority::Background,
            "noise",
            "low signal",
            0.2,
            0.1,
        );

        let accepted = wd.triage_all();
        assert_eq!(accepted.len(), 1);
        assert_eq!(wd.accepted, 1);
        assert_eq!(wd.discarded, 1);
        assert_eq!(wd.deferred, 1);
    }

    #[test]
    fn test_work_discovery_composite_score() {
        let item = WorkItem {
            id: 1,
            source: DiscoverySource::KnowledgeGap,
            priority: WorkPriority::High,
            title: "test".to_string(),
            description: "test".to_string(),
            confidence: 0.8,
            urgency: 0.7,
            created_at: 0,
            handler_hint: None,
            context_tags: vec![],
            triage_decision: None,
        };
        let score = item.composite_score();
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_work_discovery_stats() {
        let mut wd = WorkDiscoveryLoop::default();
        wd.discover(
            DiscoverySource::KnowledgeGap,
            WorkPriority::High,
            "a",
            "desc",
            0.8,
            0.7,
        );
        wd.discover(
            DiscoverySource::Curiosity,
            WorkPriority::Medium,
            "b",
            "desc",
            0.6,
            0.5,
        );
        wd.triage_all();
        let stats = wd.stats();
        assert_eq!(stats.total_discovered, 2);
        assert!(stats.discovery_rate > 0.0);
    }

    #[test]
    fn test_work_discovery_tick_returns_empty_when_not_due() {
        let mut wd = WorkDiscoveryLoop::default();
        let items = wd.run_discovery_tick(1);
        assert!(items.is_empty());
    }

    #[test]
    fn test_work_discovery_register_signal_source() {
        let mut wd = WorkDiscoveryLoop::default();
        wd.register_signal_source("ci_failure", DiscoverySource::ExternalEvent);
        assert_eq!(wd.signal_registry.len(), 1);
    }

    #[test]
    fn test_work_discovery_mark_completed() {
        let mut wd = WorkDiscoveryLoop::default();
        wd.accepted += 1;
        wd.active_items.push(WorkItem {
            id: 42,
            source: DiscoverySource::UserRequest,
            priority: WorkPriority::High,
            title: "complete me".to_string(),
            description: "test".to_string(),
            confidence: 0.9,
            urgency: 0.8,
            created_at: 0,
            handler_hint: None,
            context_tags: vec![],
            triage_decision: Some(TriageDecision::Accept),
        });
        assert!(wd.mark_completed(42));
        assert_eq!(wd.active_count(), 0);
    }
}
