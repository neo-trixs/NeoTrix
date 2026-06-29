use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProbeSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeFinding {
    pub key: String,
    pub value: String,
    pub source: String,
    pub severity: ProbeSeverity,
    pub confidence: f64,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

impl ProbeFinding {
    pub fn new(key: &str, value: &str, source: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
            source: source.to_string(),
            severity: ProbeSeverity::Info,
            confidence: 0.5,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_severity(mut self, severity: ProbeSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_meta(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub probe_name: String,
    pub target: String,
    pub success: bool,
    pub findings: Vec<ProbeFinding>,
    pub raw_data: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

impl ProbeResult {
    pub fn new(probe_name: &str, target: &str) -> Self {
        Self {
            probe_name: probe_name.to_string(),
            target: target.to_string(),
            success: false,
            findings: Vec::new(),
            raw_data: None,
            error: None,
            duration_ms: 0,
        }
    }

    pub fn with_finding(mut self, finding: ProbeFinding) -> Self {
        self.findings.push(finding);
        self
    }

    pub fn with_raw(mut self, raw: &str) -> Self {
        self.raw_data = Some(raw.to_string());
        self
    }

    pub fn with_error(mut self, err: &str) -> Self {
        self.error = Some(err.to_string());
        self
    }

    pub fn mark_success(mut self) -> Self {
        self.success = true;
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
}

pub trait IntelligenceProbe: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn probe(&self, target: &str, timeout_secs: u64) -> ProbeResult;
}

pub type ProbeBox = Box<dyn IntelligenceProbe>;
