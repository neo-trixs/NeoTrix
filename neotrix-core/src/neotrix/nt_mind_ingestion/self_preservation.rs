use std::time::{Duration, Instant};

const MAX_RECOVERY_STACK: usize = 100;

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_mb: u64,
    pub stage_count: usize,
    pub pipeline_depth: usize,
    pub cpu_seconds: f64,
}

#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub id: usize,
    pub stage: &'static str,
    pub created: Instant,
    pub snapshot: String,
}

pub struct SelfPreservation {
    checkpoints: Vec<Checkpoint>,
    max_checkpoints: usize,
    recovery_stack: Vec<String>,
    resource_guard: bool,
    started: Instant,
}

impl SelfPreservation {
    pub fn new(max_checkpoints: usize) -> Self {
        Self {
            checkpoints: Vec::with_capacity(max_checkpoints),
            max_checkpoints,
            recovery_stack: Vec::new(),
            resource_guard: false,
            started: Instant::now(),
        }
    }

    pub fn save_checkpoint(&mut self, stage: &'static str, snapshot: String) {
        if self.checkpoints.len() >= self.max_checkpoints {
            self.checkpoints.remove(0);
        }
        let id = self.checkpoints.len();
        self.checkpoints.push(Checkpoint {
            id,
            stage,
            created: Instant::now(),
            snapshot: snapshot.clone(),
        });
        if self.recovery_stack.len() >= MAX_RECOVERY_STACK {
            self.recovery_stack.remove(0);
        }
        self.recovery_stack.push(snapshot);
    }

    pub fn restore_last(&mut self) -> Option<String> {
        self.recovery_stack.pop()
    }

    pub fn restore(&self, id: usize) -> Option<&Checkpoint> {
        self.checkpoints.iter().find(|c| c.id == id)
    }

    pub fn enable_resource_guard(&mut self) {
        self.resource_guard = true;
    }

    pub fn protect(&self, usage: &ResourceUsage, memory_limit_mb: u64) -> Option<String> {
        if !self.resource_guard {
            return None;
        }
        if usage.memory_mb > memory_limit_mb {
            return Some(format!(
                "memory {}MB exceeds limit {}MB",
                usage.memory_mb, memory_limit_mb
            ));
        }
        if usage.stage_count > 100 {
            return Some(format!("stage count {} exceeds 100", usage.stage_count));
        }
        None
    }

    pub fn uptime(&self) -> Duration {
        self.started.elapsed()
    }

    pub fn health(&self) -> &'static str {
        let uptime = self.uptime();
        if uptime < Duration::from_secs(60) {
            "starting"
        } else if uptime < Duration::from_secs(3600) {
            "healthy"
        } else {
            "steady"
        }
    }
}

impl Default for SelfPreservation {
    fn default() -> Self {
        Self::new(10)
    }
}

pub fn safe_recovery(reason: &str) -> String {
    format!(
        "[recovery] degraded after: {}. preserving core state.",
        reason
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_restore_checkpoint() {
        let mut sp = SelfPreservation::new(5);
        sp.save_checkpoint("test_stage", "state_v1".to_string());
        let restored = sp.restore_last();
        assert_eq!(restored, Some("state_v1".to_string()));
    }

    #[test]
    fn test_max_checkpoints_evicts_oldest() {
        let mut sp = SelfPreservation::new(2);
        sp.save_checkpoint("s1", "v1".to_string());
        sp.save_checkpoint("s2", "v2".to_string());
        sp.save_checkpoint("s3", "v3".to_string());
        assert_eq!(sp.checkpoints.len(), 2);
        assert_eq!(sp.checkpoints[0].snapshot, "v2");
        assert_eq!(sp.checkpoints[1].snapshot, "v3");
    }

    #[test]
    fn test_restore_by_id() {
        let mut sp = SelfPreservation::new(10);
        sp.save_checkpoint("s1", "v1".to_string());
        let c = sp.restore(0);
        assert!(c.is_some());
        assert_eq!(c.unwrap().stage, "s1");
    }

    #[test]
    fn test_resource_guard_disabled_by_default() {
        let sp = SelfPreservation::new(5);
        let usage = ResourceUsage {
            memory_mb: 99999,
            stage_count: 0,
            pipeline_depth: 0,
            cpu_seconds: 0.0,
        };
        assert!(sp.protect(&usage, 1000).is_none());
    }

    #[test]
    fn test_resource_guard_trigger_memory() {
        let mut sp = SelfPreservation::new(5);
        sp.enable_resource_guard();
        let usage = ResourceUsage {
            memory_mb: 2000,
            stage_count: 5,
            pipeline_depth: 3,
            cpu_seconds: 10.0,
        };
        let warning = sp.protect(&usage, 1000);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("exceeds"));
    }

    #[test]
    fn test_resource_guard_trigger_stage_count() {
        let mut sp = SelfPreservation::new(5);
        sp.enable_resource_guard();
        let usage = ResourceUsage {
            memory_mb: 50,
            stage_count: 150,
            pipeline_depth: 10,
            cpu_seconds: 5.0,
        };
        let warning = sp.protect(&usage, 500);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("stage count"));
    }

    #[test]
    fn test_resource_guard_passes_clean() {
        let mut sp = SelfPreservation::new(5);
        sp.enable_resource_guard();
        let usage = ResourceUsage {
            memory_mb: 50,
            stage_count: 10,
            pipeline_depth: 3,
            cpu_seconds: 1.0,
        };
        assert!(sp.protect(&usage, 500).is_none());
    }

    #[test]
    fn test_uptime() {
        let sp = SelfPreservation::new(5);
        assert!(sp.uptime().as_secs() < 5);
    }

    #[test]
    fn test_health() {
        let sp = SelfPreservation::new(5);
        assert_eq!(sp.health(), "starting");
    }

    #[test]
    fn test_safe_recovery_message() {
        let msg = safe_recovery("OOM");
        assert!(msg.contains("OOM"));
        assert!(msg.contains("recovery"));
        assert!(msg.contains("core state"));
    }

    #[test]
    fn test_default_max_checkpoints() {
        let sp = SelfPreservation::default();
        assert_eq!(sp.max_checkpoints, 10);
    }

    #[test]
    fn test_recovery_stack_ordering() {
        let mut sp = SelfPreservation::new(5);
        sp.save_checkpoint("s1", "first".to_string());
        sp.save_checkpoint("s2", "second".to_string());
        assert_eq!(sp.restore_last(), Some("second".to_string()));
        assert_eq!(sp.restore_last(), Some("first".to_string()));
        assert_eq!(sp.restore_last(), None);
    }
}
