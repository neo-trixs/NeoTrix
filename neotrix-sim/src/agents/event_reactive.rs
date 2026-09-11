use crate::foundation::simulation_bus::SimEvent;
use crate::foundation::math_bridge::Vec2;
use crate::agents::sim_agent::AgentAction;
use std::collections::HashMap;

/// What an agent wants to do in response to an event
#[derive(Debug, Clone)]
pub struct ReactiveResponse {
    pub agent_id: String,
    pub action: AgentAction,
    pub priority: f32,
    pub reason: String,
}

/// Event subscription for an agent
#[derive(Debug, Clone)]
pub struct EventSubscription {
    pub agent_id: String,
    pub event_filter: EventFilter,
    pub response_template: ResponseTemplate,
}

#[derive(Debug, Clone)]
pub enum EventFilter {
    /// React to any agent death
    AgentDeath,
    /// React to resource depletion in area
    ResourceDepleted { radius: f32 },
    /// React to events involving this agent
    DirectMention,
    /// React to high-priority events
    HighPriority,
    /// React on a tick interval (every N ticks)
    Time { tick_interval: u64 },
    /// React after a duration has elapsed (ticks since subscription)
    Timer { duration: u64 },
    /// React when agent state transitions between specific values
    StateChange { from: String, to: String },
}

#[derive(Debug, Clone)]
pub enum ResponseTemplate {
    /// Flee from danger
    Flee { distance: f32 },
    /// Rest
    Rest,
    /// Alert nearby agents (talk)
    Alert { message: String },
    /// No response (just record)
    Observe,
}

/// Manages event subscriptions and reactive responses for all agents
pub struct EventReactiveSystem {
    subscriptions: Vec<EventSubscription>,
    max_pending_per_agent: usize,
    /// Track last trigger tick for Time/Timer filters: (sub_index, last_triggered_tick)
    timer_state: std::collections::HashMap<usize, u64>,
}

