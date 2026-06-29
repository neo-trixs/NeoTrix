use std::collections::HashMap;
use tokio::sync::mpsc;

/// 意识体事件 — 所有子系统可订阅的事件类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConsciousnessEvent {
    PhaseTransition(String),
    TickScheduled(String),
    ThoughtComplete,
    HandlerError(String),
    KnowledgeGap(String),
    GoalDrift(String),
    ExternalInput(String),
    Save,
    Consolidate,
    MetaTick,
    CuriousityTick,
    ExplorationTick,
    HealthCheck,
    SchedulerTick,
    AuditTick,
    NetworkTick,
    VisionTick,
    Custom(String),
}

/// 事件驱动的调度器 — 替代轮询
pub struct EventDrivenScheduler {
    tx: mpsc::Sender<ConsciousnessEvent>,
    tick_intervals: HashMap<String, u64>,
    deadlines: Vec<DeadlineEntry>,
}

struct DeadlineEntry {
    event: ConsciousnessEvent,
    trigger_at: tokio::time::Instant,
}

impl EventDrivenScheduler {
    pub fn new(capacity: usize) -> (Self, mpsc::Receiver<ConsciousnessEvent>) {
        let (tx, rx) = mpsc::channel(capacity);
        (
            Self {
                tx,
                tick_intervals: HashMap::new(),
                deadlines: Vec::new(),
            },
            rx,
        )
    }

    pub fn emit(&self, event: ConsciousnessEvent) -> usize {
        match self.tx.try_send(event) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    }

    pub fn register_interval(&mut self, event_name: &str, interval_ms: u64) {
        self.tick_intervals
            .insert(event_name.to_string(), interval_ms);
    }

    pub fn schedule_at(&mut self, event: ConsciousnessEvent, when: tokio::time::Instant) {
        self.deadlines.push(DeadlineEntry {
            event,
            trigger_at: when,
        });
    }

    pub fn process_deadlines(&mut self) -> usize {
        let now = tokio::time::Instant::now();
        let mut emitted = 0;
        let mut to_emit = Vec::new();
        let mut to_keep = Vec::with_capacity(self.deadlines.len());
        for entry in self.deadlines.drain(..) {
            if entry.trigger_at <= now {
                to_emit.push(entry.event);
            } else {
                to_keep.push(entry);
            }
        }
        self.deadlines = to_keep;
        for event in to_emit {
            emitted += self.emit(event);
        }
        emitted
    }

    pub fn sender(&self) -> mpsc::Sender<ConsciousnessEvent> {
        self.tx.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_emit_and_receive() {
        let (scheduler, mut rx) = EventDrivenScheduler::new(16);
        let emitted = scheduler.emit(ConsciousnessEvent::Save);
        assert_eq!(emitted, 1);
        let received = rx.try_recv();
        assert!(received.is_ok());
        assert_eq!(received.unwrap(), ConsciousnessEvent::Save);
    }

    #[tokio::test]
    async fn test_sender_clone_can_emit() {
        let (scheduler, mut rx) = EventDrivenScheduler::new(16);
        let sender = scheduler.sender();
        assert!(sender.try_send(ConsciousnessEvent::MetaTick).is_ok());
        let received = rx.try_recv();
        assert_eq!(received.ok(), Some(ConsciousnessEvent::MetaTick));
    }

    #[tokio::test]
    async fn test_schedule_at_and_process_deadlines() {
        let (mut scheduler, _rx) = EventDrivenScheduler::new(16);
        let soon = tokio::time::Instant::now() + Duration::from_millis(10);
        scheduler.schedule_at(ConsciousnessEvent::Consolidate, soon);
        tokio::time::sleep(Duration::from_millis(20)).await;
        let emitted = scheduler.process_deadlines();
        assert_eq!(emitted, 1);
    }

    #[tokio::test]
    async fn test_deadline_not_triggered_before_time() {
        let (mut scheduler, mut rx) = EventDrivenScheduler::new(16);
        let far = tokio::time::Instant::now() + Duration::from_secs(60);
        scheduler.schedule_at(ConsciousnessEvent::AuditTick, far);
        let emitted = scheduler.process_deadlines();
        assert_eq!(emitted, 0);
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_register_interval_and_sender_clone() {
        let (mut scheduler, _rx) = EventDrivenScheduler::new(16);
        scheduler.register_interval("save", 30000);
        let sender = scheduler.sender();
        assert!(sender.try_send(ConsciousnessEvent::Save).is_ok());
    }
}
