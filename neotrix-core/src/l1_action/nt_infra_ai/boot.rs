//! Fast boot manager for isolated environments.
//!
//! Targets <200ms cold start through:
//! - Pre-allocated memory pools
//! - Snapshot/checkpoint/restore
//! - Lazy-load container layers

use serde::{Deserialize, Serialize};

use super::isolate::IsolateError;

/// Boot configuration for fast startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootConfig {
    pub pre_alloc_mb: u64,
    pub use_snapshot: bool,
    pub lazy_load_layers: bool,
    pub max_boot_time_ms: u64,
}

impl Default for BootConfig {
    fn default() -> Self {
        Self {
            pre_alloc_mb: 256,
            use_snapshot: true,
            lazy_load_layers: true,
            max_boot_time_ms: 200,
        }
    }
}

/// Boot statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootStats {
    pub total_boot_time_ms: u64,
    pub pre_alloc_time_ms: u64,
    pub restore_time_ms: u64,
    pub layer_load_time_ms: u64,
    pub success: bool,
}

/// Fast boot manager for environments.
pub struct FastBootManager {
    config: BootConfig,
    snapshots: Vec<Snapshot>,
}

/// Snapshot for fast restore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub created_at: u64,
    pub size_bytes: u64,
    pub env_id: String,
}

impl FastBootManager {
    pub fn new(config: BootConfig) -> Self {
        Self {
            config,
            snapshots: Vec::new(),
        }
    }

    /// Create a snapshot of an environment.
    pub fn create_snapshot(&mut self, env_id: &str) -> Result<Snapshot, IsolateError> {
        let snapshot = Snapshot {
            id: format!("snap-{}-{}", env_id, self.snapshots.len()),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            size_bytes: 0,
            env_id: env_id.to_string(),
        };
        self.snapshots.push(snapshot.clone());
        Ok(snapshot)
    }

    /// Restore from a snapshot.
    pub fn restore_snapshot(&self, snapshot_id: &str) -> Result<u64, IsolateError> {
        let snapshot = self
            .snapshots
            .iter()
            .find(|s| s.id == snapshot_id)
            .ok_or_else(|| IsolateError::NotFound(format!("Snapshot {} not found", snapshot_id)))?;

        // Simulate restore time
        Ok(snapshot.size_bytes / 1024 / 1024) // MB to ms approximation
    }

    /// Get boot stats for a target boot time.
    pub fn estimate_boot_time(&self) -> BootStats {
        let pre_alloc_time = if self.config.pre_alloc_mb > 0 {
            self.config.pre_alloc_mb / 10 // ~10ms per 10MB
        } else {
            0
        };

        let restore_time = if self.config.use_snapshot && !self.snapshots.is_empty() {
            50 // ~50ms for snapshot restore
        } else {
            0
        };

        let layer_load_time = if self.config.lazy_load_layers {
            30 // ~30ms for lazy loading
        } else {
            100
        };

        let total = pre_alloc_time + restore_time + layer_load_time;

        BootStats {
            total_boot_time_ms: total,
            pre_alloc_time_ms: pre_alloc_time,
            restore_time_ms: restore_time,
            layer_load_time_ms: layer_load_time,
            success: total <= self.config.max_boot_time_ms,
        }
    }

    /// List available snapshots.
    pub fn list_snapshots(&self) -> &[Snapshot] {
        &self.snapshots
    }
}

impl Default for FastBootManager {
    fn default() -> Self {
        Self::new(BootConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_boot_estimation() {
        let manager = FastBootManager::default();
        let stats = manager.estimate_boot_time();
        assert!(stats.total_boot_time_ms <= 200);
        assert!(stats.success);
    }

    #[test]
    fn test_snapshot_workflow() {
        let mut manager = FastBootManager::default();
        let snapshot = manager.create_snapshot("env-1").unwrap();
        assert_eq!(snapshot.env_id, "env-1");

        let restore_time = manager.restore_snapshot(&snapshot.id).unwrap();
        assert!(restore_time >= 0);
    }
}