impl EventReactiveSystem {
    pub fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
            max_pending_per_agent: 3,
            timer_state: std::collections::HashMap::new(),
        }
    }

    /// Subscribe an agent to events
    pub fn subscribe(&mut self, subscription: EventSubscription) {
        let idx = self.subscriptions.len();
        self.subscriptions.push(subscription);
        // Initialize timer tracking for Time/Timer filters
        if matches!(
            self.subscriptions[idx].event_filter,
            EventFilter::Time { .. } | EventFilter::Timer { .. }
        ) {
            self.timer_state.insert(idx, 0);
        }
    }

    /// Remove all subscriptions for an agent
    pub fn unsubscribe_agent(&mut self, agent_id: &str) {
        let before = self.subscriptions.len();
        self.subscriptions.retain(|s| s.agent_id != agent_id);
        let after = self.subscriptions.len();
        if before != after {
            // Rebuild timer_state since indices shifted
            self.timer_state.clear();
            for (i, sub) in self.subscriptions.iter().enumerate() {
                if matches!(
                    sub.event_filter,
                    EventFilter::Time { .. } | EventFilter::Timer { .. }
                ) {
                    self.timer_state.insert(i, 0);
                }
            }
        }
    }

    /// Process events and generate reactive responses
    pub fn process_events(&mut self, events: &[SimEvent], tick: u64) -> Vec<ReactiveResponse> {
        let mut responses = Vec::new();

        for event in events {
            for (sub_idx, sub) in self.subscriptions.iter().enumerate() {
                if self.matches_filter(event, &sub.event_filter, tick, sub_idx) {
                    // Update timer state for Time/Timer filters
                    if matches!(
                        sub.event_filter,
                        EventFilter::Time { .. } | EventFilter::Timer { .. }
                    ) {
                        self.timer_state.insert(sub_idx, tick);
                    }
                    if let Some(response) = self.generate_response(event, sub, tick) {
                        responses.push(response);
                    }
                }
            }
        }

        self.deduplicate(responses)
    }

    fn matches_filter(&self, event: &SimEvent, filter: &EventFilter, tick: u64, sub_idx: usize) -> bool {
        match filter {
            EventFilter::AgentDeath => matches!(event, SimEvent::AgentDied { .. }),
            EventFilter::ResourceDepleted { .. } => {
                matches!(event, SimEvent::ResourceDepleted { .. })
            }
            EventFilter::DirectMention => {
                // Simplified: always matches (in real impl would check agent_id)
                true
            }
            EventFilter::HighPriority => {
                matches!(event,
                    SimEvent::AgentDied { .. }
                    | SimEvent::AgentNearDeath { .. }
                    | SimEvent::EnvironmentHazard { .. })
            }
            EventFilter::Time { tick_interval } => {
                // Match every tick_interval ticks
                let last = self.timer_state.get(&sub_idx).copied().unwrap_or(0);
                tick.saturating_sub(last) >= *tick_interval
            }
            EventFilter::Timer { duration } => {
                // Match after duration ticks since subscription
                let last = self.timer_state.get(&sub_idx).copied().unwrap_or(0);
                tick.saturating_sub(last) >= *duration
            }
            EventFilter::StateChange { from, to } => {
                // Match on AgentActed events where the action/result contains state info
                match event {
                    SimEvent::AgentActed { action, result, .. } => {
                        action.contains(from) && result.contains(to)
                    }
                    _ => false,
                }
            }
        }
    }

    fn generate_response(
        &self,
        event: &SimEvent,
        sub: &EventSubscription,
        _tick: u64,
    ) -> Option<ReactiveResponse> {
        match &sub.response_template {
            ResponseTemplate::Flee { distance } => {
                // Compute flee direction: away from event source
                let flee_dir = self.flee_direction(event, sub);
                let move_target = flee_dir * *distance;
                Some(ReactiveResponse {
                    agent_id: sub.agent_id.clone(),
                    action: AgentAction::Move { target: move_target },
                    priority: 0.9,
                    reason: "fleeing danger".to_string(),
                })
            }
            ResponseTemplate::Rest => Some(ReactiveResponse {
                agent_id: sub.agent_id.clone(),
                action: AgentAction::Rest,
                priority: 0.7,
                reason: "resting after event".to_string(),
            }),
            ResponseTemplate::Alert { message } => Some(ReactiveResponse {
                agent_id: sub.agent_id.clone(),
                action: AgentAction::Talk {
                    target_id: String::new(),
                    message: message.clone(),
                },
                priority: 0.6,
                reason: "alerting others".to_string(),
            }),
            ResponseTemplate::Observe => None,
        }
    }

    fn flee_direction(&self, event: &SimEvent, _sub: &EventSubscription) -> Vec2 {
        // Simplified: return a generic flee direction based on event type
        match event {
            SimEvent::AgentDied { .. } | SimEvent::AgentNearDeath { .. } => {
                Vec2::new(0.0, -1.0) // flee south
            }
            SimEvent::EnvironmentHazard { position, .. } => {
                Vec2::new(-position.0, -position.1).normalize_or_zero()
            }
            SimEvent::ResourceDepleted { .. } => {
                Vec2::new(1.0, 0.0) // flee east
            }
            _ => Vec2::new(0.0, 0.0),
        }
    }

    fn deduplicate(&self, responses: Vec<ReactiveResponse>) -> Vec<ReactiveResponse> {
        let mut best: HashMap<String, ReactiveResponse> = HashMap::new();
        for r in responses {
            let dominated = best.get(&r.agent_id).map_or(false, |existing| {
                r.priority <= existing.priority
            });
            if !dominated {
                best.insert(r.agent_id.clone(), r);
            }
        }
        best.into_values().collect()
    }

    pub fn subscription_count(&self) -> usize {
        self.subscriptions.len()
    }

    pub fn pending_count(&self) -> usize {
        0 // pending is cleared after process_events
    }

    pub fn max_pending_per_agent(&self) -> usize {
        self.max_pending_per_agent
    }
}

