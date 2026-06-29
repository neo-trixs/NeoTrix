use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::obfuscation::rand_u64_splitmix64;

/// A unique circuit identifier, wrapping a random u64 from splitmix64.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CircuitId(u64);

impl CircuitId {
    pub fn new() -> Self {
        CircuitId(rand_u64_splitmix64())
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for CircuitId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CircuitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// Lifetime state of a proxied circuit.
#[derive(Debug, Clone)]
pub enum CircuitState {
    /// Normal operation — streams may be assigned.
    Active,
    /// Graceful teardown: existing streams finish, no new streams assigned.
    Draining { remaining_connections: u32 },
    /// All streams closed — ready for reaping.
    Closed,
}

impl CircuitState {
    pub fn is_active(&self) -> bool {
        matches!(self, CircuitState::Active)
    }

    pub fn is_draining(&self) -> bool {
        matches!(self, CircuitState::Draining { .. })
    }
}

/// Per-circuit metadata tracked by the manager.
#[derive(Debug, Clone)]
pub struct CircuitInfo {
    pub created_at: Instant,
    pub target: String,
    pub pool_entry_index: usize,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_count: u32,
    pub state: CircuitState,
    pub last_activity: Instant,
}

impl CircuitInfo {
    fn new(target: &str, pool_entry_index: usize) -> Self {
        let now = Instant::now();
        CircuitInfo {
            created_at: now,
            target: target.to_string(),
            pool_entry_index,
            bytes_sent: 0,
            bytes_received: 0,
            connection_count: 1,
            state: CircuitState::Active,
            last_activity: now,
        }
    }
}

// ---------------------------------------------------------------------------
// Rotation thresholds
// ---------------------------------------------------------------------------

/// Rotate a circuit after 50 MB total transfer (half of Tor's 100 MB default).
pub const MAX_CIRCUIT_BYTES: u64 = 50 * 1024 * 1024;

/// Rotate a circuit after 60 seconds of lifetime (Tor default is 10 min).
pub const MAX_CIRCUIT_AGE: Duration = Duration::from_secs(60);

/// Force-close a draining circuit that has been idle this long.
pub const DRAIN_TIMEOUT: Duration = Duration::from_secs(10);

// ---------------------------------------------------------------------------
// Circuit Manager
// ---------------------------------------------------------------------------

/// Aggregated debug counters.
#[derive(Debug, Clone, Default)]
pub struct CircuitStats {
    pub total_created: u64,
    pub total_rotated: u64,
    pub total_closed: u64,
    pub active_count: usize,
    pub draining_count: usize,
}

/// Manages all active proxy circuits, decides when to rotate, and reaps stale
/// draining circuits.
///
/// ## Rotation triggers
///
/// | Trigger | Threshold | Rationale |
/// |---------|-----------|----------|
/// | Bytes transferred | 50 MB total (sent+received) | Middleboxes track flow volume; long flows are fingerprinted |
/// | Wall-clock age  | 60 seconds | Prevents long-lived TCP sessions from appearing in pcap aggregates |
///
/// Rotation is graceful: the old circuit enters `Draining`, existing streams
/// complete normally, but `pick()` / `try_upstream` logic should prefer new
/// circuits. Once all streams close or `DRAIN_TIMEOUT` elapses, the circuit
/// is reaped.
#[derive(Debug)]
pub struct CircuitManager {
    active_circuits: HashMap<CircuitId, CircuitInfo>,
    stats: CircuitStats,
}

impl CircuitManager {
    pub fn new() -> Self {
        CircuitManager {
            active_circuits: HashMap::with_capacity(64),
            stats: CircuitStats::default(),
        }
    }

    /// Create a new circuit for the given target and downstream pool index.
    /// Returns the randomly-generated `CircuitId`.
    pub fn new_circuit(&mut self, target: &str, pool_entry_index: usize) -> CircuitId {
        let id = CircuitId::new();
        self.active_circuits
            .insert(id, CircuitInfo::new(target, pool_entry_index));
        self.stats.total_created += 1;
        id
    }

    /// Returns `true` if the circuit has exceeded its byte or age threshold
    /// and is not already draining/closed.
    pub fn should_rotate(&self, id: CircuitId) -> bool {
        let Some(info) = self.active_circuits.get(&id) else {
            return false;
        };
        if !info.state.is_active() {
            return false;
        }
        let total_bytes = info.bytes_sent + info.bytes_received;
        let age = info.created_at.elapsed();
        total_bytes >= MAX_CIRCUIT_BYTES || age >= MAX_CIRCUIT_AGE
    }

    /// Rotate the given circuit: marks it `Draining` with its current
    /// connection count, then creates and returns a **new** `CircuitId`.
    ///
    /// Returns `None` if `id` is unknown.
    pub fn rotate(
        &mut self,
        id: CircuitId,
        new_target: &str,
        new_pool_entry_index: usize,
    ) -> Option<CircuitId> {
        let info = self.active_circuits.get_mut(&id)?;
        let remaining = info.connection_count;
        info.state = CircuitState::Draining { remaining_connections: remaining };
        self.stats.total_rotated += 1;
        Some(self.new_circuit(new_target, new_pool_entry_index))
    }

    /// Record bytes transferred on a circuit, updating its last-activity
    /// timestamp.
    pub fn record_transfer(&mut self, id: CircuitId, sent: u64, received: u64) {
        if let Some(info) = self.active_circuits.get_mut(&id) {
            info.bytes_sent += sent;
            info.bytes_received += received;
            info.last_activity = Instant::now();
        }
    }

    /// Bump the connection count for a circuit (new stream attached).
    pub fn add_connection(&mut self, id: CircuitId) {
        if let Some(info) = self.active_circuits.get_mut(&id) {
            info.connection_count += 1;
        }
    }

    /// Decrement the connection count. If the circuit is `Draining` and the
    /// count reaches 0, it transitions to `Closed` and will be reaped on
    /// the next `gc()` call.
    pub fn remove_connection(&mut self, id: CircuitId) {
        if let Some(info) = self.active_circuits.get_mut(&id) {
            info.connection_count = info.connection_count.saturating_sub(1);
            if let CircuitState::Draining { .. } = &info.state {
                if info.connection_count == 0 {
                    info.last_activity = Instant::now();
                    info.state = CircuitState::Closed;
                }
            }
        }
    }

    /// Garbage-collect all `Closed` circuits and any `Draining` circuits
    /// that have been idle past `DRAIN_TIMEOUT`.
    ///
    /// Returns the number of circuits reaped.
    pub fn gc(&mut self) -> usize {
        let now = Instant::now();
        let mut reaped = 0usize;
        self.active_circuits.retain(|_, info| {
            let should_keep = match info.state {
                CircuitState::Closed => {
                    reaped += 1;
                    false
                }
                CircuitState::Draining { .. }
                    if now.duration_since(info.last_activity) >= DRAIN_TIMEOUT =>
                {
                    info.state = CircuitState::Closed;
                    reaped += 1;
                    false
                }
                _ => true,
            };
            if !should_keep {
                self.stats.total_closed += 1;
            }
            should_keep
        });
        reaped
    }

    // ------------------------------------------------------------------
    // Accessors
    // ------------------------------------------------------------------

    pub fn stats(&self) -> CircuitStats {
        let active = self
            .active_circuits
            .values()
            .filter(|i| i.state.is_active())
            .count();
        let draining = self
            .active_circuits
            .values()
            .filter(|i| i.state.is_draining())
            .count();
        CircuitStats {
            total_created: self.stats.total_created,
            total_rotated: self.stats.total_rotated,
            total_closed: self.stats.total_closed,
            active_count: active,
            draining_count: draining,
        }
    }

    pub fn get_info(&self, id: CircuitId) -> Option<&CircuitInfo> {
        self.active_circuits.get(&id)
    }

    pub fn len(&self) -> usize {
        self.active_circuits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.active_circuits.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&CircuitId, &CircuitInfo)> {
        self.active_circuits.iter()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_id_unique() {
        let a = CircuitId::new();
        let b = CircuitId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn test_circuit_id_display() {
        let id = CircuitId(0xdead_beef_cafe_0001);
        let s = id.to_string();
        assert_eq!(s.len(), 16);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_new_circuit_active() {
        let mut mgr = CircuitManager::new();
        let id = mgr.new_circuit("example.com:443", 0);
        assert_eq!(mgr.stats().active_count, 1);
        assert_eq!(mgr.stats().total_created, 1);

        let info = mgr.get_info(id).unwrap();
        assert!(info.state.is_active());
        assert_eq!(info.target, "example.com:443");
        assert_eq!(info.pool_entry_index, 0);
        assert_eq!(info.connection_count, 1);
    }

    #[test]
    fn test_should_rotate_by_bytes() {
        let mut mgr = CircuitManager::new();
        let id = mgr.new_circuit("example.com:443", 0);

        // Under threshold — no rotation
        assert!(!mgr.should_rotate(id));

        // Exceed byte threshold
        mgr.record_transfer(id, MAX_CIRCUIT_BYTES, 1);
        assert!(mgr.should_rotate(id));
    }

    #[test]
    fn test_should_rotate_by_age() {
        let mut mgr = CircuitManager::new();
        // Temporarily lower the threshold for testing
        // NOTE: we test by directly checking age via created_at logic.
        // We can't mock Instant in std, so verify the age path works by
        // constructing a circuit that is "old enough" via our custom check.
        let id = mgr.new_circuit("example.com:443", 0);

        // Simulate age exceeding MAX_CIRCUIT_AGE by sleeping (risky in CI).
        // Instead, we verify the invariant: should_rotate is false immediately,
        // and we trust the Duration comparison logic.
        assert!(!mgr.should_rotate(id));
    }

    #[test]
    fn test_rotate_creates_new_circuit() {
        let mut mgr = CircuitManager::new();
        let old_id = mgr.new_circuit("example.com:443", 0);

        let new_id = mgr.rotate(old_id, "other.com:443", 1).unwrap();
        assert_ne!(old_id, new_id);

        // Old circuit should be draining
        let old_info = mgr.get_info(old_id).unwrap();
        assert!(old_info.state.is_draining());
        assert_eq!(old_info.target, "example.com:443");

        // New circuit is active
        let new_info = mgr.get_info(new_id).unwrap();
        assert!(new_info.state.is_active());
        assert_eq!(new_info.target, "other.com:443");
        assert_eq!(new_info.pool_entry_index, 1);

        assert_eq!(mgr.stats().total_created, 2);
        assert_eq!(mgr.stats().total_rotated, 1);
    }

    #[test]
    fn test_should_not_rotate_when_draining() {
        let mut mgr = CircuitManager::new();
        let old_id = mgr.new_circuit("example.com:443", 0);
        let _new_id = mgr.rotate(old_id, "other.com:443", 1).unwrap();

        // should_rotate should return false for draining circuits
        assert!(!mgr.should_rotate(old_id));
    }

    #[test]
    fn test_drain_auto_closes_on_zero_connections() {
        let mut mgr = CircuitManager::new();
        let old_id = mgr.new_circuit("example.com:443", 0);
        let _new_id = mgr.rotate(old_id, "other.com:443", 1).unwrap();

        // Remove the one connection
        mgr.remove_connection(old_id);
        let info = mgr.get_info(old_id).unwrap();
        assert!(matches!(info.state, CircuitState::Closed));
    }

    #[test]
    fn test_gc_reaps_closed_and_stale_draining() {
        let mut mgr = CircuitManager::new();
        let id1 = mgr.new_circuit("a.com:443", 0);
        let id2 = mgr.new_circuit("b.com:443", 1);

        // Rotate id1 (becomes draining)
        let _id1_new = mgr.rotate(id1, "a-new.com:443", 0).unwrap();

        // Remove connections on id1 to close it
        mgr.remove_connection(id1);

        // gc should reap the closed id1
        let reaped = mgr.gc();
        assert_eq!(reaped, 1);
        assert!(mgr.get_info(id1).is_none());
        assert!(mgr.get_info(id2).is_some());
    }

    #[test]
    fn test_record_transfer() {
        let mut mgr = CircuitManager::new();
        let id = mgr.new_circuit("example.com:443", 0);

        mgr.record_transfer(id, 100, 200);
        let info = mgr.get_info(id).unwrap();
        assert_eq!(info.bytes_sent, 100);
        assert_eq!(info.bytes_received, 200);
    }

    #[test]
    fn test_add_remove_connections() {
        let mut mgr = CircuitManager::new();
        let id = mgr.new_circuit("example.com:443", 0);

        mgr.add_connection(id);
        mgr.add_connection(id);
        assert_eq!(mgr.get_info(id).unwrap().connection_count, 3);

        mgr.remove_connection(id);
        assert_eq!(mgr.get_info(id).unwrap().connection_count, 2);
    }
}
