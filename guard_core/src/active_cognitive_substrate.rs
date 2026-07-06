use std::time::Instant;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveEvent {
    pub source: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
}

impl CognitiveEvent {
    pub fn new(source: &str, event_type: &str, payload: serde_json::Value) -> Self {
        CognitiveEvent { source: source.to_string(), event_type: event_type.to_string(), payload, timestamp: Instant::now() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_cognitive_event_new() { let e = CognitiveEvent::new("t", "i", serde_json::json!({"k":"v"})); assert_eq!(e.source, "t"); assert_eq!(e.event_type, "i"); assert_eq!(e.payload["k"], "v"); }
    #[test] fn test_cognitive_event_serialize_deserialize() { let e = CognitiveEvent::new("t", "i", serde_json::json!({"c":42})); let j = serde_json::to_string(&e).unwrap(); let d: CognitiveEvent = serde_json::from_str(&j).unwrap(); assert_eq!(d.source, "t"); assert_eq!(d.payload["c"], 42); assert!(d.timestamp.elapsed().as_secs() < 2); }
    #[test] fn test_cognitive_event_payload_access() { let e = CognitiveEvent::new("s", "e", serde_json::json!({"a":1,"b":[2,3]})); assert_eq!(e.payload["a"], 1); assert_eq!(e.payload["b"][0], 2); }
}
