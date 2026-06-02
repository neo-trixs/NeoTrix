use std::time::{SystemTime, UNIX_EPOCH};
use super::SensoryEvent;

/// Result from a real sensor capture.
#[derive(Debug, Clone)]
pub struct SensorSample {
    pub timestamp_ms: u64,
    pub raw_bytes: Vec<u8>,
    pub metadata: std::collections::HashMap<String, String>,
    pub confidence: f64,
}

impl SensorSample {
    pub fn new(raw: Vec<u8>) -> Self {
        Self {
            timestamp_ms: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64,
            raw_bytes: raw,
            metadata: std::collections::HashMap::new(),
            confidence: 0.9,
        }
    }

    pub fn with_meta(mut self, key: &str, val: &str) -> Self {
        self.metadata.insert(key.to_string(), val.to_string());
        self
    }

    pub fn size(&self) -> usize { self.raw_bytes.len() }
}

/// Trait for real (non-simulated) sensor implementations.
pub trait Sensor: Send {
    fn name(&self) -> &str;
    fn poll(&mut self) -> Option<SensorSample>;
    fn activate(&mut self);
    fn deactivate(&mut self);
    fn is_active(&self) -> bool;
    fn to_event(&self, sample: SensorSample) -> SensoryEvent;
}
