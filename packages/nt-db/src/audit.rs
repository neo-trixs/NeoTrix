use nt_domain::*;

pub trait AuditRepo: Send + Sync {
    fn append_entry(&self, entry: &AuditEntry) -> Result<(), String>;
    fn query_by_actor(&self, actor_id: uuid::Uuid, limit: usize) -> Vec<AuditEntry>;
    fn query_by_resource(&self, resource_type: &str, resource_id: &str, limit: usize) -> Vec<AuditEntry>;
    fn query_by_time_range(&self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>, limit: usize) -> Vec<AuditEntry>;
}
