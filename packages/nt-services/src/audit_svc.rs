use chrono::{DateTime, Utc};
use nt_domain::*;
use nt_db::audit::AuditRepo;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Mutex;
use uuid::Uuid;

pub struct AuditService {
    repo: Box<dyn AuditRepo>,
}

#[derive(Debug, Clone, Default)]
pub struct AuditStats {
    pub total_entries: usize,
    pub allowed_count: usize,
    pub denied_count: usize,
    pub error_count: usize,
    pub unique_actors: usize,
}

impl AuditService {
    pub fn new(repo: Box<dyn AuditRepo>) -> Self {
        Self { repo }
    }

    pub fn log(
        &self,
        actor_id: Uuid,
        actor_type: &str,
        action: &str,
        resource_type: &str,
        resource_id: &str,
        outcome: AuditOutcome,
        detail: Value,
    ) -> Result<AuditEntry, String> {
        let entry = AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            actor_id,
            actor_type: actor_type.to_string(),
            action: action.to_string(),
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            outcome,
            detail,
        };
        self.repo.append_entry(&entry).map(|_| entry)
    }

    pub fn log_allow(
        &self,
        actor_id: Uuid,
        action: &str,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<AuditEntry, String> {
        self.log(
            actor_id,
            "user",
            action,
            resource_type,
            resource_id,
            AuditOutcome::Allowed,
            serde_json::json!({}),
        )
    }

    pub fn log_deny(
        &self,
        actor_id: Uuid,
        action: &str,
        resource_type: &str,
        resource_id: &str,
        reason: &str,
    ) -> Result<AuditEntry, String> {
        self.log(
            actor_id,
            "user",
            action,
            resource_type,
            resource_id,
            AuditOutcome::Denied,
            serde_json::json!({ "reason": reason }),
        )
    }

    pub fn query_by_actor(&self, actor_id: Uuid, limit: usize) -> Vec<AuditEntry> {
        self.repo.query_by_actor(actor_id, limit)
    }

    pub fn query_by_resource(
        &self,
        resource_type: &str,
        resource_id: &str,
        limit: usize,
    ) -> Vec<AuditEntry> {
        self.repo
            .query_by_resource(resource_type, resource_id, limit)
    }

    pub fn query_recent(&self, minutes: i64, limit: usize) -> Vec<AuditEntry> {
        let now = Utc::now();
        let start = now - chrono::Duration::minutes(minutes);
        self.repo.query_by_time_range(start, now, limit)
    }

    pub fn get_stats(&self, since: DateTime<Utc>) -> AuditStats {
        let entries = self
            .repo
            .query_by_time_range(since, Utc::now(), usize::MAX);
        let total_entries = entries.len();
        let allowed_count = entries
            .iter()
            .filter(|e| e.outcome == AuditOutcome::Allowed)
            .count();
        let denied_count = entries
            .iter()
            .filter(|e| e.outcome == AuditOutcome::Denied)
            .count();
        let error_count = entries
            .iter()
            .filter(|e| e.outcome == AuditOutcome::Error)
            .count();
        let unique_actors: HashSet<Uuid> = entries.iter().map(|e| e.actor_id).collect();
        AuditStats {
            total_entries,
            allowed_count,
            denied_count,
            error_count,
            unique_actors: unique_actors.len(),
        }
    }
}

pub struct MemAuditRepo {
    entries: Mutex<Vec<AuditEntry>>,
}

impl MemAuditRepo {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
        }
    }
}

impl AuditRepo for MemAuditRepo {
    fn append_entry(&self, entry: &AuditEntry) -> Result<(), String> {
        let mut entries = self.entries.lock().map_err(|e| e.to_string())?;
        entries.push(entry.clone());
        Ok(())
    }