/// Helper trait for Vec2 normalization that returns zero on degenerate input
trait Vec2Normalize {
    fn normalize_or_zero(&self) -> Vec2;
}

impl Vec2Normalize for Vec2 {
    fn normalize_or_zero(&self) -> Vec2 {
        let len = self.length();
        if len < 1e-10 {
            Vec2::zero()
        } else {
            Vec2::new(self.x / len, self.y / len)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::simulation_bus::SimEvent;

    #[test]
    fn subscription_adds_count() {
        let mut sys = EventReactiveSystem::new();
        assert_eq!(sys.subscription_count(), 0);

        sys.subscribe(EventSubscription {
            agent_id: "a1".into(),
            event_filter: EventFilter::AgentDeath,
            response_template: ResponseTemplate::Flee { distance: 10.0 },
        });
        assert_eq!(sys.subscription_count(), 1);
    }

    #[test]
    fn unsubscribe_removes_all() {
        let mut sys = EventReactiveSystem::new();
        sys.subscribe(EventSubscription {
            agent_id: "a1".into(),
            event_filter: EventFilter::AgentDeath,
            response_template: ResponseTemplate::Flee { distance: 10.0 },
        });
        sys.subscribe(EventSubscription {
            agent_id: "a1".into(),
            event_filter: EventFilter::HighPriority,
            response_template: ResponseTemplate::Rest,
        });
        sys.subscribe(EventSubscription {
            agent_id: "a2".into(),
            event_filter: EventFilter::AgentDeath,
            response_template: ResponseTemplate::Rest,
        });
        assert_eq!(sys.subscription_count(), 3);

        sys.unsubscribe_agent("a1");
        assert_eq!(sys.subscription_count(), 1);
    }

    #[test]
    fn flee_response_on_agent_death() {
        let mut sys = EventReactiveSystem::new();
        sys.subscribe(EventSubscription {
            agent_id: "a1".into(),
            event_filter: EventFilter::AgentDeath,
            response_template: ResponseTemplate::Flee { distance: 10.0 },
        });

        let events = vec![SimEvent::AgentDied {
            agent_id: "victim".into(),
            cause: "starvation".into(),
        }];

        let responses = sys.process_events(&events, 1);
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0].agent_id, "a1");
        assert!(responses[0].priority > 0.8);
        assert!(matches!(responses[0].action, AgentAction::Move { .. }));
    }

