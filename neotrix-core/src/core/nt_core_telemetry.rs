use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum TelemetryEvent {
    AgentSpawned { agent_id: String, role: String },
    AgentCompleted { agent_id: String, success: bool, duration_ms: u64 },
    ToolCall { tool: String, success: bool, duration_ms: u64 },
    KnowledgeAbsorbed { source: String, count: u64 },
    Error { source: String, message: String, severity: u8 },
    ConsciousnessTick { phi: f64, coherence: f64, quality: f64 },
    Sealed { cycle: u64, reward: f64 },
    Custom { name: String, value: String },
}

impl TelemetryEvent {
    pub fn kind(&self) -> &str {
        match self {
            Self::AgentSpawned { .. } => "agent_spawned",
            Self::AgentCompleted { .. } => "agent_completed",
            Self::ToolCall { .. } => "tool_call",
            Self::KnowledgeAbsorbed { .. } => "knowledge_absorbed",
            Self::Error { .. } => "error",
            Self::ConsciousnessTick { .. } => "consciousness_tick",
            Self::Sealed { .. } => "sealed",
            Self::Custom { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AggregatedMetric {
    pub count: u64,
    pub error_count: u64,
    pub total_duration_ms: u64,
    pub last_seen: Instant,
}

pub struct TelemetryStore {
    events: Mutex<VecDeque<(Instant, TelemetryEvent)>>,
    max_events: usize,
    metrics: Mutex<HashMap<String, AggregatedMetric>>,
}

impl TelemetryStore {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Mutex::new(VecDeque::with_capacity(max_events.min(100_000))),
            max_events,
            metrics: Mutex::new(HashMap::new()),
        }
    }

    pub fn record(&self, event: TelemetryEvent) {
        let kind = event.kind().to_string();
        let is_error = matches!(&event, TelemetryEvent::Error { .. });
        let duration = match &event {
            TelemetryEvent::ToolCall { duration_ms, .. } => *duration_ms,
            TelemetryEvent::AgentCompleted { duration_ms, .. } => *duration_ms,
            _ => 0,
        };

        if let Ok(mut events) = self.events.lock() {
            if events.len() >= self.max_events {
                events.pop_front();
            }
            events.push_back((Instant::now(), event));
        }

        if let Ok(mut metrics) = self.metrics.lock() {
            let entry = metrics.entry(kind).or_insert(AggregatedMetric {
                count: 0,
                error_count: 0,
                total_duration_ms: 0,
                last_seen: Instant::now(),
            });
            entry.count += 1;
            if is_error {
                entry.error_count += 1;
            }
            entry.total_duration_ms += duration;
            entry.last_seen = Instant::now();
        }
    }

    pub fn summary(&self) -> Vec<(String, AggregatedMetric)> {
        let mut result: Vec<_> = match self.metrics.lock() {
            Ok(m) => m.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            Err(_) => return vec![],
        };
        result.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        result
    }

    pub fn recent_events(&self, n: usize) -> Vec<(Instant, TelemetryEvent)> {
        match self.events.lock() {
            Ok(events) => events.iter().rev().take(n).cloned().collect(),
            Err(_) => vec![],
        }
    }

    pub fn errors_in_last(&self, duration: Duration) -> u64 {
        let cutoff = Instant::now() - duration;
        match self.events.lock() {
            Ok(events) => events
                .iter()
                .filter(|(t, e)| *t > cutoff && matches!(e, TelemetryEvent::Error { .. }))
                .count() as u64,
            Err(_) => 0,
        }
    }

    pub fn clear(&self) {
        if let Ok(mut events) = self.events.lock() {
            events.clear();
        }
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.clear();
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentBehavior {
    pub agent_id: String,
    pub call_count: u64,
    pub error_count: u64,
    pub total_duration_ms: u64,
    pub tools_used: Vec<String>,
    pub last_active: Instant,
}

pub struct AgentBehaviorMap {
    agents: Mutex<HashMap<String, AgentBehavior>>,
    tool_frequencies: Mutex<HashMap<String, u64>>,
    error_patterns: Mutex<Vec<(String, String)>>,
}

impl Default for AgentBehaviorMap {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentBehaviorMap {
    pub fn new() -> Self {
        Self {
            agents: Mutex::new(HashMap::new()),
            tool_frequencies: Mutex::new(HashMap::new()),
            error_patterns: Mutex::new(Vec::new()),
        }
    }

    pub fn record_event(&self, event: &TelemetryEvent) {
        match event {
            TelemetryEvent::AgentSpawned { agent_id, role } => {
                if let Ok(mut agents) = self.agents.lock() {
                    agents.entry(agent_id.clone()).or_insert(AgentBehavior {
                        agent_id: agent_id.clone(),
                        call_count: 0,
                        error_count: 0,
                        total_duration_ms: 0,
                        tools_used: vec![],
                        last_active: Instant::now(),
                    });
                }
                if let Ok(mut freqs) = self.tool_frequencies.lock() {
                    *freqs.entry(format!("role:{}", role)).or_insert(0) += 1;
                }
            }
            TelemetryEvent::AgentCompleted { agent_id, success, duration_ms } => {
                if let Ok(mut agents) = self.agents.lock() {
                    if let Some(a) = agents.get_mut(agent_id) {
                        a.call_count += 1;
                        if !success {
                            a.error_count += 1;
                        }
                        a.total_duration_ms += duration_ms;
                        a.last_active = Instant::now();
                    }
                }
            }
            TelemetryEvent::ToolCall { tool, success, duration_ms } => {
                if let Ok(mut freqs) = self.tool_frequencies.lock() {
                    *freqs.entry(tool.clone()).or_insert(0) += 1;
                }
                if let Ok(_error_patterns) = self.error_patterns.lock() {
                    if !success {
                        // keep for future expansion
                    }
                }
                let _ = duration_ms;
            }
            TelemetryEvent::Error { source, message, .. } => {
                if let Ok(mut patterns) = self.error_patterns.lock() {
                    patterns.push((source.clone(), message.clone()));
                    let excess = patterns.len().saturating_sub(1000);
                    if excess > 0 {
                        patterns.drain(0..excess);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn top_tools(&self, n: usize) -> Vec<(String, u64)> {
        match self.tool_frequencies.lock() {
            Ok(freqs) => {
                let mut pairs: Vec<_> = freqs.iter().map(|(k, v)| (k.clone(), *v)).collect();
                pairs.sort_by(|a, b| b.1.cmp(&a.1));
                pairs.truncate(n);
                pairs
            }
            Err(_) => vec![],
        }
    }

    pub fn active_agents(&self) -> Vec<(String, u64, u64)> {
        match self.agents.lock() {
            Ok(agents) => agents
                .iter()
                .map(|(id, a)| (id.clone(), a.call_count, a.error_count))
                .collect(),
            Err(_) => vec![],
        }
    }

    pub fn high_error_tools(&self, threshold: f64) -> Vec<String> {
        match self.error_patterns.lock() {
            Ok(patterns) => {
                let mut sources: HashMap<String, u64> = HashMap::new();
                for (src, _) in patterns.iter() {
                    *sources.entry(src.clone()).or_insert(0) += 1;
                }
                let total = sources.values().sum::<u64>() as f64;
                sources
                    .into_iter()
                    .filter(|(_, count)| *count as f64 / total.max(1.0) > threshold)
                    .map(|(k, _)| k)
                    .collect()
            }
            Err(_) => vec![],
        }
    }
}

impl std::fmt::Debug for AgentBehaviorMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AgentBehaviorMap")
    }
}

static GLOBAL_TELEMETRY_STORE: std::sync::LazyLock<TelemetryStore> =
    std::sync::LazyLock::new(|| TelemetryStore::new(10_000));

static GLOBAL_AGENT_MAP: std::sync::LazyLock<AgentBehaviorMap> =
    std::sync::LazyLock::new(AgentBehaviorMap::new);

pub fn global_telemetry() -> &'static TelemetryStore {
    &GLOBAL_TELEMETRY_STORE
}

pub fn global_agent_map() -> &'static AgentBehaviorMap {
    &GLOBAL_AGENT_MAP
}

impl crate::core::nt_core_self_test::SelfTest for TelemetryStore {
    fn name(&self) -> &str {
        "TelemetryStore"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let summary = self.summary();
        let _ = summary;
        if self.max_events == 0 {
            failures.push("max_events is 0".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_store_record_and_summary() {
        let store = TelemetryStore::new(100);
        store.record(TelemetryEvent::ToolCall { tool: "bash".into(), success: true, duration_ms: 50 });
        store.record(TelemetryEvent::ToolCall { tool: "bash".into(), success: true, duration_ms: 30 });
        store.record(TelemetryEvent::Error { source: "bash".into(), message: "timeout".into(), severity: 2 });
        let summary = store.summary();
        assert!(summary.iter().any(|(k, _)| k == "tool_call"));
        assert!(summary.iter().any(|(k, _)| k == "error"));
        assert_eq!(store.errors_in_last(Duration::from_secs(60)), 1);
    }

    #[test]
    fn test_telemetry_store_max_events() {
        let store = TelemetryStore::new(10);
        for i in 0..20 {
            store.record(TelemetryEvent::Custom { name: format!("e{}", i), value: "x".into() });
        }
        let recent = store.recent_events(100);
        assert!(recent.len() <= 10);
    }

    #[test]
    fn test_agent_behavior_map() {
        let map = AgentBehaviorMap::new();
        map.record_event(&TelemetryEvent::AgentSpawned { agent_id: "a1".into(), role: "coder".into() });
        map.record_event(&TelemetryEvent::AgentCompleted { agent_id: "a1".into(), success: true, duration_ms: 100 });
        map.record_event(&TelemetryEvent::ToolCall { tool: "bash".into(), success: true, duration_ms: 10 });
        map.record_event(&TelemetryEvent::ToolCall { tool: "read".into(), success: false, duration_ms: 5 });
        let agents = map.active_agents();
        assert!(agents.iter().any(|(id, _, _)| id == "a1"));
        let tools = map.top_tools(5);
        assert!(tools.iter().any(|(t, _)| t == "bash"));
    }

    #[test]
    fn test_error_pattern_detection() {
        let map = AgentBehaviorMap::new();
        for _ in 0..8 {
            map.record_event(&TelemetryEvent::Error { source: "bash".into(), message: "timeout".into(), severity: 2 });
        }
        for _ in 0..2 {
            map.record_event(&TelemetryEvent::Error { source: "read".into(), message: "not found".into(), severity: 1 });
        }
        let high_error = map.high_error_tools(0.5);
        assert!(high_error.contains(&"bash".to_string()));
    }

    #[test]
    fn test_telemetry_store_clear() {
        let store = TelemetryStore::new(100);
        store.record(TelemetryEvent::ToolCall { tool: "test".into(), success: true, duration_ms: 1 });
        assert!(!store.summary().is_empty());
        store.clear();
        assert!(store.summary().is_empty());
    }

    #[test]
    fn test_event_kind() {
        assert_eq!(TelemetryEvent::ToolCall { tool: "x".into(), success: true, duration_ms: 0 }.kind(), "tool_call");
        assert_eq!(TelemetryEvent::AgentSpawned { agent_id: "x".into(), role: "r".into() }.kind(), "agent_spawned");
        assert_eq!(TelemetryEvent::Error { source: "x".into(), message: "m".into(), severity: 1 }.kind(), "error");
        assert_eq!(TelemetryEvent::Custom { name: "my_event".into(), value: "v".into() }.kind(), "my_event");
    }
}
