pub mod classification;
pub mod gdpr;
pub mod audit_trail;
pub mod retention;
pub mod compliance;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub timestamp: std::time::SystemTime,
    pub source_id: String,
    pub action: String,
    pub outcome: AuditOutcome,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AuditOutcome {
    Pass,
    Fail,
    Warn,
    Skip,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditTrail {
    entries: Vec<AuditEntry>,
}

impl AuditTrail {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, e: AuditEntry) {
        self.entries.push(e);
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DeletionStrategy {
    Soft,
    Hard,
    ArchiveThenDelete,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetentionRule {
    pub data_type: String,
    pub max_age: std::time::Duration,
    pub strategy: DeletionStrategy,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RetentionPolicy {
    pub rules: Vec<RetentionRule>,
    pub default_strategy: DeletionStrategy,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            default_strategy: DeletionStrategy::Soft,
        }
    }
}

impl RetentionPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rule(&mut self, r: RetentionRule) {
        self.rules.push(r);
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataRecord {
    pub id: String,
    pub source_id: String,
    pub data_type: String,
    pub created_at: std::time::SystemTime,
    pub size_bytes: u64,
}