    fn query_by_actor(
        &self,
        actor_id: uuid::Uuid,
        limit: usize,
    ) -> Vec<AuditEntry> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .filter(|e| e.actor_id == actor_id)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    fn query_by_resource(
        &self,
        resource_type: &str,
        resource_id: &str,
        limit: usize,
    ) -> Vec<AuditEntry> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .filter(|e| e.resource_type == resource_type && e.resource_id == resource_id)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    fn query_by_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
        limit: usize,
    ) -> Vec<AuditEntry> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn svc() -> AuditService {
        AuditService::new(Box::new(MemAuditRepo::new()))
    }

    #[test]
    fn test_log_entry() {
        let s = svc();
        let actor = Uuid::new_v4();

        let entry = s
            .log(
                actor,
                "admin",
                "delete",
                "workspace",
                "ws-1",
                AuditOutcome::Allowed,
                serde_json::json!({ "reason": "cleanup" }),
            )
            .unwrap();

        assert_eq!(entry.actor_type, "admin");
        assert_eq!(entry.action, "delete");
        assert_eq!(entry.resource_type, "workspace");
        assert_eq!(entry.resource_id, "ws-1");
        assert_eq!(entry.outcome, AuditOutcome::Allowed);
        assert_eq!(entry.detail["reason"], "cleanup");
    }

    #[test]
    fn test_log_allow_deny() {
        let s = svc();
        let actor = Uuid::new_v4();

        let allow = s.log_allow(actor, "create", "workspace", "ws-2").unwrap();
        assert_eq!(allow.outcome, AuditOutcome::Allowed);

        let deny = s
            .log_deny(actor, "delete", "workspace", "ws-2", "insufficient role")
            .unwrap();
        assert_eq!(deny.outcome, AuditOutcome::Denied);
        assert_eq!(deny.detail["reason"], "insufficient role");
    }

    #[test]
    fn test_query_by_actor() {
        let s = svc();
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        s.log_allow(alice, "read", "doc", "d1").unwrap();
        s.log_allow(alice, "write", "doc", "d2").unwrap();
        s.log_deny(bob, "delete", "doc", "d3", "readonly").unwrap();

        let alice_entries = s.query_by_actor(alice, 10);
        assert_eq!(alice_entries.len(), 2);
        assert!(alice_entries.iter().all(|e| e.actor_id == alice));

        let bob_entries = s.query_by_actor(bob, 10);
        assert_eq!(bob_entries.len(), 1);
    }

    #[test]
    fn test_query_by_resource() {
        let s = svc();
        let actor = Uuid::new_v4();

        s.log_allow(actor, "read", "workspace", "w1").unwrap();
        s.log_allow(actor, "write", "workspace", "w2").unwrap();
        s.log_allow(actor, "read", "doc", "d1").unwrap();

        let entries = s.query_by_resource("workspace", "w1", 10);
        assert_eq!(entries.len(), 1);

        let entries = s.query_by_resource("doc", "d1", 10);
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_query_recent() {
        let s = svc();
        let actor = Uuid::new_v4();

        s.log_allow(actor, "login", "session", "s1").unwrap();
        s.log_allow(actor, "read", "doc", "d1").unwrap();

        let recent = s.query_recent(5, 10);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_stats() {
        let s = svc();
        let actor = Uuid::new_v4();

        s.log_allow(actor, "read", "doc", "d1").unwrap();
        s.log_allow(actor, "write", "doc", "d2").unwrap();
        s.log_deny(actor, "delete", "doc", "d3", "no permission")
            .unwrap();
        s.log(
            actor,
            "user",
            "export",
            "doc",
            "d4",
            AuditOutcome::Error,
            serde_json::json!({ "error": "timeout" }),
        )
        .unwrap();

        let since = Utc::now() - Duration::hours(1);
        let stats = s.get_stats(since);

        assert_eq!(stats.total_entries, 4);
        assert_eq!(stats.allowed_count, 2);
        assert_eq!(stats.denied_count, 1);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.unique_actors, 1);
    }
}
