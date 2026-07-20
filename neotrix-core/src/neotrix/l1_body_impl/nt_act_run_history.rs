use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct RunRecord {
    pub run_id: String,
    pub handler: String,
    pub timestamp: u64,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl RunRecord {
    pub fn new(handler: &str, success: bool) -> Self {
        Self {
            run_id: format!("{}_{}", handler, Self::now_secs()),
            handler: handler.to_string(),
            timestamp: Self::now_secs(),
            duration_ms: 0,
            success,
            error: None,
            metadata: HashMap::new(),
        }
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    pub fn with_error(mut self, error: &str) -> Self {
        self.error = Some(error.to_string());
        self.success = false;
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
}

#[derive(Default)]
pub struct RunHistoryStore {
    records: Vec<RunRecord>,
    max_records: usize,
}

impl RunHistoryStore {
    pub fn new(max_records: usize) -> Self {
        Self { records: Vec::new(), max_records }
    }

    pub fn record(&mut self, record: RunRecord) {
        self.records.push(record);
        if self.records.len() > self.max_records {
            self.records.remove(0);
        }
    }

    pub fn recent(&self, n: usize) -> &[RunRecord] {
        let start = self.records.len().saturating_sub(n);
        &self.records[start..]
    }

    pub fn all(&self) -> &[RunRecord] {
        &self.records
    }

    pub fn success_rate(&self) -> f64 {
        if self.records.is_empty() {
            return 1.0;
        }
        let successes = self.records.iter().filter(|r| r.success).count();
        successes as f64 / self.records.len() as f64
    }
}
