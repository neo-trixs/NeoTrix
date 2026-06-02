use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use serde_json::json;
use crate::core::nt_core_event::BusEvent;

type HandlerList = Arc<Mutex<Vec<Box<dyn Fn(Box<dyn BusEvent>) + Send + Sync>>>>;

pub struct EventBus {
    handlers: HandlerList,
    log_file: Option<Arc<Mutex<std::fs::File>>>,
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self { handlers: self.handlers.clone(), log_file: self.log_file.clone() }
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self { handlers: Arc::new(Mutex::new(Vec::new())), log_file: None }
    }

    pub fn with_persistence(path: PathBuf) -> Self {
        let file = match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            Ok(f) => Some(f),
            Err(e) => {
                log::warn!("[event-bus] open log: {}", e);
                None
            }
        };
        Self {
            handlers: Arc::new(Mutex::new(Vec::new())),
            log_file: file.map(|f| Arc::new(Mutex::new(f))),
        }
    }

    pub fn emit(&self, event: Box<dyn BusEvent>) {
        if let Some(ref log_file) = self.log_file {
            if let Ok(mut file) = log_file.lock() {
                let json = serialize_event(&*event);
                if let Ok(line) = serde_json::to_string(&json) {
                    use std::io::Write;
                    let _ = writeln!(file, "{}", line);
                }
            }
        }
        let handlers = self.handlers.lock().expect("result");
        for handler in handlers.iter() {
            handler(event.clone_box());
        }
    }

    pub fn on<F>(&mut self, handler: F)
    where
        F: Fn(Box<dyn BusEvent>) + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.lock().expect("result");
        handlers.push(Box::new(handler));
    }
}

impl Default for EventBus {
    fn default() -> Self { Self::new() }
}

pub fn replay<F>(path: &PathBuf, callback: F)
where
    F: Fn(serde_json::Value) + Send + Sync + 'static,
{
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    for line in content.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            callback(json);
        }
    }
}

fn serialize_event(event: &dyn BusEvent) -> serde_json::Value {
    let any = event.as_any();
    let ts = chrono::Utc::now().timestamp();

    if let Some(e) = any.downcast_ref::<crate::core::nt_core_event::TaskSubmittedEvent>() {
        return json!({
            "type": "TaskSubmitted", "timestamp": ts,
            "task": e.task, "task_type": e.task_type, "priority": e.priority
        });
    }
    if let Some(e) = any.downcast_ref::<crate::core::nt_core_event::AgentFeedbackEvent>() {
        return json!({
            "type": "AgentFeedback", "timestamp": ts,
            "agent_id": e.agent_id, "feedback": e.feedback, "score": e.score
        });
    }
    if let Some(e) = any.downcast_ref::<crate::core::nt_core_event::GlobalHaltEvent>() {
        return json!({
            "type": "GlobalHalt", "timestamp": ts,
            "reason": e.reason, "source": e.source
        });
    }
    if let Some(e) = any.downcast_ref::<crate::core::nt_core_event::ExternalRewardEvent>() {
        return json!({
            "type": "ExternalReward", "timestamp": ts,
            "reward": e.reward, "source": e.source
        });
    }
    if let Some(e) = any.downcast_ref::<crate::core::nt_core_event::GoalCompletedEvent>() {
        return json!({
            "type": "GoalCompleted", "timestamp": ts,
            "goal_id": e.goal_id, "goal": e.goal,
            "iterations": e.iterations, "score": e.score
        });
    }
    if let Some(e) = any.downcast_ref::<crate::core::nt_core_event::BudgetExceededEvent>() {
        return json!({
            "type": "BudgetExceeded", "timestamp": ts,
            "goal_id": e.goal_id, "budget_used": e.budget_used, "max_budget": e.max_budget
        });
    }
    if let Some(e) = any.downcast_ref::<crate::core::nt_core_event::AgentTeamEvent>() {
        return json!({
            "type": "AgentTeam", "ts": ts,
            "agent_id": e.agent_id, "action": e.action, "event_timestamp": e.timestamp
        });
    }
    if let Some(e) = any.downcast_ref::<crate::core::nt_core_event::SystemErrorEvent>() {
        return json!({
            "type": "SystemError", "timestamp": ts,
            "component": e.component, "error": e.error, "severity": e.severity
        });
    }
    json!({ "type": "Unknown", "timestamp": ts })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus_new() {
        let bus = EventBus::new();
        assert!(bus.log_file.is_none());
    }

    #[test]
    fn test_event_bus_emit_no_handlers() {
        let bus = EventBus::new();
        let event = Box::new(MockEvent);
        bus.emit(event);
    }

    #[test]
    fn test_event_bus_on_and_emit() {
        let mut bus = EventBus::new();
        let called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let called_clone = called.clone();
        bus.on(move |_e| {
            *called_clone.lock().unwrap() = true;
        });
        bus.emit(Box::new(MockEvent));
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_serialize_unknown_event() {
        let json = serialize_event(&MockEvent);
        assert_eq!(json["type"], "Unknown");
    }

    struct MockEvent;
    impl BusEvent for MockEvent {
        fn as_any(&self) -> &dyn std::any::Any { self }
        fn clone_box(&self) -> Box<dyn BusEvent> { Box::new(MockEvent) }
    }
}