    #[test]
    fn rest_response_on_near_death() {
        let mut sys = EventReactiveSystem::new();
        sys.subscribe(EventSubscription {
            agent_id: "a1".into(),
            event_filter: EventFilter::HighPriority,
            response_template: ResponseTemplate::Rest,
        });

        let events = vec![SimEvent::AgentNearDeath {
            agent_id: "a1".into(),
            energy: 5.0,
        }];

        let responses = sys.process_events(&events, 1);
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0].agent_id, "a1");
        assert!(matches!(responses[0].action, AgentAction::Rest));
    }

    #[test]
    fn deduplication_keeps_highest_priority() {
        let mut sys = EventReactiveSystem::new();
        // Two subscriptions for same agent: one low priority (Observe->None), one high
        sys.subscribe(EventSubscription {
            agent_id: "a1".into(),
            event_filter: EventFilter::AgentDeath,
            response_template: ResponseTemplate::Alert {
                message: "danger!".into(),
            },
        });
        sys.subscribe(EventSubscription {
            agent_id: "a1".into(),
            event_filter: EventFilter::HighPriority,
            response_template: ResponseTemplate::Flee { distance: 5.0 },
        });

        let events = vec![SimEvent::AgentDied {
            agent_id: "victim".into(),
            cause: "attack".into(),
        }];

        let responses = sys.process_events(&events, 1);
        assert_eq!(responses.len(), 1);
        // Flee (0.9) should beat Alert (0.6)
        assert!(matches!(responses[0].action, AgentAction::Move { .. }));
    }

    #[test]
    fn no_response_for_observe_template() {
        let mut sys = EventReactiveSystem::new();
        sys.subscribe(EventSubscription {
            agent_id: "a1".into(),
            event_filter: EventFilter::AgentDeath,
            response_template: ResponseTemplate::Observe,
        });

        let events = vec![SimEvent::AgentDied {
            agent_id: "victim".into(),
            cause: "starvation".into(),
        }];

        let responses = sys.process_events(&events, 1);
        assert!(responses.is_empty());
    }

    #[test]
    fn multiple_agents_respond() {
        let mut sys = EventReactiveSystem::new();
        sys.subscribe(EventSubscription {
            agent_id: "a1".into(),
            event_filter: EventFilter::AgentDeath,
            response_template: ResponseTemplate::Flee { distance: 10.0 },
        });
        sys.subscribe(EventSubscription {
            agent_id: "a2".into(),
            event_filter: EventFilter::AgentDeath,
            response_template: ResponseTemplate::Rest,
        });

        let events = vec![SimEvent::AgentDied {
            agent_id: "victim".into(),
            cause: "attack".into(),
        }];

        let responses = sys.process_events(&events, 1);
        assert_eq!(responses.len(), 2);
        let ids: Vec<&str> = responses.iter().map(|r| r.agent_id.as_str()).collect();
        assert!(ids.contains(&"a1"));
        assert!(ids.contains(&"a2"));
    }

    #[test]
    fn no_match_for_unrelated_event() {
        let mut sys = EventReactiveSystem::new();
        sys.subscribe(EventSubscription {
            agent_id: "a1".into(),
            event_filter: EventFilter::AgentDeath,
            response_template: ResponseTemplate::Flee { distance: 10.0 },
        });

        let events = vec![SimEvent::ResourceRegenerated {
            resource_id: "r1".into(),
            position: (50.0, 50.0),
        }];

        let responses = sys.process_events(&events, 1);
        assert!(responses.is_empty());
    }

    #[test]
    fn flee_direction_from_hazard() {
        let mut sys = EventReactiveSystem::new();
        sys.subscribe(EventSubscription {
            agent_id: "a1".into(),
            event_filter: EventFilter::HighPriority,
            response_template: ResponseTemplate::Flee { distance: 10.0 },
        });

        let events = vec![SimEvent::EnvironmentHazard {
            position: (10.0, 0.0),
            hazard_type: "fire".into(),
        }];

        let responses = sys.process_events(&events, 1);
        assert_eq!(responses.len(), 1);
        if let AgentAction::Move { target } = &responses[0].action {
            // Should flee in negative x direction (away from hazard at x=10)
            assert!(target.x < 0.0);
        } else {
            panic!("Expected Move action");
        }
    }

    #[test]
    fn alert_response() {
        let mut sys = EventReactiveSystem::new();
        sys.subscribe(EventSubscription {
            agent_id: "a1".into(),
            event_filter: EventFilter::AgentDeath,
            response_template: ResponseTemplate::Alert {
                message: "danger nearby".into(),
            },
        });

        let events = vec![SimEvent::AgentDied {
            agent_id: "victim".into(),
            cause: "predator".into(),
        }];

        let responses = sys.process_events(&events, 1);
        assert_eq!(responses.len(), 1);
        if let AgentAction::Talk { message, .. } = &responses[0].action {
            assert_eq!(message, "danger nearby");
        } else {
            panic!("Expected Talk action");
        }
    }

    #[test]
    fn resource_depleted_triggers_flee() {
        let mut sys = EventReactiveSystem::new();
        sys.subscribe(EventSubscription {
            agent_id: "a1".into(),
            event_filter: EventFilter::ResourceDepleted { radius: 50.0 },
            response_template: ResponseTemplate::Flee { distance: 5.0 },
        });

        let events = vec![SimEvent::ResourceDepleted {
            resource_id: "r1".into(),
            position: (20.0, 20.0),
        }];

        let responses = sys.process_events(&events, 1);
        assert_eq!(responses.len(), 1);
        assert!(matches!(responses[0].action, AgentAction::Move { .. }));
    }
}
